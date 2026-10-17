pub(super) use bodies::*;

mod bodies {
    use super::super::*;

    pub(in super::super) fn run166_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_WRAP.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_WRAP.1;
        /// The block run166 was taken to widen, the word 13640's.
        const WORD_BLOCK: i64 = EAST_INDIES_WRAP_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            housed,
            standing,
            ..
        }) = widen_east_indies(
            "run166",
            "gamelog-run166-eastindies-incword.txt",
            WIDENING_EAST_INDIES_WRAP,
            &[WORD_BLOCK],
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
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        pin_eq!(
            leader_rows,
            624720,
            "120 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        // **Under the word and on it, both directions.** The leader rows
        // are who=1's make list (`MAKE[].city`) and the human's
        // `production_step`, the families run159's floor carries; `1/53` is
        // a newborn with `form` −1 against 0 and pool group 69 against 68
        // (parked 674's group id).
        let under: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            under,
            // Item 1605 took the human's `production_step` on 13601
            // (`docs/AI.md` §169).
            Vec::<String>::new(),
            "the rows under the word and on it"
        );
        // **The word's own blocks, every row new on them** (`standing`
        // keeps every row within two blocks of the word; these did not
        // stand on 13639). **The move's value diff, both halves (the
        // word's delta is in `LONG_WORD_EAST_INDIES`' comment):** until
        // the fix the Galley `1/32` — a Trireme upgraded inside the
        // window, on both sides — entered `CHAR_DEFAULT` on 13638 with
        // `end_time` 3 here and 20 there, and wrapped a third time on
        // frame 13640. Four `1/32` rows stood here, `cur_anim`,
        // `cur_time` and `last_time` on 13641 and `end_time` on 13638,
        // and one of the sheep `8/1`'s: its wrap came one draw later in
        // the stream and rolled `a0` for `a1`. `unit_graphics.xml` cites
        // "Galley Default" and `anim_graphics.xml` defines "Galley
        // default"; the original resolves it under `_wcsicmp`
        // (`docs/ANIM.md` §12), and this crate read a missing slot.
        let before = standing.get(&(WORD_BLOCK - 2)).cloned().unwrap_or_default();
        let on_word: Vec<String> = (WORD_BLOCK - 1..=WORD_BLOCK)
            .flat_map(|b| {
                let before = &before;
                standing
                    .get(&b)
                    .into_iter()
                    .flatten()
                    .filter(move |(k, _)| !before.contains_key(*k))
                    .map(move |((w, o, what), row)| format!("{b} {w}/{o} {what}: {row}"))
            })
            .collect();
        pin_eq!(
            on_word,
            Vec::<String>::new(),
            "the rows new on the word's blocks"
        );
        let one_sided: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        // Until the fix `1/32` and `8/1` changed animation on 13641 in
        // ours alone.
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        pin_eq!(housed, 1_560, "housed unit-blocks the row compared");
        // **The floor**: 295 keys standing on the window's first block —
        // everything that parted in the 1,741 blocks since run159, which no
        // capture prints — 307 before the word, 307 in all. It was
        // 295/308/495 before the fix (`ad71c9b`): `1/32`'s `end_time` under
        // the word, and 187 rows past it that the extra draw parted — `0/4`
        // from 13677, the scout `1/0` from 13642, `1/1`, `1/9` and the rest.
        // Nothing now parts past the word to the window's end. **Item 688
        // took twenty-five** (295/307/307 → 270/282/282), all who=1 `SITE`
        // rows on the first block: a founded city wears the site values
        // down (`City::fix_world_vals`, `docs/AI.md` §67).
        let under_n = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 718: trained units' `orders_x`/`orders_y` on their exit block
        // now agree (`come_out`'s `update_action`, `docs/ORDERS.md` §29).
        // Item 723 took one (270/280/280 → 269/279/279): the scout's
        // formation mirror (parked 275), an equal group's record kept.
        // Item 752 took two (269/279/279 → 267/277/277): who=1's two
        // cities' `gatherers` on the first block, counted in the building's
        // city (`docs/AI.md` §73).
        // Item 870 took ten on the first block and eleven in all (267/277/277
        // → 257/266/266): the `group` rows — `1/0` 70 here against 69,
        // `1/11` 64 against −1, `1/32`..`1/38` 68 against 66, `1/48`,
        // `1/49`, `1/52` 69 against 68, and `1/53`'s on 13594 (`docs/GROUPS.md` §29).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (153, 162, 162).
        // Item 1330 took 40 (153/155/155 → 113/114/114): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under_n, firsts.len()), (36, 36, 36), "the floor");
    }

    pub(in super::super) fn run78_s_old_word_keeps_its_value_diff() {
        let _pins = Pins::hold();
        const FIRST: i64 = 15_700;
        let Some(Widened {
            firsts,
            blocks,
            standing,
            ..
        }) = widen_east_indies(
            "run78",
            "gamelog-run78-eastindies-threeborn.txt",
            (FIRST, 15_900),
            &[15_783],
            true,
        )
        else {
            return;
        };
        pin_eq!(blocks, 200, "the walk is whole");
        let after: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > FIRST)
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect();
        pin_eq!(
            after,
            [
                // Item 723: `1/0`'s formation mirror (parked 275) is gone.
                // Item 803: its `dest_angle` (ours −1740439552 against
                // −1743650816) is gone, the disembark's `update_action`
                // being step 1's (`docs/TRANSPORT.md` §15); its order
                // point moved from (7320, 21912) and parted, (8364, 14588)
                // against (8088, 17976), until item 1164: an AI's
                // passenger is ordered where it lands (§6.4), and agrees.
                // Item 829: the pool id reads 71 here where it read 69 —
                // a closed army's group now holds its slot (`docs/ARMY.md`
                // §22), so the wagon's new one takes a later index. Item
                // 870: `1/60`'s `group` on 15784, 71 here against 70,
                // agrees — who=1's pool numbering is the original's
                // (`docs/GROUPS.md` §29).
            ] as [&str; 0],
            "every key first parting past the window's first block"
        );
        pin_eq!(
            standing.get(&15_783).map_or(0, |m| m.len()),
            // Item 718: `1/60`'s order point, two rows fewer; item 723 one
            // more, the scout's formation mirror (parked 275); item 752 two
            // fewer, who=1's cities' `gatherers` (`docs/AI.md` §73). Item
            // 870 sixteen fewer: the `group` rows, `1/11` 38 against −1 and
            // army 0's `1/48`..`1/58` 69 against 68 among them (`docs/GROUPS.md` §29).
            // Item 919 208 fewer: run78's `GUY` records are the short form,
            // so gaia's 104 animals' `cur_anim` and `cur_time` read `theirs
            // None` and are named unprinted now, not parting.
            25,
            "every row standing on the old word's block"
        );
        // **The floor**: 437 keys standing on the window's first block,
        // everything that parted in the gap since run166, which no capture
        // prints. 726 in all before item 706, 445 after it.
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 718: 443, `1/60`'s order point agreeing. Item 723: 436/441,
        // the formation mirror (parked 275) on the first block and on
        // `1/0`'s 15800, an equal group's record kept.
        // Item 752: 434/439, who=1's two cities' `gatherers` on the first
        // block, counted in the building's city (`docs/AI.md` §73).
        // Item 803: 434/438, `1/0`'s `dest_angle` on 15800 agreeing.
        // Item 870: 418/421, the first block's sixteen `group` rows (`1/0`
        // 70 here against 69, `1/11` 38 against −1, army 0's `1/48`..`1/58`
        // 69 against 68) and `1/60`'s on 15784 (`docs/GROUPS.md` §29).
        // Item 919: 210/213, the 208 gaia clock rows named unprinted (the
        // short `GUY` record prints none), not a parting closed.
        // Item 1115 took the caravan's four rows here ((210, 213) → (206, 209); `docs/CARAVAN.md` §11.3).
        // Item 1164 took two (106/109 → 106/107): `1/0`'s `orders_x/y` on
        // 15800, above.
        // Item 1330 took 40 (106/107 → 66/66): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, firsts.len()), (25, 25), "the floor");
    }

    pub(in super::super) fn run221_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_SLOT.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_SLOT.1;
        /// The block run221 was taken to widen, the word 15985's.
        const WORD_BLOCK: i64 = EAST_INDIES_SLOT_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            ..
        }) = widen_east_indies(
            "run221",
            "gamelog-run221-eastindies-slotword.txt",
            WIDENING_EAST_INDIES_SLOT,
            &[WORD_BLOCK],
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
            1785658,
            "343 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        // `RON_FIRSTS=1` prints every key's first parting, the window's
        // first block's leader rows included.
        if std::env::var("RON_FIRSTS").is_ok() {
            for r in firsts
                .iter()
                .filter(|((_, o, _), (f, _))| *f > FIRST || *o == -1)
                .map(row)
            {
                eprintln!("  first {r}");
            }
        }
        // **Under the word and on it, both directions — and the move's
        // value diff (item 708; the word's delta is in
        // `LONG_WORD_EAST_INDIES`' comment, its block here).** Until the fix
        // who=1's leader record parted twice under the word: its `rate` on
        // 15977, 93 on three goods here against 118, 110 and 100 — the
        // capped income, `min(cap, income) / 16` — and on 15984,
        // `create_units`' frame, the original offered a citizen (`t 50`,
        // `val 130909`, cat 5) at the head and into slot 5 where ours
        // offered nothing, and its slot loop bought it on the word's frame
        // (`val` /100 and the `make_stuff+0x63d` expiry), the queue and the
        // food showing it on 15986. Fifteen rows stood here. The cause was
        // the commerce cap's republic term (`docs/AI.md` §72): who=1 took
        // Republic on 15782, and `calc_resource_caps` adds
        // `REPUBLIC_COMMERCE_BONUS`, 50, to every capped good. One row
        // stands, the slot's `city` shift the floor's make-list rows carry.
        let under: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            under,
            Vec::<String>::new(),
            "the rows under the word and on it"
        );
        // **The cap, on the window's first block** (the floor's): who=1's
        // `resource_cap` was 2992 here and 3792 there on every capped good
        // until the republic term (`docs/AI.md` §72). run166's 13699 agrees
        // at 2992, and run78 (`LEADERS=1`) does not print it.
        let cap: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                *w == 1 && *o == -1 && *f == FIRST && what.starts_with("leader:resource_cap")
            })
            .map(row)
            .collect();
        pin_eq!(cap, Vec::<String>::new(), "who=0's commerce cap");
        // **The scout's draw is spent on both sides** (the stream: the
        // original spends its own `think_scout+0x941` after the extra), so
        // the scout reading dies there. Its record-level killer — ours'
        // unit for the draw, `1/31`, agreeing field for field on 15985 —
        // fires, on six rows standing since the window's first block, and
        // each is a parked family, not a term the region scan's stride
        // reads: `form` (parked 646), the pool `group` (674), and
        // `myhits`, `hits_left` and `mylos` (679). Its position and
        // orders agree through the word. Item 870 took the pool `group`,
        // 66 here against 65: who=1's numbering is the original's
        // (`docs/GROUPS.md` §29).
        let scout: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| *w == 1 && *o == 31 && *f <= WORD_BLOCK + 2)
            .map(row)
            .collect();
        // Item 1330: its `form` agrees (`docs/GROUPS.md` §24.3).
        pin_eq!(
            scout,
            [] as [&str; 0],
            "0/31's rows through the word's blocks"
        );
        pin_eq!(
            standing
                .get(&(WORD_BLOCK - 1))
                .into_iter()
                .flatten()
                .filter(|((w, o, _), _)| *w == 1 && *o == 31)
                .count(),
            // Item 870: 6 → 5, its `group` agreeing (`docs/GROUPS.md` §29).
            // Item 1111 took the AI citizens' four Militia-line rows (`myhits`, `hits:myhits`, `hits_left`, `mylos`: `docs/GOLDEN.md` §48).
            // Item 1330 took the last, its `form` (`docs/GROUPS.md` §24.3).
            0,
            "1/31's rows standing on 15985, the word's pre-state"
        );
        let one_sided: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(
            one_sided,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure moves on one side only"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 158.
        // Item 1458: 110 → 108; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
        pin_eq!(
            standing.get(&WORD_BLOCK).map_or(0, |m| m.len()),
            // 295 before item 708's fix: the fifteen under the word and
            // the five caps went, the newborn row came. Item 752 took two,
            // who=1's cities' `gatherers` (`docs/AI.md` §73). Item 870
            // took seventeen: the `group` rows, `1/0` 70 here against 69,
            // `1/11` 64 against −1, army 1's `1/48`..`1/60` 71 against 70
            // (`docs/GROUPS.md` §29).
            // Item 1330 took 40 (151 → 110): the births' `form` (`docs/GROUPS.md` §24.3).
            36,
            "every row standing on the word's block"
        );
        // **Past the word to the window's end** (the word left for 16683,
        // past run221): the human's `production_step`, and who=1's newborn
        // `1/59` on 16166 on the parked families — `form` (646), `myhits`,
        // `hits_left` and `mylos` (679) — with its `path_recursion`, the
        // leader's `peasants` 25 against 26 there, and its `gather_stamp`.
        let past: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > WORD_BLOCK)
            .map(row)
            .collect();
        pin_eq!(
            past,
            Vec::<String>::new(),
            "every key first parting past the word"
        );
        // **The floor**: 280 keys standing on the window's first block —
        // everything that parted since run166's last block, which run78's
        // `LEADERS=1` does not print — 280 before the word's frame, 290 in
        // all. It was 285/288/851 before item 708's fix: the five caps, the
        // rates, the citizen's offer and the 551 rows past the word that
        // its purchase parted.
        let under_n = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 752 took two (280/280/290 → 278/278/288): who=1's two
        // cities' `gatherers` on the first block, counted in the building's
        // city (`docs/AI.md` §73).
        // Item 870 took seventeen (278/278/288 → 261/261/271): the first
        // block's `group` rows (`1/0` 70 here against 69, `1/11` 64 against
        // −1, army 1's `1/48`..`1/60` 71 against 70) (`docs/GROUPS.md` §29).
        // Item 1115 took the caravan's four rows here ((261, 261, 271) → (257, 257, 267); `docs/CARAVAN.md` §11.3).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (157, 157, 163).
        // Item 1330 took 40 (151/151/156 → 111/111/115): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under_n, firsts.len()), (36, 36, 36), "the floor");
    }

    pub(in super::super) fn run227_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_WRAPWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_WRAPWORD.1;
        /// The block run227 was taken to widen, the word 16683's.
        const WORD_BLOCK: i64 = EAST_INDIES_WRAPWORD_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            ..
        }) = widen_east_indies(
            "run227",
            "gamelog-run227-eastindies-wrapword.txt",
            WIDENING_EAST_INDIES_WRAPWORD,
            &[WORD_BLOCK],
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
            3670230,
            "705 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for r in firsts
                .iter()
                .filter(|((_, o, _), (f, _))| *f > FIRST || *o == -1)
                .map(row)
            {
                eprintln!("  first {r}");
            }
        }
        // **Under the word and on it, both directions — and the move's
        // value diff (item 752; the word's delta is in
        // `LONG_WORD_EAST_INDIES`' comment, its block here).** The word's
        // figure was who=1's citizen `1/46`: on the original's 16683 its
        // idle (`cur_anim 3`, `end_time 123`, standing since 16529) runs out
        // and rolls, the sixth wrap, where ours' `1/46` was walking. It
        // parted on 16595: on tick 16594, its seventh idle count,
        // `think_peasant`'s `find_gather_spot` sent ours to the woodcutter
        // `1/2009` (its `gather_down` head, 46 against 61) and the
        // original's found nothing. 49 rows stood here; 24 of `1/46`, the
        // chain head, and `1/54`'s two on the word's block, where the extra
        // draw moved its roll, went with the census fix below. What stands
        // are the parked families — 646's `form`, 679's hits and LOS on the
        // newborn `1/61`–`1/63` — who=1's leader rows (the make list's
        // `city` shift, 753's `peasants`), and the scout `1/0`'s speed.
        // **Item 822 took 699's three**: `reg_known_rares[7]` and `[11]` on
        // 16529 and `known_rares` on 16536. The border fix of tick 16527
        // zeroes the array at the next frame's `check_borders`, block
        // 16529, on both sides now (`docs/AI.md` §76).
        let under: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            under,
            ["16681 1/63 form: ours -1 theirs 0"],
            "the rows under the word and on it"
        );
        // **The census, on the window's first block** (the floor's): who=1's
        // two cities counted their gatherers 11 and 11 here against 10 and
        // 12 there, the same 22 split differently, until item 752. That was
        // what sent `1/46`: `find_gather_spot`'s crossing rule walks to
        // another city's building only when its own city's `free +
        // gatherers` is more than two above the other's, and on 16594 ours
        // read London 14 against 10 where the original read 12 against 12.
        // `plan_strategy` counts a gatherer in its building's city
        // (`6babfb`), and this crate counted it in the nearest
        // (`docs/AI.md` §73).
        let census: Vec<String> = firsts
            .iter()
            .filter(|((w, o, what), (f, _))| {
                *w == 1 && *o >= 2000 && *f == FIRST && what.starts_with("city:")
            })
            .map(row)
            .collect();
        pin_eq!(
            census,
            Vec::<String>::new(),
            "who=1's cities on the window's first block"
        );
        // The readings (the stanza's): **R2 held** — its figure was `1/46`,
        // whose record parted first on its order, not on its clock; **R3 is
        // killed** — no figure was on one side only, and the roster agrees;
        // **R4 is killed** — no farm and no farmer parts under the word.
        // Before the fix six figures changed animation on one side on the
        // word's blocks, `1/46` the original's; none does now.
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(
            word,
            Vec::<(i64, i64, i64, bool, bool)>::new(),
            "a figure's animation changes on one side only"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [158, 158].
        // Item 1458: [111, 111] → [109, 109]; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
        pin_eq!(
            (WORD_BLOCK - 1..=WORD_BLOCK)
                .map(|b| standing.get(&b).map_or(0, |m| m.len()))
                .collect::<Vec<_>>(),
            // 319 and 321 before item 752's fix: `1/46`'s 24 rows, the
            // woodcutter's chain head and the two cities' counts, and on
            // the block `1/54`'s two. Item 829 one more on each: `1/0`'s
            // `group`, 70 here against 69, the pool id (689) — a closed
            // army's group now holds its slot (`docs/ARMY.md` §22). Item 850
            // two fewer on each: the scout `1/0`'s `g.avg_speed[0]` and
            // `[1]`, standing from 16681, agree (`docs/ANIM.md` §15). Item
            // 870 seventeen fewer on each: the `group` rows, `1/0` 69 here
            // against 68, `1/11` 64 against −1, army 1's `1/48`..`1/60` 71
            // against 70 among them — who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
            // Item 1106: → 274/274, who=1's `territory` 305 against 302, summed at the border pass's end (`docs/AI.md` §84).
            [37, 37],
            "every row standing on the word's pre-state and its block"
        );
        // **Past the word to the window's end** (the word left for 16982,
        // past run227): the scout `1/0`'s order point and formation mirror
        // from 16782 (743's family). who=1's make-list slot 1 on 16779,
        // 22784 here against 91136 there, went with item 767: the British
        // price of Taxation, which `check_income` read as unaffordable
        // here (`docs/AI.md` §74; the word's delta, 16982 → 17189, is in
        // `LONG_WORD_EAST_INDIES`' comment).
        let past: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f > WORD_BLOCK)
            .map(row)
            .collect();
        // Item 803: the scout's `dest_angle` (ours −2143223808
        // against −2147483648) is gone, the disembark's
        // `update_action` being step 1's (`docs/TRANSPORT.md` §15);
        // its `orders_y` moved from 38040 and still parts.
        // Item 850: the scout's `g.avg_speed`, agreeing from its
        // boarding now (`docs/ANIM.md` §15), first parted on its
        // disembark, where ours seated the figures at 0. Item 1164:
        // it lands at the average it boarded with, and an AI's is
        // ordered where it lands — `avg_speed`, `cur_anim` and
        // `orders_y` agree (`docs/TRANSPORT.md` §6.4).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        pin_eq!(
            past,
            Vec::<String>::new(),
            "every key first parting past the word"
        );
        // **The floor**: 284 keys standing on the window's first block —
        // everything standing on run221's last block, which run227 shares
        // — 306 before the word's frame, 312 in all. It was 286/333/866
        // before item 752's fix: the two cities' counts, `1/46`'s rows and
        // the 554 rows past the word that its walk parted. Item 767 took
        // one past the word (312 → 311): `MAKE[1].val` on 16779. Item 803
        // took one (311 → 310): `1/0`'s `dest_angle` on 16782. Item 822
        // took three (306/310 → 303/307): who=1's `known_rares` and
        // `reg_known_rares[7]`/`[11]`, the border fix's zero. Item 850
        // moved two past the word (303/307 → 301/307): the scout `1/0`'s
        // `g.avg_speed[0]`/`[1]` agree from 16681 and first part on its
        // disembark, 16782 (`docs/ANIM.md` §15).
        let under_n = firsts.values().filter(|(f, _)| *f < WORD_BLOCK - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 870 took seventeen on the first block (284/301/307 →
        // 267/284/290): the `group` rows — `1/0` 69 here against 68,
        // `1/11` 64 against −1, army 1's `1/48`..`1/60` 71 against 70
        // (`docs/GROUPS.md` §29).
        // Item 1106: → 267/282/288, who=1's `territory` on 16528 and who=0's `gather_stamp` on 16529, the border pass (`docs/AI.md` §84).
        // Item 1115 took the caravan's four rows here ((267, 282, 288) → (263, 278, 284); `docs/CARAVAN.md` §11.3).
        // Item 1164 took five past the word (172 → 167): the scout's
        // landing, above.
        // Item 1291 took (159, 166, 167) → (159, 166, 166): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (159, 166, 166).
        // Item 1330 took 41 (153/158/158 → 112/115/115): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!((first, under_n, firsts.len()), (36, 37, 37), "the floor");
    }

    pub(in super::super) fn run233_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_BLOCKWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_BLOCKWORD.1;
        /// The block run233 was taken to widen, the word 17189's.
        const OLD_BLOCK: i64 = EAST_INDIES_BLOCKWORD_BLOCK;
        /// The word 17403's block (item 773).
        const WORD_BLOCK: i64 = EAST_INDIES_GROUPWORD_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run233",
            "gamelog-run233-eastindies-blockword.txt",
            WIDENING_EAST_INDIES_BLOCKWORD,
            &[OLD_BLOCK, WORD_BLOCK],
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
            2665472,
            "512 blocks x 2 leaders x (1,065 + 1,531) keys"
        );
        let row = |((w, o, what), (f, row)): (&(i64, i64, String), &(i64, String))| {
            format!("{f} {w}/{o} {what}: {row}")
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for r in firsts.iter().filter(|(_, (f, _))| *f > FIRST).map(row) {
                eprintln!("  first {r}");
            }
        }
        let under = |(_, (f, _)): &(&(i64, i64, String), &(i64, String))| {
            (FIRST + 1..=WORD_BLOCK).contains(f)
        };
        let on = |b: i64, w: i64, o: i64, k: &str| {
            standing
                .get(&b)
                .and_then(|m| m.get(&(w, o, k.to_string())))
                .cloned()
        };
        // **The move's value diff (item 773).** On 17189 ours' `1/55` stood
        // blocked by `1/60` a frame before the original's, a half step
        // ahead from 17182. That was the column's `1/58`: tick 17146 gave
        // `1/55`, `1/57` and `1/58` an `ATTACK_TO`, and ours' `1/58` priced
        // its world path with who=1's danger, which `no_danger` switches off
        // under an attack-to (`docs/PATHFINDER.md` §28), and cut north out
        // of the column that gives `1/55` its half steps. With the arm, every
        // row of the three goes — 22, 13 and 25 of them — and the stand is
        // on 17190 on both sides.
        for (b, k) in [(OLD_BLOCK, "collide_o"), (OLD_BLOCK + 1, "collide_o")] {
            pin_eq!(on(b, 1, 55, k), None, "`1/55`'s {k} on block {b}");
        }
        // **Under the new word and on it, both directions** (item 773).
        // Each unit's first row, and how many of its keys part by the
        // word's block.
        let mut per_unit: std::collections::BTreeMap<(i64, i64), ((i64, String), usize)> =
            std::collections::BTreeMap::new();
        for r in firsts.iter().filter(under) {
            let at = (r.1.0, row(r));
            let e = per_unit.entry((r.0.0, r.0.1)).or_insert((at.clone(), 0));
            e.0 = e.0.clone().min(at);
            e.1 += 1;
        }
        let got: Vec<(String, usize)> = per_unit.into_values().map(|((_, r), n)| (r, n)).collect();
        let want: Vec<(String, usize)> = [
            // Item 1605 took the human's `production_step` on 17001
            // (`docs/AI.md` §169).
            // who=1's wealth family (`income`, `resources`, `leftover`,
            // `bucket` and `rate` of wealth, 1234 against 1380 from 17184)
            // left with item 839: the British take the territory tax twice
            // (`docs/ECONOMY.md` §16).
            // `1/0`'s `group` (71 against 70 from 17139) left with item
            // 829 for the first block, 70 against 69: a closed army's group
            // holds its slot (`docs/ARMY.md` §22), and the pool id is one
            // higher here from the window's start (689).
            // Item 1330 took the births' `form` (`docs/GROUPS.md` §24.3):
            // `1/63`'s first row is its guy angle now, and `1/64`..`1/67`
            // part on nothing; `1/68`'s and `1/69`'s is `orders_x`.
            // Item 1458: 4 → 3, the recycled slot's `path_recursion` (`docs/GROUPS.md` §37.2).
            ("16971 1/63 g.angle[0]: ours 1431655765 theirs 0", 1),
            // Item 870 took one each from `1/64`..`1/66` (2 → 1): their
            // `group` on 17113, 69 here against 68, agrees — who=1's pool
            // numbering is the original's (`docs/GROUPS.md` §29).

            // Item 882 took one each from `1/67`..`1/69` (2/4/4 → 1/3/3):
            // their `group` on 17363, 69 on both sides (ours −1 before) —
            // the trained squad's birth push, `come_out`'s (parked 689,
            // `docs/GROUPS.md` §31). `form` is `Unit::init`'s (646).
            // Item 1457 took `1/68`'s and `1/69`'s `orders_x/y`: the
            // member's own `come_out` gives it its orders (`docs/CITIES.md`
            // §6.5.3).
        ]
        .iter()
        .map(|(r, n)| ((*r).to_string(), *n))
        .collect();
        // `1/55`, `1/57` and `1/58` are gone (item 773), and so are `1/60`'s
        // 52 rows and `1/15`'s two (item 800). Item 773 left the word on
        // 17403, where the original's `1/60` walked under an `ATTACK_TO` (2)
        // and ours' stood under a `GROUP_ATTACK_TO` (21). What stands of
        // `1/67`..`1/69` is their birth block, 17363 — `group` −1 here and
        // 69 there, `form`, and two `orders_x/y` — and nothing after it but
        // the pool id `group`, 69 here and 71 there (689), which `firsts`
        // does not count past the first. Since item 870 only the birth
        // block's `group` stands: the original's one-block birth push, which
        // this crate does not make (`docs/GROUPS.md` §29).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            got,
            want,
            "each unit's first row under the word, and its count"
        );
        // **Item 800's value diff, on block 17403** (the state after tick
        // 17402, when army 1's siege arm issues). The sub-group sorts its
        // own list (`docs/GROUPS.md` §27): `1/60` takes the dump's own
        // `ATTACK_TO` to (38136, 42024) — before, a `GROUP_ATTACK_TO` of
        // four to (37992, 41784) — and army 1's list is `[60, 69]` on both
        // sides, where the seat's sort had re-seated the squad to `[60, 67,
        // 68, 69]`.
        for k in ["order:kind", "orders_x", "orders_y", "pos", "path:length"] {
            pin_eq!(on(WORD_BLOCK - 1, 1, 60, k), None, "`1/60`'s {k} on 17403");
        }
        for b in WORD_BLOCK - 2..=WORD_BLOCK + 2 {
            pin_eq!(
                army_lists.get(&b).and_then(|m| m.get(&(1, 1))).cloned(),
                Some((vec![60, 69], vec![60, 69])),
                "army 1's list on block {b}, ours then theirs"
            );
        }
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near either word");
        // **Item 811's value diff, on block 17405** (the state after tick
        // 17404, army 0's retarget tick). The original forms the army twice
        // there — `do_marching`'s retarget arm and then the dispatch
        // (`docs/ARMY.md` §21) — so the second move's `get_loc` takes its
        // arm 2 off the first's `(ox, oy)` (34360, 39606) and `1/48`'s order
        // (34296, 39768), and plans from (29833, 37525): cell (38, 48), and
        // the leader's chain gains the leg (29688, 38232) the original
        // walks first. Before, ours planned once, from `1/48` itself: a
        // chain of 8 against 9, `dest_y` 39000 against 38232, and every
        // follower's slot waypoint and heading off it.
        let col = [48, 49, 53, 55, 57, 58];
        for o in col {
            for k in [
                "path:length",
                "order:move.dest_x",
                "order:move.dest_y",
                "heading",
                "pos",
            ] {
                pin_eq!(on(WORD_BLOCK + 1, 1, o, k), None, "`1/{o}`'s {k} on 17405");
            }
            // What stands is the pool id, 689's: the order's `group.id` is
            // `(id + frame × 10) × 100 + order_num`, and its `order_num`
            // now agrees (1, the second move's) where the id is the army's
            // slot 0 here and the pool's 68 there.
            pin_eq!(
                on(WORD_BLOCK + 1, 1, o, "order:group.id").as_deref(),
                None,
                "`1/{o}`'s order id on 17405"
            );
        }
        // who=1's leader, whole: the make list's `city` shift (the floor's
        // family) and `scholars` from 16971. The wealth family went with
        // item 839 (the British territory tax, `docs/ECONOMY.md` §16).
        let leader: Vec<String> = firsts
            .iter()
            .filter(under)
            .filter(|((w, o, _), _)| *w == 1 && *o == -1)
            .map(row)
            .collect();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            leader,
            Vec::<String>::new(),
            "who=1's leader under the word"
        );
        // Figures whose animation changes on one side on either word's
        // blocks: none. Before item 800, on 17403, `1/60` set off there a
        // block before ours, and the column after it.
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| {
                c.3 != c.4
                    && [OLD_BLOCK, WORD_BLOCK]
                        .iter()
                        .any(|b| (b - 2..=b + 2).contains(&c.0))
            })
            .copied()
            .collect();
        pin_eq!(word, [], "a figure's animation changes on one side only");
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [168, 168, 167, 167].
        // Item 1458: [114, 114, 114, 114] → [111, 111, 111, 111]; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
        pin_eq!(
            [OLD_BLOCK - 1, OLD_BLOCK, WORD_BLOCK - 1, WORD_BLOCK]
                .iter()
                .map(|b| standing.get(b).map_or(0, |m| m.len()))
                .collect::<Vec<_>>(),
            // Item 800: 418/439 → 311/311 on 17403's pre-state and block —
            // run227's 307 and the pool id `group` of `1/60` and the squad.
            // Item 839: 307/307/311/311 → 304/304/306/306, who=1's wealth
            // family — three rows standing on 17189 (`income`,
            // `resources`, `leftover`), five on 17403 — agrees
            // (`docs/ECONOMY.md` §16). Item 870: → 284/284/283/283, the
            // `group` rows of `1/0`, `1/11`, `1/31`..`1/38`, army 0's
            // column and the squad (`docs/GROUPS.md` §29).
            [37, 37, 37, 37],
            "every row standing on each word's pre-state and its block"
        );
        // **The floor**: 293 keys standing on the window's first block —
        // everything standing on run227's last block, which run233 shares
        // — then the rows before the old word's frame, before the new
        // word's, and in all. Item 773 took it 293/363/-/1,139 →
        // 293/312/324/1,036: before the old word the column's rows go
        // and `1/63` loses two, and past it the stand's wake goes;
        // `1/67`..`1/69` from 17363 and `1/60` on 17403 are the new rows.
        // Item 811 took 116 past the new word (446 → 330): army 0's column
        // from 17405, once it is formed twice on its retarget tick. Item
        // 829 moved `1/0`'s `group` from 17139 to the first block (293 →
        // 294), the pool id one higher (`docs/ARMY.md` §22). Item 839
        // took 294/312/324/330 → 294/309/319/325: who=1's wealth family
        // from 17184 goes (three rows before the old word, five in all),
        // the British territory tax (`docs/ECONOMY.md` §16).
        let before = |b: i64| firsts.values().filter(|(f, _)| *f < b - 2).count();
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 870 took 294/309/319/325 → 277/289/299/305: the first
        // block's seventeen `group` rows (`1/0` 70 here against 69, `1/11`
        // 64 against −1, army 0's `1/48`..`1/60` 65 against 64) and the
        // squad `1/64`..`1/66`'s on 17113, 69 against 68 (`docs/GROUPS.md` §29).
        // Item 882 took 277/289/299/305 → 277/289/296/302: `1/67`..`1/69`'s
        // `group` on 17363, 69 on both sides (ours −1 before), the squad's
        // birth push, `come_out`'s (parked 689, `docs/GROUPS.md` §31).
        // Item 1115 took the caravan's four rows here ((277, 289, 296, 302) → (273, 285, 292, 298); `docs/CARAVAN.md` §11.3).
        // Item 1291 took (161, 173, 180, 186) → (160, 172, 179, 185): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (160, 172, 179, 185).
        // Item 1330 took 43 (155/165/172/178 → 112/118/122/128): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, before(OLD_BLOCK), before(WORD_BLOCK), firsts.len()),
            (36, 37, 37, 37),
            "the floor"
        );
    }

    pub(in super::super) fn run251_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_COLUMNWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_COLUMNWORD.1;
        const WORD_BLOCK: i64 = EAST_INDIES_COLUMNWORD_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run251",
            "gamelog-run251-eastindies-columnword.txt",
            WIDENING_EAST_INDIES_COLUMNWORD,
            &[WORD_BLOCK],
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
        // **Under the word, the first row of each key past the floor.**
        // Until item 811 eleven stood: `1/57` stood on 17501 with one path
        // entry where the original's held two and walked on, and on the
        // word's block the original's set off (`cur_anim 8`) toward
        // (31451, ·) while ours took a leg to (30683, ·); `1/58` beside it,
        // and `1/54`'s `dest`. **Item 811 took all eleven**: army 0 is
        // formed twice on its retarget tick 17404 (`docs/ARMY.md` §21), so
        // its leader `1/48` plans from `get_loc`'s second arm and the
        // column walks the original's route in.
        let rows: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "each key's first row under the word"
        );
        // **Item 811's value diff on the old word's block, 17502**: the keys
        // that parted there, each now absent on both sides.
        for (o, k) in [
            (54, "order:move.dest"),
            (54, "path:length"),
            (57, "g.cur_anim[0]"),
            (57, "g.stopped[0]"),
            (57, "order:move.dest_x"),
            (57, "path:length"),
            (58, "g.cur_anim[0]"),
        ] {
            pin_eq!(
                standing
                    .get(&WORD_BLOCK)
                    .and_then(|m| m.get(&(1, o, k.to_string()))),
                None,
                "`1/{o}`'s {k} on block {WORD_BLOCK}"
            );
        }
        // The figures whose animation changes on one side near the word:
        // none. Until item 811, ours' `1/57` and `1/58` on its block and
        // the two after it.
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(word, [], "a figure's animation changes on one side only");
        // Every army's list, on what its slot holds, agrees near the word.
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near the word");
        // **The floor**: every key standing on the first block (nothing
        // shared, so the column's walk since 17405 and the pool ids are all
        // in it), the keys first parting under the word, the rows standing
        // on the word's block, and every key in all. **Item 811** took it
        // 350/11/361/1,314 → 313/0/313/337: the column's walk from 17405
        // agrees, on the first block and past the word. **Item 839** took
        // it 313/0/313/337 → 308/0/308/332: who=1's wealth family —
        // `bucket`, `income`, `leftover`, `rate` and `resources` of wealth
        // — agrees once the British take the territory tax twice
        // (`docs/ECONOMY.md` §16). **Item 850** took it 308/0/308/332 →
        // 308/0/308/330: the scout `1/0`'s `g.avg_speed[0]` and `[1]`, 24
        // against 18 on 17750, agree (`docs/ANIM.md` §15).
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (169, 0, 169, 188).
        // Item 1330 took 44 (165/0/165/184 → 121/0/121/137): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            // Item 870: 308/0/308/330 → 285/0/285/307, the first block's
            // 23 `group` rows (`1/11` 64 against −1, army 1's 69 against
            // 68, the old army's 72 against 71): who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
            // Item 882: 285/0/285/307 → 285/0/285/304, `1/70`..`1/72`'s
            // `group` on 17575, 64 on both sides (ours −1 before): the
            // trained squad's birth push, `come_out`'s (parked 689,
            // `docs/GROUPS.md` §31).
            (37, 0, 37, 38),
            "the floor"
        );
    }

    pub(in super::super) fn run253_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_MARKETWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_MARKETWORD.1;
        const WORD_BLOCK: i64 = EAST_INDIES_MARKETWORD_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run253",
            "gamelog-run253-eastindies-marketword.txt",
            WIDENING_EAST_INDIES_MARKETWORD,
            &[WORD_BLOCK],
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
        // **Under the word, the first row of each key past the floor**
        // (item 811): who=1's make list and nothing else to the word's
        // frame. From 18181 (tick 18180) ours held a type-61 order — the
        // Merchant — cat 4, in slots 0 and 4, where the original held 590,
        // cat 8, in slot 0 and nothing in slot 4. **Item 822 took them**:
        // the original's `known_rares` was 0 from its recompute on 18071,
        // summing an array a border fix had zeroed (the Civic epoch of tick
        // 18032, `docs/AI.md` §76), so its Merchant arm was shut. What
        // stands is the make list's `city` shift, the floor's.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "each key's first row under the word"
        );
        // **Item 822's value diff** (the word's delta is in
        // `LONG_WORD_EAST_INDIES`' comment, the old word's block here):
        // who=1's `known_rares` read 3 here against 0 there from the first
        // block, 18177, and now parts on no block of the walk; `MAKE[0]`
        // and `MAKE[4]` read (61, cat 4, val 952,380) here against (590,
        // cat 8, val 22,784) and the fresh slot there on 18181, and now
        // agree in every field but `city`.
        pin!(
            !firsts.contains_key(&(1, -1, "leader:known_rares".to_string())),
            "who=1's known_rares parts nowhere on run253"
        );
        for k in ["t", "val", "cat", "escrow", "num"] {
            for i in [0, 4] {
                pin_eq!(
                    standing.get(&(WORD_BLOCK - 2)).and_then(|m| m.get(&(
                        1,
                        -1,
                        format!("leader:MAKE[{i}].{k}")
                    ))),
                    None,
                    "who=1's MAKE[{i}].{k} on block {}",
                    WORD_BLOCK - 2
                );
            }
        }
        // R3's unit: who=1's stockpile on 18182, every good but wealth.
        let stock: Vec<String> = standing
            .get(&(WORD_BLOCK - 1))
            .into_iter()
            .flat_map(|m| m.keys())
            .filter(|(w, o, k)| {
                *w == 1
                    && *o == -1
                    && (k.starts_with("leader:resources[") || k.starts_with("leader:leftover["))
                    && !k.contains(":wealth]")
            })
            .map(|(_, _, k)| k.clone())
            .collect();
        pin_eq!(stock, Vec::<String>::new(), "who=1's stockpile on 18182");
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        // Until item 822, ours' `0/0` set `cur_anim` 2 on 18185 where the
        // original's stayed at 0, two blocks after the market; with the
        // Merchant gone no figure changes on one side near the word.
        pin_eq!(word, [], "a figure's animation changes on one side only");
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near the word");
        // **The floor**: every key standing on the first block (nothing
        // shared, so everything standing when the walk arrives), the keys
        // first parting under the word, the rows standing on the word's
        // block, and every key in all. **Item 822** took it
        // 310/18/327/977 → 309/4/313/319: `known_rares` leaves the first
        // block, the Merchant's fourteen rows under the word go, and 658
        // keys past it with them. **Item 839** took it 309/4/313/319 →
        // 305/4/309/315: who=1's `bucket`, `income`, `rate` and
        // `resources` of wealth agree (the British territory tax,
        // `docs/ECONOMY.md` §16). `leftover[2:wealth]` stands, 2380
        // against 2372 on the first block: eight sixteenths from the gap
        // 17753..18176, which no dump compares.
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (159, 4, 163, 169).
        // Item 1330 took 44 (159/0/159/161 → 115/0/115/117): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            // Item 870: 305/4/309/315 → 276/4/280/286, the first block's
            // 29 `group` rows (`1/11` 64 against −1, `1/48`..`1/71` 72
            // against 71): who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
            // Item 1106: → 275/4/279/285, who=1's wealth, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84): `leftover[2:wealth]` 2380 against 2372 on 18177.
            (37, 0, 37, 37),
            "the floor"
        );
    }

    pub(in super::super) fn run257_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_GUARDWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_GUARDWORD.1;
        const OLD_BLOCK: i64 = EAST_INDIES_GUARDWORD_BLOCK;
        const FIG_BLOCK: i64 = EAST_INDIES_FIGUREWORD_BLOCK;
        const WORD_BLOCK: i64 = EAST_INDIES_LEADERWORD_BLOCK;
        /// The block after tick 19176, when ours bought the timber (item
        /// 839).
        const PURCHASE: i64 = 19_177;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run257",
            "gamelog-run257-eastindies-guardword.txt",
            WIDENING_EAST_INDIES_GUARDWORD,
            &[FIRST, OLD_BLOCK, FIG_BLOCK, PURCHASE, WORD_BLOCK],
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
        // **The move's value diff, on the first block** (item 829): the
        // escort `1/67`..`1/72` read `group` −1 here against 71 there until
        // `close_army` kept a closed army's group in the pool
        // (`docs/ARMY.md` §22). They read 72 against 71 now: the same
        // group, the old army 1's, holding `[69, 72]` on both sides — the
        // index is one higher here, as every pool id of who=1 is from
        // `1/11`'s up (689). **Item 870**: the index is the original's, 71
        // on both sides — who=1's pool numbering agrees
        // (`docs/GROUPS.md` §29) — so no `group` row stands.
        let first_block = standing.get(&FIRST).cloned().unwrap_or_default();
        for o in 67..=72 {
            pin_eq!(
                first_block
                    .get(&(1, o, "group".to_string()))
                    .map(String::as_str),
                None,
                "`1/{o}` points at the old army 1's group, the same index on both sides"
            );
        }
        // **And on the old word's block** (items 822 and 829): until the
        // fix `1/67`, `1/68`, `1/70` and `1/71` stood under a `GUARD` (kind
        // 12, one order, `tolerance` 144, stopped) where the original's
        // walk under its `ATTACK_TO` leg (kind 2, two orders, a one-leg
        // path), because `1/69`, a group-mate, was a hard collision here
        // and a soft one there. Nothing of the escort parts now.
        let escort: Vec<String> = firsts
            .iter()
            .filter(|((w, o, _), (f, _))| {
                *w == 1 && (67..=72).contains(o) && (FIRST + 1..=OLD_BLOCK).contains(f)
            })
            .map(row)
            .collect();
        pin_eq!(
            escort,
            Vec::<String>::new(),
            "the escort agrees on {OLD_BLOCK}"
        );
        // **Under item 829's word, 18999, the first row of each key past
        // the floor**: who=1's `reg_unpack_merch[11]` on 18976, which
        // nothing on a figure's path reads, and nothing on the word's
        // block, 19000.
        let under_fig: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIRST + 1..=FIG_BLOCK).contains(f))
            .map(row)
            .collect();
        pin_eq!(
            under_fig,
            ["18976 1/-1 leader:reg_unpack_merch[11]: ours 2 theirs 1"],
            "the keys first parting under item 829's word"
        );
        // **Item 837's value diff, on block 19000.** The draw stream
        // parted there on a figure's wrap, five against four: `1/19`,
        // `1/20`, `1/29`, `1/67` and `1/69` all wrap on both sides, and
        // `1/67` alone holds a suspended search, whose idle roll spends no
        // draw (`docs/ANIM.md` §14). Its record agrees on the block but
        // for the pool id every member of the old group carries (689):
        // `start_dist` 480, `collide` 31, `collide_frame` 18969, and guy 0
        // at `cur_anim` 0, `cur_time` 0, `end_time` 31, `last_time` −1.
        let fig_block = standing.get(&FIG_BLOCK).cloned().unwrap_or_default();
        let suspended: Vec<(&str, &str)> = fig_block
            .iter()
            .filter(|((w, o, _), _)| *w == 1 && *o == 67)
            .map(|((_, _, k), v)| (k.as_str(), v.as_str()))
            .collect();
        // Item 870: the pool id agrees too (`docs/GROUPS.md` §29).
        pin_eq!(
            suspended,
            Vec::<(&str, &str)>::new(),
            "`1/67` agrees on {FIG_BLOCK}, its pool id included"
        );
        let rows: Vec<String> = firsts
            .iter()
            .filter(|(_, (f, _))| (FIG_BLOCK + 1..WORD_BLOCK).contains(f))
            .map(row)
            .collect();
        // **Under item 837's word, 19182, the first row of each key past
        // item 829's word.** Until item 839 they were who=0's
        // `production_step` on 19001, who=1's timber bucket on 19177 (149
        // against 49) and its make list on 19182 (`MAKE[0].val` 1531
        // against 0, `MAKE[1].t` 427 against 528, …): ours'
        // `market_speculation` bought a hundred timber for 135 wealth on
        // tick 19176 out of a purse of 150, where the original's held 67.
        // The purse was the territory tax: the British take it twice
        // (`docs/ECONOMY.md` §16). who=0's `production_step` stands; its
        // readers are who=0's own `plan_strategy` and `production_ai`.
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "the keys first parting under item 837's word"
        );
        // **Item 839's value diff.** who=1's purse and wood on 19176 and
        // 19177 (the tick of the purchase), and the whole wealth family and
        // the food on the first block, which stood 116 against 27 and 114
        // against 214 from the gap 18685..18932 (the original's hundred
        // food bought with the tax ours did not collect). Only
        // `leftover[2:wealth]` stands, eight sixteenths over from the gap
        // 17753..18176.
        let economy = |b: i64| -> Vec<String> {
            standing
                .get(&b)
                .into_iter()
                .flatten()
                .filter(|((w, o, k), _)| {
                    *w == 1
                        && *o == -1
                        && ["bucket", "income", "resources", "rate", "leftover"]
                            .iter()
                            .any(|p| k.starts_with(&format!("leader:{p}[")))
                })
                .map(|((_, _, k), v)| format!("{k}: {v}"))
                .collect()
        };
        // Item 1106 closed the last of it, `leftover[2:wealth]` 3588
        // against 3580: the border pass's economy flag (`docs/AI.md` §84).
        pin_eq!(
            economy(FIRST),
            Vec::<String>::new(),
            "who=1's economy on the first block"
        );
        // Item 1106 closed it (6924 against 6916): the border pass's
        // economy flag (`docs/AI.md` §84).
        pin_eq!(
            economy(PURCHASE),
            Vec::<String>::new(),
            "who=1's purse and wood on the purchase's block"
        );
        let on_word_firsts = firsts.values().filter(|(f, _)| *f == WORD_BLOCK).count();
        // Item 839 took it 24 → 0: nothing parts first on 19183 now.
        pin_eq!(
            on_word_firsts,
            0,
            "the keys first parting on the word's block"
        );
        // The figures that change animation on one side only, near either
        // word.
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| {
                c.3 != c.4
                    && [OLD_BLOCK, FIG_BLOCK, WORD_BLOCK]
                        .iter()
                        .any(|b| (b - 2..=b + 2).contains(&c.0))
            })
            .copied()
            .collect();
        // Until item 837, `1/9` on 19002 was the original's only: its
        // `cur_anim` 36 → 8 is both sides' now. Until item 839 ours alone
        // changed `0/0` on 19183 and `1/7` on 19184, past the purchase;
        // none does now.
        pin_eq!(word, [], "a figure's animation changes on one side only");
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near the word");
        // who=1's `known_rares` agrees on every block (item 822's zero).
        pin!(
            !firsts.contains_key(&(1, -1, "leader:known_rares".to_string())),
            "who=1's known_rares parts nowhere on run257"
        );
        // **The floor**: every key standing on the first block (nothing
        // shared, so everything the gap 18434..18932 left standing), the
        // keys first parting under the word, the rows standing on the
        // word's block, and every key in all.
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // **Item 829** took it 320/103/423/1,265 → 322/1/323/863: the
        // escort's four units leave the old word's block, and 402 keys
        // past it with them; the first block gains `1/60`'s `group` (70
        // against 69) and `1/0`'s (71 against 70), both the pool id one
        // higher here (689), and `1/67`..`1/72`'s read 72 against 71 where
        // they read −1. **Item 837** took it 322/1/323/863 →
        // 322/9/355/387: the word's block is 19183 now, so the middle two
        // count under it (from 19001) and on it, and 476 keys that parted
        // behind the extra draw on 18999 agree. **Item 839** took it
        // 322/9/355/387 → 317/1/318/319: the first block loses who=1's
        // food and four of the wealth family (`leftover` stands), the
        // eight rows of the purchase under the word go, and 68 keys past
        // it with them.
        // **Item 870** took it 317/1/318/319 → 288/1/289/290: the first
        // block's 29 `group` rows (`1/0` 71 here against 70, `1/11` 64
        // against −1, army 0's 65 against 64, the old army's 72 against
        // 71) (`docs/GROUPS.md` §29).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (167, 1, 168, 169).
        // Item 1330 took 46 (161/1/162/163 → 115/1/116/117): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            // Item 1106: → 287/1/288/289, who=1's `leftover[2:wealth]` 3588 against 3580 on the first block, the border pass's economy flag (`docs/AI.md` §84).
            (37, 0, 38, 38),
            "the floor"
        );
    }

    pub(in super::super) fn run269_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_TURNWORD.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_TURNWORD.1;
        const TURN: i64 = EAST_INDIES_TURNWORD_BLOCK;
        const WORD_BLOCK: i64 = EAST_INDIES_WALKWORD_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run269",
            "gamelog-run269-eastindies-turnword.txt",
            WIDENING_EAST_INDIES_TURNWORD,
            &[FIRST, TURN, WORD_BLOCK],
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
        let under = |lo: i64, hi: i64| -> Vec<String> {
            firsts
                .iter()
                .filter(|(_, (f, _))| (lo..=hi).contains(f))
                .map(row)
                .collect()
        };
        // **Under the old word and on it — the move's value diff** (item
        // 850). Nothing parts on 19409..19413. On 19414 `1/77`, the
        // Merchant that cast its barge `1/78` in its own work on tick
        // 19413, read `g.angle[1]` −541917184 against −901447680,
        // `g.avg_speed[0]` 15 against 11, `g.avg_speed[1]` 16 against 12
        // and `g.last_speed[1]` 1 against 0 until its figures took the
        // frame `Unit::process` gives them after the work (`docs/ANIM.md`
        // §15); all four agree now. `1/78`'s `form` reads −1 here against
        // 0 there, the standing shape of every group member's.
        pin_eq!(
            under(FIRST + 1, TURN),
            ["19414 1/78 form: ours -1 theirs 0"],
            "the keys first parting under the old word"
        );
        // **Under item 850's word and on it, the move's value diff** (item
        // 857). On 19499 the original's `1/71`, suspended behind `1/75`,
        // read `collide` 11 against 0, `collide_o` 75 against −1 and
        // `collide_who` 1 against −1; from 19500 ours walked on (`pos`
        // (36866, 41713) against (36888, 41736)), and `1/70` and `1/23`
        // followed. `do_move`'s blocker probe is the quick form, so
        // `1/75`'s corner at `coll` counts and nothing is written
        // (`docs/COLLISION.md` §18). All 38 agree now: `1/71` reads
        // `collide` 11, 12, 0 on 19499..19501 with `collide_o` 75, and
        // walks from 19502 on both sides.
        let rows = under(TURN + 1, WORD_BLOCK);
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "the keys first parting under item 850's word"
        );
        // **And past it, to the capture's end** (item 857): the human
        // leader's `production_step` on 19601, ours 0 against 1, is the one
        // key. On 19606 both sides spend `1/64`'s `retry` roll
        // (`Unit::do_guard+0x8fb`, which the trace printed bare as
        // `5e656b`), and its guard's `retry` reads 8 on block 19607 on both.
        pin_eq!(
            under(WORD_BLOCK + 1, TAIL),
            Vec::<String>::new(),
            "the keys first parting past it"
        );
        // No figure's animation changes on one side only near item 850's
        // word: item 857 took `1/71`'s three (19509 there, 19510 and 19511
        // here) with the collision. No army's list parts.
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(word, [], "a figure's animation changes on one side only");
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near the word");
        // **The first block's standing rows**: who=1's twenty-nine
        // citizens hold `hits_left` 40 here against 50 there, with
        // `myhits` beside it, and `mylos` 2 against 4. They are not the
        // gap's (19190..19407, which no dump compares): the same 29 stand
        // on run257's first and last blocks, 18933 and 19189, and on this
        // map since 10165 (parked 679). No reader of either field is on
        // the draw of 19413 (item 850). who=1's `leftover[2:wealth]` is
        // eight sixteenths over, as on run253 and run257.
        let first_block = standing.get(&FIRST).cloned().unwrap_or_default();
        let hits: usize = first_block
            .iter()
            .filter(|((w, _, k), v)| *w == 1 && k == "hits_left" && *v == "ours 40 theirs 50")
            .count();
        pin_eq!(hits, 0, "who=1's citizens' hits_left on the first block");
        // **The floor**: every key standing on the first block, the keys
        // first parting under the word, the rows standing on the word's
        // block, and every key in all. **Item 850** took it 318/5/323/1,433
        // → 318/38/352/1,207: the word's block is 19510 now, so the keys
        // under it are counted from 19415 (`1/70`, `1/71` and `1/23`'s 38)
        // and on it; `1/77`'s four go, and 226 fewer keys part in all
        // behind the turn on 19413 the original spends and ours now does.
        // **Item 857** took it 318/38/352/1,207 → 318/0/319/320: the 38
        // under item 850's word go with `1/71`'s collision, the block
        // keeps the first block's 318 and `1/78`'s `form`, and 887 keys
        // past it agree. The word itself is past this capture (run277).
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (168, 0, 169, 170).
        // Item 1330 took 46 (162/0/163/164 → 116/0/117/118): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            // Item 870: 318/0/319/320 → 289/0/290/291, the first block's
            // 29 `group` rows (`1/0` 65 here against 72, `1/11` 64 against
            // −1, army 1's 70 against 69): who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
            // Item 1106: → 288/0/289/290, who=1's wealth, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84): `leftover[2:wealth]` 1938 against 1930 on 19408.
            (38, 0, 39, 39),
            "the floor"
        );
    }

    pub(in super::super) fn run277_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        const FIRST: i64 = WIDENING_EAST_INDIES_BLOCKEDWALK.0;
        const TAIL: i64 = WIDENING_EAST_INDIES_BLOCKEDWALK.1;
        const WORD_BLOCK: i64 = EAST_INDIES_BLOCKEDWALK_BLOCK;
        let Some(Widened {
            firsts,
            missing,
            blocks,
            leader_rows,
            changed,
            standing,
            army_lists,
            ..
        }) = widen_east_indies(
            "run277",
            "gamelog-run277-eastindies-blockedwalk.txt",
            WIDENING_EAST_INDIES_BLOCKEDWALK,
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
        let under = |lo: i64, hi: i64| -> Vec<String> {
            firsts
                .iter()
                .filter(|(_, (f, _))| (lo..=hi).contains(f))
                .map(row)
                .collect()
        };
        // **The first block, from the gap** (19665..20001, which no dump
        // compares). Until item 880 who=1's army group, 68 there and 71
        // here, held a group move whose slots `1/65` and `1/66` held the
        // other way round — `off` (504, 648) there against (696, 456) here,
        // and the reverse — with `facing` 0 there against 1 here on
        // `1/64`..`1/66` (item 857). **Item 880 closed all 37 of those
        // rows**: the push on tick 20000 keeps slot 69's record, `facing`
        // 1 and `order_num` 6 on both sides, so the layout is square on
        // both (`docs/GROUPS.md` §30). The rest of the block is the
        // standing set run269 ended on: `form`, who=1's citizens' hits,
        // and the leader rows.
        let first_block = standing.get(&FIRST).cloned().unwrap_or_default();
        let slot = |o: i64, k: &str| first_block.get(&(1, o, k.to_string())).cloned();
        pin_eq!(
            (
                slot(64, "order:move.facing"),
                slot(65, "order:move.off_x"),
                slot(66, "order:move.off_x"),
                slot(65, "pos")
            ),
            (None, None, None, None),
            "the two slots agree: square on both sides"
        );
        // **Under the word and on it** (item 857's). Every key that first
        // parted on 20005..20008 — `1/65`'s walk animation, `1/67`'s
        // refusal by `1/65` and its stand — closed with the slots (880):
        // the word left run277 for 20782.
        let rows = under(FIRST + 1, WORD_BLOCK);
        pin_eq!(
            rows,
            Vec::<String>::new(),
            "the keys first parting under the word"
        );
        let word: Vec<(i64, i64, i64, bool, bool)> = changed
            .iter()
            .filter(|c| c.3 != c.4 && (WORD_BLOCK - 2..=WORD_BLOCK + 2).contains(&c.0))
            .copied()
            .collect();
        pin_eq!(word, [], "no figure's animation changes on one side only");
        let lists_part: Vec<(i64, (i64, i64))> = army_lists
            .iter()
            .flat_map(|(b, m)| {
                m.iter()
                    .filter(|(_, (o, t))| o != t)
                    .map(move |(k, _)| (*b, *k))
            })
            .collect();
        pin_eq!(lists_part, [], "no army's list parts near the word");
        // **The floor**: every key standing on the first block, the keys
        // first parting under the word, the rows standing on the word's
        // block, and every key in all.
        let on_word = standing.get(&WORD_BLOCK).map_or(0, |m| m.len());
        let first = firsts.values().filter(|(f, _)| *f == FIRST).count();
        // Item 870: 352/21/371/948 → 323/21/342/916, the first block's
        // 29 `group` rows (`1/0` 65 here against 72, `1/11` 64 against
        // −1, army 1's `1/48`..`1/66` 71 against 68) and three past it:
        // who=1's pool numbering, the original's since item 870 (`docs/GROUPS.md` §29).
        // Item 880: 323/21/342/916 → 286/0/286/299, the first
        // block's 37 slot-swap rows (`1/64`..`1/66`'s `facing`, `1/65`'s
        // and `1/66`'s `off`, `x/y`, paths and `1/65`'s position) and
        // 580 keys past it; none opened (`docs/GROUPS.md` §30).
        // Item 1106: → 285/0/285/297, who=1's wealth, a region's end raising the economy flag at the border pass's pace (`docs/AI.md` §84): `leftover[2:wealth]` 5574 against 5566 on 20002 and `bucket[2:wealth]` 95 against 94 on 20130.
        // Item 1164: → 165/0/165/171, six past the first block, the
        // landings' figure speeds and orders (`docs/TRANSPORT.md` §6.4).
        // Item 1291 took (165, 0, 165, 171) → (165, 0, 165, 170): a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was (165, 0, 165, 170).
        // Item 1330 took 46 (162/0/162/167 → 116/0/116/119): the births' `form`, `Unit::init`'s (`docs/GROUPS.md` §24.3).
        pin_eq!(
            (first, rows.len(), on_word, firsts.len()),
            (38, 0, 38, 39),
            "the floor"
        );
    }
}
