pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run289_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_USEMARKET.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_USEMARKET.1;
        const WORD_BLOCK: i64 = EAST_INDIES_USEMARKET_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            standing,
            ..
        }) = widen_east_indies(
            "run289",
            "gamelog-run289-eastindies-marketword.txt",
            WIDENING_EAST_INDIES_USEMARKET,
            &[FIRST, WORD_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        pin_eq!(
            leader_rows,
            1337942,
            "257 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        let rows: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // **The first keys to part under the word are who=1's make list,
        // on block 20782**: three entries for city 1 here, valued 486, 398
        // and 486, where the original's are empty. On 20783 this crate
        // places a building the original does not (`1/2030`, and
        // `1/2023`'s `city_down`), sends the citizen `1/12` to build it
        // (`order:kind` 3 against 7) and spends timber, metal and wealth —
        // the word's `Leader::use_market+0x1ed`. The readings the stanza
        // wrote (`tools/gamelog/captures.txt`, run289): **R1 holds**
        // (21,045 frames identical to run54). **R2 stands on one row**:
        // who=1's only resource row parting before 20782 is the standing
        // `leftover[2:wealth]`, 2108 here against 2100 on the first block
        // (parked 851), and nothing else of its stock parts; **R3** has no
        // clock row parting either. No mechanism is named.
        //
        // **Item 890: the `val` rows close.** `get_cost`'s wonder arm counts
        // who=1's Pyramids site `1/2029`, so the Mausoleum (527) and the
        // Colossus (528) price at 260 wealth against the purse's 208 on both
        // sides, `check_income` answers 0, and both are offered at `val 0`
        // here as there: `MAKE[0]`, `[1]` and `[8]` `val` 486/398/486 → 0
        // (`docs/COSTS.md`, "A wonder is ramped by every wonder"; `docs/AI.md`
        // §77). What stands on 20782 is parked 325's `city` spelling alone:
        // this crate's city index is global (London is 1 of three) and the
        // dump's is the leader's own slot (0), the same city.
        let make: Vec<&String> = rows.iter().filter(|r| r.starts_with("20782 ")).collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            make,
            Vec::<&String>::new(),
            "only the list's `city` spelling parts on the word's old frame"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 3.
        pin_eq!(
            rows.len(),
            0,
            "nothing else parts on 20778..20783: {rows:?}"
        );
        // Past the old word's block, one key in the whole runway: who=0's
        // `production_step` on 20801, the human's own step, which nothing
        // on this chain reads (`docs/ECONOMY.md` §16.3's R3).
        let past: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > WORD_BLOCK)
            .map(row)
            .collect();
        pin_eq!(past, Vec::<String>::new(), "the runway past the old word");
        let wealth = standing
            .get(&FIRST)
            .and_then(|m| m.get(&(1, -1, "leader:leftover[2:wealth]".to_string())))
            .cloned();
        // **Closed by item 1106** (parked 851): a region's end raises the
        // economy flag at the border pass's pace (`docs/AI.md` §84), and
        // who=1's wealth agrees on the first block.
        pin_eq!(wealth, None, "who=1's standing wealth row (parked 851)");
        // **The floor**: every key standing on the first block, the keys
        // first parting under the word, the rows standing on the word's
        // block, and every key in all. The first block's 284 are the gap's
        // (20259..20776, which no dump compares): `form`, who=1's
        // citizens' hits and sight, the leader rows, and fourteen units'
        // orders and headings.
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 890: 284/42/326/1,222 → 284/3/287/288. Under the word the
        // three `MAKE` `val` rows and 36 keys of the building `1/2030`
        // this crate no longer places close; across the runway 934 keys
        // close, and the one that stands past the word is who=0's step.
        // Item 1106: → 283/3/286/287, who=1's wealth, the border pass's economy flag (`docs/AI.md` §84): `leftover[2:wealth]` 2108 against 2100 on 20777.
        // Item 1291 took (163, 3, 166, 167) → (162, 3, 165, 166): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (162, 3, 165, 166).
        // Item 1330 took 46 (162/0/162/163 → 116/0/116/117): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            (38, 0, 38, 38),
            "the floor"
        );
    }

    pub(in super::super) fn run299_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_WONDERPRICE.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_WONDERPRICE.1;
        const WORD_BLOCK: i64 = EAST_INDIES_WONDERPRICE_BLOCK;
        const NEW_BLOCK: i64 = EAST_INDIES_ADDANIMALS_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            standing,
            ..
        }) = widen_east_indies(
            "run299",
            "gamelog-run299-eastindies-wonderprice.txt",
            WIDENING_EAST_INDIES_WONDERPRICE,
            &[FIRST, WORD_BLOCK, NEW_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        pin_eq!(
            leader_rows,
            1337942,
            "257 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        let rows: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // **What parts under the word, and the escrow the item closed.**
        // On block 23182 who=1's head (`MAKE[0]`) and its category slot
        // (`MAKE[4]`) hold the Farm (417) in Norwich at 129,600, the city
        // rows being parked 325's spelling (this crate's global index 2,
        // the leader's slot 1). Before item 890's second fix both read
        // `escrow 0` here against 1 there: `create_buildings`' gather arm
        // escrows on every exit of its head test (listing
        // `006c3cbb`..`006c3db0`), and this crate did only on a pass. Now
        // 1 on both. On the word's frame both sides place the head's Farm
        // (`1/2031`) and walk slot 4's through the same spiral; at its
        // best site's 2x2 jitter (corner (184, 196)) this crate finds four
        // clear sub-positions and the original three, so one draw shifts,
        // the expiry that follows clears slot 4 there and not here, and
        // `1/79`'s figure draws one along. Both place `1/2032` at (35904,
        // 38016). ~~Which sub-position the original blocks is not
        // established~~ — **item 904**: the trace's seeds (57, 12, 99 there;
        // 57, 12, 99, 42 here) make it (35904, 38208), whose tile (188,
        // 200) is on the rock cell (47, 50), and `blocked_tcoord`'s rock
        // arm refuses it (`docs/AI.md` §78). The three rows on 23183 close:
        // `MAKE[4].t` 417 against −1 → −1 on both, `1/79`'s `cur_anim[0]`
        // 3 against 1 and `end_time[0]` 42 against 58 → the original's;
        // the draws on 23182 go 49 against 48 → 48 against 48.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "the keys first parting under the word"
        );
        let runway: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK + 1..=NEW_BLOCK).contains(f))
            .map(row)
            .collect();
        // **The runway to the new word, 23184..23421** (item 904). None of
        // it is a draw until 23420's. 23184's `MAKE[5].city` is parked
        // 325's spelling; who=0's `production_step` is the one 890 left
        // standing past the word; who=1's `gather_stamp` re-stamps on 23191
        // here against 23183 there, and it parts the same on the rule
        // before this item's (a mutation, restored from git) — parked 701's
        // shape, not this fix's. On 23366 `1/81` is born with the gap's
        // standing citizen rows (every citizen carries them from the first
        // block) and the census reads 29 peasants against 30 for one
        // block. ~~On the word's block the guard `1/78`'s `retry` and
        // `1/80`'s figure part with the frame's draws~~ — **item 919**: the
        // original stocks a pasture on 23420 because `1/2032` is one, and
        // it is one because `get_nearest_farm_type` did not see the
        // unstarted site `1/2031` beside it on 23182 (`docs/AI.md` §79).
        // The three rows on 23421 close: `1/78`'s `guard.retry` 8 against 6
        // → 6 on both, `1/80`'s `g.cur_anim[0]` 1 against 0 and
        // `g.end_time[0]` 58 against 31 → 0 and 31 on both; the draws on
        // 23420 go 7 against 31 → 31 against 31.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(runway, Vec::<String>::new(), "the runway to the new word");
        let escrow = standing
            .get(&WORD_BLOCK)
            .map(|m| {
                m.keys()
                    .filter(|(w, _, k)| *w == 1 && k.ends_with(".escrow"))
                    .count()
            })
            .unwrap_or(0);
        pin_eq!(escrow, 0, "the make list's escrow agrees (item 890)");
        // **The floor**: every key standing on the first block (the gap's,
        // 21046..23176, which no dump compares), the keys first parting
        // under the word, the rows standing on the word's block, and every
        // key in all.
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let on_new = standing.get(&NEW_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 904: 283/5/288/1,045 → 283/2/285/292/359. Under the word
        // `MAKE[4].t` and `1/79`'s two rows close on 23183 (the rock
        // arm, `docs/AI.md` §78); 686 keys past it close with the
        // draws; the new word's block stands 292 rows, the gap's.
        // Item 919: → 283/2/285/289/294. The pasture (`docs/AI.md` §79)
        // closes the word's three rows on 23421 and the 62 keys that
        // first parted past it; nothing parts on 23422..23433.
        // Item 1106: → 282/2/284/288/293, who=1's wealth, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84): `leftover[2:wealth]` 4990 against 4982 on 23177.
        // Item 1291 took (162, 2, 164, 164, 169) → (161, 2, 163, 163, 168): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (161, 2, 163, 163, 168).
        // Item 1330 took 46 (161/0/161/163/165 → 115/0/115/116/118): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, on_new, firsts.len()),
            (37, 0, 37, 37, 37),
            "the floor"
        );
    }

    pub(in super::super) fn east_indies_20000_s_squad_push_reads_the_record_its_slot_last_held() {
        let Some(inst) = install() else { return };
        let (Some(r269), Some(r277), Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run269-eastindies-turnword.txt"),
            dump("gamelog-run277-eastindies-blockedwalk.txt"),
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            eprintln!("skipping: no run269/run277/run54 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let slot_on = |path: &str, frame: i64, id: i64| {
            let text = crate::capture::read(path);
            let log = Log::parse(&text);
            let (_, b) = log
                .frames()
                .into_iter()
                .find(|(f, _)| *f == frame)
                .expect("the block");
            let g = crate::gamelog::groups(b)
                .into_iter()
                .find(|g| g.id == id)
                .expect("the slot");
            let last = crate::gamelog::last_group(b);
            let (units, _, _) = crate::gamelog::records(b, false);
            let facing: Vec<Option<i64>> = [64, 65, 66]
                .iter()
                .map(|&o| {
                    units
                        .iter()
                        .find(|u| u.who == 1 && u.o == o)
                        .and_then(|u| u.orders.last())
                        .and_then(|x| x.facing)
                })
                .collect();
            (g, last, facing)
        };
        // run269's last block: slot 69 is army 1's group.
        let (before, _, _) = slot_on(&r269, 19_664, 69);
        assert_eq!(
            (before.army, before.num, before.order_num, before.facing),
            (1, 16, 5, 1),
            "slot 69 on 19664: army 1's group, facing 1, order_num 5"
        );
        // run277's first block: the squad on 69, army −1, the old record's
        // `facing` and `order_num` one move on; its members' orders carry
        // the unmirrored layout.
        let (after, last, facing) = slot_on(&r277, 20_002, 69);
        let members: Vec<i64> = after.members.iter().map(|m| m.o).collect();
        assert_eq!(
            (
                after.army,
                members,
                after.stamp,
                after.order_num,
                after.facing
            ),
            (-1, vec![64, 65, 66], 20_000, 6, 1),
            "slot 69 on 20002: the squad pushed on tick 20000 over army 1's old record"
        );
        assert_eq!(last.get(1), Some(&69), "who=1's last_group on 20002");
        assert_eq!(
            facing,
            [Some(0), Some(0), Some(0)],
            "the original's orders: laid out square"
        );
        // This crate, run to the same block.
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..20_001 {
            built.tick();
        }
        let u65 = built.sim.unit_by_o(1, 65).expect("1/65");
        let s = built.sim.units[u65].group_ptr.expect("1/65 is seated");
        let ours: Vec<Option<bool>> = [64, 65, 66]
            .iter()
            .map(|&o| {
                let u = built.sim.unit_by_o(1, o).expect("the squad");
                built.sim.units[u]
                    .orders
                    .iter()
                    .find_map(|x| match &x.body {
                        sim::orders::Body::Move(m) => Some(m.facing),
                        _ => None,
                    })?
            })
            .collect();
        // **Item 880: the value diff on the word's frame.** This crate's
        // orders are laid out square too, `facing` 0 on all three, where
        // they were mirrored (1) before: the push keeps the slot's record
        // (`copy_group`), and the record's history is the original's since
        // `Group::kill` clears an emptied slot and an army's group is its
        // slot's record (`docs/GROUPS.md` §30).
        assert_eq!(ours, [Some(false); 3], "this crate's orders: square");
        // The squad's own record here: slot 69, the index's only record,
        // **`order_num` 6 and `facing` 1 on both sides** — army 1's old
        // record one move on. Before 880 it was fresh, `order_num` 1 and
        // `facing` 0; before 870 it sat beside army 0's orphan on the same
        // index, and the members pointed at army 1's slot 71 against the
        // original's 68.
        let recs: Vec<(i64, i32, bool, Vec<i16>)> = built
            .sim
            .pool_records(1, 5)
            .into_iter()
            .map(|(st, l)| (st.stamp, st.order_num, st.facing, l))
            .collect();
        assert_eq!(s, 4, "1/65 points at army 1's slot, 68 on both sides");
        assert_eq!(
            recs,
            [(20_000, 6, true, vec![64, 65, 66])],
            "this crate's slot 69: army 1's old record under the squad, alone"
        );
    }

    pub(in super::super) fn east_indies_pool_numbering_takes_who_1_s_building_groups() {
        let Some(inst) = install() else { return };
        let (Some(r45), Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run45-islands-groups.txt"),
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            eprintln!("skipping: no run45/run54 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&r45);
        let log = Log::parse(&text);
        // `(frame, id, buildings, stamp, members)` for who=1's live slots.
        let pool: Vec<(i64, i64, i64, i64, Vec<i64>)> = log
            .frames()
            .into_iter()
            .filter(|(f, _)| [2, 238, 239].contains(f))
            .flat_map(|(f, b)| {
                crate::gamelog::groups(b)
                    .into_iter()
                    .filter(|g| g.who == 1 && g.num > 0)
                    .map(move |g| {
                        let l = g.members.iter().map(|m| m.o).collect();
                        (f, g.id, g.buildings, g.stamp, l)
                    })
            })
            .collect();
        assert_eq!(
            pool,
            [
                (2, 64, 1, 1, vec![2005]),
                (2, 65, 0, 0, vec![0]),
                (238, 64, 1, 1, vec![2005]),
                (238, 65, 0, 0, vec![0]),
                (238, 66, 1, 176, vec![2000]),
                (239, 64, 0, 238, vec![0]),
                (239, 66, 1, 176, vec![2000]),
            ],
            "the original's who=1 pool: two building groups, then 1/0 onto 64"
        );
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // Block 239 is the state after tick 238.
        for _ in 0..239 {
            built.tick();
        }
        let scout = built.sim.unit_by_o(1, 0).expect("1/0");
        let rec = |s: u8| -> Vec<(i64, Vec<i16>)> {
            built
                .sim
                .pool_records(1, s)
                .into_iter()
                .map(|(st, l)| (st.stamp, l))
                .collect()
        };
        assert_eq!(
            (built.sim.pool_group_of(scout), rec(0), rec(2)),
            (64, vec![(238, vec![0])], vec![(176, vec![])]),
            "this crate: 1/0 moves onto 64, stamped 238, and 66 holds the \
             building group stamped 176"
        );
    }

    pub(in super::super) fn east_indies_pool_walk() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            return;
        };
        let names: Vec<String> = std::env::var("RON_POOL_WALK")
            .unwrap_or_default()
            .split(',')
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect();
        let stop: i64 = std::env::var("RON_POOL_WALK_STOP")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(24_000);
        let mut caps = Vec::new();
        for n in &names {
            let Some(pth) = dump(n) else {
                eprintln!("no {n}");
                continue;
            };
            caps.push((
                n.clone(),
                crate::capture::indexed::IndexedCapture::open(&pth).unwrap(),
            ));
        }
        // block -> (cap, frame index), first capture listed wins
        let mut at: std::collections::BTreeMap<i64, (usize, usize)> = Default::default();
        for (ci, (_, ix)) in caps.iter().enumerate() {
            for (fi, fr) in ix.frames().iter().enumerate() {
                at.entry(fr.number).or_insert((ci, fi));
            }
        }
        let last = (*at.keys().last().unwrap_or(&0)).min(stop);
        let loaded = crate::load::load(&inst).unwrap();
        // `RON_POOL_WALK_MAP=greatlakes` walks run53's game instead, with
        // Great Lakes' dumps in `RON_POOL_WALK` (item 880: the brief's
        // "Great Lakes on every step" wants this instrument, 881).
        let great_lakes = std::env::var("RON_POOL_WALK_MAP").is_ok_and(|m| m == "greatlakes");
        let mut built = if great_lakes {
            let Some(gl) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
                return;
            };
            let texts = sibling_texts();
            let text = crate::capture::read(&gl);
            let log = Log::parse(&text);
            let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
            let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
            let refs: Vec<&Initial> = inits.iter().collect();
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            build_sim(&loaded, &init, Tuning::RON)
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
        let mut reported = vec![false; caps.len()];
        let mut agreed = vec![0usize; caps.len()];
        let hist: Option<u8> = std::env::var("RON_POOL_WALK_HIST")
            .ok()
            .and_then(|x| x.parse().ok());
        let mut hist_last: Vec<String> = vec![String::new(); 64];
        for f in 0..last {
            built.tick();
            let n = f + 1;
            if let Some(hw) = hist {
                for s in 0..16u8 {
                    let recs = built.sim.pool_records(hw, s);
                    let line = recs
                        .iter()
                        .map(|(st, l)| {
                            format!(
                                "(on {} fc {} st {} fm {} {:?})",
                                st.order_num,
                                u8::from(st.facing),
                                st.stamp,
                                st.form,
                                l
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    if line != hist_last[s as usize] {
                        eprintln!("HIST {n} slot {} {line}", 64 * u32::from(hw) + u32::from(s));
                        hist_last[s as usize] = line;
                    }
                }
            }
            let Some(&(ci, fi)) = at.get(&n) else {
                continue;
            };
            let frame = caps[ci].1.frame_state(fi).unwrap();
            let mut bad = Vec::new();
            for t in &frame.units {
                if t.who > 1 {
                    continue;
                }
                let Some(u) = built.sim.unit_by_o(t.who as u8, t.o as i16) else {
                    continue;
                };
                if !built.sim.units[u].alive() {
                    continue;
                }
                let ours = built.sim.pool_group_of(u);
                if let Some(th) = t.group
                    && th != ours
                {
                    bad.push(format!("{}/{} ours {ours} theirs {th}", t.who, t.o));
                }
            }
            if std::env::var("RON_POOL_WALK_RECORDS").is_ok() {
                let raw = caps[ci].1.read_frame(fi).unwrap();
                let flog = Log::parse(&raw);
                let Some((_, blk)) = flog.frames().into_iter().find(|(x, _)| *x == n) else {
                    continue;
                };
                for g in crate::gamelog::groups(blk) {
                    if g.who > 1 {
                        continue;
                    }
                    let slot = (g.id - g.who * 64) as u8;
                    let recs = built.sim.pool_records(g.who as u8, slot);
                    let theirs_l: Vec<i64> = g.members.iter().map(|m| m.o).collect();
                    let (of, on, os, ol) = recs.first().map_or((0, 0, -1, vec![]), |(st, l)| {
                        (
                            i64::from(st.facing),
                            i64::from(st.order_num),
                            st.stamp,
                            l.iter().map(|&x| i64::from(x)).collect::<Vec<_>>(),
                        )
                    });
                    let mut d = Vec::new();
                    if of != g.facing {
                        d.push(format!("facing {of}/{}", g.facing));
                    }
                    if on != g.order_num {
                        d.push(format!("order_num {on}/{}", g.order_num));
                    }
                    if !d.is_empty() || recs.len() > 1 {
                        bad.push(format!(
                            "REC {} [{}] {} (stamp {os}/{}, list {ol:?}/{theirs_l:?})",
                            g.id,
                            recs.len(),
                            d.join(" "),
                            g.stamp
                        ));
                    }
                }
            }
            if bad.is_empty() {
                agreed[ci] += 1;
            } else {
                if !reported[ci] || std::env::var("RON_POOL_WALK_ALL").is_ok() {
                    eprintln!(
                        "POOL {} parting on block {n} after {} agreeing blocks: {}",
                        caps[ci].0,
                        agreed[ci],
                        bad.join(", ")
                    );
                }
                reported[ci] = true;
            }
        }
        for (ci, (name, _)) in caps.iter().enumerate() {
            if !reported[ci] {
                eprintln!("POOL {name} agrees on all {} blocks walked", agreed[ci]);
            }
        }
    }

    pub(in super::super) fn run261_s_gap_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_CLOSE.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_CLOSE.1;
        const CLOSE: i64 = EAST_INDIES_CLOSE_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run261",
            "gamelog-run261-eastindies-closeword.txt",
            WIDENING_EAST_INDIES_CLOSE,
            &[CLOSE],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        pin_eq!(
            leader_rows,
            1337942,
            "257 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        let rows: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > FIRST)
            .map(row)
            .collect();
        let on_close = standing.get(&CLOSE).cloned().unwrap_or_default();
        let escort: Vec<String> = (60..=76)
            .filter_map(|o| {
                on_close
                    .get(&(1, o, "group".to_string()))
                    .map(|r| format!("1/{o} {r}"))
            })
            .collect();
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        // **One key parts past the first block** in 257: the human
        // leader's `production_step` on 18601. Everything else standing
        // is the first block's, run253's floor carried on.
        // **Item 839**: who=1's wealth family agrees on the first block
        // now (the British territory tax, `docs/ECONOMY.md` §16), all but
        // `leftover[2:wealth]`, eight sixteenths over from the gap
        // 17753..18176, whose carry puts the purse one wealth over on
        // 18511.
        pin_eq!(
            rows,
            // Item 1106 closed `bucket[2:wealth]` 108 against 107 on
            // 18511 (and `leftover[2:wealth]` on 18428): the border
            // pass's economy flag (`docs/AI.md` §84). Item 1605 closed
            // the human's `production_step` on 18601: the human's sweep
            // arms the machine and `production_ai` disarms it (§169).
            Vec::<String>::new(),
            "the keys first parting past the window's first block"
        );
        // **R2 and R3** (the stanza's): the original's group 71 reads
        // `army 1` on 18682 and `army −1` on 18683 with 17 of its 18 still
        // listed, so the close is tick 18682, ours' tick, and the group
        // survives it. Army 0's new group takes fifteen of them on the
        // same tick. On 18683 every `group` row of the old army's units is
        // the same seat on both sides. Until item 870 it was one index
        // higher here (689): the orphan 72 against 71, army 0's 65 against
        // 64. Since 870 the index agrees too (`docs/GROUPS.md` §29).
        pin_eq!(
            escort,
            Vec::<String>::new(),
            "the old army's pointers on {CLOSE}"
        );
        pin_eq!(lists_part, [], "no army's list parts around the close");
        // **The floor**: every key standing on the first block (run253's
        // floor, carried), the keys past it, the rows standing on the
        // close's block, and every key in all. **Item 839** took it
        // 318/1/316/319 → 314/2/313/316: who=1's wealth `bucket`,
        // `income`, `rate` and `resources` leave the first block, and the
        // purse's one-wealth carry on 18511 arrives past it.
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (168, 1, 167, 169).
        // Item 1330 took 44 (160/1/159/161 → 116/1/115/117): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_close.len(), firsts.len()),
            // Item 870: → 285/2/284/287, the first block's 29 `group` rows
            // (`1/11` 64 against −1, `1/48`..`1/71` 72 against 71)
            // (`docs/GROUPS.md` §29).
            // Item 1106: → 284/1/283/285, who=1's wealth, the border pass's economy flag (`docs/AI.md` §84): `leftover[2:wealth]` on 18428 and `bucket[2:wealth]` on 18511.
            (37, 0, 37, 37),
            "the floor"
        );
    }

    pub(in super::super) fn run152_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_GATHER.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_GATHER.1;
        /// The block run152 was taken to widen, the word 10982's.
        const WORD_BLOCK: i64 = EAST_INDIES_GATHER_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            ..
        }) = widen_east_indies(
            "run152",
            "gamelog-run152-eastindies-gatherbuilding.txt",
            WIDENING_EAST_INDIES_GATHER,
            &[WORD_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            leader_rows,
            885020,
            "170 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // **The leader record under the word and on it**, both leaders,
        // both directions — and the move's value diff (item 613). Until the
        // fix 13 make-list rows parted from 10981, `create_units`' frame:
        // the Light Horse (t209, `num 2`) offered at `val 9999999` here
        // against 6945568. `offer_value`'s product wrapped negative here,
        // 64 × 40,500,000, where the original's is 256 × 40,500,000, which
        // wraps to 1,778,065,408: `check_income` answered 0x40 because the
        // price was 60/40 against 53 food, where `get_cost` takes Horses'
        // 15% off a Stable unit — 51/34, two affordable. On 10982 the
        // original then sorted the Barracks above the Light Horse, so
        // `use_market`'s `need` differed and it spent a third draw. One
        // row stands, §52.2's `city` shift. The Mine's value on 10982,
        // 316800 against 310400, went with item 629 (parked 623): the
        // Merchants' seat puts the leader's rates on the original's
        // cadence (`docs/MERCHANT.md` §3.2). The site list's fourteen rows
        // on 10976 went with item 688: a founded city wears the site values
        // down (`City::fix_world_vals`, `docs/AI.md` §67), and the sweep
        // agrees.
        let leader: Vec<String> = firsts
            .iter()
            .filter(|((_, o, _), (f, _))| *o == -1 && (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(leader, Vec::<String>::new(), "the leader record");
        // **Who changes animation, both sides** (`docs/COMBAT.md`
        // §44.2.1). Until the fix the citizen `1/11` turned on 10983 in the
        // original only, sent to the original's Mine.
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // **The word's own block, every unit.** Until the fix 21 rows parted
        // here: the Mine `1/2019` stood on (33408, 37248), on range 2 that
        // `1/2018` already mines, with no gather list, against the
        // original's (36480, 35136) on range 0 with 170 tiles; and the
        // citizens `1/11` and `1/36` took other orders. `calc_gather`'s
        // survey refuses a range whose tiles are gathered from.
        let on_word = firsts
            .iter()
            .filter(|((_, o, _), (f, _))| *o != -1 && (WORD_BLOCK - 2..=WORD_BLOCK).contains(f))
            .count();
        pin_eq!(on_word, 0, "the unit rows on the word's blocks");
        // **The floor**: 251 keys standing on the window's first block
        // (run149's residue on its last), 280 before the word — `peasants`
        // and `1/36` from 10956, the SITE list on 10976 — and 282 in all.
        // Item 629 took it from 278/309/312: the two Merchants' seats on
        // the first block, and with them the food and metal a unit off from
        // 10881 and 10900 and the Mine's value (`docs/MERCHANT.md` §3.2).
        // Before item 613's
        // fix the floor was 278/409/616, and before item 620's 278/409/412:
        // on 10959 the citizen `1/11` took the Mine `1/2018`'s tile (173,
        // 183), which `1/6` already held, where the original took (170, 182)
        // — 72 rows of the Mine's list, one slot out from index 7, and 28 of
        // `1/11`'s walk after it. A Mine's `dist_mod` weighs 3, not 10, on a
        // list under `MTN_TINY_SIZE` (`docs/ORDERS.md` §6.4).
        let under = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // **Item 632 took three** (251/280/282 → 248/277/279): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (248/277/279 → 244/273/275): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 688 took thirty-nine** (244/273/275 → 219/234/236), all who=1 `SITE` rows and nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // **Item 718 took two (219/234/236 → 219/232/234)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 723 took one (219/232/234 → 219/231/233)**: the scout's
        // formation mirror (parked 275), an equal group's record kept.
        // **Item 736 added one (219/231/233 → 219/232/234)**: the new `mirror` row
        // (`unit_masks & 2`, `docs/GROUPS.md` §26.4) parts on `1/31` from 10875, ours 1
        // against 0, and it parts so with or without item 736's fix: a turn
        // this crate flips the bit on and the original does not.
        // **Item 803 took three (219/232/234 → 219/229/231)**: run149's
        // three on `1/31`'s disembark on 10875, the same game.
        // **Item 850 took one from the first block (219 → 218)**: `1/31`'s
        // `g.avg_speed[0]`, 15 against 11 standing from the first block,
        // agrees until the disembark on 10875, where it
        // reads ours 0 against 11 (`docs/ANIM.md` §15).
        // **Item 870 took seven (218/229/231 → 211/222/224)**: the first
        // block's `group` rows (`1/0` 70 here against 69, `1/11` 64
        // against −1, `1/32`..`1/38` 68 against 66): who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 1164 took three (148/154/156 → 148/151/153): `1/31`'s
        // disembark on 10875, as run149's (`docs/TRANSPORT.md` §6.4).
        // Item 1291 took (148, 151, 153) → (148, 150, 152): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (148, 150, 152).
        // Item 1330 took 29 (141/143/144 → 112/114/115): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under, firsts.len()), (41, 41, 41), "the floor");
    }

    pub(in super::super) fn run149_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_MERCS.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_MERCS.1;
        /// The block run149 was taken to widen, the word 10782's.
        const WORD_BLOCK: i64 = EAST_INDIES_MERCS_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            ..
        }) = widen_east_indies(
            "run149",
            "gamelog-run149-eastindies-animal.txt",
            WIDENING_EAST_INDIES_MERCS,
            &[WORD_BLOCK],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        pin_eq!(
            leader_rows,
            780900,
            "150 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        pin_eq!(
            missing,
            std::collections::BTreeSet::new(),
            "no key unprinted"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // **Under the word, 10776: the census, and the move's value diff**
        // (item 608). Until the fix player 1's `reg_gather_slots` of its
        // home region (11 in the dump's numbering) parted here, 17 against
        // 20, on the first census sweep after the Mine `1/2018` was placed
        // on 10583 — no sweep runs between, on either side. The original's
        // `City::count_gather_slots@00737dc0` walks the city's chain and
        // sums every gather building's `gather_max`, finished or not: run150's
        // packet at 10765 reads the unstarted Mine (`flags 1`) holding
        // `gather_max 3` in city 1's chain. This crate skipped a building
        // that was not `active`. Now the row is gone; `SITE[7]`'s value,
        // which parted on the same sweep before the fix and spends no
        // draw, stayed (`docs/AI.md` §61) — **until item 688**: 26 against
        // 13 was the halving a founded city makes of the site values round
        // it (`City::fix_world_vals`, `docs/AI.md` §67), and nothing under
        // the word parts now.
        let census: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (10_734..WORD_BLOCK - 2).contains(f))
            .map(row)
            .collect();
        pin_eq!(census, Vec::<String>::new(), "the census under the word");
        // **The word's blocks, 10781..10783, both directions.** Until the
        // fix 20 rows parted here: on 10781, `create_units`' frame, the
        // original offered the Citizen (type 50) with `num 4` at `val
        // 3291428` — the value squares the count — so it ranked in slot 1
        // over the Dromon and the Galley and duplicated into slot 5, where
        // this crate offered `num 1` at 205714 third. The count is the
        // city's gatherer deficit bounded by its open slots, the Mine's
        // three among them. On 10782 `make_stuff` bought the four and, with
        // two Citizen slots to expire, spent two `make_stuff+0x63d` against
        // this crate's one: 10 draws against 11, parting at index 5, ours
        // `Animal::do_idle+0x83` a place early, and city `1/2007` queued 4
        // against 1. Now one row stands, §52.2's `city` shift.
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (WORD_BLOCK - 2..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(on_word, Vec::<String>::new(), "the make list");
        // **Who changes animation, both sides** (`docs/COMBAT.md`
        // §44.2.1). Until the fix gaia's `8/3` turned on 10783 and 10784 in
        // the original only: the animal whose `do_idle` roll this crate spent
        // from the original's `make_stuff` draw. Now it turns on both.
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // **The floor**: 280 keys standing on the window's first block —
        // run143's residue on its last blocks (the human's city record,
        // `form`, the hit counts of the first citizens, the scout `1/0`'s
        // group, `1/19` and `1/20` a unit off) — and two more before the
        // word: a unit of food on 10733, and `SITE[7]` on 10776. Past it
        // the gather clock (10784), the human's `production_step` (10801)
        // and the transport rider `1/31`'s order (10875). Before the fix
        // the floor was 280/283/506. Item 629 took it 280/282/291 →
        // 251/252/261: `1/19` and `1/20` are on their tile corners now
        // (`docs/MERCHANT.md` §3.2), and the city and rate rows with them.
        let under = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // **Item 632 took three** (251/252/261 → 248/249/258): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (248/249/258 → 244/245/254): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 688 took twenty-five** (244/245/254 → 220/220/229), all who=1 `SITE` rows and nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // **Item 736 added one (220/220/229 → 220/220/230)**: the new `mirror` row
        // (`unit_masks & 2`, `docs/GROUPS.md` §26.4) parts on `1/31` from 10875, ours 1
        // against 0, and it parts so with or without item 736's fix: a turn
        // this crate flips the bit on and the original does not.
        // **Item 803 took three (220/220/230 → 220/220/227)**: the
        // disembarked `1/31` on 10875 — `dest_angle` ours 35127296 against
        // 404029440, `orders_x/y` ours (40344, 22680) against (41638,
        // 34561) — agreeing now: `eject_contents`' `update_action` is step
        // 1's and `come_out`'s `set_angle` leaves `+0x58`
        // (`docs/TRANSPORT.md` §15).
        // **Item 850 took one from the first block (220/220 → 219/219)**:
        // `1/31`'s `g.avg_speed[0]`, 15 against 11 standing from the first block,
        // agrees until the disembark on 10875, where it
        // reads ours 0 against 11 (`docs/ANIM.md` §15).
        // **Item 870 took seven (219/219/227 → 212/212/220)**: the first
        // block's `group` rows (`1/0` 70 here against 69, `1/11` 64
        // against −1, `1/32`..`1/38` 68 against 66): who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 1106 took one (212/212/220 → 211/211/219): who=0's `gather_stamp`, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84), 10336 against 10344 on 10730.
        // Item 1164 took three (148/148/155 → 148/148/152): `1/31`'s figure
        // `avg_speed` ashore, 0 against 11 on 10875, and what it turned
        // with, agree — a passenger lands at the average it boarded with
        // (`docs/TRANSPORT.md` §6.4).
        // Item 1291 took (148, 148, 152) → (148, 148, 151): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (148, 148, 151).
        // Item 1330 took 29 (142/142/144 → 113/113/115): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under, firsts.len()), (41, 41, 41), "the floor");
    }

    pub(in super::super) fn run143_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_BARK.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_BARK.1;
        /// The block run143 was taken to widen, the word 10398's; item 588
        /// moved the word past it.
        const BARK_BLOCK: i64 = EAST_INDIES_BARK_BLOCK;
        /// The word's block: its frame writes the next one.
        const WORD_BLOCK: i64 = EAST_INDIES_MINE_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            ..
        }) = widen_east_indies(
            "run143",
            "gamelog-run143-eastindies-bark.txt",
            WIDENING_EAST_INDIES_BARK,
            &[BARK_BLOCK, WORD_BLOCK],
            false,
        )
        else {
            return;
        };
        use std::collections::BTreeSet;
        pin_eq!(blocks, (TAIL - FIRST + 1) as usize, "the walk is whole");
        // **The leader half is whole**: every key `leader::rows` carries is
        // on run143's record, both leaders, all 360 blocks — and, since item
        // 592, the census's 21 per-region arrays (`leader::REGION_ARRAYS`,
        // 1,531 keys a leader), which no widening had read.
        pin_eq!(
            leader_rows,
            1874160,
            "360 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        pin_eq!(missing, BTreeSet::new(), "no key unprinted");
        // **The old word's blocks, 10397..10399, and the move's value
        // diff** (item 588). Until the fix they held run99's eight rows:
        // sim-frame 10398 spent 4 draws here against 5, parting at index
        // 0 on the original's `Guy::set_anim+0x97a < Unit::do_idle+0x7d`,
        // because the Bark `1/34` had walked to the navy 21 units behind
        // the original's since its first step, halved by `do_move`'s
        // waypoint probe finding Trireme `1/32` soft beside the muster.
        // The original's `detect_unit_collision` never scans for a ship on
        // a probe that does not ask for `boats` (`docs/COLLISION.md` §13).
        // Nothing parts here now.
        let on_old: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (BARK_BLOCK - 2..=BARK_BLOCK).contains(f))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            on_old,
            Vec::<String>::new(),
            "the Bark arrives with the original's"
        );
        // **The word's blocks, 10581..10583, both directions.** Sim-frame
        // 10582 is player 1's `make_stuff`, and it places its Mine `1/2018`
        // a tile off: on 10583 at (·, 34944) against (·, 35136), with its
        // gather ring, and the Citizen `1/6` walks to it on a different
        // path. Until item 592 the make list parted under it on 10581 —
        // the original's Citizen (type 50, cat 5, `val 1714`) in slots 3
        // and 5 against this crate's two Scholars at `val 0` in slot 3,
        // and on 10582 the vals of slots 1, 2, 4 and 7 — because the
        // census counted the barge rider `1/31` free (below). Now the lists
        // agree but for §52.2's `city` shift and the Mine's own value, slot
        // 4's `val` on 10582, `create_buildings`' frame: 1,616,000 against
        // 1,584,000. That is city `1/2007`'s `open` one high here —
        // `filled` 33 against 34, residue run58 pins from 1819 — and a payoff probe
        // that set the city's picture to the original's from 7975 closed
        // the row and moved no draw (`docs/AI.md` §58.4). Item 629 closed
        // it: the Merchants' four-tile footprints are the city's missing
        // `filled` (`docs/MERCHANT.md` §3.2).
        let on_word: Vec<String> = firsts
            .iter()
            .filter(|(k, (f, _))| (WORD_BLOCK - 2..WORD_BLOCK).contains(f) && k.1 == -1)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(on_word, Vec::<String>::new(), "the make list");
        let placed: Vec<String> = firsts
            .iter()
            .filter(|(k, (f, _))| {
                *f == WORD_BLOCK
                    && ((k.1 == 2018 && k.2 == "build:y_internal") || (k.1 == 6 && k.2 == "pos"))
            })
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        // Item 597: empty. The Mine stood at y 34944 against 35136, and its
        // builder `1/6` two units off, until a mine's reach was measured to
        // the nearest solid mountain **cell** (run144's packet,
        // `docs/AI.md` §59). The spiral still scored eleven candidates
        // against seven, and the jitter landed where the original's does.
        // Item 604 laid the generator's placed templates down (§60): the
        // four residue cells are refused, the spiral scores seven against
        // seven, and the word left this window.
        pin_eq!(placed, Vec::<String>::new(), "the site and its builder");
        let per_block: Vec<(i64, usize)> = (WORD_BLOCK - 2..=WORD_BLOCK)
            .map(|b| (b, firsts.values().filter(|(f, _)| *f == b).count()))
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(10581, 2), (10582, 1), (10583, 0)].
        pin_eq!(
            per_block,
            // Item 597: 98 → 80, and the 80 were the Mine's gather list,
            // 40 `tx` and 40 `ty`: its order, which the shuffle drew from
            // a stream four spiral draws apart. Item 604: **80 → 0**, the
            // move's value diff — the spiral spends the original's seven
            // draws, so the shuffle reads the original's stream and the
            // list comes out in the original's order. The four left are
            // §52.2's `city` shift, under the word and not its cause.
            // Item 629: 4 → 3, the Mine's value (`docs/MERCHANT.md` §3.2).
            [(10581, 0), (10582, 0), (10583, 0)],
            "the word's blocks, counted"
        );
        let one_sided: Vec<(i64, i64, i64, bool, bool)> =
            changed.iter().filter(|c| c.3 != c.4).copied().collect();
        // **Who changes animation, both sides** (`docs/COMBAT.md`
        // §44.2.1), on 10397..10401 and 10581..10585: on the old word the
        // Bark stands and the Scholar `1/24` turns a page on both sides
        // now. On the new one the only one-sided change is the human's
        // `0/4` on 10585, whose move order parts on 10584, a block past
        // the word — downstream of it, not under it.
        // Item 597: empty. `0/4`'s one-sided change on 10585 went with the
        // Mine's site (§59): its move order parted on 10584 behind it.
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // **The census under the word, 10576, both directions** (item 592),
        // and the move's value diff. Player 1's sweep runs on sim-frame
        // 10575 and counted one citizen differently: `1/31`, which boarded
        // the transport barge `1/36` on 10486 on both sides and rides it to
        // (40344, 22680). The original counts a citizen off the map through
        // its container: a sea-domain container whose order is a move makes
        // it an `xport` peasant in the region of the move's point, 5 in the
        // dump's numbering, and the `reg_active` family counts it there too.
        // Until the fix this crate counted it on the map where it boarded,
        // free, in the home region 11 — eight rows here: `free_peasants`
        // 2/1, `xport_peasants` 1/2, `reg_peasants[11]` 14/13,
        // `reg_free_peasants[11]` 1/0, `reg_active[11]` 27/26,
        // `reg_active[5]` 0/1, `reg_xport_peasants[5]` 0/1 and city
        // `1/2000`'s `free` 1/0 (`docs/AI.md` §58). No unit's container
        // parts anywhere in the window (`inside`, the outermost container
        // both sides).
        let census: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == WORD_BLOCK - 7)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(census, Vec::<String>::new(), "the peasant census");
        pin!(
            !firsts.keys().any(|k| k.2 == "inside"),
            "every unit's container agrees"
        );
        // **The floor under the word**: 254 keys standing on the window's
        // first block (the human leader's census, the site list, the make
        // list's `city` shift, the seated Scholars' birth records and the
        // navy's group numbers, among the record rows run99 carries whole),
        // and 28 more before the word: the make list's `city` and a unit of
        // metal and food on 10381/10382; the scout `1/0` a unit off on
        // 10383..10407; the human's `production_step` on 10401; the seated
        // Scholars `1/29` (10437) and `1/37` (10548) with their four-row
        // birth records; the Bark `1/35`'s group number on 10466; and
        // `1/31`'s speed and `1/36`'s form and a path flag on 10486. Until
        // item 592 the peasant census on 10576 stood here too, and until
        // item 588 the Bark `1/34`'s walk, when the floor was 269/284/830.
        //
        // Item 592's widening added 19 on the first block: the human
        // leader's per-region census (18 rows: `reg_active`, `reg_cities`,
        // `reg_gather_slots`, `reg_gatherers`, `reg_land` and `reg_peasants`
        // of its home region, and `strategy` over twelve regions — this
        // crate keeps no census for the human) and player 1's `reg_land`
        // of its home region, 91 against 90, which is city `1/2007`'s
        // `filled` (§58.4). Its fix took the census's eight rows off 10576
        // and the make list's thirteen off the word; the floor was
        // 273/309/772 with the widening and before the fix.
        //
        // Item 597 moved nothing under the word and took **360** keys off
        // the blocks past it, 753 → 393: the Mine on the original's site,
        // its builder on the original's point, and what followed them.
        // Item 604 moved the word out of this window (10582 → 10782) and
        // took the Mine's 80 gather-list rows, 393 → **313**: nothing
        // under the old word moved, and nothing past it parted anew.
        // Item 629 took it 273/301/313 → 246/272/283: the Merchants `1/19`
        // and `1/20` on their tile corners, `reg_land` and `filled` with
        // their footprints, and the Mine's value (`docs/MERCHANT.md` §3.2).
        let under = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // **Item 632 took three** (246/272/283 → 243/269/280): three computer
        // citizens' `form_mod`, ours −1 against the original's 50 from the
        // first block, now agreeing. `action_move_near` writes the width
        // twin on every member, citizens too (`docs/GROUPS.md` §24).
        // **Item 661 took four** (243/269/280 → 239/265/276): who=1's two cities'
        // `trade_val` and `vans.length`, which were compared against a
        // literal 0 and agree once the rows read the sim's own (`docs/AI.md`
        // §63).
        // **Item 688 took twenty-five** (239/265/276 → 214/240/251), all on
        // the first block and all who=1 `SITE` rows, with nothing
        // arriving: a founded city wears the site values down
        // (`City::fix_world_vals`, `docs/AI.md` §67).
        // **Item 718 took two (214/240/251 → 214/240/249)**: trained units' `orders_x`/`orders_y`
        // on their exit block, `come_out`'s `update_action` on the captain
        // (`docs/ORDERS.md` §29), and nothing arrived.
        // **Item 736 added one (214/240/249 → 214/241/250)**: the new `mirror` row
        // (`unit_masks & 2`, `docs/GROUPS.md` §26.4) parts on `1/0`, the scout (parked 275's family), ours 1
        // against 0, and it parts so with or without item 736's fix: a turn
        // this crate flips the bit on and the original does not.
        // **Item 822 took one (214/241/250 → 213/241/250)**: who=1's
        // `known_rares`, 2 against 0 on the first block, now agreeing — a
        // border fix zeroes `reg_known_rares` until the next census
        // (`docs/AI.md` §76). The two counts past it are unchanged: the
        // key parts again on 10384, where ours' recompute of 10383 sums
        // the census's recount (2) and the original's waits for 10439 —
        // who=1's `gather_stamp`, 10335 against 10343 from the first
        // block, is the recompute's timing and not the zero's.
        // **Item 850 took one (213/241/250 → 213/240/249)**: `1/31`'s
        // `g.avg_speed[0]`, 15 against 11 on 10486,
        // agrees (`docs/ANIM.md` §15).
        // **Item 870 took six from the first block and nine in all
        // (213/240/249 → 207/232/240)**: `group` rows (`1/0` 66 here
        // against 65, `1/11` 64 against −1, `1/35`, `1/38` 68 against 66)
        // and the scout's `mirror` and `order:move.facing`: who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 1106 took one (207/232/240 → 205/231/239): both leaders' `gather_stamp` on the first block, 10380, close, and who=1's parts first on 10384 instead, 10383 against 10343: a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84).
        // Item 1164 took twenty (141/167/175 → 127/147/155), fourteen of
        // them from the first block: the landings walked from run54's
        // start, each passenger ashore at the average it boarded with and
        // an AI's ordered where it lands (`docs/TRANSPORT.md` §6.4).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (127, 147, 155).
        // Item 1330 took 26 (127/142/147 → 101/114/117): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        // Item 1407 took one (101/114/117 → 101/113/116): a boat's embark flag (`docs/AI.md` §105).
        pin_eq!((first, under, firsts.len()), (37, 40, 41), "the floor");
    }

    pub(in super::super) fn run73_s_window_clocks_are_the_original_s() {
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
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let setup = source.read_replay_setup().unwrap();
        let setup_log = Log::parse(&setup);
        let setup_init = setup_log.initial().unwrap();
        let (seeds, animations, guys) = source
            .with_replay_initial(|observed| {
                (
                    observed.frame_seeds.clone(),
                    observed.anim_lengths.clone(),
                    observed.frame_guys.clone(),
                )
            })
            .unwrap();
        let mut r73_source = crate::capture::indexed::IndexedCapture::open(&r73).unwrap();
        let theirs = r73_source
            .with_replay_initial(|initial| initial.frame_guys.clone())
            .unwrap();
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert!(
            window.first().is_some_and(|&n| n <= 5_565)
                && window.last().is_some_and(|&n| n >= 5_578)
                && window.len() >= 14,
            "run73's `DUMP_ALL` window, as sim-frames: {window:?}"
        );
        let last = *window.last().unwrap();
        with_sibling_initials(|refs| {
            let mut init = setup_init;
            init.frame_seeds = seeds;
            init.anim_lengths = animations;
            init.frame_guys = guys;
            init.frame_seeds.retain(|(frame, _)| *frame <= last);
            init.frame_guys.retain(|(frame, _)| *frame <= last);
            borrow_from_siblings(&mut init, refs);
            borrow_pasture(&mut init, &trace);
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
                        x.alive()
                            && i64::from(x.owner) == state.who
                            && i64::from(x.index) == state.o
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
        });
    }
}
