//! **The Persians at a held Gunpowder age, East Indies, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97, 98, item 3698): East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Persians (23)** at
//! **Hard (4)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Gunpowder: `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`. Its triple,
//! `(18, 23, 4)`, is new: the Persian lobbies are the coverage pair's and
//! the Medieval pair's `(18, 23, 5)` — this map at another difficulty —
//! the Sahara coverage pair's `(7, 23, 5)`, the Classical pair's
//! `(14, 23, 5)` and `persian_word`'s `GreatLakesPersianTier2`,
//! `(14, 23, 2)`, which left the battery for the AI word; no battery game
//! fields the Persians. Captures: run1369 (a `DUMP_ALL` start) and run1370
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §358.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const PERSIAN_GUNPOWDER: Pair = Pair {
    name: "EastIndiesPersianGunpowder",
    map_style: 18,
    ai_tribe: 23,
    difficulty: 4,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1369-eastindies-persian-gunpowder-start.txt",
    long: (
        "gamelog-run1370-eastindies-persian-gunpowder-trace.txt",
        "rontrace-run1370.log",
    ),
    // The game ended itself on 5673 (closing block 5674).
    length: 5673,
    // Item 3698, first try: parts at 1180 by the count and the sequence,
    // ours 17 draws against theirs 14, index 7 — ours
    // `Leader::create_units+0x642`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`. The AI's first military valuation of the
    // game: ours rolls the bias on eight candidates, theirs on seven
    // (`docs/AI.md` §358).
    // Item 3706: 1180 → 3020. Theirs refuses the Elite Pikemen (135):
    // `Types::finalize_grafting` takes the Persians out of its mask for the
    // Arqimmortal, which grafts it (run1383's packet, §358.2). On 3020 ours
    // spends 15 draws against theirs' 10, parting at index 0: ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`.
    // Item 3708: 3020 → 3130. `Forts::find_fort` answers only for a fort
    // `Build::activate` has listed; ours counted the unbuilt `1/2034` in
    // `compute_site_stats`' 5×5 from 2375, scored the home region's samples
    // ×3/2, and kept a different site table, so the Village went to
    // (34, 32) where theirs takes (34, 33) (run1385, run1385p, §358.3). On
    // 3130 ours spends 98 draws against theirs' 97, parting at index 54:
    // ours `Unit::think_fish+0x27a`, theirs `Guy::init_real+0x52`.
    count: 3130,
    sequence: 3130,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_GUNPOWDER_PAIR_WORD: i64 =
    if PERSIAN_GUNPOWDER.count < PERSIAN_GUNPOWDER.sequence {
        PERSIAN_GUNPOWDER.count
    } else {
        PERSIAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn persian_gunpowder_pair_captures_say_their_lobby() {
    let p = &PERSIAN_GUNPOWDER;
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
        "the Persian Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3698; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Persians — and no other Persian lobby's: the coverage pair's and the
/// Medieval pair's `(18, 23, 5)`, this map at another difficulty, the
/// Sahara coverage pair's `(7, 23, 5)`, the Classical pair's `(14, 23, 5)`
/// and the AI word `GreatLakesPersianTier2`'s `(14, 23, 2)`, other maps
/// (item 3698).
#[test]
fn persian_gunpowder_pair_is_no_other_lobby() {
    let p = &PERSIAN_GUNPOWDER;
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
        "the battery fields the Persians"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Persian lobby on this map at this difficulty"
        );
    }
    let w = &super::persian_word::PERSIAN;
    let c = &super::coverage_pair::COVERAGE;
    let s = &super::sahara_coverage::SAHARA_COVERAGE;
    let k = &super::persian_classical_pair::PERSIAN_CLASSICAL;
    let m = &super::persian_medieval_pair::PERSIAN_MEDIEVAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Persians");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3698, from 1180;
/// item 3706 extended it to 3001, the last stride block before 3020's
/// window): only run1370's stride blocks are on disk outside the windows.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_STRIDE: (i64, i64) = (101, 3001);

pub(crate) fn persian_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1370 stride",
        &[(
            PERSIAN_GUNPOWDER.long.0,
            WIDENING_PERSIAN_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_PERSIAN_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_stride_window() else {
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
    // Item 3698: 61 keys, every one a standing family — `form` and the
    // leaders on 101, the buildings' `build:extra` (which the stride does
    // not print) and who=1's groups 64 and 66's `held` and pools on 801.
    // No unit row parts before the word's window.
    // Item 3706, walked on to 3001 (30 blocks): **150** keys. 101..2701 is
    // the standing families alone (groups minted, `held`, pools, `form`,
    // the leaders, `build:extra`). **The state first parts on 2801**: the
    // Citizen `1/10` (TI 50), on an `EXPLORETOORDER` above a `BUILDORDER`
    // for the Village `1/2039` (TI 414), walks to (26088, 25416) in ours
    // against (26088, 26184) in theirs — ours places the Village 768 lower
    // (`y_internal` 24672 against 25440 on run1383b's 3014). No draw parts
    // there; on 2901 `1/10`'s `pos` parts, and on 3001 the Explorer `1/0`'s.
    // Item 3708 (`find_fort` waits for a built fort): **132** keys. The
    // Village stands where theirs does and `1/10`'s rows agree; the first
    // row that is not standing is on 2901, the Gun Mahout `1/42` (TI 263,
    // land, a `GUARDORDER` on `1/17` that has just changed posts)
    // `dest_angle` ours -1073741824 against 1556283392 — on the base too
    // (the mutation's stride prints it), and quiet again by run1383b's 3014.
    pin_eq!(w.firsts.len(), 132, "the stride's baseline");
}

/// run1370b: the word 1180's window, blocks 1095..1431 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 1101 and 250 past the word's own block 1181 (item 3698).
pub(crate) const RUN1370B: &str =
    "gamelog-run1370b-eastindies-persian-gunpowder-window-1095-1431.txt";

/// The window: block 1095 through 1431; the word 1180's own block is 1181.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_FRAME_1180: (i64, i64) = (1095, 1431);

/// **The word 1180's window**: run1370b walked from run1369's start with
/// the recorder on and every round compared (item 3698).
pub(crate) fn persian_gunpowder_frame_1180_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1370b",
        &[(RUN1370B, WIDENING_PERSIAN_GUNPOWDER_FRAME_1180.0)],
        WIDENING_PERSIAN_GUNPOWDER_FRAME_1180,
        1,
        &[1181],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_run1370b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_frame_1180_window() else {
        return;
    };
    pin_eq!(w.blocks, 336, "every captured block");
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
    // Item 3698, the first walk: **809** keys. Block 1095 carries the
    // standing families (`form`, both leaders' `SITE[i].reg` and
    // `aggression`, who=1's groups 64..66 and pools); group 67 is minted
    // on 1146. **The first row that is not standing is on the word's
    // block 1181**: who=1's Citizen `1/12` (TI 50, colliding on an
    // `EXPLORETOORDER`) `order:move.pause` ours 4 against theirs 9 — the
    // collision's pause on a seed the extra `create_units` roll shifted.
    // The make lists agree on 1181 and part on 1183 (`MAKE[6].t` ours 98,
    // theirs -1); on 1186 the Barracks `1/2014` queues the Elite Pikemen
    // (135) in ours against the `Arqimmortal` (165) in theirs.
    pin_eq!(w.firsts.len(), 52, "run1370b baseline");
    // Every round, both directions: none fly in the window.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1383b: the word 3020's window, blocks 3014..3270 at run1370b's
/// detail — six blocks before the word's own block 3021 and 250 past it
/// (item 3706).
pub(crate) const RUN1383B: &str =
    "gamelog-run1383b-eastindies-persian-gunpowder-window-3014-3270.txt";

/// The window: block 3014 through 3270; the word 3020's own block is 3021.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_FRAME_3020: (i64, i64) = (3014, 3270);

/// **The word 3020's window**: run1383b walked from run1369's start with
/// the recorder on and every round compared (item 3706).
pub(crate) fn persian_gunpowder_frame_3020_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1383b",
        &[(RUN1383B, WIDENING_PERSIAN_GUNPOWDER_FRAME_3020.0)],
        WIDENING_PERSIAN_GUNPOWDER_FRAME_3020,
        1,
        &[3021],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_run1383b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_frame_3020_window() else {
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
    // Item 3706: **672** keys. Block 3014 carries the standing families
    // and what parted on the stride before it: the Village `1/2039`'s
    // `y_internal` (24672 against 25440), `construct_hits` and
    // `job_counter`, the Citizen `1/10`'s and the Explorer `1/0`'s `pos`.
    // **The word's block**: on 3020 the Explorer `1/0` (TI 71, land) holds
    // its `EXPLORETOORDER` to (24312, 25080) in theirs and none in ours
    // (`orders.len` 0 against 1), at the same `pos` (24304, 25076); on
    // 3021 it is idle in ours (`idle` 1 against 0) and its `do_idle` roll
    // is the draw ours spends first. No round flies in the window.
    // Item 3708 (the word at 3130): **295** keys, the standing families to
    // the word's block. **On 3131** who=1's Fishermen `1/125` (TI 317, sea;
    // idle and orderless on 3130) takes a `MOVEORDER` to (34968, 44952) in
    // ours against (29592, 41880) in theirs (`orders_x`, `orders_y`,
    // `order:move.angle`); on 3132 its path is 1 node against 7.
    pin_eq!(w.firsts.len(), 295, "run1383b baseline");
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1385: the site's window, blocks 2695..2801 at run1383b's detail —
/// six blocks before the stride's 2701 and through 2801, the block the
/// state first parts on (item 3708).
pub(crate) const RUN1385: &str =
    "gamelog-run1385-eastindies-persian-gunpowder-window-2695-2801.txt";

/// The window: block 2695 through 2801.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_FRAME_2801: (i64, i64) = (2695, 2801);

/// **The site's window**: run1385 walked from run1369's start with the
/// recorder on and every round compared (item 3708).
pub(crate) fn persian_gunpowder_frame_2801_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1385",
        &[(RUN1385, WIDENING_PERSIAN_GUNPOWDER_FRAME_2801.0)],
        WIDENING_PERSIAN_GUNPOWDER_FRAME_2801,
        1,
        &[2801],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_run1385_s_site_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_frame_2801_window() else {
        return;
    };
    pin_eq!(w.blocks, 106, "every captured block");
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
    // Item 3708, before its build: **118** keys — the standing families,
    // and who=1's site table already parted on 2695 (theirs holds
    // (42, 53) at 198 in slot 8, ours (44, 52) at 247); on 2776 `SITE[9]`
    // (34, 33) at 53784 in ours against (34, 32) at 215784, and on 2778 the
    // Village `1/2039`'s `y_internal` 24672 against 25440 (cells 32 and
    // 33). Built (`find_fort` waits for `Build::activate`): **91** keys,
    // every one a standing family (`form`, the leaders' `SITE[i].reg` and
    // `aggression`, who=1's groups and pools); the site table agrees.
    pin_eq!(w.firsts.len(), 91, "run1385 baseline");
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3698): run1370's closing block 5674,
/// the whole map's units, walked from run1369's start.
#[test]
fn persian_gunpowder_pair_run1370_s_closing_state() {
    let _pins = Pins::hold();
    let p = &PERSIAN_GUNPOWDER;
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
        "Persian Gunpowder pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5674, "the closing block");
    // Item 3698, the first try: the pair parts at 1180, and the closing
    // block 5674 compares 238 units, nearly every one off and who=1's
    // `1/226`..`1/245` unlinked; every who=1 building's `myhits` reads
    // high in ours (6000 against 5500 on `1/2000`, parked 3698a).
    // Item 3706 (the word at 3020): 250 units compared, and the human's
    // `0/11` and `0/13` and who=1's `1/235` stand in ours alone (extra).
    // Item 3708 (the word at 3130): 250 compared, counts
    // `[237, 7, 1, 7, 180, 1, 83]`, the human's `0/13` the one extra.
    pin_eq!(e.compared, 250, "units compared");
    pin_eq!(e.counts(), [237, 7, 1, 7, 180, 1, 83], "the closing counts");
    pin_eq!(e.extra.clone(), vec![(0, 13)], "the closing extras");
}
