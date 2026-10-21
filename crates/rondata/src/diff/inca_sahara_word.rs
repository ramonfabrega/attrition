//! **The battery's Inca word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3385): the held-out battery's lowest open game on the measure of
//! `60438302`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Inca (2)** at Difficulty 3, seed 12345 —
//! minted by item 3383 (entry 72's rule (2): a nation held only by a closed
//! game on another map), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::inca_word`, which is the closed East Indies game. Captures:
//! run1006 (a `DUMP_ALL` start) and run1006b (the draw stream to the game's
//! end, `cover=0`, its setup and closing blocks only); the game ends itself
//! on 15632 (`docs/AI.md` §293).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA_SAHARA: Lobby = Lobby {
    name: "GreatSaharaIncaTier3",
    map_style: 7,
    ai_tribe: 2,
    difficulty: 3,
    start: "gamelog-run1006-greatsahara-inca-diff3-start.txt",
    long: (
        "gamelog-run1006b-greatsahara-inca-diff3-24k-trace.txt",
        "rontrace-run1006b.log",
    ),
    length: 15632,
    // Item 3383's first try, held by the battery's measure to `60438302`:
    // count 13127, sequence 13126 (ours 10 draws, theirs 10, at 8: ours
    // `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`). Item
    // 3385 promoted it to the AI word (`docs/AI.md` §293), and moved it
    // 13126 -> 13250 (+124): piece 841's pivot releases (run1011b's `1/72`
    // round at `0/2000` loosed on block 13112: `sx, sy, sz` (7629, 31482,
    // 652) ours, (7563, 31441, 893) theirs, `total_time` 17 against 16;
    // agreeing after), and 13250 -> 13282 (+32): the fire on the move's
    // bearing in `find_nearby_target` (run1011's block 13226: `1/72`'s
    // `cavarch_o/who` -1/-1 theirs, the city `0/2000` behind it at 179
    // degrees off its facing; 2000/0 ours, which struck two frames early),
    // and 13282 -> 13313 (+31): `UnitData::full` (run1011's woodcutter
    // `0/3`: `full` 6 theirs on block 13282, 0 ours, after the Inti
    // Macemen `1/92` and `1/91` found no open slot round it on 13278 and
    // 13279 and dropped their attacks — `find_attack_pos`'s melee return
    // 0, run1011d's packet — so the Heavy Archer `1/99` took `0/4` over
    // it, `order:target` (0, 4) theirs against (0, 3) ours), and 13313
    // -> 13416 (+103): a decoy target takes `get_damage`'s step 17
    // (run1011e's packet: the General's decoy Light Horse `1/82`, struck
    // by friendly fire on 13297, `get_damage` 19688 theirs and 6 hits
    // ours, `target_side` having written `decoy` false for every unit).
    count: 13416,
    sequence: 13416,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INCA_SAHARA_WORD: i64 = if INCA_SAHARA.count < INCA_SAHARA.sequence {
    INCA_SAHARA.count
} else {
    INCA_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn inca_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INCA_SAHARA.start, INCA_SAHARA.long) else {
        return;
    };
    eprintln!(
        "INCA_SAHARA {}: count {} sequence {} of {}; {}",
        INCA_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INCA_SAHARA.difficulty, "the lobby read back");
    pin_eq!(w.last, INCA_SAHARA.length, "the game's last frame");
    pin_eq!(w.count, INCA_SAHARA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INCA_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn inca_sahara_word_is_no_other_lobby() {
    let p = &INCA_SAHARA;
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

/// run1011: the word's window, blocks 13104..13360 at `GUYS=4`,
/// `LEADERS=9`, `GOODS=3` and `AMMO=5` (item 3385). run1006 dumps blocks
/// 0..36 and run1006b its setup and closing blocks only, so no dump on disk
/// held the word's block.
pub(crate) const RUN1011: &str = "gamelog-run1011-greatsahara-inca-diff3-window-13104-13361.txt";

/// The window: block 13104 through 13360; the word 13250's own block is
/// 13251.
pub(crate) const WIDENING_INCA_SAHARA_FRAME_13250: (i64, i64) = (13104, 13360);

pub(crate) fn inca_sahara_frame_13250_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_SAHARA.start],
        true,
        INCA_SAHARA.long,
        "run1011",
        &[(RUN1011, WIDENING_INCA_SAHARA_FRAME_13250.0)],
        WIDENING_INCA_SAHARA_FRAME_13250,
        1,
        &[13251],
        true,
    )
}

#[test]
fn run1011_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_sahara_frame_13250_window() else {
        return;
    };
    pin_eq!(w.blocks, 0, "every captured block");
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
    pin_eq!(w.firsts.len(), 0, "keys that part in the window");
}
