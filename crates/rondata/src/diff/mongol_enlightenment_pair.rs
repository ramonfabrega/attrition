//! **The Mongols at a held Enlightenment age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, item 3673): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **Mongols (17)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its triple,
//! `(14, 17, 5)`, is new: the Mongol lobbies are the successor pair's
//! `(18, 17, 3)` and `(7, 17, 3)`, `mongol_classical_pair`'s `(7, 17, 5)`
//! and `mongol_gunpowder_pair`'s `(18, 17, 5)` — this difficulty on the
//! other two maps; no battery game fields the Mongols. Captures: run1345
//! (a `DUMP_ALL` start) and run1346 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §352.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const MONGOL_ENLIGHTENMENT: Pair = Pair {
    name: "GreatLakesMongolEnlightenment",
    map_style: 14,
    ai_tribe: 17,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1345-greatlakes-mongol-enlightenment-start.txt",
    long: (
        "gamelog-run1346-greatlakes-mongol-enlightenment-trace.txt",
        "rontrace-run1346.log",
    ),
    // The game ended itself on 4949 (closing block 4950).
    length: 4949,
    // Item 3673, first try: frame 4854, by the count and the sequence —
    // theirs 18 draws against 17, parting at index 17, theirs
    // `Unit::close+0xcb6`, ours none (`docs/AI.md` §352).
    count: 4854,
    sequence: 4854,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MONGOL_ENLIGHTENMENT_PAIR_WORD: i64 =
    if MONGOL_ENLIGHTENMENT.count < MONGOL_ENLIGHTENMENT.sequence {
        MONGOL_ENLIGHTENMENT.count
    } else {
        MONGOL_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn mongol_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MONGOL_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MONGOL_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn mongol_enlightenment_pair_captures_say_their_lobby() {
    let p = &MONGOL_ENLIGHTENMENT;
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
        "the Mongol Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3673; the battery's identity
/// test refuses it), once, and none of the battery's: no battery game
/// fields the Mongols; and no other Mongol lobby's: the successor pair's
/// `(18, 17, 3)` and `GreatSaharaMongolAllTech`'s `(7, 17, 3)`, the
/// Classical pair's `(7, 17, 5)` and the Gunpowder pair's `(18, 17, 5)`
/// (item 3673).
#[test]
fn mongol_enlightenment_pair_is_no_other_lobby() {
    let p = &MONGOL_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Mongols"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Mongol lobby on this map at this difficulty"
        );
    }
    for q in [
        &super::mongol_pair::MONGOL,
        &super::sahara_mongol::SAHARA_MONGOL,
        &super::mongol_classical_pair::MONGOL_CLASSICAL,
        &super::mongol_gunpowder_pair::MONGOL_GUNPOWDER,
    ] {
        let l = (q.map_style, q.ai_tribe, q.difficulty);
        assert_eq!(l.1, p.ai_tribe, "{} fields the Mongols", q.name);
        assert_ne!(l, triple, "{} is this lobby", q.name);
    }
}

/// **The long's stride, walked back from the word 4854** (item 3673):
/// only run1346's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_MONGOL_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 4801);

pub(crate) fn mongol_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_ENLIGHTENMENT.start],
        true,
        MONGOL_ENLIGHTENMENT.long,
        "run1346 stride",
        &[(
            MONGOL_ENLIGHTENMENT.long.0,
            WIDENING_MONGOL_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_MONGOL_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_enlightenment_pair_the_long_s_stride_walks_back_from_4854() {
    let _pins = Pins::hold();
    let Some(w) = mongol_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 48, "every stride block in the window");
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
    // Item 3673: 104 keys. The standing families — 58 buildings'
    // `build:extra` (which the stride does not print), `form` on 27 units,
    // who=1's groups' `held` and pools 64..70 as each is minted — and one
    // unit row, who=1's `1/119`: its five angle fields on 4701, ours
    // 1769734144 against theirs 1848311808.
    pin_eq!(w.firsts.len(), 104, "the stride's baseline");
}

/// run1346c: the word 4854's window, blocks 4795..4950 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4801 and on to the closing block 4950, the game's end
/// (item 3673).
pub(crate) const RUN1346C: &str =
    "gamelog-run1346c-greatlakes-mongol-enlightenment-window-4795-4950.txt";

/// The window: block 4795 through 4950; the word 4854's own block is 4855.
pub(crate) const WIDENING_MONGOL_ENLIGHTENMENT_FRAME_4854: (i64, i64) = (4795, 4950);

/// **The word 4854's window**: run1346c walked from run1345's start with
/// the recorder on and every round compared (item 3673).
pub(crate) fn mongol_enlightenment_frame_4854_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[MONGOL_ENLIGHTENMENT.start],
        true,
        MONGOL_ENLIGHTENMENT.long,
        "run1346c",
        &[(RUN1346C, WIDENING_MONGOL_ENLIGHTENMENT_FRAME_4854.0)],
        WIDENING_MONGOL_ENLIGHTENMENT_FRAME_4854,
        1,
        &[4855],
        true,
    )
}

#[test]
fn mongol_enlightenment_pair_run1346c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_enlightenment_frame_4854_window() else {
        return;
    };
    pin_eq!(w.blocks, 155, "every captured block");
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
    // Item 3673, the first walk: 754 keys. Block 4795 carries the
    // standing families (`form`, who=1's leader buckets, the queue costs of
    // who=1's buildings `1/2029` and `1/2030` — ours 99 and 114 against
    // theirs 94 and 110 — and who=1's groups and pools). **On 4850 the
    // human's Citizen `0/6` (TY 50) carries `hits:damage` 35 in ours
    // against 46 in theirs**: who=1's Golden Horde `1/97` (TI 204, piece
    // 154) looses its `CHAR_ATTACK2` round at `0/6` on 4841 from its own
    // square in ours — (6264, 28728, 480), 11 frames' flight — against
    // theirs' (6180, 28775, 715) and 10: `sim::launch` holds no piece-154
    // row. Theirs' round lands on 4850, ours' on 4851. On the word 4854
    // the Musketeer `1/21`'s round (ours from its own square, 3 frames
    // against 2) kills `0/6` in theirs — `Unit::close+0xcb6`, index 17 —
    // and ours holds it on 4855.
    pin_eq!(w.firsts.len(), 754, "run1346c baseline");
    // Every round, both directions: 294 round-blocks theirs, 275 ours, 99
    // parting, the first on the window's first block 4795 (the Cannon
    // `1/91`'s, from its own square in ours).
    pin_eq!(w.rounds_read, (294, 275), "the rounds read");
    pin_eq!(w.rounds.len(), 99, "the rounds that part");
}
