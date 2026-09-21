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

/// A chapter's script, read from the tree by number.
fn chapter(n: u32) -> Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/chapter{n}.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/chapter{n}.cmd: {e}"))
}

/// What one chapter's walk measured: the three frames a chapter is scored
/// on, and the staging that produced them.
///
/// **The word is the draw stream's count**, `sequence` the first frame the
/// labels part in order, and `value` `game_random`'s own word at a frame's
/// entry — reported together because a draw stream can agree on a wrong
/// destination for a long time and only a value comparison tells the two
/// apart (`CLAUDE.md`, the working agreement).
struct Walk {
    word: i64,
    sequence: i64,
    value: Option<i64>,
}

/// **Stage a chapter into the harness and walk it against its own trace.**
///
/// One body for every chapter: chapter one had it inline, and chapter two
/// (item 415) is the reason it is a function. `run` is the directory under
/// `$RON_GOLDEN_DIR`, `n` the chapter number, and `staged` the number of
/// lines the `.cmd` file itself carries — asserted, because the capture
/// adds `!ffwd` and `!quit` of its own and a miscount there is a script
/// nobody staged.
fn walk_chapter(run: &str, n: u32, staged: usize, length: i64) -> Option<Walk> {
    let inst = crate::testenv::install()?;
    let Some((dump, tracepath)) = golden(run) else {
        eprintln!("skipping: no golden capture {run} (see docs/RUNS.md)");
        return None;
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
        return None;
    }
    // **A chapter captured without its `[Start Game]` set is not a
    // measurement** (item 415). `borrow_from_siblings` lends a capture the
    // map it could not print, so a dump whose start block is only the
    // world's seventeen scalars still stands *something* up — a simulation
    // with no leaders and no units, which walks and scores and reports a
    // word of 0 that looks exactly like a real one. run112's first attempt
    // was that: `end:` and `misc:` given, `start:` forgotten, 0 LEADERDATA
    // and 0 UNITDATA where run105 has 4 and 52, and the harness spent 80
    // draws at frame 0 against the trace's 120. The same shape as the
    // endpoint module's run28 note, and as item 364's borrowed frame
    // stream: a number that is the setup's, not the simulation's.
    let own = log.initial().expect("a start block");
    assert!(
        !own.leaders.is_empty() && !own.units.is_empty(),
        "golden capture {run} has {} leader(s) and {} unit(s) in its \
         `[Start Game]` block: the capture was taken without a `--detail \
         start:…` line and there is nothing to stand up. Re-take it with \
         run105's set — `--detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,\
         UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1`",
        own.leaders.len(),
        own.units.len()
    );
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    built.sim.trace_phases = true;
    let mut script = chapter(n);
    assert_eq!(
        script.lines().len(),
        staged,
        "chapter {n} is {staged} staged lines (tools/gamelog/golden/chapter{n}.cmd)"
    );
    let last = trace.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last >= length,
        "the golden trace is {last} frames; chapter {n} is {}",
        length + 1
    );
    let mut applied = crate::golden::Applied::default();
    let mut words: Vec<(i64, u32)> = Vec::new();
    for _ in 0..last {
        words.push((built.sim.frame, built.sim.rng.seed));
        let did = script.stage(built.sim.frame, &mut built, &loaded);
        applied.merge(&did);
        built.tick();
    }
    eprintln!(
        "chapter {n} staged: {} line(s) ran, {} unit(s), {} building(s); \
         {} carried and not acted on",
        applied.ran,
        applied.units,
        applied.buildings,
        applied.skipped_total()
    );
    for ((word, why), k) in &applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
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
    let value = trace
        .frames
        .iter()
        .find(|(f, theirs)| {
            words
                .iter()
                .find(|(k, _)| k == f)
                .is_some_and(|(_, ours)| ours != theirs)
        })
        .map(|(f, _)| *f);
    eprintln!(
        "golden chapter {n}: word parts at {word}, sequence at {sequence}, values at {value:?}"
    );
    for f in [word, sequence] {
        let Some((_, ours)) = built.frame_sites.iter().find(|(k, _)| *k == f) else {
            continue;
        };
        let theirs = trace.labels(f);
        eprintln!("  frame {f}: ours {} theirs {}", ours.len(), theirs.len());
        if let Some((_, shown)) = first_parting(ours, &theirs) {
            eprintln!("{shown}");
        }
    }
    Some(Walk {
        word,
        sequence,
        value,
    })
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

/// **Chapter one, pinned.** The script staged into the harness and the
/// whole 900 frames walked against the golden run's own trace: the frame
/// the draw stream parts is the golden word, and the handoff's `Golden:`
/// line states it.
///
/// `GOLDEN_WORD_CHAPTER_ONE` carries the history. Six staged lines in the
/// file; the `rontrace.cmd` the capture writes is eight, because
/// `golden_capture.sh` adds `37 !ffwd 1` and `900 !quit` of its own, which
/// change nothing the simulation models (`docs/RUNS.md` run101–run105).
#[test]
fn chapter_one_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("g4", 1, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word fell to {} from {GOLDEN_WORD_CHAPTER_ONE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_ONE,
        "chapter one's golden word moved; re-pin it here and on the \
         handoff's `Golden:` line together"
    );
}

/// **Chapter two, pinned** — the ranged line and the ammunition
/// (`docs/GOLDEN.md` §6, item 415, run112).
///
/// **Its own constant, and not a composition.** `docs/DECISIONS.md` 41 §1
/// gives the rules track one `Golden:` line, and how that line reads once
/// a second chapter pins is the steering pass's (parked 417): until then
/// the handoff carries chapter one's word and every further chapter pins
/// here beside it. The commander's ruling is lowest-chapter-first, which
/// matches the AI track's lower-map-first rule.
///
/// **What the capture established before this walk ever ran**, each of
/// them a prediction written into `chapter2.cmd`'s header first so that
/// the run could fail (`docs/RUNS.md`, run112): eight `INFO cmd` records
/// every one returning 1; 296 window blocks, 605..899 with no gap and the
/// `!quit` block at 901; nine units born three at a time at 611, 616 and
/// 621; and **373 `AMMO` blocks over 186 frames**, which is the falsifier
/// the chapter exists for — no arrow at all would have said the ranged arm
/// was never entered.
///
/// `GOLDEN_WORD_CHAPTER_TWO` carries what stands at the word.
#[test]
fn chapter_two_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch2", 2, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_TWO,
        "chapter two's golden word fell to {} from {GOLDEN_WORD_CHAPTER_TWO}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_TWO,
        "chapter two's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §6"
    );
    // The value diff beside the draw stream, printed rather than pinned
    // until it is the thing an item is taken on.
    eprintln!("chapter two: sequence {}, values {:?}", w.sequence, w.value);
}

/// **Chapter two's squads are seated exactly where the dump seats them,
/// and the 140 units item 415 called a seating error are five frames of
/// marching** (item 441's widening).
///
/// 415 walked to 622, found who=1's squad 140 units west of the dump's own
/// cells on all three figures, and read it as `Objects::init_unit`'s
/// `find_nearby_spot` ring going wrong on clear ground. It is not.
/// Widening every record the dump carries over 612–640 shows **no
/// divergence of any kind before 616**, the seating exact at 616 and 617,
/// and the positions parting only from 618 — after which this crate's
/// squad walks west at 28 units a frame while the original's never moves
/// at all. `622 − 617` is five frames and `5 × 28` is 140. The number was
/// right; the mechanism was invented.
///
/// What is wrong is upstream and is one order: at 616 this crate gives the
/// captain an `ATTACK`, at 617 pushes a `MOVE_TO` in front of it — the
/// chase — and at 618 a path appears and the squad sets off. The
/// original's hoplites hold **no order at all** through 634. So this test
/// asserts the two halves separately: the seating is the dump's, and the
/// drift is arithmetic.
#[test]
fn chapter_two_s_squads_stand_where_the_dump_stands_them() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
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
    let mut script = chapter(2);
    // **617, not 622.** Both `add` lines that matter have run and nothing
    // has moved yet on either side; 622 is five frames of this crate's own
    // marching later, which is what item 415 mistook for a seating error.
    for _ in 0..617 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    // The dump's own coordinates, frames 616–618, unchanged across them:
    // bowmen 0/6–0/8, hoplites 1/6–1/8.
    let dump: [(u8, i16, (i32, i32)); 6] = [
        (0, 6, (888, 7800)),
        (0, 7, (1032, 7800)),
        (0, 8, (936, 7944)),
        (1, 6, (2424, 7800)),
        (1, 7, (2568, 7800)),
        (1, 8, (2472, 7944)),
    ];
    let mut got = Vec::new();
    for (who, o, _) in dump {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == who && built.sim.units[u].index == o)
            .unwrap_or_else(|| panic!("no unit {who}/{o} after chapter two's `add` lines"));
        let p = built.sim.units[u].pos;
        got.push((who, o, (p.x, p.y)));
    }
    // **All six are the dump's own cells at 617.** `add`'s ring is right
    // for both squads; item 415's contrary finding was five frames of
    // marching read at 622.
    assert_eq!(
        got,
        dump.to_vec(),
        "chapter two's squads are not seated where the golden dump seats \
         them at 617"
    );
    // ~~**And the drift is arithmetic.** Five more frames and who=1's
    // squad stands 140 units west on all three figures, at 28 a frame,
    // while the dump holds the original's still: item 415's number, with
    // the mechanism it actually has.~~ **Closed by item 447**
    // (`docs/COMBAT.md` §31): the chase that walked them was an attack
    // order the original never issued, and `is_seen` in `valid_target`
    // retires it. So the assertion inverts — who=1's squad **does not
    // move**, which is the dump's own behaviour through 634, and the 140
    // is a number this crate can no longer produce.
    for _ in 0..5 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    for (who, o, seat) in dump.iter().skip(3) {
        let u = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == *who && built.sim.units[u].index == *o)
            .expect("the squad is still alive");
        let p = built.sim.units[u].pos;
        assert_eq!(
            (p.x, p.y),
            *seat,
            "unit {who}/{o} has walked off its seat by 622; the original's \
             hoplites hold no order at all through 634 and neither should \
             these (docs/COMBAT.md §31)"
        );
    }
    // Anti-vacuity: the gap the chapter is built on. 2424 − 1032 is 1392
    // units, 7.25 tiles, inside the Bowmen's ten and outside the Hoplites'
    // zero — which is the whole premise of the chapter.
    assert_eq!(
        dump[3].2.0 - dump[1].2.0,
        1392,
        "the two squads are not 7.25 tiles apart; chapter two's premise is \
         a ranged squad outside a melee squad's reach"
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
    let mut script = chapter(1);
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
    let mut script = chapter(1);
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
    let mut script = chapter(1);
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
    let mut script = chapter(1);
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
    let mut script = chapter(1);
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

/// **Chapter two's three squads, and the one that engages a frame nobody
/// asked it to** — item 426, the frame the rules headline stands on.
///
/// The dump's own answer, read off run112: the original issues **no**
/// attack order at 616 at all. Its first is at **621**, to the slingers
/// `0/9`–`0/11` on their own birth frame, targeting `ox 8 whom 1`; the
/// bowmen `0/6`–`0/8` and the hoplites `1/6`–`1/8` both take theirs at
/// **635**. This crate matches two of those three exactly and gives the
/// hoplite squad an attack order on its birth frame, 616 — which is the
/// whole of the golden word: the spurious order puts the captain into
/// `do_attack`, and `Unit::fight`'s one-in-five re-search spends the
/// twenty-sixth draw against the original's twenty-five.
///
/// **Two named mechanisms were ruled out by measurement rather than by
/// argument**, which is the item's product as much as the frame is:
///
/// - **The 140-unit seating error is not the cause.** Seated on the
///   dump's own cells — `(2424, 7800)`, `(2568, 7800)`, `(2472, 7944)` —
///   the squad still takes the order at 616.
/// - **`find_melee_target`'s `0x40000` arm is not the cause.** who=1's
///   hoplites do carry `unit_masks 262144` where who=0's carry 0, so the
///   arm fires; disabling it leaves 616 unchanged, because the plain
///   `unit_respond_range * 0xc0` floor already reaches 12 tiles and the
///   bowmen are 1392 units away, 7.25.
///
/// That arithmetic also **exonerates the radius as such**: the original
/// floors a melee searcher the same way (`find_melee_target@005ff9c0`,
/// the `unit_respond_range * 0xc0` line below both arms), so had its
/// hoplite captain searched on its birth frame it would have found the
/// bowmen too.
///
/// **Corrected by item 443** (`docs/COMBAT.md` §30), which is why this
/// comment no longer says it did not search. It did: `near_o` is written
/// above the range test, so `near_o = -1` at 616 says the search found
/// nothing acceptable rather than that none ran, and the bowmen it
/// refused at 616 sit in the *same* object-grid cell as the slinger the
/// same captain accepts at 635 from a seat it never leaves. `think`'s
/// auto-attack gate is ruled out with them: the slinger captain's birth
/// frame is on neither of that gate's grids, so its own search came
/// through the `idle == 1` arm and the hoplite's reached `think_attack`
/// by the same arm. What is left is a target-acceptance predicate, and
/// `chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached`
/// is the measurement that says so.
///
/// The slingers are the control that makes it a measurement: they engage
/// on **their** birth frame in both, at 8.6 tiles, further than the 7.25
/// the hoplites do not engage at. So it is not a distance threshold.
#[test]
fn chapter_two_s_first_attack_orders_are_the_dump_s() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
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
    let mut script = chapter(2);
    // `(who, o)` → the first frame this crate has an attack order on it.
    let mut first: std::collections::BTreeMap<(u8, i16), i64> = std::collections::BTreeMap::new();
    for _ in 0..640 {
        let f = built.sim.frame;
        script.stage(f, &mut built, &loaded);
        for u in 0..built.sim.units.len() {
            let un = &built.sim.units[u];
            if !un.alive() || un.owner > 1 || !(6..=11).contains(&un.index) {
                continue;
            }
            if un
                .orders
                .iter()
                .any(|o| matches!(o.body, sim::orders::Body::Attack(_)))
            {
                first.entry((un.owner, un.index)).or_insert(f);
            }
        }
        built.tick();
    }
    // The dump's own first-`ATTACKORDER` frame per unit, read from run112.
    let theirs: [((u8, i16), i64); 9] = [
        ((0, 6), 635),
        ((0, 7), 635),
        ((0, 8), 635),
        ((0, 9), 621),
        ((0, 10), 621),
        ((0, 11), 621),
        ((1, 6), 635),
        ((1, 7), 635),
        ((1, 8), 635),
    ];
    // Ours. **Pinned in no direction**, and since item 457 it is the
    // dump's own nine, frame for frame.
    //
    // The history is the point. Item 415 had the hoplites ordering at
    // **616**, nineteen frames early and on the wrong mechanism. Item 447
    // put `is_seen` into `valid_target` (`docs/COMBAT.md` §31) and the
    // spurious 616 went away without the dump's 635 arriving, because the
    // other half of `UnitData::is_seen` — `ObjectData::visible` — was not
    // modelled: three rows moved from *wrong and early* to *absent*.
    // Item 457 modelled it (`docs/VISION.md` §7), and the original's
    // `1/6` now accepts `0/10` at 635 here for the reason it does there —
    // the slinger set its own `visible` bit for player 1 at **631** by
    // attacking a hoplite (`Unit::set_attacking@005ff5b0`), and player 1's
    // line of sight never reaches that cell.
    let ours: [((u8, i16), i64); 9] = [
        ((0, 6), 635),
        ((0, 7), 635),
        ((0, 8), 635),
        ((0, 9), 621),
        ((0, 10), 621),
        ((0, 11), 621),
        ((1, 6), 635),
        ((1, 7), 635),
        ((1, 8), 635),
    ];
    let got: Vec<((u8, i16), i64)> = first.into_iter().collect();
    assert_eq!(
        got,
        ours.to_vec(),
        "chapter two's engagement timeline moved. The dump's own is \
         {theirs:?}; re-pin `ours` and say so in docs/GOLDEN.md §6"
    );
    // **All nine agree with the original exactly.** The count is asserted
    // separately from the list because it is the row that reads at a
    // glance, and because a regression that swapped one squad's rightness
    // for another's would still pass a length check.
    let agree = theirs
        .iter()
        .filter(|(k, f)| got.iter().any(|(g, n)| g == k && n == f))
        .count();
    assert_eq!(
        agree,
        theirs.len(),
        "chapter two's engagement timeline no longer matches the dump on \
         every one of its nine figures"
    );
    // The original issues nothing at all on the frame the word parts.
    assert!(
        !theirs.iter().any(|(_, f)| *f == 616),
        "the dump's own timeline now has an attack order at 616"
    );
}

/// One frame's `UNITDATA` records, read out of the raw dump text by hand.
///
/// **Deliberately not through [`crate::gamelog::Log`].** `near_o` and
/// `near_who` are the `ObjectData` half the parser leaves unparsed on
/// purpose — `ledger.rs`'s rule is that a parsed field is a *compared*
/// field, and this crate's `Unit` models neither, so parsing them would
/// buy a comparison against nothing (`docs/COMBAT.md` §30.1). This reader
/// takes the four figures §30 argues on and nothing else.
///
/// First-wins on each key, because `BEGIN GUY` repeats `who`/`o` inside
/// the record and the `SUBOBJECT`'s pair is the unit's own.
fn ch2_dump_units(text: &str, frame: i64) -> std::collections::BTreeMap<(i64, i64), [i64; 4]> {
    let head = format!("BEGIN FRAME {frame}");
    let mut out = std::collections::BTreeMap::new();
    let mut inside = false;
    let mut cur: Option<std::collections::BTreeMap<&str, i64>> = None;
    let flush = |cur: &mut Option<std::collections::BTreeMap<&str, i64>>,
                 out: &mut std::collections::BTreeMap<(i64, i64), [i64; 4]>| {
        if let Some(r) = cur.take() {
            let g = |k: &str| r.get(k).copied().unwrap_or(i64::MIN);
            out.insert(
                (g("who"), g("o")),
                [g("x_internal"), g("y_internal"), g("near_o"), g("idle")],
            );
        }
    };
    for line in text.lines() {
        let s = line.trim();
        if s.starts_with("BEGIN FRAME") {
            if inside {
                break;
            }
            inside = s == head;
            continue;
        }
        if !inside {
            continue;
        }
        if s == "BEGIN UNITDATA" {
            flush(&mut cur, &mut out);
            cur = Some(std::collections::BTreeMap::new());
            continue;
        }
        if s.starts_with("BEGIN ") {
            continue;
        }
        let Some(r) = cur.as_mut() else { continue };
        if let Some((k, v)) = s.rsplit_once(' ')
            && let Ok(n) = v.parse::<i64>()
        {
            r.entry(k).or_insert(n);
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// **The object grid's cell**, `div_3_table[v >> 8]` — floor division by
/// 768, four tiles (`docs/ATTRITION.md`, the units table; `docs/COMBAT.md`
/// §30.3). `Object::find_nearby_target@00648da0` walks *cells*, not
/// distances, so this is the quantum the search is actually coarse in.
fn cell(v: i64) -> i64 {
    v.div_euclid(768)
}

/// **No radius explains 616, and the dump says so in four numbers**
/// (item 443, `docs/COMBAT.md` §30).
///
/// `Object::find_nearby_target@00648da0` scans the object grid by **cell**
/// — `circle_x`/`circle_y` offsets from the searcher's own cell, bounded
/// by `circle_radius[min(0x20, (range + 0x2ff) / 0x300 + bonuses)]` — and
/// it writes `ObjectData::near_o`/`near_who` (+0x34/+0x36) for *every*
/// candidate that clears `valid_target` and `check_target` and is nearer
/// than the best so far, **above and independent of** the range test that
/// decides the order. So `near_o` is the search's footprint: an attack
/// order out of that function implies a `near_o` write, and `near_o = -1`
/// after it means nothing in the scanned cells was acceptable.
///
/// And `Unit::think_attack@005f5a80` passes **-1** on every path into
/// `find_melee_target` — listing-backed, `005f5d86`-`005f5da6`: the
/// AI-driven branch calls `add_to_army` and then `orl $-1, %esi` anyway —
/// so the range is computed from the unit's own stance, reach,
/// `unit_respond_range` and `unit_masks`, every one of them
/// frame-independent, and the `param_1 == 0` grid table is unreachable.
/// **The hoplite captain therefore scanned the same cells at 616 and at
/// 635.**
///
/// The four numbers this asserts:
///
/// | frame | searcher, cell | the cell in question | `near_o` |
/// | --- | --- | --- | --- |
/// | 616 | `1/6` at `(3, 10)` | bowmen `0/6`-`0/8`, `(1, 10)` | **-1** |
/// | 621 | `0/9` at `(1, 10)` | `1/6`, `(3, 10)` | **6** |
/// | 635 | `1/6` at `(3, 10)` | slinger `0/10`, `(1, 10)` | **10** |
/// | 635 | `0/6` at `(1, 10)` | `1/6`, `(3, 10)` | **6** |
///
/// One cell pair, `(1, 10)` <-> `(3, 10)`, traversed by three searches and
/// refused by a fourth — and the fourth is the word. Whatever separates
/// them, it cannot be how far the search reached, because the reach is the
/// same object-grid cell in all four rows. That closes the radius
/// programme of `docs/COMBAT.md` §26-§28 by measurement rather than by
/// argument, and it is why §30 does not propose a successor constant.
#[test]
fn chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached() {
    let Some((dump, _)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let text = crate::capture::read(&dump);
    let at = |f: i64| ch2_dump_units(&text, f);
    let (f616, f621, f635) = (at(616), at(621), at(635));
    assert!(
        !f616.is_empty() && !f621.is_empty() && !f635.is_empty(),
        "run112's dump no longer carries frames 616, 621 and 635"
    );
    // **The searcher never moves.** `1/6` stands on its seat through both
    // of its own searches, so its cell is one number in both rows.
    for f in [&f616, &f621, &f635] {
        let h = f[&(1, 6)];
        assert_eq!(
            (h[0], h[1]),
            (2424, 7800),
            "the hoplite captain has left its seat; §30's cell argument \
             assumes it stands still from 616 to 635"
        );
    }
    let hoplite = (cell(2424), cell(7800));
    assert_eq!(hoplite, (3, 10), "the hoplite captain's object-grid cell");
    // 616: every bowman sits in one cell, and the captain's `near_o` is -1
    // — the search accepted nothing from it.
    for o in 6..=8 {
        let b = f616[&(0, o)];
        assert_eq!(
            (cell(b[0]), cell(b[1])),
            (1, 10),
            "bowman 0/{o} is no longer in the cell §30 argues on"
        );
    }
    assert_eq!(
        f616[&(1, 6)][2],
        -1,
        "the hoplite captain's near_o at 616 is no longer -1; §30's \
         'the search accepted nothing' rests on it"
    );
    // 635: the slinger it *does* accept sits in the same cell the bowmen
    // sat in, and the bowmen are still there.
    let s = f635[&(0, 10)];
    assert_eq!(
        (cell(s[0]), cell(s[1])),
        (1, 10),
        "slinger 0/10 is not in the bowmen's cell at 635; §30's argument \
         is that one cell was reached and refused, then reached and taken"
    );
    assert_eq!(
        f635[&(1, 6)][2],
        10,
        "the hoplite captain's near_o at 635 is no longer 0/10"
    );
    let b7 = f635[&(0, 7)];
    assert_eq!(
        (cell(b7[0]), cell(b7[1])),
        (1, 10),
        "the bowmen have left the cell between 616 and 635, so the two \
         frames are no longer comparable"
    );
    // The two searches that cross the same pair the other way, which is
    // what makes this a measurement and not one unit's oddity.
    assert_eq!(
        (
            f621[&(0, 9)][2],
            (cell(f621[&(0, 9)][0]), cell(f621[&(0, 9)][1]))
        ),
        (6, (1, 10)),
        "the slinger captain's birth-frame search no longer finds 1/6 \
         from the bowmen's cell"
    );
    assert_eq!(
        (
            f635[&(0, 6)][2],
            (cell(f635[&(0, 6)][0]), cell(f635[&(0, 6)][1]))
        ),
        (6, (1, 10)),
        "the bowman captain no longer finds 1/6 from its own cell at 635"
    );
    // **Anti-vacuity.** Both searchers are on the arm that searches: the
    // dump's `idle` is 1 on a birth-frame search and >1 on the grid one,
    // so neither row is a unit that simply never thought.
    assert_eq!(
        (f616[&(1, 6)][3], f621[&(0, 9)][3], f635[&(1, 6)][3]),
        (1, 1, 4),
        "the idle counters that put these three rows on `Unit::think`'s \
         search arm have moved"
    );
}

/// **637's widening, both directions** — every record run112's dump carries
/// over `[633, 641)`, compared whole against this crate's own walk, on the
/// frame chapter two's word now stands (`docs/COMBAT.md` §32.3).
///
/// A word is pinned with its widening (`docs/DECISIONS.md` 43), and the
/// window moves with the word rather than being left naming a frame the
/// word has walked out of — parked 449's lesson, applied at the move. This
/// walked `[620, 628)` while the word stood at 624; item 462 closed that
/// window whole — **zero rows, both directions, on every record** — and
/// the word went to 637, so the window came with it.
///
/// What it asserts is the **shape** of the residue rather than a tally that
/// drifts on every landing. Both directions are counted, so neither a unit
/// this crate has lost nor one it has invented can hide
/// (`FrameResult::extra_units`).
#[test]
fn chapter_two_s_word_frame_is_widened_whole() {
    /// The first frame compared: run112's window opens at 605 and the
    /// slinger squad is born at 621, so this is inside both. Declared in
    /// [`WIDENING_CHAPTER_TWO`] beside the `WIDENINGS` row, so the floors
    /// guard reads the word against the same window this walks.
    const FIRST: i64 = WIDENING_CHAPTER_TWO.0;
    /// One past the last. 624 is the word; three frames either side is
    /// enough to say the parting opens *there* and not before.
    const LAST: i64 = WIDENING_CHAPTER_TWO.1;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
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
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(2);
    // Every key that parted, against the **first** frame it parted on —
    // a residue standing before the window opened is not this word's.
    let mut first: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut blocks = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1), so the block this tick produced
        // is `f + 1`.
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let r = compare(&built, &frame, players);
        blocks += 1;
        let mut note = |k: String| {
            first.entry(k).or_insert(n);
        };
        for d in &r.diverged {
            eprintln!(
                "  {n} pos {}/{} ours {:?} theirs {:?}",
                d.who, d.o, d.ours, d.theirs
            );
            note(format!("pos {}/{}", d.who, d.o));
        }
        for d in &r.order_diverged {
            eprintln!("  {n} order {}/{} {:?}", d.who, d.o, d);
        }
        for d in &r.angle_diverged {
            eprintln!("  {n} angle {}/{} {:?}", d.who, d.o, d);
        }
        for (who, o) in &r.unlinked_units {
            note(format!("unlinked {who}/{o}"));
        }
        for (who, o) in &r.extra_units {
            note(format!("extra {who}/{o}"));
        }
        for d in &r.order_diverged {
            note(format!("order {}/{}", d.who, d.o));
        }
        for d in &r.los_diverged {
            note(format!("los {}/{}", d.who, d.o));
        }
        for d in &r.angle_diverged {
            note(format!("angle {}/{}", d.who, d.o));
        }
        for d in &r.collide_diverged {
            note(format!("collide {}/{}", d.who, d.o));
        }
        for d in &r.search_diverged {
            note(format!("search {}/{}", d.who, d.o));
        }
        for d in &r.packed_diverged {
            note(format!("packed {}/{}", d.who, d.o));
        }
        for d in &r.visible_diverged {
            note(format!("visible {}/{}", d.who, d.o));
        }
    }
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize,
        "run112's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    // **Anti-vacuity**: the window has to hold the cast the chapter is
    // about, or an empty comparison reads as agreement.
    let ninth = built
        .sim
        .units
        .iter()
        .filter(|u| u.owner < 2 && u.index >= 6)
        .count();
    assert_eq!(
        ninth, 9,
        "the window does not hold chapter two's nine staged figures"
    );
    // **The map the widening exists to pin**, and its shape is the
    // finding: the *values* part at **635**, two frames before the draw
    // stream does, and the two earliest rows on that frame are both
    // `Target` — six units on both sides choosing a different one of
    // three identical, near-equidistant figures. `docs/COMBAT.md` §32.3.
    //
    // That is the same residue item 462 closed one squad over and two
    // frames earlier, and it is *not* closed by the cell chain alone:
    // `0/6`, `0/7` and `0/8` take `1/6` where the dump takes `1/8`, and
    // all three hoplites take `0/7` where the dump takes `0/11`. Every
    // later row on 636 is downstream of those — a chase planned at a
    // different target's ring walks a different way.
    //
    // `order 0/5` / `pos 0/5` at 639-640 are a **citizen** far from the
    // engagement and were in no earlier window; they are named here so a
    // regression in them cannot hide behind the engagement, and they are
    // nobody's item yet (`docs/COMBAT.md` §32.5).
    let measured = [
        ("angle 0/6", 636),
        ("angle 0/7", 636),
        ("angle 0/8", 636),
        ("order 0/11", 636),
        ("order 0/5", 639),
        ("order 0/6", 635),
        ("order 0/7", 635),
        ("order 0/8", 635),
        ("order 1/6", 635),
        ("order 1/7", 635),
        ("order 1/8", 635),
        ("pos 0/11", 636),
        ("pos 0/5", 640),
        ("pos 1/6", 636),
        ("pos 1/7", 636),
        ("pos 1/8", 636),
        ("visible 0/11", 637),
    ];
    let got: Vec<(&str, i64)> = first.iter().map(|(k, &n)| (k.as_str(), n)).collect();
    assert_eq!(
        got,
        measured.to_vec(),
        "chapter two's word frame no longer widens the way item 462 \
         measured it; re-pin this map and say so in docs/COMBAT.md §32.3"
    );
}

/// **`ObjectData::visible`'s arrivals against run112's own** — item 457's
/// oracle, and the strongest check this mechanic can have
/// (`docs/VISION.md` §7).
///
/// The field is printed inside the `OBJECT` block at every detail level,
/// so no capture was needed for it: the dump has been carrying the answer
/// since 2026-09-19. Over the chapter it records nine arrivals and five
/// clears, and what this asserts is their **shape** — which unit gains
/// which player's bit, and that it gains it at all — with the frames
/// pinned in no direction beside them.
///
/// The frames cannot be asserted equal and the reason is not this
/// mechanic. `Unit::set_attacking` fires from the tail of `Unit::fight`,
/// so `visible`'s arrival frame is the frame of the unit's *first strike*
/// — and chapter two's engagement timing is still a residue of its own
/// (`docs/COMBAT.md` §31.6). A bit that arrives two frames early here
/// arrives two frames early because the arrow did. Separating the two is
/// the whole point of pinning the shape exactly and the frames loosely:
/// a regression in the **rule** — the wrong bit, the wrong unit, a bit
/// that never comes — fails on the shape, whatever the engagement does.
///
/// The clears are read the same way and for the same reason: the dump
/// drops `0/6`'s byte at 827 and this crate does not, because this
/// crate's bowman is still carrying an `ATTACK` order there and the
/// original's is not. `Unit::work`'s 32-frame slot is exact on both sides
/// — `1/7` clears here at **794**, which is its own `f ≡ 26 (mod 32)` —
/// and what differs is the latch's input.
#[test]
fn chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
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
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(2);
    /// One past the last frame read — the chapter's last logged block.
    const LAST: i64 = 899;
    // `(who, o)` → the first frame the unit's `visible` is non-zero, and
    // the byte it holds there. One entry per side.
    let mut theirs: std::collections::BTreeMap<(i64, i64), (i64, u8)> =
        std::collections::BTreeMap::new();
    let mut ours: std::collections::BTreeMap<(i64, i64), (i64, u8)> =
        std::collections::BTreeMap::new();
    let mut read = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's frame label is the sim frame plus one throughout this
        // chapter (`docs/COMBAT.md` §30.1).
        let n = f + 1;
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        for u in &frame.units {
            if !(0..2).contains(&u.who) {
                continue;
            }
            let Some(v) = u.visible else { continue };
            read += 1;
            if v != 0 {
                theirs.entry((u.who, u.o)).or_insert((n, v as u8));
            }
            // Ours is read on the same unit-frames the dump carries, so
            // that a unit the dump has stopped printing — a dead one —
            // cannot contribute to one side and not the other.
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            let mv = built.sim.units[mine].visible;
            if mv != 0 {
                ours.entry((u.who, u.o)).or_insert((n, mv));
            }
        }
    }
    // **Anti-vacuity.** The dump has to have been read at all, and it has
    // to have carried the arrivals — a window in which every value is zero
    // would pass on a crate that never wrote the field.
    assert!(
        read > 1_000,
        "only {read} unit-frames carried `visible`; run112's dump no \
         longer prints the OBJECT block"
    );
    // **The dump's own nine**, as a guard on the oracle rather than on
    // this crate: if run112 stops saying this, every row below is void.
    let dumped: Vec<((i64, i64), (i64, u8))> = theirs.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(
        dumped,
        vec![
            ((0, 6), (636, 2)),
            ((0, 7), (636, 2)),
            ((0, 8), (636, 2)),
            ((0, 9), (646, 2)),
            ((0, 10), (631, 2)),
            ((0, 11), (640, 2)),
            ((1, 6), (672, 1)),
            ((1, 7), (698, 1)),
            ((1, 8), (665, 1)),
        ],
        "run112's own `visible` arrivals have changed"
    );
    // **The shape, asserted exactly**: the same nine units gain a bit, and
    // each gains the same one — player 1's on the bowmen and slingers,
    // player 0's on the hoplites. This is the mechanic's own claim and
    // nothing about the engagement's timing can excuse a failure here.
    let shape: Vec<((i64, i64), u8)> = ours.iter().map(|(k, v)| (*k, v.1)).collect();
    let want: Vec<((i64, i64), u8)> = theirs.iter().map(|(k, v)| (*k, v.1)).collect();
    assert_eq!(
        shape, want,
        "`visible` no longer arrives on the same units, or no longer \
         carries the same players' bits"
    );
    // **The frames, pinned in no direction** and printed beside the
    // dump's. Every one of the nine is within fourteen frames of the
    // original's, and every gap is the strike's and not the field's.
    let mine: Vec<((i64, i64), (i64, u8))> = ours.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(
        mine,
        vec![
            ((0, 6), (636, 2)),
            ((0, 7), (636, 2)),
            ((0, 8), (636, 2)),
            ((0, 9), (646, 2)),
            ((0, 10), (631, 2)),
            ((0, 11), (637, 2)),
            ((1, 6), (677, 1)),
            ((1, 7), (694, 1)),
            ((1, 8), (679, 1)),
        ],
        "chapter two's `visible` arrivals moved; re-pin them and say so \
         in docs/VISION.md §7"
    );
    // **Five of the nine land on the dump's own frame** — the three
    // bowmen, the slinger captain and, since item 462, `0/10` — and that
    // is the row that keeps the other four honest: a change that bought
    // the hoplites' frames by losing the bowmen's would fail here.
    //
    // `0/10` came over on item 462 (629 → **631**, the dump's own) and
    // `0/11` came three frames closer (633 → 637 against 640) on the
    // same change, which is the engagement's timing and not this field's:
    // the slingers' chase now walks to `find_attack_pos`' ring rather than
    // to the target's seat, so the frame of the first strike moves and
    // `visible` moves with it (`docs/COMBAT.md` §32.2).
    let exact = mine
        .iter()
        .filter(|(k, v)| theirs.get(k).is_some_and(|t| t.0 == v.0))
        .count();
    assert_eq!(
        exact, 5,
        "the five arrivals this crate puts on the dump's own frame are no \
         longer five"
    );
}
