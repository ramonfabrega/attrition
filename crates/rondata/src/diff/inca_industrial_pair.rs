//! **The Inca at a held Industrial age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, item 3699): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **Inca (2)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple,
//! `(14, 2, 5)`, is new: the Inca lobbies are `inca_word`'s `(18, 2, 3)`,
//! `inca_sahara_word`'s `(7, 2, 3)`, `inca_classical_pair`'s `(18, 2, 5)`
//! and `inca_enlightenment_pair`'s `(7, 2, 5)`, other maps; the battery
//! fields no Inca. Captures: run1371 (a `DUMP_ALL` start) and run1372 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try is
//! in `docs/AI.md` §359.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const INCA_INDUSTRIAL: Pair = Pair {
    name: "GreatLakesIncaIndustrial",
    map_style: 14,
    ai_tribe: 2,
    difficulty: 5,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1371-greatlakes-inca-industrial-start.txt",
    long: (
        "gamelog-run1372-greatlakes-inca-industrial-trace.txt",
        "rontrace-run1372.log",
    ),
    // The game ended itself on 4471 (closing block 4472).
    length: 4471,
    // Item 3699, first try: whole, 4471 of 4471 by the count and the
    // sequence (`docs/AI.md` §359).
    count: 4471,
    sequence: 4471,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INCA_INDUSTRIAL_PAIR_WORD: i64 =
    if INCA_INDUSTRIAL.count < INCA_INDUSTRIAL.sequence {
        INCA_INDUSTRIAL.count
    } else {
        INCA_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn inca_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &INCA_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "INCA_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn inca_industrial_pair_captures_say_their_lobby() {
    let p = &INCA_INDUSTRIAL;
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
        "the Inca Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3699; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Inca — and no other Inca lobby's: `inca_word`'s `(18, 2, 3)`,
/// `inca_sahara_word`'s `(7, 2, 3)`, `inca_classical_pair`'s `(18, 2, 5)`
/// and `inca_enlightenment_pair`'s `(7, 2, 5)`, other maps (item 3699).
#[test]
fn inca_industrial_pair_is_no_other_lobby() {
    let p = &INCA_INDUSTRIAL;
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
        "the battery fields the Inca"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Inca lobby on this map at this difficulty"
        );
    }
    let w = &super::inca_word::INCA;
    let s = &super::inca_sahara_word::INCA_SAHARA;
    let c = &super::inca_classical_pair::INCA_CLASSICAL;
    let e = &super::inca_enlightenment_pair::INCA_ENLIGHTENMENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Inca");
        assert_ne!(l.0, p.map_style, "{name} is on this map");
    }
}

/// run1372b: the pair's last blocks, 4216..4472 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — the word 4471 is the game's last frame and
/// 4472 its closing block (item 3699). run1372 prints units only every
/// hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, so no dump held the last blocks.
pub(crate) const RUN1372B: &str =
    "gamelog-run1372b-greatlakes-inca-industrial-window-4216-4472.txt";

/// The window: block 4216 through the closing block 4472.
pub(crate) const WIDENING_INCA_INDUSTRIAL_CLOSING: (i64, i64) = (4216, 4472);

/// **The pair's last blocks**: run1372b walked from run1371's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3699).
pub(crate) fn inca_industrial_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INCA_INDUSTRIAL.start],
        true,
        INCA_INDUSTRIAL.long,
        "run1372b",
        &[(RUN1372B, WIDENING_INCA_INDUSTRIAL_CLOSING.0)],
        WIDENING_INCA_INDUSTRIAL_CLOSING,
        1,
        &[4472],
        true,
    )
}

#[test]
fn inca_industrial_pair_run1372b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_industrial_closing_window() else {
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
    // Item 3699, the first walk: **78** keys. Block 4216 carries the
    // standing families (`form` on 27 units, who=1's groups 64..71's
    // `held` and pools, both leaders' `SITE[i].reg`, who=0's
    // `aggression[1]` ours 0 against theirs 1), the Biplanes' (287)
    // `myspeed` ours 55 against theirs 68 on 18 units (3677a's family) and
    // `1/33`'s `g.z[0]` 0 against 30; after it, the Rifleman `1/54`'s
    // `dest_angle` on 4362 (3695b's shape). The draw stream and the
    // closing block 4472 agree.
    // Item 3742: 78 → **60**, the Biplanes' `myspeed` on 18 units gone
    // with `update_speed`'s Aluminum arm (`docs/AI.md` §366.3).
    pin_eq!(w.firsts.len(), 60, "run1372b baseline");
    // Every round, both directions: 297 round-blocks each side. **10 rows
    // part, every one a launch** of group 66's Siege Mortars (TI 275, TY
    // 225, **piece 225**, which `launch::BAYS` has no row for): `1/60`'s
    // `ammo[0]` on 4236 (slot 12, `CHAR_ATTACK2`) leaves (7128, _, 179) in
    // ours — the lead figure's own square — against theirs' (7125, _,
    // 340), `v1z` 208657242 against 204420395, and its `ammo[1]` on 4291
    // the same; `1/69`'s `ammo[0]` on 4285 (slot 11) (7416, 32664, 41)
    // against (7413, 32663, 202). Each lands on the same frame both sides.
    pin_eq!(w.rounds_read, (297, 297), "the rounds read");
    pin_eq!(w.rounds.len(), 10, "the rounds that part");
}

/// **The pair's closing state** (item 3699): run1372's closing block 4472,
/// the whole map's units, walked from run1371's start.
#[test]
fn inca_industrial_pair_run1372_s_closing_state() {
    let _pins = Pins::hold();
    let p = &INCA_INDUSTRIAL;
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
        "Inca Industrial pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4472, "the closing block");
    // Item 3699, the pair whole on its first try: 156 units compared, none
    // off, torn, unlinked or extra, no building row; **one closing field**,
    // who=1's city `1/2058`'s `assimilation_timer`, ours 4471 against
    // theirs 4468 — a city taken on the game's last frame, theirs' stamp
    // walked back one `THECITIZEN_ASSIMILATION_SPEED` tick (`1 − 4`) more
    // than ours (parked 3699a). The draw stream never feels it.
    pin_eq!(e.compared, 156, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
