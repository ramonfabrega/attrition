pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run130_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_ARMY_TWO.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_ARMY_TWO.1;
        /// run125's and run130's own first blocks; below each the walk
        /// reads the capture before it.
        const R125: i64 = 11_440;
        const R130: i64 = 11_560;
        /// The block the word stood on when this widening was taken.
        const WORD_BLOCK: i64 = GREAT_LAKES_ARMY_TWO_BLOCK;
        /// The block `1/62`'s collision detour is planned on — frame 11688
        /// writes it — and the first its path parts on.
        const DETOUR_BLOCK: i64 = 11_689;
        /// The block army 2's group attack-to of frame 11512 is first seen
        /// on.
        const ORDER_BLOCK: i64 = 11_513;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r123), Some(r125), Some(r130)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run123-greatlakes-marketword.txt"),
            dump("gamelog-run125-greatlakes-armyidle.txt"),
            dump("gamelog-run130-greatlakes-armytwo.txt"),
        ) else {
            eprintln!("skipping: no run53/run123/run125/run130 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix123 = crate::capture::indexed::IndexedCapture::open(&r123).unwrap();
        let mut ix125 = crate::capture::indexed::IndexedCapture::open(&r125).unwrap();
        let mut ix130 = crate::capture::indexed::IndexedCapture::open(&r130).unwrap();
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
        // `(who, o) -> (theirs cur_anim, ours anim)` of guy 0 on the block
        // before, so a figure's animation change is seen on either side.
        let mut anims: BTreeMap<(i64, i64), (Option<i64>, Option<i64>)> = BTreeMap::new();
        // Player 1's positions either side on the word's blocks, for the
        // value diff the pin below reads.
        let mut changed: Vec<String> = Vec::new();
        // `RON_PROBE_PLACE=<frame>:<o>` — a payoff probe: before `frame`
        // ticks, player 1's unit `o` is seated where the original's dump
        // has it on the block that frame starts from.
        let probe: Option<(i64, i64)> = std::env::var("RON_PROBE_PLACE").ok().and_then(|v| {
            let (f, o) = v.split_once(':')?;
            Some((f.parse().ok()?, o.parse().ok()?))
        });
        let mut placed: Option<(i64, i64)> = None;
        // Army 2's `(o, (theirs group, stance), ours)` on the order's block.
        type Regroup = (i64, (Option<i64>, Option<i64>), Option<(i64, i64)>);
        let mut regroup: Vec<Regroup> = Vec::new();
        // `1/64`'s position on the detour's block, (theirs, ours).
        let mut neighbour: Option<((i64, i64), (i64, i64))> = None;
        for f in 0..=TAIL {
            if let Some((pf, po)) = probe
                && pf == f
            {
                let (px, py) = placed.expect("the probe's block was read");
                let u = built.sim.unit_by_o(1, po as i16).expect("the probed unit");
                let ok = built
                    .sim
                    .probe_relocate(u, sim::Pos::new(px as i32, py as i32));
                eprintln!("  probe {f}: 1/{po} seated at ({px},{py}): {ok}");
            }
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let ix = if n < R125 {
                &mut ix123
            } else if n < R130 {
                &mut ix125
            } else {
                &mut ix130
            };
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            if let Some((pf, po)) = probe
                && pf == n
            {
                placed = frame
                    .units
                    .iter()
                    .find(|t| t.who == 1 && t.o == po)
                    .map(|t| (t.pos.x, t.pos.y));
            }
            debug_watch(&built, n);
            if n == ORDER_BLOCK {
                regroup = frame
                    .units
                    .iter()
                    .filter(|t| t.who == 1 && (60..=65).contains(&t.o))
                    .map(|t| {
                        let ours = built.sim.unit_by_o(1, t.o as i16).map(|u| {
                            (
                                built.sim.pool_group_of(u),
                                i64::from(built.sim.units[u].stance),
                            )
                        });
                        (t.o, (t.group, t.stance), ours)
                    })
                    .collect();
            }
            if n == DETOUR_BLOCK {
                let theirs = frame
                    .units
                    .iter()
                    .find(|t| t.who == 1 && t.o == 64)
                    .map(|t| (t.pos.x, t.pos.y));
                let ours = built.sim.unit_by_o(1, 64).map(|u| {
                    (
                        i64::from(built.sim.units[u].pos.x),
                        i64::from(built.sim.units[u].pos.y),
                    )
                });
                neighbour = theirs.zip(ours);
            }
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (_, rows) = widen_block(&built, &frame, players, n, &mut here);
            compared += rows;
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
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            // `RON_PATH_UNIT=<o>` with `RON_DEBUG_ROWS=<lo>-<hi>` — player
            // 1's unit `o`: both path stacks, positions and current order.
            if let Some(o) = std::env::var("RON_PATH_UNIT")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                && site_window_named("RON_DEBUG_ROWS")
                    .is_some_and(|(lo, hi)| (lo..=hi).contains(&n))
                && let Some(t) = frame.units.iter().find(|t| t.who == 1 && t.o == o)
                && let Some(u) = i16::try_from(o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(1, o))
            {
                let un = &built.sim.units[u];
                eprintln!(
                    "  path 1/{o} {n}: theirs ({},{}) {:?} {:?}\n    ours ({},{}) {:?} {:?}",
                    t.pos.x,
                    t.pos.y,
                    t.path,
                    t.current_order(),
                    un.pos.x,
                    un.pos.y,
                    un.path,
                    un.orders.back(),
                );
                // Its neighbours within 480, both sides: a detour search
                // reads the unit grid, so a neighbour elsewhere is a cause.
                for t2 in frame.units.iter().filter(|t2| {
                    (t2.who, t2.o) != (1, o)
                        && (t2.pos.x - t.pos.x).abs() <= 480
                        && (t2.pos.y - t.pos.y).abs() <= 480
                }) {
                    let ours2 = u8::try_from(t2.who)
                        .ok()
                        .zip(i16::try_from(t2.o).ok())
                        .and_then(|(w, o2)| built.sim.unit_by_o(w, o2))
                        .map(|u2| {
                            let v = &built.sim.units[u2];
                            (v.pos.x, v.pos.y, v.orders.len(), v.path.len())
                        });
                    eprintln!(
                        "    near {}/{}: theirs ({},{}) orders {} path {} | ours {:?}",
                        t2.who,
                        t2.o,
                        t2.pos.x,
                        t2.pos.y,
                        t2.orders.len(),
                        t2.path.len(),
                        ours2
                    );
                }
            }
            // `RON_UNIT_TABLE=<lo>-<hi>` with `RON_DEBUG_ROWS` — player 1's
            // units `lo..=hi`: group, stance, form, order and position,
            // both sides, on every block of the window.
            if let Some((lo, hi)) = site_window_named("RON_UNIT_TABLE")
                && site_window_named("RON_DEBUG_ROWS").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for t in frame
                    .units
                    .iter()
                    .filter(|t| t.who == 1 && (lo..=hi).contains(&t.o))
                {
                    let ours = i16::try_from(t.o)
                        .ok()
                        .and_then(|o| built.sim.unit_by_o(1, o))
                        .map(|u| {
                            let un = &built.sim.units[u];
                            format!(
                                "g{} s{} f{} ({},{}) {:?}",
                                built.sim.pool_group_of(u),
                                un.stance,
                                un.form,
                                un.pos.x,
                                un.pos.y,
                                un.orders.back().map(|o| {
                                    let s = format!("{:?}", o.body);
                                    s.chars().take(120).collect::<String>()
                                })
                            )
                        });
                    eprintln!(
                        "  table {n} 1/{}: theirs g{:?} s{:?} f{:?} ({},{}) {:?} | ours {}",
                        t.o,
                        t.group,
                        t.stance,
                        t.form,
                        t.pos.x,
                        t.pos.y,
                        t.current_order().map(|o| (
                            o.kind.clone(),
                            o.x,
                            o.y,
                            o.group_id,
                            o.form_id,
                            o.in_group
                        )),
                        ours.unwrap_or_else(|| "absent".into())
                    );
                }
            }
            // **Who changes animation, both sides** (`docs/COMBAT.md`
            // §44.2.1: the `theirs` side printed before a quiet row is
            // trusted), on the word's blocks: every player-1 figure whose
            // guy 0 `cur_anim` changes on either side, with its position,
            // current order and group.
            for t in frame.units.iter().filter(|t| t.who == 1) {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = built.sim.unit_by_o(1, o16);
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((1, t.o), (ta, oa));
                if !(WORD_BLOCK - 3..=WORD_BLOCK + 2).contains(&n) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                let tg = t.guys.first().map_or_else(String::new, |g| {
                    format!(
                        "a{:?}->{:?} t{}/{}",
                        pt,
                        g.cur_anim,
                        g.cur_time.unwrap_or(-9),
                        g.end_time.unwrap_or(-9)
                    )
                });
                let og = ours.map_or_else(
                    || "absent".to_string(),
                    |u| {
                        let un = &built.sim.units[u];
                        format!(
                            "({},{}) group {} {:?} [{}]",
                            un.pos.x,
                            un.pos.y,
                            built.sim.pool_group_of(u),
                            un.orders.back().map(|o| {
                                let s = format!("{:?}", o.body);
                                s.chars().take(40).collect::<String>()
                            }),
                            un.guys.first().map_or_else(String::new, |g| format!(
                                "a{:?}->{} t{}/{}",
                                po, g.anim, g.cur_time, g.end_time
                            )),
                        )
                    },
                );
                let line = format!(
                    "{n} 1/{}: theirs ({},{}) group {:?} {:?} [{tg}] | ours {og}",
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    t.group,
                    t.current_order().map(|o| o.kind.clone()),
                );
                eprintln!("  anim {line}");
                changed.push(line);
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
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run130 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {} keys parted",
            firsts.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (WORD_BLOCK - 3..=WORD_BLOCK + 2).contains(f);
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
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        let _ = changed;
        if probe.is_some() {
            // A payoff probe changes the frames under test; the pins below
            // are the unprobed run's.
            return;
        }
        let first = |who: i64, o: i64, what: &str| firsts.get(&(who, o, what.to_string())).cloned();
        // **The word's blocks, empty: the value diff of item 557's move**
        // (`docs/GROUPS.md` §22, §23). Until 557 the original's `1/62`
        // stopped against `1/23` on 11758 and this crate's a frame later,
        // 60/45 behind, on a detour around a lagging `1/64`; eight rows
        // parted here. With the back-pointer carried, `1/64` leaves the
        // pool on 11512 as the original's does, walks uncapped, and the
        // whole chain closes — the word moved to 11806, past this window.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..=WORD_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the old word's blocks part again"
        );
        // **The chain, backwards, closed.** `1/62`'s detour of 11689 and
        // its position of 11702 no longer part, `1/64` stands on the
        // original's point on the detour's block, and `1/64`'s first step
        // after the order — 12 here against 25 until 557 — agrees.
        pin_eq!(first(1, 62, "path:length"), None, "1/62's detour");
        pin_eq!(first(1, 62, "pos"), None, "1/62's position");
        pin_eq!(
            neighbour,
            Some(((42_117, 22_691), (42_117, 22_691))),
            "1/64 on the detour's block, (theirs, ours)"
        );
        pin_eq!(first(1, 64, "pos"), None, "1/64's position");
        pin_eq!(
            first(1, 64, "g.last_speed[0]"),
            None,
            "1/64's first step after the order"
        );
        // ~~The pool pointers had parted once before, on 11424, when the
        // original's `1/62`..`1/64` arrived in pool group 69 and this
        // crate's in none; they agreed again from 11425 to 11512.~~ Since
        // item 882 `Unit::come_out` pushes the trained squad (`618900`,
        // `docs/GROUPS.md` §31), and 11424 reads 69 on both sides (ours
        // −1 before): army 2's pool pointers never part in the window.
        pin_eq!(first(1, 64, "group"), None, "army 2's first pool parting");
        // **The order's block, both sides whole** (`docs/COMBAT.md`
        // §44.2.1): the stance reaches `1/60`, `1/61`, `1/64` and `1/65`
        // and not `1/62` or `1/63`, and `1/64` is left naming no pool
        // group — on both sides since item 557 (`docs/GROUPS.md` §23),
        // where until then this crate reached all six and kept them in 66.
        pin_eq!(
            regroup,
            [
                (60, (Some(66), Some(1)), Some((66, 1))),
                (61, (Some(66), Some(1)), Some((66, 1))),
                (62, (Some(66), Some(0)), Some((66, 0))),
                (63, (Some(66), Some(0)), Some((66, 0))),
                (64, (Some(-1), Some(1)), Some((-1, 1))),
                (65, (Some(66), Some(1)), Some((66, 1))),
            ],
            "army 2 on the order's block, (o, theirs group/stance, ours)"
        );
    }

    pub(in super::super) fn run134_s_pool_list_is_the_original_s() {
        const FIRST: i64 = 11_416;
        const TAIL: i64 = 11_519;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r134)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run134-greatlakes-armypool.txt"),
        ) else {
            eprintln!("skipping: no run53/run134 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r134).unwrap();
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
        // `(block, what, ours, theirs)` for every disagreement.
        let mut parted: Vec<(i64, String, String, String)> = Vec::new();
        let mut lists: std::collections::BTreeMap<i64, (Vec<i64>, Vec<i64>)> =
            std::collections::BTreeMap::new();
        let mut blocks = 0usize;
        for f in 0..TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            blocks += 1;
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            let Some((_, block)) = flog.frames().into_iter().find(|(k, _)| *k == n) else {
                continue;
            };
            let theirs: Vec<i64> = crate::gamelog::groups(block)
                .into_iter()
                .find(|g| g.id == 66)
                .map(|g| g.members.iter().map(|m| m.o).collect())
                .unwrap_or_default();
            let ours: Vec<i64> = built
                .sim
                .pool_list(1, 2)
                .into_iter()
                .map(i64::from)
                .collect();
            if ours != theirs {
                parted.push((
                    n,
                    "list 66".into(),
                    format!("{ours:?}"),
                    format!("{theirs:?}"),
                ));
            }
            lists.insert(n, (ours, theirs));
            let (units, _, _) = crate::gamelog::records(block, false);
            for t in units
                .iter()
                .filter(|t| t.who == 1 && (60..=65).contains(&t.o))
            {
                let ours = built
                    .sim
                    .unit_by_o(1, t.o as i16)
                    .map(|u| built.sim.pool_group_of(u));
                if ours != t.group {
                    parted.push((
                        n,
                        format!("1/{} group", t.o),
                        format!("{ours:?}"),
                        format!("{:?}", t.group),
                    ));
                }
            }
        }
        for p in &parted {
            eprintln!("  {} {}: ours {} theirs {}", p.0, p.1, p.2, p.3);
        }
        assert_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        let list = |n: i64| lists.get(&n).cloned();
        assert_eq!(
            list(11_425),
            Some((vec![60, 61, 64], vec![60, 61, 64])),
            "11424's add leaves the squad's tail alone in the list"
        );
        assert_eq!(
            list(11_513),
            Some((vec![60, 61, 65, 62, 63, 64], vec![60, 61, 65, 62, 63, 64])),
            "11512's sort re-seats the squad whole at the end"
        );
        let rows: Vec<String> = parted
            .iter()
            .map(|(n, what, o, t)| format!("{n} {what}: ours {o} theirs {t}"))
            .collect();
        // ~~The one residue: on 11424 the original's `1/62`..`1/64` stand in
        // their `come_out` group 69 for one block (`Unit::come_out`'s own
        // push on frame 11423 — run134's coverage), which this crate does
        // not make.~~ Built by item 882 (parked 561, `docs/GROUPS.md` §31):
        // `1/62`..`1/64` read 69 on 11424 on both sides (ours −1 before),
        // and the pool agrees on every block of the window.
        assert_eq!(rows, Vec::<String>::new(), "run134's pool parts");
    }

    pub(in super::super) fn run135_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_CROSSING.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_CROSSING.1;
        /// run125's, run130's and run135's own first blocks; below each
        /// the walk reads the capture before it.
        const R125: i64 = 11_440;
        const R130: i64 = 11_560;
        const R135: i64 = 11_800;
        const WORD_BLOCK: i64 = GREAT_LAKES_CROSSING_BLOCK;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r123), Some(r125), Some(r130), Some(r135)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run123-greatlakes-marketword.txt"),
            dump("gamelog-run125-greatlakes-armyidle.txt"),
            dump("gamelog-run130-greatlakes-armytwo.txt"),
            dump("gamelog-run135-greatlakes-crossing.txt"),
        ) else {
            eprintln!(
                "skipping: no run53/run123/run125/run130/run135 capture (set RON_GAMELOG_DIR)"
            );
            return;
        };
        let mut ix123 = crate::capture::indexed::IndexedCapture::open(&r123).unwrap();
        let mut ix125 = crate::capture::indexed::IndexedCapture::open(&r125).unwrap();
        let mut ix130 = crate::capture::indexed::IndexedCapture::open(&r130).unwrap();
        let mut ix135 = crate::capture::indexed::IndexedCapture::open(&r135).unwrap();
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
        let (mut blocks, mut compared, mut leader_rows, mut pools) =
            (0usize, 0usize, 0usize, 0usize);
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
        // `(who, o) -> (theirs cur_anim, ours anim)` of guy 0 on the block
        // before, so a figure's animation change is seen on either side.
        let mut anims: BTreeMap<(i64, i64), (Option<i64>, Option<i64>)> = BTreeMap::new();
        // Every player-1 figure whose animation changes on either side on
        // the word's blocks: `(block, o, theirs changed, ours changed)`.
        let mut changed: Vec<(i64, i64, bool, bool)> = Vec::new();
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let ix = if n < R125 {
                &mut ix123
            } else if n < R130 {
                &mut ix125
            } else if n < R135 {
                &mut ix130
            } else {
                &mut ix135
            };
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            debug_watch(&built, n);
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (_, rows) = widen_block(&built, &frame, players, n, &mut here);
            compared += rows;
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
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            if let Some((lo, hi)) = site_window_named("RON_UNIT_TABLE")
                && site_window_named("RON_DEBUG_ROWS").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for t in frame
                    .units
                    .iter()
                    .filter(|t| t.who == 1 && (lo..=hi).contains(&t.o))
                {
                    let ours = i16::try_from(t.o)
                        .ok()
                        .and_then(|o| built.sim.unit_by_o(1, o))
                        .map(|u| {
                            let un = &built.sim.units[u];
                            format!(
                                "g{} s{} f{} ({},{}) path {:?} {:?} a{:?}",
                                built.sim.pool_group_of(u),
                                un.stance,
                                un.form,
                                un.pos.x,
                                un.pos.y,
                                un.path
                                    .iter()
                                    .map(|p| (p.to.x, p.to.y, p.flags))
                                    .collect::<Vec<_>>(),
                                un.orders.back().map(|o| {
                                    let s = format!("{:?}", o.body);
                                    s.chars().take(200).collect::<String>()
                                }),
                                un.guys.first().map(|g| (g.anim, g.cur_time, g.end_time)),
                            )
                        });
                    eprintln!(
                        "  table {n} 1/{}: theirs g{:?} s{:?} f{:?} ({},{}) path {:?} {:?} a{:?}\n      ours {}",
                        t.o,
                        t.group,
                        t.stance,
                        t.form,
                        t.pos.x,
                        t.pos.y,
                        t.path,
                        t.current_order(),
                        t.guys.first().map(|g| (g.cur_anim, g.cur_time, g.end_time)),
                        ours.unwrap_or_else(|| "absent".into())
                    );
                }
            }
            // **Who changes animation, both sides** (`docs/COMBAT.md`
            // §44.2.1: the `theirs` side printed before a quiet row is
            // trusted): every player-1 figure whose guy 0 `cur_anim`
            // changes on either side on the word's blocks.
            for t in frame.units.iter().filter(|t| t.who == 1) {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = built.sim.unit_by_o(1, o16);
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((1, t.o), (ta, oa));
                if !(WORD_BLOCK - 2..=WORD_BLOCK + 1).contains(&n) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                eprintln!(
                    "  anim {n} 1/{}: theirs ({},{}) a{:?}->{:?} | ours {:?} a{:?}->{:?}",
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    pt,
                    ta,
                    ours.map(|u| (built.sim.units[u].pos.x, built.sim.units[u].pos.y)),
                    po,
                    oa
                );
                changed.push((n, t.o, pt != ta, po != oa));
            }
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            // **The pool, on run135's blocks**: every player-1 group the
            // original lists members for, and every slot this crate does.
            if n >= R135
                && let Some((_, block)) = flog.frames().into_iter().find(|(k, _)| *k == n)
            {
                let theirs: BTreeMap<i64, Vec<i64>> = crate::gamelog::groups(block)
                    .into_iter()
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
                    pools += 1;
                    if ours != t {
                        firsts
                            .entry((1, -2, format!("pool:{}", 64 + i64::from(s))))
                            .or_insert((n, format!("ours {ours:?} theirs {t:?}")));
                    }
                }
            }
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
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run135 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {pools} pool lists, {} keys parted",
            firsts.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (R130..=TAIL).contains(f);
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
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The word's blocks, empty: the value diff of item 560's move**
        // (`docs/COLLISION.md` §12). Until 560 they parted on one unit,
        // `1/37`, 25 rows from `half_step` on 11805: the original's `1/37`
        // waits on `1/64` from 11803 to 11809 (a hard collision every
        // frame, `collide` 4 → 10), and this crate's carried `unit_masks &
        // 0x100000` out of frame 11804's sweep — a soft group-mate seen on
        // the way to `1/64` hard — spent it on 11805 to step to (40194,
        // 20971) and stopped there on 11806, the extra `move_step+0x823`
        // draw. With the one-shot set only when the walk ends without a
        // hard hit, the word moved to 11903, past this window, and the
        // keys parted on it fell 635 → 284: every row from 11805 to the
        // capture's last block, gone.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..=WORD_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the old word's blocks part again"
        );
        let past: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f >= WORD_BLOCK - 2)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(past, Vec::<String>::new(), "run135 parts past 11804");
        // **Who stops, both sides**: `1/37`'s figure changes animation on
        // neither side on the word's blocks — until 560, on 11806 and 11807
        // in this crate alone.
        let stops: Vec<(i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.1 == 37)
            .map(|c| (c.0, c.2, c.3))
            .collect();
        pin_eq!(stops, Vec::<(i64, bool, bool)>::new(), "1/37's stop");
        // **Backwards**: nothing of `1/37` parts under the word but
        // (558)'s group-order ids.
        let under: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                (*w, *o) == (1, 37) && *f < WORD_BLOCK - 2 && what != "order:group.id"
            })
            .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
            .collect();
        pin_eq!(under, Vec::<String>::new(), "1/37 parts under the word");
    }

    pub(in super::super) fn run136_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_GREAT_LAKES_DETOUR.0;
        const TAIL: i64 = WIDENING_GREAT_LAKES_DETOUR.1;
        /// Each capture's first block the walk reads it from; below each
        /// the walk reads the capture before it. run136 opens on 11840,
        /// and the twenty blocks it shares with run135 are the stanza's
        /// overlap check, so the walk takes them from run135.
        const FROM: [i64; 5] = [FIRST, 11_440, 11_560, 11_800, 11_860];
        const POOLS: i64 = 11_800;
        const WORD_BLOCK: i64 = GREAT_LAKES_DETOUR_BLOCK;
        /// The first block anything parts on past the word, since item 566.
        const GREAT_LAKES_SQUAD_HALT_BLOCK: i64 = 11_922;
        let Some(inst) = install() else { return };
        let names = [
            "gamelog-run123-greatlakes-marketword.txt",
            "gamelog-run125-greatlakes-armyidle.txt",
            "gamelog-run130-greatlakes-armytwo.txt",
            "gamelog-run135-greatlakes-crossing.txt",
            "gamelog-run136-greatlakes-detour.txt",
        ];
        let paths: Vec<Option<String>> = names.iter().map(|n| dump(n)).collect();
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        if paths.iter().any(Option::is_none) {
            eprintln!("skipping: no run123/run125/run130/run135/run136 capture");
            return;
        }
        let mut ixs: Vec<crate::capture::indexed::IndexedCapture> = paths
            .iter()
            .flatten()
            .map(|p| crate::capture::indexed::IndexedCapture::open(p).unwrap())
            .collect();
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
        let (mut blocks, mut compared, mut leader_rows, mut pools) =
            (0usize, 0usize, 0usize, 0usize);
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
        // Every player-1 figure whose animation changes on either side on
        // the word's blocks: `(block, o, theirs changed, ours changed)`.
        let mut changed: Vec<(i64, i64, bool, bool)> = Vec::new();
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let c = FROM.iter().rposition(|&lo| n >= lo).unwrap();
            let ix = &mut ixs[c];
            let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
                continue;
            };
            let frame = ix.frame_state(at).unwrap();
            blocks += 1;
            debug_watch(&built, n);
            let mut here: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
            let (_, rows) = widen_block(&built, &frame, players, n, &mut here);
            compared += rows;
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
            // `RON_STANDING=<lo>-<hi>`: every row parting on each block of
            // the window, not only the first — a standing parting keeps
            // its first block in `firsts` and is invisible there. **And the
            // first parting block prints by default** (parked 830, the
            // sixteenth pass): five landings of one tranche misread a
            // standing row — set aside as "not on this chain", read as
            // noise, or called "from the gap" off the first block alone —
            // and what answered each was `RON_STANDING` over the first
            // block, which the assertions never print. `firsts` is empty
            // exactly there; a passing test captures the lines, a failing
            // one shows them.
            if firsts.is_empty()
                || site_window_named("RON_STANDING").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for ((w, o, what), (_, row)) in &here {
                    eprintln!("  standing {n} {w}/{o} {what}: {row}");
                }
            }
            for (k, v) in here {
                firsts.entry(k).or_insert(v);
            }
            if let Some((lo, hi)) = site_window_named("RON_UNIT_TABLE")
                && site_window_named("RON_DEBUG_ROWS").is_some_and(|(a, b)| (a..=b).contains(&n))
            {
                for t in frame
                    .units
                    .iter()
                    .filter(|t| t.who == 1 && (lo..=hi).contains(&t.o))
                {
                    let ours = i16::try_from(t.o)
                        .ok()
                        .and_then(|o| built.sim.unit_by_o(1, o))
                        .map(|u| {
                            let un = &built.sim.units[u];
                            format!(
                                "g{} s{} f{} ({},{}) path {:?} {:?} a{:?}",
                                built.sim.pool_group_of(u),
                                un.stance,
                                un.form,
                                un.pos.x,
                                un.pos.y,
                                un.path
                                    .iter()
                                    .map(|p| (p.to.x, p.to.y, p.flags))
                                    .collect::<Vec<_>>(),
                                un.orders.back().map(|o| {
                                    let s = format!("{:?}", o.body);
                                    s.chars().take(200).collect::<String>()
                                }),
                                un.guys.first().map(|g| (g.anim, g.cur_time, g.end_time)),
                            )
                        });
                    eprintln!(
                        "  table {n} 1/{}: theirs g{:?} s{:?} f{:?} ({},{}) path {:?} {:?} a{:?}\n      ours {}",
                        t.o,
                        t.group,
                        t.stance,
                        t.form,
                        t.pos.x,
                        t.pos.y,
                        t.path,
                        t.current_order(),
                        t.guys.first().map(|g| (g.cur_anim, g.cur_time, g.end_time)),
                        ours.unwrap_or_else(|| "absent".into())
                    );
                }
            }
            // **Who changes animation, both sides** (`docs/COMBAT.md`
            // §44.2.1: the `theirs` side printed before a quiet row is
            // trusted): every player-1 figure whose guy 0 `cur_anim`
            // changes on either side on the word's blocks.
            for t in frame.units.iter().filter(|t| t.who == 1) {
                let Ok(o16) = i16::try_from(t.o) else {
                    continue;
                };
                let ours = built.sim.unit_by_o(1, o16);
                let ta = t.guys.first().and_then(|g| g.cur_anim);
                let oa =
                    ours.and_then(|u| built.sim.units[u].guys.first().map(|g| i64::from(g.anim)));
                let before = anims.insert((1, t.o), (ta, oa));
                if !(WORD_BLOCK - 2..=WORD_BLOCK + 1).contains(&n) {
                    continue;
                }
                let Some((pt, po)) = before else { continue };
                if pt == ta && po == oa {
                    continue;
                }
                eprintln!(
                    "  anim {n} 1/{}: theirs ({},{}) a{:?}->{:?} | ours {:?} a{:?}->{:?}",
                    t.o,
                    t.pos.x,
                    t.pos.y,
                    pt,
                    ta,
                    ours.map(|u| (built.sim.units[u].pos.x, built.sim.units[u].pos.y)),
                    po,
                    oa
                );
                changed.push((n, t.o, pt != ta, po != oa));
            }
            let raw = ix.read_frame(at).unwrap();
            let flog = Log::parse(&raw);
            // **The pool, on run135's and run136's blocks**: every
            // player-1 group the original lists members for, and every
            // slot this crate does.
            if n >= POOLS
                && let Some((_, block)) = flog.frames().into_iter().find(|(k, _)| *k == n)
            {
                let theirs: BTreeMap<i64, Vec<i64>> = crate::gamelog::groups(block)
                    .into_iter()
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
                    pools += 1;
                    if ours != t {
                        firsts
                            .entry((1, -2, format!("pool:{}", 64 + i64::from(s))))
                            .or_insert((n, format!("ours {ours:?} theirs {t:?}")));
                    }
                }
            }
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
                    if *v != y {
                        firsts
                            .entry((who as i64, -1, format!("leader:{k}")))
                            .or_insert((n, format!("ours {v} theirs {y}")));
                    }
                }
            }
        }
        eprintln!(
            "run136 widening: {blocks} blocks [{FIRST}, {TAIL}], {compared} record rows, \
             {leader_rows} leader rows, {pools} pool lists, {} keys parted",
            firsts.len()
        );
        let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            by_block
                .entry(*f)
                .or_default()
                .push(format!("{w}/{o} {what}: {row}"));
        }
        for (f, rows) in &by_block {
            let near = (FROM[4]..=TAIL).contains(f);
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
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The word's blocks, both directions — and since item 566 they
        // agree** (`docs/PATHFINDER.md` §23, §24.6). Frame 11901 writes
        // block 11902: `1/62`, stopped on `1/27`, re-plans its flag-2
        // detour from the same point on both sides. Until item 566 the two
        // plans went round `1/27` on opposite sides, 36 rows over blocks
        // 11902..11904: ours 6 entries south by (38904, 21192), the
        // original's 8 north by (38904, 21096), and `1/64` stepping and
        // stopping here where it waits there. The original's search reads
        // `1/27`'s refused pre-walk verdicts for row 441 out of the
        // pathfinder's memo, which no `kill_lists` had cleared; with the
        // memo carried, every one of the 36 agrees. This is the move's
        // value diff, pinned empty.
        //
        // **And the next parting is 11922, which spends no draw**: on
        // sim-frame 11921 army 1's whole squad — `1/37`..`1/42` and
        // `1/62`..`1/64`, nine figures — stops in the original
        // (`stopped 1`, `last_speed 0`) and walks on here. Positions agree
        // on that block; the draw word runs on to 12038 because a stop is
        // not a draw. Pinned as the value diff beside the moved word.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..GREAT_LAKES_SQUAD_HALT_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the word's blocks, and every block after them to the squad's halt"
        );
        let halt: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == GREAT_LAKES_SQUAD_HALT_BLOCK)
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        // **Item 571 named it**: who=1's barracks research converts the
        // nine walking type-82 figures on sim-frame 11921, and the
        // original's `Guy::init_real` stands each kept guy with its speeds
        // zeroed (`docs/ANIM.md` §11). With `reinit_guys` doing the same,
        // 27 of the 29 rows go. What stands is `1/41`'s idle variant —
        // until item 3287: `init_real` zeroes `guy_flags`, and the last two
        // go (`docs/AI.md` §260).
        pin_eq!(halt, Vec::<String>::new(), "the squad's halt, block 11922");
        // **Backwards**: the 284 keys under the word are run135's standing
        // floor exactly — nothing new parts on 11860..11901 — and of
        // `1/62` and `1/64` only 11424's `come_out` residue, which
        // re-agrees on 11425, and (558)'s group-order ids. Item 629 took
        // 34 of them (284 → 250): Great Lakes' Merchants on their tile
        // corners and what their footprints held off
        // (`docs/MERCHANT.md` §3.2). Item 644 took two more (250 → 248):
        // who=1's `leader:bucket[1:timber]` and `leader:leftover[1:timber]`,
        // standing from 11400, which agree once the caps read the Commerce
        // level live (`Leader::gather@006ce280:58`).
        let under = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        // **Item 661 added two** (248 → 250): the rows that read the
        // sim's own `vans.length` agree and leave, and the four that read
        // the BUILD record's `city` and `city_down` for the Barracks
        // `1/2016`, the Stable `1/2018` and the Mine `1/2021` arrive — this
        // crate never seats the two in Norwich (`docs/AI.md` §63). **Its
        // fix took twelve** (250 → 238), and nothing arrived: those four
        // BUILD rows, both cities' `trade_val`, who=1's five wealth rows
        // (`bucket`, `income`, `leftover`, `rate`, `resources`) and
        // `worst_good`, which read the wealth. **Item 688 took twenty**
        // (238 → 218), all who=1 `SITE` rows, and nothing arrived: the
        // site values are the original's once a founded city wears them
        // down (`City::fix_world_vals`, `docs/AI.md` §67).
        // **Item 718 took six (218 → 212)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 723 took one (212 → 211)**: the scout's formation mirror
        // (parked 275), once `push_group` keeps an equal group's record.
        // **Item 729 took one more (211 → 210)**: `0/frame_attacked`, `Object::take_damage`'s stamp (`docs/AI.md` §71).
        // **Item 882 took three (210 → 207)**: `1/62`..`1/64`'s `group` on 11424, 69 on both sides (ours −1 before): `come_out`'s push of the trained squad (parked 561, `docs/GROUPS.md` §31).
        // Item 1072 took four (207 → 203): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
        // Item 1206 took sixteen (199 → 183): `1/34` and `1/36`'s heading, facing, guy angles and move destination rows on 11799, a formation handed over through `update_positions`' waypoint arm (`docs/GROUPS.md` §6.8).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 183.
        // Item 1461: 112 → 106; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
        // Item 1477: 106 → 104; the census circle walk starts at entry 1 (`docs/AI.md` §122).
        pin_eq!(
            under, // Item 1444: the ruins guard removes stale site fields (batch replay).
            36,    // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor under the word"
        );
        let pair: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                *w == 1 && [62, 64].contains(o) && *f < WORD_BLOCK - 2 && what != "order:group.id"
            })
            .map(|((_, o, what), (f, _))| format!("{f} 1/{o} {what}"))
            .collect();
        pin_eq!(pair, [""; 0], "1/62 and 1/64 under the word");
        // Item 882: `group` on 11424 agrees, 69 on both sides (ours −1 before),
        // `come_out`'s push (parked 561, `docs/GROUPS.md` §31). `form` and
        // `1/64`'s `orders_x/y` are `Unit::init`'s birth values (646). Item
        // 1457: the member's own `come_out` gives it its orders there.
        // **Who stops, both sides**: until item 566, `1/64`'s figure alone
        // changed animation on one side only on 11903..11905, ours walking
        // and stopping where the original's stood. With the memo carried,
        // no figure does on either side.
        let stops: Vec<(i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.2 != c.3).copied().collect();
        pin_eq!(stops, [], "1/64's stop is gone");
    }

    pub(in super::super) fn great_lakes_run163_through_run226_are_widened_whole() {
        let requests: [(i64, &[i64]); 9] = [
            (
                WIDENING_GREAT_LAKES_UPGRADE.1,
                &[GREAT_LAKES_UPGRADE_BLOCK, 12_136, GREAT_LAKES_MAKE_BLOCK],
            ),
            (
                WIDENING_GREAT_LAKES_CIVIC.1,
                &[GREAT_LAKES_CIVIC_BLOCK, 12_537, GREAT_LAKES_RETRY_BLOCK],
            ),
            (
                WIDENING_GREAT_LAKES_COPY.1,
                &[
                    GREAT_LAKES_COPY_BLOCK,
                    GREAT_LAKES_VALS_BLOCK,
                    GREAT_LAKES_ROAD_BLOCK,
                ],
            ),
            (WIDENING_GREAT_LAKES_BIRTH.1, &[GREAT_LAKES_BIRTH_BLOCK]),
            (WIDENING_GREAT_LAKES_PATRIOT.1, &[GREAT_LAKES_PATRIOT_BLOCK]),
            (
                WIDENING_GREAT_LAKES_MIRROR.1,
                &[GREAT_LAKES_MIRROR_BLOCK, GREAT_LAKES_RECRUIT_BLOCK],
            ),
            (
                WIDENING_GREAT_LAKES_WONDER.1,
                &[GREAT_LAKES_WONDER_BLOCK, GREAT_LAKES_ATTACKED_BLOCK],
            ),
            (WIDENING_GREAT_LAKES_ESCORT.1, &[GREAT_LAKES_ESCORT_BLOCK]),
            (
                WIDENING_GREAT_LAKES_GIVEUP.1,
                &[
                    GREAT_LAKES_GIVEUP_BLOCK,
                    GREAT_LAKES_PYRAMIDS_BLOCK,
                    GREAT_LAKES_FOREST_CELL_BLOCK,
                ],
            ),
        ];
        let near: Vec<i64> = requests
            .iter()
            .flat_map(|(_, points)| points.iter().copied())
            .collect();
        let chain = great_lakes_word_chain();
        // The first five tests used these shorter prefixes. Preserve their
        // source choices in the independent migration audit, including seams.
        let chain_lengths = [
            6,
            7,
            8,
            9,
            10,
            chain.len(),
            chain.len(),
            chain.len(),
            chain.len(),
        ];
        let Some((_, results)) = widen_on_siblings_with_checkpoints(
            SIBLING_DUMPS,
            false,
            (
                "gamelog-run53-greatlakes-24k-trace.txt",
                "rontrace-run53.log",
            ),
            "run163-through-run226",
            &chain,
            WIDENING_GREAT_LAKES_GIVEUP,
            11_800,
            &near,
            false,
            &requests,
        ) else {
            return;
        };
        let checks: [fn(Widened); 9] = [
            run163_s_word_frame_is_widened_whole,
            run174_s_word_frame_is_widened_whole,
            run178_s_word_frame_is_widened_whole,
            run192_s_word_frame_is_widened_whole,
            run196_s_word_frame_is_widened_whole,
            run202_s_word_frame_is_widened_whole,
            run211_s_word_frame_is_widened_whole,
            run218_s_word_frame_is_widened_whole,
            run226_s_word_frame_is_widened_whole,
        ];
        let names = [
            "run163", "run174", "run178", "run192", "run196", "run202", "run211", "run218",
            "run226",
        ];
        // Opt-in migration audit: independently replay each original window and
        // compare every result field, including those not pinned by its test.
        let verify = std::env::var_os("RON_VERIFY_SHARED_WIDENING").is_some();
        let mut failed = Vec::new();
        for ((((end, points), result), (name, check)), chain_len) in requests
            .iter()
            .zip(results)
            .zip(names.into_iter().zip(checks))
            .zip(chain_lengths)
        {
            eprintln!("{name}: checking original widening pins");
            if verify {
                let original_chain = &chain[..chain_len];
                for block in WIDENING_GREAT_LAKES_GIVEUP.0..=*end {
                    assert_eq!(
                        chain.iter().rfind(|(_, lo)| block >= *lo),
                        original_chain.iter().rfind(|(_, lo)| block >= *lo),
                        "{name}: source selection changed at block {block}"
                    );
                }
                let independent = widen_great_lakes(
                    name,
                    original_chain,
                    (WIDENING_GREAT_LAKES_GIVEUP.0, *end),
                    11_800,
                    points,
                )
                .unwrap();
                assert_eq!(
                    result, independent,
                    "{name}: shared and independent walk differ"
                );
                // Prove the original assertions still reject missing evidence,
                // shortened walks, and changed value rows after consolidation.
                let mut missing = result.clone();
                missing.missing.insert("injected absent field".into());
                assert!(std::panic::catch_unwind(|| check(missing)).is_err());
                let mut short = result.clone();
                short.blocks -= 1;
                assert!(std::panic::catch_unwind(|| check(short)).is_err());
                let mut changed = result.clone();
                match name {
                    // These tests pin concrete first-parting values, rather
                    // than the total standing-row count at their last point.
                    "run163" | "run174" | "run178" => {
                        let (o, field) = match name {
                            "run163" => (41, "g.cur_anim[0]"),
                            "run174" => (61, "order:group.id"),
                            _ => (74, "g.angle[0]"),
                        };
                        changed
                            .firsts
                            .get_mut(&(1, o, field.into()))
                            .unwrap()
                            .1
                            .push_str(" injected wrong value");
                    }
                    _ => {
                        changed
                            .standing
                            .get_mut(&points[points.len() - 1])
                            .unwrap()
                            .insert(
                                (-1, -1, "injected value difference".into()),
                                "ours != theirs".into(),
                            );
                    }
                }
                assert!(std::panic::catch_unwind(|| check(changed)).is_err());
                eprintln!("{name}: complete report equal; all three injected faults rejected");
            }
            if std::panic::catch_unwind(|| check(result)).is_err() {
                failed.push(name);
            }
        }
        assert!(
            failed.is_empty(),
            "original widening pins failed: {failed:?}"
        );
    }

    pub(in super::super) fn run243_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const WORD_BLOCK: i64 = GREAT_LAKES_RETURN_BLOCK;
        const OLD_WORD_BLOCK: i64 = GREAT_LAKES_STAND_BLOCK;
        const RUN226_TAIL: i64 = WIDENING_GREAT_LAKES_GIVEUP.1;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        }) = widen_great_lakes(
            "run243",
            &great_lakes_word_chain(),
            WIDENING_GREAT_LAKES_STAND,
            11_800,
            &[
                RUN226_TAIL,
                GREAT_LAKES_STAND_FIRST + 2,
                OLD_WORD_BLOCK,
                WORD_BLOCK,
            ],
        )
        else {
            return;
        };
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (RUN226_TAIL - WIDENING_GREAT_LAKES_STAND.0 + 1 + WIDENING_GREAT_LAKES_STAND.1
                - GREAT_LAKES_STAND_FIRST
                + 1) as usize,
            "the walk is whole on both captures: six of run226's, 319 of run243's"
        );
        let mut by: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            *by.entry((*f, *w, *o)).or_default() += 1;
            // Every first parting past run226, with its value diff: what a
            // re-pin reads off a failing run (item 795).
            if *f > RUN226_TAIL {
                eprintln!("  firstrow {f} {w}/{o} {what}: {row}");
            }
        }
        // **What the gap carried in** (R3, the stanza's): the rows standing
        // on run243's first block that were not standing on run226's last.
        // Until item 795, four walkers on long paths — `1/40`, `1/41`,
        // `1/42` and `1/60` — `1/80`'s formation and who=1's `SITE[1].reg`
        // (59 keys). **Item 795 took `1/40`, `1/41` and `1/42` (31)**:
        // `do_move`'s TAKE reads the stack again, so the return march
        // leaves the far point on the original's detour (`docs/GROUPS.md`
        // §32, run294). **Item 899 took `1/60`'s 23**: its walk home is
        // planned on 17656 while its current order is the `ATTACK`, and
        // `AttackOrder::is_attack` answers 1, so the original plans it as a
        // citizen (`docs/PATHFINDER.md` §29).
        let head: Vec<(i64, i64, usize)> = by
            .iter()
            .filter(|((f, _, _), _)| *f == GREAT_LAKES_STAND_FIRST)
            .map(|((_, w, o), n)| (*w, *o, *n))
            .collect();
        pin_eq!(
            head,
            [(1, -1, 1), (1, 80, 1)],
            "who the gap carried onto 20500, and on how many keys"
        );
        let tail = standing.get(&RUN226_TAIL).map_or(0, BTreeMap::len);
        let carried = standing.get(&GREAT_LAKES_STAND_FIRST).map_or(0, |h| {
            h.keys()
                .filter(|k| {
                    !standing
                        .get(&RUN226_TAIL)
                        .is_some_and(|t| t.contains_key(*k))
                })
                .count()
        });
        // Item 1330 took 50 (163/5 → 113/4): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (97, 4) → (96, 4), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            (tail, carried),
            // 28 before item 899: `1/60`'s 23 went.
            // Item 1072 took four (307 → 303): `0/5`'s five damage rows (`damage_frac`, `hits:damage`, `hits:damage_frac`, `hits_left`), standing from 11400 at ours 2/2 against the original's 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (36, 2), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "run226's standing rows on its last block, and the gap's on run243's first"
        );
        // **The old word's value diff** (item 795): on 20569 the
        // original's `1/40` stands blocked by `8/0` — `collide 1`,
        // `collide_who 8`, `collide_o 0`, `stopped 1`, at (17256, 26760) —
        // and so does ours now (it was mid-walk at (17203, 26773)). No key
        // of `1/40`, `1/41` or `1/42` parts on any block of the window.
        let walkers: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), _)| *w == 1 && [40, 41, 42].contains(o))
            .map(|((_, o, what), (f, row))| format!("{f} 1/{o} {what}: {row}"))
            .collect();
        pin_eq!(walkers, Vec::<String>::new(), "1/40..1/42 part nowhere");
        // **Up to the old word, on run243's own blocks**: who=0's
        // `production_step` on 20601 and nothing else. Until item 899,
        // `1/60`'s route parted on 20510..20775, and on the word's block
        // `1/60` stood blocked by `1/64` here and walked there — the step
        // ours spent at `Unit::move_step+0x823` on 20800.
        let own: Vec<(i64, i64, i64, usize)> = by
            .iter()
            .filter(|((f, _, _), _)| (GREAT_LAKES_STAND_FIRST + 1..=WORD_BLOCK).contains(f))
            .map(|((f, w, o), n)| (*f, *w, *o, *n))
            .collect();
        pin_eq!(
            own,
            // Item 1605 took the human's `production_step` on 20601
            // (`docs/AI.md` §169).
            Vec::<(i64, i64, i64, usize)>::new(),
            "who first parts on 20501..20801, the old word's block, and on how many keys"
        );
        // **The value diff on the old word's block, 20801** (item 899):
        // `1/60` reads `collide 0`, `collide_who -1`, `collide_o -1` on both
        // sides (ours was 1, 1 and 64, blocked by `1/64`), and no key of
        // `1/60` parts on any block of the window.
        let sixty: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), _)| (*w, *o) == (1, 60))
            .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
            .collect();
        pin_eq!(sixty, Vec::<String>::new(), "1/60 parts nowhere");
        let within = |lo: i64, hi: i64| {
            firsts
                .values()
                .filter(|(f, _)| (lo..=hi).contains(f))
                .count()
        };
        // **The floor**: run226's 307 standing, the gap's 5, the one up to
        // the old word, and nothing on the runway. 307/59/26/894 before item
        // 795, with the word at 20569: the walkers' 31 at 20500 and 25 of
        // the 26 to the old word closed, and none opened. 307/28/14/423
        // before item 899: `1/60`'s 23 at 20500, its 13 to the word and its
        // 74 on the runway closed, and none opened.
        // Item 1115 took the caravan's four rows here ((303, 5, 1, 309) → (299, 5, 1, 305); `docs/CARAVAN.md` §11.3).
        // Item 1330 took 50 (163/5/1/169 → 113/4/1/118): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1455: (97, 4, 1, 102) → (96, 4, 1, 101), who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
        pin_eq!(
            (
                within(WIDENING_GREAT_LAKES_STAND.0, RUN226_TAIL),
                within(GREAT_LAKES_STAND_FIRST, GREAT_LAKES_STAND_FIRST),
                within(GREAT_LAKES_STAND_FIRST + 1, WORD_BLOCK),
                firsts.len(),
            ),
            // Item 1072 took four (307/5/1/313 → 303/5/1/309): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            (36, 2, 0, 38), // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor"
        );
        // Item 1115 took the caravan's four rows here (308 → 304; `docs/CARAVAN.md` §11.3).
        pin_eq!(
            standing.get(&WORD_BLOCK).map_or(0, BTreeMap::len),
            // 341 before item 899: `1/60`'s 29 went, and who=0's
            // `production_step` (0 here, 1 there) stands on this block and
            // not on 20800. Item 1072 took four (312 → 308): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            38, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "every row standing on the old word's block"
        );
    }

    pub(in super::super) fn run80_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        }) = widen_great_lakes(
            "run80",
            &[(
                "gamelog-run80-greatlakes-latecensus.txt",
                WIDENING_GREAT_LAKES_END.0,
            )],
            WIDENING_GREAT_LAKES_END,
            // run80 prints no `GROUPDATA`: the pool walk has nothing to read.
            i64::MAX,
            &[WIDENING_GREAT_LAKES_END.0, LONG_WORD_GREAT_LAKES],
        )
        else {
            return;
        };
        // run80's leader records are the short form: 1,063 (1,059 before
        // item 1209's `agendas[·]`, 1,061 before item 1487's `popwin_*`) of the keys this
        // crate's leader rows carry (the `MAKE` list, `SITE`, `PERSONALITY`,
        // `num_units`/`num_queued` and the economy) are on no block, so
        // they are named here rather than read as agreeing.
        pin_eq!(
            missing.len(),
            // item 3255: +47 keys a leader (the rare masks and `rares_collected`).
            1118,
            "the keys run80's record does not carry"
        );
        pin_eq!(
            blocks,
            41,
            "run80 whole: 23960..23999 and the end block 24001"
        );
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **What the gap carried in**: 310 keys standing on run80's first
        // block — no dump holds 20819..23959 — and every one of them stood
        // on run243's last block, 20818, which held 311: the gap closed
        // who=1's `SITE[1].reg` and opened nothing (`RON_STANDING` over
        // both blocks, item 899). They are the floor rows the widenings
        // have carried since run226. **Nothing parts on
        // 23961..23999**, and on the end block 24001 only `orders.len`, 48
        // units, because the quit block prints no order list.
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            [(23960, 37), (24001, 48)], // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the blocks keys first part on, and how many"
        );
        let end: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == WIDENING_GREAT_LAKES_END.1)
            .map(|((_, _, what), _)| what)
            .filter(|what| what.as_str() != "orders.len")
            .collect();
        pin_eq!(
            end,
            Vec::<&String>::new(),
            "the end block parts on `orders.len` alone"
        );
        // Item 1115 took the caravan's four rows here (306 → 302; `docs/CARAVAN.md` §11.3).
        pin_eq!(
            standing
                .get(&(WIDENING_GREAT_LAKES_END.0 + 1))
                .map_or(0, BTreeMap::len),
            // Item 1072 took four (310 → 306): `0/5`'s four damage rows, standing from 11400 at ours 2/2 against 0/0: the civilian heal (`docs/COMBAT.md` §72).
            // Item 1444: the ruins guard removes stale site fields (batch replay).
            37, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
            "the floor stands on the second block, and nothing joins it"
        );
    }

    pub(in super::super) fn run96_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            standing,
            ..
        }) = widen_east_indies(
            "run96",
            "gamelog-run96-eastindies-latecensus.txt",
            WIDENING_EAST_INDIES_END,
            &[WIDENING_EAST_INDIES_END.0, LONG_WORD_EAST_INDIES],
            true,
        )
        else {
            return;
        };
        // run96's leader records are the short form, as run80's are: 1,059
        // of the leader keys and the 1,531 region keys are on no block. Its
        // `GUY` records are the short form too, so a gaia animal's clock
        // (`cur_anim`, `cur_time`) is named here rather than read as
        // parting; its position is compared on every block.
        pin_eq!(
            missing.len(),
            // Item 2384: +7, `aggression[1..8]`, which the short form
            // does not print. Item 3255: +47, the rare masks and
            // `rares_collected`.
            1_118 + 1_531 + 2,
            "the keys run96's records do not carry"
        );
        pin_eq!(
            blocks,
            41,
            "run96 whole: 23960..23999 and the end block 24001"
        );
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **What the gap carried in**: 293 keys standing on run96's first
        // block — no dump holds 23434..23959 — and 288 of them stood on
        // run299's last block, 23433 (`RON_STANDING` over both, item
        // 919). The gap opened one unit's five: `1/82`, a citizen born in
        // it, with the `form` and hit-point rows every citizen carries
        // from its birth (`1/81` on 23366 is the same shape). **Nothing
        // parts on 23961..23999**, and on the end block 24001 `orders.len`
        // on 48 units, because the quit block prints no order list, and
        // one position.
        // Item 1106: 293 → 292, who=1's `leftover[2:wealth]` 844
        // against 836 closed (`docs/AI.md` §84).
        // Item 1115: 292 → 288, the caravan's leg faces its bearing and its trade order carries no bit 4 (`order:move.angle`, `dest_angle`, `order:flags`, `order:action`; `docs/CARAVAN.md` §11.3).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            [(23960, 37), (24001, 49)],
            "the blocks keys first part on, and how many"
        );
        let end: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == WIDENING_EAST_INDIES_END.1)
            .filter(|((_, _, what), _)| what.as_str() != "orders.len")
            .map(|((w, o, what), (_, row))| format!("{w}/{o} {what}: {row}"))
            .collect();
        // **The one position on the end block** is the human's `0/3`, (+8, −23)
        // from the original's, and it is the closing
        // dump's tear (`super::endpoint`'s `torn`: a closing dump tears by
        // one tick on at most one unit, `super::shutdown`). `0/3` stands
        // where the original's does on every block to 23999, and the
        // endpoint prints East Indies' one torn unit, as it always has.
        pin_eq!(
            end,
            ["0/3 pos: ours (5542,5934) theirs (5534,5957)"],
            "the end block parts on `orders.len` and one position"
        );
        // Item 1115 took the caravan's four rows here (292 → 288; `docs/CARAVAN.md` §11.3).
        // Item 1106: 293 → 292, who=1's `leftover[2:wealth]` (`docs/AI.md` §84).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        pin_eq!(
            standing
                .get(&(WIDENING_EAST_INDIES_END.0 + 1))
                .map_or(0, BTreeMap::len),
            37,
            "the floor stands on the second block, and nothing joins it"
        );
    }

    pub(in super::super) fn run294_s_departure_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        const RUN226_TAIL: i64 = WIDENING_GREAT_LAKES_GIVEUP.1;
        let mut chain = great_lakes_word_chain();
        chain.pop();
        chain.push((
            "gamelog-run294-greatlakes-departword.txt",
            WIDENING_GREAT_LAKES_GIVEUP.1 + 1,
        ));
        let Some(Widened {
            firsts,
            missing,
            blocks,
            ..
        }) = widen_great_lakes(
            "run294",
            &chain,
            WIDENING_GREAT_LAKES_DEPART,
            11_800,
            &[RUN226_TAIL, GREAT_LAKES_DEPART_FIRST + 2],
        )
        else {
            return;
        };
        pin!(missing.is_empty(), "the record does not carry {missing:?}");
        pin_eq!(
            blocks,
            (RUN226_TAIL - WIDENING_GREAT_LAKES_DEPART.0 + 1 + WIDENING_GREAT_LAKES_DEPART.1
                - GREAT_LAKES_DEPART_FIRST
                + 1) as usize,
            "the walk is whole on both captures: six of run226's, 160 of run294's"
        );
        let mut by: BTreeMap<(i64, i64, i64), usize> = BTreeMap::new();
        for ((w, o, what), (f, row)) in &firsts {
            *by.entry((*f, *w, *o)).or_default() += 1;
            if *f > RUN226_TAIL {
                eprintln!("  firstrow {f} {w}/{o} {what}: {row}");
            }
        }
        // **The departure** (`docs/GROUPS.md` §32): the four walkers leave
        // the far point on 19853..19883 as one group move led by `1/40`.
        // Before item 795 its first parting was `1/40`'s own waypoint on
        // 19876 — (3912, 30984), `tolerance` 384 here against the detour
        // (3732, 31380), `tolerance` 0 there, both stacks' tops at the
        // detour — and 72 keys of `1/40`, `1/41` and `1/42` parted on
        // 19876..19914. With TAKE's second `peek` none does.
        let walkers: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                *w == 1 && [40, 41, 42].contains(o) && *f > GREAT_LAKES_DEPART_FIRST
            })
            .map(|((_, o, what), (f, row))| format!("{f} 1/{o} {what}: {row}"))
            .collect();
        pin_eq!(walkers, Vec::<String>::new(), "the departure agrees");
        // **What stands on run294's first block**, past run226's last: the
        // four walkers' `group.id` (group 4 here, 5 there — parked 871;
        // read by equality alone), `1/80`'s formation, who=0's `attacked`
        // ~~and its city's `city_flags[0x2]`~~ (item 1034 carries it,
        // `docs/COMBAT.md` §69), and who=1's `SITE[1].reg`. And
        // after it, nothing. **Until item 899 `1/60`'s world route stood
        // here too** — 22 path slots, slot 15 (32640, 24960) here against
        // (32640, 24192) there — and parted again on 19934..19951 (14
        // keys): its walk home, planned on 17656 while its current order
        // was the `ATTACK`, which the original plans as a citizen
        // (`docs/PATHFINDER.md` §29).
        let parted: Vec<(i64, i64, i64, usize)> = by
            .iter()
            .filter(|((f, _, _), _)| *f > RUN226_TAIL)
            .map(|((f, w, o), n)| (*f, *w, *o, *n))
            .collect();
        pin_eq!(
            parted,
            [(19840, 1, -1, 1), (19840, 1, 80, 1)],
            "who first parts past run226, and on how many keys"
        );
        pin_eq!(
            firsts.get(&(1, 40, "order:group.id".to_string())),
            None,
            "the walkers' group number, 4 here and 5 there"
        );
    }

    pub(in super::super) fn run136_s_word_is_one_cell_of_1_62_s_search() {
        const FIRST: i64 = 11_896;
        const TAIL: i64 = 11_905;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r136)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run136-greatlakes-detour.txt"),
        ) else {
            eprintln!("skipping: no run53/run136 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r136).unwrap();
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
        let mut firsts = std::collections::BTreeMap::new();
        /// `(ours, theirs)`: `1/62`'s path stack on block 11902.
        type Plan = (Vec<(i32, i32, u8)>, Vec<(i64, i64)>);
        let mut plan: Option<Plan> = None;
        for f in 0..=TAIL {
            built.tick();
            let n = f + 1;
            if n < FIRST {
                continue;
            }
            let at = ix
                .frames()
                .iter()
                .position(|x| x.number == n)
                .expect("run136 carries the block");
            let frame = ix.frame_state(at).unwrap();
            widen_block(&built, &frame, players, n, &mut firsts);
            if n == 11_902 {
                let ours = built
                    .sim
                    .unit_by_o(1, 62)
                    .map(|u| {
                        built.sim.units[u]
                            .path
                            .iter()
                            .map(|p| (p.to.x, p.to.y, p.flags))
                            .collect()
                    })
                    .unwrap_or_default();
                let theirs = frame
                    .units
                    .iter()
                    .find(|t| (t.who, t.o) == (1, 62))
                    .map(|t| t.path.iter().map(|p| (p.to.0, p.to.1)).collect())
                    .unwrap_or_default();
                plan = Some((ours, theirs));
            }
        }
        let (ours, theirs) = plan.expect("block 11902 was read");
        assert_eq!(
            ours,
            [
                (36534, 23448, 1),
                (37320, 22680, 0),
                (38088, 21912, 0),
                (38088, 21672, 2),
                (38088, 21624, 2),
                (38664, 21048, 2),
                (38856, 21048, 2),
                (38904, 21096, 2),
            ],
            "1/62's plan, with 1/27's pre-walk verdicts in the memo"
        );
        let ours_points: Vec<(i64, i64)> = ours
            .iter()
            .map(|&(x, y, _)| (i64::from(x), i64::from(y)))
            .collect();
        assert_eq!(ours_points, theirs, "the original's plan, entry for entry");
        let parted: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > FIRST)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        assert_eq!(parted, Vec::<String>::new(), "11897..11905 part");
    }
}
