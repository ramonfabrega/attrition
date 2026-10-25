//! **The Germans from an Ancient start, Great Sahara** (DECISIONS 90,
//! 91, 93, item 3596): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Germans (12)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, `STARTING_TECHNOLOGY 0`, `ENDING_TECHNOLOGY 1`
//! — the Iroquois Ancient pair's lobby with the nation moved. The
//! Germans' other lobbies are `german_word`'s `EastIndiesGermanTier3`,
//! `(18, 12, 3)`, `german_sahara_word`'s `GreatSaharaGermanTier3`,
//! `(7, 12, 3)` — another difficulty — and `german_alltech_pair`'s
//! `EastIndiesGermanAllTech`, `(18, 12, 5)` — another map.
//! Captures: run1243 (a `DUMP_ALL` start) and run1244 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §321.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GERMAN_ANCIENT: Pair = Pair {
    name: "GreatSaharaGermanAncient",
    map_style: 7,
    ai_tribe: 12,
    difficulty: 5,
    starting_technology: 0,
    // Asked 0; the lobby's floor is 1 (`SetupWin::setup_game`, 3559a).
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1243-greatsahara-german-ancient-start.txt",
    long: (
        "gamelog-run1244-greatsahara-german-ancient-trace.txt",
        "rontrace-run1244.log",
    ),
    // The game ended itself on 5314 (closing block 5315).
    length: 5314,
    // Item 3596, first try: whole — frame 5314 of 5314 by the count and
    // the sequence (`docs/AI.md` section 321).
    count: 5314,
    sequence: 5314,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_ANCIENT_PAIR_WORD: i64 = if GERMAN_ANCIENT.count < GERMAN_ANCIENT.sequence {
    GERMAN_ANCIENT.count
} else {
    GERMAN_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn german_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_ANCIENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, `ENDING_TECHNOLOGY` 1 beside `STARTING_TECHNOLOGY` 0, and the
/// long is scoreable.
#[test]
fn german_ancient_pair_captures_say_their_lobby() {
    let p = &GERMAN_ANCIENT;
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
        "the German Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery's Germans are the scored
/// `EastIndiesGermanTier3`, `(18, 12, 3)`, and `GreatSaharaGermanTier3`,
/// `(7, 12, 3)` — another difficulty — and the late-age
/// `EastIndiesGermanAllTech` is `(18, 12, 5)`, another map (item 3596).
#[test]
fn german_ancient_pair_is_no_other_lobby() {
    let p = &GERMAN_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for other in [
        &super::german_word::GERMAN,
        &super::german_sahara_word::GERMAN_SAHARA,
    ] {
        assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
    }
    let a = &super::german_alltech_pair::GERMAN_ALLTECH;
    assert_ne!((a.map_style, a.ai_tribe, a.difficulty), triple);
}

/// **The pair's closing state**: run1244's closing dump, block 5315,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3596).
#[test]
fn german_ancient_pair_run1244_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GERMAN_ANCIENT;
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
        "German Ancient pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5315, "the closing block");
    // Item 3596, the pair whole on its first try: 171 units compared, none
    // off, torn or unlinked, and no building or city field-row.
    pin_eq!(e.compared, 171, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1244b: the pair's last blocks, 5059..5315 at the long's detail —
/// the word 5314 is the game's last frame and 5315 its closing block (item
/// 3596). run1244 prints units only every hundredth block and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start, so no dump held
/// the last blocks.
pub(crate) const RUN1244B: &str =
    "gamelog-run1244b-greatsahara-german-ancient-window-5059-5315.txt";

/// The window: block 5059 through the closing block 5315.
pub(crate) const WINDOW_GERMAN_ANCIENT_CLOSING: (i64, i64) = (5059, 5315);

/// **The pair's last blocks**: run1244b walked from run1243's start with
/// the recorder on. The word is the game's last frame (item 3596).
pub(crate) fn german_ancient_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_ANCIENT.start],
        true,
        GERMAN_ANCIENT.long,
        "run1244b",
        &[(RUN1244B, WINDOW_GERMAN_ANCIENT_CLOSING.0)],
        WINDOW_GERMAN_ANCIENT_CLOSING,
        1,
        &[5315],
        true,
    )
}

#[test]
fn german_ancient_pair_run1244b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_ancient_closing_window() else {
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
    // Item 3596, the first walk: **55** keys, every one on the window's
    // first block or a group's: block 5059 carries the standing families
    // (`form` on 19 units, who=0's `SITE[i].reg` ours 1 against theirs 0,
    // `aggression[1]`, who=1's groups' `held`/`pool`) and five rows of
    // who=1's sites, `SITE[1].rank` 3 against 4, `SITE[3].rank` 4
    // against 1, `SITE[3].val` 607488 against 404976, `SITE[4].rank` and
    // `SITE[6].rank` 2 against 3, which agree again by block 5313 (parked
    // 3596a); groups 71's `held`/`pool` on 5065 and 73's on 5179. No unit,
    // building or city field parts.
    pin_eq!(w.firsts.len(), 55, "run1244b baseline");
}
