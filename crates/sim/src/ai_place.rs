//! `Leader::produce_building` — where the AI puts a building.
//!
//! The specification is `~/ghidra-projects/reports/ai/create-buildings.md`
//! §4 (ratified in `docs/AI.md` §11): the anchor building and the city
//! permission, the spiral of cells around the anchor scored one by one, the
//! builder chosen among the citizens, the 2×2 jitter with its draws, the
//! payment, the site and the swarm order. Every sync-stream draw the
//! original takes on this path is taken here at the same point: one
//! `Random::get(0, 0xffff)` per friendless FARM/MINE candidate that passes
//! every site test, one per unblocked sub-position of the jitter.
//!
//! The spiral's table is `circle_init@006817f0`, rebuilt here in integers
//! exactly as the original builds it: for each ring `r` in `0..=64`, every
//! `(dx, dy)` of the `[−r, r]` square whose octagonal distance is `r`, `dx`
//! outer and `dy` inner ascending, `circle_radius[r]` the running count —
//! absolute cell offsets from the anchor, not deltas.
//!
//! What the flat harness world cannot supply is a named seam, each
//! answering as an empty map would: the cell's `val` byte (0 — every
//! non-gather candidate gets the same `+0xff`), `buildings_allowed` (the
//! cell flags `0x78`, always allowed), the enemy-seen flag, `danger[]`, the
//! gather amounts a `World::gather_at` would give an oil platform, and the
//! oil patches (an oil well is never placed). The census adjustments at the
//! end — `filled += 1`, `space[n − 2]` decremented for `n = 2..best_sp`,
//! and the builder taken off the gatherers or the free peasants, city and
//! region included — are kept, as the original keeps them (`docs/AI.md`
//! §2.20; run8's frame 2 shows `gatherers 5 → 4` on the farm's frame).

use std::sync::OnceLock;

use crate::build::{Ident, flags};
use crate::orders::{Body, QueuePos, index};
use crate::world::{Cell, TILES_PER_CELL, Terrain, UNITS_PER_TILE, tile, vector_dist};
use crate::{Player, Pos, Sim, cost};

/// The two draw sites of `Leader::produce_building@006e1400`, under the
/// original's own offsets. [`Sim::mark`] writes them into
/// [`Sim::phase_marks`], so a frame's draws are compared against
/// `rondata::trace`'s by name rather than by a total (`docs/SYNC.md` §5.1).
///
/// Both offsets are return addresses into `produce_building`, confirmed in
/// the listing: `0x006e2099` is followed by `cltd; mov ecx, 0x1f4; idiv` —
/// the spiral candidate's `% 500` — and `0x006e2c05` by `cltd; mov ecx,
/// 0x64; idiv`, the jitter's `% 100`.
pub const SITE_SPIRAL: &str = "Leader::produce_building+0xc99";
pub const SITE_JITTER: &str = "Leader::produce_building+0x1805";

/// `circle_x`/`circle_y`/`circle_radius`, as `circle_init` fills them.
pub struct Circle {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    /// `circle_radius[r]`: how many entries lie within ring `r`.
    pub radius: [usize; 0x41],
}

/// `circle_init@006817f0`'s cap on the point count.
const MAX_POINTS: usize = 0x3249;

pub fn circle() -> &'static Circle {
    static CIRCLE: OnceLock<Circle> = OnceLock::new();
    CIRCLE.get_or_init(|| {
        let mut c = Circle {
            x: Vec::new(),
            y: Vec::new(),
            radius: [0; 0x41],
        };
        'rings: for r in 0..=0x40i32 {
            for x in -r..=r {
                for y in -r..=r {
                    if vector_dist(x, y) == r {
                        c.x.push(x);
                        c.y.push(y);
                        if c.x.len() >= MAX_POINTS {
                            for k in r as usize..=0x40 {
                                c.radius[k] = c.x.len();
                            }
                            break 'rings;
                        }
                    }
                }
            }
            c.radius[r as usize] = c.x.len();
        }
        c
    })
}

/// The compass, `move_x`/`move_y` indices 1–8: NW, N, NE, E, SE, S, SW, W.
pub(crate) const MOVE_X: [i32; 9] = [0, -1, 0, 1, 1, 1, 0, -1, -1];
pub(crate) const MOVE_Y: [i32; 9] = [0, -1, -1, -1, 0, 1, 1, 1, 0];

/// A cell in world units — four tiles, `0x300`.
const UNITS_PER_CELL: i32 = 4 * UNITS_PER_TILE;

/// The good a gather building takes — `BuildTypeData::get_good`, by identity.
pub(crate) fn gather_good(ident: Ident) -> Option<usize> {
    use crate::economy::Resource as R;
    Some(match ident {
        Ident::Farm => R::Food.index(),
        Ident::Woodcutter => R::Timber.index(),
        Ident::Mine => R::Metal.index(),
        Ident::University => R::Knowledge.index(),
        Ident::OilWell | Ident::OilPlatform => R::Oil.index(),
        _ => return None,
    })
}

/// The good a gather enhancer raises — `BuildTypeData::get_enhancing_good
/// @00639880`, `[this+4] − 0x1a7` into a four-entry jump table (`0x6398b0`:
/// Granary food, Lumber Mill timber, Smelter metal, Refinery oil). An
/// exact-type switch, not a lineage test; none of the four has a successor
/// in `buildingrules.xml`, so the two agree.
pub(crate) fn enhancing_good(ident: Ident) -> Option<usize> {
    use crate::economy::Resource as R;
    Some(match ident {
        Ident::Granary => R::Food.index(),
        Ident::Lumbermill => R::Timber.index(),
        Ident::Smelter => R::Metal.index(),
        Ident::Refinery => R::Oil.index(),
        _ => return None,
    })
}

pub(crate) fn is_enhancer(ident: Ident) -> bool {
    matches!(
        ident,
        Ident::Granary | Ident::Lumbermill | Ident::Smelter | Ident::Refinery
    )
}

pub(crate) fn is_military_trainer(ident: Ident) -> bool {
    matches!(
        ident,
        Ident::Barracks | Ident::Stable | Ident::SiegeFactory | Ident::Factory | Ident::AutoPlant
    )
}

/// `grid_index_x`/`grid_index_y` (`.rdata`, VA `0xADECF0`/`0xADED30`): the
/// order `space_at_corner` walks the sixteen tiles under a corner, as
/// `(dx, dy)`. The centre 2×2 comes first, then the top row, the bottom
/// row, then the two side columns — so the early-out on "one of the first
/// four is blocked" is on the centre, and every 3×3 of the 4×4 contains
/// all four of them.
pub(crate) const GRID_ORDER: [(i32, i32); 16] = [
    (1, 1),
    (2, 1),
    (1, 2),
    (2, 2),
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (0, 3),
    (1, 3),
    (2, 3),
    (3, 3),
    (0, 1),
    (3, 1),
    (0, 2),
    (3, 2),
];

/// `grid_threes` (`.rdata`, VA `0xADECA0`, four rows of five): for each
/// 3×3 of the 4×4, the five of its tiles outside the centre 2×2, as indices
/// into [`GRID_ORDER`] — the corners `(1,1)`, `(0,1)`, `(0,0)`, `(1,0)` in
/// that order. With the centre known free, these five are the whole test.
pub(crate) const GRID_THREES: [[usize; 5]; 4] = [
    [9, 10, 11, 13, 15],
    [12, 14, 8, 9, 10],
    [14, 12, 4, 5, 6],
    [5, 6, 7, 13, 15],
];

impl Sim {
    /// Whether some live building of anyone has its centre in this cell —
    /// the cell flag `0x4000` the spiral skips.
    pub(crate) fn cell_has_centre(&self, cell: Cell) -> bool {
        self.buildings
            .iter()
            .any(|b| b.alive && b.pos.cell() == cell)
    }

    /// `WorldData::is_ocean`: the cell's region is water.
    pub(crate) fn cell_is_ocean(&self, cell: Cell) -> bool {
        self.world
            .region_of(cell)
            .is_some_and(|r| self.world.terrain(r) == Terrain::Sea)
    }

    /// `WorldData::space_at_corner@006b27f0(tx, ty, who, _, need_city)`:
    /// the footprint class that fits with its corner at tile `(tx, ty)`.
    /// The sixteen tiles of the 4×4 are walked in [`GRID_ORDER`] — the
    /// centre 2×2 first — and a blocked tile among those **first four**
    /// returns 0 at once. Then: none blocked → 4; fewer than eight blocked
    /// and some 3×3 free ([`GRID_THREES`]) → 3; otherwise **2** — never 0
    /// once the centre is free, however many of the rest are blocked. A
    /// tile is blocked off the map, outside every city radius when a city
    /// is needed, under a footprint, placed on, `BLOCKED`, or in a cell
    /// another leader owns. The fourth argument is not read
    /// (`docs/AI.md` §15.9).
    pub(crate) fn space_at_corner(&self, who: Player, tx: i32, ty: i32, need_city: bool) -> i32 {
        let mut free = [false; 16];
        let mut blocked = 0;
        for (k, &(i, j)) in GRID_ORDER.iter().enumerate() {
            let t = Pos::new(tx + i, ty + j);
            let ok = self.world.tile_in_bounds(t) && {
                let m = self.world.tile_mask(t);
                let owner = self.world.owner(crate::World::cell_of_tile(t)).player();
                !(need_city && m & tile::CITY_RADIUS == 0)
                    && m & tile::OBJECT != tile::OBJECT_BUILDING
                    && m & tile::PLACED == 0
                    && m & tile::BLOCKED == 0
                    && !owner.is_some_and(|o| o != who)
            };
            free[k] = ok;
            if !ok {
                if k < 4 {
                    return 0;
                }
                blocked += 1;
            }
        }
        if blocked == 0 {
            return 4;
        }
        if blocked < 8 && GRID_THREES.iter().any(|sq| sq.iter().all(|&k| free[k])) {
            return 3;
        }
        2
    }

    /// `WorldData::check_building_wcoord@006b26e0`: the best footprint
    /// class over the corner offsets `dx ∈ [−w, w]`, `dy ∈ [−h, h]` (those
    /// on an axis or with `|dx| + |dy| ≤ max`), returning 4 at once when
    /// found; 0 without looking when the cell is another leader's, occupied
    /// (`flags & 0x70`), or has `blocked == 0x10`.
    pub(crate) fn check_building_wcoord(
        &self,
        who: Player,
        cell: Cell,
        w: i32,
        h: i32,
        max: i32,
        need_city: bool,
    ) -> i32 {
        let d = self.world.cell_data(cell);
        if self.world.owner(cell).player().is_some_and(|o| o != who)
            || d.flags & 0x70 != 0
            || d.blocked == 0x10
        {
            return 0;
        }
        let mut best = 0;
        for dx in -w..=w {
            for dy in -h..=h {
                if !(dx == 0 || dy == 0 || dx.abs() + dy.abs() <= max) {
                    continue;
                }
                let sp = self.space_at_corner(who, cell.x * 4 + dx, cell.y * 4 + dy, need_city);
                if sp == 4 {
                    return 4;
                }
                best = best.max(sp);
            }
        }
        best
    }

    /// `ObjectsData::find_building_placed_at@00658c80(cell·4 + 2, who)`: the
    /// live building of `who` whose **footprint covers** the cell's centre
    /// tile — not one whose own centre is in the cell.
    ///
    /// The original walks the nine cells around `tile >> 2`, follows each
    /// one's object chain, and takes the first whose
    /// `corner ≤ tile < corner + size` on both axes; the filter on the
    /// object is `vtable[0xc]`, which `vtables.txt` names
    /// `SubObjectData::is_active`. It refuses without looking unless the
    /// tile's own mask carries `(mask & 3) == 3` or `PLACED`, so a tile no
    /// building has ever masked answers `None` for free.
    ///
    /// A five-tile Library reaches four cells; the cell it is *centred* in
    /// is one of them. That is the whole of the difference, and it is what
    /// [`Sim::find_friends`] counts (`docs/AI.md` §26).
    pub(crate) fn building_placed_at(&self, who: Player, cell: Cell) -> Option<usize> {
        let t = Pos::new(cell.x * TILES_PER_CELL + 2, cell.y * TILES_PER_CELL + 2);
        if !self.world.tile_in_bounds(t) {
            return None;
        }
        let mask = self.world.tile_mask(t);
        if mask & tile::OBJECT != tile::OBJECT_BUILDING && mask & tile::PLACED == 0 {
            return None;
        }
        self.buildings
            .iter()
            .enumerate()
            .position(|(i, b)| b.alive && b.owner == who && self.build_covers_tile(i, t))
    }

    /// `BuildTypeData::find_friends(x, y, city, who)`: neighbours of the
    /// candidate cell that count for this type, `+1` on a diagonal and `+2`
    /// on a cardinal.
    pub(crate) fn find_friends(
        &self,
        rec: usize,
        cell: Cell,
        city: Option<usize>,
        who: Player,
    ) -> i32 {
        let bt = &self.build_types[rec];
        let ident = bt.ident;
        let tower_like = matches!(ident, Ident::Tower | Ident::Lookout);
        if (bt.attack != 0 && !tower_like) || ident == Ident::Woodcutter {
            return 0;
        }
        let needs_city = !bt.has(flags::NO_CITY);
        let mut n = 0;
        for d in 1..=8usize {
            let nc = Cell::new(cell.x + MOVE_X[d], cell.y + MOVE_Y[d]);
            let Some(nb) = self.building_placed_at(who, nc) else {
                continue;
            };
            if needs_city && self.buildings[nb].city != city {
                continue;
            }
            let ni = self.building_ident(nb);
            let nt = self.buildings[nb].ty.map(|t| &self.build_types[t]);
            let nb_gather = nt.is_some_and(|t| t.has(flags::GATHER));
            let nb_wonder = ni == Ident::Wonder;
            let add = if d % 2 == 1 { 1 } else { 2 };
            if is_enhancer(ident) {
                // **The enhanced good against the neighbour's**
                // (`0x639380`–`0x6393b0`): `get_enhancing_good(this)` beside
                // `get_good` of the neighbour's type, and a friend when the
                // two are equal — a Granary counts the farms around it
                // (`docs/AI.md` §99.5). `get_good@0063bd50` answers −1 for
                // anything but the six gather types and the enhancer's side
                // is never −1, so a non-gather neighbour never counts.
                if enhancing_good(ident).is_some() && enhancing_good(ident) == gather_good(ni) {
                    n += add;
                }
            } else if is_military_trainer(ident) {
                if is_military_trainer(ni) && !self.building_is_city(nb) {
                    n += add;
                }
            } else if tower_like {
                if nb_gather && ni != Ident::University {
                    n += 2;
                } else if nb_wonder {
                    n += if ident == Ident::Lookout { 8 } else { 4 };
                }
            } else if ident == Ident::Farm {
                if matches!(ni, Ident::Farm | Ident::Granary) {
                    n += add;
                }
            } else {
                // Anything else: a gather building (not a university), an
                // enhancer or a non-city military trainer adds nothing; any
                // other non-wonder neighbour counts.
                let nothing = (nb_gather && ni != Ident::University)
                    || is_enhancer(ni)
                    || (is_military_trainer(ni) && !self.building_is_city(nb));
                if !nothing && !nb_wonder {
                    n += add;
                }
            }
        }
        n
    }

    /// `Leader::produce_building(t, near, escrow)`: `true` when a site was
    /// placed (the original's 0). `near` is the reference building — a city
    /// centre for `place_building_with_cost`, any building for the orphan
    /// form — and `city` the hint the host derived from it.
    pub fn produce_building(
        &mut self,
        who: Player,
        rec: usize,
        near: usize,
        city: Option<usize>,
        escrow: bool,
    ) -> bool {
        let bt = self.build_types[rec].clone();
        let ident = bt.ident;
        let frame = self.frame;
        // 4.1 The anchor and the permission: an active city building lends
        // its city; anything else only to a type that needs none.
        let city = city.filter(|&c| self.cities[c].alive);
        if city.is_none() && !bt.has(flags::NO_CITY) {
            return false;
        }
        let anchor = self.buildings[near].pos.cell();
        let areg = self.world.region_of(anchor);
        let mut rings = ((self.city_radius(who, self.buildings[near].ty) + 2) / 4).max(0) as usize;
        let nocity = frame != 0 && bt.has(flags::NO_CITY);
        let gather = bt.has(flags::GATHER);
        // `is(0x1b0)`, the dock **lineage** test, not `ident == Dock`: a
        // Shipyard or a Port answers it too (`docs/TRANSPORT.md`, "The
        // slot"). It gates three things here — the spiral's start, the
        // site block's extent, and the slide below.
        let is_dock = crate::build::is_dock(&self.build_types, rec);
        let tower = ident == Ident::Tower;
        let is_fort = ident == Ident::Fort;
        let circle = circle();
        let mut start = 0usize;
        let mut fortlike = false;
        if nocity {
            rings += 1;
            if !gather && !is_dock && !tower {
                start = if is_fort { 1 } else { circle.radius[3] };
            }
            fortlike = is_fort;
        }
        let scored_by_gather =
            gather && !matches!(ident, Ident::Farm | Ident::University | Ident::Mine);
        // 4.2 Oil wells walk the leader's oil patches, which the simulation
        // does not carry.
        if ident == Ident::OilWell {
            return false;
        }

        // 4.3 The spiral.
        let (w, h, max) = if !is_dock {
            let w = if bt.x_size < 4 { 5 - bt.x_size } else { 1 };
            let h = if bt.y_size < 4 { 5 - bt.y_size } else { 1 };
            let max = if bt.x_size < 4 || bt.y_size < 4 {
                w + h
            } else {
                2
            };
            (w, h, max)
        } else {
            (4, 4, 8)
        };
        let end = circle.radius[rings.min(0x40)];
        if start >= end {
            return false;
        }
        let (xs, ys) = (self.world.width(), self.world.height());
        let big = bt.x_size.max(bt.y_size);
        let unlimited = self.lobby.resources_unlimited();
        let mut step = 1;
        let mut best = 0i32;
        let mut best_sp = 0;
        let mut best_cand: Option<Pos> = None;
        // **The index steps at the *bottom* of the iteration**, by the
        // stride as it stands *then* — `local_2c = local_2c + iVar13` at
        // `006e25bb`, after the body that may have set `iVar13` to 3. An
        // increment at the top spends the old stride once more, so the
        // first strided hop starts one cell late and the whole tail of the
        // spiral is offset by one. That one cell is East Indies' dock
        // (`docs/AI.md` §20). The body is a labelled block so that every
        // arm that used to `continue` still reaches the step.
        let mut idx = start;
        while idx < end {
            'cand: {
                let cell = Cell::new(anchor.x + circle.x[idx], anchor.y + circle.y[idx]);
                if !(0..xs).contains(&cell.x)
                    || !(0..ys).contains(&cell.y)
                    || self.cell_has_centre(cell)
                {
                    break 'cand;
                }
                let sp = self.check_building_wcoord(who, cell, w, h, max, !nocity);
                if sp <= 1 {
                    break 'cand;
                }
                if big < 5 {
                    if sp < big
                        || cell.x == 0
                        || cell.x == xs - 1
                        || cell.y == 0
                        || cell.y == ys - 1
                    {
                        break 'cand;
                    }
                } else if !(frame < 1
                    || (1 < cell.x && cell.x < xs - 2 && 1 < cell.y && cell.y < ys - 2))
                {
                    break 'cand;
                }
                // `WorldData::buildings_allowed` — rock, mountain, forest and
                // the unnamed `0x40` take no building; an oil platform (0x1a6)
                // is the one type that skips the test.
                if ident != Ident::OilPlatform && !self.world.buildings_allowed(cell) {
                    break 'cand;
                }
                if self.cell_is_ocean(cell) != (bt.has(flags::WATER)) {
                    break 'cand;
                }
                let pad = |size: i32| {
                    if size < 4 {
                        (4 - size) * (UNITS_PER_TILE / 2)
                    } else {
                        0
                    }
                };
                let cand = Pos::new(
                    cell.x * UNITS_PER_CELL + bt.x_size * (UNITS_PER_TILE / 2) + pad(bt.x_size),
                    cell.y * UNITS_PER_CELL + bt.y_size * (UNITS_PER_TILE / 2) + pad(bt.y_size),
                );
                // `local_34`, the out-parameter of this very call: what the site
                // would gather. It is zero for every type but a non-flat gather
                // one, and it is what the woodcutter's branch below scores by.
                let (block, slots) = self.blocked_site_slots(Some(who), rec, cand, None);
                let mut cand = cand;
                if block != crate::place::Blocked::Clear {
                    // **The dock's slide** (`docs/AI.md` §21). Where
                    // `blocked_site` refuses the centred position, the
                    // `is(0x1b0)` arm at `006e2725` walks a block of whole
                    // tiles around it and takes the **first** that clears —
                    // `dx` outer, `dy` inner, both inclusive, under the same
                    // `|dx| + |dy| <= max` the extent carries. Nothing else
                    // may slide.
                    if !is_dock {
                        break 'cand;
                    }
                    // The base is the **unpadded** centre — the arm rebuilds
                    // it from the cell rather than reusing the padded `cand`.
                    // A dock is 4×4, so the two agree; a smaller type in this
                    // lineage would not, and this is what the listing does.
                    let base = Pos::new(
                        cell.x * UNITS_PER_CELL + bt.x_size * (UNITS_PER_TILE / 2),
                        cell.y * UNITS_PER_CELL + bt.y_size * (UNITS_PER_TILE / 2),
                    );
                    // `dy` starts at `-(w / 2)`, not `-(h / 2)`: `local_58`
                    // is loaded from the `dx` initialiser once and never
                    // reloaded. It is invisible while `w == h`.
                    let (hx, hy) = (w / 2, h / 2);
                    let mut slid = None;
                    'slide: for dx in -hx..=hx {
                        for dy in -hx..=hy {
                            if dx.abs() + dy.abs() > max {
                                continue;
                            }
                            let p = Pos::new(
                                base.x + dx * UNITS_PER_TILE,
                                base.y + dy * UNITS_PER_TILE,
                            );
                            // The out-parameter is null here, so `slots`
                            // keeps what the refused call left it — inert,
                            // because no dock is gather-scored.
                            if self.blocked_site(Some(who), rec, p, None)
                                == crate::place::Blocked::Clear
                            {
                                slid = Some(p);
                                break 'slide;
                            }
                        }
                    }
                    match slid {
                        Some(p) => cand = p,
                        None => break 'cand,
                    }
                }
                if self.world.owner(cell).player().is_some_and(|o| o != who) {
                    break 'cand;
                }
                let d = vector_dist(cell.x - anchor.x, cell.y - anchor.y);
                let mut score = 1000;
                if !nocity || unlimited || !fortlike {
                    let f = self.find_friends(rec, cell, city, who);
                    if f == 0 {
                        match ident {
                            Ident::Farm | Ident::Mine => {
                                // **A farm's own distance is not the
                                // spiral's.** The general arm subtracts the
                                // anchor's *cell* (`006e1f9a`, `>> 8`); the
                                // `0x1a1`/`0x1a3` arm at `006e2004` builds
                                // both sides again in **tiles** — the
                                // candidate as `cell * 4 + 2`, the anchor as
                                // `div_3_table[(pos ^ 0x63637) >> 6]`, which
                                // is its exact position and not its cell's
                                // centre. Four times the resolution is what
                                // separates the anchor's near neighbours
                                // from its far ones: in cells all eight are
                                // 1 (`docs/AI.md` §22).
                                let a = self.buildings[near].pos.tile();
                                let d = vector_dist(
                                    cell.x * TILES_PER_CELL + 2 - a.x,
                                    cell.y * TILES_PER_CELL + 2 - a.y,
                                )
                                .max(1);
                                self.mark(SITE_SPIRAL);
                                let r = self.rng.roll();
                                score = 4000 / d + r % 500;
                            }
                            Ident::Woodcutter if frame == 0 => {
                                score = if d > 3 { 500 } else { 1000 };
                                if d > 4 {
                                    score /= 2;
                                }
                            }
                            _ => {
                                if d > 4 {
                                    score = 333;
                                }
                            }
                        }
                    } else {
                        score = (f + 2) * 1000;
                        if tower {
                            score *= f + 2;
                        }
                    }
                    if is_enhancer(ident) && city.is_none() {
                        // `get_town(cand) == city_o`: an enhancer stays in the
                        // city it is placed for; with no city there is none.
                        break 'cand;
                    }
                } else {
                    score = d * 1000;
                    if self.world.tile_mask(cand.tile()) & tile::CITY_RADIUS != 0 {
                        score /= 2;
                    }
                }
                if !is_fort {
                    if tower {
                        let near_tower = self.buildings.iter().any(|b| {
                            b.alive
                                && b.ty
                                    .is_some_and(|t| self.build_types[t].ident == Ident::Tower)
                                && vector_dist(b.pos.x - cand.x, b.pos.y - cand.y) <= 0x600
                        });
                        if near_tower {
                            score /= 8;
                        }
                    }
                } else {
                    // `danger[]` is not kept: nothing added.
                    let o2 = self.world.second(cell).player();
                    match o2 {
                        Some(p) if p != who && !self.is_ally(who, p) => {
                            score *= if self.lobby.team_style == 2 { 4 } else { 8 };
                        }
                        _ => score /= 2,
                    }
                }
                // `w1` and `plenty` are the gather-amount weights; with no
                // `gather_at` amounts they stay at their initial values.
                let (w1, plenty) = (1, 0);
                if !scored_by_gather {
                    // `0xff − val`, `WData.val` — the map maker's own
                    // city-site value for the cell, so a *better* site
                    // scores **lower** here. It is not zero on the islands
                    // map: run38's own world dump gives the cells East
                    // Indies' second farm parts on 20, 31, 6, 21 and 27.
                    score += 0xff - self.world.cell_data(cell).val as i32;
                } else if ident == Ident::Woodcutter {
                    score *= slots * slots * slots;
                    if !(slots > 2 || frame == 0) {
                        break 'cand;
                    }
                    score = (score + plenty) * w1;
                } else {
                    // `World::gather_at` for an oil platform: no amounts here.
                    let found = false;
                    if !found {
                        break 'cand;
                    }
                    score = (score + plenty) * w1;
                }
                if score < best {
                    break 'cand;
                }
                // `circle_radius[3] < local_2c` — the **current** index, not
                // the loop's start: once a second candidate has improved on a
                // best beyond ring 3, the spiral strides by three. Comparing
                // `start` here (which is 0, 1 or exactly `radius[3]`) meant it
                // never engaged, and the extra cells were extra draws. The
                // stride set here is spent by the step at the **bottom** of
                // this iteration, not the next one's top (`docs/AI.md` §20);
                // that one cell is the fuzzed map's thirtieth candidate.
                if !scored_by_gather && best != 0 && !tower && idx > circle.radius[3] {
                    step = 3;
                }
                best = score;
                best_sp = sp;
                best_cand = Some(cand);
            }
            idx += step;
        }
        let Some(mut cand) = best_cand else {
            return false;
        };

        // 4.5 The corner tile, the builder, the jitter.
        let corner = self.tile_corner(rec, cand);
        let radius = match city {
            Some(c) => self
                .city_radius(who, self.buildings[self.cities[c].building].ty)
                .min(0x40),
            None => 0x1200,
        };
        let mut builder: Option<(usize, u8)> = None;
        if frame != 0 {
            let citizen = self.tech_tree.types.iter().position(|d| d.kind.is_unit());
            let mut best_d = i32::MAX;
            for u in 0..self.units.len() {
                let unit = &self.units[u];
                if unit.owner != who || !unit.alive() || !unit.on_map {
                    continue;
                }
                let is_citizen = citizen.is_some_and(|c| {
                    unit.ty
                        .and_then(|r| self.unit_types[r].tree)
                        .is_some_and(|t| self.tech_tree.is(t, c, false))
                });
                if !is_citizen {
                    continue;
                }
                let kind = self
                    .action_of(u)
                    .map_or(index::NONE, |i| unit.orders[i].index());
                if !matches!(kind, index::NONE | index::GATHER | index::BUILD_AT) {
                    continue;
                }
                if self.world.region_of(unit.pos.cell()) != areg {
                    continue;
                }
                // The distance in tiles, with the penalties in tiles of
                // radius (`docs/AI.md` §13 — the listing settles the scale).
                //
                // **From the corner tile, and each coordinate floored on its
                // own.** `006e28b2`–`006e28ec` reads the unit's own `x`/`y`,
                // converts each to a tile through `div_3_table` (a floor),
                // and subtracts it from `local_5c`/`local_70` — the corner
                // tile computed at `006e2656`, not the candidate's centre.
                // Taking the difference in world units and dividing once is
                // a different function wherever the two floors part, and
                // that is what run71's frame 4176 turns on: `1/11` and
                // `1/19` **tie at 27** under the original's arithmetic and
                // the earlier unit keeps the tie, where difference-then-
                // divide gave 28 against 25 and sent the wrong citizen.
                let tile = unit.pos.tile();
                let mut d = vector_dist(corner.x - tile.x, corner.y - tile.y);
                match kind {
                    index::GATHER => {
                        let target = self.action_of(u).and_then(|i| match unit.orders[i].body {
                            Body::Gather(g) => Some(g.building),
                            _ => None,
                        });
                        let good = target
                            .and_then(|b| self.buildings.get(b))
                            .map(|_| self.building_ident(target.unwrap()))
                            .and_then(gather_good);
                        d += match good {
                            Some(0) => radius * 3 / 2,
                            Some(1) => radius / 2,
                            _ => radius / 3,
                        };
                    }
                    index::BUILD_AT => {
                        let wonder = self
                            .action_of(u)
                            .is_some_and(|i| match unit.orders[i].body {
                                Body::Build(b) => self.building_ident(b) == Ident::Wonder,
                                _ => false,
                            });
                        if wonder {
                            continue;
                        }
                        d += radius * 2;
                    }
                    _ => {}
                }
                if d < best_d {
                    best_d = d;
                    builder = Some((u, kind));
                }
            }
            if builder.is_none() {
                return false;
            }
        }
        let (ex, ey) = if ident != Ident::Woodcutter {
            (1, 1)
        } else {
            ((4 - bt.x_size).max(0), (4 - bt.y_size).max(0))
        };
        if !is_dock && (ex > 0 && ey > 0) {
            if ident != Ident::Woodcutter {
                let mut best_r = -1;
                // Both bounds are **inclusive** in the original
                // (`while (uVar11 <= uVar19)`, `while (local_1c <=
                // local_58)` at `006e2a78`), so `ex == ey == 1` is a
                // 2×2 of sub-positions and four draws, not one.
                for dx in 0..=ex {
                    for dy in 0..=ey {
                        let c = Pos::new(
                            ((corner.x + dx) * 2 + bt.x_size) * (UNITS_PER_TILE / 2),
                            ((corner.y + dy) * 2 + bt.y_size) * (UNITS_PER_TILE / 2),
                        );
                        if self.blocked_site(Some(who), rec, c, None)
                            == crate::place::Blocked::Clear
                        {
                            self.mark(SITE_JITTER);
                            let r = self.rng.roll() % 100;
                            if r >= best_r {
                                best_r = r;
                                cand = c;
                            }
                        }
                    }
                }
                if best_r < 0 {
                    return false;
                }
            } else {
                // The compass ring `radius[ex]` around the corner: nine
                // entries for a one-tile margin (`ex == 1`); wider margins
                // need the full `move_x/move_y` table (`docs/AI.md` §13).
                let mut best_f = -1;
                let n = if ex >= 1 { 9 } else { 1 };
                for k in 0..n {
                    let c = Pos::new(
                        (bt.x_size + (MOVE_X[k] + corner.x) * 2) * (UNITS_PER_TILE / 2),
                        (bt.y_size + (MOVE_Y[k] + corner.y) * 2) * (UNITS_PER_TILE / 2),
                    );
                    // The same out-parameter again (`produce_building:988`):
                    // the ring's best sub-position is the one that would
                    // gather most, on a strict improvement.
                    let (block, f) = self.blocked_site_slots(Some(who), rec, c, None);
                    if block == crate::place::Blocked::Clear && f > best_f {
                        best_f = f;
                        cand = c;
                    }
                }
                if best_f < 0 {
                    return false;
                }
            }
        }

        // 4.6 Creation: paid after frame 0, the site, the builder's order.
        if frame != 0 {
            let charges = self.building_price(who, rec);
            let available = self.holdings[who as usize].available;
            // `pay_cost(who, −1, city, escrow, 0)` (`produce_building:1024`).
            cost::pay(
                &charges,
                &mut self.ledgers[who as usize],
                &available,
                escrow,
            );
            self.economy_changed(who);
        }
        let o = self.init_build(who, rec, cand, false);
        if frame != 0 {
            if let Some((u, kind)) = builder {
                if kind == index::BUILD_AT {
                    // `QUEUE_LAST`: behind the build in hand.
                    self.add_build_order(u, o, QueuePos::Last, true);
                } else {
                    // `QUEUE_NEW`: the swarm ring's approach, then the order.
                    self.clear_orders(u);
                    self.swarm_around(u, o, Body::Build(o), true);
                }
            }
        } else {
            self.activate(o, false, true);
        }
        // The census, adjusted in place (report §4.6 lines 1114–1151), so
        // a second producer in the same step machine sees the tile and the
        // citizen as spent: the city's `filled` and `space[]`, then the
        // builder taken from the gatherers or the free peasants — run8's
        // frame 2 shows `gatherers 5 → 4` for exactly this farm.
        let w = who as usize;
        if let Some(c) = city {
            let leader = &mut self.ai[w];
            if leader.city_ai.len() <= c {
                leader.city_ai.resize(c + 1, crate::ai::CityAi::default());
            }
            let ca = &mut leader.city_ai[c];
            ca.filled += 1;
            for n in 2..=best_sp.min(4) {
                let i = (n - 2) as usize;
                ca.space[i] = (ca.space[i] - 1).max(0);
            }
        }
        if frame != 0 {
            let leader = &mut self.ai[w];
            let reg = areg.map(|r| r as usize);
            let gatherer = builder.is_some_and(|(_, k)| k == index::GATHER);
            if gatherer {
                leader.census.gatherers -= 1;
                if let Some(c) = city {
                    leader.city_ai[c].gatherers -= 1;
                }
                if let Some(r) = reg
                    && let Some(v) = leader.census.reg_gatherers.get_mut(r)
                {
                    *v -= 1;
                }
            } else {
                leader.census.free_peasants -= 1;
                if let Some(c) = city {
                    let f = &mut leader.city_ai[c].free;
                    *f = f.wrapping_sub(1);
                }
                if let Some(r) = reg
                    && let Some(v) = leader.census.reg_free_peasants.get_mut(r)
                {
                    *v -= 1;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_circle_is_rings_of_the_octagonal_metric_in_the_original_s_order() {
        let c = circle();
        assert_eq!(c.radius[0], 1, "ring 0 is the centre");
        assert_eq!((c.x[0], c.y[0]), (0, 0));
        // Ring 1: the eight neighbours, x outer then y inner.
        assert_eq!(c.radius[1], 9);
        assert_eq!(
            (1..9).map(|i| (c.x[i], c.y[i])).collect::<Vec<_>>(),
            [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1)
            ]
        );
        // Every entry of ring r is at octagonal distance r, and the rings
        // are contiguous.
        for r in 1..=0x40usize {
            for i in c.radius[r - 1]..c.radius[r] {
                assert_eq!(vector_dist(c.x[i], c.y[i]), r as i32, "entry {i}");
            }
        }
        assert!(c.x.len() <= MAX_POINTS);
    }

    /// The two tables agree with each other: each `GRID_THREES` row is
    /// exactly the tiles of one 3×3 of the 4×4 that are not in the centre
    /// 2×2, and the centre 2×2 is `GRID_ORDER[0..4]`.
    #[test]
    fn grid_threes_are_the_non_centre_tiles_of_each_three_square() {
        let centre: Vec<(i32, i32)> = GRID_ORDER[..4].to_vec();
        assert_eq!(centre, [(1, 1), (2, 1), (1, 2), (2, 2)]);
        let corners = [(1, 1), (0, 1), (0, 0), (1, 0)];
        for (row, (cx, cy)) in GRID_THREES.iter().zip(corners) {
            let mut got: Vec<(i32, i32)> = row.iter().map(|&k| GRID_ORDER[k]).collect();
            got.sort_unstable();
            let mut want: Vec<(i32, i32)> = (0..3)
                .flat_map(|j| (0..3).map(move |i| (cx + i, cy + j)))
                .filter(|t| !centre.contains(t))
                .collect();
            want.sort_unstable();
            assert_eq!(got, want, "3×3 at ({cx},{cy})");
        }
    }

    /// A 4×4-cell world (16×16 tiles), two players, and a corner well
    /// inside it; `block` marks tiles `BLOCKED` by their `(dx, dy)` from
    /// the corner.
    fn space(block: &[(i32, i32)]) -> i32 {
        let mut w = crate::World::new(4, 4);
        for &(i, j) in block {
            w.set_tile_bits(Pos::new(4 + i, 4 + j), tile::BLOCKED);
        }
        let sim = Sim::new(crate::Tuning::RON, w, 2);
        sim.space_at_corner(1, 4, 4, false)
    }

    /// The early-out is on the centre 2×2, not the top row: a blocked
    /// corner tile leaves three 3×3s free and scores 3 (the harness scored
    /// 0 until run20's `CITY` record put `space[0..1]` ten cells short —
    /// `docs/AI.md` §15.9), and a blocked centre tile is 0 outright.
    #[test]
    fn a_blocked_edge_tile_leaves_a_three_square_but_a_blocked_centre_is_nothing() {
        assert_eq!(space(&[]), 4);
        assert_eq!(space(&[(0, 0)]), 3);
        assert_eq!(space(&[(3, 0)]), 3);
        assert_eq!(space(&[(0, 3), (3, 3)]), 3);
        assert_eq!(space(&[(1, 1)]), 0);
        assert_eq!(space(&[(2, 2)]), 0);
        // Both rows out: no 3×3, seven blocked or eight — 2 either way.
        assert_eq!(
            space(&[(0, 0), (1, 0), (2, 0), (3, 0), (0, 3), (1, 3), (2, 3)]),
            2
        );
        assert_eq!(
            space(&[
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (0, 3),
                (1, 3),
                (2, 3),
                (3, 3)
            ]),
            2
        );
        // Every non-centre tile blocked: still 2, never 0.
        let ring: Vec<(i32, i32)> = GRID_ORDER[4..].to_vec();
        assert_eq!(space(&ring), 2);
    }

    /// `check_building_wcoord`'s three gates on the cell itself, before any
    /// corner is looked at: another leader's, occupied (`flags & 0x70`),
    /// or `blocked == 0x10`.
    #[test]
    fn check_building_wcoord_gates_on_the_cell_before_the_corners() {
        let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(4, 4), 2);
        let cell = Cell::new(1, 1);
        assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 4);
        let mut d = sim.world.cell_data(cell);
        d.flags |= 0x10;
        sim.world.set_cell_data(cell, d);
        assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
        d.flags &= !0x70;
        d.blocked = 0x10;
        sim.world.set_cell_data(cell, d);
        assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
        d.blocked = 0;
        sim.world.set_cell_data(cell, d);
        assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 4);
        sim.world
            .set_owner(cell, crate::Owner::Player(0), crate::Owner::None);
        assert_eq!(sim.check_building_wcoord(1, cell, 0, 0, 1, false), 0);
    }

    /// **`find_friends` asks whose footprint covers a cell's centre tile,
    /// not whose centre is in the cell** (`docs/AI.md` §26).
    ///
    /// The geometry is East Indies' AI Village, off run58's own dump: 7×7
    /// at `(39264, 40032)`, corner tiles `201..207 × 205..211`. Centre
    /// tiles are four apart, so a seven-tile footprint covers two of them
    /// on each axis — the city is the neighbour of **four** cells, and its
    /// own is one of them. Every ordinary building is four tiles or fewer
    /// and covers exactly one, which is why the two readings agree
    /// everywhere else.
    #[test]
    fn the_city_centre_is_the_friend_of_four_cells() {
        let mut sim = Sim::new(crate::Tuning::RON, crate::World::new(60, 60), 2);
        let rec = sim.build_types.len();
        sim.build_types.push(crate::build::BuildType {
            ident: Ident::Village,
            x_size: 7,
            y_size: 7,
            hits: 100,
            ..crate::build::BuildType::default()
        });
        let pos = Pos::new(39264, 40032);
        let b = sim.add_building(1, pos, 8);
        sim.buildings[b].ty = Some(rec);
        assert_eq!(
            (pos.cell().x, pos.cell().y),
            (51, 52),
            "the Village's own cell — what this crate used to answer, alone"
        );
        let corner = sim.tile_corner(rec, pos);
        assert_eq!((corner.x, corner.y), (201, 205), "run58's `1/2000`");
        // The mask is the callee's first gate: a tile no building has
        // masked answers `None` before any footprint is looked at.
        for t in sim.footprint(rec, corner) {
            sim.world
                .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
        }
        let friend_of: Vec<(i32, i32)> = (50..53)
            .flat_map(|x| (51..54).map(move |y| (x, y)))
            .filter(|&(x, y)| sim.building_placed_at(1, Cell::new(x, y)) == Some(b))
            .collect();
        assert_eq!(
            friend_of,
            [(50, 51), (50, 52), (51, 51), (51, 52)],
            "the four cells whose centre tile (cell·4 + 2) the 7×7 covers"
        );
        assert_eq!(
            sim.building_placed_at(0, Cell::new(51, 52)),
            None,
            "`param_3` filters by owner"
        );
        // Four tiles or fewer: exactly one cell, and the two readings agree.
        sim.build_types[rec].x_size = 4;
        sim.build_types[rec].y_size = 4;
        sim.buildings[b].pos = Pos::new(51 * 768 + 384, 52 * 768 + 384);
        let small: Vec<(i32, i32)> = (50..53)
            .flat_map(|x| (51..54).map(move |y| (x, y)))
            .filter(|&(x, y)| sim.building_placed_at(1, Cell::new(x, y)) == Some(b))
            .collect();
        assert_eq!(small, [(51, 52)]);
    }
}
