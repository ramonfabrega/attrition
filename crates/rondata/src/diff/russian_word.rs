//! **The battery's Russian word** (DECISIONS 72, 83; item 3342): the
//! held-out battery's lowest open game once item 3340 closed the Indians
//! (the Maya being item 3338's), promoted by item 3335's recipe. Great
//! Lakes (`MAP_STYLE 14`), human Nubians (4) against **the Russians (13)**
//! at Difficulty 2, seed 12345 — one of item 1465's three first lobbies
//! (`docs/AI.md` §125), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run647 (a
//! `DUMP_ALL` start, blocks 0..36) and run644 (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the game
//! ends itself on 7303. The word's window is run976's (`docs/AI.md` §275).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const RUSSIAN: Lobby = Lobby {
    name: "GreatLakesRussianTier2",
    map_style: 14,
    ai_tribe: 13,
    difficulty: 2,
    start: "gamelog-run647-greatlakes-russian-diff2-start.txt",
    long: (
        "gamelog-run644-greatlakes-russian-diff2-24k-trace.txt",
        "rontrace-run644.log",
    ),
    length: 7303,
    // Item 1465's first try, held by the battery's measure to `826e0b6c`:
    // count 6566, sequence 6563 (ours 11 draws, theirs 11, at 4: ours
    // `Guy::set_anim+0xf2f < Guy::move+0x166`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`). Item 3342 promoted
    // it to the AI word (`docs/AI.md` §275). Item 3342, `do_move`'s unit
    // arm asking `find_collision` of the chaser's own spot before the
    // kill (`5f7f11`, `docs/COMBAT.md` §65.8): 6563 → 7303, the game's
    // last frame, count and sequence (run976's block 6562, `1/22`).
    count: 7303,
    sequence: 7303,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const RUSSIAN_WORD: i64 = if RUSSIAN.count < RUSSIAN.sequence {
    RUSSIAN.count
} else {
    RUSSIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn russian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(RUSSIAN.start, RUSSIAN.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN {}: count {} sequence {} of {}; {}",
        RUSSIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, RUSSIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, RUSSIAN.length, "the game's last frame");
    pin_eq!(w.count, RUSSIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        RUSSIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn russian_word_is_no_other_lobby() {
    let p = &RUSSIAN;
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

/// run976: the word 6563's window, blocks 6557..6814 at run970's detail
/// (`GUYS=4`; item 3342). run647 dumps blocks 0..36 and run644 its setup
/// and closing blocks only, so no dump on disk held the word's block.
pub(crate) const RUN976: &str =
    "gamelog-run976-greatlakes-russian-diff2-window-6557-6815-guys4.txt";

/// The window: block 6557 through 6814; the word 6563's own block is 6564.
pub(crate) const WIDENING_RUSSIAN_FRAME_6563: (i64, i64) = (6557, 6814);

pub(crate) fn russian_frame_6563_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN.start],
        true,
        RUSSIAN.long,
        "run976",
        &[(RUN976, WIDENING_RUSSIAN_FRAME_6563.0)],
        WIDENING_RUSSIAN_FRAME_6563,
        1,
        &[6564],
        true,
    )
}

#[test]
fn run976_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_frame_6563_window() else {
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
    // Item 3342: 847 keys parted on the first try, 27 on 6557 the
    // standing set (`form` −1 against 9, the leaders' `SITE[·].reg` 1
    // against 0, `aggression[1]`, and the army pool's `pool:65` /
    // `group:65.held` with no `GROUPDATA` printed). The first that is not:
    // block 6562, the Slinger `1/22` (TI 82, who=1's army 0) on its
    // `MOVE_TO` (2760, 31608) over an `ATTACK` on `0/5` — ours (2193,
    // 31584), `orders.len` 2, `order:kind` 10, `last_speed` 0; theirs
    // (2223, 31586), 3, 1, 30. Ours killed the move in reach (`attack_dist`
    // 986) with a mate in its block; `do_move`'s unit arm asks
    // `find_collision` first (`5f7f11`, `docs/COMBAT.md` §65.8). Built:
    // 847 → 28, every `1/22` row agreeing; what stands is 6557's 27 and
    // who=1's `SITE[1].reg` 1 against 0 on 6776, the same family.
    pin_eq!(w.firsts.len(), 28, "keys that part in the window");
}

/// The closing state's window: run644's closing block alone, the long's
/// last frame 7303 writing it (item 3342).
pub(crate) const WIDENING_RUSSIAN_CLOSING: (i64, i64) = (7302, 7305);

/// **The lobby's closing state** (item 3342, DECISIONS 55): run644 — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 7303.
#[test]
fn run644_russian_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(RUSSIAN.long.0),
        dump(RUSSIAN.start),
        trace(RUSSIAN.long.1),
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
        "Russian closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 7304, "the closing block");
    pin_eq!(e.compared, 35, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3342, the first walk (with `do_move`'s unit arm asking
    // `find_collision`, the word at 7303): 35 units compared on block
    // 7304, none apart, no building or city field-row, no closing field.
}
