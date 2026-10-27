//! **The French at a held Enlightenment age, East Indies, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97, item 3672): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **French (10)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its triple,
//! `(18, 10, 4)`, is new: the French lobbies are the third pair's
//! `(14, 10, 5)` and `(18, 10, 5)` — this map at another difficulty —
//! `french_classical_pair`'s `(7, 10, 5)` and `french_word`'s
//! `GreatSaharaFrenchTier4`, `(7, 10, 4)`, which left the battery for the
//! AI word; no battery game fields the French. Captures: run1343 (a
//! `DUMP_ALL` start) and run1344 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §351.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const FRENCH_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesFrenchEnlightenment",
    map_style: 18,
    ai_tribe: 10,
    difficulty: 4,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1343-eastindies-french-enlightenment-start.txt",
    long: (
        "gamelog-run1344-eastindies-french-enlightenment-trace.txt",
        "rontrace-run1344.log",
    ),
    // The game ended itself on 4940 (closing block 4941).
    length: 4940,
    // Item 3672, first try: frame 4505, by the count and the sequence —
    // ours 154 draws against 152, parting at index 151, ours
    // `Ammo::do_damage+0xc59`, theirs `Farms::inc_time+0x1ae`
    // (`docs/AI.md` §351). Item 3691 closes it, 4505 → 4940, the game's
    // last frame: piece 49's three release bays (`sim::launch`), so
    // who=1's Musketeer `1/37` looses its `CHAR_ATTACK2` round of 4499
    // from (6152, 9334, 303), the dump's, flying 4 frames, where it left
    // from the figure's own (6072, 9480, 127) and flew 5.
    count: 4940,
    sequence: 4940,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const FRENCH_ENLIGHTENMENT_PAIR_WORD: i64 =
    if FRENCH_ENLIGHTENMENT.count < FRENCH_ENLIGHTENMENT.sequence {
        FRENCH_ENLIGHTENMENT.count
    } else {
        FRENCH_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn french_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &FRENCH_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "FRENCH_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
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
fn french_enlightenment_pair_captures_say_their_lobby() {
    let p = &FRENCH_ENLIGHTENMENT;
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
        "the French Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3672; the battery's identity
/// test refuses it), once, and none of the battery's: no battery game
/// fields the French; and no other French lobby's: the third pair's
/// `(14, 10, 5)` and `(18, 10, 5)`, the Classical pair's `(7, 10, 5)` and
/// the AI word `GreatSaharaFrenchTier4`'s `(7, 10, 4)` (item 3672).
#[test]
fn french_enlightenment_pair_is_no_other_lobby() {
    let p = &FRENCH_ENLIGHTENMENT;
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
        "the battery fields the French"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another French lobby on this map at this difficulty"
        );
    }
    let w = &super::french_word::FRENCH;
    let c = &super::french_classical_pair::FRENCH_CLASSICAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        ("the third pair, East Indies", (18, 10, 5)),
        ("the third pair, Great Lakes", (14, 10, 5)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the French");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 4505** (item 3672):
/// only run1344's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_FRENCH_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 4501);

pub(crate) fn french_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH_ENLIGHTENMENT.start],
        true,
        FRENCH_ENLIGHTENMENT.long,
        "run1344 stride",
        &[(
            FRENCH_ENLIGHTENMENT.long.0,
            WIDENING_FRENCH_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_FRENCH_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn french_enlightenment_pair_the_long_s_stride_walks_back_from_4505() {
    let _pins = Pins::hold();
    let Some(w) = french_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 45, "every stride block in the window");
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
    // Item 3672: 171 keys. The standing families — 57 buildings'
    // `build:extra` (which the stride does not print), `form` on 56 units,
    // who=1's groups' `held` and pools 64..91 as each is minted — and two
    // unit rows, both who=1's Fusiliers (TY 136): `1/59`'s `dest_angle`
    // on 3901 (its guard order re-aimed) and **`1/48`'s `order:target`
    // on 4401, ours `0/4` against theirs `0/12`**, an army's attack.
    pin_eq!(w.firsts.len(), 171, "the stride's baseline");
}

/// **The closing state** (item 3691): run1344's closing block 4941, the
/// whole map's units, walked from run1343's start.
#[test]
fn french_enlightenment_pair_run1344_s_closing_state() {
    let _pins = Pins::hold();
    let p = &FRENCH_ENLIGHTENMENT;
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
        "French Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    for b in &fin.builds {
        if (0..2).contains(&b.who) && harness::link_building(&built.sim, b.who, b.o).is_none() {
            eprintln!(
                "    unlinked building {}/{} at {:?} type {:?}",
                b.who, b.o, b.pos, b.orig_type
            );
            for x in built
                .sim
                .buildings
                .iter()
                .filter(|x| i64::from(x.pos.x) == b.pos.x && i64::from(x.pos.y) == b.pos.y)
            {
                eprintln!(
                    "      ours {}/{} alive {} hits {} health {} ty {:?}",
                    x.owner, x.index, x.alive, x.hits, x.health, x.ty
                );
            }
        }
    }
    pin_eq!(e.frame, 4941, "the closing block");
    // Item 3691, the pair whole: 195 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 4940.
    pin_eq!(e.compared, 195, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1344b: the word 4505's window, blocks 4495..4756 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4501 and 250 after the word's block 4506 (item 3672).
pub(crate) const RUN1344B: &str =
    "gamelog-run1344b-eastindies-french-enlightenment-window-4495-4756.txt";

/// The window: block 4495 through 4756; the word 4505's own block is 4506.
pub(crate) const WIDENING_FRENCH_ENLIGHTENMENT_FRAME_4505: (i64, i64) = (4495, 4756);

/// **The word 4505's window**: run1344b walked from run1343's start with
/// the recorder on and every round compared (item 3672).
pub(crate) fn french_enlightenment_frame_4505_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[FRENCH_ENLIGHTENMENT.start],
        true,
        FRENCH_ENLIGHTENMENT.long,
        "run1344b",
        &[(RUN1344B, WIDENING_FRENCH_ENLIGHTENMENT_FRAME_4505.0)],
        WIDENING_FRENCH_ENLIGHTENMENT_FRAME_4505,
        1,
        &[4506],
        true,
    )
}

#[test]
fn french_enlightenment_pair_run1344b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_enlightenment_frame_4505_window() else {
        return;
    };
    pin_eq!(w.blocks, 261, "every captured block");
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
    // Item 3672, the first walk: 2344 keys. Block 4495 carries the
    // standing families (`form`, both leaders' `SITE[i].reg`, the human's
    // `aggression[1]`, who=1's groups and pools) and the Fusilier `1/48`'s
    // attack target the stride parts from 4401. **The rounds part from the
    // window's first block** by a pixel of launch, and on 4499 **who=1's
    // Musketeer `1/37` (TY 99) fires its `CHAR_ATTACK2` round at `0/10`
    // from its own square in ours** — (6072, 9480, 127), 5 frames' flight
    // — against theirs' (6152, 9334, 303) and 4: `sim::launch` has no
    // piece-49 `ATTACK2` row. Theirs' round retires on 4504; ours' resolves
    // on the word 4505 beside `1/45`'s, ours' extra `Ammo::do_damage`
    // pair at index 151. On 4504 the Citizen `0/11` carries theirs' 14
    // from `1/36` on 4503, ours none.
    //
    // Item 3691, piece 49's bays: 2344 → 122 keys. `1/37`'s round of 4499
    // agrees in `sx, sy, sz` and `total_time` (6152, 9334, 303 and 4, both
    // sides), and nothing parts at the word 4505. What stands is block
    // 4495's families, `1/48`'s target, group 77's `held` on 4518 and the
    // Horse Guard Grenadier `1/29`'s `damage_frac` on 4552 (TY 238; ours 5,
    // theirs 0).
    pin_eq!(w.firsts.len(), 122, "run1344b baseline");
    // Every round, both directions: 421 round-blocks theirs, 446 ours, 255
    // parting, the first on the window's first block 4495. Item 3691: 428
    // ours, 139 parting — 129 rows on six Fusiliers' (TY 136) rounds;
    // the Musketeers' `ox`/`whom` (ours −1) on three rounds; and two slot
    // swaps on 4724–4725.
    pin_eq!(w.rounds_read, (421, 428), "the rounds read");
    pin_eq!(w.rounds.len(), 139, "the rounds that part");
}
