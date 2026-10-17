//! The harness itself: one frame compared, a whole log run, and the traced runs.

mod checkpoint;
#[cfg(test)]
pub(crate) mod prefix;
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
        compared::note("UnitDump", &["who", "o", "pos"]);
        // `ObjectData::mylos`, which the object record writes at every
        // detail level — `docs/VISION.md` §2. A garrisoned unit is skipped:
        // `update_los` is not run for one, so the field is whatever it held
        // when the unit was last on the map.
        if let Some(theirs_los) = u.mylos
            && built.sim.units[link.unit].on_map
        {
            r.los_compared += 1;
            compared::note("UnitDump", &["mylos"]);
            let ours_los = built.sim.units[link.unit].mylos;
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
            compared::note("UnitDump", &["unit_masks"]);
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
        // **`ObjectData::visible`** beside them (`docs/VISION.md` §7): the
        // byte of players this unit has made itself visible to by
        // attacking them, which is `UnitData::is_seen`'s fallback arm and
        // therefore decides who may legally target it through fog. Printed
        // inside the `OBJECT` block at every detail level, so this runs on
        // every capture that carries a unit record.
        //
        // A garrisoned unit is skipped for the reason `mylos` is: neither
        // `set_attacking` nor `work`'s clear runs for one, so the field
        // holds whatever it held when the unit was last on the map and the
        // two sides' histories of that are not comparable.
        if let Some(theirs_vis) = u.visible
            && built.sim.units[link.unit].on_map
        {
            r.visible_compared += 1;
            compared::note("UnitDump", &["visible"]);
            let ours_vis = built.sim.units[link.unit].visible;
            if i64::from(ours_vis) != theirs_vis {
                r.visible_diverged.push(VisibleDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_vis,
                    theirs: theirs_vis,
                });
            }
        }
        // **The outermost container** (parked 598, item 620): the dump
        // chains a garrison through units — a seated Scholar's
        // `inside_up` is the Scholar before it — so both sides are
        // compared on the top of the chain, `ObjectData::get_inside`'s
        // walk. The chain is followed within the unit's own player
        // (`inside_up_who` is not parsed). This crate keeps a building in
        // `inside` and a boat in `inside_unit`. Item 592 wrote this walk
        // in run143's widening; it is the comparator's now, so every
        // capture reads it.
        if let Some(first) = u.inside_up {
            let mut up = first;
            while let Some(next) = frame
                .units
                .iter()
                .find(|x| x.who == u.who && x.o == up)
                .and_then(|x| x.inside_up)
                .filter(|&v| v >= 0)
            {
                up = next;
            }
            let x = &built.sim.units[link.unit];
            let ours = x
                .inside
                .map(|b| i64::from(built.sim.buildings[b].index))
                .or(x.inside_unit.map(|b| i64::from(built.sim.units[b].index)))
                .unwrap_or(-1);
            r.inside_compared += 1;
            compared::note("UnitDump", &["inside_up"]);
            if up >= 0 {
                r.inside_housed += 1;
            }
            if ours != up {
                r.inside_diverged.push(InsideDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours,
                    theirs: up,
                });
            }
        }
        // **The hit-point record, whole** (`docs/COMBAT.md` §40) —
        // `ObjectData::myhits`, `damage` and `damage_frac`, printed
        // inside the `OBJECT` block at every detail level and compared
        // on none of them until item 484. Chapter two's widening could
        // see that the original's hoplite `1/8` was gone on block 684
        // and not that it had been wounded on 656, 657, 660 and 682 to
        // get there (§38.4), because a `compare` with no hit-point row
        // reads a death as an absence.
        //
        // `myhits` is the **squad's** hits and `damage` this **figure's**
        // share of what has been taken off them (§7.3: `update_hits`
        // writes one number onto every figure and `take_damage` divides
        // it on the way in). This crate keeps the complement — `health`
        // is the squad-sized number with this figure's damage already
        // subtracted — so `damage` is `max_health − health` here and the
        // two representations line up field for field.
        //
        // **`hits_left` is deliberately not a fourth row.** It is
        // `myhits − damage` on both sides, so it parts exactly when one
        // of the two above does and would print every disagreement
        // twice; `run100_s_word_block_is_every_record_the_dump_carries`
        // reads it that way because it has no `damage` row of its own.
        //
        // Ungated on the position, like `visible` and unlike the
        // collision block: what a unit has taken is not a consequence of
        // where it is standing. Ungated on `on_map` too — a garrisoned
        // unit is healed by its building on both sides, so the pair
        // stays comparable where `mylos` and `visible` do not.
        {
            let un = &built.sim.units[link.unit];
            for (field, mine, logged) in [
                ("myhits", i64::from(un.max_health), u.myhits),
                ("damage", i64::from(un.max_health - un.health), u.damage),
                ("damage_frac", i64::from(un.damage_frac), u.damage_frac),
                // **`hold_frames`**, the fourth `ObjectData` field on the
                // same block and the last one `coverage`'s pin carried
                // as unread (item 491, §42.3). Every writer is on a dead
                // object, so what this asserts is that the dump shows a
                // **living** unit a zero — and that is the check §9.2's
                // sentence about the bump never had. It is here rather
                // than in the firing record because it is written by
                // `Object::die` and read by the slot walk, not by the
                // shot.
                ("hold_frames", i64::from(un.hold_frames), u.hold_frames),
            ] {
                let Some(theirs) = logged else { continue };
                r.hits_compared += 1;
                compared::note("UnitDump", &[field]);
                if theirs != mine {
                    r.hits_diverged.push(HitsDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field,
                        ours: mine,
                        theirs,
                    });
                }
            }
        }
        // **The firing record, whole** (`docs/COMBAT.md` §41) —
        // `UnitData::recharging` and the overkill window beside it,
        // written at `UNITDATA`'s own indent on every unit of every block
        // of every capture and compared on none of them until item 485.
        //
        // Item 484's hit-point row could see that this crate's hoplite
        // `1/8` ran one arrival behind the dump's from its first wound
        // (§40.4) and had to infer the frame of each hit from the
        // accumulator that moved. The dump dates its hits itself:
        // `damage_frame` is the sim frame `Object::do_damage` step 2
        // stamped, in the same numbering this crate's `tick` passes, and
        // `recharging` is the reload clock `Unit::fight` sets on the
        // frame it swings. Between them they say *when a shot was fired*
        // and *when it landed*, which is the whole of what a one-frame
        // lag is about.
        //
        // `damage_o`/`damage_who` are compared only where **both** sides
        // hold a live window. The original's never-hit encoding is
        // `damage_o -1, damage_who 0` and this crate's is `0, 0`, so an
        // untouched unit would part on the representation; and on the
        // frames where one side has a record and the other does not, the
        // fact is `damage_frame`'s to report, not theirs twice over.
        {
            let un = &built.sim.units[link.unit];
            let live = u.damage_frame.is_some_and(|f| f != 0) && un.combat.damage_frame != 0;
            let rows: [(&'static str, i64, Option<i64>); 5] = [
                ("recharging", i64::from(un.combat.recharging), u.recharging),
                // **`unit_masks2 & 0x10`** — the frozen frame's mark
                // (item 502, `docs/COMBAT.md` §43.2). The word is printed
                // inside `UNITDATA` at every detail level and was parsed
                // and compared nowhere; it is what says a unit reached
                // `Unit::fight`'s "still ordered, reload open, not firing
                // this frame" arm on the frame the dump was taken.
                //
                // **Only the bit is compared, and the reason is measured
                // rather than assumed.** Across every dump on this disk
                // the word takes four values — `0` (5.5 M unit-frames),
                // `0x40000` (731, run16 and run16b, `process_supply`'s
                // display flag), `4` (688, the East Indies captures
                // including run99's own value window) and `0x10` (148).
                // Neither of the other two bits has a writer here, so
                // comparing the whole word would pin a defect this
                // landing is not fixing on the **other** headline's
                // capture. On both headline windows the mask is the whole
                // field: run112 carries `0x10` on seven unit-frames and
                // zero everywhere else, and run100 carries zero
                // throughout (item 497). Widening this to the word is
                // what `0x40000` and `4` owe.
                (
                    "unit_masks2",
                    i64::from(un.unit_masks2 & sim::combat::umask2::NOT_FIRING),
                    u.unit_masks2
                        .map(|m| m & i64::from(sim::combat::umask2::NOT_FIRING)),
                ),
                ("damage_frame", un.combat.damage_frame, u.damage_frame),
                (
                    "damage_o",
                    i64::from(un.combat.damage_o),
                    u.damage_o.filter(|_| live),
                ),
                (
                    "damage_who",
                    i64::from(un.combat.damage_who),
                    u.damage_who.filter(|_| live),
                ),
            ];
            // **The squad chain, `o_up` and `o_down`** (item 523,
            // `docs/COMBAT.md` §47). Printed at the same indent on every
            // unit of every block, and compared nowhere until this row:
            // `o_up` was parsed for the stand-up alone (`diff::army`), and
            // that one read kept it off both `coverage`'s pin and the
            // ledger's uncompared list. `UnitData::is_captain` is `o_up <
            // 0`, and it gates `Unit::fight`'s one-in-five roll. When
            // run112's `1/6` died on 743 the original handed its squad to
            // `1/7`, and this crate did not. The draw that went missing
            // was nineteen frames later, on 762, and the record it read
            // had parted on 744 with nothing looking at it.
            //
            // A link is compared as the dump prints it, the linked
            // figure's own `o` or −1, and a dead slot is still a link:
            // `Unit::close` re-appends the dead figure at its chain's
            // tail, so a live captain's `o_down` can name a figure the
            // dump no longer carries.
            let link_o = |l: Option<usize>| l.map_or(-1, |x| i64::from(built.sim.units[x].index));
            let chain: [(&'static str, i64, Option<i64>); 2] = [
                ("o_up", link_o(un.o_up), u.o_up),
                ("o_down", link_o(un.o_down), u.o_down),
            ];
            for (field, mine, logged) in rows.into_iter().chain(chain) {
                let Some(theirs) = logged else { continue };
                r.firing_compared += 1;
                compared::note("UnitDump", &[field]);
                if theirs != mine {
                    r.firing_diverged.push(FiringDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field,
                        ours: mine,
                        theirs,
                    });
                }
            }
        }
        // **The collision block**, field for field
        // (`docs/COLLISION.md` §8). `UnitData::log_data` writes all five at
        // every detail level, so this is checked on every capture — and
        // the whole record is compared, not the field the mechanic happens
        // to care about. **Counted** only on unit-frames whose *positions*
        // agree: a unit that has walked somewhere else collides with
        // different things as a consequence, and counting that would
        // measure the position gap twice. On the frames the position
        // parts the same comparison is **filed** in `collide_parted`
        // instead (parked 453): a gate that stops looking on the frame a
        // position parts is blind on exactly the word's frame.
        let on_map = built.sim.units[link.unit].on_map;
        let agree = ours == theirs;
        if on_map {
            let un = &built.sim.units[link.unit];
            let mut compared = 0usize;
            let mut parts: Vec<CollideDivergence> = Vec::new();
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
                compared += 1;
                // `half_step` is a bit of `unit_masks`, the record's field.
                compared::note(
                    "UnitDump",
                    &[if field == "half_step" {
                        "unit_masks"
                    } else {
                        field
                    }],
                );
                if theirs != mine {
                    parts.push(CollideDivergence {
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
                compared += 1;
                compared::note("UnitDump", &["collide_frame"]);
                if theirs != un.collide_frame {
                    parts.push(CollideDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field: "collide_frame",
                        ours: un.collide_frame,
                        theirs,
                    });
                }
            }
            if agree {
                r.collide_compared += compared;
                r.collide_diverged.extend(parts);
            } else {
                r.collide_parted.extend(parts);
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
        // Counted on the positions agreeing for the same reason the
        // collision block is, and filed in `search_parted` otherwise.
        if on_map && let Some(theirs_dist) = u.start_dist {
            let ours_dist = i64::from(built.sim.units[link.unit].start_dist);
            if agree {
                r.search_compared += 1;
            }
            compared::note("UnitDump", &["start_dist"]);
            if ours_dist != theirs_dist {
                let d = SearchDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_dist,
                    theirs: theirs_dist,
                };
                if agree {
                    r.search_diverged.push(d);
                } else {
                    r.search_parted.push(d);
                }
            }
        }
        // The two angles, each against its own field — **counted on the
        // unit-frames where the two sides still agree on the position**. A
        // unit that has walked somewhere else is facing somewhere else as a
        // consequence, and counting that would measure the position gap
        // twice over; what is wanted here is the turn model on its own. On
        // the frames the position parts they are filed in `angle_parted`.
        // A garrisoned unit is skipped for the same reason `mylos` is:
        // nothing turns it.
        if on_map {
            let m = built.sim.units[link.unit].movement;
            let mut compared = 0usize;
            let mut parts: Vec<AngleDivergence> = Vec::new();
            let mut angle = |which: Which, ours: i32, theirs: i64| {
                compared += 1;
                if i64::from(ours) != theirs {
                    parts.push(AngleDivergence {
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
                compared::note("UnitDump", &["angle"]);
            }
            if let Some(theirs) = u.guys.first().and_then(|g| g.angle) {
                angle(Which::Facing, m.facing.0, theirs);
                compared::note("Guy", &["angle"]);
            }
            if agree {
                r.angle_compared += compared;
                r.angle_diverged.extend(parts);
            } else {
                r.angle_parted.extend(parts);
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
    // **The death-object list, both directions and field for field**
    // (`docs/COMBAT.md` §42.1). `DEATH_OBJS` is written at `DEATHS=1` and
    // above, which is every capture this crate diffs but the golden pool
    // captures (run210, run215: `DEATHS` off under `[End Frame]`, where
    // the `extra` direction below compares this crate's corpse against a
    // record nobody asked for, item 731), and no reader had
    // ever opened one: item 485 read a `first_frame` out of the raw text
    // by hand and `cur_anim 17` stood in three documents as a reading.
    //
    // Linked on `(who, o)` and never on the list index — a dumped
    // record's slot index is not an identity, and the original's list is
    // an `ObjectArray` whose slots are recycled. A record on one side and
    // not the other is a row of its own (`missing`/`extra`) rather than a
    // silent skip, which is the same rule `extra_units` above follows.
    //
    // `gpiece` is parsed and **not** compared: it is the dead unit's own
    // death piece, `DeathObj::init`'s `vtable[0x178](param_7, packed)`,
    // and this crate loads no art for it (§42.5).
    {
        let ours: Vec<&sim::combat::Death> = built.sim.deaths.iter().collect();
        for d in &frame.deaths {
            let (Some(who), Some(o)) = (d.who, d.o) else {
                continue;
            };
            if !(0..players as i64).contains(&who) {
                continue;
            }
            compared::note("DeathDump", &["who", "o"]);
            let Some(mine) = ours
                .iter()
                .find(|m| i64::from(m.who) == who && i64::from(m.o) == o)
            else {
                r.death_compared += 1;
                r.death_diverged.push(DeathDivergence {
                    frame: frame.n,
                    who,
                    o,
                    field: "missing",
                    ours: 0,
                    theirs: 1,
                });
                continue;
            };
            for (field, m, logged) in [
                ("valid", 1, d.valid),
                ("first_frame", mine.first_frame, d.first_frame),
                ("cur_anim", i64::from(mine.cur_anim), d.cur_anim),
            ] {
                let Some(theirs) = logged else { continue };
                r.death_compared += 1;
                compared::note("DeathDump", &[field]);
                if theirs != m {
                    r.death_diverged.push(DeathDivergence {
                        frame: frame.n,
                        who,
                        o,
                        field,
                        ours: m,
                        theirs,
                    });
                }
            }
        }
        if !frame.units.is_empty() {
            for m in &ours {
                let (who, o) = (i64::from(m.who), i64::from(m.o));
                if !(0..players as i64).contains(&who) {
                    continue;
                }
                if !frame
                    .deaths
                    .iter()
                    .any(|d| d.who == Some(who) && d.o == Some(o))
                {
                    r.death_compared += 1;
                    r.death_diverged.push(DeathDivergence {
                        frame: frame.n,
                        who,
                        o,
                        field: "extra",
                        ours: 1,
                        theirs: 0,
                    });
                }
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
        let Some(handle) = link_building(&built.sim, b.who, b.o) else {
            r.build_unlinked += 1;
            continue;
        };
        let ours = &built.sim.buildings[handle];
        compared::note("BuildDump", &["who", "o"]);
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
            compared::note("BuildDump", &[field]);
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
        // **Its hit points** (item 763, parked 728): `damage` and
        // `damage_frac` were parsed and compared by no site, so a building
        // taking a bomb was quiet on every window — run223's Barracks goes
        // to 45 on block 822 (`docs/GOLDEN.md` §25).
        for (field, mine, theirs) in [
            ("damage", i64::from(ours.damage), b.damage),
            ("damage_frac", i64::from(ours.damage_frac), b.damage_frac),
        ] {
            let Some(theirs) = theirs else { continue };
            r.build_compared += 1;
            compared::note("BuildDump", &[field]);
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
        // **Its hit-point figures** (item 1497, parked 1476): `myhits` is
        // `Wall::update_hits`' full figure with `BUILDING_HP_UPGRADE` in it
        // — the arm 1476 found no walk holding — and `construct_hits` the
        // site's growing one, compared only while both call it unfinished.
        for (field, mine, theirs) in [
            ("myhits", i64::from(ours.hits), b.myhits),
            (
                "construct_hits",
                i64::from(ours.construct_hits),
                b.construct_hits
                    .filter(|_| !ours.active && b.flags & 4 == 0),
            ),
        ] {
            let Some(theirs) = theirs else { continue };
            r.build_compared += 1;
            compared::note("BuildDump", &[field]);
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
        // **Who has ever seen it** (item 1563, parked 1450's ruling):
        // `ever_seen` and `ever_seen_completed` (`Wall +0x62`/`+0x63`) are
        // what first contact and a building's own reveal hang off
        // (`docs/VISION.md` §6.1, §6.4), and the instrument left both
        // uncompared until this — French East Indies parted on them for 236
        // blocks before its word did, seen only by item 1446's own helper.
        for (field, mine, theirs) in [
            ("ever_seen", i64::from(ours.ever_seen), b.ever_seen),
            (
                "ever_seen_completed",
                i64::from(ours.ever_seen_completed),
                b.ever_seen_completed,
            ),
        ] {
            let Some(theirs) = theirs else { continue };
            r.build_compared += 1;
            compared::note("BuildDump", &[field]);
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
            compared::note("BuildDump", &["pos"]);
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
        // **And `build_masks & 0x100`, the replan flag** (item 478). The
        // field is written at every detail level, `gamelog.rs` has parsed
        // it since the reader existed, and **nothing in the diff ever
        // compared it** — which is how Great Lakes' word came to sit for
        // four items on a road search whose whole cause was one bit the
        // dump prints on every building of every frame. Only the one bit
        // is compared: the rest of the word is the under-attack latch and
        // the activation mark, which move for reasons of their own.
        if let Some(mask) = b.build_masks {
            let theirs = i64::from(mask & 0x100 != 0);
            let mine = i64::from(ours.regen_roads);
            r.build_compared += 1;
            compared::note("BuildDump", &["build_masks"]);
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field: "regen_roads",
                    ours: mine,
                    theirs,
                });
            }
            // **And `& 0x80`, repeat air** (item 854, `docs/ORDERS.md`
            // §40): `Build::init@00629740:280` sets it with `0x8` on every
            // `can_carry(AIR)` building, and only the player's
            // repeat-orders toggle clears it. On all 201 dumps it is on
            // the five Airbases alone, from `BEGIN GAME` to the end.
            let theirs = i64::from(mask & 0x80 != 0);
            let mine = i64::from(ours.repeat_air);
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field: "repeat_air",
                    ours: mine,
                    theirs,
                });
            }
            // **And `& 0x40`, the infinite queue** (item 877,
            // `docs/GOLDEN.md` §33): only the player's button sets it,
            // on a training building with a train job queued; the
            // queue's own finish, refusal and emptying clear it. On
            // every dump before run285 it is clear on every building.
            let theirs = i64::from(mask & 0x40 != 0);
            let mine = i64::from(ours.queue.infinite);
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field: "infinite_queue",
                    ours: mine,
                    theirs,
                });
            }
        }
        // **And the city it belongs to, and the chain it sits on** (item
        // 661). `city` is the slot in the owner's own city array and
        // `city_down` the next member by object number, so both are
        // compared on what they hold rather than on this crate's indices:
        // the slot through `cities_of`, the chain through the city's
        // `members`, which keep the original's append order. Neither was
        // read before, and `CityData::num_buildings` — a trade route's
        // worth — walks exactly this chain (`docs/AI.md` §63).
        // both sides: both `Option`s are the dump's own, printed at every
        // detail level; this crate's side is never absent — a building in
        // no city answers −1, as the dump's does.
        if let (Some(city), Some(down)) = (b.city, b.city_down) {
            let slot = ours.city.map_or(-1, |ci| {
                built
                    .sim
                    .cities_of(ours.owner)
                    .iter()
                    .position(|&i| i == ci)
                    .map_or(-1, |i| i as i64)
            });
            let next = ours.city.map_or(-1, |ci| {
                let chain = built.sim.city_chain(ci);
                chain
                    .iter()
                    .position(|&m| m == handle)
                    .and_then(|k| chain.get(k + 1))
                    .map_or(-1, |&m| i64::from(built.sim.buildings[m].index))
            });
            for (field, mine, theirs) in [("city", slot, city), ("city_down", next, down)] {
                r.build_compared += 1;
                compared::note("BuildDump", &[field]);
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
            compared::note("BuildDump", &["queued"]);
            // Only where the depths agree: a queue one entry short would
            // otherwise report every slot after the gap and turn one fact
            // into a page of them — the same rule the mining list keeps.
            if mine == q {
                for (k, theirs) in b.queue.iter().take(q as usize).enumerate() {
                    let item = &ours.queue.items[k];
                    let id = item.tech.unwrap_or_else(|| built.unit_tree[item.ty]);
                    let ty = built.type_index.get(id).copied().unwrap_or(-1);
                    compared::note("BuildDump", &["queue"]);
                    compared::note("QueueItemDump", &["ty", "job_counter", "cost", "good"]);
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
            compared::note("BuildDump", &["gather_down"]);
            wrong("gather_down", -1, head, theirs);
        }
        // **The gather point list** (item 928, `docs/GOLDEN.md` §39):
        // `BuildData::gather`, printed last at `BUILDS=7`, empty on every
        // building of every capture before run312. Point by point only
        // where the lengths agree, as the mining list below.
        if let Some(len) = b.gather_len {
            compared::note("BuildDump", &["gather_len", "gather"]);
            let mine = ours.gather.len() as i64;
            // Not added to `build_compared`: that count is pinned on the
            // long captures (run56, run57, run58), which the gather point
            // cannot reach, and a list compared there is empty on both
            // sides every block.
            let mut diverge = |field: &'static str, ours: i64, theirs: i64| {
                if ours != theirs {
                    r.build_diverged.push(BuildDivergence {
                        frame: frame.n,
                        who: b.who,
                        o: b.o,
                        field,
                        ours,
                        theirs,
                    });
                }
            };
            diverge("gather_len", mine, len);
            if mine == len {
                for (k, &(x, y, action)) in b.gather.iter().enumerate() {
                    let p = ours.gather[k];
                    let (fx, fy, fa) = match k {
                        0 => ("gather[0].x", "gather[0].y", "gather[0].action"),
                        1 => ("gather[1].x", "gather[1].y", "gather[1].action"),
                        _ => ("gather[k].x", "gather[k].y", "gather[k].action"),
                    };
                    diverge(fx, i64::from(p.pos.x), x);
                    diverge(fy, i64::from(p.pos.y), y);
                    diverge(fa, i64::from(p.action), action);
                }
            }
        }
        // The mining list. `mining_len` is `None` below `BUILDS=7`, which
        // is what keeps a thin capture from reading as "every list empty".
        let Some(len) = b.mining_len else { continue };
        compared::note("BuildDump", &["mining_len"]);
        wrong("length", -1, ours.gather_from.len() as i64, len);
        if ours.gather_from.len() as i64 != len {
            continue;
        }
        for (k, theirs) in b.gather_from.iter().enumerate() {
            let mine = ours.gather_from[k];
            let k = k as i64;
            compared::note("BuildDump", &["gather_from"]);
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
        let Some(ci) =
            link_building(&built.sim, c.who, c.o).and_then(|b| built.sim.buildings[b].city)
        else {
            r.city_unlinked += 1;
            continue;
        };
        let ours = &built.sim.cities[ci];
        let w = c.who as usize;
        compared::note(
            "CityDump",
            &[
                "who",
                "o",
                "x",
                "y",
                "pop",
                "race",
                "city",
                "attack_stamp",
                "capture_stamp",
                "assimilation_timer",
                "capture_strength",
                "reg",
                "was_capital_flags",
                "in_port",
                "peasant_dist",
                "free",
                "busy",
                "gatherers",
                "ocean",
                "land",
                "filled",
                "bordering",
                "ocean_filled",
                "dock_tile",
                "city_flags",
                "space",
                "ter",
                "raid_stamp",
                "reduce_stamp",
                "scouted",
                "trade_val",
                "vans",
            ],
        );
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
            // `CITY_ATTACKING` and `CITY_EVER_ATTACKED`, set with `0x2` by
            // `Object::take_damage` (item 1034, `docs/COMBAT.md` §69).
            ("city_flags[0x4]", 0x4, ours.attacking),
            ("city_flags[0x8]", 0x8, ours.ever_attacked),
            ("city_flags[0x10]", 0x10, ours.capital),
            ("city_flags[0x40]", 0x40, ours.alarm),
            // **The temple mark** (item 552): `Build::activate` sets it on
            // a Temple's city and `compute_reg_territory` reads it as "this
            // city takes the temple border bonus". Chapter four's Temple
            // sets it on block 301.
            (
                "city_flags[0x80]",
                0x80,
                built.sim.city_has(ci, sim::build::Ident::Temple),
            ),
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
        // **The two fields nothing here holds**, asserted against the
        // zero this crate answered with rather than dropped: `raid_stamp`
        // (a writer since item 2235) and `scouted`, the AI's "a scout has
        // seen this city" mark. `scouted` has no writer in this crate, so its row is a claim
        // that the original never writes one either on the captures on
        // disk — a claim a capture can falsify, which is the point of
        // leaving it in.
        // Item 2106: `raid_stamp` is this crate's on a transfer
        // (`City::init`, `City::capture`); item 2235 added the raid's own
        // (`Object::do_damage`'s city alarm, `Sim::city_alarm`).
        fields.push(("raid_stamp".into(), ours.raid_stamp, c.raid_stamp));
        fields.push(("scouted".into(), 0, c.scouted));
        // `reduce_stamp` is `Object::do_damage`'s tail since item 1528.
        fields.push(("reduce_stamp".into(), ours.reduce_stamp, c.reduce_stamp));
        // **`trade_val` and `vans` are this crate's own since
        // `docs/CARAVAN.md` §7.2**, and until item 661 the rows still read
        // the zero written when neither had a writer — so a trade route
        // worth 160 against 176 printed as `ours 0`, and parked 514's
        // wealth-income gap sat beside its own cause for a month. `vans`
        // is every caravan `do_trade` has linked with this city as an end,
        // which is what `Array<CaravanLink>::add` records in the original.
        let vans = built
            .sim
            .caravans
            .iter()
            .flat_map(|l| l.slots[..l.mark].iter())
            .filter(|v| v.alive && v.linked && (v.city_a == Some(ci) || v.city_b == Some(ci)))
            .count() as i64;
        fields.push(("trade_val".into(), i64::from(ours.trade_val), c.trade_val));
        fields.push(("vans.length".into(), vans, c.vans.len() as i64));
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
    // A comma list watches several units on one run (item 795's four
    // walkers): each spec is read as the single form below.
    let Ok(all) = std::env::var("RON_DEBUG_UNIT") else {
        return;
    };
    for spec in all.split(',') {
        debug_watch_one(built, frame, spec);
    }
}

#[cfg(test)]
fn debug_watch_one(built: &Built, frame: i64, spec: &str) {
    let Some((who, o, lo, hi)) = Some(spec.to_string()).and_then(|v| {
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

/// `RON_DEBUG_GAP=<lo>-<hi>[:<who>/<o>,…]` — on every tick in the range,
/// not only on a captured block, a line each time one of the watched values
/// changes: each ledger's `gather_stamp` and `dirty`, the met matrix, the
/// war matrix, and each named unit's order stack, by kind and end (item
/// 1502). It is how a gap no capture holds is dated on this side: the
/// original's half of the gap is only its first block's stamps.
#[cfg(test)]
pub(crate) fn debug_gap(
    built: &Built,
    frame: i64,
    seen: &mut std::collections::BTreeMap<String, String>,
) {
    let Ok(v) = std::env::var("RON_DEBUG_GAP") else {
        return;
    };
    let (range, units) = v.split_once(':').unwrap_or((v.as_str(), ""));
    let Some((lo, hi)) = range
        .split_once('-')
        .and_then(|(a, b)| Some((a.trim().parse::<i64>().ok()?, b.trim().parse::<i64>().ok()?)))
    else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    let s = &built.sim;
    let mut rows: Vec<(String, String)> = Vec::new();
    for (w, l) in s.ledgers.iter().enumerate() {
        rows.push((
            format!("{w}/ledger"),
            format!("gather_stamp {} dirty {}", l.gather_stamp, l.dirty),
        ));
    }
    rows.push(("treaties".into(), format!("{:?}", s.treaties)));
    rows.push(("at_war".into(), format!("{:?}", s.at_war)));
    for u in units.split(',').filter(|u| !u.is_empty()) {
        let Some((w, o)) = u
            .split_once('/')
            .and_then(|(w, o)| Some((w.parse::<i64>().ok()?, o.parse::<i64>().ok()?)))
        else {
            continue;
        };
        let found = s
            .units
            .iter()
            .find(|x| x.alive() && i64::from(x.owner) == w && i64::from(x.index) == o);
        let val = found.map_or_else(
            || "absent".to_string(),
            |x| {
                // The stack's kinds and ends, not its waypoints, which move
                // on every step.
                let stack: Vec<String> = x
                    .orders
                    .iter()
                    .map(|o| match o.body {
                        sim::orders::Body::Move(m) => {
                            format!("{}:{:?}({},{})", o.index(), m.kind, m.dest.x, m.dest.y)
                        }
                        b => {
                            let d = format!("{b:?}");
                            format!("{}:{}", o.index(), &d[..d.len().min(90)])
                        }
                    })
                    .collect();
                // `line_ok` and `tolerance` are what `Unit::do_move` branches
                // on first (Loop 2678: 2638 took a probe commit for each).
                format!("orders {stack:?} line_ok {} tol {}", x.line_ok, x.tolerance)
            },
        );
        rows.push((format!("{w}/{o}"), val));
    }
    // Every delivery of damage this tick, by whom on whom: on a gap whose
    // draws agree, the strikes are the two sides' common events.
    let name = |o: sim::combat::Obj| match o {
        sim::combat::Obj::Unit(u) => format!("{}/{}", s.units[u].owner, s.units[u].index),
        sim::combat::Obj::Building(b) => {
            format!("{}/{}", s.buildings[b].owner, s.buildings[b].index)
        }
    };
    for h in s.hits.iter().rev().take_while(|h| h.frame + 1 >= frame) {
        if h.frame + 1 == frame {
            eprintln!(
                "  gap f{frame} hit {} -> {} dmg {} killed {}",
                name(h.attacker),
                name(h.target),
                h.damage,
                h.killed
            );
        }
    }
    for (k, val) in rows {
        if seen.get(&k) != Some(&val) {
            eprintln!("  gap f{frame} {k}: {val}");
            seen.insert(k, val);
        }
    }
}

/// `RON_DEBUG_AMMO=<lo>-<hi>` — every projectile in flight after the
/// block's frame: its shooter, target, launch, landing, clock, accuracy and
/// flags, beside the target's own position (item 1081). No capture here
/// writes an `AMMO` record, so this side is the only one it can print.
#[cfg(test)]
pub(crate) fn debug_ammo(built: &Built, frame: i64) {
    let Some((lo, hi)) = site_window_named("RON_DEBUG_AMMO") else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    let name = |o: sim::combat::Obj| match o {
        sim::combat::Obj::Unit(u) => {
            format!("{}/{}", built.sim.units[u].owner, built.sim.units[u].index)
        }
        sim::combat::Obj::Building(b) => format!("b{b}"),
    };
    // Every building whose whole-hit `damage` or sixteenths changed since
    // the last frame printed, with its index, owner and slot (item 1522:
    // which round struck `0/2000`, and when).
    {
        use std::sync::Mutex;
        static LAST: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());
        let mut last = LAST.lock().unwrap();
        let n = built.sim.buildings.len().max(last.len());
        last.resize(n, (0, 0));
        for (i, b) in built.sim.buildings.iter().enumerate() {
            if last[i] != (b.damage, b.damage_frac) {
                eprintln!(
                    "  f{frame} bld b{i} {}/{} damage {} frac {} (was {} {})",
                    b.owner, b.index, b.damage, b.damage_frac, last[i].0, last[i].1
                );
                last[i] = (b.damage, b.damage_frac);
            }
        }
        for p in &built.sim.projectiles {
            if let Some(sim::combat::Obj::Building(b)) = p.target {
                let tp = built.sim.profile(sim::combat::Obj::Building(b));
                eprintln!(
                    "  f{frame} tgt b{b} pos {:?} x_size {} y_size {} hit_target {}",
                    built.sim.buildings[b].pos,
                    tp.x_size,
                    tp.y_size,
                    sim::combat::hits_building(
                        p.landing,
                        built.sim.buildings[b].pos,
                        tp.x_size,
                        tp.y_size
                    )
                );
            }
        }
    }
    for p in &built.sim.projectiles {
        let tpos = p.target.and_then(|t| match t {
            sim::combat::Obj::Unit(u) => Some(built.sim.units[u].pos),
            sim::combat::Obj::Building(_) => None,
        });
        eprintln!(
            "  f{frame} ammo #{} {} -> {:?} at {:?} launch ({},{}) land ({},{}) t{}/{} acc {} roll {} missed {} angle {}",
            p.slot,
            name(p.shooter),
            p.target.map(name),
            tpos.map(|q| (q.x, q.y)),
            p.launch.x,
            p.launch.y,
            p.landing.x,
            p.landing.y,
            p.cur_time,
            p.total_time,
            p.accuracy,
            p.rolling,
            p.missed,
            p.angle.0
        );
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
            // The target as the dump keys it: owner/index and position.
            let target = a.target.map(|t| match t {
                sim::combat::Obj::Unit(u) => {
                    let u = &built.sim.units[u];
                    format!("unit {}/{} ({},{})", u.owner, u.index, u.pos.x, u.pos.y)
                }
                sim::combat::Obj::Building(b) => {
                    let b = &built.sim.buildings[b];
                    format!("build {}/{} ({},{})", b.owner, b.index, b.pos.x, b.pos.y)
                }
            });
            // The group's cap, its accumulator, its pool slot and leader
            // (item 1586): what `get_speed`'s last arm reads for a member.
            let g = built.sim.army_group(w as u8, slot);
            let leader = built
                .sim
                .group_find_leader(&g)
                .map(|u| format!("{}/{}", built.sim.units[u].owner, built.sim.units[u].index));
            let list: Vec<String> = g
                .list
                .iter()
                .map(|&u| format!("{}", built.sim.units[u].index))
                .collect();
            eprintln!(
                "  f{frame} army {w}/{slot} cap {}/{} pool {:?} march {} leader {:?} list [{}]",
                a.group.speed,
                a.group.new_speed,
                a.group.pool,
                a.group.march,
                leader,
                list.join(" ")
            );
            eprintln!(
                "  f{frame} army {w}/{slot} status {} target {:?} pos ({},{}) muster ({},{}) \
                 rally {} hurry {} units {} caps {} std {} [{}]",
                a.status,
                target,
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

/// **Every record the dump carries on one block, both directions** — the
/// row walk `run100_s_word_block_is_every_record_the_dump_carries` has
/// grown since item 448, lifted out whole on item 520 so a second
/// capture's widening is the same instrument and not a copy of it that
/// drifts. It notes the block each `(who, o, field)` **first** parts on,
/// with the value diff beside it, into `firsts`, and answers `compare`'s
/// own result and the number of direct rows it compared.
#[cfg(test)]
pub(crate) fn widen_block(
    built: &Built,
    frame: &Frame,
    players: usize,
    n: i64,
    firsts: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
) -> (FrameResult, usize) {
    let eager = std::env::var_os("RON_DISABLE_LAZY_ROW_LABELS").is_some();
    if std::env::var_os("RON_VERIFY_ROW_LABELS").is_some() {
        let mut reference_firsts = firsts.clone();
        let (actual, fields) =
            compared::recorded(|| widen_block_rows(built, frame, players, n, firsts, eager));
        let (reference, reference_fields) = compared::recorded(|| {
            widen_block_rows(built, frame, players, n, &mut reference_firsts, true)
        });
        assert_eq!(actual, reference, "whole frame result and row count");
        assert_eq!(
            *firsts, reference_firsts,
            "all divergence labels and values"
        );
        assert_eq!(fields, reference_fields, "all registered fields");
        actual
    } else {
        widen_block_rows(built, frame, players, n, firsts, eager)
    }
}

#[cfg(test)]
fn widen_block_rows(
    built: &Built,
    frame: &Frame,
    players: usize,
    n: i64,
    firsts: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
    eager: bool,
) -> (FrameResult, usize) {
    let r = compare(built, frame, players);
    let mut compared = 0usize;
    {
        let mut note = |who: i64, o: i64, what: String, row: String| {
            firsts.entry((who, o, what)).or_insert((n, row));
        };
        for d in &r.diverged {
            note(
                d.who,
                d.o,
                "pos".into(),
                format!(
                    "ours ({},{}) theirs ({},{})",
                    d.ours.x, d.ours.y, d.theirs.x, d.theirs.y
                ),
            );
        }
        for d in &r.order_diverged {
            note(d.who, d.o, d.what.label(), format!("{:?}", d.what));
        }
        for d in &r.angle_diverged {
            note(
                d.who,
                d.o,
                format!("angle:{:?}", d.which),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.collide_diverged {
            note(
                d.who,
                d.o,
                d.field.into(),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.los_diverged {
            note(
                d.who,
                d.o,
                "mylos".into(),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.packed_diverged {
            note(d.who, d.o, "packed".into(), format!("ours {}", d.ours));
        }
        for d in &r.search_diverged {
            note(
                d.who,
                d.o,
                "start_dist".into(),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.gather_diverged {
            note(
                d.who,
                d.o,
                format!("gather:{}[{}]", d.field, d.at),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.build_diverged {
            note(
                d.who,
                d.o,
                format!("build:{}", d.field),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.queue_diverged {
            note(
                d.who,
                d.o,
                format!("queue:{}", d.field),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.city_diverged {
            note(
                d.who,
                d.o,
                format!("city:{}", d.field),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.inside_diverged {
            note(
                d.who,
                d.o,
                "inside".into(),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for &(who, o) in &r.unlinked_units {
            note(who, o, "unlinked".into(), "the dump holds it alone".into());
        }
        for &(who, o) in &r.extra_units {
            note(who, o, "extra".into(), "this crate holds it alone".into());
        }
        // **The hit-point record from `compare` itself** (item
        // 484). The direct rows below have carried `hits_left`
        // and `myhits` since item 464, but they are this test's
        // own reading; this is the comparator's, and it is the
        // one every other capture now runs. Both agree on all
        // 909 blocks, so the overlap costs nothing and a future
        // parting is reported by whichever sees it first.
        for d in &r.hits_diverged {
            note(
                d.who,
                d.o,
                format!("hits:{}", d.field),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        // **The death-object list** (item 491, `docs/COMBAT.md`
        // §42.1) — a reader added to the harness is added to its
        // driver in the same landing, which is what keeps
        // `hold_frames` and `DEATH_OBJS` off `ledger`'s
        // single-capture half. Great Lakes' window carries no
        // death, so what this asserts here is that this crate
        // invents none either: an `extra` row would fire.
        for d in &r.death_diverged {
            note(
                d.who,
                d.o,
                format!("death:{}", d.field),
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
    }
    // **The other direction on the buildings**, which `compare`
    // does not do: it counts the dump's buildings it cannot link
    // (`build_unlinked`) and never asks whether this crate holds
    // one the block does not name.
    for b in built.sim.buildings.iter().filter(|b| b.alive) {
        let (w, o) = (i64::from(b.owner), i64::from(b.index));
        if !(0..players as i64).contains(&w) {
            continue;
        }
        if !frame.builds.iter().any(|x| x.who == w && x.o == o) {
            firsts
                .entry((w, o, "build:extra".into()))
                .or_insert((n, "this crate holds it alone".into()));
        }
    }
    // **And the dump's direction, by name** (item 644). `compare` counts a
    // dump building it cannot link in `build_unlinked` and moves on, so a
    // building the original placed and this crate did not was a count
    // here and never a row: run157's `1/2008`, placed on 1176, stood in
    // no widening for the item that named its frame.
    for b in &frame.builds {
        if !(0..players as i64).contains(&b.who) {
            continue;
        }
        let held = built
            .sim
            .buildings
            .iter()
            .any(|x| i64::from(x.owner) == b.who && i64::from(x.index) == b.o);
        if !held {
            firsts
                .entry((b.who, b.o, "build:unlinked".into()))
                .or_insert((n, "the dump holds it alone".into()));
        }
    }
    // **The death list, read directly** (item 491). `compare`
    // walks it above; this is the driver's own reading, for the
    // reason the hit-point pair has both — and on Great Lakes it
    // is an *emptiness* claim in two directions, because run100's
    // window holds no death at all. A record on either side here
    // is a finding whichever instrument sees it first.
    for d in &frame.deaths {
        let (Some(who), Some(o)) = (d.who, d.o) else {
            continue;
        };
        let ours = built
            .sim
            .deaths
            .iter()
            .find(|m| i64::from(m.who) == who && i64::from(m.o) == o);
        let row = match ours {
            None => "the dump holds it alone".to_string(),
            Some(m) => format!(
                "ours first_frame {} cur_anim {} theirs {:?} {:?}",
                m.first_frame, m.cur_anim, d.first_frame, d.cur_anim
            ),
        };
        if ours.is_none_or(|m| {
            d.first_frame.is_some_and(|f| f != m.first_frame)
                || d.cur_anim.is_some_and(|a| a != i64::from(m.cur_anim))
        }) {
            firsts
                .entry((who, o, "death:direct".into()))
                .or_insert((n, row));
        }
    }
    // **And the record's own rows, ungated.** Everything the
    // `UNITDATA` and `GUY` blocks print that `compare` either
    // does not read at all or reads only where the positions
    // already agree.
    for them in &frame.units {
        if !(0..players as i64).contains(&them.who) {
            continue;
        }
        let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
            continue;
        };
        let Some(u) = built.sim.unit_by_o(who, o) else {
            continue;
        };
        let un = &built.sim.units[u];
        let mut rows: Vec<(RowLabel, i64, Option<i64>)> = vec![
            (
                "heading".into(),
                i64::from(un.movement.heading.0),
                them.angle,
            ),
            (
                "dest_angle".into(),
                i64::from(un.movement.des_angle.0),
                them.dest_angle,
            ),
            ("orders_x".into(), i64::from(un.orders_pos.x), them.orders_x),
            ("orders_y".into(), i64::from(un.orders_pos.y), them.orders_y),
            ("tolerance".into(), i64::from(un.tolerance), them.tolerance),
            (
                "path_recursion".into(),
                i64::from(un.path_recursion),
                them.path_recursion,
            ),
            ("idle".into(), i64::from(un.idle), them.idle),
            ("stance".into(), i64::from(un.stance), them.stance),
            // **`myhits` is the max, not what is left.** The
            // record carries the pair `myhits`/`damage` —
            // `ObjectData::myhits` is the whole hit points the
            // type has after tech and `damage` what has been
            // taken off them (`crate::diff::army`'s loader has
            // read it that way since it existed) — and this row
            // compared this crate's *current* health against the
            // **maximum**, so it agreed only while the unit was
            // untouched and reported every wound as a
            // divergence. Item 464 found it on the word's own
            // block: `0/5 myhits: ours 38 theirs 40`, where the
            // dump's own `damage` is 2 and the two sides agree.
            // Both halves are compared now.
            (
                "hits_left".into(),
                i64::from(un.health),
                them.myhits.map(|h| h - them.damage.unwrap_or(0)),
            ),
            ("myhits".into(), i64::from(un.max_health), them.myhits),
            // **The sixteenths under them** (item 484): the
            // third field of the `OBJECT` block's hit-point
            // record, parsed for a building since item 394 and
            // for a unit only now. It is what decides *which
            // frame* the next whole point of `damage` lands on
            // (`docs/COMBAT.md` §7.2 step 4).
            (
                "damage_frac".into(),
                i64::from(un.damage_frac),
                them.damage_frac,
            ),
            // **`hold_frames`** (item 491, §42.3): the slot hold,
            // on the same block, and zero on every living unit of
            // both windows because every writer of it is on a
            // dead object. `compare` carries it too; this is the
            // driver's own reading, for the reason the hit-point
            // pair above has both.
            (
                "hold_frames".into(),
                i64::from(un.hold_frames),
                them.hold_frames,
            ),
            // **The overkill window** (item 485): the frame of
            // the first hit inside it and the captain and owner
            // who struck (§7.1 step 2, §41.1). Written at
            // `UNITDATA`'s own indent on every unit of every
            // block and compared nowhere until chapter two's
            // `1/8` turned out to be wounded a frame later than
            // the dump wounds it. `damage_frame` is the sim
            // frame in the same numbering `tick` passes, so it
            // compares directly; the pair beside it is read only
            // where **both** sides hold a live window, because a
            // never-hit unit is `-1`/`0` in the original and
            // `0`/`0` here and that is a difference of encoding.
            (
                "damage_frame".into(),
                un.combat.damage_frame,
                them.damage_frame,
            ),
            (
                "damage_o".into(),
                i64::from(un.combat.damage_o),
                them.damage_o.filter(|_| {
                    un.combat.damage_frame != 0 && them.damage_frame.is_some_and(|f| f != 0)
                }),
            ),
            (
                "damage_who".into(),
                i64::from(un.combat.damage_who),
                them.damage_who.filter(|_| {
                    un.combat.damage_frame != 0 && them.damage_frame.is_some_and(|f| f != 0)
                }),
            ),
            ("myspeed".into(), i64::from(un.movement.speed), them.myspeed),
            // **The namesake's two fields** (item 552): the pending period
            // `Unit::process_attrition` writes on each 32-frame refresh,
            // and `unit_masks2`'s `0x40000`, the mark a tick supply vetoed
            // leaves (`docs/SUPPLY.md`, "Behavioural check"). Printed on
            // every `UNITDATA` and compared nowhere until chapter four put
            // a squad on hostile ground; zero on both sides everywhere
            // else, which is itself a claim this row now checks.
            ("attrition".into(), i64::from(un.attrition), them.attrition),
            (
                "sheltered".into(),
                i64::from(un.sheltered),
                them.unit_masks2.map(|m| i64::from(m & 0x4_0000 != 0)),
            ),
            // **The pool slot** (item 518): `UnitData +0x80`, the
            // group every member points at, `who·64 + s`. It is
            // what `Groups::process` selects on once a frame, so a
            // group numbered wrong has its cap reset on the wrong
            // frames (`docs/GROUPS.md` §19). Printed on every
            // `UNITDATA` and compared nowhere until this row.
            ("group".into(), built.sim.pool_group_of(u), them.group),
            ("form".into(), i64::from(un.form), them.form),
            ("form_mod".into(), i64::from(un.form_width), them.form_mod),
            (
                "orders.len".into(),
                un.orders.len() as i64,
                Some(them.orders.len() as i64),
            ),
            // **The unit's own mirror, `unit_masks & 2`** (item 736):
            // carried since item 711 (`docs/GROUPS.md` §25) and read by
            // `do_guard` off its target, and compared nowhere until the
            // Despot's escort traded posts on Great Lakes 15607.
            (
                "mirror".into(),
                i64::from(un.movement.mirror),
                them.unit_masks.map(|m| i64::from(m & 2 != 0)),
            ),
        ];
        // The collision block and the two masks, **ungated by the
        // position** — the whole point of this test.
        if un.on_map {
            for r in [
                ("collide", i64::from(un.collide), them.collide),
                ("collide_o", i64::from(un.collide_o), them.collide_o),
                ("collide_who", i64::from(un.collide_who), them.collide_who),
                ("collide_guy", i64::from(un.collide_guy), them.collide_guy),
                ("safe", i64::from(un.safe), them.safe),
                (
                    "recharging",
                    i64::from(un.combat.recharging),
                    them.recharging,
                ),
                // **The fire on the move's target** (item 1508,
                // `docs/AI.md` §134): −1, −2 while the search waits for
                // its phase, or what a `v` type shoots at beside its walk.
                ("cavarch_o", built.sim.cavarch_o(u), them.cavarch_o),
                ("cavarch_who", i64::from(un.cavarch_who), them.cavarch_who),
                ("start_dist", i64::from(un.start_dist), them.start_dist),
                // The cast clock (item 1608): `get_job_time`'s type arms
                // are read on every frame of a cast, not at its end.
                ("spell_time", i64::from(un.spell_time), them.spell_time),
                ("mylos", i64::from(built.sim.units[u].mylos), them.mylos),
                (
                    "half_step",
                    i64::from(un.half_step),
                    them.unit_masks.map(|m| i64::from(m & 0x10_0000 != 0)),
                ),
                (
                    "packed",
                    i64::from(un.combat.packed),
                    them.unit_masks.map(|m| i64::from(m & 0x8_0000 != 0)),
                ),
                // **`unit_masks & 0x1000` and `& 0x10000`** (item 2798,
                // `docs/GOLDEN.md` §104): the blown and fought bits, set
                // by `fight`, dropped by `check_idle` and the 512-frame
                // `process_cloak` — read off the word, never compared.
                (
                    "blown",
                    i64::from(un.blown),
                    them.unit_masks.map(|m| i64::from(m & 0x1000 != 0)),
                ),
                (
                    "fought",
                    i64::from(un.fought),
                    them.unit_masks.map(|m| i64::from(m & 0x1_0000 != 0)),
                ),
            ] {
                rows.push((r.0.into(), r.1, r.2));
            }
        }
        // The guy record whole, run67's `GUYS=4` block — the
        // animation clock, the destination pair and the two
        // speeds, which are what the word turned out to be.
        rows.push((
            "guys.len".into(),
            un.guys.len() as i64,
            Some(them.guys.len() as i64),
        ));
        for (k, g) in them.guys.iter().enumerate() {
            let Some(og) = un.guys.get(k).copied() else {
                continue;
            };
            let (body, facing, des, des_angle) = match og.follow {
                Some(b) => (b.body, b.facing, b.des, b.des_angle),
                None if k == 0 => (
                    un.movement.body,
                    un.movement.facing,
                    un.pos,
                    un.movement.heading,
                ),
                None => (
                    un.movement.body,
                    un.movement.facing,
                    un.pos,
                    un.movement.facing,
                ),
            };
            let track = og.follow.map_or((0, 0), |b| b.track);
            for (name, ours, theirs) in [
                ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                ("g.angle", i64::from(facing.0), g.angle),
                ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                ("g.end_time", i64::from(og.end_time), g.end_time),
                ("g.last_time", i64::from(og.last_time), g.last_time),
                ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                ("g.stopped", i64::from(og.stopped), g.stopped),
                // **The attack a figure owes** (item 510): held
                // while the body is still walking or still
                // turning, queued while an attack is already
                // playing, and only the queued one is paid inside
                // `Guy::inc_time`'s wrap loop with a roll. They
                // are new to the parser in that landing, and the
                // rule that earned them applies to this record as
                // much as to chapter two's — a record widened on
                // one window and not the next is the shape
                // `crate::ledger` exists to count.
                // `docs/COMBAT.md` §45.
                ("g.hold_attack", i64::from(og.pending_attack), g.hold_attack),
                (
                    "g.queued_attack",
                    i64::from(og.queued_attack),
                    g.queued_attack,
                ),
                ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
            ] {
                rows.push((RowLabel(name, Some(k)), ours, theirs));
            }
            // **A plane's attitude and altitude** (item 759,
            // `docs/ORDERS.md` §33): guy 0's bank and pitch by their
            // bits, on every figure — zero on any that does not fly —
            // and its altitude on an aircraft's alone, since a figure on
            // the ground follows the terrain (`find_data_z`), which this
            // crate does not carry.
            if k == 0 {
                let af = un.airframe;
                for (name, ours, theirs) in [
                    ("g.bank", i64::from(af.bank.bits()), g.bank),
                    ("g.last_bank", i64::from(af.last_bank.bits()), g.last_bank),
                    ("g.pitch", i64::from(af.pitch.bits()), g.pitch),
                    (
                        "g.last_pitch",
                        i64::from(af.last_pitch.bits()),
                        g.last_pitch,
                    ),
                ] {
                    rows.push((RowLabel(name, Some(k)), ours, theirs));
                }
                if un.kind.domain == sim::attrition::Domain::Air {
                    rows.push((
                        RowLabel("g.z", Some(k)),
                        i64::from(af.z),
                        g.pos.map(|p| p.z),
                    ));
                    rows.push((
                        RowLabel("g.last_z", Some(k)),
                        i64::from(af.last_z),
                        g.last_pos.map(|p| p.z),
                    ));
                }
            }
        }
        // Reference mode retains the original eager allocation of every label.
        let mut eager_names: Vec<_> = if eager {
            rows.iter().map(|(name, _, _)| name.render()).collect()
        } else {
            Vec::new()
        };
        for (index, (name, ours, theirs)) in rows.into_iter().enumerate() {
            let Some(theirs) = theirs else { continue };
            compared += 1;
            if let Some((record, field)) =
                widened_field(if eager { &eager_names[index] } else { name.0 })
            {
                compared::note(record, &[field]);
            }
            if ours != theirs {
                firsts
                    .entry((
                        them.who,
                        them.o,
                        if eager {
                            std::mem::take(&mut eager_names[index])
                        } else {
                            name.render()
                        },
                    ))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        }
    }
    (r, compared)
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct RowLabel(&'static str, Option<usize>);
#[cfg(test)]
impl From<&'static str> for RowLabel {
    fn from(name: &'static str) -> Self {
        Self(name, None)
    }
}
#[cfg(test)]
impl RowLabel {
    fn render(self) -> String {
        match self.1 {
            Some(index) => format!("{}[{index}]", self.0),
            None => self.0.to_owned(),
        }
    }
}

#[cfg(test)]
#[test]
fn deferred_row_labels_preserve_names_and_field_registration() {
    let source = include_str!("harness.rs");
    let mapping = source
        .rsplit("fn widened_field(label:")
        .next()
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    let mut names = std::collections::BTreeSet::new();
    for (index, text) in mapping.split('"').enumerate() {
        if index % 2 == 1 {
            names.insert(text);
        }
    }
    names.insert("unknown-row");
    assert!(names.len() > 50);
    for name in names {
        for index in [None, Some(0), Some(1), Some(usize::MAX)] {
            let label = RowLabel(name, index);
            let original = match index {
                Some(k) => format!("{name}[{k}]"),
                None => name.to_string(),
            };
            assert_eq!(label.render(), original);
            assert_eq!(widened_field(label.0), widened_field(&original));
        }
    }
}

/// The parser's `Record.field` behind one of [`widen_block`]'s row labels
/// (parked 527). A label the table does not know registers nothing, and
/// the coverage pin then says its field arrived uncompared — which is the
/// right verdict for a row nobody has named the field of.
#[cfg(test)]
fn widened_field(label: &str) -> Option<(&'static str, &'static str)> {
    let base = label.split('[').next().unwrap_or(label);
    Some(match base {
        "heading" => ("UnitDump", "angle"),
        "hits_left" => ("UnitDump", "damage"),
        "sheltered" => ("UnitDump", "unit_masks2"),
        "orders.len" => ("UnitDump", "orders"),
        "guys.len" => ("UnitDump", "guys"),
        "half_step" | "packed" | "mirror" | "blown" | "fought" => ("UnitDump", "unit_masks"),
        "dest_angle" => ("UnitDump", "dest_angle"),
        "orders_x" => ("UnitDump", "orders_x"),
        "orders_y" => ("UnitDump", "orders_y"),
        "tolerance" => ("UnitDump", "tolerance"),
        "path_recursion" => ("UnitDump", "path_recursion"),
        "idle" => ("UnitDump", "idle"),
        "stance" => ("UnitDump", "stance"),
        "myhits" => ("UnitDump", "myhits"),
        "damage_frac" => ("UnitDump", "damage_frac"),
        "hold_frames" => ("UnitDump", "hold_frames"),
        "damage_frame" => ("UnitDump", "damage_frame"),
        "damage_o" => ("UnitDump", "damage_o"),
        "damage_who" => ("UnitDump", "damage_who"),
        "myspeed" => ("UnitDump", "myspeed"),
        "attrition" => ("UnitDump", "attrition"),
        "group" => ("UnitDump", "group"),
        "form" => ("UnitDump", "form"),
        "form_mod" => ("UnitDump", "form_mod"),
        "collide" => ("UnitDump", "collide"),
        "collide_o" => ("UnitDump", "collide_o"),
        "collide_who" => ("UnitDump", "collide_who"),
        "collide_guy" => ("UnitDump", "collide_guy"),
        "safe" => ("UnitDump", "safe"),
        "recharging" => ("UnitDump", "recharging"),
        "cavarch_o" => ("UnitDump", "cavarch_o"),
        "cavarch_who" => ("UnitDump", "cavarch_who"),
        "start_dist" => ("UnitDump", "start_dist"),
        "spell_time" => ("UnitDump", "spell_time"),
        "mylos" => ("UnitDump", "mylos"),
        "g.x" | "g.y" => ("Guy", "pos"),
        "g.angle" => ("Guy", "angle"),
        "g.des_x" | "g.des_y" => ("Guy", "des"),
        "g.des_angle" => ("Guy", "des_angle"),
        "g.cur_anim" => ("Guy", "cur_anim"),
        "g.cur_time" => ("Guy", "cur_time"),
        "g.end_time" => ("Guy", "end_time"),
        "g.last_time" => ("Guy", "last_time"),
        "g.gpiece" => ("Guy", "gpiece"),
        "g.stopped" => ("Guy", "stopped"),
        "g.hold_attack" => ("Guy", "hold_attack"),
        "g.queued_attack" => ("Guy", "queued_attack"),
        "g.track_dx" | "g.track_dy" => ("Guy", "track"),
        "g.last_speed" => ("Guy", "last_speed"),
        "g.avg_speed" => ("Guy", "avg_speed"),
        "g.bank" => ("Guy", "bank"),
        "g.last_bank" => ("Guy", "last_bank"),
        "g.pitch" => ("Guy", "pitch"),
        "g.last_pitch" => ("Guy", "last_pitch"),
        // The altitude is `pos`'s third slot, whose first two are `g.x`
        // and `g.y`. `last_z` is `last_pos`'s, and its `x`/`y` are not
        // compared, so it registers nothing.
        "g.z" => ("Guy", "pos"),
        _ => return None,
    })
}

/// `RON_DEBUG_TECH=<lo>-<hi>` — every computer leader's held types: the
/// whole set on the window's first frame, then each one gained or lost,
/// by frame and name, with `ages` and the four epochs beside it.
#[cfg(test)]
pub(crate) fn debug_tech(built: &Built, frame: i64) {
    use std::cell::RefCell;
    thread_local! {
        static LAST: RefCell<Vec<Vec<bool>>> = const { RefCell::new(Vec::new()) };
    }
    let Some((lo, hi)) = site_window_named("RON_DEBUG_TECH") else {
        return;
    };
    if !(lo..=hi).contains(&frame) {
        return;
    }
    let name = |t: usize| built.sim.tech_tree.types[t].name.clone();
    LAST.with(|last| {
        let mut last = last.borrow_mut();
        last.resize(built.sim.players.len(), Vec::new());
        for w in 0..built.sim.players.len() {
            if built.sim.nation[w].human || built.sim.defeated[w] {
                continue;
            }
            let p = &built.sim.tech[w];
            let held: Vec<bool> = (0..p.tech.len())
                .map(|t| p.tech[t] || p.obs.get(t).copied().unwrap_or(false))
                .collect();
            let changed: Vec<String> = if last[w].len() != held.len() {
                (0..held.len())
                    .filter(|&t| p.tech[t])
                    .map(|t| format!("{t}:{}", name(t)))
                    .collect()
            } else {
                (0..held.len())
                    .filter(|&t| held[t] != last[w][t])
                    .map(|t| {
                        format!(
                            "{}{t}:{}{}",
                            if p.tech[t] { "+" } else { "-" },
                            name(t),
                            if p.obs[t] { "(obs)" } else { "" }
                        )
                    })
                    .collect()
            };
            if !changed.is_empty() {
                eprintln!(
                    "  f{frame} T{w} ages {} epoch {:?}: {}",
                    p.ages,
                    p.epoch,
                    changed.join(" ")
                );
            }
            last[w] = held;
        }
    });
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
    for (at, b) in built
        .sim
        .buildings
        .iter()
        .enumerate()
        .filter(|(_, b)| b.alive)
    {
        let ty = b.orig_ty.map_or_else(
            || "-".to_string(),
            |t| format!("{:?}#{t}", built.sim.build_types[t].ident),
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
        // The head's target as `do_queue` computes it this frame, and the
        // population test `Build::finished` reads (item 1365).
        let head = if b.queue.items.is_empty() {
            String::new()
        } else {
            let m = &built.sim.muster[b.owner as usize];
            format!(
                " target{} cap{} control{} owned{}",
                built.sim.queue_target(at, 0),
                m.cap,
                m.control,
                m.by_type[b.queue.items[0].ty]
            )
        };
        eprintln!(
            "  f{frame} B {}/{} ty{ty} ({},{}) act{} regen{} jc{}/{} hits{} dmg{} help{} gl{} q[{}]{head}",
            b.owner,
            b.index,
            b.pos.x,
            b.pos.y,
            u8::from(b.active),
            u8::from(b.regen_roads),
            b.job_counter,
            b.constr_time,
            b.construct_hits,
            b.damage,
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
        "  f{frame} {who}/{o} #{} TY {:?} ({}) TI {:?} PACKS {:?} army {:?} gspeed {:?} at ({}, {}) in {:?} on {} ang {} hdg {} \
         danger {} stance {}/{:?} coll {}@{} tgt {:?} reach {:?} path {:?} orders {:?} {}",
        // The unit's slot in `sim.units`, which a `GroupMove`'s `leader`
        // names (item 795).
        built
            .sim
            .units
            .iter()
            .position(|x| std::ptr::eq(x, u))
            .unwrap_or(usize::MAX),
        u.ty,
        // The type's name beside `TY` (parked 1846): `TY` is this crate's
        // record number and `TI` the dump's `TypeIndex`; 272 read as a
        // `HUMMEL` when the Freighter was `TI` 322.
        u.ty.and_then(|t| built.unit_type_names.get(t))
            .map_or("?", String::as_str),
        // The `TypeIndex` the dump's `GUY` block would write (item 1555).
        u.ty.and_then(|t| built.unit_tree.get(t))
            .and_then(|id| built.type_index.get(*id)),
        u.ty.map(|t| built.sim.unit_types[t].combat.packs),
        built
            .sim
            .units
            .iter()
            .position(|x| std::ptr::eq(x, u))
            .and_then(|i| built.sim.army_of(i)),
        // The group's cap and its accumulator, `(speed, new_speed)` —
        // what `get_speed`'s last arm reads (`docs/GROUPS.md` §18).
        built
            .sim
            .units
            .iter()
            .position(|x| std::ptr::eq(x, u))
            .and_then(|i| built.sim.group_speed_pair_of(i)),
        u.pos.x,
        u.pos.y,
        u.inside_unit
            .map(|b| (built.sim.units[b].owner, built.sim.units[b].index)),
        u.on_map,
        u.movement.facing.0,
        u.movement.heading.0,
        // `unit_masks & 4` and `UnitData::stance`, which this crate keeps
        // in two places: the worker byte and the combat stance (item 530).
        u8::from(u.in_danger),
        u.stance,
        u.combat.stance,
        // `collide` and `collide_frame`: the step's last block, which the
        // widening compares only once ours has stamped one (item 795).
        u.collide,
        u.collide_frame,
        // The combat target, which an `Attack` order does not carry
        // (item 997: the attack-move's look pushed one on 4554).
        u.combat.target.map(|t| match t {
            sim::combat::Obj::Unit(v) => (
                built.sim.units[v].owner,
                i64::from(built.sim.units[v].index)
            ),
            sim::combat::Obj::Building(b) => (
                built.sim.buildings[b].owner,
                i64::from(built.sim.buildings[b].index)
            ),
        }),
        // The chase's own question (item 1002): `attack_dist` to the
        // target, the reach, and `do_move`'s in-range verdict with and
        // without the `0x90` margin, beside `mandatory`.
        u.combat.target.map(|t| {
            let me = sim::combat::Obj::Unit(
                built
                    .sim
                    .units
                    .iter()
                    .position(|x| std::ptr::eq(x, u))
                    .unwrap_or(usize::MAX),
            );
            (
                built.sim.attack_dist(me, t),
                built.sim.max_range_of(me),
                built.sim.is_in_range_at_margin(me, u.pos, t, false),
                built.sim.is_in_range_at_margin(me, u.pos, t, true),
                u.combat.mandatory,
                built.sim.find_collision_forced(
                    built
                        .sim
                        .units
                        .iter()
                        .position(|x| std::ptr::eq(x, u))
                        .unwrap_or(usize::MAX),
                    u.pos,
                ),
            )
        }),
        u.path,
        u.orders,
        clocks.join(" ")
    );
    // A caster's craft (`mana_burn`) and its cast clock (`spell_time`),
    // the dump's own two fields (item 1302: a General's decoy cast).
    if u.mana_burn != 0 || u.spell_time != 0 {
        eprintln!(
            "  f{frame} {who}/{o} craft {} spell_time {}",
            u.mana_burn, u.spell_time
        );
    }
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

/// The building a dumped `BUILDDATA` or `CITY` record names: this crate's
/// **live** building of that owner and object number, else a closed one
/// that held it. A number is reused once `Objects::find_free` hands it on
/// (`Sim::building_slot_reused`), so the first building that ever held it
/// can be a dead site while a later one stands on the ground the record
/// describes — East Indies at Toughest destroys a camp on frame 976 and
/// places another as `2009` on 1576, and until item 989 the record was
/// compared with the dead one (`docs/AI.md` §82). A dumped record's slot
/// is not an identity; what the slot holds is.
pub(crate) fn link_building(sim: &sim::Sim, who: i64, o: i64) -> Option<usize> {
    let held = |x: &sim::Building| i64::from(x.owner) == who && i64::from(x.index) == o;
    sim.buildings
        .iter()
        .position(|x| x.alive && held(x))
        .or_else(|| sim.buildings.iter().position(held))
}

#[cfg(test)]
pub(crate) mod tests {
    mod cases_00;
    mod cases_01;
    mod cases_02;
    mod cases_03;
    mod cases_04;
    mod cases_05;
    mod cases_06;
    mod cases_07;
    mod cases_08;

    use super::*;

    use crate::diff::testkit::*;

    use crate::testenv::{dump, install};

    /// [`link_building`] (item 989): a number a closed site gave up and a
    /// later building took links to the **live** one; a number only a
    /// closed building ever held still links, so a record the dump prints
    /// for a building this crate has closed is compared rather than lost.
    #[test]
    fn a_dumped_building_links_to_the_live_holder_of_its_number() {
        cases_00::a_dumped_building_links_to_the_live_holder_of_its_number();
    }

    /// **The Great Lakes word's own window, walked by the shared
    /// instrument** (parked 527): the chain every widening past run196
    /// replays, from run53's start through twelve captures, and the
    /// word's block with two on either side. `run202_s_word_frame_is_
    /// widened_whole` walks the whole window from the same chain;
    /// `coverage`'s compared pin walked only these blocks, with the
    /// recorder on, until item 1061 moved it to the second pair's word
    /// (`second::great_lakes_word_window`). **The chain is one place** so
    /// that the capture a moved word adds is added once.
    pub(crate) fn great_lakes_word_chain() -> Vec<(&'static str, i64)> {
        vec![
            (
                "gamelog-run123-greatlakes-marketword.txt",
                WIDENING_GREAT_LAKES_MIRROR.0,
            ),
            ("gamelog-run125-greatlakes-armyidle.txt", 11_440),
            ("gamelog-run130-greatlakes-armytwo.txt", 11_560),
            ("gamelog-run135-greatlakes-crossing.txt", 11_800),
            ("gamelog-run136-greatlakes-detour.txt", 11_860),
            ("gamelog-run163-greatlakes-upgradeword.txt", 11_960),
            ("gamelog-run174-greatlakes-civicword.txt", 12_400),
            (
                "gamelog-run178-greatlakes-copyword.txt",
                WIDENING_GREAT_LAKES_CIVIC.1 + 1,
            ),
            (
                "gamelog-run192-greatlakes-birthword.txt",
                WIDENING_GREAT_LAKES_COPY.1 + 1,
            ),
            (
                "gamelog-run196-greatlakes-patriotword.txt",
                WIDENING_GREAT_LAKES_BIRTH.1 + 1,
            ),
            (
                "gamelog-run202-greatlakes-mirrorword.txt",
                WIDENING_GREAT_LAKES_PATRIOT.1 + 1,
            ),
            (
                "gamelog-run211-greatlakes-wonderword.txt",
                WIDENING_GREAT_LAKES_MIRROR.1 + 1,
            ),
            (
                "gamelog-run218-greatlakes-escortword.txt",
                WIDENING_GREAT_LAKES_WONDER.1 + 1,
            ),
            (
                "gamelog-run226-greatlakes-giveupword.txt",
                WIDENING_GREAT_LAKES_ESCORT.1 + 1,
            ),
            // Sized to the word (item 785): nothing is on disk between
            // run226's last block, 17350, and this one's first, 20500. It
            // takes over from 17351 as each capture does from the last
            // one's tail, so run226's partial `!quit` block is never read.
            (
                "gamelog-run243-greatlakes-standword.txt",
                WIDENING_GREAT_LAKES_GIVEUP.1 + 1,
            ),
        ]
    }

    #[test]
    fn indexed_replay_preserves_reports_with_sibling_corrections() {
        cases_00::indexed_replay_preserves_reports_with_sibling_corrections();
    }

    #[test]
    fn observing_a_replay_preserves_the_complete_report() {
        cases_00::observing_a_replay_preserves_the_complete_report();
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
        cases_00::run57_s_four_thousand_frames_stand_where_the_original_s_do();
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
        cases_00::run58_s_five_thousand_frames_stand_where_the_original_s_do();
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
        cases_00::run69_s_three_thousand_frames_stand_where_the_original_s_do();
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
        cases_00::run71_s_five_thousand_frames_reach_past_the_word();
    }

    /// Run12's per-frame words, read straight from the dump (`docs/SYNC.md`
    /// §1): the `end_frame` record inside each `FRAME n` block's `FULL
    /// DUMP`, keyed by the engine frame.
    #[test]
    fn run12_s_end_of_frame_words_are_read() {
        cases_00::run12_s_end_of_frame_words_are_read();
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
        cases_00::frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps();
    }

    #[test]
    fn run14_s_frames_match_the_trace_draw_for_draw() {
        cases_00::run14_s_frames_match_the_trace_draw_for_draw();
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
        cases_01::run33_s_long_trace_says_where_the_word_parts();
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
        cases_01::run53_s_24000_frames_put_the_ceiling_where_run33_did();
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
        cases_01::run53_s_8186_is_find_target_s_probe_and_its_ring_walks();
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
        cases_01::great_lakes_8186_sends_the_probe_s_six_where_the_original_does();
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
        cases_01::great_lakes_8186_plans_the_chase_and_8187_delays_the_blocked_one();
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
        cases_01::great_lakes_8186_snaps_the_probe_s_walkers_onto_their_cell_centres();
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
        cases_01::great_lakes_8442_musters_the_army_where_the_original_does();
    }

    /// **The sweep, read from inside, against the original's own**
    /// (item 456, `docs/COLLISION.md` §9). run116 is the first
    /// `RON_COLLIDE_PROBE` capture: five proxies over
    /// `Unit::detect_unit_collision` and the four functions inside it,
    /// so the four steps of a sweep are on the record where no gamelog
    /// category can put them — §5.4's snap arm clears `collide_o` and
    /// `collide_who` two instructions after the probe returns.
    ///
    /// The comparison is a **rendered line against a rendered line**,
    /// and it is taken where the original's bracket sits rather than at
    /// a frame boundary: [`sim::Sim::sweep_watch`] is what makes that
    /// possible, and §9.2's note on `1/33` is what makes it necessary —
    /// read one instruction too early this crate's probe answers
    /// `(892, 469)` and the item names the occupancy index instead of
    /// the soft table.
    ///
    /// **Made to fail on purpose before landing** by restoring the seam
    /// this item closed: with `same_group_soft`'s `search.is_some()`
    /// clause dropped, the line reads `soft` where it now reads
    /// `hard 1/31`, and the word falls back to 10161.
    #[test]
    fn run116_s_sweep_is_the_original_s() {
        cases_01::run116_s_sweep_is_the_original_s();
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
        cases_01::great_lakes_9134_is_the_snap_arm_s_blocked_stand();
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
        cases_01::run97_s_build_queues_are_the_original_s();
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
        cases_02::run97_s_window_orders_are_the_original_s();
    }

    /// The frame Great Lakes' AI sends [`PROBE_SENT`]'s six from its base
    /// to the far south-west, and the one world path that carries all six.
    const PROBE_PLAN_FRAME: i64 = 8186;

    /// The entries of the probe's plan that part, per member — **the rows,
    /// not their number**, so a route that changes fails here whichever way
    /// it moves.
    ///
    /// Five of the six are empty and have been since item 354. `1/40`'s
    /// twenty-two arrived with item 471 and are **newly compared ground,
    /// not a regression**: until then this crate held 47 entries where the
    /// original holds 94, so the comparison stopped at the out leg and the
    /// return leg was not in it at all. Now both sides hold 94 and the out
    /// leg — every entry from 42 up — still agrees exactly.
    ///
    /// **What the twenty-two are.** Every one shares the original's `x` and
    /// sits 768 or 1536 south of its `y` — one or two cells — over three
    /// stretches that each re-converge within a dozen waypoints. Same start,
    /// same end, same `x` progression: a tie-break in the cost function and
    /// not a different route. ~~It is the same residue the long word stands
    /// on, where the original spends 204 draws at
    /// `PathFinder::calc_road_cost+0x46` and this crate spends six
    /// (`docs/ORDERS.md` §17.5), and closing that is what would empty this.~~
    ///
    /// **Item 475 falsified that.** `calc_road_cost` is the road search's
    /// — one caller in the executable, `astar_caravan_road` under
    /// `find_road` — and [`PROBE_PLAN_FRAME`] prices no road node at all,
    /// so the word's draws and these twenty-two are two residues
    /// (`docs/ORDERS.md` §18, and
    /// [`great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none`]).
    /// A tie is still the shape these have; **which** cost function breaks
    /// it is open, and nothing on this disk names it. §17.6's second and
    /// third falsifiers are the tests for whoever takes them.
    ///
    /// **Item 899 emptied it.** Neither residue named above: the walk home
    /// is a group plan made while `1/40`'s current order is the `ATTACK`,
    /// and `AttackOrder::is_attack` answers 1, so the original plans it
    /// without the `army` mode (`docs/PATHFINDER.md` §29). All 2,319 priced
    /// steps of the tick are the original's
    /// (`run240_s_tick_8186_prices_the_walk_home_as_a_citizen_s`), and so
    /// is all of `1/40`'s 94-entry stack.
    const PROBE_PLAN_PARTED: &[(i64, i64, &[usize])] = &[];

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
        cases_02::great_lakes_8186_plans_the_probe_s_route_the_original_s_way();
    }

    /// **The frame `0/2006`'s road replan searches on** — Great Lakes'
    /// word until item 478, and a fact about run53's trace rather than
    /// about this crate, so it is pinned as its own number and not as
    /// [`LONG_WORD_GREAT_LAKES`], which has now moved past it.
    ///
    /// `(10_234 + 2_006) % 16 == 0` is the whole of why it is this
    /// frame: the human's six buildings are flagged together when
    /// `0/2004` falls on 10230, and each replans on the frame its own
    /// object number picks out — `12_240 - o` for every one of them
    /// (`docs/ROADS.md` §1.2).
    const GREAT_LAKES_REGEN_SEARCH: i64 = 10_234;

    /// `PathFinder::calc_road_cost@00686300+0x46` — the road search's
    /// per-node jitter, and [`sim::roads::SITE_COST`]'s own address in
    /// [`trace::SITES`](crate::trace::SITES).
    const ROAD_COST: u32 = 0x0068_6346;
    /// `PathFinder::astar_caravan_road@00685990+0x52b` — the only caller
    /// of [`ROAD_COST`]'s function anywhere in the executable.
    const ROAD_COST_UP: u32 = 0x0068_5ebb;
    /// `PathFinder::find_road@00688a40+0x3a8` — and the only caller of
    /// *that*, reached from `Caravan::process@0073e000`,
    /// `Caravan::build_road@0073db10` and `BuildType::place_roads@0063c580`
    /// and from nothing else.
    const ROAD_COST_UP2: u32 = 0x0068_8de8;

    /// **The word's excess draws are a road search, and the frame the
    /// probe's route is planned on spends none of them** — item 475,
    /// `docs/ORDERS.md` §18.
    ///
    /// `docs/ORDERS.md` §17.6 read Great Lakes' long word and
    /// [`PROBE_PLAN_PARTED`]'s twenty-two entries as one residue standing
    /// in two places: a tie inside `calc_road_cost` broken one way by the
    /// original and another by this crate. This test is that reading's
    /// falsifier, and it fires — so what it pins is the **negative**, in
    /// the two places the reading needed a positive:
    ///
    /// - **8186 spends no road-cost draw at all.** It is the frame
    ///   `Group::action_move_near` plans the probe's one world path on
    ///   ([`PROBE_PLAN_FRAME`]), so a cost function that is never called
    ///   there cannot be what breaks that path's ties. The nearest road
    ///   searches are 49 frames after it and 59 before.
    /// - **10234's are the road search, by their own `ebp` chain**, every
    ///   one of the 198: `calc_road_cost+0x46 < astar_caravan_road+0x52b
    ///   < find_road+0x3a8`. `find_road` is reached only from
    ///   `Caravan::process`, `Caravan::build_road` and
    ///   `BuildType::place_roads`; no unit path can enter it.
    ///
    /// And the word's own parting draw is that search's first, at index
    /// **4** of the frame — the two streams agree on the four draws
    /// before it, which is what
    /// `run53_s_24000_frames_put_the_ceiling_where_run33_did` reported as
    /// `ours 6 theirs 204 — at 4`. So the word was held by a road search
    /// the original runs and this crate did not run at all: a *missing*
    /// behaviour rather than a divergent one.
    ///
    /// **Item 478 closed it, and the third caller is the one that runs**
    /// (`docs/ROADS.md` §1.2, `docs/ORDERS.md` §19). `BuildType::
    /// place_roads` it is, from `Build::process`'s deferred
    /// `build_masks & 0x100` arm — and the two frames are **two
    /// buildings**, not one search carried over: `0/2006` on 10234 and
    /// `0/2005` on 10235, each on the frame `(frame + o) % 16 == 0`
    /// picks out. What flagged them is a death four frames earlier —
    /// the human's farm `0/2004` falls on 10230 and
    /// `Build::remove_from_city` calls `City::regen_roads` on what is
    /// left of the city. This crate had only `Build::activate`'s call
    /// to that function, so no building was ever flagged and no search
    /// ever ran; with the second writer in, both frames spend the
    /// original's draws — 204 and 78 — node for node.
    ///
    /// The frames here stay pinned as [`GREAT_LAKES_REGEN_SEARCH`]
    /// rather than as the word, which has moved past them.
    ///
    /// **The frame list is the assertion, not the counts**, for the same
    /// reason [`PROBE_PLAN_PARTED`] pins rows rather than a number — a
    /// change that moves the original's road schedule, or a trace read
    /// off the wrong capture, fails here whichever way it moves, and
    /// 8186's absence from the list is the claim.
    ///
    /// **Made to fail first**, both halves. `road(8_235)` in place of
    /// `road(PROBE_PLAN_FRAME)` — a frame that really does price road
    /// nodes — reddens the emptiness assertion; one bit off
    /// [`ROAD_COST_UP`] makes all 198 of the word's draws strays and
    /// reddens the chain. Each was run, seen red, and put back.
    #[test]
    fn great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none() {
        cases_02::great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none();
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
        cases_02::run54_s_24000_frames_are_where_the_second_map_s_word_now_parts();
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
        cases_02::run64_s_window_clocks_are_the_original_s();
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
        cases_02::run94_s_window_clocks_are_the_original_s();
    }

    /// **run100's whole frame record, across the AI headline's own
    /// frame** — Great Lakes `[9340, 9511]`, item 408.
    ///
    /// **Nothing had ever widened 9510.** run97's window stops at 9349
    /// and run109's is `[9420, 9480)`, so the frame the long capture
    /// scores on had no compared record behind it at all: every widening
    /// on this map ran out below it. run100 is the capture that covers
    /// it — `[9340, 10899]` of the same seeded game, at run97's detail —
    /// and this walks [`compare`] over its blocks from the first to the
    /// word's own: every unit's position and order stack, every
    /// building's identity and queue slot, every field of every `CITY`.
    ///
    /// **What the word is.** 9510's two extra draws are
    /// `Guy::init_real+0x52` and `Guy::set_anim+0x97a <
    /// Unit::go_inside+0x280` — a birth and a seating, the signature
    /// `docs/CITIES.md` §6.5.2 gave the map's *first* scholar on 8272.
    /// This is its **sixth**, `1/51`, and the word block says so with
    /// nothing else beside it: of 172 blocks the only object either side
    /// holds alone is that one, on that one frame.
    ///
    /// **What the cause is, and it is 128 frames earlier.** The only
    /// record that newly parts anywhere inside the window is the
    /// production queue, on **9382**, and it parts on two buildings at
    /// once: the original queues one Scholar at University `1/2019` and
    /// this crate queues two Citizens at Village `1/2007`. The
    /// original's job then runs `job_counter` +100 a frame from 100 to
    /// 12750 and completes into the birth
    /// ([`run100_s_scholar_job_runs_to_the_word`]).
    ///
    /// **And the draw stream cannot see it.** 9382 costs eight draws on
    /// both sides, entry for entry — two `Leader::use_market+0x1ed`, two
    /// `Leader::make_stuff+0x221`, two `+0x63d`, the farm animal and the
    /// farm clock. `+0x221` counts the make list's slots holding the
    /// head's type and `+0x63d` the slot loop's expiries (§41, §42);
    /// **a purchase itself draws nothing**. So the two sides buy
    /// different things out of identically-shaped lists and no count
    /// says so. That is the case for widening the record rather than the
    /// frame, made on the frame the headline stands on.
    ///
    /// Every other residue in the window was already standing when it
    /// opened — three positions, seventeen order stacks and twenty city
    /// fields, all first parting on the window's own first block — so
    /// they are pinned as sets rather than counted, and the sets may
    /// only shrink.
    #[test]
    fn run100_s_word_frame_is_the_original_s() {
        cases_02::run100_s_word_frame_is_the_original_s();
    }

    /// One order disagreement's **field**, not merely "order"
    /// (`docs/AI.md` §54.2). An order row keyed on the unit alone reports
    /// the first field to part and hides every other for the rest of the
    /// run: `1/38` carries a `GROUPORDER` `id` residue from block 9340,
    /// and it swallowed the `coll_x`/`coll_y` pair on the word's own
    /// frame — the one field of the record that says in so many words
    /// what the original refused.
    /// **The widening [`LONG_WORD_GREAT_LAKES`] owes — every record run100
    /// carries on the AI headline's own frame, every field, both
    /// directions** (item 448, `docs/AI.md` §54).
    ///
    /// [`Self::run100_s_word_frame_is_the_original_s`] is the sibling and
    /// it is not this: it runs [`compare`] and reads four of its
    /// categories, keyed by unit. This walks the same 836 blocks and
    /// compares **every** field either side holds — the `compare`
    /// categories keyed by *field* rather than by unit, plus the twenty
    /// rows of the `UNITDATA` and `GUY` records that no test on this map
    /// had ever read: `dest_angle`, `form`, `form_mod`, `myspeed`,
    /// `myhits`, `stance`, `orders_x/y`, `tolerance`, `path_recursion`,
    /// `idle`, the guy clock whole, and the collision block **ungated by
    /// the position**, which is where the word's own answer turned out to
    /// be.
    ///
    /// **The gating is the point.** `compare` takes the collision block,
    /// the angles, `mylos` and `start_dist` only on unit-frames whose
    /// positions already agree — right, for a residue count, and blind on
    /// exactly the frame a position parts. `half_step` is a dumped field
    /// (`unit_masks & 0x100000`) and it is one of the two that name the
    /// word.
    ///
    /// **The word block is `1/38` and nothing else** — fifteen rows, one
    /// unit, and the other 103 units, 29 buildings and 3 cities of the
    /// block agree field for field. What the fifteen say, in the dump's
    /// own coordinates:
    ///
    /// | field | ours | theirs |
    /// | --- | --- | --- |
    /// | `pos` | `(42774, 22584)` | `(42754, 22582)` |
    /// | `order:coll` | `None` | `(42774, 22584)` |
    /// | `collide_guy` | `-1` | `0` |
    /// | `half_step` | `1` | `0` |
    /// | `g.cur_anim` | `8` | `0` |
    /// | `g.cur_time` / `g.end_time` | `4` / `15` | `1` / `33` |
    /// | `g.last_time` | `3` | `0` |
    /// | `g.last_speed` / `g.avg_speed` | `20` / `18` | `0` / `13` |
    /// | `g.stopped` | `0` | `1` |
    ///
    /// The original **refused the point this crate stepped onto**:
    /// `coll_x`/`coll_y` hold `(42774, 22584)` from block 10162 onward,
    /// `collide_guy` is stamped 0 and never cleared, `collide_o` and
    /// `collide_who` are back at −1, the waypoint is consumed without a
    /// step (`length` 2 → 1, `dest` 1 → 0) and the idle is rolled. That is
    /// `docs/COLLISION.md` §5.4's snap arm, field for field, and the draw
    /// stream says the same thing from the other side:
    /// [`sim::anim::SITE_SNAP_BLOCKED`] is `Guy::set_anim+0x97a <
    /// Unit::move_step+0x4e2`, which is where the original's eighth draw
    /// is spent.
    ///
    /// **Made to fail on purpose before landing**, both ways that matter:
    /// keying the order rows by unit again drops `order:coll` from the
    /// set — which is how that row was found at all — and restoring
    /// `compare`'s position gate on the collision block drops
    /// `half_step` and `collide_guy` together. The set is keyed on
    /// [`LONG_WORD_GREAT_LAKES`], so a word that moves fails it by
    /// construction: block 10163's own three rows are `g.angle[0]`,
    /// `g.des_angle[0]` and `heading`, and nothing of 10162's.
    ///
    /// **The record grew on item 463, and the field it grew by is the one
    /// the item turned on.** `recharging` — `UnitData +0xae`, the reload
    /// clock — was dumped on every block and compared on none, so the
    /// squad that reads above as a pathing divergence (three units, 44
    /// path entries against one) could not be seen for what it was: six
    /// raiders dropping an `ATTACKORDER` on a building that had just
    /// died, each two to eight frames before the original's reload let
    /// it. With the field in, it agrees on every AI unit of all 909
    /// blocks, which is what makes the kill's *timing* a measurement.
    /// The word's own block is 10234 now and it is two units: the human's
    /// citizen `0/5`, which is the word, and `1/28`, one block ahead of
    /// the original on the walk its dropped order freed.
    ///
    /// **Item 464 closed the citizen and widened the record again.**
    /// `0/5`'s twenty-two rows are three: its flight is the original's,
    /// destination and bearing and frame (`docs/COMBAT.md` §34), and
    /// what is left of it is the `GATHERORDER` this crate's
    /// `think_peasant` hands the idle citizen under the flight, with
    /// `0/2001`'s `gather_down` chain carrying it. The widening itself
    /// grew by 54,000 comparisons and **lost a false row**: `myhits` is
    /// the record's *maximum* and `damage` the accumulator beside it, so
    /// comparing this crate's current health against `myhits` reported
    /// every wound as a divergence — `hits_left` and `myhits` are both
    /// rows now and both agree on all 909 blocks.
    /// **The human's idle wait is twelve frames, and the citizen goes back
    /// to the *same* job** — item 494, `docs/ORDERS.md` §21.
    ///
    /// `think_peasant@005f5760:16` reads `LeaderOptions +0x8`
    /// (`peasants_wait`) as the **index** of a wait and not as the wait:
    /// `1 → 7, 2 → 0xc, 3 → 0x11, 4 → 0x20, 5 → 0x3e`, `default → 2`.
    /// Every capture on disk that prints leader options prints
    /// `peasants_wait 2`, so the wait is **12** — and this crate used the
    /// index, 2, until this item.
    ///
    /// **What the dump says, which is the whole assertion.** run100's
    /// `0/5` empties its order list on block 10294, raises `idle` to 1 on
    /// 10295 and 2 on 10296, and then steps it once every sixteen frames
    /// phased by `o` (`check_idle@006032c0`: `(o + game->frame) & 15`;
    /// the block number is `frame + 1`, so `o == 5` steps on 10300,
    /// 10316, … ). On **10444** `idle` reaches **12**, `gather_down`
    /// becomes 2 and `orders_x/y` turn to `(4056, 28776)` — the same
    /// building and the same point this crate used to send it to on
    /// 10295. The original is not making a different decision; it is
    /// making the same one 150 frames later.
    ///
    /// So the counter is compared frame for frame across the whole
    /// wait, not only at its end: a wait of any other length, or a
    /// cadence of any other period, parts somewhere in the hundred and
    /// fifty blocks between. **Made to fail on purpose**: with
    /// `idle_wait` back at 2 the first parting is block **10295** —
    /// `(10295, 0, 4056, 28776)` against `(10295, 1, 792, 31800)`, the
    /// citizen already walking to its next job on the frame the original
    /// first counts it idle. Note where that is *not*: the wrong wait
    /// fires on 10234 and these three fields say nothing for sixty
    /// blocks, because `orders_x/y` follow the leading transit move and
    /// the flight is still it. Only `orders.len` sees it, which is why
    /// the widening's 10234 pin is the row that names the cause.
    #[test]
    fn run100_s_citizen_waits_twelve_idle_frames_for_the_same_job() {
        cases_02::run100_s_citizen_waits_twelve_idle_frames_for_the_same_job();
    }

    /// **run123 — Great Lakes' word 11185, widened whole, both directions**
    /// (item 520). run123 is run100's game at run100's detail with
    /// `LEADERS=9`, over `[10760, 11459]`: 140 blocks shared with run100
    /// and 560 past it, the only dump that reaches the word. Every record
    /// [`widen_block`] reads on every unit, and the **leader record**
    /// whole — [`crate::diff::leader::rows`] against
    /// [`crate::diff::leader::theirs`], plus the four `epoch_get(scan)`
    /// lines nothing parsed, which are `use_market`'s commerce level — for
    /// both players on every block. Each key's first parting block is
    /// kept, with the value diff beside it.
    ///
    /// The word's frame enters on block **11185** and leaves on **11186**
    /// (the trace's frame `f` writes block `f + 1`), so both are the
    /// word's blocks; the window opens 425 blocks under it because a
    /// market frame spends what the purse has been doing since the last
    /// rotation, and the one before.
    #[test]
    fn run123_s_word_frame_is_widened_whole() {
        cases_02::run123_s_word_frame_is_widened_whole();
    }

    /// **run125 — Great Lakes' word 11531, widened whole, both directions**
    /// (item 533). run125 is run123's game at run123's detail over
    /// `[11440, 11599]`, twenty blocks shared with run123 and the only dump
    /// that reaches the word. The walk runs **backwards across both
    /// captures** — run123 from [`WIDENING_GREAT_LAKES_ARMY`]'s floor, 11250,
    /// before army 1's squad takes its group order, and run125 from its own
    /// first block — so a cause that spends no draw on the march is still
    /// a row here. Every record [`widen_block`] reads on every unit, and
    /// the leader record whole for both players, on every block; each
    /// key's first parting block is kept, with the value diff beside it.
    ///
    /// The word's frame enters on block **11531** and leaves on **11532**
    /// (the trace's frame `f` writes block `f + 1`).
    #[test]
    fn run125_s_word_frame_is_widened_whole() {
        cases_02::run125_s_word_frame_is_widened_whole();
    }

    /// **run130 — Great Lakes' word 11757, widened whole, both directions**
    /// (item 554). run130 is run125's game at run125's detail over
    /// `[11560, 11799]`, forty blocks shared with run125 and the only dump
    /// that reaches the word. The walk runs **backwards across three
    /// captures** — run123 from [`WIDENING_GREAT_LAKES_ARMY_TWO`]'s floor,
    /// 11400, under army 2's first parting, run125 from its own first block,
    /// and run130 from 11560 — so a cause that spends no draw on the march is
    /// still a row here. Every record [`widen_block`] reads on every unit,
    /// and the leader record whole for both players, on every block; each
    /// key's first parting block is kept, with the value diff beside it.
    ///
    /// The word's frame enters on block **11757** and leaves on **11758**
    /// (the trace's frame `f` writes block `f + 1`).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes — the first-parting map keeps a key's first
    /// block only, and cannot say whether it re-agreed.
    #[test]
    fn run130_s_word_frame_is_widened_whole() {
        cases_03::run130_s_word_frame_is_widened_whole();
    }

    /// **run134 — army 2's pool list, block for block** (item 557,
    /// `docs/GROUPS.md` §23). run134 is run125's game with `GROUPS=1` over
    /// [11416, 11519], the first dump on this map above run92's 7689 to
    /// print the pool's own lists, and the widening beside it had only
    /// each unit's back-pointer. Every block compares group 66's member
    /// list — the original's `GROUPDATA id 66`, this crate's
    /// [`sim::Sim::pool_list`] — and the pointer of each of army 2's six
    /// units, both directions.
    ///
    /// The two frames the chain turns on are pinned by value: on 11425 the
    /// list is `{60, 61, 64}`, because `Group::add`'s own `get_num`
    /// normalizes the squad's head and middle out as it adds the next
    /// figure; and on 11513 it is `{60, 61, 65, 62, 63, 64}` with `1/64`
    /// naming nothing, because `Group::sort` killed the stray follower and
    /// re-added its squad whole. **Made to fail on purpose** twice: with
    /// `Group::add`'s per-step normalize removed (the list is the whole
    /// squad from 11425) and with the sort removed (11513's list is the
    /// old order and `1/64` still names 66).
    #[test]
    fn run134_s_pool_list_is_the_original_s() {
        cases_03::run134_s_pool_list_is_the_original_s();
    }

    /// **run135 — Great Lakes' word 11806, widened whole, both directions**
    /// (item 560). run135 is run134's line (run125's detail with `GROUPS=1`
    /// and without `DEATHS`) over `[11760, 11859]`, forty blocks shared
    /// with run130 and the only dump that reaches the word. The walk runs
    /// **backwards across four captures** — run123 from
    /// [`WIDENING_GREAT_LAKES_CROSSING`]'s floor, 11400, then run125,
    /// run130 and run135 from 11800 — so a cause that spends no draw is
    /// still a row here. Every record [`widen_block`] reads on every unit,
    /// the leader record whole for both players, and on run135's blocks
    /// **every player-1 pool list** (`GROUPDATA`'s members against
    /// [`sim::Sim::pool_list`]); each key's first parting block is kept,
    /// with the value diff beside it.
    ///
    /// The word's frame enters on block **11806** and leaves on **11807**
    /// (the trace's frame `f` writes block `f + 1`).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_UNIT_TABLE=<lo>-<hi>` with
    /// `RON_DEBUG_ROWS=<lo>-<hi>` prints player 1's units `lo..=hi`, both
    /// sides, on every block of the rows window.
    #[test]
    fn run135_s_word_frame_is_widened_whole() {
        cases_03::run135_s_word_frame_is_widened_whole();
    }

    /// **run136 — Great Lakes' word 11903, widened whole, both directions**
    /// (item 563). run136 is run135's line over `[11840, 11959]`, twenty
    /// blocks shared with run135 and the only dump that reaches the word.
    /// The walk runs **backwards across five captures** — run123 from
    /// [`WIDENING_GREAT_LAKES_DETOUR`]'s floor, 11400, then run125, run130,
    /// run135 and run136 from 11860 — so a cause that spends no draw is
    /// still a row here. Every record [`widen_block`] reads on every unit,
    /// the leader record whole for both players, and on run135's and
    /// run136's blocks **every player-1 pool list**; each key's first
    /// parting block is kept, with the value diff beside it.
    ///
    /// The word's frame enters on block **11903** and leaves on **11904**
    /// (the trace's frame `f` writes block `f + 1`).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_UNIT_TABLE=<lo>-<hi>` with
    /// `RON_DEBUG_ROWS=<lo>-<hi>` prints player 1's units `lo..=hi`, both
    /// sides, on every block of the rows window.
    #[test]
    fn run136_s_word_frame_is_widened_whole() {
        cases_03::run136_s_word_frame_is_widened_whole();
    }

    /// One of Great Lakes' windows walked whole, both directions, across a
    /// **chain of captures** — run136's walk, made a helper when run163
    /// needed the same one (item 571). `chain` is each capture with the
    /// first block the walk reads it from; below each, the walk reads the
    /// capture before it. From run53's start with the siblings' heads:
    /// every record [`widen_block`] reads on every unit and building of
    /// every player, the leader record whole for both players, and from
    /// `pools` on every player-1 pool list. Each key's first parting block
    /// is kept with the value diff beside it; every row parting within two
    /// blocks of a `near` block is kept whole in `standing`, and every
    /// figure whose animation changes on either side there is listed
    /// (`docs/COMBAT.md` §44.2.1).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_STANDING=<lo>-<hi>` every row parting on
    /// each block of the window.
    pub(crate) fn widen_great_lakes(
        name: &str,
        chain: &[(&str, i64)],
        window: (i64, i64),
        pools: i64,
        near: &[i64],
    ) -> Option<Widened> {
        widen_great_lakes_on(
            (
                "gamelog-run53-greatlakes-24k-trace.txt",
                "rontrace-run53.log",
            ),
            name,
            chain,
            window,
            pools,
            near,
            false,
        )
    }

    /// [`widen_great_lakes`] on another Great Lakes game's start: the
    /// second pair's run347 (item 971) is run53's lobby at Toughest.
    pub(crate) fn widen_great_lakes_on(
        base: (&str, &str),
        name: &str,
        chain: &[(&str, i64)],
        window: (i64, i64),
        pools: i64,
        near: &[i64],
        records: bool,
    ) -> Option<Widened> {
        widen_on_siblings(
            SIBLING_DUMPS,
            false,
            base,
            name,
            chain,
            window,
            pools,
            near,
            records,
        )
    }

    /// [`widen_great_lakes_on`] on another map's start: `siblings` are the
    /// dumps its head is borrowed from, and `gaia` adds [`widen_gaia`]'s
    /// rows. Great Sahara (item 1066) stands up on run381 and has herds.
    #[expect(
        clippy::too_many_arguments,
        reason = "widen_great_lakes_on's inputs plus the map's siblings and gaia"
    )]
    pub(crate) fn widen_on_siblings(
        siblings: &[&str],
        gaia: bool,
        base: (&str, &str),
        name: &str,
        chain: &[(&str, i64)],
        window: (i64, i64),
        pools: i64,
        near: &[i64],
        records: bool,
    ) -> Option<Widened> {
        widen_on_siblings_with_checkpoints(
            siblings,
            gaia,
            base,
            name,
            chain,
            window,
            pools,
            near,
            records,
            &[],
        )
        .map(|(whole, _)| whole)
    }

    /// Snapshot cumulative results at the original tests' endpoints. The sim,
    /// source selection, and comparisons still execute once per block.
    #[expect(
        clippy::too_many_arguments,
        reason = "the existing walk plus checkpoint requests"
    )]
    fn widen_on_siblings_with_checkpoints(
        siblings: &[&str],
        gaia: bool,
        base: (&str, &str),
        name: &str,
        chain: &[(&str, i64)],
        window: (i64, i64),
        pools: i64,
        near: &[i64],
        records: bool,
        checkpoints: &[(i64, &[i64])],
    ) -> Option<(Widened, Vec<Widened>)> {
        let walk = |reuse, prefix| {
            widen_walk(
                siblings,
                gaia,
                base,
                name,
                chain,
                window,
                pools,
                near,
                records,
                checkpoints,
                reuse,
                prefix,
            )
        };
        let result = if std::env::var_os("RON_VERIFY_ORACLE").is_some() {
            let (actual, notes) = compared::recorded(|| walk(true, true));
            let (fresh, fresh_notes) = compared::recorded(|| walk(false, false));
            assert_eq!(
                actual, fresh,
                "{name}: oracle changed the complete report/checkpoints"
            );
            assert_eq!(notes, fresh_notes, "{name}: oracle changed compared fields");
            eprintln!(
                "{name}: oracle matches complete fresh report, checkpoints and compared fields"
            );
            actual?
        } else if std::env::var_os("RON_VERIFY_PREFIX_REUSE").is_some() {
            let (actual, notes) = compared::recorded(|| walk(true, true));
            let (fresh, fresh_notes) = compared::recorded(|| walk(true, false));
            assert_eq!(
                actual, fresh,
                "{name}: prefix reuse changed the complete report"
            );
            assert_eq!(
                notes, fresh_notes,
                "{name}: prefix reuse changed compared fields"
            );
            eprintln!(
                "{name}: prefix reuse matches complete fresh report, checkpoints and compared fields"
            );
            actual?
        } else {
            walk(true, true)?
        };
        // Explicit migration audit only: normal gates must exercise the reused
        // reader alone, so legacy reads cannot fill gaps in coverage recording.
        if std::env::var_os("RON_VERIFY_FRAME_REUSE").is_some() {
            assert_eq!(
                result,
                walk(false, false).expect("legacy audit needs the same fixtures"),
                "{name}: reused and legacy frame reads differ"
            );
            eprintln!("{name}: frame reuse matches the complete legacy report and checkpoints");
        }
        Some(result)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the existing walk plus a legacy audit mode"
    )]
    fn widen_walk(
        siblings: &[&str],
        gaia: bool,
        (base, base_trace): (&str, &str),
        name: &str,
        chain: &[(&str, i64)],
        window: (i64, i64),
        pools: i64,
        near: &[i64],
        records: bool,
        checkpoints: &[(i64, &[i64])],
        reuse: bool,
        prefix: bool,
    ) -> Option<(Widened, Vec<Widened>)> {
        let mut oracle_store = crate::diff::oracle::Store::new(reuse);
        let profiling = std::env::var_os("RON_PROFILE_REPLAY").is_some();
        let started = std::time::Instant::now();
        let mut tick_time = std::time::Duration::ZERO;
        let mut stages = [std::time::Duration::ZERO; 4];
        assert!(checkpoints.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(checkpoints.iter().all(|(tail, points)| {
            (window.0..=window.1).contains(tail) && points.iter().all(|p| near.contains(p))
        }));
        let mut snapshots = Vec::with_capacity(checkpoints.len());
        let inst = install()?;
        let Some(path) = dump(base) else {
            eprintln!("skipping: no {base} capture (set RON_GAMELOG_DIR)");
            return None;
        };
        let paths: Vec<Option<String>> = chain.iter().map(|(n, _)| dump(n)).collect();
        if paths.iter().any(Option::is_none) {
            eprintln!("skipping: {name}'s chain is not all on disk");
            return None;
        }
        let from: Vec<i64> = chain.iter().map(|(_, lo)| *lo).collect();
        let mut ixs: Vec<crate::capture::indexed::IndexedCapture> = paths
            .iter()
            .flatten()
            .map(|p| crate::capture::indexed::IndexedCapture::open(p).unwrap())
            .collect();
        let (first_block, tail) = window;
        let prefix = prefix && super::prefix::enabled();
        let loaded = if prefix {
            super::prefix::loaded(&inst)
        } else {
            std::sync::Arc::new(crate::load::load(&inst).unwrap())
        };
        let prefix_key = prefix.then(|| {
            let mut names = vec![base, base_trace];
            names.extend_from_slice(siblings);
            super::prefix::Key::new(inst.root(), &names)
        });
        let resumed = prefix_key
            .as_ref()
            .and_then(|key| super::prefix::take(key, first_block));
        let mut built = resumed.unwrap_or_else(|| {
            crate::diff::testkit::with_named_sibling_initials(siblings, |refs| {
                let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
                let text = source.read_replay_setup().unwrap();
                let log = Log::parse(&text);
                let mut init = log.replay_initial().unwrap();
                borrow_from_siblings(&mut init, refs);
                if let Some(t) = trace(base_trace) {
                    borrow_pasture(&mut init, &t);
                }
                build_sim(&loaded, &init, Tuning::RON)
            })
        });
        let resumed_at = built.sim.frame;
        let setup_time = started.elapsed();
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows, mut pool_lists) =
            (0usize, 0usize, 0usize, 0usize);
        let mut housed = 0usize;
        let row_walk: Vec<(i64, i64)> = std::env::var("RON_ROW_WALK")
            .map(|w| {
                w.split(',')
                    .filter_map(|u| {
                        let (a, b) = u.split_once('/')?;
                        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut walked: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
        let mut anims: BTreeMap<(i64, i64), (Option<i64>, Option<i64>)> = BTreeMap::new();
        let mut changed: Vec<(i64, i64, i64, bool, bool)> = Vec::new();
        let mut standing: BTreeMap<i64, BTreeMap<(i64, i64, String), String>> = BTreeMap::new();
        let is_near = |n: i64| near.iter().any(|b| (b - 2..=b + 2).contains(&n));
        // `RON_SWEEP=<frame>:<who>/<o>` — every collision sweep one unit
        // makes on one sim-frame, as run116's test prints it (item 1171).
        let sweep: Option<(i64, i32, i32)> = std::env::var("RON_SWEEP").ok().and_then(|v| {
            let (f, u) = v.split_once(':')?;
            let (w, o) = u.split_once('/')?;
            Some((f.parse().ok()?, w.parse().ok()?, o.parse().ok()?))
        });
        let mut gap_seen: BTreeMap<String, String> = BTreeMap::new();
        // Frame `f` writes block `f + 1`, so the window's last block is
        // frame `tail - 1`'s. `0..=tail` read block `tail + 1` too, which no
        // chain held until run211 followed run202 (item 722).
        for f in built.sim.frame..tail {
            if let Some((sf, w, o)) = sweep
                && sf == f
            {
                built.sim.sweep_watch = Some(sim::collide::SweepWatch::new(sf, w, o));
            }
            let tick_started = profiling.then(std::time::Instant::now);
            built.tick();
            if let Some(t) = tick_started {
                tick_time += t.elapsed();
            }
            if let Some((sf, _, _)) = sweep
                && sf == f
                && let Some(w) = built.sim.sweep_watch.take()
            {
                for l in w.rendered() {
                    eprintln!("  sweep {sf}: {l}");
                }
            }
            let n = f + 1;
            debug_gap(&built, n, &mut gap_seen);
            if n < first_block {
                continue;
            }
            let c = from.iter().rposition(|&lo| n >= lo).unwrap();
            let ix = &mut ixs[c];
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            // Decode once, then keep the same tree for leader/group records.
            // The second read/parse below exists only for the migration audit.
            let stage = profiling.then(std::time::Instant::now);
            let cached = oracle_store.enabled().then(|| oracle_store.read(ix, at));
            let reused_text = (reuse && cached.is_none()).then(|| ix.read_frame(at).unwrap());
            let reused_log = reused_text.as_deref().map(Log::parse_eager);
            let fresh_frame;
            let frame = if let Some(value) = &cached {
                &value.frame
            } else {
                fresh_frame = if let Some(log) = &reused_log {
                    crate::capture::indexed::IndexedCapture::frame_from_log(
                        log,
                        ix.frames()[at].number,
                    )
                    .unwrap()
                } else {
                    ix.frame_state(at).unwrap()
                };
                &fresh_frame
            };
            if let Some(t) = stage {
                stages[0] += t.elapsed();
            }
            let stage = profiling.then(std::time::Instant::now);
            blocks += 1;
            debug_watch(&built, n);
            debug_armies(&built, n);
            debug_leader(&built, n);
            debug_ammo(&built, n);
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (fr, rows) = widen_block(&built, frame, players, n, &mut here);
            compared += rows;
            housed += fr.inside_housed;
            if let Some(t) = stage {
                stages[1] += t.elapsed();
            }
            let stage = profiling.then(std::time::Instant::now);
            let legacy_text = (!reuse).then(|| ix.read_frame(at).unwrap());
            let legacy_log = legacy_text.as_deref().map(Log::parse);
            let flog = reused_log.as_ref().or(legacy_log.as_ref());
            let fresh_groups;
            let groups = if let Some(value) = &cached {
                &value.groups
            } else {
                fresh_groups = if records || n >= pools {
                    flog.unwrap()
                        .frames()
                        .into_iter()
                        .find(|(k, _)| *k == n)
                        .map(|(_, block)| crate::gamelog::groups(block))
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                &fresh_groups
            };
            if records {
                compared += crate::diff::second::widen_records_from_groups(
                    &built, frame, groups, players, n, &mut here,
                );
            }
            if n >= pools {
                let theirs: BTreeMap<i64, Vec<i64>> = groups
                    .iter()
                    .filter(|g| g.who == 1 && !g.members.is_empty())
                    .map(|g| (g.id - 64, g.members.iter().map(|m| m.o).collect()))
                    .collect();
                for s in 0..64u8 {
                    let ours: Vec<i64> = built
                        .sim
                        .pool_list(1, s)
                        .into_iter()
                        .map(i64::from)
                        .collect();
                    let t = theirs.get(&i64::from(s)).cloned().unwrap_or_default();
                    if ours.is_empty() && t.is_empty() {
                        continue;
                    }
                    pool_lists += 1;
                    compared::note("GroupDump", &["id", "who", "members"]);
                    compared::note("GroupMemberDump", &["o"]);
                    if ours != t {
                        here.entry((1, -2, format!("pool:{}", 64 + i64::from(s))))
                            .or_insert((n, format!("ours {ours:?} theirs {t:?}")));
                    }
                }
            }
            if let Some(t) = stage {
                stages[2] += t.elapsed();
            }
            let stage = profiling.then(std::time::Instant::now);
            for who in 0..2usize {
                let fresh_leader;
                let t = if let Some(value) = &cached {
                    let Some(t) = &value.leaders[who] else {
                        continue;
                    };
                    t
                } else {
                    let Some(block) = flog.unwrap().leader_block(n, who as i64) else {
                        continue;
                    };
                    fresh_leader = crate::diff::leader::theirs(&block);
                    &fresh_leader
                };
                let mine = crate::diff::leader::rows(&loaded, &built, who);
                if who == 1 && std::env::var("RON_SITE_SET").is_ok() {
                    let m: BTreeMap<String, i64> = mine.iter().cloned().collect();
                    let set = |g: &dyn Fn(&str) -> i64| -> Vec<String> {
                        (0..10)
                            .map(|i| {
                                let f = |k: &str| g(&format!("SITE[{i}].{k}"));
                                format!(
                                    "({},{}) v{} d{} r{} g{}",
                                    f("wx"),
                                    f("wy"),
                                    f("val"),
                                    f("dist"),
                                    f("rank"),
                                    f("reg")
                                )
                            })
                            .collect()
                    };
                    let o = set(&|k| m.get(k).copied().unwrap_or(-9));
                    let y = set(&|k| t.get(k).copied().unwrap_or(-9));
                    let (mut os, mut ys) = (o.clone(), y.clone());
                    os.sort();
                    ys.sort();
                    eprintln!(
                        "  siteset {n} {}\n    ours  {}\n    thrs  {}",
                        if os == ys { "same" } else { "DIFF" },
                        o.join(" | "),
                        y.join(" | ")
                    );
                }
                for (k, v) in &mine {
                    let Some(&y) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    if *v != y {
                        here.entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
            if gaia {
                compared += widen_gaia(&built, frame, players, n, &mut here, &mut missing);
            }
            if let Some(t) = stage {
                stages[3] += t.elapsed();
            }
            for &(w, o) in &row_walk {
                let now: Vec<String> = here
                    .iter()
                    .filter(|((hw, ho, _), _)| (*hw, *ho) == (w, o))
                    .map(|((_, _, what), (_, row))| format!("{what}: {row}"))
                    .collect();
                let e = walked.entry((w, o)).or_default();
                if *e != now {
                    eprintln!("  row walk {n} {w}/{o}: [{}]", now.join(" · "));
                    *e = now;
                }
            }
            // The first parting block prints by default (parked 830, the
            // sixteenth pass; the note at the first of these four sites).
            if firsts.is_empty()
                || site_window_named("RON_STANDING").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for ((w, o, what), (_, row)) in &here {
                    eprintln!("  standing {n} {w}/{o} {what}: {row}");
                }
            }
            if is_near(n) {
                standing.insert(
                    n,
                    here.iter()
                        .map(|(k, (_, row))| (k.clone(), row.clone()))
                        .collect(),
                );
            }
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            for t in &frame.units {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = u8::try_from(t.who)
                    .ok()
                    .and_then(|w| built.sim.unit_by_o(w, o16));
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((t.who, t.o), (ta, oa));
                if !is_near(n) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                eprintln!(
                    "  anim {n} {}/{}: theirs ({},{}) a{:?}->{:?} | ours {:?} a{:?}->{:?}",
                    t.who,
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    pt,
                    ta,
                    ours.map(|u| (built.sim.units[u].pos.x, built.sim.units[u].pos.y)),
                    po,
                    oa
                );
                changed.push((n, t.who, t.o, pt != ta, po != oa));
            }
            if let Some((end, points)) = checkpoints.get(snapshots.len())
                && n == *end
            {
                let selected = |n: i64| points.iter().any(|b| (b - 2..=b + 2).contains(&n));
                snapshots.push(Widened {
                    firsts: firsts.clone(),
                    missing: missing.clone(),
                    blocks,
                    leader_rows,
                    changed: changed
                        .iter()
                        .filter(|row| selected(row.0))
                        .copied()
                        .collect(),
                    housed,
                    standing: standing
                        .iter()
                        .filter(|(n, _)| selected(**n))
                        .map(|(n, rows)| (*n, rows.clone()))
                        .collect(),
                    army_lists: ArmyLists::new(),
                });
            }
        }
        assert_eq!(
            snapshots.len(),
            checkpoints.len(),
            "a checkpoint block was absent"
        );
        eprintln!(
            "{name} widening: {blocks} blocks [{first_block}, {tail}], {compared} record rows, \
             {leader_rows} leader rows, {pool_lists} pool lists, {} keys parted, {} keys unprinted",
            firsts.len(),
            missing.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        let last_from = from.last().copied().unwrap_or(first_block);
        for (f, rows) in &by_block {
            let window =
                site_window_named("RON_DEBUG_ROWS").is_some_and(|(lo, hi)| (lo..=hi).contains(f));
            if *f >= last_from - 100 || window {
                for r in rows {
                    eprintln!("  f{f} {r}");
                }
            } else {
                eprintln!("  f{f}: {} keys", rows.len());
            }
        }
        for (n, rows) in &standing {
            eprintln!("  near {n}: {} rows", rows.len());
        }
        if oracle_store.hits + oracle_store.misses > 0 {
            eprintln!(
                "oracle {name}: hits={} misses={} writes={}",
                oracle_store.hits, oracle_store.misses, oracle_store.writes
            );
        }
        if profiling {
            eprintln!(
                "replay profile {name}: prefix={prefix} start={resumed_at} tail={tail} blocks={blocks} total={:?} setup={setup_time:?} ticks={tick_time:?} decode={:?} values={:?} groups={:?} leaders_gaia={:?}",
                started.elapsed(),
                stages[0],
                stages[1],
                stages[2],
                stages[3]
            );
        }
        if let Some(key) = prefix_key {
            super::prefix::put(key, built);
        }
        Some((
            Widened {
                firsts,
                missing,
                blocks,
                leader_rows,
                changed,
                housed,
                standing,
                army_lists: ArmyLists::new(),
            },
            snapshots,
        ))
    }

    /// **run163 — Great Lakes' word 12038, widened whole, both directions**
    /// (item 571). run136's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_UPGRADE`]: ten blocks shared with run136, the
    /// 79 up to the word, its block, and 360 past it, which no capture had
    /// printed. [`widen_great_lakes`] from run123's 11400 across six
    /// captures, with every player-1 pool list from run135's first block.
    ///
    /// The word's frame, 12038, writes block **12039**. The words item 571
    /// and item 657 moved it to, 12135 and 12184, write blocks 12136 and
    /// **12185**, and both are in the window.
    fn run163_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_UPGRADE.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_UPGRADE.1;
        const WORD_BLOCK: i64 = GREAT_LAKES_UPGRADE_BLOCK;
        /// The block who=1's barracks research converts the nine on.
        const SQUAD_HALT_BLOCK: i64 = 11_922;
        /// The block of the word item 571's fix moved to, 12135's.
        const NEXT_BLOCK: i64 = 12_136;
        /// The block of the word item 657's fix moved to, 12184's.
        const MAKE_BLOCK: i64 = GREAT_LAKES_MAKE_BLOCK;
        let Widened {
            firsts,
            missing,
            blocks,
            changed,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        let halt: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == SQUAD_HALT_BLOCK)
            .map(row)
            .collect();
        let nine: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                *w == 1
                    && ((37..=42).contains(o) || (62..=64).contains(o))
                    && (SQUAD_HALT_BLOCK + 1..=WORD_BLOCK).contains(f)
            })
            .map(row)
            .collect();
        let word: Vec<String> = standing
            .get(&WORD_BLOCK)
            .into_iter()
            .flatten()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 62))
            .map(|((_, _, what), row)| format!("{what}: {row}"))
            .collect();
        let chain: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (SQUAD_HALT_BLOCK + 1..=MAKE_BLOCK).contains(f))
            .map(row)
            .collect();
        let newborn: Vec<String> = standing
            .get(&NEXT_BLOCK)
            .into_iter()
            .flatten()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 68))
            .map(|((_, _, what), row)| format!("{what}: {row}"))
            .collect();
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        let under = firsts
            .values()
            .filter(|(f, _)| *f < SQUAD_HALT_BLOCK)
            .count();
        // **Block 11922, the conversion** (item 571). who=1's barracks
        // research lands on sim-frame 11921, and `Unit::set_type` converts
        // the nine walking type-82 figures (`1/37`..`1/42`, `1/62`..`1/64`).
        // The original's `Guy::init_real(guy, 1)` stands each kept guy
        // `stopped` with `last_speed` and `avg_speed` 0 (`docs/ANIM.md`
        // §11). Until the fix this crate reset the clock alone, and the
        // block parted on 29 rows: `stopped` and both speeds on all nine.
        // What stands is `1/41`'s idle variant, on this block only. The
        // draws agree, so the two sides gave it different rolls.
        pin_eq!(
            halt,
            [
                "11922 1/41 g.cur_anim[0]: ours 1 theirs 2",
                "11922 1/41 g.end_time[0]: ours 64 theirs 48",
            ],
            "block 11922"
        );
        // **The move's value diff, both directions.** Until the fix,
        // `1/62`'s position parted on 11923, a step of 12 against 27,
        // because a body at speed 0 turns at once. On the word's block,
        // 12039, the original's `1/62` hard-collided with `1/64` (`collide_o
        // 64`) and went WALK → DEFAULT, the fifth draw (`Guy::set_anim+0x97a
        // < Unit::move_step+0x823`). Ours walked on, 49 units off: 21 rows.
        // Now none of the nine parts from 11923 to the word's block.
        pin_eq!(nine, Vec::<String>::new(), "the nine after the conversion");
        pin_eq!(word, Vec::<String>::new(), "1/62 on the old word's block");
        // **12135 (block 12136) was `1/68`'s idle wrap, and item 657 moved
        // it.** The newborn `1/68` was born with `form` −1 against 0, and
        // on 12058 the original listed it in army 2's pool group 66 with a
        // move order. This crate gave it a group 68 of its own and no order,
        // and on 12136 ours stood and wrapped its idle while the original's
        // walked: 27 rows. The cause was on 12024, army 2's tick.
        // `release_mustering` reads the Military library level
        // (`epoch[0]`), which is 2 here, and this crate read the age, 1. So
        // this crate released army 2 to defend, and on 12057
        // `find_local_army` found only the empty army 0 still mustering
        // (`docs/ARMY.md` §20). **The move's value diff**: 12058's ten rows
        // are gone, and so are `1/68`'s rows on 12136.
        pin_eq!(newborn, Vec::<String>::new(), "1/68 on 12135's block");
        pin_eq!(one_sided, [], "a figure changes animation on one side only");
        // ~~**The next word, 12184 (block 12185), and the rows under it.**~~
        // Ours spent 47 draws against 95, parting at index 0. Ours spent six
        // pairs of `Leader::create_buildings+0xffb`/`+0x1017`, and the
        // original a bird's thirty-round `Animal::think_bird+0x2aa`/`+0x2d3`
        // arm. who=1's production list parted on 12181: ours offered the
        // Scholar at 6,585,365 for one and the original at 0 for three,
        // because the original's purse held 55 wealth against a price of 56
        // and ours held 57. Ours bought it on 12182 and ran a second Units /
        // Buildings / Make pass the original never ran; that pass's
        // `create_buildings` spent the extra draws.
        //
        // **Item 661 moved it 12184 → 12429, and the mechanism was a
        // trade route's worth** (`docs/AI.md` §63). The wealth was parked
        // 514's 32 income: `trade_val` 160 against 176 on both of who=1's
        // cities, because this crate never seated the Barracks `1/2016` and
        // the Stable `1/2018` in Norwich. The original does it on block
        // 8734, through `Leader::gain_tech`'s Civic-level arm, which
        // re-masks every city and runs `City::find_buildings`. **The move's
        // value diff**: the thirteen leader rows on 12181–12184 and
        // `1/2019`'s queue row on 12183 are gone, and nothing opens on the
        // word's old blocks. What stands under the window is residue that
        // spends no draw:
        // - `form` −1 against the 0 `Unit::init` writes, on both newborns;
        // - their birth-block `orders_x`, one block;
        // - group 68, the original's `go_to_unit` walk group, freed on
        //   12086 there and kept here;
        // - `MAKE[2].city` from 11982, and until item 688 `SITE[4].val`
        //   from 11976, the halving a founded city makes of the site values
        //   round it (`City::fix_world_vals`, `docs/AI.md` §67).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1457: ["12086 1/-2 pool:68: ours [68] theirs []"] → []; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
        pin_eq!(chain, Vec::<String>::new(), "the rows under the next word");
        // **The floor**: 248 keys under the conversion — run136's standing
        // residue under its word, as item 644 re-pinned it — and 1,432 in
        // all, to the capture's last block. Before item 571's fix: 248 and
        // 2,044; before item 657's: 248 and 1,884.
        // **Item 661 added two** (248 and 1,432 → 250 and 1,434): the rows that read the
        // sim's own `vans.length` agree and leave, and the four that read
        // the BUILD record's `city` and `city_down` for the Barracks
        // `1/2016`, the Stable `1/2018` and the Mine `1/2021` arrive — this
        // crate never seats the two in Norwich (`docs/AI.md` §63). **Its
        // fix took twelve under the conversion** (250 → 238): those four,
        // both `trade_val`s, the five wealth rows and `worst_good`. Past the
        // word's old block **1,434 → 275**: the second pass's purchases
        // and everything downstream of them are gone.
        // **Item 669 took sixteen more** (275 → 259), all of them past the
        // conversion: `1/68`'s tile plan on 12322 and 12389, which kept the
        // group's tolerance-0 formation point once the unwind read 1 to
        // `0x60` (`docs/ORDERS.md` §4.4).
        // **Item 688 took twenty-one** (238/259 → 218/238), all who=1 `SITE` rows and nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // Item 718: trained units' `orders_x`/`orders_y` on their exit block
        // now agree (`come_out`'s `update_action`, `docs/ORDERS.md` §29).
        // Item 723 took one (212/228 → 211/227): the scout's formation
        // mirror (parked 275), an equal group's record kept.
        // Item 729 took one more (211/227 → 210/226): `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // Item 882 took three (210/226 → 207/223): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (207/223 → 207/220): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 1072 took four (207/220 → 203/216): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1206 took sixteen (199/212 → 183/196): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (183, 196).
        // Item 1330 took 44 (177/188 → 133/142): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (116, 125) → (116, 123), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (112, 118) → (106, 112); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (36, 38), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run174 — Great Lakes' word 12429, widened whole, both directions**
    /// (item 669). run163's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_CIVIC`]: ten blocks shared with run163, the 30
    /// up to the word, its block, and 469 past it, which no capture had
    /// printed. [`widen_great_lakes`] from run123's 11400 across seven
    /// captures, with every player-1 pool list from run135's first block.
    ///
    /// The word's frame, 12429, writes block **12430**.
    fn run174_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_CIVIC.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_CIVIC.1;
        const WORD_BLOCK: i64 = GREAT_LAKES_CIVIC_BLOCK;
        /// run163's last block: everything above it is run174's.
        const RUN163_TAIL: i64 = WIDENING_GREAT_LAKES_UPGRADE.1;
        /// The block `1/68`'s tile plan parts on.
        const PLAN_BLOCK: i64 = 12_322;
        /// The block of the word item 669's fix moved to, 12536's.
        const NEXT_BLOCK: i64 = 12_537;
        /// The block of the word item 673's fix moved to, 12897's.
        const RETRY_BLOCK: i64 = GREAT_LAKES_RETRY_BLOCK;
        let Widened {
            firsts,
            missing,
            blocks,
            changed,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // Every key that first parts on run174's own blocks up to the
        // word's, both newborns' included.
        let above: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (RUN163_TAIL + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // `1/68`'s plan on 12322, from run163.
        let plan: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| (*w, *o) == (1, 68) && *f == PLAN_BLOCK)
            .map(row)
            .collect();
        let word: Vec<String> = standing
            .get(&WORD_BLOCK)
            .into_iter()
            .flatten()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 67))
            .map(|((_, _, what), row)| format!("{what}: {row}"))
            .collect();
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        let under = firsts.values().filter(|(f, _)| *f <= RUN163_TAIL).count();
        let mid: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (RUN163_TAIL + 1..=NEXT_BLOCK).contains(f))
            .map(row)
            .collect();
        let past = firsts
            .values()
            .filter(|(f, _)| (NEXT_BLOCK + 1..=TAIL).contains(f))
            .count();
        let retry: Vec<String> = standing
            .get(&RETRY_BLOCK)
            .into_iter()
            .flatten()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 41))
            .map(|((_, _, what), row)| format!("{what}: {row}"))
            .collect();
        // ~~**`1/68`'s tile plan, block 12322.**~~ On 12321 both sides held
        // `[goal, (36947, 23672)]`: the final point, and the group move's
        // formation waypoint at tolerance 0. On 12322 `do_move`'s tile arm
        // unwinds the loose near waypoints and calls `find_tpath`. The
        // original keeps the formation point and plans the tiles to it,
        // 13 entries. This crate popped it, because it read the unwind's test
        // as `tolerance < 0x60`, and planned to the final goal: 17 entries,
        // fourteen rows here and two more on 12389. From 12422 `1/68` walked
        // a leg the original never walks. On the word's block, 12430, ours'
        // `1/67` hard-collided with it (`collide_o 68`) and stopped, spending
        // the sixth draw (`Unit::move_step+0x823`); the original's walked
        // on. Those were eighteen rows of `1/67` and 36 above run163.
        //
        // **Item 669 moved it 12429 → 12536** (`docs/ORDERS.md` §4.4,
        // `docs/AI.md` §64): the unwind takes a tolerance of 1 to `0x60`,
        // unsigned, and keeps 0. **The move's value diff**: every one of
        // those rows is gone, and so is every one-sided animation change.
        pin_eq!(plan, Vec::<String>::new(), "1/68's plan on 12322");
        // ~~What stands of `1/67` on 12430 is parked 646's `form` residue,
        // which spends no draw.~~ Nothing, since item 1330: the births'
        // `form` is `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(word, [] as [&str; 0], "1/67 on the old word's block");
        pin_eq!(
            above,
            Vec::<String>::new(),
            "the rows above run163, to 12430"
        );
        // ~~The one figure whose animation changes on one side only is the
        // new word's own: the original's `1/41` stands on 12898 and wraps
        // its idle on 12899, and this crate's walks on (item 673).~~ Item
        // 678 took it: with the copy tree, `1/41` walks the original's
        // sidestep from 12626 and never reaches `1/15` out of step.
        pin_eq!(one_sided, [], "a figure changes animation on one side only");
        // ~~**The word 12536 (block 12537).**~~ On 12537 the squad
        // `1/27`–`1/29` took its orders. The original gave `1/28` and `1/29`
        // a kind-2 order with a ten-entry world plan at tolerance 384, and
        // `1/27` a kind 2 with no plan. This crate gave `1/28` and `1/29`
        // kind 21 with one entry and `1/27` nothing: 33 rows.
        //
        // **Item 673 moved it 12536 → 12897** (`docs/PATHFINDER.md` §21.6,
        // `docs/AI.md` §65). All thirteen of group 66 take a
        // `GROUP_ATTACK_TO` on 12536 on both sides. `1/27`, the squad's
        // captain, steps into the standing `1/64` and its unit-grid search
        // fails. The original's `astar_path` rolls `retry` (6) and
        // `find_upath` spares the order, both behind vslot `+0x14`,
        // `is_move`. This crate gated both on `is_transit`, a move without
        // the action bit, and killed the order, so `do_group_move`'s
        // step-0 ungroup never ran and the squad marched on in formation.
        // **The move's value diff**: the 33 rows are gone, and `1/27` is the
        // dump's own on 12537 (`retry` 6, `dest_x` 40456, `coll` (36485,
        // 23365)). What stands is the group-id row, parked 674's, which
        // spends no draw. **Item 811 took its `order_num`** (…206 →
        // …207, the original's 7): an army's retarget forms it twice on
        // one tick (`docs/ARMY.md` §21), so the group's count stands one
        // higher; the pool id (2 here, 66 there) is 689's.
        pin_eq!(
            mid,
            Vec::<String>::new(),
            "the rows above run163, to the old word's block"
        );
        // ~~**The new word, 12897 (block 12898), `1/41`'s every standing
        // row.**~~ The original's `1/41` stepped into `1/15` (`collide_o
        // 15`), stopped and stood (`Unit::move_step+0x823`, the third
        // draw), and this crate's walked on: 21 rows. On 12825 the
        // original's squad `1/40`–`1/42` held kind 2 where this crate's
        // held 21, and the chain started at `1/41`'s plan on 12626, 23
        // entries against 20.
        //
        // **Item 678 moved it 12897 → 14382, past this capture**
        // (`docs/PATHFINDER.md` §26, `docs/AI.md` §66). On 12623 `1/41`
        // collides with `1/34`, and its 48-grid search copies five world
        // cells' blocks into the pathfinder's tree, runs over its budget
        // and suspends; the suspend hands the tree to the unit. On 12624
        // the resume reads those copies, so the cells it first probes see
        // `1/66` and the rest where they stood a frame before. This crate
        // read the live blocks, and routed round `1/66` where the
        // original walks row 21144. **The move's value diff**: `1/41`'s
        // plan on 12626, its lag from 12662, the squad's ungroup on
        // 12825, and `1/41`'s 21 rows here are gone — 74 keys, all on the
        // three. Nothing on the squad parts through 12899.
        pin_eq!(retry, Vec::<String>::new(), "1/41 on the old word's block");
        // Every key that first parts from 12538 to run174's last block:
        // the citizens' `myhits` 40 against 50 from 12564 (no draw, parked
        // 679) and what stands with them. The squad's 74 left with item 678,
        // and item 688 took the leader's `SITE` rank on 12576: a founded
        // city wears the site values down (`docs/AI.md` §67).
        // **Item 718 took four (134 → 130)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // Item 1354 took 29 (120 → 91): the AI Citizens' `mylos` on 12563,
        // ours 4 against 2 — the Militia line's term reaching the
        // derivation a frame before the original's cache (`docs/VISION.md`
        // §2).
        pin_eq!(past, 87, "the keys first parting after 12537");
        // **The floor**: 259 keys to run163's last block, and 2,120 to
        // run174's. The floor had been 275 to run163, as item 661 left it,
        // and 2,197 to run174. The fix took sixteen under 12399, all
        // `1/68`'s plan. **Item 673 took 1,650 above run163** (2,120 →
        // 470): the 33 on 12537 and the cascade the squad's formation
        // walk left behind it, and none under. **Item 678 took 74 more**
        // (470 → 396), `1/40`–`1/42`'s chain from 12626, and none under.
        // **Item 688 took twenty-two** (259/396 → 238/374), all who=1 `SITE` rows and nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // Item 718: trained units' `orders_x`/`orders_y` on their exit block
        // now agree (`come_out`'s `update_action`, `docs/ORDERS.md` §29).
        // Item 723 took one (228/360 → 227/359): the scout's formation
        // mirror (parked 275), an equal group's record kept.
        // Item 729 took one more (227/359 → 226/358): `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // Item 882 took three (226/358 → 223/355): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (223/355 → 220/352): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 1072 took four (220/352 → 216/348): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1206 took sixteen (212/336 → 196/320): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (196, 320).
        // Item 1330 took 46 (188/312 → 142/264): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (142/264 → 142/235): the Citizens' `mylos` on 12563, the cache (`docs/VISION.md` §2).
        // Item 1455: (125, 217) → (123, 215), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (118, 210) → (112, 204); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (38, 125), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run178 — Great Lakes' word 14382, widened whole, both directions**
    /// (item 678). run174's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_COPY`]: six blocks shared with run174, the
    /// 1,483 up to the word, its block, and 516 past it, which no capture had
    /// printed. [`widen_great_lakes`] from run123's 11400 across eight
    /// captures, with every player-1 pool list from run135's first block.
    ///
    /// The word's frame, 14382, writes block **14383**.
    ///
    /// **Item 688 moved it 14382 → 14529, inside this window** (`docs/AI.md`
    /// §67): a founded city wears the site values down round it, and the
    /// Barracks on 14383 goes up on the original's cell. The new word's
    /// frame writes block **14530**; the test keeps the move's value diff
    /// on 14383 and pins the new word's block.
    ///
    /// **Item 695 moved it 14529 → 14650, inside this window again**
    /// (`docs/CARAVAN.md` §10, `docs/ROADS.md` §10): the caravan verifies a
    /// road the stray-road sweep eroded. The new word's frame writes block
    /// **14651**; the test keeps the move's value diff on 14530.
    ///
    /// **Item 698 moved it 14650 → 14982, past this window**
    /// (`docs/COLLISION.md` §15): a gatherer whose search is suspended
    /// gives its walk up six frames after it collides. The test keeps the
    /// move's value diff on 14651; the new word is run192's.
    fn run178_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const FIRST: i64 = WIDENING_GREAT_LAKES_COPY.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_COPY.1;
        const WORD_BLOCK: i64 = GREAT_LAKES_ROAD_BLOCK;
        /// The word item 678 left, and 688 moved: its block keeps the move's
        /// value diff.
        const OLD_BLOCK: i64 = GREAT_LAKES_COPY_BLOCK;
        /// The word item 688 left, and 695 moved: its block keeps that
        /// move's value diff.
        const VALS_BLOCK: i64 = GREAT_LAKES_VALS_BLOCK;
        /// run174's last block: everything above it is run178's.
        const RUN174_TAIL: i64 = WIDENING_GREAT_LAKES_CIVIC.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // Every key that first parts on run178's own blocks under the word.
        let mid: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (RUN174_TAIL + 1..WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Every key that first parts on the word's own block, by unit.
        let mut word: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, r)) in &firsts {
            if *f == WORD_BLOCK {
                word.entry((*w, *o))
                    .or_default()
                    .push(format!("{what}: {r}"));
            }
        }
        let standing_n = standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len);
        let under = firsts.values().filter(|(f, _)| *f <= RUN174_TAIL).count();
        // **The move's value diff, on the old word's block 14383.** Until
        // item 688 who=1's new Barracks `1/2025` stood at (39552, 17472)
        // here and (42624, 19776) there: the spiral round the capital
        // scored cell (51, 22) 1252 and (55, 25) 1251 here, `1000 + 0xff −
        // val`, where the original reads (55, 25) at `val` 2 because
        // Norwich's founding halved it (`City::fix_world_vals`). With it,
        // 60 keys on 14383 — the building, Norwich's chain, `MAKE[7]`, and
        // `1/6` and `1/7` trading the build and walk orders — are gone,
        // and nothing parts on the old word's five blocks.
        let old: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (OLD_BLOCK - 2..=OLD_BLOCK + 2).contains(f))
            .map(row)
            .collect();
        pin_eq!(old, Vec::<String>::new(), "the old word's five blocks");
        // **Item 695's value diff, on 688's word's block 14530.** On it
        // `1/23`'s 24 path entries carried the road flag `0x20` here and
        // not there, and `1/28`'s figure turned here alone: the original's
        // caravan had taken a waypoint whose road the stray-road sweep had
        // eroded, verified its route (3,206 road draws a frame) and
        // stripped the flag. With the sweep (`Sim::scan_and_kill_stray_roads`)
        // and the check (`Sim::caravan_road_step`), the 17 road tiles
        // (220, 98)..(216, 115) this crate kept are gone by 14529, as on
        // run189's `WORLD` block, and nothing parts on these five blocks.
        let vals: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (VALS_BLOCK - 2..=VALS_BLOCK + 2).contains(f))
            .map(row)
            .collect();
        pin_eq!(vals, Vec::<String>::new(), "688's word's five blocks");
        // **Under the word, on run178's own blocks: the site list's slot
        // order and the newborns' residue.** The site values agree on
        // every block but three spans since item 688 (the table as a set:
        // 2,776 of 3,500 blocks, against none before). The `SITE[8]` rows
        // on 12976 are the same ten sites in another slot order, and
        // `SITE[4]` on 14176 is one low-value site the sweep samples
        // differently. The rest is what run174 already carried on every
        // newborn: `form` −1 against 9 (parked 646's shape), `myhits` 40
        // against 50 and `mylos` 2 against 4 (parked 679), a birth-block
        // `orders_x/y`. The leader's scholar count on 13556 is the block
        // after the thirteenth seating, which both sides spend on 13555.
        // ~~And on 14529 the capital~~ — gone with item 695: the Senate
        // `1/2024` moves `0x10` from `1/2000` to Norwich `1/2007` here too
        // (`docs/CITIES.md` §15). Its territory is recomputed at once here
        // and over sixteen frames there (`docs/ATTRITION.md`, "Territory"),
        // so who=0's `gather_stamp` is re-stamped on 14536 here and 14544
        // there. ~~`known_rares` on 14536 parts with or without it~~ —
        // **item 822 moved it to 14584**: the Senate's border fix zeroes
        // `reg_known_rares` (`docs/AI.md` §76), and both recomputes then
        // sum 0 from 14535. What stands is who=1's own recompute on 14583
        // here against 14543 there (its `gather_stamp` row, standing from
        // 14544): ours sums the census's recount of 14575, 4, and the
        // original waits for its recompute on 14631 (parked 701's shape).
        // ~~And on 14650 `1/43`, the word's gatherer~~ — gone with item
        // 698 (below): eight rows, `collide` 7 against 6 among them, which
        // is the counter and not a partner.
        // **Item 904 took ten**: who=1's `SITE[4]` (14176) and `SITE[8]`
        // (12976), a city site at (47, 33) that ours scored 95,326 and the
        // original never listed — `blocked_tcoord`'s rock arm refuses it
        // (`docs/AI.md` §78).
        pin_eq!(
            mid,
            // Item 1106 closed who=0's `gather_stamp` on 14537, 14536
            // against 14080: the border pass's economy flag (`docs/AI.md` §84).
            [
                // Item 1468 (group 17): every unit is born snapped to the 48-grid's centre, so the
                // trained squads' `orders_x`/`orders_y` stand where the original's do: six rows → two.
                "13556 1/74 g.angle[0]: ours 1431655765 theirs 0",
                "13737 1/75 g.angle[0]: ours 1431655765 theirs 0"
            ],
            "the rows under the word, on run178's own blocks"
        );
        // ~~**The word, 14529 (block 14530).**~~ Item 695 moved it: the
        // caravan's road check and the stray-road sweep (above).
        //
        // ~~**The word, 14650 (block 14651).**~~ The original spent 3 draws
        // where this crate spent 2, parting at index 0 on
        // `Unit::do_non_flat_gather+0x54b`, and on the block the Woodcutter's
        // Camp `1/2009`'s gather-tile list parted, 192 keys, one entry
        // along. Under it, on block 14650, `1/43` held its walk here and
        // had dropped it there: eight rows, `collide` 7 against 6.
        //
        // **Item 698's value diff, on 695's word's block 14651.** `1/43`
        // met the standing `1/18` on 14643 on both sides, and its
        // 48-grid search suspended. On 14649 the original's `do_move`
        // took its `GATHER` park: six frames after the collision, `(6 + 2)
        // & 7 == 0`, and `vector_dist(96, 240) = 259 < 0x120` to the
        // move's point. The walk died above the `collide` increment, and
        // the gather order forgot its tile, so 14650's
        // `do_non_flat_gather` drew a tile afresh. With the arm
        // (`docs/COLLISION.md` §15), the eight rows on 14650 and the 192
        // on 14651 are gone, and nothing parts on these five blocks.
        let road: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(f))
            .map(row)
            .collect();
        pin_eq!(road, Vec::<String>::new(), "695's word's five blocks");
        pin!(word.is_empty(), "nothing parts first on 14651");
        // Item 729: 317 → 316, `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // Item 904: 316 → 311, who=1's `SITE[4]`, standing from 14176 — the
        // rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (311 → 307): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (307 → 306): who=0's `gather_stamp`, the border pass's economy flag (`docs/AI.md` §84).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 166.
        // Item 1455: 98 → 97, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            standing_n, // Item 1444: the ruins guard removes stale site fields (batch replay).
            37, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 14651"
        );
        // **The floor**: run174's 374 to its last block, exactly as
        // `run174_s_word_frame_is_widened_whole` pins it — the chain is
        // the same walk — then the rows under the word on run178's own
        // blocks, and every key to run178's last block: 1,552 before item
        // 688, 1,375 after it, 1,279 after item 695, and 416 after item
        // 698 — the eight on 14650, the 192 on 14651 and 663 of the cascade
        // above it to 14899, and none under.
        // Item 1115 took the caravan's four rows here ((348, 25, 373) → (344, 25, 369); `docs/CARAVAN.md` §11.3).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (320, 13, 333).
        // Item 1330 took 48 (312/13/325 → 264/8/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (217, 6, 223) → (215, 6, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (210, 6, 216) → (204, 6, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid.len(), firsts.len()),
            // Item 718: six trained units' `orders_x`/`orders_y`; item 723
            // took one more, the scout's formation mirror (parked 275).
            // Item 718: six trained units' `orders_x`/`orders_y`; item 729
            // took one more (359/36/395 → 358/36/394), `0/frame_attacked`
            // (`Object::take_damage`'s stamp, `docs/AI.md` §71).
            // Item 882 took three (358/36/394 → 355/36/391): the squad's
            // 11424 `group` rows, `come_out`'s push (`docs/GROUPS.md` §31).
            // Item 890 took three (355/36/391 → 352/36/388): who=1's `MAKE[1]`, `[3]` and `[4]`
            // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
            // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
            // Item 904 took ten (352/36/388 → 352/26/378): who=1's `SITE[4]`
            // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
            // Item 1072 took four (352/26/378 → 348/26/374): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1106: → 348/25/373, who=0's `gather_stamp` on 14537, the border pass's economy flag (`docs/AI.md` §84).
            // Item 1206 took sixteen (336/13/349 → 320/13/333): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (125, 2, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run192 — Great Lakes' word 14982, widened whole, both directions**
    /// (item 698). run178's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_BIRTH`]: six blocks shared with run178, the 83
    /// up to the word, its block, and 56 past it. [`widen_great_lakes`]
    /// from run123's 11400 across nine captures, with every player-1 pool
    /// list from run135's first block.
    ///
    /// The word's frame, 14982, writes block **14983**. Item 706 moved
    /// the word past run192 to 15175; this keeps the move's value diff on
    /// 14946 and 14983.
    fn run192_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_BIRTH_BLOCK;
        /// run178's last block: everything above it is run192's.
        const RUN178_TAIL: i64 = WIDENING_GREAT_LAKES_COPY.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_BIRTH.1 - WIDENING_GREAT_LAKES_BIRTH.0 + 1) as usize,
            "the walk is whole"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // Every key that first parts on run192's own blocks up to the
        // word's block.
        let own: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (RUN178_TAIL + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        let mut word: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, r)) in &firsts {
            if *f == WORD_BLOCK {
                word.entry((*w, *o))
                    .or_default()
                    .push(format!("{what}: {r}"));
            }
        }
        let standing_n = standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len);
        let under = firsts.values().filter(|(f, _)| *f <= RUN178_TAIL).count();
        // **Under the word, on run192's own blocks: the free train, built
        // by item 706.** On 14946 a who=1 Barracks' free archers `1/76`–
        // `1/78` are born on both sides at (42648, 20472). They were
        // Bowmen (120) here and Longbowmen (127) there, `myhits` 70
        // against 88, because this crate's `Tribe::graft` was the
        // identity and the British `get_graft(Archers)` answered Archers
        // (`docs/TECH.md` §"The graft table"). With the table the 15 rows
        // of type, hits, piece and `num_units` are gone, and so is the
        // leader's `attack` on 14976. What stands is parked 646's `form`.
        //
        // **The old word's block, 14983, built by item 706 too**: who=1's
        // Senate finishes Despotism on tick 14982 and trains The Despot,
        // `1/79` (`docs/TECH.md` §"The government patriot"). It is born
        // where the original's is, and `gov` and `gov_hero_frame` agree.
        // Its order point does not: the original's Despot is still inside
        // the Senate (`visible 0`, `orders_x/y` its own point), and ours
        // has come out onto its ring. That is the hypothesis for the new
        // word, 15175, and it is not built here (DECISIONS 42).
        pin_eq!(
            own,
            [] as [&str; 0],
            "the rows on run192's own blocks, to the word's"
        );
        // **The old word, 14982 (block 14983).** The original spent three
        // `Guy::init_real+0x52` where this crate spent none: the Despot's
        // three figures. Since item 706 both sides birth it, the leader's
        // `active` and `num_units[302]` agree, and `1/62`'s wrap agrees.
        // `1/79` parts on three keys: `form` and the order point above.
        let words: Vec<(i64, i64, usize)> =
            word.iter().map(|(&(w, o), v)| (w, o, v.len())).collect();
        pin_eq!(
            words,
            [], // item 718: `orders_x`/`orders_y` agree on its exit
            "who parts first on 14983, and on how many keys"
        );
        // Item 718: `1/79`'s `orders_x`/`orders_y` agree on its exit block.
        // Item 723 took one (315 → 314): the scout's formation mirror
        // (parked 275), an equal group's record kept.
        // Item 729 took one more (314 → 313): `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // Item 1072 took four (313 → 309): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 169.
        // Item 1330 took 51 (164 → 113): the births' `form` (`docs/GROUPS.md` §24.3).
        // Item 1455: 97 → 96, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            standing_n, // Item 1444: the ruins guard removes stale site fields (batch replay).
            36, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 14983"
        );
        // **The floor**: run178's 416 to its last block, exactly as
        // `run178_s_word_frame_is_widened_whole` pins it — the chain is the
        // same walk — then run192's own, to the word's block and past it.
        // Item 718: twenty-two trained units' exit rows agree (416/6/422 →
        // 396/4/400).
        // Item 723 took one (396/4/400 → 395/4/399): the scout's formation
        // mirror (parked 275), an equal group's record kept.
        // Item 729 took one: `0/frame_attacked`, `Object::take_damage`'s
        // stamp (`docs/AI.md` §71).
        // Item 882 took three (394/4/398 → 391/4/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (391/4/395 → 388/4/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (388/4/392 → 378/4/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (378/4/382 → 374/4/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (374/4/378 → 373/4/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((373, 4, 377) → (369, 4, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (349/4/353 → 333/4/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (333, 4, 337).
        // Item 1330 took 53 (325/4/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, own.len(), firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run196 — Great Lakes' word 15175, widened whole, both directions**
    /// (item 706). run192's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_PATRIOT`]: six blocks shared with run192, the
    /// 136 up to the word, its block, and 56 past it. [`widen_great_lakes`]
    /// from run123's 11400 across ten captures, with every player-1 pool
    /// list from run135's first block.
    ///
    /// The word's frame, 15175, writes block **15176**.
    fn run196_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_PATRIOT_BLOCK;
        /// run192's last block: everything above it is run196's.
        const RUN192_TAIL: i64 = WIDENING_GREAT_LAKES_BIRTH.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_PATRIOT.1 - WIDENING_GREAT_LAKES_PATRIOT.0 + 1) as usize,
            "the walk is whole"
        );
        // **Under the old word, on run196's own blocks: nothing, since item
        // 711.** Until then the three free Longbowmen `1/76`–`1/78` parted
        // here on 89 keys from 15095: their guard posts on The Despot
        // `1/79` were handed out the other way round, `1/77` on (42696,
        // 22392) here and (42984, 22344) there, and `1/78` the reverse.
        // The Despot carries `unit_masks & 2`, its own mirror, and
        // `Unit::do_guard` negates `dx` for it (`docs/GROUPS.md` §25).
        // With the bit carried, the posts are the original's and no row
        // of any unit first-parts on 15040..15176.
        let mut own: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (RUN192_TAIL + 1..=WORD_BLOCK).contains(f) {
                *own.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let own: Vec<(i64, i64, i64, usize)> =
            own.into_iter().map(|((f, w, o), n)| (f, w, o, n)).collect();
        pin_eq!(
            own,
            [],
            "who parts first on run196's own blocks up to the old word's, and on how many keys"
        );
        // **The old word, 15175 (block 15176)**, and its value diff: the
        // extra `Guy::set_anim+0x97a < Unit::do_guard+0x7f4` is gone with
        // the posts, and nothing parts on the block or under it.
        pin!(
            !firsts.values().any(|(f, _)| *f == WORD_BLOCK),
            "nothing parts first on 15176"
        );
        // Nor above it: run196's runway, 15177..15232, parts nowhere.
        // The 315 rows standing on 15176 are the floor's, parted under
        // run192's tail (376 before item 711: the archers' 61 are gone;
        // item 729 took one more, `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71)).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 169.
        // Item 1455: 98 → 97, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
            // Item 723 took one (315 → 314): the scout's formation mirror.
            // Item 1072 took four (313 → 309): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11300 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1115 took four (309 → 305): the caravan's leg faces its bearing and its trade order carries no bit 4 (`order:move.angle`, `dest_angle`, `order:flags`, `order:action`; `docs/CARAVAN.md` §11.3).
            // Item 1111 took the AI citizens' four Militia-line rows (`myhits`, `hits:myhits`, `hits_left`, `mylos`: `docs/GOLDEN.md` §48).
            // Item 1330 took 50 (164 → 113): the births' `form` (`docs/GROUPS.md` §24.3).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            37, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 15176"
        );
        // **The floor**: run192's 422 to its last block, exactly as
        // `run192_s_word_frame_is_widened_whole` pins it (the chain is the
        // same walk). Then run196's own rows up to the word's block, and
        // every key to the window's end.
        let under = firsts.values().filter(|(f, _)| *f <= RUN192_TAIL).count();
        let mid = firsts
            .values()
            .filter(|(f, _)| (RUN192_TAIL + 1..=WORD_BLOCK).contains(f))
            .count();
        // Item 711: run196's own 89 and the 221 above the old word are gone.
        // **Item 718 took twenty-two (422/0/422 → 400/0/400)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // Item 723 took one (400/0/400 → 399/0/399): the scout's formation
        // mirror (parked 275), an equal group's record kept.
        // Item 729 took one more (399/0/399 → 398/0/398): `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // Item 882 took three (398/0/398 → 395/0/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (395/0/395 → 392/0/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (392/0/392 → 382/0/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (382/0/382 → 378/0/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (378/0/378 → 377/0/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((377, 0, 377) → (373, 0, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (353/0/353 → 337/0/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (337, 0, 337).
        // Item 1330 took 57 (329/0/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// Nine historical windows of the same replay. Keep their original pins
    /// and per-window observations; share only execution of their common prefix.
    #[test]
    fn great_lakes_run163_through_run226_are_widened_whole() {
        cases_03::great_lakes_run163_through_run226_are_widened_whole();
    }

    /// **run202 — Great Lakes' word 15384, widened whole, both directions**
    /// (item 711, and item 715's move inside it). run196's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_MIRROR`]: six blocks shared with run196, the
    /// 151 up to the word, its block, and 56 past it. [`widen_great_lakes`]
    /// from run123's 11400 across eleven captures, with every player-1
    /// pool list from run135's first block.
    ///
    /// Item 715's word's frame, 15384, writes block **15385**; item 711's
    /// word, 15383, wrote **15384**, and both value diffs are kept here.
    /// **Item 722 moved the word past run202, to 15608** (`docs/AI.md` §70),
    /// and its value diff is kept here too: nothing first-parts on
    /// 15385..15440. The new word's block is `run211_s_word_frame_is_widened_whole`'s.
    fn run202_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_RECRUIT_BLOCK;
        /// Item 711's word's block, which item 715 moved off.
        const OLD_BLOCK: i64 = GREAT_LAKES_MIRROR_BLOCK;
        /// run196's last block: everything above it is run202's.
        const RUN196_TAIL: i64 = WIDENING_GREAT_LAKES_PATRIOT.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_MIRROR.1 - WIDENING_GREAT_LAKES_MIRROR.0 + 1) as usize,
            "the walk is whole"
        );
        let mut own: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (RUN196_TAIL + 1..=WORD_BLOCK).contains(f) {
                *own.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let own: Vec<(i64, i64, i64, usize)> =
            own.into_iter().map(|((f, w, o), n)| (f, w, o, n)).collect();
        // ~~**Under the word, on run202's own blocks: one unit and one key.**
        // The Despot `1/79`'s move order carries `facing` 1 here and 0 there
        // from 15351: a value row that spends no draw (parked 716).~~
        // **Item 736 closed it** (`docs/GROUPS.md` §26): the Despot's
        // `ATTACK_TO` is laid out by `action_siege_attack_to`'s stack
        // sub-group, whose `facing` is `Group::clear`'s 0, where this crate
        // read army 3's 1. Nothing parts on run202's own blocks.
        pin_eq!(
            own,
            [],
            "who parts first on run202's own blocks up to the word's, and on how many keys"
        );
        // **Item 715's value diff.** Item 711's word was `1/70`'s order:
        // on 15383 the original's citizen held one and ours none, and on
        // 15384 ours stood idle, seven keys, with `1/2022`'s gather-chain
        // head `1/70` here and `1/73` there. `Wall::process`'s recruiter
        // hands `1/70` the Pyramids' build order on 15382, and none of the
        // ten rows parts anywhere up to the new word's block.
        let gone: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                (OLD_BLOCK - 1..=WORD_BLOCK).contains(f) && *w == 1 && (*o == 70 || *o == 2022)
            })
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            gone.is_empty(),
            "1/70 and 1/2022 hold on 15383..15385: {gone:?}"
        );
        pin!(
            !firsts.values().any(|(f, _)| *f == WORD_BLOCK),
            "nothing parts first on the word's own block, 15385"
        );
        // ~~**The first rows past the word**, on 15386: the make list's
        // second slot takes a Siege Factory (430) here and nothing there,
        // and `1/75`'s figure clock.~~ **Item 722 closed all three, and
        // every row above them** (`docs/AI.md` §70): `CityData::num_wonders`
        // counts the Pyramids site, so who=1's city 0 values no wonder on
        // 15384's second pass. Nothing first-parts anywhere on run202's
        // runway, 15385..15440; the word left run202 for 15608 (run211).
        let above: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f >= WORD_BLOCK)
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            above.is_empty(),
            "nothing parts first on 15385..15440, item 715's word's block and above: {above:?}"
        );
        let under = firsts.values().filter(|(f, _)| *f <= RUN196_TAIL).count();
        let mid = firsts
            .values()
            .filter(|(f, _)| (RUN196_TAIL + 1..=WORD_BLOCK).contains(f))
            .count();
        // The floor's rows, standing: 327 on 15384 before item 715, and
        // 315/316 before item 729, whose stamp closed `0/frame_attacked`,
        // and 314/315 before item 736, whose sub-group closed `1/79`'s
        // move `facing` (parked 716).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (169, 170).
        // Item 1330 took 50 (164/164 → 114/114): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (98, 98) → (97, 97), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            (
                standing.get(&OLD_BLOCK).map_or(0, BTreeMap::len),
                standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
            ),
            // Item 1072 took four (313/314 → 309/310): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (37, 37), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 15384 and on 15385"
        );
        // **The floor**: run196's walk exactly — run192's 422, and nothing
        // on run196's own blocks since item 711 — then run202's own one up
        // to the word's block (item 715: `1/70`'s nine and `1/2022`'s one
        // are gone), and every key to the window's end.
        // **Item 718 took twenty-two (422/1/646 → 400/1/624)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 722 took the other 223 (400/1/624 → 400/1/401)**: every
        // key that first-parted on 15386..15440 was the wonder pairs'
        // (`docs/AI.md` §70), and nothing arrived.
        // **Item 723 took one (400/1/401 → 399/1/400)**: the scout's
        // formation mirror (parked 275), an equal group's record kept.
        // **Item 729 took one more (399/1/400 → 398/1/399)**: `0/frame_attacked`,
        // `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // **Item 736 took the last one (398/1/399 → 398/0/398)**: `1/79`'s
        // move `facing` on 15351 (parked 716), the siege arm's sub-group
        // laying out on its own cleared record (`docs/GROUPS.md` §26).
        // Item 882 took three (398/0/398 → 395/0/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (395/0/395 → 392/0/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (392/0/392 → 382/0/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (382/0/382 → 378/0/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (378/0/378 → 377/0/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((377, 0, 377) → (373, 0, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (353/0/353 → 337/0/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (337, 0, 337).
        // Item 1330 took 57 (329/0/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run211 — Great Lakes' word 15608, widened whole, both directions**
    /// (item 722). run202's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_WONDER`]: six blocks shared with run202, the
    /// 168 up to the word, its block, and 250 past it. [`widen_great_lakes`]
    /// from run123's 11400 across twelve captures, with every player-1
    /// pool list from run135's first block.
    ///
    /// The word's frame, 15608, writes block **15609**. **Item 729 moved
    /// the word to 15619** (`docs/AI.md` §71), inside run211: its frame
    /// writes block **15620**, which this test pins, and the move's value
    /// diff on 15608 and 15609 stays here.
    fn run211_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_ATTACKED_BLOCK;
        /// Item 722's word's block, which item 729 moved off.
        const OLD_BLOCK: i64 = GREAT_LAKES_WONDER_BLOCK;
        /// run202's last block: everything above it is run211's.
        const RUN202_TAIL: i64 = WIDENING_GREAT_LAKES_MIRROR.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_WONDER.1 - WIDENING_GREAT_LAKES_WONDER.0 + 1) as usize,
            "the walk is whole"
        );
        let mut own: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (RUN202_TAIL + 1..=WORD_BLOCK).contains(f) {
                *own.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let own: Vec<(i64, i64, i64, usize)> =
            own.into_iter().map(|((f, w, o), n)| (f, w, o, n)).collect();
        // **On run211's own blocks up to the word: the Despot's escort,
        // and only the escort until the word's block.** Nothing parts on
        // 15441..15606. On 15607 the Longbowmen `1/77` and `1/78` — group
        // 67, who=1's army 3 with The Despot `1/79` and `1/76` — trade
        // guard posts: each walks to the post the other holds in the
        // original (item 711's shape, beside parked 716 and 717), and on
        // 15608 their guard orders carry the swapped posts. `1/76`'s
        // `half_step` parts on 15617 and its position on 15619, the new
        // word's frame; on its block, 15620, its heading, the two swapped
        // Longbowmen's collision, and `1/36`/`1/40`'s clocks.
        // **Item 736 closed every one** (`docs/GROUPS.md` §26): the
        // Despot's `ATTACK_TO` of 15350 was laid out on army 3's `facing`
        // (1) where the original's stack sub-group carries
        // `Group::clear`'s 0 (parked 716); the dying move handed that into
        // `1/79`'s `unit_masks & 2` on 15606 (the new `mirror` row, ours 1
        // against 0 on 15607), and `do_guard` mirrored the escort's `dx`
        // off it. Nothing parts on run211's own blocks, 15441..15620.
        pin_eq!(
            own,
            [],
            "who parts first on run211's own blocks up to the word's, and on how many keys"
        );
        // **Item 729's value diff.** Item 722's word was army 2's
        // `find_target` throwing the difficulty gate's coin for the human:
        // this crate's `0/frame_attacked` was 8186 (the probe's stamp) where
        // the original's is 10233 (`Object::take_damage`'s), so the
        // 7,200-frame gate let L=0 through. The stamp brings the human's
        // `frame_attacked` to 10233 on every block of the chain, and the
        // coin's consequences on 15609 — `1/3`'s move target and `1/61`'s
        // clock — part nowhere.
        let gone: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                (*w == 0 && what == "leader:frame_attacked")
                    || (*f == OLD_BLOCK && *w == 1 && (*o == 3 || *o == 61))
            })
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            gone.is_empty(),
            "0/frame_attacked holds on the whole chain, and 1/3 and 1/61 on 15609: {gone:?}"
        );
        // **Item 736's value diff** (`docs/GROUPS.md` §26, parked 716): the
        // Despot's move `facing`, 1 here and 0 there from 15351, and its
        // `mirror`, 1 here and 0 there on 15607, part nowhere on run202's
        // and run211's own blocks; nor does any other row of group 67's
        // (their `form`, −1 against 0 since 14946, is the floor's).
        let despot: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                *w == 1 && (76..=79).contains(o) && *f > WIDENING_GREAT_LAKES_PATRIOT.1
            })
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            despot.is_empty(),
            "group 67 — 1/76..1/79 — holds from run202's own blocks on: {despot:?}"
        );
        let under = firsts.values().filter(|(f, _)| *f <= RUN202_TAIL).count();
        let mid = firsts
            .values()
            .filter(|(f, _)| (RUN202_TAIL + 1..=WORD_BLOCK).contains(f))
            .count();
        // The floor's rows, standing: 364 on 15608 and 374 on 15609 before
        // item 729, and 363/381 before item 736, whose sub-group closed the
        // Despot's move `facing` (parked 716), its `mirror` and the escort's
        // posts: only the floor's own rows stand on either block.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (165, 165).
        // Item 1330 took 50 (162/162 → 112/112): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (96, 96) → (95, 95), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            (
                standing.get(&OLD_BLOCK).map_or(0, BTreeMap::len),
                standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
            ),
            // Item 1072 took four (309/309 → 305/305): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (35, 35), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 15609 and on the word's block, 15620"
        );
        // **The floor**: run202's walk, then run211's own keys up to the
        // word's block, and every key to the window's end. 401/78/1003
        // before items 723 and 729.
        // **Item 723 took one (401/78/1003 → 400/78/1003)**: the scout's
        // formation mirror (parked 275), an equal group's record kept.
        // **Item 729**: run202's walk loses `0/frame_attacked`, the one
        // row under run211 the stamp closes, and the word's block moves.
        // **Item 736 (399/89/1014 → 398/0/398)**: run202's walk loses
        // `1/79`'s move `facing` (parked 716), and run211's own 89 up to
        // the word and the 526 above it go with nothing arriving: the siege
        // arm's sub-group lays out on its own cleared record
        // (`docs/GROUPS.md` §26). Nothing parts on 15441..15859; the word
        // left run211 for 16460 (run218).
        // Item 882 took three (398/0/398 → 395/0/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (395/0/395 → 392/0/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (392/0/392 → 382/0/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (382/0/382 → 378/0/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (378/0/378 → 377/0/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((377, 0, 377) → (373, 0, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (353/0/353 → 337/0/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (337, 0, 337).
        // Item 1330 took 57 (329/0/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run218 — Great Lakes' word 16460, widened whole, both directions**
    /// (item 736). run211's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_ESCORT`]: six blocks shared with run211, the
    /// 601 up to the word, its block, and 250 past it. [`widen_great_lakes`]
    /// from run123's 11400 across thirteen captures, with every player-1
    /// pool list from run135's first block.
    ///
    /// The word's frame, 16460, writes block **16461**.
    fn run218_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_ESCORT_BLOCK;
        /// run211's last block: everything above it is run218's.
        const RUN211_TAIL: i64 = WIDENING_GREAT_LAKES_WONDER.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_ESCORT.1 - WIDENING_GREAT_LAKES_ESCORT.0 + 1) as usize,
            "the walk is whole"
        );
        let mut own: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (RUN211_TAIL + 1..=WORD_BLOCK).contains(f) {
                *own.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let own: Vec<(i64, i64, i64, usize)> =
            own.into_iter().map(|((f, w, o), n)| (f, w, o, n)).collect();
        // **On run218's own blocks up to the word: nothing** (item 742).
        // Before it, `1/23` — three figures, blocked by **The Despot**
        // (`collide_o 79`) from 16433 with `collide` counting 22..26 —
        // parted on 16460 on 26 keys and on 16461 on 17: on sim-frame
        // 16459 `move_step`'s give-up (`collide >= 0x1a`) widened its
        // `tolerance` to 372 and this crate then took the step to
        // (41632, 21440). The original takes no step: it pops the leg
        // where it stands (`docs/COLLISION.md` §17).
        //
        // **The value diff on 16459 (block 16460), both sides now**:
        // `1/23` at (41632, 21466), `tolerance` 372 (twice the Manhattan
        // 186 still owed to (41632, 21280)), `path:length` 26 → 25,
        // `move.dest` 0 — every one a row `widen_block` compares, and none
        // parts; on 16461 the next leg, (41824, 21024), is blocked again
        // with `collide 27`, as the original's is.
        pin_eq!(
            own,
            [],
            "who parts first on run218's own blocks up to the word's, and on how many keys"
        );
        let walked_23: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| (*w, *o) == (1, 23) && *f > RUN211_TAIL)
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            walked_23.is_empty(),
            "`1/23` parts on no new key to run218's last block: {walked_23:?}"
        );
        let group67: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                *w == 1
                    && (76..=79).contains(o)
                    && (WIDENING_GREAT_LAKES_PATRIOT.1 + 1..=WORD_BLOCK).contains(f)
            })
            .map(|((w, o, what), (f, r))| format!("{f} {w}/{o} {what}: {r}"))
            .collect();
        pin!(
            group67.is_empty(),
            "group 67 holds on run202's, run211's and run218's blocks up to the word: {group67:?}"
        );
        let under = firsts.values().filter(|(f, _)| *f <= RUN211_TAIL).count();
        let mid = firsts
            .values()
            .filter(|(f, _)| (RUN211_TAIL + 1..=WORD_BLOCK).contains(f))
            .count();
        // The floor's rows standing on the old word's block: 355 before
        // item 742, whose give-up took `1/23`'s 43 keys off it.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 168.
        // Item 1455: 99 → 98, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
            // Item 1072 took four (312 → 308): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1114 took four (308 → 304): the caravan's leg faces its bearing and its trade order carries no bit 4 (`order:move.angle`, `dest_angle`, `order:flags`, `order:action`; `docs/CARAVAN.md` §11.3).
            // Item 1111 took the AI citizens' four Militia-line rows (`myhits`, `hits:myhits`, `hits_left`, `mylos`: `docs/GOLDEN.md` §48).
            // Item 1330 took 50 (165 → 114): the births' `form` (`docs/GROUPS.md` §24.3).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            38, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on the old word's block, 16461"
        );
        // **The floor**: run211's walk (398, nothing on run211's own blocks
        // since item 736), and every key to the window's end. 398/43/957
        // before item 742: `1/23`'s 43 keys on 16460..16461 and the 516
        // above them went with the give-up's step (`docs/COLLISION.md`
        // §17), nothing arriving — **nothing parts on 15860..16711**, and
        // the word left run218 for 17099.
        // Item 882 took three (398/0/398 → 395/0/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (395/0/395 → 392/0/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (392/0/392 → 382/0/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (382/0/382 → 378/0/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (378/0/378 → 377/0/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((377, 0, 377) → (373, 0, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (353/0/353 → 337/0/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (337, 0, 337).
        // Item 1330 took 57 (329/0/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run226 — Great Lakes' word 17099, widened whole, both directions**
    /// (item 742). run218's line past its last block, over
    /// [`WIDENING_GREAT_LAKES_GIVEUP`]: six blocks shared with run218, the
    /// 388 up to the word, its block, and 250 past it. [`widen_great_lakes`]
    /// from run123's 11400 across fourteen captures, with every player-1
    /// pool list from run135's first block.
    ///
    /// The word's frame, 17099, writes block **17100**.
    fn run226_s_word_frame_is_widened_whole(result: Widened) {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_GIVEUP_BLOCK;
        /// run218's last block: everything above it is run226's.
        const RUN218_TAIL: i64 = WIDENING_GREAT_LAKES_ESCORT.1;
        let Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        } = result;
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (WIDENING_GREAT_LAKES_GIVEUP.1 - WIDENING_GREAT_LAKES_GIVEUP.0 + 1) as usize,
            "the walk is whole"
        );
        let mut own: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (RUN218_TAIL + 1..=WORD_BLOCK).contains(f) {
                *own.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let own: Vec<(i64, i64, i64, usize)> =
            own.into_iter().map(|((f, w, o), n)| (f, w, o, n)).collect();
        // **On run226's own blocks up to the old word: nothing.** Item 742
        // left who=1's economy parting on 17086; item 757 built the
        // Pyramids' two terms (`docs/ECONOMY.md` §15) and left `1/9` and
        // `1/72` walking other paths to `1/2022` from 17088. **Item 776**:
        // both world searches of tick 17087 refused no cell the original
        // refused — `invalid_loc`'s cell arm, a forest-flagged cell under
        // `valid_wcoord`'s probe (`docs/PATHFINDER.md` §27) — and with it
        // all 303 priced steps of the tick are the original's
        // (`run240_s_world_at_17087_is_the_original_s`), and both walks go.
        pin_eq!(
            own,
            [],
            "who parts first on run226's own blocks up to the old word's, and on how many keys"
        );
        // **The move's value diff**: no leader row parts under the old
        // word, the caps least of all.
        let leader: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                (*w, *o) == (1, -1) && *f <= WORD_BLOCK && *f > RUN218_TAIL
            })
            .map(|((_, _, what), (f, r))| format!("{f} {what}: {r}"))
            .collect();
        pin!(
            leader.is_empty(),
            "who=1's leader record agrees to the old word: {leader:?}"
        );
        // **Item 757's word's blocks and item 776's**: what first parts on
        // 17101..17182, by unit. Item 757 left only the two walks here
        // (`1/72`'s arrival and mirror, `1/9`'s legs); item 776 took them,
        // and nothing parted until 776's word's block, 17182, where who=1's
        // `MAKE` slots 0, 1, 2 and 8 carried three wonders here (526, 528
        // and 527) and nothing there. **Item 785** took them: an easy AI's
        // wonder arm refuses once `wonder_mark` is up, and the Pyramids
        // raised it on 17085 (`docs/AI.md` §75). **Nothing parts on
        // run226's own blocks at all**, to its last, 17350 (the floor
        // below), and the word left run226 for 20568.
        let mut past: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, _), (f, _)) in &firsts {
            if (WORD_BLOCK + 1..=GREAT_LAKES_FOREST_CELL_BLOCK).contains(f) {
                *past.entry((*f, *w, *o)).or_default() += 1;
            }
        }
        let past: Vec<(i64, i64, i64, usize)> = past
            .into_iter()
            .map(|((f, w, o), n)| (f, w, o, n))
            .collect();
        pin_eq!(
            past,
            [],
            "who parts first on 17101..17182, the new word's block, and on how many keys"
        );
        let under = firsts.values().filter(|(f, _)| *f <= RUN218_TAIL).count();
        let mid = firsts
            .values()
            .filter(|(f, _)| (RUN218_TAIL + 1..=WORD_BLOCK).contains(f))
            .count();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (167, 167, 163).
        // Item 1330 took 50 (164/164/163 → 114/114/113): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (98, 98, 97) → (97, 97, 96), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            (
                standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
                standing
                    .get(&GREAT_LAKES_PYRAMIDS_BLOCK)
                    .map_or(0, BTreeMap::len),
                standing
                    .get(&GREAT_LAKES_FOREST_CELL_BLOCK)
                    .map_or(0, BTreeMap::len),
            ),
            // 349 and 344 before item 776, whose searches took the two
            // walks' rows off both. 17182 was the floor's 311 and the
            // `MAKE` list's twelve (323) until item 785: the twelve went,
            // and so did four standing rows the wonder pass had kept —
            // `MAKE[0]`, `[1]` and `[8]`'s `city` and `SITE[1].reg`.
            // Item 1072 took four (311/311/307 → 307/307/303): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1111 took the AI citizens' four Militia-line rows (`myhits`, `hits:myhits`, `hits_left`, `mylos`: `docs/GOLDEN.md` §48).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (37, 37, 36), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on 17100 (item 742's word's block), 17129 \
             (757's) and 17182 (776's)"
        );
        // **The floor**: run218's walk (398, nothing on run211's or
        // run218's own blocks since item 742), nothing up to the old
        // word's block since item 776 (38 keys of 17088 before it), and
        // every key to the window's end (1256 before item 776, 876 before
        // item 785 — none since, to 17350).
        // Item 882 took three (398/0/398 → 395/0/395): `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 890 took three (395/0/395 → 392/0/392): who=1's `MAKE[1]`, `[3]` and `[4]`
        // `escrow` from 12382 and 12385, 0 here against 1 before; a gather
        // offer is escrowed on every exit of the head test (`docs/AI.md` §77.5).
        // Item 904 took ten (392/0/392 → 382/0/382): run178's who=1 `SITE[4]`
        // and `SITE[8]`, the rock arm's city site at (47, 33) (`docs/AI.md` §78).
        // Item 1072 took four (382/0/382 → 378/0/378): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1106 took one (378/0/378 → 377/0/377): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 14536 against 14080 on 14537.
        // Item 1115 took the caravan's four rows here ((377, 0, 377) → (373, 0, 373); `docs/CARAVAN.md` §11.3).
        // Item 1206 took sixteen (353/0/353 → 337/0/337): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (337, 0, 337).
        // Item 1330 took 57 (329/0/329 → 272/0/272): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1354 took 29 (the Citizens' `mylos` on 12563, the cache; `docs/VISION.md` §2).
        // Item 1455: (223, 0, 223) → (221, 0, 221), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        // Item 1461: (216, 0, 216) → (210, 0, 210); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            (under, mid, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
            (127, 0, 127), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
    }

    /// **run243 — Great Lakes' word 20568, widened whole, both directions**
    /// (item 785). run226's line over [`WIDENING_GREAT_LAKES_STAND`]:
    /// run226's last six blocks, then run243 whole — 68 blocks into the
    /// word, its block, and 250 of runway. **Sized to the word, not to the
    /// gap**: no dump holds 17351..20499, so the walk compares nothing
    /// there, and what first parts on 20500 is either standing on 17350 or
    /// the gap's.
    ///
    /// The word's frame, 20568, writes block **20569**. Since item 795 the
    /// word is 20800, block **20801**, inside the same window.
    #[test]
    fn run243_s_word_frame_is_widened_whole() {
        cases_03::run243_s_word_frame_is_widened_whole();
    }

    /// **run80 — Great Lakes' end, widened whole, both directions** (item
    /// 899). run80 is the same game over [23960, 24000) and its end block,
    /// 24001, at `LEADERS=9` with every unit, building, city and guy
    /// record: the last blocks any dump holds. Since item 899 the draw
    /// stream agrees on every frame of run53's trace, so the long word is
    /// the capture's own end, 24000, and this is its widening.
    #[test]
    fn run80_s_word_frame_is_widened_whole() {
        cases_03::run80_s_word_frame_is_widened_whole();
    }

    /// **run96 — East Indies' end, widened whole, both directions** (item
    /// 919). run96 is the same game over [23960, 24000) and its end block,
    /// 24001, with every unit, building, city and guy record: the last
    /// blocks any dump holds. Since item 919 the draw stream agrees on every
    /// frame of run54's trace, so the long word is the capture's own end,
    /// 24000, and this is its widening.
    #[test]
    fn run96_s_word_frame_is_widened_whole() {
        cases_03::run96_s_word_frame_is_widened_whole();
    }

    /// **run294 — Great Lakes' departure, widened whole, both directions**
    /// (item 795). run226's chain, then run294 over
    /// [`WIDENING_GREAT_LAKES_DEPART`]'s tail: the four walkers' return leg
    /// leaving the far point, a bisection of the gap run243 left.
    #[test]
    fn run294_s_departure_is_widened_whole() {
        cases_03::run294_s_departure_is_widened_whole();
    }

    /// **The payoff probe of `run136_s_word_frame_is_widened_whole`, and
    /// since item 566 no probe at all** (`docs/PATHFINDER.md` §23.4,
    /// §24.6). Item 563 refused one unit cell, (810, 441), to `1/62`'s
    /// search on sim-frame 11901 by hand, and the detour round `1/27` came
    /// out the original's entry for entry: north by (38904, 21096), eight
    /// entries. run138 then showed what refuses it. `1/27`'s `find_upath`
    /// on the same frame pre-walks its goal along row 441, and every cell is
    /// refused by `1/62`'s block. It returns early without `kill_lists`, and
    /// `1/62`'s search reads those verdicts out of the pathfinder's memo.
    /// With the memo carried ([`sim::Sim::path_memo`]), the plan is the
    /// original's with nothing placed, `1/64` waits as the original's does,
    /// and no record of any unit first parts on blocks 11897..11905.
    #[test]
    fn run136_s_word_is_one_cell_of_1_62_s_search() {
        cases_03::run136_s_word_is_one_cell_of_1_62_s_search();
    }

    /// **The payoff probe of `run130_s_word_frame_is_widened_whole`, as an
    /// assertion** (item 554, `docs/GROUPS.md` §22.3). Seat this
    /// crate's `1/64` on the original's point before frame 11688, the frame
    /// `1/62` plans its collision detour on, and nothing else: the detour
    /// is then the original's cell for cell, and `1/62` stops on the word's
    /// frame at the original's point with the original's clock. So the
    /// word is `1/64`'s lag, and the detour search is not.
    ///
    /// **Since item 557 the probe seats `1/64` where it already stands**
    /// (`docs/GROUPS.md` §23): the lag was the back-pointer, and with it
    /// carried `1/64` reaches 11688 on the original's point unaided. The
    /// test asserts that first, and keeps the counterfactual behind it.
    #[test]
    fn run130_s_word_is_1_64_s_lag() {
        cases_04::run130_s_word_is_1_64_s_lag();
    }

    #[test]
    fn run100_s_word_block_is_every_record_the_dump_carries() {
        cases_04::run100_s_word_block_is_every_record_the_dump_carries();
    }

    /// **The scholar job run100 prints, frame for frame** — the other
    /// half of item 408, and the half that says the birth at 9510 is the
    /// job queued at 9382 rather than a coincidence.
    ///
    /// `BUILDQUEUE` writes `queued` and each live slot's `type`,
    /// `job_counter` and three `(good, cost)` pairs. University `1/2019`
    /// holds one live slot from block 9383 to block 9510 — type **52**,
    /// `SCHOLARS`, 40 **wealth** (`good[0] 2`) — and its counter runs
    /// +100 a frame from 100 to 12700 and then **12750**, a clamped last
    /// step rather than a thirteenth hundred. Block 9511 has `queued 0`
    /// and the unit `1/51` standing in the record.
    ///
    /// Read off the original alone, so it stands whatever this crate
    /// does. It is the falsifier for any later claim that 9510's birth
    /// is something other than that queue emptying.
    ///
    /// **Keyed on [`GREAT_LAKES_SIXTH_SCHOLAR`] rather than on the
    /// headline since item 442**, which moved the word to 10161 and put a
    /// *second* scholar job (counter 2100 at block 10162) inside the old
    /// window. The test is about this birth; reading the window's end off
    /// a moving word made it report the headline's motion as a failure of
    /// the original's own record.
    #[test]
    fn run100_s_scholar_job_runs_to_the_word() {
        cases_04::run100_s_scholar_job_runs_to_the_word();
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
        cases_04::run97_s_window_clocks_are_the_original_s();
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
        cases_04::run98_s_window_clocks_are_the_original_s();
    }

    /// **run82's leader record, and the AI's wealth on it** (item 573).
    /// run82 is run54's game with `LEADERS=9` over [6860, 6930) — the one
    /// East Indies capture between run59's 5400 and run96's 23960 that
    /// prints the stockpile, and the nearest to 9576, where the economic
    /// script's `at_least_type(who, 130, "Wealth")` buys the sixth Scholar
    /// in the original and did not here. run98 and run99 print only
    /// `LEADERS=1`'s stub, so the wealth on 9576 itself is on no disk.
    ///
    /// **Player 1's wealth bucket read 226 against 246 on every block**,
    /// twenty short, with its `resources`, `income` and `rate` agreeing;
    /// run59's [5150, 5400] agrees to the unit, so the twenty was lost in
    /// between. It is the caravan's two first-arrival bonuses, on 6512 and
    /// 6767: `City::new_caravan@00739750` pays `(epoch[2] + 1) · 10`
    /// (`0073979f`, `LeaderDataEncrypt +0xf0`, **Commerce**), and this crate
    /// read Civic — 20 each against 30 (`docs/CARAVAN.md` §7.3, §9). Made to
    /// fail on purpose by reading Civic again.
    #[test]
    fn run82_s_leader_record_is_the_original_s_wealth() {
        cases_04::run82_s_leader_record_is_the_original_s_wealth();
    }

    /// **run99 — East Indies' word 9711, widened whole, both directions**
    /// (item 573). run99 is run98's line over `[8780, 10399]`, nine blocks
    /// shared with run98 and the only dump that reaches the word; it was
    /// taken on 2026-09-18 for this frontier and no item had walked it.
    /// The walk runs **backwards across both captures** — run98 from
    /// [`WIDENING_EAST_INDIES`]'s floor, then run99 from 8789 — so a cause
    /// that spends no draw is still a row here. Every record
    /// [`widen_block`] reads on every unit and building of every player,
    /// and the leader record for both players; each key's first parting
    /// block is kept, with the value diff beside it. Neither capture
    /// carries `GROUPS`, so there is no pool list to compare.
    ///
    /// The word's frame enters on block **9711** and leaves on **9712**
    /// (the trace's frame `f` writes block `f + 1`).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_STANDING=<lo>-<hi>` every row parting on
    /// each block of the window.
    #[test]
    fn run99_s_word_frame_is_widened_whole() {
        cases_04::run99_s_word_frame_is_widened_whole();
    }

    /// **run139 — East Indies 9983's make list, widened whole** (item
    /// 576).
    ///
    /// run99 carries the word's units and buildings at `LEADERS=1`, whose
    /// stub prints no list, no stockpile and no step; run139 is the same
    /// game at `LEADERS=9` over [`WIDENING_EAST_INDIES_MAKE`], wholly
    /// inside run99's window and byte for byte on every record but the
    /// leader's own kinds. This walks it **both ways**: every record
    /// [`widen_block`] reads on every unit and building of every player,
    /// and every key of [`crate::diff::leader::rows`] for both leaders —
    /// the stockpile, the step, the census, the per-type muster, the sites
    /// and the eleven `MAKEOBJECT` slots. Each key's first parting block is
    /// kept with the value diff beside it.
    ///
    /// `RON_MAKE_SLOTS=<lo>-<hi>` prints player 1's two lists slot for
    /// slot on each block of the window.
    #[test]
    fn run139_s_word_frame_is_widened_whole() {
        cases_04::run139_s_word_frame_is_widened_whole();
    }

    /// **run155 — East Indies' word 11069, widened whole, both directions**
    /// (item 620). run152's line past its last block, over
    /// [`WIDENING_EAST_INDIES_EXPLORE`]: ten blocks shared with run152, the
    /// 30 up to the word, its block, and 209 past it, which no capture had
    /// printed. [`widen_east_indies`] with gaia's animals; `compare` reads
    /// every unit's outermost container since this item (parked 598).
    ///
    /// The word's frame, 11069, writes block **11070**. The same item moved
    /// the word to 11590, past the window; the test keeps the move's value
    /// diff (`docs/ORDERS.md` §6.4).
    #[test]
    fn run155_s_word_frame_is_widened_whole() {
        cases_04::run155_s_word_frame_is_widened_whole();
    }

    /// **run159 — East Indies' word 11590, widened whole, both directions**
    /// (item 629). run155's line past its last block, over
    /// [`WIDENING_EAST_INDIES_IDLE`]: ten blocks shared with run155, the
    /// 310 up to the word, its block, and 308 past it, which no capture had
    /// printed. [`widen_east_indies`] with gaia's animals.
    ///
    /// The word's frame, 11590, writes block **11591**. The same item moved
    /// the word to 11747, inside the window (block 11748): a deployed
    /// Merchant's four tiles, `SpellType::cast_unpack`'s merchant arm
    /// (`docs/MERCHANT.md` §3.2). Item 642 widened 11747's blocks and moved
    /// the word to 13640, past the window: `go_here` reads the human's
    /// city count (`docs/TRANSPORT.md` §9.4). The test keeps both moves'
    /// value diffs.
    #[test]
    fn run159_s_word_frame_is_widened_whole() {
        cases_04::run159_s_word_frame_is_widened_whole();
    }

    /// **run166 — East Indies' word 13640, widened whole, both directions**
    /// (item 643). run159's line with `GROUPS=1`, over
    /// [`WIDENING_EAST_INDIES_WRAP`]: 60 blocks under the word, its block,
    /// and 59 past it. Sized to the word rather than to the 1,741 blocks
    /// from run159's last, so it shares no block with another capture and
    /// its first block carries everything that parted in the gap.
    /// [`widen_east_indies`] with gaia's animals.
    ///
    /// The word's frame, 13640, writes block **13641**. There ours spent
    /// three `Guy::inc_time+0x271` wraps — `1/21`, `1/32` and the sheep
    /// `8/1`, in that order — against the original's two, until the same
    /// item found the Galley's default by its name case-folded
    /// (`docs/ANIM.md` §12) and moved the word to 15782, past the window.
    /// The test keeps the move's value diff.
    #[test]
    fn run166_s_word_frame_is_widened_whole() {
        cases_05::run166_s_word_frame_is_widened_whole();
    }

    /// **run78 — East Indies' old word 15782, the move's value diff**
    /// (item 706). run78 is `LEADERS=1` over [15700, 15900], so the leader's
    /// make list and personality are not printed (2,585 keys unprinted),
    /// and no test had walked it. [`widen_east_indies`] with gaia's
    /// animals, over the window.
    ///
    /// On tick 15782 who=1's Senate finishes Republic (`gov` −1 → 624 on
    /// block 15783), and `Build::finished` trains its patriot, The Senator
    /// `1/60` (`docs/TECH.md` §"The government patriot"). Before item 706
    /// the dump held `1/60` alone on 15783. The extra `Guy::init_real`
    /// draws were missing, so the sheep `8/2` parted on 15789, and 281
    /// rows followed it from 15811 to the window's end: 726 keys in all.
    /// Now `1/60` is born where the original's is. It parts on `form`
    /// (parked 646) ~~and on its order point, because the original's Senator
    /// is still inside its Senate, as Great Lakes' Despot is on run192's
    /// 14983~~ — the order point agrees since item 718: `come_out`'s
    /// `update_action` puts it on the exit block (`docs/ORDERS.md` §29). Its pool group parts on 15784 (parked 674). The scout `1/0`'s
    /// order point on 15800 parts on both trees. It stands under East
    /// Indies' new word, 15985, whose delta is `Unit::think_scout`.
    #[test]
    fn run78_s_old_word_keeps_its_value_diff() {
        cases_05::run78_s_old_word_keeps_its_value_diff();
    }

    /// **run221 — East Indies' word 15985, widened whole, both directions**
    /// (item 708). run166's line without `DEATHS`, the pool's, over
    /// [`WIDENING_EAST_INDIES_SLOT`]: six blocks shared with run78, the 86
    /// up to the word, its block, and 250 past it. [`widen_east_indies`]
    /// with gaia's animals.
    ///
    /// The word's frame, 15985, writes block **15986**.
    #[test]
    fn run221_s_word_frame_is_widened_whole() {
        cases_05::run221_s_word_frame_is_widened_whole();
    }

    /// **run227 — East Indies' word 16683, widened whole, both directions**
    /// (item 752). run221's line over [`WIDENING_EAST_INDIES_WRAPWORD`]:
    /// seven blocks shared with run221, the 447 up to the word, its block,
    /// and 250 past it. [`widen_east_indies`] with gaia's animals.
    ///
    /// The word's frame, 16683, writes block **16684**.
    #[test]
    fn run227_s_word_frame_is_widened_whole() {
        cases_05::run227_s_word_frame_is_widened_whole();
    }

    /// **run233 — East Indies' word 17189, widened whole, both directions**
    /// (item 767). run227's line over [`WIDENING_EAST_INDIES_BLOCKWORD`]:
    /// six blocks shared with run227, the 255 up to the word, its block,
    /// and 250 past it. [`widen_east_indies`] with gaia's animals.
    ///
    /// The word's frame, 17189, writes block **17190**. Item 773 moved the
    /// word to **17403**, block **17404**, still inside the window: this
    /// test keeps the move's value diff on 17190 and pins the new word's
    /// block beside it.
    #[test]
    fn run233_s_word_frame_is_widened_whole() {
        cases_05::run233_s_word_frame_is_widened_whole();
    }

    /// **run251 — East Indies' word 17501, widened whole, both directions**
    /// (item 800). run233's line over [`WIDENING_EAST_INDIES_COLUMNWORD`]:
    /// the six blocks before the word's, its block, and 250 past it. No
    /// capture shares a block with it, so 17441..17495 is compared by no
    /// dump, and the floor is its first block. [`widen_east_indies`] with
    /// gaia's animals.
    ///
    /// The word's frame, 17501, writes block **17502**.
    #[test]
    fn run251_s_word_frame_is_widened_whole() {
        cases_05::run251_s_word_frame_is_widened_whole();
    }

    /// **run253 — East Indies' word 18182, widened whole, both directions**
    /// (item 811). run251's line over [`WIDENING_EAST_INDIES_MARKETWORD`]:
    /// the six blocks before the word's, its block, and 250 past it. No
    /// capture shares a block with it, so 17753..18176 is compared by no
    /// dump, and the floor is its first block. [`widen_east_indies`] with
    /// gaia's animals.
    ///
    /// The word's frame, 18182, writes block **18183**.
    #[test]
    fn run253_s_word_frame_is_widened_whole() {
        cases_05::run253_s_word_frame_is_widened_whole();
    }

    /// **run257 — East Indies' word 19182, widened whole, both directions**
    /// (items 822, 829 and 837). run253's line over
    /// [`WIDENING_EAST_INDIES_GUARDWORD`]: the six blocks before the word
    /// 18938's block, its block, and 250 past it. No capture shares a block
    /// with it, so 18434..18932 is compared by no dump, and the floor is its
    /// first block. [`widen_east_indies`] with gaia's animals.
    ///
    /// Item 822 took it for the word 18938 (block **18939**); item 829
    /// moved the word to 18999 (block **19000**) and item 837 to 19182
    /// (block **19183**), both inside it, and the walk keeps each move's
    /// value diff.
    #[test]
    fn run257_s_word_frame_is_widened_whole() {
        cases_05::run257_s_word_frame_is_widened_whole();
    }

    /// **run269 — East Indies' word 19413, widened whole, both directions**
    /// (item 839). run257's line over [`WIDENING_EAST_INDIES_TURNWORD`]:
    /// the six blocks before the word's block, the block, and 250 past it.
    /// run257 ends on 19189, so 19190..19407 is compared by no dump.
    /// [`widen_east_indies`] with gaia's animals.
    #[test]
    fn run269_s_word_frame_is_widened_whole() {
        cases_05::run269_s_word_frame_is_widened_whole();
    }

    /// **run277 — East Indies' word 20007, widened whole, both directions**
    /// (item 857). run269's line over [`WIDENING_EAST_INDIES_BLOCKEDWALK`]:
    /// the six blocks before the word's block, the block, and 250 past it.
    /// run269 ends on 19664, so 19665..20001 is compared by no dump.
    /// [`widen_east_indies`] with gaia's animals.
    #[test]
    fn run277_s_word_frame_is_widened_whole() {
        cases_05::run277_s_word_frame_is_widened_whole();
    }

    /// **run289 — East Indies' word 20782, widened whole, both directions**
    /// (item 880). run277's line over [`WIDENING_EAST_INDIES_USEMARKET`]:
    /// six blocks up to the word's block 20783, the block, and 250 past it.
    /// run277 ends on 20258, so 20259..20776 is compared by no dump; the
    /// same game is the draw stream's word against run54. [`widen_east_indies`]
    /// with gaia's animals.
    #[test]
    fn run289_s_word_frame_is_widened_whole() {
        cases_06::run289_s_word_frame_is_widened_whole();
    }

    /// **run299 — East Indies' word 23182, widened whole, both directions**
    /// (item 890), and **the word 23420 it moved to inside the window**
    /// (item 904): its block 23421 and the runway to it. run289's line over [`WIDENING_EAST_INDIES_WONDERPRICE`]:
    /// six blocks up to the word's block 23183, the block, and 250 past it.
    /// run289 ends on 21045, so 21046..23176 is compared by no dump; the
    /// same game is the draw stream's word against run54.
    /// [`widen_east_indies`] with gaia's animals.
    #[test]
    fn run299_s_word_frame_is_widened_whole() {
        cases_06::run299_s_word_frame_is_widened_whole();
    }

    /// **The squad push on tick 20000 reads the record its pool slot last
    /// held** (item 865, `docs/GROUPS.md` §28). The first killer of
    /// East Indies' word 20007 is `1/65` and `1/66` in each other's slots
    /// on run277's first block. The whole parting is the move's mirror.
    /// `add_to_army`'s `go_to` pushes `1/64`..`1/66` onto slot 69, and
    /// `Groups::copy_group@006fa690` leaves `facing` and `order_num` in the
    /// slot. The original's 69 was army 1's group on run269's last block
    /// (`facing 1`, `order_num 5`) and reads `facing 1`, `order_num 6` on
    /// 20002, so `compute_form` toggled a `facing` of 1 against a leader 90°
    /// off and laid out square. This crate's push seats a fresh record
    /// (`facing 0`, `order_num 0 → 1`) and mirrors. Both sides' values are
    /// pinned here. **This crate's half is the residue**: a slot keeps
    /// what `copy_group` does not copy only once the pool's numbering is
    /// the original's. It parted on block 239
    /// (`east_indies_pool_numbering_takes_who_1_s_building_groups`), and
    /// since item 870 it agrees through this block; the slot records'
    /// own `order_num` still part from run221's 15894 (§29).
    #[test]
    fn east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held() {
        cases_06::east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held();
    }

    /// **East Indies' pool numbering takes who=1's building groups**
    /// (items 865 and 870, `docs/GROUPS.md` §28.4, §29): block 239 was the
    /// first `group` a who=1 unit held differently, and the root of every
    /// pool-id row after it (parked 689). run45 prints the pool from the
    /// first frame. The original pushes who=1's **building groups**:
    /// `[2005]` onto 64 on frame 1, and `[2000]` onto 66 on frame 176. So `think_scout`'s
    /// repush of `[0]` on tick 238 does not equal `last_group` (66).
    /// `get_open_slot` takes 64, a building group that is not
    /// `last_group`, and 1/0 moves 65 → 64. The pushes are the AI
    /// scripts' `train_unit*` and `research_tech_with_cost`: each pushes
    /// the producing building before `Group::action_queue_up` (§28.4).
    /// Until item 870 this crate pushed none, so the repush equalled
    /// `last_group` (65) and stayed: `1/0` on 65 here against 64 there.
    #[test]
    fn east_indies_pool_numbering_takes_who_1_s_building_groups() {
        cases_06::east_indies_pool_numbering_takes_who_1_s_building_groups();
    }

    /// **The pool walk** (item 870, `docs/GROUPS.md` §29): run54's game
    /// ticked once, and on every block any listed East Indies dump holds,
    /// who=0's and who=1's `UnitData::group` (`+0x80`) against this crate's
    /// [`sim::Sim::pool_group_of`], each dump's first parting printed with
    /// its units. `RON_POOL_WALK_RECORDS=1` adds each `GROUPDATA` slot's
    /// `facing` and `order_num` against [`sim::Sim::pool_records`], and a
    /// slot holding more than one record here. A probe, not a pin:
    ///
    /// ```text
    /// RON_POOL_WALK=gamelog-run98-eastindies-valuewindow.txt,… \
    ///   [RON_POOL_WALK_STOP=<block>] [RON_POOL_WALK_ALL=1] \
    ///   cargo test --release -p rondata --lib east_indies_pool_walk -- --ignored --nocapture
    /// ```
    ///
    /// The first capture listed that holds a block reads it, so list them
    /// in run order — **and a dump that prints `GROUPDATA` before one
    /// that does not**, or the record mode is silent on the shared blocks
    /// (run42 shadowed run45's 1..901 for item 870). 865's scratch
    /// comparison reached 8369 in a minute, and 870 walked every East
    /// Indies dump with it, pointers and records.
    ///
    /// `RON_POOL_WALK_HIST=<who>` prints this crate's slots `who·64 +
    /// 0..16` every tick one changes — every record on the index, with
    /// `order_num`, `facing`, `stamp`, `form` and the list — for the
    /// history a parting needs (item 880, `docs/GROUPS.md` §30).
    /// `RON_POOL_WALK_MAP=greatlakes` walks run53's game against Great
    /// Lakes' dumps instead.
    #[test]
    #[ignore]
    fn east_indies_pool_walk() {
        cases_06::east_indies_pool_walk();
    }

    /// **run261 — East Indies' gap over army 1's close, widened whole, both
    /// directions** (item 829). run253's line over
    /// [`WIDENING_EAST_INDIES_CLOSE`]: its last six blocks and 250 into the
    /// gap 18434..18932, which run253 and run257 leave uncompared. Army 1
    /// closes on tick 18682 (`find_target`, `docs/ARMY.md` §22), and the
    /// original's group 71 reads `army −1` from block 18683 with its list
    /// kept. [`widen_east_indies`] with gaia's animals.
    #[test]
    fn run261_s_gap_is_widened_whole() {
        cases_06::run261_s_gap_is_widened_whole();
    }

    /// **run152 — East Indies' word 10982, widened whole, both directions**
    /// (item 613). run149's line past its last block, over
    /// [`WIDENING_EAST_INDIES_GATHER`]: ten blocks shared with run149, the
    /// 103 up to the word, its block, and 56 past it, which no capture had
    /// printed. [`widen_east_indies`] with gaia's animals.
    ///
    /// The word's frame, 10982, writes block **10983**. The same item moved
    /// the word to 11069, past the window; the test keeps the move's value
    /// diff (`docs/AI.md` §62).
    #[test]
    fn run152_s_word_frame_is_widened_whole() {
        cases_06::run152_s_word_frame_is_widened_whole();
    }

    /// **run149 — East Indies' word 10782, widened whole, both directions**
    /// (item 608). run143's line past its last block, over
    /// [`WIDENING_EAST_INDIES_MERCS`]: ten blocks shared with run143, the 43
    /// up to the word, its block, and 96 past it, which no capture had
    /// printed. [`widen_east_indies`] with gaia's animals, because the
    /// word's frame parts on an animal's draw here.
    ///
    /// The word's frame, 10782, writes block **10783**. The same item moved
    /// the word to 10982, past the window; the test keeps the move's value
    /// diff.
    #[test]
    fn run149_s_word_frame_is_widened_whole() {
        cases_06::run149_s_word_frame_is_widened_whole();
    }

    /// One of East Indies' `LEADERS=9` windows walked whole, both
    /// directions — the walk run143's test was, made a helper when run149
    /// needed the same one (item 608). `capture` over `window`, from run54's
    /// start with run38's `DUMP_ALL` head: every record [`widen_block`]
    /// reads on every unit and building of every player, every unit's
    /// outermost container, every key of [`crate::diff::leader::rows`] and
    /// the per-region arrays for both leaders, and — with `gaia` — gaia's
    /// animals, which `compare` never reads (parked 577). Each key's first
    /// parting block is kept with the value diff beside it; every figure
    /// whose animation changes on either side within two blocks of a `near`
    /// block is listed (`docs/COMBAT.md` §44.2.1).
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_STANDING=<lo>-<hi>` every row parting on
    /// each block of the window, the leaders' included; `RON_CITIZENS=<lo>-
    /// <hi>` player 1's census and citizens.
    fn widen_east_indies(
        name: &str,
        capture: &str,
        window: (i64, i64),
        near: &[i64],
        gaia: bool,
    ) -> Option<Widened> {
        widen_east_indies_on(
            ("gamelog-run54-islands-24k-trace.txt", "rontrace-run54.log"),
            name,
            capture,
            window,
            near,
            gaia,
            false,
        )
    }

    /// [`widen_east_indies`] on another East Indies game's start: the
    /// second pair's run346 (item 971) is run54's lobby at Toughest, so its
    /// widenings stand the simulation up from run346's own start dump and
    /// pasture, with run38's head borrowed on the same sameness gate.
    /// `records` adds [`crate::diff::second::widen_records`]'s rows — the
    /// group record, the attack order's row and each figure's aim — as
    /// Great Lakes' second-pair widenings have them since item 1061; East
    /// Indies' word window asks for them since item 1106.
    pub(crate) fn widen_east_indies_on(
        base: (&str, &str),
        name: &str,
        capture: &str,
        window: (i64, i64),
        near: &[i64],
        gaia: bool,
        records: bool,
    ) -> Option<Widened> {
        let result = widen_east_indies_walk(base, name, capture, window, near, gaia, records, true);
        if std::env::var_os("RON_VERIFY_EAST_REUSE").is_some() {
            let legacy =
                widen_east_indies_walk(base, name, capture, window, near, gaia, records, false);
            assert_eq!(
                result, legacy,
                "{name}: decoded East Indies replay changed its complete report"
            );
            eprintln!("{name}: decoded East Indies replay matches the complete legacy report");
        }
        result.map(|(report, _)| report)
    }

    fn verify_east_initial(path: &str, actual: &Initial<'_>) {
        let text = crate::capture::read(path);
        let log = Log::parse(&text);
        let mut expected = log.initial().expect("legacy replay initial");
        expected.frame_bodies.clear();
        assert!(
            *actual == expected,
            "bounded East Indies setup changed an Initial field: {path}"
        );
    }

    fn build_east_initial<'a>(
        loaded: &crate::load::Loaded,
        mut init: Initial<'a>,
        sibling: &Initial<'a>,
        trace: &crate::trace::Trace,
    ) -> Built {
        borrow_from_siblings(&mut init, &[sibling]);
        borrow_pasture(&mut init, trace);
        build_sim(loaded, &init, Tuning::RON)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Existing replay inputs plus differential reader selection"
    )]
    fn widen_east_indies_walk(
        (base, base_trace): (&str, &str),
        name: &str,
        capture: &str,
        window: (i64, i64),
        near: &[i64],
        gaia: bool,
        records: bool,
        reuse: bool,
    ) -> Option<(Widened, usize)> {
        let reuse = reuse && std::env::var_os("RON_DISABLE_EAST_REUSE").is_none();
        let mut oracle_store = crate::diff::oracle::Store::new(reuse).with_regions();
        let inst = install()?;
        let (Some(path), Some(sib), Some(tr), Some(r143)) = (
            dump(base),
            dump("gamelog-run38-islands-start.txt"),
            trace(base_trace),
            dump(capture),
        ) else {
            eprintln!("skipping: no {base}/{name} capture (set RON_GAMELOG_DIR)");
            return None;
        };
        let (first_block, tail) = window;
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r143).unwrap();
        let loaded = crate::load::load(&inst).unwrap();
        let mut built = if reuse {
            let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
            let mut sibling = crate::capture::indexed::IndexedCapture::open(&sib).unwrap();
            sibling
                .with_replay_initial(|sib_init| {
                    if std::env::var_os("RON_VERIFY_EAST_REUSE").is_some() {
                        verify_east_initial(&sib, &sib_init);
                    }
                    source
                        .with_replay_initial(|init| {
                            if std::env::var_os("RON_VERIFY_EAST_REUSE").is_some() {
                                verify_east_initial(&path, &init);
                            }
                            build_east_initial(&loaded, init, &sib_init, &tr)
                        })
                        .unwrap()
                })
                .unwrap()
        } else {
            let text = crate::capture::read(&path);
            let sib_text = crate::capture::read(&sib);
            let log = Log::parse(&text);
            let sib_log = Log::parse(&sib_text);
            let sib_init = sib_log.initial().expect("run38 is a start dump");
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &[&sib_init]);
            borrow_pasture(&mut init, &tr);
            build_sim(&loaded, &init, Tuning::RON)
        };
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows) = (0usize, 0usize, 0usize);
        let mut housed = 0usize;
        let row_walk: Vec<(i64, i64)> = std::env::var("RON_ROW_WALK")
            .map(|w| {
                w.split(',')
                    .filter_map(|u| {
                        let (a, b) = u.split_once('/')?;
                        Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut walked: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
        let mut anims: BTreeMap<(i64, i64), (Option<i64>, Option<i64>)> = BTreeMap::new();
        // Every figure whose animation changes on either side on the
        // word's blocks: `(block, who, o, theirs changed, ours changed)`.
        let mut changed: Vec<(i64, i64, i64, bool, bool)> = Vec::new();
        let mut standing: BTreeMap<i64, BTreeMap<(i64, i64, String), String>> = BTreeMap::new();
        let mut army_lists: ArmyLists = BTreeMap::new();
        for f in 0..=tail {
            built.tick();
            let n = f + 1;
            if n < first_block {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let cached = oracle_store
                .enabled()
                .then(|| oracle_store.read(&mut ix, at));
            let fresh_frame;
            let frame = if let Some(value) = &cached {
                &value.frame
            } else {
                fresh_frame = ix.frame_state(at).unwrap();
                &fresh_frame
            };
            blocks += 1;
            debug_watch(&built, n);
            // `RON_CITIZENS=<lo>-<hi>`: player 1's peasant census and every
            // citizen's container and order list, both sides (item 592).
            if site_window_named("RON_CITIZENS").is_some_and(|(a, b)| (a..=b).contains(&n)) {
                let raw = ix.read_frame(at).unwrap();
                let flog = Log::parse(&raw);
                if let Some(block) = flog.leader_block(n, 1) {
                    let t = crate::diff::leader::theirs(&block);
                    let mine: BTreeMap<String, i64> = crate::diff::leader::rows(&loaded, &built, 1)
                        .into_iter()
                        .collect();
                    let line: Vec<String> =
                        ["peasants", "free_peasants", "xport_peasants", "gatherers"]
                            .iter()
                            .map(|k| format!("{k} {:?}/{:?}", mine.get(*k), t.get(*k)))
                            .collect();
                    eprintln!("  census {n}: {}", line.join(" "));
                }
                for (u, x) in built.sim.units.iter().enumerate() {
                    if !x.alive()
                        || x.owner != 1
                        || built.sim.worker_of(u) != sim::orders::Worker::Citizen
                    {
                        continue;
                    }
                    let th = frame
                        .units
                        .iter()
                        .find(|t| t.who == 1 && t.o == i64::from(x.index));
                    let ords: Vec<i64> = x.orders.iter().map(|o| o.index() as i64).collect();
                    let tords: Vec<i64> = th
                        .map(|t| t.orders.iter().map(|o| o.index).collect())
                        .unwrap_or_default();
                    eprintln!(
                        "    {n} 1/{} ({},{}) on_map {} in {:?}/{:?} theirs in {:?} \
                         orders ours {ords:?} theirs {tords:?}",
                        x.index,
                        x.pos.x,
                        x.pos.y,
                        x.on_map,
                        x.inside.map(|b| built.sim.buildings[b].index),
                        x.inside_unit.map(|b| built.sim.units[b].index),
                        th.and_then(|t| t.inside_up),
                    );
                }
            }
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (fr, rows) = widen_block(&built, frame, players, n, &mut here);
            compared += rows;
            housed += fr.inside_housed;
            // **Every unit's container** (item 592) is `compare`'s own
            // row since item 620 (parked 598): `widen_block` notes it as
            // `inside`, the key this walk wrote.
            let raw = cached.is_none().then(|| ix.read_frame(at).unwrap());
            let flog = raw.as_deref().map(Log::parse);
            let fresh_groups;
            let groups = if let Some(value) = &cached {
                &value.groups
            } else {
                fresh_groups = if records || near.iter().any(|b| (b - 2..=b + 2).contains(&n)) {
                    flog.as_ref()
                        .unwrap()
                        .frames()
                        .into_iter()
                        .find(|(k, _)| *k == n)
                        .map(|(_, block)| crate::gamelog::groups(block))
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                &fresh_groups
            };
            // **The group record and the attack order's row** (items 1061
            // and 1106), on the windows that ask for them.
            if records {
                if cached.is_some() {
                    compared += crate::diff::second::widen_records_from_groups(
                        &built, frame, groups, players, n, &mut here,
                    );
                } else if let Some((_, block)) = flog
                    .as_ref()
                    .unwrap()
                    .frames()
                    .into_iter()
                    .find(|(k, _)| *k == n)
                {
                    compared += crate::diff::second::widen_records(
                        &built, frame, block, players, n, &mut here,
                    );
                }
            }
            for who in 0..2usize {
                let fresh_leader;
                let t = if let Some(value) = &cached {
                    let Some(t) = &value.leaders[who] else {
                        continue;
                    };
                    t
                } else {
                    let Some(block) = flog.as_ref().unwrap().leader_block(n, who as i64) else {
                        continue;
                    };
                    let mut rows = crate::diff::leader::theirs(&block);
                    rows.extend(crate::diff::leader::region_theirs(&block));
                    fresh_leader = rows;
                    &fresh_leader
                };
                let mut mine = crate::diff::leader::rows(&loaded, &built, who);
                mine.extend(crate::diff::leader::region_rows(&built, who));
                // `RON_MAKE=<lo>-<hi>`: the make list slot for slot, both
                // sides, through the comparison's own keys (item 613).
                if who == 1
                    && site_window_named("RON_MAKE").is_some_and(|(a, b)| (a..=b).contains(&n))
                {
                    let m: BTreeMap<&str, i64> =
                        mine.iter().map(|(k, v)| (k.as_str(), *v)).collect();
                    for i in 0..11 {
                        let side = |get: &dyn Fn(&str) -> Option<i64>| -> String {
                            ["t", "val", "city", "num", "cat"]
                                .iter()
                                .map(|k| format!("{k} {:?}", get(&format!("MAKE[{i}].{k}"))))
                                .collect::<Vec<_>>()
                                .join(" ")
                        };
                        eprintln!(
                            "  make {n} [{i}] ours {} | theirs {}",
                            side(&|k| m.get(k).copied()),
                            side(&|k| t.get(k).copied())
                        );
                    }
                }
                for (k, v) in &mine {
                    let region = cached
                        .as_ref()
                        .and_then(|value| value.regions[who].as_ref());
                    let Some(&y) = region.and_then(|rows| rows.get(k)).or_else(|| t.get(k)) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    if *v != y {
                        here.entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
            // **Gaia's animals** (parked 577): [`widen_gaia`].
            if gaia {
                compared += widen_gaia(&built, frame, players, n, &mut here, &mut missing);
            }
            for &(w, o) in &row_walk {
                let now: Vec<String> = here
                    .iter()
                    .filter(|((hw, ho, _), _)| (*hw, *ho) == (w, o))
                    .map(|((_, _, what), (_, row))| format!("{what}: {row}"))
                    .collect();
                let e = walked.entry((w, o)).or_default();
                if *e != now {
                    eprintln!("  row walk {n} {w}/{o}: [{}]", now.join(" · "));
                    *e = now;
                }
            }
            // The first parting block prints by default (parked 830, the
            // sixteenth pass; the note at the first of these four sites).
            if firsts.is_empty()
                || site_window_named("RON_STANDING").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for ((w, o, what), (_, row)) in &here {
                    eprintln!("  standing {n} {w}/{o} {what}: {row}");
                }
            }
            if near.iter().any(|b| (b - 2..=b + 2).contains(&n)) {
                standing.insert(
                    n,
                    here.iter()
                        .map(|(k, (_, row))| (k.clone(), row.clone()))
                        .collect(),
                );
                // The armies' lists, on what the slot holds (item 800).
                let mut lists: BTreeMap<(i64, i64), (Vec<i64>, Vec<i64>)> = BTreeMap::new();
                {
                    for g in groups {
                        if g.army < 0 || g.members.is_empty() || g.who >= players as i64 {
                            continue;
                        }
                        compared::note("GroupDump", &["who", "army", "members"]);
                        compared::note("GroupMemberDump", &["o"]);
                        lists.entry((g.who, g.army)).or_default().1 =
                            g.members.iter().map(|m| m.o).collect();
                    }
                }
                for (w, side) in built.sim.armies.iter().enumerate().take(players) {
                    for (a, x) in side.list.iter().enumerate() {
                        if !x.valid || x.units.is_empty() {
                            continue;
                        }
                        lists.entry((w as i64, a as i64)).or_default().0 = x
                            .units
                            .iter()
                            .map(|&u| i64::from(built.sim.units[u].index))
                            .collect();
                    }
                }
                army_lists.insert(n, lists);
            }
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            for t in &frame.units {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = u8::try_from(t.who)
                    .ok()
                    .and_then(|w| built.sim.unit_by_o(w, o16));
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((t.who, t.o), (ta, oa));
                if !near.iter().any(|b| (b - 2..=b + 2).contains(&n)) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                eprintln!(
                    "  anim {n} {}/{}: theirs ({},{}) a{:?}->{:?} | ours {:?} a{:?}->{:?}",
                    t.who,
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    pt,
                    ta,
                    ours.map(|u| (built.sim.units[u].pos.x, built.sim.units[u].pos.y)),
                    po,
                    oa
                );
                changed.push((n, t.who, t.o, pt != ta, po != oa));
            }
        }
        eprintln!(
            "{name} widening: {blocks} blocks [{first_block}, {tail}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted, {} keys unprinted",
            firsts.len(),
            missing.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = *f > first_block;
            let window =
                site_window_named("RON_DEBUG_ROWS").is_some_and(|(lo, hi)| (lo..=hi).contains(f));
            if near || window {
                for r in rows {
                    eprintln!("  f{f} {r}");
                }
            } else {
                eprintln!("  f{f}: {} keys", rows.len());
            }
        }
        Some((
            Widened {
                firsts,
                missing,
                blocks,
                leader_rows,
                changed,
                housed,
                standing,
                army_lists,
            },
            compared,
        ))
    }

    /// **Gaia's animals, both sides, on one block** (parked 577: `compare`
    /// walks `0..players` and never reads `who 8`). Each animal's position
    /// and its first figure's clock, `cur_anim` and `cur_time`, against this
    /// crate's animal of the same number; a figure a short `GUY` record
    /// prints no clock for is a key unprinted, not a parting (item 919).
    /// Answers the rows compared. [`widen_east_indies_on`]'s since item 577,
    /// a helper since item 1066 walked Great Sahara's herds.
    pub(crate) fn widen_gaia(
        built: &Built,
        frame: &Frame,
        players: usize,
        n: i64,
        here: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
        missing: &mut std::collections::BTreeSet<String>,
    ) -> usize {
        let mut compared = 0usize;
        for t in frame.units.iter().filter(|t| t.who >= players as i64) {
            let ours = u8::try_from(t.who)
                .ok()
                .zip(i16::try_from(t.o).ok())
                .and_then(|(w, o)| built.sim.unit_by_o(w, o));
            let Some(u) = ours else {
                here.entry((t.who, t.o, "gaia:alive".into()))
                    .or_insert((n, "ours none theirs alive".into()));
                continue;
            };
            let x = &built.sim.units[u];
            let g = x.guys.first();
            let tg = t.guys.first();
            let rows = [
                (
                    "gaia:pos",
                    (i64::from(x.pos.x), i64::from(x.pos.y)) == (t.pos.x, t.pos.y),
                    format!(
                        "ours ({},{}) theirs ({},{})",
                        x.pos.x, x.pos.y, t.pos.x, t.pos.y
                    ),
                ),
                (
                    "gaia:cur_anim",
                    g.map(|g| i64::from(g.anim)) == tg.and_then(|g| g.cur_anim),
                    format!(
                        "ours {:?} theirs {:?}",
                        g.map(|g| g.anim),
                        tg.and_then(|g| g.cur_anim)
                    ),
                ),
                (
                    "gaia:cur_time",
                    g.map(|g| i64::from(g.cur_time)) == tg.and_then(|g| g.cur_time),
                    format!(
                        "ours {:?} theirs {:?}",
                        g.map(|g| g.cur_time),
                        tg.and_then(|g| g.cur_time)
                    ),
                ),
            ];
            // A short `GUY` record (run96's) prints no clock: that
            // is a key unprinted, not a parting (item 919).
            let printed = tg.is_some_and(|g| g.cur_anim.is_some());
            for (what, same, row) in rows {
                if what != "gaia:pos" && !printed {
                    missing.insert(what.into());
                    continue;
                }
                compared += 1;
                if !same {
                    here.entry((t.who, t.o, what.into())).or_insert((n, row));
                }
            }
        }
        compared
    }

    /// What [`widen_east_indies`] found.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) struct Widened {
        pub(crate) firsts: std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
        pub(crate) missing: std::collections::BTreeSet<String>,
        pub(crate) blocks: usize,
        pub(crate) leader_rows: usize,
        /// `(block, who, o, theirs changed, ours changed)`.
        pub(crate) changed: Vec<(i64, i64, i64, bool, bool)>,
        /// Unit-blocks `compare`'s container row read on a housed unit
        /// (parked 598): what says the row saw a garrison at all.
        pub(crate) housed: usize,
        /// Every row parting on each block within two of a `near` block,
        /// keyed as `firsts` is: what a key that parted earlier says on
        /// the word's own frame, which `firsts` cannot (item 642).
        pub(crate) standing:
            std::collections::BTreeMap<i64, std::collections::BTreeMap<(i64, i64, String), String>>,
        /// Every army's member list on each block within two of a `near`
        /// block, both sides, keyed on `(who, army)` — what the pool slot
        /// holds, never its index (item 800): `(ours, theirs)` as unit
        /// numbers in list order, an empty side for an army the other
        /// side lacks. [`widen_east_indies`] alone fills it.
        pub(crate) army_lists: ArmyLists,
    }

    /// [`Widened::army_lists`]: block → `(who, army)` → `(ours, theirs)`.
    pub(crate) type ArmyLists = std::collections::BTreeMap<
        i64,
        std::collections::BTreeMap<(i64, i64), (Vec<i64>, Vec<i64>)>,
    >;

    /// **run143 — East Indies' word 10398, widened whole, both directions**
    /// (item 588). run99's line with `LEADERS` raised to 9 over
    /// [`WIDENING_EAST_INDIES_BARK`]: twenty blocks shared with run99, the
    /// word's block — run99's last — and 340 past it, which no capture had
    /// printed. A sibling of run99's and run139's walks, not an extension:
    /// every record [`widen_block`] reads on every unit and building of
    /// every player, every key of [`crate::diff::leader::rows`] for both
    /// leaders, and every figure whose animation changes on either side
    /// (`docs/COMBAT.md` §44.2.1). Each key's first parting block is kept
    /// with the value diff beside it.
    ///
    /// The word's frame, 10398, writes block **10399**.
    ///
    /// `RON_ROW_WALK=<who>/<o>,…` prints every row a unit parts on, at each
    /// block the set changes; `RON_STANDING=<lo>-<hi>` every row parting on
    /// each block of the window.
    #[test]
    fn run143_s_word_frame_is_widened_whole() {
        cases_06::run143_s_word_frame_is_widened_whole();
    }

    #[test]
    fn run73_s_window_clocks_are_the_original_s() {
        cases_06::run73_s_window_clocks_are_the_original_s();
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
        cases_07::run74_s_window_is_where_great_lakes_merchant_turns();
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
        cases_07::run75_s_window_is_the_scout_s_walk_down_the_river();
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
        cases_07::run76_s_window_is_the_ai_squad_s_march();
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
        cases_07::run79_s_window_is_the_half_step_s_second_sample();
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
        cases_07::run84_says_great_lakes_6994_belongs_to_the_second_squad();
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
        cases_07::great_lakes_6994_issues_the_second_squad_s_walk_to_the_army();
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
    /// unfinished. **270,173 → 359,336 on item 478**, which added the
    /// replan flag `build_masks & 0x100` (`docs/ROADS.md` §1.2); nothing
    /// here is wrong on it, so the flag's life agrees on this map too.
    /// **359,336 → 537,662 on item 661**, which added the building's
    /// `city` slot and `city_down` link (`docs/AI.md` §63); nothing here
    /// is wrong on either, so every member chain agrees for 5,200 frames.
    /// **537,662 → 715,988 on item 763**, a building's `damage` and
    /// `damage_frac` (parked 728); nothing here is wrong on either.
    /// **715,988 → 894,314 on item 1563**, `ever_seen` and
    /// `ever_seen_completed` (parked 1450); nothing here is wrong on either.
    /// **894,314 → 986,161 on item 1497**, `myhits` and `construct_hits`.
    const RUN58_BUILD_FIELDS: usize = 986_161;
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
        cases_07::run39_s_long_trace_says_where_the_second_map_s_word_parts();
    }

    #[test]
    fn run39_s_islands_game_is_the_second_map_s_score() {
        cases_07::run39_s_islands_game_is_the_second_map_s_score();
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
        cases_07::the_original_s_own_run_is_still_matched_frame_for_frame();
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
        cases_07::run83_s_window_is_the_last_great_lakes_hole();
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
        cases_08::run85_s_window_is_the_east_indies_word_frame();
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
        cases_08::run88_s_window_is_east_indies_word_frame();
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
        cases_08::run90_s_window_is_east_indies_shuffle();
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
        cases_08::run86_s_window_is_the_transport_ride();
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
        cases_08::run82_s_window_is_the_east_indies_ride_s_run_up();
    }
}
