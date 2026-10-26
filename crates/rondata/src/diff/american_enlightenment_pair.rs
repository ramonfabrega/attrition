//! **The Americans at a held Enlightenment age, East Indies** (DECISIONS
//! 90, 91, 93, 94, item 3650): East Indies, `MAP_STYLE 18`, human Nubians
//! (4) against the **Americans (20)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4` —
//! `indian_enlightenment_pair`'s lobby with the nation moved. Its triple,
//! `(18, 20, 5)`, is new: the battery fields no Americans (its American
//! game left it for the AI word `american_word`'s `(18, 20, 1)`), and the
//! other American lobbies are `american_lakes_word`'s `(14, 20, 4)` and
//! `american_modern_pair`'s `(7, 20, 3)`. Captures: run1313 (a `DUMP_ALL`
//! start) and run1314 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §341.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesAmericanEnlightenment",
    map_style: 18,
    ai_tribe: 20,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1313-eastindies-american-enlightenment-start.txt",
    long: (
        "gamelog-run1314-eastindies-american-enlightenment-trace.txt",
        "rontrace-run1314.log",
    ),
    // The game ended itself on 6669 (closing block 6670).
    length: 6669,
    // Item 3650, first try: frame 3215 — ours 15 draws against 16 at
    // index 9 (`docs/AI.md` §341). Item 3661, 3215 → 5583: the marines
    // dig in (`check_idle`'s trench), and their boat takes its hits and
    // speed; on 5583 ours spends 11 draws against 12, parting at index 6,
    // ours `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`.
    count: 5583,
    sequence: 5583,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AMERICAN_ENLIGHTENMENT_PAIR_WORD: i64 =
    if AMERICAN_ENLIGHTENMENT.count < AMERICAN_ENLIGHTENMENT.sequence {
        AMERICAN_ENLIGHTENMENT.count
    } else {
        AMERICAN_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn american_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AMERICAN_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4, and the long is scoreable.
#[test]
fn american_enlightenment_pair_captures_say_their_lobby() {
    let p = &AMERICAN_ENLIGHTENMENT;
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
        "the American Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3650; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no
/// Americans; and no other pinned American lobby's: `american_word`'s
/// `(18, 20, 1)`, the same map at another difficulty,
/// `american_lakes_word`'s `(14, 20, 4)` and `american_modern_pair`'s
/// `(7, 20, 3)` (item 3650).
#[test]
fn american_enlightenment_pair_is_no_other_lobby() {
    let p = &AMERICAN_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Americans"
    );
    let word = &super::american_word::AMERICAN;
    let lakes = &super::american_lakes_word::AMERICAN_LAKES;
    let modern = &super::american_modern_pair::AMERICAN_MODERN;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            lakes.name,
            (lakes.map_style, lakes.ai_tribe, lakes.difficulty),
        ),
        (
            modern.name,
            (modern.map_style, modern.ai_tribe, modern.difficulty),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Americans");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3215** (item 3650):
/// only run1314's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_AMERICAN_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 3201);

pub(crate) fn american_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_ENLIGHTENMENT.start],
        true,
        AMERICAN_ENLIGHTENMENT.long,
        "run1314 stride",
        &[(
            AMERICAN_ENLIGHTENMENT.long.0,
            WIDENING_AMERICAN_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_AMERICAN_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn american_enlightenment_pair_the_long_s_stride_walks_back_from_3215() {
    let _pins = Pins::hold();
    let Some(w) = american_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 32, "every stride block in the window");
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
    // Item 3650: 135 keys. The standing families: 51 buildings'
    // `build:extra` (which the stride does not print), `form` on 33 units,
    // who=1's groups' `held` and pools as each is minted, and the
    // garrisoned Scholar `1/14`'s (TI 52) figure `angle` from 401, ours
    // `0x55555555` against 0. **The first unit row is on block 3201,
    // who=1's Transport Galleon `1/105`** (TY 271, TI 321, sea), born on
    // 3183 with the Marines `1/29` (TY 69, TI 119) inside: `myhits` and
    // `hits_left` ours 90 theirs 160, `myspeed` 25 against 31, `pos`
    // (35137, 43021) against (34971, 43021) (`docs/AI.md` §341).
    pin_eq!(w.firsts.len(), 127, "the stride's baseline");
}

/// run1314b: the word 3215's window, blocks 3209..3465 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 3216 and 250 after (item 3650); the stride's blocks are 3201 and
/// 3301.
pub(crate) const RUN1314B: &str =
    "gamelog-run1314b-eastindies-american-enlightenment-window-3209-3465.txt";

/// The window: block 3209 through 3465; the word 3215's own block is 3216.
pub(crate) const WIDENING_AMERICAN_ENLIGHTENMENT_FRAME_3215: (i64, i64) = (3209, 3465);

/// **The word 3215's window**: run1314b walked from run1313's start with
/// the recorder on and every round compared (item 3650).
pub(crate) fn american_enlightenment_frame_3215_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_ENLIGHTENMENT.start],
        true,
        AMERICAN_ENLIGHTENMENT.long,
        "run1314b",
        &[(RUN1314B, WIDENING_AMERICAN_ENLIGHTENMENT_FRAME_3215.0)],
        WIDENING_AMERICAN_ENLIGHTENMENT_FRAME_3215,
        1,
        &[3216],
        true,
    )
}

#[test]
fn american_enlightenment_pair_run1314b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_enlightenment_frame_3215_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3650, the first walk: 1911 keys. Block 3209 carries the
    // standing families, the Scholar `1/14`'s figure `angle`, and **who=1's
    // Transport Galleon `1/105`** (TI 321) carrying the Marines `1/29`
    // (TI 119): `myhits` ours 90 theirs 160 and `myspeed` 25 against 31
    // since its birth on 3183, so theirs sails ahead — `pos` (34937,
    // 43029) against (34723, 43029). On 3214 theirs has reached its
    // path's next waypoint (`path:length` ours 11 theirs 10) and on the
    // word 3215 the move's `dest_x`/`dest_y` part, (34584, 43032)
    // against (34536, 43080): the draw on 3215, ours 15 against 16, index
    // 9, ours `Guy::set_anim+0x104b`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` (`docs/AI.md` §341).
    pin_eq!(w.firsts.len(), 102, "run1314b baseline");
    // Every round, both directions: 9 round-blocks theirs, 18 ours, 3
    // parting — all the Sloop `1/61`'s (TI 327) at the human's `0/2005`:
    // its round's end point on 3240, `ex`/`ey` (4773, 7451) against
    // (4799, 7405), and on 3367 a round this crate holds alone.
    pin_eq!(w.rounds_read, (9, 9), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1325: blocks 3178..3434 with the figures' clocks (`GUYS=4`), taken by
/// item 3661 because no dump on disk printed a figure's `cur_anim` or
/// `cur_time` before the word 3215: theirs' extra draw on 3215 is the
/// Marines `1/55`'s (TI 119) seventy-frame idle wrapping, and the record
/// that says why is `unit_masks` `0x2000000` with `unit_masks2` `0x20800` —
/// the marines in their own trench (`docs/AI.md` §341).
pub(crate) const RUN1325: &str =
    "gamelog-run1325-eastindies-american-enlightenment-window-3178-3434.txt";

/// The window: block 3178 through 3434, six before 1/47's arrival block
/// 3184 and 250 after it.
pub(crate) const WIDENING_AMERICAN_ENLIGHTENMENT_CLOCKS: (i64, i64) = (3178, 3434);

/// **The figures' clocks over the old word**: run1325 walked from run1313's
/// start, every round compared (item 3661).
pub(crate) fn american_enlightenment_clocks_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_ENLIGHTENMENT.start],
        true,
        AMERICAN_ENLIGHTENMENT.long,
        "run1325",
        &[(RUN1325, WIDENING_AMERICAN_ENLIGHTENMENT_CLOCKS.0)],
        WIDENING_AMERICAN_ENLIGHTENMENT_CLOCKS,
        1,
        &[3216],
        true,
    )
}

#[test]
fn american_enlightenment_pair_run1325_s_clocks_are_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_enlightenment_clocks_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3661, with the marines' trench and their boat built: 124 keys,
    // the standing families (`SITE[*].reg`, `form`, who=1's groups and
    // pools) and what follows the old word; unread by the draft — the
    // next session's first job (journal 2026-10-26, item 3661).
    pin_eq!(w.firsts.len(), 124, "run1325 baseline");
    pin_eq!(w.rounds_read, (9, 9), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}
