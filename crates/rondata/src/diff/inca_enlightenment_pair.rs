//! **The Inca at a held Enlightenment age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, item 3648): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Inca (2)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4` —
//! `turkish_enlightenment_pair`'s map and age with the nation moved and
//! the difficulty at 5. Its triple, `(7, 2, 5)`, is new: the battery
//! fields no Inca; the AI words `inca_word` and `inca_sahara_word` are
//! `(18, 2, 3)` and `(7, 2, 3)`, and the Classical pair East Indies'
//! `(18, 2, 5)`. Captures: run1309 (a `DUMP_ALL` start) and run1310 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §339.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const INCA_ENLIGHTENMENT: Pair = Pair {
    name: "GreatSaharaIncaEnlightenment",
    map_style: 7,
    ai_tribe: 2,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1309-greatsahara-inca-enlightenment-start.txt",
    long: (
        "gamelog-run1310-greatsahara-inca-enlightenment-trace.txt",
        "rontrace-run1310.log",
    ),
    // The game ended itself on 4045 (closing block 4046).
    length: 4045,
    // Item 3648, first try: frame 3095, by the count and the sequence —
    // ours 13 draws against 12, parting at index 9, ours
    // `Unit::close+0xcb6`, theirs `Farms::inc_time+0x1ae` (`docs/AI.md`
    // §339). Item 3659: 4045 of 4045, the pair whole — piece 753's bays
    // (`sim::launch`): the Inca Musketeers (TypeIndex 99,
    // `MUSKETEERS-AMERICAN`) left from the figure's own point, so a round
    // flew a frame long and the Citizen `0/5` was first struck on 3054
    // where theirs struck it on 3053.
    count: 4045,
    sequence: 4045,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INCA_ENLIGHTENMENT_PAIR_WORD: i64 =
    if INCA_ENLIGHTENMENT.count < INCA_ENLIGHTENMENT.sequence {
        INCA_ENLIGHTENMENT.count
    } else {
        INCA_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn inca_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &INCA_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "INCA_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
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
fn inca_enlightenment_pair_captures_say_their_lobby() {
    let p = &INCA_ENLIGHTENMENT;
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
        "the Inca Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3648; the battery's identity
/// test refuses it) and none of the battery's, which fields no Inca; and
/// no other pinned Inca lobby's: `inca_word`'s `(18, 2, 3)`,
/// `inca_sahara_word`'s `(7, 2, 3)` and the Classical pair's `(18, 2, 5)`
/// (item 3648).
#[test]
fn inca_enlightenment_pair_is_no_other_lobby() {
    let p = &INCA_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Inca"
    );
    let word = &super::inca_word::INCA;
    let sahara = &super::inca_sahara_word::INCA_SAHARA;
    let classical = &super::inca_classical_pair::INCA_CLASSICAL;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
        (
            classical.name,
            (
                classical.map_style,
                classical.ai_tribe,
                classical.difficulty,
            ),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Inca");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3095** (item 3648):
/// only run1310's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 3001);

pub(crate) fn inca_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1310 stride",
        &[(
            INCA_ENLIGHTENMENT.long.0,
            WIDENING_INCA_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_INCA_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_the_long_s_stride_walks_back_from_3095() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_stride_window() else {
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
    // Item 3648: 84 keys, every one standing: 47 buildings' `build:extra`
    // (which the stride does not print), `form` on 27 units, and who=1's
    // five groups' `held` and pools as each is minted (65 on 101 through
    // 68 on 2101). No unit's position, orders or hits part through block
    // 3001.
    pin_eq!(w.firsts.len(), 84, "the stride's baseline");
}

/// run1310b: the word 3095's window, blocks 3089..3345 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 3096 and 250 after (item 3648); the stride's blocks are 3001 and
/// 3101.
pub(crate) const RUN1310B: &str =
    "gamelog-run1310b-greatsahara-inca-enlightenment-window-3089-3345.txt";

/// The window: block 3089 through 3345; the word 3095's own block is 3096.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_FRAME_3095: (i64, i64) = (3089, 3345);

/// **The word 3095's window**: run1310b walked from run1309's start with
/// the recorder on and every round compared (item 3648).
pub(crate) fn inca_enlightenment_frame_3095_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1310b",
        &[(RUN1310B, WIDENING_INCA_ENLIGHTENMENT_FRAME_3095.0)],
        WIDENING_INCA_ENLIGHTENMENT_FRAME_3095,
        1,
        &[3096],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_run1310b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_frame_3095_window() else {
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
    // Item 3648, the first walk: 1205 keys. **The state parts in the
    // undumped gap 3002..3088**, and block 3089 carries it beside the
    // standing families (`form`, `SITE[i].reg`, who=0's `aggression[1]`,
    // who=1's groups' `held` and pools): who=1's Musketeers `1/36` (TY 49,
    // TypeIndex 99, stance Raid) holds an attack on the human's Citizen
    // `0/5` (TypeIndex 50) in ours and on the Citizen `0/12` in theirs —
    // `RON_DEBUG_UNIT` has ours take `0/5` on 3085 — and `0/5`'s
    // `damage_frame` is 3054 against 3053, its position (7536, 29352)
    // against (7525, 29330). On block 3095 theirs' `0/12` is struck
    // (`hits_left` 12 ours, 8 theirs); on the word 3095 ours kills `0/5`
    // (ours `Unit::close+0xcb6`: its damage 72 against theirs' 58, of 65)
    // where theirs' `0/5` lives, and theirs draws `Farms::inc_time+0x1ae`
    // first: ours 13 draws against 12, index 9 (`docs/AI.md` §339). On
    // 3096 who=0's `num_units[0]` is 11 against 12.
    // Item 3659: **50**, every one standing — who=1's groups' `held` and
    // pools, `form`, the leaders' `SITE[i].reg` and who=0's
    // `aggression[1]`. With piece 753's bays `0/5` is struck on 3053 and
    // `1/36` holds `0/12` on 3089 on both sides; no unit's field parts.
    pin_eq!(w.firsts.len(), 50, "run1310b baseline");
    // Every round, both directions: 104 round-blocks theirs, 138 ours. The
    // first to part is `1/36`'s, on 3092: ours at `0/5`, theirs at `0/12`.
    // Item 3659: 104 a side; four keys part, on 3239: `1/37`'s and
    // `1/38`'s rounds in flight at the Citizen `0/3`, which leaves the
    // dump on 3239 — ours clears `whom`/`ox` to −1, theirs keeps 0/3
    // (parked 3659b).
    pin_eq!(w.rounds_read, (104, 104), "the rounds read");
    pin_eq!(w.rounds.len(), 4, "the rounds that part");
}

/// run1323: the gap window over the undumped 3002..3088, blocks 3001..3094
/// at run1310b's detail — from the stride's last dumped block 3001 to the
/// block before run1310b's first (item 3659).
pub(crate) const RUN1323: &str =
    "gamelog-run1323-greatsahara-inca-enlightenment-window-3001-3094.txt";

/// The gap window: block 3001 through 3094.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_GAP: (i64, i64) = (3001, 3094);

/// **The gap 3002..3088**: run1323 walked from run1309's start with every
/// round compared (item 3659).
pub(crate) fn inca_enlightenment_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1323",
        &[(RUN1323, WIDENING_INCA_ENLIGHTENMENT_GAP.0)],
        WIDENING_INCA_ENLIGHTENMENT_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_run1323_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 93, "every captured block");
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3659, before piece 753's bays: 93 keys — `0/5` struck on 3053
    // in theirs and not at all in ours by block 3054 (`damage_frame` 0
    // against 3053, `hits_left` 65 against 51), then `0/5`'s orders and
    // position on 3064, `0/12`'s damage on 3070, and `1/36`'s target on
    // 3087 (`0/5` against `0/12`); every Musketeer round of group 64 (30
    // theirs, 35 ours) parted on its launch point, ours from the figure's
    // own square at its `z`, theirs some 130 ahead and 160–200 above,
    // `1/25`'s on 3053 flying 3 frames against 2. After: **51**, all
    // standing (groups' `held` and pools, `SITE[i].reg`,
    // `aggression[1]`), and no round parts.
    pin_eq!(w.firsts.len(), 51, "run1323 baseline");
    pin_eq!(w.rounds_read, (30, 30), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The closing state** (item 3659): run1310's closing block 4046, the
/// whole map's units, walked from run1309's start.
#[test]
fn inca_enlightenment_pair_run1310_s_closing_state() {
    let _pins = Pins::hold();
    let p = &INCA_ENLIGHTENMENT;
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
        "Inca Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4046, "the closing block");
    // Item 3659, the pair whole: 166 units, none off, torn, unlinked or
    // extra, `Closing fields: none`. With the Musketeers' bays alone the
    // closing block read 6 `build_unlinked` and 1 `city_unlinked`: who=1
    // captures the human's capital `0/2000` on 4045 (`1/2043`,
    // `capture_stamp` 4045) and its five buildings with it, where ours
    // held `0/2000` at 4 health — the Fusiliers' rounds (piece 790) left
    // the figure's point and the city's damage parted on 3815.
    pin_eq!(e.compared, 166, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1323c: the closing window, blocks 3790..4046 at run1310b's detail,
/// `cover=0` (item 3659) — 257 blocks through the closing block, so the
/// word 4045, the game's last frame, sits inside a window.
pub(crate) const RUN1323C: &str =
    "gamelog-run1323c-greatsahara-inca-enlightenment-window-3790-4046.txt";

/// The window: block 3790 through the closing block 4046.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_FRAME_4045: (i64, i64) = (3790, 4046);

/// **The closing window**: run1323c walked from run1309's start with the
/// recorder on, the rounds compared (item 3659).
pub(crate) fn inca_enlightenment_frame_4045_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1323c",
        &[(RUN1323C, WIDENING_INCA_ENLIGHTENMENT_FRAME_4045.0)],
        WIDENING_INCA_ENLIGHTENMENT_FRAME_4045,
        1,
        &[4046],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_run1323c_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_frame_4045_window() else {
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
    // Item 3659: **49**. Past the standing families (who=1's groups'
    // `held` and pools, `form`, `SITE[i].reg`, `aggression[1]`) only two
    // idle guards' `dest_angle` — `1/140` on 3886, `1/136` on 3890 — the
    // shape of parked 3567a and 3580b. Before piece 790's bays `0/2000`'s
    // `build:damage` parted on 3815 (785 ours, 791 theirs), from `1/44`'s
    // round (TypeIndex 136, Fusilier).
    pin_eq!(w.firsts.len(), 49, "run1323c baseline");
    pin_eq!(w.rounds_read, (214, 214), "the rounds read");
    // Item 3659: 214 a side. The seven keys that part are one round's,
    // the Carbineer `1/83`'s (TypeIndex 190, piece 844, a pivot piece) on
    // 4026: from (9048, 28872, 497) against (8917, 28944, 707); it lands
    // two units apart and moves no draw (parked 3659a).
    pin_eq!(w.rounds.len(), 7, "the rounds that part");
}
