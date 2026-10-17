pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run85_s_window_is_the_east_indies_word_frame() {
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
        // window carries from 6883 on — gone since item 629, which seats
        // a deployed Merchant on its tile corner (`docs/MERCHANT.md` §3.2).
        assert_eq!(
            parted,
            std::collections::BTreeMap::new(),
            "run85's parted set"
        );

        // That the crate now *spends* 7448's two draws is pinned where
        // the word is, not here: [`LONG_WORD_EAST_INDIES`] is 7529, and
        // `run54_s_24000_frames…` asserts the floor on the same game.

        // `1/19` is the unpacked Merchant standing on its trade-post
        // spot. Until item 629 it was **a constant (24, 24), on every
        // block**: the original moves it onto the tile corner (32256,
        // 36864) = 192 x (168, 192) when its unpack ends, and this crate
        // left it where it walked to. Now it is on the corner here too
        // (`docs/MERCHANT.md` §3.2).
        let merchant: Vec<(i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| (d.who, d.o) == (1, 19))
            .map(|d| (d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            merchant.is_empty(),
            "`1/19` is off its tile corner again: {} rows, {:?}",
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

    pub(in super::super) fn run88_s_window_is_east_indies_word_frame() {
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
            // Item 629: the two Merchant constants go too — nothing parts
            // below the word.
            Vec::<((i64, i64), i64)>::new(),
            "run88's parted set, whole"
        );

        // `1/19` is run85's row one window along: the unpacked Merchant
        // standing on its trade-post spot. Until item 629 it was **a
        // constant (24, 24) on every block**, and `1/20` the same from its
        // own unpack on 7663; `cast_unpack`'s merchant arm seats both on
        // their tile corners now (`docs/MERCHANT.md` §3.2).
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
            nineteen.is_empty(),
            "`1/19` is off its tile corner again: {} rows, {:?}",
            nineteen.len(),
            nineteen.first()
        );
        assert!(
            twenty.is_empty(),
            "`1/20` is off its tile corner again: {} rows, {:?}",
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
        // was the two standing Merchant constants and nothing else, on a
        // 327-block window, and item 629 took those: **the closing block
        // is empty** (`docs/MERCHANT.md` §3.2).
        let residue: Vec<(i64, i64, i32, i32, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .filter(|d| d.frame == 7_816)
            .map(|d| (d.who, d.o, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y))
            .collect();
        assert_eq!(
            residue,
            Vec::<(i64, i64, i32, i32, i32, i32)>::new(),
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
            // Item 723 took `1/0`'s 101 `Move.facing` rows (parked 275).
            // Item 1115 took `1/18`'s three: `Action` 326, `Flags` 326 and
            // `Move.angle` 324 — the trade order's bit 4 and its leg's
            // bearing (`docs/CARAVAN.md` §11.3).
            Vec::<((i64, i64, &str), usize)>::new(),
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

    pub(in super::super) fn run90_s_window_is_east_indies_shuffle() {
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
        // 2,860 → 3,080 on item 629: the column is counted where the
        // positions agree, and the seated Merchants `1/19` and `1/20` now
        // do (`docs/MERCHANT.md` §3.2).
        assert!(
            sd_compared == 3_080 && sd_diverged.is_empty(),
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
        // **Item 857 took the last two**: on 7820 the original's `1/7` kept
        // `collide_o` 6 and `collide_who` 1 across the blocker probe that
        // abandons its search on 7819, and this crate cleared them. The
        // probe is the quick form, which writes nothing
        // (`docs/COLLISION.md` §18).
        assert_eq!(
            seven_coll,
            vec![],
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
        // left to explain. **Item 857 explained it**: the blocker probe is
        // the quick form and writes nothing (`docs/COLLISION.md` §18), and
        // the window's collision fields agree whole.
        let all_coll: Vec<(i64, i64, i64, &str, i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter())
            .map(|d| (d.frame, d.who, d.o, d.field, d.ours, d.theirs))
            .collect();
        assert_eq!(
            all_coll,
            vec![],
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
        // block, and two rows apart — **and none since item 629**, which
        // seats both Merchants on their tile corners (`docs/MERCHANT.md`
        // §3.2).
        assert_eq!(
            parted
                .iter()
                .map(|(&k, &f)| (k, f))
                .collect::<Vec<((i64, i64), i64)>>(),
            Vec::<((i64, i64), i64)>::new(),
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

    pub(in super::super) fn run86_s_window_is_the_transport_ride() {
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
        // And since item 629 not the Merchant `1/19` either, which stood
        // (24, 24) off its tile corner from its unpack until
        // `cast_unpack`'s merchant arm seated it (`docs/MERCHANT.md` §3.2).
        assert_eq!(
            parted,
            std::collections::BTreeMap::new(),
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

    pub(in super::super) fn run82_s_window_is_the_east_indies_ride_s_run_up() {
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
        // from that frame to the last block. **Item 629 built that
        // teleport** — `cast_unpack`'s merchant arm, which seats the trader
        // on `tile × 0xc0` and blocks its square (`docs/MERCHANT.md`
        // §3.2) — and nothing parts in the window.
        assert_eq!(
            parted,
            std::collections::BTreeMap::new(),
            "run82's parted set"
        );
        let merchant: Vec<(i64, i32, i32)> = report
            .frames
            .iter()
            .flat_map(|f| f.diverged.iter())
            .map(|d| (d.frame, d.ours.x - d.theirs.x, d.ours.y - d.theirs.y))
            .collect();
        assert!(
            merchant.is_empty(),
            "`1/19` is off its tile corner again: {} rows, {:?}",
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
        // Item 629: the unpack's constant is gone with the seat.
        assert_eq!(
            off,
            std::collections::BTreeMap::new(),
            "run82's shutdown dump, every unit of it — `1/13`'s (11, 7) went              with the age snap (item 271, `docs/TECH.md`)"
        );
    }
}
