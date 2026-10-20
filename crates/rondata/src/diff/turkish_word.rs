//! **The battery's Turkish word** (DECISIONS 72, 83, 84; item 3380): the
//! held-out battery's second open game on the measure of `56f23ba5` (the
//! Germans being item 3378's), promoted by item 3335's recipe. Great Lakes
//! (`MAP_STYLE 14`), human Nubians (4) against **the Turks (8)** at
//! Difficulty 4, seed 12345 — minted by item 2975, scored by its first try
//! and promoted out of the battery here, so it is a scored lobby from this
//! item (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run898
//! (a `DUMP_ALL` start) and run898b (the draw stream to the game's end,
//! `cover=0`, its setup and closing blocks only); the game ends itself on
//! 15503 (`docs/AI.md` §290).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const TURKISH: Lobby = Lobby {
    name: "GreatLakesTurkishTier4",
    map_style: 14,
    ai_tribe: 8,
    difficulty: 4,
    start: "gamelog-run898-greatlakes-turkish-diff4-start.txt",
    long: (
        "gamelog-run898b-greatlakes-turkish-diff4-24k-trace.txt",
        "rontrace-run898b.log",
    ),
    length: 15503,
    // Item 2975's lobby: 1440 -> 6982 on item 3160 (`get_cost`'s
    // `TURK_CITIZEN` and `WOOL_CITIZENS_COST` arms), 6982 -> 14991 on the
    // forty-fifth tranche's fixes made for other words, re-pinned by the
    // forty-sixth pass and held by the battery's measure to `56f23ba5`:
    // count 14991, sequence 14991 (ours 20 draws, theirs 23, at 12: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`). Item 3380
    // promoted it to the AI word (`docs/AI.md` §290), and moved it 14991 ->
    // 15503 (+512), the game's end: `max_range`'s Turkish siege term
    // (`TURK_SIEGE_RANGE`, `Sim::siege_unit_range`). On 14991 the
    // Trebuchet `1/70` walked its bombard ring round Napata `0/2000` at
    // ours 2976, theirs 3552: ours drew 12 (three candidates on blocked
    // forest), theirs 15, and stopped at (7368, 29544) in run1005's block
    // 14992; agreeing after.
    count: 15503,
    sequence: 15503,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const TURKISH_WORD: i64 = if TURKISH.count < TURKISH.sequence {
    TURKISH.count
} else {
    TURKISH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn turkish_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(TURKISH.start, TURKISH.long) else {
        return;
    };
    eprintln!(
        "TURKISH {}: count {} sequence {} of {}; {}",
        TURKISH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, TURKISH.difficulty, "the lobby read back");
    pin_eq!(w.last, TURKISH.length, "the game's last frame");
    pin_eq!(w.count, TURKISH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        TURKISH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn turkish_word_is_no_other_lobby() {
    let p = &TURKISH;
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

/// run1005: the word 14991's window, blocks 14985..15241 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3380). run898 dumps blocks 0..36 and
/// run898b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN1005: &str = "gamelog-run1005-greatlakes-turkish-diff4-window-14985-15242.txt";

/// The window: block 14985 through 15241; the word 14991's own block is
/// 14992.
pub(crate) const WIDENING_TURKISH_FRAME_14991: (i64, i64) = (14985, 15241);

pub(crate) fn turkish_frame_14991_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[TURKISH.start],
        true,
        TURKISH.long,
        "run1005",
        &[(RUN1005, WIDENING_TURKISH_FRAME_14991.0)],
        WIDENING_TURKISH_FRAME_14991,
        1,
        &[14992],
        true,
    )
}

#[test]
fn run1005_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = turkish_frame_14991_window() else {
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
    // Item 3380, the first walk (the Turkish siege range built, the word at
    // the game's end): 57 keys. The standing family on 14985 (`form`, the
    // leaders' `SITE[·].reg`, `aggression[1]`, the army pools and
    // `group:N.held` with no `GROUPDATA` printed, `g.angle[0]` of nine of
    // who=1's), and the four Turkish siege units' `mylos`, ours 4 against
    // theirs 7 (`1/37`, `1/38`, `1/70`, `1/71`). Built (`TURK_SIEGE_LOS`,
    // after the packed clamp): 57 -> 53. What stands past the family:
    // `1/38`'s and `1/37`'s figures' `g.des_x/y` on 15081 and 15084 (the
    // Basilica Bombard steps 64 in y and its figures follow a frame late in
    // theirs; agreeing the block after) and `dest_angle` of `1/54` on 15214
    // and `1/50` on 15224 (parked 3380a, 3380b). Nothing on the word's
    // block 14992.
    pin_eq!(w.firsts.len(), 53, "keys that part in the window");
}

/// The closing state's window: run898b's closing block alone, the long's
/// last frame 15503 writing it (item 3380).
pub(crate) const WIDENING_TURKISH_CLOSING: (i64, i64) = (15502, 15505);

/// **The lobby's closing state** (item 3380, DECISIONS 55): run898b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 15503.
#[test]
fn run898b_turkish_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        crate::testenv::dump(TURKISH.long.0),
        crate::testenv::dump(TURKISH.start),
        trace(TURKISH.long.1),
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
        "Turkish closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 15504, "the closing block");
    pin_eq!(e.compared, 127, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3380, the first walk (the Turkish siege range and sight, the
    // word at 15503): 127 units compared on block 15504, none apart, no
    // building or city field-row, no closing field.
}
