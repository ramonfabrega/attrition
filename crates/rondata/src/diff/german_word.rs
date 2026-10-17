//! **The battery's word** (DECISIONS 72, item 2904): every pair closed at
//! its game's end (the peace pair on item 2864), and the AI word's
//! successor is the held-out battery's lowest lobby. East Indies
//! (`MAP_STYLE 18`), human Nubians (4) against **the Germans (12)** at
//! Difficulty 3, seed 12345 — `docs/AI.md` §125's first lobby, scored by
//! its first try at 576 of 13,519 and promoted out of the battery here, so
//! it is a scored lobby from this item (`battery::SCORED_LOBBIES`) and no
//! longer held out. Captures: run646 (a `DUMP_ALL` start) and run643 (the
//! draw stream to the game's end, `cover=0`, its setup and closing blocks
//! only); the word's window is run886's (`docs/AI.md` §239).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GERMAN: Lobby = Lobby {
    name: "EastIndiesGermanTier3",
    map_style: 18,
    ai_tribe: 12,
    difficulty: 3,
    start: "gamelog-run646-eastindies-german-diff3-start.txt",
    long: (
        "gamelog-run643-eastindies-german-diff3-24k-trace.txt",
        "rontrace-run643.log",
    ),
    length: 13519,
    // Item 1465: the battery's first try, count and sequence both at 576
    // (ours 52 draws, theirs 36, index 4: ours `Leader::compute_sites+0x4ac`,
    // theirs `Animal::think_bird+0x82`). Item 2904 promoted it to the AI
    // word unmoved (`docs/AI.md` §239).
    count: 576,
    sequence: 576,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const GERMAN_WORD: i64 = if GERMAN.count < GERMAN.sequence {
    GERMAN.count
} else {
    GERMAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn german_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(GERMAN.start, GERMAN.long) else {
        return;
    };
    eprintln!(
        "GERMAN {}: count {} sequence {} of {}; {}",
        GERMAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, GERMAN.difficulty, "the lobby read back");
    pin_eq!(w.last, GERMAN.length, "the game's last frame");
    pin_eq!(w.count, GERMAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        GERMAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn german_word_is_no_other_lobby() {
    let p = &GERMAN;
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

/// run886: the word 576's window, blocks 570..826 at the long's detail
/// (item 2904); run643 dumps its setup and closing blocks only.
pub(crate) const RUN886: &str = "gamelog-run886-eastindies-german-diff3-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_GERMAN_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn german_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN.start],
        true,
        GERMAN.long,
        "run886",
        &[(RUN886, WIDENING_GERMAN_FRAME_576.0)],
        WIDENING_GERMAN_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run886_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_frame_576_window() else {
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
    pin_eq!(w.firsts.len(), 262, "initial run886 baseline");
    // Item 2904: 262 keys part on blocks 570..826, the word's own block
    // being 577 (frame 576: ours 52 draws, theirs 36, index 4 — theirs one
    // round of `Leader::compute_sites+0x4ac`/`+0x50a` and
    // `Leader::make_stuff+0x221` ×2, ours five, then the same birds,
    // objects, herd and farms). On 577 theirs holds who=1's Village `1/2007`
    // (type 414, a foundation at (34656, 36192), `constr_time` 60000) and
    // ours none (`build:unlinked`); who=1's Citizen `1/2` holds theirs'
    // `BUILDORDER` on it (`orders.len` 1/2, `orders_x` 40632/35400,
    // `path` 0/5); who=1's `SITE[1].wx` 51/0 (theirs' site taken) and the
    // table below it shifted by one; `gatherers` 8/7. 35 stand on 570:
    // who=1's `income` and `resources` (food 800/880, timber 640/720),
    // `leftover` (food 5280/480, timber 800/3200), `bucket` (food 73/80,
    // timber 58/64), both leaders' `aggression` 0/1, and the control's
    // `SITE[n].reg`, `form` and groups' `held` (`docs/AI.md` §239).
}
