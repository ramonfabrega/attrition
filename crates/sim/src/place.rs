//! Placement — where a building may be founded, and what stops it.
//! `docs/CITIES.md` §2.
//!
//! Three levels, the way the original has them: `blocked_tcoord` asks one
//! tile, `blocked_location` asks the site, `blocked_site` drives both and
//! decides which verdict wins. Everything here is a predicate over state the
//! simulation already holds — the tile layer, the cell owners, the buildings
//! and cities — and returns a [`Blocked`] with the original's own numbering,
//! so a refused placement can be named the way the interface names it.

use crate::build::{self, Ident, flags};
use crate::world::{Owner, Pos, TILES_PER_CELL, UNITS_PER_TILE, tile, vector_dist};
use crate::{Player, Sim};

/// `move_x[0..=80]` / `move_y[0..=80]` — the offset spiral read from
/// `.rdata` at `0x00adcaf0` and `0x00adc400` (`crate::ai_sites` carries
/// its first 25 and its radius-5 ring): the centre, then rings of radius 1
/// to 4, which `snap_center`'s dock arm walks from index 1.
const SPIRAL_X: [i32; 81] = [
    0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2, 2, -2, -3, -2,
    -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, -1, -2, -3, -3, -3, -3, -3, -3, -4, -3, -2, -1, 0,
    1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0, -1, -2, -3, -4, -4, -4, -4, -4, -4, -4, -4,
];
const SPIRAL_Y: [i32; 81] = [
    0, -1, -1, -1, 0, 1, 1, 1, 0, -2, -2, -2, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, 2, 2, -3, -3,
    -3, -3, -3, -3, -3, -2, -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, -1, -2, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -3, -2, -1, 0, 1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0, -1, -2, -3,
];

/// Why a placement was refused — the PDB's `BlockIndex`, by value. `Clear`
/// is the original's 0. The names read inverted in two places (`Seen` is
/// returned for an *unseen* tile); the numbers are what the code returns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Blocked {
    #[default]
    Clear = 0,
    Building = 1,
    Mountain = 2,
    Rock = 3,
    Ruins = 4,
    NoOil = 5,
    Forest = 6,
    Rare = 7,
    Nearby = 8,
    NoResources = 9,
    NoForest = 0xa,
    ForestTaken = 0xb,
    NoMountain = 0xc,
    MountainTaken = 0xd,
    Water = 0xe,
    Land = 0xf,
    DockLand = 0x10,
    DockWater = 0x11,
    DockTerr = 0x12,
    CityRadius = 0x13,
    CityDistance = 0x14,
    FortDistance = 0x15,
    FortCityDistance = 0x16,
    Territory = 0x17,
    EnemyTerritory = 0x18,
    EnemyTerritory2 = 0x19,
    NeutralTerritory = 0x1a,
    PeacefulTerritory = 0x1b,
    Colonize = 0x1c,
    PortCity = 0x1d,
    PortFort = 0x1e,
    OutsideRadius = 0x1f,
    OutsideTown = 0x20,
    RazingTown = 0x21,
    OffMap = 0x22,
    Unseen = 0x23,
    Seen = 0x24,
    Road = 0x25,
    NeedRoad = 0x26,
    One = 0x27,
    OneOther = 0x28,
    Farm = 0x29,
    Wonder = 0x2a,
    NeedWall = 0x2b,
    River = 0x2c,
    LostCity = 0x2d,
    CantTransport = 0x2e,
}

/// The regions of the original are capped at 64 land and 64 sea; a region
/// index at or past this is "not a region" for the per-region counters.
const REGION_COUNTERS: u16 = 64;

/// `BuildTypeData::tile_corner@00636440` over a footprint's `x_size` and
/// `y_size` (`+0x234`, `+0x238`): the tile `pos` falls in, snapped as
/// [`Sim::snap_center`] snaps (a tile centre for an odd size, its corner for
/// an even one) and read back as a tile — `div_3_table[v >> 6]` twice in the
/// original — less half of each size, `sar`. Item 2122's sweep holds it
/// against the original on the real `div_3_table` for coordinates in
/// `0..=16000`.
pub fn tile_corner_of(x_size: i32, y_size: i32, pos: Pos) -> Pos {
    let t = pos.tile();
    let snap = |v: i32, size: i32| v * UNITS_PER_TILE + if size & 1 == 1 { 96 } else { 0 };
    let c = Pos::new(snap(t.x, x_size), snap(t.y, y_size)).tile();
    Pos::new(c.x - (x_size >> 1), c.y - (y_size >> 1))
}

/// `WallData::covers_tile@006439b0`'s test over the footprint's corner and
/// its `x_size` and `y_size`: the tile lies in `corner .. corner + size` on
/// both axes, each bound signed. Item 2187's sweep holds it, with
/// [`tile_corner_of`] for the corner, against the original.
pub(crate) fn footprint_covers(corner: Pos, x_size: i32, y_size: i32, tile: Pos) -> bool {
    tile.x >= corner.x
        && tile.x < corner.x.wrapping_add(x_size)
        && tile.y >= corner.y
        && tile.y < corner.y.wrapping_add(y_size)
}

/// The owner byte a cell's `WData` holds (`+0xf`): the player, `-1` for
/// nobody, `-2` for an ambiguous claim — what the territory predicates
/// below compare.
pub(crate) fn owner_byte(o: Owner) -> i32 {
    match o {
        Owner::Player(p) => i32::from(p),
        Owner::None => -1,
        Owner::Ambiguous => -2,
    }
}

/// The friendly-ground test `WallData::in_unfriendly_territory@0063eca0`,
/// `BuildTypeData::non_friendly_territory@006389c0`,
/// `in_enemy_territory@00638b40` and `check_land_adjacent@00638ed0` each
/// inline: the owner byte `o` is the placer's own `me` (its leader's `+8`),
/// or both `diplos` words between them are 2 — `diplos[who][o]` and
/// `diplos[o][me]`, which this crate keeps as the two `allied` flags.
/// Item 2247's sweeps hold it against all four.
pub(crate) fn friendly_ground(o: i32, me: i32, ally_to: bool, ally_from: bool) -> bool {
    o == me || (ally_to && ally_from)
}

/// `non_friendly_territory`'s rank of one land tile under the footprint:
/// 0 friendly ground, 1 nobody's (`-1`), 2 ambiguous (below `-1`) or a
/// foreign owner at peace both ways, 3 a foreign owner at war either way
/// (`diplos == 0`, the two `at_war` flags), which ends the walk. The walk
/// keeps the largest rank it met.
pub(crate) fn ground_rank(o: i32, me: i32, ally: (bool, bool), war: (bool, bool)) -> i32 {
    if o >= 0 && friendly_ground(o, me, ally.0, ally.1) {
        0
    } else if o < -1 {
        2
    } else if o < 0 {
        1
    } else if war.0 || war.1 {
        3
    } else {
        2
    }
}

/// `in_enemy_territory`'s presence clause on foreign ground: the placer
/// holds a city anywhere (`LeaderData+0x3f8`), or the cell's region (`WData`
/// `+4`, read unsigned, so a negative region has none) is below `0x40` and
/// the placer holds a city or a fort in it (`+0x125e`, `+0x12de`, a short a
/// region) — `counts(region)` answers that pair.
pub(crate) fn claim_reaches(
    city_num: i32,
    region: i32,
    counts: impl Fn(usize) -> (i32, i32),
) -> bool {
    city_num != 0
        || (u32::try_from(region).is_ok_and(|r| r < 0x40) && {
            let (cities, forts) = counts(region as usize);
            cities != 0 || forts != 0
        })
}

/// `check_land_adjacent`'s land test on a ring tile's mask: neither ocean
/// (`& 0x30 == 0x20`) nor blocked (`0x4000`).
pub(crate) fn adjacent_land(mask: u16) -> bool {
    mask & tile::SURFACE != tile::SURFACE_OCEAN && mask & tile::BLOCKED == 0
}

/// `BuildTypeData::get_town@00639b90`'s choice of finder: `find_town_at`
/// for a type that needs a town and is not city-free, `find_city_at`
/// otherwise.
pub(crate) fn wants_town(build_flags: u32) -> bool {
    build_flags & flags::NO_CITY == 0 && build_flags & flags::NEEDS_TOWN != 0
}

impl Sim {
    // ------------------------------------------------------------------
    // Geometry
    // ------------------------------------------------------------------

    /// `BuildTypeData::snap_center`'s last step: an odd footprint centres on a
    /// tile centre, an even one on a tile corner. The oil and dock searches
    /// are [`Sim::snap_center_placed`]'s.
    pub fn snap_center(&self, ty: usize, pos: Pos) -> Pos {
        let b = &self.build_types[ty];
        let t = pos.tile();
        Pos::new(
            t.x * UNITS_PER_TILE + if b.x_size & 1 == 1 { 96 } else { 0 },
            t.y * UNITS_PER_TILE + if b.y_size & 1 == 1 { 96 } else { 0 },
        )
    }

    /// `BuildTypeData::snap_center@00636190(x, y, &x, &y, who)` with its
    /// **dock arm** (item 803, `docs/GOLDEN.md` §28). A building of the
    /// Dock's lineage (`is(0x1b0)`) whose centred tile `blocked_site`
    /// refuses for `who` is moved to the first tile of the spiral
    /// `move_x/move_y[1..=80]` round it that the same test clears, and then
    /// snapped as [`Sim::snap_center`] snaps; with none, it stays. Only the
    /// row is bounds-checked, as the listing has it. `Objects::init_build@
    /// 0065d190` calls it with the building's owner, so the console's `add
    /// dock` on a shore tile lands on the water beside it: run249's Dock,
    /// asked at tile (53, 153), stands at (56, 155), `move_x/y[36]`.
    ///
    /// SEAM, none reached by a capture on file: the same arm for a
    /// Woodcutter's Camp or a Mine (`0x1a2`, `0x1a3`) placed by a player,
    /// and the Oil Well's and Oil Platform's (`0x1a5`, `0x1a6`) snap to a
    /// seen cell flagged `0x800`. `Sim::init_build` still takes the plain
    /// snap; the harness's `add` is this function's one caller.
    pub fn snap_center_placed(&self, ty: usize, pos: Pos, who: Option<Player>) -> Pos {
        if build::is_dock(&self.build_types, ty) {
            let t = pos.tile();
            let centre = |x: i32, y: i32| {
                Pos::new(
                    x * UNITS_PER_TILE + UNITS_PER_TILE / 2,
                    y * UNITS_PER_TILE + UNITS_PER_TILE / 2,
                )
            };
            if self.blocked_site(who, ty, centre(t.x, t.y), None) != Blocked::Clear {
                let rows = self.world.height() * TILES_PER_CELL;
                let cols = self.world.width() * TILES_PER_CELL;
                for k in 1..=80 {
                    let y = t.y + SPIRAL_Y[k];
                    if y < 0 || y >= rows || y >= cols {
                        continue;
                    }
                    let at = centre(t.x + SPIRAL_X[k], y);
                    if self.blocked_site(who, ty, at, None) == Blocked::Clear {
                        return self.snap_center(ty, at);
                    }
                }
            }
        }
        self.snap_center(ty, pos)
    }

    /// `BuildTypeData::tile_corner`: the top-left tile of the footprint of a
    /// building centred at `pos` ([`tile_corner_of`] over its sizes).
    pub fn tile_corner(&self, ty: usize, pos: Pos) -> Pos {
        let b = &self.build_types[ty];
        tile_corner_of(b.x_size, b.y_size, pos)
    }

    /// Every tile of a footprint with its corner at `corner`.
    pub fn footprint(&self, ty: usize, corner: Pos) -> Vec<Pos> {
        let b = &self.build_types[ty];
        let mut out = Vec::with_capacity(b.area() as usize);
        for v in 0..b.y_size {
            for u in 0..b.x_size {
                out.push(Pos::new(corner.x + u, corner.y + v));
            }
        }
        out
    }

    /// The ring of width `ring` around a footprint, minus the four outer
    /// corners — what `check_land_adjacent`, `count_mountains_adjacent` and
    /// `count_trees_adjacent` scan.
    pub(crate) fn ring(&self, ty: usize, corner: Pos, ring: i32) -> Vec<Pos> {
        let b = &self.build_types[ty];
        let (x0, y0, x1, y1) = (
            corner.x - ring,
            corner.y - ring,
            corner.x + b.x_size + ring - 1,
            corner.y + b.y_size + ring - 1,
        );
        let mut out = Vec::new();
        for v in y0..=y1 {
            for u in x0..=x1 {
                let inside = u >= corner.x
                    && u < corner.x + b.x_size
                    && v >= corner.y
                    && v < corner.y + b.y_size;
                let corner_tile = (u == x0 || u == x1) && (v == y0 || v == y1);
                if !inside && !corner_tile {
                    out.push(Pos::new(u, v));
                }
            }
        }
        out
    }

    fn tile_owner(&self, t: Pos) -> Owner {
        self.world.owner(crate::World::cell_of_tile(t))
    }

    fn is_ocean_tile(&self, t: Pos) -> bool {
        self.world.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN
    }

    // ------------------------------------------------------------------
    // Per-player counters the rules read
    // ------------------------------------------------------------------

    /// `LeaderData::city_num`: live cities.
    pub fn city_num(&self, who: Player) -> i32 {
        self.cities
            .iter()
            .filter(|c| c.alive && c.owner == who)
            .count() as i32
    }

    /// `reg_cities[reg]`: the player's live cities in a region.
    pub fn reg_cities(&self, who: Player, reg: u16) -> i32 {
        if reg >= REGION_COUNTERS {
            return 0;
        }
        self.cities
            .iter()
            .filter(|c| c.alive && c.owner == who && c.reg == Some(reg))
            .count() as i32
    }

    /// `reg_forts[reg]`: the player's active forts in a region.
    pub fn reg_forts(&self, who: Player, reg: u16) -> i32 {
        if reg >= REGION_COUNTERS {
            return 0;
        }
        self.buildings
            .iter()
            .filter(|b| {
                b.alive
                    && b.active
                    && b.owner == who
                    && b.ty.is_some_and(|t| build::is_fort(&self.build_types, t))
                    && self.world.region_of(b.pos.cell()) == Some(reg)
            })
            .count() as i32
    }

    /// `reg_terr[reg]`: the cells of a region the player owns.
    pub fn reg_terr(&self, who: Player, reg: u16) -> i32 {
        self.world
            .cells_in(reg)
            .filter(|c| self.world.owner(*c) == Owner::Player(who))
            .count() as i32
    }

    /// A placed, unfinished city of `who` — an `UnbuiltCity` entry.
    pub(crate) fn unbuilt_cities(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
        self.buildings.iter().enumerate().filter_map(move |(i, b)| {
            (b.alive
                && !b.active
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_city(&self.build_types, t)))
            .then_some(i)
        })
    }

    pub(crate) fn unbuilt_forts(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
        self.buildings.iter().enumerate().filter_map(move |(i, b)| {
            (b.alive
                && !b.active
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_fort(&self.build_types, t)))
            .then_some(i)
        })
    }

    pub(crate) fn built_forts(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
        self.buildings.iter().enumerate().filter_map(move |(i, b)| {
            (b.alive
                && b.active
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_fort(&self.build_types, t)))
            .then_some(i)
        })
    }

    fn players(&self) -> impl Iterator<Item = Player> {
        (0..self.players.len()).map(|p| p as Player)
    }

    pub(crate) fn player_alive(&self, p: Player) -> bool {
        !self.defeated[p as usize]
    }

    // ------------------------------------------------------------------
    // blocked_site
    // ------------------------------------------------------------------

    /// `BuildTypeData::blocked_site(x, y, who, exclude_o, …)`: may a building
    /// of type `ty` be centred at `pos` by `who`? `who == None` is the
    /// player-less form (the cursor's), which skips every ownership rule.
    /// `exclude` is a building of the placer's to leave out of the spacing
    /// and placed-tile tests — the site itself, when it re-checks at start.
    pub fn blocked_site(
        &self,
        who: Option<Player>,
        ty: usize,
        pos: Pos,
        exclude: Option<usize>,
    ) -> Blocked {
        self.blocked_site_slots(who, ty, pos, exclude).0
    }

    /// `blocked_site`'s `param_5` beside its verdict: the gather count
    /// `blocked_location` fills in for a non-flat gather type, clamped at
    /// zero. Every other type leaves it at the zero the original's callers
    /// initialise it to, because the original only ever writes it on that
    /// one path.
    ///
    /// `Leader::produce_building` is the reader that matters — it scores a
    /// woodcutter's camp by the cube of this number and refuses a site under
    /// three (`ai_place.rs`).
    pub fn blocked_site_slots(
        &self,
        who: Option<Player>,
        ty: usize,
        pos: Pos,
        exclude: Option<usize>,
    ) -> (Blocked, i32) {
        let b = &self.build_types[ty];
        let corner = self.tile_corner(ty, pos);
        // A human who, or whose ally, lost a city in the last 75 frames may
        // not place a city.
        if build::is_city(&self.build_types, ty)
            && let Some(w) = who
            && self.nation[w as usize].human
        {
            for i in self.players() {
                if (i == w || self.is_ally(w, i))
                    && self.player_alive(i)
                    && self.lost_city_stamp[i as usize].is_some_and(|s| self.frame - 75 < s)
                {
                    return (Blocked::LostCity, 0);
                }
            }
        }
        let mut first = Blocked::Clear;
        // The unseen tally: the footprint tiles whose fog half-cell
        // `was_seen` denies `who`, counted in the same walk (none for no
        // `who`).
        let mut unseen = 0i32;
        for t in self.footprint(ty, corner) {
            let r = self.blocked_tcoord(who, ty, t, exclude);
            if r == Blocked::OffMap {
                return (r, 0);
            }
            if r != Blocked::Clear && (first == Blocked::Clear || r == Blocked::Territory) {
                first = r;
            }
            if let Some(w) = who
                && !self.was_seen_fog(t.x >> 1, t.y >> 1, w)
            {
                unseen += 1;
            }
        }
        // `blocked_site@00636a50`: a first refusal of `0x24` is returned
        // first; then a footprint more than half unseen is `0x24` too,
        // unless the type is of the Dock's lineage (`is(0x1b0)`); then
        // water's; then `blocked_location` (twenty-fourth pass, group 12;
        // A8 row 52). The tally is item 2956's (`docs/AI.md` §241): the
        // battery's Germans scored a Town site at (47, 24), outside their
        // border, whose footprint is 29 of 49 tiles unseen, and the
        // original refuses it on run893b's packet.
        if first == Blocked::Seen {
            return (first, 0);
        }
        if b.x_size * b.y_size / 2 < unseen && !build::is_dock(&self.build_types, ty) {
            return (Blocked::Seen, 0);
        }
        if first == Blocked::Water {
            return (first, 0);
        }
        let (r, slots) = self.blocked_location(who, ty, pos, corner, exclude);
        if r != Blocked::Clear {
            return (r, slots);
        }
        let _ = b;
        (first, slots)
    }

    /// `BuildTypeData::blocked_tcoord@00636db0`: one tile, by
    /// [`crate::tcoord::blocked_tcoord_steps`] (swept against the original,
    /// item 3346); this builds its view. A tile the leader has not seen
    /// answers `Seen` (`0x24`) for every refusal but its three contents
    /// arms, which answer `Unseen` (`0x23`) once the tile is seen and
    /// `Seen` when it is not really seen; `None` is the cursor's
    /// player-less form, which sees everything.
    pub fn blocked_tcoord(
        &self,
        who: Option<Player>,
        ty: usize,
        t: Pos,
        exclude: Option<usize>,
    ) -> Blocked {
        let mut view = crate::tcoord::SimTcoord { sim: self, ty };
        let exclude = exclude.map_or(-1, |e| i32::try_from(e).expect("a building index"));
        Blocked::from_code(crate::tcoord::blocked_tcoord_steps(
            &mut view,
            who.map_or(-1, i32::from),
            t.x,
            t.y,
            exclude,
        ))
    }

    /// `WallData::covers_tile`: whether tile `t` is inside the building's
    /// footprint. A building with no type covers nothing.
    pub fn build_covers_tile(&self, b: usize, t: Pos) -> bool {
        let Some(bd) = self.buildings.get(b) else {
            return false;
        };
        bd.ty.is_some_and(|ty| {
            let c = self.tile_corner(ty, bd.pos);
            let bt = &self.build_types[ty];
            t.x >= c.x && t.x < c.x + bt.x_size && t.y >= c.y && t.y < c.y + bt.y_size
        })
    }

    /// `ObjectsData::find_building_placed_at`: a placed, not-started building
    /// of `who` whose footprint covers `t`, other than `exclude`.
    pub(crate) fn find_building_placed_at(
        &self,
        who: Player,
        t: Pos,
        exclude: Option<usize>,
    ) -> Option<usize> {
        self.buildings.iter().enumerate().position(|(i, b)| {
            Some(i) != exclude && b.alive && !b.started && b.owner == who && {
                self.build_covers_tile(i, t)
            }
        })
    }

    /// `BuildTypeData::blocked_location@006375b0`: the site, and the gather
    /// count its tail hands back through `param_7`. By
    /// [`crate::location::blocked_location_steps`] (swept against the
    /// original, item 3364; `docs/CITIES.md` §2.6); this builds its view.
    ///
    /// **A seam.** The player-less form (`who == None`, the cursor's) skips
    /// every ownership gate as the original does, but its gather survey
    /// walks `calc_gather` with `who = −1`, which has no oracle: the view
    /// answers a positive count there ([`crate::location::SimLocation`]).
    pub fn blocked_location(
        &self,
        who: Option<Player>,
        ty: usize,
        pos: Pos,
        corner: Pos,
        exclude: Option<usize>,
    ) -> (Blocked, i32) {
        let mut view = crate::location::SimLocation {
            sim: self,
            ty,
            pos,
            corner,
            out: None,
        };
        let args = crate::location::LocArgs {
            px: pos.x,
            py: pos.y,
            tx: corner.x,
            ty: corner.y,
            who: who.map_or(-1, i32::from),
            exclude: exclude.map_or(-1, |e| i32::try_from(e).expect("a building index")),
            has_out: true,
        };
        let r = crate::location::blocked_location_steps(&mut view, &args);
        (Blocked::from_code(r), view.out.unwrap_or(0))
    }

    /// `has_preq(COLONIZE_BONUS)`: the bonus row's one prerequisite — the
    /// tree names it in `roles.colonize_preq`; a tree that does not know it
    /// grants it, the way every type outside the tree is ungated
    /// ([`crate::transport`]'s `transport_preq_held` is the same rule for
    /// the third bonus). It is a **technology's** prerequisite rather than a
    /// nation's bonus: the fourth of `rules.xml`'s `TECHBONUSES`, whose
    /// `preq0` is Coinage — carried as a nation flag nothing ever set until
    /// 2026-09-02, which is why no AI in any capture could put a city on a
    /// second island (`docs/CITIES.md` §2.6.1).
    pub(crate) fn colonize_preq_held(&self, who: Player) -> bool {
        match self.tech_tree.roles.colonize_preq {
            Some(t) => self
                .tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], t),
            None => true,
        }
    }

    /// `BuildTypeData::non_friendly_territory`: over the footprint's land
    /// tiles, 0 all friendly, 1 some unowned, 2 some foreign-at-peace or
    /// ambiguous, 3 (returned at once) foreign-at-war.
    pub(crate) fn non_friendly_territory(&self, who: Player, ty: usize, corner: Pos) -> i32 {
        let mut r = 0;
        for t in self.footprint(ty, corner) {
            // Off the map and ocean tiles are skipped (item 2247: an
            // off-map tile was ranked unowned).
            if !self.world.tile_in_bounds(t) || self.is_ocean_tile(t) {
                continue;
            }
            let o = owner_byte(self.tile_owner(t));
            let rank = match u8::try_from(o) {
                Ok(p) => ground_rank(
                    o,
                    i32::from(who),
                    (self.allied_with(who, p), self.allied_with(p, who)),
                    (self.at_war_with(who, p), self.at_war_with(p, who)),
                ),
                Err(_) => ground_rank(o, i32::from(who), (false, false), (false, false)),
            };
            if rank == 3 {
                return 3;
            }
            r = r.max(rank);
        }
        r
    }

    /// `BuildTypeData::in_enemy_territory`: a tile owned by a non-friendly
    /// player where the placer already has a city anywhere, or a city or fort
    /// in that region. (The "really seen by that owner" clause is visibility,
    /// taken as false.)
    pub(crate) fn in_enemy_territory(&self, who: Player, ty: usize, corner: Pos) -> bool {
        for t in self.footprint(ty, corner) {
            if let Owner::Player(p) = self.tile_owner(t)
                && !friendly_ground(
                    i32::from(p),
                    i32::from(who),
                    self.allied_with(who, p),
                    self.allied_with(p, who),
                )
            {
                let reg = self.world.tregion(t).map_or(-1, i32::from);
                if claim_reaches(self.city_num(who), reg, |r| {
                    let r = r as u16;
                    (self.reg_cities(who, r), self.reg_forts(who, r))
                }) {
                    return true;
                }
            }
        }
        false
    }

    /// `BuildTypeData::count_water`: footprint tiles that are ocean.
    pub(crate) fn count_water(&self, ty: usize, corner: Pos) -> i32 {
        self.footprint(ty, corner)
            .into_iter()
            .filter(|t| self.is_ocean_tile(*t))
            .count() as i32
    }

    /// `BuildTypeData::check_land_adjacent`: 2 no land around, 0 friendly
    /// land found (or no player), 1 land but all foreign or unowned.
    pub(crate) fn check_land_adjacent(&self, who: Option<Player>, ty: usize, corner: Pos) -> i32 {
        let mut r = 2;
        for t in self.ring(ty, corner, 1) {
            // Off the map is skipped (item 2247: its mask read zero, land).
            if !self.world.tile_in_bounds(t) || !adjacent_land(self.world.tile_mask(t)) {
                continue;
            }
            let Some(w) = who else {
                return 0;
            };
            let o = owner_byte(self.tile_owner(t));
            let friendly = u8::try_from(o).is_ok_and(|p| {
                friendly_ground(
                    o,
                    i32::from(w),
                    self.allied_with(w, p),
                    self.allied_with(p, w),
                )
            });
            if friendly || (o == -1 && self.nation[w as usize].lakota) {
                return 0;
            }
            r = 1;
        }
        r
    }

    // ------------------------------------------------------------------
    // Which city
    // ------------------------------------------------------------------

    /// `BuildTypeData::get_town`: the city a building centred at `pos` would
    /// belong to — every footprint tile inside some city mask, then the
    /// owner's nearest covering city. Returns an index into [`Sim::cities`].
    pub fn get_town(&self, who: Player, ty: usize, pos: Pos) -> Option<usize> {
        let corner = self.tile_corner(ty, pos);
        for t in self.footprint(ty, corner) {
            // Off the map is skipped (item 2247: its mask read zero).
            if self.world.tile_in_bounds(t) && self.world.tile_mask(t) & tile::CITY_RADIUS == 0 {
                return None;
            }
        }
        let b = &self.build_types[ty];
        if wants_town(b.flags) {
            self.find_town_at(who, pos)
        } else {
            self.find_city_at(who, pos, Some(ty))
        }
    }

    /// `ObjectsData::find_city_at`: the owner's nearest live city whose radius
    /// covers the tile, a one-per-city type pushed 100 tiles toward a city
    /// that lacks one. The tile must be inside a city mask and its cell owned
    /// by nobody, the player or an ally.
    pub fn find_city_at(&self, who: Player, pos: Pos, ty: Option<usize>) -> Option<usize> {
        let t = pos.tile();
        if self.world.tile_mask(t) & tile::CITY_RADIUS == 0 {
            return None;
        }
        match self.world.owner_at(pos) {
            Owner::None => {}
            Owner::Player(p) if self.is_ally(who, p) => {}
            _ => return None,
        }
        let mut best: Option<(i32, usize)> = None;
        for (ci, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who {
                continue;
            }
            let cp = c.pos.tile();
            let mut d = vector_dist(cp.x - t.x, cp.y - t.y);
            let r = self.city_radius(c.owner, self.buildings[c.building].ty);
            if d > r {
                continue;
            }
            if let Some(ty) = ty
                && self.build_types[ty].has(flags::ONE_PER_CITY)
                && self.count_buildings(ci, self.build_types[ty].ident, false) != 0
            {
                d += 100;
            }
            if best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, ci));
            }
        }
        best.map(|(_, c)| c)
    }

    /// `ObjectsData::find_town_at`: the same over `TOWN`-lineage cities,
    /// without the penalty.
    pub fn find_town_at(&self, who: Player, pos: Pos) -> Option<usize> {
        let t = pos.tile();
        let mut best: Option<(i32, usize)> = None;
        for (ci, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who {
                continue;
            }
            let Some(bt) = self.buildings[c.building].ty else {
                continue;
            };
            if !build::is(&self.build_types, bt, Ident::Town) {
                continue;
            }
            let cp = c.pos.tile();
            let d = vector_dist(cp.x - t.x, cp.y - t.y);
            if d > self.city_radius(c.owner, Some(bt)) {
                continue;
            }
            if best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, ci));
            }
        }
        best.map(|(_, c)| c)
    }

    // ------------------------------------------------------------------
    // How many cities
    // ------------------------------------------------------------------

    /// `LeaderData::get_city_limit@006d6130`: the Civic level plus one, plus
    /// the Bantu and Pyramids allowances.
    ///
    /// **The Pyramids' arm is `has_wonder(0x20e)`** — an activated
    /// Pyramids of the leader's own standing in a city, which
    /// [`Sim::wonders_held`] reads off the buildings. Until item 1454 it
    /// read `Nation::pyramids`, which nothing in a game ever sets, so a
    /// finished Pyramids raised no limit: French East Indies' AI finishes
    /// its Pyramids on 9658 and founds a fifth city on 9777, where this
    /// crate stood at four of four. `Nation::pyramids` stays as a unit
    /// test's stand-in.
    pub fn city_limit(&self, who: Player) -> i32 {
        let n = &self.nation[who as usize];
        let mut civic = self.tech[who as usize].epoch[crate::tech::Line::Civic as usize];
        if n.bantu && civic != 0 {
            civic += self.tuning.bantu_city_limit;
        }
        let mut limit = civic + 1;
        let held = self.wonders_held(who) & (1 << crate::tech::wonder::PYRAMIDS) != 0;
        if held || n.pyramids {
            limit += self.tuning.pyramids_city_limit;
        }
        limit
    }

    /// `LeaderData::get_total_cities`: cities owned plus cities placed and not
    /// yet finished, less one for the Forbidden City.
    pub fn total_cities(&self, who: Player) -> i32 {
        let mut n = self.city_num(who) + self.unbuilt_cities(who).count() as i32;
        if self.cities.iter().any(|c| {
            c.alive && c.owner == who && self.building_ident(c.building) == Ident::ForbiddenCity
        }) {
            n -= 1;
        }
        n
    }

    /// `LeaderData::at_city_limit`.
    pub fn at_city_limit(&self, who: Player) -> bool {
        self.city_limit(who) <= self.total_cities(who)
    }

    /// The tiles a city's radius mask covers — `Wall::mask_city` over the
    /// `even_circle_*` tables, which `even_circle_init` builds once with a
    /// **rounded `sqrtf`** (the one gameplay table the original builds with a
    /// float, pinned here as its integer equivalent). The "even" circle is
    /// centred on the corner between the city's tile and the one before it:
    /// offset `u = dx + 1` for `dx ≥ 0`, `u = dx` for `dx < 0` (no zero row or
    /// column), a tile is in when `round(√(u² + v²)) ≤ radius`, i.e.
    /// `4(u² + v²) ≤ (2·radius + 1)²`. So at radius 20 the disc spans `dx ∈
    /// [−20, 19]` along the axis — one tile shy on the positive side.
    pub(crate) fn city_mask_tiles(&self, pos: Pos, radius: i32) -> Vec<Pos> {
        let c = pos.tile();
        let lim = (2 * radius + 1) * (2 * radius + 1);
        let mut out = Vec::new();
        for dy in -(radius + 1)..=radius {
            let v = if dy >= 0 { dy + 1 } else { dy };
            for dx in -(radius + 1)..=radius {
                let u = if dx >= 0 { dx + 1 } else { dx };
                if 4 * (u * u + v * v) <= lim {
                    let t = Pos::new(c.x + dx, c.y + dy);
                    if self.world.tile_in_bounds(t) {
                        out.push(t);
                    }
                }
            }
        }
        out
    }
}

/// Tiles per cell, re-exported for the tests that lay worlds out.
pub const TILES: i32 = TILES_PER_CELL;
