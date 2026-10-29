//! **The Turks at a held Information age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, 102, item 3772): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **Turks (8)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple,
//! `(18, 8, 4)`, is new (`tools/refill.py 18 8 4` reads free): the Turks
//! hold Great Lakes at 4 (`turkish_word`) and Great Sahara at 3
//! (`turkish_enlightenment_pair`, and the held-out
//! `GreatSaharaTurkishTier3`).
//! Captures: run1445 (a `DUMP_ALL` start) and run1446 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §378.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const TURKISH_INFORMATION: Pair = Pair {
    name: "EastIndiesTurkishInformation",
    map_style: 18,
    ai_tribe: 8,
    difficulty: 4,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1445-eastindies-turkish-information-start.txt",
    long: (
        "gamelog-run1446-eastindies-turkish-information-trace.txt",
        "rontrace-run1446.log",
    ),
    // The game ended itself on 5036 (closing block 5037).
    length: 5036,
    // Item 3772, first try: frame 2551 by the count and the sequence, ours
    // 33 draws against 32, index 2 — ours `Unit::do_move+0xe84`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`: the Attack Sub `1/78`
    // (type 339, sea), born on 2550 at dock `1/2020`, came out south in ours
    // onto the idle Advanced Battleship `1/51` (type 350) and east in
    // theirs (`docs/AI.md` §378).
    // Item 3780, 2551 → 3952: `come_out`'s ring asks `FILTER_ALL`'s general
    // test for a `block_radius == 0` type, so the sub comes out east in both
    // (`docs/AI.md` §378.1). 3952: ours 35 draws against 34, index 30 — ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x97a
    // < Unit::do_guard+0x7f4`.
    count: 3952,
    sequence: 3952,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const TURKISH_INFORMATION_PAIR_WORD: i64 =
    if TURKISH_INFORMATION.count < TURKISH_INFORMATION.sequence {
        TURKISH_INFORMATION.count
    } else {
        TURKISH_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn turkish_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &TURKISH_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "TURKISH_INFORMATION {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 7 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn turkish_information_pair_captures_say_their_lobby() {
    let p = &TURKISH_INFORMATION;
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
        "the Turkish Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3772; the battery's identity
/// test refuses it), once, and none of the battery's — its one Turkish
/// game is the held-out `GreatSaharaTurkishTier3`, `(7, 8, 3)` — and no
/// other pinned Turkish lobby's: the AI word's `(14, 8, 4)`, Great Lakes,
/// and the Enlightenment pair's `(7, 8, 3)`, Great Sahara (item 3772).
#[test]
fn turkish_information_pair_is_no_other_lobby() {
    let p = &TURKISH_INFORMATION;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let turkish: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        turkish,
        [("GreatSaharaTurkishTier3", (7, 8, 3))],
        "the battery's Turkish games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Turkish lobby on this map at this difficulty"
        );
    }
    let w = &super::turkish_word::TURKISH;
    let e = &super::turkish_enlightenment_pair::TURKISH_ENLIGHTENMENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Turks");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3772, from
/// 2551): only run1446's stride blocks are on disk before the word's
/// window, 101 through 2401 — and, since item 3780 moved the word to
/// 3952, through 3901, the last stride block before it.
pub(crate) const WIDENING_TURKISH_INFORMATION_STRIDE: (i64, i64) = (101, 3901);

pub(crate) fn turkish_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[TURKISH_INFORMATION.start],
        true,
        TURKISH_INFORMATION.long,
        "run1446 stride",
        &[(
            TURKISH_INFORMATION.long.0,
            WIDENING_TURKISH_INFORMATION_STRIDE.0,
        )],
        WIDENING_TURKISH_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn turkish_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = turkish_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 39, "every stride block in the window");
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
    // Item 3772: 101 keys over 24 blocks, the standing families alone —
    // the buildings' `build:extra` (48, which the stride does not print),
    // `form` on 29 units, and who=1's groups 64..75 with their pools (12
    // each). No unit row parts.
    // Item 3780, 101 → 165 over 39 blocks, walked to 3901: the first unit
    // rows are the Tornado `1/89`'s `g.z[0]` aboard its carrier on 3101
    // (ours 0, theirs 1), and on 3901 group 70's Attack Sub `1/98`, whose
    // `ATTACKTOORDER` stands at (1944, 5976) in ours and (1512, 5640) in
    // theirs (`docs/AI.md` §378.1).
    pin_eq!(w.firsts.len(), 165, "the stride's baseline");
}

/// run1446b: the word 2551's window, blocks 2495..2801 at the long's
/// detail with `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5` — six blocks
/// before the stride's last block 2501 and 250 past the word's own block
/// 2552 (item 3772). run1446 prints units only every hundred frames, no
/// `AMMO` and no `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1446B: &str =
    "gamelog-run1446b-eastindies-turkish-information-window-2495-2801.txt";

/// The window: block 2495 through 2801.
pub(crate) const WIDENING_TURKISH_INFORMATION_FRAME_2551: (i64, i64) = (2495, 2801);

/// **The word 2551's window**: run1446b walked from run1445's start with
/// the recorder on and every round compared (item 3772).
pub(crate) fn turkish_information_frame_2551_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[TURKISH_INFORMATION.start],
        true,
        TURKISH_INFORMATION.long,
        "run1446b",
        &[(RUN1446B, WIDENING_TURKISH_INFORMATION_FRAME_2551.0)],
        WIDENING_TURKISH_INFORMATION_FRAME_2551,
        1,
        &[2552],
        true,
    )
}

#[test]
fn turkish_information_pair_run1446b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = turkish_information_frame_2551_window() else {
        return;
    };
    pin_eq!(w.blocks, 306, "every captured block");
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
    // Item 3772, the first walk: **940** keys. Before the word, the
    // standing families alone: `form` on 29 units, who=1's groups 64..75'
    // `held` and pools (12 each), who=0's `SITE[..].reg` (18) and
    // `aggression[1]`. The first unit parting is **the Attack Sub `1/78`**
    // (type 339, `ATTACKSUB`, sea), born on 2550 at who=1's dock `1/2020`
    // (44160, 43392): `pos` ours (44184, 44376), one ring step south, theirs
    // (45144, 43416), one step east. The south square is the idle Advanced
    // Battleship `1/51`'s (type 350, `ADVANCEDBATTLESHIP`, group 70, at
    // (44184, 44424) in both): theirs' `Unit::come_out` refuses it, ours'
    // does not. On 2551 group 70's attack order plans the sub's path, ours
    // 1 node against 22 (`docs/AI.md` §378; the next item's hypothesis).
    pin_eq!(w.firsts.len(), 83, "run1446b baseline");
    // Every round, both directions: 56 against 56, none parting.
    pin_eq!(w.rounds_read, (56, 56), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3772): run1446's closing block 5037,
/// the whole map's units, walked from run1445's start.
#[test]
fn turkish_information_pair_run1446_s_closing_state() {
    let _pins = Pins::hold();
    let p = &TURKISH_INFORMATION;
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
        "Turkish Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5037, "the closing block");
    // Item 3772, first try: 2485 frames past the word 2551 the game has
    // parted — 156 units compared, counts [144, 8, 7, 0, 106, 2, 32],
    // nothing torn, eight of who=1's unlinked, seven of who=1's extra. A
    // floor, not a score.
    pin_eq!(e.compared, 155, "units compared");
    pin_eq!(e.counts(), [39, 9, 2, 0, 8, 0, 5], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        vec![(1, 146), (1, 35)],
        "the closing extras"
    );
}
