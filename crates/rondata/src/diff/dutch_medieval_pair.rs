//! **The Dutch at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3619): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Dutch (22)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval: `STARTING_TECHNOLOGY
//! 2`, `ENDING_TECHNOLOGY 2` — `lakota_medieval_pair`'s lobby with the
//! nation moved. Its triple, `(18, 22, 5)`, is new: the Dutch's other
//! lobbies are the AI words `dutch_word`'s `EastIndiesDutchTier4`, `(18,
//! 22, 4)` — another difficulty — and `dutch_sahara_word`'s
//! `GreatSaharaDutchTier3`, `(7, 22, 3)` — another map and difficulty —
//! and the held-out `GreatLakesDutchTier4`, `(14, 22, 4)`, another map.
//! Captures: run1272 (a `DUMP_ALL` start) and run1273 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §329; whole since item 3621, its closing state compared
//! on run1273's closing block and widened on run1273b.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const DUTCH_MEDIEVAL: Pair = Pair {
    name: "EastIndiesDutchMedieval",
    map_style: 18,
    ai_tribe: 22,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1272-eastindies-dutch-medieval-start.txt",
    long: (
        "gamelog-run1273-eastindies-dutch-medieval-trace.txt",
        "rontrace-run1273.log",
    ),
    // The game ended itself on 7095 (closing block 7096).
    length: 7095,
    // Item 3619, first try: frame 7063 of 7095 by the count and the
    // sequence. On 7063 ours spends 12 draws against theirs' 14, parting
    // at index 0 — ours `Guy::set_anim+0x97a < Unit::move_step+0x823`,
    // theirs `Unit::do_non_flat_gather+0x54b` (`docs/AI.md` section 329).
    // Item 3621: 7063 → 7095, the game's last frame — the stand-at-tile
    // arm refuses a spot equal to the unit's `avoid_x/avoid_y` (`1/146`'s
    // (33384, 35064), left by `do_move`'s gather park on 7061).
    count: 7095,
    sequence: 7095,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const DUTCH_MEDIEVAL_PAIR_WORD: i64 = if DUTCH_MEDIEVAL.count < DUTCH_MEDIEVAL.sequence {
    DUTCH_MEDIEVAL.count
} else {
    DUTCH_MEDIEVAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn dutch_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &DUTCH_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "DUTCH_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 1, and the long is scoreable.
#[test]
fn dutch_medieval_pair_captures_say_their_lobby() {
    let p = &DUTCH_MEDIEVAL;
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
        "the Dutch Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3619; the battery's identity
/// test refuses it) and none of the battery's — the held-out
/// `GreatLakesDutchTier4` is `(14, 22, 4)`, another map — nor either
/// Dutch AI word's: `EastIndiesDutchTier4` is `(18, 22, 4)`, another
/// difficulty, and `GreatSaharaDutchTier3` `(7, 22, 3)`.
#[test]
fn dutch_medieval_pair_is_no_other_lobby() {
    let p = &DUTCH_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let held_out: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.ai_tribe, l.difficulty))
        .collect();
    assert_eq!(
        held_out,
        [("GreatLakesDutchTier4", 14, 22, 4)],
        "the held-out Dutch game is another map"
    );
    let east = &super::dutch_word::DUTCH;
    assert_eq!(
        (east.name, east.map_style, east.ai_tribe, east.difficulty),
        ("EastIndiesDutchTier4", 18, 22, 4),
        "the East Indies Dutch word is another difficulty"
    );
    let sahara = &super::dutch_sahara_word::DUTCH_SAHARA;
    for (name, other) in [
        (east.name, (east.map_style, east.ai_tribe, east.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 7063** (item 3619):
/// only run1273's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_DUTCH_MEDIEVAL_STRIDE: (i64, i64) = (101, 7001);

pub(crate) fn dutch_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_MEDIEVAL.start],
        true,
        DUTCH_MEDIEVAL.long,
        "run1273 stride",
        &[(DUTCH_MEDIEVAL.long.0, WIDENING_DUTCH_MEDIEVAL_STRIDE.0)],
        WIDENING_DUTCH_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn dutch_medieval_pair_the_long_s_stride_walks_back_from_7063() {
    let _pins = Pins::hold();
    let Some(w) = dutch_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 70, "every stride block in the window");
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
    // Item 3619: 306 keys. Beside the standing families — every unit's
    // `form` (111) and `build:extra` (98), which the stride does not print,
    // and who=1's groups' `held` and pools — two value families: four of
    // who=1's Merchant Fleets (TypeIndex 318, sea; `1/142` from 4301, then
    // `1/247`, `1/156`, `1/271`) carry 192 hits in theirs against ours'
    // 172 (`hits_left`, `myhits`), and seven garrisoned Scholars (52; `1/193`
    // from 5701, `inside_up` 2008) a `g.angle[0]` of 0 in theirs against
    // ours' 1431655765. Neither spends a draw, and no unit's position or
    // orders part through block 7001.
    pin_eq!(w.firsts.len(), 306, "the stride's baseline");
}

/// run1273b: the word 7063's window, blocks 7057..7096 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 7064 through the closing block (item 3619); the long's stride
/// prints units every hundredth block (7001, then the closing 7096) and no
/// `LEADERS` past the start.
pub(crate) const RUN1273B: &str = "gamelog-run1273b-eastindies-dutch-medieval-window-7057-7096.txt";

/// The window: block 7057 through the closing block 7096; the word 7063's
/// own block is 7064.
pub(crate) const WIDENING_DUTCH_MEDIEVAL_FRAME_7063: (i64, i64) = (7057, 7096);

/// **The word 7063's window**: run1273b walked from run1272's start with
/// the recorder on (item 3619).
pub(crate) fn dutch_medieval_frame_7063_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_MEDIEVAL.start],
        true,
        DUTCH_MEDIEVAL.long,
        "run1273b",
        &[(RUN1273B, WIDENING_DUTCH_MEDIEVAL_FRAME_7063.0)],
        WIDENING_DUTCH_MEDIEVAL_FRAME_7063,
        1,
        &[7064],
        true,
    )
}

#[test]
fn dutch_medieval_pair_run1273b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_medieval_frame_7063_window() else {
        return;
    };
    pin_eq!(w.blocks, 39, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
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
    // Item 3619, the first walk: 306 keys. Block 7057 holds the standing
    // families (every unit's `form`, who=1's groups' `held` and pools,
    // who=0's `SITE[i].reg` 65 against 0 and `aggression[1]`) and the
    // stride's two value families (the Merchant Fleet `1/247`'s 172 hits
    // against 192, the Scholars' `g.angle[0]`). On the word's block 7064
    // the Citizen `1/146` (TypeIndex 50, uid 468) parts: both sides hold
    // its `GATHERORDER` at the camp `1/2074` (build type 419, tile (173,
    // 183), `wait` 1000000, `dist_mod` 10, `collide` 6 since 7054) on 7062;
    // on 7063 theirs drops the tile (`tx` −1, `goto_build`, `dist_mod` 9)
    // and spends `Unit::do_non_flat_gather+0x54b`, where ours pushes a
    // `MoveTo` (33384, 35064) — `orders.len` 2 against 1, `order:kind` 1
    // against 7. On 7064 theirs takes tile (172, 182) and `1/2074`'s list
    // swaps entries 44 and 45; ours' Citizen steps to (33411, 34906)
    // against theirs' standing (33432, 34920), `collide` 0 against 6.
    // Item 3621: 306 → 152. Ours refuses the spot that equals `1/146`'s
    // `avoid` (33384, 35064) and takes the no-spot arm; the Citizen's rows
    // and their cascade are gone, and what stands is block 7057's
    // families and two births' `form` (`1/307` on 7063, `1/308` on 7093).
    pin_eq!(w.firsts.len(), 152, "run1273b baseline");
}

/// **The closing state** (item 3621): run1273's closing block 7096, the
/// whole map's units, walked from run1272's start.
#[test]
fn dutch_medieval_pair_run1273_s_closing_state() {
    let _pins = Pins::hold();
    let p = &DUTCH_MEDIEVAL;
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
        "Dutch Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    // Item 3621, the pair whole: 321 units, none off, torn or unlinked,
    // no extra, `Closing fields: none`.
    pin_eq!(e.frame, 7096, "the closing block");
    pin_eq!(e.compared, 321, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
