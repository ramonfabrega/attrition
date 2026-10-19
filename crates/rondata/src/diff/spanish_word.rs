//! **The battery's Great Sahara word** (DECISIONS 72, 83; item 3318): the
//! held-out battery's lowest game once item 3306 closed the Iroquois,
//! promoted by item 2904's, 3107's and 3314's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Spanish (9)** at
//! Difficulty 3, seed 12345 — `docs/AI.md` §243's lobby, scored by its first
//! try at 576 of 22,700 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run896 (a `DUMP_ALL` start, blocks 0..36) and run896b
//! (the draw stream to the game's end, `cover=0`, its setup and closing
//! blocks only); the word's window is run958's (`docs/AI.md` §264).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH: Lobby = Lobby {
    name: "GreatSaharaSpanishTier3",
    map_style: 7,
    ai_tribe: 9,
    difficulty: 3,
    start: "gamelog-run896-greatsahara-spanish-diff3-start.txt",
    long: (
        "gamelog-run896b-greatsahara-spanish-diff3-24k-trace.txt",
        "rontrace-run896b.log",
    ),
    length: 22700,
    // Item 2975: the battery's first try, count and sequence both at 576.
    // Item 3318 promoted it to the AI word and moved it 576 → 7785: who=1
    // holds `leader_flags 0x1000` from `Setup::build_empire` and every
    // cell is seen to its site scoring (`docs/AI.md` §264). On 7785 ours
    // draws 16, theirs 17; index 5 is ours `Animal::do_idle+0x83`, theirs
    // `Leader::make_stuff+0x63d`. Salt's Barracks arm (`get_cost`'s head,
    // a tenth off the scaled base) moved it 7785 → 14382: run958d's block
    // 7783 prices `1/2019`'s two Hoplites 48/31 and 52/35 on both sides
    // (ours was 53/34 and 57/38). On 14382 ours draws 14, theirs 12; index
    // 5 is ours `Leader::make_stuff+0x63d`, theirs `Guy::set_anim+0x97a <
    // Guy::move+0x19f`. Relics' research arm (`train_time`'s research
    // block, a third off while the rare is held) moved it 14382 → 22692:
    // who=1's Chemistry, queued at the Library `1/2005` on 14035, is done by
    // run958e's block 14376 on both sides (`epoch[3]` 3; ours was 2, its job
    // at 34500 of 35000). On 22692, eight frames from the game's end, ours
    // draws 19, theirs 16; index 16 is ours `Farms::inc_time+0x1ae`, theirs
    // none.
    count: 22692,
    sequence: 22692,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_WORD: i64 = if SPANISH.count < SPANISH.sequence {
    SPANISH.count
} else {
    SPANISH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH.start, SPANISH.long) else {
        return;
    };
    eprintln!(
        "SPANISH {}: count {} sequence {} of {}; {}",
        SPANISH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SPANISH.difficulty, "the lobby read back");
    pin_eq!(w.last, SPANISH.length, "the game's last frame");
    pin_eq!(w.count, SPANISH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        SPANISH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_word_is_no_other_lobby() {
    let p = &SPANISH;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
    }
}

/// run958: the word 576's window, blocks 570..826 at run956's detail
/// (`LEADERS=9`, `GUYS=2`; item 3318); run896 dumps blocks 0..36 and
/// run896b its setup and closing blocks only.
pub(crate) const RUN958: &str = "gamelog-run958-greatsahara-spanish-diff3-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_SPANISH_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn spanish_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH.start],
        true,
        SPANISH.long,
        "run958",
        &[(RUN958, WIDENING_SPANISH_FRAME_576.0)],
        WIDENING_SPANISH_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run958_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_frame_576_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // `GUYS=2` prints the short gaia record, no clock (the coverage
    // pair's windows pin the same two keys).
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3318: 323 keys on the first build. The Spanish goody-sight arm
    // put the Scout `1/1` on theirs' track from 524 (308); the Spanish
    // `leader_flags |= 0x1000` (`Setup::build_empire@005abb80`) put
    // who=1's site scores on theirs' — `SITE[9].val` 1832 against 2978 at
    // (50, 12) on 570, every call from frame 0's ninth on run958c parting
    // by `(min(danger, 9) + 7) / 8` — and the window parts **the standing
    // set alone**: both leaders' `SITE[n].reg`, `aggression`, `form`, the
    // groups' `held` and the pools, from 570, and group 65's on 631. The
    // word's block 577 parts nothing; the word moved 576 → 7785, past the
    // window (`docs/AI.md` §264).
    pin_eq!(w.firsts.len(), 36, "run958 with the Spanish 0x1000");
}

/// run958d: the word 7785's window, blocks 7779..8035 at run954's detail
/// (`LEADERS=9`, `GUYS=4`; item 3318) — the `0x1000` arm moved the word
/// past run958's last block, and run896b dumps no block between.
pub(crate) const RUN958D: &str =
    "gamelog-run958d-greatsahara-spanish-diff3-window-7779-8036-guys4.txt";

/// The window: block 7779 through 8035; the word 7785's own block is 7786.
pub(crate) const WIDENING_SPANISH_FRAME_7785: (i64, i64) = (7779, 8035);

pub(crate) fn spanish_frame_7785_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH.start],
        true,
        SPANISH.long,
        "run958d",
        &[(RUN958D, WIDENING_SPANISH_FRAME_7785.0)],
        WIDENING_SPANISH_FRAME_7785,
        1,
        &[7786],
        true,
    )
}

#[test]
fn run958d_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_frame_7785_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3318: 666 keys on the first build. Past the standing set the
    // first parting was block 7783: who=1's `bucket[0]`/`bucket[4]` 37/79
    // against 47/85, and `1/2019`'s (the Barracks) two new Hoplites priced
    // 53/34 and 57/38 against 48/31 and 52/35 — Salt's tenth off the scaled
    // base, which ours did not take. With it the window parts **the
    // standing set alone** (both leaders' `SITE[n].reg`, `aggression`,
    // `form`, the groups' `held` and the pools, from 7779, and group 68's
    // on 7939); the word's block
    // 7786 parts nothing, and the word moved 7785 → 14382, past the window.
    pin_eq!(w.firsts.len(), 34, "run958d with Salt's Barracks arm");
}

/// run958e: the word 14382's window, blocks 14376..14632 at run958d's
/// detail (item 3318) — Salt's Barracks arm moved the word past run958d's
/// last block, and run896b dumps no block between.
pub(crate) const RUN958E: &str =
    "gamelog-run958e-greatsahara-spanish-diff3-window-14376-14633-guys4.txt";

/// The window: block 14376 through 14632; the word 14382's own block is
/// 14383.
pub(crate) const WIDENING_SPANISH_FRAME_14382: (i64, i64) = (14376, 14632);

pub(crate) fn spanish_frame_14382_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH.start],
        true,
        SPANISH.long,
        "run958e",
        &[(RUN958E, WIDENING_SPANISH_FRAME_14382.0)],
        WIDENING_SPANISH_FRAME_14382,
        1,
        &[14383],
        true,
    )
}

#[test]
fn run958e_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_frame_14382_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3318: 833 keys on the first build, from block 14376: who=1's
    // `epoch[3]` 2 against 3 (`epochs` 11 against 12), the Library `1/2005`
    // still `queued` in ours, and seven of who=1's units' `mylos` one or
    // two short — its Chemistry, queued on 14035 on both sides, done in
    // theirs and at 34500 of 35000 in ours, which took nothing off for the
    // Relics it held. With Relics' research arm the window parts **the
    // standing set alone**, from 14376; the word's block 14383 parts
    // nothing, and the word moved 14382 → 22692, past the window.
    pin_eq!(w.firsts.len(), 28, "run958e with Relics' research arm");
}
