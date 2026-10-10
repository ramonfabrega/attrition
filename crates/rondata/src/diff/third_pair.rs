//! The third pair: human Nubians, French AI, seed 12345 and Toughest.
//! Each map owns a fresh full-start sibling; the second pair stays pinned.

//! Item 1647: affected windows lose only `leader:caras`, now matching
//! the original after Merchant Fleet live updates (AI §177). Expected
//! counts/histograms are tightened; every comparison and window remains.

use super::testkit::*;
use super::*;
use crate::testenv::dump;

const EAST_START: &str = "gamelog-run595-islands-french-toughest-start.txt";
const EAST_LONG: (&str, &str) = (
    "gamelog-run600-islands-french-toughest-24k-trace.txt",
    "rontrace-run600.log",
);
const LAKES_START: &str = "gamelog-run597-lakes-french-toughest-start.txt";
const LAKES_LONG: (&str, &str) = (
    "gamelog-run598-lakes-french-toughest-24k-trace.txt",
    "rontrace-run598.log",
);

#[test]
fn third_pair_starts_change_the_ai_nation_in_the_same_lobby() {
    for (start, old) in [
        (EAST_START, "gamelog-run346-islands-toughest-24k-trace.txt"),
        (
            LAKES_START,
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
        ),
    ] {
        let (Some(a), Some(b)) = (dump(start), dump(old)) else {
            continue;
        };
        let (a, b) = (crate::capture::read(&a), crate::capture::read(&b));
        let (a, b) = (Log::parse(&a), Log::parse(&b));
        let (a, b) = (a.initial().unwrap(), b.initial().unwrap());
        assert_eq!(a.game_info, b.game_info, "only the AI nation input moves");
        assert!(
            a.checksums.last().is_some(),
            "the full dump carries its setup seed"
        );
        let tribes: Vec<_> = a
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe, l.leader_flags & 4 != 0))
            .collect();
        assert_eq!(tribes, [(0, 4, true), (1, 10, false)]);
        assert!(!a.heights.is_empty() && !a.herds.is_empty() && !a.goods.is_empty());
        assert_eq!(a.players.len(), b.players.len());
        for (who, (a, b)) in a.players.iter().zip(&b.players).enumerate() {
            // The final bare line is the nation-derived display name, not a setting.
            let fields = |v: &Vec<(&str, &str)>| -> Vec<(String, String)> {
                v.iter()
                    .filter(|(k, v)| v.parse::<i64>().is_ok() && !(who == 1 && *k == "tribe"))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            };
            assert_eq!(fields(a), fields(b), "player {who}'s other settings");
        }
    }
}

#[test]
fn run600_french_east_indies_word() {
    check_long(EAST_LONG.0);
    let Some(w) = third::walk_from(EAST_START, EAST_LONG) else {
        return;
    };
    assert_eq!(w.difficulty, 5);
    assert_eq!(w.last, ai_word_length("EastIndiesFrench"));
    assert!(
        w.count >= THIRD_PAIR_WORD_EAST_INDIES && w.sequence >= THIRD_PAIR_WORD_EAST_INDIES,
        "{}",
        w.row
    );
}

#[test]
fn run598_french_great_lakes_word() {
    check_long(LAKES_LONG.0);
    let Some(w) = third::walk_from(LAKES_START, LAKES_LONG) else {
        return;
    };
    assert_eq!(w.difficulty, 5);
    assert_eq!(w.last, ai_word_length("GreatLakesFrench"));
    assert!(
        w.count >= THIRD_PAIR_WORD_GREAT_LAKES && w.sequence >= THIRD_PAIR_WORD_GREAT_LAKES,
        "{}",
        w.row
    );
}

/// **French East Indies' closing whole-map state** (item 1519): the word
/// reached the trace's end, 17,379, once a guard's attack on a building took
/// no chase (`docs/AI.md` §138), and a draw stream that agrees to a game's
/// end says nothing of the end itself (item 1099). run600's closing dump,
/// block 17380, walked to and compared whole.
#[test]
fn run600_french_east_indies_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(EAST_LONG.0), dump(EAST_START), trace(EAST_LONG.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "French East Indies closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked
    );
    eprintln!(
        "Closing city fields: {:?}",
        harness::compare(&built, &fin, 8).city_diverged
    );
    pin_eq!(e.frame, 17380, "the closing block");
    // Item 1519: 154 units, none off, unlinked or extra, and every building
    // linked and agreeing. `torn` is printed, not pinned (`endpoint`'s
    // module note): one unit, `0/5`.
    pin_eq!(e.compared, 154, "units compared");
    // The 14 are one record's: the human's Village `0/2000`, whose player
    // is defeated on this block (`defeat_stamp 17380`). Ours reads 0 for
    // `land` 84, `filled` 44, `ocean` 25, `space[0..2]` 51/51/40, `ter[0,1,3]`,
    // `busy`/`gatherers` 5, `peasant_dist` 1, `dock_tile` 1 and `raid_stamp`
    // 17326. A residue of the closing, not a parity claim.
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
}

#[test]
fn run598_french_great_lakes_closing_state() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(LAKES_LONG.0), dump(LAKES_START), trace(LAKES_LONG.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "French Great Lakes closing: frame {} units {} counts {:?} torn {:?} off {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off
    );
    assert_eq!(e.frame, 5639);
    eprintln!(
        "Closing city fields: {:?}",
        harness::compare(&built, &fin, 8).city_diverged
    );
    assert_eq!(e.compared, 40);
    assert!(e.torn.is_empty());
    // Item 1477: `city_diverged` 8 → 4 (`1/2007`'s `filled` and `space[0..2]` agree once
    // the census circle starts at entry 1, `docs/AI.md` §122).
    // Item 1563: `build_diverged` 0 → 14, the shared instrument comparing
    // `ever_seen` and `ever_seen_completed` (parked 1450): on the closing
    // block the original's `1/2019`..`1/2025` carry player 0's bit too (3
    // against ours' 2, and `1/2020`'s `ever_seen` 1) — a closing residue.
    assert_eq!(e.counts(), [0, 0, 0, 0, 14, 0, 4]);
}

pub(super) fn incomplete_long(log: &Log<'_>) -> Vec<&'static str> {
    let init = log.initial().expect("initial state");
    let mut errors = Vec::new();
    if personality_brackets(&init.checksums).len() != 1 {
        errors.push("missing AI personality seed bracket");
    }
    // Shutdown has two documented shapes: trailing siblings or children of
    // the final FRAME. The latter is already read by frame_states.
    let closing = log.final_state().or_else(|| log.frame_states().pop());
    if closing.is_none_or(|f| f.units.is_empty()) {
        errors.push("missing closing units");
    }
    errors
}

fn check_long(name: &str) {
    let Some(path) = dump(name) else { return };
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    assert_eq!(
        incomplete_long(&log),
        Vec::<&str>::new(),
        "{name} is not scoreable"
    );
}

#[test]
fn run596_is_refused_as_a_complete_pair_capture() {
    let Some(path) = dump("gamelog-run596-islands-french-toughest-24k-trace.txt") else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        incomplete_long(&Log::parse(&text)),
        [
            "missing AI personality seed bracket",
            "missing closing units"
        ]
    );
}

#[test]
fn third_pair_openings_have_the_original_figures_and_personality() {
    for (start, base) in [(EAST_START, EAST_LONG), (LAKES_START, LAKES_LONG)] {
        let Some(w) = harness::tests::widen_on_siblings(
            &[start],
            true,
            base,
            start,
            &[(start, 1)],
            (1, 1),
            1,
            &[1],
            true,
        ) else {
            continue;
        };
        eprintln!(
            "opening {start}: {} blocks, {} differing keys; missing {:?}",
            w.blocks,
            w.firsts.len(),
            w.missing
        );
        assert!(w.missing.is_empty());
        assert!(
            w.firsts.keys().all(|(_, _, field)| field != "guys.len"
                && !field.starts_with("g.")
                && !field.starts_with("leader:PERSONALITY.")),
            "the starting cast and AI personality must agree: {:?}",
            w.firsts
        );
        assert_eq!(w.blocks, 1);
    }
}

/// run602: the third pair's East Indies word, whole records around frame 986.
pub(crate) fn french_east_indies_capacity_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run602",
        &[("gamelog-run602-islands-french-toughest-986.txt", 981)],
        WIDENING_FRENCH_CAPACITY,
        1,
        &[987],
        true,
    )
}

/// run642: the successor after the wonder arm's points, frame 15344
/// (item 1481). The word left it for 16857 (item 1487).
pub(crate) fn french_east_indies_15344_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run642",
        &[("gamelog-run642-islands-french-15344.txt", 15339)],
        WIDENING_FRENCH_EAST_INDIES_15344,
        1,
        &[15345],
        true,
    )
}

/// run655: the successor after the territory timer's ×100, frame 16857
/// (item 1487). The word left it for 17171 (item 1500).
pub(crate) fn french_east_indies_16857_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run655",
        &[("gamelog-run655-islands-french-16857.txt", 16851)],
        WIDENING_FRENCH_EAST_INDIES_16857,
        1,
        &[16858],
        true,
    )
}

/// run658: the gap 16869..17164 before the word 17171, blocks
/// 17018..17057 (item 1502): no word of its own, the first parting's date.
pub(crate) fn french_east_indies_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run658",
        &[("gamelog-run658-islands-french-17025.txt", 17018)],
        WIDENING_FRENCH_EAST_INDIES_GAP,
        1,
        &[17026],
        true,
    )
}

#[test]
fn run658_dates_the_gap_before_17171() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 40, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1502: 167 — 166 standing on the first block, 17018, on the tree
    // merged after 1496: run655's set with group 64's slots and `form`s
    // moved, item 1493's zero-filled slots (`group:64` 10, `group:66` 0)
    // and item 1496's five `tech_frame` keys agreeing, the group-move ids
    // of the move both sides issued on 16882, `caras` taking a new value in
    // the original, and `1/69`, `1/71` and `1/83`'s attacks on Napata
    // agreeing now that a building target the link table lacks is named;
    // then `gather_stamp` on 17033, ours 16744 against 17032, the frame the
    // dirty bit below rose for, and `treaties` 1 against 3 on both leaders
    // and Napata's `raid_stamp` 0 against 17053 on 17054, the first strike.
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 139, "initial run658 baseline"); // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
    // **The date, on the original's side** (`docs/AI.md` §131). Leader 0's
    // economy-dirty bit, `leader_flags & 0x2000000`, which no row compares:
    // the blocks of the window it stands on, read off the dump (the closing
    // block, 17063, carries it again).
    let path = dump("gamelog-run658-islands-french-17025.txt").unwrap();
    let mut ix = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
    let dirty: Vec<i64> = (0..ix.frames().len())
        .filter_map(|at| {
            let f = ix.frame_state(at).unwrap();
            let (lo, hi) = WIDENING_FRENCH_EAST_INDIES_GAP;
            let l = f.leaders.iter().find(|l| l.who == 0)?;
            ((lo..=hi).contains(&f.n) && l.leader_flags & 0x200_0000 != 0).then_some(f.n)
        })
        .collect();
    pin_eq!(
        (dirty.first().copied(), dirty.last().copied(), dirty.len()),
        (Some(17026), Some(17032), 7),
        "leader 0's dirty bit"
    );
}

/// run662: the successor after `do_attack_to`'s look was gated on the order
/// it was dispatched for, frame 17318 (item 1514), to the game's end.
pub(crate) fn french_east_indies_17318_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run662",
        &[("gamelog-run662-islands-french-17318.txt", 17312)],
        WIDENING_FRENCH_EAST_INDIES_17318,
        1,
        &[17319],
        true,
    )
}

/// run667: French East Indies' last running blocks and its closing state
/// (item 1519), the window the closed word 17379 is widened on.
pub(crate) fn french_east_indies_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run667",
        &[("gamelog-run667-islands-french-closing-window.txt", 17374)],
        WIDENING_FRENCH_EAST_INDIES_CLOSING,
        1,
        &[17380],
        true,
    )
}

#[test]
fn run667_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 7, "six running blocks and the closing state");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1519: 227. 140 stand on 17374 (run662's families: leader 0's
    // `SITE.reg`, the `form`s, the Merchants' pieces, who=1's line counts);
    // two on 17379, the Village `0/2000`'s `build:damage` 174 against 178 and
    // `damage_frac`, the family run662 parts on 17330; and 85 on the closing
    // block 17380, which are the human's defeat: this capture quits on 17379
    // (`defeat_stamp 17379`), the original clears player 0's units to idle
    // with no order (`0/1`..`0/5`) and moves leader 0's `leftover`, and the
    // harness replays no quit. The draws agree on every frame.
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 198, "initial run667 baseline"); // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
}

#[test]
fn run662_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_17318_window() else {
        return;
    };
    pin_eq!(w.blocks, 66, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1514: 562 — 138 standing on the first block, 17312 (run661's
    // families: leader 0's sites and treaties, Napata's counts, the
    // Merchants' pieces, who=1's `num_units`, `1/72` and `1/171`'s hit
    // points), none on 17313..17318, and on the word's block 17319 25: the
    // Elite Pikeman `1/115` (TypeIndex 135) walks a three-order stack, a
    // `MOVE_TO` at its head and its path 1, where the original's stands on
    // its lone `GUARD` (`cur_anim` 7 against 0, at (6122, 8925) against
    // (6120, 8952)) — the draws ours spends under `Unit::find_attack_pos`;
    // `1/69`, `1/131` and `1/177`'s figure clocks beside it. 399 more on
    // 17320..17377, the game's last blocks.
    // Item 1519: 562 → 142. `find_attack_pos` reads a guard's activity and
    // refuses it a building (AI §138): `1/115` stands on its lone `GUARD` at
    // (6120, 8952) on 17319 on both sides (`order:kind` 12, `order:length`
    // 1, `g.cur_anim[0]` 0), and its 25 rows there and the 399 after go with
    // it. 138 stand on 17312, as before; four part past the word, the draws
    // agreeing to the trace's end: the Village `0/2000`'s `build:damage` 120
    // against 121 and `damage_frac` 4 against 8 on 17330, leader 1's `caras`
    // 5 against 6 and `1/96`'s `form` −1 against 0 on 17331.
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 113, "initial run662 baseline"); // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
}

/// run661: the successor after the guard's leg and the fire on the move,
/// frame 17244 (item 1508).
pub(crate) fn french_east_indies_17244_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run661",
        &[("gamelog-run661-islands-french-17244.txt", 17238)],
        WIDENING_FRENCH_EAST_INDIES_17244,
        1,
        &[17245],
        true,
    )
}

#[test]
fn run661_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_17244_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1508: 254 — 139 standing on the first block, 17238: run657's
    // families (group 64's `off`/`curr` and the `form`s, leader 0's sites
    // and treaties, Napata's city counts, `1/72` and `1/171`'s hit points),
    // plus three that parted in the gap 17183..17237: the Merchants `1/20`,
    // `1/35` and `1/36` (TypeIndex 61) carry pieces 2123 and 14795 here
    // against 50689 on both figures there, and who=1's `num_units` reads −2
    // and 2 on the Artillery and Howitzer lines (270, 271) against 0 and 0.
    // Then one on 17244, `1/67`'s move `pause` 15 against 0, and on the word's
    // block 17245 38, `1/67` standing (`cur_anim` 0, `stopped` 1) where it
    // walks (`cur_anim` 7) — the draw ours spends under `Unit::do_move+0x11cf`
    // and the original does not; 76 more on 17246..17250.
    // Item 1514: 254 → 139, every one standing on 17238. `do_attack_to`'s
    // look runs only on the order it was dispatched for (AI §135): `1/67`'s
    // `pause` reads 0 on 17244 on both sides and it walks on 17245, and the
    // 76 rows after go with it.
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 111, "initial run661 baseline"); // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
}

/// run657: the successor after `move_step`'s snap stand, frame 17171
/// (item 1500).
pub(crate) fn french_east_indies_17171_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run657",
        &[("gamelog-run657-islands-french-17171.txt", 17165)],
        WIDENING_FRENCH_EAST_INDIES_17171,
        1,
        &[17172],
        true,
    )
}

#[test]
fn run657_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_17171_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1500: 240 — 178 standing on the first block, 17165 (run655's 173
    // less fourteen of group 64's slots and `form`s, plus `1/78`, `1/83` and
    // `1/90`'s front orders, kind 2 against 12 on Napata, leader 0's
    // `gather_stamp` 17152 against 17032, both `treaties` 1 against 3 and
    // Napata's `raid_stamp` 0 against 17154: the gap 16869..17164 parted);
    // nothing on 17166; on 17167 `1/69`'s front order, the same kind 2
    // against 12; six on 17169, `1/80`'s guy 0 swinging at `0/2000` there
    // and walking here; one on 17170; five on 17171, `1/80`'s order; then
    // 16 on the word's block 17172 and 29 on 17174..17176.
    // Item 1502: 230 → 225 on the tree merged after 1496 (1493 and 1496
    // took 240 → 230). Five `order:target` rows were ours reading
    // `None` for an attack on Napata, a building the link table lacks;
    // `target_ids` names it by `(owner, index)` now, and the five agree.
    // Item 1508: 225 → 154, all standing on 17165 but one. The guard's leg
    // killed under its attack (AI §134) takes `1/78`, `1/83` and `1/90`'s
    // stacks on 17165 and `1/69`'s and `1/80`'s on 17167 and 17171; the fire
    // on the move takes `1/80`'s swing on 17169, its reload on 17170 and its
    // walk on 17172, and `1/78`'s rows on 17174 and Napata's damage go with
    // them. Left past the first block: `group:66.role` on 17174.
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 126, "initial run657 baseline"); // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
}

#[test]
fn run655_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_16857_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1487: 186 — 173 standing on the first block, 16851; nothing on
    // 16852..16857; on the word's block 16858 five, the General `1/79`'s
    // animation (`g.cur_anim` 7 against 0, `cur_time` 16 against 1), the
    // draw ours spends through `Guy::move+0x19f` and the original through
    // `Unit::set_anim+0x56 < Unit::move_step+0x549`; then eight more on
    // 16860..16862. The word's block is 16858.
    // Item 1500: 186 → 173. The snap's stand (AI §130) takes all thirteen
    // rows past the first block: 1/79's guy 0 is `cur_anim` 0, `cur_time` 1,
    // `last_time` 0 on 16858 on both sides, and nothing parts on
    // 16852..16863. The 173 stand from the window's first block, 16851.
    // Item 1493: 173 → 170 on the tree merged after 1500 (army 65's siege
    // copy-back and the wider group-slot compare).
    // Item 1581: three fewer — the supply wagon's `myhits`, `hits_left` and `hits:myhits` agree (`docs/AI.md` §157).
    pin_eq!(w.firsts.len(), 138, "initial run655 baseline");
}

/// run649: the packet's capture, blocks 15086..15090 around who=1's army 6
/// taking its target on tick 15088 (item 1487).
pub(crate) fn french_east_indies_decision_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run649",
        &[("gamelog-run649-islands-french-packet-15088.txt", 15086)],
        WIDENING_FRENCH_EAST_INDIES_15088,
        1,
        &[15089],
        true,
    )
}

/// **The decision, both sides** (item 1487): on tick 15088 who=1's army 6
/// runs `find_target` from `do_transporting`, and with `popwin_timer` 1
/// since 14293 Napata scores 20040 against Paris's 345 (run649's packet).
/// Block 15089 prints leader 0's `frame_attacked` 15088 and `attacked_by`
/// 1; this crate's are the same, so no row of either parts.
#[test]
fn run649_s_decision_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_decision_window() else {
        return;
    };
    pin_eq!(w.blocks, 5, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin!(
        !w.firsts.keys().any(|(who, _, what)| *who == 0
            && (what.ends_with("frame_attacked") || what.ends_with("attacked_by"))),
        "the stamp of 15088 agrees"
    );
    pin!(
        !w.firsts.keys().any(|(_, _, what)| what.contains("popwin")),
        "the territory timer agrees"
    );
    // 88 on the tree merged after 1477 and 1466 (92 before), every one
    // standing on the first block 15086 (the human's census, the rows
    // run642 also stands on); nothing on 15087..15090, the decision's block
    // 15089 among them.
    pin_eq!(w.firsts.len(), 57, "run649 whole");
}

#[test]
fn run642_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_15344_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1481: 1818 on the tree merged after 1468 (1852 before it) — 100
    // standing on the first block, 15339; nothing on 15340..15344; 1447 on
    // the word's block 15345, who=1's group 64 re-formed and sent at Napata
    // there alone. Item 1487: 110 on the tree merged after 1477 and 1466
    // (114 before) — `find_target` takes Napata on 15088 as the original
    // does (`popwin_timer`'s ×100, run649), and 15344's order is ours too:
    // 94 standing on 15339, and on 15345 sixteen of the group's members'
    // `order:group.id`, 15344609 here against 15350409 there, alone.
    pin_eq!(w.firsts.len(), 78, "initial run642 baseline");
}

/// run639: the successor after Construction's clock and hit points,
/// frame 14782 (item 1476), and after the French Carpentry line, 14786
/// (item 1479), inside it. The word left it for 15344 (item 1481).
pub(crate) fn french_east_indies_14786_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run639",
        &[("gamelog-run639-islands-french-14782.txt", 14777)],
        WIDENING_FRENCH_EAST_INDIES_14786,
        1,
        &[14787],
        true,
    )
}

#[test]
fn run639_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_14786_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1476: 239 — 136 standing on the first block, 14777 (run636's
    // 134 less `caras` and three `form`s, with who=1's `best_good`,
    // `escrow[1:timber]` and `over_cap[1:timber]` and three `form`s among
    // the arrivals); nothing on 14778..14782; on the word's block 14783
    // who=1's `MAKE[8].val` 9999999 against 99999 and `1/93`'s move order
    // 192 off on each axis.
    // Item 1479: 239 → 226, the word's frame now 14786 (the French
    // Carpentry line, AI §123): who=1's nine standing timber rows and
    // `discovered` leave, so 127 stand on 14777; nothing parts on
    // 14778..14784; on 14785 who=1's `MAKE[8]` holds a Temple (437, city 2)
    // here and Tikal (532, city 3) there, four fields; on 14786 65, the site
    // `1/2054` and the food and timber it costs among them; ten on each of
    // 14787..14789. The word's block is 14787.
    // Item 1468: 226 → 192, as run635's.
    // Item 1481: 192 → 93 (226 → 127 before 1468), the word's frame now
    // 15344 (the wonder arm takes Tikal's `WONDER_VAL` 2 and the team's
    // held Pyramids, AI §124): `MAKE[8]` is Tikal at 1563477 on 14785 here
    // as there, and every row from 14785 leaves — what is left is the 93
    // standing on 14777.
    pin_eq!(w.firsts.len(), 59, "initial run639 baseline");
}

/// run636: the successor after a French General's craft rate, frame
/// 14090 (item 1470).
pub(crate) fn french_east_indies_14090_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run636",
        &[("gamelog-run636-islands-french-14090.txt", 14085)],
        WIDENING_FRENCH_EAST_INDIES_14090,
        1,
        &[14091],
        true,
    )
}

#[test]
fn run636_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_14090_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1470: 210 — 139 standing on the first block, 14085 (run635's
    // 118 less `1/112`'s `form`, `group:73.role` and `known_rares`, with
    // who=1's timber, `caras`, `discovered` and five `constr_time`s among
    // the arrivals); nothing on 14086; on block 14087 thirteen of who=1's
    // buildings' `regen_roads` 0 against 1, beside `1/49`'s order kind 6
    // against 7. The word's block is 14091.
    // Item 1476: 210 → 134, the word's frame now 14782 (Construction's
    // `BUILDINGS_FASTER_1` and `BUILDINGS_HP_1` reach the clock and the
    // hits, AI §121): the five `constr_time`s leave, `1/2047` completes on
    // 14087 on both sides, and every row from 14087 goes with it — what
    // is left is the 134 standing on 14085.
    // Item 1479: 134 → 128 — who=1's `discovered` and five timber rows
    // (`bucket`, `leftover`, `resources`, `income`, `rate`) leave: the
    // French Carpentry line, handed out with Chemistry on 12958 (AI §123).
    // Item 1468: 128 → 94, as run635's.
    pin_eq!(w.firsts.len(), 59, "initial run636 baseline");
}

/// run635: the successor after the crew's `GuyData::get_speed`, frame
/// 12952 (item 1461).
pub(crate) fn french_east_indies_12952_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run635",
        &[("gamelog-run635-islands-french-12952.txt", 12947)],
        WIDENING_FRENCH_EAST_INDIES_12952,
        1,
        &[12953],
        true,
    )
}

#[test]
fn run635_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_12952_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1461: 640 — run634's 118 standing rows on the first block, 12947
    // (`1/112`'s `form` for `1/109`'s), and nothing new before the word's
    // block 12953, where who=1's walkers step short (`1/80` 37 against 52).
    // Item 1470: 640 → 119, the word's frame now 14090 (a French General
    // recovers craft at `r = 4`, AI §119): the 118 standing rows on 12947,
    // and who=1's `leader:discovered` 33 against 34 on the last block,
    // 12959, which stood in the 640 too. Nothing parts on 12948..12958.
    // Item 1479: 119 → 118 — the `discovered` row was the French
    // Carpentry line, handed out free with Chemistry on 12958 (AI §123).
    // Item 1468: 118 → 88; the same two groups on the French pair's trained squads.
    pin_eq!(w.firsts.len(), 56, "initial run635 baseline");
}

/// run634: the successor after the French siege cost, frame 12794
/// (item 1460).
pub(crate) fn french_east_indies_12794_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run634",
        &[("gamelog-run634-islands-french-12794.txt", 12789)],
        WIDENING_FRENCH_EAST_INDIES_12794,
        1,
        &[12795],
        true,
    )
}

#[test]
fn run634_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_12794_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1461: 206 → 119, the word's frame now 12952 (run634's 87 keys past its first block leave, none arrive); a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
    pin_eq!(w.firsts.len(), 56, "initial run634 baseline");
}

/// run631: the successor after the member's re-placed slot and the
/// recycled slot's `path_recursion`, frame 11582 (item 1458).
pub(crate) fn french_east_indies_11582_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run631",
        &[("gamelog-run631-islands-french-11582.txt", 11577)],
        WIDENING_FRENCH_EAST_INDIES_11582,
        1,
        &[11583],
        true,
    )
}

#[test]
fn run631_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_11582_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1460: 216 → 120; a French unit of the Siege Factory line costs `FRENCH_SIEGE_COST` less (`docs/COSTS.md`, "A French siege unit costs less").
    pin_eq!(w.firsts.len(), 58, "initial run631 baseline");
}

/// run629: the successor after the Militia ramp and the French siege move,
/// frame 10802 (item 1455).
pub(crate) fn french_east_indies_10802_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run629",
        &[("gamelog-run629-islands-french-10802.txt", 10797)],
        WIDENING_FRENCH_EAST_INDIES_10802,
        1,
        &[10803],
        true,
    )
}

#[test]
fn run629_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_10802_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1457: 197 → 193; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 193 → 97; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    // Item 1461: 97 → 90; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
    pin_eq!(w.firsts.len(), 30, "initial run629 baseline");
}

/// run630: the muster before the word 10802 (item 1457).
pub(crate) fn french_east_indies_muster_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run630",
        &[("gamelog-run630-islands-french-muster.txt", 10760)],
        WIDENING_FRENCH_MUSTER_10765,
        1,
        &[10766],
        true,
    )
}

#[test]
fn run630_s_muster_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_muster_window() else {
        return;
    };
    pin_eq!(w.blocks, 37, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1458: 167 → 101; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    // Item 1461: 101 → 92; a tracked crew figure steps on `GuyData::get_speed` (`docs/MOVEMENT.md`, "The crew's speed").
    pin_eq!(
        w.firsts.len(),
        32,
        "run630 after the members' come-out and the pushed slot's normalize"
    );
}

/// run624: the successor after the Pyramids' city terms and `already_built`,
/// frame 10131 (item 1454).
pub(crate) fn french_east_indies_10131_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run624",
        &[("gamelog-run624-islands-french-10131.txt", 10126)],
        WIDENING_FRENCH_EAST_INDIES_10131,
        1,
        &[10132],
        true,
    )
}

#[test]
fn run624_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_10131_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1457: 97 → 95; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 95 → 90; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(
        w.firsts.len(),
        30,
        "run624 after the Militia ramp and the French siege move"
    );
}

/// run625: the Hoplite purchase on 9985 (item 1455).
pub(crate) fn french_east_indies_hoplite_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run625",
        &[("gamelog-run625-islands-french-9985.txt", 9980)],
        WIDENING_FRENCH_HOPLITE_9985,
        1,
        &[9986],
        true,
    )
}

#[test]
fn run625_s_hoplite_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_hoplite_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1457: 97 → 95; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 95 → 90; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(
        w.firsts.len(),
        30,
        "initial run625 baseline: its standing keys on 9980"
    );
}

/// run627: the frame food first parts on, 7782 (item 1455).
pub(crate) fn french_east_indies_food_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run627",
        &[("gamelog-run627-islands-french-7782.txt", 7776)],
        WIDENING_FRENCH_FOOD_7782,
        1,
        &[7783],
        true,
    )
}

#[test]
fn run627_s_food_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_food_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1457: 102 → 100; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 100 → 95; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(
        w.firsts.len(),
        39,
        "initial run627 baseline: its standing keys on 7776"
    );
}

/// run623: the successor after the gull's flight, frame 9777 (item 1453).
pub(crate) fn french_east_indies_9777_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run623",
        &[("gamelog-run623-islands-french-9777.txt", 9772)],
        WIDENING_FRENCH_EAST_INDIES_9777,
        1,
        &[9778],
        true,
    )
}

#[test]
fn run623_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_9777_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 97 → 96, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 96 → 92; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 92 → 87; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(
        w.firsts.len(),
        25,
        "run623 after the fifth city and `already_built`"
    );
}

/// run622: the successor after the idle push-back, frame 9655 (item 1452).
pub(crate) fn french_east_indies_9655_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run622",
        &[("gamelog-run622-islands-french-9655.txt", 9650)],
        WIDENING_FRENCH_EAST_INDIES_9655,
        1,
        &[9656],
        true,
    )
}

#[test]
fn run622_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_9655_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 96 → 95, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 95 → 93; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 93 → 88; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 27, "run622 after the gull's flight");
}

/// run617: the successor after `largest_gather`, frame 8840 (item 1451).
pub(crate) fn french_east_indies_8840_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run617",
        &[("gamelog-run617-islands-french-8840.txt", 8835)],
        WIDENING_FRENCH_EAST_INDIES_8840,
        1,
        &[8841],
        true,
    )
}

#[test]
fn run617_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_8840_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 111 → 110, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1458: 110 → 105; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 45, "run617 after the idle push-back");
}

/// run616: the successor after the per-building queue, frame 8385 (item
/// 1449).
pub(crate) fn french_east_indies_8385_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run616",
        &[("gamelog-run616-islands-french-8385.txt", 8380)],
        WIDENING_FRENCH_EAST_INDIES_8385,
        1,
        &[8386],
        true,
    )
}

#[test]
fn run616_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_8385_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 109 → 108, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1458: 108 → 103; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 44, "run616 after the idle push-back");
}

/// run612: the successor after a wonder's start became first contact,
/// frame 8236 (item 1446).
pub(crate) fn french_east_indies_8236_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run612",
        &[("gamelog-run612-islands-french-8236.txt", 8231)],
        WIDENING_FRENCH_EAST_INDIES_8236,
        1,
        &[8237],
        true,
    )
}

#[test]
fn run612_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_8236_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 106 → 104, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1458: 104 → 99; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 39, "run612 after the idle push-back");
}

/// run610: the successor after the university admission limit, frame 8182.
pub(crate) fn french_east_indies_8182_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run610",
        &[("gamelog-run610-islands-french-toughest-8182.txt", 8177)],
        WIDENING_FRENCH_EAST_INDIES_8182,
        1,
        &[8183],
        true,
    )
}

#[test]
fn run610_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_8182_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 109 → 107, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 107 → 105; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 105 → 100; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 41, "run610 after the idle push-back");
}

/// run611: the first capture of this game past first contact, blocks
/// 8030..8043 (item 1446).
pub(crate) fn french_east_indies_contact_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run611",
        &[("gamelog-run611-islands-french-contact.txt", 8030)],
        WIDENING_FRENCH_CONTACT,
        1,
        &[8034],
        true,
    )
}

/// run613: the contact frame's own window, blocks 7940..7952 (item 1446).
pub(crate) fn french_east_indies_contact_7946_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run613",
        &[("gamelog-run613-islands-french-contact-7946.txt", 7940)],
        WIDENING_FRENCH_CONTACT_7946,
        1,
        &[7946],
        true,
    )
}

#[test]
fn run613_s_contact_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_contact_7946_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 106 → 102, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 102 → 100; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 100 → 95; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 39, "run613 after the idle push-back");
    pin!(
        !w.firsts
            .keys()
            .any(|(_, _, field)| field.starts_with("leader:treaties") || field == "leader:wars"),
        "both met bits and the AI's war census agree on every block"
    );
}

/// **The original's own record of the contact, read off run613** (item
/// 1446): the wonder `1/2022` is started on sim-frame 7941 (block 7942),
/// and on block 7946 — the owner's `check_ever_seen` frame 7945 — both it
/// and the Senate `1/2021` beside it go from `ever_seen` 2 to 255 while
/// both leaders' met bits are set, and on no earlier block. The shared
/// instrument compares both bytes on every building of every window since
/// item 1563 (parked 1450), so the French windows' widenings hold this
/// crate to it; item 1446's own walk of them retired into that.
#[test]
fn run613_dates_first_contact_on_block_7946() {
    let Some(path) = dump("gamelog-run613-islands-french-contact-7946.txt") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
    let mut seen = Vec::new();
    for at in 0..ix.frames().len() {
        let f = ix.frame_state(at).unwrap();
        if !(7940..=7952).contains(&f.n) {
            continue;
        }
        let es = |o: i64| {
            f.builds
                .iter()
                .find(|b| (b.who, b.o) == (1, o))
                .and_then(|b| b.ever_seen)
        };
        let raw = ix.read_frame(at).unwrap();
        let log = Log::parse(&raw);
        let met = |who: i64, other: &str| {
            let block = log.leader_block(f.n, who)?;
            crate::diff::leader::theirs(&block).get(other).copied()
        };
        seen.push((
            f.n,
            es(2021),
            es(2022),
            met(1, "treaties[0]"),
            met(0, "treaties[1]"),
        ));
    }
    assert_eq!(seen.len(), 13);
    for (n, a, b, m1, m0) in seen {
        let after = n >= 7946;
        let byte = Some(if after { 255 } else { 2 });
        let bit = Some(i64::from(after));
        assert_eq!((a, b, m1, m0), (byte, byte, bit, bit), "block {n}");
    }
}

/// run620: ship `1/16` pushed by transport `1/39`, blocks 6136..6157
/// (item 1452).
pub(crate) fn french_east_indies_ship_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run620",
        &[("gamelog-run620-islands-french-ship-6141.txt", 6136)],
        WIDENING_FRENCH_SHIP_6141,
        1,
        &[6142],
        true,
    )
}

#[test]
fn run620_s_ship_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_ship_window() else {
        return;
    };
    pin_eq!(w.blocks, 22, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1457: 86 → 84; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 84 → 83; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 38, "run620 after the idle push-back");
    // `form` is the standing new-unit residue (−1 here, 0 there) every
    // window carries for a unit born after the start; nothing else of
    // either ship parts.
    pin!(
        !w.firsts
            .keys()
            .any(|(who, o, field)| *who == 1 && matches!(*o, 16 | 39) && field != "form"),
        "the pushed ship and its pusher agree on every block"
    );
}

/// run618: the caravan `1/29` boards its transport `1/60`, blocks
/// 8420..8439 (item 1452).
pub(crate) fn french_east_indies_transport_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run618",
        &[("gamelog-run618-islands-french-transport-8430.txt", 8420)],
        WIDENING_FRENCH_TRANSPORT_8430,
        1,
        &[8430],
        true,
    )
}

#[test]
fn run618_s_boarding_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_transport_window() else {
        return;
    };
    pin_eq!(w.blocks, 20, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 112 → 111, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 111 → 109; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 109 → 104; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 44, "run618 after the idle push-back");
}

/// run615: the builder `1/51`'s birth beside the wonder, blocks
/// 7958..7999 (item 1449).
pub(crate) fn french_east_indies_builder_7963_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run615",
        &[("gamelog-run615-islands-french-builder-7963.txt", 7958)],
        WIDENING_FRENCH_BUILDER_7963,
        1,
        &[7963],
        true,
    )
}

#[test]
fn run615_s_birth_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_builder_7963_window() else {
        return;
    };
    pin_eq!(w.blocks, 42, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 109 → 105, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 105 → 103; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 103 → 98; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 40, "run615 after the idle push-back");
    // **The recruit at birth** (item 1449): city `1/2008` trains citizen
    // `1/51` on frame 7962, the wonder `1/2022` recruits it on its phase the
    // same frame, and it walks to the original's spot. With the queues in a
    // pass of their own it was born idle and sent to gather from `1/2019`.
    pin!(
        !w.firsts
            .keys()
            .any(|(who, o, _)| *who == 1 && matches!(*o, 51 | 2019)),
        "the new citizen's orders, its walk and the camp it no longer joins"
    );
}

/// run614: the builder `1/56`'s birth and its approach to the wonder,
/// blocks 8140..8159 (item 1449).
pub(crate) fn french_east_indies_builder_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run614",
        &[("gamelog-run614-islands-french-builder-8144.txt", 8140)],
        WIDENING_FRENCH_BUILDER_8156,
        1,
        &[8156],
        true,
    )
}

#[test]
fn run614_s_builder_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_builder_window() else {
        return;
    };
    pin_eq!(w.blocks, 20, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 111 → 108, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 108 → 106; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 106 → 101; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 41, "run614 after the idle push-back");
    pin!(
        !w.firsts
            .keys()
            .any(|(who, o, _)| *who == 1 && matches!(*o, 51 | 56)),
        "both builders agree on every block: 1/51 where it stands, 1/56 where it is sent"
    );
}

#[test]
fn run611_s_contact_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_contact_window() else {
        return;
    };
    pin_eq!(w.blocks, 14, "every captured block");
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1455: 110 → 107, who=1's food and its queued Citizens' costs — a Citizen is ramped by the Militia line too (`docs/COSTS.md`).
    // Item 1457: 107 → 105; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1458: 105 → 100; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 40, "run611 after the idle push-back");
}

/// run603: the successor word after French timber capacity, frame 7356.
pub(crate) fn french_east_indies_scholar_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EAST_START],
        true,
        EAST_LONG,
        "run603",
        &[("gamelog-run603-islands-french-toughest-7356.txt", 7351)],
        WIDENING_FRENCH_SCHOLAR,
        1,
        &[7357],
        true,
    )
}

#[test]
fn run603_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_scholar_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    // Item 1458: 97 → 93; a member's slot across a coast is re-placed on slot 0's, and a recycled slot keeps its `path_recursion` (`docs/GROUPS.md` §37).
    pin_eq!(w.firsts.len(), 38, "run603 after the idle push-back");
    assert!(
        !w.firsts.keys().any(|(who, o, field)| *who == 1
            && ((*o == 45 && field == "extra")
                || (*o == 2016 && field == "queue:queued")
                || (*o == -1
                    && matches!(
                        field.as_str(),
                        "leader:scholars"
                            | "leader:active"
                            | "leader:control"
                            | "leader:num_units[2]"
                            | "leader:num_queued[2]"
                    )))),
        "the surplus queue, birth and census counts agree"
    );
}

/// run601: the third pair's Great Lakes word, whole records around frame 2576.
pub(crate) fn french_great_lakes_ruins_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKES_START],
        true,
        LAKES_LONG,
        "run601",
        &[("gamelog-run601-lakes-french-toughest-2576.txt", 2571)],
        WIDENING_FRENCH_GREAT_LAKES,
        1,
        &[2577],
        true,
    )
}

pub(crate) fn french_great_lakes_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKES_START],
        true,
        LAKES_LONG,
        "run609",
        &[(
            "gamelog-run609-lakes-french-toughest-closing-window.txt",
            5633,
        )],
        WIDENING_FRENCH_LAKES_CLOSING,
        1,
        &[5639],
        true,
    )
}

#[test]
fn run609_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_great_lakes_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 7, "six running blocks and closing state");
    pin!(
        w.missing.is_empty(),
        "every record key is read: {:?}",
        w.missing
    );
    // Item 1457: 110 → 108; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    // Item 1502: 95 → 80 on the tree merged after 1496 (whose own re-pin
    // took 100 → 95). Fifteen `order:target` rows on 5633 (`1/9`..
    // `1/23`) were ours reading `None` for an attack on `0/2000`, a
    // building the link table lacks; named by `(owner, index)`, they agree.
    // Item 1563: 80 → 94; the shared instrument compares `ever_seen` and
    // `ever_seen_completed` (parked 1450), and on the closing block 5639
    // the seven of player 1's buildings `1/2019`..`1/2025` read 3 in the
    // original against 2 (`1/2020`'s `ever_seen` 1) — the closing residue
    // `run598_french_great_lakes_closing_state` counts.
    pin_eq!(
        w.firsts.len(),
        67, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
        "run609 closing residue, not whole-record parity"
    );
}

#[test]
fn run602_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_east_indies_capacity_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    pin_eq!(w.firsts.len(), 31, "run602 after French worker capacity");
    assert!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 2010)),
        "the preserved camp agrees in every compared field"
    );
    assert!(
        w.firsts
            .keys()
            .all(|(who, _, field)| *who != 1 || !field.ends_with("gather_slots[1:timber]")),
        "the opening timber slot deficit is gone"
    );
}

#[test]
fn run601_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_great_lakes_ruins_window() else {
        return;
    };
    pin_eq!(w.blocks, 13, "every captured block");
    pin!(
        w.missing.is_empty(),
        "all record keys are read: {:?}",
        w.missing
    );
    // Item 1457: 71 → 69; a member comes out with its orders, a pushed stack group's point is (0, 0), every push normalizes the last slot (`docs/GROUPS.md` §36).
    pin_eq!(w.firsts.len(), 40, "run601 after ruins placement");
    pin!(
        w.firsts.keys().all(|(who, _, field)| *who != 1
            || !(field.starts_with("leader:SITE[8].") || field.starts_with("leader:SITE[9]."))),
        "both formerly swapped site records agree throughout the window"
    );
}

/// An empty projectile comparison still checks both sides, while a missing
/// group dumper must never masquerade as an empty original pool.
#[test]
fn third_pair_windows_check_groups_and_projectiles_on_both_sides() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    for (start, base, capture, window) in [
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run662-islands-french-17318.txt",
            WIDENING_FRENCH_EAST_INDIES_17318,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run661-islands-french-17244.txt",
            WIDENING_FRENCH_EAST_INDIES_17244,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run657-islands-french-17171.txt",
            WIDENING_FRENCH_EAST_INDIES_17171,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run655-islands-french-16857.txt",
            WIDENING_FRENCH_EAST_INDIES_16857,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run642-islands-french-15344.txt",
            WIDENING_FRENCH_EAST_INDIES_15344,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run639-islands-french-14782.txt",
            WIDENING_FRENCH_EAST_INDIES_14786,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run636-islands-french-14090.txt",
            WIDENING_FRENCH_EAST_INDIES_14090,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run635-islands-french-12952.txt",
            WIDENING_FRENCH_EAST_INDIES_12952,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run634-islands-french-12794.txt",
            WIDENING_FRENCH_EAST_INDIES_12794,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run631-islands-french-11582.txt",
            WIDENING_FRENCH_EAST_INDIES_11582,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run629-islands-french-10802.txt",
            WIDENING_FRENCH_EAST_INDIES_10802,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run624-islands-french-10131.txt",
            WIDENING_FRENCH_EAST_INDIES_10131,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run623-islands-french-9777.txt",
            WIDENING_FRENCH_EAST_INDIES_9777,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run622-islands-french-9655.txt",
            WIDENING_FRENCH_EAST_INDIES_9655,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run617-islands-french-8840.txt",
            WIDENING_FRENCH_EAST_INDIES_8840,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run616-islands-french-8385.txt",
            WIDENING_FRENCH_EAST_INDIES_8385,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run612-islands-french-8236.txt",
            WIDENING_FRENCH_EAST_INDIES_8236,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run611-islands-french-contact.txt",
            WIDENING_FRENCH_CONTACT,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run613-islands-french-contact-7946.txt",
            WIDENING_FRENCH_CONTACT_7946,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run610-islands-french-toughest-8182.txt",
            WIDENING_FRENCH_EAST_INDIES_8182,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run603-islands-french-toughest-7356.txt",
            WIDENING_FRENCH_SCHOLAR,
        ),
        (
            EAST_START,
            EAST_LONG,
            "gamelog-run602-islands-french-toughest-986.txt",
            WIDENING_FRENCH_CAPACITY,
        ),
        (
            LAKES_START,
            LAKES_LONG,
            "gamelog-run601-lakes-french-toughest-2576.txt",
            WIDENING_FRENCH_GREAT_LAKES,
        ),
        (
            LAKES_START,
            LAKES_LONG,
            "gamelog-run609-lakes-french-toughest-closing-window.txt",
            WIDENING_FRENCH_LAKES_CLOSING,
        ),
    ] {
        let (Some(start), Some(base_path), Some(path)) = (dump(start), dump(base.0), dump(capture))
        else {
            continue;
        };
        let (start_text, base_text) =
            (crate::capture::read(start), crate::capture::read(base_path));
        let (start_log, base_log) = (Log::parse(&start_text), Log::parse(&base_text));
        let sibling = start_log.initial().unwrap();
        let mut init = base_log.initial().unwrap();
        assert!(
            same_start(&init, &sibling),
            "all starting building identities and positions"
        );
        assert_eq!(init.game_info, sibling.game_info, "the same lobby");
        assert_eq!(
            init.checksums.last().unwrap().seed,
            sibling.checksums.last().unwrap().seed
        );
        borrow_from_siblings(&mut init, &[&sibling]);
        if let Some(t) = trace(base.1) {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut ix = crate::capture::indexed::IndexedCapture::open(path).unwrap();
        let mut compared = 0;
        let mut ammo_counts = (0, 0, 0);
        let mut ammo_differences = std::collections::BTreeSet::new();
        let mut ammo_unmodelled = std::collections::BTreeSet::new();
        for n in 1..=window.1 {
            built.tick();
            if n < window.0 {
                continue;
            }
            let at = ix
                .frames()
                .iter()
                .position(|f| f.number == n)
                .expect("every window block");
            let raw = ix.read_frame(at).unwrap();
            let log = Log::parse(&raw);
            let (_, block) = log.frames().into_iter().find(|(f, _)| *f == n).unwrap();
            assert!(
                !crate::gamelog::groups(block).is_empty(),
                "{capture} block {n}: group dumper missing"
            );
            let mut theirs = std::collections::BTreeMap::new();
            for (a, fields) in crate::diff::ammo::blocks(&raw) {
                assert_eq!(fields, 27, "every printed ammo field");
                assert_eq!(a.flags & 3, 2, "live projectile, no unmodelled crash round");
                assert!(theirs.insert((a.who, a.o, n - a.cur_time), a).is_none());
                ammo_unmodelled.insert((
                    a.traj,
                    a.dx,
                    a.start_roll_angle,
                    a.bank_dx,
                    a.bank_dy,
                    a.gpiece,
                    a.graph_index,
                    a.flags & !0x1f,
                ));
            }
            let sim = &built.sim;
            let mut ours = std::collections::BTreeMap::new();
            for p in &sim.projectiles {
                let (who, o) = super::golden::obj_ident(sim, p.shooter);
                assert!(
                    ours.insert((who, o, n - i64::from(p.cur_time)), p)
                        .is_none()
                );
            }
            ammo_counts.0 += theirs.len();
            ammo_counts.1 += ours.len();
            let keys: std::collections::BTreeSet<_> =
                theirs.keys().chain(ours.keys()).copied().collect();
            for key in keys {
                let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                    ammo_differences.insert((
                        n,
                        key,
                        "presence",
                        i64::from(ours.contains_key(&key)),
                        i64::from(theirs.contains_key(&key)),
                    ));
                    continue;
                };
                let mut rows = super::golden::ammo_value_rows(sim, p, a);
                rows.push(("index", i64::from(p.slot), a.index));
                for (field, mine, dumped) in rows {
                    ammo_counts.2 += 1;
                    if mine != dumped {
                        ammo_differences.insert((n, key, field, mine, dumped));
                    }
                }
            }
            compared += 1;
        }
        assert_eq!(compared, window.1 - window.0 + 1);
        eprintln!(
            "{capture}: ammo counts {ammo_counts:?}, differences {ammo_differences:?}, unmodelled {ammo_unmodelled:?}"
        );
        if window == WIDENING_FRENCH_LAKES_CLOSING {
            assert_eq!(
                ammo_counts,
                (7, 7, 140),
                "every lifetime and twenty comparisons per record"
            );
            assert_eq!(
                ammo_differences,
                std::collections::BTreeSet::from([
                    (5633, (1, 22, 5625), "angle", -1111097344, -1111425024),
                    (5633, (1, 22, 5625), "sx", 4944, 4945),
                    (5633, (1, 22, 5625), "sy", 30710, 30709),
                ]),
                "three launch-geometry differences remain explicit"
            );
            assert_eq!(
                ammo_unmodelled,
                std::collections::BTreeSet::from([
                    (1, 155512146, 0, 0, 0, 60348, 90, 0),
                    (1, 161070679, 0, 0, 0, 60348, 91, 0),
                    (1, 161980988, 0, 0, 0, 60348, 92, 0),
                ]),
                "engine-only fields are evidence, not sim parity"
            );
        } else if window == WIDENING_FRENCH_EAST_INDIES_17318 {
            // Item 1514: run662, the word 17318's window, to the game's end.
            // Before the word's block 17319 the only rows are the Dragoons'
            // release offset (piece 60162, as run657's `1/90` and run661's
            // `1/80`): `1/80`'s round launched on 17309 and `1/90`'s on 17311
            // leave from the unit's square here and the release node there.
            // From 17321 on, the rounds are the word's parting.
            let mut want = std::collections::BTreeSet::new();
            for (field, mine, dumped) in [
                ("angle", -133758976, -142147584),
                ("ex", 6315, 6319),
                ("sx", 6648, 6640),
                ("sy", 9000, 8853),
                ("sz", 228, 436),
                ("v1z", 12225000, -39775002),
            ] {
                want.insert((17312, (1, 80, 17309), field, mine, dumped));
            }
            for n in 17312..=17315 {
                for (field, mine, dumped) in [
                    ("angle", -61276160, -68419584),
                    ("ex", 6176, 6179),
                    ("sx", 6456, 6474),
                    ("sy", 9240, 9179),
                    ("sz", 186, 448),
                    ("v1z", 27618750, -24781250),
                ] {
                    want.insert((n, (1, 90, 17311), field, mine, dumped));
                }
            }
            let before: std::collections::BTreeSet<_> = ammo_differences
                .iter()
                .filter(|d| d.0 < 17319)
                .cloned()
                .collect();
            assert_eq!(before, want, "the Dragoons' launches, explicit");
            // Item 1519: (70, 73, 840) and 344 → (70, 74, 1400) and 442 once
            // `1/115` stood (AI §138): more rounds pair and are compared, and
            // every row past the word is the same release-offset family
            // (`sx`/`sy`/`sz`/`v1z`/`angle` at launch, `ex`/`ey`, a frame of
            // `total_time`) or a round's `presence` a frame early.
            assert_eq!(
                (ammo_counts, ammo_differences.len() - before.len()),
                ((70, 74, 1400), 442),
                "the rounds past the word"
            );
            assert_eq!(
                ammo_unmodelled.len(),
                20,
                "engine-only fields are evidence, not sim parity"
            );
        } else if window == WIDENING_FRENCH_EAST_INDIES_17244 {
            // Item 1508: run661, the word 17244's window. The Dragoon
            // `1/80`'s round, launched on 17242 on both sides, lives the same
            // three blocks and leaves from another point: ours the unit's
            // square, 100 up, theirs the piece's release node — the same
            // unmeasured Dragoon offset (piece 60162) as run657's `1/90`.
            assert_eq!(
                ammo_counts,
                (3, 3, 60),
                "three lifetimes, twenty fields each"
            );
            let mut want = std::collections::BTreeSet::new();
            for n in 17243..=17245 {
                for (field, mine, dumped) in [
                    ("angle", -133758976, -131334144),
                    ("ex", 6362, 6361),
                    ("ey", 7290, 7291),
                    ("sx", 6648, 6611),
                    ("sy", 9000, 8852),
                    ("sz", 228, 451),
                    ("v1z", 12225000, -43525002),
                ] {
                    want.insert((n, (1, 80, 17242), field, mine, dumped));
                }
            }
            assert_eq!(ammo_differences, want, "1/80's launch, explicit");
            assert_eq!(
                ammo_unmodelled,
                std::collections::BTreeSet::from([(1, 395223114, 0, 0, 0, 60162, 5, 0)]),
                "engine-only fields are evidence, not sim parity"
            );
        } else if window == WIDENING_FRENCH_EAST_INDIES_17171 {
            // Item 1500: run657 is the first French East Indies window with
            // rounds in flight. `1/90`'s round, launched on 17165, leaves
            // from another point on 17166..17169: `1/90` is one of the
            // three units whose front order the gap 16869..17164 parted
            // (an attack on Napata in the original, a move in ours).
            assert_eq!(
                ammo_counts,
                (4, 4, 80),
                "four lifetimes, twenty fields each"
            );
            let mut want = std::collections::BTreeSet::new();
            for n in 17166..=17169 {
                for (field, mine, dumped) in [
                    ("angle", -61276160, -69140480),
                    ("ex", 6212, 6216),
                    ("sx", 6456, 6467),
                    ("sy", 9240, 9094),
                    ("sz", 186, 394),
                    ("v1z", 27618750, -13981250),
                ] {
                    want.insert((n, (1, 90, 17165), field, mine, dumped));
                }
            }
            assert_eq!(ammo_differences, want, "1/90's launch, explicit");
            assert_eq!(
                ammo_unmodelled,
                std::collections::BTreeSet::from([(1, 355165588, 0, 0, 0, 60162, 0, 0)]),
                "engine-only fields are evidence, not sim parity"
            );
        } else {
            assert_eq!(
                ammo_counts,
                (0, 0, 0),
                "both pools empty on earlier windows"
            );
        }
    }
}

thread_local!(static PREV: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) });

/// **The gulls fly where the original's do** (item 1453, `docs/SYNC.md`
/// §3.29): every frame of run620's and run622's call windows on which a
/// flyer with a dock for its goal took a step, the crate's gull of that
/// goal stands on the original's point after the same frame — both
/// gulls, from birth (gull 1 about 3,900 frames before run620's window).
/// Made to fail by dropping the gull's birth snap (gull 1 is about a
/// thousand units off on 6137) or its flight (it never moves).
#[test]
fn french_east_indies_gulls_fly_where_the_original_s_do() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(start), Some(base_path)) = (dump(EAST_START), dump(EAST_LONG.0)) else {
        return;
    };
    // frame → [(goal, the point the flyer was put on)]
    type Flight = Vec<((i32, i32), (i32, i32))>;
    let mut want: std::collections::BTreeMap<i64, Flight> = std::collections::BTreeMap::new();
    for t in ["rontrace-run620.log", "rontrace-run622.log"] {
        let Some(tr) = trace(t) else { return };
        for a in tr.air_frames() {
            if let Some(to) = a.to {
                want.entry(a.frame).or_default().push((a.goal, to));
            }
        }
    }
    let loaded = crate::load::load(&inst).unwrap();
    let (start_text, base_text) = (crate::capture::read(start), crate::capture::read(base_path));
    let (start_log, base_log) = (Log::parse(&start_text), Log::parse(&base_text));
    let sibling = start_log.initial().unwrap();
    let mut init = base_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    if let Some(t) = trace(EAST_LONG.1) {
        borrow_pasture(&mut init, &t);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let last = *want.keys().last().unwrap();
    let mut checked = 0;
    // Block n is the state after the original's frame n − 1.
    for n in 1..=last + 1 {
        built.tick();
        let Some(rows) = want.get(&(n - 1)) else {
            continue;
        };
        let sim = &built.sim;
        for &(goal, to) in rows {
            // A gull is the flyer whose goal is its dock; a wild bird's
            // patrol point names no dock, and is §3.9's to check.
            let Some(u) = (0..sim.units.len()).find(|&u| {
                sim.units[u].alive() && sim.gull_dock(u).is_some_and(|p| (p.x, p.y) == goal)
            }) else {
                continue;
            };
            let p = sim.units[u].pos;
            assert_eq!((p.x, p.y), to, "frame {}: the gull of {goal:?}", n - 1);
            checked += 1;
        }
    }
    // run620 holds gull 1 alone over 6134..6158 and run622 both gulls
    // over 9648..9663.
    assert!(checked >= 50, "{checked} gull frames compared");
}

/// Item 1454: French East Indies' fifth city, bought on 9777. The AI
/// holds the Pyramids from 9658, so its limit is five (`get_city_limit`'s
/// `has_wonder(0x20e)`) and a city costs a third less (`get_cost`'s
/// `PYRAMIDS_CITY_DISCOUNT`): 140 food and 140 timber where the ramp asks
/// 210. run623's buckets, who=1: food 501 → 361 and timber 158 → 18
/// from block 9777 to 9778.
#[test]
fn french_east_indies_buys_its_fifth_city_at_a_third_off() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(start), Some(base_path)) = (dump(EAST_START), dump(EAST_LONG.0)) else {
        return;
    };
    if dump("gamelog-run623-islands-french-9777.txt").is_none() {
        return;
    }
    let loaded = crate::load::load(&inst).unwrap();
    let (start_text, base_text) = (crate::capture::read(start), crate::capture::read(base_path));
    let (start_log, base_log) = (Log::parse(&start_text), Log::parse(&base_text));
    let sibling = start_log.initial().unwrap();
    let mut init = base_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    if let Some(t) = trace(EAST_LONG.1) {
        borrow_pasture(&mut init, &t);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    // After n ticks the state is block n.
    for _ in 0..9777 {
        built.tick();
    }
    let sim = &built.sim;
    let village = (0..sim.build_types.len())
        .find(|&t| sim.build_types[t].ident == sim::build::Ident::Village)
        .unwrap();
    assert_eq!(sim.city_limit(1), 5, "four cities and the Pyramids");
    let price = sim.building_price(1, village);
    assert_eq!((price[0], price[1]), (140, 140), "the fifth city's price");
    // Food stands three above the original's from before run623's window
    // (`leader:bucket[0:food]`, 500 against 497 on 9772); timber agrees.
    let before = sim.ledgers[1].bucket;
    assert_eq!(before[1], 158);
    built.tick();
    let after = built.sim.ledgers[1].bucket;
    assert_eq!((before[0] - after[0], before[1] - after[1]), (140, 140));
    assert_eq!(after[1], 18);
}
