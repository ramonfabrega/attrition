//! The frame: [`Sim::tick`], the stream's draw [`Sim::rnd`], the frame
//! constants and the attrition [`Tick`] a frame reports.
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6):
//! the tick's body is the same statements in the same order, with the same
//! `mark` phases, so the random stream and the trace's vocabulary are the
//! base's.

use crate::*;

/// Frames per second. The original's whole clock, and the unit every attrition
/// period is quoted in.
pub const FRAMES_PER_SECOND: i32 = 15;

/// How often a unit's periodic work runs at all. `Unit::process` gates a whole
/// block of per-unit upkeep on this.
pub const UNIT_UPKEEP_FRAMES: i64 = 16;

/// How often a unit's attrition period is recomputed.
///
/// Not every frame. This is the single most consequential fact about the
/// cadence: a unit that walks out of hostile territory keeps its stale period
/// until the next refresh, and one that walks in takes nothing until then.
pub const ATTRITION_REFRESH_FRAMES: i64 = 32;

/// How often `ObjectData::targeted` is quartered —
/// `Unit::process@00610bc0`'s `(frame + o) % 16` slot, the one
/// [`ATTRITION_REFRESH_FRAMES`]' is nested inside. `docs/COMBAT.md` §33.
pub const TARGETED_DECAY_FRAMES: i64 = 16;

/// One figure taking one tick of attrition damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    pub unit: usize,
    pub frame: i64,
    /// Sixteenths of a hit point dealt — [`attrition::damage`] for the
    /// figure's squad size.
    pub sixteenths: i32,
    /// Whole hit points actually deducted this tick, after the fractional
    /// accumulator. Zero on most ticks for a figure in a squad of four.
    pub lost: i32,
    pub killed: bool,
}

impl Sim {
    /// `GameAccess::rnd(n)@0043cca0` — `Random::get(game_random, 0, 0xffff)
    /// % n`, and **zero without a draw** when `n <= 1`. The early return is
    /// the whole reason this is a function rather than a `%`: a caller that
    /// passes a one-wide span spends nothing, and a frame's draw count is
    /// what the diff compares (`docs/ORDERS.md` §6.5).
    pub(crate) fn rnd(&mut self, n: i32) -> i32 {
        if n <= 1 {
            return 0;
        }
        self.rng.roll() % n
    }

    /// Advances one frame.
    ///
    /// The cadence is the part of this mechanic that would be easiest to get
    /// plausibly wrong, and it is not a countdown. `Unit::process` recomputes
    /// the period every 32 frames and, on **every** frame, applies damage when
    /// `(frame + unit index) % period == 0`. Two things follow that a
    /// countdown would not give:
    ///
    /// - **The damage is phase-locked to the global frame**, not to when the
    ///   unit entered hostile ground. Change a period from 48 to 24 and the
    ///   unit re-locks to the 24-frame grid immediately, mid-interval.
    /// - **Leaving hostile territory does not stop the bleeding at once.** The
    ///   stale period survives until the next refresh, so a unit can take one
    ///   more tick up to 32 frames after walking out — and, symmetrically,
    ///   takes nothing for up to 32 frames after walking in.
    pub fn tick(&mut self) -> Vec<Tick> {
        self.in_play = true;
        let frame = self.frame;
        let mut events = Vec::new();
        // The staged input's marks, taken before the clear and put back at
        // the head of the frame's own — `do_frame`'s entry, which is where
        // the cheat channel spends them (`docs/INPUT.md` §11.1).
        let staged: Vec<(String, u32)> = self.staged_marks.drain(..).collect();
        if self.trace_phases {
            self.phase_marks.clear();
            self.phase_marks.extend(staged);
        }
        self.mark("income");

        // Income first. `Game::do_frame` runs `Leaders::process_all` before
        // `Objects::process_all`, so every player is paid for the frame before
        // any unit in it moves, fights or bleeds. That ordering is observable:
        // a citizen that dies this frame was already paid for it.
        for who in 0..self.players.len() {
            let player = u8::try_from(who).expect("too many players");
            // `Leader::process@006b88b0`'s head: the frame's retarget
            // count starts again at zero (`docs/COLLISION.md` §14).
            self.retargets[who] = 0;
            // `Leader::calc_gather` assembles its inputs from the live state
            // inside the same gate it recomputes under, so the holdings are
            // rebuilt only on the frames the rate is actually reassembled —
            // which makes income appear on exactly the frame it does there
            // (`crates/sim/src/economy/holdings.rs`).
            if self.holdings_due(player, frame) {
                self.assemble_holdings(player);
            }
            // `Leader::gather@006ce280:58` runs `calc_resource_caps` on
            // every frame, outside the reassembly gate, and that reads the
            // Commerce level straight off `LeaderDataEncrypt + 0xf0`. So a
            // level raised between two reassemblies caps the very next
            // frame's income: `library who=1 2` at 600 is 2992 on run157's
            // first block, 605, where the level read at the last
            // reassembly still said 1392 (item 644, parked 633).
            self.holdings[who].commerce =
                self.tech[who].epoch[tech::Line::Commerce.index()].max(0) as usize;
            // And its wonder arm reads `has_wonder` there too: the
            // Pyramids `1/2026` activate on Great Lakes' sim-frame 17084
            // and who=1's food and wealth caps are 300, not 250, on 17085
            // (`docs/ECONOMY.md` §15).
            self.holdings[who].wonders = self.wonders_held(player);
            // `do_gather`'s escrow arm reads `escrow_rate` on every frame,
            // and a human never escrows (`docs/AI.md` §98).
            self.holdings[who].escrow_rate = if self.nation[who].human {
                [0; economy::RESOURCES]
            } else {
                self.ai[who].census.escrow_rate
            };
            economy::process(
                &self.tuning,
                &mut self.ledgers[who],
                &self.holdings[who],
                player,
                frame,
            );
            // `Leader::gather`'s own tail, every frame: `rare = rare_owned |
            // rare_conquest`, and a change raises **`0xc000000`**
            // (`006ce280`, the `operator!=` arm's last line) — the
            // unit-stats pass below *and* the wall-stats one, which
            // re-bakes every unfinished building's construction clock.
            // `rare_owned` only moves on a recompute frame, so this can
            // only fire on one — but the original tests it every frame and
            // so does this ([`crate::rares`]).
            //
            // **The wall half arrived with item 261.** It was `0x4000000`
            // alone here, and the missing half is a whole rare's building
            // bonus never reaching a site: Great Lakes' AI collects
            // Tobacco on 6751 and the original's Tower drops from 100000
            // to **90909** on 6752, finishing at 7176 where this crate's
            // ran on (`docs/CITIES.md` §3.2).
            let rare = self.holdings[who].rare_owned;
            if self.ledgers[who].rare != rare {
                // `Leader::calc_gather`'s own border arm, and it is one
                // bit wide: the writer compares `rare.ptr[2] >> 7` before
                // and after and calls `Regions::fix_all_borders` only when
                // **the gem bit** moved, because no other rare is in the
                // border table. `fix_all_borders` merely zeroes each
                // region's border stamp, so the recompute it schedules
                // reads the *new* mask; this crate recomputes on the spot
                // and therefore assigns first (`docs/ATTRITION.md`,
                // "Territory"; `docs/audit/2026-09-05-economy-vs-code.md`
                // R18).
                let gem = 1u64 << (economy::GEMS - economy::BASE_RARE);
                let gems_moved = (self.ledgers[who].rare ^ rare) & gem != 0;
                self.ledgers[who].rare = rare;
                self.unit_stats_dirty[who] = true;
                self.wall_stats_dirty[who] = true;
                if gems_moved {
                    self.sync_territory();
                }
                // And then `calc_pop_cap(this)` (`006ceee0:353`), whose one
                // rare arm is Peacocks: Great Sahara's AI holds them, its
                // cap is 55 where the lobby's Military table gives 50, and
                // the Mercenaries offer's `cap × 5 / 6 < effective_pop` read
                // 41 < 44 here against 45 there (item 1147).
                self.recompute_pop_caps();
            }
            // `Leader::process` also answers the wall-stats dirty flag here,
            // before any building is touched.
            if self.wall_stats_dirty[who] {
                self.calc_wall_stats(player);
            }
            // And then `0x4000000` — `Leader::calc_unit_stats`, which is
            // where a rare that appeared in this frame's `Leader::gather`
            // reaches the units. `Leaders::process_all` runs before
            // `Objects::process_all`, so the new speed is the one the
            // frame's own step uses (`docs/ECONOMY.md`, "What an owned
            // rare does").
            if self.unit_stats_dirty[who] {
                self.unit_stats_dirty[who] = false;
                self.calc_unit_stats(player);
            }
            // `Leader::process`'s victory clocks, after the stats: a
            // stopped timer's debt paid back (`crate::victory`).
            self.refresh_victory_timers(who);
        }

        // The peace's end, between the two (`Game::do_frame:202`).
        self.end_the_peace();

        self.mark("strategy_all");
        // `Leaders::strategy_all` — the production AI, between the income
        // and the objects (`Game::do_frame` line 267; `docs/AI.md` §2.1).
        self.strategy_all();

        // `GameDaemon::process_all`'s **first** act, before anything it
        // does for vision or the market: every player's repath pressure is
        // halved and snapped to zero under three
        // (`docs/PATHFINDER.md` §8). It is what keeps the collision
        // throttle of `docs/COLLISION.md` §6 step 6 a *rate* rather than a
        // lifetime count — without it the counter only climbs, and a unit
        // that has collided four times in a game is throttled for the rest
        // of it. It draws nothing, so it takes no mark.
        for r in &mut self.repaths {
            *r /= 2;
            if *r < 3 {
                *r = 0;
            }
        }

        // `GameDaemon::process_all` → `process_victory`, its second act:
        // the territory timers (`crate::victory`). It draws nothing.
        self.process_victory_territory();

        // `GameDaemon::process_all` → `calc_danger`, the third thing that
        // function does: the danger map, rebuilt from scratch every two
        // hundredth frame (`crate::danger`). It draws nothing, so it takes
        // no mark.
        if frame % crate::danger::PERIOD == 0 {
            self.calc_danger();
        }

        // `GameDaemon::process_all` → `update_all_seen`, the fourth thing
        // that function does and the one before `calc_markets`
        // (`docs/VISION.md` §6). It draws nothing, so it takes no mark.
        if frame % 100 == 0x21 {
            self.update_all_seen();
        }

        self.mark("markets");
        // `GameDaemon::process_all` → `calc_markets`: the market's price
        // cycle, between the AI and the objects (`docs/SYNC.md` §3.1). On
        // frame 0 it is eighteen draws, the frame's 2nd to 19th.
        self.calc_markets(frame);

        // `GameDaemon::process_all`'s region pass, between `calc_markets`
        // and `check_borders`: a region a gather building closed in last
        // frame's objects or this frame's AI turn is re-walked by the camps
        // there in this frame's objects (`docs/ECONOMY.md` §17.2). It draws
        // nothing, so it takes no mark.
        self.world.cycle_gather_flags();

        // `GameDaemon::process_all` → `check_borders`, after the market and
        // before the pool: a border fix since the last one zeroes every
        // leader's `reg_known_rares` (`docs/AI.md` §76), and the fixed
        // pass writes its 256 cells (`border_pass`, item 1106).
        self.check_borders();

        // `GameDaemon::process_all` → `Groups::process`, its last act: one
        // pool slot per player has its prune and its speed reset
        // (`docs/GROUPS.md` §19). It draws nothing, so it takes no mark.
        self.groups_process(frame);

        self.mark("armies");
        // `Armies::process_all` — after the daemon, before the objects
        // (`Game::do_frame` line 272; `docs/ARMY.md` §5).
        self.armies_process_all();

        self.mark("unit-loop");
        // `Objects::process_all` rotates the owners: slot `(frame + i) % 10`
        // goes `i`-th, so player `frame % 10`'s units run first this frame
        // and, within an owner, in **object** order (`docs/SYNC.md` §3.2).
        // The sync stream sees the rotation — at frame 101 the AI's farmers
        // draw their re-targets before the human's (§4.1) — so the order of
        // the visits is the order of the draws. An owner outside the ten
        // slots (none today) would go last, in index order.
        //
        // **The bound is re-read every iteration, and that is observable.**
        // `Objects::process_all@0065dce0`'s inner loop tests
        // `o < unit_mark[who]` at the bottom out of the array rather than
        // out of a local, so a unit created *inside* the loop with an
        // object number above the one being walked is processed on the
        // frame it is born. The transport barge is exactly that unit: it is
        // cast by `Unit::do_cast` from a unit whose `o` is lower, takes the
        // caster's move order and steps — which sets its guy's animation to
        // the walk before `Objects::inc_time` reaches it, so its clock
        // never wraps. Every other newborn in run57 is a **trained** unit,
        // born in `Build::do_queue` in the second loop, and every one of
        // them does wrap on its birth frame (`docs/TRANSPORT.md` §13).
        let mut slots: Vec<std::collections::BTreeMap<i16, usize>> =
            vec![std::collections::BTreeMap::new(); 10];
        // `unit_mark[who]`, and the highest slot actually filled — a test
        // that seats a unit by hand does not go through [`Sim::find_free`],
        // so the mark alone would leave it unwalked.
        let mut bound = [0i16; 10];
        for (w, b) in bound.iter_mut().enumerate() {
            *b = self.marks[w].unit;
        }
        // A dead slot's occupant: `process_all`'s other arm, `hold_frames`
        // down by one (`docs/COMBAT.md` §59). The latest unit numbered `o`
        // is the one the slot holds.
        let mut dead: Vec<std::collections::BTreeMap<i16, usize>> =
            vec![std::collections::BTreeMap::new(); 10];
        for i in 0..self.units.len() {
            let u = &self.units[i];
            if u.owner < 10 {
                let w = u.owner as usize;
                bound[w] = bound[w].max(u.index.saturating_add(1));
                if u.alive() {
                    slots[w].insert(u.index, i);
                    dead[w].remove(&u.index);
                } else if !slots[w].contains_key(&u.index) {
                    dead[w].insert(u.index, i);
                }
            }
        }
        let mut seen = self.units.len();
        for slot in 0..10 {
            let who = u8::try_from((frame + slot).rem_euclid(10)).expect("a slot");
            let w = who as usize;
            let mut o: i16 = 0;
            while o < bound[w] {
                if let Some(&i) = slots[w].get(&o) {
                    // Killed this frame before its visit: the original reads
                    // the flag at the visit, so it is a dead slot now.
                    if !self.units[i].alive() && self.units[i].hold_frames != 0 {
                        self.units[i].hold_frames -= 1;
                    }
                    self.process_unit(i, frame, &mut events);
                } else if let Some(&i) = dead[w].get(&o)
                    && self.units[i].hold_frames != 0
                {
                    self.units[i].hold_frames -= 1;
                }
                // The re-read: anything the step just created joins the
                // walk, in its own owner's band and at its own slot.
                if self.units.len() > seen {
                    for j in seen..self.units.len() {
                        let u = &self.units[j];
                        if u.owner < 10 {
                            let b = u.owner as usize;
                            bound[b] = bound[b].max(u.index.saturating_add(1));
                            if u.alive() {
                                slots[b].insert(u.index, j);
                            }
                        }
                    }
                    seen = self.units.len();
                }
                bound[w] = bound[w].max(self.marks[w].unit);
                o += 1;
            }
        }
        for i in (0..self.units.len())
            .filter(|&i| self.units[i].owner >= 10)
            .collect::<Vec<_>>()
        {
            self.process_unit(i, frame, &mut events);
        }

        self.mark("buildings");
        // Then the buildings — **after** every unit, not before.
        // `Objects::process_all@0065dce0` is two loops: the units, rotated by
        // owner, and then a *second, unrotated* one over each player's
        // buildings (object numbers from 2,000) and then their walls (from
        // 3,000). `docs/SYNC.md` §3.2 has said so since it was written; the
        // tick ran the buildings first until 2026-08-30, and East Indies'
        // frame 219 is where it first showed: the original spends a farmer's
        // `Unit::do_job+0x67` re-target and *then* the frame's road search,
        // and this crate spent them the other way round.
        //
        // What the order costs is a frame of latency in both directions, and
        // both are the original's: a unit a queue hands over this frame waits
        // for the next one to move, and the per-frame counters `Build::process`
        // clears — `helpers`, `gather_bumped` — are cleared *behind* the
        // gatherers and builders that set them rather than in front, which is
        // the same net state at the start of a frame.
        //
        // **`Build::process@0061edf0`, per building, in the original's
        // order** (twenty-fourth pass, group 18; A8 rows 20–22 and 27):
        // `Wall::process` and the launch, then the tower's attack, then
        // `Build::do_queue`, then the gather re-entries
        // (`docs/ECONOMY.md` §17.2), then the city block and the road
        // replan — so a tower sees only what the lower-numbered buildings
        // have trained, and a captured city's queue advances before the
        // re-test clears it.
        //
        // **And the walk is `Objects::process_all`'s**: each leader's
        // buildings in object-number order (`2000..build_mark`), then the
        // walls (`3000..wall_mark`), not the `Vec`'s creation order — a
        // reused low object number runs before the higher ones.
        //
        // The queue is per building (item 1449): `Build::do_queue` runs
        // inside the same building's `Build::process`, so a unit a
        // lower-numbered building trains this frame is on the map when a
        // higher-numbered one's `Wall::process` looks (run615: city `1/2008`
        // trains citizen `1/51` on 7962 and the wonder `1/2022`'s recruiter
        // sends it straight to the site).
        let mut trained = Vec::new();
        let at_start = self.buildings.len();
        let mut order: Vec<usize> = (0..at_start).collect();
        order.sort_by_key(|&b| {
            let bd = &self.buildings[b];
            (bd.index >= WALL_BASE, bd.owner, bd.index)
        });
        // A building created mid-pass joins the walk after the rest, as the
        // `Vec` walk always took it.
        let mut known = at_start;
        let mut at = 0;
        while at < order.len() {
            let b = order[at];
            at += 1;
            self.buildings[b].gather_bumped = false;
            self.log_building_step(b, "head");
            // The head's answer is `Wall::process`'s `is_active` gate: a
            // site never reaches the tower or the tail (the roads' replan
            // is below it, `crate::roads` §1), though its queue is asked.
            let live = self.process_building(b, frame);
            if live {
                self.log_building_step(b, "tower");
                self.process_building_combat(b, frame);
                self.patriot_repair(b, frame);
            }
            self.log_building_step(b, "queue");
            self.process_queue(b, &mut trained);
            self.log_building_step(b, "gather");
            self.gather_region_building(b);
            if live {
                self.kremlin_spy(b, frame);
                self.gov_hero_respawn(b, frame);
                self.persian_market_caravan(b, frame);
                self.terra_cotta_army(b);
                self.log_building_step(b, "tail");
                self.process_building_tail(b, frame);
            }
            if self.buildings.len() > known {
                order.extend(known..self.buildings.len());
                known = self.buildings.len();
            }
        }
        // A wonder that activated in this pass takes its entry in its
        // leader's wonder list, as `Build::activate` does
        // (`Wonders::init_wonder`, `docs/AI.md` §75).
        self.note_wonders();

        self.mark("gaia");
        // The tail of `Objects::process_all`: the birds' sampling every 32
        // frames and one herd's walk every 64 (`gaia.rs`).
        self.process_gaia(frame);
        // `Objects::inc_time`'s first statement is `Nuke::do_damage`
        // (`crate::nuke`, item 1091).
        self.nuke_do_damage(frame);
        self.mark("guys_inc_time");
        // `Objects::inc_time`: every guy's animation clock first — leader
        // order, no rotation (`anim.rs`) — then the ammo list, then the
        // farms' crop cells (`farms.rs`), and the sites' hit points
        // refreshed from the progress the builders just made,
        // `Wall::inc_time`.
        self.guys_inc_time();
        self.mark("projectiles");
        // `Ammo::inc_time`'s own per-frame bump of a **dead** shooter's
        // `hold_frames` (`docs/COMBAT.md` §42.3), then the walk, then
        // `DeathObj::inc_time`'s bump: the death list runs after the ammo
        // list, so a death a round makes is bumped on its own frame (item
        // 3873, `docs/GOLDEN.md` §156.1 — a packet sees the order).
        self.hold_frames_tick();
        self.process_projectiles(frame);
        self.death_objs_inc_time();
        self.mark("farms");
        self.farms_inc_time();
        self.mark("end");
        self.refresh_site_hits();
        self.frame += 1;
        // `Game::do_frame`, after the increment: the stray-road sweep
        // (`crate::mesh`, `docs/ROADS.md` §10).
        self.scan_and_kill_stray_roads();
        events
    }

    /// The building walk's step log, for the test that holds `Build::process`'s
    /// sequence (group 18); nothing outside a test.
    #[cfg(test)]
    pub(crate) fn log_building_step(&mut self, b: usize, step: &'static str) {
        self.building_log.push((b, step));
    }

    #[cfg(not(test))]
    pub(crate) fn log_building_step(&mut self, _b: usize, _step: &'static str) {}
}
