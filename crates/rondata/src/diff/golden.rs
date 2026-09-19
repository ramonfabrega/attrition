//! The golden record in the harness: chapter one, staged and walked.
//!
//! `docs/DECISIONS.md` entry 41 §1 and `docs/INPUT.md` §11. The rules
//! track's capture is one game with the Leader AI silenced, driven from
//! `tools/gamelog/golden/chapter1.cmd`; [`crate::golden::Script`] is the
//! interpreter and this is where its output is measured against the
//! original's own trace.
//!
//! **What "the golden word" is measured on, stated because it is not the
//! long captures' basis.** The golden record's `GAMEINFO` is byte-identical
//! to run11's — map 14, seed 12345, size 2, every option field — and its
//! setup checksum word is the same `0x3bd39ae9`, because the script's first
//! line runs at frame 0's `do_frame` entry, *after* `Game::init`. So:
//!
//! - the **setup** is shared and its checksum trace is borrowed from the
//!   siblings, which is what seeds the stream entering frame 0
//!   ([`the_golden_record_s_setup_is_run11_s`] asserts the borrowed word is
//!   this run's own, against the run's own trace);
//! - the **frames** are not, and `frame_seeds`/`frame_guys` are refused.
//!   `borrow_from_siblings` cannot tell the two apart — no gate on the map,
//!   the lobby or the setup word can distinguish a staged run from an
//!   unstaged one — so the caller refuses them by hand.
//!
//! Without that, fourteen of run12/run13's per-frame words are installed by
//! [`Built::tick`] at frames 0–3 and 94–103 and the early comparison says
//! nothing. It does not move any *pinned* number — see
//! [`refusing_the_borrowed_frame_stream_does_not_move_great_lakes_word`] —
//! but it would have made this one meaningless.

use super::*;
use crate::diff::testkit::*;
use crate::gamelog::{Initial, Log};
use crate::golden::Script;

/// A golden run's dump and trace, from `$RON_GOLDEN_DIR` (default
/// `~/ron-golden`). The captures live outside the `Logs` archive because
/// entry 41 §2 regenerates rather than stores them; `docs/RUNS.md`
/// run101–run105 has the command that makes each one in about half a
/// minute.
fn golden(run: &str) -> Option<(String, String)> {
    let home = std::env::var("HOME").ok()?;
    let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
    let dump = format!("{dir}/{run}/map-14/gamelog.txt");
    let trace = format!("{dir}/{run}/map-14/rontrace.log");
    (std::path::Path::new(&dump).is_file() && std::path::Path::new(&trace).is_file())
        .then_some((dump, trace))
}

/// The golden record stood up: the dump's own state, the siblings' map, the
/// run's own pasture, and **no borrowed frame stream**.
fn stand_up(
    loaded: &crate::load::Loaded,
    log: &Log<'_>,
    refs: &[&Initial<'_>],
    trace: &crate::trace::Trace,
) -> Built {
    let mut init = log.initial().expect("the golden dump has a BEGIN GAME");
    let own_seeds = init.frame_seeds.clone();
    let own_guys = init.frame_guys.clone();
    borrow_from_siblings(&mut init, refs);
    init.frame_seeds = own_seeds;
    init.frame_guys = own_guys;
    borrow_pasture(&mut init, trace);
    build_sim(loaded, &init, Tuning::RON)
}

fn siblings(texts: &[String]) -> Vec<Log<'_>> {
    texts.iter().map(|t| Log::parse(t)).collect()
}

/// The script chapter one is staged from, read from the tree.
fn chapter_one() -> Script {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/gamelog/golden/chapter1.cmd"
    );
    Script::read(std::path::Path::new(path)).expect("tools/gamelog/golden/chapter1.cmd")
}

/// **The setup is shared, and this is what says so.** The golden capture
/// carries no `game_random` record of its own, so `borrow_from_siblings`
/// hands it run11's setup trace whole. That is only legitimate if the two
/// runs' setups really are one — and the run's own instrument settles it:
/// the borrowed trace's last word must be the word the golden run's trace
/// records at its first frame-0 draw.
#[test]
fn the_golden_record_s_setup_is_run11_s() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let own = log.initial().unwrap().checksums.len();
    assert_eq!(
        own, 0,
        "the golden capture now carries its own checksum trace; the borrow \
         below is no longer what seeds it"
    );
    let built = stand_up(&loaded, &log, &refs, &trace);
    let theirs = trace
        .frame_draws(0)
        .first()
        .map(|d| d.seed)
        .expect("the golden trace has a frame-0 draw");
    assert_eq!(
        built.sim.rng.seed, theirs,
        "the borrowed setup trace does not end on the golden run's own \
         frame-0 word: the two setups are not one game and nothing may be \
         borrowed from the siblings"
    );
    assert_eq!(
        built.frame_seeds.len(),
        0,
        "the golden run took a sibling's per-frame words"
    );
}

/// **Chapter one, pinned.** The script staged into the harness and the whole
/// 900 frames walked against the golden run's own trace: the frame the draw
/// stream parts is the golden word, and the handoff's `Golden:` line states
/// it.
///
/// `GOLDEN_WORD_CHAPTER_ONE` carries the history.
#[test]
fn chapter_one_holds_to_the_golden_word() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    built.sim.trace_phases = true;
    let mut script = chapter_one();
    // Six in the file; the staged `rontrace.cmd` is eight, because
    // `golden_capture.sh` adds `37 !ffwd 1` and `900 !quit` — the capture's
    // own staging, which changes nothing the simulation models
    // (`docs/RUNS.md` run101–run105's receipt).
    assert_eq!(
        script.lines().len(),
        6,
        "chapter one is six staged lines (tools/gamelog/golden/chapter1.cmd)"
    );
    let last = trace.frames.last().map_or(0, |(n, _)| *n);
    assert!(
        last >= 900,
        "the golden trace is {last} frames; chapter one is 901"
    );
    let mut applied = crate::golden::Applied::default();
    // `game_random`'s word at each frame's entry, ours — the value diff's
    // side of the comparison, taken as the walk runs.
    let mut words: Vec<(i64, u32)> = Vec::new();
    for _ in 0..last {
        words.push((built.sim.frame, built.sim.rng.seed));
        let did = script.stage(built.sim.frame, &mut built, &loaded);
        applied.merge(&did);
        built.tick();
    }
    eprintln!(
        "chapter one staged: {} line(s) ran, {} unit(s), {} building(s); \
         {} carried and not acted on",
        applied.ran,
        applied.units,
        applied.buildings,
        applied.skipped_total()
    );
    for ((word, why), n) in &applied.skipped {
        eprintln!("  {n:5} {word}: {why}");
    }

    let word = built
        .frame_sites
        .iter()
        .find(|(f, ours)| ours.len() != trace.labels(*f).len())
        .map(|(f, _)| *f)
        .unwrap_or(last);
    let sequence = built
        .frame_sites
        .iter()
        .find(|(f, ours)| *ours != trace.labels(*f))
        .map(|(f, _)| *f)
        .unwrap_or(last);
    // **The value diff beside the draw stream**, which is the whole reason
    // the word is not reported alone: `game_random`'s word at each frame's
    // entry, ours against the trace's own.
    let value = trace
        .frames
        .iter()
        .find(|(f, theirs)| {
            words
                .iter()
                .find(|(n, _)| n == f)
                .is_some_and(|(_, ours)| ours != theirs)
        })
        .map(|(f, _)| *f);
    eprintln!(
        "golden chapter one: word parts at {word}, sequence at {sequence}, values at {value:?}"
    );
    for f in [word, sequence] {
        let Some((_, ours)) = built.frame_sites.iter().find(|(n, _)| *n == f) else {
            continue;
        };
        let theirs = trace.labels(f);
        eprintln!("  frame {f}: ours {} theirs {}", ours.len(), theirs.len());
        if let Some((_, shown)) = first_parting(ours, &theirs) {
            eprintln!("{shown}");
        }
    }
    assert!(
        word >= GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word fell to {word} from {GOLDEN_WORD_CHAPTER_ONE}"
    );
    assert_eq!(
        word, GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word moved; re-pin it here and on the \
         handoff's `Golden:` line together"
    );
}

/// **The squad is seated where the original seats it** — six coordinates
/// the golden dump prints for itself, and the one oracle this crate has
/// for `Objects::init_unit`'s `find_nearby_spot` ring (`docs/ANIM.md`
/// §6.3, `docs/INPUT.md` §11.5).
///
/// The second captain's is the assertion that matters. `add hoplite
/// who=0 4,40` and `who=1 5,40` ask for points one tile apart; with the
/// first squad stacked on its captain the near ground stays free and the
/// second `add` lands at `(1080, 7800)`, where the original — whose first
/// squad is spread over three points — is pushed out to `(1368, 7992)`,
/// eighteen tiles from where a stacked crate puts it. So this fails on
/// the *first* squad's seating and on the *second* squad's spot alike,
/// which is why it is one test and not two.
#[test]
fn chapter_one_s_two_squads_are_seated_where_the_dump_says() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter_one();
    // Far enough for both `add` lines (610 and 615) and no further: the
    // units walk from 616 on, and this is about where they are *born*.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    // The dump's own `GUY` blocks for frames 616 and 617, `who`/`o`/`x`/`y`
    // (`docs/RUNS.md` run101–run105). `o` 6, 7, 8 are the three members of
    // each `add`, threaded `down`/`up` in that order.
    let want: [(u8, i16, (i32, i32)); 6] = [
        (0, 6, (888, 7800)),
        (0, 7, (1032, 7800)),
        (0, 8, (936, 7944)),
        (1, 6, (1368, 7992)),
        (1, 7, (1512, 7992)),
        (1, 8, (1416, 8136)),
    ];
    let mut got = Vec::new();
    for (who, o, _) in want {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == who && built.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"));
        let p = built.sim.units[u].pos;
        got.push((who, o, (p.x, p.y)));
    }
    assert_eq!(
        got,
        want.to_vec(),
        "the staged squads are not seated where the golden dump puts them"
    );
}

/// **The engagement frame's six range verdicts** — `docs/COMBAT.md`
/// §12.5, and the one thing the golden record's frame 615 turns on.
///
/// Both squads are seated by frame 615 and the six cross-player pairs sit
/// between 198 and 339 position units apart. The original's answer is
/// printed in its own dump and is not symmetric-looking: **`1/6` and `0/7`
/// strike each other from their seats and the other four walk.** `1/6`
/// carries `in_range 1` and `recharging 32` at dump-617 without having
/// moved off `(1368, 7992)`; `0/7` the same at dump-619 off `(1032,
/// 7800)`; `1/7`, `1/8`, `0/6` and `0/8` are all on the march by 619 with
/// `in_range 0`.
///
/// That is `is_in_range@006486b0`'s melee arm asking the attacker `is(0x84,
/// 0)` and taking `0xf6` rather than `0x66`. With the `0x66` reading —
/// which is what this crate had, `combat::in_range`'s `hoplites` argument
/// being hard-wired `false` — the two `true` rows come back `false` and
/// **nobody in chapter one ever swings**; the four `false` rows are right
/// either way, which is exactly why only the whole set is an oracle.
///
/// The distances are the assertion as much as the verdicts: a reach that
/// happened to be right for the wrong extent would pass the second column
/// and fail the first.
#[test]
fn chapter_one_s_captains_reach_each_other_and_nobody_else() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter_one();
    // Frame 615 stepped: both `add` lines are in, the squads are seated,
    // and nothing has walked off its seat yet.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    // (attacker, target, attack_dist, is_in_range). Every pair that
    // crosses the two squads, in `who`/`o` order.
    /// (who, o) — the pair the dump names a unit by.
    type Who = (u8, i16);
    /// attacker, target, `attack_dist`, `is_in_range`.
    type Reach = (Who, Who, i32, bool);
    let want: [Reach; 6] = [
        ((1, 6), (0, 6), 339, false),
        ((1, 6), (0, 7), 198, true),
        ((1, 6), (0, 8), 288, false),
        ((1, 7), (0, 7), 339, false),
        ((1, 8), (0, 7), 316, false),
        ((0, 7), (1, 6), 198, true),
    ];
    let mut got = Vec::new();
    for (a, t, _, _) in want {
        let (ua, ut) = (find(&built, a.0, a.1), find(&built, t.0, t.1));
        let (oa, ot) = (sim::combat::Obj::Unit(ua), sim::combat::Obj::Unit(ut));
        got.push((
            a,
            t,
            built.sim.attack_dist(oa, ot),
            built.sim.is_in_range(oa, ot),
        ));
    }
    assert_eq!(
        got,
        want.to_vec(),
        "the engagement frame's reach is not the original's: the HOPLITES \
         line takes 0xf6 where everything else takes 0x66 \
         (docs/COMBAT.md §13.2)"
    );
}

/// **The captain's pick, and the only measurement that could have named
/// it** — `docs/COMBAT.md` §18, item 386.
///
/// `1/6`'s three candidates are three identical undamaged hoplites at 198,
/// 288 and 339, and §12.2's score puts all three in the same `/ 0xc0`
/// bucket, so the pick is decided entirely inside `compare_target`. The
/// dump cannot say how: `near_o` records the **nearest** candidate, not the
/// winner, and the order that lands records only the winner. The trace's
/// call proxies can, and run108 does (`docs/RUNS.md`):
///
/// ```text
/// attack_dist o=8 = 288 · compare_target o=8 in_range=1 ai=1 =  2155
/// attack_dist o=7 = 198 · compare_target o=7 in_range=1 ai=1 = 10771
/// attack_dist o=6 = 339 · compare_target o=6 in_range=1 ai=1 =  2155
/// find_nearby_target max_dist=4608 add_order=1 cavarch=0 flags=0 = 7
/// ```
///
/// The candidates arrive in the cell's `down` order (`0/8` first), all
/// three carry `in_range = 1`, and the two the attacker cannot reach come
/// back at a **fifth** of the one it can. That is `compare_target`'s own
/// `is_in_range` call at `0064f1ed`, which this crate had inverted: it
/// divided when the *caller* said out of range, where the original divides
/// when the caller says in range and its **own** test disagrees.
///
/// The assertion is the pick and the shape of the three values, not their
/// absolute size — the trace's `ai=1` branch (`v /= dmg`, the AI
/// multipliers) is not modelled here, so the crate's own numbers differ
/// while the factor of five does not.
#[test]
fn chapter_one_s_captain_picks_the_one_it_can_reach() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101–run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter_one();
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        sim::combat::Obj::Unit(
            (0..b.sim.units.len())
                .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
                .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines")),
        )
    };
    let captain = find(&built, 1, 6);
    // **The frame's own answer**, not a re-run: the search ran inside tick
    // 615 and left its target on the unit, and a second call here would
    // see the `targeted` bump the first one made. The dump's `1/6` reads
    // `ox 7 whom 0 uid 14` under frame 616, and so do `1/7` and `1/8` —
    // `Group::target_opportunity` hands the captain's find to the squad
    // (`docs/GROUPS.md` §13), which this crate does not model, so only the
    // captain's is asserted.
    let sim::combat::Obj::Unit(ci) = captain else {
        unreachable!()
    };
    assert_eq!(
        built.sim.units[ci].combat.target,
        Some(find(&built, 0, 7)),
        "the golden record's `1/6` attacks `0/7`; the candidates arrive \
         `0/8`, `0/7`, `0/6` in the cell's `down` order and a tie would \
         keep the first (docs/COMBAT.md §18)"
    );
    // The three values the pick rests on, in the trace's own candidate
    // order. `in_range` is the search's permission-to-test, `true` for
    // every candidate of a non-guarding AGGRESSIVE unit.
    let v: Vec<i32> = [(0, 8), (0, 7), (0, 6)]
        .into_iter()
        .map(|(w, o)| built.sim.compare_target(captain, find(&built, w, o), true))
        .collect();
    assert_eq!(
        v[0], v[2],
        "the two candidates out of reach must value alike: {v:?}"
    );
    assert!(
        v[1] > v[0],
        "the reachable candidate must outvalue the two out of reach — \
         `compare_target`'s `/5` at `0064f1ed`: {v:?}"
    );
    // **And the falsifier.** `in_range = false` is the caller saying "do
    // not test", which is the arm the old reading took for every
    // candidate: the three then come back equal, the score's `/ 0xc0`
    // bucket is 3 for all three, and the tie hands the pick to whichever
    // the cell's `down` chain reached first — `0/8`. One bit of one
    // predicate is the whole distance between this frame and the
    // original's.
    let flat: Vec<i32> = [(0, 8), (0, 7), (0, 6)]
        .into_iter()
        .map(|(w, o)| built.sim.compare_target(captain, find(&built, w, o), false))
        .collect();
    assert!(
        flat[0] == flat[1] && flat[1] == flat[2],
        "undiscounted, three identical hoplites must tie — that tie is why \
         reading `in_range` as the verdict cost the frame: {flat:?}"
    );
}

/// **The hit is the captain's** (item 391, `docs/COMBAT.md` §18.2). `1/6`
/// strikes `0/7` during frame 616; at that frame's end the golden dump has
/// the ATTACKORDER on `0/6` — `0/7`'s captain, the head of who=0's `o_up`
/// chain — reading `ox 6 whom 1 uid 12`, while `0/7` and `0/8` carry no
/// order at all and take theirs a frame later.
///
/// Both halves are asserted, because answering on the victim also passes
/// "somebody retaliated": the captain has the attacker as its target and
/// neither of the two members has anything. Until item 391 this crate
/// answered on `0/7` itself, which struck `1/6` on 617 rather than 618 and
/// spent the golden word's two missing draws at 617 — `Unit::fight+0x9b0`
/// and `Guy::set_anim+0xf2f < Guy::move+0x166` — on the wrong frames.
#[test]
fn chapter_one_s_hit_is_answered_by_the_victim_s_captain() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101-run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter_one();
    // Through the end of frame 616 — the dump block labelled 617.
    for _ in 0..617 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    let striker = sim::combat::Obj::Unit(find(&built, 1, 6));
    let (cap, hit, other) = (find(&built, 0, 6), find(&built, 0, 7), find(&built, 0, 8));
    // The `o_up` chain the dump prints for who=0: `0/6` is `o_up -1`, and
    // `0/7`/`0/8` hang off it (`docs/COMBAT.md` §18.2).
    assert!(
        built.sim.units[cap].captain
            && !built.sim.units[hit].captain
            && !built.sim.units[other].captain,
        "who=0's squad is one captain and two members"
    );
    assert_eq!(
        (
            built.sim.units[cap].combat.target,
            built.sim.units[hit].combat.target,
            built.sim.units[other].combat.target,
        ),
        (Some(striker), None, None),
        "the dump's frame 617 has the ATTACKORDER on `0/6` alone"
    );
}

/// **The squad is handed its captain's target, and the dump's own
/// coordinates are the check** (item 395, `docs/COMBAT.md` §21).
///
/// Two stages of one walk, both against the golden dump:
///
/// - **block 616**, the end of frame 615 — the frame who=1's squad is born.
///   All three carry `type 10 ox 7 whom 0`: the captain `1/6` searched
///   (`near_o 7 / near_who 0`) and `1/7`/`1/8` read `near_o -1` and hold its
///   answer on the same frame, because `Unit::think`'s first statement
///   mirrors the captain and `1/6` is processed first.
/// - **block 618**, the end of frame 617 — the three ordered destinations
///   §19's ring is answered from. `1/7` to `(1320, 7800)` is the one that
///   moved: this crate had it at `(1176, 7944)` because `1/7` searched for
///   itself and found `0/6`, and two of the six cells the original's ring
///   rejects hang on that point. `0/6`'s own `(1080, 8280)` is the
///   consequence and the golden word's frame.
///
/// Made to fail on purpose by letting the members search: with the mirror
/// removed `1/7` takes `0/6` and is ordered to `(1176, 7944)`, `0/6` walks
/// to `(1224, 7704)`, and the word falls to 619.
#[test]
fn chapter_one_s_squad_is_handed_its_captain_s_target() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture (see docs/RUNS.md run101-run105)");
        return;
    };
    let trace = crate::trace::Trace::read(std::path::Path::new(&tracepath))
        .expect("a finalized golden trace")
        .expect("missing RONT header");
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&dump);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    if refs.is_empty() {
        eprintln!("skipping: no sibling dumps");
        return;
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter_one();
    let find = |b: &Built, who: u8, o: i16| {
        (0..b.sim.units.len())
            .find(|&u| b.sim.units[u].owner == who && b.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter one's two `add` lines"))
    };
    // Through the end of frame 615 — the dump block labelled 616.
    for _ in 0..616 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let hit = sim::combat::Obj::Unit(find(&built, 0, 7));
    let (c, m1, m2) = (find(&built, 1, 6), find(&built, 1, 7), find(&built, 1, 8));
    assert!(
        built.sim.units[c].captain && !built.sim.units[m1].captain && !built.sim.units[m2].captain,
        "who=1's squad is one captain and two members"
    );
    assert_eq!(
        (
            built.sim.units[c].combat.target,
            built.sim.units[m1].combat.target,
            built.sim.units[m2].combat.target,
        ),
        (Some(hit), Some(hit), Some(hit)),
        "the dump's block 616 has `ox 7 whom 0` on all three of who=1"
    );
    // On to the end of frame 617 — block 618, and the ordered points.
    for _ in 616..618 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    let at = |u: usize| {
        let p = built.sim.units[u].orders_pos;
        (p.x, p.y)
    };
    assert_eq!(
        (at(m1), at(m2), at(find(&built, 0, 6))),
        ((1320, 7800), (1176, 8088), (1080, 8280)),
        "block 618's `orders_x, orders_y` for `1/7`, `1/8` and `0/6`"
    );
}

/// **Great Lakes' word does not rest on the borrowed frame stream.** Five
/// Great Lakes captures share run11/run12/run13's setup word and therefore
/// take fourteen of their per-frame words (frames 0–3, 94–103), which
/// [`Built::tick`] installs; East Indies' setup word differs and it takes
/// none. This walks run53's whole 24,000 frames both ways and asserts the
/// word is the same number — the install is a no-op, because the simulation
/// already produces the original's word at each of those fourteen frames.
///
/// It is here rather than beside run53's own test because the question is
/// the golden record's: the same mechanism is what made this file's first
/// measurement an artefact (item 364).
#[test]
fn refusing_the_borrowed_frame_stream_does_not_move_great_lakes_word() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(tr)) = (
        crate::testenv::dump("gamelog-run53-greatlakes-24k-trace.txt"),
        trace("rontrace-run53.log"),
    ) else {
        eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let texts = sibling_texts();
    let logs = siblings(&texts);
    let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = inits.iter().collect();
    let last = tr.frames.last().map_or(0, |(n, _)| *n);
    let mut answers = Vec::new();
    for refuse in [false, true] {
        let mut init = log.initial().unwrap();
        let own_seeds = init.frame_seeds.clone();
        let own_guys = init.frame_guys.clone();
        borrow_from_siblings(&mut init, &refs);
        if refuse {
            init.frame_seeds = own_seeds.clone();
            init.frame_guys = own_guys.clone();
        }
        let installs = init.frame_seeds.len();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..last {
            built.tick();
        }
        let count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != tr.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let sequence = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != tr.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        eprintln!(
            "run53 refuse={refuse}: {installs} installed frame word(s); word parts at \
             {count}, sequence at {sequence}"
        );
        answers.push((installs, count, sequence));
    }
    // The anti-vacuity half: the borrow must actually have fired, or the
    // two columns are the same run twice.
    assert_eq!(
        (answers[0].0, answers[1].0),
        (14, 0),
        "run53 no longer takes the siblings' fourteen per-frame words, so \
         this test compares nothing"
    );
    assert_eq!(
        (answers[0].1, answers[0].2),
        (answers[1].1, answers[1].2),
        "Great Lakes' word moves when the borrowed frame stream is refused: \
         the pinned number rests on another capture's per-frame data"
    );
}
