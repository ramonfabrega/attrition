//! **The Japanese at a held Modern age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, 99, item 3722): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Japanese (15)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 15, 4)`, is new: the Japanese hold this map at 5
//! (`japanese_medieval_pair`), East Indies at 4
//! (`japanese_enlightenment_pair`) and 5 (`japanese_ancient_pair`,
//! `japanese_industrial_pair`), and Great Sahara at 1 (`japanese_word`)
//! and 5 (`japanese_gunpowder_pair`); the battery fields no Japanese.
//! Captures: run1402 (a `DUMP_ALL` start) and run1403 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §369.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_MODERN: Pair = Pair {
    name: "GreatLakesJapaneseModern",
    map_style: 14,
    ai_tribe: 15,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1402-greatlakes-japanese-modern-start.txt",
    long: (
        "gamelog-run1403-greatlakes-japanese-modern-trace.txt",
        "rontrace-run1403.log",
    ),
    // The game ended itself on 3866 (closing block 3867).
    length: 3866,
    // Item 3722, first try: count 3160, sequence 3160 of 3866 — on 3160
    // ours 45 draws against theirs' 44, index 44 ours'
    // `Farms::inc_time+0x1ae` where theirs spends nothing (`docs/AI.md`
    // §369). **Item 3745: whole, 3866 of 3866** — piece 1108's five
    // `launch::BAYS` rows (the Infantry, Asian art) and piece 199's three
    // `pivot::RELEASES` rows (the Chi-Ha), off run1424's packet: `1/18`'s
    // rounds of 3157 leave (4203, 33991, 607) on both sides, and the
    // Citizen `0/4` is struck on 3158 (`docs/AI.md` §369.1).
    count: 3866,
    sequence: 3866,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_MODERN_PAIR_WORD: i64 =
    if JAPANESE_MODERN.count < JAPANESE_MODERN.sequence {
        JAPANESE_MODERN.count
    } else {
        JAPANESE_MODERN.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn japanese_modern_pair_captures_say_their_lobby() {
    let p = &JAPANESE_MODERN;
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
        "the Japanese Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3722; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Japanese — and no other Japanese lobby's: `japanese_medieval_pair`'s
/// `(14, 15, 5)`, this map at another difficulty,
/// `japanese_enlightenment_pair`'s `(18, 15, 4)`, `japanese_ancient_pair`'s
/// and `japanese_industrial_pair`'s `(18, 15, 5)`, `japanese_word`'s
/// `(7, 15, 1)` and `japanese_gunpowder_pair`'s `(7, 15, 5)`, other maps
/// (item 3722).
#[test]
fn japanese_modern_pair_is_no_other_lobby() {
    let p = &JAPANESE_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the Japanese"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Japanese lobby on this map at this difficulty"
        );
    }
    let w = &super::japanese_word::JAPANESE;
    let m = &super::japanese_medieval_pair::JAPANESE_MEDIEVAL;
    let e = &super::japanese_enlightenment_pair::JAPANESE_ENLIGHTENMENT;
    let a = &super::japanese_ancient_pair::JAPANESE_ANCIENT;
    let i = &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL;
    let g = &super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Japanese");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3722, from
/// 3160): only run1403's stride blocks are on disk before the word's
/// window, 101 through 3001.
pub(crate) const WIDENING_JAPANESE_MODERN_STRIDE: (i64, i64) = (101, 3001);

pub(crate) fn japanese_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_MODERN.start],
        true,
        JAPANESE_MODERN.long,
        "run1403 stride",
        &[(JAPANESE_MODERN.long.0, WIDENING_JAPANESE_MODERN_STRIDE.0)],
        WIDENING_JAPANESE_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = japanese_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 30, "every stride block in the window");
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
    // Item 3722: 95 keys over 30 blocks. The standing families — `form`
    // on 27 units, the buildings' `build:extra` (which the stride does not
    // print) and who=1's groups with their pools — and one unit row, the
    // Infantry `1/18`'s `dest_angle` on 1801.
    pin_eq!(w.firsts.len(), 95, "the stride's baseline");
}

/// run1403b: the word 3160's window, blocks 3095..3411 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3101 and 250 past the word's own block 3161 (item 3722).
/// run1403 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1403B: &str =
    "gamelog-run1403b-greatlakes-japanese-modern-window-3095-3411.txt";

/// The window: block 3095 through 3411.
pub(crate) const WIDENING_JAPANESE_MODERN_FRAME_3160: (i64, i64) = (3095, 3411);

/// **The word 3160's window**: run1403b walked from run1402's start with
/// the recorder on and every round compared (item 3722).
pub(crate) fn japanese_modern_frame_3160_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[JAPANESE_MODERN.start],
        true,
        JAPANESE_MODERN.long,
        "run1403b",
        &[(RUN1403B, WIDENING_JAPANESE_MODERN_FRAME_3160.0)],
        WIDENING_JAPANESE_MODERN_FRAME_3160,
        1,
        &[3161],
        true,
    )
}

#[test]
fn japanese_modern_pair_run1403b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_modern_frame_3160_window() else {
        return;
    };
    pin_eq!(w.blocks, 316, "every captured block");
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
    // Item 3722, the first walk: **850** keys. Block 3095 carries the
    // standing families (`form`, both leaders' `SITE[i].reg` and who=0's
    // `aggression[1]`, who=1's groups 64..70 and their pools). The first
    // unit parting past them is **the Citizen `0/4`** (TI 50, the farmer of
    // `0/2003`) on 3159: `hits:damage` ours 52 against theirs 59,
    // `hits_left` 33 against 26, and its city `0/2000`'s `raid_stamp` 3155
    // against 3158. Group 65's Infantry (TI 102, piece 52, which neither
    // `launch` nor `pivot` holds) attack it; `1/18`'s two rounds of 3157
    // leave the guy's square (4200, 34056, z 489) in ours and fly three
    // frames, (4203, 33991, z 607) in theirs and fly two. On 3160 theirs'
    // `0/4` reaches its flee's end and holds its `GATHERORDER` alone; ours'
    // takes a second flee to (2472, 31704) on the late wound. The same
    // frame theirs spends three `Farms::inc_time+0x1ae` chance draws
    // against ours' four (`docs/AI.md` §369).
    pin_eq!(w.firsts.len(), 59, "run1403b baseline");
    // Every round, both directions: 269 in ours against 320, 247 rows
    // parting, the first group 65's Infantry rounds from 3143 (`1/21`,
    // `1/23`, `1/28`: `sx, sy, sz`, `angle`, `v1z`), the launch point.
    pin_eq!(w.rounds_read, (269, 269), "the rounds read");
    pin_eq!(w.rounds.len(), 2, "the rounds that part");
}

/// **The pair's closing state** (item 3722): run1403's closing block 3867,
/// the whole map's units, walked from run1402's start.
#[test]
fn japanese_modern_pair_run1403_s_closing_state() {
    let _pins = Pins::hold();
    let p = &JAPANESE_MODERN;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
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
        "Japanese Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3867, "the closing block");
    // Item 3722, first try: 706 frames past the word 3160 the game has
    // parted widely — 130 units compared, counts [81, 0, 0, 8, 39, 1, 8],
    // `1/14` and `1/20` torn, nothing extra or unlinked. A floor, not a
    // score.
    pin_eq!(e.compared, 130, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
