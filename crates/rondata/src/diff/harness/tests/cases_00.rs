pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn a_dumped_building_links_to_the_live_holder_of_its_number() {
        let mut w = sim::world::World::new(16, 16);
        w.fill_region(
            sim::world::Terrain::Land,
            sim::world::Cell::new(0, 0),
            sim::world::Cell::new(15, 15),
        );
        let mut s = sim::Sim::new(sim::tuning::Tuning::RON, w, 2);
        let ty = s.add_build_type(sim::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..sim::build::BuildType::default()
        });
        let at = |t: i32| sim::Pos::new(t * 192 + 96, t * 192 + 96);
        let dead = s.init_build(1, ty, at(5), false);
        s.close_building(dead, false);
        let live = s.init_build(1, ty, at(20), false);
        let o = i64::from(s.buildings[live].index);
        assert_eq!(
            i64::from(s.buildings[dead].index),
            o,
            "the number is reused"
        );
        assert_eq!(link_building(&s, 1, o), Some(live), "the live holder");
        s.close_building(live, false);
        assert_eq!(
            link_building(&s, 1, o),
            Some(dead),
            "none live: the first that held it"
        );
        assert_eq!(link_building(&s, 0, o), None, "another owner's number");
    }

    pub(in super::super) fn indexed_replay_preserves_reports_with_sibling_corrections() {
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

    pub(in super::super) fn observing_a_replay_preserves_the_complete_report() {
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

    pub(in super::super) fn run57_s_four_thousand_frames_stand_where_the_original_s_do() {
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

        // **The hit-point record, whole** — `ObjectData::myhits`,
        // `damage` and `damage_frac` on every linked unit-frame of the
        // capture, compared on none of them until item 484
        // (`docs/COMBAT.md` §40). East Indies' side of the row: 4,000
        // frames of the map the headline leads on, and the widest
        // single window this comparison runs in.
        //
        // **Not one of run57's 484,779 unit records carries a wound** —
        // neither player fights inside the capture — so what this pins
        // is the *maximum* and the two accumulators at rest: this
        // crate's squad-sized `myhits` is the original's on every
        // linked unit-frame, and neither side invents damage nobody
        // dealt. The live half of the row is chapter two's
        // (`chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame`,
        // 580 wounded unit-frames), and a capture of a fight on this
        // map would be the one thing that widens it further.
        let hits: usize = report.frames.iter().map(|f| f.hits_compared).sum();
        let hits_bad: Vec<HitsDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.hits_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run57 hit points: {hits} field-frames compared, {} wrong",
            hits_bad.len()
        );
        assert!(
            hits_bad.is_empty(),
            "the hit-point record agrees on every comparable field-frame \
             of {hits}: {hits_bad:?}"
        );
        assert!(
            hits >= 205_302,
            "three fields on every linked unit-frame, and the count only \
             grows: {hits}"
        );

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
        // **197,932 → 263,095 on item 478** — the replan flag, one more
        // field on every linked building-frame (`docs/ROADS.md` §1.2).
        // **263,095 → 393,421 on item 661** — the `city` slot and the
        // `city_down` link, two more, none wrong (`docs/AI.md` §63).
        // **393,421 → 523,747 on item 763** — `damage` and `damage_frac`,
        // two more, none wrong (parked 728). **523,747 → 654,073 on item
        // 1563** — `ever_seen` and `ever_seen_completed`, none wrong
        // (parked 1450). **654,073 → 721,679 on item 1497** — `myhits`
        // and an unfinished site's `construct_hits`, none wrong.
        assert_eq!(
            builds, 721_679,
            "the site and the clock on every linked building-frame"
        );
        assert!(
            coll >= 350_928,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
    }

    pub(in super::super) fn run58_s_five_thousand_frames_stand_where_the_original_s_do() {
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
        // 1. ~~**`who 0`, the human's city — thirteen fields, every
        //    frame.**~~ **Closed by item 1605** (`docs/AI.md` §169):
        //    `Leaders::strategy_all@006ed430`'s gate is `leader_flags & 3
        //    == 3`, with no test for a human, and the human's sweep runs on
        //    its phase frame but for step 16's army (`plan_strategy:1642`).
        //    The human's `peasant_dist` moves on frame 401, leader 0's
        //    phase, in both.
        // 2. ~~**The AI's second city, `1/2007`** — `land`, `filled` and the
        //    three `space` counts, one apart, from the frame its circle is
        //    first swept.~~ **Closed by item 2106** (`docs/AI.md` §209): the
        //    157 frames from 1819 were the city's founding to its owner's
        //    first sweep, and `City::init@00737050` writes `land` 9 and
        //    `filled` 1 there (`*(this + 0x61) = 0x01090000`).
        // 3. ~~**A gatherer filed under the wrong city** from 2576 — the
        //    totals agree, the attribution does not.~~ **Closed by item
        //    752**: `plan_strategy` counts a gatherer in its building's
        //    city, not the nearest (`docs/AI.md` §73), and the three rows
        //    went. Until item 592 a free
        //    citizen parted beside it from 4176 for 200 frames: a citizen
        //    riding a transport barge, which the original's census counts
        //    through its container and never free (`docs/AI.md` §58).
        // Item 1477: `filled` x3383 → x157 and the three `space` rows (x3226)
        // went (the census walks the circle from entry 1, `docs/AI.md` §122);
        // item 2106 took the last two, `land` and `filled` x157.
        let city_want: Vec<String> = Vec::new();
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
        // …and `mylos` has no disagreement up to the word. It had exactly
        // one, the AI scout's Science level on 202, which this crate's pure
        // function reported on the frame the level landed and the
        // original's cached field one level later (item 35); **item 1354**
        // made `mylos` the cache the original keeps (`docs/VISION.md` §2).
        // Over 94,338 unit-frames the AI's two Fishermen — the only units
        // on this capture that are ever packed — agree with it.
        let los_early: Vec<LosDivergence> = los_bad
            .iter()
            .copied()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert_eq!(
            los_early,
            Vec::<LosDivergence>::new(),
            "mylos is the original's up to the word"
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

    pub(in super::super) fn run69_s_three_thousand_frames_stand_where_the_original_s_do() {
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

    pub(in super::super) fn run71_s_five_thousand_frames_reach_past_the_word() {
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

    pub(in super::super) fn run12_s_end_of_frame_words_are_read() {
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

    pub(in super::super) fn frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps() {
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

    pub(in super::super) fn run14_s_frames_match_the_trace_draw_for_draw() {
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
}
