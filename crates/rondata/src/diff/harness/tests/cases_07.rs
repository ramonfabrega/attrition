pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run74_s_window_is_where_great_lakes_merchant_turns() {
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

    pub(in super::super) fn run75_s_window_is_the_scout_s_walk_down_the_river() {
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

    pub(in super::super) fn run76_s_window_is_the_ai_squad_s_march() {
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
        // 8,032 → 8,387 on item 629: the column is counted where the
        // positions agree, and Great Lakes' seated Merchants now do
        // (`docs/MERCHANT.md` §3.2).
        assert!(
            sd_compared == 8_387 && sd_diverged.is_empty(),
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
            0,
            "the group-id stand-in is gone since item 1926: the pool slot is the army's own"
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

    pub(in super::super) fn run79_s_window_is_the_half_step_s_second_sample() {
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

    pub(in super::super) fn run84_says_great_lakes_6994_belongs_to_the_second_squad() {
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

    pub(in super::super) fn great_lakes_6994_issues_the_second_squad_s_walk_to_the_army() {
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

    pub(in super::super) fn run39_s_long_trace_says_where_the_second_map_s_word_parts() {
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

    pub(in super::super) fn run39_s_islands_game_is_the_second_map_s_score() {
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

    pub(in super::super) fn the_original_s_own_run_is_still_matched_frame_for_frame() {
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

    pub(in super::super) fn run83_s_window_is_the_last_great_lakes_hole() {
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
        // **Item 629**: `1/24` and `1/25` are Merchants, and their
        // (24, 24) was the seat `cast_unpack`'s merchant arm gives them on
        // their tile corners (`docs/MERCHANT.md` §3.2); neither parts now.
        assert_eq!(
            inherited,
            Vec::<(i64, i64)>::new(),
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
            1,
            "run83's parted set should be the quit frame's `0/3` alone \
             since item 629: {parted:?}"
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
                // Item 723 added the scout's formation mirror (parked 275),
                // 0 against the original's 1 from 6864, once `push_group`
                // keeps an equal group's record: non-scoring, no draw.
                // Item 870 took it: the AI's building groups number who=1's
                // pool as the original's do, so the scout's push on this
                // map takes a fresh slot where the original's does
                // (`docs/GROUPS.md` §29).
                // Item 1115 took the caravan `1/23`'s three: its trade
                // order carries no bit 4 and its leg faces its bearing
                // (`docs/CARAVAN.md` §11.3).
                // Item 1926 took the three `Group.id` stand-ins: the army's
                // id is its pool slot, which the original's agrees on.
            ],
            "run83's order rows are the caravan's three, carried into the \
             window"
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
}
