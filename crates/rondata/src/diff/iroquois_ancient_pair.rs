//! **The Iroquois from an Ancient start, Great Sahara** (DECISIONS 90,
//! 91, 93, item 3584): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Iroquois (18)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, `STARTING_TECHNOLOGY 0` — the Bantu's
//! Ancient-start lobby on the land map with the nation moved. The
//! profile's `ENDING_TECHNOLOGY=0` reads back as `ENDING_TECHNOLOGY 1`
//! (3559a), so this is an Ancient start capped at Classical. The
//! Iroquois' other lobbies are `iroquois_sahara_word`'s
//! `GreatSaharaIroquoisTier3`, `(7, 18, 3)`, and `iroquois_word`'s
//! `GreatLakesIroquoisTier3`, `(14, 18, 3)` — another difficulty.
//! Captures: run1227 (a `DUMP_ALL` start) and run1228 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §317.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const IROQUOIS_ANCIENT: Pair = Pair {
    name: "GreatSaharaIroquoisAncient",
    map_style: 7,
    ai_tribe: 18,
    difficulty: 5,
    starting_technology: 0,
    // Asked 0; the lobby's floor is 1 (`SetupWin::setup_game`, 3559a).
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1227-greatsahara-iroquois-ancient-start.txt",
    long: (
        "gamelog-run1228-greatsahara-iroquois-ancient-trace.txt",
        "rontrace-run1228.log",
    ),
    // The game ended itself on 8420 (closing block 8421).
    length: 8420,
    // Item 3584, first try: whole — frame 8420 of 8420 by the count and
    // the sequence (`docs/AI.md` section 317).
    count: 8420,
    sequence: 8420,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const IROQUOIS_ANCIENT_PAIR_WORD: i64 =
    if IROQUOIS_ANCIENT.count < IROQUOIS_ANCIENT.sequence {
        IROQUOIS_ANCIENT.count
    } else {
        IROQUOIS_ANCIENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn iroquois_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &IROQUOIS_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "IROQUOIS_ANCIENT {}: count {} sequence {} of {}; {}",
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
fn iroquois_ancient_pair_captures_say_their_lobby() {
    let p = &IROQUOIS_ANCIENT;
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
        "the Iroquois Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery's Iroquois are the held-out
/// `GreatSaharaIroquoisTier3`, `(7, 18, 3)`, and the scored
/// `GreatLakesIroquoisTier3`, `(14, 18, 3)` — another difficulty (item
/// 3584).
#[test]
fn iroquois_ancient_pair_is_no_other_lobby() {
    let p = &IROQUOIS_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for other in [
        &super::iroquois_sahara_word::IROQUOIS_SAHARA,
        &super::iroquois_word::IROQUOIS,
    ] {
        assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
    }
}

/// **The pair's closing state**: run1228's closing dump, block 8421,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3584).
#[test]
fn iroquois_ancient_pair_run1228_s_closing_state() {
    let _pins = Pins::hold();
    let p = &IROQUOIS_ANCIENT;
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
        "Iroquois Ancient pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 8421, "the closing block");
    // Item 3584, the pair whole on its first try: 259 units compared, none
    // off, torn or unlinked, and no building or city field-row.
    pin_eq!(e.compared, 259, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1228b: the pair's last blocks, 8165..8421 at the long's detail —
/// the word 8420 is the game's last frame and 8421 its closing block (item
/// 3584). run1228 prints units only every hundredth block and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start, so no dump held
/// the last blocks.
pub(crate) const RUN1228B: &str =
    "gamelog-run1228b-greatsahara-iroquois-ancient-window-8165-8421.txt";

/// The window: block 8165 through the closing block 8421.
pub(crate) const WINDOW_IROQUOIS_ANCIENT_CLOSING: (i64, i64) = (8165, 8421);

/// **The pair's last blocks**: run1228b walked from run1227's start with
/// the recorder on. The word is the game's last frame (item 3584).
pub(crate) fn iroquois_ancient_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS_ANCIENT.start],
        true,
        IROQUOIS_ANCIENT.long,
        "run1228b",
        &[(RUN1228B, WINDOW_IROQUOIS_ANCIENT_CLOSING.0)],
        WINDOW_IROQUOIS_ANCIENT_CLOSING,
        1,
        &[8421],
        true,
    )
}

#[test]
fn iroquois_ancient_pair_run1228b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_ancient_closing_window() else {
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
    // Item 3584, the first walk: **67** keys. Block 8165 carries the
    // standing families (50 rows: `form`, `SITE[i].reg`, `aggression[1]`,
    // who=1's groups' `held`/`pool`) and `1/88`'s `blown`; groups 68's on
    // 8181, `1/16`'s `blown` on 8177, and six `dest_angle` rows (3567a's
    // shape, parked 3580b) from 8225. **Two rows are wounds**, standing
    // through the game's last frame 8420: the Mohawk Spearmen `1/52`
    // (TypeIndex 167, land, uid 81), `hits:damage` ours 42 against theirs
    // 41 on 8191 and 42 against 38 on 8420, `damage_frac` ours 9 against
    // theirs 0; and the Javelineers `1/30` (TypeIndex 83, land, uid 55),
    // 5 against 4 from 8411, `damage_frac` 10 against 0. Theirs heals
    // both and ours does not; the closing compare reads no hits.
    pin_eq!(w.firsts.len(), 67, "run1228b baseline");
}
