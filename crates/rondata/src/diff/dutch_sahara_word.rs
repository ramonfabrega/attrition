//! **The battery's Dutch word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3369): the held-out battery's lowest open game on the measure of
//! `efc5d4c1`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Dutch (22)** at Difficulty 3, seed 12345
//! — minted by item 3365 (entry 72's rule (2): a nation held only by a
//! closed game on another map), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::dutch_word`, which is the closed East Indies game at Difficulty
//! 4. Captures: run990 (a `DUMP_ALL` start, blocks 0..36) and run990b (the
//! draw stream to the game's end, `cover=0`, its setup and closing blocks
//! only); the game ends itself on 16109 (`docs/AI.md` §285).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const DUTCH_SAHARA: Lobby = Lobby {
    name: "GreatSaharaDutchTier3",
    map_style: 7,
    ai_tribe: 22,
    difficulty: 3,
    start: "gamelog-run990-greatsahara-dutch-diff3-start.txt",
    long: (
        "gamelog-run990b-greatsahara-dutch-diff3-24k-trace.txt",
        "rontrace-run990b.log",
    ),
    length: 16109,
    // Item 3365's first try, held by the battery's measure to `efc5d4c1`:
    // count 311, sequence 311 (ours 7 draws, theirs 6, at 1: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Farms::inc_time+0x1ae`). Item 3369 promoted it to the AI word
    // (`docs/AI.md` §285), and moved it 311 → 15725 (+15414): the Dutch
    // merchant's `think_attack` leg and that function's `go_to_city` tail
    // (run995's block 310: `1/1` (`MERCHANTDUTCH`) `group` −1 ours, 64
    // theirs, `orders.len` 0/1, `ATTACK_TO` (41304, 17304); agreeing
    // after), and a merchant re-pieced as its pack begins (block 311:
    // figure 0's `gpiece` 12 ours, 6348 theirs, `end_time` 3/149;
    // agreeing after). 15725: ours 9 draws, theirs 13, at 4: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69`
    // — and moved it 15725 -> 16109 (+384), the game's last frame: an
    // unpacked merchant's "over time" piece (run995d's block 15717: who=1's
    // Armed Merchant `1/51` re-pieces both figures to 50689, `total − 5`
    // by the Dutch `BUILD_CONTINENT 1`, with the cart on its own point,
    // where ours put figure 0 on 2124 and left figure 1 on 14796 and its
    // offset; agreeing after).
    count: 16109,
    sequence: 16109,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const DUTCH_SAHARA_WORD: i64 = if DUTCH_SAHARA.count < DUTCH_SAHARA.sequence {
    DUTCH_SAHARA.count
} else {
    DUTCH_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn dutch_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(DUTCH_SAHARA.start, DUTCH_SAHARA.long) else {
        return;
    };
    eprintln!(
        "DUTCH_SAHARA {}: count {} sequence {} of {}; {}",
        DUTCH_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, DUTCH_SAHARA.difficulty, "the lobby read back");
    pin_eq!(w.last, DUTCH_SAHARA.length, "the game's last frame");
    pin_eq!(
        w.count,
        DUTCH_SAHARA.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        DUTCH_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn dutch_sahara_word_is_no_other_lobby() {
    let p = &DUTCH_SAHARA;
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

/// run995: the word 311's window, blocks 305..561 at `GUYS=4`, `LEADERS=9`
/// and `GOODS=3` (item 3369). run990 dumps blocks 0..36 and run990b its
/// setup and closing blocks only, so no dump on disk held the word's block.
pub(crate) const RUN995: &str = "gamelog-run995-greatsahara-dutch-diff3-window-305-562.txt";

/// The window: block 305 through 561; the word 311's own block is 312.
pub(crate) const WIDENING_DUTCH_SAHARA_FRAME_311: (i64, i64) = (305, 561);

pub(crate) fn dutch_sahara_frame_311_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_SAHARA.start],
        true,
        DUTCH_SAHARA.long,
        "run995",
        &[(RUN995, WIDENING_DUTCH_SAHARA_FRAME_311.0)],
        WIDENING_DUTCH_SAHARA_FRAME_311,
        1,
        &[312],
        true,
    )
}

#[test]
fn run995_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_sahara_frame_311_window() else {
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
    // Item 3369: 433 keys on the first try, the standing family on 305
    // (`form` −1 against 9, the leaders' `SITE[·].reg` 1 against 0,
    // `aggression`, `group:64`/`65.held` and `pool:65` with no `GROUPDATA`
    // printed) and then, on block 310, the Armed Merchant `1/1`
    // (`MERCHANTDUTCH`, type 62) drafted into who=1's army in theirs —
    // `group` −1 against 64, `orders.len` 0 against 1 (`ATTACK_TO`
    // (41304, 17304)), `form_mod` −1 against 50 — the frame after its
    // unpack seated it.
    // Built (the merchant's `think_attack` leg, the `go_to_city` tail,
    // the pack's re-piece): 433 -> 39, the standing family on 305.
    pin_eq!(w.firsts.len(), 39, "keys that part in the window");
}

/// run995c: the word 15725's window, blocks 15719..15975 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3369). run995 closes on 561, and
/// run990b dumps its setup and closing blocks only.
pub(crate) const RUN995C: &str = "gamelog-run995c-greatsahara-dutch-diff3-window-15719-15976.txt";

/// The window: block 15719 through 15975; the word 15725's own block is
/// 15726.
pub(crate) const WIDENING_DUTCH_SAHARA_FRAME_15725: (i64, i64) = (15719, 15975);

pub(crate) fn dutch_sahara_frame_15725_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_SAHARA.start],
        true,
        DUTCH_SAHARA.long,
        "run995c",
        &[(RUN995C, WIDENING_DUTCH_SAHARA_FRAME_15725.0)],
        WIDENING_DUTCH_SAHARA_FRAME_15725,
        1,
        &[15726],
        true,
    )
}

#[test]
fn run995c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_sahara_frame_15725_window() else {
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
    // Item 3369: 748 keys on the first try. The window opens apart: the
    // standing family on 15719 and who=1's Armed Merchant `1/51`, on its
    // pack's first frame two after its unpack ended, whose figure 1 is
    // still walking out to its deployed spot (17597, 25530) in theirs
    // from beside the merchant's point (17689, 25362), and stands on it
    // in ours; on the word's block 15726 its `g.end_time[1]` is 150 ours,
    // 60 theirs. run995d walks it back to the unpack's last frame, 15717.
    // Built (an unpacked merchant's "over time" piece): 748 -> 34, the
    // standing family on 15719 (`form`, the leaders' `SITE[·].reg`,
    // `aggression[1]`, the army pools and `group:N.held` with no
    // `GROUPDATA` printed) and `pool:69`/`group:69.held` on 15796.
    pin_eq!(w.firsts.len(), 34, "keys that part in the window");
}

/// The closing state's window: run990b's closing block alone, the long's
/// last frame 16109 writing it (item 3369).
pub(crate) const WIDENING_DUTCH_SAHARA_CLOSING: (i64, i64) = (16108, 16111);

/// **The lobby's closing state** (item 3369, DECISIONS 55): run990b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 16109.
#[test]
fn run990b_dutch_sahara_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        crate::testenv::dump(DUTCH_SAHARA.long.0),
        crate::testenv::dump(DUTCH_SAHARA.start),
        trace(DUTCH_SAHARA.long.1),
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
        "Dutch Sahara closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 16110, "the closing block");
    pin_eq!(e.compared, 86, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3369, the first walk (with the merchant's "over time" piece, the
    // word at 16109): 86 units compared on block 16110, none apart, no
    // building or city field-row, no closing field.
}

/// run995d: the walk back from the word 15725, blocks 15462..15718 at the
/// same detail (item 3369) — run995c opened apart on 1/51's crew, and no
/// dump held the unpack before it.
pub(crate) const RUN995D: &str = "gamelog-run995d-greatsahara-dutch-diff3-window-15462-15719.txt";

/// The window: block 15462 through 15718; `1/51`'s unpack ends on 15717.
pub(crate) const WIDENING_DUTCH_SAHARA_GAP_995D: (i64, i64) = (15462, 15718);

pub(crate) fn dutch_sahara_gap_995d_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_SAHARA.start],
        true,
        DUTCH_SAHARA.long,
        "run995d",
        &[(RUN995D, WIDENING_DUTCH_SAHARA_GAP_995D.0)],
        WIDENING_DUTCH_SAHARA_GAP_995D,
        1,
        &[15717],
        true,
    )
}

#[test]
fn run995d_s_walk_back_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_sahara_gap_995d_window() else {
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
    // Item 3369, first read after the build: 32 keys, the standing family
    // on 15462 alone. Before it, `1/51`'s crew parted on 15717 — both
    // figures on 50689 at the merchant's point in theirs, figure 0 on 2124
    // and figure 1 on 14796 and its offset in ours.
    pin_eq!(w.firsts.len(), 32, "keys that part in the window");
}
