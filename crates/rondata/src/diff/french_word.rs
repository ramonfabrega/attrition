//! **The battery's French word** (DECISIONS 72, 83, 84; item 3347): the
//! held-out battery's second open game once item 3343 closed the Bantu,
//! promoted by item 3343's recipe (`diff::french_word`). Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the French (10)** at
//! Difficulty 4, seed 12345 — the lobby item 3343 minted under entry 72's
//! rule (2) (`docs/AI.md` §125, §276), scored by its first try at 13422 of
//! 16,764 and promoted out of the battery here, so it is a scored lobby
//! from this item (`battery::SCORED_LOBBIES`) and no longer held out.
//! Captures: run977 (a `DUMP_ALL` start) and run977b (the draw stream to the
//! game's end on 16764, `cover=0`, its setup and closing blocks only); the
//! word's window is `docs/AI.md` §278's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const FRENCH: Lobby = Lobby {
    name: "GreatSaharaFrenchTier4",
    map_style: 7,
    ai_tribe: 10,
    difficulty: 4,
    start: "gamelog-run977-greatsahara-french-diff4-start.txt",
    long: (
        "gamelog-run977b-greatsahara-french-diff4-24k-trace.txt",
        "rontrace-run977b.log",
    ),
    length: 16764,
    // Item 3343, the battery's first try, count and sequence both at
    // 13422: ours 14 draws, theirs 19, at 6 ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::init_real+0x52`. Item 3347
    // promoted it to the AI word. Item 3347, `train_time`'s French siege
    // arm (`FRENCH_SIEGE_SPEED`, the Siege Factory `1/2026`'s Trebuchet
    // 30000 → 24000, born on 13422): 13422 → 13843, ours 8 draws, theirs
    // 10, at 5 ours `Farms::inc_time+0x1ae`, theirs
    // `Ammo::do_damage+0xc59`. Item 3347, the Crossbowman's and the
    // Cavalry Archer's release nodes (`launch::MEASURED`, pieces 122 and
    // 138; `1/39`'s round from (9401, 30970, 842), not its guy's (9480,
    // 30936, 614), 13 frames against 14): 13843 → 14604, ours 15 draws,
    // theirs 16, at 6 ours `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`,
    // theirs `Unit::fight+0x9b0`.
    count: 14604,
    sequence: 14604,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const FRENCH_WORD: i64 = if FRENCH.count < FRENCH.sequence {
    FRENCH.count
} else {
    FRENCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn french_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(FRENCH.start, FRENCH.long) else {
        return;
    };
    eprintln!(
        "FRENCH {}: count {} sequence {} of {}; {}",
        FRENCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, FRENCH.difficulty, "the lobby read back");
    pin_eq!(w.last, FRENCH.length, "the game's last frame");
    pin_eq!(w.count, FRENCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        FRENCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn french_word_is_no_other_lobby() {
    let p = &FRENCH;
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

/// run982: the word 13422's window and the word 13843's, blocks 13416..14093
/// at `GUYS=2`, `AMMO=5` (item 3347); run977 dumps blocks 1, 36 and 37 and
/// run977b its setup and closing blocks only.
pub(crate) const RUN982: &str =
    "gamelog-run982-greatsahara-french-diff4-window-13416-14093-ammo.txt";

/// The window: block 13416 through 14093; the words' own blocks are 13423
/// and 13844.
pub(crate) const WIDENING_FRENCH_FRAME_13422: (i64, i64) = (13416, 14093);

pub(crate) fn french_frame_13422_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH.start],
        true,
        FRENCH.long,
        "run982",
        &[(RUN982, WIDENING_FRENCH_FRAME_13422.0)],
        WIDENING_FRENCH_FRAME_13422,
        1,
        &[13423, 13844],
        true,
    )
}

#[test]
fn run982_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_frame_13422_window() else {
        return;
    };
    pin_eq!(w.blocks, 677, "every captured block");
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
    // Item 3347. 41 keys after the release nodes (`cf3e7e2c2`): 36 standing
    // on 13416, as run978's — the leader's sites and aggression, the
    // formations, who=1's groups 64/65/68 and four of its guys' waypoints;
    // who=1's group and pool 66 on 13424 (the Trebuchet `1/86`, held in a
    // group ours forms and theirs does not) and 67 on 13600; and `1/82`'s
    // `dest_angle` on 13856. `0/0`'s wound on 13870 and `0/2000`'s raid stamp
    // on 13941 left with the Cavalry Archer's and the Dragoon's nodes.
    pin_eq!(w.firsts.len(), 41, "run982 with the release nodes");
}
