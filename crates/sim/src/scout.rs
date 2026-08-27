//! The idle scout's explore target — `Unit::think_scout@005f6010`.
//!
//! `docs/SCOUT.md` is the mechanic; this is its city branch, the only one
//! any capture on disk reaches. An AI scout with nothing to do walks rings
//! of cells outward from each city it knows about, scores every cell that
//! is inside its own region and **not really seen**, and sends itself to
//! the cheapest one as a one-member group with an `EXPLORE_TO` move.
//!
//! It is three draws a ring at most, and it is the last unattributed block
//! of frame 0 on run20 and on the fuzzer's control map (`docs/SYNC.md`
//! §4.2): ten draws at `+0x436` ×4, `+0x458` ×2, `+0x64c` ×4 on both. The
//! counts fall out of the ring walk's two guards and the cell filter, and
//! `docs/SCOUT.md` §10 is the sequence, ring by ring.

use crate::ai_load::role;
use crate::attrition::Domain;
use crate::group::Group;
use crate::movement::Angle;
use crate::orders::{MoveKind, QueuePos};
use crate::world::{Cell, Pos, UNITS_PER_TILE, tile, vector_dist};
use crate::{Player, Sim};

/// `TypeIndex::PEASANTS` and `PEASANTSKOREAN` — `UnitTypeData +0x4`, the two
/// citizen ids `think_scout` sends down the region scan (§3).
pub const PEASANTS: i32 = 0x32;
pub const PEASANTSKOREAN: i32 = 0x33;

/// The cell budget one call spends before it abandons the city loop —
/// `local_58 > 0x600` (§7).
pub const CELL_BUDGET: i32 = 0x600;

/// The score the city loop's winner must come in under for the region scan
/// not to run as well — the original's test is `199 < best` (§3).
pub const REGION_SCAN_ABOVE: i32 = 199;

/// `best` before anything is found — the original's `99999999`, tested at
/// the tail as `0x5f5e0fe < best`.
pub const NOTHING: i32 = 99_999_999;

/// The deepest ring the first city examined is walked to; every city after
/// it gets 8 (§5).
pub const FIRST_MAX_RING: i32 = 12;

/// The three draw sites, under the original's own offsets from
/// `think_scout@005f6010`. `Sim::mark` writes them into
/// [`Sim::phase_marks`](crate::Sim::phase_marks) while the harness is
/// tracing, which is what lets `rondata::diff` compare our draw *sequence*
/// against `rondata::trace`'s rather than our count against a total
/// (`docs/SCOUT.md` §10, §12).
pub const SITE_ROTATION: &str = "Unit::think_scout+0x436";
pub const SITE_PHASE: &str = "Unit::think_scout+0x458";
pub const SITE_CELL: &str = "Unit::think_scout+0x64c";

/// The address range those three fall in — `Unit::think_scout` up to
/// `Unit::think`, the next function in the export. What
/// `rondata::trace::Trace::run_in` takes to isolate this mechanic's own
/// draws from the ones its callees take.
pub const CODE: std::ops::Range<u32> = 0x005f_6010..0x005f_6e40;

/// The `max_ring` an AI unit uses around a **foreign** leader's city (§6).
pub const FOREIGN_MAX_RING: i32 = 3;

// ----------------------------------------------------------------------
// §4 — the circle tables, `circle_init@006817f0`
// ----------------------------------------------------------------------

/// `circle_x`, `circle_y` and `circle_radius` (`00cb7e90`, `00cbb0e0`,
/// `00cbe330`), re-derived.
///
/// `circle_init` sweeps `r = 0..=0x40`; for each `r` it scans the square
/// `[-r, r]²` in x-major order, appends every offset whose octagonal
/// distance is exactly `r`, and records the running total. So **ring `r` is
/// the index range `[radius[r - 1], radius[r])`** — which is how the
/// function reads it, through a base four bytes below `circle_radius`.
///
/// The tables live in `.bss` and are filled at startup, so nothing can be
/// read out of the executable: this is the loader, not a transcription.
/// Ring 1 coming out as the eight neighbours is the check that the sweep
/// order is right.
pub mod circle {
    use std::sync::OnceLock;

    /// `circle_points`' cap: the sweep stops once it has appended one past
    /// this.
    pub const MAX_POINTS: usize = 0x3248;
    /// `circle_radius`' length — the sweep's last ring is `0x40`.
    pub const RINGS: usize = 0x41;

    /// The three tables together.
    pub struct Table {
        pub x: Vec<i8>,
        pub y: Vec<i8>,
        /// `circle_radius[r]`: points at octagonal distance `<= r`.
        pub radius: [i32; RINGS],
    }

    impl Table {
        /// The half-open index range of ring `r`, for `1 <= r < RINGS`.
        pub fn ring(&self, r: i32) -> (i32, i32) {
            (self.radius[(r - 1) as usize], self.radius[r as usize])
        }
    }

    /// The distance `circle_init` compares against the ring number: the
    /// engine's octagonal `vector_dist` shape, inlined there.
    const fn approx(a: i32, b: i32) -> i32 {
        let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
        if hi == 0 {
            return 0;
        }
        if lo < 60_000 {
            (lo * lo) / (hi * 2) + hi
        } else {
            (lo + hi * 2) >> 1
        }
    }

    fn build() -> Table {
        let mut t = Table {
            x: Vec::new(),
            y: Vec::new(),
            radius: [0; RINGS],
        };
        // `local_14` walks down from 0 as the ring number walks up, so the
        // square scanned for ring `r` is `[-r, r]` on both axes.
        let mut lo = 0i32;
        for r in 0..RINGS as i32 {
            for x in lo..=r {
                for y in lo..=r {
                    if approx(x.abs(), y.abs()) != r {
                        continue;
                    }
                    t.x.push(x as i8);
                    t.y.push(y as i8);
                    if t.x.len() > MAX_POINTS {
                        // The overflow exit fills every remaining ring with
                        // the count reached and returns.
                        let n = t.x.len() as i32;
                        for slot in t.radius.iter_mut().skip(r as usize) {
                            *slot = n;
                        }
                        return t;
                    }
                }
            }
            t.radius[r as usize] = t.x.len() as i32;
            lo -= 1;
        }
        t
    }

    /// The tables, built on first use.
    pub fn table() -> &'static Table {
        static TABLE: OnceLock<Table> = OnceLock::new();
        TABLE.get_or_init(build)
    }
}

/// The call's running state — the original's locals, under its own shape.
struct Scan {
    /// `local_28`: the best score so far, over the whole call.
    score: i32,
    /// `local_18`/`local_24`: the winning **tile**, `4 * cell + 2` both ways.
    tile: Pos,
    /// `local_10`: the lowest ring a candidate has come from, reset per
    /// city, and what stops the walk one ring past the winner.
    ring: i32,
    /// `local_58`: cells examined, over the whole call.
    budget: i32,
    /// Whether [`CELL_BUDGET`] blew, which abandons every loop at once.
    over: bool,
    /// `local_c`: the leader whose city the current ring belongs to.
    leader: Player,
}

impl Sim {
    // ------------------------------------------------------------------
    // §2 — the gate
    // ------------------------------------------------------------------

    /// `Unit::think`'s tail (§2): a scout or a spy, not in an army, whose
    /// leader is not a plain human.
    ///
    /// The human arm of `Unit::think` returns before the tail because
    /// `unit_masks & 0x40000` is clear on its units — `Unit::init` sets that
    /// bit exactly when `(leader_flags & 0xc) != 4` — so reaching here is
    /// the whole of "an AI unit"; [`Sim::ai_driven`] is the seam that
    /// answers it.
    pub fn scout_thinks(&self, u: usize) -> bool {
        let unit = &self.units[u];
        let Some(rec) = unit.ty else { return false };
        if self.unit_types[rec].kind.supply_unit {
            return false;
        }
        if !self.unit_is_scout(u) && !self.unit_is_spy(u) {
            return false;
        }
        self.ai_driven(unit.owner) && self.army_of(u).is_none()
    }

    /// `role & 0x10` on the unit's type — the scout bit
    /// ([`crate::ai_load::role::SCOUT`]).
    pub fn unit_is_scout(&self, u: usize) -> bool {
        let Some(rec) = self.units[u].ty else {
            return false;
        };
        let cols = self.unit_types[rec].cols.role;
        if cols != 0 {
            return cols & role::SCOUT != 0;
        }
        // A type the loader never filled — a harness fixture. The tree id is
        // what `role_word` needs for its `Scout` lineage test.
        self.unit_types[rec]
            .tree
            .is_some_and(|t| self.role_word(t, rec) & role::SCOUT != 0)
    }

    /// **Seam** — `ObjectData::is(SPY, 0)`, the second half of §2's gate and
    /// one of the three tests that route a unit to the region scan (§3). No
    /// spy stands in any capture on disk and the lineage test is unmodelled.
    pub(crate) fn unit_is_spy(&self, _u: usize) -> bool {
        false
    }

    /// `unit_masks & 0x40000`, set at `Unit::init@00612100:586` for every
    /// unit whose leader is not a plain human — `(leader_flags & 0xc) != 4`.
    /// `Nation::human` is `leader_flags & 4` and is what
    /// [`Sim::think_join_army`](crate::Sim::think_join_army) already reads;
    /// the AI-driven-human bit `0x8` is the seam left in it.
    pub(crate) fn ai_driven(&self, who: Player) -> bool {
        self.nation.get(who as usize).is_some_and(|n| !n.human)
    }

    // ------------------------------------------------------------------
    // §7 — the fog read that is this mechanic's own
    // ------------------------------------------------------------------

    /// `WorldData::was_really_seen@006b54f0` — the **bare** fog read, with
    /// none of `was_seen`'s ally-territory shortcut
    /// (`crate::ai_sites`' `site_was_seen` is that one, and they are
    /// different functions with one letter between their names). Answers
    /// true when no fog grid is loaded, which is the flat harness world.
    ///
    /// The two leader exits (`leader_flags & 0x800`, `LeaderData +0x59e4`)
    /// are seams: no run sets either.
    pub(crate) fn was_really_seen(&self, c: Cell, who: Player) -> bool {
        self.was_really_seen_fog(2 * c.x + 1, 2 * c.y + 1, who)
    }

    /// The same read at the fog grid's **own** coordinates — the half-cell
    /// pair `was_really_seen` actually takes.
    ///
    /// Its two callers hand it different things and the difference shows.
    /// [`Sim::was_really_seen`] samples a whole cell at `2c + 1`, the
    /// second half-cell each way, because that is what `Unit::think_scout`
    /// and the site census do. `PathFinder::calc_cost` instead asks about
    /// the **point** it is stepping to, `div_3_table[to >> 7]` — the
    /// half-cell that contains it, `to / 0x180` — so a step landing in a
    /// cell's first half reads `2c`, not `2c + 1`
    /// (`docs/PATHFINDER.md` §5).
    pub(crate) fn was_really_seen_fog(&self, fx: i32, fy: i32, who: Player) -> bool {
        if who >= 8 || self.lobby.reveal_map == 3 {
            return true;
        }
        let Some(bits) = self.world.seen2(fx, fy) else {
            return true;
        };
        let mask = self.ai.get(who as usize).map_or(0, |a| a.census.ally_mask) | (1 << who);
        u32::from(bits) & mask != 0
    }

    // ------------------------------------------------------------------
    // §3 — `Unit::think_scout(this, 0)`
    // ------------------------------------------------------------------

    /// The whole of it, for the one call site `Unit::think`'s tail uses.
    /// Returns whether an explore order was issued.
    pub fn think_scout(&mut self, u: usize) -> bool {
        let who = self.units[u].owner;
        let Some(rec) = self.units[u].ty else {
            return false;
        };
        let domain = self.unit_types[rec].kind.domain;
        let type_index = self.units[u].type_index;
        let Some(region) = self.world.tregion(self.units[u].pos.tile()) else {
            return false;
        };

        // The head. `find_goody_box` and `think_spellcaster` are seams
        // (`docs/SCOUT.md` §13 items 2 and 8): neither fires in any capture,
        // and neither draws when it does not.

        // §3's branch. `unit_masks & 0x100` is a seam (clear on every scout
        // in every run on disk); the two citizen ids, the spy and the naval
        // domain are what route a unit away from the city loop.
        let city_branch = type_index != PEASANTS
            && type_index != PEASANTSKOREAN
            && !self.unit_is_spy(u)
            && domain != Domain::Sea;
        if !city_branch {
            self.scout_region_scan(region);
            return false;
        }

        let scan = self.scout_city_loop(u, who, region);

        // §3's tail. Above 199 the region scan runs as well and can win it;
        // the simulation takes that scan's first draw and no more (§11).
        if scan.score > REGION_SCAN_ABOVE {
            self.scout_region_scan(region);
            if scan.score >= NOTHING {
                return false;
            }
        }
        self.scout_issue(u, scan.tile);
        true
    }

    /// §5 — the leader loop and the city loop, with §6's ring walk inside.
    fn scout_city_loop(&mut self, u: usize, who: Player, region: u16) -> Scan {
        let mut scan = Scan {
            score: NOTHING,
            tile: Pos::default(),
            ring: 0,
            budget: 0,
            over: false,
            leader: who,
        };
        if self.city_num(who) == 0 {
            return scan;
        }
        // `max_ring` starts at 12 and drops to 8 after the **first city
        // examined**, whichever leader's it was. It is not reset per city.
        let mut max_ring = FIRST_MAX_RING;
        // `find_city(unit.pos, SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)`:
        // the rotation offset into the scout's own leader's city list.
        let near = self.scout_find_city(u, who, region);
        let land = self.units[u]
            .ty
            .is_some_and(|r| self.unit_types[r].kind.domain == Domain::Land);

        for i in 0..8u8 {
            let leader = (who + i) & 7;
            if !self.scout_leader_in_play(leader) {
                continue;
            }
            if self.reg_cities(leader, region) == 0 {
                continue;
            }
            if leader != who && self.is_ally(leader, who) {
                continue;
            }
            let list = self.cities_of(leader);
            let n = list.len() as i32;
            if n <= 0 {
                continue;
            }
            let mut j = 0;
            while j < n {
                let rot = if leader == who { near } else { 0 };
                let c = list[((rot + j) % n) as usize];
                j += 1;
                if !self.cities[c].alive {
                    continue;
                }
                if land && self.cities[c].reg != Some(region) {
                    continue;
                }
                // §5's `CityData +0x4c` mark is not carried: only a plain
                // human's scout can ever set it and no human's scout reaches
                // this function (§2, `docs/SCOUT.md` §13 item 6).

                // §6's two arms. An AI unit takes every other ring around
                // its own leader's city, and a shallow three-ring walk
                // around anybody else's.
                let (step, cap) = if leader == who {
                    (2, max_ring)
                } else {
                    (1, FOREIGN_MAX_RING)
                };
                scan.ring = cap;
                scan.leader = leader;
                let city = self.cities[c].pos.cell();
                let end = self.scout_rings(u, city, step, cap, &mut scan);
                if end == if leader == who { 12 } else { 8 } {
                    // Unreachable for an AI scout — the ring counter comes
                    // out at 13 or 9 with `step == 2`, and at 3 on the
                    // foreign arm. Kept so the shape is the original's.
                    debug_assert!(false, "an AI scout cannot exhaust a city");
                }
                max_ring = 8;
                if scan.over {
                    return scan;
                }
                if leader != who {
                    break;
                }
            }
        }
        scan
    }

    /// §6 — the ring walk around one city. Returns the ring counter as it
    /// leaves the loop, which is what §5's `scouted` mark tests.
    fn scout_rings(
        &mut self,
        u: usize,
        city: Cell,
        step: i32,
        max_ring: i32,
        scan: &mut Scan,
    ) -> i32 {
        let table = circle::table();
        let who = self.units[u].owner;
        let frame_phase = (self.frame.rem_euclid(8)) as i32;
        if max_ring <= 1 {
            return 1;
        }
        let mut ring = 1;
        while ring < max_ring {
            let phase = ring / 4 + frame_phase;
            let stride = phase + 1;
            let (start, end) = table.ring(ring);
            // `+0x436`: the rotation, modulo the **cumulative** count.
            self.mark(SITE_ROTATION);
            let rot = if end > 1 { self.rng.roll() % end } else { 0 };
            // `+0x458`: the phase, skipped when `ring / 4 + frame % 8 == 0`.
            self.mark(SITE_PHASE);
            let idx = if phase >= 1 {
                self.rng.roll() % stride
            } else {
                0
            };
            let size = end - start;
            let mut i = start + idx;
            let mut broke = false;
            while i < end {
                // The early exit: one ring past the ring the winner came
                // from. Tested per cell, before the cell is even read.
                if ring > scan.ring + 1 {
                    broke = true;
                    break;
                }
                let k = (start + (rot + i) % size) as usize;
                let c = Cell::new(
                    city.x + i32::from(table.x[k]),
                    city.y + i32::from(table.y[k]),
                );
                self.scout_cell(u, who, c, scan, ring);
                if scan.over {
                    return max_ring + 1 + step;
                }
                i += stride;
            }
            ring = if broke { max_ring + 1 } else { ring } + step;
        }
        ring
    }

    /// §7 and §8 — one candidate cell: the filter, the draw, the score.
    fn scout_cell(&mut self, u: usize, who: Player, c: Cell, scan: &mut Scan, ring: i32) {
        let Some(rec) = self.units[u].ty else { return };
        let domain = self.unit_types[rec].kind.domain;
        if c.x < 0 || c.y < 0 || c.x >= self.world.width() || c.y >= self.world.height() {
            return;
        }
        if domain != Domain::Air {
            if self.world.region_of(c) != self.world.tregion(self.units[u].pos.tile()) {
                return;
            }
            // The surface tile the original reads here is `(4x, 4y + 2)` —
            // the y centred and the x not. Transcribed as it stands.
            let t = Pos::new(c.x * 4, c.y * 4 + 2);
            let ocean = self.world.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN;
            if (domain == Domain::Land) == ocean {
                return;
            }
        }
        scan.budget += 1;
        if scan.budget > CELL_BUDGET {
            scan.over = true;
            return;
        }
        let owner = self.world.owner(c).player().unwrap_or(who);
        if self.was_really_seen(c, who) {
            if owner != who {
                return;
            }
            // The second arm of the escape — `WData +0`'s unnamed `short`,
            // negative. Zero on every world the harness builds.
            if self.scout_wdata_head(c) >= 0 {
                return;
            }
        }
        let tile = Pos::new(c.x * 4 + 2, c.y * 4 + 2);
        if self.invalid_loc(u, tile, false, false, false, false, false) != 0 {
            return;
        }
        let here = self.units[u].pos.cell();
        // `+0x64c`: the jitter that breaks ties between equidistant cells.
        self.mark(SITE_CELL);
        let mut score = vector_dist(here.x - c.x, here.y - c.y) * 8 + self.rng.roll() % 8;

        // §8. `treaties[leader] & 3 == 0` is `diplos == 0`, which
        // `LeaderData::is_enemy` reads as war — so a scout is pushed away
        // from a hostile leader's cities, and twice as far from a
        // computer's as from a human's.
        if scan.leader != who && self.at_war_with(who, scan.leader) {
            score *= 2;
            if self.ai_driven(scan.leader) {
                score *= 2;
            }
        }
        score += self.scout_danger(who, c);
        if owner != who {
            score += 4;
        }
        let goods = self.world.cell_data(c).goods;
        let cities = self.city_num(who);
        if (cities == 0 && goods & 0x02 != 0) || (cities == 1 && goods & 0x10 != 0) {
            score /= 2;
        }
        if score >= scan.score {
            return;
        }
        if self.scout_unit_near(u, c) {
            return;
        }
        scan.score = score;
        scan.tile = tile;
        scan.ring = scan.ring.min(ring);
    }

    /// **Seam** — `WData +0`'s `short`, unnamed in the type record and the
    /// second arm of §7's seen escape. Nothing here writes it.
    fn scout_wdata_head(&self, _c: Cell) -> i16 {
        0
    }

    /// **Seam** — `WorldData::danger[who]` at half the cell resolution
    /// (`(y >> 1) * reg_xs + (x >> 1)`), the grid `crate::ai_sites`'
    /// `site_danger` names. Zero in every capture, and keyed differently
    /// from [`crate::world::World::danger`], so it is not routed there.
    fn scout_danger(&self, _who: Player, _c: Cell) -> i32 {
        0
    }

    /// `find_unit_ordered(<same type, mine, not me, within 0x600>)` (§8):
    /// whether one of the scout's own units of the same type already stands
    /// within two cells of the candidate, which rejects it. The original
    /// filters on `basic_type` and adds `Search::valid_search`'s
    /// leader-visibility layer, which for a search of one's own units is a
    /// no-op (`docs/SCOUT.md` §13 item 5).
    fn scout_unit_near(&self, u: usize, c: Cell) -> bool {
        let who = self.units[u].owner;
        let ty = self.units[u].ty;
        let half = crate::world::UNITS_PER_CELL / 2;
        let at = Pos::new(
            c.x * crate::world::UNITS_PER_CELL + half,
            c.y * crate::world::UNITS_PER_CELL + half,
        );
        self.units.iter().enumerate().any(|(o, other)| {
            o != u
                && other.alive()
                && other.on_map
                && other.owner == who
                && other.ty == ty
                && vector_dist(other.pos.x - at.x, other.pos.y - at.y) <= 0x600
        })
    }

    /// §5's `find_city` — the index, **within the scout's own leader's city
    /// list**, of that leader's nearest live city in the scout's region.
    fn scout_find_city(&self, u: usize, who: Player, region: u16) -> i32 {
        let here = self.units[u].pos;
        let mut best = (i32::MAX, -1i32);
        for (i, &c) in self.cities_of(who).iter().enumerate() {
            let city = &self.cities[c];
            if city.reg != Some(region) {
                continue;
            }
            let d = vector_dist(city.pos.x - here.x, city.pos.y - here.y);
            if d <= best.0 {
                best = (d, i as i32);
            }
        }
        best.1.max(0)
    }

    /// `leader_flags & 2` — the leader is in play. The simulation keeps no
    /// flag word; a leader with a live city or a live unit is the standing
    /// answer everywhere else.
    fn scout_leader_in_play(&self, who: Player) -> bool {
        who < 8
            && (self.city_num(who) > 0 || self.units.iter().any(|u| u.owner == who && u.alive()))
    }

    /// §11 — the region fallback, taken as far as it is reproducible: the
    /// stride draw, and no further. `Region.coords`' order is the map
    /// generator's and no dump carries it, so the cell walk (and its draw
    /// per accepted cell) is not modelled and this issues no order.
    fn scout_region_scan(&mut self, region: u16) {
        let n = self.world.region_size(region);
        let stride = n.div_euclid(100).max(0) + i32::from(n % 100 != 0);
        let stride = stride.max(1) + (self.frame.rem_euclid(8)) as i32;
        if stride > 1 {
            self.rng.roll();
        }
    }

    /// §9 — the one-member group and its `EXPLORE_TO` move. `push_group` is
    /// called with `force = 1`, which is the one case a group of fewer than
    /// two installs (`docs/GROUPS.md` §3.2).
    fn scout_issue(&mut self, u: usize, tile: Pos) {
        let who = self.units[u].owner;
        let mut g = Group::stack(who);
        self.group_add(&mut g, u);
        if !self.push_group(&g, true) {
            return;
        }
        let to = Pos::new(
            tile.x * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            tile.y * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        );
        self.group_action_move_to(
            &g,
            to,
            QueuePos::New,
            false,
            Angle(0),
            MoveKind::ExploreTo,
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::world::{Cell, Terrain, World};

    /// How many draws a call spent — the LCG steps once per draw.
    fn draws(before: u32, after: u32) -> i32 {
        let mut r = crate::combat::Rng::new(before);
        for n in 0..64 {
            if r.seed == after {
                return n;
            }
            r.roll();
        }
        panic!("more than 64 draws, or the seed never matched");
    }

    /// A 40 × 40 land world, player 1 a computer with a city at cell
    /// (20, 20) and a scout standing in it, player 0 a human with one.
    /// `fog` installs an all-unseen grid; without one nothing is seen and
    /// [`Sim::was_really_seen`] answers true everywhere, which is the flat
    /// world the harness has always had.
    fn scout_sim(fog: bool) -> (Sim, usize, usize) {
        let mut world = World::new(40, 40);
        let r = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        if fog {
            assert!(world.set_fog(vec![0; 80 * 80]));
        }
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        let t = s.add_unit_type(crate::UnitType {
            hits: 20,
            moves: 40,
            ..crate::UnitType::default()
        });
        s.unit_types[t].cols.role = role::SCOUT;

        let centre = Pos::new(20 * 768 + 384, 20 * 768 + 384);
        let bldg = s.add_building(1, centre, 4);
        s.cities.push(crate::city::City {
            alive: true,
            owner: 1,
            race: Some(1),
            founder: 1,
            building: bldg,
            members: Vec::new(),
            reg: Some(r),
            pos: centre,
            capital: true,
            founding_capital: true,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            no_muster: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 1,
            has_citizen: false,
            source: None,
        });

        let spawn = |s: &mut Sim, who: Player, at: Pos| {
            let mut u = crate::Unit::new(who, s.units.len() as i16, at, 20);
            u.ty = Some(t);
            s.add_unit(u)
        };
        let ai = spawn(&mut s, 1, centre);
        let human = spawn(&mut s, 0, Pos::new(5 * 768 + 384, 5 * 768 + 384));
        (s, ai, human)
    }

    /// §4: `circle_init`'s first rings, and the check that the sweep order
    /// is right — ring 1 is the eight neighbours and nothing else.
    #[test]
    fn the_circle_tables_start_with_the_eight_neighbours() {
        let t = circle::table();
        assert_eq!(
            &t.radius[..8],
            &[1, 9, 21, 45, 69, 105, 145, 185],
            "the cumulative counts of docs/SCOUT.md §4"
        );
        let (lo, hi) = t.ring(1);
        let mut ring: Vec<(i8, i8)> = (lo..hi)
            .map(|k| (t.x[k as usize], t.y[k as usize]))
            .collect();
        ring.sort_unstable();
        assert_eq!(
            ring,
            vec![
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1)
            ],
            "ring 1 is the eight neighbours"
        );
        assert_eq!(t.x.len(), 12_873, "circle_points");
        assert_eq!(t.x.len(), t.y.len());
    }

    /// §6, the ring walk on its own. Nothing is unseen, so no cell is ever
    /// accepted, `best_ring` never drops, and the walk runs to its bound:
    /// an AI scout around its own leader's city takes rings 1, 3, 5, 7, 9
    /// and 11 — six rotations — and the phase draw only from ring 4 up,
    /// which is four of them. Ten draws, none of them a cell.
    #[test]
    fn an_ai_scout_takes_every_other_ring_and_the_phase_draw_from_ring_four() {
        let (mut s, ai, _) = scout_sim(false);
        let before = s.rng.seed;
        assert!(
            !s.think_scout(ai),
            "everything is seen, so nothing is found"
        );
        assert_eq!(
            draws(before, s.rng.seed),
            6 + 4 + 1,
            "six rings, four phases, the §11 stride"
        );
        assert!(s.units[ai].orders.is_empty(), "and no order is issued");
    }

    /// §7 and §6's early exit together. With every cell unseen the eight
    /// cells of ring 1 are all accepted — one draw each — which drops
    /// `best_ring` to 1, and ring 3 then breaks at its first cell, after
    /// its rotation draw. Two rotations and eight cells; the phase draw
    /// fires at neither ring, both being under 4.
    #[test]
    fn an_unseen_ring_one_is_taken_whole_and_stops_the_walk_two_rings_later() {
        let (mut s, ai, _) = scout_sim(true);
        let before = s.rng.seed;
        assert!(s.think_scout(ai), "a target is found");
        assert_eq!(draws(before, s.rng.seed), 2 + 8);
        // §9: the one-member group's `EXPLORE_TO`, at the tile centre of a
        // neighbouring cell's `4c + 2`.
        let order = *s.units[ai].orders.front().expect("an order");
        let crate::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, MoveKind::ExploreTo);
        let c = m.dest.cell();
        assert!(
            (c.x - 20).abs() <= 1 && (c.y - 20).abs() <= 1 && c != Cell::new(20, 20),
            "a neighbour of the city, not the city: {c:?}"
        );
    }

    /// §2's gate, from both sides: a human's scout never reaches
    /// `think_scout` (its `unit_masks & 0x40000` is clear), and neither
    /// does a unit without the scout bit.
    #[test]
    fn the_gate_takes_the_computers_scout_and_nothing_else() {
        let (mut s, ai, human) = scout_sim(true);
        assert!(s.scout_thinks(ai), "the computer's scout");
        assert!(!s.scout_thinks(human), "the human's does not");

        let plain = s.add_unit_type(crate::UnitType {
            hits: 20,
            ..crate::UnitType::default()
        });
        let mut u = crate::Unit::new(1, s.units.len() as i16, s.units[ai].pos, 20);
        u.ty = Some(plain);
        let other = s.add_unit(u);
        assert!(!s.scout_thinks(other), "no scout bit, no explore");
    }

    /// §5: a leader with no city of its own walks nobody's rings — the
    /// `city_num == 0` gate at the head of the city loop. What is left is
    /// the region fallback's stride draw, which fires because a region of
    /// 1,600 cells gives it a stride of 16 (§11).
    #[test]
    fn a_leader_with_no_city_falls_through_to_the_region_stride_draw() {
        let (mut s, ai, _) = scout_sim(true);
        s.cities[0].alive = false;
        let before = s.rng.seed;
        assert!(!s.think_scout(ai));
        assert_eq!(draws(before, s.rng.seed), 1, "the stride draw of §11");
    }
}
