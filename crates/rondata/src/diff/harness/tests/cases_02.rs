pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run97_s_window_orders_are_the_original_s() {
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
        //
        // **Item 471 took five of the eight off it** — `1/27`, `1/28`,
        // `1/29`, `1/41` and `1/42`, the five of §12's probe that are not
        // its leader. Their whole residue was the one wrong number their
        // group move was laid out from: the formation angle, which this
        // crate read as nought because it asked where the leader *stands*
        // for a move queued behind an attack rather than where it ends up
        // (`docs/ORDERS.md` §17). `1/40` stays because its own leg is the
        // one the plan is made from, and what is left of it is the
        // pathfinder's tie-break ([`PROBE_PLAN_PARTED`]).
        let mut units: Vec<(i64, i64)> = scoring.iter().map(|d| (d.who, d.o)).collect();
        units.sort_unstable();
        units.dedup();
        //
        // **Item 539 took `1/33` off it**, the bowman whose soft flags
        // the fast path's stride over an empty world cell had parted
        // (`docs/COLLISION.md` §4.2); the word moved up on the same item,
        // which only adds rows below it, so the fall is the fix's.
        //
        // **Item 899 took `1/40` off it**: its walk home is planned while its
        // current order is the `ATTACK`, which the original plans without
        // the `army` mode (`docs/PATHFINDER.md` §29), so its stack is the
        // original's whole and so is every order it walks.
        //
        // **Item 1115 took `1/23` off it**, the caravan: its trade order
        // carries no bit 4 and its leg faces its bearing
        // (`docs/CARAVAN.md` §11.3). The residue is empty.
        assert_eq!(
            units,
            Vec::<(i64, i64)>::new(),
            "run97's order residue reached a unit item 368 did not leave it on"
        );
        // The count is floored beneath the set, so a known unit growing
        // a new kind of row is caught too. It may only come down, and since
        // item 1115 it is at zero, so the floor is exact.
        assert_eq!(
            scoring.len(),
            ORDER_RESIDUE_RUN97,
            "run97's order residue grew past {ORDER_RESIDUE_RUN97}: {} rows, first {:?}",
            scoring.len(),
            scoring.first()
        );
    }

    pub(in super::super) fn great_lakes_8186_plans_the_probe_s_route_the_original_s_way() {
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
            // - ~~**a whole second leg** under `1/40`'s: run97 block 8187
            //   gives it 92 entries where this crate gives 47, and the
            //   46 extra sit *beneath* — a queued move this crate does
            //   not hold.~~ **It holds it since item 471** — 94 entries
            //   against 94 — because the `QUEUE_LAST` walk home is now
            //   planned from where the leader *ends up* rather than from
            //   its feet (`docs/ORDERS.md` §17), which is also what makes
            //   the plan a real leg instead of a step of seventeen units.
            //   So the comparison below covers twice what it did, and the
            //   half it gained is [`PROBE_PLAN_PARTED`].
            //
            // §22.4 names the first as a successor.
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
            let parted: Vec<usize> = (0..k)
                .filter(|&i| ours_route[i] != their_route[i])
                .collect();
            let want: &[usize] = PROBE_PLAN_PARTED
                .iter()
                .find(|&&(w, x, _)| (w, x) == (who, o))
                .map_or(&[], |&(_, _, v)| v);
            assert_eq!(
                parted,
                want,
                "{who}/{o}'s world plan is not run97 block 8187's, waypoint for \
                 waypoint: ours {:?} theirs {:?}",
                parted.iter().map(|&i| ours_route[i]).collect::<Vec<_>>(),
                parted.iter().map(|&i| their_route[i]).collect::<Vec<_>>(),
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
            // **And these agree everywhere, including where the route does
            // not.** `1/40`'s twenty-two parted waypoints carry the
            // original's own `tolerance` and `flags` on every one of them,
            // which is worth an assertion of its own: a leg planned to the
            // wrong cell but stamped right is a **cost** answer, where a leg
            // stamped wrong would be a different call. So this list is empty
            // for all six and is deliberately **not** [`PROBE_PLAN_PARTED`]
            // — reusing that expectation here is what made this test red on
            // item 471's first gate.
            let rest_parted: Vec<usize> = (0..ours_rest.len())
                .filter(|&i| ours_rest[i] != their_rest[i])
                .collect();
            assert!(
                rest_parted.is_empty(),
                "{who}/{o}'s waypoint tolerances and flags are not run97 block \
                 8187's at {rest_parted:?}: ours {:?} theirs {:?}",
                rest_parted
                    .iter()
                    .map(|&i| ours_rest[i])
                    .collect::<Vec<_>>(),
                rest_parted
                    .iter()
                    .map(|&i| their_rest[i])
                    .collect::<Vec<_>>(),
            );
            checked += 1;
        }
        assert_eq!(checked, 6, "the probe's six");
    }

    pub(in super::super) fn great_lakes_s_word_draws_are_a_road_search_and_8186_spends_none() {
        let Some(trace) = trace("rontrace-run53.log") else {
            eprintln!("skipping: no run53 trace (set RON_GAMELOG_DIR)");
            return;
        };
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 23_000,
            "run53's traced length is {last}, wanted 23,000+ — the wrong file"
        );
        let road = |f: i64| -> Vec<crate::trace::Draw> {
            trace
                .frame_draws(f)
                .into_iter()
                .filter(|d| d.site == ROAD_COST)
                .collect()
        };
        // **Every frame in the word's neighbourhood that prices a road
        // node**, and 8186 is not one of them. The window opens on the
        // probe's own planning frame and closes past the word's second
        // half.
        let tail = GREAT_LAKES_REGEN_SEARCH + 13;
        let schedule: Vec<(i64, usize)> = (PROBE_PLAN_FRAME..=tail)
            .map(|f| (f, road(f).len()))
            .filter(|(_, n)| *n > 0)
            .collect();
        eprintln!("  run53 road searches in [{PROBE_PLAN_FRAME}, {tail}]: {schedule:?}");
        assert_eq!(
            schedule,
            vec![
                (8_235, 196),
                (8_236, 217),
                (8_241, 308),
                (8_541, 127),
                (8_543, 228),
                (10_234, 198),
                (10_235, 73),
            ],
            "run53's road searches between the probe's plan and the word are not the seven they are"
        );
        assert!(
            road(PROBE_PLAN_FRAME).is_empty(),
            "{PROBE_PLAN_FRAME} — the frame the probe's world path is planned on — spends a \
             road-cost draw, so `calc_road_cost` could be what breaks that path's ties after \
             all (`docs/ORDERS.md` §18)"
        );
        // **The chain, on every one of the word's own 198.** A count says
        // the function ran; the `ebp` chain says which search ran it, and
        // that is the whole of the falsification.
        let word = road(GREAT_LAKES_REGEN_SEARCH);
        assert_eq!(word.len(), 198, "the replan search's road-cost draws");
        // Folded before it is asserted: 198 copies of one chain is a
        // failure nobody reads, and the claim is about which chains
        // appear rather than how many draws each took.
        let mut strays: Vec<String> = word
            .iter()
            .filter(|d| !d.up.contains(&ROAD_COST_UP) || !d.up.contains(&ROAD_COST_UP2))
            .map(|d| format!("{:x} < {:x}", d.up[0], d.up[1]))
            .collect();
        strays.sort_unstable();
        strays.dedup();
        assert_eq!(
            strays,
            Vec::<String>::new(),
            "a road-cost draw on the word reached `calc_road_cost` from somewhere other than \
             `astar_caravan_road+0x52b < find_road+0x3a8`"
        );
        // **And it is the word's own parting draw**: index 4 of the
        // frame, the first the two streams do not share.
        let all = trace.frame_draws(GREAT_LAKES_REGEN_SEARCH);
        let first_road = all.iter().position(|d| d.site == ROAD_COST);
        assert_eq!(
            first_road,
            Some(4),
            "the first road-cost draw on the word is not the fifth draw of the frame: {:?}",
            all.iter()
                .map(|d| format!("{:x}", d.site))
                .collect::<Vec<_>>()
        );
    }

    pub(in super::super) fn run54_s_24000_frames_are_where_the_second_map_s_word_now_parts() {
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
        let unit_window = site_window();
        for f in 0..last {
            built.tick();
            // `RON_DEBUG_SITES=<lo>-<hi>`: each draw with the unit that
            // spent it, and the original's labels where they part — run53's
            // window, which this map's long test lacked until item 643
            // asked whose `Guy::inc_time` wrap 13640's extra draw is.
            if unit_window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
                for (label, who) in attributed_sites(&built) {
                    eprintln!("  f{f} {who}: {label}");
                }
                let theirs = trace.labels(f);
                if theirs != built.frame_sites.last().map_or(&[][..], |(_, v)| v) {
                    eprintln!("  f{f} PARTS: theirs {theirs:?}");
                }
            }
            // `RON_DEBUG_LEADER=<lo>-<hi>`: East Indies' word 9983 is a
            // `make_stuff` (item 576), and the list it spends is printed
            // here or nowhere on this crate's side.
            debug_leader(&built, f);
            // `RON_DEBUG_TECH` and `RON_DEBUG_BUILDS` on the traced walk
            // too (item 1476): a tech the original gains and this crate
            // does not is dated by the word's own walk.
            debug_tech(&built, f);
            debug_builds(&built, f);
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
        // **The pasture 23420 stocks** (item 919, `docs/AI.md` §79): who=1's
        // Farm `1/2032`, placed on 23182 beside the unstarted site `1/2031`,
        // is a pasture here as there, and its five animals are gaia's pigs
        // — the original's five coins on 23420 are all odd. No dump prints
        // an owner-9 object, so the draws are the original's side of this.
        let pasture = built.sim.building_by_o(1, 2032).expect("1/2032");
        assert_eq!(
            built.sim.buildings[pasture].farm.farm_type & sim::farms::ANIMAL_FARM,
            sim::farms::ANIMAL_FARM,
            "1/2032 is a pasture"
        );
        let stock: Vec<(u8, Option<sim::tech::TypeId>)> = built
            .sim
            .units
            .iter()
            .filter(|u| u.alive() && u.farm_animal.is_some_and(|fa| fa.build == pasture))
            .map(|u| (u.owner, u.ty.and_then(|t| built.sim.unit_types[t].tree)))
            .collect();
        assert_eq!(
            stock,
            [(9, Some(sim::farms::FARMPIG)); 5],
            "its five animals: gaia's pigs"
        );
        // **A word at the trace's end is not a cap reading as agreement**
        // (item 899's check, item 919's map): the trace holds a `FRAME`
        // record for every frame 0..=24000, the harness walked and compared
        // every one of 0..24000, and both sides spent the same draws.
        let traced: std::collections::BTreeSet<i64> =
            trace.frames.iter().map(|(n, _)| *n).collect();
        let walked: Vec<i64> = built.frame_sites.iter().map(|(f, _)| *f).collect();
        let (ours_draws, theirs_draws): (usize, usize) = built
            .frame_sites
            .iter()
            .map(|(f, o)| (o.len(), trace.labels(*f).len()))
            .fold((0, 0), |(a, b), (x, y)| (a + x, b + y));
        eprintln!(
            "run54: {} traced frames, {} walked, draws ours {ours_draws} theirs {theirs_draws}",
            traced.len(),
            walked.len()
        );
        if LONG_WORD_EAST_INDIES == last {
            assert_eq!(
                (last, traced.len(), walked.len()),
                (24_000, 24_001, 24_000),
                "run54's trace is 24,000 frames and every one is walked"
            );
            assert!(
                (0..last).all(|n| traced.contains(&n)) && walked.iter().copied().eq(0..last),
                "a frame of 0..24000 is missing from the trace or the walk"
            );
            assert_eq!(ours_draws, theirs_draws, "the draw totals");
        }
    }

    pub(in super::super) fn run64_s_window_clocks_are_the_original_s() {
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

    pub(in super::super) fn run94_s_window_clocks_are_the_original_s() {
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

    pub(in super::super) fn run100_s_word_frame_is_the_original_s() {
        /// run100's first complete block.
        const FIRST: i64 = 9_340;
        /// **The human citizen's gather residue** (item 464,
        /// `docs/COMBAT.md` §34.4): with its attack order gone `0/5`
        /// takes the original's `FLEE_TO` and then a `GATHERORDER` the
        /// original's citizen does not, so the list is two deep where the
        /// dump carries one. It opened **on** the word until item 465
        /// moved the headline one frame past it; naming it here rather
        /// than widening `opens` is the same choice `RAID` is, and for
        /// the same reason — the exemption has to name the row, so that
        /// the day the row closes this fails.
        const CITIZEN_GATHER: i64 = 10_233;
        /// The human city's raid, which this crate does not hold at all:
        /// `CityData::raid_stamp` is `Leader::raid`'s and `0x2` is
        /// `no_heal`, both named as unheld in [`compare`]'s own city
        /// block. They are the one thing besides the queue that opens
        /// inside the window, and they are who=0's. **Since item 1034
        /// `0x2` is held** (`docs/COMBAT.md` §69) and agrees; `raid_stamp`
        /// opened here alone. **Closed on item 2235** (`docs/GOLDEN.md`
        /// §81): `Object::do_damage`'s city alarm writes it, and the row
        /// no longer opens; the exemption stays so the history reads.
        const RAID: i64 = 9_451;
        /// ~~**`1/28` one world unit east**, item 477's row and the
        /// block the word stood on until item 478 moved it past.~~
        /// **Item 515 moved it seven blocks, 10234 → 10241**, and the
        /// row is still one world unit east — but of a different frame
        /// and for a different reason, so the number moves and the
        /// exemption keeps naming exactly one row.
        ///
        /// 477's row was the **first** frame of the raid's walk: both
        /// sides left `(4776, 30168)` on sim-frame 10234 with the same
        /// facing, and this crate stepped 26 where the original stepped
        /// 25, because it had no group speed cap at all
        /// (`docs/GROUPS.md` §18). With the cap the first seven frames
        /// are exact, and what is left opens on **10241**, the frame
        /// the original's cap drops back to 25 and this crate's stays
        /// at 26: §18's residue, and it chains to `1/40`'s order stack
        /// rather than to the cap. Named here rather than widening
        /// `opens`, for `RAID`'s reason — the exemption names the row,
        /// so the day the row closes this fails.
        ///
        /// **Item 518 closed it**: the original's cap goes back to 25 on
        /// 10241 because `Groups::process` resets `group 65` on that
        /// frame, and this crate's now does the same (`docs/GROUPS.md`
        /// §19). The row is gone, so the exemption is too — and the
        /// residue set below lost `(1, 28)`.
        /// ~~**`1/29` stops where the original walks on**, and **`1/27`
        /// then walks into it** — the two rows item 483's word move
        /// pulled below the line, at 10241 and 10242.~~
        ///
        /// **Item 487 closed `1/29`'s row and `RAIDER_STOPS` with it**,
        /// so only `1/27`'s collision is exempted here now. The 483 row
        /// this replaces read "`1/29` drops its move order", which the
        /// widening falsified: `order:kind ours 10 theirs 1` is *this
        /// crate* holding an `ATTACK` where the original already holds
        /// the plain `MOVE_TO` it ungrouped into, and this crate's
        /// `1/29` never took a step in the whole window. What was
        /// missing was `Unit::ungroup_move_order`'s **re-head** and
        /// `do_move`'s `vector_dist < 0x481` gate on the dead-target
        /// re-path (`docs/ORDERS.md` §20); with both, `1/29` walks the
        /// original's own 43-node route home from 10242 and leaves the
        /// window's residue set entirely.
        ///
        /// `1/27`'s own row stays and is still 10242's: it opens two
        /// blocks before the word did and is a position, not a
        /// collision. Named here rather than widening `opens`, for
        /// `RAID`'s reason — the exemption has to name the row, so the
        /// day the row closes this fails.
        ///
        /// **Item 518 closed it with `1/28`'s**: `1/27` walks the same
        /// capped pace a block behind, and the exemption went with it.
        /// **`1/35` twelve world units short**, item 497's first
        /// exposure: the word moved 10303 → 10582 and this window runs
        /// to the word, so 279 blocks nothing had ever compared came
        /// under the line in one landing. The raider's `half_step` parts
        /// on 10353 and its position a block later, 12 west and 12 north
        /// of the dump's — the same shape `RAIDER_X` is, one raider
        /// over. Named rather than widening `opens`, for `RAID`'s
        /// reason. **It is not 497's**: 497 moved the word past it and
        /// the row was already standing.
        const RAIDER_SHORT: i64 = 10_353;
        /// **The human city's `free`**, the second row item 497's move
        /// exposed: `CityData::free` parts on 10400, ours 0 against the
        /// dump's 1, on the same `0/2000` whose `raid_stamp` `RAID`
        /// already names. Also not 497's, and also pre-existing.
        const CITY_FREE: i64 = 10_400;
        /// ~~**The word's own queue row** (item 497)~~ — **item 506
        /// closed it and exposed the row under it.** The Stable's queue
        /// is the original's on 10583 now; what parts is the **price**,
        /// two hundred frames on and inside the window the word's move
        /// opened. On **10782** both sides queue Horse Archers at
        /// `1/2018` and this crate charges `cost[0] 60`/`cost[1] 40`
        /// where the original charges **57**/**38** — a discount of
        /// about a twentieth, on the queue entry rather than on the
        /// stockpile, so it is `get_cost`'s and not the ledger's.
        /// Named here rather than widened away, for `RAID`'s reason:
        /// the day the row closes this fails.
        const STABLE_PRICE: i64 = 10_782;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r100)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run100-greatlakes-valuewindow2.txt"),
        ) else {
            eprintln!("skipping: no run53/run100 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
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
        let players = built.sim.players.len();
        use std::collections::BTreeMap;
        let mut blocks = 0usize;
        let mut totals = [0usize; 8];
        // The block each key **first** parts on, which is the number
        // this item is about: a residue standing before the window
        // opened is not the word's, and one that opens inside it is.
        let mut units: BTreeMap<(i64, i64), i64> = BTreeMap::new();
        let mut queues: BTreeMap<(i64, i64), (i64, String)> = BTreeMap::new();
        let mut cities: BTreeMap<(i64, i64, String), i64> = BTreeMap::new();
        let mut orders: BTreeMap<(i64, i64), i64> = BTreeMap::new();
        type Roster = Vec<(i64, i64)>;
        let mut word: Option<(Roster, Roster)> = None;
        for f in 0..=LONG_WORD_GREAT_LAKES {
            built.tick();
            if f + 1 < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == f + 1) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            let r = compare(&built, &frame, players);
            blocks += 1;
            for (t, c) in totals.iter_mut().zip([
                r.diverged.len(),
                r.unlinked_units.len(),
                r.extra_units.len(),
                r.build_diverged.len(),
                r.queue_diverged.len(),
                r.city_diverged.len(),
                r.order_diverged.len(),
                r.gather_diverged.len(),
            ]) {
                *t += c;
            }
            for d in &r.diverged {
                units.entry((d.who, d.o)).or_insert(f);
            }
            for d in &r.queue_diverged {
                queues.entry((d.who, d.o)).or_insert_with(|| {
                    (
                        f,
                        format!("{} ours {} theirs {}", d.field, d.ours, d.theirs),
                    )
                });
            }
            for d in &r.city_diverged {
                cities.entry((d.who, d.o, d.field.clone())).or_insert(f);
            }
            for d in &r.order_diverged {
                orders.entry((d.who, d.o)).or_insert(f);
            }
            if f == GREAT_LAKES_SIXTH_SCHOLAR {
                word = Some((r.unlinked_units.clone(), r.extra_units.clone()));
            }
        }
        eprintln!(
            "run100 word window: {blocks} blocks [{FIRST}, {}] — pos {} unlinked {} \
             extra {} build {} queue {} city {} order {} gather {}",
            LONG_WORD_GREAT_LAKES + 1,
            totals[0],
            totals[1],
            totals[2],
            totals[3],
            totals[4],
            totals[5],
            totals[6],
            totals[7],
        );
        eprintln!("  units  {units:?}");
        eprintln!("  queues {queues:?}");
        eprintln!("  orders {orders:?}");
        eprintln!("  cities {cities:?}");
        // **The label is `GREAT_LAKES_SIXTH_SCHOLAR`'s, because that is
        // the block `word` is taken on.** It read `LONG_WORD_GREAT_LAKES
        // + 1` until item 456 and so named a block the row was not
        // measured at — harmless to the assertion and misleading to
        // anyone reading the output, which is what a diagnostic is for.
        eprintln!("  scholar block {GREAT_LAKES_SIXTH_SCHOLAR}: {word:?}");
        assert!(
            blocks >= 172,
            "run100's own blocks are missing — the wrong file: {blocks}"
        );
        // ~~**The word's own record, whole.** Of every unit either side
        // holds on the frame the headline stands on, exactly one is
        // unmatched, and it is the sixth scholar.~~ **Closed by item 442**
        // (`docs/AI.md` §53): the sixth scholar is born here on both
        // sides, so the block is `([], [])` — nothing missing and nothing
        // extra. The row is kept, and kept keyed on
        // `GREAT_LAKES_SIXTH_SCHOLAR` rather than on the headline, because
        // it is the assertion that says the birth *stayed* closed as the
        // word moves on past it.
        assert_eq!(
            word,
            Some((Vec::new(), Vec::new())),
            "block {GREAT_LAKES_SIXTH_SCHOLAR} is not clean — the sixth \
             scholar has come unstuck"
        );
        // ~~**The cause, as the record states it.** Two buildings, one
        // frame, and the frame is 128 before the birth.~~ **Also closed**:
        // the two Citizens at `1/2007` are gone and the University's job
        // is this crate's too, so the queue record parts nowhere in the
        // window. Item 408 measured the parting; this is its absence,
        // asserted so it cannot come back quietly.
        //
        // ~~**Item 497 put one back, and it is the word's own.**~~
        // **Item 506 took it off the word and left the row two hundred
        // frames under it**: the queue record parts nowhere inside the
        // window but on `STABLE_PRICE`, which is `1/2018`'s price and
        // not its contents, and the word's own frame carries no queue
        // row at all.
        let q: Vec<(i64, i64, i64, &str)> = queues
            .iter()
            .filter(|(_, (f, _))| *f != STABLE_PRICE)
            .map(|(&(w, o), (f, s))| (w, o, *f, s.as_str()))
            .collect();
        assert_eq!(
            q,
            Vec::new(),
            "run100's queues part somewhere in the window — item 408's \
             parting is back"
        );
        let on_stable: Vec<(i64, i64, i64, &str)> = queues
            .iter()
            .filter(|(_, (f, _))| *f == STABLE_PRICE)
            .map(|(&(w, o), (f, s))| (w, o, *f, s.as_str()))
            .collect();
        // ~~`1/2018`'s Horse Archers at 60 timber against the original's
        // 57~~ — **closed by item 545**: 57/38 is 60/40 less
        // `MILITARY_UNIT_DISCOUNT`'s 5% at `epoch[0]` 2 against the unit's
        // level 1, which this crate loaded and never applied
        // (`docs/AI.md` §56). The Stable's queue parts nowhere now.
        assert_eq!(
            on_stable,
            Vec::new(),
            "the Stable's queue parts on {STABLE_PRICE} again: {queues:?}"
        );
        // **Nothing else opens inside the window.** Every other
        // divergence was already standing on the window's first block —
        // which is what makes 9382 the frame rather than one of a
        // hundred and seventy-two.
        //
        // **The word's own frame is excluded since item 442**, and that
        // is not a loosening: the window runs to the headline, so once
        // the headline moved past this birth the *word's* parting fell
        // inside it, and a filter that swallowed it would have reported
        // the headline advancing as this test failing. What opens on the
        // word is asserted below instead, by name.
        let opens = |f: &i64| *f > FIRST - 1 && *f < LONG_WORD_GREAT_LAKES;
        // ~~**`0/5`'s order row is the *previous* word's own**~~ — item
        // 464 closed it. The citizen's order row stood one frame below
        // the headline because this crate gave it an `ATTACKORDER` on
        // the frame it was hit; `Unit::think`'s step 3 now carries the
        // military bit that keeps a citizen out of `think_attack`, and
        // `Unit::target_opportunity`'s flee arm answers the hit the way
        // the original does (`docs/COMBAT.md` §34). What is left of the
        // citizen is one row **on** the word, asserted by name below.
        let inside: Vec<String> = units
            .iter()
            .filter(|(k, f)| opens(f) && (**k, **f) != ((1, 35), RAIDER_SHORT))
            .map(|(k, f)| format!("unit {k:?} f{f}"))
            .chain(
                orders
                    .iter()
                    .filter(|(k, f)| opens(f) && (**k, **f) != ((0, 5), CITIZEN_GATHER))
                    .map(|(k, f)| format!("order {k:?} f{f}")),
            )
            .chain(
                cities
                    .iter()
                    .filter(|((w, o, _), f)| {
                        opens(f)
                            && (*w, *o, **f) != (0, 2_000, RAID)
                            && (*w, *o, **f) != (0, 2_000, CITY_FREE)
                    })
                    .map(|(k, f)| format!("city {k:?} f{f}")),
            )
            .collect();
        assert!(
            inside.is_empty(),
            "a record other than the queue parts inside run100's window: {inside:?}"
        );
        // ~~**And what opens on the word is one unit's position.**~~
        // ~~**Item 456 closed 10161**~~ … ~~a **three-unit squad**~~ …
        // ~~**Item 463 closed the squad**~~ … ~~**item 464 closed the
        // citizen's half** … what opens on 10233 is **one** position,
        // `1/28`, one frame ahead of the original on the walk its dropped
        // order freed~~ — **item 465 closed that frame**, and the key is
        // `1/28` again for a different reason one block along.
        //
        // The two sides now plan on the same frame. ~~What parts on 10234
        // is where the plan **goes**: the original spends 204 draws there
        // (`PathFinder::calc_road_cost+0x46`) and this crate six, and the
        // 43-node route they each come back with is 240 apart in `x`,
        // because the formation slot underneath it is (`docs/ORDERS.md`
        // §16).~~ **Item 471 closed the route and item 475 named the
        // draws**: all 42 nodes agree, and the 204 are a *road* search
        // (`astar_caravan_road < find_road`) the original runs on 10234
        // and this crate does not run at all — nothing of `1/28`'s
        // (`docs/ORDERS.md` §18). The set is kept keyed on the headline deliberately; what
        // it asserts is that the word's own frame is a *small, named* set
        // and not a reshuffle, and `run100_s_word_block_is_every_record_
        // the_dump_carries` holds the value diff beside it.
        let on_word: Vec<(i64, i64)> = units
            .iter()
            .filter(|(_, f)| **f == LONG_WORD_GREAT_LAKES)
            .map(|(k, _)| *k)
            .collect();
        // ~~**Item 478 moved the word past `1/28`'s block**, so the word's
        // own frame carries no position row at all now~~; `1/28`'s is
        // exempted by name as `RAIDER_X` above and counted below the
        // line with the standing residue.
        //
        // ~~**Item 487 moved the word to 10277 and it carries one**~~ —
        // ~~`1/40` is 4 west and 25 north of the original's~~ — **item
        // 489 closed it**: the collision with `1/41` was §4.3's group
        // arm declining because [`sim::Sim::same_group_soft`] read
        // `army_of` where the original reads `UnitData +0x80`, and a
        // **pushed** group's members are in no army at all
        // (`docs/COLLISION.md` §11). The word is 10294 now and it
        // carries **no position row**: its block is the human's citizen
        // `0/5`, six rows and none of them a `pos`, which is the value
        // diff `run100_s_word_block_is_every_record_the_dump_carries`
        // holds beside this.
        assert_eq!(
            on_word,
            Vec::<(i64, i64)>::new(),
            "the word's own frame is not the empty set it is: {units:?}"
        );
        // ~~**And one order row, which is the residue item 464 left.**~~
        // The citizen's `GATHERORDER` residue is still there and still
        // `0/5`'s — `Unit::think` falls through to `think_peasant` on the
        // frame the flee arm fires and this crate's `find_gather_spot`
        // hands the idle citizen a job the original's does not — but it
        // opens on **10233** and the word is 10234, so it counts below
        // the line now rather than on it. The set the word carries is
        // empty on this record too.
        let order_on_word: Vec<(i64, i64)> = orders
            .iter()
            .filter(|(_, f)| **f == LONG_WORD_GREAT_LAKES)
            .map(|(k, _)| *k)
            .collect();
        assert_eq!(
            order_on_word,
            Vec::<(i64, i64)>::new(),
            "the word's own order row is not the empty set it is: {orders:?}"
        );
        // **The standing residues, as sets.** Counts over a window the
        // word controls rise when the word does (`ORDER_RESIDUE_RUN97`'s
        // own lesson), so what is pinned is *which* keys, not how many
        // rows they made. Each may only shrink.
        assert_eq!(
            units
                .iter()
                .filter(|(_, f)| **f < LONG_WORD_GREAT_LAKES)
                .map(|(k, _)| *k)
                .collect::<Vec<_>>(),
            // **Six → four on item 518**: `1/27` and `1/28` leave it,
            // the raiders `Groups::process`'s reset put back on the
            // original's pace (`docs/GROUPS.md` §19).
            //
            // **Four → three on item 539**: `1/35` leaves it, a Longbowman
            // of the squad whose soft flags the fast path's stride over an
            // empty world cell had parted (`docs/COLLISION.md` §4.2). The
            // word moved *up* on the same item, which can only add keys
            // below it, so the fall is the fix's.
            //
            // **Three → none on item 629**: `1/24`, `1/25` and `1/26` are
            // Great Lakes' Merchants, and each stood (24, 24) off its tile
            // corner since its unpack; `cast_unpack`'s merchant arm seats
            // them (`docs/MERCHANT.md` §3.2).
            Vec::<(i64, i64)>::new(),
            "the window's standing position residue is not the three it is"
        );
        // **Counted below the word**, not over the whole map: the word's
        // own row joined this set when the headline moved past 10232, and
        // a residue count that swallows the word reports the headline's
        // motion as its own failure — `ORDER_RESIDUE_RUN97`'s lesson, and
        // it cost this line a red gate on item 456.
        //
        // **18 → 17 on item 464**, and the unit that left is `0/5`: its
        // order row used to open one frame *below* the word and now
        // opens **on** it, as the gather residue asserted above.
        //
        // **17 → 18 on item 465**, and it is `0/5` coming back over the
        // line without moving: the word went 10233 → 10234 and the
        // citizen's gather residue, which opens on 10233, is below it
        // again. Nothing about that residue changed — this is the
        // headline's motion counted as the set's, which is exactly what
        // `ORDER_RESIDUE_RUN97`'s lesson warns of and why the keys are
        // printed in the message.
        //
        // **18 → 17 on item 494, and this time `0/5` is gone for a
        // reason.** The human's idle wait is `peasants_wait`'s switch and
        // not its value — 12, where this crate read 2 — so the citizen is
        // never handed the `GATHERORDER` at all and the residue that has
        // crossed this line twice on the headline's motion has no row
        // left to cross it with (`docs/ORDERS.md` §21). The remaining
        // seventeen are all who=1's.
        // **17 → 16 on item 723**: the scout's formation mirror (parked
        // 275) closes once `push_group` keeps an equal group's record.
        // **16 → 15 on item 1115**: the caravan's trade order carries no
        // bit 4 and its leg faces its bearing (`docs/CARAVAN.md` §11.3).
        // **15 → 0 on item 1926**: all fifteen were the `group.id` stand-in,
        // the army's number against the pool slot the original's id names.
        assert_eq!(
            orders
                .values()
                .filter(|f| **f < LONG_WORD_GREAT_LAKES)
                .count(),
            0,
            "the window's standing order residue is not empty: {orders:?}"
        );
        // **Twenty → twenty-one on item 497**, and the extra field is
        // the window's, not the simulation's: the word moved 279 frames
        // and `0/2000`'s `free` parts on 10400, which no run had ever
        // compared because every earlier word stopped short of it
        // (`CITY_FREE`). The set is pinned by count here and by name in
        // the `opens` filter above, so a *new* city field fails both.
        //
        // **Twenty-one → nineteen on item 661**, and the two that left
        // were never residue: `vans.length` on who=1's two cities was
        // compared against a literal 0, and the sim's own linked caravan
        // agrees. `trade_val` stays, and now says what it is — 160 against
        // 176, the Barracks and Stable this crate never seats in Norwich
        // (`docs/AI.md` §63). **Nineteen → seventeen on the same item's
        // fix**: both `trade_val` rows agree once the two join Norwich.
        // **Seventeen → sixteen on item 1034**: the human city's
        // `city_flags[0x2]`, which opened with `raid_stamp` on 9451
        // (`RAID`), agrees — `Object::take_damage` sets it and the city
        // building's 200-frame decay clears it (`docs/COMBAT.md` §69);
        // `raid_stamp` stands alone.
        // **Sixteen → fourteen on item 1477** (`docs/AI.md` §122): two of
        // who=1's city rows (the census's `land`/`filled`/`space`) agree
        // now that the circle walk starts at entry 1. **Fourteen → one on
        // item 1605** (`docs/AI.md` §169): the human's thirteen census rows
        // agree once its sweep runs; `raid_stamp` stands alone. **One →
        // zero on item 2235** (`docs/GOLDEN.md` §81): `do_damage`'s city
        // alarm writes `raid_stamp`, and the window's city records agree.
        assert_eq!(
            cities.len(),
            0,
            "the window's standing city residue is not empty: {cities:?}"
        );
        assert!(
            cities.keys().all(|(w, o, _)| (*w, *o) != (1, 2_019)),
            "the University's own city record parts: {cities:?}"
        );
    }

    pub(in super::super) fn run100_s_citizen_waits_twelve_idle_frames_for_the_same_job() {
        /// Three blocks before the flight's arrival, so the counter is
        /// compared from 0.
        const LO: i64 = 10_291;
        /// Six past the re-task, far enough to carry the reset and the
        /// first steps of the walk it opens.
        const HI: i64 = 10_450;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r100)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run100-greatlakes-valuewindow2.txt"),
        ) else {
            eprintln!("skipping: no run53/run100 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
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
        // `(block, idle, orders_x, orders_y)` either side, every block of
        // the wait. A row is a field here as everywhere else, but the
        // three travel together and the tuple is what names the event.
        let mut ours: Vec<(i64, i64, i64, i64)> = Vec::new();
        let mut theirs: Vec<(i64, i64, i64, i64)> = Vec::new();
        for f in 0..HI {
            built.tick();
            let n = f + 1;
            if n < LO {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            let Some(them) = frame.units.iter().find(|u| u.who == 0 && u.o == 5) else {
                continue;
            };
            let (Some(i), Some(x), Some(y)) = (them.idle, them.orders_x, them.orders_y) else {
                continue;
            };
            theirs.push((n, i, x, y));
            let u = built.sim.unit_by_o(0, 5).expect("the human's citizen 0/5");
            let un = &built.sim.units[u];
            ours.push((
                n,
                i64::from(un.idle),
                i64::from(un.orders_pos.x),
                i64::from(un.orders_pos.y),
            ));
        }
        assert!(
            theirs.len() > 150,
            "run100 does not carry the whole wait: {} blocks",
            theirs.len()
        );
        let first = (0..ours.len()).find(|&i| ours[i] != theirs[i]);
        assert_eq!(
            first.map(|i| (ours[i], theirs[i])),
            None,
            "the citizen's idle counter or its destination parts inside \
             the wait"
        );
        // And the event itself, named rather than left implicit in the
        // stream above: the block the original re-tasks on, the counter
        // it re-tasks at, and the point it re-tasks to.
        let retask = theirs
            .iter()
            .find(|(_, _, x, _)| *x == 4056)
            .copied()
            .expect("the original re-tasks the citizen inside the window");
        assert_eq!(
            retask,
            (10_445, 0, 4056, 28_776),
            "the original's citizen goes back to work on 10445, the block \
             after `idle` reaches 12"
        );
        assert_eq!(
            theirs.iter().find(|(_, i, _, _)| *i == 12).map(|r| r.0),
            Some(10_444),
            "`idle` reaches 12 on 10444"
        );
    }

    pub(in super::super) fn run123_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_MARKET.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_MARKET.1;
        /// **The block the word stood on when this widening was taken** —
        /// 11185's (item 520). Item 327 moved the word to 11531, past
        /// run123's last block, so this test no longer straddles the
        /// headline (the [`WIDENINGS`] row says who owes that); what it
        /// holds on this block is the value diff of the move.
        const WORD_BLOCK: i64 = GREAT_LAKES_MARKET_BLOCK;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r123)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run123-greatlakes-marketword.txt"),
        ) else {
            eprintln!("skipping: no run53/run123 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r123).unwrap();
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
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows) = (0usize, 0usize, 0usize);
        // `(block, who) -> (key -> (ours, theirs))`, kept for the walk and
        // the word's own printout.
        let mut kept: BTreeMap<(i64, usize), BTreeMap<String, (i64, i64)>> = BTreeMap::new();
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            debug_watch(&built, n);
            let (_, rows) = widen_block(&built, &frame, players, n, &mut firsts);
            compared += rows;
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            for who in 0..2usize {
                let Some(block) = flog.leader_block(n, who as i64) else {
                    continue;
                };
                let t = crate::diff::leader::theirs(&block);
                let mine = crate::diff::leader::rows(&loaded, &built, who);
                let row = kept.entry((n, who)).or_default();
                for (k, v) in &mine {
                    let Some(&y) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    row.insert(k.clone(), (*v, y));
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run123 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted",
            firsts.len()
        );
        // **Both sides printed once on the word's blocks** before any
        // quiet row is trusted (`docs/COMBAT.md` §44.2.1): the purse, the
        // make list's head, the commerce level.
        for n in [WORD_BLOCK, WORD_BLOCK + 1] {
            let Some(r) = kept.get(&(n, 1)) else { continue };
            let pick = |pre: &str| -> String {
                r.iter()
                    .filter(|(k, _)| k.starts_with(pre))
                    .map(|(k, (o, t))| format!("{k} {o}/{t}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            eprintln!("  block {n} who 1 ours/theirs: {}", pick("bucket"));
            eprintln!("    {}", pick("epoch"));
            eprintln!("    {}", pick("MAKE[0]"));
            eprintln!("    {}", pick("MAKE[1]"));
            eprintln!("    {}", pick("income"));
        }
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (WORD_BLOCK - 3..=WORD_BLOCK + 3).contains(f);
            let window =
                site_window_named("RON_DEBUG_ROWS").is_some_and(|(lo, hi)| (lo..=hi).contains(f));
            if near || window || *f == FIRST {
                for r in rows {
                    eprintln!("  f{f} {r}");
                }
            } else {
                eprintln!("  f{f}: {} keys", rows.len());
            }
        }
        // `RON_LEADER_WALK=<key>,…` — the key's `ours/theirs` at every
        // block it changes on either side, player 1.
        if let Ok(w) = std::env::var("RON_LEADER_WALK") {
            for k in w.split(',') {
                let mut prev = None;
                let mut ticks = Vec::new();
                for ((n, who), r) in &kept {
                    if *who != 1 {
                        continue;
                    }
                    let Some(&p) = r.get(k) else { continue };
                    if prev != Some(p) {
                        ticks.push(format!("{n}:{}/{}", p.0, p.1));
                        prev = Some(p);
                    }
                }
                eprintln!("  walk {k}: {}", ticks.join(" "));
            }
        }
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (TAIL - FIRST + 1) as usize,
            "run123's window is whole"
        );
        let pair = |n: i64, k: &str| -> (i64, i64) {
            *kept[&(n, 1)]
                .get(k)
                .unwrap_or_else(|| panic!("{k} on block {n}"))
        };
        // **The word's block, the four readings the stanza named**
        // (`tools/gamelog/captures.txt`, run123), each with the value diff
        // beside it. `use_market` needs the costs of the first
        // `max(1, epoch[Commerce])` slots with `t > 0`: the commerce level
        // agrees (R3 dead), the purse is *lower* in the original (R1 dead:
        // its wealth is short of no need at all), and the stock is under
        // the head's 60 timber and 60 metal on both sides (R4 dead). What
        // parts is **slot 1**: a Cataphract here, costing 65 wealth, and
        // an emptied slot (`t −1`) in the original — so this crate's
        // wealth is short and spends the third sell draw (R2). **Item 327
        // closed it**: the re-offered Merchant empties slot 1 here too,
        // and the Cataphract's `(227, -1)` is `(-1, -1)`.
        pin_eq!(
            (
                pair(WORD_BLOCK, "epoch[2]"),
                pair(WORD_BLOCK, "bucket[2:wealth]"),
                pair(WORD_BLOCK, "bucket[1:timber]"),
                pair(WORD_BLOCK, "bucket[4:metal]"),
                pair(WORD_BLOCK, "MAKE[0].t"),
                pair(WORD_BLOCK, "MAKE[1].t"),
            ),
            // Timber 1/6 → 6/6 and metal 44/47 → 47/47 on item 545: the
            // Hoplites 11183 buys cost the original's 61/42 here too, and
            // the Horse Archers of 10782 its 57/38 (`MILITARY_UNIT_DISCOUNT`,
            // `docs/AI.md` §56). ~~The wealth is (514)'s income gap.~~
            // Wealth 11/6 → 6/6 on item 661: the gap was the trade routes,
            // worth 160 against 176 until the Barracks and Stable joined
            // Norwich on 8734 (`docs/AI.md` §63).
            ((2, 2), (6, 6), (6, 6), (47, 47), (430, 430), (-1, -1)),
            "the word's inputs moved — 11185's `use_market` need is the \
             head and slot 1, and slot 1 agrees since item 327"
        );
        // **And what the emptied slot was**: the original's rotation
        // enters `Make` on 11182 holding a **Merchant** (`t 61`, cat 4,
        // val 909,090) in slot 1. Until item 327 this crate held a
        // Cataphract there, because `known_rares` had no writer and
        // `civilian_value`'s merchant arm was dead; now both sides hold
        // the Merchant (`docs/AI.md` §55).
        pin_eq!(
            (pair(11_182, "MAKE[1].t"), pair(11_182, "MAKE[1].cat")),
            ((61, 61), (4, 4)),
            "slot 1 entering the rotation's `Make` is the Merchant on both sides"
        );
        // Every key that first parts on the word's two blocks, the dump's
        // numbers beside this crate's. The rows are the claim.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == WORD_BLOCK || *f == WORD_BLOCK + 1)
            .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
            .collect();
        pin_eq!(
            on_word,
            // Item 327 took ten to one: the Merchant is offered, slot 1 is
            // the original's emptied slot, and the word left this block.
            // What is left is a make-list value 77/75 of the original's —
            // the same ratio as run117's `MAKE[3].val` and `MAKE[4].val`,
            // `docs/ECONOMY.md` §14.4's `get_cost` row. Item 629's
            // Merchant seat took it (`docs/MERCHANT.md` §3.2); which input
            // of the value it moved is not read.
            Vec::<String>::new(),
            "the word's blocks part on a different set"
        );
    }

    pub(in super::super) fn run125_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_ARMY.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_ARMY.1;
        /// run125's own first block; below it the walk reads run123.
        const R125: i64 = 11_440;
        /// The block the word stood on when this widening was taken.
        const WORD_BLOCK: i64 = GREAT_LAKES_ARMY_BLOCK;
        /// The block the word moved to on item 539 — frame 11582 writes it —
        /// and stood on until item 545 moved it to 11757, past this capture.
        const NEW_WORD_BLOCK: i64 = 11_583;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r123), Some(r125)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run123-greatlakes-marketword.txt"),
            dump("gamelog-run125-greatlakes-armyidle.txt"),
        ) else {
            eprintln!("skipping: no run53/run123/run125 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix123 = crate::capture::indexed::IndexedCapture::open(&r123).unwrap();
        let mut ix125 = crate::capture::indexed::IndexedCapture::open(&r125).unwrap();
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
        let players = built.sim.players.len();
        use std::collections::{BTreeMap, BTreeSet};
        let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let mut missing: BTreeSet<String> = BTreeSet::new();
        let (mut blocks, mut compared, mut leader_rows) = (0usize, 0usize, 0usize);
        // `(who, o) -> [(block, theirs at, ours at, theirs current order,
        // ours order count, theirs guy 0 clock, ours guy 0 clock)]` for the
        // squad, kept for the arrival and the word's printout.
        type Row = (i64, (i64, i64), (i64, i64), String, usize, String, String);
        let mut squad: BTreeMap<i64, Vec<Row>> = BTreeMap::new();
        let walk: Vec<String> = std::env::var("RON_LEADER_WALK")
            .map(|w| w.split(',').map(str::to_owned).collect())
            .unwrap_or_default();
        let mut walked: BTreeMap<String, Vec<(i64, i64, i64)>> = BTreeMap::new();
        let sweep: Option<(i64, i32, i32)> = std::env::var("RON_SWEEP").ok().and_then(|v| {
            let (f, u) = v.split_once(':')?;
            let (w, o) = u.split_once('/')?;
            Some((f.parse().ok()?, w.parse().ok()?, o.parse().ok()?))
        });
        for f in 0..=TAIL {
            if let Some((sf, w, o)) = sweep
                && sf == f
            {
                built.sim.sweep_watch = Some(sim::collide::SweepWatch::new(sf, w, o));
            }
            built.tick();
            if let Some((sf, _, _)) = sweep
                && sf == f
            {
                for l in built
                    .sim
                    .sweep_watch
                    .take()
                    .expect("the recorder")
                    .rendered()
                {
                    eprintln!("  sweep {sf}: {l}");
                }
            }
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let ix = if n < R125 { &mut ix123 } else { &mut ix125 };
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            debug_watch(&built, n);
            let (_, rows) = widen_block(&built, &frame, players, n, &mut firsts);
            compared += rows;
            // `RON_SOFT_AUDIT` — every block a squad member's soft flag
            // parts, and whether the squad's unit cells agree on it.
            if std::env::var("RON_SOFT_AUDIT").is_ok() {
                let uc = |x: i64| x.div_euclid(48);
                let mut cells_off = Vec::new();
                let mut flags_off = Vec::new();
                for t in frame
                    .units
                    .iter()
                    .filter(|t| t.who == 1 && (31..=39).contains(&t.o))
                {
                    let Some(u) = built
                        .sim
                        .units
                        .iter()
                        .find(|u| u.alive() && u.owner == 1 && i64::from(u.index) == t.o)
                    else {
                        continue;
                    };
                    let (ox, oy) = (i64::from(u.pos.x), i64::from(u.pos.y));
                    if (uc(t.pos.x), uc(t.pos.y)) != (uc(ox), uc(oy)) {
                        cells_off
                            .push(format!("{}:({},{})v({},{})", t.o, t.pos.x, t.pos.y, ox, oy));
                    }
                    let theirs = t.unit_masks.unwrap_or(0) & 0x100000 != 0;
                    if theirs != u.half_step {
                        flags_off.push(format!(
                            "{}:{}",
                            t.o,
                            if theirs { "theirs" } else { "ours" }
                        ));
                    }
                }
                if !flags_off.is_empty() || !cells_off.is_empty() {
                    eprintln!(
                        "  soft {n}: flags [{}] cells [{}]",
                        flags_off.join(" "),
                        cells_off.join(" ")
                    );
                }
            }
            for t in frame
                .units
                .iter()
                .filter(|t| t.who == 1 && (31..=39).contains(&t.o))
            {
                let Some(u) = built
                    .sim
                    .units
                    .iter()
                    .find(|u| u.alive() && u.owner == 1 && i64::from(u.index) == t.o)
                else {
                    continue;
                };
                let tg = t.guys.first().map_or_else(String::new, |g| {
                    format!(
                        "a{} t{}/{}",
                        g.cur_anim.unwrap_or(-9),
                        g.cur_time.unwrap_or(-9),
                        g.end_time.unwrap_or(-9)
                    )
                });
                let og = u.guys.first().map_or_else(String::new, |g| {
                    format!("a{} t{}/{}", g.anim, g.cur_time, g.end_time)
                });
                if std::env::var("RON_SQUAD_PATH")
                    .ok()
                    .and_then(|v| v.parse::<i64>().ok())
                    == Some(t.o)
                    && site_window_named("RON_DEBUG_ROWS")
                        .is_some_and(|(lo, hi)| (lo..=hi).contains(&n))
                {
                    eprintln!(
                        "  path 1/{} {n}: theirs ({},{}) {:?} {:?} | ours ({},{}) {:?} [{}]",
                        t.o,
                        t.pos.x,
                        t.pos.y,
                        t.path,
                        t.current_order().map(|o| (&o.kind, o.flags)),
                        u.pos.x,
                        u.pos.y,
                        u.path,
                        u.guys.first().map_or(String::new(), |g| format!(
                            "a{} {}",
                            g.anim, u.movement.facing.0
                        )),
                    );
                }
                squad.entry(t.o).or_default().push((
                    n,
                    (t.pos.x, t.pos.y),
                    (i64::from(u.pos.x), i64::from(u.pos.y)),
                    t.current_order()
                        .map_or_else(|| "-".into(), |o| o.kind.clone()),
                    u.orders.len(),
                    tg,
                    og,
                ));
            }
            // `RON_PRICE=<t>,…` — player 1's price of each type and the
            // count it can afford, on the `RON_DEBUG_ROWS` window.
            if let Ok(ts) = std::env::var("RON_PRICE")
                && site_window_named("RON_DEBUG_ROWS")
                    .is_some_and(|(lo, hi)| (lo..=hi).contains(&n))
            {
                for t in ts.split(',').filter_map(|t| t.parse::<u16>().ok()) {
                    let t = t as sim::tech::TypeId;
                    eprintln!(
                        "  preq {n} t{t}: {:?} level {} epoch {:?} researched {:?}",
                        built.sim.tech_tree.types[t].preq,
                        built.sim.tech_tree.military_level_of(t),
                        built.sim.tech[1].epoch,
                        built
                            .sim
                            .unit_types
                            .iter()
                            .position(|u| u.tree == Some(t))
                            .map(|r| built.sim.muster[1].researched[r]),
                    );
                    eprintln!(
                        "  price {n} 1/t{t} {}: {:?} affordable {} / {}",
                        built.sim.tech_tree.types[t as usize].name,
                        built.sim.type_price(1, t),
                        built.sim.type_affordable(1, t, true),
                        built.sim.type_affordable(1, t, false),
                    );
                }
            }
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            for who in 0..2usize {
                let Some(block) = flog.leader_block(n, who as i64) else {
                    continue;
                };
                let t = crate::diff::leader::theirs(&block);
                let mine = crate::diff::leader::rows(&loaded, &built, who);
                for (k, v) in &mine {
                    let Some(&y) = t.get(k) else {
                        missing.insert(k.clone());
                        continue;
                    };
                    leader_rows += 1;
                    if who == 1 && walk.iter().any(|w| k.starts_with(w.as_str())) {
                        let e = walked.entry(k.clone()).or_default();
                        if e.last().map(|l: &(i64, i64, i64)| (l.1, l.2)) != Some((*v, y)) {
                            e.push((n, *v, y));
                        }
                    }
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        // `RON_LEADER_WALK=<prefix>,…` — every player-1 leader key under
        // the prefixes, `ours/theirs` at each block either side changes.
        for (k, ticks) in &walked {
            let t: Vec<String> = ticks
                .iter()
                .map(|(n, o, y)| format!("{n}:{o}/{y}"))
                .collect();
            eprintln!("  walk {k}: {}", t.join(" "));
        }
        eprintln!(
            "run125 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted",
            firsts.len()
        );
        // **Both sides printed on the word's blocks** before any quiet row
        // is trusted (`docs/COMBAT.md` §44.2.1): the two bowmen whose idle
        // rolls part, their position, order and figure clock.
        for (o, rows) in &squad {
            for r in rows
                .iter()
                .filter(|r| (WORD_BLOCK - 3..=WORD_BLOCK + 3).contains(&r.0))
            {
                eprintln!(
                    "  block {} 1/{o}: theirs {:?} {} [{}] | ours {:?} orders {} [{}]",
                    r.0, r.1, r.3, r.5, r.2, r.4, r.6
                );
            }
        }
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (WORD_BLOCK - 3..=WORD_BLOCK + 3).contains(f);
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
        // `RON_SQUAD_WALK=<o>` — the unit's position gap, theirs minus
        // ours, at every block it changes.
        if let Some(o) = std::env::var("RON_SQUAD_WALK")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
        {
            let mut prev = None;
            for r in squad.get(&o).into_iter().flatten() {
                let gap = (r.1.0 - r.2.0, r.1.1 - r.2.1);
                if prev != Some(gap) {
                    eprintln!("  walk 1/{o} {}: gap {gap:?} theirs {:?} {}", r.0, r.1, r.3);
                    prev = Some(gap);
                }
            }
        }
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The move's value diff** (item 539). 533's word was `1/34`
        // reaching its `ATTACK_TO` point on 11531 here and 11533 in the
        // original, on a lag the squad's march had set. The march's
        // partings were soft-collision flags, and the two that parted with
        // every unit cell agreeing — `1/31` on 11305 and `1/35` on 11357 —
        // were `collide_here`'s fast path stepping past an empty world
        // cell where the original does not advance (`docs/COLLISION.md`
        // §4.2). With it in, the squad walks the original's own points on
        // every block of the window and both bowmen arrive with it.
        let arrival = |o: i64| -> (Option<i64>, Option<i64>) {
            let rows = squad.get(&o).map_or(&[][..], |v| v.as_slice());
            let at = |pick: &dyn Fn(&Row) -> bool| {
                rows.iter()
                    .filter(|r| r.0 >= 11_500)
                    .find(|r| pick(r))
                    .map(|r| r.0)
            };
            (at(&|r| r.3 == "-"), at(&|r| r.4 == 0))
        };
        pin_eq!(
            (arrival(34), arrival(36)),
            ((Some(11_533), Some(11_533)), (Some(11_530), Some(11_530))),
            "the two bowmen's arrivals, (theirs, ours)"
        );
        let parted: Vec<(i64, i64)> = squad
            .iter()
            .flat_map(|(o, rows)| rows.iter().filter(|r| r.1 != r.2).map(move |r| (*o, r.0)))
            .collect();
        pin!(
            parted.is_empty(),
            "a squad member's position parts: {:?}",
            &parted[..parted.len().min(12)]
        );
        // The old word's two blocks, pinned **empty**: nothing first parts
        // on 11531 or 11532 any more.
        let on_old: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == WORD_BLOCK || *f == WORD_BLOCK + 1)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(on_old, Vec::<String>::new(), "the old word's blocks part");
        // **The old word's block, 11583, as the move's value diff** (item
        // 545). 539 left the word on 11582's frame with 38 rows over
        // 11580–11583: ours placed a Mine (`1/2022`) and walked `1/9` to it,
        // the original queued the Phalanx research at `1/2016`. The cause
        // was two missing arms of `get_cost` (`docs/AI.md` §56):
        // `MILITARY_UNIT_DISCOUNT` left ours three metal short from 11183,
        // so Militia read unaffordable on 11579 and sank below the Mine;
        // and the research arm, with `produce_tech`'s unit route, is what
        // lets ours queue the Phalanx for the original's 90 food and 54
        // metal. What stays is `1/0`'s figure on 11580, which spends no draw.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (NEW_WORD_BLOCK - 3..=NEW_WORD_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        // Item 1461: `1/0`'s figure on 11580 leaves too; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the new word's blocks part on a different set"
        );
    }
}
