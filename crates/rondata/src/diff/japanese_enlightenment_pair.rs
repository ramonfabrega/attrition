//! **The Japanese at a held Enlightenment age, East Indies, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97, 98, item 3696): East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Japanese (15)** at
//! **Hard (4)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Enlightenment: `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its
//! triple, `(18, 15, 4)`, is new: the Japanese lobbies are the Ancient and
//! Industrial pairs' `(18, 15, 5)` — this map at another difficulty — the
//! Medieval pair's `(14, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and
//! `japanese_word`'s `GreatSaharaJapaneseTier1`, `(7, 15, 1)`, which left
//! the battery for the AI word; no battery game fields the Japanese.
//! Captures: run1365 (a `DUMP_ALL` start) and run1366 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §356.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesJapaneseEnlightenment",
    map_style: 18,
    ai_tribe: 15,
    difficulty: 4,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1365-eastindies-japanese-enlightenment-start.txt",
    long: (
        "gamelog-run1366-eastindies-japanese-enlightenment-trace.txt",
        "rontrace-run1366.log",
    ),
    // The game ended itself on 4701 (closing block 4702).
    length: 4701,
    // Item 3696, first try: whole, 4701 of 4701 by the count and the
    // sequence (`docs/AI.md` §356).
    count: 4701,
    sequence: 4701,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_ENLIGHTENMENT_PAIR_WORD: i64 =
    if JAPANESE_ENLIGHTENMENT.count < JAPANESE_ENLIGHTENMENT.sequence {
        JAPANESE_ENLIGHTENMENT.count
    } else {
        JAPANESE_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn japanese_enlightenment_pair_captures_say_their_lobby() {
    let p = &JAPANESE_ENLIGHTENMENT;
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
        "the Japanese Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3696; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Japanese — and no other Japanese lobby's: the Ancient and Industrial
/// pairs' `(18, 15, 5)`, this map at another difficulty, the Medieval
/// pair's `(14, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and the AI word
/// `GreatSaharaJapaneseTier1`'s `(7, 15, 1)`, other maps (item 3696).
#[test]
fn japanese_enlightenment_pair_is_no_other_lobby() {
    let p = &JAPANESE_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the Japanese"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Japanese lobby on this map at this difficulty"
        );
    }
    let w = &super::japanese_word::JAPANESE;
    let a = &super::japanese_ancient_pair::JAPANESE_ANCIENT;
    let m = &super::japanese_medieval_pair::JAPANESE_MEDIEVAL;
    let g = &super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER;
    let i = &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Japanese");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The pair's closing state** (item 3696): run1366's closing block 4702,
/// the whole map's units, walked from run1365's start.
#[test]
fn japanese_enlightenment_pair_run1366_s_closing_state() {
    let _pins = Pins::hold();
    let p = &JAPANESE_ENLIGHTENMENT;
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
        "Japanese Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4702, "the closing block");
    // Item 3696, the pair whole on its first try: 214 units compared, none
    // off, torn, unlinked or extra, `Closing fields: none`.
    pin_eq!(e.compared, 214, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1366b: the pair's last blocks, 4446..4702 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — the word 4701 is the game's last frame and
/// 4702 its closing block (item 3696). run1366 prints units only every
/// hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, so no dump held the last blocks.
pub(crate) const RUN1366B: &str =
    "gamelog-run1366b-eastindies-japanese-enlightenment-window-4446-4702.txt";

/// The window: block 4446 through the closing block 4702.
pub(crate) const WIDENING_JAPANESE_ENLIGHTENMENT_CLOSING: (i64, i64) = (4446, 4702);

/// **The pair's last blocks**: run1366b walked from run1365's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3696).
pub(crate) fn japanese_enlightenment_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[JAPANESE_ENLIGHTENMENT.start],
        true,
        JAPANESE_ENLIGHTENMENT.long,
        "run1366b",
        &[(RUN1366B, WIDENING_JAPANESE_ENLIGHTENMENT_CLOSING.0)],
        WIDENING_JAPANESE_ENLIGHTENMENT_CLOSING,
        1,
        &[4702],
        true,
    )
}

#[test]
fn japanese_enlightenment_pair_run1366b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_enlightenment_closing_window() else {
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
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3696, the first walk: **148** keys. Block 4446 carries the
    // standing families (`form`, who=1's groups and pools, both leaders'
    // `SITE[i].reg`, and who=0's `aggression[1]` ours 0 against theirs 1);
    // after it, who=1's groups 93 and 97 and six more `form` rows. **Two
    // families are not standing**: who=0's city `0/2000` (the Gun
    // Samurai's target) `damage` on 4557, ours 715 against theirs 722
    // (`damage_frac` 3 against 13), theirs' round from `1/53` landing a
    // frame sooner; and the Musketeers (99) `1/129` and `1/111`'s
    // `dest_angle` on 4619 and 4643 (3695b's shape). The draw stream and
    // the closing block 4702 agree.
    pin_eq!(w.firsts.len(), 148, "run1366b baseline");
    // Every round, both directions: 102 round-blocks ours, 115 theirs.
    // **72 rows part, every one a launch** of group 66's three Gun Samurai
    // (161, `ELITESAMURAI`) `1/53`, `1/54` and `1/55` at the city: ours
    // fires from the unit's own position — `1/53`'s `ammo[0]` on 4555 has
    // `sx, sy, sz` (6840, 8616, 281) against theirs' (6842, 8472, 439),
    // `v1z` -1025000 against -66268745, `total_time` 4 against 3, and
    // `ex, ey` within ten — and theirs from an offset launch point (3695a's
    // shape).
    pin_eq!(w.rounds_read, (102, 115), "the rounds read");
    pin_eq!(w.rounds.len(), 72, "the rounds that part");
}
