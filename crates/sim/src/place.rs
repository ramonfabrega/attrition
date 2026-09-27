//! Placement — where a building may be founded, and what stops it.
//! `docs/CITIES.md` §2.
//!
//! Three levels, the way the original has them: `blocked_tcoord` asks one
//! tile, `blocked_location` asks the site, `blocked_site` drives both and
//! decides which verdict wins. Everything here is a predicate over state the
//! simulation already holds — the tile layer, the cell owners, the buildings
//! and cities — and returns a [`Blocked`] with the original's own numbering,
//! so a refused placement can be named the way the interface names it.

use crate::build::{self, BuildDomain, Ident, flags};
use crate::world::{Cell, Owner, Pos, TILES_PER_CELL, Terrain, UNITS_PER_TILE, tile, vector_dist};
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

/// The sea-region size below which a body of water is not "a real sea" for
/// the colonise rule.
const REAL_SEA_CELLS: i32 = 24;

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
    /// building centred at `pos`.
    pub fn tile_corner(&self, ty: usize, pos: Pos) -> Pos {
        let b = &self.build_types[ty];
        let p = self.snap_center(ty, pos).tile();
        Pos::new(p.x - (b.x_size >> 1), p.y - (b.y_size >> 1))
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
    fn ring(&self, ty: usize, corner: Pos, ring: i32) -> Vec<Pos> {
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
    fn unbuilt_cities(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
        self.buildings.iter().enumerate().filter_map(move |(i, b)| {
            (b.alive
                && !b.active
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_city(&self.build_types, t)))
            .then_some(i)
        })
    }

    fn unbuilt_forts(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
        self.buildings.iter().enumerate().filter_map(move |(i, b)| {
            (b.alive
                && !b.active
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_fort(&self.build_types, t)))
            .then_some(i)
        })
    }

    fn built_forts(&self, who: Player) -> impl Iterator<Item = usize> + '_ {
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

    fn player_alive(&self, p: Player) -> bool {
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
        for t in self.footprint(ty, corner) {
            let r = self.blocked_tcoord(who, ty, t, exclude);
            if r == Blocked::OffMap {
                return (r, 0);
            }
            if r != Blocked::Clear && (first == Blocked::Clear || r == Blocked::Territory) {
                first = r;
            }
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

    /// `BuildTypeData::blocked_tcoord`: one tile. Visibility is taken as
    /// granted everywhere (`docs/CITIES.md` §11).
    pub fn blocked_tcoord(
        &self,
        who: Option<Player>,
        ty: usize,
        t: Pos,
        exclude: Option<usize>,
    ) -> Blocked {
        let b = &self.build_types[ty];
        if !self.world.tile_in_bounds(t) {
            return Blocked::OffMap;
        }
        let cell = crate::World::cell_of_tile(t);
        let mask = self.world.tile_mask(t);
        let Some(reg) = self.world.region_of(cell) else {
            return Blocked::Ruins;
        };
        if mask & tile::OBJECT == tile::OBJECT_BUILDING {
            return Blocked::Building;
        }
        let dock = build::is_dock(&self.build_types, ty);
        if !dock {
            match b.domain() {
                BuildDomain::Water => {
                    if mask & tile::SURFACE != tile::SURFACE_OCEAN {
                        return Blocked::Land;
                    }
                }
                BuildDomain::Land => {
                    if mask & tile::SURFACE == tile::SURFACE_OCEAN {
                        return Blocked::Water;
                    }
                }
                BuildDomain::Both => {}
            }
        }
        if mask & tile::BLOCKED != 0 {
            if mask & tile::SURFACE == tile::SURFACE_FOREST {
                return Blocked::Forest;
            }
            if mask & tile::OBJECT == tile::OBJECT_MOUNTAIN {
                return Blocked::Mountain;
            }
            return Blocked::Rare;
        }
        if mask & tile::AS_BUILDING != 0 {
            return Blocked::Building;
        }
        // The cell's rock and oil (`006370b2`, `00637105`): an Oil Well or
        // Oil Platform (`is(0x1a5)`, `is(0x1a6)`) needs the cell's oil,
        // and every other type is refused a rock cell. The cell's flags are
        // the map's (`World::set_oil_at` is the oil bit's only writer, from
        // the terrain groups and the editor), so this is a fixed map fact
        // the spiral's `buildings_allowed` also reads — but only on the
        // candidate's own cell, and a jitter's footprint can reach the next
        // one (`docs/AI.md` §78).
        let cell_flags = self.world.cell_data(cell).flags;
        if build::is(&self.build_types, ty, build::Ident::OilWell)
            || build::is(&self.build_types, ty, build::Ident::OilPlatform)
        {
            if cell_flags & crate::world::cell::OIL == 0 {
                return Blocked::NoOil;
            }
        } else if cell_flags & crate::world::cell::ROCK != 0 {
            return Blocked::Rock;
        }
        if mask & tile::PLACED != 0
            && let Some(w) = who
            && self.find_building_placed_at(w, t, exclude).is_some()
        {
            return Blocked::Building;
        }
        if !b.has(flags::NO_CITY) && mask & tile::CITY_RADIUS == 0 {
            return Blocked::OutsideRadius;
        }
        let city = build::is_city(&self.build_types, ty);
        let fort = build::is_fort(&self.build_types, ty);
        if (city || fort)
            && let Some(w) = who
            && !self.nation[w as usize].lakota
        {
            let dutch_anywhere =
                fort && self.nation[w as usize].dutch && self.tuning.dutch_fort_placement != 0;
            if !dutch_anywhere && self.world.owner(cell) == Owner::None {
                // Unowned ground: only as the first city or fort in this
                // region — the colonisation foothold.
                let taken = self.reg_cities(w, reg) != 0
                    || self.reg_forts(w, reg) != 0
                    || self
                        .unbuilt_cities(w)
                        .chain(self.unbuilt_forts(w))
                        .any(|i| {
                            Some(i) != exclude
                                && self.world.region_of(self.buildings[i].pos.cell()) == Some(reg)
                        });
                if taken {
                    return if fort {
                        Blocked::NeutralTerritory
                    } else {
                        Blocked::Territory
                    };
                }
            }
        }
        if b.has(flags::NEED_WALL) {
            return Blocked::NeedWall;
        }
        if mask & tile::RIVER != 0 {
            return Blocked::River;
        }
        // A flat gather type — the Farm, the Oil Well, the Oil Platform —
        // needs its good on the tile's land (`00637532`: the tile form of
        // `get_land`, then `LandData::get_amount`), so a Farm is refused
        // sand, rock, a forest cell and an oil cell.
        if b.has(flags::GATHER) && b.has(flags::FLAT) {
            let good = crate::ai_place::gather_good(b.ident).map_or(-1, |g| g as i32);
            let amount = usize::try_from(self.world.land_class_tile(t))
                .ok()
                .and_then(|i| crate::world::LANDS.get(i))
                .and_then(|l| l.good.iter().position(|&g| g == good).map(|k| l.amount[k]))
                .unwrap_or(0);
            if amount == 0 {
                return Blocked::NoResources;
            }
        }
        Blocked::Clear
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
    fn find_building_placed_at(
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

    /// `BuildTypeData::blocked_location`: the site, and the gather count its
    /// tail hands back through `param_7`.
    pub fn blocked_location(
        &self,
        who: Option<Player>,
        ty: usize,
        pos: Pos,
        corner: Pos,
        exclude: Option<usize>,
    ) -> (Blocked, i32) {
        let r = self.blocked_location_verdict(who, ty, pos, corner, exclude);
        if r != Blocked::Clear {
            return (r, 0);
        }
        self.gather_verdict(who, ty, corner, exclude)
    }

    /// 2.6.7, `blocked_location@006375b0:679–716` — the last thing the
    /// function does, and the only place it writes `param_7`.
    ///
    /// A **non-flat gather type** surveys what its site would gather, by
    /// the same `calc_gather` count `max_gatherers` reads
    /// ([`Sim::site_gather_count`]), and a site with nothing under it is
    /// refused outright: `NoMountain` for a mine, `NoForest` for a camp,
    /// `NoResources` for anything else, and the `Taken` pair when the count
    /// came back negative because every candidate cell was already being
    /// gathered from. The flat types — farm, oil well, oil platform — and
    /// every non-gather building skip it and leave the count at zero.
    ///
    /// **A seam.** The player-less form (`who == None`, the cursor's) skips
    /// the survey: the walk's cell-owner test wants a player, and the
    /// original's `who = −1` behaviour there has no oracle. No caller in
    /// this crate passes it.
    fn gather_verdict(
        &self,
        who: Option<Player>,
        ty: usize,
        corner: Pos,
        exclude: Option<usize>,
    ) -> (Blocked, i32) {
        let b = &self.build_types[ty];
        if !b.has(flags::GATHER) || b.has(flags::FLAT) {
            return (Blocked::Clear, 0);
        }
        let Some(w) = who else {
            return (Blocked::Clear, 0);
        };
        let n = self.site_gather_count(ty, w, corner, exclude);
        let camp = b.ident == Ident::Woodcutter;
        let verdict = if n == 0 {
            match b.ident {
                Ident::Mine => Blocked::NoMountain,
                Ident::Woodcutter => Blocked::NoForest,
                _ => Blocked::NoResources,
            }
        } else if n < 0 {
            if camp {
                Blocked::ForestTaken
            } else {
                Blocked::MountainTaken
            }
        } else {
            Blocked::Clear
        };
        (verdict, n.max(0))
    }

    /// `has_preq(COLONIZE_BONUS)`: the bonus row's one prerequisite — the
    /// tree names it in `roles.colonize_preq`; a tree that does not know it
    /// grants it, the way every type outside the tree is ungated
    /// ([`crate::transport`]'s `transport_preq_held` is the same rule for
    /// the third bonus).
    fn colonize_preq_held(&self, who: Player) -> bool {
        match self.tech_tree.roles.colonize_preq {
            Some(t) => self
                .tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], t),
            None => true,
        }
    }

    fn blocked_location_verdict(
        &self,
        who: Option<Player>,
        ty: usize,
        pos: Pos,
        corner: Pos,
        exclude: Option<usize>,
    ) -> Blocked {
        let b = &self.build_types[ty];
        let city = build::is_city(&self.build_types, ty);
        let fort = build::is_fort(&self.build_types, ty);
        let dock = build::is_dock(&self.build_types, ty);
        let cell = pos.cell();
        let reg = self.world.region_of(cell);
        let mut coastal = false;

        if let Some(w) = who {
            let n = &self.nation[w as usize];
            // 2.6.1 Territory.
            if !city && !fort {
                if !b.has(flags::TERRITORY_EXEMPT) && !dock {
                    let r = self.non_friendly_territory(w, ty, corner);
                    if r >= 3 {
                        return Blocked::EnemyTerritory;
                    }
                    if r == 2 || (r == 1 && !n.lakota) {
                        return Blocked::NeutralTerritory;
                    }
                }
            } else {
                if self.in_enemy_territory(w, ty, corner) {
                    return if self.city_num(w) != 0 {
                        Blocked::EnemyTerritory
                    } else {
                        Blocked::EnemyTerritory2
                    };
                }
                if let Some(reg) = reg
                    && self.reg_forts(w, reg) == 0
                    && self.reg_cities(w, reg) == 0
                    && self.city_num(w) != 0
                    && self.tuning.first_city_near_coast != 0
                {
                    // `has_preq(COLONIZE_BONUS 0x2af)`, and it is a
                    // **technology's** prerequisite rather than a nation's
                    // bonus: the fourth of `rules.xml`'s `TECHBONUSES`,
                    // whose `preq0` is Coinage. Carried as a nation flag
                    // nothing ever set until 2026-09-02, which is why no
                    // AI in any capture could put a city on a second
                    // island (`docs/CITIES.md` §2.6.1).
                    if !self.colonize_preq_held(w) {
                        return Blocked::Colonize;
                    }
                    let r = self.tuning.first_city_near_coast / 4;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if vector_dist(dx, dy) > r {
                                continue;
                            }
                            let c = Cell::new(cell.x + dx, cell.y + dy);
                            if let Some(sr) = self.world.region_of(c)
                                && self.world.terrain(sr) == Terrain::Sea
                                && self.world.region_size(sr) > REAL_SEA_CELLS
                            {
                                coastal = true;
                            }
                        }
                    }
                    if !coastal {
                        return if city {
                            Blocked::PortCity
                        } else {
                            Blocked::PortFort
                        };
                    }
                }
            }
            // 2.6.2 City spacing.
            if city && let Some(reg) = reg {
                let mut spacing = self.tuning.city_spacing;
                if self.world.region_size(reg) * 9 / 10 <= self.reg_terr(w, reg) {
                    spacing -= self.tuning.relax_city_spacing;
                }
                let site = self.snap_center(ty, pos).tile();
                for i in self.players() {
                    if !self.player_alive(i) {
                        continue;
                    }
                    for u in self.unbuilt_cities(i) {
                        if (i != w || Some(u) == exclude) && !self.buildings[u].started {
                            continue;
                        }
                        let bp = self.buildings[u].pos;
                        if self.world.region_of(bp.cell()) == Some(reg)
                            && vector_dist(bp.tile().x - site.x, bp.tile().y - site.y) <= spacing
                        {
                            return Blocked::CityDistance;
                        }
                    }
                    if i == w || self.reg_cities(i, reg) != 0 {
                        for c in self.cities.iter().filter(|c| c.alive && c.owner == i) {
                            let cp = c.pos.tile();
                            if c.reg == Some(reg)
                                && vector_dist(cp.x - site.x, cp.y - site.y) <= spacing
                            {
                                return Blocked::CityDistance;
                            }
                        }
                    }
                }
            }
            // 2.6.3 Fort spacing.
            if fort && let Some(reg) = reg {
                let site = self.snap_center(ty, pos).tile();
                for i in self.players() {
                    if !self.player_alive(i) || i == w {
                        continue;
                    }
                    let s = if self.is_ally(w, i) {
                        self.tuning.fort_spacing
                    } else {
                        self.tuning.fort_to_enemy_city_spacing
                    };
                    for u in self.unbuilt_cities(i) {
                        let bd = &self.buildings[u];
                        if bd.started
                            && self.world.region_of(bd.pos.cell()) == Some(reg)
                            && vector_dist(bd.pos.tile().x - site.x, bd.pos.tile().y - site.y) <= s
                        {
                            return Blocked::FortCityDistance;
                        }
                    }
                    if self.reg_cities(i, reg) != 0 {
                        for c in self.cities.iter().filter(|c| c.alive && c.owner == i) {
                            let cp = c.pos.tile();
                            if c.reg == Some(reg) && vector_dist(cp.x - site.x, cp.y - site.y) <= s
                            {
                                return Blocked::FortCityDistance;
                            }
                        }
                    }
                }
                for i in self.players() {
                    if !self.player_alive(i) {
                        continue;
                    }
                    let verdict = if i == w {
                        Blocked::FortDistance
                    } else {
                        Blocked::FortCityDistance
                    };
                    for u in self.unbuilt_forts(i) {
                        let bd = &self.buildings[u];
                        if i == w && Some(u) == exclude {
                            continue;
                        }
                        if i != w && !bd.started {
                            continue;
                        }
                        if self.world.region_of(bd.pos.cell()) == Some(reg)
                            && vector_dist(bd.pos.tile().x - site.x, bd.pos.tile().y - site.y)
                                <= self.tuning.fort_spacing
                        {
                            return verdict;
                        }
                    }
                    // Built forts are distance-checked without a region test
                    // (the cities above have one).
                    if i == w || self.reg_forts(i, reg) != 0 {
                        for f in self.built_forts(i) {
                            let fp = self.buildings[f].pos.tile();
                            if vector_dist(fp.x - site.x, fp.y - site.y) <= self.tuning.fort_spacing
                            {
                                return verdict;
                            }
                        }
                    }
                }
            }
            // 2.6.4 Must belong to a city.
            if !b.has(flags::NO_CITY) {
                let Some(c) = self.get_town(w, ty, pos) else {
                    return if b.has(flags::NEEDS_TOWN) {
                        Blocked::OutsideTown
                    } else {
                        Blocked::OutsideRadius
                    };
                };
                if b.has(flags::ONE_PER_CITY) && self.count_buildings(c, b.ident, false) != 0 {
                    let site = self.snap_center(ty, pos).tile();
                    for (ci, other) in self.cities.iter().enumerate() {
                        if ci == c || !other.alive || other.owner != w {
                            continue;
                        }
                        let op = other.pos.tile();
                        let r = self
                            .city_radius(other.owner, self.buildings[other.building].ty)
                            .min(64);
                        if vector_dist(op.x - site.x, op.y - site.y) <= r
                            && self.count_buildings(ci, b.ident, false) == 0
                        {
                            return Blocked::OneOther;
                        }
                    }
                    return Blocked::One;
                }
                if b.ident == Ident::Farm
                    && self.count_buildings(c, Ident::Farm, false) >= self.farm_limit(c)
                {
                    return Blocked::Farm;
                }
                if b.wonder
                    && b.ident != Ident::RedFort
                    && self.num_wonders(c, true) > i32::from(n.egyptians)
                {
                    return Blocked::Wonder;
                }
            }
        }
        // 2.6.5 Water and land.
        let water = self.count_water(ty, corner);
        if !dock {
            match b.domain() {
                BuildDomain::Water if water < b.area() => return Blocked::Land,
                BuildDomain::Land if water != 0 => return Blocked::Water,
                _ => {}
            }
        } else {
            let adjacent = self.check_land_adjacent(who, ty, corner);
            if water < 3 * b.area() / 4 {
                if water != 0 && adjacent != 0 {
                    return Blocked::DockTerr;
                }
                return Blocked::DockWater;
            }
            if adjacent == 1 {
                return Blocked::DockTerr;
            }
            if adjacent == 2 {
                return Blocked::DockLand;
            }
        }
        // 2.6.6 Clear ground around a city or fort.
        if city || fort {
            let ring = if coastal { 1 } else { 2 };
            for t in self.ring(ty, corner, ring) {
                let m = self.world.tile_mask(t);
                if m & tile::OBJECT == tile::OBJECT_MOUNTAIN {
                    return Blocked::Mountain;
                }
            }
            for t in self.ring(ty, corner, ring) {
                let m = self.world.tile_mask(t);
                if m & tile::SURFACE == tile::SURFACE_FOREST {
                    return Blocked::Forest;
                }
            }
        }
        Blocked::Clear
    }

    /// `BuildTypeData::non_friendly_territory`: over the footprint's land
    /// tiles, 0 all friendly, 1 some unowned, 2 some foreign-at-peace or
    /// ambiguous, 3 (returned at once) foreign-at-war.
    fn non_friendly_territory(&self, who: Player, ty: usize, corner: Pos) -> i32 {
        let mut r = 0;
        for t in self.footprint(ty, corner) {
            if self.is_ocean_tile(t) {
                continue;
            }
            match self.tile_owner(t) {
                Owner::Player(p) if self.is_ally(who, p) => {}
                Owner::Player(p) => {
                    if self.is_enemy(who, p) {
                        return 3;
                    }
                    r = 2;
                }
                Owner::Ambiguous => r = 2,
                Owner::None => {
                    if r < 1 {
                        r = 1;
                    }
                }
            }
        }
        r
    }

    /// `BuildTypeData::in_enemy_territory`: a tile owned by a non-friendly
    /// player where the placer already has a city anywhere, or a city or fort
    /// in that region. (The "really seen by that owner" clause is visibility,
    /// taken as false.)
    fn in_enemy_territory(&self, who: Player, ty: usize, corner: Pos) -> bool {
        for t in self.footprint(ty, corner) {
            if let Owner::Player(p) = self.tile_owner(t)
                && !self.is_ally(who, p)
            {
                let reg = self.world.tregion(t);
                if self.city_num(who) != 0
                    || reg.is_some_and(|r| {
                        self.reg_cities(who, r) != 0 || self.reg_forts(who, r) != 0
                    })
                {
                    return true;
                }
            }
        }
        false
    }

    /// `BuildTypeData::count_water`: footprint tiles that are ocean.
    fn count_water(&self, ty: usize, corner: Pos) -> i32 {
        self.footprint(ty, corner)
            .into_iter()
            .filter(|t| self.is_ocean_tile(*t))
            .count() as i32
    }

    /// `BuildTypeData::check_land_adjacent`: 2 no land around, 0 friendly
    /// land found (or no player), 1 land but all foreign or unowned.
    fn check_land_adjacent(&self, who: Option<Player>, ty: usize, corner: Pos) -> i32 {
        let mut r = 2;
        for t in self.ring(ty, corner, 1) {
            let m = self.world.tile_mask(t);
            if m & tile::SURFACE == tile::SURFACE_OCEAN || m & tile::BLOCKED != 0 {
                continue;
            }
            let Some(w) = who else {
                return 0;
            };
            match self.tile_owner(t) {
                Owner::Player(p) if self.is_ally(w, p) => return 0,
                Owner::None if self.nation[w as usize].lakota => return 0,
                _ => r = 1,
            }
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
            if self.world.tile_mask(t) & tile::CITY_RADIUS == 0 {
                return None;
            }
        }
        let b = &self.build_types[ty];
        if !b.has(flags::NO_CITY) && b.has(flags::NEEDS_TOWN) {
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

    /// `LeaderData::get_city_limit`: the Civic level plus one, plus the Bantu
    /// and Pyramids allowances.
    pub fn city_limit(&self, who: Player) -> i32 {
        let n = &self.nation[who as usize];
        let mut civic = self.tech[who as usize].epoch[crate::tech::Line::Civic as usize];
        if n.bantu && civic != 0 {
            civic += self.tuning.bantu_city_limit;
        }
        let mut limit = civic + 1;
        if n.pyramids {
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
