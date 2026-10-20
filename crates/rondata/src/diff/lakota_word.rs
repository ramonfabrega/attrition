//! **The battery's Lakota word** (DECISIONS 72, 83, 84; item 3345): the
//! held-out battery's lowest open game once item 3343 closed the Bantu,
//! promoted by item 3335's recipe. Great Lakes (`MAP_STYLE 14`), human
//! Nubians (4) against **the Lakota (19)** at Difficulty 2, seed 12345 —
//! minted by item 3342 under entry 72's rule (2) as amended by 84, scored
//! by its first try and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Not the peace pair's lobby (`diff::lakota_pair`, East Indies
//! at Difficulty 3): the nation is the same, the game is not. Captures:
//! run975 (a `DUMP_ALL` start, blocks 0..36) and run975b (the draw stream
//! to the game's end, `cover=0`, its setup and closing blocks only); the
//! game ends itself on 7902. The word's window is run980's (`docs/AI.md`
//! §277).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const LAKOTA: Lobby = Lobby {
    name: "GreatLakesLakotaTier2",
    map_style: 14,
    ai_tribe: 19,
    difficulty: 2,
    start: "gamelog-run975-greatlakes-lakota-diff2-start.txt",
    long: (
        "gamelog-run975b-greatlakes-lakota-diff2-24k-trace.txt",
        "rontrace-run975b.log",
    ),
    length: 7902,
    // Item 3342's first try, held by the battery's measure to `ec062f94`:
    // count 6071, sequence 6071 (ours 2 draws, theirs 3, at 0: ours
    // `Ammo::init+0xcd9`, theirs `Unit::do_move+0xe84`). Item 3345
    // promoted it to the AI word (`docs/AI.md` §277). Item 3345, the
    // `NA`-style Slinger's release bays (piece 1440, `sim::launch`): 6071
    // → 7902, the game's last frame, count and sequence (run980b's block
    // 6047, `1/18`'s stone at `0/2` from (4962, 30153) in 7 frames).
    count: 7902,
    sequence: 7902,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const LAKOTA_WORD: i64 = if LAKOTA.count < LAKOTA.sequence {
    LAKOTA.count
} else {
    LAKOTA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn lakota_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(LAKOTA.start, LAKOTA.long) else {
        return;
    };
    eprintln!(
        "LAKOTA {}: count {} sequence {} of {}; {}",
        LAKOTA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, LAKOTA.difficulty, "the lobby read back");
    pin_eq!(w.last, LAKOTA.length, "the game's last frame");
    pin_eq!(w.count, LAKOTA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        LAKOTA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn lakota_word_is_no_other_lobby() {
    let p = &LAKOTA;
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

/// run980: the word 6071's window, blocks 6065..6321 at run976's detail
/// (`GUYS=4`; item 3345). run975 dumps blocks 0..36 and run975b its setup
/// and closing blocks only, so no dump on disk held the word's block.
pub(crate) const RUN980: &str = "gamelog-run980-greatlakes-lakota-diff2-window-6065-6322-guys4.txt";

/// The window: block 6065 through 6321; the word 6071's own block is 6072.
pub(crate) const WIDENING_LAKOTA_FRAME_6071: (i64, i64) = (6065, 6321);

pub(crate) fn lakota_frame_6071_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run980",
        &[(RUN980, WIDENING_LAKOTA_FRAME_6071.0)],
        WIDENING_LAKOTA_FRAME_6071,
        1,
        &[6072],
        true,
    )
}

#[test]
fn run980_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_6071_window() else {
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
    // Item 3345: 1147 keys parted on the first try, the window opening
    // apart: the Lakota Slingers' stones flew a frame long — `1/18`'s at
    // the citizen `0/2`, thrown on 6046, landed on 6053 against 6052
    // (`damage_frame`), and `1/20`'s at the city centre on 6059 against
    // 6058 (`raid_stamp`), so `0/2` fled a frame late. The Slinger of the
    // `NA` art style is piece 1440, which `sim::launch` had no bays for;
    // run980b prints the stone's `sx, sy` (4962, 30153) and `total_time`
    // 7, where ours threw from the unit's own (5016, 30264) in 8. Built:
    // 1147 → 30. What stands: 6065's standing 27 (`form` −1 against 9, the
    // leaders' `SITE[·].reg` 1 against 0, `aggression[1]`, the army pool's
    // `pool:64`/`66`/`67` and `group:64..67.held` with no `GROUPDATA`
    // printed), who=1's `SITE[1].reg` on 6176, the same family, and two
    // Slingers' `order:target` — `1/12` on 6085 and `1/6` on 6092, ours
    // the citizen `0/2` and theirs the city centre `0/2000` (parked by
    // item 3345); the draw stream agrees past both to the game's end.
    pin_eq!(w.firsts.len(), 30, "keys that part in the window");
}

/// The closing state's window: run975b's closing block alone, the long's
/// last frame 7902 writing it (item 3345).
pub(crate) const WIDENING_LAKOTA_CLOSING: (i64, i64) = (7901, 7904);

/// **The lobby's closing state** (item 3345, DECISIONS 55): run975b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 7902.
#[test]
fn run975b_lakota_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(LAKOTA.long.0),
        dump(LAKOTA.start),
        trace(LAKOTA.long.1),
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
        "Lakota closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 7903, "the closing block");
    pin_eq!(e.compared, 26, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3345, the first walk (with piece 1440's bays, the word at
    // 7902): 26 units compared on block 7903, none apart, no building or
    // city field-row, no closing field.
}
