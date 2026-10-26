//! **The Aztecs at a held Medieval age, Great Sahara** (DECISIONS 90, 91,
//! 93, 94, item 3647): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Aztecs (0)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2` — `bantu_medieval_pair`'s
//! lobby with the nation moved. Its triple, `(7, 0, 5)`, is new: the
//! battery's one Aztec game is the held-out `EastIndiesAztecTier1`,
//! `(18, 0, 1)`; the AI word `aztec_word` is `(7, 0, 2)`, the AllTech pair
//! `(7, 0, 3)` and the Ancient-start pair East Indies' `(18, 0, 5)`.
//! Captures: run1307 (a `DUMP_ALL` start) and run1308 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §338.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AZTEC_MEDIEVAL: Pair = Pair {
    name: "GreatSaharaAztecMedieval",
    map_style: 7,
    ai_tribe: 0,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1307-greatsahara-aztec-medieval-start.txt",
    long: (
        "gamelog-run1308-greatsahara-aztec-medieval-trace.txt",
        "rontrace-run1308.log",
    ),
    // The game ended itself on 4602 (closing block 4603).
    length: 4602,
    // Item 3647, first try: frame 2556, by the count and the sequence —
    // ours 185 draws against 189, parting at index 180, ours
    // `Unit::close+0xcb6`, theirs `Unit::plunder+0x4e5` (`docs/AI.md`
    // §338). Item 3657, 2556 -> 2569: `take_damage`'s Aztec slay arm
    // (`plunder(v, k, who, 1, 0)` for a Stable killer) pays who=1 45
    // timber on 2556, 12759 on block 2557 on both sides. 2569: ours 6
    // draws against 8, parting at index 1, ours `Farms::inc_time+0x1ae`,
    // theirs `Ammo::do_damage+0xc59`. Item 3658: **4602 of 4602, the pair
    // whole** — the Heavy Horse Archer's piece 842 reads piece 138's pivot
    // rows (`sim::pivot::SAME_AS`), so `1/22`'s `CHAR_ATTACKWALK` round of
    // 2556 leaves (7020, 31282, 873) and comes down on 2569, its target
    // gone, through `do_damage`'s puncture pair on both sides.
    count: 4602,
    sequence: 4602,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AZTEC_MEDIEVAL_PAIR_WORD: i64 = if AZTEC_MEDIEVAL.count < AZTEC_MEDIEVAL.sequence {
    AZTEC_MEDIEVAL.count
} else {
    AZTEC_MEDIEVAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn aztec_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AZTEC_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AZTEC_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2, and the long is scoreable.
#[test]
fn aztec_medieval_pair_captures_say_their_lobby() {
    let p = &AZTEC_MEDIEVAL;
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
        "the Aztec Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3647; the battery's identity
/// test refuses it) and none of the battery's: its one Aztec game is the
/// held-out `EastIndiesAztecTier1`, `(18, 0, 1)`; and no other pinned
/// Aztec lobby's: `aztec_word`'s `(7, 0, 2)`, the AllTech pair's
/// `(7, 0, 3)` and the Ancient-start pair's `(18, 0, 5)` (item 3647).
#[test]
fn aztec_medieval_pair_is_no_other_lobby() {
    let p = &AZTEC_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let aztecs: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        aztecs,
        [("EastIndiesAztecTier1", (18, 0, 1))],
        "the battery's Aztec games"
    );
    let word = &super::aztec_word::AZTEC;
    let alltech = &super::aztec_alltech_pair::AZTEC_ALLTECH;
    let ancient = &super::aztec_ancient_pair::AZTEC_ANCIENT;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            alltech.name,
            (alltech.map_style, alltech.ai_tribe, alltech.difficulty),
        ),
        (
            ancient.name,
            (ancient.map_style, ancient.ai_tribe, ancient.difficulty),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Aztecs");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 2556** (item 3647):
/// only run1308's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_AZTEC_MEDIEVAL_STRIDE: (i64, i64) = (101, 2501);

pub(crate) fn aztec_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_MEDIEVAL.start],
        true,
        AZTEC_MEDIEVAL.long,
        "run1308 stride",
        &[(AZTEC_MEDIEVAL.long.0, WIDENING_AZTEC_MEDIEVAL_STRIDE.0)],
        WIDENING_AZTEC_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn aztec_medieval_pair_the_long_s_stride_walks_back_from_2556() {
    let _pins = Pins::hold();
    let Some(w) = aztec_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 25, "every stride block in the window");
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
    // Item 3647: 81 keys, every one standing: 44 buildings' `build:extra`
    // (which the stride does not print), `form` on 27 units, and who=1's
    // five groups' `held` and pools as each is minted (65 on 101 through
    // 68 on 1901). No unit's position, orders or hits part through block
    // 2501.
    pin_eq!(w.firsts.len(), 81, "the stride's baseline");
}

/// run1308b: the word 2556's window, blocks 2550..2806 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 2557 and 250 after (item 3647); the stride's blocks are 2501 and
/// 2601.
pub(crate) const RUN1308B: &str =
    "gamelog-run1308b-greatsahara-aztec-medieval-window-2550-2806.txt";

/// The window: block 2550 through 2806; the word 2556's own block is 2557.
pub(crate) const WIDENING_AZTEC_MEDIEVAL_FRAME_2556: (i64, i64) = (2550, 2806);

/// **The word 2556's window**: run1308b walked from run1307's start with
/// the recorder on and every round compared (item 3647).
pub(crate) fn aztec_medieval_frame_2556_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AZTEC_MEDIEVAL.start],
        true,
        AZTEC_MEDIEVAL.long,
        "run1308b",
        &[(RUN1308B, WIDENING_AZTEC_MEDIEVAL_FRAME_2556.0)],
        WIDENING_AZTEC_MEDIEVAL_FRAME_2556,
        1,
        &[2557],
        true,
    )
}

#[test]
fn aztec_medieval_pair_run1308b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_medieval_frame_2556_window() else {
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
    // Item 3647, the first walk: 2414 keys. Block 2550 carries the standing
    // families (`form`, `SITE[i].reg`, who=0's `aggression[1]`, who=1's
    // groups' `held` and pools). **The state first parts on block 2557,
    // the word's own, in one field**: who=1's `bucket[1:timber]`, ours
    // 12714 against theirs 12759. On 2556 both sides kill the human's
    // Citizen `0/13` (TypeIndex 50) with the round of who=1's Heavy Horse
    // Archer `1/29` (TY 138, TypeIndex 188, stance Raid) and close it;
    // theirs first runs `Unit::plunder` for the Aztec killer — four draws,
    // one a good available, and +45 timber — and ours does not
    // (`Sim::unit_plunder`'s `SEAM`: the Aztec slay bonus is not carried).
    // The draw on 2556: ours 185 against 189, index 180, ours
    // `Unit::close+0xcb6`, theirs `Unit::plunder+0x4e5` (`docs/AI.md`
    // §338). Positions part from 2563 (the Citizen `1/1`'s `move.pause`
    // on 2558 first, then group 64's Heavy Horse Archer `1/22`).
    // Item 3657, 2414 -> 69: the slay arm pays the 45 timber and nothing
    // else parts on 2557. 48 of the 69 are block 2550's standing
    // families; the first rows past them are the human's building
    // `0/2000` on 2585 (`build:damage` ours 2 theirs 4, `city:raid_stamp`
    // ours 2566 theirs 2584), group 66's pool on 2586, and `0/3`'s
    // wound on 2721 (`hits:damage` 17 against 35).
    // Item 3658, 69 -> 52: piece 842 releases through 138's pivot, and
    // every row past block 2550's standing families leaves but group 66's
    // pool on 2586 and group 70's on 2655 — the standing pool family as
    // each group is minted. `0/2000`'s `build:damage` and
    // `city:raid_stamp` on 2585 agree.
    pin_eq!(w.firsts.len(), 52, "run1308b baseline");
    // Every round, both directions: 163 round-blocks theirs, 141 ours. The
    // first to part is `1/29`'s, on the window's first block, at the
    // Citizen `0/13`: ours leaves from (7599, 31648, 659), theirs from
    // (7556, 31716, 901), the same flight time — it kills `0/13` on 2556
    // on both sides. `1/22`'s round launched on 2556 parts the same way.
    // Item 3657: ours reads 167 (141 before), 36 parting (49). Item 3658:
    // 163 a side, 20 parting. On 2556 `1/22`'s round leaves (7020, 31282)
    // on both sides (ours left its own square, (7018, 31206)). What parts:
    // `whom`/`ox` on 2557, which theirs keeps on a dead target and ours
    // forgets; the slot shifts from 2683 on, theirs' spent arrows holding
    // their slots (`docs/COMBAT.md` §46.7); and three launches one unit
    // off, `1/29`'s on 2601 and 2651 and `1/22`'s on 2631 (3658a).
    pin_eq!(w.rounds_read, (163, 163), "the rounds read");
    pin_eq!(w.rounds.len(), 20, "the rounds that part");
}

/// run1322: the closing window, blocks 4347..4603 at run1308b's detail,
/// `cover=0` (item 3658) — 256 blocks through the closing block, so the
/// word 4602, the game's last frame, sits inside a window.
pub(crate) const RUN1322: &str = "gamelog-run1322-greatsahara-aztec-medieval-window-4347-4603.txt";

/// The window: block 4347 through the closing block 4603.
pub(crate) const WIDENING_AZTEC_MEDIEVAL_FRAME_4602: (i64, i64) = (4347, 4603);

/// **The closing window**: run1322 walked from run1307's start with the
/// recorder on, the rounds compared (item 3658).
pub(crate) fn aztec_medieval_frame_4602_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AZTEC_MEDIEVAL.start],
        true,
        AZTEC_MEDIEVAL.long,
        "run1322",
        &[(RUN1322, WIDENING_AZTEC_MEDIEVAL_FRAME_4602.0)],
        WIDENING_AZTEC_MEDIEVAL_FRAME_4602,
        1,
        &[4603],
        true,
    )
}

#[test]
fn aztec_medieval_pair_run1322_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_medieval_frame_4602_window() else {
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
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((_, o, _), _)| *o >= 0)
            .filter(|((_, _, k), _)| k != "form")
            .map(|((who, o, k), (f, _))| format!("{f} {who}/{o} {k}"))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        [
            "4376 1/89 orders_x",
            "4376 1/89 orders_y",
            "4494 1/69 orders_x",
            "4494 1/69 orders_y",
            "4496 1/118 dest_angle",
        ],
        "the object rows past `form`"
    );
    // Item 3658, the pair whole: 57 keys. Past the standing families —
    // `form`, both leaders' `SITE[i].reg`, who=0's `aggression[1]`, who=1's
    // groups' `held` and pools — three guards' bookkeeping (3658c): the
    // Heavy Archers `1/89` (TypeIndex 173) and `1/118` and the Xopilli
    // Atl-Atl `1/69` (87) hold `orders_x/y` at their guard point where
    // theirs print the unit's own, and `1/118`'s `dest_angle` parts on
    // 4496. No position, hit or draw parts through the closing block.
    pin_eq!(w.firsts.len(), 57, "run1322 baseline");
    pin_eq!(w.rounds_read, (362, 362), "the rounds read");
    // Every round, both directions: 362 a side, 38 parting — the slot
    // shifts from theirs' spent arrows (`docs/COMBAT.md` §46.7), the Heavy
    // Horse Archers' one-unit launches (3658a), and the Trebuchets'
    // (TypeIndex 266, `1/56`, `1/79`, `1/107`, `1/124`) from 4553 on, which
    // leave their own square in ours and about 300 out and 693 up in
    // theirs (3658b); their landings and flights agree.
    pin_eq!(w.rounds.len(), 38, "the rounds that part");
}

/// **The closing state** (item 3658): run1308's closing block 4603, the
/// whole map's units, walked from run1307's start.
#[test]
fn aztec_medieval_pair_run1308_s_closing_state() {
    let _pins = Pins::hold();
    let p = &AZTEC_MEDIEVAL;
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
        "Aztec Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    for &(who, o) in &e.extra {
        if let Some(u) = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
        {
            let name = u.ty.and_then(|ty| loaded.unit_names.get(ty).cloned());
            eprintln!("    extra {who}/{o} {name:?} at {:?}", u.pos);
        }
    }
    pin_eq!(e.frame, 4603, "the closing block");
    // Item 3658, the pair whole: 211 units, none off, torn, unlinked or
    // extra, `Closing fields: none`.
    pin_eq!(e.compared, 211, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
