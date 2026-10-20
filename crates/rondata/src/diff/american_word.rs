//! **The battery's American word** (DECISIONS 72, 83, 84; item 3374): the
//! held-out battery's lowest open game on the measure of `13bebf18` once
//! item 3370 closed the East Indies Spanish and the Dutch went to item
//! 3369, promoted by item 3335's recipe. East Indies (`MAP_STYLE 18`),
//! human Nubians (4) against **the Americans (20)** at Difficulty 1, seed
//! 12345 — minted by item 3314 (DECISIONS 72, a nation no dump held, a tier
//! no lobby on the map had), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run955 (a
//! `DUMP_ALL` start) and run955b (the draw stream to 24000, `cover=0`, its
//! setup and closing blocks only); the game does not end itself
//! (`docs/AI.md` §287).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN: Lobby = Lobby {
    name: "EastIndiesAmericanTier1",
    map_style: 18,
    ai_tribe: 20,
    difficulty: 1,
    start: "gamelog-run955-eastindies-american-diff1-start.txt",
    long: (
        "gamelog-run955b-eastindies-american-diff1-24k-trace.txt",
        "rontrace-run955b.log",
    ),
    length: 24000,
    // Item 3314's lobby, held by the battery's measure to `13bebf18`:
    // count 9382, sequence 9382 (ours 14 draws, theirs 12, at 1: ours
    // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`).
    // Item 3374 promoted it to the AI word (`docs/AI.md` §287), and built
    // `get_cost`'s American arm (a quarter off a land unit's upgrade,
    // `get_cost:528`–`531`): 9382 → 10982, ours 41 draws, theirs 44, at 3:
    // ours `Unit::think_scout+0x941`, theirs `Leader::make_stuff+0x63d`.
    // Value diff, run999 block 9376: who=1's `bucket[0:food]` ours 87,
    // theirs 107, `bucket[4:metal]` 158 against 178 → 107/107, 178/178.
    // Then `get_cost`'s government arm (`:1048`–`1053`, a government costs
    // an American leader nothing): 10982 → 17215, ours 9 draws, theirs 15,
    // at 0: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`. Value diff, run999b block
    // 10983: who=1's `bucket[0:food]` ours 64, theirs 134, `bucket[4:metal]`
    // 27 against 97 (ours paid Republic's 70 and 70; theirs queued it
    // free, and a Dock beside it) → 134/134, 97/97. Then
    // `construct_time`'s first arm (an American's first wonder takes one
    // frame of clock while no rival has the same one unfinished): 17215 →
    // **24000**, the long's last frame. Value diff, run999c block 17209:
    // the Pyramids `1/2031`'s `construct_hits` ours 1101, theirs 2200 →
    // 2200/2200, and who=1's `wonder_mark` 1 on 17214 on both sides.
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AMERICAN_WORD: i64 = if AMERICAN.count < AMERICAN.sequence {
    AMERICAN.count
} else {
    AMERICAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn american_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AMERICAN.start, AMERICAN.long) else {
        return;
    };
    eprintln!(
        "AMERICAN {}: count {} sequence {} of {}; {}",
        AMERICAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, AMERICAN.difficulty, "the lobby read back");
    pin_eq!(w.last, AMERICAN.length, "the game's last frame");
    pin_eq!(w.count, AMERICAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        AMERICAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn american_word_is_no_other_lobby() {
    let p = &AMERICAN;
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

/// run999: the word 9382's window, blocks 9376..9632 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3374). run955 dumps blocks 0..36 and
/// run955b its setup and closing blocks only, so no dump on disk held the
/// leader's ledger on the word's block.
pub(crate) const RUN999: &str =
    "gamelog-run999-eastindies-american-diff1-window-9376-9633-leaders.txt";

/// The window: block 9376 through 9632; the word 9382's own block is 9383.
pub(crate) const WIDENING_AMERICAN_FRAME_9382: (i64, i64) = (9376, 9632);

pub(crate) fn american_frame_9382_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN.start],
        true,
        AMERICAN.long,
        "run999",
        &[(RUN999, WIDENING_AMERICAN_FRAME_9382.0)],
        WIDENING_AMERICAN_FRAME_9382,
        1,
        &[9383],
        true,
    )
}

#[test]
fn run999_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_frame_9382_window() else {
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
    // Item 3374: 601 keys parted on the first try, the window opening
    // apart on who=1's `bucket[0:food]` 87 against 107 and
    // `bucket[4:metal]` 158 against 178 on block 9376 with `income`
    // agreeing: a quarter of an 80-food, 80-metal land upgrade
    // `AMERICANS_UPGRADE_DISCOUNT` takes off and this crate did not. Built:
    // 601 → 51. On 9382 the food then covers less, so `use_market` spends
    // its second sell pass, as theirs does. What stands: the standing
    // family alone (`SITE[·].reg` 65 against 0, `aggression`, `form` −1
    // against 0/9, the army `pool:n` and `group:n.held` with no `GROUPDATA`
    // printed, the newborn `g.angle[0]`).
    pin_eq!(w.firsts.len(), 51, "keys that part in the window");
}

/// run999b: the word 10982's window, blocks 10976..11232 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3374). run999 closes on 9632, before
/// the word.
pub(crate) const RUN999B: &str =
    "gamelog-run999b-eastindies-american-diff1-window-10976-11233-leaders.txt";

/// The window: block 10976 through 11232; the word 10982's own block is
/// 10983.
pub(crate) const WIDENING_AMERICAN_FRAME_10982: (i64, i64) = (10976, 11232);

pub(crate) fn american_frame_10982_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN.start],
        true,
        AMERICAN.long,
        "run999b",
        &[(RUN999B, WIDENING_AMERICAN_FRAME_10982.0)],
        WIDENING_AMERICAN_FRAME_10982,
        1,
        &[10983],
        true,
    )
}

#[test]
fn run999b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_frame_10982_window() else {
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
    // Item 3374: 575 keys parted before the government arm, the window
    // opening apart on block 10983 on who=1's `bucket[0:food]` 64 against
    // 134 and `bucket[4:metal]` 27 against 97 — Republic's 70 and 70, which
    // `get_cost:1048`–`1053` makes free for an American leader — and the
    // make list's row 8 (`val` 5358976 against 53589: theirs took the
    // head). Built: 575 → 55, the standing family alone (`SITE[·].reg`,
    // `aggression`, `form`, the army `pool:n` and `group:n.held`, the
    // newborn `g.angle[0]`).
    pin_eq!(w.firsts.len(), 55, "keys that part in the window");
}

/// run999c: the word 17215's window, blocks 17209..17465 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3374). run999b closes on 11232, before
/// the word.
pub(crate) const RUN999C: &str =
    "gamelog-run999c-eastindies-american-diff1-window-17209-17466-leaders.txt";

/// The window: block 17209 through 17465; the word 17215's own block is
/// 17216.
pub(crate) const WIDENING_AMERICAN_FRAME_17215: (i64, i64) = (17209, 17465);

pub(crate) fn american_frame_17215_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN.start],
        true,
        AMERICAN.long,
        "run999c",
        &[(RUN999C, WIDENING_AMERICAN_FRAME_17215.0)],
        WIDENING_AMERICAN_FRAME_17215,
        1,
        &[17216],
        true,
    )
}

#[test]
fn run999c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_frame_17215_window() else {
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
    // Item 3374: 879 keys parted before the free first wonder, the window
    // opening apart on block 17209 on the Pyramids `1/2031`'s
    // `construct_hits`, ours 1101 against 2200 — `construct_time`'s
    // American arm, which this crate carried as `ClockMods` and nothing
    // set — then who=1's `wonder_mark` 0 against 1 on 17214, when theirs
    // finished it. Built: 879 → 50, the standing family alone.
    pin_eq!(w.firsts.len(), 50, "keys that part in the window");
}

/// The closing state's window: run955b's closing block alone, the long's
/// last frame 24000 writing it (item 3374).
pub(crate) const WIDENING_AMERICAN_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3374, DECISIONS 55): run955b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run955b_american_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(AMERICAN.long.0),
        dump(AMERICAN.start),
        trace(AMERICAN.long.1),
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
        "American closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 79, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3374, the first walk (the word at 24000): 79 units compared on
    // block 24001, none apart, no building field-row, no closing field;
    // the human's `0/4` is torn (the dump's own n − 1 position).
}
