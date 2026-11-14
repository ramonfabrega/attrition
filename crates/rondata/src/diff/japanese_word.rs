//! **The battery's Japanese word** (DECISIONS 72, 83; item 3331): the
//! held-out battery's lowest game once item 3326 closed the Greeks, promoted
//! by item 2904's, 3107's, 3314's, 3318's and 3326's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Japanese (15)** at
//! Difficulty 1, seed 12345 — the lobby item 3326 minted (`docs/AI.md`
//! §266), scored by its first try at 576 of 24,000 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run960 (a
//! `DUMP_ALL` start, blocks 0..36) and run960b (the draw stream to 24000,
//! `cover=0`, its setup and closing blocks only); the word's window is
//! run965's (`docs/AI.md` §268).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const JAPANESE: Lobby = Lobby {
    name: "GreatSaharaJapaneseTier1",
    map_style: 7,
    ai_tribe: 15,
    difficulty: 1,
    start: "gamelog-run960-greatsahara-japanese-diff1-start.txt",
    long: (
        "gamelog-run960b-greatsahara-japanese-diff1-24k-trace.txt",
        "rontrace-run960b.log",
    ),
    length: 24000,
    // Item 3326: the battery's first try, count and sequence both at 576;
    // ours 56 draws, theirs 35, at 4 ours `Leader::compute_sites+0x4ac`,
    // theirs `Animal::think_bird+0x82`. Item 3331 promoted it to the AI word.
    // Ours' who=1 runs `compute_sites` five times on 576 against theirs'
    // once; run965 (570..826) has who=1 apart already on block 570, timber
    // 52 against 78 and food income 800 against 960. The Japanese Farm cost
    // arm (`get_cost:1007`, `JAPANESE_FARMS_CHEAPER`, half off a Farm) closed
    // the timber; the Japanese farm food arm (`calc_gather:983`,
    // `JAPANESE_FISHING_BOATS`, a quarter more per farmer) the food, and the
    // word moved 576 → 24000, the long's last frame. run960b's closing
    // block 24001 is compared whole (`run960b_japanese_closing_state`).
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const JAPANESE_WORD: i64 = if JAPANESE.count < JAPANESE.sequence {
    JAPANESE.count
} else {
    JAPANESE.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn japanese_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(JAPANESE.start, JAPANESE.long) else {
        return;
    };
    eprintln!(
        "JAPANESE {}: count {} sequence {} of {}; {}",
        JAPANESE.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, JAPANESE.difficulty, "the lobby read back");
    pin_eq!(w.last, JAPANESE.length, "the game's last frame");
    pin_eq!(w.count, JAPANESE.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        JAPANESE.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn japanese_word_is_no_other_lobby() {
    let p = &JAPANESE;
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

/// run965: the word 576's window, blocks 570..826 at run961's detail
/// (`LEADERS=9`, `GUYS=4`; item 3331); run960 dumps blocks 0..36 and run960b
/// its setup and closing blocks only.
pub(crate) const RUN965: &str =
    "gamelog-run965-greatsahara-japanese-diff1-window-570-827-guys4.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_JAPANESE_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn japanese_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE.start],
        true,
        JAPANESE.long,
        "run965",
        &[(RUN965, WIDENING_JAPANESE_FRAME_576.0)],
        WIDENING_JAPANESE_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run965_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_frame_576_window() else {
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
    // Item 3331. On the first try block 570 parts, past the standing set,
    // who=1's `bucket[1:timber]` 52 against 78, `bucket[0:food]` 54 against
    // 64, and `income[0:food]` 800 against 960 (with `resources` and
    // `leftover`): a Japanese Farm at half price (`get_cost:1007`) and a
    // Japanese farmer's food at five quarters (`calc_gather:983`). With the
    // Farm arm the timber closes and the window parts the food; with the
    // food arm it parts **the standing set alone**, and the word moved 576
    // → 24000.
    pin_eq!(w.firsts.len(), 34, "run965 with the Japanese farm arms");
}

/// The closing state's window: run960b's closing block alone, the long's
/// last frame 24000 writing it (item 3331).
pub(crate) const WIDENING_JAPANESE_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3331, DECISIONS 55): run960b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run960b_japanese_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(JAPANESE.long.0),
        dump(JAPANESE.start),
        trace(JAPANESE.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
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
        "Japanese closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 24001, "the closing block");
    pin_eq!(e.compared, 61, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3331, the first walk (with the Japanese farm arms, the word at
    // 24000): 61 units compared on block 24001, none apart (the human's
    // `0/4` is the dump's own tear), no building or city field-row, no
    // closing field.
}
