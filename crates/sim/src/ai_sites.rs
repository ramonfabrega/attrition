//! City sites and the city AI — `Leader::compute_sites@006cc950`,
//! `Leader::compute_site_stats@006cd040`, `Leader::found_cities@006c7a60`
//! and `Leader::produce_city@006cb120`; `docs/AI.md` §2.7, §2.13, §2.12,
//! §2.17.
//!
//! **Coordinates.** The original's "tile" — its `WData` record, `world+0x134`
//! indexed by `world.xs` — is this simulation's **cell**, and its `0x300`
//! coordinate step is [`UNITS_PER_CELL`]. A site's `wx, wy` are therefore
//! cell coordinates, a site position is `Pos::new(wx * UNITS_PER_CELL, …)`
//! (the cell's *corner*, which is what `blocked_site` and `init_build` are
//! passed), and every `vector_dist` on this path is measured in cells. The
//! original's finer `TData` grid (`world+0x138`, four by four per `WData`)
//! is this simulation's tile grid, and the two masks these functions read
//! are indexed at the cell's **centre tile** (`(4x + 2, 4y + 2)`, the `+ 4`
//! byte bias in the decompile's address arithmetic).
//!
//! **What the flat harness world cannot supply is a named seam**, each
//! answering as an empty, fully-explored map would; the list is repeated in
//! the module's report:
//!
//! * `WorldData::was_seen` — always true (the lobby is `REVEAL_MAP`).
//! * `GameAccessConst::find_tcoord_z` — [`World::tile_z`], a table the map
//!   loader pins from a `DUMP_ALL` dump's `master_land_heights`; 0 on a
//!   flat world, where the `z / 25` term of a candidate tile's score
//!   vanishes. (Seam retired 2026-08-24: on run9 it is the whole difference
//!   between the sampler's `(51,16)/291` and the original's `(52,14)/370`.)
//! * `WorldData::danger[who]` (`world+0x13c`, a half-cell-resolution `int`
//!   grid — **not** territory; `docs/AI.md` §2.13 step 8 and §2.12 name it
//!   "my territory at the site's cell", which the PDB contradicts) —
//!   answered 0 by [`Sim::site_danger`].
//! * `LeaderData::get_target` (team style 2) — no target.
//! * `leader_flags2 & 0x10` (the `found_cities` early-out) and
//!   `leader_flags & 8` (the human-cap bypass) — never set.
//! * `LeaderData::village_num` — 0; run8's leader block shows it 0 with a
//!   Small City standing, so it is not the settlement count.
//! * The map type `world+0x30` is taken as [`crate::ai::Lobby::map_style`],
//!   which is also `GameInfo.map_style`; the original reads two fields.
//! * `Region.coords` order — the original walks the region's own coordinate
//!   list (flood-fill order); [`crate::world::World::cells_in`] is
//!   row-major, so the sampler visits a different subset of the same region.
//! * `ObjectsData::find_city`/`find_unit` search *friendly* objects (mine
//!   and my allies'); here they search mine. Both are only used as presence
//!   flags on this path.
//! * The per-player object arrays are iterated as this simulation's global
//!   `units` / `buildings` / `cities` lists filtered by owner, in index
//!   order; the original iterates object numbers.
//! * `TypeData.type == PEASANTS || == PEASANTSKOREAN` (an exact pair) is
//!   taken as the citizen *lineage*, the same predicate `ai_place.rs` uses
//!   for `FILTER_BASE_TYPE 0x32`.

use crate::ai::{Census, SITES, Site};
use crate::build::{self, Ident};
use crate::economy::RESOURCES;
use crate::orders::{Body, index};
use crate::place::Blocked;
use crate::tech::{Kind, TypeId};
use crate::world::{Cell, Owner, Pos, Terrain, UNITS_PER_CELL, tile, vector_dist};
use crate::{Player, Sim, cost};

/// `move_x[0..25]` / `move_y[0..25]` — read from `.rdata` at `0x00adcaf0`
/// and `0x00adc400` (`?move_x@@3QBHB` / `?move_y@@3QBHB` in `rise_z.map`,
/// `compass.obj`). Entry 0 is the centre, 1..8 the compass ring, 9..24 the
/// sixteen cells of the next ring: the 5×5 square, each cell once.
const MOVE_X: [i32; 25] = [
    0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2, 2, -2,
];
const MOVE_Y: [i32; 25] = [
    0, -1, -1, -1, 0, 1, 1, 1, 0, -2, -2, -2, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, 2, 2,
];

/// `move_x[81..121]` / `move_y[81..121]` — the same tables' radius-5 ring,
/// the forty cells of the 11×11 border, walked clockwise from `(−5, −5)`.
/// (81 = 1 + 8 + 16 + 24 + 32, the cells inside radius 4.)
const RING5_X: [i32; 40] = [
    -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3,
    -4, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5,
];
const RING5_Y: [i32; 40] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4,
];

/// `corner_x[0..5]` / `corner_y[0..5]` at `0x00adc3e0` / `0x00adc3c0` — the
/// centre and the four corners, the nomad's wood test.
const CORNER_X: [i32; 5] = [0, -1, 1, 1, -1];
const CORNER_Y: [i32; 5] = [0, -1, -1, 1, 1];

/// What `compute_site_stats` writes back: the score, the distance figure,
/// and the site's coordinates, which a `keep` call may have moved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SiteStats {
    val: i32,
    dist: i32,
    wx: i32,
    wy: i32,
}

/// `a * b`, wrapping — the original's arithmetic is 32-bit `imul` and the
/// score genuinely overflows on a rich tile (`found_cities` clamps the
/// product to 999,999 afterwards, which only makes sense if it wraps).
const fn mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// `a * n / d`, the product wrapping and the division truncating toward
/// zero, as `imul` then `idiv` do.
const fn mul_div(a: i32, n: i32, d: i32) -> i32 {
    a.wrapping_mul(n) / d
}

impl Sim {
    // ------------------------------------------------------------------
    // Seams and small helpers
    // ------------------------------------------------------------------

    /// `WorldData::danger[who]` at a site's cell — `world+0x13c`, an `int`
    /// grid at half the cell resolution (`(wy >> 1) * reg_xs + (wx >> 1)`).
    /// Nothing writes it here, so it answers 0, which makes the divisor of
    /// §2.13 step 8 one and the `> 0` test of §2.12 false. Named as a seam
    /// rather than routed to [`crate::world::World::danger`], which is
    /// keyed by region and is a different grid.
    fn site_danger(&self, _who: Player, _wx: i32, _wy: i32) -> i32 {
        0
    }

    /// `WorldData::is_ocean` — the region's terrain.
    fn site_is_ocean(&self, c: Cell) -> bool {
        self.world
            .region_of(c)
            .is_some_and(|r| self.world.terrain(r) == Terrain::Sea)
    }

    /// `Forts::find_fort(wx, wy, who)`: a live fort of `who` standing on
    /// exactly this cell.
    fn fort_at(&self, who: Player, c: Cell) -> bool {
        self.buildings.iter().any(|b| {
            b.alive
                && b.owner == who
                && b.ty.is_some_and(|t| build::is_fort(&self.build_types, t))
                && b.pos.cell() == c
        })
    }

    /// `types.list[BASE_UNITTYPES]` — the tree's first unit, the Citizen.
    fn citizen_root(&self) -> Option<TypeId> {
        self.tech_tree.types.iter().position(|d| d.kind.is_unit())
    }

    /// A unit in the Citizen line (the seam for `type == PEASANTS ||
    /// type == PEASANTSKOREAN`).
    fn unit_is_citizen(&self, u: usize) -> bool {
        let Some(root) = self.citizen_root() else {
            return false;
        };
        self.units[u]
            .ty
            .and_then(|r| self.unit_types[r].tree)
            .is_some_and(|t| self.tech_tree.is(t, root, false))
    }

    /// The build record of a city level, by ident.
    fn city_record(&self, ident: Ident) -> Option<usize> {
        self.build_types.iter().position(|b| b.ident == ident)
    }

    /// The six goods' tree ids, in `econ`/`Resource` order.
    fn good_types(&self) -> Vec<TypeId> {
        self.tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == Kind::Good)
            .map(|(i, _)| i)
            .take(RESOURCES)
            .collect()
    }

    /// `blocked_site(t, wx * 0x300, wy * 0x300, who, −1, 0) == 0` — the
    /// cell-corner form every call on this path takes.
    fn site_clear(&self, who: Player, rec: usize, wx: i32, wy: i32) -> bool {
        self.blocked_site(
            Some(who),
            rec,
            Pos::new(wx * UNITS_PER_CELL, wy * UNITS_PER_CELL),
            None,
        ) == Blocked::Clear
    }

    /// The reference objects `compute_sites` hands `compute_site_stats`:
    /// with cities of mine in the region, the nearest friendly city (and
    /// its centre object); with none, the nearest friendly citizen. Only
    /// their *presence* is read downstream, so this answers two flags.
    fn site_reference(&self, who: Player, reg: u16, at: Pos) -> (bool, bool) {
        if Census::reg(&self.ai[who as usize].census.reg_cities, reg) == 0 {
            let unit = self.units.iter().enumerate().any(|(u, x)| {
                x.owner == who
                    && x.alive()
                    && x.on_map
                    && self.world.region_of(x.pos.cell()) == Some(reg)
                    && self.unit_is_citizen(u)
            });
            (false, unit)
        } else {
            let found = self
                .cities
                .iter()
                .any(|c| c.alive && c.owner == who && c.reg == Some(reg));
            let _ = at;
            (found, found)
        }
    }

    // ------------------------------------------------------------------
    // compute_site_stats — §2.13
    // ------------------------------------------------------------------

    /// `Leader::compute_site_stats@006cd040`, read whole (506 lines).
    ///
    /// `city` and `unit` are the two object arguments as presence flags
    /// (the original passes indices and only ever tests their sign).
    /// `keep` is the original's eighth argument: with it set, the site may
    /// **slide** to the best-valued open cell of its own 5×5, and the moved
    /// coordinates come back in the result — the sampler's calls set it,
    /// the re-score of a kept site does not. Every early return leaves
    /// `val = dist = 0`.
    #[allow(clippy::too_many_arguments)]
    fn compute_site_stats(
        &self,
        who: Player,
        wx: i32,
        wy: i32,
        city: bool,
        unit: bool,
        reg: u16,
        keep: bool,
    ) -> SiteStats {
        let w = who as usize;
        // An early return leaves `val = dist = 0` — but **not** the site's
        // coordinates: the original's out-parameters are written by the
        // slide in step 3 and are not rolled back, so a return after the
        // slide reports the moved cell with a score of zero.
        let zero = |x: i32, y: i32| SiteStats {
            val: 0,
            dist: 0,
            wx: x,
            wy: y,
        };
        let (world_w, world_h) = (self.world.width(), self.world.height());
        let here = Cell::new(wx, wy);

        // 1. The centre tile inside a city radius, unseen, another
        //    leader's, or water.
        if self.world.tile_mask(here.centre_tile()) & tile::CITY_RADIUS != 0 {
            return zero(wx, wy);
        }
        // `was_seen(2wx + 1, 2wy + 1, who)` — the seam, always true.
        let mut ally_land = false;
        if let Owner::Player(o) = self.world.owner(here)
            && o != who
        {
            // A Conquer-the-World game, before my first city, may site on
            // an ally's land at a tenth of the score.
            if !self.lobby.conquest || self.city_num(who) != 0 || !self.is_ally(o, who) {
                return zero(wx, wy);
            }
            ally_land = true;
        }
        if self.site_is_ocean(here) {
            return zero(wx, wy);
        }

        // 2. The base is the cell's `val` byte, zeroed when the **Town's**
        //    footprint does not fit (not the Village's).
        let town = self.city_record(Ident::Town);
        let mut base = i32::from(self.world.cell_data(here).val);
        if town.is_none_or(|t| !self.site_clear(who, t, wx, wy)) {
            base = 0;
        }

        // 3. The 5×5. `danger`, `water`, `forts` and `target_adj` are read
        //    around the *original* centre even after the site slides.
        let mut danger = 0i32;
        let mut water = 0i32;
        let mut forts = 0i32;
        let mut target_adj = 1i32;
        let (mut sx, mut sy) = (wx, wy);
        for k in 0..25 {
            let (x, y) = (wx + MOVE_X[k], wy + MOVE_Y[k]);
            if x < 0 || y < 0 || x >= world_w || y >= world_h {
                continue;
            }
            // `was_seen` again — the seam.
            let cc = Cell::new(x, y);
            let owner = self.world.owner(cc);
            let coastal = self.world.cell_data(cc).flags & 0x100 != 0;
            if !self.site_is_ocean(cc) && !coastal {
                match owner {
                    Owner::Player(o) if o == who => {}
                    Owner::Player(o) => {
                        if self.lobby.team_style == 2 {
                            // `get_target` — the seam; no target, so this
                            // arm never fires.
                            if self.leader_target(who) == Some(o) {
                                target_adj += 1;
                                danger += 2;
                            }
                        } else if !self.is_ally(who, o) {
                            danger += 2;
                        }
                    }
                    _ => danger += if self.city_num(who) > 2 { 2 } else { 1 },
                }
            } else {
                water += 1;
                danger = (danger - 1).max(0);
            }

            // The slide, on a cell that is mine (or my ally's, on ally
            // land) and outside every city radius.
            let mine = match owner.player() {
                Some(o) => o == who || (ally_land && self.is_ally(who, o)),
                None => false,
            };
            if !(mine && keep && self.world.tile_mask(cc.centre_tile()) & tile::CITY_RADIUS == 0) {
                continue;
            }
            if self.fort_at(who, cc) {
                forts += 1;
            }
            let mut q = i32::from(self.world.cell_data(cc).val);
            if self.world.contains(Cell::new(x + 1, y + 1)) {
                q += i32::from(self.world.cell_data(Cell::new(x + 1, y)).val)
                    + i32::from(self.world.cell_data(Cell::new(x + 1, y + 1)).val)
                    + i32::from(self.world.cell_data(Cell::new(x, y + 1)).val);
            }
            // `find_tcoord_z(wx × 0x300 + 0x180, …) / 25` — the height of
            // the cell's centre tile, from the pinned table (`World::tile_z`;
            // 0 on a flat world).
            let q = (q >> 2) + self.world.tile_z(cc.centre_tile()) / 25;
            if base < q && town.is_some_and(|t| self.site_clear(who, t, x, y)) {
                sx = x;
                sy = y;
                base = q;
            }
        }

        // 4.
        if base < 1 {
            return zero(sx, sy);
        }
        let parity = base & 3;
        let mut v = base;

        // 5. Many landmasses and no dock: an ocean cell on the radius-5
        //    ring is worth thirty times as much.
        if self.world.landmasses() > 2
            && self
                .city_record(Ident::Dock)
                .is_none_or(|d| self.buildings_of_line(who, d) == 0)
        {
            for k in 0..40 {
                let (x, y) = (sx + RING5_X[k], sy + RING5_Y[k]);
                if x < 0 || y < 0 || x >= world_w || y >= world_h {
                    continue;
                }
                if self.site_is_ocean(Cell::new(x, y)) {
                    v = mul(base, 30);
                    break;
                }
            }
        }

        // 6. Water, danger and forts.
        let cities = self.city_num(who);
        v = mul(v, 250) / (water + 1);
        if cities == 1 {
            if danger != 0 {
                v = mul_div(danger.min(9) + 7, v, 8);
            }
        } else if cities == 2 {
            if danger != 0 {
                v = mul_div(danger.min(4) + 7, v, 8);
            }
        } else if forts != 0 {
            v = mul_div(v, 3, 2);
        }
        v = mul_div(target_adj + 1, v, 2);

        // 7. The map edge, on the moved coordinates.
        let map = self.lobby.map_style;
        if map != 0xc && map != 0x11 {
            let (lo_x, hi_x, lo_y, hi_y) = if cities < 3 {
                (world_w / 5, world_w * 4 / 5, world_h / 5, world_h * 4 / 5)
            } else {
                (
                    world_w / 10,
                    world_w * 9 / 10,
                    world_h / 10,
                    world_h * 9 / 10,
                )
            };
            if sx < lo_x || hi_x < sx {
                v /= 4;
            }
            if !(lo_y <= sy && sy <= hi_y) {
                v /= 4;
            }
        }

        // 8. Danger at the site, then the cell's *second* claimant.
        let site = Cell::new(sx, sy);
        v /= self.site_danger(who, sx, sy).max(1);
        match self.world.second(site).player() {
            Some(o) if o == who => {}
            Some(o) => {
                if self.is_ally(o, who) {
                    v /= 2;
                } else {
                    v = mul(v, 4);
                }
            }
            // `Ambiguous` is the original's −2 and reads as unowned.
            None => v = mul(v, 2),
        }

        // 9. The region's crowding.
        if self.lobby.map_style != 0x14 {
            let c = &self.ai[w].census;
            let rc = Census::reg(&c.reg_cities, reg);
            let rl = Census::reg(&c.reg_land, reg);
            if rc == 0 {
                v = mul(v, 2);
            }
            if rc == 1 {
                v = mul(v, 2);
            }
            if rl == 0 {
                v = mul(v, 4);
            }
            if rl < 3 {
                v = mul(v, 4);
            }
            if rl < rc {
                v = mul(v, 2);
            }
            if rl < rc * 2 {
                v = mul(v, 2);
            }
        }

        let goods = self.world.cell_data(site).goods;

        // 11 (the decompile's order: the nomad block precedes the goods
        //    loop). A nomad scores by the distance to its nearest citizen,
        //    and needs wood in one of the five corner cells.
        if cities == 0 {
            let centre = Pos::new(
                sx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                sy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            );
            let mut best: Option<i32> = None;
            for u in 0..self.units.len() {
                let x = &self.units[u];
                if x.owner != who || !x.alive() || !self.unit_is_citizen(u) {
                    continue;
                }
                let d = vector_dist(x.pos.x - centre.x, x.pos.y - centre.y);
                if best.is_none_or(|b| d <= b) {
                    best = Some(d);
                }
            }
            if let Some(d) = best {
                v /= d / 0xc00 + 1;
            }
            if self.lobby.starting_resources != 8 {
                let mut wood = false;
                for k in 0..5 {
                    let (x, y) = (sx + CORNER_X[k], sy + CORNER_Y[k]);
                    if x < 0 || y < 0 || x >= world_w || y >= world_h {
                        continue;
                    }
                    if self.world.cell_data(Cell::new(x, y)).goods & 2 != 0 {
                        wood = true;
                        break;
                    }
                }
                if !wood {
                    return zero(sx, sy);
                }
            }
        }

        // 10. The goods near the cell against the economy's wants.
        let mut have = 0i32;
        let mut lack = 0i32;
        for (g, t) in self.good_types().into_iter().enumerate() {
            if self.type_avail(who, t) == 0 {
                continue;
            }
            let econ = self.ai[w].econ[g];
            if goods & (1u8 << g) == 0 {
                lack |= econ & 6;
            } else {
                have |= econ & 6;
                if econ & 2 != 0 {
                    v = mul_div(v, 3, 2);
                }
                if econ & 4 != 0 {
                    v = mul_div(v, 3, 2);
                }
            }
        }
        if have == 0 {
            v = mul_div(v, 2, 3);
        }
        if lack & !have != 0 {
            if parity == 0 {
                v /= 2;
            } else {
                v = mul_div(v, 3, 4);
            }
        }

        // 12. The distance term. **Settled** (`docs/AI.md` §13's first open
        //     item): the two tests inside the per-city loop are `return`s,
        //     not `break`s — at `006cd9e7` and `006cd9f4` both branches
        //     jump to `006cdc68`, the epilogue, which is *past* the stores
        //     of `val` and `dist` at `006cdc60`. A `break` would have to
        //     land on `006cda91`, and could not: the accumulator and the
        //     city pointer share `%ecx` inside the loop. So a dead city
        //     slot, or one city of mine outside the site's region, scores
        //     the site **0**.
        if !unit {
            // No reference object at all: the constants, and the loop is
            // never entered.
            let stats = SiteStats {
                val: mul(3, v),
                dist: 9,
                wx: sx,
                wy: sy,
            };
            return self.site_stats_tail(who, stats, sx, sy, cities, ally_land);
        }
        let k = {
            let mut k = 1i32;
            if city {
                for c in self.cities.iter().filter(|c| c.owner == who) {
                    if !c.alive || c.reg != Some(reg) {
                        return zero(sx, sy);
                    }
                    let cp = c.pos.cell();
                    let d = vector_dist(sx - cp.x, sy - cp.y);
                    if d < 5 {
                        v /= 2;
                    }
                    if d < 8 {
                        v /= 2;
                    }
                    k += if cities > 2 { d.min(10) } else { d };
                }
            }
            k
        };
        let stats = SiteStats {
            val: mul(k, v),
            dist: mul(mul(k, k), 2),
            wx: sx,
            wy: sy,
        };
        self.site_stats_tail(who, stats, sx, sy, cities, ally_land)
    }

    /// `LeaderData::get_target` — the seam; team style 2 is not modelled.
    fn leader_target(&self, _who: Player) -> Option<Player> {
        None
    }

    /// Steps 13 and 14: enemy proximity, then the ally-land tenth.
    fn site_stats_tail(
        &self,
        who: Player,
        mut stats: SiteStats,
        sx: i32,
        sy: i32,
        cities: i32,
        ally_land: bool,
    ) -> SiteStats {
        let map = self.lobby.map_style;
        if map != 0xc && map != 0x11 && (cities == 1 || self.lobby.team_style == 2) {
            let mut sum = 0i32;
            let nations = self.players.len() as i32;
            for i in 0..self.players.len() {
                let l = i as Player;
                if self.defeated[i] || l == who || self.is_ally(who, l) {
                    continue;
                }
                if self.lobby.team_style == 2 && cities >= 2 && self.leader_target(who) != Some(l) {
                    continue;
                }
                for c in self.cities.iter().filter(|c| c.owner == l) {
                    // `city_flags & 0x11 == 0x11`: alive **and capital**,
                    // not "every active city" as `docs/AI.md` §2.13 has it.
                    if !(c.alive && c.capital) {
                        continue;
                    }
                    let cp = c.pos.cell();
                    sum = sum.wrapping_add(vector_dist(sx - cp.x, sy - cp.y) / nations.max(1));
                }
            }
            if self.lobby.team_style == 2 {
                sum = mul(sum, 100);
            }
            if sum != 0 {
                stats.val /= sum;
            }
        }
        if ally_land {
            stats.val /= 10;
        }
        stats
    }

    // ------------------------------------------------------------------
    // compute_sites — §2.7
    // ------------------------------------------------------------------

    /// `compute_sites(force)`: re-score the kept sites, sample every region
    /// where the leader has peasants, keep the best ten, rank them. Draws
    /// from the sync stream per large region (§2.7). `force` is the
    /// argument `place_city_with_cost` passes (1).
    pub fn compute_sites(&mut self, who: Player, force: bool) {
        let w = who as usize;

        if self.frame == 0 {
            self.ai[w].sites = [Site::default(); SITES];
        } else {
            for i in 0..SITES {
                let s = self.ai[w].sites[i];
                if s.rank < 6 || s.val < 1 {
                    self.ai[w].sites[i] = Site::default();
                    continue;
                }
                let reg = s.reg.clamp(0, i32::from(u16::MAX)) as u16;
                let centre = Pos::new(
                    s.wx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                    s.wy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                );
                let (city, unit) = self.site_reference(who, reg, centre);
                let st = self.compute_site_stats(who, s.wx, s.wy, city, unit, reg, false);
                let e = &mut self.ai[w].sites[i];
                e.val = st.val;
                e.dist = st.dist;
                e.wx = st.wx;
                e.wy = st.wy;
            }
        }

        // `param_1 != 0 || (leader_flags & 0xc) != 4` — forced, or not a
        // human leader.
        if !(force || !self.nation[w].human) {
            return;
        }

        for r in 0..self.world.region_count() {
            let reg = r as u16;
            if Census::reg(&self.ai[w].census.reg_peasants, reg) == 0 {
                continue;
            }
            let cells: Vec<Cell> = self.world.cells_in(reg).collect();
            let Some(first) = cells.first().copied() else {
                continue;
            };
            let at = Pos::new(
                first.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                first.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            );
            let (city, unit) = self.site_reference(who, reg, at);

            let size = cells.len() as i32;
            let mut stride = 2i32;
            let mut start = 0i32;
            if size >= 200 {
                // Two draws, each skipped when its modulus is degenerate.
                let world_w = self.world.width();
                let r0 = if world_w <= 1 {
                    0
                } else {
                    self.rng.roll() % world_w
                };
                let den = world_w - r0 / 2;
                stride = (size / den).max(10);
                start = (self.ai[w].site_mark % stride as u32) as i32;
                let q = stride / 4;
                let r1 = if q <= 1 { 0 } else { self.rng.roll() % q };
                self.ai[w].site_mark = self.ai[w].site_mark.wrapping_add((r1 + 3) as u32);
            }

            let mut i = start;
            while i < size {
                let c = cells[i as usize];
                let st = self.compute_site_stats(who, c.x, c.y, city, unit, reg, true);
                i += stride;

                // The insertion, transliterated: the running minimum starts
                // at `(0, −1)` and is *replaced* unless the old minimum is
                // strictly smaller **and** (it is positive, or the new
                // score does not beat this slot). An empty slot's 0 is
                // always taken; a positive slot the new site beats can
                // displace an empty one already chosen.
                let mut min = 0i32;
                let mut idx: i32 = -1;
                let mut duplicate = false;
                for j in 0..SITES {
                    let s = self.ai[w].sites[j];
                    if s.wx == st.wx && s.wy == st.wy && s.val > 0 {
                        duplicate = true;
                        break;
                    }
                    if !(min < s.val && (min > 0 || st.val <= s.val)) {
                        min = s.val;
                        idx = j as i32;
                    }
                }
                if duplicate || idx < 0 {
                    continue;
                }
                let e = &mut self.ai[w].sites[idx as usize];
                e.wx = st.wx;
                e.wy = st.wy;
                e.val = st.val;
                e.reg = r as i32;
                e.dist = st.dist;
            }
        }

        // `rank[i]` = how many of the ten score no more than site `i` —
        // itself included, so a set of one 370 and nine zeroes ranks 10 and
        // 9. (`docs/AI.md` §2.7's "1 + count" counts the other nine.)
        for i in 0..SITES {
            let v = self.ai[w].sites[i].val;
            let rank = (0..SITES).filter(|j| self.ai[w].sites[*j].val <= v).count();
            self.ai[w].sites[i].rank = rank as i32;
        }
    }

    // ------------------------------------------------------------------
    // found_cities — §2.12
    // ------------------------------------------------------------------

    /// `found_cities`: the city AI — offers the best site to the make list
    /// and, under the city limit, buys it on the spot (§2.12).
    ///
    /// The type loop runs `BASE_BUILDTYPES .. 0x19f`, i.e. exactly once, on
    /// the Small City.
    pub fn found_cities(&mut self, who: Player) {
        let w = who as usize;
        // `leader_flags2 & 0x10` — the seam, never set.
        let Some(village) = self.city_record(Ident::Village) else {
            return;
        };
        let Some(village_t) = self.build_types[village].tree else {
            return;
        };

        let mut want = false;
        if self.total_cities(who) < self.city_limit(who)
            && self.type_available(who, village_t)
            && (self.ai[w].census.free_peasants != 0 || self.ai[w].census.gatherers != 0)
            && !self.human_capped(who)
        {
            for i in 0..SITES {
                want |= self.offer_site(who, village, village_t, i);
            }
        }

        if want && self.ai[w].make_list.list[0].t != -1 && self.make_stuff(who) {
            self.ai[w].make_list.clear();
        }
    }

    /// The human cap: unless `leader_flags & 8` (the seam), an AI on
    /// difficulty 0 or 1 stops at the most cities any human holds (at least
    /// two), on 2 at one more than that; 3 and up are uncapped.
    fn human_capped(&self, who: Player) -> bool {
        let m = self.max_human_cities();
        if m <= 0 {
            return false;
        }
        let total = self.total_cities(who);
        match self.ai_difficulty() {
            0 | 1 => m.max(2) <= total,
            2 => (m + 1).max(2) <= total,
            _ => false,
        }
    }

    /// `Leaders::max_human_cities`: the most cities-plus-queued any alive
    /// human holds, less one for the Forbidden City. `−1` when there is no
    /// human.
    fn max_human_cities(&self) -> i32 {
        let mut best = -1;
        for i in 0..self.nation.len() {
            if self.defeated[i] || !self.nation[i].human {
                continue;
            }
            best = best.max(self.total_cities(i as Player));
        }
        best
    }

    /// One of the ten sites, scored into the make list. Returns whether it
    /// set `want` — i.e. whether the leader is under its own city limit.
    fn offer_site(&mut self, who: Player, village: usize, village_t: TypeId, i: usize) -> bool {
        let w = who as usize;
        let s = self.ai[w].sites[i];
        if s.val <= 0 {
            return false;
        }
        let reg = s.reg.clamp(0, i32::from(u16::MAX)) as u16;
        let c = &self.ai[w].census;
        if Census::reg(&c.reg_free_peasants, reg) == 0 && Census::reg(&c.reg_gatherers, reg) == 0 {
            return false;
        }
        // With two or three cities on a many-landmass map, expand only off
        // the home region.
        let cities = self.city_num(who);
        if !(self.world.landmasses() < 4
            || self.lobby.map_style == 0x14
            || !(2..=3).contains(&cities)
            || s.reg != self.ai[w].census.home_reg)
        {
            return false;
        }
        if !self.site_clear(who, village, s.wx, s.wy) {
            return false;
        }

        let c = &self.ai[w].census;
        let rc = Census::reg(&c.reg_cities, reg);
        let mut v = s.val;
        let cmp = if rc == 0 {
            v = mul(v, 4);
            0
        } else {
            v = mul_div(Census::reg(&c.reg_pop, reg) + 1, v, rc * 2);
            if Census::reg(&c.strategy, reg) & 1 == 0 {
                v /= 2;
            }
            rc
        };
        let queued = self.num_sites_of(who, village);
        if queued == 0 {
            let c = &self.ai[w].census;
            if Census::reg(&c.reg_pop, reg) <= Census::reg(&c.reg_peasants, reg) {
                v = mul(v, 2);
            }
            if cmp <= Census::reg(&c.reg_free_peasants, reg) {
                v = mul(v, 2);
            }
        }

        // Any live city — mine, or any leader's who holds one in this
        // region — within five cells drops the site.
        for l in 0..self.players.len() {
            let other = l as Player;
            if self.defeated[l] {
                continue;
            }
            if !(other == who || self.reg_cities(other, reg) != 0) {
                continue;
            }
            for b in &self.buildings {
                if b.owner != other
                    || !b.alive
                    || !b.ty.is_some_and(|t| build::is_city(&self.build_types, t))
                {
                    continue;
                }
                let bp = b.pos.cell();
                if vector_dist(s.wx - bp.x, s.wy - bp.y) < 5 {
                    return false;
                }
            }
        }
        if v == 0 {
            return false;
        }

        let c = &self.ai[w].census;
        let rl = Census::reg(&c.reg_land, reg);
        if rl < rc {
            v = mul(v, 2);
        }
        if rl < rc * 2 {
            v = mul_div(v, 3, 2);
        }

        let mut want = false;
        if self.total_cities(who) < self.city_limit(who) {
            want = true;
            v = mul(v, 30);
        }

        let pop_cap = self.muster[w].cap;
        if pop_cap < 200 {
            let eff = self.ai[w].effective_pop;
            if eff > pop_cap * 3 / 4 {
                // `(city_num + village_num) / 5`; `village_num` is the
                // seam (0).
                let n = cities / 5;
                v = mul_div(n + 2, v, n + 1);
            }
            if eff > pop_cap - 4 {
                v = mul_div(v, 3, 2);
            }
        }

        // A thin region I am not the weaker side in.
        if Census::reg(&self.ai[w].census.strategy, reg) & 5 == 1 {
            let n = cities;
            let p = self.ai[w].census.peasants;
            if p < n * 5 {
                if n * 3 <= p {
                    v = mul_div(v, 3, 2);
                }
            } else {
                v = mul(v, 2);
            }
            if n * 2 <= p {
                if n < 4 {
                    v = mul(v, 2);
                } else if n < 8 {
                    v = mul_div(v, 3, 2);
                }
            }
        }

        let mut vv = mul(v, 10);
        if vv < 0 {
            vv = 999_999;
        }
        if self.site_danger(who, s.wx, s.wy) > 0 {
            vv /= 3;
        }
        if self.world.owner(Cell::new(s.wx, s.wy)) != Owner::Player(who) {
            vv /= 3;
        }
        // An unaffordable site is quartered.
        let f = if self.type_affordable(who, village_t, true) > 0 {
            0x100
        } else {
            0x40
        };
        let scaled = mul(vv / 256, f) / 256;
        let val = scaled / (queued + 1);
        self.ai[w]
            .make_list
            .make_me(village_t as i32, val, 1, 9, -1, 0, 1, s.wx, s.wy);
        want
    }

    // ------------------------------------------------------------------
    // produce_city — §2.17
    // ------------------------------------------------------------------

    /// `produce_city(t, wx, wy, escrow)`: `true` when the site was placed
    /// and a citizen sent (the original's 0).
    pub fn produce_city(&mut self, who: Player, t: TypeId, wx: i32, wy: i32, escrow: i32) -> bool {
        let w = who as usize;
        let Some(rec) = self.build_record(t) else {
            return false;
        };
        let site = Cell::new(wx, wy);
        let site_reg = self.world.region_of(site);

        // The best citizen: on the map, idle, gathering or exploring, in
        // the site's region, nearest by cells — a gatherer handicapped by
        // twenty-four.
        let mut best = 99_999_999i32;
        let mut best_u: Option<usize> = None;
        // `local_10` in the original is *not* reset per candidate and is
        // read again after the loop: the census decrement below is keyed by
        // the **last examined** citizen's action, not the chosen one's.
        // Confirmed in the listing — one stack slot, `-0xc(%ebp)`, written
        // at `006cb1fb`/`006cb49b` and read at `006cb44f`.
        let mut last_kind = index::NONE;
        for u in 0..self.units.len() {
            let unit = &self.units[u];
            if unit.owner != who || !unit.alive() || !unit.on_map {
                continue;
            }
            if !self.unit_is_citizen(u) {
                continue;
            }
            last_kind = self
                .action_of(u)
                .map_or(index::NONE, |i| self.units[u].orders[i].index());
            if !matches!(last_kind, index::NONE | index::GATHER | index::EXPLORE_TO) {
                continue;
            }
            let up = self.units[u].pos.cell();
            if self.world.region_of(up) != site_reg {
                continue;
            }
            let d = vector_dist(wx - up.x, wy - up.y);
            let score = if last_kind == index::GATHER {
                d + 24
            } else {
                d
            };
            if score >= best {
                continue;
            }
            best = score;
            best_u = Some(u);
        }
        let Some(chosen) = best_u else {
            return false;
        };

        // The charge lands before the site is checked — the original's
        // order, and reachable through `place_city_with_cost`.
        let _ = escrow;
        let charges = self.building_price(who, rec);
        let available = self.holdings[w].available;
        cost::pay(&charges, &mut self.ledgers[w], &available, false);
        self.economy_changed(who);

        if !self.site_clear(who, rec, wx, wy) {
            return false;
        }
        let o = self.init_build(
            who,
            rec,
            Pos::new(wx * UNITS_PER_CELL, wy * UNITS_PER_CELL),
            false,
        );

        // A nomad sends every citizen it owns; anyone else sends the one.
        let group: Vec<usize> = if self.city_num(who) == 0 {
            (0..self.units.len())
                .filter(|u| {
                    self.units[*u].owner == who
                        && self.units[*u].alive()
                        && self.unit_is_citizen(*u)
                })
                .collect()
        } else {
            vec![chosen]
        };
        for u in group {
            self.clear_orders(u);
            self.swarm_around(u, o, Body::Build(o), true);
        }

        // The census, adjusted in place. The nearest city of mine to the
        // *chosen citizen* loses the peasant.
        let cp = self.units[chosen].pos.cell();
        let centre = Pos::new(
            cp.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            cp.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        let ureg = self.world.region_of(cp);
        let mut near: Option<(i32, usize)> = None;
        for (ci, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who || c.reg != ureg {
                continue;
            }
            let d = vector_dist(c.pos.x - centre.x, c.pos.y - centre.y);
            if near.is_none_or(|(bd, _)| d <= bd) {
                near = Some((d, ci));
            }
        }
        let reg = site_reg.unwrap_or(0);
        let gatherer = last_kind == index::GATHER;
        if self.ai[w].city_ai.len() < self.cities.len() {
            self.ai[w]
                .city_ai
                .resize(self.cities.len(), crate::ai::CityAi::default());
        }
        if let Some((_, ci)) = near {
            if gatherer {
                self.ai[w].city_ai[ci].gatherers -= 1;
            } else {
                self.ai[w].city_ai[ci].free -= 1;
            }
        }
        let census = &mut self.ai[w].census;
        let slot = reg as usize;
        if gatherer {
            census.gatherers -= 1;
            if let Some(x) = census.reg_gatherers.get_mut(slot) {
                *x -= 1;
            }
        } else {
            census.free_peasants -= 1;
            if let Some(x) = census.reg_free_peasants.get_mut(slot) {
                *x -= 1;
            }
        }
        true
    }

    /// `ScenarioFuncSet::place_city_with_cost`'s body: a forced sweep, then
    /// the city AI. `ai_host.rs`'s `Sim::place_city_ai` is the seam that
    /// should call this.
    pub fn place_city_ai_impl(&mut self, who: Player) {
        self.compute_sites(who, true);
        self.found_cities(who);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BuildType, flags};
    use crate::orders::{GatherOrder, Order, flag};
    use crate::tech::{TechTree, TypeDef, UnitTraits};
    use crate::world::World;
    use crate::{Tuning, Unit, UnitType};

    /// A citizen of `owner` at `pos`.
    fn spawn(sim: &mut Sim, owner: Player, ty: usize, pos: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = Unit::new(owner, index, pos, 100);
        u.ty = Some(ty);
        sim.add_unit(u)
    }

    /// Owns the cells around the corner of the map, so a site at cell
    /// `(1, 1)` stands in friendly territory.
    fn own_around(sim: &mut Sim, who: Player) {
        for x in 0..4 {
            for y in 0..4 {
                sim.world
                    .set_owner(Cell::new(x, y), Owner::Player(who), Owner::Player(who));
            }
        }
    }

    /// Puts a `GATHER` order on a unit, so it counts as a gatherer.
    fn set_gathering(sim: &mut Sim, u: usize) {
        sim.units[u].orders.push_back(Order {
            flags: flag::ACTION,
            body: Body::Gather(GatherOrder {
                building: 0,
                tile: None,
                wait: 0,
                goto_build: false,
                dist_mod: 0,
                been_there: false,
            }),
        });
        sim.update_action(u);
    }

    // ------------------------------------------------------------------
    // The static tables
    // ------------------------------------------------------------------

    #[test]
    fn the_25_offsets_are_the_5_by_5_each_cell_once() {
        let mut seen: Vec<(i32, i32)> = (0..25).map(|k| (MOVE_X[k], MOVE_Y[k])).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 25, "no offset repeats");
        for (x, y) in &seen {
            assert!(
                x.abs() <= 2 && y.abs() <= 2,
                "({x}, {y}) is outside the 5×5"
            );
        }
        assert_eq!((MOVE_X[0], MOVE_Y[0]), (0, 0), "entry 0 is the centre");
        // Entries 1..8 are the compass ring, in `ai_place`'s order rotated
        // to start north-west.
        assert_eq!(
            (1..9).map(|k| (MOVE_X[k], MOVE_Y[k])).collect::<Vec<_>>(),
            [
                (-1, -1),
                (0, -1),
                (1, -1),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0)
            ]
        );
    }

    #[test]
    fn the_outer_ring_is_the_forty_cells_at_chebyshev_five() {
        let mut seen: Vec<(i32, i32)> = (0..40).map(|k| (RING5_X[k], RING5_Y[k])).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 40);
        for k in 0..40 {
            assert_eq!(
                RING5_X[k].abs().max(RING5_Y[k].abs()),
                5,
                "entry {k} is off the ring"
            );
        }
        // 11 × 11 minus 9 × 9 = 40.
        assert_eq!(11 * 11 - 9 * 9, 40);
    }

    // ------------------------------------------------------------------
    // A world with the types these functions need
    // ------------------------------------------------------------------

    struct Kit {
        village: usize,
        citizen_unit: usize,
    }

    /// A `w × h`-cell land world, two players, with a tree of six goods, a
    /// citizen and the three city levels, and the matching records.
    fn kit(w: i32, h: i32, players: usize) -> (Sim, Kit) {
        let mut world = World::new(w, h);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(w - 1, h - 1));
        let mut sim = Sim::new(Tuning::RON, world, players);

        let mut tree = TechTree::new();
        for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            tree.types.push(TypeDef::good(n));
        }
        let citizen = tree.types.len();
        tree.types
            .push(TypeDef::unit("Citizen", UnitTraits::default()));
        let village_t = tree.types.len();
        tree.types.push(TypeDef::new(
            "Small City",
            Kind::Building {
                auto: false,
                wonder: false,
            },
        ));
        let town_t = tree.types.len();
        tree.types.push(TypeDef::new(
            "Large City",
            Kind::Building {
                auto: false,
                wonder: false,
            },
        ));
        tree.types[town_t].from = Some(village_t);
        tree.finalize();
        sim.set_tech_tree(tree);

        let village = sim.add_build_type(BuildType {
            ident: Ident::Village,
            tree: Some(village_t),
            x_size: 7,
            y_size: 7,
            flags: flags::parse("ean"),
            job_time: 100,
            hits: 100,
            price: cost::Price {
                kind: cost::Kind::Building,
                ..cost::Price::free().with_base(crate::economy::Resource::Timber, 10)
            },
            ..BuildType::default()
        });
        sim.add_build_type(BuildType {
            ident: Ident::Town,
            from: Some(village),
            tree: Some(town_t),
            x_size: 7,
            y_size: 7,
            flags: flags::parse("ean"),
            job_time: 100,
            hits: 100,
            price: cost::Price {
                kind: cost::Kind::Building,
                ..cost::Price::free().with_base(crate::economy::Resource::Timber, 10)
            },
            ..BuildType::default()
        });
        let citizen_unit = sim.add_unit_type(UnitType {
            tree: Some(citizen),
            ..UnitType::default()
        });
        for l in &mut sim.ledgers {
            l.bucket = [10_000; RESOURCES];
        }
        for a in &mut sim.ai {
            a.census.resize(64, players);
        }
        (
            sim,
            Kit {
                village,
                citizen_unit,
            },
        )
    }

    // ------------------------------------------------------------------
    // The ranking and the insertion
    // ------------------------------------------------------------------

    /// Run8's block: one site scores and the other nine are empty, so the
    /// scorer ranks 10 and the empties rank 9 — `rank` counts every site
    /// that scores no more, itself included.
    #[test]
    fn the_rank_counts_the_sites_that_score_no_more_including_itself() {
        let (mut sim, _) = kit(8, 8, 2);
        sim.frame = 1;
        sim.nation[1].human = false;
        // The nomad's wood test would otherwise zero every site.
        sim.lobby.starting_resources = 8;
        sim.ai[1].census.reg_peasants[0] = 1;
        // One valuable cell, on an even row-major index so the stride-2
        // sampler visits it.
        let c = Cell::new(4, 4);
        assert_eq!((4 * 8 + 4) % 2, 0);
        let mut d = sim.world.cell_data(c);
        d.val = 100;
        sim.world.set_cell_data(c, d);

        sim.compute_sites(1, true);
        let scored: Vec<&Site> = sim.ai[1].sites.iter().filter(|s| s.val > 0).collect();
        assert_eq!(scored.len(), 1, "exactly one cell scores");
        assert_eq!((scored[0].wx, scored[0].wy), (4, 4));
        assert_eq!(
            scored[0].dist, 9,
            "no reference object: dist is the constant"
        );
        assert_eq!(scored[0].rank, 10, "the best of ten ranks ten");
        for s in &sim.ai[1].sites {
            if s.val == 0 {
                assert_eq!(s.rank, 9, "a zero site ranks nine");
            }
        }
    }

    /// The running-minimum quirk, transliterated from `006cce8c`: with an
    /// empty slot 0 and a scored slot 1 the new site beats, the *scored*
    /// slot is the one displaced.
    #[test]
    fn the_insertion_takes_a_beaten_positive_slot_over_an_earlier_empty_one() {
        let pick = |sites: [i32; SITES], val: i32| -> i32 {
            let mut min = 0i32;
            let mut idx = -1i32;
            for (j, s) in sites.iter().enumerate() {
                if !(min < *s && (min > 0 || val <= *s)) {
                    min = *s;
                    idx = j as i32;
                }
            }
            idx
        };
        // All empty: the last slot wins (every 0 is "taken").
        assert_eq!(pick([0; SITES], 50), 9);
        // Slot 1 scores 5 and we score 100: slot 1 is taken, then slots
        // 2..9 are empty and the last empty wins.
        let mut s = [0; SITES];
        s[1] = 5;
        assert_eq!(pick(s, 100), 9);
        // With every slot filled, the smallest wins, ties to the later.
        let full = [90, 30, 70, 30, 80, 60, 50, 95, 99, 40];
        assert_eq!(pick(full, 100), 3, "the later of the two 30s");
        // A site that beats no slot and finds no empty one is dropped
        // outright: the running minimum never leaves `(0, -1)`.
        assert_eq!(pick(full, 1), -1);
    }

    // ------------------------------------------------------------------
    // The sampler
    // ------------------------------------------------------------------

    /// A region of 200+ cells takes two draws and samples with the strided
    /// offset; a small one takes none and strides by two.
    #[test]
    fn a_large_region_strides_by_the_two_draws_and_a_small_one_by_two() {
        // 20 × 20 = 400 cells, one region.
        let (mut sim, _) = kit(20, 20, 2);
        sim.frame = 1;
        sim.nation[1].human = false;
        sim.ai[1].census.reg_peasants[0] = 1;
        sim.ai[1].site_mark = 16;
        let before = sim.rng.seed;
        sim.compute_sites(1, true);
        assert_ne!(before, sim.rng.seed, "a 400-cell region draws");

        // Reproduce the arithmetic the sampler ran.
        let mut rng = crate::combat::Rng::new(before);
        let world_w = 20;
        let r0 = rng.roll() % world_w;
        let stride = (400 / (world_w - r0 / 2)).max(10);
        let start = 16 % stride;
        let q = stride / 4;
        let r1 = if q <= 1 { 0 } else { rng.roll() % q };
        assert_eq!(rng.seed, sim.rng.seed, "exactly two draws, in that order");
        assert_eq!(
            sim.ai[1].site_mark,
            16 + (r1 + 3) as u32,
            "site_mark advances by the draw plus three"
        );
        assert!(stride >= 10 && start < stride);

        // A small region draws nothing.
        let (mut small, _) = kit(8, 8, 2);
        small.frame = 1;
        small.nation[1].human = false;
        small.ai[1].census.reg_peasants[0] = 1;
        small.ai[1].site_mark = 16;
        let seed = small.rng.seed;
        small.compute_sites(1, true);
        assert_eq!(seed, small.rng.seed, "64 cells is under the 200 threshold");
        assert_eq!(small.ai[1].site_mark, 16, "and site_mark does not move");
    }

    /// A human leader takes neither the draws nor the ranking unless the
    /// call is forced.
    #[test]
    fn a_human_leader_is_only_sampled_when_forced() {
        let (mut sim, _) = kit(20, 20, 2);
        sim.frame = 1;
        sim.nation[1].human = true;
        sim.ai[1].census.reg_peasants[0] = 1;
        let seed = sim.rng.seed;
        sim.compute_sites(1, false);
        assert_eq!(seed, sim.rng.seed, "no sweep for a human");
        sim.compute_sites(1, true);
        assert_ne!(seed, sim.rng.seed, "forced, it sweeps");
    }

    /// Frame 0 zeroes all ten before sampling.
    #[test]
    fn frame_zero_clears_every_site_first() {
        let (mut sim, _) = kit(8, 8, 2);
        sim.frame = 0;
        sim.nation[1].human = false;
        sim.ai[1].sites[3] = Site {
            wx: 4,
            wy: 4,
            val: 900,
            reg: 0,
            dist: 8,
            rank: 9,
        };
        sim.compute_sites(1, true);
        assert_eq!(sim.ai[1].sites[3].val, 0);
        assert_eq!(sim.ai[1].sites[3].wx, 0);
    }

    /// Off frame 0, a site is kept and re-scored only with `rank ≥ 6` and
    /// `val ≥ 1`.
    #[test]
    fn a_kept_site_needs_rank_six_and_a_positive_score() {
        let (mut sim, _) = kit(8, 8, 2);
        sim.frame = 1;
        sim.nation[1].human = true;
        sim.ai[1].sites[0] = Site {
            wx: 4,
            wy: 4,
            val: 900,
            reg: 0,
            dist: 8,
            rank: 5,
        };
        sim.ai[1].sites[1] = Site {
            wx: 5,
            wy: 5,
            val: 0,
            reg: 0,
            dist: 8,
            rank: 9,
        };
        sim.compute_sites(1, false);
        assert_eq!(sim.ai[1].sites[0], Site::default(), "rank 5 is cleared");
        assert_eq!(sim.ai[1].sites[1], Site::default(), "val 0 is cleared");
    }

    // ------------------------------------------------------------------
    // compute_site_stats
    // ------------------------------------------------------------------

    /// §13's settled item: a city of mine outside the site's region ends
    /// the score at zero, and so does a dead slot — the loop `return`s.
    #[test]
    fn a_city_in_another_region_scores_the_site_zero() {
        let (mut sim, kit) = kit(16, 16, 2);
        // Two regions: the left half is region 0, the right half region 1.
        let right = sim
            .world
            .fill_region(Terrain::Land, Cell::new(8, 0), Cell::new(15, 15));
        for c in [Cell::new(2, 2), Cell::new(3, 3)] {
            sim.world.set_owner(c, Owner::Player(1), Owner::Player(1));
        }
        let mut d = sim.world.cell_data(Cell::new(2, 2));
        d.val = 200;
        sim.world.set_cell_data(Cell::new(2, 2), d);

        // One city of mine, in the right-hand region.
        let b = sim.init_build(
            1,
            kit.village,
            Pos::new(10 * UNITS_PER_CELL, 2 * UNITS_PER_CELL),
            false,
        );
        sim.activate(b, false, true);
        assert_eq!(sim.world.region_of(Cell::new(10, 2)), Some(right));
        assert!(sim.city_num(1) > 0, "the city stands");

        let st = sim.compute_site_stats(1, 2, 2, true, true, 0, false);
        assert_eq!(st.val, 0, "the city is in another region: the loop returns");
        assert_eq!(st.dist, 0);
    }

    /// With no reference object at all the distance term is the constant
    /// `k = 3`, `dist = 9` — the loop is skipped.
    #[test]
    fn no_reference_object_gives_k_three_and_dist_nine() {
        let (mut sim, _) = kit(16, 16, 2);
        // Skip the nomad's wood test (`city_num == 0` here).
        sim.lobby.starting_resources = 8;
        let c = Cell::new(8, 8);
        sim.world.set_owner(c, Owner::Player(1), Owner::Player(1));
        let mut d = sim.world.cell_data(c);
        d.val = 40;
        sim.world.set_cell_data(c, d);
        let st = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
        assert_eq!(st.dist, 9, "dist is the constant, not 2k²");
        assert!(st.val > 0);
    }

    /// The base is zero — and so is the score — where the **Town's**
    /// footprint is blocked, even though a Village would fit.
    #[test]
    fn a_zero_value_cell_scores_nothing() {
        let (sim, _) = kit(16, 16, 2);
        // Every cell's `val` byte is 0 on a flat world.
        let st = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
        assert_eq!(
            st,
            SiteStats {
                val: 0,
                dist: 0,
                wx: 8,
                wy: 8
            }
        );
    }

    /// `keep` slides the site onto the best-valued open cell of its 5×5;
    /// without it the site stays put.
    #[test]
    fn keep_slides_the_site_to_the_best_cell_of_its_five_by_five() {
        let (mut sim, _) = kit(16, 16, 2);
        sim.lobby.starting_resources = 8;
        for x in 6..12 {
            for y in 6..12 {
                sim.world
                    .set_owner(Cell::new(x, y), Owner::Player(1), Owner::Player(1));
            }
        }
        let mut d = sim.world.cell_data(Cell::new(8, 8));
        d.val = 10;
        sim.world.set_cell_data(Cell::new(8, 8), d);
        // A four-cell block of 200s two cells away: its 2×2 mean is 200.
        for c in [
            Cell::new(10, 9),
            Cell::new(11, 9),
            Cell::new(10, 10),
            Cell::new(11, 10),
        ] {
            let mut d = sim.world.cell_data(c);
            d.val = 200;
            sim.world.set_cell_data(c, d);
        }
        let stay = sim.compute_site_stats(1, 8, 8, false, false, 0, false);
        assert_eq!((stay.wx, stay.wy), (8, 8), "keep = 0 never moves");
        let slid = sim.compute_site_stats(1, 8, 8, false, false, 0, true);
        assert_eq!((slid.wx, slid.wy), (10, 9), "keep = 1 slides to the 200s");
        assert!(slid.val > stay.val);
    }

    // ------------------------------------------------------------------
    // found_cities
    // ------------------------------------------------------------------

    /// A leader with a peasant in the region, under its city limit, on an
    /// empty region — the multipliers, on a hand-built census.
    fn city_ai_fixture() -> (Sim, usize, TypeId) {
        let (mut sim, k) = kit(16, 16, 2);
        sim.frame = 1;
        sim.nation[0].human = false;
        sim.nation[1].human = false;
        sim.ai[1].census.free_peasants = 1;
        sim.ai[1].census.reg_free_peasants[0] = 1;
        sim.ai[1].census.home_reg = 0;
        // Own the cell (else the score is divided by three) and lift the
        // population cap out of the way.
        sim.world
            .set_owner(Cell::new(8, 8), Owner::Player(1), Owner::Player(1));
        sim.muster[1].cap = 200;
        sim.ai[1].sites[0] = Site {
            wx: 8,
            wy: 8,
            val: 1_000,
            reg: 0,
            dist: 8,
            rank: 9,
        };
        let village_t = sim.build_types[k.village].tree.unwrap();
        (sim, k.village, village_t)
    }

    #[test]
    fn the_city_ai_scores_a_site_by_its_census() {
        let (mut sim, village, village_t) = city_ai_fixture();
        assert!(
            sim.offer_site(1, village, village_t, 0),
            "under the city limit, so `want` is set"
        );
        let head = sim.ai[1].make_list.list[0];
        assert_eq!(head.t, village_t as i32);
        assert_eq!((head.wx, head.wy), (8, 8));
        assert_eq!(head.cat, 9, "a city site is category 9");
        assert_eq!(head.escrow, 1);
        assert_eq!(head.num, 1);
        // x4 (no city in the region), x2 (reg_pop <= reg_peasants),
        // x2 (reg_cities <= reg_free_peasants), x30 (under the city limit),
        // then x10, then `/256 * 0x100 / 256` and `/(queued + 1)`.
        let v = 1_000 * 4 * 2 * 2 * 30;
        assert_eq!(v, 480_000);
        assert_eq!(
            head.val,
            (v * 10 / 256) * 0x100 / 256,
            "the value as scored"
        );
        assert_eq!(head.val, 18_750);
    }

    /// Not owning the cell costs a third, and an unaffordable site is
    /// quartered — the last two terms before `make_me`.
    #[test]
    fn an_unowned_or_unaffordable_site_is_thirded_and_quartered() {
        let (mut sim, village, village_t) = city_ai_fixture();
        sim.world
            .set_owner(Cell::new(8, 8), Owner::None, Owner::None);
        sim.offer_site(1, village, village_t, 0);
        assert_eq!(sim.ai[1].make_list.list[0].val, 18_750 / 3);

        let (mut sim, village, village_t) = city_ai_fixture();
        for l in &mut sim.ledgers {
            l.bucket = [0; RESOURCES];
        }
        assert_eq!(sim.type_affordable(1, village_t, true), 0);
        sim.offer_site(1, village, village_t, 0);
        assert_eq!(sim.ai[1].make_list.list[0].val, 18_750 / 4);
    }

    /// A live city within five cells drops the site outright.
    #[test]
    fn a_city_within_five_cells_drops_the_site() {
        let (mut sim, village, village_t) = city_ai_fixture();
        let b = sim.init_build(
            1,
            village,
            Pos::new(11 * UNITS_PER_CELL, 8 * UNITS_PER_CELL),
            false,
        );
        sim.activate(b, false, true);
        assert!(vector_dist(8 - 11, 0) < 5);
        assert!(!sim.offer_site(1, village, village_t, 0));
        assert_eq!(sim.ai[1].make_list.list[0].t, -1, "nothing was offered");
    }

    /// The human cap: on difficulty 0 the AI stops at `max(the best
    /// human's total, 2)`, and `found_cities` then offers nothing at all —
    /// the sentinel in the make list survives untouched.
    #[test]
    fn the_human_cap_stops_the_ai_at_the_best_human_s_count() {
        let (mut sim, k) = kit(16, 16, 2);
        sim.frame = 1;
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        sim.lobby.difficulty = 0;
        sim.ai[1].census.free_peasants = 1;
        sim.ai[1].census.reg_free_peasants[0] = 1;
        sim.ai[1].sites[0] = Site {
            wx: 3,
            wy: 3,
            val: 1_000,
            reg: 0,
            dist: 8,
            rank: 9,
        };
        // The human holds one city; the cap is max(1, 2) = 2.
        let h = sim.init_build(
            0,
            k.village,
            Pos::new(UNITS_PER_CELL, UNITS_PER_CELL),
            false,
        );
        sim.activate(h, false, true);
        assert_eq!(sim.max_human_cities(), 1);
        assert!(!sim.human_capped(1), "with no cities the AI is not capped");

        // Give the AI two: now it is.
        for (x, y) in [(12, 12), (12, 4)] {
            let b = sim.init_build(
                1,
                k.village,
                Pos::new(x * UNITS_PER_CELL, y * UNITS_PER_CELL),
                false,
            );
            sim.activate(b, false, true);
        }
        assert_eq!(sim.total_cities(1), 2);
        assert!(sim.human_capped(1));

        sim.ai[1].make_list.list[0].t = 4242;
        sim.found_cities(1);
        assert_eq!(
            sim.ai[1].make_list.list[0].t, 4242,
            "the capped leader offered nothing and bought nothing"
        );
    }
    // produce_city
    // ------------------------------------------------------------------

    /// The nearest citizen is sent — and a gatherer is handicapped
    /// twenty-four cells, so a gatherer at ten loses to an idler at thirty.
    #[test]
    fn the_nearest_citizen_is_sent_with_the_gatherer_s_penalty() {
        let (mut sim, k) = kit(64, 8, 2);
        sim.frame = 1;
        let village_t = sim.build_types[k.village].tree.unwrap();
        // A city of mine, so this is not the nomad path (which sends every
        // citizen at once) — far enough away not to crowd the site, with
        // the site's own cells owned so the placement is in my territory.
        let home = sim.init_build(
            1,
            k.village,
            Pos::new(50 * UNITS_PER_CELL, 4 * UNITS_PER_CELL),
            false,
        );
        sim.activate(home, false, true);
        assert_eq!(sim.city_num(1), 1);
        own_around(&mut sim, 1);

        // A gatherer ten cells away and an idler thirty away.
        let gatherer = spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(11 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        let idler = spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(31 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        set_gathering(&mut sim, gatherer);
        assert_eq!(sim.order_type(gatherer), index::GATHER);

        assert_eq!(
            sim.blocked_site(
                Some(1),
                k.village,
                Pos::new(UNITS_PER_CELL, UNITS_PER_CELL),
                None
            ),
            Blocked::Clear
        );
        assert!(sim.produce_city(1, village_t, 1, 1, 0));
        // 10 + 24 = 34 > 30: the idler goes.
        assert!(
            sim.units[idler]
                .orders
                .iter()
                .any(|o| matches!(o.body, Body::Build(_))),
            "the idler was ordered onto the site"
        );
        assert!(
            !sim.units[gatherer]
                .orders
                .iter()
                .any(|o| matches!(o.body, Body::Build(_))),
            "the gatherer's twenty-four kept it out"
        );
        assert!(
            sim.buildings.iter().any(|b| b.owner == 1 && !b.active),
            "a site was placed"
        );
    }

    /// A nomad (`city_num == 0`) sends every citizen it owns, not one.
    #[test]
    fn a_nomad_sends_every_citizen() {
        let (mut sim, k) = kit(64, 8, 2);
        sim.frame = 1;
        let village_t = sim.build_types[k.village].tree.unwrap();
        let a = spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(11 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        let b = spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(31 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        assert_eq!(sim.city_num(1), 0);
        assert!(sim.produce_city(1, village_t, 1, 1, 0));
        for u in [a, b] {
            assert!(
                sim.units[u]
                    .orders
                    .iter()
                    .any(|o| matches!(o.body, Body::Build(_))),
                "citizen {u} was sent"
            );
        }
    }

    /// A citizen in another region is not a candidate, and with no
    /// candidate at all the producer refuses without placing or paying.
    #[test]
    fn no_citizen_in_the_region_means_no_city() {
        let (mut sim, k) = kit(16, 16, 2);
        sim.frame = 1;
        let village_t = sim.build_types[k.village].tree.unwrap();
        sim.world
            .fill_region(Terrain::Land, Cell::new(8, 0), Cell::new(15, 15));
        spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(12 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        let before = sim.ledgers[1].bucket;
        assert!(!sim.produce_city(1, village_t, 2, 2, 0));
        assert_eq!(sim.ledgers[1].bucket, before, "nothing was charged");
        assert!(!sim.buildings.iter().any(|b| b.owner == 1));
    }

    /// The census decrement follows the *last examined* citizen's action,
    /// not the chosen one's — the original reuses one stack slot
    /// (`-0xc(%ebp)`), and the listing confirms it.
    #[test]
    fn the_census_decrement_uses_the_last_examined_citizen_s_action() {
        let (mut sim, k) = kit(64, 8, 2);
        sim.frame = 1;
        let village_t = sim.build_types[k.village].tree.unwrap();
        let home = sim.init_build(
            1,
            k.village,
            Pos::new(50 * UNITS_PER_CELL, 4 * UNITS_PER_CELL),
            false,
        );
        sim.activate(home, false, true);
        own_around(&mut sim, 1);
        sim.ai[1].census.free_peasants = 5;
        sim.ai[1].census.gatherers = 5;
        sim.ai[1].census.reg_free_peasants[0] = 5;
        sim.ai[1].census.reg_gatherers[0] = 5;

        // The idler comes first in the list and wins; the gatherer is
        // examined after and leaves its action behind.
        spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(2 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        let gatherer = spawn(
            &mut sim,
            1,
            k.citizen_unit,
            Pos::new(40 * UNITS_PER_CELL, UNITS_PER_CELL),
        );
        set_gathering(&mut sim, gatherer);

        assert!(sim.produce_city(1, village_t, 1, 1, 0));
        assert_eq!(sim.ai[1].census.gatherers, 4, "the gatherer's count fell");
        assert_eq!(
            sim.ai[1].census.free_peasants, 5,
            "though the idler is the one that went"
        );
    }
}
