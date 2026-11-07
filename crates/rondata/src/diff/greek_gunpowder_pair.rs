//! **The Greeks at a held Gunpowder age, Great Sahara** (DECISIONS 90, 91,
//! 93, 94, item 3626): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Greeks (5)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `egyptian_gunpowder_pair`'s lobby with the
//! nation moved. Its triple, `(7, 5, 5)`, is new: the battery fields no
//! Greeks, `greek_word`'s `GreatSaharaGreekTier1` is `(7, 5, 1)`, another
//! difficulty, and `greek_classical_pair`'s `(18, 5, 5)` is another map.
//! Captures: run1284 (a `DUMP_ALL` start) and run1285 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §334.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GREEK_GUNPOWDER: Pair = Pair {
    name: "GreatSaharaGreekGunpowder",
    map_style: 7,
    ai_tribe: 5,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1284-greatsahara-greek-gunpowder-start.txt",
    long: (
        "gamelog-run1285-greatsahara-greek-gunpowder-trace.txt",
        "rontrace-run1285.log",
    ),
    // The game ended itself on 4074 (closing block 4075).
    length: 4074,
    // Item 3626, first try: **whole** — the draw count and the sequence
    // agree through the game's last frame, 4074 of 4074, and the closing
    // state compares (`greek_gunpowder_pair_run1285_s_closing_state`).
    count: 4074,
    sequence: 4074,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GREEK_GUNPOWDER_PAIR_WORD: i64 =
    if GREEK_GUNPOWDER.count < GREEK_GUNPOWDER.sequence {
        GREEK_GUNPOWDER.count
    } else {
        GREEK_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn greek_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GREEK_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GREEK_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3, and the long is scoreable.
#[test]
fn greek_gunpowder_pair_captures_say_their_lobby() {
    let p = &GREEK_GUNPOWDER;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Greek Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3626; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no Greeks
/// since `GreatSaharaGreekTier1`, `(7, 5, 1)`, left it for the AI word
/// (another difficulty), and the Classical pair's `(18, 5, 5)` is another
/// map (item 3626).
#[test]
fn greek_gunpowder_pair_is_no_other_lobby() {
    let p = &GREEK_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Greeks"
    );
    for (name, other) in [
        (
            super::greek_word::GREEK.name,
            (
                super::greek_word::GREEK.map_style,
                super::greek_word::GREEK.ai_tribe,
                super::greek_word::GREEK.difficulty,
            ),
        ),
        (
            super::greek_classical_pair::GREEK_CLASSICAL.name,
            (
                super::greek_classical_pair::GREEK_CLASSICAL.map_style,
                super::greek_classical_pair::GREEK_CLASSICAL.ai_tribe,
                super::greek_classical_pair::GREEK_CLASSICAL.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The pair's closing state**: run1285's closing dump, block 4075,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3626).
#[test]
fn greek_gunpowder_pair_run1285_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GREEK_GUNPOWDER;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
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
        "Greek Gunpowder pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 4075, "the closing block");
    // Item 3626, the pair whole on its first try: 211 units compared, none
    // off, torn or unlinked, and no building or city field-row.
    pin_eq!(e.compared, 211, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1285b: the pair's last blocks, 3819..4075 at the long's detail with
/// `AMMO=5` — the word 4074 is the game's last frame and 4075 its closing
/// block (item 3626). run1285 prints units only every hundredth block and
/// no `LEADERS`, `BUILDS` or `CITIES` record past the start, so no dump
/// held the last blocks.
pub(crate) const RUN1285B: &str =
    "gamelog-run1285b-greatsahara-greek-gunpowder-window-3819-4075.txt";

/// The window: block 3819 through the closing block 4075.
pub(crate) const WINDOW_GREEK_GUNPOWDER_CLOSING: (i64, i64) = (3819, 4075);

/// **The pair's last blocks**: run1285b walked from run1284's start with
/// the recorder on. The word is the game's last frame (item 3626).
pub(crate) fn greek_gunpowder_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK_GUNPOWDER.start],
        true,
        GREEK_GUNPOWDER.long,
        "run1285b",
        &[(RUN1285B, WINDOW_GREEK_GUNPOWDER_CLOSING.0)],
        WINDOW_GREEK_GUNPOWDER_CLOSING,
        1,
        &[4075],
        true,
    )
}

#[test]
fn greek_gunpowder_pair_run1285b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_gunpowder_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3626, the first walk: **65** keys. Block 3819 carries the
    // standing families (`form` on 27 units, `SITE[i].reg` on both sides,
    // who=0's `aggression[1]` ours 0 against theirs 1, who=1's groups 64,
    // 65, 67 and 68's `held`/`pool`), and 3904 group 66's. **Seventeen rows
    // are not standing families**: `dest_angle` on `GUARDORDER`s — group
    // 64's guards of the Bombard `1/72` (TypeIndex 267): the Arquebusiers
    // (98) `1/33` and `1/92` (3828), `1/55` (3864), `1/57` (3870), `1/59`
    // (3871), `1/44` (3873), `1/36` (3876), `1/32` (3878), `1/48` (3882),
    // `1/67` (3947), `1/34` and `1/53` (3961), `1/49` (4002), the Elite
    // Pikemen (135) `1/63` (3873), the Crossbowmen (172) `1/96` (3892) and
    // the Supply Wagon (63) `1/71` (3940); and group 65's Crossbowman
    // `1/146` (3864), guarding `1/118` — 3624a's shape (parked 3626a). No
    // other unit, building or city field parts, and the closing block 4075
    // agrees.
    pin_eq!(w.firsts.len(), 65, "run1285b baseline");
}
