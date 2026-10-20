//! **The battery's Korean word** (DECISIONS 72, 83, 84; item 3365): the
//! held-out battery's next open game once item 3361 closed the Aztecs and
//! the Inca went to item 3360, promoted by item 3335's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Koreans (16)** at
//! Difficulty 3, seed 12345 — minted by item 3331 (DECISIONS 72, a nation
//! no dump held), scored by its first try and promoted out of the battery
//! here, so it is a scored lobby from this item (`battery::SCORED_LOBBIES`)
//! and no longer held out. Captures: run964 (a `DUMP_ALL` start) and
//! run964b (the draw stream to the game's end, `cover=0`, its setup and
//! closing blocks only); the game ends itself on 15944 (`docs/AI.md`
//! §283).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const KOREAN: Lobby = Lobby {
    name: "GreatSaharaKoreanTier3",
    map_style: 7,
    ai_tribe: 16,
    difficulty: 3,
    start: "gamelog-run964-greatsahara-korean-diff3-start.txt",
    long: (
        "gamelog-run964b-greatsahara-korean-diff3-24k-trace.txt",
        "rontrace-run964b.log",
    ),
    length: 15944,
    // Item 3331's lobby, held by the battery's measure to `b91a71a1`:
    // count 9185, sequence 9185 (ours 14 draws, theirs 13, at 1: ours
    // `Leader::use_market+0x1ed`, theirs `Leader::make_stuff+0x221`).
    // Item 3365 promoted it to the AI word (`docs/AI.md` §283), and moved
    // it 9185 -> 15291 (+6106): `get_cost`'s Korean tower step, a third
    // off a Tower-line building (run991's block 9179: who=1's wealth 86
    // and metal 175 against 96 and 192). 15291: ours 8 draws, theirs 10,
    // at 5: ours `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`.
    count: 15291,
    sequence: 15291,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const KOREAN_WORD: i64 = if KOREAN.count < KOREAN.sequence {
    KOREAN.count
} else {
    KOREAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn korean_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(KOREAN.start, KOREAN.long) else {
        return;
    };
    eprintln!(
        "KOREAN {}: count {} sequence {} of {}; {}",
        KOREAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, KOREAN.difficulty, "the lobby read back");
    pin_eq!(w.last, KOREAN.length, "the game's last frame");
    pin_eq!(w.count, KOREAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        KOREAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn korean_word_is_no_other_lobby() {
    let p = &KOREAN;
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

/// run991: the word 9185's window, blocks 9179..9435 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3365). run964 dumps blocks 0..36 and
/// run964b its setup and closing blocks only, so no dump on disk held the
/// leaders' goods or the market on the word's block.
pub(crate) const RUN991: &str = "gamelog-run991-greatsahara-korean-diff3-window-9179-9436-goods.txt";

/// The window: block 9179 through 9435; the word 9185's own block is 9186.
pub(crate) const WIDENING_KOREAN_FRAME_9185: (i64, i64) = (9179, 9435);

pub(crate) fn korean_frame_9185_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[KOREAN.start],
        true,
        KOREAN.long,
        "run991",
        &[(RUN991, WIDENING_KOREAN_FRAME_9185.0)],
        WIDENING_KOREAN_FRAME_9185,
        1,
        &[9186],
        true,
    )
}

#[test]
fn run991_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_frame_9185_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3365: 563 keys on the first try, the window opening apart on
    // who=1's `bucket[2:wealth]` 86 against 96 and `bucket[4:metal]` 175
    // against 192, income agreeing on every good: ours had paid its towers
    // whole. Built (`get_cost`'s Korean tower step): 563 -> 27, the
    // standing family alone — `form` −1 against 9, the leaders'
    // `SITE[·].reg` 1 against 0, `aggression[1]`, and the army pools
    // `pool:64`/`66` and `group:64`/`66.held` with no `GROUPDATA` printed.
    pin_eq!(w.firsts.len(), 27, "keys that part in the window");
}
