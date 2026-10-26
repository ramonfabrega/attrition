//! **The Persians at a held Classical age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, item 3652): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Persians (23)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical:
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`. Its triple,
//! `(14, 23, 5)`, is new: the Persian lobbies are the coverage pair's and
//! `persian_medieval_pair`'s `(18, 23, 5)`, Great Sahara's `(7, 23, 5)`
//! and the AI word `GreatLakesPersianTier2`'s `(14, 23, 2)`, and the
//! battery fields no Persians. The first stand-up on Great Lakes on the
//! `Late ages` line. Captures: run1317 (a `DUMP_ALL` start) and run1318
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §343.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const PERSIAN_CLASSICAL: Pair = Pair {
    name: "GreatLakesPersianClassical",
    map_style: 14,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1317-greatlakes-persian-classical-start.txt",
    long: (
        "gamelog-run1318-greatlakes-persian-classical-trace.txt",
        "rontrace-run1318.log",
    ),
    // The game ended itself on 11633 (closing block 11634).
    length: 11633,
    // Item 3652, first try: frame 10937, by the count and the sequence —
    // ours 34 draws against 36, parting at index 28, ours
    // `Farms::inc_time+0x1ae`, theirs `Ammo::do_damage+0xc59`
    // (`docs/AI.md` §343). Item 3663: 10937 → 11028 (piece 1793's bays,
    // the Persian Javelineers') → **11633, the game's last frame**
    // (piece 1881's, the Persian Archers'), the pair whole.
    count: 11633,
    sequence: 11633,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_CLASSICAL_PAIR_WORD: i64 =
    if PERSIAN_CLASSICAL.count < PERSIAN_CLASSICAL.sequence {
        PERSIAN_CLASSICAL.count
    } else {
        PERSIAN_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn persian_classical_pair_captures_say_their_lobby() {
    let p = &PERSIAN_CLASSICAL;
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
        "the Persian Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3652; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Persians; and no other Persian lobby's: the coverage and Medieval
/// pairs' `(18, 23, 5)`, Great Sahara's `(7, 23, 5)` and the AI word
/// `GreatLakesPersianTier2`'s `(14, 23, 2)` are another map or another
/// difficulty (item 3652).
#[test]
fn persian_classical_pair_is_no_other_lobby() {
    let p = &PERSIAN_CLASSICAL;
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
        "the battery fields the Persians"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Persian lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::persian_medieval_pair::PERSIAN_MEDIEVAL,
        &super::coverage_pair::COVERAGE,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
}

/// **The long's stride, walked back from the word 10937** (item 3652):
/// only run1318's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_PERSIAN_CLASSICAL_STRIDE: (i64, i64) = (101, 10901);

pub(crate) fn persian_classical_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_CLASSICAL.start],
        true,
        PERSIAN_CLASSICAL.long,
        "run1318 stride",
        &[(
            PERSIAN_CLASSICAL.long.0,
            WIDENING_PERSIAN_CLASSICAL_STRIDE.0,
        )],
        WIDENING_PERSIAN_CLASSICAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn persian_classical_pair_the_long_s_stride_walks_back_from_10937() {
    let _pins = Pins::hold();
    let Some(w) = persian_classical_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 109, "every stride block in the window");
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
    // Item 3652: 143 keys, all of them the standing families — 96
    // buildings' `build:extra` (which the stride does not print), `form`
    // on 27 units, and who=1's groups' `held` and pools 64..73 as each is
    // minted. **No unit row parts on any stride block to 10901**: the
    // parting is in 10902..10937, which is run1318b's.
    pin_eq!(w.firsts.len(), 143, "the stride's baseline");
}

/// run1318b: the word 10937's window, blocks 10895..11188 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 10901 and 250 after the word's block 10938 (item 3652).
pub(crate) const RUN1318B: &str =
    "gamelog-run1318b-greatlakes-persian-classical-window-10895-11188.txt";

/// The window: block 10895 through 11188; the word 10937's own block is
/// 10938.
pub(crate) const WIDENING_PERSIAN_CLASSICAL_FRAME_10937: (i64, i64) = (10895, 11188);

/// **The word 10937's window**: run1318b walked from run1317's start with
/// the recorder on and every round compared (item 3652).
pub(crate) fn persian_classical_frame_10937_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_CLASSICAL.start],
        true,
        PERSIAN_CLASSICAL.long,
        "run1318b",
        &[(RUN1318B, WIDENING_PERSIAN_CLASSICAL_FRAME_10937.0)],
        WIDENING_PERSIAN_CLASSICAL_FRAME_10937,
        1,
        &[10938],
        true,
    )
}

#[test]
fn persian_classical_pair_run1318b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_classical_frame_10937_window() else {
        return;
    };
    pin_eq!(w.blocks, 293, "every captured block");
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
    // Item 3652, the first walk: 8523 keys. Block 10895 carries the
    // standing families (`form`, both leaders, who=1's groups and pools);
    // **the records first part on 10939**, after the round below: who=1's
    // `1/26`'s move order (`x` ours 41208 theirs 41592, `off_x` 504
    // against 120), and the guards' `retry` on `1/257`, `1/318`, `1/331`.
    // Item 3663, pieces 1793's, 1881's, 113's and 1975's bays: **58**,
    // the standing families alone (`SITE[i].reg`, `aggression[1]`,
    // `form`, who=1's groups' `held` and pools as each is minted).
    pin_eq!(w.firsts.len(), 58, "run1318b baseline");
    // Every round, both directions: 520 round-blocks theirs, 699 ours, 114
    // parting. **The first is on 10931, who=1's Javelineers `1/312`** (TY
    // 83) at the human's Citizen `0/12`: theirs launches from (5054,
    // 31417) at `sz` 669, ours from the figure's own square (5064, 31368)
    // at 499 — 170 under theirs, the Javelineers' bay height — and on
    // 10935 theirs' round holds `whom` 0, `ox` 12 where ours' holds -1;
    // theirs' 10937 draws `Ammo::do_damage+0xc59` at index 28 where ours
    // draws `Farms::inc_time+0x1ae`. The hypothesis: the Persian
    // Javelineers' piece has no `launch::BAYS` row (`docs/AI.md` §343).
    // Item 3663: `1/312`'s launch agrees (piece 1793), and so do the
    // Persian Archers' (piece 1881) — `1/299`'s on 10980 from (5528,
    // 30923) where ours threw from (5592, 30984), 15 frames against 16,
    // and `0/11` struck on 10993 on both sides — and the Anusiya's
    // (piece 113). 520 a side, 68 parting: the dead target's `whom`/`ox`
    // (theirs keeps it, §338) and the slot shifts theirs' spent arrows
    // make (COMBAT §46.7).
    pin_eq!(w.rounds_read, (520, 520), "the rounds read");
    pin_eq!(w.rounds.len(), 68, "the rounds that part");
}

/// run1327b: the closing window, blocks 11378..11634 at run1318b's detail,
/// `cover=0` (item 3663) — 257 blocks through the closing block, so the
/// word 11633, the game's last frame, sits inside a window.
pub(crate) const RUN1327B: &str =
    "gamelog-run1327b-greatlakes-persian-classical-window-11378-11634.txt";

/// The window: block 11378 through the closing block 11634.
pub(crate) const WIDENING_PERSIAN_CLASSICAL_FRAME_11633: (i64, i64) = (11378, 11634);

/// **The closing window**: run1327b walked from run1317's start with the
/// recorder on, the rounds compared (item 3663).
pub(crate) fn persian_classical_frame_11633_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_CLASSICAL.start],
        true,
        PERSIAN_CLASSICAL.long,
        "run1327b",
        &[(RUN1327B, WIDENING_PERSIAN_CLASSICAL_FRAME_11633.0)],
        WIDENING_PERSIAN_CLASSICAL_FRAME_11633,
        1,
        &[11634],
        true,
    )
}

#[test]
fn persian_classical_pair_run1327b_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_classical_frame_11633_window() else {
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3663: **59** — the standing families, and three idle guards'
    // `dest_angle` (the Javelineers `1/172` on 11403 and `1/173` on 11613,
    // the Archer `1/124` on 11406; parked 3663a). Before pieces 113's and
    // 1975's bays: 65, with the human's `0/2000`'s `build:damage` (414
    // against 420 on 11378), `1/296`'s `damage_o` and `city_flags[0x4]`.
    pin_eq!(w.firsts.len(), 59, "run1327b baseline");
    pin_eq!(w.rounds_read, (1278, 1278), "the rounds read");
    // 1278 a side, 24 parting: dead targets' `whom`/`ox` and slot shifts.
    // Before pieces 113's and 1975's bays: 1278 against 1310, 254 parting,
    // the Anusiya's arrows (`1/333`'s on 11381 from (3912, 31080, 543)
    // against (3821, 31024, 721), 3 frames against 2) and the Catapults'
    // stones (`1/119`'s on 11511 from 208 against 453).
    pin_eq!(w.rounds.len(), 24, "the rounds that part");
}

/// **The closing state** (item 3663): run1318's closing block 11634, the
/// whole map's units, walked from run1317's start.
#[test]
fn persian_classical_pair_run1318_s_closing_state() {
    let _pins = Pins::hold();
    let p = &PERSIAN_CLASSICAL;
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
        "Persian Classical pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 11634, "the closing block");
    // Item 3663, the pair whole: 444 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 11633.
    pin_eq!(e.compared, 444, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
