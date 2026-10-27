//! **The Americans at a held Industrial age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, 97 §3, item 3669): Great Sahara, `MAP_STYLE 7`, human
//! Nubians (4) against the **Americans (20)** at Toughest (5), seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Industrial: `STARTING_TECHNOLOGY
//! 5`, `ENDING_TECHNOLOGY 5`. Its triple, `(7, 20, 5)`, is new: the other
//! American lobbies are `american_word`'s `(18, 20, 1)`,
//! `american_lakes_word`'s `(14, 20, 4)`, `american_modern_pair`'s `(7,
//! 20, 3)` — this map at another difficulty — and
//! `american_enlightenment_pair`'s `(18, 20, 5)`, another map; the battery
//! fields no Americans. Captures: run1337 (a `DUMP_ALL` start) and run1338
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §348.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN_INDUSTRIAL: Pair = Pair {
    name: "GreatSaharaAmericanIndustrial",
    map_style: 7,
    ai_tribe: 20,
    difficulty: 5,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1337-greatsahara-american-industrial-start.txt",
    long: (
        "gamelog-run1338-greatsahara-american-industrial-trace.txt",
        "rontrace-run1338.log",
    ),
    // The game ended itself on 5861 (closing block 5862).
    length: 5861,
    // Item 3669, first try: frame 3174, by the count and the sequence —
    // ours 24 draws against 25, parting at index 1, ours
    // `Unit::fight+0x824`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (`docs/AI.md` §348). Item 3686 moved it to
    // 5678: piece 87's bays (the round of 3105 leaves the rifle's muzzle)
    // and the CEO's heal (`1/36` repaired a point on block 3167) — ours
    // 56 draws against 55, parting at index 21, ours `Unit::fight+0x9b0`,
    // theirs `Unit::fight+0x824` (`docs/AI.md` §348.1). Item 3690 moved
    // it to 5701: piece 167's bays and the packed-siege shortcut in
    // `Group::action_attack`'s member search (`1/131` takes its army's
    // `0/2000` mandatory on 5678) — ours 20 draws against 19, parting at
    // index 2, ours `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
    // `Guy::set_anim+0x97a < Animal::do_idle+0x19` (`docs/AI.md` §348.2).
    count: 5701,
    sequence: 5701,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AMERICAN_INDUSTRIAL_PAIR_WORD: i64 =
    if AMERICAN_INDUSTRIAL.count < AMERICAN_INDUSTRIAL.sequence {
        AMERICAN_INDUSTRIAL.count
    } else {
        AMERICAN_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn american_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AMERICAN_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5, and the long is scoreable.
#[test]
fn american_industrial_pair_captures_say_their_lobby() {
    let p = &AMERICAN_INDUSTRIAL;
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
        "the American Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3669; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Americans; and no other pinned American lobby's: `american_word`'s
/// `(18, 20, 1)`, `american_lakes_word`'s `(14, 20, 4)`,
/// `american_modern_pair`'s `(7, 20, 3)` — this map at another difficulty
/// — and `american_enlightenment_pair`'s `(18, 20, 5)` (item 3669).
#[test]
fn american_industrial_pair_is_no_other_lobby() {
    let p = &AMERICAN_INDUSTRIAL;
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
        "the battery fields the Americans"
    );
    let word = &super::american_word::AMERICAN;
    let lakes = &super::american_lakes_word::AMERICAN_LAKES;
    let modern = &super::american_modern_pair::AMERICAN_MODERN;
    let enlightenment = &super::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            lakes.name,
            (lakes.map_style, lakes.ai_tribe, lakes.difficulty),
        ),
        (
            modern.name,
            (modern.map_style, modern.ai_tribe, modern.difficulty),
        ),
        (
            enlightenment.name,
            (
                enlightenment.map_style,
                enlightenment.ai_tribe,
                enlightenment.difficulty,
            ),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Americans");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3174** (item 3669):
/// only run1338's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_AMERICAN_INDUSTRIAL_STRIDE: (i64, i64) = (101, 3101);

pub(crate) fn american_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_INDUSTRIAL.start],
        true,
        AMERICAN_INDUSTRIAL.long,
        "run1338 stride",
        &[(
            AMERICAN_INDUSTRIAL.long.0,
            WIDENING_AMERICAN_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_AMERICAN_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn american_industrial_pair_the_long_s_stride_walks_back_from_3174() {
    let _pins = Pins::hold();
    let Some(w) = american_industrial_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 31, "every stride block in the window");
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
    // Item 3669: 100 keys, all of them the standing families — the
    // buildings' `build:extra` (which the stride does not print), `form`
    // on the units, who=1's groups' `held` and pools 64..71 as each is
    // minted, and `1/14`'s `g.angle[0]` from 401 (ours 1431655765, the
    // figure's unset facing, against theirs' 0; the American Modern
    // pair's standing row). **No unit row parts on any stride block to
    // 3101**: the parting is in 3102..3174, which is run1338b's.
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1338b: the word 3174's window, blocks 3095..3425 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3101 and 250 after the word's block 3175 (item 3669).
pub(crate) const RUN1338B: &str =
    "gamelog-run1338b-greatsahara-american-industrial-window-3095-3425.txt";

/// The window: block 3095 through 3425; the word 3174's own block is 3175.
pub(crate) const WIDENING_AMERICAN_INDUSTRIAL_FRAME_3174: (i64, i64) = (3095, 3425);

/// **The word 3174's window**: run1338b walked from run1337's start with
/// the recorder on and every round compared (item 3669).
pub(crate) fn american_industrial_frame_3174_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INDUSTRIAL.start],
        true,
        AMERICAN_INDUSTRIAL.long,
        "run1338b",
        &[(RUN1338B, WIDENING_AMERICAN_INDUSTRIAL_FRAME_3174.0)],
        WIDENING_AMERICAN_INDUSTRIAL_FRAME_3174,
        1,
        &[3175],
        true,
    )
}

#[test]
fn american_industrial_pair_run1338b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_industrial_frame_3174_window() else {
        return;
    };
    pin_eq!(w.blocks, 330, "every captured block");
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
    // Item 3669, the first walk: 2030 keys. **Block 3095 already carries
    // who=1's two Airbases' queues parted**, beside the standing families
    // (`form`, `build:extra`, who=1's groups' `held` and pools, the human
    // leader's `SITE[*].reg` and `aggression[1]`, `1/14`'s `g.angle[0]`):
    // `1/2022` and `1/2025` queue Biplanes (287) at `cost[0]` 107, 113,
    // 117 in ours against 90, 96, 99 in theirs, and who=1's `MAKE[2].val`
    // (447 against 468) and its wealth and oil buckets stand parted with
    // them — ours carries no `AMERICANS_AIRCRAFT` (`docs/NATIONS.md`).
    // **The first record to part after it is a round, on 3105**: who=1's
    // Anti-tank Rifle `1/18` (TY 137, `ATRIFLE`, attacking the human's
    // Citizen `0/10`) launches `ammo[0]` from (9528, 31080) at `sz` 612 in
    // ours — the figure's square — and from (9379, 31019) at 695 in
    // theirs. On 3142 `1/44`'s round does the same, `total_time` 3
    // against 2, so theirs' lands on `0/10` on 3142 (12 damage,
    // `hits_left` 53) and its Citizen flees on 3143 where ours' stands;
    // on the word 3174 ours' `Unit::fight` draws where theirs'
    // `Unit::move_step` does (`docs/AI.md` §348).
    //
    // Item 3686: 2030 → 76 keys. Piece 87's bays put the round of 3105 on
    // theirs' (9379, 31019) at 695, and the CEO's heal repairs `1/36` on
    // block 3167. After block 3095's standing families, nothing but
    // who=1's groups' `held` and pools parts until one round on 3399:
    // `1/18`'s `ammo[0]` names no target in ours (`whom`, `ox` −1) and the
    // Citizen `0/13` in theirs.
    pin_eq!(w.firsts.len(), 76, "run1338b baseline");
    // Rounds are in flight on both sides over the window.
    pin_eq!(w.rounds_read, (326, 326), "the rounds read");
    pin_eq!(w.rounds.len(), 2, "the rounds that part");
}

/// run1356: the word 5678's window, blocks 5595..5861 at run1338b's
/// detail — six blocks before the stride's block 5601 and on to the game's
/// own end, 5861 (item 3686).
const RUN1356: &str = "gamelog-run1356-greatsahara-american-industrial-window-5595-5861.txt";

/// The window: block 5595 through 5861; the word 5678's own block is 5679.
pub(crate) const WIDENING_AMERICAN_INDUSTRIAL_FRAME_5678: (i64, i64) = (5595, 5861);

/// **The word 5678's window**: run1356 walked from run1337's start with
/// the recorder on and every round compared (item 3686).
pub(crate) fn american_industrial_frame_5678_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INDUSTRIAL.start],
        true,
        AMERICAN_INDUSTRIAL.long,
        "run1356",
        &[(RUN1356, WIDENING_AMERICAN_INDUSTRIAL_FRAME_5678.0)],
        WIDENING_AMERICAN_INDUSTRIAL_FRAME_5678,
        1,
        // The word's block, and since item 3690 the new word 5701's (5702).
        &[5679, 5702],
        true,
    )
}

#[test]
fn american_industrial_pair_run1356_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_industrial_frame_5678_window() else {
        return;
    };
    pin_eq!(w.blocks, 266, "every captured block");
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
    // Item 3686, the first walk: 2670 keys. **Block 5595 carries the
    // standing families** (`form`, who=1's groups' `held` and pools, the
    // human leader's `SITE[*].reg` and `aggression[1]`, `1/14`'s
    // `g.angle[0]`) **and 3669a's**: the Airbases' queues at `cost[0]` 165
    // against 147 and who=1's buckets with them. **The first record to
    // part after it is a round, on 5620**: who=1's Armored Car `1/104` (TY
    // 217, `GREYHOUND`, piece 167, firing on the move in group 65)
    // launches `ammo[0]` from (7926, 30423) at `sz` 655 in ours and from
    // (7758, 30368) at 672 in theirs, and flies 3 frames against 2; its
    // round of 5646 parts the same way. On the word's block 5679 who=1's
    // group-65 members' move orders part (`1/145`'s `x` 12216 against
    // 11448), and `1/131` takes a second order in theirs (`docs/AI.md`
    // §348.1).
    // Item 3690: piece 167's bays put `1/104`'s round of 5620 on theirs'
    // (7758, 30368) at `sz` 672 — both sides fly it 3 frames — and the
    // packed-siege shortcut in `Group::action_attack`'s member search
    // gives `1/131` its army's `0/2000` mandatory on the word's block
    // 5679, as theirs' (`[ATTACK mandatory 1 → 0/2000, CASTORDER 652
    // paid 0]`; ours had a fresh non-mandatory `ATTACK` and re-searched).
    // The first round to part is now 5735's (`1/104` again); group 65's
    // move orders still part on 5679 (`1/145`'s `x` 12216 against 11448).
    pin_eq!(w.firsts.len(), 2153, "run1356 after item 3690");
    pin_eq!(w.rounds_read, (166, 49), "the rounds read");
    pin_eq!(w.rounds.len(), 27, "the rounds that part");
}
