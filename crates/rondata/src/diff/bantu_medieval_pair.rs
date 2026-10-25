//! **The Bantu at a held Medieval age, Great Sahara** (DECISIONS 90, 91,
//! 93, 94, item 3624): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Bantu (3)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval: `STARTING_TECHNOLOGY
//! 2`, `ENDING_TECHNOLOGY 2` — `russian_medieval_pair`'s lobby with the
//! nation moved. Its triple, `(7, 3, 5)`, is new: the battery fields no
//! Bantu, `bantu_word`'s `GreatSaharaBantuTier4` is `(7, 3, 4)`, another
//! difficulty, and `bantu_ancient_pair`'s `(18, 3, 5)` is another map.
//! Captures: run1280 (a `DUMP_ALL` start) and run1281 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §332.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BANTU_MEDIEVAL: Pair = Pair {
    name: "GreatSaharaBantuMedieval",
    map_style: 7,
    ai_tribe: 3,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1280-greatsahara-bantu-medieval-start.txt",
    long: (
        "gamelog-run1281-greatsahara-bantu-medieval-trace.txt",
        "rontrace-run1281.log",
    ),
    // The game ended itself on 4914 (closing block 4915).
    length: 4914,
    // Item 3624, first try: **whole** — the draw count and the sequence
    // agree through the game's last frame, 4914 of 4914, and the closing
    // state compares (`bantu_medieval_pair_run1281_s_closing_state`).
    count: 4914,
    sequence: 4914,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BANTU_MEDIEVAL_PAIR_WORD: i64 = if BANTU_MEDIEVAL.count < BANTU_MEDIEVAL.sequence {
    BANTU_MEDIEVAL.count
} else {
    BANTU_MEDIEVAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn bantu_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BANTU_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BANTU_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2, and the long is scoreable.
#[test]
fn bantu_medieval_pair_captures_say_their_lobby() {
    let p = &BANTU_MEDIEVAL;
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
        "the Bantu Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3624; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no Bantu
/// since `GreatSaharaBantuTier4`, `(7, 3, 4)`, left it for the AI word
/// (another difficulty), and the Ancient-start pair's `(18, 3, 5)` is
/// another map (item 3624).
#[test]
fn bantu_medieval_pair_is_no_other_lobby() {
    let p = &BANTU_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Bantu"
    );
    for (name, other) in [
        (
            super::bantu_word::BANTU.name,
            (
                super::bantu_word::BANTU.map_style,
                super::bantu_word::BANTU.ai_tribe,
                super::bantu_word::BANTU.difficulty,
            ),
        ),
        (
            super::bantu_ancient_pair::BANTU_ANCIENT.name,
            (
                super::bantu_ancient_pair::BANTU_ANCIENT.map_style,
                super::bantu_ancient_pair::BANTU_ANCIENT.ai_tribe,
                super::bantu_ancient_pair::BANTU_ANCIENT.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The pair's closing state**: run1281's closing dump, block 4915,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3624).
#[test]
fn bantu_medieval_pair_run1281_s_closing_state() {
    let _pins = Pins::hold();
    let p = &BANTU_MEDIEVAL;
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
        "Bantu Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4915, "the closing block");
    // Item 3624, the pair whole on its first try: 255 units compared, none
    // off, torn or unlinked, and no building or city field-row.
    pin_eq!(e.compared, 255, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1281b: the pair's last blocks, 4659..4915 at the long's detail with
/// `AMMO=5` — the word 4914 is the game's last frame and 4915 its closing
/// block (item 3624). run1281 prints units only every hundredth block and
/// no `LEADERS`, `BUILDS` or `CITIES` record past the start, so no dump
/// held the last blocks.
pub(crate) const RUN1281B: &str =
    "gamelog-run1281b-greatsahara-bantu-medieval-window-4659-4915.txt";

/// The window: block 4659 through the closing block 4915.
pub(crate) const WINDOW_BANTU_MEDIEVAL_CLOSING: (i64, i64) = (4659, 4915);

/// **The pair's last blocks**: run1281b walked from run1280's start with
/// the recorder on. The word is the game's last frame (item 3624).
pub(crate) fn bantu_medieval_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_MEDIEVAL.start],
        true,
        BANTU_MEDIEVAL.long,
        "run1281b",
        &[(RUN1281B, WINDOW_BANTU_MEDIEVAL_CLOSING.0)],
        WINDOW_BANTU_MEDIEVAL_CLOSING,
        1,
        &[4915],
        true,
    )
}

#[test]
fn bantu_medieval_pair_run1281b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_medieval_closing_window() else {
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
    // Item 3624, the first walk: **60** keys. Block 4659 carries the
    // standing families (`form` on 27 units, `SITE[i].reg` on both sides,
    // who=0's `aggression[1]` ours 0 against theirs 1, who=1's groups 64,
    // 65, 66 and 69's `held`/`pool`), and later blocks groups 67 and 68's
    // (4701, 4710). **Eight rows are not standing families**: `dest_angle`
    // on group 64's guards of `1/67` — the Impi (TypeIndex 95) `1/24`
    // (4726), `1/26` and `1/42` (4736), `1/110` (4749), `1/43` (4767),
    // `1/90` (4893), `1/29` (4895), and the Pikemen (134) `1/55` (4765) —
    // 3567a's shape (parked 3624a). No other unit, building or city field
    // parts, and the closing block 4915 agrees.
    pin_eq!(w.firsts.len(), 60, "run1281b baseline");
}
