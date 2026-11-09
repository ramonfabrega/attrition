//! The idle scout's explore target — `Unit::think_scout@005f6010`.
//!
//! `docs/SCOUT.md` is the mechanic; this is both its branches. An AI scout
//! with nothing to do walks rings of cells outward from each city it knows
//! about, scores every cell that is inside its own region and **not really
//! seen**, and sends itself to the cheapest one as a one-member group with
//! an `EXPLORE_TO` move — and when the rings turn up nothing at all, it
//! strides through the whole region's coordinate list instead (§11).
//!
//! The city branch is three draws a ring at most, and it is the last
//! unattributed block of frame 0 on run20 and on the fuzzer's control map
//! (`docs/SYNC.md` §4.2): ten draws at `+0x436` ×4, `+0x458` ×2, `+0x64c`
//! ×4 on both. The counts fall out of the ring walk's two guards and the
//! cell filter, and `docs/SCOUT.md` §10 is the sequence, ring by ring. The
//! fallback is one `+0x941` and one `+0xaba` a cell, and run39's frame
//! 1373 is the only frame in the corpus that reaches it.

use crate::ai_load::role;
use crate::attrition::Domain;
use crate::group::Group;
use crate::movement::Angle;
use crate::orders::{MoveKind, QueuePos};
use crate::world::{Cell, Pos, UNITS_PER_TILE, tile, vector_dist};
use crate::{Player, Sim};

/// `SPY` — the `TypeIndex` §2's gate and §3's branch test the lineage of.
pub(crate) const SPY: crate::tech::TypeId = 0x3a;

/// `TypeIndex::PEASANTS` and `PEASANTSKOREAN` — `UnitTypeData +0x4`, the two
/// citizen ids `think_scout` sends down the region scan (§3).
const PEASANTS: i32 = crate::ids::ty::PEASANTS as i32;
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
/// §11's two, in the region fallback: the starting offset into
/// `Region.coords`, once a call, and the jitter, once per accepted cell.
pub const SITE_REGION_STRIDE: &str = "Unit::think_scout+0x941";
pub const SITE_REGION_CELL: &str = "Unit::think_scout+0xaba";

/// The address range those three fall in — `Unit::think_scout` up to
/// `Unit::think`, the next function in the export. What
/// `rondata::trace::Trace::run_in` takes to isolate this mechanic's own
/// draws from the ones its callees take.
pub const CODE: std::ops::Range<u32> = 0x005f_6010..0x005f_6e40;

/// The `max_ring` an AI unit uses around a **foreign** leader's city (§6).
pub const FOREIGN_MAX_RING: i32 = 3;

/// The order indices `ObjectsData::find_unit_ordered` accepts — the move
/// family, `{1, 2, 3, 4, 0x12, 0x13, 0x15}` (`docs/ORDERS.md` §1.2), which
/// is what makes the function's name literal: a unit with no order, or with
/// a gather or a build, is not "ordered" anywhere and never rejects a cell.
pub(crate) const MOVE_FAMILY: [u8; 7] = [1, 2, 3, 4, 0x12, 0x13, 0x15];

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
        if self.tables.unit_types[rec].kind.supply_unit {
            return false;
        }
        if !self.unit_is_scout(u) && !self.unit_is_spy(u) {
            return false;
        }
        self.ai_driven(unit.owner) && self.army_of(u).is_none()
    }

    /// `role & 0x10` on the unit's type — the scout bit
    /// ([`crate::ai_load::role::ROLE_SCOUT`]).
    pub fn unit_is_scout(&self, u: usize) -> bool {
        let Some(rec) = self.units[u].ty else {
            return false;
        };
        let cols = self.tables.unit_types[rec].cols.role;
        if cols != 0 {
            return cols & role::ROLE_SCOUT != 0;
        }
        // A type the loader never filled — a harness fixture. The tree id is
        // what `role_word` needs for its `Scout` lineage test.
        self.tables.unit_types[rec]
            .tree
            .is_some_and(|t| self.role_word(t, rec) & role::ROLE_SCOUT != 0)
    }

    /// `ObjectData::is(SPY, 0)` — the type's own lineage test through its
    /// vtable's `+0x60`, `SPY` being `0x3a`. The second half of §2's gate
    /// (`005f75ea`) and one of the three tests that route a unit to the
    /// region scan (§3, `005f688e`). A seam answering false until item 664:
    /// run171's Spy `1/9` is the first spy on disk, and its birth frame, 617,
    /// is the region scan whole (§15).
    pub(crate) fn unit_is_spy(&self, u: usize) -> bool {
        self.unit_line_is(u, SPY)
    }

    /// `UnitData::is_special@0046cea0`, which the vtable resolves to
    /// `unit_flags2 & 0x10` — [`crate::ai_load::uflags2::FLAG2_SCOUT`]. Read
    /// twice: by `Unit::think`'s human block (§2) and by §11's `local_74`.
    pub(crate) fn unit_is_special(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|r| {
            self.tables.unit_types[r]
                .cols
                .flag2(crate::ai_load::uflags2::FLAG2_SCOUT)
        })
    }

    /// `unit_masks & 0x40000`, set at `Unit::init@00612100:586` for every
    /// unit whose leader is not a plain human — `(leader_flags & 0xc) != 4`.
    /// `Nation::human` is `leader_flags & 4` and is what
    /// [`Sim::think_join_army`](crate::Sim::think_join_army) already reads;
    /// the AI-driven-human bit `0x8` is the seam left in it.
    pub(crate) fn ai_driven(&self, who: Player) -> bool {
        self.cities_state
            .nation
            .get(who as usize)
            .is_some_and(|n| !n.human)
    }

    /// `unit_masks & 0x40000` as a **unit** carries it: [`Sim::ai_driven`]
    /// until the leader is defeated, when `Leader::defeat@006ecb00:104-124`
    /// clears the bit on every live unit it has (twenty-fourth pass, group
    /// 6; A4 row 19). The leader-level reads (`leader_flags & 4`) stay on
    /// `ai_driven`.
    pub(crate) fn unit_ai_bit(&self, who: Player) -> bool {
        self.ai_driven(who) && !self.defeated.get(who as usize).copied().unwrap_or(false)
    }

    /// `Object::find_nearby_target@00648e6e`'s `local_40` — the flag
    /// [`Sim::compare_target`] takes as its fourth argument and which
    /// inverts the damage weight (`docs/COMBAT.md` §33.2):
    ///
    /// ```text
    /// (leaders[who].flags & 4) == 0 && get_diff(leaders[who]) == 0
    ///     && (game.semaphore[1] & 2) == 0
    /// ```
    ///
    /// The first term is [`Sim::ai_driven`] and the second
    /// [`Sim::ai_difficulty`], which `LeaderData::get_diff@006ec000`
    /// answers off the lobby outside a multiplayer game. **The third is a
    /// seam**: `game.semaphore[1] & 2` can only ever *clear* the flag, so
    /// this errs toward the AI arm on a setting no capture uses, and
    /// `get_diff`'s own multiplayer branches are not carried either.
    pub fn search_ai(&self, who: Player) -> bool {
        self.ai_driven(who) && self.ai_difficulty() == 0
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
    /// The two leader exits are [`Sim::sees_every_unit`]'s (item 2780,
    /// `docs/AI.md` §233): `leader_flags & 0x800` — the Space Program, or
    /// `REVEAL_ENEMY_BONUS` — and `LeaderData +0x59e4`, which is
    /// `num_units[0x141]`, a Fouché standing. Either and every cell is
    /// seen (`006b54f0`'s third and fourth tests).
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
        // The two leader exits, ahead of the grid: the peace pair's who=1
        // holds the Space Program from before 5401 (run867's `leader_flags`
        // `0xa3f03`), so its scout's region scan accepts no cell on 5564.
        if self.sees_every_unit(who) {
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
        let domain = self.tables.unit_types[rec].kind.domain;
        let type_index = self.units[u].type_index;

        // The head, and it comes before the region read: a land unit that
        // can see an untaken box walks to that instead of thinking, and
        // spends no draw doing it (`docs/GOODY.md` §7).
        if domain == Domain::Land && self.find_goody_box(u) {
            return true;
        }
        // Then a computer leader's caster is offered its crafts
        // (`think_scout+0x7c`, `unit_flags2 & 2`): `think_spellcaster`,
        // whose AI arm draws from difficulty 2 up (`docs/AI.md` §80.5).
        if !self.cities_state.nation[who as usize].human
            && self.tables.unit_types[rec]
                .cols
                .flag2(crate::ai_load::uflags2::CASTER)
            && self.think_spellcaster(u)
        {
            return true;
        }

        let Some(region) = self.world.tregion(self.units[u].pos.tile()) else {
            return false;
        };

        // §3's branch. `unit_masks & 0x100` is a seam (clear on every scout
        // in every run on disk); the two citizen ids, the spy and the naval
        // domain are what route a unit away from the city loop.
        let city_branch = type_index != PEASANTS
            && type_index != PEASANTSKOREAN
            && !self.unit_is_spy(u)
            && domain != Domain::Sea;
        let mut scan = if city_branch {
            self.scout_city_loop(u, who, region)
        } else {
            // `005f686b`: a citizen, a spy or a naval unit skips the city
            // loop whole and takes the region scan on an untouched `best`.
            Scan {
                score: NOTHING,
                tile: Pos::default(),
                ring: 0,
                budget: 0,
                over: false,
                leader: who,
            }
        };

        // §3's tail. Above 199 the region scan runs as well and can win it.
        if !city_branch || scan.score > REGION_SCAN_ABOVE {
            self.scout_region_scan(u, who, region, &mut scan);
        }
        if scan.score >= NOTHING {
            // §3's own tail, at `005f6d74`: nothing anywhere.
            //
            // A non-air unit **marks its region scouted** for this leader
            // — `Region.scouted`, the bit `think_civilian_transport` then
            // reads, and the reason a scout that has given up on home does
            // not choose home again (`docs/TRANSPORT.md` §7). A sea unit
            // joins an army instead. A citizen that is not exploring and
            // has no city in its region stops here. Everything else asks
            // for an island.
            if domain != Domain::Air {
                self.world.mark_region_scouted(region, who);
            }
            if domain == Domain::Sea {
                // `5f6db5`: `cmpl $1, 0x218(type)`, then `call
                // Unit::add_to_army` and `setns` — a sea unit with nothing
                // left to see joins the nearest sea army, and walks to its
                // first member (`docs/SCOUT.md` §13 item 1b; item 1297's
                // Caravel `1/35` on East Indies' 11523).
                return self.add_to_army(u).is_some();
            }
            let citizen = type_index == PEASANTS || type_index == PEASANTSKOREAN;
            if citizen && self.reg_cities(who, region) == 0 {
                return false;
            }
            return self.think_civilian_transport(u, false);
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
            .is_some_and(|r| self.tables.unit_types[r].kind.domain == Domain::Land);

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
                if !self.cities_state.cities[c].alive {
                    continue;
                }
                if land && self.cities_state.cities[c].reg != Some(region) {
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
                let city = self.cities_state.cities[c].pos.cell();
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
        let domain = self.tables.unit_types[rec].kind.domain;
        if c.x < 0 || c.y < 0 || c.x >= self.world.width() || c.y >= self.world.height() {
            return;
        }
        if domain != Domain::Air {
            if self.world.region_of(c) != self.world.tregion(self.units[u].pos.tile()) {
                return;
            }
            // The surface tile is the cell **centre**, `(4x + 2, 4y + 2)` —
            // the same tile `invalid_loc` is handed below. The listing at
            // `005f6542` is `movb 0x4(%eax,%ecx,2)` over `ecx =
            // (4y + 2)·tile_xs + 4x`, and `TData` is two bytes wide with
            // its `mask` at `+0`, so the `+4` is **two elements**, not a
            // field offset: tile index `ecx + 2` (`docs/SCOUT.md` §7).
            let t = Pos::new(c.x * 4 + 2, c.y * 4 + 2);
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
            // The second arm of the escape — `WData +0` read as a
            // *signed* short, so "negative" is bit `0x8000`: `GOODY`, a
            // cell carrying a goody box (`docs/SCOUT.md` §7). Great Lakes
            // has 22 and none of them is ever a candidate in any capture,
            // but the bit is loaded, so this reads it rather than a seam.
            if self.world.cell_data(c).flags & crate::world::cell::GOODY == 0 {
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

        // §8. `treaties[leader] & 3 == 0` — `LeaderData +0x94`, **not**
        // `diplos` at `+0x74` (`005f6688`: `0xe3a424` is leader 0's
        // `+0x94`) — is "these two have not met": bit 0 is the met bit
        // `Leader::meet` sets. So a scout is pushed away from a stranger's
        // cities, twice as far from a computer's as from a human's, and
        // stops being pushed once the two have met (§8.3).
        if scan.leader != who && self.scout_treaty(who, scan.leader) & 3 == 0 {
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

    /// `leaders[who].treaties[other]` (`LeaderData +0x94`), raw: the met
    /// bit and whatever else is set, with no defeated test — the read
    /// `think_scout`'s score makes at `005f6688` (`docs/SCOUT.md` §8.3).
    fn scout_treaty(&self, who: Player, other: Player) -> i32 {
        self.treaties
            .get(who as usize)
            .and_then(|r| r.get(other as usize))
            .copied()
            .unwrap_or(0)
    }

    /// `WorldData::danger[who][(y >> 1) * reg_xs + (x >> 1)]`, the term
    /// both of `think_scout`'s scans add to a candidate's score. The
    /// listing settles that they are the **same** expression twice —
    /// `sarl` on each cell coordinate, `imull 0x24(%ecx)` for `reg_xs`,
    /// the row at `0x13c(%ecx,who,4)`, then the load and the `addl` that
    /// carries the not-mine four with it: `005f66d2` in the city loop
    /// (`Unit::think_scout+0x6c2`) and `005f6b45` in the region scan
    /// (`+0xb35`).
    ///
    /// ~~**Seam** … keyed differently from `World::danger`, so it is not
    /// routed there.~~ It is not keyed differently:
    /// [`crate::world::World::danger_half`] is `(c.y / 2) * reg_xs +
    /// (c.x / 2)` on a non-negative cell, which is the listing's shift
    /// exactly, and the grid behind it is diff-backed value for value
    /// against run64's own `danger[who][scan]` (`docs/DANGER.md` §8).
    /// The seam was the routing and nothing else, and it cost Great
    /// Lakes a hundred frames: `docs/SCOUT.md` §8.1.
    fn scout_danger(&self, who: Player, c: Cell) -> i32 {
        self.world.danger_half(who, c)
    }

    /// `find_unit_ordered(<same lineage, mine, not me, within 0x600>)` (§8):
    /// whether one of the scout's own units **of the same lineage** is
    /// already **on its way** to within two cells of the candidate, which
    /// rejects it. `Search::valid_search`'s leader-visibility layer, which
    /// for a search of one's own units is a no-op (`docs/SCOUT.md` §13
    /// item 5).
    ///
    /// **"Same type" is `FILTER_BASE_TYPE`, the root of the `FROM` chain**
    /// (item 3857, `docs/GOLDEN.md` §153.4): the value `think_scout` hands
    /// the search is the scout's type's vslot `+0xe4`, which on a packet is
    /// `ObjectTypeData::basic_type@006619e0` — `from` (`TypeData +0x3c`)
    /// followed to the type whose `from` is negative, and that type's own
    /// index. The fourth melee's Russian Fluyt (`TI` 332, 331 → 324 → 323)
    /// and Aegis Cruiser (`TI` 330, 329 → … → 323) share the Bark: on 1957
    /// the cruiser's explore point sits 659 units from the Fluyt's best
    /// cell, and the original refuses that cell where an exact-type test
    /// took it.
    ///
    /// **The "ordered" in the name is two predicates, not a synonym for
    /// "standing"** (`ObjectsData::find_unit_ordered@0065bc40`, read
    /// 2026-09-01): a candidate unit must have a **non-empty order list**
    /// (`+0xdc`), and its current order's `get_type()` must be one of
    /// `{1, 2, 3, 4, 0x12, 0x13, 0x15}` — the move family exactly
    /// (`docs/ORDERS.md` §1.2). A unit standing still, gathering or
    /// building is never a reason to skip a cell.
    ///
    /// **And `0x200` is a flag, not a second radius**: the argument the
    /// call passes beside `0x600` is `param_6`, and the only bit the
    /// function reads out of it asks that the candidate unit's own cell
    /// **region** equal the target cell's — `WData +4` on both sides, the
    /// plain cell region rather than `get_tregion`.
    ///
    /// **And "on its way" is literal: the distance is to where the sibling
    /// is going.** `0065be35` reads `UnitData +0x70/+0x74` — `orders_x`/
    /// `orders_y` — against the candidate's centre; the XOR-ed position at
    /// `+0x10/+0x14` is read only for the region test above
    /// (`docs/SCOUT.md` §8.1; item 1156). The merchant's
    /// `Sim::merchant_refused` had it from the same listing.
    fn scout_unit_near(&self, u: usize, c: Cell) -> bool {
        let who = self.units[u].owner;
        let base = self.unit_base_type(u);
        let half = crate::world::UNITS_PER_CELL / 2;
        let at = Pos::new(
            c.x * crate::world::UNITS_PER_CELL + half,
            c.y * crate::world::UNITS_PER_CELL + half,
        );
        // `sVar1`, read once at the head under `param_6 & 0x200`.
        let region = self.world.region_of(c);
        (0..self.units.len()).any(|o| {
            let other = &self.units[o];
            o != u
                && other.alive()
                && other.on_map
                && other.owner == who
                && self.unit_base_type(o) == base
                && MOVE_FAMILY.contains(&self.order_type(o))
                && self.world.region_of(other.pos.cell()) == region
                && vector_dist(other.orders_pos.x - at.x, other.orders_pos.y - at.y) <= 0x600
        })
    }

    /// `ObjectTypeData::basic_type@006619e0` on a unit's type: `from`
    /// followed to the root of the lineage. A type outside the tree is its
    /// own record, so two such units match only on the same record.
    fn unit_base_type(&self, u: usize) -> Result<crate::tech::TypeId, Option<usize>> {
        let rec = self.units[u].ty;
        let Some(mut t) = rec.and_then(|i| self.tables.unit_types[i].tree) else {
            return Err(rec);
        };
        let mut guard = 0;
        while let Some(f) = self.tables.tech_tree.types.get(t).and_then(|d| d.from) {
            t = f;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        Ok(t)
    }

    /// §5's `find_city` — the index, **within the scout's own leader's city
    /// list**, of that leader's nearest live city in the scout's region.
    fn scout_find_city(&self, u: usize, who: Player, region: u16) -> i32 {
        let here = self.units[u].pos;
        let mut best = (i32::MAX, -1i32);
        for (i, &c) in self.cities_of(who).iter().enumerate() {
            let city = &self.cities_state.cities[c];
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

    /// §11 — the region fallback, whole.
    ///
    /// The city loop leaves `best > 199` — no city in the scout's region,
    /// or nothing near enough — and the function scans the scout's **whole
    /// region** instead, striding through `Region.coords`. That list's
    /// order was carried as unrecoverable until 2026-08-31 (item 110):
    /// `Regions::rebuild_coords@0067f800` is the last writer map load
    /// reaches, and it is a plain row-major sweep of the cell grid, so
    /// [`World::region_coords_strided`](crate::world::World::region_coords_strided)
    /// is it and this branch is the original's draw for draw.
    ///
    /// Two draw sites: `+0x941` for the starting offset, once a call, and
    /// `+0xaba` for every cell that passes the fog, the location and the
    /// surface tests.
    fn scout_region_scan(&mut self, u: usize, who: Player, region: u16, scan: &mut Scan) {
        let Some(rec) = self.units[u].ty else { return };
        let domain = self.tables.unit_types[rec].kind.domain;

        // `local_74`, stored at `005f68dc` and read twice in the score. The
        // branch at `005f6888` skips the store when the leader **is**
        // human, so what carries it is an AI leader's spy, scout or naval
        // unit — and every scout in every capture is one.
        let quartered = self.ai_driven(who)
            && (self.unit_is_spy(u) || self.unit_is_special(u) || domain == Domain::Sea);

        let n = self.world.region_size(region);
        let mut stride = (n + 99) / 100;
        if stride < 1 {
            stride = 1;
        }
        stride += (self.frame.rem_euclid(8)) as i32;
        // `+0x941`: the starting offset, skipped when the stride is 1.
        self.mark(SITE_REGION_STRIDE);
        let start = if stride > 1 {
            self.rng.roll() % stride
        } else {
            0
        };

        let here = self.units[u].pos.cell();
        for c in self.world.region_coords_strided(region, start, stride) {
            // `was_really_seen`, inlined at `005f69cd` rather than called.
            if self.was_really_seen(c, who) {
                continue;
            }
            let tile = Pos::new(c.x * 4 + 2, c.y * 4 + 2);
            if self.invalid_loc(u, tile, false, false, false, false, false) != 0 {
                continue;
            }
            // The surface test, by domain. The land arm's probe is the cell
            // **centre** tile, the same decompiler fold §7's is: `movb
            // 0x4(%eax,%ecx,2)` at `005f6a88` over `ecx = (4y + 2)·tile_xs
            // + 4x`, and `TData` is two bytes wide.
            let ocean = self.world.tile_mask(tile) & tile::SURFACE == tile::SURFACE_OCEAN;
            let ok = match domain {
                Domain::Air => true,
                // `get_inside(this) >= 0` takes a carried naval unit down
                // the land arm; it is otherwise unreachable for `Land`.
                Domain::Land => !ocean,
                Domain::Sea if self.units[u].inside.is_some() => !ocean,
                Domain::Sea => self.world.is_ocean(c),
            };
            if !ok {
                continue;
            }

            // `+0xaba`: the jitter, one draw per accepted cell. The
            // distance is scaled by **16** here, not the city loop's 8.
            self.mark(SITE_REGION_CELL);
            let mut score = vector_dist(here.x - c.x, here.y - c.y) * 16 + self.rng.roll() % 8;
            let owner = self.world.owner(c).player().unwrap_or(who);
            if quartered {
                if !self.is_ally(who, owner) {
                    score /= 4;
                }
                if self.world.tile_mask(tile) & tile::CITY_RADIUS != 0 {
                    score /= 2;
                }
            } else {
                score += self.scout_danger(who, c);
                if owner != who {
                    score += 4;
                }
            }
            // The winner is compared and stored **doubled** — `leal
            // (%ecx,%ecx), %eax` at `005f6bb9` — so a region-scan cell has
            // to be twice as good as a city-loop one to take the frame.
            let score = score * 2;
            if score >= scan.score {
                continue;
            }
            if self.scout_unit_near(u, c) {
                continue;
            }
            scan.score = score;
            scan.tile = tile;
        }
    }

    /// §9 — the one-member group and its `EXPLORE_TO` move. `push_group` is
    /// called with `force = 1`, which is the one case a group of fewer than
    /// two installs (`docs/GROUPS.md` §3.2).
    fn scout_issue(&mut self, u: usize, tile: Pos) {
        let who = self.units[u].owner;
        let mut g = Group::stack(who);
        self.group_add(&mut g, u);
        if !self.push_group(&mut g, true) {
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
mod tests;

#[cfg(test)]
mod unit_bit_tests;
