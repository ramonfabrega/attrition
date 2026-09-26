//! The `WORLD` record: terrain, roads, fog, farms and gaia's own walks.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;
    use crate::gamelog::Block;

    use crate::testenv::{dump, install};

    /// The map, from the dump: run9 (`gamelog-run9-world6.txt`, 2026-08-24)
    /// is run7's lobby and seed logged with `WORLD=6` under `[Start Game]`,
    /// so its start block carries every cell's `WData` and every tile's
    /// mask (`docs/ORACLE.md`, "The map is a dump too"). With the cells'
    /// `val` bytes real, the AI's fourth farm (site 2006) lands on the
    /// original's tile and its builder `1/1` — the first divergence on
    /// every earlier dump — tracks the whole run.
    #[test]
    fn run9_s_world_dump_puts_the_ai_s_farm_on_the_original_s_tile() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        assert_eq!(built.sim.world.width(), 60);
        assert_eq!(built.region_map.len(), 8, "the dump's eight regions");
        assert_eq!(built.sim.world.region_count(), 8);
        assert!(
            built.notes.iter().any(|n| n.contains("57600 tile masks")),
            "{:?}",
            built.notes
        );
        // A forest tile and an ocean tile exist, so the layers are there.
        let masks = |bits: u16| {
            (0..240)
                .flat_map(|y| (0..240).map(move |x| Pos::new(x, y)))
                .filter(|&t| built.sim.world.tile_mask(t) & sim::world::tile::SURFACE == bits)
                .count()
        };
        // Counted from the dump's `tdata[scan].mask` histogram: 10,749
        // tiles with the ocean surface, ~1,800 with forest.
        assert_eq!(masks(sim::world::tile::SURFACE_OCEAN), 10749);
        assert!(masks(sim::world::tile::SURFACE_FOREST) > 1500);

        let report = run(&loaded, &log, Tuning::RON, None).unwrap();
        assert_eq!(report.frames.len(), 36);
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        assert_eq!(unlinked, 0);
        let by_unit = report.first_divergence_by_unit();
        assert!(
            !by_unit.iter().any(|&(w, o, _)| w == 1 && o == 1),
            "the farm's builder diverged in position: {by_unit:?}"
        );
        assert!(
            !report
                .order_divergence_by_unit()
                .iter()
                .any(|(w, o, _, _)| *w == 1 && *o == 1),
            "the farm's builder diverged in its orders"
        );
    }

    /// Run20's `Farms` list, and the pasture in it. Six farms, one of them
    /// `farm_type == 1` — the AI's `o 2002` — which grows nothing and
    /// carries five animals of **owner 9** that no dump prints (run20's
    /// first `FULL DUMP` has 104 `ANIMALDATA` records and not one `who 9`).
    ///
    /// The stream is the assertion. `rontrace-run20.log` places frame 0's
    /// 175 draws by site: 104 `Animal::do_idle` idle rolls for the dumped
    /// animals, then **five more and one `Animal::think_farm_animal`** —
    /// the pasture's — and **five** `Farms::inc_time` draws, not six. So
    /// the harness's frame 0 moves from 160 to 165 and its frame 2, which
    /// is the farms alone on both sides, from 6/5 to **5/5**
    /// (`docs/SYNC.md` §3.6, §4.2).
    #[test]
    fn run20_s_pasture_grows_nothing_and_its_five_animals_draw_six() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut init = log.initial().unwrap();

        // The list, as `Farms::log_data` wrote it.
        let farms = &init.farms;
        assert_eq!(farms.len(), 6, "run20's six farms");
        assert!(farms.iter().all(|f| f.valid == 1), "every slot live");
        let pastures: Vec<_> = farms.iter().filter(|f| f.farm_type == 1).collect();
        assert_eq!(pastures.len(), 1, "one pasture");
        assert_eq!(
            (pastures[0].who, pastures[0].o),
            (1, 2002),
            "the AI's first"
        );
        assert_eq!(
            farms.iter().map(|f| f.farm_type).collect::<Vec<_>>(),
            vec![1, 0, 0, 0, 0, 4],
            "the crops, in the list's order"
        );

        // The five animals, and the farm they hang off. run20's own trace
        // reached the setup, so the five are borrowed from it rather than
        // stood on the farm's centre — a **second** capture through
        // `Trace::add_animals`, and a different pasture from run39's
        // (`docs/SYNC.md` §3.11).
        if let Some(tr) = trace("rontrace-run20.log") {
            borrow_pasture(&mut init, &tr);
            assert_eq!(init.pasture.len(), 1, "run20's one pasture, from its trace");
            assert!(
                init.pasture[0].iter().all(|a| a.chicken)
                    || init.pasture[0].iter().all(|a| !a.chicken),
                "a pasture is one species — five even coins or five odd, never a mix"
            );
            assert!(
                init.pasture[0]
                    .iter()
                    .all(|a| (-0xc0..0xc0).contains(&a.dx) && (-0xc0..0xc0).contains(&a.dy)),
                "and each offset is `% 0x180 - 0xc0`, so inside a tile either way"
            );
        }
        let built = build_sim(&loaded, &init, Tuning::RON);
        let animals: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.units[u].owner == 9)
            .collect();
        assert_eq!(animals.len(), 5, "five animals of owner 9");
        let farm = built.sim.units[animals[0]].farm_animal.unwrap().build;
        assert_eq!(built.sim.buildings[farm].owner, 1);
        assert_eq!(built.sim.buildings[farm].index, 2002);
        assert!(
            animals
                .iter()
                .all(|&u| built.sim.units[u].farm_animal.unwrap().build == farm),
            "all five on the same farm"
        );
        assert_eq!(
            animals
                .iter()
                .map(|&u| built.sim.units[u].farm_animal.unwrap().slot)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4],
            "slotted 0..5, which is what phases their think tick"
        );
        assert!(
            init.units.iter().all(|u| u.who != 9),
            "and no dump prints them"
        );

        // The counts, on the original's own stream.
        let report = run_traced(&loaded, &log, Tuning::RON, Some(4), None, &[], None).unwrap();
        let count = |f: i64| -> (Option<u32>, Option<u32>) {
            let &(_, ours, theirs) = report
                .rng_frames
                .iter()
                .find(|(n, _, _)| *n == f)
                .unwrap_or_else(|| panic!("frame {f} was not traced"));
            (ours, theirs)
        };
        assert_eq!(
            count(0),
            (Some(175), Some(175)),
            "frame 0, with `Unit::think_scout`'s ten in (`docs/SCOUT.md` §10)"
        );
        assert_eq!(
            count(2),
            (Some(5), Some(5)),
            "frame 2 is the five crop farms and nothing else, on both sides"
        );

        // And the islands' opening carries **both angles exactly**, on
        // every unit-frame — the smallest of the angle checks and the only
        // one at zero (`docs/MOVEMENT.md`, "Two angles"). It is the one
        // that would fail first if `Unit::init`'s 0x55555555 or the two
        // fields' assignment were put back the way they were.
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        let bad: Vec<AngleDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter().copied())
            .collect();
        assert_eq!(angles, 72);
        assert_eq!(bad, vec![], "every heading and every facing");
    }

    /// **Run20's frame 1** — the AI's fourth farm, and the walk that stopped
    /// asking for a detour (items 25 and 28; `docs/SYNC.md` §3.8, §6).
    ///
    /// Frame 1 is not draw-for-draw yet, so this is a **ratchet on the two
    /// sites the session moved** rather than a whole-frame `assert_eq!`:
    ///
    /// - `Farms::add`'s **ambience pair**, one `+0x23f` and one `+0x25b`,
    ///   and **no** `+0x128` — the AI's new farm lands in the city that
    ///   already holds the map's pasture, so `others != crops` decides the
    ///   type with no coin, and the city has two crops and no emitter yet.
    ///   The sim spent neither of these until `Sim::farms_add` existed.
    /// - `Unit::do_move+0xe84` **once**, where it used to be twice and the
    ///   original spends it not at all. The one that went was the
    ///   woodcutter's, closed by `find_path`'s pull-back (`docs/ORDERS.md`
    ///   §4.6): its goal is a forest tile, and the original walks the goal
    ///   back out of the forest before marching. **The one that remains is
    ///   the AI scout's**, whose straight line clips a *building* several
    ///   tiles short of its waypoint — `go_around_building@005fc350`, still
    ///   a seam. When that lands this row is `0` and the assertion below
    ///   must be edited to say so.
    ///
    /// - **`Leader::produce_building` draw for draw** since 2026-08-26:
    ///   `+0xc99` ×39 and `+0x1805` ×4, the original's own split, and the
    ///   farm the call places lands on the original's own tile. The
    ///   frame's only residue is now the `do_move` above, so the total
    ///   reads 54 against 53 — it no longer cancels, which is the point.
    ///
    /// The old note here said the frame read 53/53 and was *still wrong*,
    /// because one missing `produce_building` draw cancelled the stray
    /// `do_move`. Three defects were behind that one number
    /// (`docs/AI.md` §2.20): the jitter's two loops are inclusive, so
    /// `ex == ey == 1` is a 2×2 and four draws rather than one; the
    /// stride-by-three test read the loop's start index instead of the
    /// current one; and `WorldData::buildings_allowed` was not modelled at
    /// all, so a forest cell scored as a candidate and drew.
    #[test]
    fn run20_s_frame_1_spends_the_farm_s_ambience_pair() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run20-islands-dumpall.txt"),
            trace("rontrace-run20.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // Frame 0 through the harness, so frame 1 starts on the original's
        // own word and its clocks; then frame 1 raw, so the marks survive.
        built.tick();
        built.sim.tick();
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        let theirs = trace.labels(1);
        assert_eq!(theirs.len(), 53, "the original's frame 1");

        let count = |v: &[String], site: &str| v.iter().filter(|l| *l == site).count();
        for (site, n) in [
            (sim::farms::SITE_AMBIENCE_X, 1),
            (sim::farms::SITE_AMBIENCE_Y, 1),
            (sim::farms::SITE_TYPE_COIN, 0),
        ] {
            assert_eq!(count(&theirs, site), n, "the original's {site}");
            assert_eq!(count(&ours, site), n, "ours: {site}");
        }
        // And they are adjacent, x before y, which a count cannot say.
        let at = ours
            .iter()
            .position(|l| l == sim::farms::SITE_AMBIENCE_X)
            .expect("the x offset");
        assert_eq!(
            ours.get(at + 1).map(String::as_str),
            Some(sim::farms::SITE_AMBIENCE_Y)
        );
        assert_eq!(
            theirs.iter().position(|l| l == sim::farms::SITE_AMBIENCE_X),
            Some(43),
            "the original spends them right after `produce_building`'s 43"
        );

        // The grid draw: neither side's now. The AI scout's line clips a
        // building three tiles short of a clear goal, and since
        // `go_around_building` landed (item 29) the sim finds the same
        // detour the original does and spends nothing on it.
        assert_eq!(count(&theirs, sim::orders::SITE_MOVE_GRID), 0);
        assert_eq!(count(&ours, sim::orders::SITE_MOVE_GRID), 0);
        // And with that, the whole frame is one sequence.
        if let Some((_, shown)) = first_parting(&ours, &theirs) {
            panic!("run20 frame 1: {shown}");
        }
        assert_eq!(ours, theirs, "run20: frame 1, draw for draw");

        // **The detour itself, against the original's own stack.** The unit
        // whose line clips a building on this frame is `1/1` — not `1/0`,
        // which is what `docs/SYNC.md` §6 named while the count was the
        // only measurement; the AI's farm moved and with it which unit
        // pays. Its stack at the end of frame 1 is the goal plus exactly
        // one `go_around_building` waypoint, and the log's frame-2 record
        // carries the same two entries, tolerance and flags included. The
        // skew — `off % 0xc0 / 2` off the tile's low corner rather than the
        // centre — is in that number, so a detour placed at the centre
        // fails here.
        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 1)
            .expect("the AI's unit 1");
        let theirs_path = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 2)
            .expect("frame 2")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 1)
            .expect("1/1 at frame 2")
            .path;
        let ours_path: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs_path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs_path,
            vec![(41640, 39384, 0, 1), (40644, 39036, 0, 0)],
            "the original's 1/1: the goal and one detour waypoint"
        );
        assert_eq!(ours_path, theirs_path, "1/1's stack, entry for entry");

        // `produce_building`, site for site: the spiral's friendless
        // FARM/MINE candidates and the 2×2 jitter's unblocked
        // sub-positions, both the original's counts.
        for site in [sim::ai_place::SITE_SPIRAL, sim::ai_place::SITE_JITTER] {
            assert_eq!(count(&ours, site), count(&theirs, site), "{site}");
        }
        assert_eq!(count(&theirs, sim::ai_place::SITE_SPIRAL), 39);
        assert_eq!(count(&theirs, sim::ai_place::SITE_JITTER), 4);
        // And they are one call, in the original's order: every spiral
        // draw before every jitter draw.
        let last_spiral = ours
            .iter()
            .rposition(|l| *l == sim::ai_place::SITE_SPIRAL)
            .expect("a spiral draw");
        let first_jitter = ours
            .iter()
            .position(|l| *l == sim::ai_place::SITE_JITTER)
            .expect("a jitter draw");
        assert!(
            last_spiral < first_jitter,
            "the spiral scores, then jitters"
        );

        // The site itself, which is what the draws are for: the AI's new
        // farm is `who 1, o 2006` at (41856, 39552) in the run's own
        // `BUILDDATA`, and the sim now puts it there. Before the jitter
        // was a 2×2 it landed one sub-position away.
        let farm = built
            .sim
            .buildings
            .iter()
            .find(|b| b.alive && b.owner == 1 && b.index == 2006)
            .expect("the AI's new farm");
        assert_eq!(
            (farm.pos.x, farm.pos.y),
            (41856, 39552),
            "the original's own tile"
        );
    }

    /// **The fuzzed map's frame 1** — the second capture the 2×2 jitter is
    /// checked on, and the one that makes it a rule rather than a run20
    /// coincidence.
    ///
    /// `gamelog-fuzz-424242-*` is the fuzzer's control run on a lobby
    /// nobody tuned against (`docs/SYNC.md` §4.2). Its frame 1 spends
    /// `Leader::produce_building+0x1805` **three** times where run20
    /// spends four: the jitter walks the same 2×2 and `blocked_site`
    /// refuses one of the sub-positions. A one-draw-per-call reading
    /// cannot produce either number.
    ///
    /// The row above it was one draw *short* of the original's thirty for
    /// five days, and it is **30 against 30** since 2026-09-01: the
    /// stride-by-three stepped the spiral's index at the *top* of the
    /// iteration, by the stride as it stood before the body ran, where
    /// the original steps it at the bottom by the stride the body has
    /// just set (`docs/AI.md` §20). One cell, one candidate, one draw.
    /// A `Unit::do_non_flat_gather+0x54b` is still short, so the frame
    /// reads 44 against 45; that residue is asserted as it stands so that
    /// closing it shows up as a failure.
    #[test]
    fn the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-fuzz-424242-heights.txt"),
            trace("rontrace-fuzz-424242.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        built.tick();
        built.sim.tick();
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        let theirs = trace.labels(1);
        assert_eq!(theirs.len(), 45, "the original's frame 1");

        let count = |v: &[String], site: &str| v.iter().filter(|l| *l == site).count();
        assert_eq!(count(&theirs, sim::ai_place::SITE_JITTER), 3);
        assert_eq!(
            count(&ours, sim::ai_place::SITE_JITTER),
            3,
            "one of the 2×2's four sub-positions is blocked on this map"
        );
        // The residues, as they stand.
        assert_eq!(
            (
                count(&theirs, sim::ai_place::SITE_SPIRAL),
                count(&ours, sim::ai_place::SITE_SPIRAL),
            ),
            (30, 30),
            "the spiral walks the original's own candidates"
        );
        assert_eq!(
            (
                count(&theirs, sim::orders::SITE_TILE_WAIT),
                count(&ours, sim::orders::SITE_TILE_WAIT),
            ),
            (5, 4),
            "and one citizen picks no tile — still open"
        );
        assert_eq!(ours.len(), 44, "so the frame is 44 against 45");
    }

    /// Run20's AI scout at frame 0 — `Unit::think_scout`'s ten draws, on
    /// the original's own stream (`docs/SCOUT.md` §10).
    ///
    /// **The sequence is the assertion, not the count.** The trace is read
    /// here rather than in Python (`crate::trace`), filtered to the draws
    /// `think_scout` took itself, and compared site for site against the
    /// harness's own marks. A total cannot tell four rotations and two
    /// phases from three and three; this can, and it is what the ring
    /// walk's two guards need checking against.
    ///
    /// The seed is installed rather than reached, and that is deliberate:
    /// this check has to hold **while the stream that reaches the mechanic
    /// is wrong**, which is the position it landed in. It was worth
    /// keeping — the frame's own stream reached the scout two draws early
    /// until the stand/wrap swap closed, and `think_scout`'s count depends
    /// on the stream, so on the fuzzed map the same ten came out as
    /// eleven. Since 2026-08-26 the sim reaches this word on its own too
    /// (`frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`);
    /// installing it keeps the mechanic checkable the next time something
    /// upstream moves.
    #[test]
    fn run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // Two scouts stand up on this map, one a side — the two units the
        // trace shows standing two figures each before either thinks. Only
        // the computer's reaches `think_scout`.
        let scouts: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.unit_is_scout(u))
            .collect();
        assert_eq!(scouts.len(), 2, "one scout a side");
        let thinking: Vec<usize> = scouts
            .iter()
            .copied()
            .filter(|&u| built.sim.scout_thinks(u))
            .collect();
        assert_eq!(
            thinking.len(),
            1,
            "the human's does not — its `unit_masks & 0x40000` is clear"
        );
        let scout = thinking[0];
        assert_eq!(built.sim.units[scout].owner, 1, "the computer's");

        // The trace's own frame-0 draws, filtered to the ones
        // `Unit::think_scout` took itself.
        let Some(trace) = trace("rontrace-run20.log") else {
            eprintln!("skipping the sequence half: no rontrace-run20.log");
            return;
        };
        let draws = trace.run_in(0, sim::scout::CODE.start, sim::scout::CODE.end);
        // Named through `trace::SITES`, which is the same string
        // `sim::scout`'s own marks write — so the two sides of the
        // comparison below share one vocabulary.
        let theirs: Vec<String> = draws.iter().map(|d| trace.label(d)).collect();
        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        assert_eq!(
            theirs,
            vec![
                SITE_ROTATION,
                SITE_ROTATION,
                SITE_ROTATION,
                SITE_PHASE,
                SITE_CELL,
                SITE_CELL,
                SITE_CELL,
                SITE_CELL,
                SITE_ROTATION,
                SITE_PHASE,
            ],
            "the trace's own frame-0 sequence (docs/SCOUT.md §10)"
        );

        // Ours, from the first of those, site for site rather than by
        // count — four rotations and two phases, not three and three.
        let first = draws[0].seed;
        assert_eq!(first, 0x9c59_1b2b, "the trace's draw 24");
        built.sim.trace_phases = true;
        built.sim.phase_marks.clear();
        built.sim.rng = sim::combat::Rng::new(first);
        assert!(built.sim.think_scout(scout), "a target is found");
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        assert_eq!(
            ours, theirs,
            "our ten draws, at the original's sites, in the original's order"
        );
        // Which leaves the stream where the trace's next draw found it.
        assert_eq!(
            draws_between(first, built.sim.rng.seed),
            Some(theirs.len() as u32)
        );

        // §9: the explore order, at a tile centre inside ring 5 of the
        // AI's city.
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        let ci = built
            .sim
            .cities
            .iter()
            .position(|c| c.alive && c.owner == 1)
            .expect("the AI's city");
        let city = built.sim.cities[ci].pos.cell();
        let d = sim::world::vector_dist(m.dest.cell().x - city.x, m.dest.cell().y - city.y);
        assert_eq!(d, 5, "ring 5, where the four cells were");
    }

    /// **Every footprint's blocked bits, against the original's own tile
    /// masks** (item 44). A `WORLD ≥ 6` or `DUMP_ALL` start dump prints all
    /// 57,600 `tdata[scan].mask` words, so the map the harness loads *is*
    /// the oracle for the map the harness then stamps its buildings onto:
    /// `build_sim` re-marks every one of them through `Wall::mask_me`, and
    /// if the template is wrong the two disagree on the spot.
    ///
    /// They used to. This crate blocked every non-flat footprint whole,
    /// which is right for a Mine (`2x2 solid`) and wrong for the
    /// Woodcutter's Camp beside it (`2x2 gather`, which blocks nothing) and
    /// wrong for every `extra space` mask, where a 7×7 city blocks 6×6. The
    /// cost was one citizen sent 48 units past the camp's own tile, because
    /// `find_nearby_spot` refuses a `0x4000` tile and the original's camp
    /// does not carry one (`docs/ORDERS.md` §10, `docs/DATALAYER.md`).
    ///
    /// Only the object field and the blocked bit are compared: the rest of
    /// the word is terrain, fog and city radii that the setup does not
    /// re-derive.
    #[test]
    fn every_footprint_takes_the_blocked_bits_the_original_s_map_shows() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut ran = 0;
        for name in [
            "gamelog-run10-world6-long.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run9-world6.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let tiles = crate::gamelog::world_tiles(&init.world);
            let built = build_sim(&loaded, &init, Tuning::RON);
            let tw = (built.sim.world.width() * sim::world::TILES_PER_CELL) as usize;
            if tiles.len() != tw * tw {
                continue;
            }
            ran += 1;
            let keep = sim::world::tile::OBJECT | sim::world::tile::BLOCKED;
            let bad: Vec<(usize, u16, u16)> = tiles
                .iter()
                .enumerate()
                .filter_map(|(i, &theirs)| {
                    let t = Pos::new((i % tw) as i32, (i / tw) as i32);
                    let ours = built.sim.world.tile_mask(t);
                    (ours & keep != theirs & keep).then_some((i, ours, theirs))
                })
                .collect();
            assert!(
                bad.is_empty(),
                "{name}: {} tiles differ on the object/blocked bits, first {:?} \
                 (tile {:?})",
                bad.len(),
                bad.first(),
                bad.first().map(|&(i, _, _)| ((i % tw), (i / tw))),
            );
        }
        assert!(ran > 0, "no start dump with a tile map on this machine");
    }

    /// The long run with the map: run10 (`gamelog-run10-world6-long.txt`,
    /// 1,772 frames, run7's lobby and length, no input, `WORLD=6` at start),
    /// on the original's own sync stream (run11's trace) with the terrain
    /// heights (run3). The three citizens of frame 1 train on the original's
    /// frames (100, 206, 320) and every unit tracks as before; what no
    /// longer matches is the **script's branch**: on the sim's own stream
    /// its eight `rand_int(1, 10)` at frame 1 happened to pick the boom
    /// order (and `1/9` trained at 1297, the original's frame); on the
    /// original's stream, displaced by the ~130 per-frame draws the sim does
    /// not model (units, herds, farms, ammo — `docs/AI.md` §12.1), they pick
    /// the rush order, two farms go down at step 3 and the city is never
    /// placed. So `1/9` (1297) and `1/10` (1505) are the ceiling, 744
    /// unit-frames, and the per-frame draws are what move it. The earlier
    /// **Every traced frame, draw for draw** — the whole-run form of
    /// `frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`, and
    /// the sub-score the untraced stretch is worked against.
    ///
    /// Run14 is run10's own lobby and seed with the draw-site instrument
    /// attached, and it carries **284 frames** rather than the nine a
    /// `DUMP_ALL` window can afford. The dump's words are installed at the
    /// ends of frames 0–3 and 94–103 as always; everywhere else the
    /// simulation's stream is its own, and this compares the *sites*
    /// rather than the values — so a frame can be checked draw for draw on
    /// a stretch where nothing else can reach it at all.
    ///
    /// The number is a floor and it is what says whether a stream item
    /// converged. History:
    ///
    ///   2026-08-28  **179** of 284 (the first pin)
    ///   2026-08-28  **184** of 284 (the bird, `docs/SYNC.md` §3.9): the
    ///               nine frames of `Animal::think_bird`'s three draws
    ///               between 104 and 168, and the sampling's own count.
    ///   2026-08-28  **192** of 284 (the bird's wing beat, §3.9): its two
    ///               animation lengths, read out of the install's own
    ///               `.bha` files (`crate::artdata`), and with them the
    ///               hatch frame's wrap, the birth coin and the wraps at
    ///               127 and 142.
    ///   2026-08-28  **198** of 284 (the residue's names, item 53): five
    ///               draws this simulation was already taking under a
    ///               coarse mark — the script VM's `rand_int`, the herd
    ///               animal's wander coin and its three step draws, the
    ///               farmer's cell re-pick and the arrival stand — given
    ///               a [`crate::trace::SITES`] row each. No mechanic
    ///               changed; the marks got finer, which is the incentive
    ///               below.
    ///
    ///   2026-08-28  **173** of 284, and the *fall* is the item (item 55,
    ///               the road). Two things landed together: the height
    ///               grid stopped being borrowed from another game
    ///               ([`borrow_from_siblings`]), and with the search
    ///               therefore exact, `Sim::plan_roads` came on. Frames 10
    ///               and 11 now match — 226 and 254 draws against 6 and 6
    ///               — and **the first frame whose draws differ at all
    ///               moved from 10 to 18**, which is the number below that
    ///               says so. What fell is the tail: past the first
    ///               divergence both sides are running on words that have
    ///               parted, and which of two wrong streams happens to
    ///               label a frame the same way is luck. 198 was that luck
    ///               with the divergence at 10; 173 is it with the
    ///               divergence at 18.
    ///
    /// A frame is counted only when the two label sequences are equal, so
    /// a coarse mark on our side (`unit 1/9` against a `GameAccess::rnd`
    /// the table does not name) fails it even where the counts agree.
    /// Making a mark finer therefore *raises* this number, which is the
    /// intended incentive: `docs/SYNC.md` §5.1's "mark the phase before
    /// believing the total".
    ///
    /// **The sharper number is the first frame that differs**, and it is
    /// asserted too: a matched *count* rewards luck past the divergence,
    /// where this cannot. It has gone 4 → 10 → **18**.
    /// **The rings lay nothing new.** `BuildType::place_roads` runs on
    /// run14's frames 0 and 10 to 15 and again from 167, and the original's
    /// own map says what it does to the world: the tile masks of run13's
    /// `DUMP_ALL` at sim-frame 95 are **identical, all 57,600 of them**, to
    /// run10's start-of-game `WORLD=6` block. Ninety-five frames of road
    /// regeneration changed not one tile, because every tile a ring or a
    /// road reaches is already a road or is blocked.
    ///
    /// That is the whole oracle for `crate::roads`' §3, and it is sharp:
    /// widening the city's ring by the one column the *third* `place_roads`
    /// arm uses lays twenty-six new roads here and fails this at once (tried,
    /// 2026-08-28). The surface field is what a road changes, so that is
    /// what this compares; `PLACED` and the footprint bits move for reasons
    /// of their own.
    #[test]
    fn run14_s_road_rings_change_no_tile_the_original_does_not() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        // Capture inputs are needed only while constructing the owned simulation.
        let mut built = with_sibling_initials(|refs| {
            crate::capture::indexed::IndexedCapture::open(&path)
                .unwrap()
                .with_replay_initial(|init| {
                    let mut init = init;
                    borrow_from_siblings(&mut init, refs);
                    build_sim(&loaded, &init, Tuning::RON)
                })
                .unwrap()
        });
        let (tw, th) = (
            built.sim.world.width() * sim::world::TILES_PER_CELL,
            built.sim.world.height() * sim::world::TILES_PER_CELL,
        );
        let surfaces = |s: &sim::Sim| -> Vec<u16> {
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .map(|(x, y)| s.world.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE)
                .collect()
        };
        let before = surfaces(&built.sim);
        assert_eq!(before.len(), 57_600, "run14's map is 240 tiles square");
        for _ in 0..285 {
            built.tick();
        }
        let after = surfaces(&built.sim);
        let moved: Vec<(i32, i32, u16, u16)> = (0..before.len())
            .filter(|&i| before[i] != after[i])
            .map(|i| {
                let (x, y) = (i as i32 % tw, i as i32 / tw);
                (x, y, before[i], after[i])
            })
            .collect();
        assert!(
            moved.is_empty(),
            "the rings laid {} tiles the original leaves alone: {:?}",
            moved.len(),
            &moved[..moved.len().min(8)]
        );
    }

    /// **The road two fresh sites lay, tile for tile** — item 55's oracle,
    /// and the first time this mechanic is checked against a *path* rather
    /// than against three totals.
    ///
    /// run32 (`docs/ORACLE.md`) is run10–14's game with two enhancers
    /// dropped from the cheat channel at sim-frame 100 on ground the map
    /// has never had a road on: a **Granary** at tile `(6, 171)` and a
    /// **Smelter** at `(33, 161)`. `add` without `NEW` runs
    /// `Build::activate`, and `Wall::start` → `Wall::mask_me(1,
    /// REGEN_FORCE)` → `BuildType::place_roads` plans the road **there and
    /// then** — which is why the roads are on the map four frames before
    /// the scheduled replans of `docs/ROADS.md` §1 come round.
    ///
    /// The world before is **run13's** `FRAME 100` block: the same game,
    /// the same map, ninety-nine frames in, and nothing between them lays a
    /// road (the trace has no `calc_road_cost` draw before frame 100). The
    /// world after is run32's `FRAME 104`. The heights are run13's own —
    /// the *pre*-terraform grid — because `TerrainOut::terraform_for_build\
    /// ing` runs **after** the road is planned: under the frame-104 grid,
    /// which has both footprints flattened, the first search costs 967
    /// nodes against the original's 1,043, and under the pre-terraform one
    /// it costs 1,046.
    ///
    /// The jitters are not left to chance: `calc_road_cost` takes one
    /// `Random::get(0, 0xffff) % 20` a node and the trace records the
    /// generator's word **before** every draw, so each search starts on the
    /// original's own state. The boundary between the two searches is in
    /// the trace too, and it is not a road draw: `Wall::activate` plays a
    /// sound off a *different* generator, so the one non-sync draw inside
    /// frame 100 splits the 2,913 road draws into the Granary's 1,043 and
    /// the Smelter's 1,870.
    ///
    /// **What this pins, and what it leaves open.** The tiles are exact —
    /// 62 of them, both roads and both rings. The counts are not: 1,046
    /// against 1,043 and 1,460 against 1,870, with the right road either
    /// way. Every search this simulation has been shown a *frame's own*
    /// world for — run14's frames 10 and 11, run32's 104 to 107 — matches
    /// node for node, so what is left is particular to the frame a
    /// building is placed on (`docs/ROADS.md` §7).
    #[test]
    fn run32_s_two_fresh_roads_are_the_original_s_tile_for_tile() {
        let Some(inst) = install() else { return };
        let (Some(r32), Some(r13), Some(tr)) = (
            dump("gamelog-run32-roadpath.txt"),
            dump("gamelog-run13-window-95-105.txt"),
            trace("rontrace-run32.log"),
        ) else {
            eprintln!("skipping: no run32 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let (t32, t13) = (crate::capture::read(&r32), crate::capture::read(&r13));
        let (l32, l13) = (Log::parse(&t32), Log::parse(&t13));

        let world_at = |log: &Log, n: i64, heights: &[i64]| -> World {
            let (_, body) = log
                .dumps()
                .into_iter()
                .find(|(f, _)| *f == n)
                .unwrap_or_else(|| panic!("frame {n} is dumped"));
            let w = body.kid("WORLD").expect("a WORLD block");
            let mut notes = Vec::new();
            world_from(&w.fields().to_vec(), heights, &mut notes).0
        };
        let roads = |w: &World| -> std::collections::BTreeSet<(i32, i32)> {
            let (tw, th) = (w.width() * 4, w.height() * 4);
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    w.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE
                        == sim::world::tile::SURFACE_ROAD
                })
                .collect()
        };

        let heights = l13.frame_heights(100);
        assert_eq!(heights.len(), 58_081, "run13's frame-100 height grid");
        let before = world_at(&l13, 100, &heights);
        let after = world_at(&l32, 104, &heights);
        let was = roads(&before);
        let theirs: std::collections::BTreeSet<(i32, i32)> =
            roads(&after).difference(&was).copied().collect();
        assert_eq!(theirs.len(), 62, "the tiles the original laid at frame 100");

        // The two searches, and the boundary between them: the one draw of
        // frame 100 that is *not* on `game_random` is `Wall::activate`'s
        // sound, and it is emitted after the first building's plan.
        const COST: u32 = 0x0068_6346;
        let draws = tr.run_in(100, COST, COST + 1);
        assert_eq!(draws.len(), 2913, "frame 100's road draws");
        let sound = tr
            .draws
            .iter()
            .position(|d| d.frame == 100 && !d.sync())
            .expect("Wall::activate's sound draw");
        let first = tr.draws[..sound]
            .iter()
            .filter(|d| d.frame == 100 && d.sync() && d.site == COST)
            .count();
        assert_eq!(
            (first, draws.len() - first),
            (1043, 1870),
            "the Granary's nodes and the Smelter's"
        );

        // Stand run13's frame-100 buildings up on run13's frame-100 map,
        // then place the two the channel placed, in the order it placed
        // them, each seeded where the original's own search began.
        let mut sim = loaded.sim(Tuning::RON, before.clone(), 2);
        for pass in [true, false] {
            for b in l13
                .frame_builds(100)
                .iter()
                .filter(|b| (0..2).contains(&b.who))
            {
                let Some(ty) = b
                    .orig_type
                    .and_then(|t| loaded.build_of_type_index(t as i32))
                else {
                    continue;
                };
                if sim::build::is_city(&loaded.build_types, ty) != pass {
                    continue;
                }
                let h = sim.init_build(b.who as sim::Player, ty, pos_of(b.pos), false);
                sim.buildings[h].index = b.o as i16;
                sim.activate(h, false, false);
            }
        }
        // Standing them up moved the map (footprints, `mask_city`,
        // `sync_territory`); the dump's map is the one the searches read.
        sim.world = before.clone();
        sim.plan_roads = true;

        // run62 is this same capture with the three road proxies on
        // (`tools/trace/README.md`): every candidate's coordinate, every
        // node's price, and the bracket delimiting the two searches. With
        // it the two counts stop being a score and become a sequence.
        let theirs_nodes = trace("rontrace-run62.log")
            .map(|t| t.road_nodes(100))
            .unwrap_or_default();
        sim.trace_costs = !theirs_nodes.is_empty();

        let mut laid: std::collections::BTreeSet<(i32, i32)> = std::collections::BTreeSet::new();
        let mut costed: Vec<u32> = Vec::new();
        for (id, tile, seed) in [
            (sim::build::Ident::Granary, Pos::new(6, 171), draws[0].seed),
            (
                sim::build::Ident::Smelter,
                Pos::new(33, 161),
                draws[first].seed,
            ),
        ] {
            let ty = loaded
                .build_types
                .iter()
                .position(|b| b.ident == id)
                .unwrap_or_else(|| panic!("a {id:?} type"));
            let b = sim.init_build(
                0,
                ty,
                Pos::new(
                    tile.x * sim::world::UNITS_PER_TILE + sim::world::UNITS_PER_TILE / 2,
                    tile.y * sim::world::UNITS_PER_TILE + sim::world::UNITS_PER_TILE / 2,
                ),
                false,
            );
            assert_eq!(
                sim.buildings[b].city,
                Some(0),
                "{id:?} joins the human's city, as `Build::find_city` puts it"
            );
            let mine = roads(&sim.world);
            sim.rng.seed = seed;
            // Not `place_roads` directly: `Build::activate` → `Wall::start`
            // → `mask_me(1, REGEN_FORCE)` → `place_roads` is the chain the
            // channel's `add` ran, and nothing else on it draws.
            sim.activate(b, false, false);
            costed.push(draws_between(seed, sim.rng.seed).unwrap_or(0));
            laid.extend(roads(&sim.world).difference(&mine).copied());
        }

        // **The whole record, node for node.** The count was all this
        // could compare until run62: `calc_road_cost` is handed a pooled
        // `PathNode *`, so its own proxy record carries no coordinate, and
        // `valid_roadcoord` -- the gate that admitted the tile -- is
        // proxied beside it for that. What the pair gives is the original's
        // own sequence of `(tile, from, dir, price)`, which is what turned
        // two numbers that were 3 and 410 out into two mechanics
        // (`docs/ROADS.md` 7.3, 7.4).
        if !theirs_nodes.is_empty() {
            let ours = &sim.road_marks;
            let at = (0..ours.len().max(theirs_nodes.len()))
                .find(|&i| ours.get(i) != theirs_nodes.get(i));
            if let Some(i) = at {
                for j in i.saturating_sub(4)..(i + 5).min(ours.len().max(theirs_nodes.len())) {
                    eprintln!(
                        "  {j:>5} ours   {:?}\n        theirs {:?}",
                        ours.get(j),
                        theirs_nodes.get(j)
                    );
                }
            }
            assert_eq!(
                (at, ours.len()),
                (None, theirs_nodes.len()),
                "run62's 2,913 priced nodes: the first index that parts, and the count"
            );
        }
        assert_eq!(
            laid, theirs,
            "the two roads and their rings are not the original's, tile for tile"
        );
        assert_eq!(
            costed,
            vec![1043, 1870],
            "the nodes costed, and they are the original's since run62 \
             (`docs/ROADS.md` §7.3, §7.4)"
        );
    }

    /// **The scheduled replans cost the original's nodes, exactly.** Four
    /// searches, on the world of the frame that ran them.
    ///
    /// run32's two placements flag their city, so `(frame + o) % 16 == 0`
    /// walks the whole of it: the Smelter on sim-frame 104, the Granary on
    /// 105, the Market on 106 and the Library on 107. Each re-runs the same
    /// search over a map that already has the road, and the trace says what
    /// each cost — **332, 231, 232 and 60**. This simulation, given the
    /// frame's own world through [`sim_at_frame`] and the original's word
    /// at the search's first draw, costs the same four, and lays no tile
    /// (there is nothing left to lay).
    ///
    /// This is what settled item 55: nothing was wrong with the search. The
    /// harness had been feeding it the *heights of another game* — run3's,
    /// which shares the seed, the style and the size but not `GAME_RULES`,
    /// and whose grid differs on 237 corners from (7, 83) to (230, 163)
    /// because its starting buildings terraformed elsewhere
    /// ([`borrow_from_siblings`]).
    #[test]
    fn run32_s_scheduled_replans_cost_the_original_s_nodes() {
        let Some(inst) = install() else { return };
        let (Some(r32), Some(tr)) = (
            dump("gamelog-run32-roadpath.txt"),
            trace("rontrace-run32.log"),
        ) else {
            eprintln!("skipping: no run32 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&r32);
        let log = Log::parse(&text);
        const COST: u32 = 0x0068_6346;
        let roads = |w: &World| -> std::collections::BTreeSet<(i32, i32)> {
            let (tw, th) = (w.width() * 4, w.height() * 4);
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    w.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE
                        == sim::world::tile::SURFACE_ROAD
                })
                .collect()
        };
        let mut seen = Vec::new();
        for (frame, o) in [(104i64, 2008i64), (105, 2007), (106, 2006), (107, 2005)] {
            let draws = tr.run_in(frame, COST, COST + 1);
            let mut at = sim_at_frame(&loaded, &log, frame, Tuning::RON)
                .expect("run32's frame block stands up");
            let b = at
                .build(0, o)
                .unwrap_or_else(|| panic!("frame {frame}: no building 0/{o}"));
            let mine = roads(&at.sim.world);
            at.sim.plan_roads = true;
            at.sim.rng.seed = draws[0].seed;
            at.sim.place_roads(b);
            let laid: Vec<(i32, i32)> = roads(&at.sim.world).difference(&mine).copied().collect();
            assert!(
                laid.is_empty(),
                "frame {frame}: the replan laid {laid:?}, and the original laid nothing"
            );
            seen.push((
                frame,
                draws_between(draws[0].seed, at.sim.rng.seed).unwrap_or(0),
                draws.len() as u32,
            ));
        }
        assert_eq!(
            seen,
            vec![
                (104, 332, 332),
                (105, 231, 231),
                (106, 232, 232),
                (107, 60, 60)
            ],
            "(frame, ours, the original's) nodes costed"
        );
    }

    /// The simulation's `Farms` list as the dump writes it: one row per
    /// record in `Sim::farm_order`, the cells in the record's own memory
    /// order (`status[dx][dy]`, index `dx * 4 + dy`).
    fn farm_rows(sim: &Sim) -> Vec<crate::gamelog::FarmDump> {
        sim.farm_order
            .iter()
            .map(|&b| {
                let bd = &sim.buildings[b];
                crate::gamelog::FarmDump {
                    who: i64::from(bd.owner),
                    o: i64::from(bd.index),
                    valid: i64::from(bd.farm.valid),
                    farm_type: i64::from(bd.farm.farm_type),
                    status: bd.farm.state.iter().map(|&s| i64::from(s)).collect(),
                    adds: bd.farm.adds.iter().map(|&a| i64::from(a)).collect(),
                }
            })
            .collect()
    }

    /// **The farm record, whole — every cell of every farm, on every frame
    /// two `DUMP_ALL` captures of this game print one.**
    ///
    /// `Farms::log_data` writes each `FarmStruct` as flat fields of the
    /// dump: `who`, `o`, then sixteen `percent`/`status` pairs, the
    /// twenty-five corner heights, `valid` and `farm_type`. The harness
    /// read four of those fields and threw the thirty-two cells away, and
    /// the thirty-two are the whole of the farm clock — a crop cell's
    /// state and its age in `0.005f` adds. Two hundred frames of that
    /// clock decide when a farmer's cell ripens under it and it walks off
    /// to another, which is two draws on the sync stream and a `MOVE_TO`
    /// in front of its gather.
    ///
    /// Run12 dumps frames 1–3 and run13 frames 95–104 of the same game
    /// run10 records, so this is ninety-five frames of simulation checked
    /// against the original's own arithmetic, cell for cell. What it pins:
    ///
    /// - **The clock.** `Farms::inc_time`'s `0.005f` a frame and the
    ///   farmer's `Farms::grow` on top, and the 201st add crossing `1.0f`
    ///   — every growing cell's count, every frame.
    /// - **The sprout's cell.** Sim-frame 101 sprouts one cell of the AI's
    ///   `1/2003` (`docs/SYNC.md` §4.1's draw 15/16), and it is **memory
    ///   index 5** — which fixes the order `nth_empty` counts in.
    /// - **The farmer's cell**, but only up to frame 101. Every starting
    ///   farmer stands on `(2, 2)`, which is its own transpose, and the
    ///   six that re-pick on 101 do not *reach* their new cells until 109
    ///   — past the last dumped frame. So the transposed index item 61
    ///   fixed (`status[dy][dx]` for `status[dx][dy]`) is **not** caught
    ///   here; the trace is what catches it, and
    ///   `run14_s_frames_match_the_trace_draw_for_draw` pins the
    ///   re-target's own frames for that reason.
    ///
    /// Made to fail on purpose, both halves: dropping `Farm::advance`'s
    /// add parts this at frame 1, and transposing `nth_empty`'s walk
    /// parts it at frame 2 on the AI's `1/2003`.
    #[test]
    fn run12_and_run13_s_farm_records_are_the_original_s_cell_for_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r12), Some(r13)) = (
            dump("gamelog-run10-world6-long.txt"),
            dump("gamelog-run12-dumpall-seeds.txt"),
            dump("gamelog-run13-window-95-105.txt"),
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

        // Every frame either capture dumps the list on, by frame number.
        // A `FRAME n` block's `FULL DUMP` is the state after `n` ticks —
        // the same alignment `run13_s_world_at_frame_95…` stands on.
        let (t12, t13) = (crate::capture::read(&r12), crate::capture::read(&r13));
        let (l12, l13) = (Log::parse(&t12), Log::parse(&t13));
        let mut want: Vec<(i64, Vec<crate::gamelog::FarmDump>)> = Vec::new();
        for l in [&l12, &l13] {
            for (n, b) in l.frames() {
                let farms = crate::gamelog::farms_of(b.kid("FULL DUMP").unwrap_or(b));
                if !farms.is_empty() {
                    want.push((n, farms));
                }
            }
        }
        want.sort_by_key(|(n, _)| *n);
        assert!(
            want.len() >= 13 && want.first().map(|(n, _)| *n) == Some(1),
            "run12's frames 1–3 and run13's 95–104: {:?}",
            want.iter().map(|(n, _)| *n).collect::<Vec<_>>()
        );

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let last = want.last().map_or(0, |(n, _)| *n);
        let mut parted: Vec<String> = Vec::new();
        for frame in 1..=last {
            built.tick();
            for (_, theirs) in want.iter().filter(|(n, _)| *n == frame) {
                let ours = farm_rows(&built.sim);
                if ours == *theirs {
                    continue;
                }
                for (i, (a, b)) in ours.iter().zip(theirs.iter()).enumerate() {
                    if a != b {
                        parted.push(format!(
                            "frame {frame} slot {i}: ours {a:?}\n              theirs {b:?}"
                        ));
                    }
                }
                if ours.len() != theirs.len() {
                    parted.push(format!(
                        "frame {frame}: ours {} records, theirs {}",
                        ours.len(),
                        theirs.len()
                    ));
                }
            }
        }
        assert!(
            parted.is_empty(),
            "the farm records parted from the original's:\n{}",
            parted[..parted.len().min(6)].join("\n")
        );
    }

    /// **The map itself, ninety-five frames in.** The road search reads the
    /// world and nothing else — a cell's owner decides a 240-unit term, its
    /// `ROCK` flag a 120, and a tile's mask decides whether the tile is
    /// valid at all (`docs/ROADS.md` §5.2) — so "is our map the original's"
    /// is the first question any count that comes out wrong has to answer,
    /// and until this it had only ever been asked of the *start* dump the
    /// map was loaded from, which is circular.
    ///
    /// Run13's `DUMP_ALL` writes a whole `WORLD` block at sim-frame 95:
    /// 3,600 cells and 57,600 tile masks of the same game run10 records.
    /// Ninety-five frames of simulation later the two worlds agree on
    /// **every cell's owner and every tile's mask**, and on every cell's
    /// flags but one.
    ///
    /// **The one:** cell `(52, 22)` carries `cell::BUILDING` there and not
    /// here, because nothing in this simulation ever *sets* that bit — every
    /// cell that has it got it from the start dump, and a building finished
    /// after frame 0 in a cell that had none leaves it clear. It is read
    /// (`crate::army`'s muster search classes a cell by it), so it is a real
    /// gap; it is pinned here as the one known difference rather than
    /// waived, and `docs/QUEUE.md` carries it.
    ///
    /// Made to fail twice before landing: once by comparing the world
    /// *before* the ninety-five frames, which the tile masks catch, and
    /// once by moving one cell's owner, which the borders do.
    #[test]
    fn run13_s_world_at_frame_95_is_the_original_s_cell_for_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r13)) = (
            dump("gamelog-run10-world6-long.txt"),
            dump("gamelog-run13-window-95-105.txt"),
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
        for _ in 0..95 {
            built.tick();
        }
        // Run13's own frame-95 block, read the way `Initial` reads a start
        // dump: `world_from` takes any `WORLD` block's fields.
        let t13 = crate::capture::read(&r13);
        let l13 = Log::parse(&t13);
        let block = l13
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 95)
            .map(|(_, b)| b)
            .expect("run13 traced frame 95");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("run13's frame 95 carries a WORLD block");
        let mut notes = Vec::new();
        let (theirs, _) = world_from(&w.fields().to_vec(), &[], &mut notes);
        let (xs, ys) = (theirs.width(), theirs.height());
        assert_eq!((xs, ys), (60, 60), "the cell grid of run10's map");

        let cells: Vec<sim::world::Cell> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .collect();
        let owners: Vec<String> = cells
            .iter()
            .filter(|&&c| built.sim.world.owner(c) != theirs.owner(c))
            .map(|&c| {
                format!(
                    "cell ({}, {}) ours {:?} theirs {:?}",
                    c.x,
                    c.y,
                    built.sim.world.owner(c),
                    theirs.owner(c)
                )
            })
            .collect();
        assert!(
            owners.is_empty(),
            "the borders parted from the original's by frame 95: {owners:?}"
        );

        let flags: Vec<(i32, i32, u16, u16)> = cells
            .iter()
            .map(|&c| {
                (
                    c.x,
                    c.y,
                    built.sim.world.cell_data(c).flags,
                    theirs.cell_data(c).flags,
                )
            })
            .filter(|(_, _, a, b)| a != b)
            .collect();
        // **Nothing parts, on any of the 3,600 cells.** This pinned the
        // one exception `vec![(52, 22, 0x200, 0x4200)]` until item 352 —
        // the AI's second building, missing `BUILDING 0x4000` because
        // nothing in this crate wrote `BuildType::mask_me`'s first
        // statement (`docs/ARMY.md` §13, `docs/CITIES.md` §3.6). It is a
        // third capture's opinion of that bit, on run10's map rather than
        // Great Lakes', and it was the whole of what was left here.
        assert!(
            flags.is_empty(),
            "run13's cell flags part from the original's by frame 95: {flags:?}"
        );

        let (tw, th) = (
            xs * sim::world::TILES_PER_CELL,
            ys * sim::world::TILES_PER_CELL,
        );
        let masks: Vec<(i32, i32, u16, u16)> = (0..th)
            .flat_map(|y| (0..tw).map(move |x| Pos::new(x, y)))
            .map(|t| (t.x, t.y, built.sim.world.tile_mask(t), theirs.tile_mask(t)))
            .filter(|(_, _, a, b)| a != b)
            .collect();
        assert!(
            masks.is_empty(),
            "{} of 57,600 tile masks differ at frame 95: {:?}",
            masks.len(),
            &masks[..masks.len().min(8)]
        );
    }

    /// **run61 — a bird's flight, against the oracle owner 9 never had**
    /// (`docs/SYNC.md` §3.9, queue item 120).
    ///
    /// run60's game with three more call proxies open across all 5,400
    /// frames: `Unit::do_air_physics`, whose entry and return **bracket** a
    /// bird's frame and whose two arguments are the patrol point it steers
    /// at; `Unit::air_turn_speed`, whose answer is the bank angle scaled
    /// onto the type's rate; and `Unit::set_new_location`, which is where
    /// the step landed. Eleven flying units, **47,533 air frames**.
    /// `rngcmp.py` against run60: 5,401 frames, **zero differing**, so the
    /// proxies cost the stream nothing and this is run54's game.
    ///
    /// **What it settles.** Seeded only with `Unit::init`'s own birth state
    /// — the patrol point's tile centre, and `Angle::INITIAL` — and fed the
    /// original's own goal each frame, `crates/sim/src/air.rs` reproduces
    /// every one of the ten wild birds' flights **exactly, to the last
    /// frame of the capture**: up to 5,272 consecutive frames, position for
    /// position. The bank's zero-crossings agree too, frame for frame,
    /// which is what the `air_turn_speed` records are: the original calls
    /// it once on a frame its bank is turning the heading and not at all on
    /// a frame the bank is zero.
    ///
    /// So the flight was never the residue. **The birth was**, and it is
    /// one line: `Unit::init@00612100` snaps a new unit onto the centre of
    /// its 48-unit tile — `div_3_table[p >> 4] · 0x30 + 0x18`, which for a
    /// cell centre is `+24` on each axis — and `Gaia::spawn_bird` handed
    /// the cell centre straight through. Twenty-four position units at
    /// birth is what put the coin at 5404 instead of 5437.
    ///
    /// The eleventh flyer is the dock's gull, born at 3579 on a
    /// `StrafeOrder` rather than a patrol: `Unit::do_strafe` points it
    /// before the first physics frame, so its heading is due west and not
    /// `Angle::INITIAL`, and it is not what this asserts.
    #[test]
    fn run61_s_birds_fly_where_the_original_s_do() {
        let Some(tr) = trace("rontrace-run61.log") else {
            eprintln!("skipping: no run61 trace (set RON_GAMELOG_DIR)");
            return;
        };
        let air = tr.air_frames();
        let mut by_unit: std::collections::BTreeMap<u32, Vec<&crate::trace::AirFrame>> =
            std::collections::BTreeMap::new();
        for a in &air {
            by_unit.entry(a.unit).or_default().push(a);
        }
        assert!(air.len() > 47_000, "run61 folds 47,533 air frames");

        // `Unit::init`'s snap: the centre of the unit's 48-unit tile.
        let tile = sim::world::UNITS_PER_TILE / 4;
        let births = tr.air_births();
        let mut birds = 0;
        let mut frames = 0usize;
        for (unit, rows) in &by_unit {
            let goal = rows[0].goal;
            let Some(&(_, birth)) = births.get(unit) else {
                panic!("{unit:#x} flew without ever being put down")
            };
            // **The wild bird's discriminator is its own birth.** Hatched on
            // a cell, its patrol point is that cell's centre and `Unit::init`
            // puts it twenty-four units into the tile — so `birth == goal +
            // 24` on both axes is exactly "this one was born flying a
            // patrol". The gull is put down 168 short of its dock and
            // `Unit::do_strafe` points it before the first physics frame.
            if birth != (goal.0 + tile / 2, goal.1 + tile / 2) {
                assert_eq!(*unit, 0x1478_729c, "run61's one non-patrol flyer");
                continue;
            }
            birds += 1;
            let mut s = sim::Sim::new(sim::tuning::Tuning::RON, sim::world::World::new(64, 64), 2);
            let ty = s.add_unit_type(sim::UnitType {
                hits: 1,
                moves: 35,
                turn_speed: sim::movement::degrees_to_angle(5).0,
                kind: sim::attrition::UnitKind {
                    domain: sim::attrition::Domain::Air,
                    ..sim::attrition::UnitKind::default()
                },
                ..sim::UnitType::default()
            });
            // `Unit::init`'s snap, and its `0x55555555` — the whole of the
            // birth state a bird's flight is downstream of.
            let at = sim::world::Pos::new(birth.0, birth.1);
            let mut u = sim::Unit::new(sim::gaia::BIRD_OWNER, 0, at, 1);
            u.ty = Some(ty);
            u.kind = s.unit_types[ty].kind;
            u.movement.speed = 35;
            u.movement.turning = sim::turning_of(&s.unit_types[ty]);
            let b = s.add_unit(u);
            assert_eq!(s.units[b].movement.heading, sim::movement::Angle::INITIAL);
            s.gaia
                .bird_goals
                .push((b, sim::world::Pos::new(goal.0, goal.1)));

            for r in rows {
                let Some(want) = r.to else { continue };
                s.do_air_physics(b, sim::world::Pos::new(r.goal.0, r.goal.1), r.frame);
                let got = s.units[b].pos;
                assert_eq!(
                    (got.x, got.y),
                    want,
                    "{unit:#x} f{}: the step, against run61's own",
                    r.frame
                );
                // `bank_aircraft` reaches `air_turn_speed` on exactly the
                // frames its bank is not zero — and the bank is stored
                // **negated**, so a settled one is `-0.0` and a bitwise
                // test against `ZERO` calls it turning.
                let banked = s.flight(b).roll.bits() & 0x7fff_ffff != 0;
                assert_eq!(
                    !r.turn_speed.is_empty(),
                    banked,
                    "{unit:#x} f{}: the bank's zero-crossing",
                    r.frame
                );
                frames += 1;
            }
        }
        assert_eq!(birds, 10, "run61's ten wild birds");
        eprintln!("run61: {frames} air frames of {birds} birds reproduced exactly");
        assert!(frames > 45_000);
    }

    /// **run72 — Great Lakes' word, node for node** (2026-09-03, item 202).
    ///
    /// run71's game to 4,810 frames with a `DUMP_ALL` window on
    /// `[4800, 4806)` and the three `docs/ROADS.md` §7.2 proxies over
    /// `[4799, 4807]` — run64's instrument, one map over.
    ///
    /// What it is for: Great Lakes' word parts at **4803**, and the frame
    /// is a **building's** road plan. Player 1's Market `o 2015` finishes
    /// there and `place_roads` runs the cardinal search from its far
    /// corner tile (228, 83) to London's centre tile (220, 84); the
    /// original prices **277** nodes and this crate prices **266**.
    /// Everything from 4809 on is downstream of those eleven — the
    /// position parting at 4827 included, where `1/15` re-picks its farm
    /// cell off a seed that is nobody's (`docs/QUEUE.md`, item 201's
    /// re-diagnosis).
    ///
    /// A count is not a sequence (run62's lesson), so this asks the
    /// original for every node: the gate carries the candidate's world
    /// coordinate and the price carries the answer, and the difference's
    /// arithmetic names the term — a multiple of three is the climb,
    /// exactly twice is `was_seen`.
    ///
    /// And the world beside it, for run64's reason: this is the first road
    /// search on a **mid-game** grid — 4,800 frames of border growth, of
    /// roads other buildings laid, and of terraforms this crate ran
    /// itself.
    #[test]
    fn run72_s_road_nodes_are_where_great_lakes_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r72)) = (
            dump("gamelog-run71-greatlakes-5k.txt"),
            dump("gamelog-run72-greatlakes-marketroad.txt"),
        ) else {
            eprintln!("skipping: no run71/run72 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(t72) = trace("rontrace-run72.log") else {
            eprintln!("skipping: no run72 trace");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(tr) = trace("rontrace-run71.log") {
            borrow_pasture(&mut init, &tr);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Sim-frame 4803 is the 4,804th tick, so this stops at the end of
        // 4802 — the world the search is about to read, and the state
        // run72's `FRAME 4803` block prints.
        for _ in 0..4_803 {
            built.tick();
        }

        // **The world the search reads, whole** — the height grid first,
        // because a mid-game building re-terraforms it and this is the
        // first search on a grid this crate wrote rather than borrowed.
        let text72 = crate::capture::read(&r72);
        let l72 = Log::parse(&text72);
        let block = l72
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 4_803)
            .map(|(_, b)| b)
            .expect("run72 dumped frame 4803");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("a WORLD block");
        let mut notes = Vec::new();
        let heights = l72.frame_heights(4_803);
        assert!(
            heights.len() > 1_000,
            "run72's block carries the height grid"
        );
        let (theirs_world, _) = world_from(&w.fields().to_vec(), &heights, &mut notes);
        let (xs, ys) = (theirs_world.width(), theirs_world.height());
        let mut z_off = Vec::new();
        let mut mask_off = Vec::new();
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                if built.sim.world.tile_z(q) != theirs_world.tile_z(q) {
                    z_off.push(format!(
                        "t({tx},{ty}) ours {} theirs {}",
                        built.sim.world.tile_z(q),
                        theirs_world.tile_z(q)
                    ));
                }
                let (om, tm) = (built.sim.world.tile_mask(q), theirs_world.tile_mask(q));
                if om != tm {
                    mask_off.push(format!("t({tx},{ty}) ours {om:#x} theirs {tm:#x}"));
                }
            }
        }
        let owners: Vec<String> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .filter(|&c| built.sim.world.owner(c) != theirs_world.owner(c))
            .map(|c| {
                format!(
                    "cell ({}, {}) ours {:?} theirs {:?}",
                    c.x,
                    c.y,
                    built.sim.world.owner(c),
                    theirs_world.owner(c)
                )
            })
            .collect();
        eprintln!(
            "run72 world at 4802: {} height rows, {} tiles off, {} masks off, \
             {} cells off; mesh {} elements, {} tiles laid by `set_diags`",
            heights.len(),
            z_off.len(),
            mask_off.len(),
            owners.len(),
            built.sim.mesh.len(),
            built.sim.mesh.made()
        );
        // **The mesh's own invariant**, which nothing else here would
        // catch: a road tile has a `RoadElementCandidate` and a tile that
        // is not a road does not. A leak either way is what would make
        // `get_orthog_connects` answer for a tile that has no element, and
        // that gate is most of §9.
        let mut no_elem = Vec::new();
        let mut stray = 0;
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                let road = built.sim.world.tile_mask(q) & sim::world::tile::SURFACE
                    == sim::world::tile::SURFACE_ROAD;
                match (road, built.sim.mesh.elem(&built.sim.world, q).is_some()) {
                    (true, false) => no_elem.push((tx, ty)),
                    (false, true) => stray += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(
            (no_elem.len(), stray, built.sim.mesh.len()),
            (0, 0, 123),
            "every road tile of Great Lakes at 4802 has an element and \
             nothing else does: {no_elem:?}"
        );
        // And the mesh has laid **nothing** in 4,802 frames — 4803 is the
        // first corner on this map that needs filling. The height grid,
        // the masks and the borders above are what says the 4,802 frames
        // of it running changed nothing they should not have.
        assert_eq!(
            built.sim.mesh.made(),
            0,
            "the mesh lays its first tile on 4803"
        );
        for l in z_off
            .iter()
            .take(400)
            .chain(mask_off.iter().take(60))
            .chain(owners.iter().take(60))
        {
            eprintln!("  {l}");
        }

        // The search itself, node for node.
        built.sim.trace_costs = true;
        built.sim.road_marks.clear();
        built.tick();
        let ours = built.sim.road_marks.clone();
        let theirs = t72.road_nodes(4_803);
        eprintln!(
            "run72 road nodes on 4803: ours {} theirs {}",
            ours.len(),
            theirs.len()
        );
        let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i));
        if let Some(i) = at {
            for j in i.saturating_sub(4)..(i + 6).min(ours.len().max(theirs.len())) {
                eprintln!(
                    "  {j:>4} ours   {:?}\n       theirs {:?}",
                    ours.get(j),
                    theirs.get(j)
                );
            }
        }
        // **The height grid is the original's, all 921,600 tiles of it.**
        // It was 62 tiles out — the Market's own 4 × 4 and the taper around
        // it, the map generator's ground where the original had already
        // flattened — until the terraform moved from `Wall::start` to
        // `Wall::init` ([`sim::Sim::init_build`], §7.6). Nothing weaker
        // than the whole grid finds that: the road touches one tile of the
        // 62.
        assert!(
            z_off.is_empty(),
            "the height grid parted from the original's by 4802: {z_off:?}"
        );
        assert!(owners.is_empty(), "the borders parted by 4802: {owners:?}");

        // **The search is whole, and the mesh is why** (2026-09-03, item
        // 202). `World::set_road_at@006b43b0` ends in
        // `Roads::road_added@008954d0` → `Roads::add_roads@0088f4b0` →
        // `Roads::set_diags@0088e9d0`, and `set_diags` lays road tiles of
        // its own (`set_road_at(x + corner_x[i], y, 1, 0, 1)`) where a road
        // stands diagonally from a new one with nothing between them.
        //
        // The ring `place_roads` lays is sixteen tiles, the border of
        // `[224, 228] × [79, 83]`; the original lays a **seventeenth**,
        // `(223, 79)`, which is `set_diags` closing the corner between the
        // ring's brand-new `(224, 79)` and the road already standing at
        // `(223, 80)`. Priced as plain ground at 387 here and as road at 27
        // there, it cost the search eleven nodes — 266 against 277 — and
        // everything downstream of 4809, the position parting at 4827
        // included. With `crate::mesh` it is **node for node the
        // original's, all 277**.
        //
        // The world at 4802 does not move for it: 4,802 frames of a
        // mid-game with the mesh running lay the original's road tiles and
        // no others.
        assert_eq!(
            (at, ours.len(), theirs.len()),
            (None, 277, 277),
            "the Market's road parts at node {at:?} — ours {:?}, theirs {:?}",
            at.and_then(|i| ours.get(i)),
            at.and_then(|i| theirs.get(i)),
        );

        // **The 32 tile masks are a different mechanic altogether**, and
        // the first reading of §9 had them wrong. Every one of them is bit
        // `0x4` — `World::set_behind@006b4230`'s low arm — and its writers
        // are `Wall::mark_behind_tiles@0063d230` (from `Wall::start`,
        // `Wall::close`, `refresh_nearby_tiles` and `cast_bribe`) and
        // `Mountains::add_mountain`. Nothing in `Roads` writes it at all.
        //
        // It was **33** for a session, and three of those carried a second
        // difference: `(217, 124)`, `(218, 124)` and `(219, 124)` were road
        // here and are not there. They are player 1's Farm `o 2012`, and
        // the arm that takes a gatherer's footprint road away is
        // `BuildType::mask_me@006312a0`'s (`docs/ROADS.md` §9.5, item 206).
        // One of the three is now identical and the other two differ by
        // `0x4` alone; the mesh is three elements lighter for it, which is
        // the assertion above.
        //
        // Pinned as a count that may only fall.
        assert_eq!(
            mask_off.len(),
            32,
            "the tile-mask residue is `Wall::mark_behind_tiles`' `0x4` and \
             only falls: {mask_off:?}"
        );
    }

    /// **run72's world on the frame *after* the road went down** — the
    /// widening the ROADS docs-versus-code audit asked for (2026-09-05).
    ///
    /// [`run72_s_road_nodes_are_where_great_lakes_word_parts`] stops at the
    /// end of 4802 and compares the world the search is about to *read*. It
    /// never looks at the world the search **wrote**, and that is where the
    /// order the road is laid in shows: `World::set_road_at@006b43b0` ends
    /// in `Roads::road_added@008954d0` → `Roads::add_roads@0088f4b0` →
    /// `Roads::set_diags@0088e9d0`, and `set_diags` lays road tiles of its
    /// own where a road stands diagonally from a new one with nothing
    /// between them. *Which* tiles those are depends on which neighbours
    /// are already road when each tile goes down, so a road laid
    /// end-to-end backwards can put the same sixteen tiles on the map and
    /// a different seventeenth.
    ///
    /// `DUMP_ALL` covers `[4800, 4806)`, so 4804 is dumped and is the first
    /// block in which the Market's road exists. One more tick than the
    /// sibling test, and the same 921,600 tiles.
    ///
    /// **The audit's row.** `crate::roads`' `place_roads` walks
    /// `reconstruct`'s vector **front to back**, which is the near-*goal*
    /// end first; `crate::caravan`'s `lay_caravan_road` walks the same
    /// vector with `.rev()` and its own comment calls that "the
    /// near-*start* end first". Two consumers of one vector disagree about
    /// which end of it is which, and only one of them can match
    /// `BuildType::place_roads@0063c580`, which pops its `Stack<PathData>`
    /// from the top. Until this ran, nothing on the corpus could tell them
    /// apart.
    #[test]
    fn run72_s_world_after_the_market_s_road_is_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r72)) = (
            dump("gamelog-run71-greatlakes-5k.txt"),
            dump("gamelog-run72-greatlakes-marketroad.txt"),
        ) else {
            eprintln!("skipping: no run71/run72 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        // Capture inputs are needed only while constructing the owned simulation.
        let mut built = with_sibling_initials(|refs| {
            crate::capture::indexed::IndexedCapture::open(&path)
                .unwrap()
                .with_replay_initial(|init| {
                    let mut init = init;
                    borrow_from_siblings(&mut init, refs);
                    if let Some(tr) = trace("rontrace-run71.log") {
                        borrow_pasture(&mut init, &tr);
                    }
                    build_sim(&loaded, &init, Tuning::RON)
                })
                .unwrap()
        });
        // 4,804 ticks stops at the end of sim-frame 4803 — the frame the
        // Market finishes and `place_roads` runs — so this is the world
        // run72's `FRAME 4804` block prints.
        for _ in 0..4_804 {
            built.tick();
        }

        let mut source = crate::capture::indexed::IndexedCapture::open(&r72).unwrap();
        let Some(index) = source.frames().iter().position(|f| f.number == 4_804) else {
            eprintln!("skipping: run72 has no FRAME 4804 block");
            return;
        };
        let text72 = source.read_frame(index).unwrap();
        let l72 = Log::parse(&text72);
        let Some(block) = l72
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 4_804)
            .map(|(_, b)| b)
        else {
            eprintln!("skipping: run72 has no FRAME 4804 block");
            return;
        };
        let Some(w) = block.kid("FULL DUMP").unwrap_or(block).kid("WORLD") else {
            eprintln!("skipping: run72's 4804 block carries no WORLD");
            return;
        };
        let heights = l72.frame_heights(4_804);
        if heights.len() <= 1_000 {
            eprintln!("skipping: run72's 4804 block carries no height grid");
            return;
        }
        let mut notes = Vec::new();
        let (theirs_world, _) = world_from(&w.fields().to_vec(), &heights, &mut notes);
        let (xs, ys) = (theirs_world.width(), theirs_world.height());

        // The road tiles first, on their own, because they are the point: a
        // tile that is a road on one side and not on the other is either a
        // tile `place_roads` laid in the wrong place or one `set_diags`
        // reached from the wrong direction.
        let road = |m: u16| m & sim::world::tile::SURFACE == sim::world::tile::SURFACE_ROAD;
        let mut road_only_ours = Vec::new();
        let mut road_only_theirs = Vec::new();
        let mut mask_off = Vec::new();
        // Whether each mask difference is exactly `World::set_behind`'s low
        // bit, set on the original's side and clear on ours.
        const BEHIND: u16 = 0x4;
        let mut behind_only: Vec<bool> = Vec::new();
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                let (om, tm) = (built.sim.world.tile_mask(q), theirs_world.tile_mask(q));
                match (road(om), road(tm)) {
                    (true, false) => road_only_ours.push((tx, ty)),
                    (false, true) => road_only_theirs.push((tx, ty)),
                    _ => {}
                }
                if om != tm {
                    mask_off.push(format!("t({tx},{ty}) ours {om:#x} theirs {tm:#x}"));
                    behind_only.push(tm ^ om == BEHIND && tm & BEHIND != 0);
                }
            }
        }
        eprintln!(
            "run72 world at 4803: road ours-only {}, theirs-only {}, masks off {}; \
             mesh {} elements, {} tiles laid by `set_diags`",
            road_only_ours.len(),
            road_only_theirs.len(),
            mask_off.len(),
            built.sim.mesh.len(),
            built.sim.mesh.made()
        );
        // **The road tiles are the original's, laid or not.** This is the
        // audit row's answer, and it is negative: `place_roads` walking
        // `reconstruct` in the other direction puts the *same* tiles on
        // the map here — the sixteen of the ring and `set_diags`'
        // seventeenth at `(223, 79)` — so the order is a genuine
        // disagreement between `crate::roads` and `crate::caravan` that
        // this capture cannot see. A capture where two of a road's tiles
        // are diagonal neighbours of *different* standing roads could;
        // none on disk is.
        assert_eq!(
            (road_only_ours.as_slice(), road_only_theirs.as_slice()),
            (&[][..], &[][..]),
            "the road tiles part on the frame the Market lays its road"
        );

        // **What the widening did find**: the residue is 32 tiles at 4802
        // and 45 at 4803, and every one of the thirteen the road frame
        // adds is the same single bit.
        //
        // `0x4` is `World::set_behind@006b4230`'s low arm, whose writers
        // are `Wall::mark_behind_tiles@0063d230` — from `Wall::start`,
        // `Wall::close`, `refresh_nearby_tiles` and `cast_bribe` — and
        // `Mountains::add_mountain`. Nothing in `Roads` writes it, so the
        // thirteen are not the road: they are the Market **finishing** on
        // 4803 and this crate not running `mark_behind_tiles` for it. They
        // sit where that says they should, around the Market's own ring at
        // `[223, 227] × [78, 81]`.
        //
        // Asserted as a property and not only a count, because the count
        // alone would accept a residue that had changed in kind: every
        // differing tile must differ by `0x4` and nothing else, with the
        // bit set on the original's side. That is what makes this a
        // pin on one unimplemented writer rather than a tolerance.
        let not_behind: Vec<&String> = mask_off
            .iter()
            .zip(behind_only.iter())
            .filter(|&(_, &only)| !only)
            .map(|(l, _)| l)
            .collect();
        assert!(
            not_behind.is_empty(),
            "a tile-mask difference that is not `set_behind`'s `0x4`: {not_behind:?}"
        );
        assert_eq!(
            mask_off.len(),
            45,
            "the `set_behind` residue at 4803 — 32 of them are 4802's, and the \
             thirteen the road frame adds are the Market's own \
             `Wall::mark_behind_tiles`. It may only fall: {mask_off:?}"
        );
    }

    /// **run64 — the caravan's road, node for node, and the world it
    /// reads** (2026-09-02).
    ///
    /// run54's game to 6,180 frames with a `DUMP_ALL` window on
    /// `[6164, 6172)` and the three `docs/ROADS.md` §7.2 proxies over
    /// `[6163, 6172]`. `rngcmp.py rontrace-run54.log rontrace-run64.log`:
    /// **6,181 frames, zero differing**, so it is run54's game and the
    /// window and the proxies cost the stream nothing — the fourth capture
    /// in a row of which that is true.
    ///
    /// What it is for: East Indies' word had parted at 6166 on 3,204 road
    /// draws this crate spent none of (`docs/QUEUE.md` item 174), and a
    /// count is not a sequence. This asks the original for every one of
    /// them.
    ///
    /// The five frames are **one search**: `astar_caravan_road` answers −1
    /// on 6166, 6167, 6168 and 6169, parking its three containers in the
    /// `CaravanData` each time, and 1 on 6170. Its bracket names the
    /// endpoints — `oA 2000`, `whoA 1`, `oB 2007`, `caravan 0` — which is
    /// leader 1's two cities, and its `caravan >= 0` is what opens the four
    /// diagonals and swaps the heuristic (`docs/CARAVAN.md` §5).
    #[test]
    fn run64_s_caravan_road_is_the_original_s_node_for_node() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(trace), Some(r64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run64-islands-caravanroad.txt"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(t64) = crate::diff::testkit::trace("rontrace-run64.log") else {
            eprintln!("skipping: no run64 trace");
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6166 {
            built.tick();
        }

        // **The world the search reads, whole.** run64's `FRAME 6166`
        // block is the end of sim-frame 6165, and the tile masks and the
        // height grid it carries are what `valid_roadcoord` and
        // `calc_road_cost` are about to read. The heights are the harder
        // half: a mid-game building re-terraforms them
        // (`crate::terrain`), and the AI's three **farms** had moved 182
        // tiles of them here until `Wall::init`'s `!= FARM` gate landed.
        let text64 = crate::capture::read(&r64);
        let l64 = Log::parse(&text64);
        let block = l64
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 6166)
            .map(|(_, b)| b)
            .expect("run64 dumped frame 6166");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("a WORLD block");
        let mut notes = Vec::new();
        let heights = l64.frame_heights(6166);
        assert!(
            heights.len() > 1000,
            "run64's block carries the height grid"
        );
        let (theirs, _) = world_from(&w.fields().to_vec(), &heights, &mut notes);
        let (xs, ys) = (theirs.width(), theirs.height());
        let mut z_off = Vec::new();
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                if built.sim.world.tile_z(q) != theirs.tile_z(q) && z_off.len() < 8 {
                    z_off.push(format!(
                        "t({tx},{ty}) ours {} theirs {}",
                        built.sim.world.tile_z(q),
                        theirs.tile_z(q)
                    ));
                }
            }
        }
        assert!(
            z_off.is_empty(),
            "the height grid parted from the original's by 6165: {z_off:?}"
        );
        let owners: Vec<String> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .filter(|&c| built.sim.world.owner(c) != theirs.owner(c))
            .map(|c| format!("cell ({}, {})", c.x, c.y))
            .collect();
        assert!(owners.is_empty(), "the borders parted by 6165: {owners:?}");

        // The five frames of the search, each against the original's own
        // priced nodes: the tile, the direction it was reached from, and
        // the price. Frame 6170 is the one that arrives.
        let mut counts = Vec::new();
        for f in 6166..=6170 {
            built.sim.trace_costs = true;
            built.sim.road_marks.clear();
            built.tick();
            let ours = built.sim.road_marks.clone();
            let theirs = t64.road_nodes(f);
            counts.push((f, ours.len(), theirs.len()));
            // **And 6170's prices are the search's again.** They were the
            // stream's for one item: East Indies' word parted on 6169,
            // where the original spent two `Guy::set_anim+0x97a <
            // Guy::inc_time+0x271` draws this crate did not, so by 6170 the
            // jitter was two draws out of phase and every price shifted.
            // Those two are the caravan's own crew figures wrapping their
            // empty packet (`docs/ANIM.md` §3.6), and with them spent the
            // whole node — tile, direction **and** price — is the
            // original's on all five frames.
            let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i));
            assert_eq!(
                at,
                None,
                "frame {f}: the search parts at node {at:?} — ours {:?}, theirs {:?} \
                 (ours {} nodes, theirs {})",
                at.and_then(|i| ours.get(i)),
                at.and_then(|i| theirs.get(i)),
                ours.len(),
                theirs.len()
            );
        }
        eprintln!("run64 road nodes: {counts:?}");
        assert_eq!(
            counts,
            vec![
                (6166, 3204, 3204),
                (6167, 3204, 3204),
                (6168, 3204, 3204),
                (6169, 3202, 3202),
                (6170, 151, 151),
            ],
            "the five frames' node counts — four budgets of 0xc80 and the \
             arrival"
        );
        // The route itself, from the `astar_caravan_road` bracket.
        let brackets = t64.calls_in(6166, 5);
        assert_eq!(brackets.len(), 1, "one road plan on 6166");
        assert_eq!(
            (
                brackets[0].args[1],
                brackets[0].args[2],
                brackets[0].args[3]
            ),
            (2000, 1, 2007),
            "the endpoints are leader 1's two cities"
        );
        let van = &built.sim.caravans[1].slots[0];
        assert!(
            !van.making_road && van.search.is_none(),
            "the plan finished on 6170 and the parked search went with it"
        );
        // **The route's own stack, whole** — twenty-six `PATHDATA`, and
        // the record has printed them since the first `DUMP_ALL`
        // (item 87's ledger again). run64's `FRAME 6171` block is the end
        // of sim-frame 6170, the frame `build_road` answered 1 on, and the
        // eight window frames before it all read an empty stack: this is
        // the only block in the capture that carries a laid road.
        //
        // What it pins is `build_road`'s own loop rather than the search:
        // the world coordinates the crate had been throwing away (its road
        // was a list of *tiles*, so `set_road_at` was handed numbers thirty
        // thousand tiles off the map and every trade road went unlaid), the
        // `0x60` tolerance, and the flag byte — `0x20` on every node that
        // laid tarmac and `1` on the two ends. A water node would carry
        // `0x180` and no `0x20`; this road has none, so the sampling arm
        // is still reading-only.
        let van_road = van.road.clone();
        let frame71 = l64
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 6171)
            .map(|(_, b)| b)
            .expect("run64 dumped frame 6171");
        // The record sits under the leader's own block, which sits under
        // `FULL DUMP`; walk the subtree rather than pin the depth.
        fn vans_of<'b>(b: Block<'b>, out: &mut Vec<Block<'b>>) {
            for c in b.children() {
                if c.name() == "CARAVAN" {
                    out.push(c);
                }
                vans_of(c, out);
            }
        }
        let mut vans = Vec::new();
        vans_of(frame71, &mut vans);
        let theirs_road: Vec<sim::orders::PathData> = vans
            .into_iter()
            .find(|c| c.int("o") == Some(18))
            .map(|c| {
                crate::gamelog::path_of(c)
                    .into_iter()
                    .map(|p| sim::orders::PathData {
                        to: Pos::new(
                            i32::try_from(p.to.0).expect("to_x"),
                            i32::try_from(p.to.1).expect("to_y"),
                        ),
                        tolerance: i32::try_from(p.tolerance).expect("tolerance"),
                        flags: u8::try_from(p.flags).expect("flags"),
                    })
                    .collect()
            })
            .expect("the AI caravan's own CARAVAN record");
        assert_eq!(
            theirs_road.len(),
            26,
            "run64's frame-6171 block: `length 26`"
        );
        assert_eq!(
            van_road, theirs_road,
            "the route's stack parted from the original's"
        );
    }

    /// **The danger map, whole** — `GameDaemon::calc_danger`'s eight rows
    /// against run64's own `danger[who][scan]`, 7,200 values.
    ///
    /// Item 87's ledger again, and this row was the expensive one: the
    /// `WORLD` block has printed the map since the first `DUMP_ALL`
    /// capture and nothing ever read the field, because this crate had no
    /// writer and every reader answered zero. It is not a decoration —
    /// `calc_cost`'s world arm prices a step by `danger / 8`, and around a
    /// leader's own city the map is 65 to 135 *negative*, which is 8 to 16
    /// off every expensive step there. That is what sent East Indies'
    /// caravan south-west on frame 6167 where the original walks straight
    /// at its city (`docs/CARAVAN.md` §4.1, `docs/DANGER.md`).
    ///
    /// The map is rebuilt every two hundredth frame and nothing decays it,
    /// so run64's window (6163–6170) shows the frame-6000 build unchanged
    /// — which makes the eight dumped frames one assertion repeated, and
    /// the check is against the first.
    #[test]
    fn run64_s_danger_map_is_the_original_s() {
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
        let l64 = Log::parse(&text64);
        let theirs = l64
            .dumps()
            .into_iter()
            .find_map(|(n, body)| {
                (n == 6168).then(|| {
                    let w = body.kid("WORLD")?;
                    let d = crate::gamelog::world_danger(&w.fields().to_vec());
                    (!d.is_empty()).then_some(d)
                })?
            })
            .expect("run64's frame-6168 block carries the danger map");
        assert_eq!(
            theirs.len(),
            8 * 900,
            "eight rows of a 60x60 map's reg_size"
        );

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6167 {
            built.tick();
        }
        let reg_xs = built.sim.world.reg_xs();
        assert_eq!(
            (reg_xs, built.sim.world.reg_ys()),
            (30, 30),
            "East Indies' half-cell grid"
        );

        let mut wrong: Vec<String> = Vec::new();
        let mut nonzero = 0usize;
        for who in 0..8u8 {
            let ours = built.sim.world.danger_row(who);
            for i in 0..900usize {
                let t = theirs[who as usize * 900 + i];
                let o = i64::from(ours.get(i).copied().unwrap_or(0));
                if t != 0 {
                    nonzero += 1;
                }
                if t != o && wrong.len() < 20 {
                    wrong.push(format!(
                        "leader {who} half-cell ({},{}): ours {o} theirs {t}",
                        i % 30,
                        i / 30
                    ));
                }
            }
        }
        assert!(
            nonzero >= 100,
            "the dump's own map is nearly empty ({nonzero} nonzero) — wrong capture"
        );
        assert!(wrong.is_empty(), "the danger map parted: {wrong:#?}");
    }

    /// **run189 — the world on Great Lakes' block 14529, whole** (item 695,
    /// `docs/ROADS.md` §10). The original's caravan `1/23` verifies its
    /// route on tick 14529 because the tile under its next waypoint,
    /// (220, 101), is no longer road; here it was `0x190`, placed on and
    /// still road. run189 prints the `WORLD` block, and the only surface
    /// residue at 14529 was **17 road tiles**, (220, 98) south to
    /// (216, 115): the trade road laid on 5573, which the original's
    /// stray-road sweep (`Roads::scan_and_kill_stray_roads@008956a0`) took
    /// down one tile a visit from 5905 on — a stub at the south end first,
    /// then each tile whose element claimed a road no longer there. With
    /// the sweep, **no tile's surface parts**, and what remains is bit
    /// `0x4` alone (`World::set_behind`, as run73's pin has it).
    #[test]
    fn run189_s_world_has_the_original_s_roads_at_14529() {
        const BLOCK: i64 = 14_529;
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r189)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run189-greatlakes-roadword.txt"),
        ) else {
            eprintln!("skipping: no run53/run189 capture (set RON_GAMELOG_DIR)");
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Block N is the state after tick N − 1: 14,529 ticks.
        for _ in 0..BLOCK {
            built.tick();
        }
        let text189 = crate::capture::read(&r189);
        let l189 = Log::parse(&text189);
        let block = l189
            .frames()
            .into_iter()
            .find(|(n, _)| *n == BLOCK)
            .map(|(_, b)| b)
            .expect("run189 dumped block 14529");
        let w = block.kid("WORLD").expect("a WORLD block");
        let mut notes = Vec::new();
        let heights = l189.frame_heights(BLOCK);
        let (theirs, _) = world_from(&w.fields().to_vec(), &heights, &mut notes);
        let (xs, ys) = (theirs.width(), theirs.height());
        let mut surface = Vec::new();
        let mut other = Vec::new();
        let mut behind = 0;
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                let (om, tm) = (built.sim.world.tile_mask(q), theirs.tile_mask(q));
                if om == tm {
                    continue;
                }
                let row = format!("t({tx},{ty}) ours {om:#x} theirs {tm:#x}");
                if (om ^ tm) & sim::world::tile::SURFACE != 0 {
                    surface.push(row);
                } else if om ^ tm == 0x4 {
                    behind += 1;
                } else {
                    other.push(row);
                }
            }
        }
        assert_eq!(surface, Vec::<String>::new(), "a road parts on 14529");
        assert_eq!(other, Vec::<String>::new(), "a mask parts on 14529");
        assert_eq!(behind, 203, "the `0x4` residue on 14529");
        // The two ends the sweep stopped at, element for element.
        let n = |t| built.sim.mesh.elem(&built.sim.world, t).map(|e| e.flags);
        assert_eq!(n(Pos::new(220, 97)), Some(sim::mesh::dir::N), "(220, 97)");
        assert_eq!(n(Pos::new(220, 98)), None, "(220, 98)");
        assert_eq!(
            n(Pos::new(216, 116)),
            Some(sim::mesh::dir::SW),
            "(216, 116)"
        );
    }

    /// **run240 — the world on Great Lakes' block 17087, whole** (item
    /// 776). Tick 17087 plans `1/9`'s and `1/72`'s walks to `1/2022` on the
    /// world-cell grid, and the two plans part on block 17088. The
    /// original's `1/72` goes right round the Pyramids where this crate's
    /// cuts through cell (56, 18). Whether the two search over the same
    /// world is this block's `WORLD` record: the fog, every cell, every
    /// tile mask and the danger map, as they stand before the tick.
    #[test]
    fn run240_s_world_at_17087_is_the_original_s() {
        const BLOCK: i64 = 17_087;
        let Some(inst) = install() else { return };
        let (Some(path), Some(t53), Some(r240)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run240-greatlakes-worldword.txt"),
        ) else {
            eprintln!("skipping: no run53/run240 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r240).unwrap();
        let at = ix
            .frames()
            .iter()
            .position(|f| f.number == BLOCK)
            .expect("run240 has no block 17087");
        let body = ix.read_frame(at).unwrap();
        let parsed = Log::parse(&body);
        let block = parsed
            .frames()
            .into_iter()
            .find(|(n, _)| *n == BLOCK)
            .map(|(_, b)| b)
            .expect("run240's block 17087 did not re-parse");
        let world = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("block 17087 has no WORLD record");
        let fields = world.fields().to_vec();
        let theirs_fog = crate::gamelog::world_fog(&fields);
        let theirs_tiles = crate::gamelog::world_tiles(&fields);
        let theirs_cells = crate::gamelog::world_cells(&fields);
        let theirs_danger = crate::gamelog::world_danger(&fields);
        assert_eq!(
            (theirs_fog.len(), theirs_tiles.len(), theirs_cells.len()),
            (14_400, 57_600, 3_600),
            "run240's block 17087 does not carry a whole WORLD scan"
        );

        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &t53);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Block N is the state after tick N − 1.
        for _ in 0..BLOCK {
            built.tick();
        }
        let w = &built.sim.world;
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        let fog_bad: Vec<(i32, i32, u8, u8)> = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_fog[(y * fw + x) as usize];
                let o = w.seen2(x, y).unwrap_or(0);
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        let cell_bad: Vec<(i32, i32, String)> = (0..w.height())
            .flat_map(|y| (0..w.width()).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let c = sim::world::Cell::new(x, y);
                let d = w.cell_data(c);
                let t = &theirs_cells[(y * w.width() + x) as usize];
                let who = match w.owner(c) {
                    sim::world::Owner::Player(p) => i64::from(p),
                    _ => -1,
                };
                let ours = (
                    i64::from(d.flags),
                    who,
                    i64::from(d.blocked),
                    i64::from(d.solid),
                    i64::from(d.bad),
                );
                let theirs = (t.flags, t.who, t.blocked, t.solid, t.bad);
                (ours != theirs).then(|| (x, y, format!("{ours:?} v {theirs:?}")))
            })
            .collect();
        let (tw, th) = (w.width() * 4, w.height() * 4);
        let tile_bad: Vec<(i32, i32, u16, u16)> = (0..th)
            .flat_map(|y| (0..tw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_tiles[(y * tw + x) as usize];
                let o = w.tile_mask(sim::Pos::new(x, y));
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        let reg = (w.reg_xs() * w.reg_ys()) as usize;
        let mut danger_bad: Vec<String> = Vec::new();
        for who in 0..8u8 {
            let ours = w.danger_row(who);
            for i in 0..reg {
                let t = theirs_danger
                    .get(who as usize * reg + i)
                    .copied()
                    .unwrap_or(0);
                let o = i64::from(ours.get(i).copied().unwrap_or(0));
                if t != o {
                    danger_bad.push(format!(
                        "leader {who} ({},{}): ours {o} theirs {t}",
                        i as i32 % w.reg_xs(),
                        i as i32 / w.reg_xs()
                    ));
                }
            }
        }
        eprintln!(
            "run240 17087: {} fog, {} cells, {} tiles, {} danger part",
            fog_bad.len(),
            cell_bad.len(),
            tile_bad.len(),
            danger_bad.len()
        );
        // **W's killer fires: the world is the original's where the two
        // searches look.** Every cell of the map but one agrees, and that
        // one is (2, 40), sixty cells west, on a bit the searches do not
        // read (`0x2`).
        assert_eq!(
            cell_bad,
            vec![(2, 40, "(128, 0, 0, 0, 4) v (130, 0, 0, 0, 4)".to_string())],
            "the cells at 17087"
        );
        // Every tile mask that parts parts on `0x4` alone —
        // `Wall::mark_behind_tiles`' residue, which `run72`'s and
        // `run189`'s pins carry the same way — and `invalid_loc` reads the
        // cell, not that bit (`docs/PATHFINDER.md` §27).
        assert!(
            tile_bad.iter().all(|&(_, _, o, t)| o ^ t == 0x4),
            "a tile parts on more than `0x4`: {:?}",
            tile_bad
                .iter()
                .filter(|&&(_, _, o, t)| o ^ t != 0x4)
                .collect::<Vec<_>>()
        );
        assert_eq!(tile_bad.len(), 262, "the `0x4` residue at 17087");
        // who=1's danger map is the original's whole; the human's parts on
        // nine half-cells of one corner, (27..29, 9..11), by 3 to 5.
        assert!(
            danger_bad.iter().all(|r| r.starts_with("leader 0 ")),
            "who=1's danger map parts: {danger_bad:?}"
        );
        assert_eq!(danger_bad.len(), 9, "leader 0's nine half-cells");
        // The fog parts on 44 half-cells round the Pyramids (`1/2026`,
        // half-cell (116, 38)): every player's bit there, `0xff`, against
        // who=1's alone here. who=1's own bit — the one `calc_cost`'s fog
        // read asks — agrees on all 44.
        assert!(
            fog_bad.iter().all(|&(x, y, o, t)| (o ^ t) & 0x2 == 0
                && (112..=118).contains(&x)
                && (35..=42).contains(&y)),
            "the fog parts outside the Pyramids' ring, or on who=1's bit: {fog_bad:?}"
        );
        assert_eq!(fog_bad.len(), 44, "the Pyramids' ring");

        // Tick 17087's searches, step for step, against the original's own
        // `calc_cost` calls (run240's trace proxied every one).
        let Some(t240) = trace("rontrace-run240.log") else {
            return;
        };
        built.sim.trace_costs = true;
        built.sim.cost_marks.clear();
        built.tick();
        let ours: Vec<(sim::path::CostKey, i32, usize)> = built
            .sim
            .cost_marks
            .iter()
            .map(|m| (m.key(), m.cost, m.unit))
            .collect();
        let theirs: Vec<(sim::path::CostKey, i32)> = t240
            .calls_in(BLOCK, crate::trace::call_site::CALC_COST)
            .iter()
            .filter_map(|c| Some((c.cost_key()?, c.ret)))
            .collect();
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i).map(|o| (o.0, o.1)) != theirs.get(i).copied());
        if let Some(i) = at {
            for j in i.saturating_sub(6)..(i + 12).min(ours.len().max(theirs.len())) {
                let who = ours.get(j).map(|o| {
                    let u = &built.sim.units[o.2];
                    (u.owner, u.index)
                });
                eprintln!(
                    "  {j:>4} ours   {:?} {who:?}\n       theirs {:?}",
                    ours.get(j).map(|o| (o.0, o.1)),
                    theirs.get(j)
                );
            }
        }
        // **P holds, and the planner was one clause.** `1/9`'s first
        // expansion from cell (56, 19) priced N and W here and not there:
        // cells (56, 18) and (55, 19) are forest-flagged, and
        // `invalid_loc@00607c30`'s land arm refuses a `WData.flags & 0x70`
        // cell under `valid_wcoord`'s `param_3` and `param_6`. With the
        // clause, **all 303 priced steps of the tick** — six searches,
        // `1/9`'s 42 and `1/72`'s 169 among them — are the original's,
        // key and price. 82 agreed before it; the 83rd was `1/9`'s N.
        assert_eq!(
            (at, ours.len(), theirs.len()),
            (None, 303, 303),
            "tick 17087's priced steps part at {at:?}"
        );
    }

    /// **run248 — the world on East Indies' block 17146, and tick 17146's
    /// world searches** (item 773). Tick 17146 moves army 0's column —
    /// `1/55`, `1/57` and `1/58` — to three formation slots eight cells
    /// north, and each plans a world path. `1/55`'s and `1/57`'s agree with
    /// the original's; `1/58`'s parts on block 17147: ours goes straight
    /// north in 10 entries, the original's round the west in 18, in the
    /// column, where its `1/58` gives `1/55` the half steps of 17183 and
    /// 17185 that put the word's stand on 17190.
    #[test]
    fn run248_s_world_at_17146_is_the_original_s() {
        const BLOCK: i64 = 17_146;
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(t54)) = (
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
        borrow_pasture(&mut init, &t54);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Block N is the state after tick N − 1.
        for _ in 0..BLOCK {
            built.tick();
        }
        let w = &built.sim.world;
        let cell_row = |x: i32, y: i32| {
            let c = sim::world::Cell::new(x, y);
            let d = w.cell_data(c);
            let who = match w.owner(c) {
                sim::world::Owner::Player(p) => i64::from(p),
                _ => -1,
            };
            (
                i64::from(d.flags),
                who,
                i64::from(d.blocked),
                i64::from(d.solid),
                i64::from(d.bad),
            )
        };
        let Some(r248) = dump("gamelog-run248-eastindies-worldword.txt") else {
            eprintln!("skipping: no run248 capture");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r248).unwrap();
        let at = ix
            .frames()
            .iter()
            .position(|f| f.number == BLOCK)
            .expect("run248 has no block 17146");
        let body = ix.read_frame(at).unwrap();
        let parsed = Log::parse(&body);
        let block = parsed
            .frames()
            .into_iter()
            .find(|(n, _)| *n == BLOCK)
            .map(|(_, b)| b)
            .expect("run248's block 17146 did not re-parse");
        let world = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("block 17146 has no WORLD record");
        let fields = world.fields().to_vec();
        let theirs_fog = crate::gamelog::world_fog(&fields);
        let theirs_tiles = crate::gamelog::world_tiles(&fields);
        let theirs_cells = crate::gamelog::world_cells(&fields);
        let theirs_danger = crate::gamelog::world_danger(&fields);
        eprintln!(
            "run248 WORLD: {} fog, {} tiles, {} cells, {} danger",
            theirs_fog.len(),
            theirs_tiles.len(),
            theirs_cells.len(),
            theirs_danger.len()
        );
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        let fog_bad: Vec<(i32, i32, u8, u8)> = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_fog[(y * fw + x) as usize];
                let o = w.seen2(x, y).unwrap_or(0);
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        let cell_bad: Vec<(i32, i32, String)> = (0..w.height())
            .flat_map(|y| (0..w.width()).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = &theirs_cells[(y * w.width() + x) as usize];
                let ours = cell_row(x, y);
                let theirs = (t.flags, t.who, t.blocked, t.solid, t.bad);
                (ours != theirs).then(|| (x, y, format!("{ours:?} v {theirs:?}")))
            })
            .collect();
        let (tw, th) = (w.width() * 4, w.height() * 4);
        let tile_bad: Vec<(i32, i32, u16, u16)> = (0..th)
            .flat_map(|y| (0..tw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_tiles[(y * tw + x) as usize];
                let o = w.tile_mask(sim::Pos::new(x, y));
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        let reg = (w.reg_xs() * w.reg_ys()) as usize;
        let mut danger_bad: Vec<String> = Vec::new();
        for who in 0..8u8 {
            let ours = w.danger_row(who);
            for i in 0..reg {
                let t = theirs_danger
                    .get(who as usize * reg + i)
                    .copied()
                    .unwrap_or(0);
                let o = i64::from(ours.get(i).copied().unwrap_or(0));
                if t != o {
                    danger_bad.push(format!(
                        "leader {who} ({},{}): ours {o} theirs {t}",
                        i as i32 % w.reg_xs(),
                        i as i32 / w.reg_xs()
                    ));
                }
            }
        }
        eprintln!(
            "run248 17146: {} fog, {} cells, {} tiles, {} danger part",
            fog_bad.len(),
            cell_bad.len(),
            tile_bad.len(),
            danger_bad.len()
        );
        if std::env::var_os("RON_WORLD_ROWS").is_some() {
            eprintln!("  cells {cell_bad:?}");
            eprintln!(
                "  tiles not 0x4: {:?}",
                tile_bad
                    .iter()
                    .filter(|&&(_, _, o, t)| o ^ t != 0x4)
                    .collect::<Vec<_>>()
            );
            eprintln!("  danger {danger_bad:?}");
            eprintln!("  fog {fog_bad:?}");
        }
        if std::env::var_os("RON_CELLS").is_some() {
            for y in 40..57 {
                let row: Vec<String> = (36..50)
                    .map(|x| {
                        let c = cell_row(x, y);
                        format!("{:>3x}/{:>2}/{:>2}", c.0, c.1, c.2)
                    })
                    .collect();
                eprintln!("  cells y{y}: {}", row.join(" "));
            }
        }
        built.sim.trace_costs = true;
        built.sim.cost_marks.clear();
        built.tick();
        let ours: Vec<(sim::path::CostKey, i32, usize)> = built
            .sim
            .cost_marks
            .iter()
            .map(|m| (m.key(), m.cost, m.unit))
            .collect();
        if std::env::var_os("RON_COSTS").is_some() {
            for (i, (k, c, u)) in ours.iter().enumerate() {
                let x = &built.sim.units[*u];
                eprintln!("  {i:>4} {}/{} {k:?} {c}", x.owner, x.index);
            }
        }
        eprintln!("tick {BLOCK}: {} priced steps here", ours.len());
        let Some(t248) = trace("rontrace-run248.log") else {
            return;
        };
        let theirs: Vec<(sim::path::CostKey, i32)> = t248
            .calls_in(BLOCK, crate::trace::call_site::CALC_COST)
            .iter()
            .filter_map(|c| Some((c.cost_key()?, c.ret)))
            .collect();
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i).map(|o| (o.0, o.1)) != theirs.get(i).copied());
        if let Some(i) = at {
            for j in i.saturating_sub(6)..(i + 12).min(ours.len().max(theirs.len())) {
                let who = ours.get(j).map(|o| {
                    let u = &built.sim.units[o.2];
                    (u.owner, u.index)
                });
                eprintln!(
                    "  {j:>4} ours   {:?} {who:?}\n       theirs {:?}",
                    ours.get(j).map(|o| (o.0, o.1)),
                    theirs.get(j)
                );
            }
        }
        eprintln!(
            "tick {BLOCK}: parts at {at:?}, {} here, {} there",
            ours.len(),
            theirs.len()
        );
    }

    /// **run73 — the caravan's own road on Great Lakes, node for node, and
    /// the road a farm takes away** (2026-09-03, item 206).
    ///
    /// run73's `callwin` covers `[5563, 5581]`, which holds the whole of
    /// `1/23`'s trade-route search: `astar_caravan_road` answers −1 on 5566
    /// through 5572 and **1** on 5573, its bracket naming leader 1's two
    /// cities as the endpoints. Eight frames, 22,145 priced nodes, and the
    /// last of them is where Great Lakes' word parted — 1,754 here against
    /// the original's 1,528.
    ///
    /// A count is not a sequence (run62's lesson, and run72's), so this
    /// asks the original for **every node**: the `valid_roadcoord` that
    /// admitted the candidate carries its world coordinate and the
    /// `calc_road_cost` that follows carries the answer, so a difference
    /// names its own tile.
    ///
    /// **What it found is not in the search at all.** The first node to
    /// part is 2,170 of frame 5572 — the diagonal from tile `(216, 123)`
    /// to `(217, 124)`, which this crate admitted and the original refused.
    /// `(217, 124)` is a footprint tile of player 1's **Farm** `o 2012`,
    /// and it was a **road** here and plain ground there:
    /// `roadcoord_tile`'s occupied arm lets a road through a footprint and
    /// refuses everything else, so one stray tile of tarmac opened a door
    /// the original keeps shut. It is the third of run72's three
    /// unexplained residues (`docs/ROADS.md` §9.4, item 203), 4,802 frames
    /// earlier and still there.
    ///
    /// The cause is a road arm of `BuildType::mask_me@006312a0` that this
    /// crate did not have. A footprint tile of a type that does **not**
    /// connect to roads has its road taken away — `set_road_at(…, 0, 0, 0)`
    /// through the mesh door, gated on `!is_city && (is_gather_type ||
    /// NO_CITY) && !is(UNIVERSITY)` — and a tile of one that does, whose
    /// collision-template byte is not 1, has one **laid**. The two gates
    /// are the same predicate read from opposite sides
    /// (`docs/CITIES.md` §3.6, `docs/ROADS.md` §9.5). The road under the
    /// farm was laid on frame 1104 by somebody else's plan; the farm
    /// started on 3465 and the original took it away there.
    ///
    /// With it: **all eight frames node for node**, and Great Lakes' word
    /// runs 5573 → 5786.
    #[test]
    fn run73_s_caravan_road_is_the_original_s_node_for_node() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace), Some(r73)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run73-greatlakes-caravanstart.txt"),
        ) else {
            eprintln!("skipping: no run53/run73 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(t73) = crate::diff::testkit::trace("rontrace-run73.log") else {
            eprintln!("skipping: no run73 trace");
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Sim-frame 5566 is the 5,567th tick, so this stops at the end of
        // 5565 — the world the search is about to read, and the state
        // run73's `FRAME 5566` block prints.
        for _ in 0..5_566 {
            built.tick();
        }

        // **The world the search reads, whole**, on the map's first
        // `DUMP_ALL` window: 921,600 heights, every cell owner, every tile
        // mask.
        let text73 = crate::capture::read(&r73);
        let l73 = Log::parse(&text73);
        let block = l73
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 5_566)
            .map(|(_, b)| b)
            .expect("run73 dumped frame 5566");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("a WORLD block");
        let mut notes = Vec::new();
        let heights = l73.frame_heights(5_566);
        assert!(
            heights.len() > 1_000,
            "run73's block carries the height grid"
        );
        let (theirs_world, _) = world_from(&w.fields().to_vec(), &heights, &mut notes);
        let (xs, ys) = (theirs_world.width(), theirs_world.height());
        let mut z_off = Vec::new();
        let mut mask_off = Vec::new();
        let mut road_off = Vec::new();
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                if built.sim.world.tile_z(q) != theirs_world.tile_z(q) {
                    z_off.push(format!(
                        "t({tx},{ty}) ours {} theirs {}",
                        built.sim.world.tile_z(q),
                        theirs_world.tile_z(q)
                    ));
                }
                let (om, tm) = (built.sim.world.tile_mask(q), theirs_world.tile_mask(q));
                if om != tm {
                    mask_off.push(format!("t({tx},{ty}) ours {om:#x} theirs {tm:#x}"));
                }
                if om & sim::world::tile::SURFACE != tm & sim::world::tile::SURFACE {
                    road_off.push(format!("t({tx},{ty}) ours {om:#x} theirs {tm:#x}"));
                }
            }
        }
        let owners: Vec<String> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .filter(|&c| built.sim.world.owner(c) != theirs_world.owner(c))
            .map(|c| format!("cell ({}, {})", c.x, c.y))
            .collect();
        eprintln!(
            "run73 world at 5565: {} tiles off, {} masks off ({} of them a \
             surface), {} cells off",
            z_off.len(),
            mask_off.len(),
            road_off.len(),
            owners.len()
        );
        for l in mask_off.iter().take(60) {
            eprintln!("  {l}");
        }
        assert!(
            z_off.is_empty(),
            "the height grid parted from the original's by 5565: {z_off:?}"
        );
        assert!(owners.is_empty(), "the borders parted by 5565: {owners:?}");
        // **Every remaining mask residue is bit `0x4` and nothing else** —
        // `World::set_behind@006b4230`'s low arm, item 203's, which nothing
        // in this crate writes. The three that carried a *surface*
        // difference too — `(217, 124)`, `(218, 124)`, `(219, 124)`, the
        // Farm's own footprint — are gone with `mask_me`'s road arm, and
        // that is what this pins: a surface residue is a road mechanic and
        // may not come back.
        assert!(
            road_off.is_empty(),
            "a tile's surface parted from the original's by 5565: {road_off:?}"
        );
        let not_behind: Vec<&String> = mask_off
            .iter()
            .filter(|l| {
                let hex: Vec<u16> = l
                    .split("ours ")
                    .nth(1)
                    .map(|r| {
                        r.split(" theirs ")
                            .filter_map(|h| {
                                u16::from_str_radix(h.trim_start_matches("0x"), 16).ok()
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                hex.len() != 2 || hex[0] ^ hex[1] != 0x4
            })
            .collect();
        assert!(
            not_behind.is_empty() && mask_off.len() <= 45,
            "the tile-mask residue is `Wall::mark_behind_tiles`' `0x4` alone \
             and only falls: {} rows, {not_behind:?}",
            mask_off.len()
        );

        // **The search itself, node for node, over all eight frames.**
        let mut counts = Vec::new();
        for f in 5_566..=5_573 {
            built.sim.trace_costs = true;
            built.sim.road_marks.clear();
            built.tick();
            let ours = built.sim.road_marks.clone();
            let theirs = t73.road_nodes(f);
            counts.push((f, ours.len(), theirs.len()));
            let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i));
            if let Some(i) = at {
                for j in i.saturating_sub(3)..(i + 4).min(ours.len().max(theirs.len())) {
                    eprintln!(
                        "  {j:>4} ours   {:?}\n       theirs {:?}",
                        ours.get(j),
                        theirs.get(j)
                    );
                }
            }
            assert_eq!(
                at,
                None,
                "frame {f}: the search parts at node {at:?} — ours {:?}, theirs {:?} \
                 (ours {} nodes, theirs {})",
                at.and_then(|i| ours.get(i)),
                at.and_then(|i| theirs.get(i)),
                ours.len(),
                theirs.len()
            );
        }
        eprintln!("run73 road nodes: {counts:?}");
        assert_eq!(
            counts,
            vec![
                (5_566, 3_204, 3_204),
                (5_567, 3_200, 3_200),
                (5_568, 3_200, 3_200),
                (5_569, 3_201, 3_201),
                (5_570, 3_200, 3_200),
                (5_571, 3_200, 3_200),
                (5_572, 3_206, 3_206),
                (5_573, 1_528, 1_528),
            ],
            "the eight frames' node counts — seven budgets of 0xc80 and the \
             arrival"
        );
        // The endpoints, from `astar_caravan_road`'s own bracket.
        let brackets = t73.calls_in(5_566, 5);
        assert_eq!(brackets.len(), 1, "one road plan on 5566");
        assert_eq!(
            (
                brackets[0].args[1],
                brackets[0].args[2],
                brackets[0].args[3]
            ),
            (2000, 1, 2007),
            "the endpoints are leader 1's two cities"
        );
    }

    /// **Run33's frame 361 — the scout's second explore target, and the
    /// surface probe that had been reading the wrong tile.**
    ///
    /// `run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings` pins the
    /// mechanic's *opening* call, ten draws over four rings on a stream
    /// that is installed rather than reached. This pins the one call in
    /// the whole corpus where the cell filter actually decides something:
    /// the AI scout `1/0` re-targets on run33's frame 361, three hundred
    /// frames into a game it walked into on its own stream, and the
    /// original spends **twenty-seven** draws there over seven rings.
    ///
    /// The filter's surface read (`docs/SCOUT.md` §7) is what this
    /// caught. The listing at `005f6542` is
    /// `movb 0x4(%eax,%ecx,2)` over `ecx = (4y + 2)·tile_xs + 4x`, and
    /// `TData` is two bytes wide with its `mask` at `+0` — so the `+4` is
    /// **two elements**, and the tile read is the cell centre
    /// `(4x + 2, 4y + 2)`, not `(4x, 4y + 2)`. Cell `(48, 23)` carries
    /// ocean at tile `(192, 94)` and land at `(194, 94)`: with the wrong
    /// probe it was refused, the call spent twenty-six draws instead of
    /// twenty-seven, and the scout went to `(48, 20)` instead.
    ///
    /// Two oracles, and each catches the transposition on its own: the
    /// trace's own site sequence for the frame, and the destination the
    /// dump prints for the order the call issues.
    #[test]
    fn run33_s_scout_re_targets_at_361_on_the_original_s_cell() {
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
        for _ in 0..363 {
            built.tick();
        }

        // The original's side: frame 361's draws, filtered to the ones
        // `Unit::think_scout` took itself.
        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        let theirs: Vec<String> = trace
            .run_in(361, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| trace.label(d))
            .collect();
        // Seven rings, and thirteen cells taken across three of them —
        // `docs/SCOUT.md` §10's table for this frame.
        let ring = [SITE_ROTATION, SITE_PHASE];
        let mut want: Vec<&str> = Vec::new();
        for cells in [0, 0, 0, 6, 0, 2, 5] {
            want.extend(ring);
            want.extend(std::iter::repeat_n(SITE_CELL, cells));
        }
        assert_eq!(theirs, want, "the trace's own frame-361 sequence");

        // Ours: the same frame's marks, filtered the same way. The stream
        // is **reached**, not installed — the simulation walks its own way
        // to frame 361 and the trace agrees draw for draw to 431.
        let sites = [SITE_ROTATION, SITE_PHASE, SITE_CELL];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 361)
            .map(|(_, v)| v.clone())
            .expect("frame 361's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our twenty-seven draws, at the original's sites, in its order"
        );

        // And the target the call chose, against the dump's own. The order
        // reaches the unit through its group, so the first frame that
        // prints it is 362, one past the frame that thought.
        let frames = log.frame_states();
        let dest = frames
            .iter()
            .find(|f| f.n == 362)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run33's frame-362 order for 1/0");
        assert_eq!(
            dest,
            (37_368, 18_168),
            "the original: inside tile (194, 94)"
        );
        let scout = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].index == 0)
            .expect("1/0");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "cell (48, 23), whose centre tile is land where its first is ocean"
        );
    }

    /// **Run33's frame 482 — the scout's third explore target, and the fog
    /// the vision projection had been throwing to the wrong half-cell.**
    ///
    /// The AI scout `1/0` re-targets again on frame 482, and the ring walk
    /// is the same shape as frame 361's: seven rings around two cities, the
    /// first its own leader's at cell `(55, 21)` walked every other ring to
    /// twelve, then a three-ring look at the human's at `(4, 40)`. What
    /// this pins is the **cell filter's fog read** (`docs/SCOUT.md` §7):
    /// the original scores **three** cells in city one's ring 7 where this
    /// simulation scored two, because cell `(56, 28)` was seen here and not
    /// there.
    ///
    /// Why it was seen here. `Object::update_seen` throws a small land
    /// unit's disc a half-cell forward of its nose, and the angle it
    /// projects along is `UnitData +0x50` — the unit's own `angle`, the
    /// heading `Unit::set_angle` writes toward the next waypoint — not the
    /// guy's eased facing (`docs/VISION.md` §3). This crate had the guy's.
    /// On frame 168 the scout was mid-turn: the dump's `UNITDATA angle` is
    /// −51.6° and its guy's is −83.0°, and the two projections land in
    /// different fog cells. The disc thrown along the guy's angle reached
    /// fog `(113, 57)` and the original's did not, so `(56, 28)` was
    /// "already seen" here three hundred frames later and the scan spent
    /// thirty draws where the original spends thirty-one.
    ///
    /// The score arithmetic is what identifies the cell, and it is worth
    /// keeping because the trace cannot: the four cells this simulation
    /// refuses in that ring are `(50, 26)`, `(56, 28)`, `(58, 28)`,
    /// `(48, 20)` and `(48, 24)`, and the scan's own `dist × 8` puts three
    /// of them — 48, 0 and 32 — **below** the 76 that wins the frame. Only
    /// `(56, 28)` at 96 and `(58, 28)` at 104 can be scored without
    /// changing the target the dump prints, and only `(56, 28)`'s reveal
    /// falls on a frame the scout was turning.
    #[test]
    fn run33_s_scout_re_targets_at_482_on_the_original_s_ring() {
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
        for _ in 0..484 {
            built.tick();
        }

        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        let theirs: Vec<String> = trace
            .run_in(482, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| trace.label(d))
            .collect();
        // Seven rings and six cells: three in the first city's ring 7, then
        // one and two in the human city's rings 1 and 2.
        let ring = [SITE_ROTATION, SITE_PHASE];
        let mut want: Vec<&str> = Vec::new();
        for cells in [0, 0, 0, 3, 0, 1, 2] {
            want.extend(ring);
            want.extend(std::iter::repeat_n(SITE_CELL, cells));
        }
        assert_eq!(theirs, want, "the trace's own frame-482 sequence");

        let sites = [SITE_ROTATION, SITE_PHASE, SITE_CELL];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 482)
            .map(|(_, v)| v.clone())
            .expect("frame 482's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our thirty-one draws, at the original's sites, in its order"
        );

        // Cell `(56, 28)` is the one the projection decided, and the fog is
        // where it shows: unseen for player 1 on the original's stream, and
        // unseen here now.
        assert!(
            built.sim.world.seen2(2 * 56 + 1, 2 * 28 + 1).unwrap_or(0) & 2 == 0,
            "cell (56, 28) is not seen by player 1 at frame 482"
        );

        // And the target, against the dump's own — cell `(52, 28)`, which
        // both sides pick and which the extra draw does not move.
        let frames = log.frame_states();
        let dest = frames
            .iter()
            .find(|f| f.n == 483)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run33's frame-483 order for 1/0");
        assert_eq!(
            dest,
            (40_440, 22_008),
            "the original: inside tile (210, 114)"
        );
        let scout = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].index == 0)
            .expect("1/0");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "cell (52, 28), seven rings out and the cheapest unseen one"
        );
    }

    /// **The fog grid, whole, on ten consecutive frames** — the record the
    /// `WORLD` dump has always printed and nothing compared.
    ///
    /// `seen2` is a 120 × 120 byte grid on this map, one bit a player, and
    /// three things read it: the pathfinder's unseen-cell preference, the
    /// scout's cell filter and the AI's site census. Until this test it was
    /// installed from a frame-0 dump and then grown by `crate::vision`
    /// with **nothing checking the growth** — a wrong disc, a wrong centre
    /// or a wrong radius would show up only when some later mechanic read
    /// a cell it had got wrong, three hundred frames downstream and wearing
    /// somebody else's name. That is exactly how it went: item 79 was a
    /// scout's ring walk and turned out to be this grid.
    ///
    /// run13 is run10's own game with `DUMP_ALL` over frames 95–104, so it
    /// prints the grid ten times. The simulation is stood up on run33's
    /// start (the same game again) and walked forward with nothing
    /// installed; each frame's grid is compared cell for cell against the
    /// dump taken at the **start** of the next frame.
    #[test]
    fn run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r13)) = (
            dump("gamelog-run33-longtrace.txt"),
            dump("gamelog-run13-window-95-105.txt"),
        ) else {
            eprintln!("skipping: no run33/run13 capture (set RON_GAMELOG_DIR)");
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

        let t13 = crate::capture::read(&r13);
        let l13 = Log::parse(&t13);
        let grids: Vec<(i64, Vec<u8>)> = l13
            .dumps()
            .into_iter()
            .filter_map(|(n, body)| {
                let w = body.kid("WORLD")?;
                let fog = crate::gamelog::world_fog(&w.fields().to_vec());
                (!fog.is_empty()).then_some((n, fog))
            })
            .collect();
        assert_eq!(
            grids.iter().map(|(n, _)| *n).collect::<Vec<i64>>(),
            (95..=104).collect::<Vec<i64>>(),
            "run13's ten dumped worlds"
        );

        let (fw, fh) = (built.sim.world.fog_xs(), built.sim.world.fog_ys());
        assert_eq!((fw, fh), (120, 120), "Great Lakes' fog grid");
        for f in 1..=103i64 {
            built.tick();
            // The dump at the head of frame `f + 1` is the state this many
            // ticks have produced.
            let Some((_, theirs)) = grids.iter().find(|(n, _)| *n == f + 1) else {
                continue;
            };
            assert_eq!(theirs.len(), (fw * fh) as usize, "frame {f}'s grid size");
            let bad: Vec<(i32, i32, u8, u8)> = (0..fh)
                .flat_map(|y| (0..fw).map(move |x| (x, y)))
                .filter_map(|(x, y)| {
                    let t = theirs[(y * fw + x) as usize];
                    let o = built.sim.world.seen2(x, y).unwrap_or(0);
                    (o != t).then_some((x, y, o, t))
                })
                .collect();
            assert_eq!(
                bad,
                vec![],
                "the fog grid parts after {f} ticks, against run13's FRAME {}",
                f + 1
            );
        }
    }

    /// **The fog plane at the parting block, and the price it buys**
    /// (item 320, 2026-09-17) — run95.
    ///
    /// The test above says this crate's world is the original's at block
    /// 7932. **Seventy blocks later it is not**, and the difference is the
    /// whole of Great Lakes' 8031: `1/0`'s `find_wpath` prices the step
    /// into cell `(3, 39)` — one of the four the human capital's footprint
    /// stands on — at **9**, the unseen scouting price, where the original
    /// prices it at **328**, the seen one (`docs/PATHFINDER.md` §20).
    ///
    /// run95's `callwin` proxy carries that number directly, and its
    /// `DUMP_ALL` window carries the plane the number comes from, so both
    /// halves are pinned here:
    ///
    /// - **14 half-cells** of 14,400 part at block 8002, in two patches.
    ///   The larger is the 3 × 5 block `x` 6–8, `y` 78–82 — which is two
    ///   radius-1 discs (`circle_radius[1]`, the 3 × 3) centred on `(7, 79)`
    ///   and `(7, 81)`, the `2c + 1` half-cells of cells `(3, 39)` and
    ///   `(3, 40)`. The smaller is `(10, 73)` and `(11, 73)`, single points
    ///   beside the human's `0/1` and `0/2`. Every one is the **AI's** bit
    ///   over ground the **human** occupies.
    /// - the original's own `calc_cost` for the two steps, off the trace.
    ///
    /// **Both are now zero** (item 322, 2026-09-17), and the writer was
    /// neither `visible` nor a disc. The dump prints `ever_seen` on every
    /// `BUILDDATA` record, and at block 8002 **exactly two** of player 0's
    /// seven buildings carry `3` — the capital `0/2000` at half-cell
    /// `(8, 80)` and `0/2001` at `(11, 74)` — where the other five carry
    /// `1`. Those two are the centres of the two patches. What writes it is
    /// `Wall::check_ever_seen@0063ce70`, every eighth frame from
    /// `Wall::process`, and the reveal it triggers is
    /// `Wall::update_local_seen@0063ed50` over the footprint **grown by one
    /// tile each way** — which for the Small City's 7 × 7 at corner tile
    /// `(13, 157)` is tiles 12–20 × 156–164, exactly half-cells 6–10 × 78–82.
    /// `docs/VISION.md` §6.1.
    ///
    /// So the assertion now pins the plane **exact** — 14,400 of 14,400 —
    /// and it still fails in both directions: losing the reveal brings the
    /// fourteen back, and lighting the wrong rectangle adds cells of its
    /// own.
    #[test]
    fn run95_s_block_8002_is_where_the_fog_parts_and_the_price_with_it() {
        let Some(inst) = install() else { return };
        let (Some(seed_dump), Some(tr53), Some(scan), Some(tr95)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run95-greatlakes-scoutcosts.txt"),
            trace("rontrace-run95.log"),
        ) else {
            eprintln!("skipping: no run53/run95 capture (set RON_GAMELOG_DIR)");
            return;
        };

        // **The original's own price, off the proxy.** `calc_cost` is
        // entered once per step the search considers, so the *presence* of
        // a record is itself a verdict: a `valid_wcoord` refusal happens
        // upstream of the call, and there is no refusal here — the step is
        // priced, and priced as seen ground with nine blocked tiles.
        let costs = tr95.calls_in(8001, crate::trace::call_site::CALC_COST);
        assert_eq!(
            costs.len(),
            83,
            "run95's frame 8001 no longer carries 83 calc_cost records — \
             the whole of `1/0`'s one search, and the only search the \
             window holds"
        );
        let priced = |from: (i32, i32), to: (i32, i32)| -> Vec<i32> {
            costs
                .iter()
                .filter(|c| {
                    (c.args[0], c.args[1]) == (from.0, from.1)
                        && (c.args[2], c.args[3]) == (to.0, to.1)
                })
                .map(|c| c.ret)
                .collect()
        };
        assert_eq!(
            priced((3456, 29568), (2688, 30336)),
            vec![328],
            "the original's step from cell (4,38) into the capital's cell \
             (3,39) is no longer 328 — 128 base at a scout's seen rate, \
             8 of danger, 4 of enemy ground, 20 x 9 of terrain and the \
             diagonal's 8. This crate prices it 9."
        );
        assert_eq!(
            priced((3456, 29568), (2688, 29568)),
            vec![1],
            "the original's step into cell (3,38) is no longer 1 — it is \
             the unseen scouting price, and it is what says the difference \
             is the fog rather than the terrain: (3,38) carries nine \
             blocked tiles too"
        );

        // **And the plane those prices read.** run95's DUMP_ALL window
        // covers 7999–8003; block 8002 is the one the search runs on.
        let mut ix = crate::capture::indexed::IndexedCapture::open(&scan).unwrap();
        let at = ix
            .frames()
            .iter()
            .position(|f| f.number == 8002)
            .expect("run95 has no block 8002 — the wrong file");
        let body = ix.read_frame(at).unwrap();
        let parsed = Log::parse(&body);
        let block = parsed
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 8002)
            .map(|(_, b)| b)
            .expect("run95's block 8002 did not re-parse");
        let world = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("block 8002 has no WORLD record");
        let theirs = crate::gamelog::world_fog(&world.fields().to_vec());
        assert_eq!(theirs.len(), 14_400, "block 8002's fog plane");

        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&seed_dump);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        crate::diff::setup::borrow_pasture(&mut init, &tr53);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..8002 {
            built.tick();
        }
        let w = &built.sim.world;
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        let bad: Vec<(i32, i32, u8, u8)> = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs[(y * fw + x) as usize];
                let o = w.seen2(x, y).unwrap_or(0);
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        eprintln!("run95 8002: {} half-cells part {bad:?}", bad.len());
        assert_eq!(
            bad,
            vec![],
            "run95's block 8002 fog plane parts from this crate's. Until \
             item 322 it parted on fourteen half-cells — (10,73), (11,73) \
             and the block x 6-8, y 78-82, every one the AI's bit over \
             ground the human holds — which is the reveal \
             `Wall::check_ever_seen` makes when a scout first lays eyes on \
             a building (`docs/VISION.md` §6.1). Those fourteen coming back \
             is that mechanic lost; anything else is a reveal lighting the \
             wrong rectangle"
        );

        // **And `ever_seen` itself, record for record.** This is the byte
        // the reveal hangs off, and the dump has printed it on every
        // `BUILDDATA` since `BUILDS=1`: the capital `0/2000` and `0/2001`
        // are the only two of player 0's seven the AI has laid eyes on, and
        // they are the centres of the two patches. Comparing the byte and
        // not only the plane is what tells "the reveal fired late" from
        // "the reveal fired on the wrong building".
        let theirs_builds = parsed.frame_builds(8002);
        assert_eq!(
            theirs_builds.len(),
            28,
            "run95's block 8002 no longer carries 28 buildings — 7 of \
             player 0's and 21 of player 1's — so it is a different game"
        );
        let mut parting = Vec::new();
        for r in &theirs_builds {
            let Some(want) = r.ever_seen else { continue };
            let Some(b) = built
                .sim
                .buildings
                .iter()
                .position(|b| i64::from(b.owner) == r.who && i64::from(b.index) == r.o)
            else {
                continue;
            };
            let got = i64::from(built.sim.buildings[b].ever_seen);
            if got != want {
                parting.push((r.who, r.o, got, want));
            }
            // `ever_seen_completed` rides along: it is the same scan under
            // `is_active`, and the two unfinished buildings `1/2019` and
            // `1/2020` are what tell the pair apart — `2` against `0`.
            if let Some(wc) = r.ever_seen_completed {
                let gc = i64::from(built.sim.buildings[b].ever_seen_completed);
                if gc != wc {
                    parting.push((r.who, -r.o, gc, wc));
                }
            }
        }
        assert_eq!(
            parting,
            vec![],
            "`ever_seen` parts on run95's block 8002, as (who, o, ours, \
             theirs), `o` negated for `ever_seen_completed`. The two that \
             matter are player 0's 2000 and 2001, which read 3 — the AI \
             has seen them — where the other five read 1 \
             (`docs/VISION.md` §6.1)"
        );
    }

    /// **The mid-game world, whole, against the original's own scan**
    /// (item 320, 2026-09-17).
    ///
    /// `docs/VISION.md` §7 said "no dump on disk carries a *second* fog
    /// plane to diff against" and costed the capture that would fix it.
    /// **Three already do.** A `DUMP_ALL` *window* prints the whole `WORLD`
    /// scan on every block it covers, and three Great Lakes archives carry
    /// one mid-game: run13 at 95–104 (the test above), **run73 at
    /// 5564–5580**, and **run93 at 7929–7936** — the last of them 7,932
    /// frames in, seventy frames under this map's own word. So the fog
    /// plane, every cell's `WData` and every tile's mask are diffable at
    /// the frame the pathfinder actually reads them, and this is that diff.
    ///
    /// **What it says.** At block 7932 this crate's world is the
    /// original's: **14,400 of 14,400** fog half-cells, **3,600 of 3,600**
    /// cells on `flags`/`who`/`blocked`/`solid`/`bad`, and 57,429 of
    /// **57,600** tile masks. That is what closed item 320's first two
    /// hypotheses: the AI scout's route parts at block 8002 through the two
    /// cells of the human capital's footprint, and neither the fog it reads
    /// nor the terrain it prices is the difference
    /// (`docs/PATHFINDER.md` §20).
    ///
    /// **The 171 tiles that do differ are pinned, not waived**, because
    /// they are a finding rather than noise: every one is in a single
    /// cluster at tiles `(209..=215, 74..=77)` — cells `(52..=53, 18..=19)`,
    /// the AI's own base, two hundred cells east of the scout — and the
    /// differing bits are the low `0x4` and `0x2000`. Nothing on this map's
    /// word depends on them; a successor that fixes them should shrink this
    /// number, and one that grows it has broken something.
    #[test]
    fn run93_s_block_7932_is_this_crate_s_world_cell_for_cell() {
        let Some(inst) = install() else { return };
        let (Some(seed_dump), Some(tr), Some(scan)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run93-greatlakes-firsttarget.txt"),
        ) else {
            eprintln!("skipping: no run53/run93 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // **The scan is read one block at a time.** run93 is 308 MB and the
        // block wanted is 60 MB of it; `IndexedCapture` is what keeps this
        // test inside the gate's memory ceiling (`tools/memcap.sh`).
        let mut ix = crate::capture::indexed::IndexedCapture::open(&scan).unwrap();
        let at = ix
            .frames()
            .iter()
            .position(|f| f.number == 7932)
            .expect("run93 has no block 7932 — the wrong file");
        let body = ix.read_frame(at).unwrap();
        let parsed = Log::parse(&body);
        let block = parsed
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 7932)
            .map(|(_, b)| b)
            .expect("run93's block 7932 did not re-parse");
        let world = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("block 7932 has no WORLD record — the wrong detail");
        let fields = world.fields().to_vec();
        let theirs_fog = crate::gamelog::world_fog(&fields);
        let theirs_tiles = crate::gamelog::world_tiles(&fields);
        let theirs_cells = crate::gamelog::world_cells(&fields);
        assert_eq!(
            (theirs_fog.len(), theirs_tiles.len(), theirs_cells.len()),
            (14_400, 57_600, 3_600),
            "run93's block 7932 does not carry a whole WORLD scan"
        );

        // This crate's own world at the same block: run53's initial, its
        // trace's pasture, and 7,932 ticks. No per-frame seed is installed
        // — the point is that the free-running simulation is still on the
        // original's world seventy frames under the word.
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&seed_dump);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        crate::diff::setup::borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..7932 {
            built.tick();
        }
        let w = &built.sim.world;
        let (fw, fh) = (w.fog_xs(), w.fog_ys());
        assert_eq!((fw, fh, w.width(), w.height()), (120, 120, 60, 60));

        let fog_bad: Vec<(i32, i32, u8, u8)> = (0..fh)
            .flat_map(|y| (0..fw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_fog[(y * fw + x) as usize];
                let o = w.seen2(x, y).unwrap_or(0);
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        assert_eq!(
            fog_bad,
            vec![],
            "the fog plane parts at block 7932 — 7,932 frames of every reveal \
             this simulation makes, against the original's own scan"
        );

        let cell_bad: Vec<(i32, i32, String)> = (0..w.height())
            .flat_map(|y| (0..w.width()).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let c = sim::world::Cell::new(x, y);
                let d = w.cell_data(c);
                let t = &theirs_cells[(y * w.width() + x) as usize];
                let who = match w.owner(c) {
                    sim::world::Owner::Player(p) => i64::from(p),
                    _ => -1,
                };
                let ours = (
                    i64::from(d.flags),
                    who,
                    i64::from(d.blocked),
                    i64::from(d.solid),
                    i64::from(d.bad),
                );
                let theirs = (t.flags, t.who, t.blocked, t.solid, t.bad);
                (ours != theirs).then(|| (x, y, format!("{ours:?} v {theirs:?}")))
            })
            .collect();
        eprintln!("run93 7932: {} cells part", cell_bad.len());
        // **The 12 that part are pinned by count and by place**, because
        // they are a finding rather than noise: every one is in the AI's
        // own base (`x >= 37`, `y` 18-34), thirty cells east of the human
        // capital this item's scout walks past, and the differences are a
        // blocked/solid count one or two short — a mark a finished building
        // leaves. **None is in the western half of the map at all**, which
        // is what makes `docs/PATHFINDER.md` §20's elimination stand.
        //
        // **27 → 12 on item 352**, and what went was the whole of the
        // `BUILDING 0x4000` half: `BuildType::mask_me`'s first statement
        // sets the bit on a building's own cell and nothing in this crate
        // wrote it (`docs/ARMY.md` §13). This block is the only mid-game
        // world dump on disk for Great Lakes, so it is also **the capture
        // that would have refused the fix** — 7,932 frames and every
        // building the AI has started, cell for cell, and the bit agrees on
        // all 3,600.
        //
        // **12 → 1 on item 629**, and what went was every blocked/solid
        // count: the "mark a finished building leaves" was the three
        // Merchants' footprints — `cast_unpack`'s merchant arm blocks the
        // two-by-two under each (`docs/MERCHANT.md` §3.2). What stands is
        // cell (54, 28)'s flag `0x80` here against 0 there, which the arm
        // does not write and which parted before it.
        //
        // **1 → 0 on item 695**: (54, 28)'s `0x80` is the cell's road flag,
        // and its four tiles of the trade road, (216, 112..115), are the
        // first four the stray-road sweep takes, 5905 to 7448
        // (`docs/ROADS.md` §10). All 3,600 cells agree.
        assert_eq!(
            cell_bad.len(),
            0,
            "run93's block 7932 parts on a cell again — if the AI's \
             base has been fixed this pin is the one to lower, and if it has \
             grown the landing that grew it is the bug: {:?}",
            &cell_bad[..cell_bad.len().min(8)]
        );
        assert!(
            cell_bad
                .iter()
                .all(|&(x, y, _)| x >= 37 && (18..=34).contains(&y)),
            "a cell parts outside the AI's base — a second site is a second \
             finding: {:?}",
            &cell_bad[..cell_bad.len().min(8)]
        );

        let (tw, th) = (w.width() * 4, w.height() * 4);
        let tile_bad: Vec<(i32, i32, u16, u16)> = (0..th)
            .flat_map(|y| (0..tw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let t = theirs_tiles[(y * tw + x) as usize];
                let o = w.tile_mask(sim::Pos::new(x, y));
                (o != t).then_some((x, y, o, t))
            })
            .collect();
        eprintln!("run93 7932: {} tile masks part", tile_bad.len());
        // **171 → 146 on item 629**: two Merchants' four-tile footprints,
        // tiles (150–151, 79–80) and (167–168, 111–112) — `0x4000` and its
        // `0x2000` halo there, nothing here — agree now, and the third's,
        // (209–212, 74–77), agrees on both bits and keeps only the `0x4`
        // the rest of this cluster lacks. No tile parts that did not
        // before (`docs/MERCHANT.md` §3.2).
        //
        // **146 → 142 on item 695**: the trade road's (216, 112..115),
        // road here and plain there, which the stray-road sweep takes
        // between 5905 and 7448 (`docs/ROADS.md` §10).
        assert!(
            tile_bad
                .iter()
                .all(|&(x, y, _, _)| !(x == 216 && (112..=115).contains(&y))),
            "the sweep's four tiles part again at 7932"
        );
        assert_eq!(
            tile_bad.len(),
            142,
            "run93's block 7932 no longer parts on 142 tile masks — if the \
             cluster has been fixed this pin is the one to lower, and if it \
             has grown the landing that grew it is the bug"
        );
        // The same forty-three cells as above, tile for tile: `x` 37–58,
        // `y` 18–34 in cells, which is the AI's own base and nothing else.
        assert!(
            tile_bad
                .iter()
                .all(|&(x, y, _, _)| x / 4 >= 37 && (18..=34).contains(&(y / 4))),
            "the differing tiles are no longer confined to the AI's own base \
             — a second site is a second finding: {:?}",
            &tile_bad[..tile_bad.len().min(8)]
        );
    }

    /// **A scout whose city loop finds nothing scans its whole region**
    /// (2026-08-31, item 110) — the mechanic behind East Indies' word going
    /// 1373 → 1570, asserted as a sequence and as a destination.
    ///
    /// Run39's frame 1373 is the one frame in the corpus that reaches
    /// `docs/SCOUT.md` §11. The AI scout `1/0` walks its own city's six
    /// rings — `+0x436`/`+0x458` six times, and **not one `+0x64c`**,
    /// because by 1373 every cell within twelve of that city has been seen
    /// — so the city loop comes out with `best` still at 99,999,999, the
    /// tail's `199 < best` sends it to the region fallback, and the
    /// original spends one `+0x941` and five `+0xaba` there.
    ///
    /// That branch had been read but not implemented, on the ground that
    /// it strides through `Region.coords` and no dump prints that list. It
    /// does not need one. `Regions::find_all@0067eff0` appends coordinates
    /// in its flood order, merges, sorts — and then **frees the list and
    /// calls `Regions::rebuild_coords@0067f800`**, which refills every
    /// region's array by a plain row-major sweep of the cell grid:
    /// `for y { for x { coords[wdata[xs·y + x].region].push((x, y)) } }`.
    /// So the order is the grid's own, the sim's per-cell region map from
    /// the `WORLD` dump is enough to rebuild it, and this branch is now
    /// the original's draw for draw.
    ///
    /// Two oracles, as with the two run33 re-targets: the trace's own site
    /// sequence for the frame, and the destination the dump prints for the
    /// order the call issues. Made to fail three ways: the scan's own
    /// `+0x941` mark removed — which is the defect this found, the stride
    /// draw inheriting `SITE_PHASE` and reading as a seventh ring; the
    /// coordinate sweep transposed to column-major, which takes **one**
    /// cell where the original takes five; and the surface probe put back
    /// on `4x` rather than the cell centre `4x + 2` — §7's own trap, which
    /// costs exactly one cell here as it did on run33's 361.
    ///
    /// What this frame does **not** separate, and what therefore stays a
    /// reading in `docs/SCOUT.md` §13: the `× 16` distance scale (the city
    /// loop's is `× 8`), the doubling of the winner at `005f6bb9`, and the
    /// sense of `local_74`. Each was inverted in turn and the word held at
    /// 1570 — the five cells are ordered the same way either way, and
    /// `best` is still 99,999,999 when the scan starts, so nothing here
    /// compares a region score against a city one.
    #[test]
    fn a_scout_with_no_city_near_scans_its_whole_region() {
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
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..1375 {
            built.tick();
        }

        // The original's side: frame 1373's draws, filtered to the ones
        // `Unit::think_scout` took itself.
        use sim::scout::{
            SITE_CELL, SITE_PHASE, SITE_REGION_CELL, SITE_REGION_STRIDE, SITE_ROTATION,
        };
        let theirs: Vec<String> = tr
            .run_in(1373, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| tr.label(d))
            .collect();
        let mut want: Vec<&str> = Vec::new();
        for _ in 0..6 {
            want.extend([SITE_ROTATION, SITE_PHASE]);
        }
        want.push(SITE_REGION_STRIDE);
        want.extend(std::iter::repeat_n(SITE_REGION_CELL, 5));
        assert_eq!(theirs, want, "the trace's own frame-1373 sequence");

        // Ours: the same frame's marks, filtered the same way, on a stream
        // this simulation **reached** rather than had installed.
        let sites = [
            SITE_ROTATION,
            SITE_PHASE,
            SITE_CELL,
            SITE_REGION_STRIDE,
            SITE_REGION_CELL,
        ];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 1373)
            .map(|(_, v)| v.clone())
            .expect("frame 1373's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our twelve ring draws and the region scan's six, in the original's order"
        );

        // And the cell the scan chose, against the dump's own. The order
        // reaches the unit through its group, so the first frame that
        // prints it is 1374.
        let dest = log
            .frame_states()
            .iter()
            .find(|f| f.n == 1374)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run39's frame-1374 order for 1/0");
        assert_eq!(
            dest,
            (37_368, 33_528),
            "cell (48, 43), whose centre tile is (194, 174) — the dump's own \
             `orig_x/orig_y` are that centre and `dest` is it plus §13 item 8b's 24"
        );
        let scout = built
            .sim
            .unit_by_o(1, 0)
            .expect("run39 dumps player 1's `1/0`");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "the region scan's winner, not the ring walk's"
        );
    }

    /// **A far wander sweeps from a literal bearing, not the animal's own**
    /// (2026-08-31, item 102) — the mechanic behind East Indies' word going
    /// 742 → 867, asserted where the word's own count cannot see it.
    ///
    /// `Animal::do_idle@005d7460`'s far branch hands
    /// `UnitType::find_nearby_spot` **`0x55555555`** as the angle its
    /// thirty-one bearings sweep out from — the ninth argument, where
    /// `do_gather`, `do_build`, `do_garrison` and every other call site in
    /// the executable passes a real heading. It is
    /// [`sim::movement::Angle::INITIAL`]: the 120° `Unit::init` writes into
    /// a unit that has never turned, a literal with nothing behind it. So
    /// every far wander any herd makes starts its sweep from the same
    /// direction, whichever way the animal happens to be looking.
    ///
    /// This crate passed `Movement::facing`, and run39's `8/3` was facing
    /// **south** (`UNITDATA angle -2147483648`) when its coin came up on
    /// frame 736 — 180° out. Two things follow, and the second is the one
    /// the item was booked as:
    ///
    /// - **The walk.** From the herd centre `(28800, 23936)` the original's
    ///   first ring at `0xc0` refuses 120° and 142.5° and takes **97.5°**,
    ///   `k = −1` of the sweep, which snaps to `(28968, 23976)`; this crate
    ///   took a bearing near due north and walked to `(28728, 24120)`. The
    ///   dump prints the walk step for step and the two share only its
    ///   first frame.
    /// - **The stand.** Six frames later the original's step is refused by
    ///   the herd-mate `8/2` standing at `(28856, 24197)`, and `move_step`
    ///   spends the blocked stand — `Guy::set_anim+0x97a <
    ///   Unit::move_step+0x823` — before its give-up tests
    ///   (`docs/COLLISION.md` §5). Ours was two hundred units west of that
    ///   and walked on. **The collision model was already right**: with the
    ///   bearing corrected the refusal, the `QUEUE_NEW` clear and the stand
    ///   all fall on the original's own frames with nothing else changed.
    ///
    /// Both halves are here. The first is `8/3`'s whole walk against the
    /// dump, frame for frame; the second is that the walk **stops** on the
    /// frame the original's does, at the point the original's does. Made to
    /// fail by putting `Movement::facing` back, which parts the walk on its
    /// second frame and never reaches the stand.
    #[test]
    fn a_far_wander_sweeps_from_the_literal_bearing() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // **Gaia's positions, on every frame the dump prints them.** run39
        // carries no `GUY` clocks, so `Built::tick` re-seats nothing here
        // (`Sim::reseat_animal` needs one) and the animals free-run for the
        // whole capture — which is what makes this a comparison at all.
        // Owner 8 only: the pasture's five and the birds are owner 9 and no
        // dump prints owner 9.
        let mut walk: Vec<(i64, i32, i32)> = Vec::new();
        let (mut agree, mut seen) = (0usize, 0usize);
        // The first animal-frame that is not the original's: the frame, the
        // `o`, ours and theirs. Printed rather than asserted — it is the
        // successor item the next time the floor moves.
        let mut first_bad: Option<String> = None;
        let mut first_bad_frame: i64 = i64::MAX;
        let mut last = 0i64;
        for f in log.frame_states() {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in f.units.iter().filter(|u| u.who == 8) {
                let Some(unit) = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(8, o))
                else {
                    continue;
                };
                let ours = built.sim.units[unit].pos;
                if u.o == 3 && (730..=750).contains(&f.n) {
                    walk.push((f.n, ours.x, ours.y));
                }
                seen += 1;
                if i64::from(ours.x) == u.pos.x && i64::from(ours.y) == u.pos.y {
                    agree += 1;
                } else if first_bad.is_none() {
                    first_bad_frame = f.n;
                    first_bad = Some(format!(
                        "frame {} 8/{}: ours ({}, {}) theirs ({}, {})",
                        f.n, u.o, ours.x, ours.y, u.pos.x, u.pos.y
                    ));
                }
            }
        }
        eprintln!(
            "run39 gaia: {agree} of {seen} dumped animal-frames on the original's point, \
             first {first_bad:?}"
        );
        // `8/3`'s whole walk: the coin comes up on 736, the first step
        // lands on 737, and 742's is refused — after which the original
        // holds the point for the rest of the capture.
        assert_eq!(
            walk,
            vec![
                (730, 28728, 24408),
                (731, 28728, 24408),
                (732, 28728, 24408),
                (733, 28728, 24408),
                (734, 28728, 24408),
                (735, 28728, 24408),
                (736, 28728, 24408),
                (737, 28728, 24408),
                (738, 28741, 24384),
                (739, 28754, 24360),
                (740, 28767, 24336),
                (741, 28780, 24312),
                (742, 28793, 24288),
                (743, 28793, 24288),
                (744, 28793, 24288),
                (745, 28793, 24288),
                (746, 28793, 24288),
                (747, 28793, 24288),
                (748, 28793, 24288),
                (749, 28793, 24288),
                (750, 28793, 24288),
            ],
            "run39's `8/3` walks the original's ground and is refused on 742"
        );
        // And the walk is dropped where it stands rather than pathed round
        // (`docs/COLLISION.md` §6 step 0): no order, and the point held.
        let u = built.sim.unit_by_o(8, 3).expect("run39 dumps gaia's `8/3`");
        assert!(built.sim.units[u].orders.is_empty());
        // **The whole-capture floor**, and it is the wider claim: gaia's
        // animals are the one population this capture lets free-run for
        // 1,850 frames with nothing installed, and the first animal-frame
        // that is not the original's is `8/3` again on frame **983**, where
        // the original wanders off the point it was refused at and this
        // crate has not yet. That frame is the number with meaning and it
        // may only rise.
        //
        // The whole-capture *count* is the weaker half, and it is not
        // monotone. Every frame past the word's own parting is drawn from a
        // stream that is nobody's — a herd's next coin is whatever the
        // frames before it happened to spend — so a mechanic that moves the
        // word forward re-rolls all of them: item 104's goody box took the
        // word 867 → 879 and the count 190,417 → 189,843, with **983
        // unmoved**. Read the count as a floor on the same word, not as a
        // score across words. Item 106 then moved both: 879 → 1256 took
        // the count to 190,690 and the first parting to **1261**, five
        // frames past the word, because the walk to the box put the whole
        // stream back on the original's for another four hundred frames.
        // Item 109's bird step takes 1256 → 1373 and carries both with it:
        // **191,173** of 192,504 and the first parting to **1381**, `8/3`
        // a step off the original's point again. Item 110's region
        // fallback takes 1373 → 1570 and both again: **191,876** and the
        // first parting to **1658**, `8/0` seventeen units short.
        assert!(
            first_bad_frame >= 1658 && agree >= 191_876 && seen == 192_504,
            "run39's gaia positions fell: {agree} of {seen}, first {first_bad:?}"
        );
    }

    /// **A herd's wander centre jitters about its home cell, and does not
    /// walk** (2026-08-31, item 112) — the mechanic behind Great Lakes'
    /// word going 986 → 1372, asserted against the record the word's own
    /// count cannot see.
    ///
    /// `Herd::process@00741760` takes two draws and writes
    /// `wx = cx − 1 + p % 3`, `wy = cy − 1 + p % 3`: it **reads `cx`/`cy`
    /// and writes `wx`/`wy`** — `HerdData +0x0/+0x4` into `+0x8/+0xc` — so
    /// the wander centre is never more than one cell from the home cell,
    /// however many times the herd is processed. This crate read the
    /// destination as the source and random-walked it. The two agree until
    /// a herd's *second* walk, and only one herd in thirteen gets a second
    /// inside run33's 1,850 frames: `(frame >> 6) % 13`, so herd 0 walks on
    /// frames 0 and 832 and nothing else does twice.
    ///
    /// What one cell is worth is `Animal::do_idle`'s far wander
    /// (`docs/ANIM.md` §7), whose whole ring is drawn about
    /// `herd_centre(cx, cy, wx, wy)`. Herd 0's `wy` ends frame 832 at 34
    /// here against the original's **33**, which moves that centre 240
    /// units south — and on frame 981 the sheep `8/3` is sent to
    /// `(17688, 26616)` where the original sends it to `(17688, 26328)`,
    /// a spot our ring does not contain at any of its nine radii.
    ///
    /// Five frames later the difference is a *draw*: both animals are
    /// refused by the same neighbour, but ours has walked five steps of
    /// `(+15, −5)` against the original's five of `(+12, −10)` and reaches
    /// the refusal one frame early, spending `Guy::set_anim+0x97a <
    /// Unit::move_step+0x823` on 986 where the original spends it on 987.
    /// That one frame was the whole of the word's parting at 986.
    ///
    /// Both halves are here: `8/3`'s walk against the dump step for step,
    /// and the whole capture's animal-frames. Made to fail by putting
    /// `wx`/`wy` back on the right-hand side, which walks `8/3` to the
    /// wrong point on frame 982 and never reaches the original's.
    #[test]
    fn run33_s_herd_centre_jitters_about_its_home_cell() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run33-longtrace.txt") else {
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
        let mut walk: Vec<(i64, i32, i32)> = Vec::new();
        let (mut agree, mut seen) = (0usize, 0usize);
        let mut first_bad: Option<String> = None;
        let mut first_bad_frame: i64 = i64::MAX;
        let mut last = 0i64;
        for f in log.frame_states() {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in f.units.iter().filter(|u| u.who == 8) {
                let Some(unit) = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(8, o))
                else {
                    continue;
                };
                let ours = built.sim.units[unit].pos;
                if u.o == 3 && (980..=990).contains(&f.n) {
                    walk.push((f.n, ours.x, ours.y));
                }
                seen += 1;
                if i64::from(ours.x) == u.pos.x && i64::from(ours.y) == u.pos.y {
                    agree += 1;
                } else if first_bad.is_none() {
                    first_bad_frame = f.n;
                    first_bad = Some(format!(
                        "frame {} 8/{}: ours ({}, {}) theirs ({}, {})",
                        f.n, u.o, ours.x, ours.y, u.pos.x, u.pos.y
                    ));
                }
            }
        }
        eprintln!(
            "run33 gaia: {agree} of {seen} dumped animal-frames on the original's point, \
             first {first_bad:?}"
        );
        // The coin comes up on 981, the first step lands on 982, and the
        // fifth is refused: five steps of `(+12, −10)` toward
        // `(17688, 26328)`, then the point held. The frames are the dump's
        // own `FRAME n` blocks, which carry engine frame `n − 1`'s end.
        assert_eq!(
            walk,
            vec![
                (980, 17112, 26808),
                (981, 17112, 26808),
                (982, 17112, 26808),
                (983, 17124, 26798),
                (984, 17136, 26788),
                (985, 17148, 26778),
                (986, 17160, 26768),
                (987, 17172, 26758),
                (988, 17172, 26758),
                (989, 17172, 26758),
                (990, 17172, 26758),
            ],
            "run33's `8/3` walks the original's ground and is refused on 987"
        );
        // The whole-capture floor, read the same way run39's is: the first
        // animal-frame that is not the original's is the number with
        // meaning, and the count beside it is a floor on this word rather
        // than a score across words.
        //
        // 2026-08-31, item 114: 73,608 of 74,040 with a first bad frame of
        // 1420 -> **74,040 of 74,040, and there is no first bad frame**.
        // Great Lakes' gaia is the original's on every dumped animal-frame
        // of the whole capture, which follows from the word running past
        // its length: the residue at 1420 was a herd re-rolling off a
        // stream that had been nobody's since 1372. Half of queue item
        // 105/45 is closed on this map; run39's stands.
        assert!(
            first_bad_frame == i64::MAX && agree == 74_040 && seen == 74_040,
            "run33's gaia positions fell: {agree} of {seen}, first {first_bad:?}"
        );
    }

    /// **A scout re-aims its walk at a goody box it has seen**
    /// (2026-08-31, item 106) — the mechanic behind East Indies' word going
    /// 879 → 1256, asserted against the record the word's own count cannot
    /// see.
    ///
    /// `Unit::do_explore_to@005f24a0` is not `do_move`. One frame in
    /// fifteen, phased by `o`, a captain still walking the same
    /// `EXPLORE_TO` runs `Unit::find_goody_box@005f2540`: a 49-cell sweep
    /// in `move_x`/`move_y` order for a cell of its own region carrying
    /// `WData.flags & 0x8000`, and the first one it accepts is re-issued
    /// as an `EXPLORE_TO` to that cell's **centre** by
    /// `Unit::get_goody_box@005f7690`. None of it spends a draw.
    ///
    /// run39's scout `1/0` is inside the sweep's range of `(45, 49)` from
    /// frame 796 on, and the original does not re-aim until **825**. The
    /// gate that holds it is not the cell's `WorldData::was_seen`, which
    /// the box's own borders answer yes to from frame 0, but the **item's**
    /// `ItemData::is_seen` — the bare accumulated fog, with no
    /// ally-territory shortcut — which the scout's own line of sight does
    /// not reach until it is two cells out. `docs/GOODY.md` §7.2.
    ///
    /// What the frame's draw count cannot see, and this does — every one
    /// of these is a field of run39's own `UNITDATA`:
    ///
    /// - **`FRAME 797`**: `orders_x/y 35064/37368` and a path of three,
    ///   `(35040, 37344)`, `(35064, 38904)`, `(35064, 39672)`. That is
    ///   `think_scout`'s target and it is untouched through 824.
    /// - **`FRAME 826`**: `orders_x/y 34968/38040`, a path of **one** at
    ///   `(34944, 38016)` — `45 × 0x300 + 0x180`, `49 × 0x300 + 0x180`,
    ///   the box's cell centre — and an order list still holding **one**
    ///   order. A unit-level `QUEUE_FIRST` would leave two; the group's
    ///   halts and re-issues as `QUEUE_NEW` (`docs/GROUPS.md` §17).
    /// - **`FRAME 879`**: the walk is over, the list is empty, and
    ///   `orders_x/y` is `34944/38016`. That idle frame is the sixteen
    ///   draws the word had been short: two `Unit::set_anim` stands and
    ///   `think_scout`'s six ring pairs with two cell draws.
    /// - **`FRAME 880`**: the re-think's own answer, `orders_x/y
    ///   31224/39672` over four legs from `(31200, 39648)`.
    ///
    /// Made to fail by dropping the item gate, which fires the sweep on
    /// **796** instead — the frame `think_scout` itself runs on — and
    /// parts the word there.
    #[test]
    fn a_scout_re_aims_its_walk_at_a_goody_box_it_has_seen() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // The scout's order, its path stack and how many orders it holds —
        // `orders_x`/`orders_y`, `STACK<PathData>` and the order list's
        // own length, in the dump's order.
        let state = |b: &Built| {
            let u = b.sim.unit_by_o(1, 0).expect("run39 dumps player 1's `1/0`");
            let un = &b.sim.units[u];
            (
                (un.orders_pos.x, un.orders_pos.y),
                un.path.iter().map(|p| (p.to.x, p.to.y)).collect::<Vec<_>>(),
                un.orders.len(),
            )
        };

        // `Built::tick` stamps the frame it is about to run, so 797 ticks
        // leave `Sim::frame` on 797 and the state is `FRAME 797`'s.
        for _ in 0..797 {
            built.tick();
        }
        assert_eq!(built.sim.frame, 797);
        let think_scout_target = (
            (35_064, 37_368),
            vec![(35_040, 37_344), (35_064, 38_904), (35_064, 39_672)],
            1,
        );
        assert_eq!(
            state(&built),
            think_scout_target,
            "`FRAME 797`: frame 796's `think_scout` target and its three legs"
        );

        // Through 824 the sweep finds the box in range and refuses it:
        // 810 is a fifteenth frame and the item is still dark there.
        for f in 797..825 {
            built.tick();
            assert_eq!(
                built.sim.frame,
                f + 1,
                "the tick counter, so the frames below name themselves"
            );
            assert_eq!(
                state(&built).0,
                think_scout_target.0,
                "`FRAME {}`: the box is in range and its item is not yet seen",
                f + 1
            );
        }

        // Frame 825 is the retarget, and the box's cell centre is where it
        // aims.
        built.tick();
        assert_eq!(built.sim.frame, 826);
        assert_eq!(
            state(&built),
            ((34_968, 38_040), vec![(34_944, 38_016)], 1),
            "`FRAME 826`: the box's cell centre, one leg, and **one** order"
        );

        // …and it costs nothing: every frame from 796 to 878 spends the
        // original's draws in the original's order.
        for f in 826..879 {
            built.tick();
            assert_eq!(built.sim.frame, f + 1);
        }
        let ours: Vec<(i64, Vec<String>)> = built
            .frame_sites
            .iter()
            .filter(|(f, _)| (796..879).contains(f))
            .cloned()
            .collect();
        let bad: Vec<i64> = ours
            .iter()
            .filter(|(f, s)| *s != tr.labels(*f))
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(
            bad,
            Vec::<i64>::new(),
            "the sweep spends no draw, so 796…878 stay the original's"
        );

        // Frame 879: the arrival, and the re-think it lets happen.
        assert_eq!(built.sim.frame, 879);
        assert_eq!(
            state(&built),
            ((34_944, 38_016), vec![], 0),
            "`FRAME 879`: the walk is over and the list is empty"
        );
        built.tick();
        let f879 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 879)
            .map(|(_, s)| s.clone())
            .expect("frame 879's sites");
        assert_eq!(
            f879,
            tr.labels(879),
            "frame 879 is two stands, six ring pairs, two cell draws and five farms"
        );
        assert_eq!(
            f879.iter()
                .filter(|s| *s == sim::scout::SITE_ROTATION)
                .count(),
            6,
            "an AI scout skips the even rings, so `max_ring` 12 walks six"
        );
        assert_eq!(
            state(&built),
            (
                (31_224, 39_672),
                vec![
                    (31_200, 39_648),
                    (32_760, 40_440),
                    (33_528, 39_672),
                    (34_296, 38_904)
                ],
                1,
            ),
            "`FRAME 880`: the re-think's own target, over four legs"
        );
    }

    /// **A goody box draws once for every good its finder can gather**
    /// (2026-08-31, item 104) — the mechanic behind East Indies' word going
    /// 867 → 879, asserted where the word's own count cannot see it.
    ///
    /// `Unit::set_new_location@005f8d20+0x3cc` calls
    /// `Unit::explore_goody@005f9780` whenever a unit that is not an animal,
    /// not a placement ghost and of a land type enters a **new cell** whose
    /// `WData` first `short` is negative — bit `0x8000`, `GOODY`. run39's
    /// world carries seven such cells, which is the `WORLD` record's own
    /// `goodies 7`, and on frame 867 player 1's scout `1/0` walks south out
    /// of cell `(45, 50)` into `(45, 49)`, one of them. The original spends
    /// **three** draws there and this crate spent none.
    ///
    /// Three, not six: the lottery walks goods 0…5, skips `KNOWLEDGE`
    /// outright and skips anything `LeaderData::type_avail(good, 1)` does
    /// not call available — and in the Ancient age that is knowledge, metal
    /// and oil, so the candidates are food, timber and wealth. **The draw
    /// count is the candidate count**, which is what makes this frame a test
    /// of `type_avail` over the goods rather than of the lottery.
    ///
    /// What the frame's count cannot see, and this does:
    ///
    /// - **The cell.** The bit at `(45, 49)` is set through frame 866 and
    ///   clear after 867, and the other six are untouched — so no unit can
    ///   take the same ruins twice, and none of the other six has been
    ///   consumed by a walk that merely passed nearby.
    /// - **The pile.** `epoch[3] * GOODY_BOX_AGE + GOODY_BOX`, and the
    ///   `epoch` is the **Science** library level rather than the age
    ///   (`docs/GOODY.md` §3). Player 1 is on Science 1 by 867, so the pile
    ///   is `1 × 25 + 25 = 50` where the age reading would pay 25.
    /// - **The good.** Wealth, and it goes to the finder's bucket and to
    ///   `goody_box_resources`, the score counter `Leader::reset_score`
    ///   zeroes beside `goody_box_techs` and `goody_box_units`.
    ///
    /// Made to fail by dropping the availability guard, which spends five
    /// draws and parts the word back at 867. **The pile is the half no dump
    /// on disk can check**: run39's leader detail is written once, at start,
    /// where every `bucket` and every `epoch_get(scan)` is still its opening
    /// value. `docs/GOODY.md` §6 names the capture that would settle it.
    #[test]
    fn a_goody_box_draws_once_for_each_good_its_finder_can_gather() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // The seven, from the dump's own cells — `goodies 7`.
        let goody_cells = |s: &sim::Sim| -> Vec<(i32, i32)> {
            let mut out = Vec::new();
            for y in 0..s.world.height() {
                for x in 0..s.world.width() {
                    let c = sim::world::Cell::new(x, y);
                    if s.world.cell_data(c).flags & sim::world::cell::GOODY != 0 {
                        out.push((x, y));
                    }
                }
            }
            out
        };
        assert_eq!(
            goody_cells(&built.sim),
            vec![
                (52, 5),
                (22, 22),
                (53, 26),
                (8, 28),
                (36, 31),
                (38, 34),
                (45, 49),
            ],
            "run39's `WORLD` record carries `goodies 7`, and these are they"
        );

        // `Built::tick` stamps the frame it is *about* to run, so 867 ticks
        // leave frames 0…866 behind and `Sim::frame` on 867.
        for _ in 0..867 {
            built.tick();
        }
        assert_eq!(built.sim.frame, 867);
        let before = built.sim.ledgers[1].bucket;
        assert_eq!(
            goody_cells(&built.sim).len(),
            7,
            "nothing has taken a goody through frame 866"
        );
        assert_eq!(built.sim.ledgers[1].goody_box_resources, 0);

        built.tick();
        // The frame itself, against the trace: three of the lottery's draws
        // and then the six the farms spend.
        let ours = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 867)
            .map(|(_, s)| s.clone())
            .expect("frame 867's sites");
        assert_eq!(
            ours,
            tr.labels(867),
            "frame 867 is the goody's three draws and the farms' six"
        );
        assert_eq!(
            ours.iter().filter(|s| *s == sim::goody::SITE_PICK).count(),
            3,
            "food, timber and wealth are the candidates; knowledge, metal and oil are not"
        );

        // The cell is spent and its six neighbours in the list are not.
        assert_eq!(
            goody_cells(&built.sim),
            vec![(52, 5), (22, 22), (53, 26), (8, 28), (36, 31), (38, 34),],
            "`(45, 49)` is taken and only `(45, 49)`"
        );
        // And the finder is where run39's own `FRAME 868` dump puts it.
        let u = built
            .sim
            .unit_by_o(1, 0)
            .expect("run39 dumps player 1's `1/0`");
        assert_eq!(built.sim.units[u].pos.cell(), sim::world::Cell::new(45, 49));

        // The pile: fifty wealth, and nothing anywhere else.
        let after = built.sim.ledgers[1].bucket;
        let moved: Vec<(usize, i32)> = (0..6)
            .filter(|&g| after[g] != before[g])
            .map(|g| (g, after[g] - before[g]))
            .collect();
        assert_eq!(
            moved,
            vec![(2, 50)],
            "one good, `epoch[3] × GOODY_BOX_AGE + GOODY_BOX` of it, and player 1 \
             is on Science 1"
        );
        assert_eq!(built.sim.ledgers[1].goody_box_resources, 50);
    }

    /// **A building's own line of sight, and the scout's path that reads
    /// it** (2026-08-31, item 99) — the mechanic behind the word's
    /// 413 → 576, asserted where a count cannot see it.
    ///
    /// run39's AI finishes its sixth farm on frame **219** at cell
    /// `(54, 51)`; `Wall::update_los` gives it `mylos 8` (`LOS 6` plus half
    /// its `X_SIZE 4`), so `Build::activate`'s closing `update_seen(0)`
    /// lights a radius-4 disc that reaches the three cells of column 56
    /// beside it — and stops short of `(56, 54)`, which
    /// `Unit::think_scout` still needs dark to pick as a target.
    ///
    /// Nineteen frames later the scout re-targets, and the path it plans is
    /// the whole point: with those cells dark a scout's `EXPLORE_TO` search
    /// prices them at a base of **8** against a seen cell's `0x400` and
    /// runs straight through them; with them lit it walks the seen column
    /// 55, which is what the original's dumped stack says it does. Both
    /// halves are here, and both were made to fail first — by dropping
    /// `update_seen_build` from `Sim::activate`, which puts the three cells
    /// back in the dark and the path back on column 56.
    #[test]
    fn run39_s_sixth_farm_lights_the_cells_its_scout_then_paths_around() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // The three cells the farm lights, and the one it must not: their
        // `2c + 1` half-cells, which is what both readers sample.
        let lit = [(56, 50), (56, 51), (56, 52)];
        let seen = |b: &Built, (cx, cy): (i32, i32)| {
            b.sim
                .world
                .seen2(2 * cx + 1, 2 * cy + 1)
                .is_some_and(|v| v & 2 != 0)
        };
        for _ in 0..218 {
            built.tick();
        }
        assert!(
            lit.iter().all(|&c| !seen(&built, c)),
            "column 56 is lit before the farm finishes"
        );
        for _ in 218..220 {
            built.tick();
        }
        assert!(
            lit.iter().all(|&c| seen(&built, c)),
            "the finished farm did not light column 56"
        );
        assert!(
            !seen(&built, (56, 54)),
            "the disc reached (56, 54), which think_scout needs dark"
        );

        // And the path the scout plans on frame 238, entry for entry
        // against the original's own stack. Its middle three are the whole
        // of the disagreement: column 55 where this crate ran column 56.
        for _ in 220..=238 {
            built.tick();
        }
        let theirs: Vec<(i64, i64, i64, i64)> = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 239)
            .expect("the dump's frame 239 is sim-frame 238")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("the AI scout")
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs,
            vec![
                (43488, 41952, 0, 1),
                (42744, 41208, 384, 0),
                (42744, 40440, 384, 0),
                (42744, 39672, 384, 0),
                (42744, 38904, 384, 0),
                (42744, 38136, 384, 0),
                (41976, 37368, 384, 0),
            ],
            "the original's own seven entries"
        );
        let v = built
            .units
            .iter()
            .find(|l| l.who == 1 && l.o == 0)
            .map(|l| l.unit)
            .expect("the AI scout");
        let ours: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        assert_eq!(ours, theirs, "the scout's stack on the frame it re-targets");
    }

    /// **A bird that lands, and the sixty draws it spends looking**
    /// (2026-08-31, item 100) — the mechanic behind the second map's word
    /// going 576 -> 645, asserted at the frame rather than as a total.
    ///
    /// `Animal::think_bird@005d79e0`'s third draw is `rnd % spell_time`,
    /// and `== 100` or `> 799` opens the landing search: thirty rounds
    /// over the cell list of the region the bird's patrol point sits in,
    /// two draws a round — the cell (`+0x2aa`, skipped for a region of one)
    /// and a `% 0x32 + 1` score (`+0x2d3`). The modulus of the roll that
    /// opens it *is* the counter, so it cannot fire before a bird has
    /// flown a hundred think-cycles, which is why `docs/SYNC.md` §3.9
    /// recorded the branch as unreached and left its draws unmodelled.
    ///
    /// run39 reaches it. Gaia's `9/8` fires on frame **576**, the frame
    /// the AI also founds its second city on, and the sixty draws are more
    /// than half of that frame's 118 — this crate spent 56. What the check
    /// asserts is the frame's whole draw sequence against the original's,
    /// which is the only oracle there is: the search's *score* is computed
    /// and never compared (`if (-1 < score)` cannot fail), so the cell it
    /// settles on is the thirtieth sampled and no dump prints owner 9
    /// anyway. Made to fail first by dropping the call from `think_bird`,
    /// which puts the frame back at 56 draws against 118.
    #[test]
    fn run39_s_bird_lands_on_576_and_spends_the_search_s_sixty() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..=576 {
            built.tick();
        }

        // The landing itself: one bird, on the original's frame.
        assert_eq!(
            built.sim.gaia.bird_landings,
            vec![(576, 8)],
            "gaia's `9/8` is the only bird to land by 576, and it lands there"
        );

        // The frame's sixty, in the original's own alternation — thirty
        // pairs, never a `+0x2aa` skipped, so the region has more than one
        // cell in it.
        let ours = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 576)
            .map(|(_, v)| v.clone())
            .expect("frame 576 drew");
        let search: Vec<&str> = ours
            .iter()
            .map(String::as_str)
            .filter(|s| {
                *s == sim::gaia::SITE_BIRD_SEARCH_CELL || *s == sim::gaia::SITE_BIRD_SEARCH_SCORE
            })
            .collect();
        let want: Vec<&str> = (0..sim::gaia::BIRD_SEARCH_ROUNDS)
            .flat_map(|_| {
                [
                    sim::gaia::SITE_BIRD_SEARCH_CELL,
                    sim::gaia::SITE_BIRD_SEARCH_SCORE,
                ]
            })
            .collect();
        assert_eq!(search, want, "thirty rounds of two, in order");

        // And the frame whole, which is what says the sixty fall in the
        // right *place* as well as in the right number.
        assert_eq!(ours.len(), 118, "frame 576's draw count");
        assert_eq!(ours, tr.labels(576), "frame 576, draw for draw");
    }

    /// **A bird's counter takes one step more than its flight** — every
    /// landing frame of a traced game, against the trace's own.
    ///
    /// The landing roll is `rnd % spell_time`, so the *counter* is the
    /// only state behind it and its value is unobservable: no dump prints
    /// owner 9 at all ([`run39_s_bird_lands_on_576_and_spends_the_search_s_sixty`]).
    /// What is observable is the frame a search fires on, thirty `+0x2aa`
    /// draws at a time, and a counter one step out puts a landing on a
    /// frame the original does not have — which is what East Indies'
    /// word saw at **1256**.
    ///
    /// The step is `Unit::do_air_patrol@005ea620`'s own tail. After
    /// `do_air_physics` returns — 1 for a wild bird on every path it
    /// takes — the caller branches on `vtable+0x30`,
    /// `SubObjectData::is_animal` (`docs/SYNC.md` §3.14; the map folds
    /// all three `Animal` vtables' slot onto `Buffer::is_pending_load`,
    /// `return 1`), and the animal arm is
    /// `else if (spell_time == 0) spell_time = 1`. `think_bird`
    /// increments the counter on **every** frame it runs, so it is only
    /// ever 0 there on a frame the landing search has just zeroed it:
    /// the branch is one extra step per landing and nothing else.
    ///
    /// run39's original lands on 576, 944 twice over, 1016, 1144 and
    /// 1368, and this simulation now lands on exactly those. The list is
    /// cut at the word (`first_part`) because past it the draws are
    /// nobody's — the original's own later landings, 1376 onward, are
    /// rolls this stream never sees. Made to fail by dropping the branch,
    /// which puts a seventh landing on **1256** and takes the word back
    /// there with it.
    #[test]
    fn a_bird_s_landing_frames_are_the_trace_s_own() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        for _ in 0..last {
            built.tick();
        }
        // Only the frames this stream is still the original's speak to a
        // landing: past the word both sides roll different numbers.
        let end = built
            .frame_sites
            .iter()
            .find(|(f, ours)| **ours != tr.labels(*f))
            .map_or(last, |(f, _)| *f);
        // The original's own landings: one for every thirty `+0x2aa`
        // draws a frame spends, and run39's frame 944 spends sixty.
        let rounds = sim::gaia::BIRD_SEARCH_ROUNDS as usize;
        let mut theirs: Vec<i64> = Vec::new();
        for (f, _) in &tr.frames {
            if *f >= end {
                break;
            }
            let n = tr
                .labels(*f)
                .iter()
                .filter(|l| l.as_str() == sim::gaia::SITE_BIRD_SEARCH_CELL)
                .count();
            for _ in 0..n / rounds {
                theirs.push(*f);
            }
        }
        let ours: Vec<i64> = built
            .sim
            .gaia
            .bird_landings
            .iter()
            .map(|(f, _)| *f)
            .filter(|f| *f < end)
            .collect();
        // 2026-09-01: four more (1736, 1776, 1784, 1800) arrived with item
        // 125 — not because a bird changed, but because `end` did. The
        // list runs to wherever the two streams still agree, and East
        // Indies' now agrees for the whole capture.
        assert_eq!(
            ours,
            vec![
                576, 944, 944, 1016, 1144, 1368, 1376, 1736, 1776, 1784, 1800
            ],
            "run39's landings up to the word at {end}"
        );
        assert_eq!(ours, theirs, "run39's landing frames, the trace's own");
    }

    /// **A pasture herder walks only on its own 256-frame phase**, and it
    /// is the rule of `docs/SYNC.md` §3.15 against run39's own record.
    ///
    /// `Unit::do_gather@005ef2a0:5efd77` takes the whole of the function
    /// when `FarmsData::get_farm_type` answers 1: the citizen shows the
    /// sow animation, and unless `(o · 7 + frame + who) % 256` is zero it
    /// returns having drawn nothing. A pasture is the one farm
    /// `Farms::inc_time` skips, so no cell under it ripens on the clock —
    /// which is why the crop switch below is not merely the wrong branch
    /// but a branch whose *trigger* never comes on time. This crate ran
    /// the herder through it, and the farmer's own `Farms::grow` alone
    /// carried its cell to `RIPE_ADDS` on the two hundredth frame instead
    /// of the hundredth: two draws on East Indies' frame 201, where the
    /// original spends none.
    ///
    /// Three things are asserted, and each of them can fail:
    ///
    /// - **The record.** Over run39's 1,850 frames the herder's dumped
    ///   position changes on **42** of them, in **four** runs, and every
    ///   run's first frame is `phase + 2` — the order is issued on the
    ///   phase frame and the first step lands the frame after. Three of
    ///   the seven phase frames move it nowhere, because the inner 2 × 2
    ///   holds four tiles and the roll may name the one it stands on.
    /// - **The trace.** All **seven** phase frames of the capture spend
    ///   the `GameAccess::rnd+0x20 < Unit::do_job+0x67` pair, and the pair
    ///   is two draws rather than one because `x_size / 2` is 2 and
    ///   `GameAccess::rnd` only skips its draw at 1 or less.
    /// - **The port.** This crate's herder walks the original's first run
    ///   frame for frame — sim-frames 235 to 242 — and takes **no** step
    ///   on any frame outside a phase's window, over the whole capture.
    ///   Under the crop switch it walks from 202 and the first assertion
    ///   fails at once.
    ///
    /// Great Lakes cannot check this: run33's AI built seven farms and no
    /// pasture, which is why the residue only ever showed on the second
    /// map.
    #[test]
    fn a_pasture_herder_walks_only_on_its_own_256_frame_phase() {
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
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        // The pasture, and the one citizen registered at it. Named rather
        // than searched for, so a capture whose setup changes says so.
        let pasture = (0..built.sim.buildings.len())
            .find(|&b| built.sim.buildings[b].farm.farm_type & sim::farms::ANIMAL_FARM != 0)
            .expect("run39's AI built one pasture");
        let herder = *built.sim.buildings[pasture]
            .gatherers
            .first()
            .expect("and put a citizen on it");
        let (who, o) = (
            i64::from(built.sim.units[herder].owner),
            i64::from(built.sim.units[herder].index),
        );
        assert_eq!(
            (who, o, built.sim.buildings[pasture].index),
            (1, 3, 2002),
            "run39's pasture is the AI's `1/2002` and its herder `1/3`"
        );
        // `(o · 7 + frame + who) % 256 == 0`, in sim-frames.
        let on_phase = |f: i64| (o * 7 + f + who) % 0x100 == 0;

        // **The record.** `FRAME n` is the state at the end of sim-frame
        // `n − 1`, so a position that differs between `FRAME n − 1` and
        // `FRAME n` is a step taken on sim-frame `n − 1`.
        let states = log.frame_states();
        let theirs: Vec<(i64, crate::gamelog::Pos)> = states
            .iter()
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == who && u.o == o)?;
                Some((f.n - 1, u.pos))
            })
            .collect();
        assert_eq!(theirs.len(), 1851, "the herder is in every frame's dump");
        let moved: Vec<i64> = theirs
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| w[1].0)
            .collect();
        assert_eq!(moved.len(), 42, "the herder's steps: {moved:?}");
        let starts: Vec<i64> = moved
            .iter()
            .copied()
            .filter(|f| !moved.contains(&(f - 1)))
            .collect();
        assert_eq!(
            starts,
            vec![235, 747, 1003, 1515],
            "four walks, and each begins the frame after a phase frame"
        );
        for f in &starts {
            assert!(
                on_phase(f - 1),
                "the walk that starts on {f} was ordered on {}, which is not a phase frame",
                f - 1
            );
        }

        // **The trace.** Every phase frame of the capture spends the pair.
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        let phases: Vec<i64> = (0..last).filter(|&f| on_phase(f)).collect();
        assert_eq!(
            phases,
            vec![234, 490, 746, 1002, 1258, 1514, 1770],
            "seven phase frames in 1,850"
        );
        for f in &phases {
            let n = tr
                .labels(*f)
                .iter()
                .filter(|l| *l == sim::orders::SITE_FARM_CELL)
                .count();
            assert!(
                n >= 2,
                "the original spends {n} `{}` draws on phase frame {f}, wanted the pair",
                sim::orders::SITE_FARM_CELL
            );
        }

        // **The port.** Our herder's own steps, over the whole capture.
        let mut ours: Vec<(i64, sim::Pos)> = Vec::new();
        for _ in 0..last {
            let f = built.sim.frame;
            built.tick();
            ours.push((f, built.sim.units[herder].pos));
        }
        let ours_moved: Vec<i64> = ours
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| w[1].0)
            .collect();
        let ours_starts: Vec<i64> = ours_moved
            .iter()
            .copied()
            .filter(|f| !ours_moved.contains(&(f - 1)))
            .collect();
        for f in &ours_starts {
            assert!(
                on_phase(f - 1),
                "this crate's herder starts walking on {f}, off its phase: {ours_moved:?}"
            );
        }
        // The first walk is the original's, frame for frame. Past the
        // word's own parting at 219 the rolls are on a stream that is
        // nobody's, so which of the later phases moves it is not a fact
        // about this simulation — but *that* it only ever moves on one is.
        let first_walk: Vec<i64> = moved.iter().copied().take_while(|&f| f < 300).collect();
        assert_eq!(
            ours_moved
                .iter()
                .copied()
                .take_while(|&f| f < 300)
                .collect::<Vec<_>>(),
            first_walk,
            "the herder's first walk is the original's"
        );
    }
}
