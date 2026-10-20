//! **The battery's East Indies Spanish word** (DECISIONS 72, 83, 84; item
//! 3367): the held-out battery's lowest open game on the measure of
//! `9e63ec11`, promoted by item 3335's recipe. East Indies (`MAP_STYLE
//! 18`), human Nubians (4) against **the Spanish (9)** at Difficulty 3,
//! seed 12345 — minted by item 3360 under entry 72's rule (2), scored by
//! its first try and promoted out of the battery here, so it is a scored
//! lobby from this item (`battery::SCORED_LOBBIES`) and no longer held
//! out. Captures: run986 (a `DUMP_ALL` start) and run986b (the draw
//! stream to 24000, `cover=0`, its setup and closing blocks only); the
//! game does not end itself (`docs/AI.md` §284). `diff::spanish_word` is
//! the nation's other game, the closed Great Sahara one; the two are kept
//! apart.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_EAST: Lobby = Lobby {
    name: "EastIndiesSpanishTier3",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 3,
    start: "gamelog-run986-eastindies-spanish-diff3-start.txt",
    long: (
        "gamelog-run986b-eastindies-spanish-diff3-24k-trace.txt",
        "rontrace-run986b.log",
    ),
    length: 24000,
    // Item 3360's first try, held by the battery's measure to `9e63ec11`:
    // count 9028, sequence 9028 (ours 2 draws, theirs 12, at 0: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`). Item 3367 promoted it to
    // the AI word (`docs/AI.md` §284). **9028 → 12756** (item 3367): the
    // danger pass asks a building's own `ever_seen` (`BuildData::is_seen`),
    // not the fog a Spanish leader sees whole — run993b's block 8959,
    // `danger[1]` at half-cell (4, 4) ours 125, theirs 60, and the scout
    // `1/0` chose cell (7, 11) over theirs' (9, 8). **12756 → 14942**
    // (item 3367): `Build::train` writes a Dock's byte 0 into the computer
    // Bark `1/49`'s `combat.stance` too (run993c prints it `stance 0`; ours
    // held RAID there), and `think_attack` ends the think only on a unit
    // find — so the Caravel's idle search on 12742 takes `0/2004` and still
    // runs `think_scout`, and its cavarch search takes `0/2004` on 12753.
    // **14942 → 24000, the long's last frame** (item 3370): a caravan's
    // slot goes with it onto the Merchant Fleet `cast_transport` builds
    // (`+0x86`, `cast_transport@00670db0:117`–`121`) and back off at
    // `eject_contents`, so the fleet `1/79` (TI 318, sea), taking a
    // road-flagged waypoint over a built tile on 14942, verifies its
    // route's road — 3203 `calc_road_cost` draws theirs, none ours — and
    // the region check pushes the stripped top (run993d block 14943,
    // `1/79`'s `path[19].flags` ours 36, theirs 4).
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_EAST_WORD: i64 = if SPANISH_EAST.count < SPANISH_EAST.sequence {
    SPANISH_EAST.count
} else {
    SPANISH_EAST.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_east_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH_EAST.start, SPANISH_EAST.long) else {
        return;
    };
    eprintln!(
        "SPANISH_EAST {}: count {} sequence {} of {}; {}",
        SPANISH_EAST.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SPANISH_EAST.difficulty, "the lobby read back");
    pin_eq!(w.last, SPANISH_EAST.length, "the game's last frame");
    pin_eq!(
        w.count,
        SPANISH_EAST.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SPANISH_EAST.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_east_word_is_no_other_lobby() {
    let p = &SPANISH_EAST;
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

/// run993: the word 9028's window, blocks 9022..9278 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3367). run986 dumps blocks 0..36 and
/// run986b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN993: &str =
    "gamelog-run993-eastindies-spanish-diff3-window-9022-9279-leaders.txt";

/// The window: block 9022 through 9278; the word 9028's own block is 9029.
pub(crate) const WIDENING_SPANISH_EAST_FRAME_9028: (i64, i64) = (9022, 9278);

pub(crate) fn spanish_east_frame_9028_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_EAST.start],
        true,
        SPANISH_EAST.long,
        "run993",
        &[(RUN993, WIDENING_SPANISH_EAST_FRAME_9028.0)],
        WIDENING_SPANISH_EAST_FRAME_9028,
        1,
        &[9029],
        true,
    )
}

#[test]
fn run993_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_east_frame_9028_window() else {
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
    // Item 3367: 519 keys on the first try, the window opening with the
    // Scout `1/0` (TI 69, land) already apart — ours walking an
    // `ExploreTo` to (5880, 8952), theirs at (7588, 6631) on its way to
    // (7416, 6648). Walked back: both chose on 8959 with the same draws
    // over the same three cells around the human's city, and the score's
    // danger term parted — `danger[1]` at half-cell (4, 4) ours 125,
    // theirs 60 (run993b), the human's buildings unhalved for a viewer
    // whose fog `was_seen` lights whole (`leader_flags & 0x1000`, the
    // Spanish bonus) and whose `ever_seen` bit they do not carry. Built:
    // `danger_build_is_seen` is `build_is_seen`. 519 → 49, the standing
    // family alone (`SITE[·].reg`, `form`, `aggression`, the army `pool:n`
    // and `group:n.held`, the newborn `g.angle[0]`).
    pin_eq!(w.firsts.len(), 49, "keys that part in the window");
}

/// run993c: the word 12756's window, blocks 12750..13006 at `GUYS=4`,
/// `LEADERS=9` and `AMMO=5` (item 3367). run993 closes on 9278, before the
/// word.
pub(crate) const RUN993C: &str =
    "gamelog-run993c-eastindies-spanish-diff3-window-12750-13007-ammo.txt";

/// The window: block 12750 through 13006; the word 12756's own block is
/// 12757.
pub(crate) const WIDENING_SPANISH_EAST_FRAME_12756: (i64, i64) = (12750, 13006);

pub(crate) fn spanish_east_frame_12756_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_EAST.start],
        true,
        SPANISH_EAST.long,
        "run993c",
        &[(RUN993C, WIDENING_SPANISH_EAST_FRAME_12756.0)],
        WIDENING_SPANISH_EAST_FRAME_12756,
        1,
        &[12757],
        true,
    )
}

#[test]
fn run993c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_east_frame_12756_window() else {
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
    // Item 3367, the first try on 12756: 765 keys. The Caravel `1/49` (TI
    // 325, sea, born a Bark on 10369 out of a Dock) parts first on 12752,
    // `cavarch_o` ours −2, theirs −1: theirs' 12751 search took `0/2004`.
    // 765 → 61 by the same item's move to 14942: `1/49` agrees through the
    // window; the standing family stands, and `0/2004`'s `damage` parts on
    // 12774 (ours 2, theirs 3).
    pin_eq!(w.firsts.len(), 61, "keys that part in the window");
}

/// run993d: the word 14942's window, blocks 14936..15192 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3367). run993c closes on 13006, before
/// the word.
pub(crate) const RUN993D: &str =
    "gamelog-run993d-eastindies-spanish-diff3-window-14936-15193-leaders.txt";

/// The window: block 14936 through 15192; the word 14942's own block is
/// 14943.
pub(crate) const WIDENING_SPANISH_EAST_FRAME_14942: (i64, i64) = (14936, 15192);

pub(crate) fn spanish_east_frame_14942_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_EAST.start],
        true,
        SPANISH_EAST.long,
        "run993d",
        &[(RUN993D, WIDENING_SPANISH_EAST_FRAME_14942.0)],
        WIDENING_SPANISH_EAST_FRAME_14942,
        1,
        &[14943],
        true,
    )
}

#[test]
fn run993d_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_east_frame_14942_window() else {
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
    // Item 3367, the first try on 14942: 1006 keys. Past the standing
    // family (and `0/2000`'s `raid_stamp`, ours 12774 against 12773, parked
    // 3367a's neighbour), the first row is on the word's own block 14943:
    // `1/79`'s `path[19].flags`, ours 36, theirs 4 — the frame theirs spends
    // 3211 draws in `PathFinder::calc_road_cost`. Item 3370 built both
    // halves (the fleet's caravan slot, the stripped top): 1006 → 61, the
    // word 14942 → 24000. What stands: the standing family (`SITE[·].reg`,
    // `aggression[1]`, `form`, the army `pool:n` and `group:n.held`, the
    // newborn `g.angle[0]`, `0/2000`'s `raid_stamp`) and `1/87`'s
    // `dest_angle` on 15129 (ours 793640960, theirs 1008795648), which the
    // draw stream never feels.
    pin_eq!(w.firsts.len(), 61, "keys that part in the window");
}

/// The closing state's window: run986b's closing block alone, the long's
/// last frame 24000 writing it (item 3370).
pub(crate) const WIDENING_SPANISH_EAST_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3370, DECISIONS 55): run986b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run986b_spanish_east_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(SPANISH_EAST.long.0),
        dump(SPANISH_EAST.start),
        trace(SPANISH_EAST.long.1),
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
        "Spanish East closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 160, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    // Item 3370, the first walk (the word at 24000): 160 units compared on
    // block 24001, none apart, no building field-row; one closing field,
    // city `0/2000`'s `raid_stamp`, ours 12774 against theirs 12773 — the
    // human's city, standing since run993c's window (parked 3367a's
    // neighbour), which the draw stream never feels.
}
