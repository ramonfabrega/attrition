//! **The Americans at a held Information age, Great Lakes** (DECISIONS
//! 90, 91, 93, 94, 97, 98, 99, 101, item 3725): Great Lakes, `MAP_STYLE
//! 14`, human Nubians (4) against the **Americans (20)** at difficulty 5,
//! seed 12345, `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple,
//! `(14, 20, 5)`, is new: the Americans hold this map at 4
//! (`american_lakes_word`), East Indies at 1 (`american_word`) and 5
//! (`american_enlightenment_pair`), and Great Sahara at 3
//! (`american_modern_pair`) and 5 (`american_industrial_pair`); the
//! battery fields no Americans. Captures: run1408 (a `DUMP_ALL` start)
//! and run1409 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §372.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AMERICAN_INFORMATION: Pair = Pair {
    name: "GreatLakesAmericanInformation",
    map_style: 14,
    ai_tribe: 20,
    difficulty: 5,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1408-greatlakes-american-information-start.txt",
    long: (
        "gamelog-run1409-greatlakes-american-information-trace.txt",
        "rontrace-run1409.log",
    ),
    // The game ended itself on 4320 (closing block 4321).
    length: 4320,
    // Item 3725, first try: count 2385, sequence 2385 of 4320 — on 2385
    // ours 9 draws against theirs' 10, index 6 ours'
    // `Guy::set_anim+0x97a` against theirs' sixth
    // `Leader::make_stuff+0x63d` (`SITE_EXPIRE_SLOT`): who=1's make list
    // holds one more slot to expire in theirs (`docs/AI.md` §372).
    // Item 3755, `get_cost`'s American aircraft arm built: 2385 → 3653 —
    // on 3653 ours 10 draws against theirs' 11, index 2, ours'
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` against theirs'
    // `< Unit::do_idle+0x7d` (`docs/AI.md` §372.1).
    // Item 3756, piece 170's pivot rows (the Armored Cavalry's turret):
    // 3653 → 3898 — on 3898 ours 8 draws against theirs' 9, index 0, ours'
    // `Unit::do_air_physics+0xba` against theirs' `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`: the human's `0/12` walks off the Abrams
    // `1/29`'s blow of 3896 in theirs alone (`docs/AI.md` §372.2).
    // Item 3758, piece 204's pivot rows (the Abrams' turret): 3898 → 4291
    // — on 4291 ours 112 draws against theirs' 113, index 109, ours'
    // `Farms::inc_time+0x1ae` against theirs' `Object::take_damage+0xe1`
    // (`docs/AI.md` §372.3).
    // Item 3760, the Strategic Bomber's two bays (piece 255): 4291 →
    // **4320, whole** — `1/45`'s bombs of 4270..4282 had left its own
    // square, and the one of 4274 came down on (3238, 30788) against the
    // original's (3253, 30681), the block before theirs' blow of 4291. The
    // Jet Fighter's guns (245), the TOW's rows (91) and the Abrams' four
    // tracers (a node past the restriction count is its own entry alone)
    // land with it and move the windows, not the word (`docs/AI.md`
    // §372.4).
    count: 4320,
    sequence: 4320,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AMERICAN_INFORMATION_PAIR_WORD: i64 =
    if AMERICAN_INFORMATION.count < AMERICAN_INFORMATION.sequence {
        AMERICAN_INFORMATION.count
    } else {
        AMERICAN_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn american_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AMERICAN_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_INFORMATION {}: count {} sequence {} of {}; {}",
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
fn american_information_pair_captures_say_their_lobby() {
    let p = &AMERICAN_INFORMATION;
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
        "the American Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3725; the battery's identity
/// test refuses it), once, and none of the battery's — which fields no
/// Americans — and no other American lobby's: `american_lakes_word`'s
/// `(14, 20, 4)`, this map at another difficulty, `american_word`'s
/// `(18, 20, 1)`, `american_enlightenment_pair`'s `(18, 20, 5)`,
/// `american_modern_pair`'s `(7, 20, 3)` and `american_industrial_pair`'s
/// `(7, 20, 5)`, other maps (item 3725).
#[test]
fn american_information_pair_is_no_other_lobby() {
    let p = &AMERICAN_INFORMATION;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert_eq!(
        battery::BATTERY
            .iter()
            .filter(|l| l.ai_tribe == p.ai_tribe)
            .count(),
        0,
        "the battery fields the Americans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another American lobby on this map at this difficulty"
        );
    }
    let k = &super::american_lakes_word::AMERICAN_LAKES;
    let w = &super::american_word::AMERICAN;
    let e = &super::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT;
    let m = &super::american_modern_pair::AMERICAN_MODERN;
    let i = &super::american_industrial_pair::AMERICAN_INDUSTRIAL;
    for (name, l) in [
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Americans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3725, from
/// 2385): only run1409's stride blocks are on disk before the word's
/// window, 101 through 2301. Item 3758 walks it to 4201, the last stride
/// block before the word 4291's window.
pub(crate) const WIDENING_AMERICAN_INFORMATION_STRIDE: (i64, i64) = (101, 4201);

pub(crate) fn american_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1409 stride",
        &[(
            AMERICAN_INFORMATION.long.0,
            WIDENING_AMERICAN_INFORMATION_STRIDE.0,
        )],
        WIDENING_AMERICAN_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn american_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = american_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 42, "every stride block in the window");
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
    // Item 3725: 78 keys over 23 blocks, the standing families alone —
    // the buildings' `build:extra` (40, which the stride does not print),
    // `form` on 27 units, who=1's groups 64..68 with their pools (10), and
    // `1/14`'s `g.angle[0]` from 401. No position, order or hit parts.
    // Item 3758, the stride walked to 4201 (42 blocks): 94 → **121** keys,
    // the same families on the six blocks 3701..4201 — `build:extra` on
    // nine more buildings, `mirror` on more of who=1's units, and groups
    // 69..71 with their pools. No position, order or hit parts.
    pin_eq!(w.firsts.len(), 121, "the stride's baseline");
}

/// run1409b: the word 2385's window, blocks 2295..2635 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2301 and 250 past the word's own block 2386 (item 3725).
/// run1409 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1409B: &str =
    "gamelog-run1409b-greatlakes-american-information-window-2295-2635.txt";

/// The window: block 2295 through 2635.
pub(crate) const WIDENING_AMERICAN_INFORMATION_FRAME_2385: (i64, i64) = (2295, 2635);

/// **The word 2385's window**: run1409b walked from run1408's start with
/// the recorder on and every round compared (item 3725).
pub(crate) fn american_information_frame_2385_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1409b",
        &[(RUN1409B, WIDENING_AMERICAN_INFORMATION_FRAME_2385.0)],
        WIDENING_AMERICAN_INFORMATION_FRAME_2385,
        1,
        &[2386],
        true,
    )
}

#[test]
fn american_information_pair_run1409b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_information_frame_2385_window() else {
        return;
    };
    pin_eq!(w.blocks, 340, "every captured block");
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
    // Item 3725, the first walk: **1392** keys. Block 2295 carries 63
    // standing rows: `form` (27), who=0's and who=1's `aggression` (0
    // against 1), who=1's `SITE[i]` (14), groups 64..67 and their pools,
    // `1/14`'s `g.angle[0]`, **who=1's wealth and oil buckets** (17727
    // against 17799, 17415 against 17487) and `MAKE[3].val` (2112 against
    // 2188), and **the Airbase `1/2021`'s four queued Jet Fighters (TI
    // 295)** at `cost[0]` and `cost[1]` 89, 91, 93, 95 in ours against 71,
    // 73, 75, 77 — 4 × 18 = 72, the buckets' gap. `get_cost`'s American
    // aircraft arm (`AMERICANS_AIRCRAFT`, 20 %) is not carried (parked
    // 3669a). **The make list parts on 2384**: `MAKE[3]` reads `t` −1,
    // `num` 1, `escrow` 1 in ours against the Jet Fighter's `t` 295, `num`
    // 10, `escrow` 0, and `MAKE[2].val` 180836 against 191072; on 2385
    // theirs expires one more slot (the word). No round flies in the window
    // on either side (`docs/AI.md` §372). Item 3755, the arm built: **60**.
    // The price rows, the buckets, `MAKE[3]` and the make list on 2384 are
    // gone; past the standing families the first row is the second
    // Airbase's Strategic Bomber (305), `1/2021`'s `queue[3].cost` 91
    // against 86 on 2583 (§372.1).
    pin_eq!(w.firsts.len(), 60, "run1409b baseline");
    // Every round, both directions: `AMMO=5` was staged and none prints
    // on either side over 2295..2635.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1434b: the word 3653's window, blocks 3495..3903 at run1409b's
/// detail (item 3755): six blocks before the stride's block 3501, the
/// first after the walk's last quiet stride block, and 250 past the word's
/// own block 3654. Nothing on disk printed 3502..3600 or any leader,
/// building or round past run1409b's 2635. (run1434 itself left out
/// run1408's `--profile` lobby and parts from frame 0: a dead take.)
pub(crate) const RUN1434B: &str =
    "gamelog-run1434b-greatlakes-american-information-window-3495-3903.txt";

/// The window: block 3495 through 3903.
pub(crate) const WIDENING_AMERICAN_INFORMATION_FRAME_3653: (i64, i64) = (3495, 3903);

/// **The word 3653's window**: run1434b walked from run1408's start with
/// the recorder on and every round compared (item 3755).
pub(crate) fn american_information_frame_3653_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1434b",
        &[(RUN1434B, WIDENING_AMERICAN_INFORMATION_FRAME_3653.0)],
        WIDENING_AMERICAN_INFORMATION_FRAME_3653,
        1,
        &[3654],
        true,
    )
}

#[test]
fn american_information_pair_run1434b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_information_frame_3653_window() else {
        return;
    };
    pin_eq!(w.blocks, 408, "every captured block");
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
    // Item 3755, the first walk: **544** keys. Block 3495 stands on the
    // standing families and on who=1's Strategic Bombers (305) at the
    // Airbase `1/2030`, `cost[0]` and `cost[1]` 110, 120 against 100, 110,
    // the buckets 53 apart. **On 3591 `treaties` part** (who=0's `[1]` and
    // who=1's `[0]`, 1 in ours against 3) and who=0's Citizen `0/13` takes
    // a 20-point blow in theirs alone and walks off on 3592; who=0's `0/4`
    // on 3636 likewise (`docs/AI.md` §372.1). Item 3756, piece 170's pivot
    // rows: 544 → **96** keys, rounds 126 → 111 in theirs and 55 → 7
    // parting. The first round to part is the Abrams `1/29`'s on 3895,
    // (4296, 34056, 474) in ours against (4153, 33763, 605), and the
    // human's `0/12` parts on 3897 (`docs/AI.md` §372.2). Item 3758,
    // piece 204's pivot rows: 96 → **71** keys, rounds (110, 111) →
    // (110, 110) and 7 → **0** parting: `1/29`'s round of 3895 leaves
    // (4153, 33763, 605) as theirs, and `0/12` agrees (`docs/AI.md`
    // §372.3). What still parts is the standing families and `mirror`.
    pin_eq!(w.firsts.len(), 71, "run1434b baseline");
    pin_eq!(w.rounds_read, (110, 110), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1437b: the word 4291's window, blocks 4285..4321 at run1434b's
/// detail (item 3758) — six blocks before the word to the closing block
/// 4321. run1409's stride prints units only every hundred frames (4201,
/// then 4301), no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES` record
/// past the start, and run1434b ends on 3903.
pub(crate) const RUN1437B: &str =
    "gamelog-run1437b-greatlakes-american-information-closing-window-4285-4321.txt";

/// The window: block 4285 through the closing block 4321.
pub(crate) const WIDENING_AMERICAN_INFORMATION_FRAME_4291: (i64, i64) = (4285, 4321);

/// **The word 4291's window**: run1437b walked from run1408's start with
/// the recorder on and every round compared (item 3758).
pub(crate) fn american_information_frame_4291_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1437b",
        &[(RUN1437B, WIDENING_AMERICAN_INFORMATION_FRAME_4291.0)],
        WIDENING_AMERICAN_INFORMATION_FRAME_4291,
        1,
        &[4292],
        true,
    )
}

#[test]
fn american_information_pair_run1437b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_information_frame_4291_window() else {
        return;
    };
    pin_eq!(w.blocks, 36, "every captured block");
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
    // Item 3758: **154** keys. The window opens parted: on 4285 stand the
    // standing families (who=0's `SITE[0..9].reg` and `aggression[1]`,
    // `form`, who=1's groups and pools, its wealth and oil buckets 53
    // apart, `1/14`'s `g.angle[0]`) and who=0's building `0/2006`'s
    // `damage`, 2 against 3; `0/2002`, `0/2005` part on 4292..4294, and
    // who=1's `1/112` takes another move on 4295 (`docs/AI.md` §372.3).
    // Item 3760, the Jet Fighter's guns, the Strategic Bomber's bays, the
    // TOW's rows and the Abrams' tracers: 154 → **68**, the standing
    // families alone (`form`, `mirror`, who=1's groups, pools and buckets,
    // `1/14`'s `g.angle[0]`); no building's `damage` parts (§372.4).
    pin_eq!(w.firsts.len(), 68, "run1437b baseline");
    pin_eq!(w.rounds_read, (493, 493), "the rounds read");
    // Every round, both directions: 291 parting. On 4285 the Strategic
    // Bombers' (`1/45`, TI 305; `1/57`, `1/88`) already part on
    // `accuracy`, `sx, sy` and `ex, ey`; on the word's frame 4291 the TOW
    // `1/17`'s (TI 141) `ammo[6]` leaves its own square, (5832, 31320,
    // 384) against (5742, 31270, 531), and `1/18`'s on 4302 the same.
    // Item 3760: (493, 504) → (493, 493) read, 291 → **30** parting — the
    // strafers' `angle` and the Bombers' `accuracy` (`fight.rs`'s bomb-arm
    // SEAM), every launch point alike.
    pin_eq!(w.rounds.len(), 30, "the rounds that part");
}

/// run1439: the gap 3903..4285 between run1434b's window and run1437b's,
/// blocks 3897..4292 at the same detail (item 3760) — six blocks before
/// run1434b's last block to the word 4291 and one past it. run1437b opens
/// parted on 4285, and run1409's stride prints no `AMMO`, leader, building
/// or city record past the start.
pub(crate) const RUN1439: &str =
    "gamelog-run1439-greatlakes-american-information-gap-window-3897-4292.txt";

/// The window: block 3897 through block 4292.
pub(crate) const WIDENING_AMERICAN_INFORMATION_GAP_3903_4285: (i64, i64) = (3897, 4292);

/// **The gap's window**: run1439 walked from run1408's start with the
/// recorder on and every round compared (item 3760).
pub(crate) fn american_information_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1439",
        &[(RUN1439, WIDENING_AMERICAN_INFORMATION_GAP_3903_4285.0)],
        WIDENING_AMERICAN_INFORMATION_GAP_3903_4285,
        1,
        &[4292],
        true,
    )
}

#[test]
fn american_information_pair_run1439_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_information_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 395, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3760: **76** keys, the standing families (`form`, `mirror`,
    // who=1's groups 66..71 and pools, its food bucket from 4120, 100
    // short in ours) and nothing else. Before the rows, building
    // `0/2006`'s `damage_frac` parted on 4242, 8 against 11, under the Jet
    // Fighter `1/57`'s splash (`docs/AI.md` §372.4).
    pin_eq!(w.firsts.len(), 76, "run1439 baseline");
    pin_eq!(w.rounds_read, (420, 420), "the rounds read");
    // Every round, both directions: **21** parting, on `angle` (the
    // strafers' rounds: theirs carries the figure's own angle), `accuracy`
    // (the Bombers') and `1/28`'s and `1/33`'s `whom`, `ox` on 3924 and
    // 4123 (ours −1). Before the rows: 424 read in theirs, 106 parting.
    pin_eq!(w.rounds.len(), 21, "the rounds that part");
}

/// **The pair's closing state** (item 3725): run1409's closing block 4321,
/// the whole map's units, walked from run1408's start.
#[test]
fn american_information_pair_run1409_s_closing_state() {
    let _pins = Pins::hold();
    let p = &AMERICAN_INFORMATION;
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
        "American Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4321, "the closing block");
    // Item 3725, first try: 1935 frames past the word 2385 the game has
    // parted — 136 units compared, counts [118, 0, 7, 5, 145, 0, 63],
    // nothing torn or unlinked, ours' `1/128`..`1/134` extra. A floor, not
    // a score.
    pin_eq!(e.compared, 136, "units compared");
    // Item 3758, piece 204's pivot rows: [59, 0, 0, 7, 3, 1, 0] → [4, 0,
    // 0, 0, 3, 0, 1]; four of who=1's units off (`1/88`, `1/112`, `1/124`,
    // `1/127`) and three buildings' `damage`. Item 3760, the pair whole:
    // [4, 0, 0, 0, 3, 0, 1] → [0, 0, 0, 0, 0, 0, 1] — every unit and
    // building agrees; the one city row is `1/2055`'s
    // `assimilation_timer`, 4320 against 4317 (3742c's family).
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}

/// **An American aircraft costs a fifth less** (item 3755):
/// `get_cost:268`–`282`, an American leader (`has_tribe_bonus(0x14)`)
/// takes `AMERICANS_AIRCRAFT` (20) off a type trained at the Airbase
/// (`0x1bf`), again off one that `is(0x15f, 0)` (the Aircraft Carrier's
/// lineage) and again off one that `is(0x134, 0)` (the Fighter Bomber's),
/// each its own truncating step. Priced on run1408's start for who=1, the
/// Americans: wealth and oil, the two resources an aircraft costs.
///
/// Made to fail first: with `AMERICANS_AIRCRAFT` at 0, who=1 pays the
/// whole price (the right-hand column).
#[test]
fn american_information_pair_an_american_aircraft_costs_a_fifth_less() {
    let Some(inst) = install() else {
        return;
    };
    let Some(path) = dump(AMERICAN_INFORMATION.start) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = replay_initial(&log).unwrap();
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let sim = &mut built.sim;
    let types: Vec<_> = [
        ("Jet Fighter", 0x127),
        ("Fighter Bomber", 0x134),
        ("Aircraft Carrier", 0x15f),
    ]
    .into_iter()
    .map(|(name, ti)| {
        let ty = loaded.unit_named(name).expect(name);
        let tree = sim.unit_types[ty].tree.expect("in the tree");
        assert_eq!(loaded.type_index(tree), ti, "{name}'s TypeIndex");
        (name, ty)
    })
    .collect();
    let read = |sim: &sim::Sim| -> Vec<_> {
        types
            .iter()
            .map(|&(name, ty)| {
                let p = sim.price_of(1, ty);
                (name, sim.american_aircraft_discounts(1, ty), p[2], p[5])
            })
            .collect()
    };
    let with = read(sim);
    sim.tuning.americans_aircraft = 0;
    let without = read(sim);
    // The Fighter Bomber takes one step, not two: its trainer in the
    // tree is not the Airbase, so only its own lineage's test enters.
    assert_eq!(
        with,
        [
            ("Jet Fighter", [20, 0, 0], 72, 72),
            ("Fighter Bomber", [0, 0, 20], 72, 72),
            ("Aircraft Carrier", [0, 20, 0], 520, 520),
        ]
    );
    assert_eq!(
        without,
        [
            ("Jet Fighter", [0, 0, 0], 90, 90),
            ("Fighter Bomber", [0, 0, 0], 90, 90),
            ("Aircraft Carrier", [0, 0, 0], 650, 650),
        ]
    );
}
