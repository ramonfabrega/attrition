//! The harness itself: one frame compared, a whole log run, and the traced runs.

mod checkpoint;
pub use checkpoint::ReplaySession;

use super::*;

/// Compares one logged frame against the simulation as it stands.
pub fn compare(built: &Built, frame: &Frame, players: usize) -> FrameResult {
    let mut r = FrameResult {
        frame: frame.n,
        ..FrameResult::default()
    };
    // The order list and the path stack are written only at `UNITS=3`. Below
    // it every unit reads back with an empty list, and comparing would say
    // the simulation had invented every order it holds — so the whole check
    // is gated on the frame carrying an order somewhere. The blind spot that
    // leaves is a `UNITS=3` frame in which *no* unit holds an order, which
    // then goes uncompared; it corrects itself on the next frame that does.
    let orders_logged = frame.units.iter().any(|u| !u.orders.is_empty());
    for u in &frame.units {
        if !(0..players as i64).contains(&u.who) {
            continue;
        }
        // A unit the dump started with is in the link table; one trained
        // since is found by its number — `find_free` hands out the same
        // per-player `o` the original did, which is what makes a trained
        // unit comparable at all.
        let trained;
        let link = match built.units.iter().find(|l| l.who == u.who && l.o == u.o) {
            Some(l) => l,
            None => {
                let found = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o));
                match found {
                    Some(unit) => {
                        trained = UnitLink {
                            who: u.who,
                            o: u.o,
                            unit,
                            kind: built.sim.units[unit].ty,
                        };
                        &trained
                    }
                    None => {
                        r.unlinked += 1;
                        r.unlinked_units.push((u.who, u.o));
                        continue;
                    }
                }
            }
        };
        let ours = built.sim.units[link.unit].pos;
        let theirs = pos_of(u.pos);
        r.compared += 1;
        // `ObjectData::mylos`, which the object record writes at every
        // detail level — `docs/VISION.md` §2. A garrisoned unit is skipped:
        // `update_los` is not run for one, so the field is whatever it held
        // when the unit was last on the map.
        if let Some(theirs_los) = u.mylos
            && built.sim.units[link.unit].on_map
        {
            r.los_compared += 1;
            let ours_los = built.sim.unit_los(link.unit);
            if i64::from(ours_los) != theirs_los {
                r.los_diverged.push(LosDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_los,
                    theirs: theirs_los,
                });
            }
        }
        // `unit_masks & 0x80000`, the **packed** bit, beside it. It is the
        // one bit of that word this crate keeps, and it is the whole state
        // the pack/unpack crafts change (`docs/ORDERS.md` §6.9): a type
        // that packs is born with it, `cast_unpack` clears it, and
        // `think_fish`'s cadence gate and `unit_los`'s clamp both read it.
        if let Some(masks) = u.unit_masks {
            r.packed_compared += 1;
            let ours_packed = built.sim.units[link.unit].combat.packed;
            if ours_packed != (masks & 0x8_0000 != 0) {
                r.packed_diverged.push(PackedDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_packed,
                });
            }
        }
        // **The collision block**, field for field
        // (`docs/COLLISION.md` §8). `UnitData::log_data` writes all five at
        // every detail level, so this is checked on every capture — and
        // the whole record is compared, not the field the mechanic happens
        // to care about. Only on unit-frames whose *positions* agree: a
        // unit that has walked somewhere else collides with different
        // things as a consequence, and counting that would measure the
        // position gap twice.
        if ours == theirs && built.sim.units[link.unit].on_map {
            let un = &built.sim.units[link.unit];
            for (field, mine, logged) in [
                ("collide", i64::from(un.collide), u.collide),
                ("collide_o", i64::from(un.collide_o), u.collide_o),
                ("collide_who", i64::from(un.collide_who), u.collide_who),
                ("collide_guy", i64::from(un.collide_guy), u.collide_guy),
                ("safe", i64::from(un.safe), u.safe),
                // **`unit_masks & 0x100000`, the one-shot half step**
                // (`docs/COLLISION.md` §4.3, `docs/MOVEMENT.md`): a soft
                // collision's whole product. `detect_unit_collision` sets
                // it at the end of the nine-cell sweep and the *next*
                // frame's `move_step` spends it, so an end-of-frame dump
                // holds exactly the flag the following step will read.
                // It is printed on every capture that prints the record
                // and nothing compared it until item 267.
                (
                    "half_step",
                    i64::from(un.half_step),
                    u.unit_masks.map(|m| i64::from(m & 0x10_0000 != 0)),
                ),
            ] {
                let Some(theirs) = logged else { continue };
                r.collide_compared += 1;
                if theirs != mine {
                    r.collide_diverged.push(CollideDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field,
                        ours: mine,
                        theirs,
                    });
                }
            }
            // `collide_frame` starts at −1 in the original and at 0 here,
            // so it is compared only once a collision has actually
            // happened on both sides.
            if let Some(theirs) = u.collide_frame
                && theirs >= 0
                && un.collide_frame > 0
            {
                r.collide_compared += 1;
                if theirs != un.collide_frame {
                    r.collide_diverged.push(CollideDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field: "collide_frame",
                        ours: un.collide_frame,
                        theirs,
                    });
                }
            }
        }
        // **`UnitData::start_dist` (`+0x130`)** — the dumped witness that
        // a 48-grid search suspended on this unit, and the one field of
        // `astar_path`'s hand-over that outlives the hand-over
        // (`docs/PATHFINDER.md` §18.6). It is written by the suspend block
        // and by nothing else that writes a value, and cleared only at a
        // unit's birth, so it is a permanent stamp exactly the way
        // `collide_frame` is — and unlike `collide_frame` it is compared
        // in both directions from zero, because a stamp this crate does
        // *not* have is the interesting half: it says the original gave up
        // on a search here and this crate did not.
        //
        // Gated on the positions agreeing for the same reason the
        // collision block is: a unit that has walked somewhere else
        // suspends on different searches as a consequence, and counting
        // that would measure the position gap twice.
        if ours == theirs
            && built.sim.units[link.unit].on_map
            && let Some(theirs_dist) = u.start_dist
        {
            r.search_compared += 1;
            let ours_dist = i64::from(built.sim.units[link.unit].start_dist);
            if ours_dist != theirs_dist {
                r.search_diverged.push(SearchDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_dist,
                    theirs: theirs_dist,
                });
            }
        }
        // The two angles, each against its own field — **on the unit-frames
        // where the two sides still agree on the position**. A unit that has
        // walked somewhere else is facing somewhere else as a consequence,
        // and counting that would measure the position gap twice over; what
        // is wanted here is the turn model on its own. A garrisoned unit is
        // skipped for the same reason `mylos` is: nothing turns it.
        if ours == theirs && built.sim.units[link.unit].on_map {
            let m = built.sim.units[link.unit].movement;
            let mut angle = |which: Which, ours: i32, theirs: i64| {
                r.angle_compared += 1;
                if i64::from(ours) != theirs {
                    r.angle_diverged.push(AngleDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        which,
                        ours,
                        theirs,
                    });
                }
            };
            if let Some(theirs) = u.angle {
                angle(Which::Heading, m.heading.0, theirs);
            }
            if let Some(theirs) = u.guys.first().and_then(|g| g.angle) {
                angle(Which::Facing, m.facing.0, theirs);
            }
        }
        if orders_logged {
            r.order_compared += 1;
            r.order_diverged
                .extend(compare_orders(built, link, u, frame.n));
        }
        if ours != theirs {
            r.diverged.push(Divergence {
                frame: frame.n,
                who: u.who,
                o: u.o,
                ours,
                theirs,
            });
        }
    }
    // **And the other direction.** A unit this simulation holds for a
    // player the frame does not name at all. `UnitData::log_data` writes
    // every unit of every player at any detail level, so a frame that names
    // none is a capture below the roster rather than an empty world; such a
    // frame is skipped rather than reported as an invented army.
    if frame
        .units
        .iter()
        .any(|u| (0..players as i64).contains(&u.who))
    {
        for un in built.sim.units.iter().filter(|u| u.alive()) {
            let who = i64::from(un.owner);
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let o = i64::from(un.index);
            if !frame.units.iter().any(|u| u.who == who && u.o == o) {
                r.extra_units.push((who, o));
            }
        }
    }
    // **The gather record, whole.** `BuildData::gather_from` is the one
    // input `crates/sim/src/gather.rs` surveys the slot count out of, and
    // until this the dump's copy of it went uncompared on every frame of
    // every capture — the whole list, printed for every building at
    // `BUILDS=7`, beside a `gather_down` printed at `BUILDS=1`. A camp
    // whose list is empty here and 73 tiles long there answers no
    // question the city record's `gather_slots` can, because the count is
    // *derived* from the list: the derived figure is the symptom and the
    // list is the cause (`docs/QUEUE.md` item 85).
    //
    // The two halves are compared independently, and the per-entry loop
    // only runs where the lengths agree — a list one entry short would
    // otherwise report every entry after the gap as wrong and turn one
    // fact into seventy.
    for b in &frame.builds {
        if !(0..players as i64).contains(&b.who) {
            continue;
        }
        let Some(handle) = built
            .sim
            .buildings
            .iter()
            .position(|x| i64::from(x.owner) == b.who && i64::from(x.index) == b.o)
        else {
            r.build_unlinked += 1;
            continue;
        };
        let ours = &built.sim.buildings[handle];
        // **Where it stands, and what it was made as.** The dump writes
        // `x_internal`/`y_internal` at every detail level and `orig_type`
        // from `BUILDS=6`; both were parsed and neither compared, so an AI
        // that chose a different *site* for the same object number linked
        // cleanly and read as agreement. That is what East Indies' word at
        // 3021 turned out to be.
        // **And the construction clock** (item 261). `constr_time` is
        // `Wall::update_construct_time`'s baked answer — the type's
        // `job_time × 100` through the owner's nation, wonder, **rare**
        // and tech modifiers — and `job_counter` the hundredths of a
        // frame the site has earned towards it. Both are written from
        // `BUILDS=1` and neither was compared until Great Lakes' word sat
        // at 7176 for a Tower whose clock read 100000 here and **90909**
        // there: the Tobacco rare's ten per cent, which this crate
        // collected on the original's own frame and never baked
        // (`docs/CITIES.md` §3.2).
        //
        // A finished building's clock is not state either side owns —
        // the original zeroes `job_counter` on completion and this crate
        // leaves it climbing — so the counter is compared only while
        // **both** sides still call the site unfinished. `constr_time` is
        // compared always: it is a baked constant, and a wrong one is
        // wrong whether the building has finished or not.
        let clock: &[(&'static str, i64, Option<i64>)] = &[
            ("constr_time", i64::from(ours.constr_time), b.constr_time),
            (
                "job_counter",
                i64::from(ours.job_counter),
                b.job_counter.filter(|_| !ours.active && b.flags & 4 == 0),
            ),
        ];
        for &(field, mine, theirs) in clock {
            let Some(theirs) = theirs else { continue };
            r.build_compared += 1;
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field,
                    ours: mine,
                    theirs,
                });
            }
        }
        for (field, mine, theirs) in [
            ("x_internal", i64::from(ours.pos.x), b.pos.x),
            ("y_internal", i64::from(ours.pos.y), b.pos.y),
        ] {
            r.build_compared += 1;
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field,
                    ours: mine,
                    theirs,
                });
            }
        }
        // **The production queue, whole.** `BuildQueue::log_data` writes
        // it for every building on every frame from `BUILDS=1`, and until
        // this it was compared in one test against one capture — so a
        // queue that filled a frame late, or a clock that ran a hundredth
        // slow, read as agreement on every other run the harness makes.
        // `docs/PRODUCTION.md`, "The queue record"; `docs/QUEUE.md` item
        // 87's ledger.
        if let Some(q) = b.queued {
            let mut off = |field: String, ours: i64, theirs: i64| {
                r.queue_compared += 1;
                if ours != theirs {
                    r.queue_diverged.push(QueueDivergence {
                        frame: frame.n,
                        who: b.who,
                        o: b.o,
                        field,
                        ours,
                        theirs,
                    });
                }
            };
            let mine = ours.queue.items.len() as i64;
            off("queued".into(), mine, q);
            // Only where the depths agree: a queue one entry short would
            // otherwise report every slot after the gap and turn one fact
            // into a page of them — the same rule the mining list keeps.
            if mine == q {
                for (k, theirs) in b.queue.iter().take(q as usize).enumerate() {
                    let item = &ours.queue.items[k];
                    let id = item.tech.unwrap_or_else(|| built.unit_tree[item.ty]);
                    let ty = built.type_index.get(id).copied().unwrap_or(-1);
                    off(format!("queue[{k}].type"), i64::from(ty), theirs.ty);
                    off(
                        format!("queue[{k}].job_counter"),
                        i64::from(item.job_counter),
                        theirs.job_counter,
                    );
                    for j in 0..3 {
                        off(
                            format!("queue[{k}].cost[{j}]"),
                            i64::from(item.cost[j]),
                            theirs.cost[j],
                        );
                        off(
                            format!("queue[{k}].good[{j}]"),
                            i64::from(item.good[j]),
                            theirs.good[j],
                        );
                    }
                }
            }
        }
        let mut wrong = |field, at, ours: i64, theirs| {
            r.gather_compared += 1;
            if ours != theirs {
                r.gather_diverged.push(GatherDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field,
                    at,
                    ours,
                    theirs,
                });
            }
        };
        // `BuildData::gather_down` is the head of the chain of registered
        // gatherers, by object number; this crate keeps the chain newest
        // first, so the head is the front of the vector.
        if let Some(theirs) = b.gather_down {
            let head = ours
                .gatherers
                .first()
                .map_or(-1, |&u| i64::from(built.sim.units[u].index));
            wrong("gather_down", -1, head, theirs);
        }
        // The mining list. `mining_len` is `None` below `BUILDS=7`, which
        // is what keeps a thin capture from reading as "every list empty".
        let Some(len) = b.mining_len else { continue };
        wrong("length", -1, ours.gather_from.len() as i64, len);
        if ours.gather_from.len() as i64 != len {
            continue;
        }
        for (k, theirs) in b.gather_from.iter().enumerate() {
            let mine = ours.gather_from[k];
            let k = k as i64;
            wrong("tx", k, i64::from(mine.x), theirs.0);
            wrong("ty", k, i64::from(mine.y), theirs.1);
        }
    }
    // **The `CITY` record, whole.** `CityData::log_data` runs for every
    // live city on every frame a dump writes, at every detail level, and
    // forty fields of it went uncompared on every capture until item 154 —
    // one test read four of them, at frame 1, on one run. The rate's own
    // inputs are in here (`gatherers`, `busy`, `free`, `filled`,
    // `space[3]`, `ter[6]`), and so is every stamp the capture and
    // assimilation clocks keep.
    //
    // The link is the city **building**: `CityData::o` is the centre's
    // object number, which is the same identity a `BUILDDATA` row carries,
    // so a city is matched the way a building is rather than by position.
    for c in &frame.cities {
        if !(0..players as i64).contains(&c.who) {
            continue;
        }
        let Some(ci) = built
            .sim
            .buildings
            .iter()
            .position(|b| i64::from(b.owner) == c.who && i64::from(b.index) == c.o)
            .and_then(|b| built.sim.buildings[b].city)
        else {
            r.city_unlinked += 1;
            continue;
        };
        let ours = &built.sim.cities[ci];
        let w = c.who as usize;
        // The AI's half of the record. The original keeps these on
        // `CityData` itself, where one leader's sweep writes them for its
        // own cities; this crate keeps a copy per leader, so the owner's is
        // the one the dump is showing.
        let ai = built.sim.ai[w].city_ai.get(ci).copied().unwrap_or_default();
        // `city_flags`, bit by bit rather than as a word: four of its bits
        // are the "an active TEMPLE / GRANARY / LUMBERMILL / MARKET stands
        // here" marks of §1.4, which this crate keeps on the economy's own
        // city record instead, and a whole-word compare would report every
        // frame of every game rather than the bit that moved.
        let flags = [
            ("city_flags[0x1]", 0x1, ours.alive),
            ("city_flags[0x2]", 0x2, ours.no_heal),
            ("city_flags[0x10]", 0x10, ours.capital),
            ("city_flags[0x40]", 0x40, ours.alarm),
            ("city_flags[0x100]", 0x100, ours.unassimilated),
            ("city_flags[0x2000]", 0x2000, ours.no_muster),
            ("city_flags[0x4000]", 0x4000, ours.founding_capital),
            ("city_flags[0x8000]", 0x8000, ours.was_founding_capital),
        ];
        let mut fields: Vec<(String, i64, i64)> = vec![
            ("x".into(), i64::from(ours.pos.x), c.x),
            ("y".into(), i64::from(ours.pos.y), c.y),
            ("pop".into(), i64::from(ours.pop), c.pop),
            ("race".into(), ours.race.map_or(-1, i64::from), c.race),
            (
                "city".into(),
                built
                    .sim
                    .cities_of(c.who as sim::Player)
                    .iter()
                    .position(|&i| i == ci)
                    .map_or(-1, |i| i as i64),
                c.city,
            ),
            ("attack_stamp".into(), ours.attack_stamp, c.attack_stamp),
            ("capture_stamp".into(), ours.capture_stamp, c.capture_stamp),
            (
                "assimilation_timer".into(),
                ours.assimilation_timer,
                c.assimilation_timer,
            ),
            (
                "capture_strength".into(),
                i64::from(ours.capture_strength),
                c.capture_strength,
            ),
            // **Through the region map, not raw.** The simulation numbers
            // its regions as it finds them and the dump numbers them as the
            // generator wrote them; `Built::region_map` is the translation
            // the census check already goes through, and without it every
            // city on every capture reads as one region wrong.
            (
                "reg".into(),
                ours.reg.map_or(-1, |r| {
                    built
                        .region_map
                        .iter()
                        .find(|&&(_, s)| s == r)
                        .map_or(-1, |&(d, _)| d)
                }),
                c.reg,
            ),
            (
                "was_capital_flags".into(),
                ours.was_capital as i64,
                c.was_capital_flags,
            ),
            ("in_port".into(), i64::from(ai.in_port), c.in_port),
            (
                "peasant_dist".into(),
                i64::from(ai.peasant_dist),
                c.peasant_dist,
            ),
            ("free".into(), i64::from(ai.free), c.free),
            ("busy".into(), i64::from(ai.busy), c.busy),
            ("gatherers".into(), i64::from(ai.gatherers), c.gatherers),
            ("ocean".into(), i64::from(ai.ocean), c.ocean),
            ("land".into(), i64::from(ai.land), c.land),
            ("filled".into(), i64::from(ai.filled), c.filled),
            ("bordering".into(), i64::from(ai.bordering), c.bordering),
            (
                "ocean_filled".into(),
                i64::from(ai.ocean_filled),
                c.ocean_filled,
            ),
            ("dock_tile".into(), i64::from(ai.dock_tile), c.dock_tile),
        ];
        for (name, bit, mine) in flags {
            fields.push((
                name.to_string(),
                i64::from(mine),
                i64::from(c.city_flags & bit != 0),
            ));
        }
        for (k, &theirs) in c.space.iter().enumerate() {
            fields.push((format!("space[{k}]"), i64::from(ai.space[k]), theirs));
        }
        for (k, &theirs) in c.ter.iter().enumerate() {
            fields.push((format!("ter[{k}]"), i64::from(ai.ter[k]), theirs));
        }
        // **The four fields nothing here holds**, asserted against the
        // zero this crate answers with rather than dropped: `raid_stamp`
        // and `reduce_stamp` are `Leader::raid`'s and the reduce order's
        // clocks, `scouted` is the AI's "a scout has seen this city" mark
        // and `trade_val` is `City::compute_trade`'s output. None has a
        // writer in this crate, so each row is a claim that the original
        // never writes one either on the captures on disk — a claim a
        // capture can falsify, which is the point of leaving it in.
        for (name, theirs) in [
            ("raid_stamp", c.raid_stamp),
            ("reduce_stamp", c.reduce_stamp),
            ("scouted", c.scouted),
            ("trade_val", c.trade_val),
            ("vans.length", c.vans.len() as i64),
        ] {
            fields.push((name.into(), 0, theirs));
        }
        for (field, ours, theirs) in fields {
            r.city_compared += 1;
            if ours != theirs {
                r.city_diverged.push(CityDivergence {
                    frame: frame.n,
                    who: c.who,
                    o: c.o,
                    field,
                    ours,
                    theirs,
                });
            }
        }
    }
    r.scores = frame.leaders.iter().map(|l| (l.who, l.score)).collect();
    r
}

/// Builds the simulation from the log's initial state and steps it through
/// every logged frame, comparing as it goes. `limit` caps the frames.
pub fn run(loaded: &Loaded, log: &Log, tuning: Tuning, limit: Option<usize>) -> Option<Report> {
    run_with(loaded, log, tuning, limit, None)
}

/// The diff, optionally fed the recorded order stream.
///
/// Without a stream this is exactly [`run`]: the simulation stands its
/// roster up from the initial dump and then runs on the engine's own
/// start-of-game logic. With one, each frame's commands are applied
/// immediately before the tick that produces the gamelog's next frame —
/// see [`crate::input`] for the frame convention and for why the AI's units
/// can never be driven this way.
pub fn run_with(
    loaded: &Loaded,
    log: &Log,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
) -> Option<Report> {
    run_traced(loaded, log, tuning, limit, stream, &[], None)
}

/// `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>` — one unit's position, orders and
/// path over a window of frames, on **any** capture [`run_traced`] drives.
///
/// Either half of `<who>/<o>` may be `*`. `*` in the object slot prints
/// **every unit that is moving** — one whose current order is a move, or
/// which still holds a path — because "who moved on this frame" is the
/// question a one-draw turn or step divergence asks first, and answering
/// it by guessing unit numbers costs a run apiece (item 207).
///
/// The line also carries the unit's **army slot** ([`sim::Sim::army_of`]).
/// The dump's `group` is the group and not the army, so a divergence over
/// an AI order — item 249's, where the original orders a squad this crate
/// leaves standing — asks "is it even in the army?" before it asks
/// anything else, and that answer used to cost a patch.
///
/// The shape is run54's own probe (`docs/JOURNAL.md`, item 175), which was
/// welded into that one test's hand-rolled tick loop; the third capture
/// to want it graduates it here. run54's keeps its own copy because it
/// also prints the figure clocks and does not go through
/// [`run_traced`]. Costs nothing when the variable is unset, and exists
/// only in test builds.
#[cfg(test)]
pub(crate) fn debug_watch(built: &Built, frame: i64) {
    let Some((who, o, lo, hi)) = std::env::var("RON_DEBUG_UNIT").ok().and_then(|v| {
        let (u, w) = v.split_once('@')?;
        let (who, o) = u.split_once('/')?;
        let (lo, hi) = w.split_once('-')?;
        let num = |s: &str| match s.trim() {
            "*" => Some(None),
            "**" => Some(Some(i64::MIN)),
            n => n.parse::<i64>().ok().map(Some),
        };
        Some((
            num(who)?,
            num(o)?,
            lo.trim().parse::<i64>().ok()?,
            hi.trim().parse::<i64>().ok()?,
        ))
    }) else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    let matches: Vec<&sim::Unit> = built
        .sim
        .units
        .iter()
        .filter(|x| x.alive())
        .filter(|x| who.is_none_or(|w| i64::from(x.owner) == w))
        .filter(|x| match o {
            // `**` is every unit alive, standing ones included: an
            // animation clock's question is "who was *due* to wrap",
            // and a standing guy is exactly what the moving filter hides.
            Some(i64::MIN) => true,
            Some(n) => i64::from(x.index) == n,
            // The wildcard's own filter: moving, by order or by path.
            None => !x.path.is_empty() || x.orders.back().is_some_and(sim::orders::Order::is_move),
        })
        .collect();
    if matches.is_empty() {
        let (w, n) = (debug_name(who), debug_name(o));
        eprintln!("  f{frame} {w}/{n} absent");
        return;
    }
    for u in matches {
        debug_unit(built, u, frame);
    }
}

/// `RON_DEBUG_ARMIES=<lo>-<hi>` — every valid army's slot, status, target,
/// counts and membership over a window, on any capture [`run_traced`] or a
/// hand-rolled loop drives.
///
/// The third probe of this family, and the one the dump cannot supply: no
/// capture writes an `ARMYDATA` record, so an army's target, muster and
/// membership are only ever visible from this side. What the dump *does*
/// give is the unit's `group` number and the `orig_x`/`orig_y` of the
/// `GROUPATTACKTOORDER` it issues, and those are what this line is read
/// against (item 350).
#[cfg(test)]
pub(crate) fn debug_armies(built: &Built, frame: i64) {
    let Some((lo, hi)) = std::env::var("RON_DEBUG_ARMIES").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.trim().parse::<i64>().ok()?, b.trim().parse::<i64>().ok()?))
    }) else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    for (w, armies) in built.sim.armies.iter().enumerate() {
        for (slot, a) in armies.valid() {
            let members: Vec<String> = a
                .units
                .iter()
                .map(|&u| format!("{}/{}", built.sim.units[u].owner, built.sim.units[u].index))
                .collect();
            eprintln!(
                "  f{frame} army {w}/{slot} status {} target {:?} pos ({},{}) muster ({},{}) \
                 rally {} hurry {} units {} caps {} std {} [{}]",
                a.status,
                a.target,
                a.pos.x,
                a.pos.y,
                a.muster.x,
                a.muster.y,
                a.rally_dist,
                a.hurry,
                a.num_units,
                a.num_captains,
                a.num_standard,
                members.join(" ")
            );
        }
    }
}

/// `RON_DEBUG_LEADER=<lo>-<hi>` — every computer leader's step machine,
/// make list and stockpile over a window, on any capture [`run_traced`] or
/// a hand-rolled loop drives.
///
/// The fourth probe of this family, and the one a *leader* divergence
/// asks for. `Leader::use_market` and `Leader::make_stuff`'s two expiry
/// walks are the only draws the production AI spends on a quiet frame,
/// and both are functions of the make list alone: the market's shortfall
/// vector is the first `epoch[Commerce]` slots' cost, and the head walk
/// draws once per slot repeating the head's type. So "how many draws did
/// this leader spend" is answered by the list, and nothing else in the
/// harness prints it.
///
/// `run97`'s `LEADERDATA` is the `LEADERS=1` detail — `who`, `tribe`,
/// `score`, `leader_flags` — so the original's own list is **not** on
/// this capture and this line is read against the *draws* rather than
/// against a dumped list. `docs/AI.md` §41 names the capture that would
/// print the other side.
#[cfg(test)]
pub(crate) fn debug_leader(built: &Built, frame: i64) {
    let Some((lo, hi)) = std::env::var("RON_DEBUG_LEADER").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.trim().parse::<i64>().ok()?, b.trim().parse::<i64>().ok()?))
    }) else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    for w in 0..built.sim.players.len() {
        if built.sim.nation[w].human || built.sim.defeated[w] {
            continue;
        }
        let l = &built.sim.ai[w];
        let name = |t: i32| -> String {
            usize::try_from(t)
                .ok()
                .and_then(|t| built.sim.tech_tree.types.get(t))
                .map_or_else(|| "-".to_string(), |r| r.name.clone())
        };
        let slots: Vec<String> = l
            .make_list
            .list
            .iter()
            .enumerate()
            .map(|(k, m)| {
                format!(
                    "{k}:t{}({}) v{} c{} cat{} n{} e{} cost{:?}",
                    m.t,
                    name(m.t),
                    m.val,
                    m.city,
                    m.cat,
                    m.num,
                    m.escrow,
                    usize::try_from(m.t)
                        .ok()
                        .filter(|&t| t < built.sim.tech_tree.types.len())
                        .and_then(|t| built.sim.type_price(w as sim::Player, t))
                        .unwrap_or([0; 6]),
                )
            })
            .collect();
        eprintln!(
            "  f{frame} L{w} step {} ({:?}) sstep {} mark {} bucket {:?} escrow {:?} \
             avail {:?} commerce {} | {}",
            l.step.number(),
            l.step,
            l.script_step,
            l.site_mark,
            built.sim.ledgers[w].bucket,
            built.sim.ledgers[w].escrow,
            built.sim.holdings[w].available,
            built.sim.tech[w].epoch[sim::tech::Line::Commerce.index()],
            slots.join(" ")
        );
    }
}

/// `RON_DEBUG_BUILDS=<lo>-<hi>` — every building's construction clock and
/// queue over a window, on any capture [`run_traced`] or a hand-rolled loop
/// drives. The dump's `BUILDDATA` prints `orig_type`, `job_counter`,
/// `constr_time` and the queue's live entries for the same frames, so the
/// two lines sit side by side without a translation step.
#[cfg(test)]
pub(crate) fn debug_builds(built: &Built, frame: i64) {
    let Some((lo, hi)) = std::env::var("RON_DEBUG_BUILDS").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.trim().parse::<i64>().ok()?, b.trim().parse::<i64>().ok()?))
    }) else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    for b in built.sim.buildings.iter().filter(|b| b.alive) {
        let ty = b.orig_ty.map_or_else(
            || "-".to_string(),
            |t| format!("{:?}", built.sim.build_types[t].ident),
        );
        let q: Vec<String> = b
            .queue
            .items
            .iter()
            .map(|i| {
                let t = i.tech.map_or_else(
                    || built.sim.unit_types[i.ty].type_index.to_string(),
                    |x| format!("tech{x}"),
                );
                format!("{t}@{}", i.job_counter)
            })
            .collect();
        eprintln!(
            "  f{frame} B {}/{} ty{ty} ({},{}) act{} jc{}/{} hits{} help{} gl{} q[{}]",
            b.owner,
            b.index,
            b.pos.x,
            b.pos.y,
            u8::from(b.active),
            b.job_counter,
            b.constr_time,
            b.construct_hits,
            b.helpers,
            b.gather_from.len(),
            q.join(" ")
        );
    }
}

/// `*` or the number, for [`debug_watch`]'s absent line.
#[cfg(test)]
pub(crate) fn debug_name(v: Option<i64>) -> String {
    v.map_or_else(|| "*".to_string(), |n| n.to_string())
}

/// One line of [`debug_watch`], for one unit.
///
/// The type index and its `packs` bit lead the line because together they
/// are the answer to "could this unit have spent that draw": `packs` is
/// one of `guy_flags & 8`'s two writers, so a packing type asks
/// `Guy::do_turn` for a turn animation whatever its art says, and a type
/// whose packet has no `CHAR_TURN_RIGHT` pays the idle roll instead
/// (`docs/ANIM.md` §4.8). On the frame Great Lakes' word parted, six units
/// were moving and exactly one packed.
#[cfg(test)]
pub(crate) fn debug_unit(built: &Built, u: &sim::Unit, frame: i64) {
    let (who, o) = (u.owner, u.index);
    // The **figure clocks** beside the position, which is run54's own
    // probe folded in here: a draw this crate spends in `guys_inc_time`
    // belongs to a guy, not to a unit, and past the last `DUMP_ALL`
    // window the clock is the only place to read the cadence from. The
    // body and its destination come with it, because `Guy::set_anim`'s
    // idle request returns without a roll while a walking guy's body has
    // not arrived (`docs/ANIM.md` §4 step 1).
    let clocks: Vec<String> = u
        .guys
        .iter()
        .map(|g| {
            let (bx, by, dx, dy) = g.follow.map_or_else(
                || {
                    (
                        u.movement.body.pos.x,
                        u.movement.body.pos.y,
                        u.pos.x,
                        u.pos.y,
                    )
                },
                |f| (f.body.pos.x, f.body.pos.y, f.des.x, f.des.y),
            );
            // `stopped` and the pair of angles are the **arrival
            // gate's own inputs** — `Guy::move`'s standing arm draws on
            // `cur_anim == 8 && stopped` and only once `angle` has
            // reached `des_angle` — so a one-draw arrival divergence is
            // read off this line rather than guessed at (item 210).
            let (fa, da) = g.follow.map_or_else(
                || (u.movement.facing.0, u.movement.heading.0),
                |f| (f.facing.0, f.des_angle.0),
            );
            let body = g.follow.map_or(u.movement.body, |f| f.body);
            format!(
                "[a{} t{}/{} l{} g{} s{} b({bx},{by})->({dx},{dy}) fa{fa}/{da} sp{}/{}]",
                g.anim,
                g.cur_time,
                g.end_time,
                g.last_time,
                g.gpiece,
                u8::from(g.stopped),
                body.last_speed,
                body.avg_speed
            )
        })
        .collect();
    eprintln!(
        "  f{frame} {who}/{o} TY {:?} PACKS {:?} army {:?} at ({}, {}) in {:?} on {} ang {} hdg {} path {:?} orders {:?} {}",
        u.ty,
        u.ty.map(|t| built.sim.unit_types[t].combat.packs),
        built
            .sim
            .units
            .iter()
            .position(|x| std::ptr::eq(x, u))
            .and_then(|i| built.sim.army_of(i)),
        u.pos.x,
        u.pos.y,
        u.inside_unit
            .map(|b| (built.sim.units[b].owner, built.sim.units[b].index)),
        u.on_map,
        u.movement.facing.0,
        u.movement.heading.0,
        u.path,
        u.orders,
        clocks.join(" ")
    );
}

/// `RON_DEBUG_SITES=<lo>-<hi>` — the frame window the site prints widen to.
/// `RON_DEBUG_SITES=1`, with no dash, is the whole-sequence switch and no
/// window, which is why this returns `None` for it.
#[cfg(test)]
pub(crate) fn site_window() -> Option<(i64, i64)> {
    site_window_named("RON_DEBUG_SITES")
}

/// [`site_window`] over any named variable: `<lo>-<hi>`, or `None`.
#[cfg(test)]
pub(crate) fn site_window_named(var: &str) -> Option<(i64, i64)> {
    let v = std::env::var(var).ok()?;
    let (lo, hi) = v.split_once('-')?;
    Some((lo.trim().parse().ok()?, hi.trim().parse().ok()?))
}

/// This frame's draw sites, each with the unit that spent it.
///
/// [`mark_sites`] keeps the site label and drops the enclosing `unit
/// who/o` mark, because that is the form the trace is compared in. But
/// *which* unit spent a draw is the question a one-draw divergence
/// almost always asks, and recovering it afterwards costs a second run;
/// the marks still hold it. Call after [`Built::tick`], which clears
/// `phase_marks` on entry, not on exit.
#[cfg(test)]
pub(crate) fn attributed_sites(built: &Built) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut who = String::from("-");
    let marks = &built.sim.phase_marks;
    for (i, (label, from)) in marks.iter().enumerate() {
        if label.starts_with("unit ") {
            who = label.clone();
        } else if !label.contains("::") {
            // A tick phase (`unit-loop`, `farms`, `end`): the unit loop is
            // over, so its last unit no longer owns what follows. Only a
            // site mark — the ones with a `Class::method` in them — stays
            // inside the unit whose turn it is.
            who = String::from("-");
        }
        let to = marks.get(i + 1).map_or(built.sim.rng.seed, |m| m.1);
        let Some(n) = draws_between(*from, to) else {
            continue;
        };
        for _ in 0..n {
            out.push((label.clone(), who.clone()));
        }
    }
    out
}

/// [`run_with`], with what this dump lacks borrowed from **siblings** —
/// other dumps of the same lobby and seed, hence the same map and the same
/// setup stream: the setup path's checksum trace (run11 has it; run9 and
/// run10 were captured before it was found — `docs/ORACLE.md`, "The setup
/// path's checksum trace is the RNG state") and the terrain's height grid
/// (only a `DUMP_ALL` dump prints it; run3 is this map's). The dump's own
/// data wins; the first sibling that has each thing supplies it.
///
/// **`trace` is a source too, and the score depends on it.** A pasture's
/// five animals are owner 9 and no dump prints them (`docs/SYNC.md` §3.6);
/// only [`crate::trace::Trace::add_animals`] has them. They roll an idle
/// every frame, so a map whose AI built a pasture and a run that borrowed
/// none are **not the same simulation** — the stream parts within a few
/// frames of the pasture going up and every later figure is on a stream
/// that is nobody's. East Indies' score sat at 167 for two days for
/// exactly that reason (2026-08-31, item 69); with the trace it is 1374,
/// which is the word's own parting. Pass the trace wherever one exists.
pub fn run_traced<'a, 'b: 'a>(
    loaded: &Loaded,
    log: &Log<'a>,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
    siblings: &[&Initial<'b>],
    trace: Option<&crate::trace::Trace>,
) -> Option<Report> {
    run_traced_observed(
        loaded,
        log,
        tuning,
        limit,
        stream,
        siblings,
        trace,
        |_, _, _| {},
    )
}

/// The same replay as `run_traced`, with a read-only observation after each
/// comparison. `Built::tick` corrections have already been applied; exporters
/// must label those inputs rather than calling this autonomous continuation.
#[expect(
    clippy::too_many_arguments,
    reason = "Existing replay inputs plus a read-only observer"
)]
pub fn run_traced_observed<'a, 'b: 'a>(
    loaded: &Loaded,
    log: &Log<'a>,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
    siblings: &[&Initial<'b>],
    trace: Option<&crate::trace::Trace>,
    mut observe: impl FnMut(&Built, &Frame, &FrameResult),
) -> Option<Report> {
    let mut init = log.replay_initial()?;
    borrow_from_siblings(&mut init, siblings);
    if let Some(tr) = trace {
        borrow_pasture(&mut init, tr);
    }
    let mut replay = Replay::new(loaded, &init, tuning);
    drop(init);
    let mut stream = stream;
    log.visit_frame_states(limit, |frame| {
        replay.step(&frame, &mut stream, &mut observe)
    });
    Some(replay.finish())
}

#[derive(Clone)]
struct Replay {
    built: Built,
    players: usize,
    report: Report,
    last: i64,
}
impl Replay {
    fn new(loaded: &Loaded, init: &Initial<'_>, tuning: Tuning) -> Self {
        let players = player_count(init);
        let mut built = build_sim(loaded, init, tuning);
        let report = Report {
            notes: std::mem::take(&mut built.notes),
            ..Report::default()
        };
        Self {
            built,
            players,
            report,
            last: 0,
        }
    }
    fn with_siblings<'a>(
        loaded: &Loaded,
        mut init: Initial<'a>,
        tuning: Tuning,
        siblings: &[&Initial<'a>],
        trace: Option<&crate::trace::Trace>,
    ) -> Self {
        borrow_from_siblings(&mut init, siblings);
        if let Some(tr) = trace {
            borrow_pasture(&mut init, tr);
        }
        Self::new(loaded, &init, tuning)
    }
    fn step(
        &mut self,
        f: &Frame,
        stream: &mut Option<&mut crate::input::Stream>,
        observe: &mut impl FnMut(&Built, &Frame, &FrameResult),
    ) {
        // `FRAME n` is the state at the end of frame n; step up to it.
        while self.last < f.n {
            // The package for recording frame `last` is processed by the
            // game frame the log then reports as `FRAME last + 1`, which is
            // the tick about to run.
            if let Some(s) = stream.as_deref_mut() {
                let did = s.apply(self.last as i32, &mut self.built);
                self.report.applied.merge(&did);
            }
            self.built.tick();
            self.last += 1;
            #[cfg(test)]
            debug_watch(&self.built, self.last);
        }
        let result = compare(&self.built, f, self.players);
        observe(&self.built, f, &result);
        self.report.frames.push(result);
    }
    fn finish(mut self) -> Report {
        self.report.notes.append(&mut self.built.notes);
        self.report.rng_frames = self.built.rng_frames.clone();
        self.report.first_divergence = (0..self.players as i64)
            .map(|who| {
                let first = self
                    .report
                    .frames
                    .iter()
                    .find(|f| f.diverged.iter().any(|d| d.who == who))
                    .map(|f| f.frame);
                (who, first)
            })
            .collect();
        self.report
    }
}

/// The same replay state machine over a finalized indexed file. Only setup,
/// correction observations, the current frame and the report stay resident.
#[expect(
    clippy::too_many_arguments,
    reason = "Same inputs as the existing replay observer"
)]
pub fn run_indexed_observed(
    loaded: &Loaded,
    source: &mut crate::capture::indexed::IndexedCapture,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
    siblings: &[&Initial<'_>],
    trace: Option<&crate::trace::Trace>,
    mut observe: impl FnMut(&Built, &Frame, &FrameResult),
) -> std::io::Result<Report> {
    let mut replay = source
        .with_replay_initial(|init| Replay::with_siblings(loaded, init, tuning, siblings, trace))?;
    let mut stream = stream;
    for frame in source.frame_states().take(limit.unwrap_or(usize::MAX)) {
        replay.step(&frame?, &mut stream, &mut observe);
    }
    source.validate()?;
    Ok(replay.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;

    use crate::testenv::{dump, install};

    #[test]
    fn indexed_replay_preserves_reports_with_sibling_corrections() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sibling)) = (
            dump("gamelog-run6-ancient-nubian-builds7.txt"),
            dump("gamelog-run13-window-95-105.txt"),
        ) else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let sibling_text = std::fs::read_to_string(sibling).unwrap();
        let sibling_log = Log::parse(&sibling_text);
        let initial = sibling_log.initial().unwrap();
        for limit in [0, 12] {
            let expected = run_traced(
                &loaded,
                &log,
                Tuning::RON,
                Some(limit),
                None,
                &[&initial],
                None,
            )
            .unwrap();
            let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
            let mut observed = Vec::new();
            let actual = run_indexed_observed(
                &loaded,
                &mut source,
                Tuning::RON,
                Some(limit),
                None,
                &[&initial],
                None,
                |_, _, r| observed.push(r.clone()),
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(observed, actual.frames);
        }
    }

    #[test]
    fn observing_a_replay_preserves_the_complete_report() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run6-ancient-nubian-builds7.txt") else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let log = Log::parse(&text);
        let mut complete_setup = log.initial().unwrap();
        assert!(
            !complete_setup.frame_bodies.is_empty(),
            "fixture must exercise the audit series"
        );
        complete_setup.frame_bodies.clear();
        assert_eq!(log.replay_initial().unwrap(), complete_setup);
        drop(complete_setup);
        let baseline = run_traced(&loaded, &log, Tuning::RON, Some(8), None, &[], None).unwrap();
        let mut observed = Vec::new();
        let report = run_traced_observed(
            &loaded,
            &log,
            Tuning::RON,
            Some(8),
            None,
            &[],
            None,
            |built, frame, result| {
                assert!(!built.sim.units.is_empty());
                assert_eq!(frame.n, result.frame);
                observed.push(result.clone());
            },
        )
        .unwrap();
        assert_eq!(observed.len(), 8);
        assert_eq!(observed, report.frames);
        assert_eq!(report, baseline);
    }

    /// **run57, the thousand frames past run56** — the same game at the
    /// same detail carried to 4,000, asked for both position records at
    /// once because the sim run is what costs.
    ///
    /// It is a drop-in longer run56 and the tools said so before it was
    /// read: `rngcmp.py` against run54 is 4,001 frames with none
    /// differing, `samegame.py` against run56 is 3,000 in common with none
    /// differing (`docs/RUNS.md`, "run57"). So it inherits run38's
    /// siblings, and what it adds is the only independent evidence anyone
    /// has about frames 3,000–4,000 of East Indies.
    ///
    /// It was captured for a divergence at 3021 that a widening of run56
    /// answered instead, and it has since named and then closed two more
    /// items. Its own statement was that everything past 3,000 was one
    /// thing's consequence — seventeen units first parting from 2978 —
    /// which the dock's slide (`docs/AI.md` §21) and the swarm ring's
    /// terrain test (`docs/ORDERS.md` §10) settled; what was left after
    /// that was two of the AI's farms sited in `x` alone, and those were
    /// the FARM/MINE arm's own distance (`docs/AI.md` §22).
    ///
    /// **Nothing is wrong in the collision record, and nothing is wrong in
    /// the building record up to the word.** Four of the capture's units
    /// ever leave the original's point at all, the earliest at 3647 —
    /// which is *before* the word, and not a contradiction: a unit can
    /// drift without spending a draw for it, so the two numbers are pinned
    /// separately (`docs/SYNC.md` §3.23). The building half is scoped
    /// there deliberately, and item 133 is why: for one item this test
    /// asserted every one of the 4,000 frames, and the assertion was
    /// **luck past the parting**. A building the AI sites after the two
    /// streams have parted is sited from draws that are nobody's, so it
    /// stands wherever this stream puts it — `1/2012` on 3977 came back
    /// one tile north the moment the word moved 3579 → 3608 on a change
    /// that has nothing to do with it (the dock's gull). What has teeth is
    /// the half a shared stream backs; the rest is printed, and named
    /// here, rather than pinned.
    ///
    /// `builds` is structural — two fields on every linked building-frame —
    /// and stays an equality. `coll` counts *agreeing* unit-frames, so it
    /// moves with the simulation's quality and is a floor.
    ///
    /// **The rescope is marked `FABLE:` for the next ratification pass**
    /// (`docs/ORACLE.md`, run57, "what may be asserted past the parting").
    /// It is a code-changing verdict rather than a proposal, the argument
    /// against it is real — a placement defect introduced past the word now
    /// passes quietly, and the scope shrinks with every capture longer than
    /// the word — and a third way, a ratchet on the count rather than a cut
    /// to the range, was named and not costed.
    #[test]
    fn run57_s_four_thousand_frames_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run57-islands-4k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run57.log"),
        ) else {
            eprintln!("skipping: no run57 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 4_000,
            "run57's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        let mut first_build: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &build_bad {
            if !first_build.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first_build.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run57 buildings: {builds} fields compared, {} wrong on {} building(s)",
            build_bad.len(),
            first_build.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first_build {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run57 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **Every building of both players, on every frame up to the word,
        // at the original's own point.** The two residues this test was
        // written around — `1/2011` one cell east on 3177 and `1/2012` one
        // tile on 3977 — were one defect, and it was the FARM/MINE arm's
        // own distance (`docs/AI.md` §22); 3177 is inside the scope and is
        // what this still guards.
        let build_early: Vec<&BuildDivergence> = build_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            build_early.is_empty(),
            "the AI's buildings stand where the original's do up to the word \
             ({LONG_WORD_EAST_INDIES}): {build_early:?}"
        );
        // **Not one collision field wrong in 349,794**, and only two of
        // the capture's units ever leave the original's point at all. The
        // block is scoped to unit-frames whose positions still agree, so
        // this is the capture's size and not a score.
        assert!(
            coll_bad.is_empty(),
            "the collision block agrees on every comparable field-frame of {coll}: {coll_bad:?}"
        );
        // **Nothing leaves the original's point inside the capture.** It
        // was 3647 and two units until the tile grid's own tolerance
        // landed (`docs/PATHFINDER.md` §7): a unit that can transport is
        // given an *exact* waypoint, and `1/13` — granted `unit_masks &
        // 0x800000` on 3580, the frame after its Dock finished — had been
        // cutting three frames off every leg since. The one parting left
        // is `0/5` on **4001**, the capture's last frame and run57's own
        // word. Before that: 3582 and eleven units until the colonist arm
        // landed (`docs/SYNC.md` §3.23), four until the shore reads did
        // (§3.25).
        assert!(
            parted.values().copied().min().is_none_or(|f| f >= 4_001),
            "no unit leaves the original's point before the capture's last \
             frame: {parted:?}"
        );
        assert!(
            parted.len() <= 1,
            "one unit ever leaves the original's point in 4,000 frames: {parted:?}"
        );
        assert_eq!(
            builds, 197_932,
            "the site and the clock on every linked building-frame"
        );
        assert!(
            coll >= 350_928,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
    }

    /// **run58 — East Indies at 5,200 frames, the successor sized past the
    /// word** (2026-09-01, item 140). run57 stops at 4,001 and East
    /// Indies' long word had reached 4020, so the standing rule owed a
    /// longer one: run39's recipe unchanged, `MAP_STYLE 18`, seed 12345,
    /// no input, carried to 5,200. Forty-eight minutes and 1.41 GB.
    ///
    /// **It is the same game twice over, and the tools said so before it
    /// was read**: `rngcmp.py` against run54 is 5,201 frames with **zero**
    /// differing, and `samegame.py` against run57 is 4,000 frames in
    /// common with **zero** differing. So it inherits run39's siblings and
    /// run54's word, and it is a drop-in longer run57 — which is why the
    /// two tests stand side by side rather than one replacing the other:
    /// run57's is the shorter, tighter floor and this one is the reach.
    ///
    /// The item it was booked for was answered without it, by the trace
    /// and the decompile while it ran (`docs/SYNC.md` §3.25). What it is
    /// *for* is item 141: the frames the word now parts on, 4275 and 4288,
    /// are past every other dump on disk.
    #[test]
    fn run58_s_five_thousand_frames_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run58-islands-5k2.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run58.log"),
        ) else {
            eprintln!("skipping: no run58 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 5_000,
            "run58's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        let mut first_build: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &build_bad {
            if !first_build.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first_build.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run58 buildings: {builds} fields compared, {} wrong on {} building(s)",
            build_bad.len(),
            first_build.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first_build {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }

        // The production queues, whole — every building, every live slot.
        let queues: usize = report.frames.iter().map(|f| f.queue_compared).sum();
        let queue_bad: Vec<&QueueDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.queue_diverged.iter())
            .collect();
        eprintln!(
            "run58 queues: {queues} fields compared, {} wrong",
            queue_bad.len()
        );
        for d in queue_bad.iter().take(24) {
            eprintln!(
                "  f{} {}/{} {}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }

        // **The `CITY` record, whole** — item 154(a), and the widest
        // widening the ledger has taken: forty fields on every live city of
        // every frame, where one test read four of them at frame 1
        // (`docs/CITIES.md` §5.7).
        let cities: usize = report.frames.iter().map(|f| f.city_compared).sum();
        let unlinked: usize = report.frames.iter().map(|f| f.city_unlinked).sum();
        let city_bad: Vec<&CityDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.city_diverged.iter())
            .collect();
        // One row per `(who, o, field)`, in the order each first parted,
        // with the frame it parted on and how many frames it stayed parted.
        let mut city_rows: Vec<(i64, i64, &str, i64, usize)> = Vec::new();
        for d in &city_bad {
            if let Some(row) = city_rows
                .iter_mut()
                .find(|r| (r.0, r.1, r.2) == (d.who, d.o, d.field.as_str()))
            {
                row.4 += 1;
            } else {
                city_rows.push((d.who, d.o, &d.field, d.frame, 1));
            }
        }
        let city_said: Vec<String> = city_rows
            .iter()
            .map(|&(who, o, field, frame, n)| format!("{who}/{o} {field} f{frame} x{n}"))
            .collect();
        eprintln!(
            "run58 cities: {cities} fields compared, {} wrong, {unlinked} unlinked",
            city_bad.len()
        );
        for r in &city_said {
            eprintln!("  {r}");
        }
        assert!(
            cities >= 600_000,
            "the city record is being read: {cities} fields"
        );
        assert_eq!(unlinked, 0, "every city links to a building of ours");
        // **What is left, pinned as it stands** rather than filtered out,
        // so a change that moves any of it fails rather than passing
        // quietly (the `gather_slots` precedent). Three open seams, and
        // nothing else in the record parts on any of the 5,201 frames —
        // `ter[6]` included, on both of the AI's cities, since
        // `World::gather_at` landed (`docs/AI.md` §24):
        //
        // 1. **`who 0`, the human's city — thirteen fields, every frame.**
        //    `Leaders::strategy_all@006ed430`'s gate is `leader_flags & 3
        //    == 3`: in play and not defeated, with **no** test for a human,
        //    and `Leader::production_ai@006c1960` is where a human without
        //    computer assist bails — to the switch's `default`, which
        //    clears the step machine, so the sweep re-arms and runs again on
        //    the leader's next phase frame. This crate skips the human in
        //    `Sim::strategy_all` instead, so its census never runs and the
        //    whole site picture stays zero. The dump says the same as the
        //    decompile: the human's `peasant_dist` moves on frame 401, and
        //    leader 0's phase is `frame % 200 == 0` (`docs/AI.md` §2.2).
        // 2. **The AI's second city, `1/2007`** — `land`, `filled` and the
        //    three `space` counts, one apart, from the frame its circle is
        //    first swept.
        // 3. **A gatherer filed under the wrong city** from 2576, and a
        //    free citizen from 4176 — the totals agree, the attribution
        //    does not.
        let city_want = vec![
            "0/2000 peasant_dist f1 x5201",
            "0/2000 busy f1 x5201",
            "0/2000 gatherers f1 x5201",
            "0/2000 ocean f1 x5201",
            "0/2000 land f1 x5201",
            "0/2000 filled f1 x5201",
            "0/2000 dock_tile f1 x5201",
            "0/2000 space[0] f1 x5201",
            "0/2000 space[1] f1 x5201",
            "0/2000 space[2] f1 x5201",
            "0/2000 ter[0] f1 x5201",
            "0/2000 ter[1] f1 x5201",
            "0/2000 ter[3] f1 x5201",
            "1/2007 land f1819 x157",
            "1/2007 filled f1819 x3383",
            "1/2007 space[0] f1976 x3226",
            "1/2007 space[1] f1976 x3226",
            "1/2007 space[2] f1976 x3226",
            "1/2000 gatherers f2576 x800",
            "1/2007 peasant_dist f2576 x800",
            "1/2007 gatherers f2576 x800",
            "1/2007 free f4176 x200",
        ];
        assert_eq!(
            city_said, city_want,
            "the `CITY` record's open rows moved; every other field of every \
             city of every frame agrees"
        );

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run58 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **Everything this asserts is scoped to before the word**, and
        // run58 is the first capture long enough for that to matter. Past
        // 4275 the two streams are running on draws that are nobody's, so
        // a unit standing somewhere else there is not a defect — nineteen
        // of them do, all first parting between 4300 and 5085, and the
        // list is printed above rather than pinned. Before the word the
        // capture is evidence, and there it is exact.
        let build_early: Vec<&BuildDivergence> = build_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            build_early.is_empty(),
            "the AI's buildings stand where the original's do up to the word \
             ({LONG_WORD_EAST_INDIES}): {build_early:?}"
        );
        let coll_early: Vec<&CollideDivergence> = coll_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            coll_early.is_empty(),
            "the collision block agrees on every comparable field-frame up \
             to the word: {coll_early:?}"
        );
        // **And the production queues, whole** — the widening that found
        // the word itself. Before it, the queue record was compared in
        // `run39_s_build_queues_are_the_original_s_clock` alone, on a
        // capture 2,600 frames too short to reach the AI's first ship; the
        // twenty-eight frames of `queued ours 1 theirs 0` at `1/2010` were
        // sitting in this dump the whole time (`docs/PRODUCTION.md`, "The
        // tail's first caller").
        //
        // **And the word has now outrun this capture.** run54's word is
        // 5376 and run58 is 5,201 frames, so "before the word" is the
        // whole file.
        //
        // The tail used to be twenty-four frames of `1/2005 queued ours 0
        // theirs 1` — the AI's library holding a **Coinage** job from
        // frame 5177 that this crate never started. It was
        // `economy::Holdings::available`, all-true from frame 0 since it
        // existed: knowledge is not available before the Classical Age, so
        // the original charges Coinage's `14k` as two hundred and ten food
        // and this crate asked for a hundred and forty knowledge nobody
        // can hold (`docs/COSTS.md`, "Three of the six resources are not
        // available from the start"; `docs/ECONOMY.md`, "Availability, and
        // where a price lands instead"). Those rows are gone.
        //
        // What is left is the capture's **last frame and nothing else**.
        // On 5201 the original prints `queued 0` for both buildings that
        // held a job on 5200 — `1/2005`'s research and `1/2010`'s
        // Fisherman — and prints the whole `BUILDDATA` list a second time,
        // truncated; the run was quitting. Frames 5195..=5200 are stable
        // and agree. The two rows are asserted **as they stand** rather
        // than filtered out, so a change that moves them fails rather than
        // passing quietly (the `gather_slots` precedent,
        // `run40_s_census_…`).
        let queue_early: Vec<&QueueDivergence> = queue_bad
            .iter()
            .filter(|d| d.frame < RUN58_QUEUE_TAIL)
            .copied()
            .collect();
        assert!(
            queue_early.is_empty(),
            "the AI's queues run the original's clock up to \
             {RUN58_QUEUE_TAIL}: {queue_early:?}"
        );
        let queue_tail: Vec<(i64, i64, &str, i64, i64)> = queue_bad
            .iter()
            .filter(|d| d.frame >= RUN58_QUEUE_TAIL && d.frame < LONG_WORD_EAST_INDIES)
            .map(|d| (d.frame, d.o, d.field.as_str(), d.ours, d.theirs))
            .collect();
        let want: Vec<(i64, i64, &str, i64, i64)> =
            vec![(5201, 2005, "queued", 1, 0), (5201, 2010, "queued", 1, 0)];
        assert_eq!(
            queue_tail, want,
            "run58's queues are the original's on every frame it dumps \
             whole; only 5201, the truncated last one, disagrees"
        );
        assert!(
            queues >= 109_435,
            "the queue record is being read: {queues} fields"
        );
        // **The path stack's own rows, before the word.** run39's
        // widening (`run39_s_path_stack_agrees_row_for_row`) scores
        // `tolerance` and `flags` beside each waypoint's point, and it
        // passed for a week: run39 is 1,850 frames and the tile grid's
        // tolerance only stops being `0x60` for a unit that can transport
        // — which on East Indies is frame **3580**, past the end of that
        // capture. So the check was right and the capture was short. Here
        // it is on the long one, where a `tolerance` row is what the
        // thirteen-frame lead of item 141 actually was.
        //
        // **And the two AI Fishermen are inside it now.** Their sea
        // routes were the one exception this check carried: `1/14` from
        // 4464 and `1/16` from 4870 planned the same nineteen cells from
        // (57, 55) to the fish at (49, 39) staircased two cells north of
        // the original's, because the world grid's same-region test was
        // asking `get_tregion` where the original reads `WData.region`
        // (`docs/PATHFINDER.md` §16). With the raw field both stacks agree
        // row for row on the frame each is planned, so there is no
        // partition and no pin — every waypoint of every unit is asserted.
        let path_rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| {
                d.frame < LONG_WORD_EAST_INDIES
                    && matches!(
                        d.what,
                        OrderMismatch::PathField { .. } | OrderMismatch::PathTo { .. }
                    )
            })
            .collect();
        assert!(
            path_rows.is_empty(),
            "every waypoint agrees row for row up to the word: {} rows,              first {:?}",
            path_rows.len(),
            path_rows.first()
        );
        // **The packed bit and the line of sight, whole and everywhere.**
        // Both are written at every detail level and both are what the
        // fishing boat's deploy moves: `unit_masks 786440 → 262152` and
        // `mylos 4 → 6` on the same frame, 4989. The AI's two Fishermen
        // are the only units on this capture that carry the bit at all,
        // and before item 149 neither ever lost it — the deploy died the
        // frame after it was queued, so the pair here is the item's own
        // oracle (`docs/ORDERS.md` §6.9).
        let packed_seen: usize = report.frames.iter().map(|f| f.packed_compared).sum();
        let packed_bad: Vec<PackedDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.packed_diverged.iter().copied())
            .collect();
        let los_seen: usize = report.frames.iter().map(|f| f.los_compared).sum();
        let los_bad: Vec<LosDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.los_diverged.iter().copied())
            .collect();
        eprintln!(
            "run58 packed: {packed_seen} unit-frames, {} wrong; \
             los: {los_seen} unit-frames, {} wrong",
            packed_bad.len(),
            los_bad.len()
        );
        for d in packed_bad.iter().take(4) {
            eprintln!("  packed f{} {}/{}: ours {}", d.frame, d.who, d.o, d.ours);
        }
        for d in los_bad.iter().take(4) {
            eprintln!(
                "  los f{} {}/{}: ours {} theirs {}",
                d.frame, d.who, d.o, d.ours, d.theirs
            );
        }
        // **The packed bit is asserted over the whole capture, not up to
        // the word**, and deliberately: the boat deploys on 4989, one
        // frame *past* it, so an assertion fenced to the word could not
        // see the thing the item is about. It is exact on all 94,935
        // unit-frames, and it is what fails the minute `do_cast` stops
        // waiting out the forty.
        assert!(
            packed_bad.is_empty(),
            "the packed bit is the original's on every unit-frame: {:?}",
            packed_bad.first()
        );
        // …and `mylos` has exactly the one disagreement run39 pins on its
        // own 1,850 frames — the AI scout's Science level, which this
        // crate's pure function reports on the frame the level lands and
        // the original's cached field reports one level later (item 35,
        // `docs/VISION.md` §7). Over 94,338 unit-frames here it is still
        // the only one, and the AI's two Fishermen — the only units on
        // this capture that are ever packed — are not among them.
        let los_early: Vec<LosDivergence> = los_bad
            .iter()
            .copied()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert_eq!(
            los_early,
            vec![LosDivergence {
                frame: 202,
                who: 1,
                o: 0,
                ours: 6,
                theirs: 4,
            }],
            "mylos is the original's up to the word but for the cache"
        );
        assert!(
            packed_seen >= RUN58_PACKED_FRAMES && los_seen >= RUN58_PACKED_FRAMES,
            "both are read on every linked unit-frame: {packed_seen} / {los_seen}"
        );

        let early: std::collections::BTreeMap<_, _> = parted
            .iter()
            .filter(|(_, f)| **f < LONG_WORD_EAST_INDIES)
            .collect();
        assert_eq!(
            early.len(),
            RUN58_PARTED,
            "before the word, no unit leaves the original's point: {early:?}"
        );
        assert_eq!(
            builds, RUN58_BUILD_FIELDS,
            "the site and the clock on every linked building-frame"
        );
        assert!(
            coll >= RUN58_COLL_FIELDS,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
    }

    /// **run69 — Great Lakes at 3,000 frames, the capture the map had
    /// earned and nobody had taken** (2026-09-03, item 193).
    ///
    /// The standing rule (`docs/DECISIONS.md` 29): when a map's word
    /// crosses the newest full-detail capture it has, the next one is
    /// sized to the word. Great Lakes' word is run53's **2419** and its
    /// only full-detail run was run33's 1,850, so every frame of the
    /// parting fell past the end of the only dump that could show it —
    /// the same shape that owed run56 on East Indies two days earlier.
    ///
    /// run33's recipe unchanged and nothing else: `MAP_STYLE 14`, seed
    /// 12345, run10's `-config check.ini` lobby, run10's detail, no
    /// input, carried to **3,000**. Only the *length* changed, so it is a
    /// drop-in longer run33 and both same-game tools speak.
    ///
    /// **What it settles**, and it is the whole of item 193: the word
    /// parts at 2419 on one draw, a `Guy::set_anim+0x97a <
    /// Unit::do_non_flat_gather+0xb99` — the AI woodcutter `1/9`'s
    /// return-to-camp stand — which this crate spends on 2419 and the
    /// original on 2420. The clock behind it is the same on both sides:
    /// the tile choice's `+0x54b` draw on frame **1959** returns 4 on
    /// both, `400 + 4 % 200` is **404**, and 404 frames of chopping put
    /// the walk home wherever the *countdown starts*. It starts on
    /// arrival at the tile, and this crate's woodcutter arrives on 2015
    /// where the original's arrives on **2016**.
    ///
    /// The frame is a **route**, and this capture prints it. `1/9`'s
    /// `MOVEORDER` waypoints (`dest_x`/`dest_y`, live while `dest` is 1)
    /// go `(40536, 17592)`, `(40584, 17544)`, **`(40728, 17544)`**,
    /// **`(40824, 17640)`**, `(40968, 17640)`, `(41016, 17640)` here and
    /// `…, (40776, 17544), (40872, 17640), …` in the original: two middle
    /// slots, each one **48-grid step** short, on a route whose ends,
    /// total length and switch frame all agree. That is item 191's
    /// signature exactly — run68's `1/13` on East Indies parts on the
    /// same two middle slots of its own stack from block 6686 — so the
    /// lower map's seam and the higher map's frontier are one defect
    /// (`docs/PATHFINDER.md`, "The middle nodes"). The rows are asserted
    /// **as they stand** rather than filtered out, so the day the route
    /// is right this fails rather than passing quietly.
    #[test]
    fn run69_s_three_thousand_frames_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run69-greatlakes-3k.txt"),
            trace("rontrace-run69.log"),
        ) else {
            eprintln!("skipping: no run69 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        // A test-selected fidelity failure, not every known comparator residue.
        // Reuse the same loaded data, parsed log, siblings and trace; no inferred CLI.
        if let Some(index) = report.frames.iter().position(|f| {
            !f.diverged.is_empty()
                || !f.build_diverged.is_empty()
                || (f.frame < LONG_WORD_GREAT_LAKES
                    && f.order_diverged.iter().any(|d| {
                        matches!(
                            d.what,
                            OrderMismatch::Move {
                                field: "dest_x" | "dest_y",
                                ..
                            }
                        )
                    }))
        }) {
            let artifact = (|| -> Result<_, Box<dyn std::error::Error>> {
                let output =
                    crate::debug_view::failure_path("run69-position-building-or-waypoint")?;
                let meta = crate::debug_view::Metadata {
                    capture: path.clone(), source_bytes: text.len(),
                    siblings: refs.iter().enumerate().map(|(i,_)| format!("already-parsed sibling Initial {i} from testkit::sibling_texts()" )).collect(),
                    trace: Some("already-parsed testkit::trace(rontrace-run69.log)".into()),
                    reproduce: "cargo test -p rondata --release run69_s_three_thousand_frames_stand_where_the_original_s_do -- --exact diff::harness::tests::run69_s_three_thousand_frames_stand_where_the_original_s_do --nocapture".into(),
                    ..Default::default()
                };
                crate::debug_view::write_failure(
                    &report,
                    Some((
                        index,
                        "unit positions and building fields must agree throughout run69; current waypoints must agree before the word",
                    )),
                    &meta,
                    &output,
                    |window| {
                        run_traced_observed(
                            &loaded,
                            &log,
                            Tuning::RON,
                            None,
                            None,
                            &refs,
                            Some(&tr),
                            |built, frame, result| window.observe(built, frame, result),
                        )
                    },
                )?;
                Ok(output)
            })();
            match artifact {
                Ok(path) => eprintln!("differential failure artifact: {}", path.display()),
                Err(error) => eprintln!("differential artifact unavailable: {error}"),
            }
        }
        assert!(
            report.frames.len() >= 3_000,
            "run69's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        eprintln!(
            "run69 buildings: {builds} fields compared, {} wrong",
            build_bad.len()
        );
        for d in build_bad.iter().take(8) {
            eprintln!(
                "  {}/{} f{}: {} ours {} theirs {}",
                d.who, d.o, d.frame, d.field, d.ours, d.theirs
            );
        }

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run69 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **The waypoint, before the word.** `dest_x`/`dest_y` is the
        // move order's *current* step (§4.1), and it is the record item
        // 193 turned out to be. Folded to one row a unit and a field —
        // who, what, the first frame and how many — because a route that
        // parts stays parted for the rest of its walk and the count is
        // the walk's length, not a second finding.
        let mut waypoints: Vec<(i64, i64, &str, i64, usize)> = Vec::new();
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| d.frame < LONG_WORD_GREAT_LAKES)
        {
            let OrderMismatch::Move { field, .. } = d.what else {
                continue;
            };
            if field != "dest_x" && field != "dest_y" {
                continue;
            }
            match waypoints
                .iter_mut()
                .find(|r| (r.0, r.1, r.2) == (d.who, d.o, field))
            {
                Some(r) => r.4 += 1,
                None => waypoints.push((d.who, d.o, field, d.frame, 1)),
            }
        }
        eprintln!("run69 waypoint rows before the word: {waypoints:?}");

        let build_early: Vec<&BuildDivergence> = build_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_GREAT_LAKES)
            .collect();
        assert!(
            build_early.is_empty(),
            "the AI's buildings stand where the original's do up to the word \
             ({LONG_WORD_GREAT_LAKES}): {build_early:?}"
        );
        assert!(
            coll_bad.is_empty(),
            "the collision block agrees on every comparable field-frame of \
             {coll}: {coll_bad:?}"
        );
        // **Nothing leaves the original's point before the word**, and it
        // took three items to say so. A unit used to part on **1993**, and
        // that was item 191: the woodcutter `1/9`'s two middle waypoints,
        // one 48-grid step each, and the sixty-fourth-frame repaint of a
        // standing guy's collision disc behind them
        // (`docs/COLLISION.md` §2.2). Another parted on **2804** — the AI
        // citizen `1/1`, which finishes building `2010` on 2803 and, in
        // the original, walks to the next site rather than gathering at
        // the one it just built (`docs/ORDERS.md` §5.5, item 194). Six
        // more parted between **2935** and 2996, and one draw took all
        // six: the pasture animal's reference object, which this crate
        // read off the chain's length rather than off
        // `num_gatherers(1, 0)`'s arrived count (`docs/SYNC.md` §3.6,
        // item 196).
        //
        // So the list below is not merely empty before the word — **no
        // unit in this capture's 3,000 frames ever stands where the
        // original's does not**, and `parted` is empty outright. The word
        // is now 4241, past the capture's whole length, so Great Lakes is
        // owed a longer full-detail run (`docs/DECISIONS.md` 29) and this
        // one has nothing left to say about position.
        let early: Vec<(i64, i64, i64)> = parted
            .iter()
            .filter(|&(_, &f)| f < LONG_WORD_GREAT_LAKES)
            .map(|(&(w, o), &f)| (w, o, f))
            .collect();
        assert_eq!(
            early,
            Vec::new(),
            "no unit leaves the original's point before the word \
             ({LONG_WORD_GREAT_LAKES})"
        );
        // And no waypoint disagrees before the word either. `1/9`'s two
        // slots — `dest_x` from 1993 and `dest_y` from 2000 — were pinned
        // here **as they stood** so that the day the route was right this
        // would fail rather than pass quietly. It did.
        assert_eq!(
            waypoints,
            Vec::new(),
            "no move order's current waypoint parts before the word"
        );
        assert!(
            builds >= 95_476,
            "two fields on every linked building-frame: {builds}"
        );
        assert!(
            build_bad.is_empty(),
            "and the AI's buildings stand where the original's do for the \
             whole capture, not only to the word: {build_bad:?}"
        );
        assert!(
            coll >= 254_924,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
        // And the stronger form, which only became true with item 196:
        // the whole capture, not only the stretch before the word.
        assert!(
            parted.is_empty(),
            "no unit leaves the original's point in run69's 3,000 frames: \
             {parted:?}"
        );
    }

    /// **run71 — Great Lakes at 5,000 frames, the first dump that reaches
    /// past the map's word** (2026-09-03, item 197).
    ///
    /// run69 is 3,000 frames and the word is 4241, so every frame of the
    /// parting fell past the end of the only full-detail dump Great Lakes
    /// had — `docs/DECISIONS.md` 29's standing rule, one map later than
    /// run56 and run69 answered it for East Indies.
    ///
    /// run33's recipe unchanged and only the *length* changed: `MAP_STYLE
    /// 14`, seed 12345, run10's `-config check.ini` lobby, run10's detail,
    /// no input, carried to **5,000**. The capture checks itself before it
    /// is believed — `rngcmp` against run53 is 5,001 identical frames and
    /// 0 differing, and `samegame` against run69 differs on none of their
    /// 3,000 common frames — so it is a drop-in longer run69 and both
    /// same-game tools speak.
    ///
    /// What it is for is the **word's own frame**: 4241 is where run53's
    /// trace says the draws part, and until this capture nothing on disk
    /// printed that frame's units, buildings or orders at all.
    #[test]
    fn run71_s_five_thousand_frames_reach_past_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run71-greatlakes-5k.txt"),
            trace("rontrace-run71.log"),
        ) else {
            eprintln!("skipping: no run71 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let report = with_sibling_initials(|refs| {
            run_traced(&loaded, &log, Tuning::RON, None, None, refs, Some(&tr)).unwrap()
        });
        assert!(
            report.frames.len() >= 5_000,
            "run71's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        eprintln!(
            "run71 buildings: {builds} fields compared, {} wrong",
            build_bad.len()
        );
        for d in build_bad.iter().take(12) {
            eprintln!(
                "  {}/{} f{}: {} ours {} theirs {}",
                d.who, d.o, d.frame, d.field, d.ours, d.theirs
            );
        }

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run71 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        // The parting itself, at the earliest frames, with both sides.
        let mut rows: Vec<&Divergence> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .collect();
        rows.sort_by_key(|d| (d.frame, d.who, d.o));
        for d in rows.iter().take(24) {
            eprintln!(
                "  f{} {}/{}: ours ({},{}) theirs ({},{})",
                d.frame, d.who, d.o, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y
            );
        }

        // The waypoints, folded to one row a unit and a field.
        let mut waypoints: Vec<(i64, i64, &str, i64, usize)> = Vec::new();
        for d in report.frames.iter().flat_map(|f| f.order_diverged.iter()) {
            let OrderMismatch::Move { field, .. } = d.what else {
                continue;
            };
            if field != "dest_x" && field != "dest_y" {
                continue;
            }
            match waypoints
                .iter_mut()
                .find(|r| (r.0, r.1, r.2) == (d.who, d.o, field))
            {
                Some(r) => r.4 += 1,
                None => waypoints.push((d.who, d.o, field, d.frame, 1)),
            }
        }
        eprintln!("run71 waypoint rows, whole capture: {waypoints:?}");

        // **Everything run69 proved still holds**, and it has to: run71 is
        // the same game, so its first 3,000 frames are run69's.
        let early: Vec<(i64, i64, i64)> = parted
            .iter()
            .filter(|&(_, &f)| f < 3_000)
            .map(|(&(w, o), &f)| (w, o, f))
            .collect();
        assert_eq!(
            early,
            Vec::new(),
            "run69's 3,000 frames are clean, so run71's first 3,000 are too"
        );
        let build_early: Vec<&BuildDivergence> =
            build_bad.iter().filter(|d| d.frame < 3_000).collect();
        assert!(
            build_early.is_empty(),
            "and so are its buildings over the same stretch: {build_early:?}"
        );

        // **Nothing in this capture parts on position at all** (item 202,
        // 2026-09-03). It was 4177 for a session, and the two units that
        // went there — `1/11` stopping dead where the original walked
        // +7,+24, `1/19` turning +14,-20 against the original's +25,0 —
        // were **one defect**, not two. Frame 4176 is where the AI places
        // its farm `2014` and pulls a citizen off gathering to build it,
        // and the two sides pulled a **different citizen**: the original's
        // `1/11` takes the `BUILDORDER` and the walk to (41736,22584),
        // while this crate handed both to `1/19`.
        // `produce_building`'s builder loop measures the tile distance from
        // the **corner tile** to the unit's own tile, each coordinate
        // floored on its own; this crate divided the world-unit difference
        // once, and the two functions part wherever the floors do. `1/11`
        // and `1/19` tie at 27 under the original's arithmetic and the
        // earlier unit keeps the tie (`docs/AI.md` §2.20).
        //
        // It was **4827** for a day after that — `1/15` re-picking its farm
        // cell off a seed that was nobody's, because the *draw* word had
        // parted twenty-four frames earlier on a road search eleven nodes
        // short. `crate::mesh` is the eleven nodes (`docs/ROADS.md` §9),
        // and with them the whole capture is in lockstep on position.
        //
        // Pinned as "never", so that the day a unit moves again this fails
        // rather than quietly passing.
        let first_part = parted.values().copied().min().unwrap_or(i64::MAX);
        assert_eq!(
            (first_part, parted.len()),
            (i64::MAX, 0),
            "Great Lakes parts on position nowhere in run71 — a change here \
             is the score moving, and it moves the queue's Scoreboard line \
             with it: {parted:?}"
        );

        // The counts only grow; a fall here is a capture that got shorter or
        // a link that stopped being made.
        assert!(
            builds >= 180_076,
            "two fields on every linked building-frame: {builds}"
        );
        assert!(
            coll >= 513_465,
            "five fields on every agreeing unit-frame: {coll}"
        );
        assert!(
            coll_bad.is_empty(),
            "the collision block agrees on every comparable field-frame of \
             {coll}: {coll_bad:?}"
        );
        // **Every building of both players stands on the original's own
        // point for the whole 5,000 frames.** The one that used to move —
        // `1/2015`'s `y_internal`, 425 fields from 4577, 15936 here against
        // 15744 — was downstream of the builder pick after all: it came
        // right the moment the right citizen was sent, without being
        // touched.
        assert!(
            build_bad.is_empty(),
            "a building stands somewhere the original's does not: {:?}",
            &build_bad[..build_bad.len().min(4)]
        );
    }

    /// Run12's per-frame words, read straight from the dump (`docs/SYNC.md`
    /// §1): the `end_frame` record inside each `FRAME n` block's `FULL
    /// DUMP`, keyed by the engine frame.
    #[test]
    fn run12_s_end_of_frame_words_are_read() {
        let Some(path) = dump("gamelog-run12-dumpall-seeds.txt") else {
            eprintln!("skipping: no gamelog-run12-dumpall-seeds.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        assert_eq!(
            log.frame_seeds(),
            vec![
                (0, 0xb619_4ba1),
                (1, 0x4554_ec0f),
                (2, 0xab3b_035d),
                (3, 0xc242_06bb)
            ]
        );
        let init = log.initial().unwrap();
        assert_eq!(init.checksums.last().map(|c| c.seed), Some(0x3bd3_9ae9));
        assert_eq!(
            draws_between(0x3bd3_9ae9, 0xb619_4ba1),
            Some(120),
            "frame 0's draws"
        );
        assert_eq!(draws_between(0xb619_4ba1, 0x4554_ec0f), Some(54));
        assert_eq!(draws_between(0x4554_ec0f, 0xab3b_035d), Some(6));
        assert_eq!(draws_between(0xab3b_035d, 0xc242_06bb), Some(6));
    }

    /// **Run20's whole frame 0, draw for draw** — item 27, and the swap it
    /// was the instrument for.
    ///
    /// Frame 0 counted 175 against 175 from the moment the scout landed,
    /// and the count was hiding two errors that cancelled. Every mechanic
    /// the frame touches now marks its own draw sites under the original's
    /// offsets (`sim::ai_sites`, `sim::market`, `sim::anim`, `sim::scout`,
    /// `sim::farms`, `sim::gaia`), and [`crate::trace::SITES`] names the
    /// same addresses out of the trace's `ebp` chain — so the frame is one
    /// `Vec<String>` on each side and the comparison is an `assert_eq!`.
    ///
    /// **What it found on its first run, which is the whole of the swap.**
    /// The two sequences agreed for 22 draws and parted: ours spent a
    /// `set_anim` roll for each gathering citizen, at a call the original
    /// does not make. The camp-arrival stand in `do_non_flat_gather` was
    /// the sim's own — the branch is a two-way `CHAR_DUMP_WOOD` /
    /// `CHAR_DUMP_ORE`, and the listing at `5f0b5e`–`5f0b89` has no third
    /// `set_anim`. Removing it did not cost two draws; it moved four. The
    /// stand had been resetting the citizens' clocks, so the four wraps
    /// the original spends in phase 7 — `Guy::set_anim+0x97a` under
    /// `Guy::inc_time`, between the herd walk and the farms — never fell
    /// due here. Both halves of `docs/SYNC.md` §6's stand/wrap swap were
    /// the one line, and no `GUYS=4` capture was needed to see it.
    ///
    /// The assertion is the whole sequence. A count of 175 cannot fail
    /// this way twice.
    #[test]
    fn frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        // (dump, trace, the frame-0 word, draws, farm draws, wraps)
        //
        // Two lobbies, and they are not the same shape: run20's islands
        // have two woodcutters a side and a pasture among its six farms
        // (five crop draws); run12's world6 has four and no pasture (six).
        // Both spend four `Guy::inc_time` wraps at the tail, and on run12
        // those are the four woodcutters `docs/ANIM.md` §5 could not place.
        let maps = [
            (
                "gamelog-run20-islands-dumpall.txt",
                "rontrace-run20.log",
                0x2f50_5213u32,
                175usize,
                5usize,
            ),
            (
                "gamelog-run12-dumpall-seeds.txt",
                "rontrace-run14.log",
                0x3bd3_9ae9,
                120,
                6,
            ),
        ];
        let mut ran = 0;
        for (dump_name, trace_name, word, draws, farms) in maps {
            let (Some(path), Some(trace)) = (dump(dump_name), trace(trace_name)) else {
                eprintln!("skipping {dump_name}: set RON_GAMELOG_DIR");
                continue;
            };
            ran += 1;
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let init = log.initial().unwrap();
            let mut built = build_sim(&loaded, &init, Tuning::RON);

            // The two sides start on the same word, or nothing below means
            // anything: the dump's last setup checksum and the word the
            // trace's own first draw stepped are one number seen from two
            // instruments. It is the *draw's* word rather than the `FRAME`
            // record's, because run14 predates the record carrying it —
            // its frame 0 reports `0x03fc45c6` and its first draw steps
            // `0x3bd39ae9`, which is what run12's dump also says.
            assert_eq!(
                trace.frame_draws(0).first().map(|d| d.seed),
                Some(word),
                "{trace_name}'s first frame-0 draw"
            );
            assert_eq!(built.sim.rng.seed, word, "{dump_name}: and the harness's");

            built.sim.trace_phases = true;
            built.sim.tick();
            let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
            let theirs = trace.labels(0);
            assert_eq!(theirs.len(), draws, "{trace_name}'s frame 0");

            // Every draw of the original's frame 0 is a site the sim
            // models: a bare hex address here would be a mechanic with no
            // mark, and the comparison below could not read it.
            let unnamed: Vec<&String> = theirs
                .iter()
                .filter(|l| !crate::trace::SITES.iter().any(|(_, _, n)| n == l))
                .collect();
            assert!(
                unnamed.is_empty(),
                "{trace_name}: frame 0 has draws no mechanic marks: {unnamed:?}"
            );

            if let Some((_, shown)) = first_parting(&ours, &theirs) {
                panic!("{dump_name}: {shown}");
            }
            assert_eq!(ours, theirs, "{dump_name}: frame 0, draw for draw");

            // And the counts the swap turned on, stated so a regression
            // reads as itself rather than as an index: four idle rolls for
            // the two scouts' figures, no camp stand at all, and four
            // phase-7 wraps.
            let count = |site: &str| ours.iter().filter(|l| *l == site).count();
            assert_eq!(
                (
                    count(sim::anim::SITE_IDLE_UNIT),
                    count(sim::anim::SITE_STAND_GATHER),
                    count(sim::anim::SITE_WRAP),
                ),
                (4, 0, 4),
                "{dump_name}: the scouts' four, no camp stand, four wraps"
            );
            // The wraps sit between the herd walk and the farms — the
            // frame's tail, which `docs/SYNC.md` §6 read as "the 4 draws
            // at 110–113".
            let mut fold: Vec<(String, usize)> = Vec::new();
            for l in &ours {
                match fold.last_mut() {
                    Some((last, n)) if last == l => *n += 1,
                    _ => fold.push((l.clone(), 1)),
                }
            }
            let last_three: Vec<(&str, usize)> = fold
                .iter()
                .rev()
                .take(3)
                .map(|(l, n)| (l.as_str(), *n))
                .collect();
            assert_eq!(
                last_three,
                vec![
                    (sim::farms::SITE_CHANCE, farms),
                    (sim::anim::SITE_WRAP, 4),
                    (sim::gaia::SITE_HERD_Y, 1),
                ],
                "{dump_name}: the frame ends farms ← wraps ← herd"
            );
        }
        if ran == 0 {
            eprintln!("skipping: neither traced map is on this machine");
        }
    }

    #[test]
    fn run14_s_frames_match_the_trace_draw_for_draw() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run10-world6-long.txt"),
            trace("rontrace-run14.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert_eq!(last, 284, "run14's traced length");
        for _ in 0..last {
            built.tick();
        }
        let mut matched = 0usize;
        let mut parted: Vec<String> = Vec::new();
        for (frame, ours) in &built.frame_sites {
            let theirs = trace.labels(*frame);
            if *ours == theirs {
                matched += 1;
                continue;
            }
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            parted.push(format!(
                "frame {frame}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            ));
        }
        // The first frame whose draw *sequence* is not the original's —
        // the number the tail cannot flatter.
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        assert!(
            first_part >= 284,
            "the stream parts at frame {first_part}; the floor is 284\n{}",
            parted.first().cloned().unwrap_or_default()
        );
        assert!(
            matched >= 284,
            "the trace floor fell: {matched} of {last} frames match, the floor is 284\n{}",
            parted.join("\n")
        );
        // **And a stricter floor beside it: the first frame whose draw
        // *count* differs.** Frame 99's disagreement is an attribution and
        // not a divergence — eight draws either side, and the one that
        // differs is the same address under a different caller (ours
        // `Unit::do_idle+0x7d`, the original's `Guy::inc_time+0x271`, the
        // standing swap `docs/SYNC.md` §6 names).
        //
        // The word was the original's through 121 and parted at **122** on
        // the blocked stand this simulation did not take. It takes it now
        // (item 49, `docs/COLLISION.md` §5), and with item 61's cell index
        // it ran to **201** — where the AI's farmer `1/4` re-picked a
        // second time and this simulation did not, because its walk to a
        // cell a sibling was already working was not refused by the
        // collision. With item 63's waypoint test it is, and the word ran
        // to **232**, where the disagreement was an arrival stand
        // (`Guy::set_anim+0x97a < Guy::move+0x19f`) the original takes and
        // this simulation did not.
        //
        // With item 66 — `find_nearby_spot`'s own collision half — it runs
        // to the **end of the capture**: all 284 frames spend the same
        // number of draws, and 282 of them are the original's draw for
        // draw. The two that are not were 99, the ~~attribution swap~~
        // above, and 100 — and with the frame's two loops (item 60) they
        // are the original's too: **284 of 284, draw for draw**. It was
        // never an attribution question. `Objects::process_all` runs the
        // buildings *after* the units, so the citizen trained on 99 is
        // never reached by that frame's unit loop and it is
        // `Objects::inc_time` that wraps its `end_time 0` clock
        // (`docs/SYNC.md` §3.16). **This capture is spent**: it can no
        // longer say where the simulation next parts from the original,
        // and the long traces — run33's and run39's — are what do.
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        assert!(
            first_count >= 284,
            "the stream's *word* parts at frame {first_count}; the floor is 284"
        );
        // **The farmer's re-target, frame for frame — item 61's own row.**
        // Two `orders::SITE_FARM_CELL` draws are one farmer picking a new
        // cell because the one under it ripened, and the frame it happens
        // on is a hundred frames of the farm clock plus the cell it was
        // sowing. The cell index was transposed (`status[dy][dx]` for
        // `status[dx][dy]`, `docs/ORDERS.md` §6.5), so from frame 101 —
        // the first time a farmer stands anywhere but the symmetric
        // `(2, 2)` — six farmers sowed six wrong cells, and the first
        // re-target fell on **185** where the original's falls on 199.
        // The farm-record diff cannot see this (its captures stop at 104,
        // before any farmer reaches its new cell), so it is pinned here.
        let cell_frames = |at: &dyn Fn(i64) -> Vec<String>| -> Vec<(i64, usize)> {
            (0..=last)
                .map(|f| {
                    let n = at(f)
                        .iter()
                        .filter(|l| **l == sim::orders::SITE_FARM_CELL)
                        .count();
                    (f, n)
                })
                .filter(|&(_, n)| n > 0)
                .collect()
        };
        let ours_at = |f: i64| -> Vec<String> {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or_else(Vec::new, |(_, s)| s.clone())
        };
        let theirs_at = |f: i64| -> Vec<String> { trace.labels(f) };
        let (mine, theirs) = (cell_frames(&ours_at), cell_frames(&theirs_at));
        assert_eq!(
            theirs,
            vec![
                (101, 12),
                (199, 2),
                (201, 2),
                (211, 4),
                (217, 4),
                (218, 2),
                (220, 2),
                (241, 2)
            ],
            "the original's own re-targets: six farmers on 101, then one a \
             cell at a time as each ripens"
        );
        // And ours is that list entire up to the frame the word parts on.
        // It used to be a prefix of two — 101 and 199, with 211 and 217
        // checked separately and the tail past the word's divergence
        // unusable. Item 63's waypoint collision test bought the 201 row,
        // which is `1/4` re-picking a second time after its walk to `1/2`'s
        // cell was refused; with it the whole schedule to 241 is the
        // original's, frame for frame and draw for draw.
        //
        // Item 64 added one row of our own at **243**, past the frame the
        // word parted on, so the assertion was split: everything the
        // comparable stretch carries must be the original's exactly, and
        // anything extra must be past that frame. Item 66 took the word to
        // the end of the capture, and with it the extra row went — the
        // whole schedule to 241 is the original's, frame for frame and
        // draw for draw, and there is nothing of ours outside it.
        assert_eq!(
            mine.iter()
                .copied()
                .filter(|&(f, _)| f < first_count)
                .collect::<Vec<_>>(),
            theirs
                .iter()
                .copied()
                .filter(|&(f, _)| f < first_count)
                .collect::<Vec<_>>(),
            "every farm re-target before the word parts, on the original's frames"
        );
        assert!(
            mine.iter()
                .all(|&(f, _)| f >= first_count || theirs.iter().any(|&(g, _)| g == f)),
            "and no re-target of ours inside the comparable stretch that the \
             original does not make: {mine:?} against {theirs:?}"
        );
        assert_eq!(
            mine.iter()
                .filter(|&&(f, _)| !theirs.iter().any(|&(g, _)| g == f))
                .copied()
                .collect::<Vec<_>>(),
            Vec::new(),
            "no re-target of ours the original does not make"
        );
        // **The bird's own row, and it is the original's now.** Every
        // eighth frame carries three `Animal::think_bird` draws per living
        // bird, and with item 59 this simulation hatches its first on
        // **frame 96 — the original's own frame** — so the beat agrees row
        // for row from 104 to 192, the last eighth-frame before the
        // original's second bird. (Before item 59 ours hatched at 32 and
        // 128 and the rows could not be compared at all.) The second bird
        // is still drift: the sampling reads cells off a stream that parts
        // at 122, so ours hatches at 224 where the original's hatches at
        // 192, and every row from 200 on is ours rather than the
        // original's.
        let think = |sites: &[String]| -> usize {
            sites
                .iter()
                .filter(|l| l.starts_with("Animal::think_bird"))
                .count()
        };
        let ours_beat: Vec<(i64, usize)> = built
            .frame_sites
            .iter()
            .map(|(f, s)| (*f, think(s)))
            .filter(|&(f, n)| n > 0 && f <= 192)
            .collect();
        let theirs_beat: Vec<(i64, usize)> = (0..=192)
            .map(|f| (f, think(&trace.labels(f))))
            .filter(|&(_, n)| n > 0)
            .collect();
        assert_eq!(
            ours_beat, theirs_beat,
            "three draws a bird an eighth-frame, on the original's frames"
        );
        assert_eq!(
            ours_beat.len(),
            12,
            "every eighth frame from the hatch to the original's second bird"
        );
        assert_eq!(
            trace.labels(96)[6],
            sim::anim::SITE_INIT_REAL,
            "the hatching roll is draw 6 of the original's sampling frame"
        );
        // **The hatch frame is a mechanism again, not drift.** run14's
        // birds hatch at 96, 192 and 256, and the sampling that decides a
        // hatch reads cells off the sync stream — so a hatch is the
        // original's exactly as far as the word is. With item 61 the word
        // runs to 201 and **the first two hatches are the original's own
        // frames**: 96 (where the pin has stood since item 59) and now
        // **192**, which the old `[96]` pin could not reach because the
        // word parted at 185, seven frames short of it. The tail —
        // ours at 224 and a pair at 256 — is off a stream that is no
        // longer the original's after 201, and the count of live birds
        // with it. The history of this pin is the history of the word:
        // `[96, 224]` off a divergence at 122, `[96]` off 185, `[96, 192]`
        // off 201, `[96, 192, 256, 256]` off 232 — and with item 66's word
        // running the whole capture, **`[96, 192, 256]`, the original's
        // three hatches and nothing else**. There is no tail left to
        // excuse: every hatch this simulation makes is one the original
        // makes, on its frame.
        let hatches: Vec<i64> = built.sim.gaia.bird_spawns.iter().map(|(f, _)| *f).collect();
        assert_eq!(
            hatches,
            vec![96, 192, 256],
            "the original's own three hatch frames, and no other"
        );
        assert_eq!(built.sim.live_birds(), 3, "alive at the end");
        // The wing beat, which item 52 bought and item 59 put on the
        // original's frames: the hatch frame's wrap (`Guy::init_real`
        // leaves `end_time` at zero, so the same frame's `inc_time`
        // overflows it at once), the birth coin `do_air_physics` throws
        // the frame after, and a coin at every wrap the animation's own
        // length places. *Which* animation is a coin, so the spacing
        // alternates between *Bird Flap*'s 23 frames and *Bird Soar*'s 31.
        // The lengths are the install's; the frames are the original's —
        // **97, 127, 142, 150** on both sides, and the next coin is past
        // the word divergence at 122 and is ours.
        let coin_frames = |at: &dyn Fn(i64) -> Vec<String>, upto: i64| -> Vec<i64> {
            (0..=upto)
                .filter(|f| at(*f).iter().any(|l| *l == sim::anim::SITE_BIRD_COIN))
                .collect()
        };
        let ours_at = |f: i64| -> Vec<String> {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or_else(Vec::new, |(_, s)| s.clone())
        };
        let theirs_at = |f: i64| -> Vec<String> { trace.labels(f) };
        assert_eq!(
            coin_frames(&ours_at, 160),
            coin_frames(&theirs_at, 160),
            "the birth coin the frame after the hatch, then a coin at every \
             wrap — on the original's frames while the word is still its own"
        );
        assert_eq!(
            coin_frames(&ours_at, 160),
            vec![97, 127, 142, 150],
            "the hatch's birth coin, then Soar's 31 and Flap's 23"
        );
        // Item 53's own rows, each stated so a regression reads as itself.
        //
        // Frame 1 is the AI's opening: eight `rand_int(1, 10)` inside the
        // script VM, and it is the whole of that frame's script draws.
        // Naming them took the frame from "54 against 54 in the wrong
        // vocabulary" to a match.
        let frame_one = &built
            .frame_sites
            .iter()
            .find(|(n, _)| *n == 1)
            .expect("frame 1")
            .1;
        assert_eq!(
            &frame_one[..8],
            &[sim::ai_host::SITE_RAND_INT; 8],
            "the script VM's eight, and no others"
        );
        // Frame 101 is the farmers' re-target: six farmers, two
        // `GameAccess::rnd(4)` each, one mark carrying the pair.
        let count = |f: i64, label: &str| -> usize {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or(0, |(_, s)| s.iter().filter(|l| *l == label).count())
        };
        assert_eq!(
            count(101, sim::orders::SITE_FARM_CELL),
            12,
            "six farmers' cell re-pick, two draws each"
        );
        // Frame 108 is one herd animal's whole wander: the three-in-ten
        // coin, then the direction and the two step counts, in that order.
        let hundred_eight = &built
            .frame_sites
            .iter()
            .find(|(n, _)| *n == 108)
            .expect("frame 108")
            .1;
        let wander: Vec<&String> = hundred_eight
            .iter()
            .filter(|l| l.starts_with("Animal::do_idle"))
            .collect();
        assert_eq!(
            wander,
            vec![
                sim::gaia::SITE_WANDER_ROLL,
                sim::gaia::SITE_WANDER_DIR,
                sim::gaia::SITE_WANDER_X,
                sim::gaia::SITE_WANDER_Y,
            ],
            "the wander's four, in order"
        );
        // `Guy::move+0x19f` is the arrival stand, which this simulation
        // takes on its own frames. Without the chain it reads as a bare
        // `5dac7a` and the residue is unreadable, which is the whole of
        // item 53.
        assert!(
            trace
                .labels(232)
                .iter()
                .any(|l| l == sim::anim::SITE_ARRIVE),
            "the arrival stand is named on the trace's frame 232"
        );
        // **The blocked stand, on the original's own frames** (item 49).
        // `Unit::move_step+0x823` is the idle a unit re-rolls the moment
        // its step is refused, before the three give-up tests. run14 has
        // three of them; this simulation takes the first two on the
        // original's frames, which is what carries the word from 122 to
        // 185. The third is past the divergence and neither side is
        // measuring the same game by then.
        let blocked = |sites: &[String]| sites.iter().any(|l| l == sim::anim::SITE_BLOCKED);
        let theirs_blocked: Vec<i64> = (0..=last).filter(|&f| blocked(&trace.labels(f))).collect();
        assert_eq!(
            theirs_blocked,
            vec![122, 184, 256],
            "the original's three blocked stands"
        );
        let ours_blocked: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| blocked(s))
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(
            ours_blocked
                .iter()
                .copied()
                .filter(|&f| f <= 185)
                .collect::<Vec<i64>>(),
            vec![122, 184],
            "the two before the word parts are the original's, frame for frame"
        );

        // And it flies, which is what keeps it out of the occupancy grid a
        // citizen walks on — `collide.rs`'s `is_air` is the loaded domain
        // now rather than the seam it was (`docs/COLLISION.md` §2).
        let bird = built
            .sim
            .units
            .iter()
            .find(|u| u.owner == sim::gaia::BIRD_OWNER)
            .expect("a bird");
        assert_eq!(bird.kind.domain, sim::attrition::Domain::Air);
    }

    /// **The long trace, and where the word parts past run14's 284.**
    ///
    /// run14 traced 284 of run10's 1,772 frames, and since item 66 its
    /// word — the per-frame draw *count* — matched for every one of them.
    /// A trace that agrees to its own end cannot say where the simulation
    /// next parts by *site*, which is what item 38 was: **run33** is run10's
    /// own game captured again under the traced executable, with run10's
    /// exact dump settings and `cover=1`, quitting at frame 1,850
    /// (`docs/RUNS.md`, "run33"). `tools/gamelog/samegame.py` is the
    /// proof it is the same game — every frame block of run33 and run10
    /// digests identically — so this capture inherits run10's siblings and
    /// supersedes run14 wherever the two overlap.
    ///
    /// Two numbers come out of it, and they are the sub-scores the residue
    /// chase is steered by:
    ///
    /// - **the word**: the first frame whose draw *count* is not the
    ///   original's, and
    /// - **the sequence**: the first frame whose draws are not the
    ///   original's site for site, which is at or before it.
    #[test]
    fn run33_s_long_trace_says_where_the_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 1_800,
            "run33's traced length is {last}, wanted 1,800+"
        );
        for _ in 0..last {
            built.tick();
        }
        let mut matched = 0usize;
        let mut parted: Vec<String> = Vec::new();
        for (frame, ours) in &built.frame_sites {
            let theirs = trace.labels(*frame);
            if *ours == theirs {
                matched += 1;
                continue;
            }
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            parted.push(format!(
                "frame {frame}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            ));
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        eprintln!(
            "run33: word parts at {first_count}, sequence at {first_part}; \
             {words} of {last} frames spend the original's number of draws, \
             {matched} of them draw for draw"
        );
        for p in parted.iter().take(4) {
            eprintln!("{p}");
        }
        // The count divergence is the score, so print its own row too: it
        // is the successor item every time this number moves.
        if let Some(p) = parted
            .iter()
            .find(|p| p.starts_with(&format!("frame {first_count}:")))
        {
            eprintln!("count: {p}");
        }
        // **The word: 307.** run14's capture agreed to its own end at 284;
        // this one carries 1,566 frames more, and the simulation's
        // per-frame draw *count* is the original's for twenty-three of
        // them before it parts.
        //
        // What parts it is a **non-flat gather**. The original's frame 307
        // spends eleven draws and this simulation nine, and the two it
        // does not spend are one unit's: a stand issued from inside
        // `Unit::do_non_flat_gather+0x10f`, and `+0x54b`, the gather's own
        // roll — so on the original a gatherer is working a non-flat
        // resource on that frame and here it is not. The other nine are
        // the same on both sides and in the same order: an
        // `Animal::do_idle` roll, a phase-7 wrap, and seven farms.
        // **345 since item 68** (2026-08-29). The two draws frame 307 was
        // short were `1/7`'s, and `1/7` was not at its camp to spend them
        // because the AI's army had marched it away on 252: `Unit::think`'s
        // tail was joining any attacker to an army, where the original
        // joins only a supply wagon or a hero, and run33's own coverage
        // says `Unit::add_to_army@005f7740` is never entered in this game
        // at all. With the tail as the listing has it the word runs to
        // **345**.
        //
        // **What parts it at 345 is not the ninth citizen.** Item 68's
        // note guessed it was — `1/8`'s Woodcutter's Camp, the row that
        // pinned the headline three frames earlier — and item 70 fixed
        // exactly that and moved this number not at all. The row is an
        // animation draw against a farm's: on 345 this simulation spends
        // eight where the original spends seven, and the very first is
        // ours `Guy::set_anim < Guy::inc_time` against the original's
        // `Farms::inc_time+0x1ae`. It is the same row before and after
        // item 70, byte for byte. The **totals** are what moved.
        //
        // **361 with item 71** (2026-08-29), and what was wrong at 345 was
        // a **length**. The human scout's guy 0 rolled `CHAR_IDLE1` on
        // frame 284; no dump has ever shown that slot's length for its
        // piece, so `Art::lengths` had none and the roll fell back to
        // `CHAR_DEFAULT` — 61 frames instead of 76. The clock wrapped on
        // 345 where the original's runs to 360, and in the five frames
        // between, the original's *dog* — whose mirrored `cur_time` had
        // run past its own `end_time` — re-rolled once a frame from
        // `Unit::do_idle`, which this simulation never reached. Four
        // draws of drift by frame 349, and the farmer `0/4`'s two `% 4`
        // re-target draws came off the wrong words (item 71's own
        // symptom, the headline).
        //
        // The lengths now come from the install: `unit_graphics.xml` for
        // every `<UNIT>` entry, placed at the piece
        // `GraphicPieces::get_unit_gpiece` hands out
        // (`crate::artdata::piece_lengths`). That exposed a second one —
        // `unit_masks & 0x78000000`, the carrying walk, which this crate
        // had been reading off the gather order's `goto_build` instead.
        // With the carrying slots' lengths known the stand-in stopped
        // being invisible: `1/7`'s first walk to its camp became a
        // `WALK_WITH_WOOD` and lost the arrival stand the original spends
        // at frame 232. The mask is `do_non_flat_gather`'s own, and with
        // it the word runs to **361**.
        //
        // **432 with item 76**, and what was wrong at 361 was a **tile**.
        // The cell filter's surface probe (`docs/SCOUT.md` §7) was reading
        // `(4x, 4y + 2)` where the listing reads the cell **centre**: the
        // `+4` in `movb 0x4(%eax,%ecx,2)` is two `TData` elements, not a
        // field offset. Cell `(48, 23)` is ocean at its first tile and
        // land at its centre, so the AI scout's re-target refused a
        // candidate the original takes — twenty-six draws against
        // twenty-seven — and went to `(48, 20)` instead. What parts the
        // word at 432 is a gather stand: ours spends
        // `Guy::set_anim < Unit::do_non_flat_gather+0xb99` where the
        // original spends a farm's `Farms::inc_time+0x1ae`, twenty-three
        // draws against twenty-two.
        //
        // **482 with item 78**, and what was wrong at 432 was a **write
        // that went to the wrong order**. `Sim::store_gather` wrote the
        // gather order's fields back to `orders.front_mut()`, but every
        // walk `do_non_flat_gather` issues goes in *front* of the gather
        // (`QUEUE_FIRST`, no action bit) — so on the return-to-camp branch
        // the `goto_build = 1` and `wait = 32` that follow
        // `add_move_order` were written to the move and silently dropped.
        // The original holds a pointer to the order object for the whole
        // function, which is the same rule `is_gathering_at` already
        // needed on the *read* side (`docs/ORDERS.md` §6, §6.4).
        //
        // The human's woodcutter `0/2` finished its shift on frame 426,
        // set off for its camp on 427 and arrived on 431 — and then, with
        // `goto_build` still 0 and `wait` still −1, re-entered the same
        // branch on 432 and spent the stand again. It never unloaded, never
        // took another tile, and re-issued that walk every other frame for
        // the remaining 1,400 frames of the capture.
        //
        // **571 with item 79**, and what was wrong at 482 was an **angle**.
        // The scout re-targets on 482 and the original spends thirty-one
        // draws over its seven rings where this spent thirty: cell
        // `(56, 28)` was already seen here and not there, so the cell
        // filter refused a candidate the original scores
        // (`run33_s_scout_re_targets_at_482_on_the_original_s_ring`).
        //
        // It was seen here because of a reveal three hundred frames
        // earlier. `Object::update_seen` throws a small land unit's disc
        // half a cell forward of its nose, and the angle it projects along
        // is `UnitData +0x50` — the unit's own heading — not the guy's
        // eased facing; the `project` at `00651d05` is handed it by
        // `movl 0x50(%ecx), %ecx` two instructions earlier. This crate had
        // the guy's. On frame 168 the scout was mid-turn, the two angles
        // were −51.6° and −83.0°, and the disc thrown along the wrong one
        // lit a fog cell the original's never reached (`docs/VISION.md`
        // §3).
        //
        // What parts the word at 571 is a **collision**: the original
        // spends a draw at `5fa882`, inside
        // `Unit::resolve_unit_collision@005f9d30`, that this simulation
        // does not — eight draws against seven.
        //
        // **776 with item 81**, and what was wrong at 576 was a **price**.
        // The original's frame 576 spends sixty draws over five
        // `ScenarioFuncSet::place_city_with_cost` calls — the script's
        // `city_placement` under `defensive.bhs` step 11, once per turn of
        // its `num_loops = 5` loop — and this simulation spent forty,
        // because its *first* call bought the city and the guard at
        // `009f5898` (`city_limit <= total_cities`) then returned −1 four
        // times without a draw. The original could not pay: a second Small
        // City costs **sixty** food and sixty timber, not twenty-two, and
        // the AI held 69 and **59**.
        //
        // `TypeData::get_cost` has two ramps. The unit arm reads
        // `UNIT_COST_FACTOR` and all four `*_RAMP_MAX`; the building arm
        // (`00665787`..`00665b5a`) reads `BUILD_COST_FACTOR` and
        // `BUILD_SUPPORT_FACTOR` and **no ceiling at all**. This crate gave
        // every building `RampClass::default()` — the military 125% — which
        // clamped the city's `SUPPORT 50` ramp to 12 and priced it at 22.
        // run40 and run41 measure both ends of it
        // (`run40_s_census_prices_the_ai_s_second_city_at_sixty`).
        //
        // What parted the word at 776 was that same block a second time:
        // the original spent five draws there and this simulation ten,
        // because on 776 it was the *original* that bought on its first
        // call and this simulation that could not — its AI was thirty-two
        // food short of the original's on every frame from 202 on. Item 74
        // is those thirty-two: twenty from the farm the AI finishes on
        // frame 166 (`Build::do_bonus`, `docs/ECONOMY.md`) and twelve from
        // the City State waiting in its library when Written Word lands on
        // 201 (`Build::refund_cost`, `docs/COSTS.md`). With them the city
        // is bought on 776 here too, and **the word parts at 780**.
        //
        // **986 with item 84**, and what was wrong at 780 was a **byte
        // that was not the guy's**. `do_gather`'s farm switch tests
        // `guy[0].cur_anim` — `*(char *)(**(int **)&this->field_0xf4 +
        // 0x9c)`, `GuyData +0x9c` — and this crate kept a `farm_anim`
        // flag written only by the farm branch and cleared only by a
        // step. The AI's `1/4` stands on farm `b10`'s cell 7, reaps it
        // until `inc_time` decays it empty on 777, re-picks a tile on 778
        // and is **blocked** on 779: the blocked stand
        // (`Unit::move_step+0x823`) replaces the guy's reap animation
        // without moving the body, so on 780 the original sows the cell
        // and this crate re-picked again. Two draws it does not spend, a
        // seventh `Farms::inc_time` from 781 — the farm keeps five empty
        // cells here and drops to four there — and what parts the word at
        // 986 is a blocked stand of its own, seven draws against six.
        //
        // **1372 with item 112**, and what was wrong at 986 was a **herd's
        // wander centre, which does not walk**. `Herd::process@00741760`
        // writes `wx = cx − 1 + p % 3` and `wy = cy − 1 + p % 3` — it
        // reads `HerdData +0x0/+0x4` and stores into `+0x8/+0xc`, so the
        // jitter is about the **home** cell every time. This crate read
        // the destination as the source and random-walked it, which is the
        // same thing until a herd's *second* walk; `(frame >> 6) % 13`
        // gives herd 0 its second on frame 832 and no other herd one at
        // all inside the capture. Its `wy` ended 34 here against the
        // original's 33, and the far wander's ring is drawn about
        // `herd_centre` (`docs/ANIM.md` §7): on 981 the sheep `8/3` was
        // sent to `(17688, 26616)` where the original sends it to
        // `(17688, 26328)`, five steps of `(+15, −5)` against five of
        // `(+12, −10)`, so the same neighbour refused it a frame early and
        // the blocked stand fell on 986 instead of 987. Pinned step for
        // step by `run33_s_herd_centre_jitters_about_its_home_cell`.
        //
        // **1802 with item 114**, and what was wrong at 1372 was a
        // **pasture nothing stocked**. `Build::activate`'s gather tail
        // reaches `Farms::add_animals` for every *food* gather building it
        // completes — `LAB_00625a36` is the fall-through from
        // `do_bonus(0, FOOD_BONUS_FOR_FARM)` and the `iVar18 == 0` arm of
        // `switchD_006259b5_caseD_2` is where every unpaid exit lands, and
        // both are the same call — so the AI's fourth farm, which
        // `Farms::add` had already made a pasture, costs twenty draws on
        // the frame it finishes: a species coin, a `y` and an `x` per
        // animal (`+0x92`, `+0x134`, `+0x182`) and `Guy::init_real`'s
        // variant inside `Objects::init_unit`. Five more follow in the
        // same frame's `Objects::inc_time`, because a newborn guy's
        // `end_time` is zero and its clock wraps at once. Twenty-five
        // draws where this crate spent none, and `Farms::inc_time`'s three
        // were all that was left. The harness had the building and the
        // type right — its `b15` activates on 1372 with `farm_type 1` —
        // and only the stocking was missing (`docs/SYNC.md` §3.11).
        //
        // The parting frame goes 1372 -> **1802**, which is past run10's
        // whole 1,772: this map now spends the original's draws in the
        // original's order for longer than its own capture runs. What
        // parts it at 1802 is one draw at
        // `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3`, a gaia
        // bird's flight physics, and it is the **only** time that site is
        // reached in all 1,851 frames.
        assert!(
            first_count >= FLOORS[1].word,
            "the word parts at frame {first_count}; the floor is {}\n{}",
            FLOORS[1].word,
            parted.first().cloned().unwrap_or_default()
        );
        // **The sequence: 576**, and getting there was the whole of the
        // ~~99~~ attribution swap run14's capture had always shown — the
        // same address under a different caller, ours `Unit::do_idle+0x7d`
        // where the original has `Guy::inc_time+0x271`, with the draw count
        // equal either side (queue item 62). It was not an attribution
        // question at all: the trained citizen is created in
        // `Objects::process_all`'s **second** loop, after every unit, so
        // the original never reaches it in the unit loop on its birth
        // frame and `Objects::inc_time` wraps its `end_time 0` clock
        // instead. `docs/SYNC.md` §3.16. It is a floor here so that a
        // regression that moved it would read as itself.
        // **576 -> 780 with item 100**, and the two numbers meet: naming
        // `make_stuff`'s expiry walk (`+0x221`) is what moved it, because
        // an unnamed draw takes the last mark's name and five frames of
        // `place_city_with_cost` at 576 read as `compute_sites+0x50a`.
        // Nothing about the simulation's arithmetic changed there; what
        // changed is that the comparison stopped lying about it.
        // **780 -> 986 with item 84**, the two still meeting: the farm
        // stand's byte is the guy's live `cur_anim`, and nothing between
        // 780 and 986 is a naming question.
        // **986 -> 1372 with item 112**, and the two meet a third time:
        // the herd's wander centre is jitter about the home cell, and
        // nothing between 986 and 1372 is a naming question either.
        // **1372 -> 1802 with item 114**, a fourth time: the pasture's
        // twenty draws carry names of their own from the frame they are
        // first spent on, and nothing between 1372 and 1802 is a naming
        // question. The two numbers have now been equal for four items
        // running, which is what it looks like when every hole left is a
        // mechanic rather than a label.
        assert!(
            first_part >= FLOORS[1].word,
            "the draw sequence parts at frame {first_part}; the floor is {}",
            FLOORS[1].word
        );
        // And the totals over the whole 1,850, which is what says whether a
        // change past the divergence helped or only moved the noise: 618
        // frames spend the original's number of draws and 460 of them are
        // its draws in its order. Past the part frame both sides are off
        // streams of their own, so these are weak numbers — but a fall in
        // them is worth reading.
        //
        // 668 / 556 → **618 / 460** with item 68, the only time either has
        // fallen while the score rose. Every frame before 345 matches on
        // both counts, so the whole of the fall is past the divergence: the
        // stream this simulation is on after `1/8` takes the wrong job is a
        // *different* wrong stream from the one it was on after `1/7` was
        // marched off, and it happens to coincide with the original's less
        // often. The number that is not luck is `first_count`, 307 → 345.
        //
        // 618 / 460 → **635 / 488** with item 70, and this is the useful
        // half of that item on this capture: `first_count` did not move,
        // but seventeen more frames spend the original's number of draws
        // and twenty-eight more spend them in its order, because the AI's
        // citizens are at the buildings the original has them at for the
        // rest of the run.
        //
        // 635 / 488 → **696 / 523** with item 71, the largest move either
        // has made, and the two rose together with `first_count`.
        //
        // 696 / 523 → **724 / 606** with item 76, and `matched` moved most:
        // eighty-three more frames are the original's draws in the
        // original's order, which is what a scout sent to the original's
        // cell buys downstream.
        //
        // 724 / 606 → **752 / 622** with item 78.
        //
        // 752 / 622 → **791 / 662** with item 79, and the two rose with
        // `first_count` again — thirty-nine more frames on the original's
        // word, forty more of them draw for draw, because the AI's scout
        // now walks the original's fog as well as its ground.
        //
        // 791 / 662 → 802 / 688 with item 80, and → **944 / 828** with item
        // 81 — the largest move either has made, and both rose with
        // `first_count`: a hundred and forty-two more frames spend the
        // original's number of draws because the AI's whole economy is two
        // hundred frames closer to the original's from 576 on.
        //
        // 944 / 828 → **951 / 838** with item 74, and the three rose
        // together: the AI's second city is founded on the original's
        // frame, so its ninth citizen is trained on the original's frame
        // too and the roster is one-sided again at 268 + 0.
        //
        // 951 / 838 → **954 / 843** with items 36 and 93 together. This is
        // the number item 93 was held back for: the arm alone, before the
        // farmers' angles were the original's, cost `first_count` 780 →
        // 584 and run10's orders 776 → 586. With item 36 in front of it
        // the word holds at 780 and both totals rise
        // (`docs/SYNC.md` §3.11, §3.12).
        //
        // 954 / 843 -> **938 / 841** with item 95, the animal's hurry, and
        // this is the second time either has fallen while the score rose
        // (item 68 was the first). The fall is entirely noise and it can
        // be said exactly where it starts: with the hurry in and out, this
        // capture's per-frame draw counts are **identical up to frame
        // 1108** — 328 frames past the word's own parting at 780, and past
        // every one of the twelve units' first divergence. `first_count`
        // holds at 780 and `first_part` at 99; what moved is which of two
        // wrong streams the simulation is on after 1108. The number that
        // is not luck is the *other* map's, where the same change takes
        // the word 69 -> 91.
        //
        // 938 / 841 -> **943 / 827** with the blocked animal's dropped
        // walk, and the same argument holds a second time with the same
        // shape: with that rule in and out, this capture's per-frame draws
        // are **identical — the count and the sequence both — up to frame
        // 1128**, 348 frames past the word's own parting at 780 and past
        // eleven of the twelve units' first divergence. `first_count`
        // holds at 780, `first_part` at 99, run10's ticks and orders at
        // 572/776, and the other map's word goes 91 -> 201
        // (`docs/SYNC.md` §3.14).
        //
        // 943 / 827 -> **943 / 830** with the frame's two loops (item 60),
        // and what moved with them is the *sequence*: 99 -> 576. The word
        // holds at 780, run10's ticks and orders at 572/776, and the other
        // map's word goes 219 -> 274 (`docs/SYNC.md` §3.16).
        //
        // 943 / 830 -> **964 / 864** with item 100, and again it is the
        // *sequence* that moves: **576 -> 780**, so this map's two numbers
        // are now the same frame. Two things landed together and only one
        // of them is a mechanic. `make_stuff`'s expiry walk had always
        // spent its draws and never **named** them, so every one read as
        // whatever site marked last — here `compute_sites+0x50a`, five
        // frames of it at 576 — and a hole that was only a missing label
        // sat in front of the holes that are real. The mechanic is the
        // bird's landing search (`docs/SYNC.md` §3.9), sixty draws a
        // landing, which is what takes the *other* map's word 576 -> 645.
        //
        // 964 / 864 -> **977 / 866** with item 101, the chopping guy's own
        // wait: both numbers rise while the word and the sequence hold at
        // 780, because a woodcutter that stays at its tile three times as
        // long is at the original's tile on hundreds of the frames past
        // the parting. The other map's word goes 645 -> 742.
        //
        // 977 / 866 -> **986 / 884** with item 102, the far wander's
        // literal bearing (`docs/SYNC.md` §3.19): this map's herd takes
        // the same branch East Indies' does, so nine more frames spend the
        // original's number of draws and eighteen more are draw for draw,
        // while the word and the sequence hold at 780. The other map's
        // word goes 742 -> 867.
        //
        // 986 / 884 -> **986 / 892** with item 106, the scout's walk to a
        // goody box (`docs/GOODY.md` §7): Great Lakes has 22 boxes, and
        // eight more frames past the parting come out draw for draw while
        // the word and the sequence hold at 780. The other map's word goes
        // 879 -> 1256.
        //
        // 986 / 892 -> **959 / 876** with item 109, the bird's landing
        // step (`docs/SYNC.md` §3.9) — **the first fall these two numbers
        // have taken**, and it is the noise this comment's own opening
        // paragraph names. Every frame whose verdict changed is **1209 or
        // later**, 429 past the parting: 31 gained, 47 lost. What moved
        // them is that a bird's counter is one higher after each landing,
        // so this map's landings go 904, 936, 1080, 1136, 1152, 1288,
        // 1416, 1520, 1728, 1784 to 904, 936, 1080, 1136, 1152, **1184,
        // 1232, 1456, 1640**, 1784 — a stream that is nobody's rolling a
        // different number, on frames where the original's own landings
        // (816, 904, 1024, 1040, 1144, 1208, 1344, 1448, 1456, 1592,
        // 1744, 1816, 1832) it now meets **two** of rather than one. The
        // word and the sequence hold at 780, East Indies' word goes
        // 1256 -> 1373, and run39's landings up to its word are the
        // original's exactly
        // ([`a_bird_s_landing_frames_are_the_trace_s_own`]).
        //
        // 959 / 876 -> **953 / 877** with item 69's collision pause
        // (`docs/ORDERS.md` §4.4, `goto STEP`), the second fall and the
        // same kind of noise: the word and the sequence hold at **780**,
        // and every frame whose verdict changed is past it, on a stream
        // that is nobody's. What the item did move is the score — this
        // map's ticks 572 -> 781 and East Indies' 167 -> 1374 — which is
        // the number these two are instruments beside.
        //
        // 953 / 877 -> **1182 / 1114** with item 84, the largest move
        // either has ever made and the first since item 81 that is not
        // noise: the word itself goes 780 -> 986, so two hundred of those
        // frames are frames both sides genuinely agree on rather than
        // coincidences past a parting.
        //
        // 1182 / 1114 -> **1471 / 1439** with item 112, larger again and
        // not noise either: the word goes 986 -> 1372, so three hundred
        // more of these are frames both sides genuinely agree on. The gap
        // between the two also narrows — 68 frames spend the original's
        // number of draws in some other order, against 68 before — which
        // is what a stretch of real agreement rather than coincidence
        // looks like.
        //
        // 1471 / 1439 -> **1467 / 1436** with item 113, the second fall
        // these two have taken, and the argument is exact rather than
        // statistical this time: **the word and the sequence both hold at
        // 1372**, and `first_count` is by construction the first frame
        // whose draw *count* differs, so every frame before 1372 spends
        // the original's number of draws in the original's order on both
        // sides of the change. Every one of the seven verdicts that moved
        // is at 1372 or later, past the parting, on the two wrong streams
        // this comment's opening paragraph names. What the item moved is
        // the headline: this map's ticks go 910 -> **1375** and its orders
        // 776 -> **791**, the AI's `1/1` stops parting at all, and East
        // Indies holds at 1374/1373.
        //
        // 1467 / 1436 -> **1832 / 1830** with item 114, the largest move
        // either has ever made and the last one that can be large: 1,832
        // of the 1,850 frames spend the original's number of draws and
        // 1,830 of them draw for draw, so what is left is eighteen frames
        // and all of them past 1802. The gap between the two is down to
        // **two**, from 31 — a stretch of real agreement rather than
        // coincidence has no room for a frame that spends the right
        // number of draws in the wrong order.
        assert!(
            words >= 1832 && matched >= 1830,
            "the trace floor fell: {words} frames on the original's word, \
             {matched} draw for draw; the floors are 1832 and 1830"
        );
    }

    /// **run53 — the same game, thirteen times as long, and the ceiling is
    /// the same frame** (2026-08-31, item 91).
    ///
    /// `run33_s_long_trace_says_where_the_word_parts` is 1,850 frames, and
    /// once item 114 took the word to 1802 the obvious worry was that 1802
    /// was an artifact of the capture running out rather than a mechanic
    /// boundary. run53 is the same game — same seed, same lobby, no input —
    /// traced whole over **24,000** frames, and `tools/gamelog/rngcmp.py`
    /// says its `game_random` word is run33's on all 1,851 overlapping
    /// frames with **zero** differing. So this is not a second opinion; it
    /// is the same opinion with twelve times more of it.
    ///
    /// **The word still parts at 1802**, and what parts it is one draw at
    /// `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3` — a gaia
    /// bird's flight physics, reached **once** in 1,851 frames and, as this
    /// capture now shows, rarely enough to stay the boundary over 24,000.
    /// That is the successor item, and this capture is what makes it
    /// scoreable without another one.
    ///
    /// **Only the two frame numbers are pinned here, deliberately.** Past
    /// 1802 both sides run on streams that are nobody's, so of the 24,000
    /// frames 4,231 spend the original's number of draws and 2,040 do it in
    /// the original's order — coincidence at about one frame in six, and a
    /// number that moves with every unrelated change. `docs/QUEUE.md` item
    /// 89(c) is exactly this trap, and run33's own totals are only worth
    /// pinning because most of its frames fall *before* its parting. Here
    /// they would be a non-monotone score pinned as monotone, so they are
    /// printed and not asserted.
    #[test]
    fn run53_s_24000_frames_put_the_ceiling_where_run33_did() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
        ) else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        // The whole point of this capture is its length; a short one would
        // pass every assertion below and mean nothing.
        assert!(
            last >= 23_000,
            "run53's traced length is {last}, wanted 23,000+ — this is the \
             long capture, and a short file here is a wrong file"
        );
        // `RON_DEBUG_UNIT` and the attributed fold, on the long capture:
        // `Built::tick` drops the marks' unit attribution when it folds
        // them into `frame_sites`, and item 204's whole question is *which*
        // unit spends the extra draw. Both cost nothing when unset.
        let unit_window = site_window();
        // **The second squad's membership** (items 249 and 250): 6994's
        // two draws are `1/31`, `1/32` and `1/33`'s, born on 6993 and
        // given an `ATTACK_TO` apiece by the frame's end
        // (`run84_says_great_lakes_6994_belongs_to_the_second_squad`).
        // This is what said the gap was an order rather than a membership
        // bug — all six in **one army** by the frame's end — and it stays
        // as the cheap guard on that half now that
        // `great_lakes_6994_issues_the_second_squad_s_walk_to_the_army`
        // pins the order itself.
        let mut squad_army: Vec<(i64, Option<usize>)> = Vec::new();
        for _ in 0..last {
            let f = built.sim.frame;
            built.tick();
            debug_watch(&built, f);
            debug_builds(&built, f);
            debug_armies(&built, f);
            debug_leader(&built, f);
            if f == GREAT_LAKES_SECOND_SQUAD {
                squad_army = [27, 28, 29, 31, 32, 33]
                    .into_iter()
                    .map(|o| {
                        let at = built.sim.units.iter().position(|u| {
                            u.alive() && i64::from(u.owner) == 1 && i64::from(u.index) == o
                        });
                        (o, at.and_then(|i| built.sim.army_of(i)))
                    })
                    .collect();
            }
            if unit_window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
                for (label, who) in attributed_sites(&built) {
                    eprintln!("  f{f} {who}: {label}");
                }
            }
        }
        assert_eq!(squad_army.len(), 6, "the parting frame was never reached");
        assert!(
            squad_army
                .iter()
                .all(|&(_, a)| a.is_some() && a == squad_army[0].1),
            "the six Great Lakes archers are not one army at              {GREAT_LAKES_SECOND_SQUAD}: {squad_army:?}"
        );
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        let matched = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *ours == trace.labels(*f))
            .count();
        eprintln!(
            "run53: word parts at {first_count}, sequence at {first_part} of {last}; \
             {words} frames on the original's count, {matched} draw for draw \
             (both mostly past the parting — printed, not pinned)"
        );
        // The two parting frames, named — run54 has carried this since
        // East Indies became the second map, and Great Lakes is the
        // headline one. `RON_DEBUG_SITES=<lo>-<hi>` widens it to a window
        // and `RON_DEBUG_SITES=1` prints both sequences whole.
        let window = site_window();
        for (f, ours) in built.frame_sites.iter() {
            let named = *f == first_count || *f == first_part;
            if !named && !window.is_some_and(|(lo, hi)| (lo..=hi).contains(f)) {
                continue;
            }
            let theirs = trace.labels(*f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            eprintln!(
                "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at)
            );
            if std::env::var("RON_DEBUG_SITES").is_ok() {
                for (i, l) in ours.iter().enumerate() {
                    eprintln!("    ours  {i}: {l}");
                }
                for (i, l) in theirs.iter().enumerate() {
                    eprintln!("    thrs  {i}: {l}");
                }
            }
        }
        // **The army's release tick, entry for entry** (item 317,
        // `docs/ARMY.md` §16.8). 7930 is the first `Army::find_target`
        // draw in the whole game and the frame the AI's first army leaves
        // the muster: §6's dispatch `if`s re-read `status`, so
        // `do_mustering`'s `status = 2` reaches `do_marching` in the same
        // tick holding `target_o = -1`, and the original's `iVar4 < 0`
        // arm goes straight to `find_target`. This crate spent nothing
        // here until the arm was modelled. The head is the whole claim —
        // the difficulty gate's coin once, then the per-candidate score
        // twice — and the equality behind it is what says the rest of the
        // frame did not move to pay for it.
        let ours_7930 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 7930)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_7930,
            trace.labels(7930),
            "Great Lakes 7930 is the army's release tick and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_7930.iter().take(3).collect::<Vec<_>>(),
            vec![
                sim::army::SITE_FIND_TARGET_COIN,
                sim::army::SITE_FIND_TARGET_SCORE,
                sim::army::SITE_FIND_TARGET_SCORE,
            ],
            "7930 opens with find_target's coin and two candidate scores"
        );
        // **Great Lakes 8272, the game's first scholar** (item 338,
        // `docs/CITIES.md` §6.5.2). The frame is 38 draws on both sides —
        // the count never parted here — and the only entry that moved was
        // index 33, where `Unit::go_inside`'s forced `CHAR_DEFAULT` sits
        // between the birth at 32 and the `inc_time` wraps. The whole
        // frame is the assertion, because "one draw replaced one draw"
        // is exactly the claim a count cannot make.
        let ours_8272 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 8272)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_8272,
            trace.labels(8272),
            "Great Lakes 8272 is the first scholar's seating and must \
             agree entry for entry"
        );
        assert_eq!(
            (ours_8272.get(32), ours_8272.get(33)),
            (
                Some(&sim::anim::SITE_INIT_REAL.to_string()),
                Some(&sim::anim::SITE_GO_INSIDE.to_string())
            ),
            "8272's 33rd and 34th draws are the birth and the seating"
        );
        // **And the seating's whole schedule**, which is what says the
        // gate is `is_scholar` and not something that happens to hold on
        // one frame: `report.py … when` dates the original's at exactly
        // these fourteen frames in 24,000, and every earlier birth —
        // 6612, 6993, 7212, 7439 and 7674, three units apiece — spends
        // none. Only the frames below the word are asserted; past it both
        // streams are nobody's (item 89(c)).
        let seatings: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| l == sim::anim::SITE_GO_INSIDE))
            .map(|(f, _)| *f)
            .filter(|f| *f < LONG_WORD_GREAT_LAKES)
            .collect();
        // **Against the original's own list, not a literal.** Item 352's
        // standing trio taught the shape: a literal here is
        // window-length arithmetic and breaks the moment the word moves
        // — 8680's scholar came under the word on item 354 and this
        // assertion, not the mechanic, is what failed. The trace's own
        // seatings below the word are the right right-hand side, and the
        // literal stays beside it so a reader sees real frames.
        let theirs_seatings: Vec<i64> = (0..LONG_WORD_GREAT_LAKES)
            .filter(|f| {
                trace
                    .labels(*f)
                    .iter()
                    .any(|l| l == sim::anim::SITE_GO_INSIDE)
            })
            .collect();
        assert_eq!(
            seatings, theirs_seatings,
            "Great Lakes' scholar seatings below the word are not the \
             original's own"
        );
        assert_eq!(
            seatings,
            vec![8272, 8680, 9_087, 9_201, 9_322],
            "below the word Great Lakes seats five scholars, on 8272, 8680, \
             9087, 9201 and 9322"
        );
        // **Great Lakes 9134, the snap arm's blocked stand** (item 360,
        // `docs/COLLISION.md` §5.4). The frame is **one draw on each
        // side** and the count never parted here: the original's was a
        // bare `5dac7a`, `Guy::set_anim+0x97a` under a twelfth `ebp`
        // chain the table did not name, and this crate spent the eleventh
        // — `Unit::move_step+0x823`, §5's partial-step block — where the
        // original spends `+0x4e2`, the snap's. One draw replacing one
        // draw is exactly the claim a count cannot make, so the frame is
        // the assertion and the label beside it.
        let ours_9134 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 9134)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_9134,
            trace.labels(9134),
            "Great Lakes 9134 is `move_step`'s snap arm and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_9134.as_slice(),
            [sim::anim::SITE_SNAP_BLOCKED.to_string()],
            "9134 is one draw, and it is the snap's stand and not §5's"
        );
        // **Great Lakes 8582, the AI's first market draw** (item 348,
        // `docs/AI.md` §40). `make_stuff` calls `use_market` first thing,
        // so the draw sits at index 0 and the frame is otherwise the two
        // expiry walks it always was — which is the claim a count cannot
        // make, so the whole frame is the assertion.
        let ours_8582 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 8582)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        assert_eq!(
            ours_8582,
            trace.labels(8582),
            "Great Lakes 8582 is the market's first draw and must agree \
             entry for entry"
        );
        assert_eq!(
            ours_8582.first(),
            Some(&sim::ai_make::SITE_MARKET_SELL.to_string()),
            "8582 opens with `use_market`'s sell rotation"
        );
        // **And the whole schedule below the word**, which is what says
        // the gate is Coinage and not something that happens to hold on
        // one frame: `report.py … when Leader::use_market` dates the
        // original's first two at 8582 and 8585 — two leaders three
        // frames apart — and **none** earlier in the 24,000. This crate's
        // own shortfall vector met the sell branch's condition on 8185
        // and 8382 as well, and the tech is the only thing that refuses
        // them. Past the word both streams are nobody's (item 89(c)).
        let markets: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| s.iter().any(|l| l == sim::ai_make::SITE_MARKET_SELL))
            .map(|(f, _)| *f)
            .filter(|f| *f < LONG_WORD_GREAT_LAKES)
            .collect();
        // The original's own list below the word, for the same reason as
        // the seatings above: 8782 and 8982 came under the word on item
        // 354 and a literal cannot follow it.
        let theirs_markets: Vec<i64> = (0..LONG_WORD_GREAT_LAKES)
            .filter(|f| {
                trace
                    .labels(*f)
                    .iter()
                    .any(|l| l == sim::ai_make::SITE_MARKET_SELL)
            })
            .collect();
        assert_eq!(
            markets, theirs_markets,
            "Great Lakes' market draws below the word are not the \
             original's own"
        );
        assert_eq!(
            markets,
            vec![8582, 8585, 8782, 8982, 9_182, 9_382],
            "below the word Great Lakes takes exactly six market draws — and \
             9182 is item 385's own: the frame the sequence used to part on \
             is a `use_market` sell on both sides now"
        );
        assert!(
            first_count >= LONG_WORD_GREAT_LAKES && first_part >= LONG_WORD_GREAT_LAKES,
            "run53's ceiling fell: word {first_count}, sequence {first_part}; \
             the floor is {LONG_WORD_GREAT_LAKES} on both"
        );
    }

    /// **Great Lakes 8186 is `Army::find_target`'s two-unit probe, and the
    /// draw stream bounds its cast** — item 324, `docs/COMBAT.md` §17.
    ///
    /// The word and the sequence both part here, six draws against
    /// fifty-four, and forty-six of the fifty-four are one address:
    /// `Unit::find_attack_pos@00601280+0xea9`, which
    /// [`trace::SITES`](crate::trace::SITES) printed raw until this item
    /// named it. `report.py … when` says the address is reached on
    /// **two** frames of run53's 24,000 — this one and 17656 — and both
    /// are `Army::find_target` frames, which is what says the two belong
    /// to one mechanic rather than to combat in general.
    ///
    /// **What the frame is.** `find_target@006f69b0:1120`-`1199` is the
    /// probe `crates/sim/src/army.rs` files as "a group order (seam);
    /// nothing changes": two units off the army (`ArmyData::get_unit(0)`
    /// and `get_unit(count − 1)`), `Groups::push_group`, a
    /// `find_building(SEARCH_FRIENDLY, …, 0x200, FILTER_TYPE, 0x1a1)` —
    /// type 417 is `FARM` — then `action_stance(5)`,
    /// `action_attack(farm, mandatory)` and `action_move_to(leader)`.
    /// `Group::action_attack@00712490:215` calls `find_attack_pos` once,
    /// on the leader; every member that takes the order then runs its own
    /// call from `Unit::fight@005fd4d0+0xcb4`'s chase.
    ///
    /// **The bound is the assertion.** One call cannot draw more than
    /// [`sim::fight::ATTACK_POS_CAP_RANGED`] times, so forty-six draws
    /// need at least four calls — a lower bound on the cast taken from
    /// the draw stream **alone**, with no hypothesis about who is on the
    /// frame. run19's blocks 8186 and 8187 then name the cast
    /// independently: six of player 1's units take the mandatory attack
    /// order on the farm `0/2004`. Four ≤ six, and if a later change
    /// makes either number move the wrong way this says so.
    #[test]
    fn run53_s_8186_is_find_target_s_probe_and_its_ring_walks() {
        let (Some(dpath), Some(tr)) = (
            dump("gamelog-run19-window-8174-8192.txt"),
            trace("rontrace-run53.log"),
        ) else {
            eprintln!("skipping: no run19/run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        use sim::fight::{ATTACK_POS_CAP_RANGED, SITE_ATTACK_POS_FIGHT, SITE_ATTACK_POS_GROUP};
        // **The frame, entry for entry, off the original's own trace.**
        let labels = tr.labels(PROBE_FRAME);
        let want: Vec<&str> = std::iter::once(sim::army::SITE_FIND_TARGET_COIN)
            .chain(std::iter::repeat_n(sim::army::SITE_FIND_TARGET_SCORE, 3))
            .chain(std::iter::repeat_n(SITE_ATTACK_POS_GROUP, 2))
            .chain(std::iter::once(sim::anim::SITE_IDLE_ANIMAL))
            .chain(std::iter::repeat_n(SITE_ATTACK_POS_FIGHT, 46))
            .chain(std::iter::once(sim::farms::SITE_CHANCE))
            .collect();
        assert_eq!(
            labels, want,
            "Great Lakes {PROBE_FRAME} is not the probe's frame any more"
        );
        // 17656 is the same shape with a different cast — the second and
        // last frame in 24,000 that reaches the address at all. It is here
        // so a reading of 8186 alone cannot be mistaken for the mechanic.
        let other = tr.labels(17_656);
        assert_eq!(
            (
                other.iter().filter(|l| *l == SITE_ATTACK_POS_GROUP).count(),
                other.iter().filter(|l| *l == SITE_ATTACK_POS_FIGHT).count(),
            ),
            (4, 10),
            "17656's two ring-walk families moved"
        );
        // **The bound, from the stream alone.**
        let chases = labels
            .iter()
            .filter(|l| *l == SITE_ATTACK_POS_FIGHT)
            .count();
        let at_least = chases.div_ceil(ATTACK_POS_CAP_RANGED);
        assert_eq!(
            (chases, at_least),
            (46, 4),
            "the chase family's size or its per-call ceiling moved"
        );
        // **And the cast, from the dump.** The probe's orders land in the
        // block *after* the frame that issues them, so 8186 → 8187 is the
        // value diff beside the word.
        let text = crate::capture::read(&dpath);
        let log = Log::parse(&text);
        let states = log.frame_states();
        let at = |n: i64| states.iter().find(|f| f.n == n).expect("block");
        let (before, after) = (at(PROBE_FRAME), at(PROBE_FRAME + 1));
        let attackers = |f: &crate::gamelog::Frame| -> Vec<(i64, i64)> {
            f.units
                .iter()
                .filter(|u| {
                    u.orders.iter().any(|o| {
                        o.named_index() == Some(10)
                            && o.mandatory == Some(1)
                            && (o.ox, o.whom, o.uid) == (Some(2004), Some(0), Some(4))
                    })
                })
                .map(|u| (u.who, u.o))
                .collect()
        };
        assert!(
            attackers(before).is_empty(),
            "the farm is already under a mandatory attack order at {PROBE_FRAME}: {:?}",
            attackers(before)
        );
        assert_eq!(
            attackers(after),
            PROBE_CAST,
            "the probe's cast moved — {} units take the farm order",
            attackers(after).len()
        );
        assert!(
            at_least <= PROBE_CAST.len(),
            "the draw stream needs {at_least} `find_attack_pos` calls and the \
             dump names only {} units to make them",
            PROBE_CAST.len()
        );
        // **Where each of them is sent** — the chase's own answer, one
        // plain `MOVEORDER` a unit, and the half of this frame a draw-for-
        // draw agreement could never check on its own.
        let sent: Vec<(i64, i64, i64, i64)> = after
            .units
            .iter()
            .filter(|u| PROBE_CAST.contains(&(u.who, u.o)))
            .map(|u| {
                let m = u
                    .orders
                    .iter()
                    .find(|o| o.kind == "MOVEORDER")
                    .expect("the cast's chase move");
                (u.who, u.o, m.x.unwrap_or(-1), m.y.unwrap_or(-1))
            })
            .collect();
        assert_eq!(
            sent, PROBE_SENT,
            "the six chase destinations moved; the farm `0/2004` stands at \
             (2112, 31296)"
        );
        // **Where the six come from**: `get_unit(0)` and `get_unit(n − 1)`
        // are the army group's first and last **captains**, and
        // `Group::add`'s subordinate recursion (`docs/GROUPS.md` §4.1)
        // brings each one's two followers. The army's standing group is
        // fifteen units in five squads of three, and that is what makes
        // two adds into six orders.
        let squad: Vec<(i64, i64)> = before
            .units
            .iter()
            .filter(|u| u.who == 1 && u.group == Some(64))
            .map(|u| (u.o, u.o_up.unwrap_or(i64::MIN)))
            .collect();
        assert_eq!(
            squad, PROBE_SQUADS,
            "the army's standing group is no longer five squads of three"
        );
        // The stance the probe sets, and the group it pushes: both are
        // `action_stance(5)` and `push_group`'s slot, and both are in the
        // record, so neither is a reading.
        let stance = |f: &crate::gamelog::Frame, k: (i64, i64)| {
            f.units
                .iter()
                .find(|u| (u.who, u.o) == k)
                .and_then(|u| Some((u.stance?, u.group?)))
        };
        for k in PROBE_CAST {
            assert_eq!(
                (stance(before, *k), stance(after, *k)),
                (Some((0, 64)), Some((5, 65))),
                "{k:?} does not take the probe's stance and group"
            );
        }
    }

    /// The frame Great Lakes' long word has parted on since item 323 — the
    /// probe's own. Named because three assertions share it and its
    /// successor block; the floor itself is
    /// [`LONG_WORD_GREAT_LAKES`], which this must not be confused with.
    const PROBE_FRAME: i64 = 8186;

    /// The six units of player 1 that take the probe's mandatory attack
    /// order on the farm `0/2004`, from run19's block 8187. `1/40` is the
    /// leader — every member's `GroupMoveOrder` carries `oxx 40` — and its
    /// own `form_id` is 3 rather than 0.
    const PROBE_CAST: &[(i64, i64)] = &[(1, 27), (1, 28), (1, 29), (1, 40), (1, 41), (1, 42)];

    /// The army's standing group at block 8186 — `(o, o_up)` for every
    /// unit of player 1 in slot 64, in the record's own order. Five
    /// captains at `o_up −1`, each followed by two units chained on the
    /// one before, which is what turns the probe's two `Group::add` calls
    /// into six orders (`docs/GROUPS.md` §4.1, `docs/COMBAT.md` §17.1).
    const PROBE_SQUADS: &[(i64, i64)] = &[
        (27, -1),
        (28, 27),
        (29, 28),
        (31, -1),
        (32, 31),
        (33, 32),
        (34, -1),
        (35, 34),
        (36, 35),
        (37, -1),
        (38, 37),
        (39, 38),
        (40, -1),
        (41, 40),
        (42, 41),
    ];

    /// Where the chase sends each of them: the plain `MOVEORDER` that is
    /// in front of the attack order on block 8187 and was not there on
    /// 8186. `1/40`, `1/41` and `1/42` are put within half a tile of the
    /// farm's own corner; `1/27`, `1/28` and `1/29` are sent to a second
    /// cluster two thousand units out, which this item has **not**
    /// established the arm of.
    /// **The crate side of the probe** (item 328, 2026-09-17) — the
    /// answer `crates/sim` gives to the frame the test above pins the
    /// original's answer to.
    ///
    /// `run53_s_8186_is_find_target_s_probe_and_its_ring_walks` says what
    /// the original does on 8186 and takes no simulation at all; this
    /// ticks the simulation to 8187 and asserts that the six units the
    /// probe sends are walking to **run19's own coordinates**, to the
    /// unit, out of `docs/COMBAT.md` §17's ring walk.
    ///
    /// **Why a destination and not a draw count.** Six calls that draw
    /// forty-six times between them can be six wrong ring walks whose
    /// budgets happen to add up: the first build of §17 drew
    /// 15 + 15 + 15 + 2 + 2 + 2 = 51 and put *all three* archers on one
    /// spot, and the build before the `find_ordered_collision` group pass
    /// went in would have agreed with the trace on nothing but the shape.
    /// A coordinate is what tells a right answer from a lucky one, and
    /// these six are the dump's, not this crate's.
    ///
    /// Made to fail on purpose by moving the arc's entry step off
    /// `(steps_per_side / 2) + 1`: the three archers move together, to
    /// `(4392, 29784)`, `(4488, 29976)` and `(4248, 29544)`, and the
    /// three melee figures — whose `steps_per_side` is 1, so the arc has
    /// nowhere to go — do not move at all. Which is itself worth knowing:
    /// half this cast cannot see an error in the arc, and a test written
    /// on `1/40`-`1/42` alone would have passed.
    #[test]
    fn great_lakes_8186_sends_the_probe_s_six_where_the_original_does() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame <= PROBE_FRAME {
            built.tick();
        }
        let ours: Vec<(i64, i64, i64, i64)> = PROBE_SENT
            .iter()
            .map(|&(who, o, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .unwrap_or_else(|| panic!("no live {who}/{o} at {}", PROBE_FRAME + 1));
                // The chase's move goes in **front** of the mandatory
                // attack order, so it is the unit's current order.
                let m = built.sim.units[u]
                    .orders
                    .iter()
                    .find_map(|od| match od.body {
                        sim::orders::Body::Move(m) => Some(m),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("{who}/{o} carries no move at {}", PROBE_FRAME + 1));
                (who, o, i64::from(m.dest.x), i64::from(m.dest.y))
            })
            .collect();
        assert_eq!(
            ours, PROBE_SENT,
            "the probe's six are not standing where run19 puts them"
        );
        // And the cast is a mandatory attack on the farm, which is what
        // makes the move a chase rather than a walk.
        for &(who, o, _, _) in PROBE_SENT {
            let u = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                .expect("the cast");
            assert!(
                built.sim.units[u].combat.mandatory,
                "{who}/{o} is not under a mandatory attack order"
            );
            assert_eq!(
                built.sim.units[u].combat.stance,
                sim::combat::Stance::HoldFire,
                "{who}/{o}'s stance is not the probe's `action_stance(5)`"
            );
        }
    }

    /// **The chase is planned on the frame it is ordered, and a blocked
    /// chaser is delayed rather than cancelled** — item 329,
    /// `docs/ORDERS.md` §7.10 and `docs/PATHFINDER.md` §21.
    ///
    /// Its sibling above pins where the probe's six are *sent*. This pins
    /// **when** they set off and what happens to the one that cannot, and
    /// both halves were wrong until this item: the crate planned a frame
    /// late (so nobody moved on 8187) and then killed the chase of the
    /// unit whose 48-grid search failed (so `1/28` was back in `fight` on
    /// 8188 with an empty stack).
    ///
    /// Every number here is run19's own. Blocks 8187-8190 of
    /// `gamelog-run19-window-8174-8192.txt` carry:
    ///
    /// - block **8187** (the end of sim-frame 8186, the frame the chase is
    ///   ordered): all six with a path stack already on them — 43, 43, 43,
    ///   94, 47, 46 entries — and the attack order's `UNITORDER flags 20`,
    ///   which is `ACTION | 0x10`, the re-entry latch.
    /// - blocks **8188**-**8190**: `1/28` frozen at `(36456, 23592)` with
    ///   `collide 1`, `collide_o 36`, `retry` 8 → 7 → 6 and `safe` 30 → 29
    ///   → 28, while `1/27` and `1/29` walk.
    ///
    /// **The rolled 8 is the row that matters.** A draw spent in the right
    /// place with the wrong arithmetic agrees with the trace and disagrees
    /// here; so does a delay taken from `pause` (`+0x18`) instead of
    /// `retry` (`+0x1c`), which is the mistake the four adjacent counters
    /// invite.
    ///
    /// Made to fail on purpose twice. With `Sim::chase_reentry`'s body
    /// replaced by a `return` it stops on `1/27`'s latch — the six plan
    /// nothing on 8186 and nothing is latched. With
    /// `Sim::roll_upath_retry`'s roll suppressed it stops on the delay
    /// table: `1/28` keeps colliding with `1/36`, so the collision
    /// assertion still holds, and what gives it away is `retry 0`,
    /// `safe 0` and a unit that has walked on.
    #[test]
    fn great_lakes_8186_plans_the_chase_and_8187_delays_the_blocked_one() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let find = |built: &Built, who: i64, o: i64| -> usize {
            built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                .unwrap_or_else(|| panic!("no live {who}/{o}"))
        };
        while built.sim.frame <= PROBE_FRAME {
            built.tick();
        }
        // The frame the chase was ordered is over: every one of the six is
        // already carrying a plan, and the attack under it is latched.
        for &(who, o, _, _) in PROBE_SENT {
            let u = find(&built, who, o);
            assert!(
                !built.sim.units[u].path.is_empty(),
                "{who}/{o} planned nothing on {PROBE_FRAME}; run19's block \
                 {} has all six with a path stack",
                PROBE_FRAME + 1
            );
            let a = built
                .sim
                .action_of(u)
                .unwrap_or_else(|| panic!("{who}/{o} has no action"));
            assert_ne!(
                built.sim.units[u].orders[a].flags & sim::orders::flag::FIGHT_REENTRY,
                0,
                "{who}/{o}'s action is not latched; run19 prints `flags 20`"
            );
        }
        // Then the three frames run19 dumps of the blocked chaser and its
        // two walking neighbours, value for value.
        const BLOCKED: &[(i64, i64, i64, i64, i64)] = &[
            // frame, x, y, retry, safe
            (8187, 36_456, 23_592, 8, 30),
            (8188, 36_456, 23_592, 7, 29),
            (8189, 36_456, 23_592, 6, 28),
        ];
        const WALKING: &[(i64, i64, i64, i64, i64)] = &[
            // frame, 1/27's x and y, 1/29's x and y
            (8187, 36_486, 23_472, 36_578, 23_324),
            (8188, 36_477, 23_484, 36_556, 23_344),
            (8189, 36_468, 23_496, 36_534, 23_364),
        ];
        let mut blocked: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
        let mut walking: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
        for _ in 0..3 {
            let f = built.sim.frame;
            built.tick();
            let u = find(&built, 1, 28);
            let unit = &built.sim.units[u];
            let retry = match unit.orders.front().map(|o| o.body) {
                Some(sim::orders::Body::Move(m)) => i64::from(m.retry),
                _ => -1,
            };
            blocked.push((
                f,
                i64::from(unit.pos.x),
                i64::from(unit.pos.y),
                retry,
                i64::from(unit.safe),
            ));
            assert_eq!(
                (unit.collide, unit.collide_o),
                (1, 36),
                "{f}: `1/28` is not blocked by `1/36`, which run19 has it \
                 colliding with on every one of these frames"
            );
            let (a, b) = (find(&built, 1, 27), find(&built, 1, 29));
            walking.push((
                f,
                i64::from(built.sim.units[a].pos.x),
                i64::from(built.sim.units[a].pos.y),
                i64::from(built.sim.units[b].pos.x),
                i64::from(built.sim.units[b].pos.y),
            ));
        }
        assert_eq!(
            blocked, BLOCKED,
            "`1/28`'s delay is not run19's: (frame, x, y, retry, safe)"
        );
        assert_eq!(
            walking, WALKING,
            "the two walking chasers are not on run19's coordinates: \
             (frame, 1/27 x, y, 1/29 x, y)"
        );
    }

    /// **`Unit::fight`'s entry puts the unit on its cell centre** (item
    /// 336, 2026-09-17, `docs/ORDERS.md` §7.11).
    ///
    /// The two tests above pin where the probe's six are *sent* and when
    /// they set off. This pins what happens to the three that were
    /// **mid-cell** when the order arrived. `Unit::fight@005fd4d0`'s first
    /// statement, `LAB_005fd648`, is
    ///
    /// ```text
    /// set_new_location(this, div_3_table[x >> 4] * 0x30 + 0x18,
    ///                        div_3_table[y >> 4] * 0x30 + 0x18, 1, 0)
    /// ```
    ///
    /// — `(v / 48) * 48 + 24` on both axes, the 48-unit cell centre. The
    /// probe's three archers were already standing on theirs, so nothing
    /// visible happens to them; `1/40`, `1/41` and `1/42` were walking, and
    /// run19's blocks 8186 and 8187 catch all three crossing onto the grid
    /// in one frame:
    ///
    /// ```text
    ///          block 8186            block 8187
    ///   1/40   (39133, 21131)   →    (39144, 21144)
    ///   1/41   (36915, 23248)   →    (36936, 23256)
    ///   1/42   (36883, 23073)   →    (36888, 23064)
    /// ```
    ///
    /// Every one of the six coordinates is its own axis's `(v / 48) * 48 +
    /// 24`, and none of the three is a step: `1/42`'s is 10 units *back*
    /// along the march it had been walking at 26 a frame.
    ///
    /// **Why the last row is the one that matters.** Without the snap this
    /// crate left all three where they stood, and from 8187 on they walked
    /// the original's own velocity from a position offset by a constant —
    /// `1/42` by exactly `(−5, +9)` — for the rest of the game. A draw
    /// stream cannot see that: it agreed for fifteen more frames and then
    /// parted on a single blocked stand at 8201. Block **8201** is run19's
    /// last, 15 frames past the snap, and it is asserted here for all
    /// three because a position is what tells a right walk from a lucky
    /// one.
    ///
    /// Made to fail on purpose by deleting the snap: the pre-snap row
    /// passes, `1/40` reads `(39133, 21131)` where run19 has `(39144,
    /// 21144)`, and 8201 is out by the same constant on all three.
    #[test]
    fn great_lakes_8186_snaps_the_probe_s_walkers_onto_their_cell_centres() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The three of the probe's six that were walking when it fired.
        const WALKERS: [(i64, i64); 3] = [(1, 40), (1, 41), (1, 42)];
        let at = |built: &Built| -> Vec<(i64, i64)> {
            WALKERS
                .iter()
                .map(|&(who, o)| {
                    let u = built
                        .sim
                        .units
                        .iter()
                        .position(|u| {
                            u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o
                        })
                        .unwrap_or_else(|| panic!("no live {who}/{o}"));
                    let p = built.sim.units[u].pos;
                    (i64::from(p.x), i64::from(p.y))
                })
                .collect()
        };
        // Block 8186 — the frame before the probe, all three mid-cell.
        while built.sim.frame < PROBE_FRAME {
            built.tick();
        }
        assert_eq!(
            at(&built),
            vec![(39_133, 21_131), (36_915, 23_248), (36_883, 23_073)],
            "the three walkers are not on run19's block {PROBE_FRAME}              coordinates, so the snap below would prove nothing"
        );
        // And not one of them is on a cell centre, which is the whole
        // premise: a snap is invisible on a unit that is already there.
        for &(x, y) in &at(&built) {
            assert!(
                (x % 48, y % 48) != (24, 24),
                "({x}, {y}) is already a cell centre at block {PROBE_FRAME}"
            );
        }
        // Block 8187 — the probe's own frame, and the snap.
        built.tick();
        assert_eq!(
            at(&built),
            vec![(39_144, 21_144), (36_936, 23_256), (36_888, 23_064)],
            "the probe's walkers are not on run19's block {} coordinates",
            PROBE_FRAME + 1
        );
        for &(x, y) in &at(&built) {
            assert_eq!(
                (x % 48, y % 48),
                (24, 24),
                "({x}, {y}) is not a 48-unit cell centre"
            );
        }
        // Blocks 8188 and 8189: the walk resumes from the snapped point.
        const AFTER: [[(i64, i64); 3]; 2] = [
            [(39_117, 21_155), (36_932, 23_284), (36_877, 23_091)],
            [(39_090, 21_165), (36_928, 23_312), (36_866, 23_118)],
        ];
        for (i, want) in AFTER.iter().enumerate() {
            built.tick();
            assert_eq!(
                at(&built),
                want.to_vec(),
                "the walkers are not on run19's block {} coordinates",
                PROBE_FRAME + 2 + i as i64
            );
        }
        // **Block 8201, run19's last** — the frame the long word parted on
        // before this item, and the reason the snap is worth a test.
        const LAST_BLOCK: i64 = 8201;
        while built.sim.frame < LAST_BLOCK {
            built.tick();
        }
        assert_eq!(
            at(&built),
            vec![(38_766, 21_288), (36_880, 23_648), (36_734, 23_442)],
            "the walkers have drifted off run19's block {LAST_BLOCK}              coordinates"
        );
    }

    const PROBE_SENT: &[(i64, i64, i64, i64)] = &[
        (1, 27, 4344, 29736),
        (1, 28, 4440, 29880),
        (1, 29, 4200, 29496),
        (1, 40, 2424, 30888),
        (1, 41, 2280, 30888),
        (1, 42, 2568, 31320),
    ];

    /// The frame Great Lakes' AI army retargets to London — `PROBE_FRAME`'s
    /// successor, named on the frame by item 350 and closed by item 352.
    const MUSTER_FRAME: i64 = 8442;

    /// **run97 block 8443** — the nine units of player 1's army, each with
    /// its `x_internal`/`y_internal` and the **bottom** of its path stack,
    /// which is the goal (`gamelog::PathDump`). Their leading order is the
    /// `GROUPATTACKTOORDER` whose `orig` is (44851, 22480) for all nine;
    /// `1/37` is the leader (`oxx 37`) and its own goal *is* that origin.
    const MUSTER_CAST: &[(i64, i64, i64, i64, i64)] = &[
        (1, 31, 36_692, 23_098, 44_709),
        (1, 32, 36_644, 23_194, 44_576),
        (1, 33, 36_741, 22_954, 44_841),
        (1, 34, 36_389, 23_815, 45_106),
        (1, 35, 36_340, 23_958, 44_973),
        (1, 36, 36_437, 23_672, 45_238),
        (1, 37, 36_575, 23_689, 44_851),
        (1, 38, 36_534, 23_820, 44_718),
        (1, 39, 36_617, 23_557, 44_983),
    ];

    /// The y of each row of [`MUSTER_CAST`], same order — kept apart only
    /// because a five-tuple of coordinates reads worse than two.
    const MUSTER_GOAL_Y: &[i64] = &[
        22_698, 22_755, 22_640, 22_527, 22_584, 22_469, 22_480, 22_537, 22_422,
    ];

    /// **Great Lakes 8442 musters the army on the original's own cell** —
    /// item 352, `docs/ARMY.md` §13 and §16.9.
    ///
    /// 8442 is the `Army::find_target` that follows `PROBE_FRAME`'s probe.
    /// Both sides spend its **two** `find_target+0x7df` score draws, so the
    /// candidates, their order and the rolls all agreed; what did not was
    /// where the army then went. This crate mustered at cell (50, 27) and
    /// run97 block 8443's `GROUPATTACKTOORDER` carries `orig` (44851,
    /// 22480) for all nine members — which is cell (58, 29)'s centre
    /// stepped one tile (`0xc0 × num_groups`) along `muster_angle` after
    /// `find_target`'s `+= 0x80000000`, and is that for **no other cell on
    /// the map**. So (58, 29) is the original's muster, read off a record
    /// that never names an army.
    ///
    /// Two predicates of §13's ring were why, and each is one token:
    ///
    /// - the same-owner spacing test is `vector_dist <= 4`, not `< 4`
    ///   (`6f633a`: `cmp $0x4` then **`jle`**; `<= 2` on the enemy arm at
    ///   `6f6313`). Army 1/2's own muster sat exactly 4 from a cell it
    ///   therefore kept, which left (50, 27) free for army 1/1;
    /// - `BuildType::mask_me@006312a0`'s **first** write, `W.flags |=
    ///   0x4000` on the building's own cell, was in `docs/CITIES.md` §3.6
    ///   from the first reading and in no code. It is the ring score's one
    ///   reader in the export, so every cell of the AI's own town scored as
    ///   open ground — and six ring-7 candidates tied at the maximum 2,304
    ///   where the original sees 1,024 to 2,048.
    ///
    /// With both, the ring's best is (58, 29) at 2,048 and the nine units
    /// stand on run97's own coordinates with run97's own path goals on the
    /// very frame — `1/37`'s ten-segment path included. The band item 347
    /// parked as a free `walk_variant` choice is **empty** below the word
    /// (`run97_s_window_clocks_are_the_original_s`).
    ///
    /// Made to fail on purpose, one fix at a time. With the spacing test
    /// back at `< 4` the muster is **(50, 27)** and every one of the nine
    /// rows below is off; with `mask_me`'s flag write removed and the
    /// spacing test kept it is **(55, 29)** — a cell whose own 3 × 3 holds
    /// two of the AI's buildings — which is the half that says the flag is
    /// doing work rather than riding along.
    #[test]
    fn great_lakes_8442_musters_the_army_where_the_original_does() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame <= MUSTER_FRAME {
            built.tick();
        }
        // The army that holds the nine — one slot, found by its members
        // rather than by its number, so a pool reshuffle says so here
        // instead of reading a stale slot.
        let u31 = built
            .sim
            .units
            .iter()
            .position(|u| u.alive() && u.owner == 1 && u.index == 31)
            .expect("1/31 is alive at 8442");
        let slot = built.sim.army_of(u31).expect("1/31 is in an army at 8442");
        let a = &built.sim.armies[1].list[slot];
        assert_eq!(
            (a.muster.x, a.muster.y),
            (58, 29),
            "the army's muster is not run97 block 8443's own cell"
        );
        assert_eq!(
            a.num_units, 9,
            "the army is not the nine run97 block 8443 orders"
        );
        // And `find_target`'s own two writes on the frame it takes a
        // target: the target is London's centre and `rally_dist` is
        // `0x1200` because the target changed (§12, "Taking it").
        assert_eq!(
            a.rally_dist, 0x1200,
            "the retarget did not re-arm rally_dist"
        );
        let rows: Vec<(i64, i64, i64, i64, i64)> = MUSTER_CAST
            .iter()
            .map(|&(who, o, _, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .unwrap_or_else(|| panic!("no live {who}/{o} at {MUSTER_FRAME}"));
                let g = built.sim.units[u]
                    .path
                    .first()
                    .unwrap_or_else(|| panic!("{who}/{o} carries no path at {MUSTER_FRAME}"));
                (
                    who,
                    o,
                    i64::from(built.sim.units[u].pos.x),
                    i64::from(built.sim.units[u].pos.y),
                    i64::from(g.to.x),
                )
            })
            .collect();
        assert_eq!(
            rows, MUSTER_CAST,
            "the army's nine are not standing where run97 block 8443 puts them"
        );
        let ys: Vec<i64> = MUSTER_CAST
            .iter()
            .map(|&(who, o, _, _, _)| {
                let u = built
                    .sim
                    .units
                    .iter()
                    .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                    .expect("the cast");
                i64::from(built.sim.units[u].path.first().expect("a path").to.y)
            })
            .collect();
        assert_eq!(
            ys, MUSTER_GOAL_Y,
            "the army's nine do not hold run97 block 8443's own path goals"
        );
    }

    /// **Great Lakes 9134's value diff: `1/32` stands, and three fields
    /// say which arm stood it** (item 360, `docs/COLLISION.md` §5.4,
    /// §8.9).
    ///
    /// The word moved on a draw-site name, and a draw stream can agree on
    /// a wrong destination for a long time; this is the value comparison
    /// beside it. run97 dumps `UNITS=3`, so the three fields that part the
    /// snap arm from §5's are all on disk:
    ///
    /// - `x_internal`/`y_internal` — the snap arm takes **no step**;
    /// - `coll_x`/`coll_y` — the probe *did* return a hard collision, so
    ///   the point it refused is stored into the order;
    /// - `collide_o`/`collide_who` — and are then **cleared** two
    ///   instructions later (`field_0x8a = 0xffff`, `field_0xb3 = 0xff`),
    ///   which §5's arm never does;
    /// - `collide_frame` — nine hundred frames stale, because
    ///   `resolve_unit_collision` did not run. The original's own resolve
    ///   lands on 9135 and dates it there.
    ///
    /// The path stack is the fifth: `1/32` walks its formation slot in
    /// ~24-unit hops and the hop it snaps to is a **middle** leg, so the
    /// pop returns 1 and the real goal `(44576, 22755)` is still on the
    /// stack at 9135. That is the half `collide::tests::a_blocked_snap_
    /// stands_and_eats_its_waypoint_without_resolving` cannot reach, its
    /// own waypoint being the final one.
    ///
    /// Made to fail by taking the arm back out: the 9135 row then reads
    /// `pos (42792, 22824)` and an empty path, which is the original's
    /// **9136**.
    #[test]
    fn great_lakes_9134_is_the_snap_arm_s_blocked_stand() {
        const LAST: i64 = 9_136;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut rows = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..LAST {
            built.tick();
            // The block numbered `f + 1` is the state this tick left
            // behind, the same convention every run97 test here uses.
            if !(9_133..LAST).contains(&f) {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f + 1)
                .map(|(_, b)| b)
            else {
                continue;
            };
            let (units, _, _) = crate::gamelog::records(block, false);
            let Some(theirs) = units.iter().find(|u| u.who == 1 && u.o == 32) else {
                continue;
            };
            let ours = built
                .sim
                .units
                .iter()
                .find(|x| x.alive() && i64::from(x.owner) == 1 && i64::from(x.index) == 32)
                .unwrap_or_else(|| {
                    panic!("run97 block {} has 1/32 and this crate does not", f + 1)
                });
            rows += 1;
            let mine = (
                i64::from(ours.pos.x),
                i64::from(ours.pos.y),
                i64::from(ours.collide_o),
                i64::from(ours.collide_who),
                ours.collide_frame,
                ours.orders
                    .front()
                    .and_then(|o| match &o.body {
                        sim::orders::Body::Move(m) => Some(m.coll),
                        _ => None,
                    })
                    .flatten()
                    .map(|c| (i64::from(c.x), i64::from(c.y))),
                ours.path.len(),
            );
            let theirs_coll = theirs
                .current_order()
                .and_then(|o| match (o.coll_x, o.coll_y) {
                    (Some(0), Some(0)) | (None, _) | (_, None) => None,
                    (Some(x), Some(y)) => Some((x, y)),
                });
            let yours = (
                theirs.pos.x,
                theirs.pos.y,
                theirs.collide_o.unwrap_or(-1),
                theirs.collide_who.unwrap_or(-1),
                theirs.collide_frame.unwrap_or(0),
                theirs_coll,
                theirs.path.len(),
            );
            if mine != yours {
                wrong.push(format!("block {}: ours {mine:?} theirs {yours:?}", f + 1));
            }
        }
        assert_eq!(rows, 3, "run97's blocks 9134-9136 are not all here: {rows}");
        assert!(
            wrong.is_empty(),
            "Great Lakes 9134 is `move_step`'s snap arm standing still, and \
             the record says so field for field: {wrong:?}"
        );
    }

    /// **run97's build queues, every building, every frame of the window**
    /// — Great Lakes `[8030, 9349]`, item 358.
    ///
    /// The capture's `BUILDDATA` carries a whole `BUILDQUEUE` per
    /// building — `queue_size` slots of `type`, `job_counter` and three
    /// `(good, cost)` pairs — and `queued` says how many of them are
    /// live. Nothing had ever compared them on this map. The first run of
    /// this widening found **one** divergence in 1,320 frames and it was
    /// the word's own: the AI's University holds three scholars on 8985
    /// and this crate could pay for one.
    ///
    /// That is the whole case for widening the record rather than the
    /// field. The word's frame said only "a market draw for an expiry
    /// draw"; the queue said *what the original bought with the wealth
    /// this crate did not have*, which named `do_sell` in one line
    /// (`docs/ECONOMY.md` §12).
    ///
    /// **`1/2020` closed, item 362.** The third scholar was never a
    /// second *purchase*: 8985 costs three `make_stuff+0x221` and
    /// **zero** `+0x63d` on both sides, so the original's three arrive
    /// in one `make_this`, and the quantity is the make list's own
    /// `num`. `create_units` computes it per arm and this crate's
    /// `civilian_value` threw it away, so every civilian went out as a
    /// batch of one. `docs/AI.md` §42.
    ///
    /// The residue is **floored, not asserted away**, and the one row
    /// left is 9182's own:
    ///
    /// - ~~`1/2007` from 9182~~ — a citizen this crate queued at the city
    ///   and the original does not. **Gone on item 385**: the head at 9182
    ///   is the original's Mercenaries once the met bit is set, so the
    ///   Citizen is never bought and **the window is silent**, 1,320
    ///   frames with no queue row at all.
    #[test]
    fn run97_s_build_queues_are_the_original_s() {
        const FIRST: i64 = 8_030;
        const LAST: i64 = 9_349;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut blocks = 0usize;
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut first: std::collections::BTreeMap<(i64, i64), i64> = Default::default();
        for f in 0..=LAST {
            built.tick();
            if f < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f + 1)
                .map(|(_, b)| b)
            else {
                continue;
            };
            blocks += 1;
            let (_, builds, _) = crate::gamelog::records(block, false);
            let alive = built.sim.buildings.iter().filter(|x| x.alive).count();
            assert_eq!(
                alive,
                builds.len(),
                "run97 block {} has {} buildings and this crate {alive}",
                f + 1,
                builds.len()
            );
            for b in &builds {
                let ours = built
                    .sim
                    .buildings
                    .iter()
                    .find(|x| x.alive && i64::from(x.owner) == b.who && i64::from(x.index) == b.o)
                    .unwrap_or_else(|| panic!("run97 block {} has no {}/{}", f + 1, b.who, b.o));
                // `queued` is how many slots are live; the rest of the
                // array holds whatever it was last left with.
                let live = usize::try_from(b.queued.unwrap_or(0)).unwrap_or(0);
                let theirs: Vec<(i64, i64)> = b
                    .queue
                    .iter()
                    .take(live)
                    .map(|q| (q.ty, q.job_counter))
                    .collect();
                let mine: Vec<(i64, i64)> = ours
                    .queue
                    .items
                    .iter()
                    .map(|i| {
                        (
                            i.tech.map_or_else(
                                || i64::from(built.sim.unit_types[i.ty].type_index),
                                |x| i64::from(loaded.type_index(x)),
                            ),
                            i64::from(i.job_counter),
                        )
                    })
                    .collect();
                compared += 1;
                if mine != theirs {
                    first.entry((b.who, b.o)).or_insert(f);
                    if wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {f}: {}/{} ours {mine:?} theirs {theirs:?}",
                            b.who, b.o
                        ));
                    }
                }
                if let (Some(jc), Some(ct)) = (b.job_counter, b.constr_time) {
                    assert_eq!(
                        (jc, ct),
                        (i64::from(ours.job_counter), i64::from(ours.constr_time)),
                        "frame {f}: {}/{}'s construction clock is not run97's",
                        b.who,
                        b.o
                    );
                }
            }
        }
        eprintln!(
            "run97 queues: {blocks} blocks, {compared} building-frames, {} queue(s) wrong on \
             {} building(s) — {first:?}",
            wrong.len(),
            first.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            blocks >= 1_300 && compared >= 36_000,
            "run97's own blocks are missing — the wrong file: {blocks} blocks, {compared} \
             building-frames"
        );
        // **Nothing at all, over the whole window** — item 385. The last
        // row, `1/2007` from 9182, was the Citizen this crate bought when
        // its make list put a Scholar at the head where the original puts
        // Mercenaries; with the met bit the head is the original's, the
        // purchase does not happen, and this window is silent. A building
        // parting anywhere in [8030, 9349] is now a divergence this test
        // was written to catch.
        assert_eq!(
            first.keys().copied().collect::<Vec<_>>(),
            Vec::new(),
            "run97's build queues part from the original's: {first:?}"
        );
        // **The batch itself, in the queue it filled.** 8985's
        // University is the frame two items were spent on: item 358's
        // `do_sell` paid for the second scholar, and item 362's `num`
        // bought the third in the same `make_this`. The row is gone from
        // `wrong` entirely now, so the assertion is the whole window's
        // silence on `1/2020` — made to fail by putting `num` back to
        // one, which restores `8985: 1/2020 ours [(52, 400), (52, 0)]
        // theirs [(52, 400), (52, 0), (52, 0)]` as the first row.
        assert!(
            !wrong.iter().any(|w| w.contains("1/2020")),
            "the AI's University parts from the original's somewhere in the \
             window — the scholar batch is not the original's: {:?}",
            wrong.first()
        );
        // And the whole window's silence, which is the same statement as
        // the empty map above and is kept because it prints. Made to fail
        // on purpose: putting the `human` skip back into `Sim::has_met`
        // restores `frame 9182: 1/2007 ours [(50, 100)] theirs []` as the
        // first row.
        assert!(
            wrong.is_empty(),
            "run97's window is not silent on the build queues: {:?}",
            wrong.first()
        );
    }

    /// **run97's order lists, every unit, every frame of the window** —
    /// Great Lakes `[8030, 9349]`, item 368.
    ///
    /// The capture writes a `STACK<TYPE>` per unit with every live order
    /// and its own fields — a `GATHERORDER`'s tile, phase and countdown,
    /// a `MOVEORDER`'s goal, a group order's leader and slot — and
    /// **36,483 of them sit in this file uncompared**. run97 had its
    /// clocks read (item 346), its positions and leading goal (item 350)
    /// and its build queues (item 358); the order stacks that name *what
    /// each citizen is working on* had never been looked at on this map.
    ///
    /// That is the record this item wanted. 9182's residue is the
    /// leader's own ledger — three `use_market` draws against one — and
    /// the ledger is a function of the stockpile, which is a function of
    /// which good each gatherer is carrying. `LEADERS=1` does not print
    /// the stockpile (item 369 is the capture that would), but the
    /// gatherers are printed in full, and a citizen working a different
    /// camp on either side is the one cause of a stockpile gap that a
    /// capture on this disk can still refute.
    #[test]
    fn run97_s_window_orders_are_the_original_s() {
        const FIRST: i64 = 8_030;
        /// **run97's last complete block**, and the bound this loop takes
        /// instead of the word.
        ///
        /// The file's final `BEGIN FRAME 9361` is **truncated mid-record**
        /// — the click-free lane gives up by stopping the process, so the
        /// last block is whatever had been written — and every unit in it
        /// has a short order stack. While the word was 9182 that block was
        /// above it and never read; item 385 moved the word to 9415 and the
        /// truncation arrived as 47 `Length` rows on 39 units, including
        /// player 0's, which reads exactly like a simulation that has come
        /// apart. It is not. The sibling test above
        /// (`run97_s_build_queues_are_the_original_s`) has carried this
        /// same bound since it was written; this one took the word and did
        /// not.
        const LAST: i64 = 9_349;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut blocks = 0usize;
        let mut compared = 0usize;
        let mut gather_rows = 0usize;
        let mut gather_compared = 0usize;
        let mut rows: Vec<OrderDivergence> = Vec::new();
        for f in 0..LONG_WORD_GREAT_LAKES.min(LAST + 1) {
            built.tick();
            if f < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f + 1)
                .map(|(_, b)| b)
            else {
                continue;
            };
            blocks += 1;
            let (units, _, _) = crate::gamelog::records(block, false);
            for u in &units {
                if !(0..8).contains(&u.who) {
                    continue;
                }
                let Some(at) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == u.who && i64::from(x.index) == u.o
                }) else {
                    continue;
                };
                let link = crate::diff::setup::UnitLink {
                    who: u.who,
                    o: u.o,
                    unit: at,
                    kind: None,
                };
                compared += 1;
                gather_compared += built.sim.units[at]
                    .orders
                    .iter()
                    .filter(|o| matches!(o.body, sim::orders::Body::Gather(_)))
                    .count();
                for d in compare_orders(&built, &link, u, f) {
                    if matches!(d.what, crate::diff::order::OrderMismatch::Gather { .. }) {
                        gather_rows += 1;
                    }
                    rows.push(d);
                }
            }
        }
        // **The `id` stand-in is not a divergence** — `GroupData +0x4` is
        // a counter this crate stands in for with the army group's slot
        // (`OrderMismatch::scores`), and it puts one row on every grouped
        // unit-frame by construction. Everything else counts.
        let scoring: Vec<&OrderDivergence> = rows.iter().filter(|d| d.what.scores()).collect();
        eprintln!(
            "run97 orders: {blocks} blocks, {compared} unit-frames below the word, \
             {} rows ({} scoring, {gather_rows} of them gather fields over \
             {gather_compared} gather orders)",
            rows.len(),
            scoring.len(),
        );
        let mut by_kind: std::collections::BTreeMap<String, (usize, i64, i64, i64)> =
            Default::default();
        for d in &scoring {
            let key = match &d.what {
                crate::diff::order::OrderMismatch::Move { field, .. } => format!("move/{field}"),
                crate::diff::order::OrderMismatch::Gather { field, .. } => {
                    format!("gather/{field}")
                }
                crate::diff::order::OrderMismatch::Group { field, .. } => format!("group/{field}"),
                w => format!("{w:?}")
                    .split_whitespace()
                    .next()
                    .unwrap_or("?")
                    .to_string(),
            };
            let e = by_kind.entry(key).or_insert((0, d.frame, d.who, d.o));
            e.0 += 1;
        }
        for (k, (n, f, who, o)) in &by_kind {
            eprintln!("  {k}: {n} rows, first f{f} {who}/{o}");
        }
        let mut by_unit: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        for d in &scoring {
            *by_unit.entry((d.who, d.o)).or_default() += 1;
        }
        let mut units: Vec<_> = by_unit.into_iter().collect();
        units.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        eprintln!("  units: {:?}", units.iter().take(20).collect::<Vec<_>>());
        assert!(
            blocks >= 1_100 && compared >= 30_000 && gather_compared >= 30_000,
            "run97's own blocks are missing — the wrong file: {blocks} blocks, \
             {compared} unit-frames, {gather_compared} gather orders"
        );
        // **The product: not one gatherer disagrees.** 36,458 gather
        // orders over the capture's own window — 31,448 of them when the
        // word was 9182 and item 368 wrote this — and every `tx`, `ty`,
        // `wait`, `goto_build`, `been_there` and `dist_mod` of every one
        // of them is the original's. Made to fail on purpose — `+ 1` on
        // the comparison's own `tx` puts a row on all of them, one
        // apiece — so the nought is a measurement and not a branch that
        // never ran. What it closes is a whole family of explanation for the
        // leader's stockpile: no citizen on this map is working a camp
        // the original does not, on any frame below the word.
        assert_eq!(
            gather_rows,
            0,
            "a gatherer's order parts from run97's: {:?}",
            scoring
                .iter()
                .find(|d| matches!(d.what, crate::diff::order::OrderMismatch::Gather { .. }))
        );
        // **And the residue is confined to units already named.** The
        // window opens 412 frames before the word and this crate's army
        // is off its position from 8442 (`docs/ARMY.md` §3.4's
        // successor), which drags the orders those six hold with it;
        // `1/23` is a standing unit carrying an `Action` order the dump
        // does not and a `Move` angle to match, older than this window.
        // The **set** is the assertion, because a row count cannot say
        // whether a ninth unit joined them — and item 385 is why that
        // matters: the truncated final block briefly put 39 of them in
        // here, and only the set said so.
        let mut units: Vec<(i64, i64)> = scoring.iter().map(|d| (d.who, d.o)).collect();
        units.sort_unstable();
        units.dedup();
        assert_eq!(
            units,
            vec![
                (1, 23),
                (1, 27),
                (1, 28),
                (1, 29),
                (1, 33),
                (1, 40),
                (1, 41),
                (1, 42)
            ],
            "run97's order residue reached a unit item 368 did not leave it on"
        );
        // The count is floored beneath the set, so a known unit growing
        // a new kind of row is caught too. It may only come down.
        assert!(
            scoring.len() <= ORDER_RESIDUE_RUN97,
            "run97's order residue grew past {ORDER_RESIDUE_RUN97}: {} rows, first {:?}",
            scoring.len(),
            scoring.first()
        );
    }

    /// The frame Great Lakes' AI sends [`PROBE_SENT`]'s six from its base
    /// to the far south-west, and the one world path that carries all six.
    const PROBE_PLAN_FRAME: i64 = 8186;

    /// **Great Lakes 8186 plans the probe's route the original's way** —
    /// item 354, `docs/PATHFINDER.md` §22.
    ///
    /// `Group::action_move_near` plans **one** `find_wpath` on the group's
    /// leader and gives every member the same chain offset by its
    /// formation slot (`docs/GROUPS.md` §6.7), so one wrong mode costs six
    /// units their whole route — and does it silently, because a wrong
    /// waypoint four hundred frames ahead spends no draw until the unit
    /// reaches it. This crate's chain agreed with run97 block 8187's for
    /// thirteen waypoints and then took a five-step detour that rejoined
    /// it, and the first position to part was 393 frames later.
    ///
    /// What was wrong is `find_wpath`'s `army` mode: its `is_attacking`
    /// clause calls the current order's `+0x18` virtual, and that slot is
    /// `0041bff0` — a bare `return 0` — in every one of the seventeen
    /// order vtables the executable ships (§22.1, read out of the PE). The
    /// clause cannot fire, and reading it as "the unit has a combat
    /// target" turned the mode off for precisely the unit it exists for:
    /// an AI army walking to an `ATTACK_TO`. With the mode on, a cell
    /// flagged `NEARBLOCK` costs an army 1024 against 32 and the two
    /// equal-geometry routes stop tying.
    ///
    /// The whole stack is the assertion, every entry, because "the sixth
    /// waypoint is right" is exactly the claim a route cannot make. The
    /// **top** entry's `tolerance`/`flags` are compared apart: the dump
    /// prints the current waypoint as `t0 f1` on every frame of the march
    /// while this crate keeps the planner's `t384 f0` on it, which is a
    /// residue of its own and not this item's (it is the same on every
    /// block in the window, before the fix and after).
    #[test]
    fn great_lakes_8186_plans_the_probe_s_route_the_original_s_way() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // One block of a 616 MB capture, the way run93's world diff reads
        // its own — `IndexedCapture` is what keeps this inside the gate's
        // memory ceiling.
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r97).unwrap();
        let at = ix
            .frames()
            .iter()
            .position(|f| f.number == PROBE_PLAN_FRAME + 1)
            .expect("run97 has no block 8187 — the wrong file");
        let body = ix.read_frame(at).unwrap();
        let parsed = Log::parse(&body);
        let block = parsed
            .frames()
            .into_iter()
            .find(|(n, _)| *n == PROBE_PLAN_FRAME + 1)
            .map(|(_, b)| b)
            .expect("run97's block 8187 did not re-parse");
        let (theirs, _, _) = crate::gamelog::records(block, false);

        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame <= PROBE_PLAN_FRAME {
            built.tick();
        }
        let mut checked = 0usize;
        for &(who, o, _, _) in PROBE_SENT {
            let them = theirs
                .iter()
                .find(|u| u.who == who && u.o == o)
                .unwrap_or_else(|| panic!("run97 block 8187 has no {who}/{o}"));
            let u = built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
                .unwrap_or_else(|| panic!("no live {who}/{o} at {PROBE_PLAN_FRAME}"));
            let ours = &built.sim.units[u];
            assert_eq!(
                (i64::from(ours.pos.x), i64::from(ours.pos.y)),
                (them.pos.x, them.pos.y),
                "{who}/{o} is not standing where run97 block 8187 puts it"
            );
            // **The leg this frame planned, top down.** Two things below
            // it are residues of their own, printed rather than asserted
            // because neither belongs to the route:
            //
            // - **entry 0**, the raw formation slot
            //   `Group::action_move_near` pushes under the destination
            //   (`docs/PATHFINDER.md` §12's second item). This crate puts
            //   `1/27`, `1/28` and `1/29` on one `y` and the original
            //   spreads them, which is `Group::update_positions`'
            //   rotation by the unit's angle (`docs/GROUPS.md` §6.6);
            // - **a whole second leg** under `1/40`'s: run97 block 8187
            //   gives it 92 entries where this crate gives 47, and the
            //   46 extra sit *beneath* — a queued move this crate does
            //   not hold. Its own 46 are the current leg and they agree
            //   entry for entry, which is what the comparison below says.
            //
            // §22.4 names both as successors.
            let k = ours.path.len().min(them.path.len()) - 1;
            let ours_route: Vec<(i64, i64)> = ours.path[ours.path.len() - k..]
                .iter()
                .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
                .collect();
            let their_route: Vec<(i64, i64)> = them.path[them.path.len() - k..]
                .iter()
                .map(|p| p.to)
                .collect();
            eprintln!(
                "  8187 {who}/{o}: {} entries ours, {} theirs; slot ours {:?} theirs {:?}",
                ours.path.len(),
                them.path.len(),
                ours.path.first().map(|p| (p.to.x, p.to.y)),
                them.path.first().map(|p| p.to)
            );
            assert_eq!(
                ours_route, their_route,
                "{who}/{o}'s world plan is not run97 block 8187's, waypoint for \
                 waypoint"
            );
            // The same span's tolerances and flags, whole — except the
            // **top** entry, which the dump prints as `t0 f1` on every
            // frame of the march while this crate keeps the planner's
            // `t384 f0`. That is the third residue and it is the same
            // before this item and after.
            let ours_rest: Vec<(i64, i64)> = ours.path[ours.path.len() - k..ours.path.len() - 1]
                .iter()
                .map(|p| (i64::from(p.tolerance), i64::from(p.flags)))
                .collect();
            let their_rest: Vec<(i64, i64)> = them.path[them.path.len() - k..them.path.len() - 1]
                .iter()
                .map(|p| (p.tolerance, p.flags))
                .collect();
            assert_eq!(
                ours_rest, their_rest,
                "{who}/{o}'s waypoint tolerances and flags are not run97 block \
                 8187's"
            );
            checked += 1;
        }
        assert_eq!(checked, 6, "the probe's six");
    }

    /// **run54 — East Indies at thirteen times the scored length, read at
    /// last** (2026-09-01).
    ///
    /// run54 was taken with run53 and sat unread for a day: while run39's
    /// own word parted at 1647 there was no question a longer capture could
    /// answer. Item 125 took that word to the end of run39, so the scored
    /// capture stopped being the boundary and this one became the only
    /// thing that says where the boundary is.
    ///
    /// It is the same game as run38/run39 — `tools/gamelog/rngcmp.py` says
    /// its `game_random` word is run39's on all 1,851 overlapping frames,
    /// zero differing — traced whole over **24,000**, and its `[End Frame]`
    /// is `MISC` alone, so it carries the stream and no per-frame record.
    /// That is what it is for: the word is scoreable from the trace, and
    /// ticks and orders are not scoreable here at all.
    ///
    /// **The pasture is a source here too** — run54's own trace reached the
    /// setup, so the five owner-9 animals come from it rather than from
    /// run39's (`docs/SYNC.md` §3.11). Without them this is a different
    /// game within a few hundred frames and every number below is nobody's.
    ///
    /// Only the two frame numbers are pinned, for run53's reason: past the
    /// parting both sides are on streams that are nobody's, and the totals
    /// there are coincidence that moves with every unrelated change.
    #[test]
    fn run54_s_24000_frames_are_where_the_second_map_s_word_now_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(trace)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            eprintln!("skipping: no run54 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &trace);
        assert_eq!(
            init.pasture.len(),
            1,
            "run54's trace reached the setup and East Indies' AI has one pasture"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 23_000,
            "run54's traced length is {last}, wanted 23,000+ — this is the \
             long capture, and a short file here is a wrong file"
        );
        // `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>` prints one unit's position,
        // order and figure clocks over a window of frames. The word says
        // *that* a frame parted; a cadence — a crew figure wrapping every
        // second frame rather than every third — needs the clock itself,
        // and past the last `DUMP_ALL` window there is nothing else to
        // read it from.
        let watch: Option<(i64, i64, i64, i64)> =
            std::env::var("RON_DEBUG_UNIT").ok().and_then(|v| {
                let (u, w) = v.split_once('@')?;
                let (who, o) = u.split_once('/')?;
                let (lo, hi) = w.split_once('-')?;
                Some((
                    who.trim().parse().ok()?,
                    o.trim().parse().ok()?,
                    lo.trim().parse().ok()?,
                    hi.trim().parse().ok()?,
                ))
            });
        for f in 0..last {
            built.tick();
            // `RON_DEBUG_FOLD=<lo>-<hi>` prints [`Built::phase_fold`] over
            // a window: the frame's draws attributed to the mark that was
            // standing when each was spent. It answers a question the site
            // list cannot — whether two draws at one site are two units or
            // **one unit's two figures**, which is what East Indies 6571
            // turned on. It folds by *site*, so the unit mark is lost
            // wherever a site mark stands inside it (`docs/QUEUE.md` 187).
            if let Ok(w) = std::env::var("RON_DEBUG_FOLD")
                && let Some((lo, hi)) = w.split_once('-')
                && let (Ok(lo), Ok(hi)) = (lo.trim().parse::<i64>(), hi.trim().parse::<i64>())
                && (lo..=hi).contains(&f)
            {
                eprintln!("  fold {f}: {:?}", built.phase_fold());
            }
            let Some((who, o, lo, hi)) = watch else {
                continue;
            };
            if !(lo..=hi).contains(&f) {
                continue;
            }
            let Some(u) = (0..built.sim.units.len()).find(|&i| {
                let x = &built.sim.units[i];
                x.alive() && i64::from(x.owner) == who && i64::from(x.index) == o
            }) else {
                continue;
            };
            let x = &built.sim.units[u];
            let clocks: Vec<String> = x
                .guys
                .iter()
                .map(|g| {
                    // The **body** and its destination beside the clock:
                    // `Guy::set_anim`'s idle request returns without a
                    // roll while a walking guy's body has not arrived
                    // (`docs/ANIM.md` §4 step 1), so a draw this crate
                    // spends and the original does not is read here or
                    // nowhere.
                    let (bx, by, dx, dy) = g.follow.map_or_else(
                        || {
                            (
                                x.movement.body.pos.x,
                                x.movement.body.pos.y,
                                x.pos.x,
                                x.pos.y,
                            )
                        },
                        |f| (f.body.pos.x, f.body.pos.y, f.des.x, f.des.y),
                    );
                    format!(
                        "[a{} t{}/{} l{} b({bx},{by})->({dx},{dy})]",
                        g.anim, g.cur_time, g.end_time, g.last_time
                    )
                })
                .collect();
            // The **collision** half of the line, added for item 276:
            // East Indies' word at 7806 is a stop, and every frame of the
            // shuffle behind it is a `collide` / wait-flag / `pause`
            // question (`docs/COLLISION.md` §6). All five are dumped
            // fields, so a line that prints them is a row a capture over
            // the gap can refuse.
            let pause = match x.orders.front().map(|ord| &ord.body) {
                Some(sim::orders::Body::Move(m)) => m.pause,
                _ => -1,
            };
            eprintln!(
                "  f{f} {who}/{o} at ({}, {}) ang {} hdg {} path {} top {:?} order {:?} \
                 coll {}/{}/{} wait {} pause {pause} {}",
                x.pos.x,
                x.pos.y,
                x.movement.facing.0,
                x.movement.heading.0,
                x.path.len(),
                x.path.last().map(|p| (p.to.x, p.to.y, p.flags)),
                x.orders.front().map(|ord| ord.index()),
                x.collide,
                x.collide_who,
                x.collide_o,
                x.waiting_on,
                clocks.join(" ")
            );
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map_or(last, |(f, _)| *f);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map_or(last, |(f, _)| *f);
        eprintln!("run54: word parts at {first_count}, sequence at {first_part} of {last}");
        // `RON_DEBUG_SITES=<lo>-<hi>` widens the two printed frames to a
        // window. A residue in a *cadence* — a figure's clock wrapping every
        // second frame rather than every third — is invisible at the frame
        // it finally parts on and obvious over a dozen either side.
        let window = site_window();
        for (f, ours) in built.frame_sites.iter() {
            let named = *f == first_count || *f == first_part;
            if !named && !window.is_some_and(|(lo, hi)| (lo..=hi).contains(f)) {
                continue;
            }
            let theirs = trace.labels(*f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            eprintln!(
                "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at)
            );
            // `RON_DEBUG_SITES=1` prints both sequences whole. The line
            // above names the first *index* that parts, which is enough
            // when the two sides are the same length and useless when
            // they are not: the four draws East Indies' word is missing
            // are at the **head** of the original's frame, so every later
            // entry reads as a mismatch and the first one names a bird.
            if std::env::var("RON_DEBUG_SITES").is_ok() {
                for (i, l) in ours.iter().enumerate() {
                    eprintln!("    ours  {i}: {l}");
                }
                for (i, l) in theirs.iter().enumerate() {
                    eprintln!("    thrs  {i}: {l}");
                }
            }
        }
        // **`gull_o` is the one field of the `DOCK` record nothing has ever
        // compared**, and this capture is where it can be. run22 is this
        // same game with a `DUMP_ALL` window on `[3579, 3582)`
        // (`docs/ORACLE.md`), and its block 3580 reads `DOCK dock 0, o
        // 2010, reg 65, gull_o 15, who 1, dock_flags 1`. Owner 9 is absent
        // from a dump, so the gull's own record is not there to compare —
        // its **object number** is, and it is what says this crate hands
        // gaia its slots where the original does.
        let gull = built.sim.docks[1].slots[0]
            .gull
            .expect("the AI's dock spawned its gull");
        assert_eq!(built.sim.units[gull].owner, 9, "the gull is gaia's");
        assert_eq!(
            built.sim.units[gull].index, 15,
            "run22's block 3580: `DOCK … gull_o 15`"
        );
        assert!(
            first_count >= LONG_WORD_EAST_INDIES && first_part >= LONG_WORD_EAST_INDIES,
            "run54's ceiling fell: word {first_count}, sequence {first_part}; \
             the floor is {LONG_WORD_EAST_INDIES} on both"
        );
    }

    /// **run64's animation clocks, frame for frame** — every `GUY`
    /// block's `cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`
    /// and `stopped`, over the eight frames of the `DUMP_ALL` window.
    ///
    /// Item 87's ledger, one more row of it paid: `Initial::frame_guys`
    /// has been parsed since the animation clock was first read and
    /// nothing ever *compared* it — [`Built::tick`] **installs** it, which
    /// is the opposite of a check. Every capture that carries the field
    /// therefore said nothing about it, and the caravan's two crew
    /// figures walked on the citizen's art for as long as the window has
    /// existed.
    ///
    /// Nothing here is installed: run64's own `frame_guys` never reach the
    /// simulation, which is built from run54's start and driven forward
    /// 6,163 frames. Every row is a prediction the dump can refuse.
    ///
    /// The window holds the caravan, and the caravan is why this exists.
    /// `CARAVAN-DEFAULT-AGE0-CREW1` and `-CREW2` are two of the sixty
    /// shipped `<UNIT>` entries with **no `<ANIM>` child at all**: the
    /// piece loads, its packet is empty, so every slot is
    /// `AnimationPacket::get_game_frames`' three frames and none of them
    /// loops. Its crew walks three frames, falls to the idle, rolls, and
    /// is put back on the walk by `Guy::move` the next frame — two draws
    /// every third frame for the rest of the game (`docs/ANIM.md` §3.6).
    #[test]
    fn run64_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run64-islands-caravanroad.txt"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let text64 = crate::capture::read(&r64);
        let theirs = Log::parse(&text64)
            .initial()
            .expect("run64 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            window,
            vec![6163, 6164, 6165, 6166, 6167, 6168, 6169, 6170],
            "run64's `DUMP_ALL` window, as sim-frames"
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The window's own clocks are never installed: this is the check.
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut crew_rows = 0usize;
        let mut unmatched = 0usize;
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so their clocks are not a
                // prediction of this crate's; the players' are.
                if !(0..8).contains(&state.who) {
                    continue;
                }
                // Matched on the simulation's own `(owner, o)` rather than
                // through [`UnitLink`], which only knows the start dump's
                // units: by 6,163 frames most of the AI's economy was
                // trained mid-game and a link table would compare a
                // fourteenth of the window.
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    if n >= sim::anim::SQUAD_SIZE {
                        crew_rows += 1;
                    }
                    // **The whole of what this crate models of the
                    // record**, not the clock alone: `des`, `des_angle`
                    // and the speed pair were parsed and compared
                    // nowhere until item 210, and they are the arrival
                    // draw's own inputs — `Guy::move` draws on `cur_anim
                    // == 8 && stopped` once `angle` reaches `des_angle`,
                    // and whether the slot lands on 8 rather than 9 is
                    // `avg_speed`'s doing (`docs/ANIM.md` §4.3).
                    let (body, facing, des, des_angle) = match ours.follow {
                        Some(b) => (b.body, b.facing, b.des, b.des_angle),
                        None => (
                            built.sim.units[u].movement.body,
                            built.sim.units[u].movement.facing,
                            built.sim.units[u].pos,
                            // **A trackless crew guy's `des_angle` is guy
                            // 0's `angle`, not its heading** — the crew
                            // loop of `Guy::set_angle` hands it the angle
                            // just turned to, so it is settled on every
                            // frame and never owed a turn of its own. The
                            // dump says it outright: run64's `1/18` guy 0
                            // reads `angle 1145324629, des_angle
                            // 560594944` on the frame it starts turning,
                            // and its two crew figures read `1145324629`
                            // for both. That is what
                            // [`sim::Sim::guys_follow`]'s `settled ||
                            // g >= SQUAD_SIZE` already models.
                            if n >= sim::anim::SQUAD_SIZE {
                                built.sim.units[u].movement.facing
                            } else {
                                built.sim.units[u].movement.heading
                            },
                        ),
                    };
                    let rows: [(&str, i64, Option<i64>); 15] = [
                        ("x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                        ("y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                        ("angle", i64::from(facing.0), g.angle),
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                        ("gpiece", i64::from(ours.gpiece), g.gpiece),
                        ("stopped", i64::from(ours.stopped), g.stopped),
                        ("des_x", i64::from(des.x), g.des.map(|p| p.x)),
                        ("des_y", i64::from(des.y), g.des.map(|p| p.y)),
                        ("des_angle", i64::from(des_angle.0), g.des_angle),
                        ("last_speed", i64::from(body.last_speed), g.last_speed),
                        ("avg_speed", i64::from(body.avg_speed), g.avg_speed),
                        ("guy_num", i64::try_from(n).unwrap(), g.guy_num),
                    ];
                    for (name, ours, theirs) in rows {
                        let Some(theirs) = theirs else { continue };
                        compared += 1;
                        if ours == theirs {
                            continue;
                        }
                        let row = format!(
                            "frame {f}: {}/{} guy {n} {name} ours {ours} theirs {theirs}",
                            state.who, state.o
                        );
                        if wrong.len() < 12 {
                            wrong.push(row);
                        }
                    }
                }
            }
        }
        eprintln!(
            "run64 clocks: {compared} fields compared over {crew_rows} crew rows, \
             {unmatched} of the dump's units this crate has no unit for"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 2_000 && crew_rows >= 24,
            "the window's own rows: {compared} fields, {crew_rows} of them a \
             crew figure's — a capture with neither is the wrong file"
        );
        // **The caravan's own rows used to be thirty-four of these**, and
        // they were the walk rather than the clock: `do_trade`'s move to
        // the near city went in with a waypoint at `(37752, 41592)`, so
        // `1/18` set off south-west where the original heads straight at
        // `(39288, 40056)`. The waypoint was never the bug — the danger
        // map was. Without it `calc_cost` priced eight of the caravan's
        // own world-grid steps 8 to 16 too dear, the search took a seventh
        // expansion the original does not, and a route came back where the
        // original's search stops at the goal's own neighbour and pushes
        // nothing (`crates/sim/src/danger.rs`, `docs/DANGER.md` §6). All
        // 2,061 fields are the original's since 2026-09-02.
        assert!(wrong.is_empty(), "run64's clocks parted: {wrong:?}");
    }

    /// **run73 — Great Lakes' own window, and the second `set_anim` a
    /// moving frame makes** (2026-09-03, item 205).
    ///
    /// The first `DUMP_ALL` window this map has ever had. Great Lakes'
    /// word parted at **5571** on two `Guy::set_anim+0x97a <
    /// Guy::inc_time+0x271` draws, and the site fold named them: the
    /// caravan `1/23`, trained on 5564, and its **two crew figures** —
    /// `docs/ANIM.md` §3.6's three-frame metronome. The cadence was the
    /// whole clue. A standing caravan's crew mirrors guy 0 and wraps on
    /// alternate frames; a walking one wraps every third. The original's
    /// wraps run 5569, 5572, 5575 and this crate's ran 5569, **5571**,
    /// 5574 — the standing cadence one frame too long.
    ///
    /// `MAP_STYLE 14`, seed 12345, run10's lobby, no input, to 5,590
    /// frames with the window on `[5564, 5580)` and the road proxies over
    /// `[5563, 5581]`. `rngcmp.py` against run53: **5,591 frames, zero
    /// differing**, so it is run53's game and the sixth capture in a row
    /// for which a window costs the stream nothing.
    ///
    /// **What it settled, in one field.** Guy 0's clock is this crate's on
    /// every frame of the window — the driver never differed. The crew's
    /// parts once: on 5571, the frame the caravan first walks, the
    /// original's figure reads `cur_anim 8, cur_time 2, last_time 1` where
    /// this crate read `cur_anim 0, cur_time 0, last_time −1`. A
    /// `last_time` of 1 says the clock stood at **1** before the step, and
    /// the figure came off the mirror at **4** — the length, 3, taken off.
    ///
    /// `Guy::set_anim`'s walk arm only subtracts for the slot **already
    /// playing** (`005db4d1`: a slot *change* rescales, and the rescale
    /// passes the old slot to both `get_anim_time` calls, so it is
    /// `cur_time · t / t`). One call could therefore never produce a 1.
    /// There are two: `Unit::move_step` calls `Unit::set_anim(CHAR_WALK,
    /// 0, 1)` on every guy immediately before `set_new_location`
    /// (`move_step:304` on the partial step, `:355` on the snap), and
    /// `Guy::move` calls it again in the body follow. The first is the
    /// change — `CHAR_SLOG → CHAR_WALK`, clock kept at 4 — and the second
    /// is then the same slot and subtracts. This crate made only the
    /// second (`docs/ANIM.md` §4.9).
    ///
    /// Nothing here is installed: run73's own `frame_guys` never reach the
    /// simulation, which is built from run53's start and driven forward
    /// 5,579 frames. Every row is a prediction the dump can refuse.
    /// **run94's animation clocks, frame for frame** — every `GUY`
    /// block's `cur_anim`, `cur_time`, `end_time` and `last_time` over
    /// the 291 frames of Great Lakes' `[7753, 8043]` window (item 340).
    ///
    /// The clock is art the simulation reads rather than computes, and
    /// until this existed **no Great Lakes capture had ever compared
    /// one**: run73's window is `[5564, 5578]`, fourteen frames three
    /// thousand before the map's word, and the two Islands windows
    /// (run64, run67) are another game. So a length this crate had wrong
    /// for a piece run73's window does not hold could survive every
    /// other test in the suite and show up only as a missing wrap, which
    /// is exactly the shape Great Lakes 8374 turned out to be.
    ///
    /// **What it said is a negative, and the negative is the point.**
    /// 66,300 fields, zero differing — so the lengths, the variants and
    /// the wrap cadence of every one of the players' guys are the
    /// original's over 291 consecutive frames three hundred short of the
    /// word. That is what turned item 340 from "some clock is wrong"
    /// into "the wrong clock belongs to a `(piece, slot)` pair this
    /// window never saw", and at 8374 there were exactly four of those:
    /// the scholar's was one (`docs/ANIM.md` §4.11).
    ///
    /// Nothing here is installed: run94's own `frame_guys` never reach
    /// the simulation, which is built from run53's start and driven
    /// forward 8,043 frames. Every row is a prediction the dump can
    /// refuse.
    #[test]
    fn run94_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r94)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run94-greatlakes-scoutrepath.txt"),
        ) else {
            eprintln!("skipping: no run53/run94 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &trace);

        let text94 = crate::capture::read(&r94);
        let theirs = Log::parse(&text94)
            .initial()
            .expect("run94 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first() == Some(&7_753) && window.last() == Some(&8_043) && window.len() >= 291,
            "run94's clock window is not [7753, 8043]: {:?}..{:?} ({})",
            window.first(),
            window.last(),
            window.len()
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The window's own clocks are never installed: this is the check.
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so their clocks are not
                // this crate's prediction; the players' are.
                if !(0..8).contains(&state.who) {
                    continue;
                }
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    let rows: [(&str, i64, Option<i64>); 4] = [
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                    ];
                    for (name, o, t) in rows {
                        let Some(t) = t else { continue };
                        compared += 1;
                        if o == t {
                            continue;
                        }
                        if wrong.len() < 20 {
                            wrong.push(format!(
                                "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                state.who, state.o
                            ));
                        }
                    }
                }
            }
        }
        eprintln!(
            "run94 clocks: {compared} fields over {} frames, {unmatched} of the \
             dump's units this crate has no unit for",
            window.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 60_000,
            "run94's own clock rows are missing — a capture without \
             `GUYS` detail is the wrong file: {compared} fields"
        );
        assert!(wrong.is_empty(), "run94's clocks parted: {wrong:?}");
    }

    /// **run97's animation clocks, frame for frame** — Great Lakes'
    /// `[8030, 9349]`, 1,320 consecutive frames of `GUYS` detail across
    /// the map's word (item 346).
    ///
    /// The first test to read run97 at all. run94 stops at 8043 and the
    /// word was 8404, so between them lay 361 frames in which no Great
    /// Lakes clock had ever been compared — and 8404 was a wrap.
    #[test]
    fn run97_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r97)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run97-greatlakes-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run53/run97 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &trace);

        let theirs = {
            let text97 = crate::capture::read(&r97);
            Log::parse(&text97)
                .initial()
                .expect("run97 carries a start block")
                .frame_guys
        };
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first() == Some(&8_029)
                && window.last() == Some(&9_348)
                && window.len() >= 1_320,
            "run97's clock window is not [8029, 9348] in sim-frames: {:?}..{:?} ({})",
            window.first(),
            window.last(),
            window.len()
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut above = 0usize;
        let mut unmatched = 0usize;
        let mut walk = 0usize;
        // The walk-slot rows themselves, not only their count — item 352
        // closed the band at zero, so anything here is a handful and worth
        // naming rather than summing.
        let mut walkw: Vec<String> = Vec::new();
        let mut wrong: Vec<String> = Vec::new();
        let mut posw: Vec<String> = Vec::new();
        let mut body_bad = 0usize;
        let mut body_all = 0usize;
        let mut body_trio = 0usize;
        let mut frames_below = 0usize;
        let mut posfirst: std::collections::BTreeMap<(i64, i64), i64> = Default::default();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            if f < LONG_WORD_GREAT_LAKES {
                frames_below += 1;
            }
            for state in units {
                if !(0..8).contains(&state.who) {
                    continue;
                }
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                // **The rest of the record** (item 350). The clocks
                // below are four fields of a `GUY` block; the `UNITDATA`
                // that carries them prints the unit's own point and the
                // goal of its leading move order beside them, and for a
                // month neither was compared on this capture. The first
                // run of this widening found sixteen units off position
                // from 8442 and named the cause in twenty minutes
                // (`docs/ARMY.md` §3.4).
                {
                    let un = &built.sim.units[u];
                    let ourgoal = un.orders.front().and_then(|o| match &o.body {
                        sim::orders::Body::Move(m) => Some((m.dest.x, m.dest.y)),
                        _ => None,
                    });
                    // `-1` for "no leading move order" on both sides:
                    // whether a unit *has* a goal is half the comparison,
                    // and a `None`/`Some` mismatch is a divergence.
                    let rows: [(&str, i64, i64); 4] = [
                        ("x", i64::from(un.pos.x), state.pos.x),
                        ("y", i64::from(un.pos.y), state.pos.y),
                        (
                            "goal_x",
                            ourgoal.map_or(-1, |(x, _)| i64::from(x)),
                            state.goal.map_or(-1, |g| g.x),
                        ),
                        (
                            "goal_y",
                            ourgoal.map_or(-1, |(_, y)| i64::from(y)),
                            state.goal.map_or(-1, |g| g.y),
                        ),
                    ];
                    for (name, o, t) in rows {
                        if f >= LONG_WORD_GREAT_LAKES {
                            continue;
                        }
                        body_all += 1;
                        if o == t {
                            continue;
                        }
                        // The **standing trio**, older than any window
                        // on this map and already excused: `1/24`, `1/25`
                        // and `1/26` stand a constant **(24, 24)** off
                        // their cell, from before this capture opens
                        // (`docs/ORACLE.md`, run76/run83's rows). They
                        // never move and never draw, so they are counted
                        // apart rather than mixed into the live residue.
                        if state.who == 1 && (24..=26).contains(&state.o) {
                            body_trio += 1;
                        } else {
                            body_bad += 1;
                        }
                        posfirst.entry((state.who, state.o)).or_insert(f);
                        if posw.len() < 40 {
                            posw.push(format!(
                                "frame {f}: {}/{} {name} ours {o} theirs {t}",
                                state.who, state.o
                            ));
                        }
                    }
                }
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    // The known seam, and the only one left below the
                    // word: a guy on the **walk category** whose slot
                    // this crate resolves to a different one of `SLOG` /
                    // `WALK` / `JOG` (`docs/ANIM.md` §4.3's speed test).
                    // Three slots of one category cost no draw, which is
                    // why the stream runs 139 frames past the first of
                    // them. Counted and floored rather than listed; a
                    // disagreement anywhere else is a failure.
                    let walking = i64::from(sim::anim::category(ours.anim)) == 8
                        || g.cur_anim.map(|a| sim::anim::category(a as i8)) == Some(8);
                    let rows: [(&str, i64, Option<i64>); 4] = [
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                    ];
                    for (name, o, t) in rows {
                        let Some(t) = t else { continue };
                        if f >= LONG_WORD_GREAT_LAKES {
                            above += 1;
                            continue;
                        }
                        compared += 1;
                        if o == t {
                            continue;
                        }
                        if walking {
                            walk += 1;
                            if walkw.len() < 20 {
                                walkw.push(format!(
                                    "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                    state.who, state.o
                                ));
                            }
                            continue;
                        }
                        if wrong.len() < 20 {
                            wrong.push(format!(
                                "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                state.who, state.o
                            ));
                        }
                    }
                }
            }
        }
        eprintln!(
            "run97 clocks: {compared} fields below the word, {walk} of them the \
             walk-slot seam, {above} above (printed, not pinned), {unmatched} \
             unmatched"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        for w in &walkw {
            eprintln!("  WALK {w}");
        }
        eprintln!(
            "run97 bodies: {body_bad} of {body_all} point-and-goal fields below the word wrong ({body_trio} more on the standing trio over {frames_below} frames), on {} unit(s) — {posfirst:?}",
            posfirst.len()
        );
        for w in &posw {
            eprintln!("  BODY {w}");
        }
        assert!(
            compared >= 120_000,
            "run97's own clock rows are missing — the wrong file: {compared} below"
        );
        assert!(
            wrong.is_empty(),
            "run97's clocks parted outside the walk-slot seam: {wrong:?}"
        );
        // **The walk-slot band is closed** (item 352). It was on player
        // 1's army from run97 block 8443 to the word, and it cost the
        // same ~33.3 fields a frame while it was fifteen units' — 4,615
        // over the 139 frames `[8443, 8582)` on item 346 and 5,853 over
        // the 176 of `[8443, 8619)` on item 348, 33.5 apiece. Item 350
        // took it to 1,747 over the 185 frames `[8443, 8628)`, 9.4
        // apiece, by taking the probe's six out of the army
        // (`docs/ARMY.md` §3.4); item 352 took it to **zero** by putting
        // the army's muster on the original's own cell (§13's two
        // predicates — the `<= 4` spacing test and `mask_me`'s `0x4000`).
        // Measured on 352's own window, `[8443, 8663)`, the base is
        // **2,898, 13.2 a frame** — which is the division this comment
        // keeps asking for, run against itself: 1,747 and 2,898 are the
        // same simulation through two window widths.
        //
        // So this is an **equality** now, not a floor: item 347 read the
        // band as a free `walk_variant` choice costing no draw, and the
        // truth was that the nine were walking somewhere else. There is
        // no remaining seam for a walking guy's slot to differ in below
        // the word, and a successor that reopens one should say so here
        // rather than raise a ceiling.
        //
        // **Item 385 says so.** Moving the word 9182 → 9415 brought 233
        // frames that had never been compared under it, and two of them
        // carry a row: `1/35`'s guy 0 on 9338 and 9339, `cur_anim` 8
        // against 7 — one slot of the walk category, the seam this band
        // was always about and not a new one. The band over item 352's
        // own `[8443, 9182)` is still **empty**, which is what the
        // assertion below says by naming the two rows rather than
        // counting them: a count would have hidden which frames they are
        // on, and the frames are the whole point.
        assert_eq!(
            walkw,
            vec![
                "frame 9338: 1/35 guy 0 cur_anim ours 8 theirs 7".to_string(),
                "frame 9339: 1/35 guy 0 cur_anim ours 8 theirs 7".to_string(),
            ],
            "run97's walk-slot band is not the two rows item 385 left it on, \
             over the {} frames below the word",
            LONG_WORD_GREAT_LAKES - 8_443
        );
        assert!(
            walkw.iter().all(|w| !w.starts_with("frame 91")
                && !w.starts_with("frame 90")
                && !w.starts_with("frame 8")),
            "the walk-slot band reopened *below* 9182, which item 352 closed \
             at zero: {walkw:?}"
        );
        // **The rest of the record, floored** (item 350). Three families,
        // and only the second is anyone's current item:
        //
        // - the **standing trio** `1/24`/`1/25`/`1/26`, a constant
        //   (24, 24) older than every window on this map and already
        //   excused — counted apart, and exactly `3 × 2 × frames`, which
        //   is what the equality below says rather than a literal that a
        //   wider window breaks;
        // - ~~**`1/31`–`1/39` from 8442**~~ — **closed by item 352**: the
        //   frame is an `Army::find_target`, both sides spend its two
        //   `+0x7df` score draws, and the army's muster is the original's
        //   cell (58, 29) from `docs/ARMY.md` §13's two corrected
        //   predicates. All nine stand on the original's own
        //   `x_internal`/`y_internal` and hold its own path goal on 8442;
        // - **`1/27`/`1/28`/`1/29`/`1/41` from 8579**, and `1/33` from
        //   8584, the late drift on the long walk southwest — what the
        //   601 below is, together with `1/24`–`1/26`'s goal rows;
        // - and **`1/35` from 9329** — item 385's own two fields, on
        //   frames the word had always covered until it moved 9182 →
        //   9415. Below 9182 the count is still 6. Its neighbour is the
        //   walk-slot pair on 9338–9339 above: one unit, one late walk,
        //   and the only thing this item changed about it is that it is
        //   now compared.
        assert!(
            body_all >= 121_224,
            "run97's own bodies are missing — the wrong file: {body_all} fields"
        );
        assert_eq!(
            body_trio,
            6 * frames_below,
            "the standing trio is not off on exactly its own two fields a \
             frame over {frames_below} frames: {body_trio}"
        );
        assert!(
            body_bad <= 8,
            "run97's point-and-goal residue grew: {body_bad} of {body_all} \
             fields below the word, the floor is 8 — {posfirst:?}"
        );
    }

    /// **run98's animation clocks, and the seated scholar's own** — East
    /// Indies `[7879, 8788]`, 910 frames of `GUYS` detail across the map's
    /// word at 8495 (item 340).
    ///
    /// **This is the value diff for the scholar's teach slot.** Great
    /// Lakes' 8374 was settled by arithmetic — the map's first scholar was
    /// seated on 8272 and the original wrapped its animation 103 frames
    /// later, which is `SCHOLAR-DEFAULT-AGE0`'s slot **27** and no idle
    /// variant of that piece — but nothing on Great Lakes dumps a clock
    /// after 8043, so the slot itself was an inference. run98 dumps the
    /// other map's first scholar, `1/22`, from the frame it is seated:
    /// `cur_anim 25, cur_time 1, end_time 30`, and 25 is `variant 0 +
    /// 0x19`. The original says outright that a scholar inside its
    /// university plays a `Scholar Teach` slot rather than an idle
    /// (`docs/ANIM.md` §4.11), and this crate now reads the same clock on
    /// every frame of it.
    ///
    /// **The residue is `1/0`'s walk slot**, seven rows on 8242–8248 where
    /// this crate reads `CHAR_SLOG` and the original `CHAR_WALK` — the
    /// AI's scout, `walk_variant`'s speed test (`docs/ANIM.md` §4.3), and
    /// nothing to do with this item. They are pinned by value so the
    /// successor has to come back and change them rather than let them
    /// drift.
    ///
    /// Above the word both streams are nobody's (item 89(c)), so those
    /// rows are counted and printed and not asserted.
    ///
    /// Nothing here is installed: run98's own `frame_guys` never reach the
    /// simulation, which is built from run54's start and driven forward
    /// 8,788 frames.
    #[test]
    fn run98_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r98)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run98-eastindies-valuewindow.txt"),
        ) else {
            eprintln!("skipping: no run54/run98 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text98 = crate::capture::read(&r98);
        let theirs = Log::parse(&text98)
            .initial()
            .expect("run98 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first() == Some(&7_879) && window.last() == Some(&8_788) && window.len() >= 910,
            "run98's clock window is not [7879, 8788]: {:?}..{:?} ({})",
            window.first(),
            window.last(),
            window.len()
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut above = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        // The seating's own rows, by value: the first frame this crate
        // and the dump agree the scholar is on a teach slot, and the
        // clock it runs on afterwards.
        let mut seating: Vec<(i64, i64, i64, i64)> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                if !(0..8).contains(&state.who) {
                    continue;
                }
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                let is_scholar = built.sim.worker_of(u) == sim::orders::Worker::Scholar;
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    if is_scholar
                        && n == 0
                        && (EAST_INDIES_FIRST_SCHOLAR..EAST_INDIES_FIRST_SCHOLAR + 3).contains(&f)
                        && let (Some(a), Some(t), Some(e)) = (g.cur_anim, g.cur_time, g.end_time)
                    {
                        seating.push((f, a, t, e));
                    }
                    let rows: [(&str, i64, Option<i64>); 4] = [
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                    ];
                    for (name, o, t) in rows {
                        let Some(t) = t else { continue };
                        // Past the word both streams are nobody's.
                        if f >= LONG_WORD_EAST_INDIES {
                            above += 1;
                            continue;
                        }
                        compared += 1;
                        if o == t {
                            continue;
                        }
                        if wrong.len() < 20 {
                            wrong.push(format!(
                                "frame {f}: {}/{} guy {n} {name} ours {o} theirs {t}",
                                state.who, state.o
                            ));
                        }
                    }
                }
            }
        }
        eprintln!(
            "run98 clocks: {compared} fields below the word, {above} above \
             (printed, not pinned), {unmatched} unmatched"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        // **The whole window is below the word since item 346** — East
        // Indies' 8495 was the seated scholar's first wrap and the fix
        // carried the word to 9711, past 8788. So `above` is zero by
        // construction now, and the floor is the one that matters: every
        // one of run98's 126,508 clock fields is a checked prediction.
        assert!(
            compared >= 126_000,
            "run98's own clock rows are missing — the wrong file: \
             {compared} below, {above} above"
        );
        // **The scholar's teach slot, off the original's own dump.**
        assert_eq!(
            seating,
            vec![
                (EAST_INDIES_FIRST_SCHOLAR, 25, 1, 30),
                (EAST_INDIES_FIRST_SCHOLAR + 1, 25, 2, 30),
                (EAST_INDIES_FIRST_SCHOLAR + 2, 25, 3, 30),
            ],
            "run98's first scholar is seated on slot 25 — `variant 0 + \
             0x19` — and runs a 30-frame clock from the frame it enters"
        );
        // The whole residue, pinned by value: `1/0`'s walk slot on seven
        // frames, and nothing else in 84,888 fields.
        assert_eq!(
            wrong,
            vec![
                "frame 8242: 1/0 guy 1 cur_anim ours 7 theirs 8",
                "frame 8243: 1/0 guy 0 cur_anim ours 7 theirs 8",
                "frame 8243: 1/0 guy 1 cur_anim ours 7 theirs 8",
                "frame 8244: 1/0 guy 0 cur_anim ours 7 theirs 8",
                "frame 8244: 1/0 guy 1 cur_anim ours 7 theirs 8",
                "frame 8245: 1/0 guy 0 cur_anim ours 7 theirs 8",
                "frame 8248: 1/0 guy 0 cur_anim ours 7 theirs 8",
            ],
            "run98's clock residue below the word, whole"
        );
    }

    #[test]
    fn run73_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r73)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run73-greatlakes-caravanstart.txt"),
        ) else {
            eprintln!("skipping: no run53/run73 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &trace);

        let text73 = crate::capture::read(&r73);
        let theirs = Log::parse(&text73)
            .initial()
            .expect("run73 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first().is_some_and(|&n| n <= 5_565)
                && window.last().is_some_and(|&n| n >= 5_578)
                && window.len() >= 14,
            "run73's `DUMP_ALL` window, as sim-frames: {window:?}"
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The window's own clocks are never installed: this is the check.
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut crew_rows = 0usize;
        let mut caravan_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so their clocks are not
                // this crate's prediction; the players' are.
                if !(0..8).contains(&state.who) {
                    continue;
                }
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    if n >= sim::anim::SQUAD_SIZE {
                        crew_rows += 1;
                    }
                    if (state.who, state.o) == (1, 23) {
                        caravan_rows += 1;
                    }
                    // **The whole of what this crate models of the
                    // record**, not the clock alone: `des`, `des_angle`
                    // and the speed pair were parsed and compared
                    // nowhere until item 210, and they are the arrival
                    // draw's own inputs — `Guy::move` draws on `cur_anim
                    // == 8 && stopped` once `angle` reaches `des_angle`,
                    // and whether the slot lands on 8 rather than 9 is
                    // `avg_speed`'s doing (`docs/ANIM.md` §4.3).
                    let (body, facing, des, des_angle) = match ours.follow {
                        Some(b) => (b.body, b.facing, b.des, b.des_angle),
                        None => (
                            built.sim.units[u].movement.body,
                            built.sim.units[u].movement.facing,
                            built.sim.units[u].pos,
                            // **A trackless crew guy's `des_angle` is guy
                            // 0's `angle`, not its heading** — the crew
                            // loop of `Guy::set_angle` hands it the angle
                            // just turned to, so it is settled on every
                            // frame and never owed a turn of its own. The
                            // dump says it outright: run64's `1/18` guy 0
                            // reads `angle 1145324629, des_angle
                            // 560594944` on the frame it starts turning,
                            // and its two crew figures read `1145324629`
                            // for both. That is what
                            // [`sim::Sim::guys_follow`]'s `settled ||
                            // g >= SQUAD_SIZE` already models.
                            if n >= sim::anim::SQUAD_SIZE {
                                built.sim.units[u].movement.facing
                            } else {
                                built.sim.units[u].movement.heading
                            },
                        ),
                    };
                    let rows: [(&str, i64, Option<i64>); 15] = [
                        ("x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                        ("y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                        ("angle", i64::from(facing.0), g.angle),
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                        ("gpiece", i64::from(ours.gpiece), g.gpiece),
                        ("stopped", i64::from(ours.stopped), g.stopped),
                        ("des_x", i64::from(des.x), g.des.map(|p| p.x)),
                        ("des_y", i64::from(des.y), g.des.map(|p| p.y)),
                        ("des_angle", i64::from(des_angle.0), g.des_angle),
                        ("last_speed", i64::from(body.last_speed), g.last_speed),
                        ("avg_speed", i64::from(body.avg_speed), g.avg_speed),
                        ("guy_num", i64::try_from(n).unwrap(), g.guy_num),
                    ];
                    for (name, ours, theirs) in rows {
                        let Some(theirs) = theirs else { continue };
                        compared += 1;
                        if ours == theirs {
                            continue;
                        }
                        let row = format!(
                            "frame {f}: {}/{} guy {n} {name} ours {ours} theirs {theirs}",
                            state.who, state.o
                        );
                        if wrong.len() < 12 {
                            wrong.push(row);
                        }
                    }
                }
            }
        }
        eprintln!(
            "run73 clocks: {compared} fields over {crew_rows} crew rows and \
             {caravan_rows} of the caravan's, {unmatched} of the dump's units \
             this crate has no unit for"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 2_000 && caravan_rows >= 24,
            "the window's own rows: {compared} fields, {caravan_rows} of them the \
             caravan's — a capture with neither is the wrong file"
        );
        assert!(wrong.is_empty(), "run73's clocks parted: {wrong:?}");
    }

    /// **run74 — the Merchant that turns** (2026-09-04, item 207).
    ///
    /// run53's game to 5,800 frames with the **cheap** per-frame dump
    /// narrowed to `[5700, 5800)`, at run33's own `[End Frame]` detail. The
    /// question is a position, a facing and a path stack, all of which
    /// `UNITS=3` writes, so this needs no `DUMP_ALL` window and costs 15 MB
    /// where run73's sixteen frames cost a gigabyte.
    ///
    /// Great Lakes' word parts at **5786** on a single draw,
    /// `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389` —
    /// `move_step`'s **far** turn-in-place arm, which spends the idle roll
    /// for a guy asked for a turn animation its packet does not have. It is
    /// the only turn draw either side spends in the whole 5,800 frames, and
    /// of the six units moving on that frame exactly one **packs**, so
    /// exactly one carries `guy_flags & 8`: the AI's Merchant `1/24`, born
    /// on 5753. `MERCHANT` is `docs/ANIM.md` §4.8's own row — it packs,
    /// names no turn in `unit_graphics.xml`, and carries the bit anyway —
    /// so the unit was named before the capture was booked.
    ///
    /// This crate's Merchant never takes either turn arm. It pops the
    /// world-grid waypoint `(44088, 17208)` on **5781** — `manh` 377
    /// against that node's `tolerance` 384 — which swings the heading
    /// 49.25° and leaves 43.93° owed after the frame's turn, 1.07° under
    /// `move_step`'s 45° gate, so it walks the turn out over 5781–5788
    /// instead of standing for one frame of it.
    ///
    /// Driven through [`run_traced`], so this is the whole record and not
    /// the merchant's: positions, angles, order lists, path stacks,
    /// `mylos`, the packed bit, the collision block and the buildings, on
    /// every frame the window carries.
    #[test]
    fn run74_s_window_is_where_great_lakes_merchant_turns() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run74-greatlakes-merchantturn.txt"),
            trace("rontrace-run74.log"),
        ) else {
            eprintln!("skipping: no run74 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first().is_some_and(|&n| n <= 5_701)
                && blocks.last().is_some_and(|&n| n >= 5_790)
                && blocks.len() >= 90,
            "run74's window, as the frames that carry a unit record: {:?}..{:?} \
             ({} blocks) — a file with fewer is the wrong file",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );

        // Every unit that ever leaves the original's point, and the frame it
        // does — the Merchant's own question, and everybody else's beside it.
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let orders: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run74: {} unit fields, {orders} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 24))
            .take(24)
        {
            eprintln!("  order {}/{} f{}: {:?}", d.who, d.o, d.frame, d.what);
        }
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 24))
            .take(8)
        {
            eprintln!("  angle {d:?}");
        }
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 24))
            .take(8)
        {
            eprintln!("  pos {d:?}");
        }
        assert!(
            orders >= 1_000 && angles >= 1_000,
            "the window's own rows: {orders} order fields and {angles} angles — \
             a capture below `UNITS=3` is the wrong file"
        );
    }

    /// **run75 — the scout's walk down the river, and the speed layer
    /// nothing here had** (2026-09-04, item 208).
    ///
    /// run53's game to 6,160 frames with the **cheap** per-frame dump
    /// narrowed to `[5845, 6160)` — run74's recipe with a window six
    /// hundred frames later and three times as long, 75 MB and five
    /// minutes. `rngcmp.py rontrace-run53.log rontrace-run75.log`:
    /// **6,161 frames, zero differing**, so it is run53's game and the
    /// seventh capture in a row for which a window costs the stream
    /// nothing.
    ///
    /// **What it was booked for.** Great Lakes' word parted at 6080 on 41
    /// draws the original spends none of: the AI scout `1/0` arrives at
    /// its explore target, goes idle, and runs the whole of
    /// `docs/SCOUT.md` on a frame the original's is still walking. The
    /// original's scout takes the *same* order to the *same* cell on the
    /// same frame — 5851, `(40440, 30456)`, an eight-node path this
    /// capture prints node for node identical — and reaches it **71
    /// frames later**, so its own `think_scout` is at 6151.
    ///
    /// The seventy-one frames are a **speed** this crate did not have.
    /// `UnitData::get_speed@00608720`'s land arm halves the step for a
    /// unit standing on a tile whose mask carries `0x800` while its own
    /// `z_internal` is not above zero, and `crates/sim` implemented none
    /// of that function's third layer at all — [`sim::Sim::get_speed`]
    /// returned the cached aura speed for every unit that is not an
    /// animal. The dump is unambiguous about both halves: `z_internal` is
    /// 14 on 5948, **0** on every frame from 5949 to 6089, and 17 on 6090,
    /// and the step is 34 outside that span and 17 inside it, on a
    /// `myspeed` of 34 throughout. It is the river bed south of the AI's
    /// second city.
    ///
    /// **And the `^ 0x63637` is not a predicate.** The decompilation of
    /// this function, of its `+0x2f` wrapper and of
    /// `SubObjectData::log_data` all XOR the coordinates with `0x63637`,
    /// which reads like a guard on the `z` test — `(z ^ 0x63637) > 0`
    /// skipping the halving would be `z < 0`, and the capture halves at
    /// `z == 0`. The listing settles it in one line: `00608705` is
    /// `xorl $0x63637, %eax` on `x_internal` before it is *passed*, so
    /// **the three coordinates are stored obfuscated** and every reader,
    /// the gamelog's printer included, decodes them. The predicate is
    /// `z > 0`.
    ///
    /// With the halving in, the scout is on the original's point for
    /// **every one of the window's 315 frames**, and Great Lakes' word
    /// runs 6080 -> **6151**, where the next frame is one figure draw
    /// (`Guy::move+0x19f` against `Guy::inc_time+0x271`).
    ///
    /// Driven through [`run_traced`], so this is the whole record and not
    /// the scout's.
    #[test]
    fn run75_s_window_is_the_scout_s_walk_down_the_river() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run75-greatlakes-scoutwalk.txt"),
            trace("rontrace-run75.log"),
        ) else {
            eprintln!("skipping: no run75 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first().is_some_and(|&n| n <= 5_846)
                && blocks.last().is_some_and(|&n| n >= 6_150)
                && blocks.len() >= 300,
            "run75's window, as the frames that carry a unit record: {:?}..{:?} \
             ({} blocks) — a file with fewer is the wrong file",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );

        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let orders: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run75: {} unit fields, {orders} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 0))
            .take(8)
        {
            eprintln!("  pos {d:?}");
        }
        assert!(
            !parted.contains_key(&(1, 0)),
            "the scout leaves the original's point at {:?} — the river's \
             halving is what put it there",
            parted.get(&(1, 0))
        );
        // And nobody else does either, up to the word.
        //
        // **The quit block is not a frame.** The window is `[5845, 6160)`
        // and the file carries one block past it — 6161, written as the
        // process tears down, a record of a frame that was still being
        // simulated. The human's `0/5` "parts" there and nowhere else,
        // and it used to be excluded by accident: the word was 6151, ten
        // frames below it. With the word past the whole window (item 210)
        // it has to be excluded on purpose, which is the first block after
        // the window's own contiguous run.
        let quit = blocks
            .windows(2)
            .find(|w| w[1] != w[0] + 1)
            .map_or(i64::MAX, |w| w[1]);
        assert!(
            quit > 6_150,
            "run75's window is not contiguous where it should be: first gap \
             before {quit}"
        );
        assert!(
            parted
                .values()
                .all(|&f| f >= LONG_WORD_GREAT_LAKES || f >= quit),
            "a unit leaves the original's point before the word \
             ({LONG_WORD_GREAT_LAKES}), and not in the quit block ({quit}): \
             {parted:?}"
        );
        assert!(
            orders >= 1_000 && angles >= 1_000,
            "the window's own rows: {orders} order fields and {angles} angles — \
             a capture below `UNITS=3` is the wrong file"
        );
    }

    /// **run76 — the AI's Archer squad marching, and where it actually is**
    /// (2026-09-04, item 223).
    ///
    /// run75's recipe with the window moved to `[6640, 6870)` — opened
    /// before `Army::do_forming` issues the squad's group order on 6650, so
    /// the whole march is on disk for the first time.
    ///
    /// **What it was booked for.** Great Lakes' word parts at 6848 on one
    /// `Guy::set_anim+0x97a < Unit::move_step+0x823`: the Archer `1/28`, a
    /// follower of the squad (`docs/ORDERS.md` §15), steps onto its
    /// formation slot one unit cell north and `docs/COLLISION.md` §4.2's
    /// leading edge hits `(890, 511)`, a cell of the standing citizen
    /// `1/13`'s block. Every step of that is forced once the two blocks
    /// overlap — and they do — so the **predicate** the item was booked
    /// against is not what is wrong. The trace says by how much: the
    /// original's own first blocked stand in that neighbourhood is at
    /// **6860**, twelve frames and about 1.7 tiles later.
    ///
    /// No dump on this map reached the squad at all. The three archers are
    /// born on 6612; run18b's `DUMP_ALL` window closes at 6590 and run75,
    /// the longest, stops at 6160. A walk spends no draws, so the trace
    /// cannot place them either.
    ///
    /// **What it found** (2026-09-04). Not the march: the squad is already
    /// apart on 6640, the window's own first frame, twenty-eight frames
    /// after its birth and ten before the group order. This crate stacked
    /// all three Archers on the captain's spot, because `come_out` searched
    /// once and placed the squad; the original recurses on `o_down` and
    /// each member searches for itself (`00617c10:535`, `docs/CITIES.md`
    /// §6.5.1).
    ///
    /// **And it does not search around the same thing** (item 227): a unit
    /// that is not its squad's captain replaces the host `get_inside` gave
    /// it with `get_captain()` at `618022`..`618044`, so the members sweep
    /// the **captain's** ring — `[block_radius, + UNIT_DISEMBARK_DISTANCE]`
    /// from the captain's own `angle` — and not the trainer's. With that
    /// built, all three Archers stand on the original's own points for
    /// every block up to the group order, which the loop above now
    /// asserts; the parting moves 6640 → **6652**.
    ///
    /// **And the march was item 219, closed 2026-09-06.** The captain
    /// steps `26, 13, 26, 26, 13, …` out of 6650 where this crate stepped
    /// `26` flat, and the halving is neither the speed nor the group cap:
    /// it is `unit_masks & 0x100000`, the **one-shot half step** a *soft*
    /// collision leaves behind (`detect_unit_collision@00617060`, `00617817`), spent
    /// and cleared by `move_step@005faf30`'s `005fb1f4`–`005fb219` on the arm that owes
    /// less than 45°. A squadmate in the way is squeezed past rather than
    /// stopped for, and the price is the next frame's half step — so the
    /// three Archers step 13 exactly on the frames after they crowd each
    /// other, and 26 for the rest of the march. The bit was written here
    /// and nothing read it (`docs/COLLISION.md` §7); reading it walks all
    /// three on the original's own point from 6652 to **6861**, and moved
    /// run53's word 6848 → **6862**.
    ///
    /// **And 6862 was the formation's end, not a stand** (item 236,
    /// 2026-09-06). The record said so before any reading did: on 6861
    /// all three Archers' order kind goes `GROUPATTACKTOORDER` →
    /// `ATTACKTOORDER`, `1/28` loses `PATHED` and its whole path stack,
    /// the leader `1/27`'s `dest` goes 1 → 0 with its stack kept, and
    /// `1/29` — processed after the ungroup, in the same frame — plans a
    /// fresh **nine-entry world-grid path**, 768 apart with `tolerance
    /// 384`, and steps the full 26 to `(42968, 24407)` where this crate
    /// stepped 22 to `(42971, 24409)` still in formation. That is
    /// `ungroup_move_order` from end to end.
    ///
    /// What fires it is the follower's own tail: `move_step` answers
    /// **0** from three places — blocked and still owing a turn, blocked
    /// and handed to `resolve_unit_collision`, and a tile the world
    /// refused — and `do_group_move` ungroups the squad on every one of
    /// them (`5e856d`–`5e8660`, `docs/ORDERS.md` §8.3). `1/28` is
    /// squeezed onto its own cell centre `(42792, 24648)` on 6861 by
    /// `resolve_unit_collision`, which is the 0; this crate did the same
    /// snap and answered `Did::Something`, so the formation held. With
    /// the two arms answering `Did::Nothing` and the follower reading
    /// them, all three Archers hold the original's own point for the
    /// **whole** window, and run53's word moved 6862 → **6982**.
    ///
    /// Driven through [`run_traced`], so this is the whole record and not
    /// the squad's.
    #[test]
    fn run76_s_window_is_the_ai_squad_s_march() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run76-greatlakes-archermarch.txt"),
            trace("rontrace-run76.log"),
        ) else {
            eprintln!("skipping: no run76 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        // **The second capture that dates a suspended search, and the
        // whole of the queue's item 304** (item 301). `UnitData::start_dist`
        // (`+0x130`) is written by `astar_path@00683770`'s suspend block
        // and by nothing else in the executable, so the block it first
        // reads non-zero is the block after the frame a 48-grid search ran
        // out of budget short of its goal. Over run76's window that is
        // `1/28` alone, on block **6861**, reading **7680** — the
        // start-to-goal Manhattan of the plan
        // `resolve_unit_collision` step 6 started on frame 6860, and the
        // same search this crate runs there (its own start is 11 units
        // further out, at 7669).
        //
        // And the row that makes it matter: on the **same frame** the
        // original's order goes `GROUP_ATTACK_TO` → `ATTACK_TO`, and that
        // enqueue frees the search through `clear_partial_path`, so block
        // 6862 plans afresh on the world grid — ten waypoints at tolerance
        // 0x180, which no 48-grid reconstruction can produce. This crate is
        // already on a plain `ATTACK_TO` before 6858, so its formation
        // ended early and nothing is left to free the stash; that is why
        // `docs/PATHFINDER.md` §18.3's block is landed unwired.
        let states = log.frame_states();
        let stamped: Vec<(i64, i64, i64, i64)> = states
            .iter()
            .flat_map(|f| {
                f.units
                    .iter()
                    .filter_map(move |u| Some((f.n, u.who, u.o, u.start_dist?)))
            })
            .filter(|r| r.3 != 0)
            .collect();
        assert!(
            stamped.iter().all(|r| (r.1, r.2, r.3) == (1, 28, 7_680))
                && stamped.first().map(|r| r.0) == Some(6_861)
                && stamped.len() == 9,
            "run76's `start_dist`: {} rows, first {:?} — the suspend's only \
             dumped witness is `1/28`'s, 7680, from 6861",
            stamped.len(),
            stamped.first()
        );
        let kinds = |n: i64| -> Vec<i64> {
            states
                .iter()
                .find(|f| f.n == n)
                .into_iter()
                .flat_map(|f| f.units.iter())
                .filter(|u| (u.who, u.o) == (1, 28))
                .flat_map(|u| u.orders.iter().map(|o| o.index))
                .collect()
        };
        assert_eq!(
            (kinds(6_860), kinds(6_861)),
            (vec![21], vec![2]),
            "`1/28`'s orders either side of the suspend: a `GROUP_ATTACK_TO` \
             on 6860, a bare `ATTACK_TO` on 6861"
        );

        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();

        // **And `UnitData::start_dist`, on every unit of every block**
        // (item 308, `docs/PATHFINDER.md` §19). The field is written at
        // every detail level and nothing in this crate compared it until
        // 308 gave it a column of its own; with the suspend wired it
        // agrees over the whole window, so `1/28` stamps 7680 on block 6861 — the frame §18.5's
        // `kill_current_path` fix lands on — and holds it.
        //
        // It would have been all disagreement a day ago — an unwired
        // suspend stamps nothing — so this is the check that the wiring
        // stays wired, and it is a value diff rather than a draw stream.
        let sd_compared: usize = report.frames.iter().map(|f| f.search_compared).sum();
        let sd_diverged: Vec<_> = report
            .frames
            .iter()
            .flat_map(|f| f.search_diverged.iter())
            .collect();
        assert!(
            sd_compared == 8_032 && sd_diverged.is_empty(),
            "run76's `start_dist` column: {sd_compared} compared, {} apart — \
             first {:?}",
            sd_diverged.len(),
            sd_diverged.first()
        );
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first().is_some_and(|&n| n <= 6_641)
                && blocks.last().is_some_and(|&n| n >= 6_860)
                && blocks.len() >= 220,
            "run76's window, as the frames that carry a unit record: {:?}..{:?} \
             ({} blocks) — a file with fewer is the wrong file",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );

        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let orders: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run76: {} unit fields, {orders} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        // **The ungroup, asserted** (items 236 and 237). The positions
        // above say the three Archers walk where the original walks; this
        // says they walk there for the original's reason.
        //
        // The window's own fact first, read straight off the dump so that
        // nothing this crate does can move it: the original holds a
        // `GROUPATTACKTOORDER` (`type 21`) for the squad from 6651 to
        // 6860 and a plain `ATTACKTOORDER` after. Item 236 asserted this
        // through the comparison's `Kind` rows, which could not say it —
        // the crate spelled a grouped attack-move `ATTACK_TO` (2), so the
        // row fired on all 630 marching frames whatever the crate did and
        // marked the frames the *original* was grouped. Item 237 gave
        // `Order::index` the original's `get_type()`, so the two halves
        // are separable now: the dump's run, and the crate's agreement
        // with it.
        let their_group: Vec<i64> = log
            .frame_states()
            .iter()
            .filter(|f| {
                f.units.iter().any(|u| {
                    u.who == 1 && (27..=29).contains(&u.o) && u.orders.iter().any(|o| o.index == 21)
                })
            })
            .map(|f| f.n)
            .collect();
        assert!(
            their_group.first() == Some(&6_651) && their_group.last() == Some(&6_860),
            "the original's group order does not run 6651..6860 in this file: \
             {:?}..{:?}",
            their_group.first(),
            their_group.last()
        );
        // And now the rows that can fail. `GROUPATTACKTOORDER` is 21 on
        // both sides, so a squad that stays in formation past 6861 — or
        // leaves one early — disagrees on the `Kind` row on the frame it
        // happens rather than three frames later in a position; and the
        // `GROUPORDER` row beneath it says the squad is grouped the same
        // *way*: the leader `find_leader` chose (`oxx`/`whose`), the
        // member's slot (`form_id`), the bearing its slot was laid out on
        // (`group_angle`) and `in_group`. Five of the six agree on all
        // 630 marching unit-frames. Made to fail on purpose: dropping the
        // follower's `move_step` zero (item 236's own fix) puts a `Kind`
        // row on every frame from 6861 to the end of the capture, and
        // negating the slot angle puts a `group_angle` row on all 630.
        use crate::diff::order::OrderMismatch as OM;
        let squad = |d: &&OrderDivergence| d.who == 1 && (27..=29).contains(&d.o);
        let rows = |f: fn(&OM) -> bool| -> Vec<&OrderDivergence> {
            report
                .frames
                .iter()
                .flat_map(|fr| fr.order_diverged.iter())
                .filter(|d| squad(d) && f(&d.what))
                .collect()
        };
        let kinds = rows(|w| {
            matches!(
                w,
                OM::Kind { .. } | OM::Unspellable { .. } | OM::Header { .. }
            ) || matches!(w, OM::Group { field, .. } if *field != "id")
        });
        // The sixth is `id` — `GroupData +0x4`, which this crate stands in
        // for with the army group's slot and so cannot match (§1.7, and
        // `OrderMismatch::scores`). It is pinned at exactly one row per
        // grouped unit-frame so that the stand-in cannot quietly start
        // failing for a second reason.
        let ids = rows(|w| matches!(w, OM::Group { field: "id", .. }));
        eprintln!(
            "run76: the original's group order runs {:?}..{:?} ({} frames); \
             squad kind/group rows: {} scoring, {} `id` (the stand-in)",
            their_group.first(),
            their_group.last(),
            their_group.len(),
            kinds.len(),
            ids.len()
        );
        assert!(
            kinds.is_empty(),
            "the squad's order kind or group row parts from the original's: {:?}",
            kinds.iter().take(6).collect::<Vec<_>>()
        );
        assert_eq!(
            ids.len(),
            their_group.len() * 3,
            "the group-id stand-in should be one row per grouped unit-frame"
        );
        // **The ungroup, pinned** (item 236). From the frame the formation
        // ends, the three Archers agree with the original on every order
        // and path field the dump prints — the nine-entry world-grid path
        // each plans for itself, its `tolerance 384` rows, the `PATHED`
        // bit two of them lose, and the waypoint and `coll_x/coll_y` that
        // follow. Before the follower read `move_step`'s 0 this window
        // carried `PathLength`, `PathTo`, `PathField`, `Move` and `Coll`
        // rows from 6861 to its end, because all three were still walking
        // a formation the original had already dissolved.
        let post: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| d.who == 1 && (27..=29).contains(&d.o) && d.frame >= 6_861)
            .collect();
        assert!(
            post.is_empty(),
            "the squad's orders part after the ungroup: {:?}",
            post.iter().take(6).collect::<Vec<_>>()
        );
        // The squad itself, printed whole: the three archers are the units
        // the item is about, and their first disagreeing field is what says
        // whether the slot, the speed or the step is wrong.
        for d in report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| d.who == 1 && (27..=29).contains(&d.o))
            .take(24)
        {
            eprintln!("  squad {d:?}");
        }
        assert!(
            orders >= 1_000 && angles >= 1_000,
            "the window's own rows: {orders} order fields and {angles} angles — \
             a capture below `UNITS=3` is the wrong file"
        );
        // **The birth, pinned** (item 227). The window opens twenty-eight
        // frames after the three Archers are born and ten before the group
        // order on 6650, so every block up to and including 6650 is their
        // exit placement standing still. All three must be on the
        // original's own point for all of it: the captain off the
        // trainer's ring, `1/28` and `1/29` off the **captain's**
        // (`docs/CITIES.md` §6.5.1). Before that was found this asserted
        // nothing and the two members stood on the captain's spot from the
        // window's first frame.
        for o in 27..=29 {
            let f = parted.get(&(1, o)).copied().unwrap_or(i64::MAX);
            assert!(
                f > 6_650,
                "Archer 1/{o} parts at {f}: the squad's exit spots are the \
                 birth placement, and every frame to the group order on 6650 \
                 is it standing still"
            );
            // **And the march itself** (item 219, then 236). The
            // one-shot half step carries all three from the group order
            // to 6861, and the ungroup carries them past the standing
            // citizen `1/13` to the end of the capture: not one of the
            // three leaves the original's point in the whole window, so
            // this is pinned against the last block rather than a frame
            // inside it.
            assert!(
                f > blocks.last().copied().unwrap_or(6_860),
                "Archer 1/{o} parts at {f}: the march is the one-shot half \
                 step and then the ungroup, and neither of the three leaves \
                 the original's point before the window ends"
            );
        }
    }

    /// **run79 — the second squad's march, and the half step's second
    /// sample** (2026-09-06, item 219).
    ///
    /// run53's game again, window `[6910, 7250)`: a squad of three
    /// Longbowmen born at 6994 out of the same Barracks run76's Archers
    /// came from, its whole march to the window's end, and a second squad
    /// born at 7213. `rngcmp` puts it on run53 over 7,301 frames.
    ///
    /// **What it is for.** The one-shot half step (`unit_masks &
    /// 0x100000`, `docs/ORDERS.md` §15.1) was established on run76's two
    /// halvings; this is 256 more frames of it, on a different squad of a
    /// different type. The correlation is exact and printed by this test:
    /// **every** frame of `1/31`'s march whose predecessor carried the bit
    /// steps 13, 14 or 15 against a march of 25 to 30, and the one frame
    /// that carries the bit and does *not* halve is a turn-in-place, which
    /// returns before the halving block and so keeps the bit for the frame
    /// after — `move_step@005faf30`'s shape, asserted rather than argued.
    ///
    /// The whole window is **past** run53's own parting at 6862, so this
    /// pins the original's own record and not the two sides' agreement:
    /// the assertions are about the dump, and the positions are printed.
    #[test]
    fn run79_s_window_is_the_half_step_s_second_sample() {
        let Some(path) = dump("gamelog-run79-greatlakes-secondsquad.txt") else {
            eprintln!("skipping: no run79 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        // `1/31` is the second squad's captain — `o_up −1`, `o_down 32`,
        // group 66 — born on 6994 and marching to the window's end.
        let walk: Vec<(i64, LogPos, i64)> = log
            .frame_states()
            .into_iter()
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 31)?;
                Some((f.n, u.pos, u.unit_masks.unwrap_or(0)))
            })
            .collect();
        assert!(
            walk.len() >= 250 && walk.first().is_some_and(|r| r.0 <= 6_995),
            "run79's `1/31`: {} frames from {:?} — the wrong file",
            walk.len(),
            walk.first().map(|r| r.0)
        );
        // `vector_dist`, the original's own metric for a step.
        let dist = |dx: i64, dy: i64| {
            let (a, b) = (dx.abs(), dy.abs());
            a.max(b) + a.min(b) / 2
        };
        let mut flagged: Vec<(i64, i64)> = Vec::new();
        let mut plain: Vec<i64> = Vec::new();
        for w in walk.windows(2) {
            let (_, from, mask) = w[0];
            let (frame, to, _) = w[1];
            let step = dist(to.x - from.x, to.y - from.y);
            if mask & 0x0010_0000 != 0 {
                flagged.push((frame, step));
            } else {
                plain.push(step);
            }
        }
        eprintln!(
            "run79: {} marching frames, {} of them after a flagged one: {flagged:?}",
            plain.len(),
            flagged.len(),
        );
        assert!(
            flagged.len() >= 5,
            "run79 carries {} flagged frames — the sample is the point",
            flagged.len()
        );
        // The march's own pace, so the halved frames have something to be
        // half **of**: the flag is rare and the plain step is not.
        let marching = plain.iter().filter(|&&d| d >= 24).count();
        assert!(
            marching >= 150,
            "only {marching} full steps in the window — the wrong unit"
        );
        // **Every** frame after a flagged one is halved, bar the frame the
        // unit spends turning in place: `move_step` returns from the
        // turn-in-place arm before the halving block, so the bit survives
        // it and is spent on the frame after — which is 7020 and 7021.
        for &(frame, step) in &flagged {
            assert!(
                step <= 15 || step == 0,
                "frame {frame} follows a `unit_masks & 0x100000` and steps \
                 {step}: the one-shot half step is not one-shot"
            );
        }
        assert!(
            flagged.iter().filter(|&&(_, d)| d == 0).count() <= 1,
            "more than one standing frame among {flagged:?} — the \
             turn-in-place reading covers exactly one"
        );
        // **And the converse, which is what makes this a check and not an
        // illustration**: no *unflagged* frame of the march is a half step
        // either. Six of `1/31`'s 256 frames step 13, 14 or 15, and the
        // flag accounts for all six — so the bit is not one explanation
        // among several, it is the explanation.
        assert!(
            plain.iter().all(|&d| !(13..=15).contains(&d)),
            "an unflagged half step in the march: {:?}",
            plain
                .iter()
                .filter(|&&d| (13..=15).contains(&d))
                .collect::<Vec<_>>()
        );
    }

    /// **run84 — who actually spends Great Lakes 6994's two draws** (item
    /// 249, 2026-09-06).
    ///
    /// The word parted at 6994 on two draws of
    /// `Unit::do_move+0xe84 < Unit::do_attack_to+0x11 < Unit::do_job+0x4b`
    /// — `sim::orders::SITE_MOVE_GRID`, the `% 5` roll a move spends when
    /// `find_path` refuses its straight line — with a third on 6995. The
    /// obvious reading is that they belong to the **marching** squad, the
    /// three Archers `1/27`–`1/29` that are the only units on an
    /// `ATTACK_TO` when the frame opens. run84 says they cannot be: all
    /// three open 6994 with `dest = 1` and `unit_masks & 8` set, and
    /// `do_move` reaches the roll only past `dest == 0` (which takes a
    /// waypoint and clears the bit) or with the bit already clear. Their
    /// records across the frame are a step and a tolerance arrival,
    /// nothing else, and this crate reproduces them to the unit.
    ///
    /// **The draws are the *second* squad's.** `1/31`, `1/32` and `1/33`
    /// are born on 6993 — the three `Guy::init_real < Unit::init <
    /// Objects::init_unit` draws that frame — stand orderless when 6994
    /// opens, and by 6995 each carries an `ATTACK_TO` of its own, a path,
    /// and the marching squad's group. A fresh move plans, takes its first
    /// waypoint, clears `line_ok`, and is exactly the shape that reaches
    /// the roll; two of the three reach it on 6994 and the third on 6995.
    /// So the seam is not the pathfinder refusing a line this crate
    /// accepts — it is **an order this crate never issues**, and the
    /// mechanic behind it is `Unit::come_out`'s group move rather than
    /// anything in `do_move`.
    ///
    /// What this pins is the original's own record, frame for frame, so
    /// the successor starts from the units rather than re-deriving them:
    /// the newcomers' three destinations, their shared heading, the group
    /// they land in, and — the part that says the order went to them and
    /// not to the group — that the marching squad's order point does not
    /// move across the same frame.
    ///
    /// The one crate-side row is the join: this simulation already puts
    /// all six in **one army** by 6995 (`Sim::army_of`, `docs/ARMY.md`
    /// §4), which is what makes the missing order an order and not a
    /// membership bug.
    #[test]
    fn run84_says_great_lakes_6994_belongs_to_the_second_squad() {
        let Some(path) = dump("gamelog-run84-greatlakes-makelist.txt") else {
            eprintln!("skipping: no run84 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let states = log.frame_states();
        let at = |n: i64| {
            states
                .iter()
                .find(|f| f.n == n)
                .unwrap_or_else(|| panic!("run84 carries no block {n}"))
        };
        let unit = |n: i64, o: i64| {
            at(n)
                .units
                .iter()
                .find(|u| u.who == 1 && u.o == o)
                .unwrap_or_else(|| panic!("{n}: no 1/{o}"))
        };
        let (before, after) = (GREAT_LAKES_SECOND_SQUAD, GREAT_LAKES_SECOND_SQUAD + 1);
        assert_eq!(
            before, 6994,
            "this test is written about the second squad, not about the headline"
        );

        // **The marching squad cannot be spending the draws.** `dest` is
        // the "I hold a waypoint" flag and `unit_masks & 8` is `line_ok`
        // (`docs/ORDERS.md` §4.4): with both set, `do_move` runs from the
        // speed straight to `move_step` and never reaches the roll.
        for o in [27, 29] {
            let u = unit(before, o);
            let od = u.orders.last().expect("the current order");
            assert_eq!(od.kind, "ATTACKTOORDER", "1/{o} is the marching squad");
            assert_eq!(od.dest, Some(1), "1/{o} opens {before} holding a waypoint");
            assert_eq!(
                u.unit_masks.unwrap_or(0) & 8,
                8,
                "1/{o} opens {before} with `line_ok` set"
            );
        }
        // And the frame leaves their **order** where it was — only the
        // waypoint is consumed. This is what says the new order was not a
        // group order over all six.
        for o in [27, 28, 29] {
            let (a, b) = (unit(before, o), unit(after, o));
            assert_eq!(
                (a.orders_x, a.orders_y),
                (b.orders_x, b.orders_y),
                "1/{o}'s order point moved across {before}"
            );
        }

        // **The second squad, orderless when the frame opens.**
        for o in [31, 32, 33] {
            let u = unit(before, o);
            assert!(
                u.orders.is_empty(),
                "1/{o} already has an order at {before}"
            );
            assert!(u.path.is_empty(), "1/{o} already has a path at {before}");
        }
        // And ordered when it closes: one `ATTACK_TO` each, at three
        // points on one heading, every one snapped to its 48-unit cell
        // centre (`u × 0x30 + 0x18`, `docs/ORDERS.md` §4.1).
        let want = [
            (31, 41_352, 22_920),
            (32, 41_256, 23_016),
            (33, 41_448, 22_824),
        ];
        for (o, x, y) in want {
            let u = unit(after, o);
            assert_eq!(u.orders.len(), 1, "1/{o} carries one order at {after}");
            let od = &u.orders[0];
            assert_eq!(od.kind, "ATTACKTOORDER", "1/{o}");
            assert_eq!(
                od.index,
                i64::from(sim::orders::index::ATTACK_TO),
                "1/{o}'s order index"
            );
            assert_eq!((od.x, od.y), (Some(x), Some(y)), "1/{o}'s destination");
            assert_eq!((x % 0x30, y % 0x30), (0x18, 0x18), "1/{o}: cell centre");
            assert_eq!(od.angle, Some(-560_070_656), "1/{o}: the shared heading");
            assert!(!u.path.is_empty(), "1/{o} planned nothing");
        }
        // One group, all six — so the newcomers joined the marching
        // squad's rather than forming their own.
        let groups: Vec<Option<i64>> = [27, 28, 29, 31, 32, 33]
            .into_iter()
            .map(|o| unit(after, o).group)
            .collect();
        assert!(
            groups.iter().all(|g| *g == groups[0]) && groups[0].is_some_and(|g| g >= 0),
            "the six are not one group at {after}: {groups:?}"
        );
        // The planners they came out of differ, and the successor has to
        // reproduce both: `1/31` is on tile waypoints (tolerance 384) and
        // the other two on the fine grid (96). That difference is why two
        // of the three reach the roll on 6994 and the third on 6995.
        let tol = |o: i64| unit(after, o).path.last().map(|p| p.tolerance);
        assert_eq!(tol(31), Some(384), "1/31's stack is the tile grid's");
        assert_eq!(tol(32), Some(96), "1/32's stack is the fine grid's");
        assert_eq!(tol(33), Some(96), "1/33's stack is the fine grid's");
        eprintln!(
            "run84: {before} is 1/31, 1/32 and 1/33's order, not 1/27-1/29's;              stacks {:?}",
            [31, 32, 33].map(|o| unit(after, o).path.len())
        );
    }

    /// **The order this crate now issues** (item 250, 2026-09-07) — the
    /// crate side of the frame above, against the original's own record.
    ///
    /// `run84_says_great_lakes_6994_belongs_to_the_second_squad` pins what
    /// the original does; this pins that the simulation does the same
    /// thing, to the coordinate. The mechanic is
    /// `Unit::add_to_army@005f7740`'s middle limb (`docs/ARMY.md` §4.3):
    /// with an army already holding units, the newcomer is walked to
    /// `ArmyData::get_unit(0)` — the army's **first member**, not its
    /// muster point and not the group's destination — through
    /// `go_to_unit` and `go_to`, which take a `find_nearby_spot` around
    /// that unit and give the whole squad one group move to it.
    ///
    /// Three things this asserts that the arithmetic could get wrong and
    /// the draw count could not: the **anchor** `(41352, 22920)`, which is
    /// the spot the sweep finds around `1/27` rather than anything on the
    /// army record; the two flanking slots the formation lays out either
    /// side of it; and the **shared heading**, which comes out of
    /// `action_move_near`'s own `set_angle 0` arm rather than from a value
    /// the caller passes. Made to fail on purpose by moving the sweep's
    /// bias angle off `0x55555555`: the anchor lands on a different tile
    /// and all three destinations go with it. It had already failed for
    /// real once — the first build of §4.3 missed
    /// `find_nearby_spot`'s `uber_unit` argument and answered
    /// `(41256, 22872)`, one ring in, **with the whole draw stream
    /// unchanged and the word already 182 frames further on**. That is
    /// what this test is for: a draw-stream gain is not a correctness
    /// proof.
    #[test]
    fn great_lakes_6994_issues_the_second_squad_s_walk_to_the_army() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(theirs)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run84-greatlakes-makelist.txt"),
        ) else {
            eprintln!("skipping: no run53/run84 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let after = GREAT_LAKES_SECOND_SQUAD + 1;
        while built.sim.frame < after {
            built.tick();
        }

        let text84 = crate::capture::read(&theirs);
        let log84 = Log::parse(&text84);
        let states = log84.frame_states();
        let at = states
            .iter()
            .find(|f| f.n == after)
            .unwrap_or_else(|| panic!("run84 carries no block {after}"));
        let of = |o: i64| {
            built
                .sim
                .units
                .iter()
                .position(|u| u.alive() && i64::from(u.owner) == 1 && i64::from(u.index) == o)
                .unwrap_or_else(|| panic!("this crate has no live 1/{o} at {after}"))
        };
        for o in [31, 32, 33] {
            let t = at
                .units
                .iter()
                .find(|u| u.who == 1 && u.o == o)
                .unwrap_or_else(|| panic!("{after}: no 1/{o}"));
            let want = t.orders.first().expect("the original's order");
            let u = of(o);
            let ours = built
                .sim
                .current_order(u)
                .unwrap_or_else(|| panic!("1/{o} carries no order at {after}"));
            let sim::orders::Body::Move(m) = ours.body else {
                panic!("1/{o}'s order is not a move: {:?}", ours.body)
            };
            assert_eq!(
                (i64::from(m.dest.x), i64::from(m.dest.y)),
                (want.x.unwrap(), want.y.unwrap()),
                "1/{o}'s destination at {after}"
            );
            assert_eq!(
                i64::from(m.angle.0),
                want.angle.unwrap(),
                "1/{o}'s heading at {after}"
            );
            assert_eq!(
                i64::from(ours.index()),
                want.index,
                "1/{o}'s order index at {after}"
            );
        }
        // And the marching squad is left alone: the walk is the newcomers'
        // own group move, not a group order over all six.
        for o in [27, 28, 29] {
            let t = at.units.iter().find(|u| u.who == 1 && u.o == o).unwrap();
            let u = of(o);
            let ours = built.sim.current_order(u).expect("the marching order");
            let sim::orders::Body::Move(m) = ours.body else {
                panic!("1/{o}'s order is not a move")
            };
            assert_eq!(
                (Some(i64::from(m.dest.x)), Some(i64::from(m.dest.y))),
                (t.orders_x, t.orders_y),
                "1/{o}'s order point moved across {GREAT_LAKES_SECOND_SQUAD}"
            );
        }
        eprintln!(
            "run84: this crate issues 1/31, 1/32 and 1/33's walk to 1/27 at \
             {GREAT_LAKES_SECOND_SQUAD}, on the original's own three points"
        );
    }

    /// East Indies' word on the **long** capture — the number that took
    /// over as the headline when run39's own length stopped bounding it
    /// (`docs/DECISIONS.md` entry 29's first counter). It is not in
    /// [`FLOORS`] because `FLOORS` is the scored captures' scoreboard and
    /// this map's scored capture is closed; the queue states both.
    ///
    /// **5376** — and the frame is `Leader::produce_building`'s jitter,
    /// not a boat's: the AI sites something and this crate does not, and
    /// the whole of the fishing lineage now agrees for 5,375 frames.
    ///
    /// It was **5285** for the second half of one item, and that half was
    /// **`WData.down`** (`docs/ORDERS.md` §6.8). A deployed boat is the
    /// head of its cell's object chain, so the *second* Fisherman's search
    /// must refuse the fish the first is sitting on — and
    /// `think_fish`'s claim test read the start dump's snapshot, which
    /// still held the good's own terminator. One extra accepted cell, one
    /// extra draw: 178 against the original's 177 (item 48's chain, one
    /// reader at a time).
    ///
    /// It was **5106** for one item, and that item was
    /// `UnitData::calc_gather@00609180` (`docs/ORDERS.md` §6.10) — the
    /// head of `think_fish`, which asks a deployed boat once in 1,024
    /// frames whether it may stay where it is. run54's `1/14` is standing
    /// on its fish there and the original answers yes and spends four
    /// draws; this crate had no answer at all, sent it back through the
    /// 17 × 17 and spent 165. The **first** tile of the spiral to carry
    /// `TData & 0x200` is what the search takes, and the fish is not under
    /// the boat's own tile — which is why run58's dump prints `good_obj 1`
    /// and not 0.
    ///
    /// It was **4988** for one item, and the frame is the last of the
    /// fishing boat's deploy:
    /// the original's guy finishes `CHAR_UNPACK` there and pays the wrap's
    /// idle roll (`Guy::set_anim+0x97a < Guy::inc_time+0x271`), where this
    /// crate spends a farm's. The boat's guy carries **no piece** — the
    /// piece table is seeded from the start dump's own `GUY` blocks and no
    /// Fisherman is in one — so its `end_time` is [`sim::anim::UNKNOWN`]
    /// and no animation of its ever wraps (item 152).
    ///
    /// It was **4950** for one item before that, and that item was the
    /// deploy itself.
    /// `Unit::do_cast` killed every craft but the transport on the frame
    /// after it was queued, so the boat came back idle on 4950, still
    /// packed, and searched its 17 × 17 a second time — 165 draws to none.
    /// The craft table is now loaded (`craftrules.xml`, 55 rows), `0x292`
    /// waits out its `JOB_TIME` of **40**, and `SpellType::cast_unpack`
    /// clears `unit_masks & 0x80000` on frame 4989 exactly as run58's own
    /// `unit_masks 786440 → 262152` does (`docs/ORDERS.md` §6.9).
    ///
    /// It was **4945** for one item, and that item was the **turning
    /// stand** (`docs/ANIM.md` §4.8). `Guy::do_turn` asks a guy with
    /// `guy_flags & 8` for `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, and
    /// `Guy::init_real` sets that bit for every type that **packs** as well
    /// as for a piece that names the turn — so the AI's Fisherman asks for
    /// an animation `unit_graphics.xml` never gives it, `Guy::set_anim`
    /// rewrites the request to `CHAR_DEFAULT`, and a boat on the walk
    /// category pays an idle roll. Run26's dump states the flag by type and
    /// is what makes the two writers a diff rather than a reading.
    ///
    /// **4462** — the sequence and the count part on the same frame, and
    /// the frame is the new ship's first think: theirs makes 55 draws
    /// opening at `Unit::think_fish+0x27a` (`5f4eda`) where this crate
    /// makes 2. `think_fish` has no counterpart here at all.
    ///
    /// It was **4461** for one item, and that item was the AI's Dock —
    /// booked as "a late *decision* or a slow *counter*", and it was
    /// neither: the queue record agrees field for field from the frame the
    /// job is queued (**4376**) to the frame before it lands, and parts on
    /// the **target**. Theirs caps at `job_counter` **8481**, this crate
    /// ran on to 11,280, and 11280 × 100 / 133 is 8481 to the unit — the
    /// **British ship bonus**. Player 1 is British (`tribe 11`),
    /// `BRITISH_SHIP_SPEED` is 33, and `ObjectData::train_time@006508c0`'s
    /// national block applies it to every type whose domain is the sea.
    /// The unit is a **Fisherman** (`TypeIndex` 317), which is why the
    /// frame after it is `think_fish`. `docs/PRODUCTION.md`, "The tail's
    /// first caller"; the second job's 12,030 → 9,045 is the same ratio on
    /// the ramp's next step, so the arm is checked twice over.
    ///
    /// **And it was found by widening, not by reading.** The queue record
    /// was parsed on every capture and compared on one — run39's, which is
    /// 2,600 frames too short to reach the AI's first ship. Moving the loop
    /// into [`compare`] put it on run58 and the twenty-eight frames of
    /// `queued ours 1 theirs 0` were there the same minute.
    ///
    /// It was **4313** for one item, and that item was booked as the
    /// scout's re-think and was a **citizen's** (`docs/SCOUT.md` §11.1).
    /// The 28 draws are one `Unit::think_scout+0x941` and 27 `+0xaba` —
    /// §11's region scan, not the city loop — and the caller is
    /// `Unit::think_peasant+0x2ac`, `think_scout`'s **second** call site,
    /// not `Unit::think`'s tail. `think_peasant`'s AI tail was
    /// untranscribed below `find_gather_spot`: a worker standing in a
    /// region where its leader has no city and which none of the ten
    /// `Sites` claims explores instead. The AI's citizen `1/15` reached
    /// cell (39, 34) on 4312, went idle on 4313, and took the whole
    /// hundred-cell region scan of region 8 at stride 2 — 50 cells
    /// visited, 27 scored, the winner tile (162, 138).
    ///
    /// It was **4275** for one item, and that item was booked as a
    /// woodcutter's clock and was **the tile grid's own tolerance**
    /// (`docs/PATHFINDER.md` §7). The clock was real — the AI's citizen
    /// `1/13` reached its wood tile thirteen frames early and started its
    /// `wait` down from 452 there — but the thirteen frames were three a
    /// leg of the walk out, and the walk was wrong because
    /// `astar_path`'s reconstruction gives a unit that **can transport**
    /// an exact waypoint (`00684bfc`: tolerance 0, not `0x60`) and this
    /// crate gave every tile-grid waypoint `0x60`. `1/13` is granted
    /// `unit_masks & 0x800000` on frame **3580**, the frame after its
    /// Dock finishes, and had been cutting the corner off every leg
    /// since — which is also the whole of `1/13`'s parting at 3647, the
    /// last unit in run57 or run58 to leave the original's point before
    /// the word.
    ///
    /// It was **4020** for one item, and that item was booked as the
    /// scout's and was the **colonist's** (`docs/SYNC.md` §3.25). The
    /// frame's extra `do_cast` pair is the AI's citizen `1/15` becoming
    /// its own transport, not the scout `1/0`, and the scout is right on
    /// both sides — it takes cell (48, 30) on 4005 and re-thinks on 4110,
    /// which run54's own trace shows the original doing too. What was
    /// wrong was two reads, three hundred frames earlier:
    /// `Region::coast_here`'s neighbour probe is `WorldData::get_tregion`
    /// and this crate asked the plain cell region, so
    /// `think_civilian_transport` could not see the shore and sent `1/15`
    /// to cell (37, 36) where the original sends it to **(39, 34)**; and
    /// `Unit::move_step`'s `path.flags & 4` — the waypoint's own
    /// turn-in-place bit, written here since the pathfinder landed and
    /// never read — let it walk through the turn the original stands still
    /// for, reaching its embark point a frame early.
    ///
    /// It was **3978** for one item, and that item was three things in a
    /// row, all of them `Unit::come_out@00617c10` and its caller.
    ///
    /// The draw is the **army coin** at `come_out`'s tail (`+0x25ca`,
    /// `61a1da`), unnamed in the trace and named here from the listing: an
    /// AI unit leaving whatever carried it joins an army unless a coin says
    /// otherwise, and only the **scout** and **naval-scout** lineages throw
    /// one — `is_special` is `is(SCOUT)` by the loader, and the other arm is
    /// `is(BARK)`, `% 3` at `+0x25b0`. run54 reaches the site eleven times
    /// in 24,000 frames and both arms are there (`docs/ARMY.md` §4).
    ///
    /// Then the scout had to come out where the original's does, and that
    /// is `come_out`'s **host** arm — a ring of the *boat's* `block_radius`
    /// out to `+ UNIT_DISEMBARK_DISTANCE`, swept from the *boat's* own
    /// angle, and the passenger turned to that angle with its crew seated
    /// on the track offset. And the ring only reaches land because a boat
    /// lying on the water half of a coastal cell **marks no collision
    /// cells there**: `docs/COLLISION.md` §2's region gate is
    /// `WorldData::get_tregion` of the figure's own tile, which answers a
    /// coastal cell's `region2` for an ocean tile, and this crate asked the
    /// plain `region_of` — so the barge filled its own cell and pushed its
    /// passenger four hundred units inland.
    ///
    /// Then it had to keep walking, and `Object::eject_contents@0064cd20`
    /// is `cast_transport` run backwards: for a passenger of `uber_size`
    /// 1 the boat's whole order list moves back onto it and the boat's
    /// path stack is inverted and popped onto its own, the top's embark
    /// flag cleared. run57 block 3979 has the scout's order and both
    /// waypoints field for field against the barge's in block 3978
    /// (`docs/TRANSPORT.md` §6.4).
    ///
    /// It was **3687** for one item, and the frame was the AI's citizen
    /// `1/11` — three draws against seven, ours opening with a
    /// `Guy::set_anim+0x97a < Unit::move_step+0x823` the original does not
    /// spend and then throwing seven gaia wing-beat coins where the
    /// original throws five. Both were one unit's, and it had left the
    /// original's point ninety-five frames earlier: `Unit::think_peasant`
    /// offers an AI **citizen** to the boat before it looks for work
    /// (`005f5760:53`, `think_civilian_transport(1)`), and this crate had
    /// only the scout's `colonise = 0` caller, so the citizen that the
    /// original sends north to colonise cell (50, 31) on frame 3581 went
    /// back to its gather instead (`docs/TRANSPORT.md` §7,
    /// `docs/SYNC.md` §3.23).
    ///
    /// It was **3608** for one item, and the frame was the transport
    /// barge's own guy. `SpellType::cast_transport` first runs there — the
    /// dock's own shadow, the level granted at 3579 being what lets a unit
    /// become its own transport at the shore — and this crate wrapped the
    /// barge's brand-new animation clock where the original wrapped no
    /// barge's, on that frame or any after. The reason is the loop, not the
    /// clock: `Objects::process_all@0065dce0` re-reads `unit_mark[who]` at
    /// the bottom of its inner loop, so the barge — cast by a unit whose
    /// `o` is lower — takes its first step on the frame it is born, and
    /// that step sets its guy to the walk before `Objects::inc_time` gets
    /// there. Every other newborn in run57 is a *trained* unit, born in the
    /// second loop, and every one of those does wrap on its birth frame
    /// (`docs/SYNC.md` §3.22, `docs/TRANSPORT.md` §13).
    ///
    /// It was **3579** for one item, and the frame was the dock's gull.
    /// A finished Dock spawns a **`GULLBIRD` of owner 9** a tile
    /// north-west of itself (`Dock::init@00740a80`: `Objects::init_unit(9,
    /// GULLBIRD, x − 0xc0, y − 0xc0)`) and rolls its facing as `(r % 7) ×
    /// 0xaaaaaaa − 0x40000000`; this crate spawned it and left the roll
    /// unmarked, so the sequence read a second `Guy::init_real+0x52`, and
    /// the gull carried no `type_index`, so it never flew: the count parted
    /// a frame later on `do_air_physics`'s own `set_anim(CHAR_WALK, 0, 1)`
    /// (`docs/TRANSPORT.md` §5.2, `docs/SYNC.md` §3.4).
    ///
    /// It was **3435** for one item, and the frame was three draws against
    /// two: a `Farms::inc_time+0x1ae` this crate spent and the original did
    /// not. That was a symptom of the farm below rather than a clock: with
    /// player 1's farms on the original's own cells the extra growth tick
    /// is gone.
    ///
    /// It was **3177** for the length of one item — the AI's farm `1/2011`
    /// going up one cell east of the original's, and `1/2012` one tile.
    /// `produce_building`'s FARM/MINE arm does not reuse the spiral's own
    /// distance: it rebuilds both sides in **tiles** and measures from the
    /// anchor's exact position, so a straight neighbour and a diagonal one
    /// score `4000/4` against `4000/6` where in cells both are `4000/1`.
    /// The same arm's `0xff` is `0xff − WData.val`, and the map's value
    /// byte is not zero (`docs/AI.md` §22).
    ///
    /// It was **3021** for two items — four draws against five, the
    /// missing one an idle request from `Unit::move_step` rather than from
    /// `Guy::inc_time` — and both of them were the AI's Dock. `o 2010`
    /// went up two cells north of the original's because
    /// `produce_building`'s `is(0x1b0)` slide was not modelled, and the
    /// slide's effect is not where the dock lands but *which* candidate
    /// the spiral accepts, and so the phase of its stride of three
    /// (`docs/AI.md` §21). With the dock right the seam moved to the
    /// builder's approach and `find_nearby_spot` let a citizen stand in
    /// the sea (`docs/ORDERS.md` §10); with both, every unit of run56
    /// stands where the original's does for the whole capture.
    ///
    /// It was **2665** twice over, and the second of those was the scout's
    /// **dog**. A unit is one or more figures, and a crew figure whose
    /// piece names a track offset does not stand on the guy it follows: it
    /// walks a body of its own toward a destination its leader rewrites,
    /// and it is still walking four frames after the man has arrived. Those
    /// are four frames on which the two figures answer `Guy::set_anim`'s
    /// walking-guy early return differently, and this crate — which handed
    /// every guy the unit's body — re-rolled the dog's idle where the
    /// original did not. With the second body landed the frame agrees, and
    /// so does every figure of every player's unit on all 3,000 frames of
    /// run56 ([`run56_s_figures_stand_where_the_original_s_do`]).
    ///
    /// Two readings carried it, and both are in `docs/MOVEMENT.md` under
    /// "The follower's destination": `Guy::set_new_location` and
    /// `Guy::set_angle` both rewrite the crew's `des` — the first from guy
    /// 0's new position when it moves, the second from the angle it has
    /// just turned to when it stands — and `GuyData::turn_speed@005de340`
    /// fences its whole first half behind `guy_num < squad_size`, so a
    /// tracked crew guy turns a flat quarter turn a frame and can never
    /// give a frame up to turning.
    ///
    /// The floor before that was **2176**, item 126's — the frame the
    /// original placed player 1's second Woodcutter's Camp and this
    /// simulation placed nothing, because `produce_building` scored a camp
    /// site by the forest tiles in a one-tile ring rather than by what the
    /// site would gather. `blocked_site`'s out-parameter is that number
    /// (`docs/CITIES.md` §2.6.7), and with it the camp goes up at the
    /// original's own frame, tile and object number.
    /// run58's own three numbers (`run58_s_five_thousand_frames_stand_
    /// where_the_original_s_do`). The two field counts are structural —
    /// what a 5,200-frame capture holds — and `RUN58_PARTED` is the score:
    /// how many of its units ever walk off the original's point **before
    /// the word**. It was 1 — `1/13`, from 3647 — until the tile grid's
    /// tolerance landed; it was **0**; it was **1** while the AI's first
    /// Fisherman walked a sea route this crate planned two cells north of
    /// the original's; and it is **0** again now that the world grid's
    /// same-region test reads `WData.region` rather than `get_tregion`
    /// (`docs/PATHFINDER.md` §16). Both Fishermen's routes now agree row
    /// for row on the frame each is planned, so the boats have no pin of
    /// their own any more — they are inside the ordinary assert.
    const RUN58_PARTED: usize = 0;
    /// 178,326 until item 261 widened the building row from the site
    /// alone to the site **and the construction clock** — `constr_time`
    /// always, `job_counter` while both sides still call the site
    /// unfinished.
    const RUN58_BUILD_FIELDS: usize = 270_173;
    const RUN58_COLL_FIELDS: usize = 449_279;
    /// Unit-frames carrying `unit_masks` and `mylos` — one apiece per
    /// linked unit-frame, which is every one, so the floor only grows.
    /// 94,935 and 94,338 as this was pinned; `mylos` is the smaller
    /// because a garrisoned unit's is not compared.
    const RUN58_PACKED_FRAMES: usize = 94_338;

    /// **The second map's word, and where it parts.**
    ///
    /// run33 gave Great Lakes a word — the first frame whose draw *count*
    /// is not the original's — and it is the sub-score a dozen items were
    /// steered by. East Indies had none: run39 was scored on ticks and
    /// orders alone, and `rontrace-run39.log` sat unread beside its dump.
    ///
    /// It parts at **19**, which is 148 frames before the order-list
    /// divergence at 168 that the queue had been calling this map's first.
    /// Everything run39 diverges on after 19 is on a stream that is
    /// nobody's — **its ticks and orders of 167 included** — so this is
    /// the number to move, and the score beside it is the early window
    /// rather than a total over the game (a total past the parting is
    /// noise: a more faithful simulation can score worse on a stream that
    /// is nobody's, and this one measurably does).
    ///
    /// What parts it is **the pasture's five animals**. East Indies' AI
    /// starts with an animal farm (`farm_type 1`, `o 2003`) and Great
    /// Lakes has none, which is why run33 never saw any of this. Three
    /// things are wrong with them here, and the trace names all three:
    ///
    /// 1. **They have no `type_index`**, so `Sim::slot_length` cannot
    ///    reach the install's gaia table and every one carries
    ///    `sim::anim::UNKNOWN`. A clock that never wraps costs no draw,
    ///    and that is **three** of the original's six `Guy::inc_time`
    ///    wraps on frame 29.
    /// 2. **They are the wrong species.** `Farms::add_animals@008d8f30`
    ///    throws `(rnd & 1) == 0 ? FARMCHICKEN : FARMPIG` per animal, and
    ///    a pasture is therefore always **one** species: the four draws
    ///    are a fixed stride, and `Random::get(0, 0xffff)`'s low bit is
    ///    the complement of the seed's, which the LCG flips every step —
    ///    five even coins or five odd ones, never a mix. run39's, read
    ///    back out of its own trace at `add_animals+0x92` with the seeds
    ///    the record carries, are even: **chickens**, whose
    ///    `CHAR_DEFAULT` is 30 frames where a pig's is 90.
    /// 3. **The walk is read and not issued.** The animal whose
    ///    `think_farm_animal` phase hits frame 0 — `o` 0 of 0–4, slot 0;
    ///    the trace's own phases `{0, 108, 116, 122, 126}` are
    ///    `(o·(slot+1)) % 128` for exactly that assignment — is handed a
    ///    `MOVE_TO` this crate does not add. It walks, and its arrival
    ///    spends the **two** `Animal::do_idle` set_anim draws of frames
    ///    19 and 20, after which its clock is nineteen frames behind the
    ///    other four and wraps at 49 rather than 29. The signature
    ///    repeats all game: every `think_farm_animal` draw is followed
    ///    nine to twenty-five frames later by a pair of `Animal::do_idle`
    ///    draws on consecutive frames.
    ///
    /// All three are landed (2026-08-30). The animals' **positions** were
    /// the wall — `add_animals` places each of the five at the farm ±
    /// `% 0x180 − 0xc0` on each axis, up to a whole tile, from two draws
    /// inside `Setup::build_empire`, whose stream the harness does not
    /// replay — and [`borrow_pasture`] takes them from the run's own
    /// trace, the way the heights and the herds are taken from a sibling
    /// dump. run39's five are `(−143, 40)`, `(−187, −148)`, `(−39, −144)`,
    /// `(−83, −76)`, `(−63, 56)` as `(dy, dx)`, and the test asserts them.
    ///
    /// **What is left at 19 is not the pasture's.** With the walk issued
    /// the arrival's *second* draw, frame 20, comes right and the first,
    /// frame 19, does not — and the two residues behind that are movement
    /// and animation, not this mechanic:
    ///
    /// - **The animal arrives one frame late.** Its 455-unit walk takes
    ///   nineteen 25-unit steps here and eighteen there; the arrival test
    ///   is `dist ≤ tolerance` and this crate's straight-line goal carries
    ///   `tolerance 0` (`path.rs:1036`). Give the chicken one more unit of
    ///   speed and frame 19 matches the original **draw for draw** — that
    ///   is how the two halves were told apart.
    /// - **An arrival costs two `Animal::do_idle` draws, not one.** The
    ///   pair is on consecutive frames, every time, all game. One is the
    ///   walk-to-idle transition this crate spends; the other needs the
    ///   guy to be playing something non-idle on the following frame,
    ///   which is what `Guy::move`'s turn arm would do — the unit is still
    ///   easing onto the order's angle on both frames (queue items 36, 37).
    ///
    /// History:
    ///   2026-08-30  word parts at **19**; of the first 64 frames 49
    ///               spend the original's number of draws and 47 draw for
    ///               draw (the first reading of this trace).
    ///   2026-08-30  the pasture's five landed whole — the species and the
    ///               `type_index`, the borrowed positions, and
    ///               `think_farm_animal`'s `MOVE_TO`. The window goes
    ///               49/47 → **62/55**; frames 20, 29 and 32 come right
    ///               and the word holds at 19 on the arrival frame alone.
    ///               The feared cost never arrived: ticks and orders stay
    ///               at 167 and player 0 at 219, because the walk is what
    ///               (1) and (2) were missing rather than a second
    ///               perturbation. A pasture animal that carries a
    ///               `MOVE_TO` it cannot step — `movement.speed` unset —
    ///               *does* cost player 0 two frames, which is what the
    ///               first attempt measured.
    ///   2026-08-30  the pair, landed behind item 36: word **19 → 69** and
    ///               the window **64 of 64 on the count and 64 draw for
    ///               draw**. `Unit::init`'s snap on the animal's birth
    ///               point and `Guy::move`'s turn arm — either alone is
    ///               worse than neither (20 and 60/53; 58/57), and both
    ///               together cost the *other* map 196 frames of word
    ///               until the farmers' angles were the original's
    ///               (`docs/SYNC.md` §3.11's last section, §3.12).
    ///   2026-08-30  word **69 -> 91** with item 95, **the animal's
    ///               hurry**: `AnimalData::get_speed@005d8380` walks an
    ///               animal at `speed * 3 / 2` while it is more than
    ///               `0x180` from its order's goal, and this crate walked
    ///               it at `speed`. Gaia's `8/2` therefore took ten steps
    ///               over ground the original crosses in nine, reached its
    ///               blocked stand a frame late, and spent
    ///               `sim::anim::SITE_BLOCKED` on 70 where the original
    ///               spends it on 69. The window holds at 64/64.
    ///               `docs/SYNC.md` §3.13, and
    ///               [`an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`]
    ///               is the rule against the record.
    ///   2026-08-30  word **91 -> 201**, **the blocked animal's dropped
    ///               walk**: `Unit::resolve_unit_collision`'s first
    ///               statement is `SubObjectData::is_animal`, and when it
    ///               answers the body is the `QUEUE_NEW` clear —
    ///               `docs/COLLISION.md` §6 step 0. Gaia's `8/2`, blocked
    ///               by its herd-mate on 69, stands there for the rest of
    ///               the capture; this crate sidestepped, snapped it onto
    ///               its cell centre and walked it round to the goal,
    ///               where it then blocked `8/0`'s wander spot on 89. The
    ///               window holds at 64/64. `docs/SYNC.md` §3.14, and
    ///               [`a_blocked_animal_drops_its_walk_where_it_stands`]
    ///               is the rule against the record.
    ///   2026-08-30  word **201 -> 219**, **the pasture's herder**:
    ///               `Unit::do_gather@005ef2a0:5efd77` takes the whole of
    ///               the function when the farm's `farm_type & 1` is set —
    ///               a herder shows the sow animation and draws only on
    ///               the frames where `(o · 7 + frame + who) % 256` is
    ///               zero, then walks to one of the farm's inner four
    ///               tiles. `docs/ORDERS.md` §6.5 had the arm from its
    ///               first writing and `do_farm` never did, so the AI's
    ///               `1/3` ran the crop switch instead; a pasture is the
    ///               one farm `Farms::inc_time` skips, so its cell reached
    ///               `RIPE_ADDS` on the herder's own adds alone — the two
    ///               hundredth frame rather than the hundredth — and it
    ///               re-picked a tile on 201 where the original spends its
    ///               pair on **234**, its first phase frame. **Every one
    ///               of the 219 frames is now draw for draw**, not only
    ///               the first 64 (274 as of the next entry). `docs/SYNC.md` §3.15, and
    ///               [`a_pasture_herder_walks_only_on_its_own_256_frame_phase`]
    ///               is the rule against the record.
    ///   2026-08-30  word **219 -> 274**, **the frame's two loops**:
    ///               `Objects::process_all@0065dce0` is the units,
    ///               rotated by owner, and then a *second, unrotated* pass
    ///               over each player's buildings and then their walls.
    ///               `docs/SYNC.md` §3.2 had said so since it was written
    ///               and `Sim::tick` ran the buildings first, so frame
    ///               219's citizen re-target fell behind the frame's road
    ///               search instead of in front of it — and the search
    ///               that looked 152 nodes against 129 was the same
    ///               search on a different world. With the order right it
    ///               is 129 against 129 and the frame is draw for draw.
    ///               `docs/SYNC.md` §3.16. Two rules moved with it: a
    ///               unit created this frame is *not* skipped by
    ///               `Objects::inc_time`, and `think_peasant`'s idle
    ///               threshold is **1** for an AI-driven worker.
    ///   2026-08-30  **413** (item 98, `docs/AI.md` §17): a script's
    ///               `static` is one variable on `Script::static_vars`,
    ///               not a frame slot, and this crate's mirror wiped
    ///               every one of them on the second call. So
    ///               `economic.bhs`'s `needed_citizens` was zero from
    ///               the second call on, its every-call
    ///               `train_unit_with_need` trained nobody, and the
    ///               citizen the original's city hall finishes at 274 was
    ///               never queued. With the statics live the queue clock
    ///               runs 100 to 9,750 on the original's own frames and
    ///               the guy is born on 274.
    ///   2026-08-31  word **413 -> 576** (item 99), **the building's own
    ///               line of sight**: `Build::activate@00623e20`'s last
    ///               statement is `update_seen(0)` — the whole fog disc —
    ///               and nothing here threw it, so the fog grew only where
    ///               units walked. The AI's sixth farm finishes on frame
    ///               219 at cell `(54, 51)` with `mylos 8`, which lights
    ///               the three cells of column 56 east of it; nineteen
    ///               frames later `Unit::think_scout` re-targets and the
    ///               `EXPLORE_TO` path runs *through* those cells here — a
    ///               scout prices unseen ground at a base of 8 against a
    ///               seen cell's `0x400` — where the original, which can
    ///               see them, walks the seen column 55 instead. Its scout
    ///               therefore arrived on 412 and idled on 413 while this
    ///               one was still twelve frames short. `docs/VISION.md`
    ///               §2.1.
    ///   2026-08-31  word **576 -> 645** (item 100), **the bird's landing
    ///               search**: `Animal::think_bird@005d79e0`'s tail is
    ///               thirty rounds over the cell list of the region the
    ///               patrol point sits in, two draws a round, and
    ///               `docs/SYNC.md` §3.9 had recorded it as unreachable
    ///               because no capture had reached it. run39's gaia bird
    ///               `9/8` reaches it on frame **576** — its landing roll
    ///               is the `% spell_time` at `+0x1f8`, and the counter
    ///               only passes 100 after ~90 frames of flight — so the
    ///               original spends **sixty** draws there and this crate
    ///               spent none. The frame's 118 against 56 was that,
    ///               plus the two the frame's own animal birth then falls
    ///               out of step over.
    ///
    ///               **What the frame was not is its AI**, which is what
    ///               the item was booked as. Frame 576 is
    ///               `place_city_with_cost` five times over, and this
    ///               crate spends every one of its twenty draws already:
    ///               `make_stuff`'s expiry walk simply had no
    ///               [`sim::Sim::mark`] on it, so each of its draws read
    ///               as the last site marked — `compute_sites+0x50a` —
    ///               and the sequence appeared to part on a draw that was
    ///               in fact correct. An unnamed draw is a lie in this
    ///               comparison, not a gap.
    ///   2026-08-31  word **742 -> 867** (item 102), **the far wander's
    ///               literal bearing**: `Animal::do_idle@005d7460` hands
    ///               `UnitType::find_nearby_spot` the constant
    ///               `0x55555555` as its sweep's starting angle — the same
    ///               120° `Unit::init` writes into a unit that has never
    ///               turned — where every other call site in the
    ///               executable passes a real bearing, and this crate
    ///               passed the animal's facing. Gaia's `8/3` was 180° out,
    ///               so on frame 736 it walked due north from the herd
    ///               centre where the original walks east-north-east; six
    ///               frames later the original's step is refused by its
    ///               herd-mate `8/2` and spends the blocked stand
    ///               (`Guy::set_anim+0x97a < Unit::move_step+0x823`,
    ///               `docs/COLLISION.md` §5), while ours walked on into
    ///               open ground. **The item was booked as a blocked
    ///               stand and the blocked stand was already right**: with
    ///               the bearing corrected the whole walk is the
    ///               original's step for step — `(28741, 24384)`,
    ///               `(28754, 24360)` … `(28793, 24288)` — and the
    ///               refusal, the dropped walk and the stand all fall
    ///               where they fall in the dump, with no change to the
    ///               collision model at all. Frame 867 is next, and it is
    ///               `Unit::explore_goody+0x27c < Unit::set_new_location
    ///               +0x3cc < Unit::move_step+0x8f4`, three draws.
    ///               `docs/SYNC.md` §3.19, and
    ///               [`a_far_wander_sweeps_from_the_literal_bearing`] is
    ///               the rule against the record.
    ///   2026-08-31  word **867 -> 879** (item 104), **the goody box**:
    ///               `Unit::set_new_location@005f8d20+0x3cc` calls
    ///               `Unit::explore_goody@005f9780` on any non-animal land
    ///               unit that enters a new cell carrying `WData.flags &
    ///               0x8000`, and run39's world has seven of them —
    ///               the `WORLD` record's own `goodies 7`. Player 1's
    ///               scout `1/0` walks into `(45, 49)` on 867, and the box
    ///               holds a lottery: one draw for each good the finder can
    ///               gather, scored `draw % 25 + bucket[good]`, lowest
    ///               wins, knowledge never a candidate. In the Ancient age
    ///               that is **three** — food, timber and wealth — and this
    ///               crate spent none, so the frame read 5 against 9 and
    ///               the sixth `Farms::inc_time` draw (`+0x1de`, the
    ///               sprout) fell out with them. Frame 879 is next and it
    ///               is the scout: two `Unit::set_anim` stands and then
    ///               `Unit::think_scout+0x436`/`+0x458` six times over with
    ///               `+0x64c` twice — sixteen draws this crate does not
    ///               spend. `docs/GOODY.md`, and
    ///               [`a_goody_box_draws_once_for_each_good_its_finder_can_gather`]
    ///               is the rule against the record.
    ///   2026-08-31  word **879 -> 1256** (item 106), **the walk to the
    ///               box**: frame 879 was the scout going idle and
    ///               re-thinking, and the reason it was idle twelve frames
    ///               after the box is that its explore order had been
    ///               *re-aimed at the box* — `Unit::do_explore_to@005f24a0`
    ///               calls `Unit::find_goody_box@005f2540` one frame in
    ///               fifteen, and on frame 825 the sweep found `(45, 49)`
    ///               and re-issued the walk to that cell's centre. The
    ///               gate is not the cell's `was_seen`, which the box's
    ///               own borders answer yes from frame 0, but the **item's**
    ///               `ItemData::is_seen` — the bare accumulated fog, which
    ///               only reaches the box between 811 and 825. So the
    ///               retarget lands on 825, the arrival on 879, and
    ///               `think_scout` runs there with the original's own six
    ///               ring pairs. Three hundred and seventy-seven frames,
    ///               four `think_scout` frames (879, 1021, 1143, 1231) and
    ///               the goody's own second box at 1659 all pass; 1256 is
    ///               next and it is a **bird**: ours lands (87 draws, the
    ///               third bird's `Animal::think_bird+0x2aa`) where the
    ///               original's eight birds only think (27).
    ///               `docs/GOODY.md` §7, and
    ///               [`a_scout_re_aims_its_walk_at_a_goody_box_it_has_seen`]
    ///               is the rule against the record.
    ///   2026-08-31  word **1256 -> 1373** (item 109), **the bird's
    ///               landing step**: frame 1256 was a bird landing that
    ///               the original's does not, and the counter is the only
    ///               state behind it. `Unit::do_air_patrol@005ea620`'s
    ///               tail, after `do_air_physics` returns, branches on
    ///               `vtable+0x30` — `SubObjectData::is_animal`, folded
    ///               onto `Buffer::is_pending_load`'s `return 1` on all
    ///               three `Animal` vtables — and the animal arm is
    ///               `else if (spell_time == 0)
    ///               spell_time = 1`. `think_bird` steps the counter on
    ///               every frame it runs, so it is 0 there only on a
    ///               frame the landing search has just zeroed: one extra
    ///               step per landing, and nothing else. run39's second
    ///               bird had landed on 944, so its counter reached 351
    ///               rather than 352 and `27127 % 351` is exactly the
    ///               `== 100` the branch tests for. With the step the
    ///               original's own landings at **1368** and 1376 fall
    ///               where they fall; 1373 is next and it is the AI
    ///               scout, which takes an eighth `+0x436`/`+0x458` ring
    ///               pair where the original stops at seven and goes to
    ///               `Unit::think_scout+0x941` and five `+0xaba`.
    ///               `docs/SYNC.md` §3.9, and
    ///               [`a_bird_s_landing_frames_are_the_trace_s_own`] is
    ///               the rule against the record.
    ///   2026-08-31  word **1373 -> 1570** (item 110), **the region
    ///               fallback's cell walk**. The eighth ring pair the row
    ///               above blames was never a ring: `scout_region_scan`
    ///               took §11's stride draw with no `mark` of its own, so
    ///               the stream inherited `SITE_PHASE` and the sequence
    ///               *read* as a seventh ring. The original walks its six
    ///               rings, finds nothing, leaves the city loop on
    ///               `best > 199` and scans the whole region — one
    ///               `+0x941` and one `+0xaba` per accepted cell, five of
    ///               them here — and that walk had been called
    ///               unreproducible because `Region.coords`' order is
    ///               nobody's dump. It is the **cell grid's own**:
    ///               `Regions::rebuild_coords@0067f800`, the last writer
    ///               `Regions::find_all` reaches, refills every list by a
    ///               row-major sweep. `docs/SCOUT.md` §11, and
    ///               [`a_scout_with_no_city_near_scans_its_whole_region`]
    ///               is the rule against the record.
    ///   2026-08-31  word **1570 -> 1647** (item 121), **two frames of the
    ///               wood dump**, one in the art loader and one in the
    ///               order. `AnimMgr::force_load@0053ade0` drops a
    ///               **non-looping** animation's last key before the
    ///               conversion — `times = key_times[n-2]` when
    ///               `loopings[i] == 0` — so `lumberjack_dump.bha`'s 2157,
    ///               2190 is 32 frames and not 33, which is what every
    ///               `GUY` block in the corpus prints for `cur_anim 27`
    ///               and what no dump this suite read had ever been asked
    ///               ([`the_install_s_piece_lengths_match_the_dumps`],
    ///               five dumps too short). And the frame that gave back:
    ///               `Unit::set_angle(a, a, **0**)` is what every order in
    ///               the ordinary path calls, so the guy takes `des_angle`
    ///               and its own `angle` stands — `Guy::move` finds a body
    ///               at its destination owed a turn and spends the **turn
    ///               arm**, `set_anim(CHAR_WALK, 0, 1)`, over whatever the
    ///               order just asked for. run44's citizen `0/2` is the
    ///               record: at the camp on 542 its `wait` steps 32 → 31
    ///               and its guy plays the carrying walk restarted; the
    ///               dump is 543. `docs/ANIM.md` §3.1 and §4.7. Next is
    ///               frame 1647, and it is not this family: nineteen draws
    ///               against four, opening `Guy::set_anim+0x97a <
    ///               Unit::do_idle+0x7d`.

    #[test]
    fn run39_s_long_trace_says_where_the_second_map_s_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").map(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .expect("invalid finalized trace")
                    .expect("missing RONT header")
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        assert_eq!(
            init.pasture.len(),
            1,
            "run39's trace reached the setup and East Indies' AI has one pasture"
        );
        assert!(
            init.pasture[0].iter().all(|a| a.chicken),
            "a pasture is one species, and run39's five coins are even"
        );
        assert_eq!(
            init.pasture[0]
                .iter()
                .map(|a| (a.dy, a.dx))
                .collect::<Vec<_>>(),
            vec![(-143, 40), (-187, -148), (-39, -144), (-83, -76), (-63, 56)],
            "the five offsets `report.py <log> draws setup` prints"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 1_800,
            "run39's traced length is {last}, wanted 1,800+"
        );
        for _ in 0..last {
            built.tick();
        }
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != tr.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        // Past the parting the totals over the whole game are noise — a
        // more faithful simulation can score worse on a stream that is
        // nobody's — so the number pinned beside the word is the **early
        // window**: of the first 64 frames, how many spend the original's
        // number of draws, and how many draw for draw.
        const WINDOW: i64 = 64;
        // And the frame the *sequence* parts on, which since the herder
        // (§3.15) is the same frame: every draw of every frame before it
        // is the original's, in its order, not merely its count.
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| **ours != tr.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *f < WINDOW && ours.len() == tr.labels(*f).len())
            .count();
        let matched = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *f < WINDOW && **ours == tr.labels(*f))
            .count();
        eprintln!(
            "run39: word parts at {first_count}, sequence at {first_part}; of the first \
             {WINDOW} frames {words} spend the original's number of draws and {matched} \
             draw for draw"
        );
        // The parting frame's own row, and the first few inside the window
        // — the row is the successor item every time the number moves, so
        // the run prints it rather than leaving it to be re-derived.
        let row = |f: i64, ours: &Vec<String>| {
            let theirs = tr.labels(f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            format!(
                "frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            )
        };
        let mut shown = 0;
        for (f, ours) in built.frame_sites.iter().take(WINDOW as usize) {
            if *ours == tr.labels(*f) {
                continue;
            }
            eprintln!("{}", row(*f, ours));
            shown += 1;
            if shown == 4 {
                break;
            }
        }
        for (f, ours) in built.frame_sites.iter() {
            if *f == first_part {
                eprintln!("parts: {}", row(*f, ours));
            }
            if *f == first_count && first_count != first_part {
                eprintln!("counts: {}", row(*f, ours));
            }
        }
        assert!(
            first_count >= FLOORS[0].word
                && first_part >= FLOORS[0].word
                && words >= 64
                && matched >= 64,
            "the second map's word fell: parts at {first_count}, its sequence at \
             {first_part}, {words} of the first {WINDOW} frames on the count, \
             {matched} draw for draw — the floor is {}, {}, 64 and 64",
            FLOORS[0].word,
            FLOORS[0].word
        );
    }

    #[test]
    fn run39_s_islands_game_is_the_second_map_s_score() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        // **The pasture is a source, and without it this is a different
        // game.** East Indies' AI builds one, its five animals are owner 9
        // and no dump prints them, and they roll an idle every frame — so
        // a score run that borrows none parts from the original within a
        // few frames of the pasture going up, and every figure below is
        // then on a stream that is nobody's. That is what pinned this map
        // at 167 (item 69). The assertion is the guard: a run that loses
        // the pasture fails here rather than quietly scoring low.
        assert_eq!(
            tr.add_animals().len(),
            1,
            "run39's trace reached the setup and East Indies' AI has one pasture"
        );
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        // 1,851: the `1850 !quit` runs at the top of frame 1850, and the
        // block the quit interrupts is written too.
        assert_eq!(report.frames.len(), 1851, "run39's length");
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: frame 0: ours 175 draws, the original's 175")),
            "frame 0 on the second map: {:?}",
            report.notes
        );
        //   2026-08-30  ticks **505**, orders **482**; player 0 @ **687**,
        //               player 1 @ **506** (item 78: **the gather order's
        //               write-back went to the front of the list**).
        //               `Sim::store_gather` wrote to `orders.front_mut()`,
        //               and every walk `do_gather` and
        //               `do_non_flat_gather` issue is a `QUEUE_FIRST` move
        //               *in front of* the gather order — so the
        //               `goto_build = 1` / `wait = 32` that the
        //               return-to-camp branch writes **after**
        //               `add_move_order` landed on the move and were
        //               dropped. The original holds the order pointer for
        //               the whole function; §6 of `docs/ORDERS.md` had
        //               already established the same rule for the *read*
        //               side (`is_gathering_at` matches `get_action`, not
        //               the front), and the write side was never made to
        //               match.
        //
        //               The human's `0/2` finished its shift on 426,
        //               reached its camp on 431 and then re-entered the
        //               same branch for the rest of the capture: it never
        //               unloaded, never took another tile, and spent a
        //               `+0xb99` stand every other frame. run33's word
        //               goes 432 → **482** and its totals 724/606 →
        //               **752/622**.
        //
        //               Every one of the twelve compared units improves.
        //               Player 0 recovers item 76's fall and passes it,
        //               450 → 687 (`0/5` 450 → 687, `0/3` 455 → 702,
        //               `0/4` 577 → 703); player 1 goes 437 → 506, and
        //               what pins it is `1/8` at 506. East Indies is
        //               unmoved at 167/167 — its own divergence is an
        //               order-list length at 168, item 69.
        //   2026-08-30  ticks **571**, orders **571**; player 0 @ 574,
        //               player 1 @ 572 (item 79: **the vision projection
        //               was thrown along the guy's angle**).
        //               `Object::update_seen` throws a small land unit's
        //               disc half a cell forward of its nose, and the
        //               angle it projects along is `UnitData +0x50` — the
        //               unit's own heading, what the dump prints as
        //               `UNITDATA angle` — not `GuyData::angle`, the eased
        //               facing the body actually wears. The `project` at
        //               `00651d05` is handed it by `movl 0x50(%ecx), %ecx`
        //               two instructions earlier, with `ecx` the
        //               `units.list[who][o]` the branch above it read.
        //
        //               While a unit turns the two differ by as much as
        //               thirty degrees and the disc lands in a different
        //               fog cell. On run33's frame 168 the AI scout was
        //               mid-turn — heading −51.6°, facing −83.0° — and the
        //               disc thrown along the facing lit fog `(113, 57)`,
        //               which the original's never reached. Three hundred
        //               frames later, on **482**, `Unit::think_scout`'s
        //               cell filter refused cell `(56, 28)` as "already
        //               seen" and spent thirty draws where the original
        //               spends thirty-one. run33's word goes 482 → **571**
        //               and its totals 752/622 → **791/662**.
        //
        //               **Every player-1 unit improves or holds** — `1/8`
        //               506 → 778, `1/4` and `1/5` 550 → 673 and 663,
        //               `1/0` 637 → 722 — which is the AI walking the
        //               original's fog as well as its ground. **Player 0's
        //               three farmers fall**, 687/702/703 → 579/574/577,
        //               and it is the same downstream effect item 76 had:
        //               their old numbers were all past the word's own
        //               parting at 482, on a stream that was nobody's, and
        //               the new ones sit three frames past the new parting
        //               at 571. The whole capture now diverges within
        //               eight frames of the word, which is what
        //               convergence looks like. East Indies is again
        //               unmoved at 167/167.
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        eprintln!(
            "run39: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
        //   2026-08-30  ticks and orders hold at **167**; player 1 holds
        //               at 168 and **player 0 goes 219 -> 217** with item
        //               95, the animal's hurry. It is the one fall the
        //               item costs, and it is downstream of the word: the
        //               word on this map now parts at 91, so by 217 the
        //               two sides have been on different mid-frame draw
        //               orders for a hundred and twenty frames. What moved
        //               is which of player 0's citizens parts first —
        //               `0/3` and `0/5` at 219 before, `0/4` at 217 after,
        //               each of them a step out on a walk no animal
        //               touches. The animal's own step is now the
        //               original's on **264 of 264** dumped gaia steps
        //               across both maps
        //               ([`an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`]),
        //               which is a stronger statement about this
        //               simulation than two frames of a citizen.
        //   2026-08-31  ticks **1374**, orders **536**; player 0 @ **1411**,
        //               player 1 @ **1375** (item 69: **the score run was
        //               not the run the word measures**). Every figure
        //               above was taken on a simulation with no pasture in
        //               it: `run_traced` borrowed the siblings' initial
        //               state and the trace was handed only to the word's
        //               own check, so the five owner-9 animals no dump
        //               prints were missing from the one run that scored.
        //               They roll an idle a frame, so the stream parted
        //               within a few frames of the pasture going up and
        //               everything downstream — `1/4`'s farm re-target at
        //               167, `1/5`'s at 186, player 0's citizens at 217 —
        //               was a different game's arithmetic, not a mechanic.
        //               With the trace passed in, ticks land **one frame
        //               past the word's own parting at 1373**, which is
        //               what a converged capture looks like: the
        //               simulation holds position for as long as it holds
        //               the stream, and parts when the stream does.
        //
        //               Orders followed to **1373** on the same item's
        //               second half — the collision pause's own frame
        //               (`docs/ORDERS.md` §4.4, `goto STEP`) — so both
        //               numbers now sit **on the word's parting**: this
        //               capture holds position and intent for exactly as
        //               long as it holds the stream. The next figure to
        //               move is the word's, and 1373 is item 110's.
        //
        //               The comparison behind `orders` is also wider than
        //               it was: the same item added `OrderMismatch::Move`,
        //               the whole `MOVEORDER` row, of which `dest`,
        //               `facing` and `last_x/last_y` are reported and do
        //               not score (see [`OrderMismatch::scores`]).
        //   2026-08-31  ticks **1374 -> 1477**, orders **1373 -> 1476**;
        //               player 0 @ **1657**, player 1 @ **1478** (item
        //               110, the region fallback's cell walk). Both
        //               numbers stay pinned to the word, which parts at
        //               1570 — a hundred frames of the original's stream
        //               that nothing here reads, spent on units this
        //               capture does not dump every frame — and both moved
        //               by the same 103 the word moved by less its own
        //               lead. Nothing fell.
        //   2026-09-01  ticks **1477 -> 1851**, orders **1476 -> 1850**,
        //               and **neither player diverges at all** (item 125:
        //               `WData.blocked` is a count of the cell's blocked
        //               tiles and nothing here was keeping it). The whole
        //               of run39 — every frame of the capture, every unit,
        //               position and order list — is the original's. It is
        //               the first capture on either map to be matched end
        //               to end, and with Great Lakes already exact over
        //               its own 1,772 it closes both.
        //
        //               What it was: player 1's second city went up on
        //               tile (180, 188), and the four cells under it kept
        //               a `blocked` of zero, so `calc_cost` charged an
        //               empty field where the original charges nine
        //               sixteenths of one — 304 against 128 — and §5.1's
        //               corner-cutting never even opened. The AI scout
        //               therefore walked *through* the city on frame 1476
        //               and arrived six frames early. run55's per-step
        //               cost dump is what said so, in one reading: 103 of
        //               110 steps already agreed, and all seven that did
        //               not were steps into those four cells
        //               ([`run55_s_frame_1477_prices_are_the_originals`]).
        let none = report.first_divergence.iter().all(|&(_, f)| f.is_none());
        assert!(
            ticks >= FLOORS[0].ticks && orders >= FLOORS[0].orders && none,
            "the second map's score fell: ticks {ticks}, orders {orders}, first \
             divergence {:?} — the floor is ticks {}, orders {}, and **no player \
             diverging anywhere in the capture**",
            report.first_divergence,
            FLOORS[0].ticks,
            FLOORS[0].orders
        );
    }

    /// The oracle, as a regression guard.
    ///
    /// Every other test in this crate checks the harness against something we
    /// wrote. This one checks it against **the original's own 432 frames**,
    /// and pins the state of the port on 2026-08-21 so that a change which
    /// quietly un-does it fails here rather than in six weeks' reading of a
    /// number nobody remembers.
    ///
    /// The assertions are chosen to be the *invariants*, not the readings: a
    /// field the simulation models must never disagree where it is
    /// comparable, and the one unit the harness can drive end to end must
    /// keep doing so. Counts that are expected to move as mechanics land —
    /// `length`, `kind`, the path-stack pair — are bounded rather than fixed,
    /// so landing the pathfinder does not fail this test, it improves it.
    #[test]
    fn the_original_s_own_run_is_still_matched_frame_for_frame() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run6-ancient-nubian-builds7.txt") else {
            // Not a silent skip: say which file is missing.
            eprintln!(
                "skipping: no gamelog-run6-ancient-nubian-builds7.txt \
                 (set RON_GAMELOG_DIR; docs/ORACLE.md says how to capture one)"
            );
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);

        // The lobby, read from `GAMEINFO`: `docs/INPUT.md` §2's settings,
        // and MAP_STYLE 14 named through rules.xml's `mapstyles` order.
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        assert_eq!(built.sim.lobby.difficulty, 0, "Easiest");
        assert_eq!(built.sim.lobby.starting_town, 2, "Small Town");
        assert_eq!(built.sim.lobby.starting_resources, 1);
        assert_eq!(built.sim.lobby.map_style, 14);
        assert_eq!(built.sim.lobby.map_style_name, "Great Lakes");
        assert_eq!(built.sim.lobby.rush_rules, 0);
        assert_eq!(built.sim.lobby.victory, 0);
        assert!(!built.sim.lobby.no_nation_powers);

        // Run6 is the run7/run9/run10 lobby and seed: the siblings' setup
        // trace and end-of-frame words put the AI's frame-1 coin on the
        // original's stream (`docs/SYNC.md` §5); without them the branch is
        // the sim's own stream's luck.
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, None).unwrap();

        assert_eq!(report.frames.len(), 432, "the dump's frame count");
        assert!(report.orders_seen(), "run6 is a UNITS=3 dump");
        // Every unit the game *started* with links. Later frames unlink a
        // growing number, and that is not a fault: over 432 frames both
        // players train units, and the harness stands its roster up from the
        // initial dump and has no production. It is worth pinning as a
        // ceiling, because the day production is wired in it should fall.
        assert_eq!(
            report.frames[0].unlinked, 0,
            "a unit in the first logged frame has no simulation unit"
        );
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        // 2026-08-24: zero. The AI trains the three citizens the original
        // does, on the same frames, and `find_free` numbers them so they
        // link. This is now exact, not a ceiling.
        assert_eq!(
            unlinked, 0,
            "unlinked unit-frames: {unlinked} — a unit the original has that \
             the simulation never trained"
        );

        // The start-of-game rule, derived without reading the log.
        let checks = check_start_orders(
            &build_sim(&loaded, &log.initial().unwrap(), Tuning::RON),
            &log,
        );
        let held: Vec<_> = checks
            .iter()
            .filter(|c| c.ours.is_some() || c.theirs.is_some())
            .collect();
        assert_eq!(held.len(), 10, "ten starting citizens");
        assert!(held.iter().all(|c| c.agrees()), "{held:?}");

        // **The invariant**: every field the simulation actually models
        // agrees wherever both sides name it. A regression in the order
        // system shows up here first, and in nothing else.
        //
        // With one named exception, 2026-08-24: the farmers' re-target
        // (`docs/ORDERS.md` §6.5) picks its tile with two sync-stream
        // draws, and the stream is the original's only for the four frames
        // run12 traced — so the *second* re-target, some hundred frames
        // after the first one's walk, lands a frame or two apart on the two
        // sides (the first, on the log's frame 102, agrees). Those are the
        // three farmers per player, `o` 3–5, after frame 200, and they go
        // away with a longer trace (`docs/SYNC.md` §6). The second cycle
        // begins once the re-sown cell ripens, a hundred-odd frames after
        // the walk — anything past 150 is it; with the unit loop rotated
        // (`docs/SYNC.md` §3.2) the sim's own-stream luck once put a
        // farmer's flags mismatch on frame 200 exactly.
        let modelled: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| {
                matches!(
                    d.what,
                    OrderMismatch::Action { .. } | OrderMismatch::Target { .. }
                ) || matches!(d.what, OrderMismatch::Flags { .. })
            })
            .collect();
        // **The carve-out is gone, 2026-08-29.** It used to except a farmer
        // (`o` 3–5, or any citizen trained during the run) after frame 150,
        // for the second re-target's two draws off a stream that had
        // drifted. Item 70 — `find_gather_spot` scored on the headroom
        // under the commerce cap rather than on distance alone — put every
        // citizen of this capture on the building the original puts it on,
        // and with that the exception had nothing left in it.
        //
        // **Scoped to each unit's own first divergence, 2026-08-29
        // (item 71).** Run6 is not traced past its start, so the stream is
        // the simulation's own from frame 4 on, and a farmer's *second*
        // re-target is two draws off it. With the animation lengths read
        // from the install the clocks moved, and `1/5`'s second re-target
        // now lands one frame apart: it parts in **position** on frame
        // 421, and its order list follows. Rows a unit produces after its
        // own position has parted are not evidence about the order system
        // — the same reasoning the collision block above is scoped by — so
        // what is asserted is emptiness up to each unit's own parting, and
        // any single row before it fails.
        let parted_orders: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let modelled: Vec<&OrderDivergence> = modelled
            .into_iter()
            .filter(|d| {
                parted_orders
                    .get(&(d.who, d.o))
                    .is_none_or(|&f| d.frame < f)
            })
            .collect();
        assert!(
            modelled.is_empty(),
            "a modelled order field disagrees: {:?}",
            &modelled[..modelled.len().min(4)]
        );

        // **The unit that tracks**: player 0's first woodcutter citizen
        // matched the original's position and its whole order list for the
        // entire run once `gather_from` arrived. If that stops being true,
        // something took the harness backwards.
        let by_unit = report.first_divergence_by_unit();
        assert!(
            !by_unit.iter().any(|&(w, o, _)| w == 0 && o == 1),
            "0/1 diverged in position: {by_unit:?}"
        );
        assert!(
            !report
                .order_divergence_by_unit()
                .iter()
                .any(|&(w, o, _, _)| w == 0 && o == 1),
            "0/1 diverged in its order list"
        );

        // **The pathfinder's pin** (2026-08-23): the second woodcutter's
        // citizen, whose stack was the stub's visible gap, now agrees with
        // the original's path stack for 427 straight frames — its first
        // disagreement of any kind is an order-list Length at frame 428,
        // and only after that fork do its stacks differ. The mismatches
        // that remain are player 1's mirror units, whose straight lines
        // cross forest the harness's flat world does not carry
        // (`docs/PATHFINDER.md` §10) — a world-data gap, not a search gap.
        assert!(
            !report
                .frames
                .iter()
                .filter(|f| f.frame < 428)
                .flat_map(|f| f.order_diverged.iter())
                .any(|d| d.who == 0 && d.o == 2),
            "0/2 disagreed before frame 428"
        );

        // The rest is expected to shrink, never grow. These are ceilings.
        // Re-based 2026-08-24 from 1,279/783: the AI's three trained
        // citizens now link and are compared — unit-frames the harness had
        // never seen before — and they idle where
        // the original sends them to gather, which is the census's job
        // (`docs/AI.md` §12.1).
        let farmer = |d: &&OrderDivergence| (3..=5).contains(&d.o);
        let orders: usize = report.frames.iter().map(|f| f.order_only().count()).sum();
        let paths: usize = report.frames.iter().map(|f| f.path_only().count()).sum();
        let farmer_orders: usize = report
            .frames
            .iter()
            .map(|f| f.order_only().filter(farmer).count())
            .sum();
        let farmer_paths: usize = report
            .frames
            .iter()
            .map(|f| f.path_only().filter(farmer).count())
            .sum();
        // Re-based again 2026-08-24, split: the farmers now re-target
        // (`docs/ORDERS.md` §6.5), and on the sim's own stream past run12's
        // four traced frames their tiles, walks and second re-targets are
        // their own — 527 order and 357 path disagreements that a longer
        // trace removes (`docs/SYNC.md` §6). Everyone else's fell, from
        // 1,784/1,123 to 1,238/875.
        // 1,238/875 → 1,220/866 with run13's words installed at frame 94
        // (the siblings' traced frames are pooled, `borrow_from_siblings`).
        // 1,220/866 → 1,242/877 with the animation clock (`docs/ANIM.md`):
        // the idle wraps and the training rolls are draws the original
        // makes too, but on the sim's own stream between the traced frames
        // they move the woodcutters' later waits (`% 50 + 100`) to other
        // values — noise in the untraced stretch, not a mechanic lost; the
        // traced frames 0–3 and 94–103 all held or improved.
        // 1,242/877 → 1,246/879 with the camp-arrival stand removed
        // (`orders.rs`, the stand/wrap swap): the woodcutter now unloads on
        // its first frame at the camp where it used to stand idle, so its
        // clock and its `% 50 + 100` wait land elsewhere in the untraced
        // stretch. Four order-frames and two path-frames of that noise
        // against three maps' frame 0 becoming exact — run20 and the fuzzed
        // map draw for draw, run10's 128 against 120 now 120 against 120.
        // 1,246/879 → 1,267/892 with `produce_building`'s three placement
        // defects fixed (`docs/AI.md` §2.20, 2026-08-26). **The totals
        // fell**: 1,679/1,199 to 1,552/1,160. What rose is only this
        // split's non-farmer half, because the AI now puts its buildings
        // somewhere else on this map too and the farmers' share of the
        // disagreements fell further than the rest (433/320 farmer-frames
        // to 285/268). Every traced capture held or improved — run20's
        // frame 1 lost its `produce_building` residue entirely, the fuzzed
        // map went 48/45 to 43/45, the Great Lakes did not move.
        // 1,267/892 → **1,181/1,602** on 2026-08-27, when a trained unit
        // started carrying its type (`docs/VISION.md` §8). The orders half
        // fell; the paths half rose by 710, and the reason is that the AI's
        // trained citizens used to **idle** — a unit with no order has one
        // `Length` disagreement a frame and no path stack at all, and a
        // unit that gathers has a stack that differs in detail on every one
        // of the four hundred frames it lives. So the ceiling rose because
        // the simulation started doing the thing, not because it stopped.
        // What settles that this is not a regression is that **every traced
        // check held**: run20 frame 0 175/175, frame 1 53/53, frame 2 5/5
        // and its world chain entry for entry; the fuzzed map 195/195,
        // 43/45 and 1,377/1,221 unmoved to the number; run10's window
        // 98–103 unmoved but for its own row. And the measurement that
        // motivated it: `mylos` over run10's 26,433 unit-frames went from
        // 5,170 disagreements to **1**.
        // 1,181/1,602 → **1,212/1,594** on 2026-08-27, item 34, and the
        // totals fell hard: 1,793/2,043 to **1,516/1,911**. The body stopped
        // chasing the unit at eleven eighths and started being written onto
        // it, and every unit that turns instantly from a standstill started
        // doing so (`docs/MOVEMENT.md`). What moved is the *split*: the
        // farmers' share fell from 612/441 to 304/317 — they now walk the
        // original's frames, so their later re-targets land elsewhere in
        // the untraced stretch — and 31 order-frames crossed out of it.
        // Every traced check held or improved: run13's window frame 102
        // went 22/6 to **6/6** (the row item 34 was booked on), run20's
        // frame 0 175/175 and frame 2 5/5 unmoved, and run10's scout is on
        // the original's position and both of its angles for the whole of
        // frames 57 to 91.
        // 1,212/1,594 → **1,957/1,254** on 2026-08-27, item 25, and the two
        // halves moved for two different reasons that are worth keeping
        // apart:
        //
        // - **run6 was being run against the wrong map.** Its `WORLD` block
        //   is `BUILDS=7`'s — seventeen scalars, no cells, no tile masks —
        //   so the harness stood a flat, region-less, treeless world up and
        //   every number below was measured on it. `borrow_from_siblings`
        //   now takes the whole block from a sibling whose scalars match
        //   field for field, and the effect is the one that settles it:
        //   **run6 and run10 are the same game, and their diffs now agree
        //   exactly** — the same headline, the same first divergence for
        //   every unit. Before, run6 said player 1 broke at frame 2 and
        //   run10 said frame 4. That took the totals to 1,362/1,674
        //   (736/1,254 without the farmers), and moved the farmers' share
        //   *up*, from 304/317 to 626/420: their tiles were never being
        //   compared against real terrain.
        // - **The gather order is now diffed whole** — `tx`, `ty`, `wait`,
        //   `goto_build`, `been_there`, `dist_mod`, which the harness
        //   parsed and never compared. That is the other 1,221 order rows,
        //   all of them player 1's `1/1`, the citizen the original turns
        //   into a builder and this simulation keeps at the woodcutter
        //   (first row: frame 167). None of them is before the score.
        // 1,957/1,254 → **2,084/1,242** with item 43's exit ring, and the
        // totals fell: 2,583/1,674 to 2,591/1,613 with the farmers' share
        // down from 626/420 to 507/371. The AI's trained citizens are alive
        // on the map from the frame the original creates them instead of
        // standing on their city, so they are compared where they used not
        // to be — and the check that says this is the right trade is the
        // first-divergence list, every entry of which held or improved.
        // 2,084/1,254 → **848/1,050** with item 49 (the blocked stand):
        // run6 is run10's game, so the two blocked stands go with it, and
        // the ceiling comes down to what the run now measures rather than
        // to a peak nobody has reached since.
        // 832/1,048 measured → **853/1,024** with item 63 (the waypoint's
        // own collision test): the paths half fell by 24 and the orders
        // half rose by 21, and all of both is `1/6`, `1/7` and `1/8` — the
        // AI's citizens, whose own first divergences are 208, 210 and 323
        // and did not move. What settles that this is the untraced tail
        // rather than a loss is the first-divergence list, which is run10's
        // to the frame: **six of the ten units improved and four held**,
        // `0/3` and `0/5` 213 → 326, `0/4` 220 → 356, `1/3` 219 → 345,
        // `1/4` 201 → 316, `1/5` 219 → 243.
        // 853/1,024 → **697/1,044** with item 64 (`is_flat`): the orders
        // half fell by 156 and the paths half rose by 20, and again the
        // first-divergence list is what says which way the run moved —
        // **every unit held or improved, and one left the list**. `0/4`
        // now tracks the original's position for the whole run, `0/3`
        // 326 → 331, `1/3` 345 → 411, `1/4` 316 → 318 and `1/6` 208 →
        // 253; `0/5`, `1/5`, `1/7` and `1/8` held. The twenty extra path
        // frames are `1/6`'s own: it walks for another forty-five frames
        // instead of standing still re-making the same order, and a stack
        // it carries is a stack that can disagree.
        assert!(
            orders - farmer_orders <= 697 && paths - farmer_paths <= 1_044,
            "disagreements grew: orders {orders} ({farmer_orders} farmers'), paths {paths} ({farmer_paths} farmers')"
        );
        // Printed so a re-base reads the numbers off `--nocapture`.
        eprintln!(
            "run6: orders {orders} ({farmer_orders} farmers'), paths {paths} ({farmer_paths} farmers')"
        );
        // Re-based a third time, 2026-08-24 (run13): the re-target's modulus
        // is 4, not 3 — sixteen cells to be sent to, not nine — and the
        // unit loop now rotates by owner, so the AI's farmers draw first at
        // frame 101. On run6's own stream past frame 3 both change which
        // tiles the six farmers are sent to, and the counts moved from
        // 527/357 to 662/432 — then to **588/372** once run13's words were
        // pooled in and the first re-target ran on the original's stream.
        // The farmers' pin with teeth is run13's, where the AI's goals are
        // compared (`run13_s_window_counts_and_the_ai_farmers_re_targets_are_matched`).
        // 588/372 → 612/441 with the animation clock (`docs/ANIM.md`): the
        // farmers' second re-target, past the last traced frame, rolls on
        // the sim's own stream, and the idle wraps and the training rolls
        // now sit in front of it — the tiles it sends them to are as much
        // its own as before, only different ones.
        // 612/441 → **304/317** with item 34: the farmers walk on the
        // original's frames now, so the tiles their second re-target picks
        // are fewer frames' worth of drift away from the original's.
        // 304/317 → **626/420** with item 25's whole-`WORLD` borrow, and
        // this is a re-base rather than a regression: 304/317 was measured
        // against a flat treeless world of the harness's own making. run10
        // — the same game, with its own map — has always read the farmers
        // the way run6 reads them now, and the two captures now agree unit
        // for unit.
        // 626/420 → **675/346** with item 46 (collision). The paths half
        // fell by 74 and the orders half rose by 49, and the split is the
        // familiar one: six farmers standing in a cluster around one farm
        // collide constantly, so their walks now go round each other on the
        // 48-grid. Everyone else's share fell on both halves (1,979/1,194
        // to 1,930/1,120), and run10 — the same game — went 122 → 170.
        // 675/420 → **719/404** with item 50 (the bird, `docs/SYNC.md`
        // §3.9), and this is the same re-base every stream change makes
        // here: run6 is run10's game, so a bird hatches in it too and
        // spends three draws every eighth frame from then on. The farmers'
        // *second* re-target rolls past the last traced word, on the sim's
        // own stream, so its tiles move whenever the stream does — in
        // either direction, and here both (orders +44, paths −16). The
        // half of this test with teeth is `rest`, which is held to the
        // letter and did not move; run10's headline went 181 → 185.
        // 719/404 → **739/390** with item 59 (the builder's animation), and
        // the totals fell hard: 2,591/1,613 to **1,588/1,415**. run6 is
        // run10's game, so the builders' arrival stand goes with it and the
        // stream is the original's for 81 more frames; what that buys is
        // 1,003 fewer order disagreements and 198 fewer path ones across
        // everyone. The farmers' own share moved twenty rows the other way,
        // because their second re-target still rolls past the last traced
        // word — the same drift this split has always carried.
        // 739/390 → **503/290** with item 49 (the blocked stand), and the
        // totals with them: 1,588/1,415 to **1,351/1,340**. Six farmers in
        // a cluster round one farm are what collides most in this capture,
        // so the idle a refused step re-rolls is theirs more often than
        // anyone's — and it puts them back on the original's stream.
        // 503/290 → **378/312** with item 61 (the farmer's cell index),
        // and the totals 1,351/1,340 → **1,210/1,360**. The orders half
        // fell 141 across everyone: six farmers now sow the cell the
        // original sows and leave it on the original's frame. The paths
        // half rose 20, and it is the drift this split has always
        // carried — the *second* re-target rolls past the last traced
        // word, so its tiles move whenever the stream does.
        assert!(
            farmer_orders <= 378 && farmer_paths <= 312,
            "the farmers' disagreements grew: orders {farmer_orders}, paths {farmer_paths}"
        );
    }

    /// **run83 — the last Great Lakes hole, and the deflection inside
    /// it** (2026-09-06, item 239).
    ///
    /// run53's game once more, window `[6864, 6916)` plus the shutdown's
    /// own block: 53 blocks over the only stretch of either map's run-up
    /// that no archive held. run76 stops at 6869 and run79 starts at 6910,
    /// so 6870–6909 existed nowhere and "in lockstep to 6982" rested, for
    /// forty of those frames, on the draw stream alone. `rngcmp.py` puts
    /// it on run53 over 6,931 frames and `samegame.py` puts its twelve
    /// overlapping blocks on run76 and run79 with none differing
    /// (`docs/RUNS.md`, "run83").
    ///
    /// **What is inside it.** No unit is born and none dies — 77 units on
    /// all 53 blocks. Five take a new order. There is exactly one blocked
    /// stand, `1/29`'s on **6892** against the standing citizen `1/17`,
    /// which registers nothing: the whole interaction is written on the
    /// walker. And the AI puts the three Archers **back** into a formation
    /// on 6907, 46 frames after run76 watched them leave one.
    ///
    /// **And the block is a deflection, not a stop** — but it is not the
    /// sidestep. `1/17` holds a `GATHERORDER`, so both of the fences that
    /// read *the other unit's* order kind fail (`docs/COLLISION.md` §6
    /// steps 4 and 5) and what runs is **step 6, the repath**: `1/29` is
    /// put on its own cell centre — the step "backwards" — its order's
    /// `dest` is cleared, and a three-entry `find_upath` plan goes on the
    /// stack above the world-grid one. The top two entries share the cell
    /// row it was snapped onto, so it slides due west at full speed for
    /// six frames with `y` pinned and takes its diagonal again on 6900.
    /// `idle` is 0 for all of it. Anything modelling a block as "stand
    /// still and repath" is six frames and about 1.6 tiles wrong here.
    ///
    /// Driven through [`run_traced`], so this is the whole record and not
    /// the walker's.
    #[test]
    fn run83_s_window_is_the_last_great_lakes_hole() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run83-greatlakes-wordwindow.txt"),
            trace("rontrace-run83.log"),
        ) else {
            eprintln!("skipping: no run83 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();

        // **The window's own facts, read straight off the dump**, so that
        // nothing this crate does can move them. 53 blocks: `[6864, 6916)`
        // entire, and then the **quit frame**, 6931, which the shutdown
        // dumps on its way out. A file with fewer is the wrong file.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs
                == (6_864..=6_915)
                    .chain(std::iter::once(6_931))
                    .collect::<Vec<_>>(),
            "run83's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );
        // No birth and no death across the hole: the same 77 units, by
        // `(who, o)`, on every block.
        let census: std::collections::BTreeSet<Vec<(i64, i64)>> = states
            .iter()
            .map(|f| {
                let mut v: Vec<(i64, i64)> = f.units.iter().map(|u| (u.who, u.o)).collect();
                v.sort_unstable();
                v
            })
            .collect();
        assert_eq!(
            census.len(),
            1,
            "the roster changes across run83's window — a birth or a death"
        );
        assert_eq!(
            census.iter().next().map(Vec::len),
            Some(77),
            "run83 should carry 77 units on every block"
        );

        // **And the five new orders**, which are everything else that
        // happens in the hole: the units whose `orders_x`/`orders_y` move
        // between blocks, with `1/17`'s falling on the same frame the
        // squad re-groups. None of the five is in the parted set below, so
        // this crate takes all five where the original does.
        let mut retargets: Vec<(i64, i64, i64)> = Vec::new();
        for pair in states.windows(2) {
            if pair[1].n > 6_915 {
                break;
            }
            for u in &pair[1].units {
                if let Some(was) = pair[0].units.iter().find(|p| p.who == u.who && p.o == u.o)
                    && (was.orders_x, was.orders_y) != (u.orders_x, u.orders_y)
                {
                    retargets.push((pair[1].n, u.who, u.o));
                }
            }
        }
        assert_eq!(
            retargets,
            vec![
                (6_865, 1, 5),
                (6_871, 1, 15),
                (6_875, 0, 5),
                (6_891, 0, 4),
                (6_907, 1, 17),
            ],
            "the window's own new orders"
        );

        // **The block itself, off the dump.** `1/29`'s track over
        // 6892..=6900, with the collision fields, the order's `dest` and
        // the path stack's depth beside it.
        let walker: Vec<(i64, LogPos, i64, i64, i64, usize, i64)> = states
            .iter()
            .filter(|f| (6_892..=6_900).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 29)?;
                let ord = u.current_order()?;
                Some((
                    f.n,
                    u.pos,
                    u.collide?,
                    u.collide_o?,
                    u.collide_frame?,
                    u.path.len(),
                    ord.dest?,
                ))
            })
            .collect();
        assert_eq!(walker.len(), 9, "run83's `1/29` over 6892..=6900");
        eprintln!("run83: 1/29 across the block: {walker:?}");
        // The three collision fields have three different lives, and a
        // comparison that reads `collide_o` a frame late sees −1 and calls
        // it agreement.
        assert!(
            walker[0].2 == 0 && walker[0].4 == -1 && walker[0].5 == 9,
            "1/29 should be unblocked on 6892 with its nine-entry world-grid \
             plan: {:?}",
            walker[0]
        );
        assert!(
            walker[1].2 == 1 && walker[1].3 == 17 && walker[1].4 == 6_892,
            "1/29 should name `1/17` on 6893: {:?}",
            walker[1]
        );
        assert!(
            walker[2..].iter().all(|r| r.3 == -1),
            "`collide_o` names the blocker for exactly one block: {walker:?}"
        );
        assert!(
            walker[1..].iter().all(|r| r.4 == 6_892),
            "`collide_frame` keeps the stamp: {walker:?}"
        );
        assert!(
            walker[1..7].iter().all(|r| r.2 == 1) && walker[7..].iter().all(|r| r.2 == 0),
            "`collide` should latch 1 over 6893..=6898 and clear on 6899: {walker:?}"
        );
        // **It is step 6, the repath.** The dump prints all three of its
        // parts: the unit is put on **its own** 48-cell centre, its
        // order's `dest` is cleared, and a fresh `find_upath` plan of
        // three cell-centre entries (`tolerance 0`, `flags 2`) sits on the
        // stack above the world-grid one, which is untouched beneath.
        assert_eq!(
            (walker[1].1.x, walker[1].1.y),
            (42_408, 23_880),
            "6893 should put `1/29` on its own 48-cell centre"
        );
        assert!(
            walker[1].6 == 0 && walker[1].5 == 12,
            "6893 should clear `dest` and carry the nine-entry plan plus a \
             three-entry `find_upath`: {:?}",
            walker[1]
        );
        let plan: Vec<(i64, i64, i64, i64)> = states
            .iter()
            .find(|f| f.n == 6_893)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 29))
            .map(|u| {
                u.path
                    .iter()
                    .skip(9)
                    .map(|w| (w.to.0, w.to.1, w.tolerance, w.flags))
                    .collect()
            })
            .unwrap();
        assert_eq!(
            plan,
            vec![
                (41_784, 23_400, 0, 2),
                (42_264, 23_880, 0, 2),
                (42_360, 23_880, 0, 2),
            ],
            "the repath's `find_upath` plan, top last"
        );
        // And the walk it produces, which is what "stand still and repath"
        // gets wrong: six frames due west at the full 26, truncated only
        // where the unit lands on a waypoint.
        let slide: Vec<(i64, i64)> = walker
            .windows(2)
            .map(|w| (w[1].1.x - w[0].1.x, w[1].1.y - w[0].1.y))
            .collect();
        eprintln!("run83: 1/29's steps across the block: {slide:?}");
        assert_eq!(
            slide,
            vec![
                (20, 22),  // 6893: the snap onto its own cell centre
                (-26, 0),  // 6894: the first upath leg, at full speed
                (-22, 0),  // 6895: truncated — it lands on (42360, 23880)
                (-26, 0),  // 6896
                (-26, 0),  // 6897
                (-26, 0),  // 6898
                (-18, 0),  // 6899: truncated — it lands on (42264, 23880)
                (-26, -1), // 6900: the diagonal again
            ],
            "the block's whole track"
        );
        assert!(
            states
                .iter()
                .filter(|f| (6_892..=6_900).contains(&f.n))
                .filter_map(|f| f.units.iter().find(|u| u.who == 1 && u.o == 29))
                .all(|u| u.idle == Some(0)),
            "`1/29` never idles across the block"
        );
        // **And the blocker registers nothing.** `1/17` is a standing
        // citizen — `myspeed 25`, `form 9`, a `GATHERORDER` whose target
        // is where it stands, which is why steps 4 and 5 both fall through
        // — carrying an ancient stamp it does not touch.
        let blocker: Vec<(i64, LogPos, i64, i64)> = states
            .iter()
            .filter(|f| (6_890..=6_899).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 17)?;
                Some((f.n, u.pos, u.collide?, u.collide_frame?))
            })
            .collect();
        assert!(
            blocker.len() == 10
                && blocker
                    .iter()
                    .all(|r| r.1.x == 42_360 && r.1.y == 23_736 && r.2 == 0 && r.3 == 3_831),
            "`1/17` should stand still through the block, registering nothing: {blocker:?}"
        );

        // Now the comparison.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&6_864) && blocks.last() == Some(&6_931) && blocks.len() == 53,
            "run83's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let ord_fields: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run83: {} unit fields, {ord_fields} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        use crate::diff::order::OrderMismatch as OM;
        let mut tally: std::collections::BTreeMap<(i64, i64, String), (usize, i64)> =
            std::collections::BTreeMap::new();
        for d in report.frames.iter().flat_map(|f| f.order_diverged.iter()) {
            let field = match &d.what {
                OM::Group { field, .. } => format!("Group.{field}"),
                OM::Move { field, .. } => format!("Move.{field}"),
                w => format!("{w:?}")
                    .split('{')
                    .next()
                    .unwrap()
                    .trim()
                    .to_string(),
            };
            let e = tally.entry((d.who, d.o, field)).or_insert((0, d.frame));
            e.0 += 1;
        }
        for ((who, o, what), (n, first)) in &tally {
            eprintln!("  order {who}/{o} {what}: {n} rows, first {first}");
        }
        assert!(
            ord_fields >= 1_000 && angles >= 1_000,
            "the window's own rows: {ord_fields} order fields and {angles} angles \
             — a capture below `UNITS=3` is the wrong file"
        );

        // **What the hole holds: nothing new.** Every disagreement in the
        // window is one this crate carried into it, the group-id stand-in,
        // or the quit frame — so the forty frames nobody had ever compared
        // add no divergence of their own.
        //
        // Positions, three units and each accounted for: `1/24` and `1/25`
        // are already off inside run76's window (6640 and 6745 there) and
        // so part here on the window's **first** block; `0/3` parts only
        // on **6931**, the shutdown's own block, sixteen frames past the
        // window's end where nothing is observed.
        let inherited: Vec<(i64, i64)> = parted
            .iter()
            .filter(|&(_, &f)| f <= 6_864)
            .map(|(&k, _)| k)
            .collect();
        assert_eq!(
            inherited,
            vec![(1, 24), (1, 25)],
            "the units already off position when the window opens: {parted:?}"
        );
        let inside: Vec<((i64, i64), i64)> = parted
            .iter()
            .filter(|&(_, &f)| f > 6_864 && f <= 6_915)
            .map(|(&k, &f)| (k, f))
            .collect();
        assert!(
            inside.is_empty(),
            "a unit parts from the original's position **inside** the hole \
             nothing had ever compared: {inside:?}"
        );
        assert_eq!(
            parted.len(),
            3,
            "run83's parted set should be `1/24`, `1/25` and the quit \
             frame's `0/3`: {parted:?}"
        );

        // Orders: three families and no fourth. `1/23` is the caravan, off
        // on the same three rows from the window's first block and so
        // carried in; the three Archers carry `Group.id` — `GroupData
        // +0x4`, the stand-in run76's window pins the same way — and
        // nothing else.
        let families: Vec<(i64, i64, String, usize, i64)> = tally
            .iter()
            .map(|((w, o, f), (n, first))| (*w, *o, f.clone(), *n, *first))
            .collect();
        assert_eq!(
            families,
            vec![
                (1, 23, "Action".into(), 52, 6_864),
                (1, 23, "Flags".into(), 52, 6_864),
                (1, 23, "Move.angle".into(), 52, 6_864),
                (1, 27, "Group.id".into(), 9, 6_907),
                (1, 28, "Group.id".into(), 9, 6_907),
                (1, 29, "Group.id".into(), 9, 6_907),
            ],
            "run83's order rows are the caravan's three, carried into the \
             window, and the group-id stand-in from the re-group"
        );

        // **The re-group on 6907, which nothing had ever seen.** The AI
        // puts the three Archers back into a `GROUPATTACKTOORDER` (`type
        // 21`) nine blocks before the window ends — 46 frames after run76
        // watched the same squad *leave* one on 6861 — and this crate
        // follows it on the frame it happens: the only rows the three
        // carry from 6907 on are the `id` stand-in, one per grouped
        // unit-frame. A squad that misses the re-group, or takes it late,
        // puts a `Kind` row here.
        let regrouped: Vec<i64> = states
            .iter()
            .filter(|f| {
                f.units.iter().any(|u| {
                    u.who == 1 && (27..=29).contains(&u.o) && u.orders.iter().any(|o| o.index == 21)
                })
            })
            .map(|f| f.n)
            .collect();
        assert_eq!(
            regrouped,
            (6_907..=6_915).collect::<Vec<_>>(),
            "the original's re-group should run 6907 to the window's end — \
             and be over by the quit block, 6931"
        );
        let squad_rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| {
                d.who == 1
                    && (27..=29).contains(&d.o)
                    && !matches!(d.what, OM::Group { field: "id", .. })
            })
            .collect();
        assert!(
            squad_rows.is_empty(),
            "the squad parts from the original's order record: {:?}",
            squad_rows.iter().take(6).collect::<Vec<_>>()
        );

        // **And the walker's own record, which is what the item is
        // about.** `1/29` is blocked on 6892 and recovers by the repath;
        // this crate does the same, so it holds the original's position
        // for the whole window and carries no order or path row across the
        // block. Made to fail on purpose: making `Sim::unit_step`'s
        // collision arm return without calling `resolve` — the stand — put
        // `1/29` off position from 6893 to the window's end.
        assert!(
            !parted.contains_key(&(1, 29)),
            "`1/29` parts at {:?}: the block on 6892 is a deflection, not a \
             stop, and six frames of it are 1.6 tiles",
            parted.get(&(1, 29))
        );
        let block_rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| d.who == 1 && d.o == 29 && (6_890..=6_905).contains(&d.frame))
            .collect();
        assert!(
            block_rows.is_empty(),
            "`1/29`'s order or path record parts across the block: {:?}",
            block_rows.iter().take(6).collect::<Vec<_>>()
        );
    }

    /// **run85 — East Indies' word window, and the walker that is not
    /// there** (2026-09-06, item 241).
    ///
    /// run54's game, window `[7400, 7480)` plus the quit block 7496: 81
    /// blocks over the frame the second map's word parts on, which no
    /// dump had come within 3,200 frames of. `rngcmp.py` puts it on run54
    /// over 7,496 frames with none differing (`docs/RUNS.md`, "run85").
    ///
    /// This is the whole record, not the walker's, and that is what it is
    /// for: the word's own frame names `1/20`, and the window says the
    /// unit is nowhere near the animal it collides with.
    #[test]
    fn run85_s_window_is_the_east_indies_word_frame() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run85-eastindies-blockedstand.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run85.log"),
        ) else {
            eprintln!("skipping: no run85 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];

        // The window's own facts, read straight off the dump.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs
                == (7_400..=7_479)
                    .chain(std::iter::once(7_496))
                    .collect::<Vec<_>>(),
            "run85's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );

        // **The blocked stand itself, off the disk.** `1/20` walks
        // south-west at (-13, -19) a frame, is put *back* to (29256,
        // 24888) on 7449 with `collide 1, collide_o 0, collide_who 8` —
        // gaia's animal — stands four frames while its angle snaps due
        // west, and slides west with `y` pinned from 7454.
        let walker: Vec<(i64, i64, i64, i64, i64, i64)> = states
            .iter()
            .filter(|f| (7_446..=7_455).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 20)?;
                Some((
                    f.n,
                    u.pos.x,
                    u.pos.y,
                    u.collide?,
                    u.collide_o?,
                    u.collide_who?,
                ))
            })
            .collect();
        assert_eq!(
            walker,
            vec![
                (7_446, 29_268, 24_914, 0, -1, -1),
                (7_447, 29_255, 24_895, 0, -1, -1),
                (7_448, 29_242, 24_876, 0, -1, -1),
                (7_449, 29_256, 24_888, 1, 0, 8),
                (7_450, 29_256, 24_888, 1, -1, -1),
                (7_451, 29_256, 24_888, 1, -1, -1),
                (7_452, 29_256, 24_888, 1, -1, -1),
                (7_453, 29_256, 24_888, 1, -1, -1),
                (7_454, 29_233, 24_888, 1, -1, -1),
                (7_455, 29_210, 24_888, 0, -1, -1),
            ],
            "run85's blocked stand: the walker's own record"
        );

        // Now the comparison — the whole record over the window.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&7_400) && blocks.last() == Some(&7_496) && blocks.len() == 81,
            "run85's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let ord_fields: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run85: {} unit fields, {ord_fields} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        let unlinked: std::collections::BTreeSet<(i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.unlinked_units.iter().copied())
            .collect();
        eprintln!("run85: unlinked {unlinked:?}");
        use crate::diff::order::OrderMismatch as OM;
        let mut tally: std::collections::BTreeMap<(i64, i64, String), (usize, i64)> =
            std::collections::BTreeMap::new();
        for d in report.frames.iter().flat_map(|f| f.order_diverged.iter()) {
            let field = match &d.what {
                OM::Group { field, .. } => format!("Group.{field}"),
                OM::Move { field, .. } => format!("Move.{field}"),
                w => format!("{w:?}")
                    .split('{')
                    .next()
                    .unwrap()
                    .trim()
                    .to_string(),
            };
            let e = tally.entry((d.who, d.o, field)).or_insert((0, d.frame));
            e.0 += 1;
        }
        for ((who, o, what), (n, first)) in &tally {
            eprintln!("  order {who}/{o} {what}: {n} rows, first {first}");
        }

        // **What the word is missing, off the trace.** Frame 7448 spends
        // 34 draws and two of them are the blocked stand — not two
        // figures of two units but `Unit::set_anim@00616f40`'s two loops,
        // the squad's and the crew's, over one Merchant with `CREW_SIZE
        // 1` (`docs/COLLISION.md` §8.2).
        let theirs_7448 = tr.labels(7_448);
        assert_eq!(theirs_7448.len(), 34, "run85's 7448 spends 34 draws");
        assert_eq!(
            theirs_7448
                .iter()
                .filter(|l| *l == sim::anim::SITE_BLOCKED)
                .count(),
            2,
            "the two draws the word is short are both the blocked stand"
        );
        for f in report.frames.iter().filter(|f| {
            f.frame == 7_400 || f.frame == 7_448 || f.frame == 7_449 || f.frame == 7_479
        }) {
            for d in &f.diverged {
                eprintln!(
                    "  f{} {}/{} ours ({}, {}) theirs ({}, {})",
                    d.frame, d.who, d.o, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y
                );
            }
        }
        assert!(
            ord_fields >= 1_000 && angles >= 1_000,
            "the window's own rows: {ord_fields} order fields and {angles} angles \
             — a capture below `UNITS=3` is the wrong file"
        );

        // **One unit, and the walker is not it.** It was three until
        // 2026-09-06 — `1/20` opened the window (36, 792) behind and
        // `0/5` parted at 7468 downstream of it — and both went with the
        // barge's speed (item 241, run86): `1/20` now rides the transport
        // ashore on the original's own frame, walks into the animal on
        // time, and the stand at 7448 is this crate's too. What is left
        // is `1/19`'s unpack constant, which is the same row run82's
        // window carries from 6883 on.
        assert_eq!(
            parted,
            [((1, 19), 7_400)].into_iter().collect(),
            "run85's parted set"
        );

        // That the crate now *spends* 7448's two draws is pinned where
        // the word is, not here: [`LONG_WORD_EAST_INDIES`] is 7529, and
        // `run54_s_24000_frames…` asserts the floor on the same game.

        // `1/19` is the unpacked Merchant standing on its trade-post
        // spot: **a constant (24, 24), on every block, both sides
        // still**. The original moved it onto the tile corner (32256,
        // 36864) = 192 x (168, 192) somewhere in the 470 frames no dump
        // covers, and this crate left it where run82's closing block had
        // it (`docs/MERCHANT.md`, and the successor in the queue).
        let merchant: Vec<(i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 19))
            .map(|d| (d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            merchant.len() == 81 && merchant.iter().all(|&d| d == (24, 24)),
            "`1/19` is a standing constant: {} rows, {:?}",
            merchant.len(),
            merchant.first()
        );

        // **And `1/20` holds no divergence of any kind now** — not a
        // position, not a waypoint. It used to hold both, and the pair
        // was the measurement item 241 was taken from: the waypoint's
        // **column** agreed for 28 blocks while the **row** was a
        // waypoint out, which is what "behind on its own chain rather
        // than beside it" looks like in the order record, and the lag
        // opened at exactly (36, 792). A unit on a different route parts
        // on both at once; this one was late, and the lateness was the
        // barge's speed.
        assert!(
            tally.keys().all(|(who, o, _)| (*who, *o) != (1, 20)),
            "`1/20` holds an order divergence again: {:?}",
            tally
                .keys()
                .filter(|(who, o, _)| (*who, *o) == (1, 20))
                .collect::<Vec<_>>()
        );
        let lag: Vec<(i64, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 20))
            .map(|d| (d.frame, d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            lag.is_empty(),
            "`1/20` is on the original's point for all 81 blocks: {:?}",
            lag.first()
        );
    }

    /// **run88 — East Indies' word is a gather countdown one tick long**
    /// (2026-09-07, the capture lane).
    ///
    /// run54's game over `[7474, 7800)` at run85's detail **exactly** —
    /// **327 blocks, 186,777,592 bytes**, six of deliberate overlap over
    /// run85's tail (`differ: 0`, byte for byte and with no `--exclude`
    /// at all) and 326 above it. Until this capture East Indies had **no
    /// dumped frame above 7480** except run85's own `GameLog::end_game`
    /// block at 7496, so [`LONG_WORD_EAST_INDIES`] — the second of
    /// `docs/DECISIONS.md` entry 29's two counters — was the one that
    /// could not be read as a field at all.
    ///
    /// **It was one field, and it was off by one.** `1/13` is an AI
    /// citizen gathering at `gather_down 12`; its `GATHERORDER`'s `wait`
    /// countdown read **`theirs + 1` on all 55 blocks** of the cycle that
    /// ends at the word, so the original's reached its end on 7529 (`wait
    /// −1`, the order done) while this crate still held **1**, and the
    /// original spent its extra `Guy::set_anim+0x97a <
    /// Unit::do_non_flat_gather+0xb99` draw there a frame before this
    /// crate did. The original's own value diff was one 24-unit step: it
    /// takes its next job on 7530 — `orders_x/y` (40536, 37560) →
    /// (40344, 38520) — and steps to (40560, 37560) on 7531.
    ///
    /// **Item 271 closed it 545 frames upstream, and not in the gather
    /// code.** The countdown is seeded by the *arrival* frame and this
    /// crate arrived one late: run86's block 6937 is where player 1 takes
    /// the Classical Age, and `Leader::gain_tech`'s `is_age_type` arm
    /// re-places every one of its units where it already stands, which
    /// hands the one unit that is mid-turn its heading for free
    /// (`docs/TECH.md`, "An age snaps every figure"). With the snap in,
    /// `1/13`'s `wait` agrees on every block of this window and its
    /// position never parts; [`LONG_WORD_EAST_INDIES`] moved 7529 → 7806.
    ///
    /// What is left below the new word is the **standing Merchant
    /// constant**, twice: `1/19` from the first block and `1/20` from
    /// 7663, each a flat (24, 24) (`docs/MERCHANT.md`).
    #[test]
    fn run88_s_window_is_east_indies_word_frame() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run88-eastindies-wordframe.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run88.log"),
        ) else {
            eprintln!("skipping: no run88 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];

        // The window's own extent: the 326 blocks and the
        // `GameLog::end_game` block the `!quit` writes, which is run85's
        // shape one window along.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs
                == (7_474..=7_799)
                    .chain(std::iter::once(7_816))
                    .collect::<Vec<_>>(),
            "run88's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );

        // **The gather countdown, straight off the dump.** `1/13` stands
        // on its resource with `orders_x/y` equal to its own position
        // until 7530, when the original gives it the next job.
        let job: Vec<(i64, i64, i64, i64, i64, i64)> = states
            .iter()
            .filter(|f| (7_526..=7_532).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 13)?;
                let wait = u.orders.iter().find_map(|o| o.wait)?;
                Some((f.n, u.pos.x, u.pos.y, u.orders_x?, u.orders_y?, wait))
            })
            .collect();
        assert_eq!(
            job,
            vec![
                (7_526, 40_536, 37_560, 40_536, 37_560, 3),
                (7_527, 40_536, 37_560, 40_536, 37_560, 2),
                (7_528, 40_536, 37_560, 40_536, 37_560, 1),
                (7_529, 40_536, 37_560, 40_536, 37_560, -1),
                (7_530, 40_536, 37_560, 40_344, 38_520, 32),
                (7_531, 40_560, 37_560, 40_344, 38_520, 32),
                (7_532, 40_584, 37_560, 40_344, 38_520, 32),
            ],
            "the original's `1/13`: the countdown out on 7529, the next job \
             on 7530 with `wait` re-rolled, and the first step on 7531"
        );

        // Now the comparison — every unit of every block.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&7_474) && blocks.last() == Some(&7_816) && blocks.len() == 327,
            "run88's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let ord_fields: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        eprintln!(
            "run88: {} unit fields, {ord_fields} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        assert!(
            ord_fields >= 1_000 && angles >= 1_000,
            "the window's own rows: {ord_fields} order fields and {angles} angles \
             — a capture below `UNITS=3` is the wrong file"
        );

        // **Two units part below the word and no more, and both are the
        // same standing Merchant constant.** `1/13` was the third and is
        // gone. `1/6` was the fourth, on the closing block 7816, and is
        // gone too: item 289 put the sidestep waypoint's arrival rule
        // right, so the citizen that walked one step past the original on
        // every collision now walks the original's shuffle and stands
        // where the original stands on the block the `!quit` catches.
        // `1/7` was the fifth — the other half of that shuffle, §6 step
        // 5's wait-versus-pause row, which 289 did not touch — and it went
        // on 2026-09-17 with the suspended-search block (items 301 and
        // 304, `docs/PATHFINDER.md` §18.5): the original's repath leaves
        // the search suspended where this crate walked the goal back onto
        // the unit's own cell, and with the block wired the stack is the
        // original's and the position never parts.
        assert_eq!(
            parted
                .iter()
                .map(|(&k, &f)| (k, f))
                .collect::<Vec<((i64, i64), i64)>>(),
            vec![((1, 19), 7_474), ((1, 20), 7_663)],
            "run88's parted set, whole"
        );

        // `1/19` is run85's row one window along: the unpacked Merchant
        // standing on its trade-post spot, **a constant (24, 24) on every
        // block, both sides still** (`docs/MERCHANT.md`).
        let merchant = |o: i64| -> Vec<(i32, i32)> {
            report
                .frames
                .iter()
                .flat_map(|f| f.diverged.iter())
                .filter(|d| (d.who, d.o) == (1, o))
                .map(|d| (d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
                .collect()
        };
        let (nineteen, twenty) = (merchant(19), merchant(20));
        assert!(
            nineteen.len() == 327 && nineteen.iter().all(|&d| d == (24, 24)),
            "`1/19` is a standing constant: {} rows, {:?}",
            nineteen.len(),
            nineteen.first()
        );
        // `1/20` is the same shape one Merchant along: it stands on its own
        // trade-post spot from 7663 and is a flat (24, 24) for every block
        // after, both sides still.
        assert!(
            twenty.len() == 138 && twenty.iter().all(|&d| d == (24, 24)),
            "`1/20` is the second standing constant: {} rows, {:?}",
            twenty.len(),
            twenty.first()
        );

        use crate::diff::order::OrderMismatch as OM;

        // **The closing block is the word's own residue, and this is its
        // value diff** (item 276, 2026-09-07). `LONG_WORD_EAST_INDIES` is
        // 7806, ten frames under this block and past every dumped frame
        // on the map, so the question the item asked was whether the
        // `!quit`'s own block explains it. It does not explain the
        // *mechanism* — 7800..7815 is undumped and that is where the
        // shuffle happens — but it named the two units and priced them:
        // `1/6` was 46 east and 51 south of the original's, `1/7` six west
        // and twelve south, and `1/19`/`1/20` are the two standing
        // Merchant constants that were already here.
        //
        // A closing dump is frame *n* except for the one unit the quit
        // caught mid-update (item 257, open), so the rows below are worth
        // exactly as much as a corroborating instrument makes them. The
        // trace was that instrument and it is not a dump: `1/6`'s stop was
        // one frame late on it, which no torn record can fake.
        //
        // **And `1/6`'s row is gone** (item 289, 2026-09-07). run90 dumped
        // the shuffle 7800..7815 that this block could only stand
        // downstream of, and the frame was the sidestep waypoint's arrival
        // rule (`docs/COLLISION.md` §8.7). The citizen the closing block
        // priced at (+46, +51) now stands where the original stands on the
        // block the `!quit` catches. `1/7`, the other half of the shuffle,
        // is still (−6, +12) out: that is §6 step 5's wait-versus-pause
        // row, which 289 did not touch and which was the word.
        //
        // **And `1/7`'s row is gone too** (items 301 and 304, 2026-09-17).
        // run90 dumped that half of the shuffle as well, and the mechanism
        // was the suspended search: the original's repath on 7810 leaves
        // the stack as the move's own goal and the search suspended, where
        // this crate walked the goal back onto the unit's own cell.
        // `do_move`'s §4.4 step 2 block is what models it, and
        // `kill_current_path`'s `clear_partial_path` is what let it be
        // wired (`docs/PATHFINDER.md` §18.5). The closing block's residue
        // is now the two standing Merchant constants and nothing else, on
        // a 327-block window.
        let residue: Vec<(i64, i64, i32, i32, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| d.frame == 7_816)
            .map(|d| (d.who, d.o, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y))
            .collect();
        assert_eq!(
            residue,
            vec![
                (1, 19, 32_280, 36_888, 32_256, 36_864),
                (1, 20, 28_632, 24_024, 28_608, 24_000),
            ],
            "the closing block's whole residue, both sides' coordinates"
        );

        // **And the order records, every field, over every block.** The
        // position comparison above is a quarter of what this capture
        // dumps; the rest went untallied until item 276 and it is a
        // standing set, not a moving one. `1/6` and `1/7` hold **no**
        // order row at all — which is what says their divergence opens
        // after 7799 rather than earlier and quietly.
        let mut tally: std::collections::BTreeMap<(i64, i64, String), usize> = Default::default();
        for d in report.frames.iter().flat_map(|f| f.order_diverged.iter()) {
            let field = match &d.what {
                OM::Group { field, .. } => format!("Group.{field}"),
                OM::Move { field, .. } => format!("Move.{field}"),
                w => format!("{w:?}")
                    .split('{')
                    .next()
                    .unwrap()
                    .trim()
                    .to_string(),
            };
            *tally.entry((d.who, d.o, field)).or_default() += 1;
        }
        assert_eq!(
            tally
                .iter()
                .map(|((w, o, f), n)| ((*w, *o, f.as_str()), *n))
                .collect::<Vec<_>>(),
            vec![
                ((1, 0, "Move.facing"), 101),
                ((1, 18, "Action"), 326),
                ((1, 18, "Flags"), 326),
                ((1, 18, "Move.angle"), 324),
            ],
            "run88's order-field residue, whole — `1/18` is the parked \
             Transport Barge and `1/0` the AI's first citizen"
        );

        // **The field the word used to be.** `1/13`'s `wait` read
        // `theirs + 1` on all 56 blocks up to 7529, where the original's
        // reached its end (`-1`) and this crate still held 1. The age snap
        // put the arrival frame right and there is no row left at all.
        let wait: Vec<(i64, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 13))
            .filter_map(|d| match d.what {
                OM::Gather {
                    field: "wait",
                    ours,
                    theirs,
                } => Some((d.frame, ours, theirs)),
                _ => None,
            })
            .collect();
        assert!(
            wait.is_empty(),
            "`1/13`'s gather countdown, over the whole window: {} rows, \
             {:?}..{:?}",
            wait.len(),
            wait.first(),
            wait.last()
        );
        // **And no other worker's either.** `1/7`'s own `wait` used to open
        // at 375 against 334 — forty-one out, downstream of the word and
        // proof at the time that the comparison could see something other
        // than a `+1`. It went with the same fix, so the assertion is now
        // over the whole map: not one gather countdown on it disagrees,
        // and the `job` rows above are what say the field is there to
        // disagree about.
        let all_waits: Vec<(i64, i64, i64, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter_map(|d| match d.what {
                OM::Gather {
                    field: "wait",
                    ours,
                    theirs,
                } => Some((d.frame, d.who, d.o, ours, theirs)),
                _ => None,
            })
            .collect();
        assert!(
            all_waits.is_empty(),
            "gather countdowns still disagreeing on run88: {} rows, {:?}",
            all_waits.len(),
            all_waits.first()
        );

        // **And the value diff is gone with it.** `1/13` used to part on
        // 7531 by one 24-unit step and never get it back; it now holds the
        // original's own point on every block of the window, the two job
        // changes above included.
        let step: Vec<(i64, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 13))
            .map(|d| (d.frame, d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            step.is_empty(),
            "`1/13` is on the original's point for all 327 blocks: {:?}",
            step.first()
        );

        // **The original's own draw stream at 7529**, off this run's
        // trace, and it is unchanged: three draws there, one of them the
        // gather that fires, and a single `inc_time` on 7530. This crate
        // used to spend its `do_non_flat_gather` on the later of the two;
        // the check below is that the whole window's counts now agree.
        const GATHER_ANIM: &str = "Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99";
        let at_7529 = tr.labels(7_529);
        let after = tr.labels(7_530);
        assert!(
            at_7529.len() == 3
                && at_7529.iter().any(|l| l == GATHER_ANIM)
                && after.len() == 1
                && !after.iter().any(|l| l == GATHER_ANIM),
            "the original's draws either side of 7529: {at_7529:?} then {after:?}"
        );

        // **And the draw counts themselves, block for block.** The word is
        // the frame the two streams part, and on this window they do not:
        // every traced block of [7474, 7800) spends what the original
        // spends, which is what took [`LONG_WORD_EAST_INDIES`] past the
        // end of this capture to 7806 (`run54_s_24000_frames…`).
        let apart: Vec<(i64, Option<u32>, Option<u32>)> = report
            .rng_frames
            .iter()
            .filter(|(n, _, _)| (7_474..7_800).contains(n))
            .filter(|(_, ours, theirs)| ours != theirs)
            .copied()
            .collect();
        assert!(
            apart.is_empty(),
            "run88's draw counts part inside its own window: {:?}",
            apart.first()
        );
    }

    /// **run90 — East Indies' word is a sidestep waypoint this crate walks
    /// to and the original abandons** (2026-09-07, the capture lane).
    ///
    /// run54's game over `[7790, 7900)` at run88's detail **exactly** —
    /// **111 blocks, 71,645,977 bytes**, ten of deliberate overlap over
    /// run88's tail (`differ: 0`, byte for byte and with no `--exclude`)
    /// and 100 above it. It is the first dump of any frame in
    /// `[7800, 7899]` on either map, which is where
    /// [`LONG_WORD_EAST_INDIES`] — the second of `docs/DECISIONS.md` entry
    /// 29's counters — actually turns.
    ///
    /// **It was booked to be refused and it refused.** Item 276 dated the
    /// word off the trace and wrote this crate's `collide`/`pause`/`wait`
    /// rows into the capture's stanza as predictions. Two held exactly:
    /// `1/7`'s `pause` reads **3, 3, 2, 1, 0** on blocks 7803-7807 on both
    /// sides — the first non-zero `MOVEORDER pause` any dump on this disk
    /// has ever printed — and both sides make the first collision on 7803
    /// with `collide_o 7` / `collide_who 1`.
    ///
    /// **The two that were refused are the finding.** `1/6`'s position
    /// parts on block **7805**, two blocks *below* the draw stream's word,
    /// and the cause is the sidestep waypoint §6 step 4 pushes. Both sides
    /// push the **same point** — `(39720, 38808)` with `flags 2` on 7803,
    /// `(39672, 38808)` on 7807, `(39624, 38808)` on 7811 — but the
    /// original's path drops from length 5 to 4 on the **next** block with
    /// the unit at (39729, 38802), which is not that point: it takes one
    /// full step along the bearing and abandons the waypoint. This crate
    /// keeps it and walks the remainder, a short `(−9, +6)` step, so every
    /// collision cycle costs it one frame — the original's is four blocks
    /// (block, step, turn, step) and this crate's is five.
    ///
    /// So `docs/COLLISION.md` §8.5's candidate — "one frame of
    /// turn-versus-step *after* a waypoint pop" — is refused: the frame is
    /// lost in the step *to* the waypoint, one block earlier, and the pop
    /// is not late but never arrives.
    ///
    /// **And §8.5's unestablished suspicion is established.** The
    /// `SITE_PAUSE` the original spends on sim-frame 7810 is `1/7`'s: its
    /// `MOVEORDER` carries **`pause 8`** on block 7811, held while
    /// `collide` counts 1..9 through 7819 and counted down over
    /// 7822-7828. This crate sets §6 step 5's wait flag there and rolls
    /// nothing, so the wait-versus-repath predicate is wrong — the shape
    /// `docs/audit/README.md` says the errors take.
    ///
    /// The capture itself moved no word; what it bought is that East
    /// Indies 7806 stopped being a draw-stream report and became a field
    /// with a value diff beside it, two frames earlier than the stream
    /// noticed.
    ///
    /// **And then item 289 read the rule and this test turned over**
    /// (2026-09-07, `docs/COLLISION.md` §8.7). The arrival rule is
    /// `move_step`'s ordinary post-step test — Manhattan, against
    /// **`UnitData::tolerance`**, and only on a frame whose step was
    /// accepted — and `resolve_unit_collision@005f9d30`'s `LAB_005fa37a`
    /// never writes that field: it pushes the entry and stores the order's
    /// `+0x2c`/`+0x30`, and the entry's own `tolerance 0` reaches the unit
    /// only through `do_move`'s `dest == 0` take, which this waypoint never
    /// goes through. So the sidestep is walked under the interrupted leg's
    /// tolerance — 384 — and fifteen units is arrival. This crate zeroed it
    /// with the push and had to walk the remainder.
    ///
    /// One deleted line. `1/6` parts at **7827** instead of 7805, its
    /// collision fields and its whole order record agree over the cycle,
    /// and [`LONG_WORD_EAST_INDIES`] moves 7806 → **7812** — where the
    /// remaining frame is `1/7`'s wait-versus-pause row, the second of
    /// run90's two refusals.
    #[test]
    fn run90_s_window_is_east_indies_shuffle() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run90-eastindies-shuffle.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run90.log"),
        ) else {
            eprintln!("skipping: no run90 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];

        // The window's own extent: 110 blocks and the `GameLog::end_game`
        // block the `!quit` writes, which is run88's shape one window
        // along.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs
                == (7_790..=7_899)
                    .chain(std::iter::once(7_916))
                    .collect::<Vec<_>>(),
            "run90's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );

        // **The original's own sidestep, straight off the dump, before any
        // comparison.** The path stack goes 4 → 5 → 4 across the
        // collision: the waypoint is pushed on the blocked block and gone
        // on the next, with the unit **not** on it. That is the whole
        // mechanism and it is one record's own three rows.
        // `(block, x, y, path length, top.x, top.y, top.flags)`.
        let sidestep: Vec<[i64; 7]> = states
            .iter()
            .filter(|f| (7_802..=7_806).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 6)?;
                let top = u.path.last()?;
                Some([
                    f.n,
                    u.pos.x,
                    u.pos.y,
                    u.path.len() as i64,
                    top.to.0,
                    top.to.1,
                    top.flags,
                ])
            })
            .collect();
        assert_eq!(
            sidestep,
            vec![
                [7_802, 39_750, 38_787, 4, 38_856, 38_232, 0],
                [7_803, 39_750, 38_787, 5, 39_720, 38_808, 2],
                [7_804, 39_729, 38_802, 4, 38_856, 38_232, 0],
                [7_805, 39_729, 38_802, 4, 38_856, 38_232, 0],
                [7_806, 39_708, 38_789, 4, 38_856, 38_232, 0],
            ],
            "the original's `1/6`: the sidestep waypoint (39720, 38808) is \
             pushed on the blocked block 7803 and **gone on 7804** with the \
             unit at (39729, 38802), which is not it — one step along the \
             bearing and the waypoint is abandoned, not walked to"
        );

        // **The first non-zero `MOVEORDER pause` on this disk.** Every one
        // of run88's 1,467 is 0 (`docs/COLLISION.md` §9), so the `% 9 + 1`
        // roll had never been witnessed as a value. `1/7` carries 3 on the
        // block it is first blocked and counts it down; the second roll,
        // an 8, is the one §8.5 could not attribute.
        //
        // **And the countdown is frozen while the unit is still
        // colliding**, which no reading had said: the 8 stands on eleven
        // blocks (7811..7821, where `collide` runs 1..9 and then clears)
        // and only then ticks down, one a block, to 0 on 7829. The rows
        // are folded to their transitions so the shape is readable.
        let pauses: Vec<(i64, i64)> = states
            .iter()
            .filter(|f| (7_800..=7_830).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 7)?;
                Some((f.n, u.orders.iter().find_map(|o| o.pause)?))
            })
            .collect();
        let mut runs: Vec<(i64, i64, i64)> = Vec::new();
        for &(n, v) in &pauses {
            match runs.last_mut() {
                Some(last) if last.2 == v => last.1 = n,
                _ => runs.push((n, n, v)),
            }
        }
        assert_eq!(
            runs,
            vec![
                (7_800, 7_802, 0),
                (7_803, 7_804, 3),
                (7_805, 7_805, 2),
                (7_806, 7_806, 1),
                (7_807, 7_810, 0),
                (7_811, 7_821, 8),
                (7_822, 7_822, 7),
                (7_823, 7_823, 6),
                (7_824, 7_824, 5),
                (7_825, 7_825, 4),
                (7_826, 7_826, 3),
                (7_827, 7_827, 2),
                (7_828, 7_828, 1),
                (7_829, 7_830, 0),
            ],
            "the original's `1/7` `pause` as (from, to, value): the 3 this \
             crate also rolls, and the **8** on 7811 that names the \
             `SITE_PAUSE` §8.5 could not attribute — held for eleven \
             blocks while `collide` runs, then ticked down"
        );

        // **`1/7` names a collider on twelve blocks and `resolve` runs on
        // two of them** (item 294) — and the claim is a *change*, because
        // `collide_frame` is a permanent stamp and a value test on it says
        // nothing. `resolve_unit_collision` step 5 writes `collide += 1`
        // and `collide_frame = frame` together, so the stamp moving is the
        // one dumped witness that the resolver ran. It moves on **7803**
        // and **7811** and nowhere else in the window.
        //
        // Block **7809** is the third naming, and it is the one this crate
        // did not make. `collide_o 6` is written with the stamp left at
        // 7802, `collide` left at 1, and the **position unchanged** from
        // 7808: a `detect_unit_collision` that ran and a resolver that did
        // not. That is `do_move`'s waypoint probe on its own, on a frame
        // `move_step` never reaches its own probe — it turns 44° in place
        // and returns at `005fb2e6` (`docs/COLLISION.md` §8.8). The
        // remaining nine namings, 7812..7820, are `do_move`'s
        // suspended-search block counting `collide` up with the stamp
        // untouched.
        let seven_rows: Vec<(i64, i64, i64, i64, i64, i64)> = states
            .iter()
            .filter(|f| (7_790..7_900).contains(&f.n))
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == 1 && u.o == 7)?;
                Some((
                    f.n,
                    u.pos.x,
                    u.pos.y,
                    u.collide?,
                    u.collide_o?,
                    u.collide_frame?,
                ))
            })
            .collect();
        let names: Vec<i64> = seven_rows
            .iter()
            .filter(|r| r.4 >= 0)
            .map(|r| r.0)
            .collect();
        assert_eq!(
            names,
            vec![
                7_803, 7_809, 7_811, 7_812, 7_813, 7_814, 7_815, 7_816, 7_817, 7_818, 7_819, 7_820
            ],
            "the blocks the original's `1/7` names a collider on"
        );
        let mut stamps: Vec<(i64, i64)> = Vec::new();
        let mut prev = seven_rows.first().map_or(0, |r| r.5);
        for r in &seven_rows {
            if r.5 != prev {
                stamps.push((r.0, r.5));
                prev = r.5;
            }
        }
        assert_eq!(
            stamps,
            vec![(7_803, 7_802), (7_811, 7_810)],
            "`collide_frame` moves on exactly the two blocks the resolver \
             ran — and 7809, which names a collider, is not one of them"
        );
        let at_seven = |n: i64| -> (i64, i64, i64, i64, i64, i64) {
            *seven_rows
                .iter()
                .find(|r| r.0 == n)
                .expect("run90 has the block")
        };
        assert_eq!(
            (at_seven(7_808), at_seven(7_809)),
            (
                (7_808, 39_672, 38_664, 1, -1, 7_802),
                (7_809, 39_672, 38_664, 1, 6, 7_802)
            ),
            "block 7809: the collider is named, the stamp and the counter \
             are untouched, and the unit has not moved"
        );

        // **And the block the word now sits on, from the original's side
        // alone.** The repath on 7810 leaves `1/7` with a **one-entry**
        // stack whose top is the move order's own goal, and the unit stands
        // on its snapped cell centre for ten blocks while the search that
        // was suspended is resumed — `docs/COLLISION.md` §8.8's successor.
        // This crate rewrites that goal and walks to the rewrite, which is
        // the 7811 order divergence and the 7812 position below.
        let standing: Vec<(i64, i64, i64, usize, i64, i64)> = seven_rows
            .iter()
            .filter(|r| (7_811..=7_820).contains(&r.0))
            .map(|r| {
                let f = states.iter().find(|f| f.n == r.0).expect("the block");
                let u = f
                    .units
                    .iter()
                    .find(|u| u.who == 1 && u.o == 7)
                    .expect("`1/7`");
                let top = u.path.last().expect("a one-entry stack");
                (r.0, r.1, r.2, u.path.len(), top.to.0, top.to.1)
            })
            .collect();
        assert_eq!(
            standing,
            (7_811..=7_820)
                .map(|n| (n, 39_672, 38_664, 1, 39_624, 38_760))
                .collect::<Vec<_>>(),
            "the original's `1/7` over 7811..7820: standing on its cell \
             centre with the move's own goal as its whole path stack"
        );

        // **And the field that says the search was suspended, whole
        // cast and whole window** (item 301). `UnitData::start_dist`
        // (`+0x130`) has exactly one writer in the executable —
        // `astar_path@00683770`'s suspend block — so a non-zero value is
        // the original telling us a 48-grid search stopped short of its
        // goal and handed its five containers to the unit
        // (`docs/PATHFINDER.md` §18). Over all 111 blocks and every unit
        // of each, it is non-zero for **`1/7` alone**, from block **7811**
        // — the block after the repath — to the end of the capture, and
        // its value is 144: exactly the Manhattan from `1/7`'s snapped
        // cell centre (39672, 38664) to its move order's own goal
        // (39624, 38760), which is the search `resolve_unit_collision`
        // step 6 started on frame 7810 and never finished.
        //
        // Nothing clears the field, so this is asserted as a **change**
        // rather than a value, the way 294 asserts `collide_frame`'s two
        // stamps: it is 0 for every unit on every block up to 7810 and
        // 144 for `1/7` from 7811 on.
        let dists: Vec<(i64, i64, i64, i64)> = states
            .iter()
            .flat_map(|f| {
                f.units
                    .iter()
                    .filter_map(move |u| Some((f.n, u.who, u.o, u.start_dist?)))
            })
            .filter(|r| r.3 != 0)
            .collect();
        let (first, last) = (dists.first().copied(), dists.last().copied());
        assert!(
            dists.iter().all(|r| (r.1, r.2, r.3) == (1, 7, 144))
                && first.map(|r| r.0) == Some(7_811)
                && last.map(|r| r.0) == Some(7_899)
                && dists.len() == 89,
            "run90's `start_dist`: {} rows, {first:?} .. {last:?} — the \
             suspend's only dumped witness is `1/7`'s, 144, from 7811",
            dists.len()
        );
        assert_eq!(
            (39_672 - 39_624) + (38_760 - 38_664),
            144,
            "and 144 is the Manhattan from `1/7`'s cell centre to the \
             move order's goal — the search that suspended"
        );

        // Now the comparison — every unit of every block.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();

        // **And `UnitData::start_dist`, on every unit of every block**
        // (item 308, `docs/PATHFINDER.md` §19). The field is written at
        // every detail level and nothing in this crate compared it until
        // 308 gave it a column of its own; with the suspend wired it
        // agrees over the whole window, so `1/7` stamps 144 on block 7811 and holds it to 7899,
        // the blocks and the values the original's own record carries.
        //
        // It would have been all disagreement a day ago — an unwired
        // suspend stamps nothing — so this is the check that the wiring
        // stays wired, and it is a value diff rather than a draw stream.
        let sd_compared: usize = report.frames.iter().map(|f| f.search_compared).sum();
        let sd_diverged: Vec<_> = report
            .frames
            .iter()
            .flat_map(|f| f.search_diverged.iter())
            .collect();
        assert!(
            sd_compared == 2_860 && sd_diverged.is_empty(),
            "run90's `start_dist` column: {sd_compared} compared, {} apart — \
             first {:?}",
            sd_diverged.len(),
            sd_diverged.first()
        );
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&7_790) && blocks.last() == Some(&7_916) && blocks.len() == 111,
            "run90's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        let ord_fields: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        assert!(
            ord_fields >= 300 && angles >= 300,
            "the window's own rows: {ord_fields} order fields and {angles} angles \
             — a capture below `UNITS=3` is the wrong file"
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        eprintln!(
            "run90: {} unit fields, {ord_fields} order/path fields, {angles} angles \
             over {} blocks; {} unit(s) ever off position",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            blocks.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **`1/6` does not part at all now, and that is the second
        // tightening of this row.** It was the window's first parted block
        // at 7805 — this crate on its own sidestep waypoint (39720, 38808)
        // where the original had already left it behind at (39729, 38802),
        // the row item 289's arrival rule was read from — and item 289 took
        // it to 7827, fifteen blocks past the word, where the unit turned
        // north-east onto a leg the original does not take. That remainder
        // was the unwired suspend: with item 301's block wired (item 304,
        // `docs/PATHFINDER.md` §18.5) `1/6` is on the original's point for
        // all 111 blocks. So the pin is emptiness, in every record the
        // window carries, and any reappearance fails.
        let six: Vec<(i64, i32, i32, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 6))
            .map(|d| (d.frame, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y))
            .collect();
        assert_eq!(
            six,
            vec![],
            "`1/6`'s parted blocks, both sides' coordinates — it walked the \
             whole shuffle at 7827 before the suspend wired"
        );

        // **The cycle, which is the mechanism and not the symptom.** The
        // original's blocked cycle is four blocks — block, step, turn,
        // step — and this crate's was five, because it walked the
        // remainder of the sidestep as a second step. Two records say the
        // cycle now matches, and neither is a position:
        //
        // - **the collision fields**, `collide`/`collide_o`/`collide_who`/
        //   `collide_guy`/`collide_frame`, on every block of the window:
        //   `1/6` never disagrees, so its `collide_o 7` falls on 7803,
        //   7807 and 7811 exactly as the dump's own rows above say;
        // - **the path stack and the move order**, likewise: `1/6`'s first
        //   order-record disagreement is 7828, one block after the
        //   position parts, so the stack goes 4 → 5 → 4 on the original's
        //   blocks and `dest_x/dest_y` name the original's points.
        //
        // A four-block cycle whose stack length and whose `collide_o` are
        // both the original's is the same cycle, and the five-block one
        // could not have produced either.
        //
        // Made to fail on purpose, and the two are not the same kind of
        // fence. The **order** row is the one that failed on the old code:
        // with the sidestep's tolerance zeroed again it reads 7804, the
        // block the original pops the waypoint and this crate does not, so
        // it dates the fault one block below even the position. The
        // **collision** row held before item 289 as well — the cycle's
        // `collide_o` was always the original's, which is exactly why the
        // fault took three sessions to find — so its teeth were shown the
        // other way, by pointing it at `1/7`, whose three rows on 7809 it
        // named at once.
        let six_coll: Vec<(i64, &str, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 6))
            .map(|d| (d.frame, d.field, d.ours, d.theirs))
            .collect();
        assert!(
            six_coll.is_empty(),
            "`1/6`'s collision fields part over the shuffle: {six_coll:?}"
        );
        let six_ord: Vec<i64> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 6))
            .map(|d| d.frame)
            .collect();
        assert_eq!(
            six_ord,
            Vec::<i64>::new(),
            "`1/6`'s order record — the path stack included — parts at \
             {:?}; it read 7828 while the suspend was unwired, one block \
             after the position, and the whole four-block cycle was below it",
            six_ord.first()
        );

        // **`1/7` does not part either, and it was East Indies' word.**
        // This was the second of run90's two refusals and the one item 289
        // did not touch: on 7809 the original had it colliding with `1/6`
        // (`collide 1`, `collide_o 6`) where this crate had it clear, two
        // blocks later its order kind parted — this crate taking §6 step 5's
        // wait where the original repaths and rolls a `pause 8` — and its
        // position went at 7812, which *was* [`LONG_WORD_EAST_INDIES`].
        // Item 301 read the mechanism (the original suspends the search
        // rather than walking the goal back onto the unit's own cell) and
        // banked it unwired; item 304 paid its Great Lakes cost, and with
        // the block wired `1/7` holds the original's point through the
        // window's last block and the word runs to 8193.
        let seven: Vec<(i64, i32, i32, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 7))
            .map(|d| (d.frame, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y))
            .collect();
        assert_eq!(
            seven,
            vec![],
            "`1/7`'s parted blocks, both sides' coordinates — it parted at \
             7812, the old word, before the suspend wired"
        );
        let seven_coll: Vec<(i64, &str, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 7))
            .map(|d| (d.frame, d.field, d.ours, d.theirs))
            .collect();
        assert_eq!(
            seven_coll,
            vec![(7_820, "collide_o", -1, 6), (7_820, "collide_who", -1, 1),],
            "`1/7`'s whole collision divergence over run90"
        );

        // **And the whole cast's, over the whole window** — every unit of
        // every block, all five fields. It was `1/7`'s three rows on 7809,
        // item 289 emptied it, and wiring the suspend put **two rows back
        // on 7820**: the original names the blocker there
        // (`collide_who 1`, `collide_o 6`) and this crate leaves both at
        // −1 while `collide` itself, the count, agrees. That is a residue
        // and not a fault the window can price — every position, every
        // angle, every order record and every draw count in these 111
        // blocks agrees, so nothing downstream of it is visible here. It
        // is pinned rather than scored, and it is the one row run90 has
        // left to explain.
        let all_coll: Vec<(i64, i64, i64, &str, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter())
            .map(|d| (d.frame, d.who, d.o, d.field, d.ours, d.theirs))
            .collect();
        assert_eq!(
            all_coll,
            vec![
                (7_820, 1, 7, "collide_o", -1, 6),
                (7_820, 1, 7, "collide_who", -1, 1),
            ],
            "run90's collision fields, whole cast and whole window"
        );

        // **And `1/7`'s order record holds too, where it parted at 7811**
        // — one block below the old word, and never a collision fault (item
        // 294). The original's repath on 7810 leaves the stack as the
        // move's own goal and the search **suspended**; this crate's
        // `find_upath` used to walk that goal back onto the unit's own cell
        // and return it as a one-entry final leg, which the unit reached on
        // 7812. The suspended-search block is what makes the stack the
        // original's, so this row is the wiring's own assertion: it fails
        // the moment the block stops holding.
        let seven_ord: Vec<i64> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 7))
            .map(|d| d.frame)
            .collect();
        assert_eq!(
            seven_ord,
            Vec::<i64>::new(),
            "`1/7`'s order record parts over run90: {seven_ord:?}"
        );

        // **The whole parted set is now the two Merchant constants, and
        // nothing else.** `1/19` and `1/20` are the standing rows run88
        // already carries (`docs/MERCHANT.md`), both open on the window's
        // first block as they did on run88's. Everything that used to sit
        // beside them went with the suspend: `1/6` at 7827 and `1/7` at
        // 7812, and the three that were downstream wake rather than faults
        // — `(0, 4)` 7887, `(1, 2)` 7872 and `(1, 5)` 7895 — which is what
        // "the shuffle's wake" always predicted and nothing had shown.
        // 111 blocks of the East Indies shuffle, every unit of every
        // block, and two rows apart.
        assert_eq!(
            parted
                .iter()
                .map(|(&k, &f)| (k, f))
                .collect::<Vec<((i64, i64), i64)>>(),
            vec![((1, 19), 7_790), ((1, 20), 7_790)],
            "run90's parted set, whole"
        );

        // **And the draw counts, block for block, up to the word.** The
        // stream agrees through 7805 and parts at
        // [`LONG_WORD_EAST_INDIES`], which is the two frames the position
        // is already wrong for.
        let apart: Vec<i64> = report
            .rng_frames
            .iter()
            .filter(|(n, _, _)| (7_790..7_900).contains(n))
            .filter(|(_, ours, theirs)| ours != theirs)
            .map(|(n, _, _)| *n)
            .collect();
        assert!(
            apart.is_empty(),
            "run90's per-block draw counts part: {:?}",
            apart.first()
        );

        // **The trace and the dump, tied to each other on the same
        // frames.** This is the corroboration §8.5 asked for and could not
        // have: the original's [`sim::anim::SITE_BLOCKED`] falls on
        // sim-frames 7802, 7806 and 7810 — the three whose *blocks* (7803,
        // 7807, 7811) are exactly the ones whose `UNITDATA` carries
        // `collide_o 7`, three blocks apart. A torn dump could fake one
        // row; it cannot fake three that line up with an instrument that
        // is not a dump.
        let stands: Vec<i64> = (7_795..7_815)
            .filter(|&n| tr.labels(n).iter().any(|l| l == sim::anim::SITE_BLOCKED))
            .collect();
        assert_eq!(
            stands,
            vec![7_802, 7_806, 7_810],
            "the original's blocked stands over the shuffle, off run90's own trace"
        );
        let flagged: Vec<i64> = states
            .iter()
            .filter(|f| (7_796..7_816).contains(&f.n))
            .filter(|f| {
                f.units
                    .iter()
                    .any(|u| (u.who, u.o) == (1, 6) && u.collide_o == Some(7))
            })
            .map(|f| f.n)
            .collect();
        assert_eq!(
            flagged,
            stands.iter().map(|n| n + 1).collect::<Vec<_>>(),
            "the blocks whose `1/6` names `collide_o 7` are the stands' own \
             blocks — the trace and the dump on the same three frames"
        );
        // And the three are four apart — the cycle length, off the
        // instrument that is not a dump. This crate's own three were five
        // apart (7802, 7807, 7812) until item 289; the `six_coll` and
        // `six_ord` assertions above are the same claim from the dump's
        // side, and both are needed because only the trace dates a *draw*.
        assert!(
            stands.windows(2).all(|w| w[1] - w[0] == 4),
            "the original's cycle is four frames: {stands:?}"
        );
    }

    /// **run86 — the transport ride, whole, and the field the word was
    /// short of.**
    ///
    /// East Indies' word stood at 7448 for three sessions: `1/20`, the
    /// AI's Merchant, reaches the animal it is blocked by 792 units late
    /// (run85). The 792 is made between run82's 6929 and run85's 7400,
    /// and this capture is the **whole** of that gap — 486 blocks at
    /// run85's own detail, butting against both neighbours (six blocks
    /// over run82, ten over run85, `differ: 0` on each).
    ///
    /// **It is one field.** The Transport Barge `1/22` is born on 7094
    /// and prints `myspeed` **30** on a type whose `MOVES` is 25: the
    /// Whales rare's `+WHALES_SHIPS_MOVE%`, which this crate has applied
    /// since run63 to every unit born through [`sim::Sim::spawn_unit`]
    /// and did **not** apply in [`sim::Sim::cast_transport`], the one
    /// place a boat is born. 190 frames of 25 against 30 is the lag; the
    /// passenger rides it ashore and never catches up.
    ///
    /// And it settles `docs/TRANSPORT.md` §13's second row on the way
    /// past: **a passenger's position while aboard is frozen at the
    /// boarding point**. `1/20` prints (34530, 32268) on all 191 blocks
    /// from 7093 to 7283 and is put down at (30408, 28152) on 7284 —
    /// which is what [`sim::Sim::board`] does, believed harmless and now
    /// witnessed.
    #[test]
    fn run86_s_window_is_the_transport_ride() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run86-eastindies-transportride.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run86.log"),
        ) else {
            eprintln!("skipping: no run86 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];

        // The window's own facts, off the disk, before this crate runs.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs.first() == Some(&6_924) && theirs.contains(&7_409) && theirs.len() >= 486,
            "run86's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );
        let at = |n: i64, o: i64| -> Option<&crate::gamelog::UnitDump> {
            states
                .iter()
                .find(|f| f.n == n)?
                .units
                .iter()
                .find(|u| u.who == 1 && u.o == o)
        };

        // **The barge, and the field.** Born 7094, `myspeed` 30 — and the
        // type it is born from carries `MOVES` 25, so the dump itself is
        // the evidence for the bonus rather than a reading of it.
        let boat = at(7_094, 22).expect("run86 block 7094 has the barge `1/22`");
        assert_eq!(
            (boat.pos.x, boat.pos.y, boat.myspeed),
            (34_653, 32_075, Some(30)),
            "the barge's birth row"
        );
        assert!(
            at(7_093, 22).is_none(),
            "7093 is the frame before the barge exists"
        );
        let boat_ty = loaded
            .unit_types
            .iter()
            .position(|t| t.tree == Some(sim::transport::ty::TRANSPORTBARGE))
            .expect("the install has a Transport Barge");
        assert_eq!(
            loaded.unit_types[boat_ty].moves, 25,
            "`MOVES` is 25 and the dump says 30 — the difference is the rare"
        );

        // **The passenger, frozen at the boarding point** (§13's second
        // row). 7093 is the last frame it walks; 7284 is the ring.
        let held: Vec<(i64, i64)> = (7_093..=7_283)
            .filter_map(|n| at(n, 20).map(|u| (u.pos.x, u.pos.y)))
            .collect();
        assert!(
            held.len() == 191 && held.iter().all(|&p| p == (34_530, 32_268)),
            "`1/20` is frozen at its boarding point for the ride: {} blocks, {:?}",
            held.len(),
            held.first()
        );
        assert_eq!(
            at(7_284, 20).map(|u| (u.pos.x, u.pos.y)),
            Some((30_408, 28_152)),
            "`come_out`'s ring puts the Merchant ashore on 7284"
        );
        assert_eq!(
            at(7_283, 22).map(|u| (u.pos.x, u.pos.y)),
            Some((30_521, 28_243)),
            "the boat's last position, which the ring is measured from"
        );
        assert!(
            at(7_284, 22).is_none(),
            "the barge dies on the frame it unloads"
        );

        // Now the comparison, over the whole window.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&6_924) && blocks.last() == Some(&7_426) && blocks.len() == 487,
            "run86's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        // This one quit *into* its frame block rather than after it, so
        // its state is 7426's own child and [`Log::final_state`] has
        // nothing to add — the other shape, and the reason that reader
        // returns an option (`run82_s_window_is_the_east_indies_ride_s_run_up`).
        assert!(
            log.final_state().is_none(),
            "run86's shutdown dump is inside FRAME 7426, not after it"
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        eprintln!(
            "run86: {} unit fields, {} order/path fields, {} angles over {} blocks; parted {:?}",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            report
                .frames
                .iter()
                .map(|f| f.order_compared)
                .sum::<usize>(),
            report
                .frames
                .iter()
                .map(|f| f.angle_compared)
                .sum::<usize>(),
            blocks.len(),
            parted
        );
        assert!(
            report
                .frames
                .iter()
                .map(|f| f.order_compared)
                .sum::<usize>()
                >= 10_000,
            "run86 is a `UNITS=3` capture — a file without orders is the wrong one"
        );
        assert!(
            report.frames.iter().all(|f| f.unlinked_units.is_empty()),
            "every unit of every block links, the barge included"
        );

        // **The ride is exact, both halves of it.** Neither the boat nor
        // its passenger is ever off the original's position — through the
        // cast, 190 frames of open water, and the ring it is put down on.
        // Before the fix `1/22` parted on its **birth frame** and was 933
        // units adrift by 7283, and `1/20` was 4,122 out the moment it
        // came ashore.
        assert_eq!(
            parted,
            [((1, 19), 6_924)].into_iter().collect(),
            "run86's parted set: the ride is not in it, and neither is the age"
        );

        // **And the age, which is the rest of this window** (item 271,
        // `docs/TECH.md`, "An age snaps every figure"). `1/13` parted at
        // 6938 until 2026-09-07 and the cause was one field on the block
        // before: the **only** angle disagreement in all 487 blocks was
        // its facing on 6937, where this crate turned it 7.5° and the
        // original put it on its heading outright.
        let angles: Vec<(i64, i64, i64, i32, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter())
            .map(|d| (d.frame, d.who, d.o, d.ours, d.theirs))
            .collect();
        assert!(
            angles.is_empty(),
            "run86's angle disagreements — 6937 was `1/13`'s and the age              answers it: {angles:?}"
        );

        // The block itself, off the disk: player 1's buildings take the
        // age, and **every one of its 22 units** has its figure's `last_x`
        // on its own `x` — the `Guy::set_new_location(…, 1)` the leader's
        // loop hands each of them. Its neighbours have only the standing
        // ones, and no other player moves.
        let block = |n: i64| states.iter().find(|f| f.n == n).expect("the block");
        // A figure whose `last_x`/`last_y` are its **own** position was
        // *placed* rather than stepped: `Guy::move` writes them from the
        // pre-step point, and only `Guy::set_new_location(..., 1)` writes
        // them from the point it is putting the figure on.
        let placed = |n: i64, who: i64| -> (usize, usize) {
            let live: Vec<&crate::gamelog::UnitDump> =
                block(n).units.iter().filter(|u| u.who == who).collect();
            let placed = live
                .iter()
                .filter(|u| {
                    u.guys
                        .first()
                        .and_then(|g| g.last_pos)
                        .is_some_and(|l| (l.x, l.y) == (u.pos.x, u.pos.y))
                })
                .count();
            (placed, live.len())
        };
        assert_eq!(
            [6_936_i64, 6_937, 6_938].map(|n| placed(n, 1)),
            [(14, 22), (22, 22), (15, 22)],
            "player 1's figures over the age: all 22 are placed on 6937, and \
             only the standing ones — fourteen and fifteen — on either side"
        );
        assert_eq!(
            [6_936_i64, 6_937, 6_938].map(|n| placed(n, 0)),
            [(5, 6), (5, 6), (5, 6)],
            "player 0's are untouched across the block — an age is one \
             leader's own loop, and the same five of its six read as placed \
             on all three"
        );
        let aged = |n: i64| -> (usize, usize) {
            let mine: Vec<&crate::gamelog::BuildDump> =
                block(n).builds.iter().filter(|b| b.who == 1).collect();
            (
                mine.iter().filter(|b| b.max_age == Some(1)).count(),
                mine.len(),
            )
        };
        assert_eq!(
            [6_936_i64, 6_937].map(aged),
            [(0, 14), (14, 14)],
            "the Classical Age lands on block 6937 — `max_age` 0 -> 1 on \
             every one of player 1's buildings, which is what dates it"
        );
    }

    /// **run82's own window had never been compared, and its shutdown
    /// dump reaches seventeen frames further than anyone had read.**
    ///
    /// Item 241 booked a capture over the transport ride on the sentence
    /// "nothing on disk covers 6930–7399". The disk was re-grepped first
    /// and one thing does: `!quit` left run82 a shutdown dump labelled
    /// **6946**, inside the band, holding all 28 player units — and
    /// nothing had ever parsed it, on this capture or any other
    /// ([`Log::final_state`]). So this test is both halves: the window
    /// [6860, 6930) that had no diff at all, and the block past it.
    ///
    /// **What it settles for 241: the 792 is born in the ride.** `1/20`
    /// is on the original's own position for all seventy blocks *and* at
    /// 6946, so the lag run85 opens with at 7400 is not a run-up that
    /// drifted — every unit of it is made after 6946, in the cast, the
    /// barge's own plan or the ring it is put ashore on
    /// (`docs/TRANSPORT.md` §13).
    #[test]
    fn run82_s_window_is_the_east_indies_ride_s_run_up() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run82-islands-merchantcast.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run82.log"),
        ) else {
            eprintln!("skipping: no run82 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];

        // The window's own shape, off the disk. 70 blocks, and then the
        // two the `!quit` writes — 6945 and 6946 — which carry no records
        // of their own because the state lands *after* them.
        let states = log.frame_states();
        let theirs: Vec<i64> = states.iter().map(|f| f.n).collect();
        assert!(
            theirs == (6_860..=6_929).chain([6_945, 6_946]).collect::<Vec<_>>(),
            "run82's blocks: {:?}..{:?} ({}) — the wrong file",
            theirs.first(),
            theirs.last(),
            theirs.len()
        );
        assert!(
            states
                .iter()
                .filter(|f| f.n >= 6_945)
                .all(|f| f.units.is_empty()),
            "the quit frames are empty — the dump is their sibling"
        );

        // **The shutdown dump, which is the whole point.** The same 28
        // player units the window carries, plus gaia's 104 animals, plus
        // the buildings and the leaders — a whole-map state at 6946.
        let fin = log.final_state().expect("run82 quit, so it wrote one");
        assert_eq!(
            (
                fin.n,
                fin.units.len(),
                fin.builds.len(),
                fin.leaders.len(),
                fin.cities.len()
            ),
            (6_946, 132, 21, 4, 3),
            "run82's shutdown dump"
        );

        // The window first: the whole record, block by block.
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let blocks: Vec<i64> = report
            .frames
            .iter()
            .filter(|f| f.compared > 0)
            .map(|f| f.frame)
            .collect();
        assert!(
            blocks.first() == Some(&6_860) && blocks.last() == Some(&6_929) && blocks.len() == 70,
            "run82's compared blocks: {:?}..{:?} ({})",
            blocks.first(),
            blocks.last(),
            blocks.len()
        );
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        eprintln!(
            "run82: {} unit fields, {} order/path fields, {} angles over {} blocks; parted {:?}",
            report.frames.iter().map(|f| f.compared).sum::<usize>(),
            report
                .frames
                .iter()
                .map(|f| f.order_compared)
                .sum::<usize>(),
            report
                .frames
                .iter()
                .map(|f| f.angle_compared)
                .sum::<usize>(),
            blocks.len(),
            parted
        );
        assert!(
            report
                .frames
                .iter()
                .map(|f| f.order_compared)
                .sum::<usize>()
                >= 1_000,
            "run82 is a `UNITS=3` capture — a file without orders is the wrong one"
        );

        // **One unit parts in the whole window, and it is the unpack.**
        // `1/19` is the Merchant `cast_unpack` teleports onto the tile
        // corner on 6883 (`docs/RUNS.md`, "run82"); this crate leaves it
        // on the half-tile it walked to, so the gap is a constant (24, 24)
        // from that frame to the last block.
        assert_eq!(
            parted,
            [((1, 19), 6_883)].into_iter().collect(),
            "run82's parted set"
        );
        let merchant: Vec<(i64, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .map(|d| (d.frame, d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            merchant.len() == 47
                && merchant.first().map(|d| d.0) == Some(6_883)
                && merchant.iter().all(|d| (d.1, d.2) == (24, 24)),
            "`1/19` is a constant (24, 24) from 6883: {} rows, {:?}",
            merchant.len(),
            merchant.first()
        );

        // **And now the block nobody had read.** Step the same simulation
        // seventeen frames past the window's end and compare the shutdown
        // dump whole.
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..fin.n - 1 {
            built.tick();
        }
        // Stepping to 6945 and letting [`compare_shutdown`] take the last
        // frame is what separates the dump's **own** one-tick tear from a
        // divergence that is ours (`docs/ORACLE.md`, "The shutdown dump").
        // When this test first landed it did not, and `0/3` was booked as a
        // gap of (-18, 18) that this crate did not have: the closing dump
        // simply holds that unit on its 6945 position, which is where the
        // simulation had it a tick before.
        let end = compare_shutdown(&mut built, &fin, 8);
        let off = end.off.clone();
        eprintln!(
            "run82 6946: {} compared, {} unlinked, torn {:?}, off {off:?}",
            end.compared,
            end.unlinked.len(),
            end.torn
        );
        assert_eq!(
            (end.compared, end.unlinked.len()),
            (28, 0),
            "the shutdown dump's 28 player units all link"
        );
        assert_eq!(
            end.torn,
            vec![(0, 3)],
            "`0/3` is the closing dump's tear, not a divergence"
        );

        // **`1/20` is exact at 6946 — the whole of item 241 in one row.**
        // run85 opens at 7400 with it 792 short (`docs/TRANSPORT.md`
        // §13); nothing of that lag exists here, so the transport ride
        // between the two owns all of it.
        assert!(
            !off.contains_key(&(1, 20)),
            "`1/20` is on the original's own position at 6946: {off:?}"
        );

        // The rest of the block, whole: the unpack's constant, and **one
        // unit that parts inside the sixteen frames after the window** —
        // `1/13`, which run86 also has parting at 6938 (item 253). It is
        // off at 6945 too, by (33, 19), so it is not the dump's tear.
        assert_eq!(
            off,
            [((1, 19), (24, 24))].into_iter().collect(),
            "run82's shutdown dump, every unit of it — `1/13`'s (11, 7) went              with the age snap (item 271, `docs/TECH.md`)"
        );
    }
}
