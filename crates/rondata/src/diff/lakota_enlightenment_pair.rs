//! **The Lakota at a held Enlightenment age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, item 3695): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Lakota (19)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its triple,
//! `(14, 19, 5)`, is new: the Lakota lobbies are the battery's
//! `GreatSaharaLakotaTier2`, `(7, 19, 2)`, `lakota_word`'s
//! `GreatLakesLakotaTier2`, `(14, 19, 2)`, the peace pair's `(18, 19, 3)`,
//! `lakota_medieval_pair`'s `(18, 19, 5)` and `lakota_gunpowder_pair`'s
//! `(7, 19, 5)`, another map or another difficulty. Captures: run1363 (a
//! `DUMP_ALL` start) and run1364 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §355.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const LAKOTA_ENLIGHTENMENT: Pair = Pair {
    name: "GreatLakesLakotaEnlightenment",
    map_style: 14,
    ai_tribe: 19,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1363-greatlakes-lakota-enlightenment-start.txt",
    long: (
        "gamelog-run1364-greatlakes-lakota-enlightenment-trace.txt",
        "rontrace-run1364.log",
    ),
    // The game ended itself on 4673 (closing block 4674).
    length: 4673,
    // Item 3695, first try: whole, 4673 of 4673 by the count and the
    // sequence (`docs/AI.md` §355).
    count: 4673,
    sequence: 4673,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const LAKOTA_ENLIGHTENMENT_PAIR_WORD: i64 =
    if LAKOTA_ENLIGHTENMENT.count < LAKOTA_ENLIGHTENMENT.sequence {
        LAKOTA_ENLIGHTENMENT.count
    } else {
        LAKOTA_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &LAKOTA_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "LAKOTA_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn lakota_enlightenment_pair_captures_say_their_lobby() {
    let p = &LAKOTA_ENLIGHTENMENT;
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
        "the Lakota Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3695; the battery's identity
/// test refuses it), once, and none of the battery's: its one Lakota game,
/// `GreatSaharaLakotaTier2`'s `(7, 19, 2)`, is another map; and no other
/// Lakota lobby's: the AI word `GreatLakesLakotaTier2`'s `(14, 19, 2)`,
/// another difficulty, the peace pair's `(18, 19, 3)`, the Medieval pair's
/// `(18, 19, 5)` and the Gunpowder pair's `(7, 19, 5)`, other maps (item
/// 3695).
#[test]
fn lakota_enlightenment_pair_is_no_other_lobby() {
    let p = &LAKOTA_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    for l in battery::BATTERY.iter().filter(|l| l.ai_tribe == p.ai_tribe) {
        assert_ne!(
            (l.map_style, l.ai_tribe, l.difficulty),
            triple,
            "{} is this lobby",
            l.name
        );
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Lakota lobby on this map at this difficulty"
        );
    }
    let w = &super::lakota_word::LAKOTA;
    let peace = &super::lakota_pair::LAKOTA;
    let m = &super::lakota_medieval_pair::LAKOTA_MEDIEVAL;
    let g = &super::lakota_gunpowder_pair::LAKOTA_GUNPOWDER;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (
            peace.name,
            (peace.map_style, peace.ai_tribe, peace.difficulty),
        ),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Lakota");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The pair's closing state** (item 3695): run1364's closing block 4674,
/// the whole map's units, walked from run1363's start.
#[test]
fn lakota_enlightenment_pair_run1364_s_closing_state() {
    let _pins = Pins::hold();
    let p = &LAKOTA_ENLIGHTENMENT;
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
        "Lakota Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4674, "the closing block");
    // Item 3695, the pair whole on its first try: 188 units compared, none
    // off, torn, unlinked or extra, `Closing fields: none`.
    pin_eq!(e.compared, 188, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1364b: the pair's last blocks, 4418..4674 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — the word 4673 is the game's last frame and
/// 4674 its closing block (item 3695). run1364 prints units only every
/// hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, so no dump held the last blocks.
pub(crate) const RUN1364B: &str =
    "gamelog-run1364b-greatlakes-lakota-enlightenment-window-4418-4674.txt";

/// The window: block 4418 through the closing block 4674.
pub(crate) const WIDENING_LAKOTA_ENLIGHTENMENT_CLOSING: (i64, i64) = (4418, 4674);

/// **The pair's last blocks**: run1364b walked from run1363's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3695).
pub(crate) fn lakota_enlightenment_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[LAKOTA_ENLIGHTENMENT.start],
        true,
        LAKOTA_ENLIGHTENMENT.long,
        "run1364b",
        &[(RUN1364B, WIDENING_LAKOTA_ENLIGHTENMENT_CLOSING.0)],
        WIDENING_LAKOTA_ENLIGHTENMENT_CLOSING,
        1,
        &[4674],
        true,
    )
}

#[test]
fn lakota_enlightenment_pair_run1364b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_enlightenment_closing_window() else {
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
    // Item 3695, the first walk: **57** keys. Block 4418 carries the
    // standing families (`form`, groups and pools, both leaders'
    // `SITE[i].reg`, who=0's `aggression[1]` ours 0 against theirs 1, and
    // leader 1's food: `bucket[0]` 10305 against 10333, `leftover[0]` 6144
    // against 5352). **Three families are not standing**: who=1's rounds
    // below; who=0's city `0/2003` (uid 3, the riders' target) `damage` on
    // 4461, ours 203 against theirs 210, theirs' round from `1/71` landing a
    // frame sooner; and the Musketeers (99) `1/39` and `1/38`'s
    // `dest_angle` on 4656 and 4661 (parked 3695a, 3695b). The draw stream
    // and the closing block 4674 agree.
    pin_eq!(w.firsts.len(), 57, "run1364b baseline");
    // Every round, both directions: 120 round-blocks ours, 123 theirs.
    // **45 rows part, every one a launch**: the Repeating Rifle Horse
    // (208, `RIFLEHORSE`) `1/71` and `1/76` and the Cannon (268) `1/75` fire
    // from the unit's own position in ours — `1/71`'s `ammo[1]` on 4424 has
    // `sx, sy, sz` (5976, 32664, 276) against theirs' (5938, 32613, 538),
    // `v1z` 95724998 against 30225000, and `ex, ey` within three — and from
    // an offset launch point in theirs; `1/71`'s `ammo[0]` on 4459 flies
    // `total_time` 4 against 3 (parked 3695a).
    pin_eq!(w.rounds_read, (120, 123), "the rounds read");
    pin_eq!(w.rounds.len(), 45, "the rounds that part");
}
