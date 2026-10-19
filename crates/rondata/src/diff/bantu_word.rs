//! **The battery's Bantu word** (DECISIONS 72, 83; item 3343): the held-out
//! battery's lowest open game once item 3338 closed the Maya, promoted by
//! item 2904's, 3107's, 3314's, 3318's, 3326's, 3331's, 3332's and 3338's
//! recipe. Great Sahara (`MAP_STYLE 7`), human Nubians (4) against **the
//! Bantu (3)** at Difficulty 4, seed 12345 — the lobby item 3338 minted
//! (`docs/AI.md` §125, §273), scored by its first try at 4 of 23,627 and
//! promoted out of the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run972 (a
//! `DUMP_ALL` start) and run972b (the draw stream to the game's end on
//! 23627, `cover=0`, its setup and closing blocks only); the word's window
//! is `docs/AI.md` §276's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BANTU: Lobby = Lobby {
    name: "GreatSaharaBantuTier4",
    map_style: 7,
    ai_tribe: 3,
    difficulty: 4,
    start: "gamelog-run972-greatsahara-bantu-diff4-start.txt",
    long: (
        "gamelog-run972b-greatsahara-bantu-diff4-24k-trace.txt",
        "rontrace-run972b.log",
    ),
    length: 23627,
    // Item 3338, the battery's first try, count and sequence both at 4:
    // ours 8 draws, theirs 6, at 0 ours `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`, theirs `Farms::inc_time+0x1ae`. Item 3343
    // promoted it to the AI word. Item 3343, `update_speed`'s Bantu arm
    // (`BANTU_UNITS_MOVE`, citizens 25 → 31): 4 → 3184, ours 34 draws,
    // theirs 32, at 30 ours `Guy::set_anim+0x97a < Guy::move+0x19f`,
    // theirs `Farms::inc_time+0x1ae`.
    count: 3184,
    sequence: 3184,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const BANTU_WORD: i64 = if BANTU.count < BANTU.sequence {
    BANTU.count
} else {
    BANTU.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn bantu_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(BANTU.start, BANTU.long) else {
        return;
    };
    eprintln!(
        "BANTU {}: count {} sequence {} of {}; {}",
        BANTU.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, BANTU.difficulty, "the lobby read back");
    pin_eq!(w.last, BANTU.length, "the game's last frame");
    pin_eq!(w.count, BANTU.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        BANTU.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn bantu_word_is_no_other_lobby() {
    let p = &BANTU;
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

/// run978: the word 4's window, blocks 1..256 at `GUYS=4` (item 3343);
/// run972 dumps blocks 1, 36 and 37 and run972b its setup and closing
/// blocks only.
pub(crate) const RUN978: &str = "gamelog-run978-greatsahara-bantu-diff4-window-1-256-guys4.txt";

/// The window: block 1 through 256; the word 4's own block is 5.
pub(crate) const WIDENING_BANTU_FRAME_4: (i64, i64) = (1, 256);

pub(crate) fn bantu_frame_4_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978",
        &[(RUN978, WIDENING_BANTU_FRAME_4.0)],
        WIDENING_BANTU_FRAME_4,
        1,
        &[5],
        true,
    )
}

#[test]
fn run978_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_frame_4_window() else {
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
    // Item 3343. Without `update_speed`'s Bantu arm 419 keys part, the
    // first value rows on block 1: who=1's `myspeed`, ours 25 against
    // theirs 31 on the five citizens `1/1`..`1/5` and 34 against 42 on
    // the Scout `1/0` (its `WHERE` the Barracks) — `BANTU_UNITS_MOVE`; on
    // block 2 theirs' `1/1` holds a sidestep waypoint (38232, 15864) and
    // `coll` (38257, 15834), its first step of 31 blocked, and from there
    // the two citizens meet a frame later than ours (the word's
    // `move_step` stop on 4 against theirs' on 5). With the arm, 37; with
    // `calc_pop_cap`'s Bantu flag written (who=1's `pop_cap` 25 against
    // 50 on block 1), **36**: the standing set alone (both leaders'
    // `SITE[n].reg` and `aggression`, every unit's `form`, who=1's groups'
    // `held` and pools), and the word moved 4 → 3184.
    pin_eq!(w.firsts.len(), 36, "run978 with the Bantu arm");
}
