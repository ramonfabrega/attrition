//! **The Lakota at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, item 3561): East Indies, `MAP_STYLE 18`, human Nubians (4) against
//! the **Lakota (19)** at Difficulty 5, seed 12345, `STARTING_RESOURCES
//! 7`, the age held at Medieval: `STARTING_TECHNOLOGY 2`,
//! `ENDING_TECHNOLOGY 2` — the Persians' held Medieval lobby with the
//! nation moved. The Lakota's other pair on this map is `lakota_pair`, the
//! peace pair, at Difficulty 3. Captures: run1203 (a `DUMP_ALL` start) and
//! run1204 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §314.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const LAKOTA_MEDIEVAL: Pair = Pair {
    name: "EastIndiesLakotaMedieval",
    map_style: 18,
    ai_tribe: 19,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1203-eastindies-lakota-medieval-start.txt",
    long: (
        "gamelog-run1204-eastindies-lakota-medieval-trace.txt",
        "rontrace-run1204.log",
    ),
    // The game ended itself on 4973 (closing block 4974).
    length: 4973,
    // Item 3561, first try: frame 4168 by the count and the sequence, ours
    // 90 draws against 89, index 87 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x1ed`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` section 314).
    count: 4168,
    sequence: 4168,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const LAKOTA_MEDIEVAL_PAIR_WORD: i64 =
    if LAKOTA_MEDIEVAL.count < LAKOTA_MEDIEVAL.sequence {
        LAKOTA_MEDIEVAL.count
    } else {
        LAKOTA_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &LAKOTA_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "LAKOTA_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — `ENDING_TECHNOLOGY` beside `STARTING_TECHNOLOGY` — and the long
/// is scoreable.
#[test]
fn lakota_medieval_pair_captures_say_their_lobby() {
    let p = &LAKOTA_MEDIEVAL;
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
        "the Lakota Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(18, 19, 5)` is no held-out game's — the
/// battery's one Lakota game is `GreatSaharaLakotaTier2`, `(7, 19, 2)`, and
/// `GreatLakesLakotaTier2`'s `(14, 19, 2)` was promoted to a scored word
/// (item 3345) — and no other pinned pair's: the Lakota's other pair is
/// `lakota_pair`'s `(18, 19, 3)`, the peace pair on this map at
/// Difficulty 3 and no held age. Both `GAME INFO`s are read.
#[test]
fn lakota_medieval_pair_is_no_other_lobby() {
    let p = &LAKOTA_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let lakota: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        lakota,
        [("GreatSaharaLakotaTier2", 7, 2)],
        "the battery's Lakota games"
    );
    let other = &super::lakota_pair::LAKOTA;
    assert_eq!(other.ai_tribe, p.ai_tribe);
    assert_ne!(
        (other.map_style, other.ai_tribe, other.difficulty),
        triple,
        "{} is this lobby",
        other.name
    );
    let info = |name: &str| {
        let path = dump(name)?;
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial()?;
        let get = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        Some((
            get("MAP_STYLE"),
            get("DIFFICULTY"),
            get("STARTING_TECHNOLOGY"),
            get("ENDING_TECHNOLOGY"),
        ))
    };
    let (Some(peace), Some(ours)) = (info(other.long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(peace.0, Some(18), "the peace pair's map");
    assert_eq!(peace.1, Some(3), "the peace pair's difficulty");
    assert_eq!(
        ours,
        (Some(18), Some(5), Some(2), Some(2)),
        "this pair's lobby"
    );
}

/// run1204b: the word 4168's window, blocks 4163..4419 at the long's
/// detail — six blocks before the word's block 4169 and 250 after (item
/// 3561); the stride's blocks are 4101 and 4201.
pub(crate) const RUN1204B: &str =
    "gamelog-run1204b-eastindies-lakota-medieval-window-4163-4419.txt";

/// The window: block 4163 through 4419; the word 4168's own block is 4169.
pub(crate) const WIDENING_LAKOTA_MEDIEVAL_FRAME_4168: (i64, i64) = (4163, 4419);

/// **The word 4168's window**: run1204b walked from run1203's start with
/// the recorder on (item 3561).
pub(crate) fn lakota_medieval_frame_4168_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_MEDIEVAL.start],
        true,
        LAKOTA_MEDIEVAL.long,
        "run1204b",
        &[(RUN1204B, WIDENING_LAKOTA_MEDIEVAL_FRAME_4168.0)],
        WIDENING_LAKOTA_MEDIEVAL_FRAME_4168,
        1,
        &[4169],
        true,
    )
}

#[test]
fn lakota_medieval_pair_run1204b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_medieval_frame_4168_window() else {
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
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3561, the first walk: 2163 keys. Block 4163 carries the
    // standing families (`form`, `SITE[i].reg`). **The state first parts
    // on block 4169**, at the Lakota Heavy Archers `1/19` (TypeIndex 173,
    // land, uid 39), standing at (6264, 10008) in an `ATTACKORDER` on the
    // fleeing Nubian Citizen `0/10` (uid 17): theirs re-aims the attack at
    // the Citizen `0/5` (uid 12; `new_ord` 1, `in_range` 0, `orders.len`
    // 2, `near` 0/2000) and pushes its move to (5400, 9000) on 4170; ours
    // keeps `0/10` and pushes a chase to (6696, 9528) on 4169 (`order:kind`
    // ours 1 theirs 10, `orders.len` 3 against 2). **The word 4168** (block
    // 4169), ours 90 draws against 89, index 87 ours `Guy::set_anim+0x97a
    // < Guy::inc_time+0x1ed` (`1/19`), theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` section 314).
    pin_eq!(w.firsts.len(), 2163, "run1204b baseline");
}
