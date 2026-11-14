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
    // Item 3343, a script's research priced against the whole bucket
    // (`BuildData::can_queue`'s `can_pay_cost(…, 1)`; the Classical Age on
    // 2776), with a group's `queue_up` drawing the escrow down: 3184 →
    // 4376, ours 226 draws, theirs 93, at 53 ours
    // `Build::find_gather_tiles+0x10a`, theirs `GameDaemon::calc_market+0x54`.
    // Item 3343, a tech already in research refused (`BuildData::can_make`'s
    // `researching` arm; the script's Art of War asked twice on 4376):
    // 4376 → 15582, ours 15 draws, theirs 14, at 4 ours
    // `Leader::make_stuff+0x63d`, theirs `Guy::set_anim+0x97a <
    // Unit::do_non_flat_gather+0xb99`.
    // Item 3343, `get_cost`'s rares' tail (Sugar on food — who=1's
    // Enlightenment Age 382 food where 425 left it unaffordable on 15578, so
    // `check_income`'s 0x40 quartered the offer): 15582 → 23627, the
    // game's end — the closing state is scored
    // (`run972b_bantu_closing_state`).
    count: 23627,
    sequence: 23627,
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
    pin_eq!(w.blocks, 255, "every captured block");
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

/// run978c: the gap from run978's last block to the word 3184's window,
/// one block in fifty (`--stride 50`): blocks 258, 308, …, 3158, at
/// `GUYS=2` (item 3343).
pub(crate) const RUN978C: &str =
    "gamelog-run978c-greatsahara-bantu-diff4-gap-257-3175-stride50.txt";

/// The window: block 258 through 3158, one in fifty.
pub(crate) const WIDENING_BANTU_GAP_978C: (i64, i64) = (258, 3158);

pub(crate) fn bantu_gap_978c_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978c",
        &[(RUN978C, WIDENING_BANTU_GAP_978C.0)],
        WIDENING_BANTU_GAP_978C,
        1,
        &[],
        true,
    )
}

#[test]
fn run978c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_gap_978c_window() else {
        return;
    };
    pin_eq!(w.blocks, 59, "every captured block");
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
    // Item 3343. On the first try (with the Bantu speed arm and pop cap)
    // 47 keys: past the standing set, who=1's `escrow[0:food]` ours 18
    // against theirs 2 on block 1608 — the script's citizen of 1576 paid
    // against the reservation in theirs (run978d) — and on 2808 theirs'
    // Classical Age (`tech_frame` 2776, `bucket[0:food]` 25 against 275,
    // `script_step` 20 against 18). With a group's `queue_up` drawing the
    // escrow down and the research priced against the whole bucket,
    // **37**: the standing set alone.
    pin_eq!(w.firsts.len(), 37, "run978c, quiet to its last block");
}

/// run978d: blocks 1556..1611 at `GUYS=2` (item 3343), around run978c's
/// first value row, who=1's `escrow[0:food]` on block 1608.
pub(crate) const RUN978D: &str = "gamelog-run978d-greatsahara-bantu-diff4-window-1556-1611.txt";

/// The window: block 1556 through 1611.
pub(crate) const WIDENING_BANTU_ESCROW_978D: (i64, i64) = (1556, 1611);

pub(crate) fn bantu_escrow_978d_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978d",
        &[(RUN978D, WIDENING_BANTU_ESCROW_978D.0)],
        WIDENING_BANTU_ESCROW_978D,
        1,
        &[],
        true,
    )
}

#[test]
fn run978d_s_escrow_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_escrow_978d_window() else {
        return;
    };
    pin_eq!(w.blocks, 55, "every captured block");
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
    // Item 3343. On the first try 32 keys: the standing set and who=1's
    // `escrow[0:food]` ours 16 against theirs 0 on block 1577 — the
    // script's citizen of 1576, queued through `Group::action_queue_up`,
    // whose `Build::queue_up(ty, 1)` hands `Type::pay_cost` the escrow
    // flag, so theirs drew the reservation down (16 − 30 → 0) where ours
    // paid it as an ordinary order. With the flag, **31**.
    pin_eq!(w.firsts.len(), 31, "run978d with the group's escrow flag");
}

/// run978b: the word 3184's window, blocks 3178..3434 at `GUYS=2` (item
/// 3343).
pub(crate) const RUN978B: &str = "gamelog-run978b-greatsahara-bantu-diff4-window-3178-3434.txt";

/// The window: block 3178 through 3434; the word 3184's own block is 3185.
pub(crate) const WIDENING_BANTU_FRAME_3184: (i64, i64) = (3178, 3434);

pub(crate) fn bantu_frame_3184_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978b",
        &[(RUN978B, WIDENING_BANTU_FRAME_3184.0)],
        WIDENING_BANTU_FRAME_3184,
        1,
        &[3185],
        true,
    )
}

#[test]
fn run978b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_frame_3184_window() else {
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
    // Item 3343. The word 3184's window (the caravan `1/15`'s guys' two
    // `Guy::move` draws): with the script's research priced against the
    // whole bucket the word left it (3184 → 4376), and **29** keys stand,
    // the standing set alone.
    pin_eq!(w.firsts.len(), 29, "run978b");
}

/// run978e: the word 4376's window, blocks 4370..4626 at `GUYS=2` (item
/// 3343).
pub(crate) const RUN978E: &str = "gamelog-run978e-greatsahara-bantu-diff4-window-4370-4626.txt";

/// The window: block 4370 through 4626; the word 4376's own block is 4377.
pub(crate) const WIDENING_BANTU_FRAME_4376: (i64, i64) = (4370, 4626);

pub(crate) fn bantu_frame_4376_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978e",
        &[(RUN978E, WIDENING_BANTU_FRAME_4376.0)],
        WIDENING_BANTU_FRAME_4376,
        1,
        &[4377],
        true,
    )
}

#[test]
fn run978e_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_frame_4376_window() else {
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
    // Item 3343. The word 4376's window (who=1's economic script asking for
    // The Art of War twice in one run, ours granting the second and walking
    // 24 → 34 → 19 into Woodcutter `1/2019`): 265 keys; with a tech in
    // research refused the word left it (4376 → 15582) and **30** stand —
    // the standing set, `group:64.held` beside it, `1/24`'s `g.angle[0]`
    // on 4467 (ours 0x55555555, theirs 0) and who=1's `SITE[1].reg` on
    // 4576.
    pin_eq!(w.firsts.len(), 30, "run978e");
}

/// run978f: the word 15582's window, blocks 15576..15832 at `GUYS=2` (item
/// 3343).
pub(crate) const RUN978F: &str = "gamelog-run978f-greatsahara-bantu-diff4-window-15576-15832.txt";

/// The window: block 15576 through 15832; the word 15582's own block is
/// 15583.
pub(crate) const WIDENING_BANTU_FRAME_15582: (i64, i64) = (15576, 15832);

pub(crate) fn bantu_frame_15582_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU.start],
        true,
        BANTU.long,
        "run978f",
        &[(RUN978F, WIDENING_BANTU_FRAME_15582.0)],
        WIDENING_BANTU_FRAME_15582,
        1,
        &[15583],
        true,
    )
}

#[test]
fn run978f_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_frame_15582_window() else {
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
    // Item 3343. The word 15582's window: **776** keys, already apart on
    // 15576 (`MAKE[2].num`, `MAKE[5].num`) and on 15579 who=1's Enlightenment
    // Age offered at 9999999 in theirs against 3564000 — `check_income`
    // 0x100 against 0x40, the age 382 food (Sugar's ten percent, `get_cost`'s
    // rares' tail) where ours priced it 425 against a purse of 388; on
    // 15583 theirs bought it. With the tail the word left for the game's end
    // (15582 → 23627) and **41** stand: the standing set, the `g.angle[0]`
    // family (ours 0x55555555, theirs 0) and two `dest_angle`s on 15624 and
    // 15653.
    pin_eq!(w.firsts.len(), 41, "run978f with the rares' tail");
}

/// The closing state's window: run972b's closing block alone, the long's
/// last frame 23627 writing it (item 3343).
pub(crate) const WIDENING_BANTU_CLOSING: (i64, i64) = (23626, 23629);

/// **The lobby's closing state** (item 3343, DECISIONS 55): run972b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
#[test]
fn run972b_bantu_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(BANTU.long.0), dump(BANTU.start), trace(BANTU.long.1))
    else {
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
        "Bantu closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 23628, "the closing block");
    pin_eq!(e.compared, 250, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}
