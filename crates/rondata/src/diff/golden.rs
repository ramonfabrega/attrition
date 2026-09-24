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
    script_named(&format!("chapter{n}"))
}

/// A script under `tools/gamelog/golden/` by its stem — a chapter, or a
/// chapter's control (`chapter7_control`, item 578), which is its own file
/// so that it can never be an edit of the chapter.
fn script_named(stem: &str) -> Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/{stem}.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/{stem}.cmd: {e}"))
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
    walk_script(run, &format!("chapter{n}"), n, staged, length)
}

/// [`walk_chapter`] on a script named by its stem: chapter seven's control
/// (`chapter7_control`, item 578) walks through here.
fn walk_script(run: &str, stem: &str, n: u32, staged: usize, length: i64) -> Option<Walk> {
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
    let mut script = script_named(stem);
    assert_eq!(
        script.lines().len(),
        staged,
        "chapter {n} is {staged} staged lines (tools/gamelog/golden/{stem}.cmd)"
    );
    let last = trace.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last >= length,
        "the golden trace is {last} frames; chapter {n} is {}",
        length + 1
    );
    let mut applied = crate::golden::Applied::default();
    let mut words: Vec<(i64, u32)> = Vec::new();
    // **Which unit spent the draw**, on a window this run names —
    // `RON_GOLDEN_SITES=<lo>-<hi>`, the golden walk's copy of the window
    // [`crate::diff::harness`] has carried for Great Lakes since item
    // 204. A one-draw parting asks "who", `Built::tick` drops the marks'
    // unit attribution when it folds them into `frame_sites`, and item
    // 496 rebuilt this by hand as a scratch test before noticing the
    // Great Lakes walk already had it — which is the second time that
    // has happened (item 432 was the first), so it graduates here.
    // The original's own labels go beside ours on a frame they part,
    // because the delta is the question and one side alone never
    // answers it.
    let sites = crate::diff::harness::site_window_named("RON_GOLDEN_SITES");
    for _ in 0..last {
        words.push((built.sim.frame, built.sim.rng.seed));
        let did = script.stage(built.sim.frame, &mut built, &loaded);
        applied.merge(&did);
        let f = built.sim.frame;
        built.tick();
        if sites.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
            for (label, who) in crate::diff::harness::attributed_sites(&built) {
                eprintln!("  f{f} {who}: {label}");
            }
            let theirs = trace.labels(f);
            let ours = built.frame_sites.last().map_or(&[][..], |(_, v)| v);
            if theirs != ours {
                eprintln!("  f{f} PARTS: ours {} theirs {theirs:?}", ours.len());
            }
        }
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

/// **Chapter five, pinned** — the water (`docs/GOLDEN.md` §9, item 535,
/// run127). Four staged lines: `!ai off` and three hulls on sea region 70.
///
/// **What the capture established before this walk ran**, each written
/// into `chapter5.cmd`'s header first (`docs/RUNS.md`, run127): six
/// `INFO cmd` records each returning 1; 297 frame blocks; one `UNITDATA`
/// per `add` on 611, 616 and 621; and none of §9's three falsifiers. Every
/// hull is on an `OCEAN` cell and the triremes exchange 506 `AMMO` blocks.
///
/// `GOLDEN_WORD_CHAPTER_FIVE` carries what stands at the word.
#[test]
fn chapter_five_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch5", 5, 4, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FIVE,
        "chapter five's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FIVE}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FIVE,
        "chapter five's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §9"
    );
    eprintln!(
        "chapter five: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

/// **Chapter six, pinned** — the air and the bird (`docs/GOLDEN.md` §10,
/// item 648, run168). Six staged lines: `!ai off`, `library 6` for both
/// players, a Fighter for who=0, a Bomber for who=1, and `bird`.
///
/// **What the capture established before this walk ran**, each written
/// into `chapter6.cmd`'s header first (`docs/RUNS.md`, run168): eight
/// `INFO cmd` records each returning 1; 297 blocks; the Fighter `0/6` on
/// 611 at (888, 7800) and the Bomber `1/6` on 616 at (2424, 7800), the
/// predicted seats; the bird's birth draw on 700 and a seventh
/// `think_bird` from 704. **§10's second falsifier fired**: neither
/// aircraft moves, takes an order or leaves `air_alt` 0 to 899, and no
/// `AMMO` block is written.
///
/// `GOLDEN_WORD_CHAPTER_SIX` carries what stands at the word.
#[test]
fn chapter_six_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch6", 6, 6, 900) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_SIX,
        "chapter six's golden word fell to {} from {GOLDEN_WORD_CHAPTER_SIX}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SIX,
        "chapter six's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §10"
    );
    eprintln!("chapter six: sequence {}, values {:?}", w.sequence, w.value);
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
    //
    // **The fourth argument is this crate's own answer, checked against
    // the trace's** — run108 printed `ai=1` on this very captain, and
    // since item 470 [`sim::Sim::search_ai`] computes it. That equality is
    // the one place on disk where the predicate at `00648e6e` is measured
    // rather than read (`docs/COMBAT.md` §33.2).
    let ai = built.sim.search_ai(1);
    assert!(
        ai,
        "run108's proxy printed `ai=1` on who=1's captain; `search_ai`          disagrees, so the gate at `00648e6e` is modelled wrong"
    );
    let v: Vec<i32> = [(0, 8), (0, 7), (0, 6)]
        .into_iter()
        .map(|(w, o)| {
            built
                .sim
                .compare_target(captain, find(&built, w, o), true, ai)
        })
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
        .map(|(w, o)| {
            built
                .sim
                .compare_target(captain, find(&built, w, o), false, ai)
        })
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

/// **Chapter one's word frame, widened whole** (item 445,
/// `docs/COMBAT.md` §48): every record run105 dumps over
/// [`WIDENING_CHAPTER_ONE`], every unit of both real players, every field
/// this crate carries, both directions. The figure's animation clock comes
/// from run110.
///
/// The word stood at 626 from item 405 with no widening on file. The
/// tests above it each pin one mechanism: the seating, the reach, the
/// captain's pick, the hit, the hand-off. None of them compares the cast.
/// This is `compare` plus the record's own rows, ungated by the position,
/// which is the shape Great Lakes'
/// `run100_s_word_block_is_every_record_the_dump_carries` has and
/// chapter two's widening lacked until items 484, 510 and 523 added to it
/// one at a time.
///
/// **Two captures, one game.** run105 (`g4`) is the scored capture and
/// prints `GUYS=2`, which stops a `GUY` block after `ox`. run110 (`g6`) is
/// the same lobby, seed and script at `GUYS=9` over `[610, 630)`, and
/// `rngcmp.py` has its trace identical to run105's for all 641 frames
/// (`docs/RUNS.md` run110). Every key both files print on a `GUY` block is
/// asserted equal before a key only run110 prints is read from it, as
/// chapter two does with run118.
#[test]
fn chapter_one_s_word_frame_is_widened_whole() {
    /// run110's window (`docs/RUNS.md` run110): its clocks cover these
    /// blocks and nothing else.
    const RUN110: (i64, i64) = (610, 630);
    const FIRST: i64 = WIDENING_CHAPTER_ONE.0;
    const LAST: i64 = WIDENING_CHAPTER_ONE.1;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("g4") else {
        eprintln!("skipping: no golden capture g4 (see docs/RUNS.md run105)");
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
    let mut clocks =
        golden("g6").and_then(|(d, _)| crate::capture::indexed::IndexedCapture::open(&d).ok());
    if clocks.is_none() {
        eprintln!("no g6 capture: the GUY clock rows are unchecked (docs/RUNS.md run110)");
    }
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(1);
    use std::collections::BTreeMap;
    // Every key that parted, against the **first** block it parted on and
    // the value diff there.
    let mut first: BTreeMap<String, (i64, String)> = BTreeMap::new();
    let mut note = |k: String, n: i64, row: String| {
        first.entry(k).or_insert((n, row));
    };
    let mut blocks = 0usize;
    let mut rows = 0usize;
    let mut clock_blocks = 0usize;
    let mut clock_rows = 0usize;
    let mut near_read = 0usize;
    let mut group_blocks = 0usize;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        // The dump's label is the sim frame plus one, as in chapter two:
        // the tick that spends the word's draws, frame 626, writes block
        // 627.
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        let raw = ix.read_frame(at).unwrap();
        blocks += 1;
        // `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>`: what this crate's own
        // record held on the block, which the rows below never say.
        crate::diff::harness::debug_watch(&built, n);
        let r = compare(&built, &frame, players);
        for d in &r.diverged {
            note(
                format!("pos {}/{}", d.who, d.o),
                n,
                format!("ours {:?} theirs {:?}", d.ours, d.theirs),
            );
        }
        for d in &r.order_diverged {
            note(
                format!("order:{} {}/{}", d.what.label(), d.who, d.o),
                n,
                format!("{:?}", d.what),
            );
        }
        for (tag, list) in [("angle", &r.angle_diverged), ("angle~", &r.angle_parted)] {
            for d in list {
                note(
                    format!("{tag}:{:?} {}/{}", d.which, d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for (tag, list) in [("", &r.collide_diverged), ("~", &r.collide_parted)] {
            for d in list {
                note(
                    format!("{}{tag} {}/{}", d.field, d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for (tag, list) in [("", &r.search_diverged), ("~", &r.search_parted)] {
            for d in list {
                note(
                    format!("start_dist{tag} {}/{}", d.who, d.o),
                    n,
                    format!("ours {} theirs {}", d.ours, d.theirs),
                );
            }
        }
        for d in &r.los_diverged {
            note(
                format!("mylos {}/{}", d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.packed_diverged {
            note(
                format!("packed {}/{}", d.who, d.o),
                n,
                format!("ours {}", d.ours),
            );
        }
        for d in &r.visible_diverged {
            note(
                format!("visible {}/{}", d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.hits_diverged {
            note(
                format!("{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.firing_diverged {
            note(
                format!("{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.death_diverged {
            note(
                format!("death:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.gather_diverged {
            note(
                format!("gather:{}[{}] {}/{}", d.field, d.at, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.build_diverged {
            note(
                format!("build:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.queue_diverged {
            note(
                format!("queue:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for d in &r.city_diverged {
            note(
                format!("city:{} {}/{}", d.field, d.who, d.o),
                n,
                format!("ours {} theirs {}", d.ours, d.theirs),
            );
        }
        for &(who, o) in &r.unlinked_units {
            note(
                format!("unlinked {who}/{o}"),
                n,
                "the dump holds it alone".into(),
            );
        }
        for &(who, o) in &r.extra_units {
            note(
                format!("extra {who}/{o}"),
                n,
                "this crate holds it alone".into(),
            );
        }
        // The other direction on the buildings, which `compare` does not
        // take.
        for b in built.sim.buildings.iter().filter(|b| b.alive) {
            let (w, o) = (i64::from(b.owner), i64::from(b.index));
            if (0..players as i64).contains(&w)
                && !frame.builds.iter().any(|x| x.who == w && x.o == o)
            {
                note(
                    format!("build:extra {w}/{o}"),
                    n,
                    "this crate holds it alone".into(),
                );
            }
        }
        // `near_o`/`near_who`, which no parser carries: read off the
        // block's own text, as chapter two's near test does.
        let near = raw_near(&raw);
        let clock = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.frame_state(at).ok()
        });
        if clock.is_some() {
            clock_blocks += 1;
        }
        // **The army's group record**, which run110 alone prints
        // (`GROUPS=9`, item 399) and which no row read until item 530. The
        // pool slot is not an identity (`CLAUDE.md`): the group is matched
        // on what the slot holds, a live group of this player's army.
        let pool_text = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.read_frame(at).ok()
        });
        if let Some(text) = pool_text.as_deref() {
            let plog = Log::parse(text);
            for (_, b) in plog.frames() {
                for g in crate::gamelog::groups(b) {
                    let (Ok(who), Ok(slot)) = (usize::try_from(g.who), usize::try_from(g.army))
                    else {
                        continue;
                    };
                    if g.num == 0 || g.buildings != 0 || who >= players {
                        continue;
                    }
                    let Some(a) = built.sim.armies.get(who).and_then(|a| a.list.get(slot)) else {
                        continue;
                    };
                    group_blocks += 1;
                    let st = &a.group;
                    for (name, ours, theirs) in [
                        ("facing", i64::from(st.facing), g.facing),
                        ("order_num", i64::from(st.order_num), g.order_num),
                        ("form", i64::from(st.form), g.form),
                        ("speed", i64::from(st.speed), g.speed),
                        ("new_speed", i64::from(st.new_speed), g.new_speed),
                    ] {
                        rows += 1;
                        if ours != theirs {
                            note(
                                format!("group:{name} {who}/army{slot}"),
                                n,
                                format!("ours {ours} theirs {theirs}"),
                            );
                        }
                    }
                }
            }
        }
        // **The record's own rows, ungated by the position**: everything
        // the `UNITDATA` and `GUY` blocks print that `compare` either does
        // not read or reads only where the positions agree.
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = built.sim.unit_by_o(who, o) else {
                continue;
            };
            let un = &built.sim.units[u];
            let near_ours = match un.near {
                Some(sim::combat::Obj::Unit(x)) => (
                    i64::from(built.sim.units[x].owner),
                    i64::from(built.sim.units[x].index),
                ),
                _ => (-1, -1),
            };
            let near_theirs = near.get(&(them.who, them.o)).copied();
            if near_theirs.is_some() {
                near_read += 1;
            }
            let mut own: Vec<(String, i64, Option<i64>)> = vec![
                ("near_o".into(), near_ours.1, near_theirs.map(|p| p.0)),
                ("near_who".into(), near_ours.0, near_theirs.map(|p| p.1)),
                (
                    "heading".into(),
                    i64::from(un.movement.heading.0),
                    them.angle,
                ),
                (
                    "dest_angle".into(),
                    i64::from(un.movement.des_angle.0),
                    them.dest_angle,
                ),
                ("orders_x".into(), i64::from(un.orders_pos.x), them.orders_x),
                ("orders_y".into(), i64::from(un.orders_pos.y), them.orders_y),
                ("tolerance".into(), i64::from(un.tolerance), them.tolerance),
                (
                    "path_recursion".into(),
                    i64::from(un.path_recursion),
                    them.path_recursion,
                ),
                ("idle".into(), i64::from(un.idle), them.idle),
                ("stance".into(), i64::from(un.stance), them.stance),
                // `unit_masks & 4`, the in-danger latch §6.6 step 6 of
                // `docs/GROUPS.md` exempts a group move by (item 530).
                (
                    "in_danger".into(),
                    i64::from(un.in_danger) * 4,
                    them.unit_masks.map(|m| m & 4),
                ),
                ("myspeed".into(), i64::from(un.movement.speed), them.myspeed),
                ("group".into(), built.sim.pool_group_of(u), them.group),
                ("form".into(), i64::from(un.form), them.form),
                ("form_mod".into(), i64::from(un.form_width), them.form_mod),
                (
                    "orders.len".into(),
                    un.orders.len() as i64,
                    Some(them.orders.len() as i64),
                ),
                (
                    "guys.len".into(),
                    un.guys.len() as i64,
                    Some(them.guys.len() as i64),
                ),
            ];
            if un.on_map {
                for (k, ours, theirs) in [
                    ("collide", i64::from(un.collide), them.collide),
                    ("collide_o", i64::from(un.collide_o), them.collide_o),
                    ("collide_who", i64::from(un.collide_who), them.collide_who),
                    ("collide_guy", i64::from(un.collide_guy), them.collide_guy),
                    ("safe", i64::from(un.safe), them.safe),
                    ("start_dist", i64::from(un.start_dist), them.start_dist),
                    (
                        "half_step",
                        i64::from(un.half_step),
                        them.unit_masks.map(|m| i64::from(m & 0x10_0000 != 0)),
                    ),
                ] {
                    own.push((format!("{k}!"), ours, theirs));
                }
            }
            // The same unit in the clock capture, by `(who, o)`: the two
            // files are one game, so the slot is the same unit.
            let g6 = clock
                .as_ref()
                .and_then(|c| c.units.iter().find(|x| x.who == them.who && x.o == them.o));
            for (k, g) in them.guys.iter().enumerate() {
                let Some(og) = un.guys.get(k).copied() else {
                    continue;
                };
                let g = match g6.and_then(|x| x.guys.get(k)) {
                    Some(c) => {
                        // Every key run105 prints on a `GUY` block at
                        // `GUYS=2`; `des` is not one of them.
                        assert_eq!(
                            (g.pos, g.angle, g.des.or(c.des)),
                            (c.pos, c.angle, c.des),
                            "block {n}, {who}/{o} guy {k}: run110 is not run105's game"
                        );
                        clock_rows += 1;
                        c
                    }
                    None => g,
                };
                let (body, facing, des, des_angle) = match og.follow {
                    Some(b) => (b.body, b.facing, b.des, b.des_angle),
                    None if k == 0 => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.heading,
                    ),
                    None => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.facing,
                    ),
                };
                let track = og.follow.map_or((0, 0), |b| b.track);
                // What the figure last swung at, `(o, who)` (item 530).
                let aim = match og.aim {
                    Some(sim::combat::Obj::Unit(x)) => (
                        i64::from(built.sim.units[x].index),
                        i64::from(built.sim.units[x].owner),
                    ),
                    Some(sim::combat::Obj::Building(b)) => (
                        i64::from(built.sim.buildings[b].index),
                        i64::from(built.sim.buildings[b].owner),
                    ),
                    None => (-1, -1),
                };
                for (name, ours, theirs) in [
                    ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                    ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                    ("g.angle", i64::from(facing.0), g.angle),
                    ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                    ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                    ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                    ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                    ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                    ("g.end_time", i64::from(og.end_time), g.end_time),
                    ("g.last_time", i64::from(og.last_time), g.last_time),
                    ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                    ("g.ox", aim.0, g.ox),
                    ("g.whom", aim.1, g.whom),
                    // `guy_flags & 0x20`, which `Unit::set_in_danger`
                    // raises beside the unit's bit; run110 alone prints it
                    // (item 530).
                    (
                        "g.flags&0x20",
                        i64::from(un.guy_flag_0x20) * 0x20,
                        g.guy_flags.map(|f| f & 0x20),
                    ),
                    ("g.stopped", i64::from(og.stopped), g.stopped),
                    ("g.hold_attack", i64::from(og.pending_attack), g.hold_attack),
                    (
                        "g.queued_attack",
                        i64::from(og.queued_attack),
                        g.queued_attack,
                    ),
                    ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                    ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                    ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                    ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
                ] {
                    own.push((format!("{name}[{k}]"), ours, theirs));
                }
            }
            for (name, ours, theirs) in own {
                let Some(theirs) = theirs else { continue };
                rows += 1;
                if ours != theirs {
                    note(
                        format!("{name} {who}/{o}"),
                        n,
                        format!("ours {ours} theirs {theirs}"),
                    );
                }
            }
        }
    }
    let on: Vec<String> = first
        .iter()
        .map(|(k, (n, row))| format!("{n} {k}: {row}"))
        .collect();
    eprintln!(
        "ch1 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {clock_rows} clock rows over {clock_blocks} blocks, {near_read} near pairs, \
         {} keys parted",
        first.len()
    );
    for r in &on {
        eprintln!("  {r}");
    }
    // **Anti-vacuity.** Every block of the window is in run105, run110
    // covers its own window whole, and the near pairs and clock rows were
    // actually read: each is a row whose agreement would otherwise be a
    // silence.
    //
    // run105 has **no block 900**, as run112 has none: its window is
    // 605..899 and the `!quit` block is 901. Since item 530 the window
    // runs to the capture's end, so that one absence is the dump's own
    // shape and not a truncation.
    const RUN105_NO_BLOCK: i64 = 900;
    let absent = usize::from((FIRST..LAST).contains(&RUN105_NO_BLOCK));
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run105 no longer carries every block of [{FIRST}, {LAST})"
    );
    if clocks.is_some() {
        assert_eq!(
            clock_blocks,
            (RUN110.1 - RUN110.0) as usize,
            "run110 no longer carries every block of [{}, {})",
            RUN110.0,
            RUN110.1
        );
        // 379 when it was written: sixteen to nineteen living figures on
        // each of run110's twenty blocks.
        assert!(
            clock_rows >= 350,
            "only {clock_rows} guy records came from run110; the clock rows compare nothing"
        );
    }
    if clocks.is_some() {
        // Item 530: who=1's army group on each of run110's blocks from its
        // muster at 615: fourteen.
        assert!(
            group_blocks >= 14,
            "only {group_blocks} army groups came from run110's pool; the group rows compare nothing"
        );
    }
    // 2993 when it was written: every unit of both players on every block.
    assert!(
        near_read >= 2_500,
        "only {near_read} unit-frames carried a near pair; the raw read found nothing"
    );
    let staged = built
        .sim
        .units
        .iter()
        .filter(|u| u.owner < 2 && u.index >= 6)
        .count();
    assert_eq!(
        staged, 6,
        "the window does not hold chapter one's six hoplites"
    );
    // **The map, and its shape is the finding** (item 445). Before the
    // enemy ladder landed, the window `[605, 630)` held 87 keys and the
    // earliest row on the word's own frame was `0/8` on **625**, a block
    // under the word: position, order list, path, `collide`, the clock,
    // `recharging`, `visible`, and `damage 1/6` on 626 downstream of it.
    // All of those close with §6 step 3's arm B (`docs/COMBAT.md` §48),
    // and the word went 626 → 774. The window moved with it.
    //
    // **Item 530 moved the word 774 → 900, the capture's end**, and the
    // window with it (`docs/ORDERS.md` §22). Four keys went into the
    // record first and two of them were the answer: `in_danger`
    // (`unit_masks & 4`) and `g.flags&0x20` parted on **617** on all six
    // hoplites, 148 frames under the order kind the item was booked on;
    // the army group's `facing` parted on 617 from run110's pool; and the
    // guy's `ox`/`whom`. `stance` closed as an instrument row: the
    // original writes one byte for every stance panel and this crate
    // wrote only the combat half. The map fell from 88 keys to 46, and
    // every `dest_angle` and `orders_x/y` row closed with
    // `Unit::work`'s unconditional `update_action`.
    //
    // What stands, by family, and none spends a draw in the capture:
    //
    // - **605, the floor**: the city record of `0/2000`, which this crate
    //   holds empty (`busy`, `filled`, `land`, `space`, `ter`, and
    //   `1/2000`'s by one), and `form` on the ten pre-existing units.
    //   All older than the chapter's first staged line.
    // - **610, `g.gpiece`** on the ten pre-existing units: one age bracket
    //   under the dump's, chapter two's same ten (`docs/ANIM.md`).
    // - **611 and 616, the births**: `form` −1 against 0 on all six
    //   hoplites.
    // - **617–623, `g.end_time`** on three citizens: 33 against 56, an
    //   idle length, beside the `gpiece` bracket.
    // - **616, the army group's `speed`/`new_speed`**, 0 against 25 on
    //   every block run110 prints: `Group::add` ends in `compute_speed`
    //   for a group with an id, and this crate's army group takes its
    //   speed later.
    let got: Vec<(&str, i64)> = first.iter().map(|(k, (n, _))| (k.as_str(), *n)).collect();
    let measured = [
        ("city:busy 0/2000", 605),
        ("city:filled 0/2000", 605),
        ("city:filled 1/2000", 605),
        ("city:gatherers 0/2000", 605),
        ("city:land 0/2000", 605),
        ("city:land 1/2000", 605),
        ("city:peasant_dist 0/2000", 605),
        ("city:space[0] 0/2000", 605),
        ("city:space[1] 0/2000", 605),
        ("city:space[2] 0/2000", 605),
        ("city:ter[0] 0/2000", 605),
        ("city:ter[1] 0/2000", 605),
        ("city:ter[3] 0/2000", 605),
        ("city:ter[4] 0/2000", 605),
        ("form 0/0", 605),
        ("form 0/1", 605),
        ("form 0/2", 605),
        ("form 0/3", 605),
        ("form 0/4", 605),
        ("form 0/5", 605),
        ("form 0/6", 611),
        ("form 0/7", 611),
        ("form 0/8", 611),
        ("form 1/1", 605),
        ("form 1/2", 605),
        ("form 1/3", 605),
        ("form 1/4", 605),
        ("form 1/5", 605),
        ("form 1/6", 616),
        ("form 1/7", 616),
        ("form 1/8", 616),
        ("g.end_time[0] 0/1", 617),
        ("g.end_time[0] 0/2", 623),
        ("g.end_time[0] 1/2", 619),
        ("g.gpiece[0] 0/1", 610),
        ("g.gpiece[0] 0/2", 610),
        ("g.gpiece[0] 0/3", 610),
        ("g.gpiece[0] 0/4", 610),
        ("g.gpiece[0] 0/5", 610),
        ("g.gpiece[0] 1/1", 610),
        ("g.gpiece[0] 1/2", 610),
        ("g.gpiece[0] 1/3", 610),
        ("g.gpiece[0] 1/4", 610),
        ("g.gpiece[0] 1/5", 610),
        ("group:new_speed 1/army0", 616),
        ("group:speed 1/army0", 616),
    ];
    assert_eq!(
        got,
        measured.to_vec(),
        "chapter one's word frame no longer widens the way item 445 \
         measured it; re-pin this map and say so in docs/COMBAT.md §48"
    );
}

/// `near_o`/`near_who` for every `UNITDATA` of one block's raw text, keyed
/// on `(who, o)`. No parser carries the pair (`ch2_dump_near` reads it the
/// same way from a whole file).
fn raw_near(block: &str) -> std::collections::BTreeMap<(i64, i64), (i64, i64)> {
    let mut out = std::collections::BTreeMap::new();
    let mut cur: Option<std::collections::BTreeMap<&str, i64>> = None;
    let flush = |cur: &mut Option<std::collections::BTreeMap<&str, i64>>,
                 out: &mut std::collections::BTreeMap<(i64, i64), (i64, i64)>| {
        if let Some(r) = cur.take()
            && let (Some(&who), Some(&o), Some(&no), Some(&nw)) =
                (r.get("who"), r.get("o"), r.get("near_o"), r.get("near_who"))
        {
            out.insert((who, o), (no, nw));
        }
    };
    for line in block.lines() {
        let s = line.trim();
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
/// **Deliberately not through [`crate::gamelog::Log`].** ~~`near_o` and
/// `near_who` are the `ObjectData` half the parser leaves unparsed on
/// purpose — a parsed field is a *compared* field, and this crate's
/// `Unit` models neither, so parsing them would buy a comparison against
/// nothing.~~ **Closed by item 479**: [`sim::Unit::near`] is the field
/// now, and
/// [`chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame`] is the
/// comparison the note said could not exist. This reader still takes the
/// four figures §30 argues on and nothing else; the pair has a reader of
/// its own in [`ch2_dump_near`].
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

/// **680's widening, both directions** — every record run112's dump carries
/// over `[606, 684)`, compared whole against this crate's own walk, on the
/// frame chapter two's word now stands (`docs/COMBAT.md` §37.6).
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
    /// run112's one missing block inside its own window: 605..899, then
    /// the `!quit` block at 901.
    const RUN112_NO_BLOCK: i64 = 900;
    /// run118's last block (`docs/RUNS.md` run118): its clocks stop here.
    const RUN118_LAST_BLOCK: i64 = 846;
    /// The first frame compared: **run112's own first block**, which is
    /// the only floor that cannot hide a row. It stood at 633 until item
    /// 470, three frames under the word — and 633 was above a live
    /// divergence: `order 0/10` and `pos 0/10` part at **630**, on a
    /// chase this crate drops one frame before the original does, and no
    /// run had ever said so. A floor chosen to sit "just under the word"
    /// is the same shape as §33.4's unearned green: an instrument that
    /// agrees because it is not looking. Declared in
    /// [`WIDENING_CHAPTER_TWO`] beside the `WIDENINGS` row, so the floors
    /// guard reads the word against the same window this walks.
    const FIRST: i64 = WIDENING_CHAPTER_TWO.0;
    /// One past the last. 680 is the word; four frames past it is enough
    /// to say the parting opens *there* and not later.
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
    // **The clock capture** (item 510). run112 was taken at `GUYS=2`, and
    // `GuyData::log_data@005de6c0` stops a `GUY` block after `ox` at that
    // level: `cur_anim`, `cur_time`, `end_time`, `last_time`,
    // `des_angle`, `hold_attack` and `queued_attack` are **not in the
    // file**. run118 is the same lobby, the same seed and the same
    // `chapter2.cmd` at `GUYS=4` (`docs/RUNS.md` run118), truncated at
    // block 846 — which contains the whole of this window.
    //
    // It is opened as a *second* source rather than as the comparator,
    // because the draw stream this chapter is scored on is run112's
    // trace. [`clock_agrees`] below is what makes borrowing one capture's
    // field into another's walk honest: every key both files print is
    // asserted equal on every compared block, so a clock from a different
    // game cannot arrive quietly.
    let mut clocks =
        golden("ch2g4").and_then(|(d, _)| crate::capture::indexed::IndexedCapture::open(&d).ok());
    if clocks.is_none() {
        eprintln!("no ch2g4 capture: the GUY clock rows are unchecked (docs/RUNS.md run118)");
    }
    let mut clock_rows = 0usize;
    let mut clock_blocks = 0usize;
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let players = built.sim.players.len();
    let mut script = chapter(2);
    // Every key that parted, against the **first** frame it parted on —
    // a residue standing before the window opened is not this word's.
    let mut first: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
    let mut blocks = 0usize;
    let mut deaths = 0usize;
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
        // **The hit-point record** (item 484), keyed on the **field**
        // rather than the unit: a key that collapses a record's fields
        // hides all but the first for the rest of the run, which is
        // parked 452's lesson and `CLAUDE.md`'s "a row is a field, never
        // a unit". The value diff goes beside it, because a walk that
        // can see a death and not the wounds that made it is what this
        // row exists to fix (`docs/COMBAT.md` §38.4).
        for d in &r.hits_diverged {
            eprintln!(
                "  {n} {} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("{} {}/{}", d.field, d.who, d.o));
        }
        // **The firing record** (item 485), on the same terms: the reload
        // clock and the overkill window are written at `UNITDATA`'s own
        // indent on every unit of every block and were compared nowhere,
        // so 484's ladder could say that a wound arrived a frame late and
        // not whether the *shot* was fired on the original's frame.
        for d in &r.firing_diverged {
            eprintln!(
                "  {n} {} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("{} {}/{}", d.field, d.who, d.o));
        }
        // **The death-object list** (item 491, §42.1) — `DEATH_OBJS`,
        // both directions, keyed on the field. run112's window carries
        // exactly one: `1/8`, `first_frame 683`, `cur_anim 17`, on every
        // block from 684 to the end of the capture.
        deaths += r.death_compared;
        for d in &r.death_diverged {
            eprintln!(
                "  {n} death:{} {}/{} ours {} theirs {}",
                d.field, d.who, d.o, d.ours, d.theirs
            );
            note(format!("death:{} {}/{}", d.field, d.who, d.o));
        }
        // **The four `FrameResult` carries that this walk ignored until
        // item 466.** They are empty over this window — chapter two
        // stages nine soldiers and no building — but "widened whole"
        // has to mean every vector the comparison produces, not the ten
        // the mechanic happened to care about, or the claim decays into
        // the shape `CLAUDE.md` warns about: an instrument that agrees
        // because it is not looking. Adding them changed no row, which
        // is the only way to find that out.
        for d in &r.gather_diverged {
            note(format!("gather {}/{}", d.who, d.o));
        }
        for d in &r.build_diverged {
            note(format!("build {}/{}", d.who, d.o));
        }
        for d in &r.queue_diverged {
            note(format!("queue {}/{}", d.who, d.o));
        }
        for d in &r.city_diverged {
            note(format!("city {}/{}", d.who, d.o));
        }
        // **The `GUY` record whole** (item 510), which this walk had
        // never opened. `compare` carries no guy row at all — the one
        // window that compares a figure's animation clock is Great
        // Lakes' `run100_s_word_block_is_every_record_the_dump_carries`,
        // and chapter two's word has been an *animation* draw since 695.
        // Eleven items ran on a window that could not see the clock the
        // draw is spent on.
        //
        // `hold_attack` and `queued_attack` are new to the parser in this
        // landing and are the two that decide the item: an attack the
        // swing frame defers lands in the first and costs the wrap one
        // draw; one asked for while an attack is already playing lands in
        // the second and costs it two (`docs/COMBAT.md` §45).
        let clock = clocks.as_mut().and_then(|c| {
            let at = c.frames().iter().position(|x| x.number == n)?;
            c.frame_state(at).ok()
        });
        if clock.is_some() {
            clock_blocks += 1;
        }
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = built.sim.unit_by_o(who, o) else {
                continue;
            };
            // The same unit in the clock capture, by `(who, o)` — the
            // two files are the same game, so the slot is the same unit.
            let g4 = clock
                .as_ref()
                .and_then(|c| c.units.iter().find(|x| x.who == them.who && x.o == them.o));
            let un = &built.sim.units[u];
            for (k, g) in them.guys.iter().enumerate() {
                let Some(og) = un.guys.get(k).copied() else {
                    continue;
                };
                // **The clock's own record, and the check that it is this
                // game's.** Every key run112 prints on a `GUY` block is
                // asserted equal in run118 before a key run112 does *not*
                // print is read from it.
                let g = match g4.and_then(|x| x.guys.get(k)) {
                    Some(c) => {
                        assert_eq!(
                            (g.pos, g.angle),
                            (c.pos, c.angle),
                            "block {n}, {who}/{o} guy {k}: run118 is not run112's game"
                        );
                        clock_rows += 1;
                        c
                    }
                    None => g,
                };
                let (body, facing, des, des_angle) = match og.follow {
                    Some(b) => (b.body, b.facing, b.des, b.des_angle),
                    None if k == 0 => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.heading,
                    ),
                    None => (
                        un.movement.body,
                        un.movement.facing,
                        un.pos,
                        un.movement.facing,
                    ),
                };
                let track = og.follow.map_or((0, 0), |b| b.track);
                for (name, ours, theirs) in [
                    ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                    ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                    ("g.angle", i64::from(facing.0), g.angle),
                    ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                    ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                    ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                    ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                    ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                    ("g.end_time", i64::from(og.end_time), g.end_time),
                    ("g.last_time", i64::from(og.last_time), g.last_time),
                    ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                    ("g.stopped", i64::from(og.stopped), g.stopped),
                    ("g.hold_attack", i64::from(og.pending_attack), g.hold_attack),
                    (
                        "g.queued_attack",
                        i64::from(og.queued_attack),
                        g.queued_attack,
                    ),
                    ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                    ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                    ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                    ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
                ] {
                    let Some(theirs) = theirs else { continue };
                    if ours != theirs {
                        let key = format!("{name}[{k}] {who}/{o}");
                        let before = first.len();
                        first.entry(key.clone()).or_insert(n);
                        if first.len() != before {
                            eprintln!("  {n} {key}: ours {ours} theirs {theirs}");
                        }
                    }
                }
            }
        }
    }
    // run112 has **no block 900**: its window is 605..899 and the
    // `!quit` block is 901 (`docs/RUNS.md` run112). Since item 523 the
    // window runs to the capture's end, so that one absence is the
    // dump's own shape and not a truncation.
    let absent = usize::from((FIRST..LAST).contains(&RUN112_NO_BLOCK));
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run112's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    // **Anti-vacuity for the borrowed clock** (item 510), and it is the
    // one row here whose agreement would otherwise be a silence twice
    // over: run112 prints no `cur_anim`, `cur_time`, `end_time`,
    // `last_time`, `des_angle`, `gpiece`, `hold_attack` or
    // `queued_attack` on a per-frame `GUY` block at all, so before this
    // landing every one of those rows compared `Some(ours)` against
    // `None` and was skipped. A reader that stopped finding run118's
    // blocks would do exactly the same thing and print nothing, so the
    // count of guy records actually taken from it is pinned. Nine staged
    // figures, ten pre-existing ones and one guy each, over 123 blocks.
    if clocks.is_some() {
        // run118 is truncated at 846, and since item 523 the window runs
        // past it to run112's end: the clock rows cover `[FIRST, 847)` and
        // the frames above are compared on run112's own `GUYS=2` fields.
        let clocked = LAST.min(RUN118_LAST_BLOCK + 1);
        assert_eq!(
            clock_blocks,
            (clocked - FIRST) as usize,
            "run118 no longer carries every block of [{FIRST}, {clocked}); it is \
             truncated at {RUN118_LAST_BLOCK}"
        );
        assert!(
            clock_rows >= 2000,
            "only {clock_rows} guy records came from run118; the clock rows \
             are comparing nothing"
        );
        eprintln!("ch2 clock: {clock_rows} guy records read from run118");
    }
    // **Anti-vacuity for the death list**, which is the one row here
    // whose agreement is a *silence*: three fields on every block from
    // 684 to the ceiling, for the one death run112's window carries. A
    // reader that stopped parsing `DEATH_OBJS` would print no
    // divergence and neither would one that matched nothing, so the
    // count is pinned beside the map (parked 449's shape). It was
    // fifteen blocks while the ceiling stood at 699; item 502 moved the
    // word to 725 and the ceiling to 729, so it is forty-five — and
    // that the count is still exactly `3 × (LAST − 684)` is itself the
    // check that the window gained no **second** death.
    //
    // **Item 495 moved the ceiling to 766 and the window holds three
    // deaths now**, each on the original's own frame: `1/8` from 684,
    // `0/11` (dies 729) from 730 and `1/6` (dies 743) from 744 — the
    // hoplite the original's bowmen had been shooting since 696, which
    // this crate now targets too. Every one of the 420 comparisons
    // agrees.
    //
    // **Item 523 ran the window to run112's end and it holds four**:
    // `1/7`, the last hoplite, dies on 816 and is carried from 817. Each
    // record is on every block from its first to the end, less the one
    // block run112 does not have.
    let carried =
        |from: i64| (LAST - from) as usize - usize::from((from..LAST).contains(&RUN112_NO_BLOCK));
    assert_eq!(
        deaths,
        3 * (carried(684) + carried(730) + carried(744) + carried(817)),
        "the death-object comparison's own width moved; 1/8 dies on 683          and the dump carries its record on every block from 684 to the          end of run112"
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
    // finding. Item 462 measured the *values* parting at **635** on six
    // `Target` rows — every one of them a tie among near-equidistant
    // identical figures that this crate's ranking broke the wrong way.
    //
    // Item 466 found the six to be **two** faults, one per squad, and
    // landed one of them (`docs/COMBAT.md` §33). What landed is §33.1:
    // `ObjectData::targeted` is a decaying crowding penalty quartered
    // every sixteenth frame, which this crate bumped per order — three
    // times for a squad handed one target through the mirror — and never
    // decayed, so a stale `+3` pushed `1/8` out of the bowmen's exact
    // tie. The bowmen's three rows are gone; `order 0/6`, `0/7` and `0/8`
    // no longer appear at all, and nor do the `angle` rows that followed
    // them on 636.
    //
    // **The hoplites' three closed on item 470**, which landed §33.2:
    // `compare_target`'s fourth argument **divides** by the damage it
    // would deal for a computer leader where a human's multiplies, so
    // who=1's hoplites rank two slingers above three fatter bowmen and
    // this crate now does too. `order 1/6`, `1/7` and `1/8` move 635 →
    // **636** and the values no longer part at 635 at all. The cost is
    // `chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`,
    // which is **red on purpose** — `docs/COMBAT.md` §33.4 and §35.
    //
    // **`order 0/10` / `pos 0/10` at 630 are what the old floor hid.**
    // They are not item 470's: they stand identically with §33.2 disabled
    // (measured both ways on this window), and they are the first row on
    // the map now that it starts at run112's own first block. This crate
    // drops `0/10`'s chase on the frame it first reads in range; the
    // original drops it one frame later (`docs/COMBAT.md` §35.2).
    //
    // `order 0/5` / `pos 0/5` at 639-640 are a **citizen** far from the
    // engagement; they are named here so a regression in them cannot hide
    // behind the engagement, and they are nobody's item yet
    // (`docs/COMBAT.md` §32.5).
    //
    // **Item 479 closed all five of 472's rows and the map moved 645 →
    // 671**, sixty-five frames of run112 — every record the dump carries,
    // both directions, from its own first block to 670 — going to nought.
    // `do_move`'s captain retarget and `Unit::change_target` under it
    // (`docs/COMBAT.md` §37): `0/9` retargets to `1/6` on 645 as the dump
    // does, so `pos 0/9` no longer walks on, `order 0/10` and `order
    // 0/11` take the new target down the `o_down` chain on the same
    // block, and `visible 0/9` arrives on 646 rather than 648.
    //
    // **Item 481 closed all six of 671's rows and they were not a tie.**
    // `do_move@005f7b30:207`'s `ptype->max_range != 0` opens a *block*
    // whose brace closes past the captain retarget, and item 479 wrote
    // the gate as a conjunct of the in-range kill alone — so a **melee**
    // captain reached the retarget. run112's hoplite captain `1/6`
    // carries `near_o 10` from block 621 to the end of the window, and
    // on 671 the plain reach admits `0/10`, so this crate switched all
    // three hoplites off `0/11` down the `o_down` chain. The dump has
    // `1/6`, `1/7` and `1/8` on `ox 11 whom 0 uid 18` on **every block
    // of the capture** and never retargets them at all; 479's reading
    // that this was §33's tie mirrored onto who=1 is falsified by the
    // dump's own field. `docs/COMBAT.md` §38.
    //
    // **What stands now is 684, and it is a death.** The original's
    // `1/8` dies on 683 — `DEATH_OBJS` with `first_frame 683`, at its
    // 40-hit share of `myhits 120` (`docs/ATTRITION.md`'s figure table)
    // after `damage` 0 → 8 → 17 → 25 → 34 on 656/657/660/682 — and this
    // crate's is still alive, so `extra 1/8` is the row. `order 1/4` at
    // 685 and `pos 1/4` at 686 are a **citizen** forty thousand units
    // from the engagement, the same family as `order 0/5` (§32.5), and
    // all three stood identically with 481's change reverted and this
    // window: they were hidden by the old ceiling at 684, not opened by
    // the fix.
    //
    // **The window's ceiling is what hid the death, and `compare` is
    // what hides its cause**: [`crate::diff::compare`] carries no hit
    // -point row at all — no `myhits`, no `damage`, no `hits_left` —
    // where `run100_s_word_block_is_every_record_the_dump_carries` has
    // had both since §34.4. So this walk can see that `1/8` is *gone*
    // and not that it was wounded differently; the successor item owes
    // that row before it names a mechanism.
    //
    // **Item 484 put the hit-point record in `compare` and the map grew
    // from three rows to eight — the earliest of them 656, twenty-seven
    // blocks *under* the word.** `myhits`, `damage` and `damage_frac`
    // are printed inside the `OBJECT` block on every block of every
    // capture and were compared on none, so this walk was reporting
    // agreement over `[606, 684)` that it had never measured: the floor
    // hid `0/10` until item 470 moved it, the ceiling hid `1/8`'s death
    // until item 481 raised it, and a **field** hid the wounds that made
    // that death until item 484 added it. Three ways for the same
    // instrument to agree because it is not looking.
    //
    // None of the five new rows is this landing's doing — it writes no
    // simulation code — and all five are the same fact from two ends:
    // `Object::take_damage` divides the squad's `myhits` by `uber_size`
    // on the way in (`docs/COMBAT.md` §7.3) and this crate does not, so
    // its hoplite figures absorb a squad's worth apiece and `1/8` never
    // reaches the 40 that kills the original's. `damage 1/8` runs one
    // hit behind from 656 and `extra 1/8` at 684 is its consequence.
    // `docs/COMBAT.md` §40 is the record; naming the mechanism is item
    // 485's.
    //
    // **Item 491 moved the word 683 → 695 and the ceiling with it**, and
    // the map is now thirteen rows that are really two. `order 1/4` — the
    // far-off citizen — is gone, and nothing replaced it under 686.
    //
    // The first is `damage 1/6` at 680, which is §41.3's unmeasured
    // release node, untouched by this landing and still the only row this
    // capture cannot settle.
    //
    // The other twelve are **one arrow**. run112's `0/8` fires at 677 at
    // `1/8`; `0/7`'s arrow kills `1/8` on 683; `0/8`'s finds its target
    // dead, and because it carries `Ammo` flag 4 it does not puncture —
    // it rolls on (§42.2) and comes down two frames later where
    // `check_hit` finds **`1/7`**, for `19+5` on 685. This crate rolled
    // it on too and could not land it until item 495, which also found
    // that the retarget at 696 was `compare_target`'s bearing and not the
    // wound (`docs/COMBAT.md` §46).
    //
    // **Item 496 moved the three `order` rows 696 → 684**, and it moved
    // no code in the simulation: the comparison could not see them.
    // `crate::diff::order::compare_orders` reported a target only when
    // **both** sides named one, so this crate's attack order carrying
    // *nothing* against the dump's `ox 8 whom 1` read as agreement.
    // `Sim::forget` drops a dead object from every attacker's target
    // slot on the frame it dies; the original does not, and run112's
    // three bowmen hold the dead `1/8` from 683 until their reload opens
    // on 695. Thirty-six unit-frames, under the word, quiet.
    // `docs/COMBAT.md` §43.
    //
    // **Item 502 landed §43.2's arm whole and the word moved 695 → 725**,
    // which moved this window's ceiling to 729 with it. The three
    // `order` rows go **back** to 696 — the twelve frames item 496 opened
    // are agreement now, because this crate holds the dead `1/8` where
    // the original does — and all three `recharging` rows close, because
    // `0/7` and `0/8` strike on 695 through `Unit::fight`'s follower
    // inherit and `0/6` does not strike at all.
    //
    // `unit_masks2` joined `compare` in the same landing and reports
    // **nothing on this window**: the bit the dump carries on `0/6` at
    // block 696 is this crate's too. The chapter-wide check is
    // `chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_frame`,
    // which is where that row's anti-vacuity lives — this window holds
    // one of the seven unit-frames run112 ever marks.
    //
    // **`damage_frac 1/6` at 712 is the raised ceiling's**, not this
    // landing's — the same shape as `extra 1/8` at 684 on item 481 and
    // `1/8`'s wounds at 656 on item 484, and the third time this window
    // has grown a row by being allowed to look further. It is downstream
    // of the parked arrow: this crate's `1/7` is unwounded, so §33's
    // damage weight sends all three bowmen to `1/7` where the dump sends
    // them to `1/6`, and from 712 the two sides are wounding different
    // hoplites. `docs/COMBAT.md` §42.5 is what it waits on.
    //
    // **Item 510 opened the `GUY` record and the map went from eleven
    // rows to thirty-nine.** `compare` carries no guy row at all and this
    // walk built none, so for eleven items the window could not see the
    // animation clock — which is the record chapter two's word has been
    // spent in since 695. Twenty-eight of the new rows were invisible
    // twice over: run112 was taken at `GUYS=2` and prints **no**
    // `cur_anim`, `cur_time`, `end_time`, `last_time`, `des_angle`,
    // `gpiece`, `hold_attack` or `queued_attack` on a per-frame `GUY`
    // block, so even a walk that had built the rows would have compared
    // `Some(ours)` against `None` and skipped every one. The clock is
    // borrowed from run118, asserted to be the same game key by key
    // above. `docs/COMBAT.md` §45.
    //
    // The three families, and none of them is this landing's doing:
    //
    // - **`g.cur_anim` / `g.end_time` / `g.stopped` / `g.hold_attack` on
    //   `0/7` and `0/8` at 726 and `0/6` at 727** are the word's own
    //   delta, seen as values for the first time. The original's figure
    //   comes out of its attack-end wrap on an **attack** slot
    //   (`cur_anim 11`, `12`) and this crate's on the idle (`0`), with
    //   `hold_attack 1` where the dump has 0: the swing that asked while
    //   the attack was still playing lands in `queued_attack` for the
    //   original and in `hold_attack` here, and only the first is paid
    //   inside `Guy::inc_time`'s own loop with a roll. That is the two
    //   `Guy::set_anim+0xf2f < Guy::inc_time+0x271` the word is.
    // - **`g.des_angle` beside each `g.angle`**: the same parting as the
    //   unit-level `angle` rows, one level down, and the reason for the
    //   four above — `Unit::fight` writes the guy's `des_angle` to the
    //   attack angle and `Guy::set_anim` defers the attack while it
    //   differs from the figure's own `angle` (§45.1).
    // - **`g.gpiece` on ten pre-existing units at 606**, the window's
    //   floor, on both players and five types: this crate's piece is
    //   exactly [`sim::anim::PIECES_PER_AGE`] below the dump's on every
    //   one of them — one age bracket. The animation clock agrees on all
    //   ten for the whole window, so the two pieces share their lengths
    //   here and it spends no draw; it is named so it cannot hide, and it
    //   is `docs/ANIM.md`'s.
    // **Item 495 landed the rolled arrow and the map fell from
    // thirty-nine rows to twenty** (`docs/COMBAT.md` §46). Every row from
    // 686 to 727 is gone: `damage`, `damage_frac` and `damage_frame 1/7`
    // close because `0/8`'s shot now comes down on 685 where the
    // original's does; the bowmen's `order`, `angle` and `des_angle` rows
    // close because `compare_target` now ranks at the real bearing and
    // picks `1/6`; and the animation clock 510 opened at 726-727 closes
    // with them. Nothing parts on the word's own frames, 762 and 763.
    // What stands is `damage 1/6` at 680 (§41.3's release node), the ten
    // `g.gpiece` rows at 606 (parked, `docs/ANIM.md`), and the two rows
    // past the word: `0/10`'s clock at 764 and `1/7` — the original's
    // last hoplite — taking a long walk at 765 that this crate's does not.
    //
    // **Item 523 ran the window to run112's end and the map fell from
    // twenty rows to eleven** (`docs/COMBAT.md` §47). The nine rows past
    // 762 were one mechanism: `1/7` was never promoted when its captain
    // `1/6` died on 743, so on 762 it skipped the one-in-five roll the
    // original's captain spends. With the promotion, the lead's
    // `avg_speed` and the hit's launch-to-landing bearing, **nothing
    // parts on any frame from 606 to the end of the capture** except the
    // two standing residues: `damage 1/6` at 680 and the ten `g.gpiece`
    // rows at 606. Both are older than this window's word and neither
    // spends a draw.
    let measured = [
        ("damage 1/6", 680),
        ("g.gpiece[0] 0/1", 606),
        ("g.gpiece[0] 0/2", 606),
        ("g.gpiece[0] 0/3", 606),
        ("g.gpiece[0] 0/4", 606),
        ("g.gpiece[0] 0/5", 606),
        ("g.gpiece[0] 1/1", 606),
        ("g.gpiece[0] 1/2", 606),
        ("g.gpiece[0] 1/3", 606),
        ("g.gpiece[0] 1/4", 606),
        ("g.gpiece[0] 1/5", 606),
    ];
    let got: Vec<(&str, i64)> = first.iter().map(|(k, &n)| (k.as_str(), n)).collect();
    assert_eq!(
        got,
        measured.to_vec(),
        "chapter two's word frame no longer widens the way item 523 \
         measured it; re-pin this map and say so in docs/COMBAT.md §47"
    );
}

/// **`ObjectData::near_o`/`near_who` against run112's own**, on every
/// unit of every block of the widening window (`docs/COMBAT.md` §37.1).
///
/// The pair is the search's **footprint** rather than its answer — the
/// nearest candidate `Object::find_nearby_target@00648da0` saw that
/// cleared `check_target`, written above the `max_dist` gate and above
/// the scoring, cleared to `-1` when the nearest one is past `0xf00`. It
/// is printed in the `OBJECT` block at every detail level, so the dump
/// has carried it since 2026-09-19, and **nothing ever compared it**:
/// [`ch2_dump_units`] says in so many words that parsing it "would buy a
/// comparison against nothing" because this crate modelled neither field.
/// Item 479 gave it one ([`sim::Unit::near`]), which turns that note into
/// a row.
///
/// It is the only check §37.1's rule has. The arm that reads the field
/// (§37.2) is diff-backed through `order 0/9` on 645, but *which*
/// candidate the field holds, and when it is cleared, are otherwise a
/// reading — and a wrong incumbent is invisible to every other row until
/// the frame it is acted on.
///
/// What is pinned is the **tally and its shape**: how many unit-frames
/// the dump carries a pair for, how many of those carry a *live* pair,
/// how many this crate agrees with, and the exact list of those it does
/// not. Pinning the disagreements by name rather than counting them is
/// what keeps a later landing from trading one unit's footprint for
/// another's; pinning `live` beside `read` is what keeps the empty list
/// from being an instrument that stopped looking, because 4678 of the
/// 4848 unit-frames are animals and idle citizens whose pair is `-1` on
/// both sides.
///
/// **What it catches, measured by making it fail** (`CLAUDE.md`, "the
/// checks with teeth"): deleting the write turns the 161 live frames
/// into three parted units — `0/9` from 621, `0/6` and `1/6` from 635,
/// which are exactly chapter two's three searching captains.
///
/// **What it does not catch, measured the same way and stated because
/// the green would otherwise read as more than it is**: taking the
/// *last* qualifying candidate rather than the nearest, and removing the
/// `0xf00` clear, both leave it green. run112 runs so few searches
/// inside the window — three, all of them before 636 — that neither
/// rule is exercised. So this row backs the field's **value** on every
/// frame it is read on; §37.1's write rule stays a reading until a
/// capture with a crowded, far search reaches it.
#[test]
fn chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame() {
    const FIRST: i64 = WIDENING_CHAPTER_TWO.0;
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
    let mut built = stand_up(&loaded, &log, &refs, &trace);
    let mut script = chapter(2);
    let mut read = 0usize;
    let mut agree = 0usize;
    let mut live = 0usize;
    let mut parted: std::collections::BTreeMap<(i64, i64), i64> = std::collections::BTreeMap::new();
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        for ((who, o), pair) in ch2_dump_near(&text, n) {
            let Some(mine) = i16::try_from(o)
                .ok()
                .and_then(|x| built.sim.unit_by_o(who as sim::Player, x))
            else {
                continue;
            };
            read += 1;
            let ours = match built.sim.units[mine].near {
                Some(sim::combat::Obj::Unit(u)) => (
                    i64::from(built.sim.units[u].owner),
                    i64::from(built.sim.units[u].index),
                ),
                // A building incumbent and "none" are both `(-1, -1)` to
                // this comparison: the original prints one pair for both
                // classes and this crate carries a unit's only, so a
                // building would read as a parting here and be one.
                _ => (-1, -1),
            };
            let theirs = (pair.1, pair.0);
            if theirs != (-1, -1) {
                live += 1;
            }
            if ours == theirs {
                agree += 1;
            } else {
                parted.entry((who, o)).or_insert(n);
            }
        }
    }
    // **Anti-vacuity**: the dump has to have been read, and the window has
    // to hold the nine.
    assert!(
        read > 2_000,
        "only {read} unit-frames carried a near pair; run112's dump no \
         longer prints the OBJECT block over [{FIRST}, {LAST})"
    );
    eprintln!("near: {agree} of {read} agree, {live} live; parted {parted:?}");
    let got: Vec<((i64, i64), i64)> = parted.iter().map(|(k, &n)| (*k, n)).collect();
    assert_eq!(
        got,
        NEAR_PARTED.to_vec(),
        "chapter two's `near_o` footprint no longer parts where item 479 \
         measured it; re-pin this and say so in docs/COMBAT.md §37.1"
    );
    assert_eq!(
        (read, live, agree),
        NEAR_TALLY,
        "the near comparison's own width moved; a tally that shrinks is \
         an instrument that stopped looking"
    );
}

/// The units whose `near` pair parts from run112's, and the first block
/// each parts on — item 479's measurement, pinned by name. It is
/// **empty**: `4848` of `4848` unit-frames over `[606, 687)` carry the
/// dump's own `near_o`/`near_who`, animals included — 4668 of 4668 over
/// `[606, 684)` on the landing that gave this crate the field, and the
/// same on the window item 481 widened.
///
/// **Item 523 ran the window to run112's end, and two rows part on 847**,
/// both past every draw the chapter spends differently (none). On sim
/// frame 846 the bowmen `0/7` and `0/8` drop their attack on the dead
/// `1/7`. The original's `near_o 7 near_who 1` stands through the drop and
/// after it; this crate's pair goes to `-1`. The search throttle is not
/// the reason (`waiting` stays 0 on both sides), and no mechanism is named
/// here. Parked for the commander to book by its frame.
const NEAR_PARTED: &[((i64, i64), i64)] = &[((0, 7), 847), ((0, 8), 847)];
/// `(unit-frames read, live pairs among them, unit-frames agreeing)` for
/// the row above. All three are pinned because an empty disagreement
/// list is worthless without them: a reader that stopped parsing would
/// print no partings, and so would one that only ever saw `-1`.
///
/// **Re-pinned upward by item 481**, 4668/161 → 4848/170, because the
/// widening window's ceiling followed the word 680 → 683 and this test
/// walks the same window. The agreement is still total, and it is the
/// row that says a melee captain still *reads* its incumbent (§38.5) —
/// `1/6` carries `near_o 10` on every block of the window and no longer
/// acts on it. **And again by item 491**, 4848/170 → 5568/206, on the
/// ceiling's move 687 → 699 with the word 683 → 695 (§42.4): twelve more
/// blocks, thirty-six more live pairs, and the agreement still total
/// across the death of `1/8` and the three bowmen's retarget after it.
///
/// **And by item 502**, 5568/206 → 7368/296, on the ceiling's move 699 →
/// 729 with the word 695 → 725. The agreement is still **total**, and
/// that is a result rather than bookkeeping: item 496 measured
/// [`sim::Sim::forget`]'s clear removed *alone* and it turned `near_o
/// 0/7` and `0/8` red at 696, because this crate's invalid-target arm
/// searched on every member of the squad where the original searches
/// only on the captain. With §43.2's arm landed whole the followers
/// never reach the search at all — they take their captain's target in
/// place at `Unit::fight`'s head — so the footprint stays `-1` on both
/// of them, which is exactly what the dump says.
///
/// **And by item 495**, 7368/296 → 9530/409, on the ceiling's move 729 →
/// 766 with the word 725 → 762 — still total, across two more deaths and
/// the bowmen's retarget onto `1/6`, which this crate now makes as the
/// original does.
///
/// **And by item 523**, 9530/409/9530 → 17219/859/17113, on the ceiling's
/// move 766 → 901 with the word 762 → 900: the whole of run112. The 106
/// that disagree are [`NEAR_PARTED`]'s two bowmen from 847 to the end.
const NEAR_TALLY: (usize, usize, usize) = (17219, 859, 17113);

/// One frame's `near_o`/`near_who` per unit, read out of the raw dump
/// text — [`ch2_dump_units`]'s sibling, for the pair that reader takes
/// only `near_o` of.
fn ch2_dump_near(text: &str, frame: i64) -> std::collections::BTreeMap<(i64, i64), (i64, i64)> {
    let head = format!("BEGIN FRAME {frame}");
    let mut out = std::collections::BTreeMap::new();
    let mut inside = false;
    let mut cur: Option<std::collections::BTreeMap<&str, i64>> = None;
    let flush = |cur: &mut Option<std::collections::BTreeMap<&str, i64>>,
                 out: &mut std::collections::BTreeMap<(i64, i64), (i64, i64)>| {
        if let Some(r) = cur.take() {
            let g = |k: &str| r.get(k).copied().unwrap_or(i64::MIN);
            if g("near_o") != i64::MIN {
                out.insert((g("who"), g("o")), (g("near_o"), g("near_who")));
            }
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

/// **The hit-point record against run112's own, on every unit-frame of
/// the whole chapter** — item 484, and the anti-vacuity half of the row
/// [`crate::diff::compare`] gained there (`docs/COMBAT.md` §40).
///
/// The widening above walks seventy-eight blocks around the word. This
/// walks all 898 and asserts two things the widening cannot:
///
/// - that the comparison **read live values** — a crate that never wrote
///   `damage` would pass a window in which every value on both sides is
///   zero, which is the whole of chapter two before block 631. The
///   dump's own wounded unit-frames are counted separately from the
///   unit-frames read, because only the first says the row is looking at
///   anything;
/// - that `myhits` and `damage_frac`, the two halves nothing else pins,
///   are the dump's — `myhits` on **every** unit-frame of the chapter,
///   which is what says this crate's squad-sized maximum is the
///   original's for all nine staged figures and the three citizens
///   beside them.
///
/// **`damage` is not asserted equal**, and the reason is the successor
/// item's: `Object::take_damage` divides the squad's `myhits` by
/// `uber_size` on the way in (`docs/COMBAT.md` §7.3) and this crate does
/// not, so its hoplite figures absorb three times what the original's do
/// and `1/8` does not die on 683. The row here is what makes that
/// visible; naming it is item 485's and pinning it here in either
/// direction would pin the defect. What *is* pinned is the shape — which
/// units the dump ever wounds, and that this crate wounds the same ones.
#[test]
fn chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame() {
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
    /// One past the last frame read — the chapter's last logged block,
    /// as [`chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`]
    /// reads it.
    const LAST: i64 = 899;
    let mut compared = 0usize;
    // Unit-frames on which the **dump** carries a non-zero `damage` or
    // `damage_frac`: the live values, without which the row proves
    // nothing.
    let mut live = 0usize;
    // `(who, o)` → the first block the dump ever wounds it on, and ours.
    let mut wounded_theirs: std::collections::BTreeMap<(i64, i64), i64> =
        std::collections::BTreeMap::new();
    let mut wounded_ours: std::collections::BTreeMap<(i64, i64), i64> =
        std::collections::BTreeMap::new();
    // Every field that ever parted, against the first block it parted on.
    let mut first: std::collections::BTreeMap<(&'static str, i64, i64), (i64, String)> =
        std::collections::BTreeMap::new();
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
        let r = compare(&built, &frame, players);
        compared += r.hits_compared;
        for d in &r.hits_diverged {
            first
                .entry((d.field, d.who, d.o))
                .or_insert((n, format!("ours {} theirs {}", d.ours, d.theirs)));
        }
        for u in &frame.units {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            if u.damage.is_some_and(|d| d != 0) || u.damage_frac.is_some_and(|d| d != 0) {
                live += 1;
                wounded_theirs.entry((u.who, u.o)).or_insert(n);
            }
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            let un = &built.sim.units[mine];
            if un.health < un.max_health || un.damage_frac != 0 {
                wounded_ours.entry((u.who, u.o)).or_insert(n);
            }
        }
    }
    // **Anti-vacuity, both halves.** The dump has to have been read at
    // all, and it has to have carried wounds: a window in which every
    // value is zero on both sides passes on a crate that never writes
    // the field, which is exactly what `compare` was before this row.
    assert!(
        compared > 1_000,
        "only {compared} hit-point comparisons were made; run112's dump \
         no longer prints `myhits`/`damage`/`damage_frac` in the OBJECT \
         block"
    );
    eprintln!("ch2: {compared} hit-point comparisons, {live} of them live");
    assert!(
        live > 100,
        "only {live} of run112's unit-frames carry a wound; this row \
         would agree on a crate that never took damage at all"
    );
    // **`myhits` is the dump's on every unit-frame of the chapter**, and
    // so is `damage_frac`'s own agreement where the two sides agree on
    // the damage — the maximum is a baked constant and a wrong one is
    // wrong whatever the engagement does.
    let myhits: Vec<String> = first
        .iter()
        .filter(|((field, ..), _)| *field == "myhits")
        .map(|((f, who, o), (n, v))| format!("{f} {who}/{o} first at {n} \u{2014} {v}"))
        .collect();
    assert_eq!(
        myhits,
        Vec::<String>::new(),
        "a unit's `myhits` parts from run112's: this crate's squad-sized \
         maximum is no longer the original's"
    );
    // **The shape: the same units are wounded on both sides.** The
    // *frames* are not pinned, for the reason
    // `chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame`
    // does not pin its own — a wound arrives when the arrow does — and
    // the *amounts* are not pinned because §7.3's divide is item 485's.
    let theirs: Vec<(i64, i64)> = wounded_theirs.keys().copied().collect();
    let ours: Vec<(i64, i64)> = wounded_ours.keys().copied().collect();
    assert_eq!(
        theirs,
        vec![(0, 9), (0, 10), (0, 11), (1, 6), (1, 7), (1, 8)],
        "run112's own wounded set has changed"
    );
    // Item 485 added `(0, 9)`: with the figure's share as the death
    // threshold (§7.2 step 8) the hoplite squad loses `1/8` on the
    // original's own block and its survivors then reach a second
    // slinger, which the dump also wounds. `(0, 10)` was the one the
    // dump wounded and this crate did not — and **item 502 closed it**,
    // so the two sets are now identical and the assertion is written
    // that way rather than against a copy of the list. §43.2's arm is
    // what reaches it: the original's hoplites fight on where this
    // crate's went idle for a frame each time a target died, and the
    // frames they gain back are the ones that reach `0/10`.
    //
    // ~~**Item 495 took `(0, 9)` back out, and past the word.**~~ The dump
    // first wounds `0/9` on block 792, and **item 523 put it back**:
    // with `1/7` promoted to captain on 743 it spends the one-in-five
    // roll, walks the original's walk from 765, and reaches `0/9` as the
    // dump's does (`docs/COMBAT.md` §47). The sets are identical again,
    // and the assertion is written that way.
    assert_eq!(ours, theirs, "the set this crate wounds has moved");
    assert!(
        theirs.iter().all(|w| ours.contains(w)),
        "a unit the dump wounds and this crate does not"
    );
    // **What parts, printed and pinned by its field.** `damage` and
    // `damage_frac` part on the engagement and the reason is §7.3's
    // divide — item 485's, not this row's.
    let rows: Vec<(&str, i64, i64, i64, &str)> = first
        .iter()
        .map(|((field, who, o), (n, v))| (*field, *who, *o, *n, v.as_str()))
        .collect();
    for r in rows.iter().take(24) {
        eprintln!("  {} {}/{} first at {} — {}", r.0, r.1, r.2, r.3, r.4);
    }
    let fields: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.0).collect();
    assert!(
        !fields.contains("myhits"),
        "`myhits` is in the parting set and the assertion above missed it"
    );
}

/// **The frozen frames of the whole chapter** — `unit_masks2 & 0x10` on
/// every unit-frame of run112's 898 blocks, both directions (item 502,
/// `docs/COMBAT.md` §43.2).
///
/// The widening window above holds **one** of the seven unit-frames the
/// capture ever marks, so its silence on the row [`crate::diff::compare`]
/// gained in the same landing is not much of a check. This is: the mark
/// is a one-frame state written inside `Unit::fight`'s invalid-target arm
/// and cleared at the head of the next frame's `Unit::process`, so every
/// unit-frame that carries it is a frame on which a unit's target went
/// invalid with its reload open — and every figure of that unit stood its
/// animation clock still.
///
/// What is pinned is the **exact list**, by block and unit, on both
/// sides. run112 carries seven unit-frames over six blocks, all of them
/// in the second fight, and this crate reproduces **five**:
///
/// ```text
///   696  0/6     ours    the chapter-two word's own frame
///   736  1/6     ours
///   745  0/9     —
///   756  0/7     ours
///   756  0/8     ours
///   757  0/6     ours
///   762  1/7     —
/// ```
///
/// The two it misses are **past the word**, which stands at 725: from
/// there the draw stream has already parted and nothing downstream is
/// owed. Four of the five it does reproduce are past the word too, so
/// they are a bonus rather than the measurement — the measurement is
/// 696, which is the frame §43.2's arm was landed for. Both lists are
/// pinned rather than only the difference, because a shrinking list on
/// either side is an instrument that stopped looking.
///
/// **Measured by making it fail** (`CLAUDE.md`, "the checks with
/// teeth"): with §43.2's arm off this crate's list is **empty**, all
/// seven rows are reported, and the widening's own map gains
/// `unit_masks2 0/6` at 696.
#[test]
fn chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_frame() {
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
    /// One past the last frame read — the chapter's last logged block.
    const LAST: i64 = 899;
    let mut compared = 0usize;
    let mut theirs: Vec<(i64, i64, i64)> = Vec::new();
    let mut ours: Vec<(i64, i64, i64)> = Vec::new();
    let mut parted: Vec<String> = Vec::new();
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
        let r = compare(&built, &frame, players);
        for d in &r.firing_diverged {
            if d.field == "unit_masks2" {
                parted.push(format!(
                    "{n} {}/{} ours {} theirs {}",
                    d.who, d.o, d.ours, d.theirs
                ));
            }
        }
        for u in &frame.units {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            let Some(bit) = u.unit_masks2 else { continue };
            compared += 1;
            if bit & i64::from(sim::combat::umask2::NOT_FIRING) != 0 {
                theirs.push((n, u.who, u.o));
            }
            let Some(mine) = i16::try_from(u.o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o))
            else {
                continue;
            };
            if built.sim.units[mine].unit_masks2 & sim::combat::umask2::NOT_FIRING != 0 {
                ours.push((n, u.who, u.o));
            }
        }
    }
    // **Anti-vacuity**: the dump has to have been read at all, and it has
    // to have carried marks. An empty list on both sides is what a reader
    // that stopped parsing looks like, and what a crate that never writes
    // the bit looks like.
    assert!(
        compared > 5_000,
        "only {compared} unit-frames carried `unit_masks2`; run112's dump          no longer prints it in the UNITDATA block"
    );
    eprintln!(
        "ch2 frozen frames: {compared} unit-frames, {} marked",
        theirs.len()
    );
    assert_eq!(
        theirs,
        vec![
            (696, 0, 6),
            (736, 1, 6),
            (745, 0, 9),
            (756, 0, 7),
            (756, 0, 8),
            (757, 0, 6),
            (762, 1, 7),
        ],
        "run112's own frozen frames have moved"
    );
    // **Item 495 closed the two this crate missed**, `0/9` at 745 and
    // `1/7` at 762: with the rolled arrow landed and the bowmen on `1/6`,
    // the engagement is the original's through the word, and so is every
    // frame a unit finds its target gone.
    assert_eq!(
        ours, theirs,
        "this crate's frozen frames have moved; they were the original's \
         own seven on item 495"
    );
    // The two the dump has and this crate does not, by name and with the
    // reason: both are past the word, where the draw stream has already
    // parted. Pinned so that a landing which closes them has to say so.
    let missing: Vec<(i64, i64, i64)> = theirs
        .iter()
        .filter(|r| !ours.contains(r))
        .copied()
        .collect();
    assert_eq!(
        missing,
        Vec::<(i64, i64, i64)>::new(),
        "the frozen frames this crate misses have moved"
    );
    assert!(
        missing.iter().all(|&(n, ..)| n > GOLDEN_WORD_CHAPTER_TWO),
        "a frozen frame is missed at or under the word ({GOLDEN_WORD_CHAPTER_TWO}), \
         which is a divergence the word itself should be reporting"
    );
    // Every block the two sides *both* mark agrees field for field, which
    // is the row `compare` gained in the same landing.
    let past: Vec<String> = parted
        .iter()
        .filter(|r| {
            r.split_whitespace()
                .next()
                .and_then(|n| n.parse::<i64>().ok())
                .is_some_and(|n| n <= GOLDEN_WORD_CHAPTER_TWO)
        })
        .cloned()
        .collect();
    assert_eq!(
        past,
        Vec::<String>::new(),
        "`unit_masks2` parts at or under chapter two's word"
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
            ((0, 11), (640, 2)),
            ((1, 6), (672, 1)),
            ((1, 7), (698, 1)),
            ((1, 8), (665, 1)),
        ],
        "chapter two's `visible` arrivals moved; re-pin them and say so \
         in docs/VISION.md §7"
    );
    // **The three hoplites moved on item 470**, 677/694/679 →
    // 669/678/675 against the dump's 672/698/665, and they moved because
    // all three now chase a slot of their **own**:
    // `find_ordered_collision`'s group pass reads the asker's group and
    // not the last one pushed (`docs/COMBAT.md` §35.1). Before that,
    // `1/8` never struck at all and the shape row above was red; two of
    // the three are closer to the dump's frame and `1/7` is twenty
    // frames under it. Pinned in no direction, as the doc comment says.
    //
    // **All nine land on the dump's own frame.** It was five until item
    // 472, which brought the three hoplites over — 669/678/675 against
    // 672/698/665, the engagement's own residue for four items (§31.6,
    // §35.4) — and eight until item 479, whose ninth is `0/9`'s own: 648
    // → **646**, the dump's. `0/9` is the slinger captain that retargets
    // to `1/6` on block 645 under `do_move`'s captain arm, and it strikes
    // a frame later because `change_target` leaves the attack to the next
    // frame rather than dispatching it (`docs/COMBAT.md` §37.2). Before
    // that it kept walking to `1/8` and struck two frames late.
    //
    // **`visible`'s arrival frame is the frame of a unit's first strike**
    // — `Unit::set_attacking` fires from the tail of `Unit::fight` — so
    // nine of nine says chapter two's whole engagement, both squads and
    // both directions, now opens fire on the original's own frames. That
    // is the strongest thing this capture can say and it is why the
    // count is pinned exactly: a change that bought one squad's frames by
    // losing another's fails here, in both directions.
    let exact = mine
        .iter()
        .filter(|(k, v)| theirs.get(k).is_some_and(|t| t.0 == v.0))
        .count();
    assert_eq!(
        exact, 9,
        "the nine arrivals this crate puts on the dump's own frame are \
         no longer nine"
    );
}

/// Chapter two stood up and not ticked — the dump's path, its text, and
/// the world the siblings' grid makes — for the tests that read the
/// ground rather than the game.
fn chapter_two_stood_up() -> Option<(String, String, Built)> {
    let inst = crate::testenv::install()?;
    let Some((dump, tracepath)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
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
    let built = stand_up(&loaded, &log, &refs, &trace);
    drop(log);
    Some((dump, text, built))
}

/// **The ground under every object, against the dump's own `z`**
/// (`docs/COMBAT.md` §46.2).
///
/// Three heights the original prints and this crate now reads, each swept
/// over the whole of run112 rather than the frame that asked for it:
///
/// - **a figure's `z`**, which `Guy::update_z@005d9950` writes as
///   `TerrainOut::find_data_z(x, y, 0)` for any unit not of sea domain —
///   the surface a rolled shot comes down on, read by
///   [`sim::World::data_z`] in [`sim::single::Single`] arithmetic;
/// - **a unit's own `z`**, which `Unit::update_z@00606590` writes as
///   `find_tcoord_z` at its tile — [`sim::World::tile_z`], and one of the
///   two heights `get_damage`'s step 23 compares;
/// - **a building's**, the other side of that comparison, from the start
///   dump's records.
///
/// And it pins how many figure reads touched a corner the six-decimal
/// print cannot name one single for: ten, all `1/0` walking the shore at
/// z 8–15 between 713 and 726, and every one of the ten still agrees.
#[test]
fn every_object_s_z_is_the_ground_it_stands_on() {
    let Some((dump, text, built)) = chapter_two_stood_up() else {
        return;
    };
    let log = Log::parse(&text);
    let world = &built.sim.world;
    let tile = |p: crate::gamelog::Pos| sim::Pos::new(p.x as i32, p.y as i32).tile();
    // ---- buildings, from the start dump.
    let init = log.initial().expect("the golden dump has a BEGIN GAME");
    let mut wrong_builds = Vec::new();
    for b in &init.builds {
        if i64::from(world.tile_z(tile(b.pos))) != b.pos.z {
            wrong_builds.push((b.who, b.o, b.pos, world.tile_z(tile(b.pos))));
        }
    }
    // ---- units and their figures, every block.
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let (mut figures, mut units, mut inexact, mut sea) = (0usize, 0usize, 0usize, 0usize);
    let mut wrong_units = Vec::new();
    let mut wrong_figures = Vec::new();
    let mut points = std::collections::BTreeSet::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let frame = ix.frame_state(at).unwrap();
        for u in &frame.units {
            units += 1;
            if i64::from(world.tile_z(tile(u.pos))) != u.pos.z {
                wrong_units.push((n, u.who, u.o, u.pos, world.tile_z(tile(u.pos))));
            }
            // `Guy::update_z` writes 0 for a unit of sea domain and never
            // reads the ground: Gaia's fish and whales.
            let is_sea = u8::try_from(u.who)
                .ok()
                .zip(i16::try_from(u.o).ok())
                .and_then(|(w, o)| built.sim.unit_by_o(w, o))
                .is_none_or(|i| {
                    matches!(
                        built.sim.profile(sim::combat::Obj::Unit(i)).domain,
                        sim::attrition::Domain::Sea
                    )
                });
            if is_sea {
                sea += u.guys.len();
                continue;
            }
            for (k, g) in u.guys.iter().enumerate() {
                let Some(p) = g.pos else { continue };
                let (z, exact) = world.data_z(p.x as i32, p.y as i32);
                figures += 1;
                points.insert((p.x, p.y));
                if !exact {
                    inexact += 1;
                }
                if i64::from(z) != p.z {
                    wrong_figures.push((n, u.who, u.o, k, p.x, p.y, p.z, z));
                }
            }
        }
    }
    eprintln!(
        "  {} buildings, {units} unit-blocks, {figures} figure-blocks at {} points \
         ({sea} sea figures skipped), {inexact} inexact reads",
        init.builds.len(),
        points.len()
    );
    assert!(
        wrong_builds.is_empty(),
        "building z is not tile_z: {wrong_builds:?}"
    );
    assert!(
        wrong_units.is_empty(),
        "unit z is not tile_z: {:?}",
        &wrong_units[..wrong_units.len().min(10)]
    );
    assert!(
        wrong_figures.is_empty(),
        "figure z is not find_data_z: {:?}",
        &wrong_figures[..wrong_figures.len().min(10)]
    );
    assert_eq!(inexact, 10, "reads on a corner the print cannot pin");
    // The instrument, pinned: a sweep that stopped reading would pass.
    assert!(init.builds.len() > 10, "{} buildings", init.builds.len());
    assert_eq!(
        (figures, points.len()),
        (5310, 947),
        "the figure sweep's reach"
    );
}

/// **Chapter two's shots, `v1z` to the last printed digit**
/// (`docs/COMBAT.md` §46.1) — run109's test on the golden record's own
/// 373 `AMMO` blocks, bowmen and slingers both. The rolled `0/8` shot is
/// among them, printed `18.134823` from 677 to 685; its `sz`, `ez` and
/// flight time are integers this crate reproduces, and
/// [`sim::combat::arc_v1z`] over them is that single exactly.
#[test]
fn chapter_two_s_v1z_is_arc_v1z_to_the_last_digit() {
    let Some((dump, _)) = golden("ch2") else {
        eprintln!("skipping: no golden capture ch2 (see docs/RUNS.md run112)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let (mut read, mut spent, mut wrong) = (0usize, 0usize, Vec::new());
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let body = ix.read_frame(at).unwrap();
        for (a, _) in super::ammo::blocks(&body) {
            // A spent shot (`flags 1`) keeps its launch `v1z` but carries
            // its landing's `total_time` and `ez`, which
            // [`every_rolled_shot_comes_down_where_the_original_s_does`]
            // reads instead.
            if a.flags & 2 == 0 {
                spent += 1;
                continue;
            }
            read += 1;
            let v = sim::combat::arc_v1z(a.sz as i32, a.ez as i32, a.total_time as i32);
            if super::ammo::tests::printed(v) != a.v1z {
                wrong.push((n, a.o, a.sz, a.ez, a.total_time, a.v1z));
            }
        }
    }
    assert!(wrong.is_empty(), "v1z is not arc_v1z's single: {wrong:?}");
    assert_eq!(
        (read, spent),
        (134, 239),
        "run112 carries 373 AMMO blocks, 134 of them live"
    );
}

/// **Every rolled shot of run112 comes down where the original's does**
/// (`docs/COMBAT.md` §46.1) — `Ammo::inc_time`'s rolling arm replayed from
/// the dump's own launch record, against the dump's own landing.
///
/// A shot that finds nothing where it was due (`flags 14`) flies on along
/// its line and down its arc until the ground under it is not below it.
/// Three do in run112: `0/8`'s on 683, the one chapter two's word turned
/// on, and `0/6`'s and `0/7`'s on 778 and 779. For each, from its launch
/// record alone — `sx sy sz ex ey ez total_time`, all integers — this
/// steps [`sim::combat::arc_point`] and [`sim::combat::arc_z`] against
/// [`sim::World::data_z`] from the due frame on, and asserts that the
/// first step the arc is not above the ground is the original's landing:
///
/// - for a shot that stuck in the ground, its spent record (`flags 1`)
///   prints the landing — `total_time` is the step, `ex ey` the point and
///   `ez` is `(int)z` there — so all four are asserted;
/// - for `0/8`'s, which hit `1/7` and was closed, the landing is the step
///   after its last printed block, and `damage 1/7` on 686 is the rest.
#[test]
fn every_rolled_shot_comes_down_where_the_original_s_does() {
    let Some((dump, _, built)) = chapter_two_stood_up() else {
        return;
    };
    let world = &built.sim.world;
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    // Per shot, keyed on its launch: the live record, whether it rolled,
    // its last live cur_time, and its spent record if it stuck.
    type Shot = (super::ammo::Ammo, bool, i64, Option<super::ammo::Ammo>);
    let mut shots: std::collections::BTreeMap<(i64, i64, i64), Shot> =
        std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let body = ix.read_frame(at).unwrap();
        for (a, _) in super::ammo::blocks(&body) {
            let e = shots
                .entry((a.o, a.sx, a.sy))
                .or_insert((a, false, a.cur_time, None));
            if a.flags & 2 != 0 {
                e.1 |= a.flags & 8 != 0;
                e.2 = e.2.max(a.cur_time);
            } else if e.3.is_none() {
                e.3 = Some(a);
            }
        }
    }
    let mut rolled = Vec::new();
    for ((o, ..), (a, rolls, last, spent)) in &shots {
        if !rolls {
            continue;
        }
        let (launch, landing) = (
            sim::Pos::new(a.sx as i32, a.sy as i32),
            sim::Pos::new(a.ex as i32, a.ey as i32),
        );
        let (sz, total) = (a.sz as i32, a.total_time as i32);
        let v1z = sim::combat::arc_v1z(sz, a.ez as i32, total);
        let mut down = None;
        for t in total..=3 * total {
            let at = sim::combat::arc_point(launch, landing, t, total);
            let z = sim::combat::arc_z(v1z, sz, t);
            let (ground, exact) = world.data_z(at.x, at.y);
            assert!(exact, "shot {o}: the ground at {at:?} is not pinned");
            if !z.gt(sim::single::Single::from_i32(ground)) {
                down = Some((t, z.to_i32(), at, ground));
                break;
            }
        }
        let (t, z, at, ground) = down.unwrap_or_else(|| panic!("shot {o} never comes down"));
        match spent {
            Some(s) => {
                assert_eq!(
                    (i64::from(t), i64::from(at.x), i64::from(at.y), i64::from(z)),
                    (s.total_time, s.ex, s.ey, s.ez),
                    "shot {o}: lands on step {t} at {at:?}, z {z}; the original's spent \
                     record says step {} at ({}, {}), z {}",
                    s.total_time,
                    s.ex,
                    s.ey,
                    s.ez
                );
            }
            None => assert_eq!(
                i64::from(t),
                last + 1,
                "shot {o}: lands on step {t}; the original closed it after step {last}"
            ),
        }
        rolled.push((*o, t, z, at, ground));
    }
    eprintln!("  rolled shots (o, step, z, point, ground): {rolled:?}");
    assert_eq!(
        rolled.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>(),
        vec![(6, 11), (7, 10), (8, 9)],
        "run112's three rolled shots and their landing steps"
    );
}

/// **Chapter five's word, widened whole, both directions** (item 535,
/// `docs/DECISIONS.md` 43). Every record run127 carries on every block
/// of [`WIDENING_CHAPTER_FIVE`]: [`crate::diff::harness::widen_block`] on
/// every unit and figure, the leader record whole for both players
/// ([`crate::diff::leader::rows`] against [`crate::diff::leader::theirs`]),
/// and the **`AMMO` record**, which neither of the other two widenings
/// compares. It is compared shot by shot: the live rounds (`flags & 2`)
/// keyed on the shooter and the pool slot, against this crate's
/// [`sim::combat::Projectile`]s. Each key's first parting block is kept
/// with the value diff beside it.
///
/// The trace's frame `f` writes block `f + 1`, so the word 617 enters on
/// block 617 and leaves on 618. Both blocks are printed once, both sides,
/// for the ships, before any quiet row is trusted.
#[test]
fn chapter_five_s_word_frame_is_widened_whole() {
    use std::collections::{BTreeMap, BTreeSet};
    const FIRST: i64 = WIDENING_CHAPTER_FIVE.0;
    const LAST: i64 = WIDENING_CHAPTER_FIVE.1;
    /// run127's window is 605..899 and its `!quit` block is 901.
    const RUN127_NO_BLOCK: i64 = 900;
    /// **The frame item 549 moved the word from.** With the word at the
    /// capture's end there is no word block to print, so the record the
    /// last word was spent in is printed instead: `1/6` on 739 and 740,
    /// where this crate's swing was `CHAR_ATTACK3` three frames long, and
    /// the round on 742 it then failed to release.
    const MOVED_FROM: i64 = 739;
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch5") else {
        eprintln!("skipping: no golden capture ch5 (see docs/RUNS.md run127)");
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
    let mut script = chapter(5);
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows) = (0usize, 0usize, 0usize);
    let (mut ammo_theirs, mut ammo_ours) = (0usize, 0usize);
    let ship = |who: i64, o: i64| who < 2 && o >= 6;
    for f in 0..LAST - 1 {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = ix.frame_state(at).unwrap();
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&built, &frame, players, n, &mut firsts);
        rows += k;
        let raw = ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&loaded, &built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    firsts
                        .entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **The `AMMO` record, both directions.** A live round is keyed on
        // its shooter and its pool slot, the order `Objects::inc_time`
        // steps it in (`docs/COMBAT.md` §46.4).
        let theirs: BTreeMap<(i64, i64, i64), super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0)
            .map(|(a, _)| ((a.who, a.o, a.index), a))
            .collect();
        let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = built
            .sim
            .projectiles
            .iter()
            .filter_map(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => {
                    let un = &built.sim.units[u];
                    Some((
                        (i64::from(un.owner), i64::from(un.index), i64::from(p.slot)),
                        *p,
                    ))
                }
                sim::combat::Obj::Building(_) => None,
            })
            .collect();
        ammo_theirs += theirs.len();
        ammo_ours += ours.len();
        // **The value diff on the frame the word moved** (item 549): the
        // round the dump held alone on 742, now in both airs.
        if n == MOVED_FROM + 3 {
            for (key, a) in theirs.iter().filter(|(k, _)| (k.0, k.1) == (1, 6)) {
                eprintln!("  block {n} 1/6 ammo{key:?} theirs {a:?}");
                eprintln!("  block {n} 1/6 ammo{key:?} ours {:?}", ours.get(key));
            }
        }
        let keys: BTreeSet<_> = theirs.keys().chain(ours.keys()).copied().collect();
        for key @ (who, o, slot) in keys {
            let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                let side = if theirs.contains_key(&key) {
                    "the dump"
                } else {
                    "this crate"
                };
                firsts
                    .entry((who, o, format!("ammo[{slot}]")))
                    .or_insert((n, format!("{side} holds it alone")));
                continue;
            };
            let target = p.target.map_or((-1, -1), |t| match t {
                sim::combat::Obj::Unit(u) => {
                    let tu = &built.sim.units[u];
                    (i64::from(tu.owner), i64::from(tu.index))
                }
                sim::combat::Obj::Building(_) => (-2, -2),
            });
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("whom", target.0, a.whom),
                ("ox", target.1, a.ox),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
                // Item 542: the launch height, the aim height, the
                // bearing, the arc and the roll flag, the rest of the
                // record this crate carries. `traj`, `dx`, the roll and
                // bank angles, `gpiece` and `graph_index` it does not.
                ("sz", i64::from(p.sz), a.sz),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
                // Item 602: flag 4 is `flags & 4`, and `rolling` is
                // `AmmoData +0x5`, a byte `Ammo::init` zeroes on every
                // round (see `ammo_flag_rows`).
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                if mine != dumped {
                    firsts
                        .entry((who, o, format!("ammo[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
        // **Both sides printed once on the word's two blocks**, for the
        // three hulls: the record the draw is spent in, before any quiet
        // row is trusted.
        if (MOVED_FROM..=MOVED_FROM + 1).contains(&n) {
            for them in frame.units.iter().filter(|u| ship(u.who, u.o)) {
                let mine = u8::try_from(them.who)
                    .ok()
                    .zip(i16::try_from(them.o).ok())
                    .and_then(|(w, o)| built.sim.unit_by_o(w, o))
                    .map(|u| &built.sim.units[u]);
                eprintln!("  block {n} {}/{} theirs {them:?}", them.who, them.o);
                match mine {
                    Some(u) => eprintln!(
                        "  block {n} {}/{} ours pos {:?} heading {:?} facing {:?} \
                         orders {:?} guys {:?}",
                        them.who,
                        them.o,
                        u.pos,
                        u.movement.heading,
                        u.movement.facing,
                        u.orders,
                        u.guys
                    ),
                    None => eprintln!("  block {n} {}/{} ours: absent", them.who, them.o),
                }
            }
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        let near = (MOVED_FROM - 4..=GOLDEN_WORD_CHAPTER_FIVE + 2).contains(f);
        if near || rows.len() <= 4 {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys", rows.len());
        }
    }
    eprintln!(
        "ch5 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {leader_rows} leader rows, ammo {ammo_theirs} theirs / {ammo_ours} ours, \
         {} keys parted",
        firsts.len()
    );
    // `LEADERS=2` prints the goods block and nothing after it, so the
    // leader rows this capture can compare are the goods' own. The keys it
    // does not print are absent rather than divergent, and what is pinned
    // is that the goods rows were actually taken on every block.
    eprintln!(
        "ch5: {} leader keys not printed at LEADERS=2",
        missing.len()
    );
    assert_eq!(
        leader_rows,
        2 * 88 * blocks,
        "the leader rows LEADERS=2 prints (88 a player) are not compared on \
         every block"
    );
    // **Anti-vacuity for the `AMMO` record**: run127's 506 live rounds
    // are all read, and this crate's side is not empty.
    assert_eq!(ammo_theirs, 506, "run127's live rounds are not all read");
    assert!(ammo_ours > 0, "this crate fired no round in the window");
    // **The map's shape.** Three standing families from the capture's
    // first block, none of them the water's: `build:extra` (the end
    // detail prints no `BUILDDATA`), the unmodelled `form`, and two
    // `filled_gather_slots`. `form` is also every hull's birth row.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    assert!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 26,
        "the standing rows on run127's first block moved: {at_floor:?}"
    );
    // **Nothing parts under the word's own block.** The trace's frame
    // `f` writes block `f + 1`. Item 535's broadside (`docs/COMBAT.md`
    // §49) closed the four `1/6` angle rows on block 617, item 542's
    // release frame and keel nodes (§50) the first round on 622, and item
    // 543's human rare-collector arm (`docs/ORDERS.md` §23) the fisher
    // `0/7`'s: its birth `[MOVE_TO, CAST 0x292]` on 621, the walk from
    // 622, and the deploy on 665 with `packed` and `mylos` 4 → 6. Made to
    // fail once with the arm removed: the word fell back to 664, all
    // fifteen of the fisher's rows came back, and so did its leader's
    // food and wealth rows from 673, the deployed boat's pay.
    //
    // **Item 549 took the word to the capture's end** (`docs/ANIM.md`
    // §4.13): a rolled `CHAR_ATTACK3` the Trireme's packet does not name
    // plays `CHAR_ATTACK2`, forty frames, not three. With the fallback
    // removed the word falls back to 739 and the map holds 224 keys, the
    // first `1/6 ammo[0]` on 742; with it, 30.
    // One row stands under the word and it is not the water's: the
    // explore order of who=1's scout `1/0` carries the formation mirror
    // `facing` 1 against the dump's 0 from 847. That field is declared
    // non-scoring (`OrderMismatch::scores`, parked 275), and Great Lakes'
    // `1/0` carries the same row on 8002. It is pinned by name and value
    // so it cannot stand in for anything else.
    let under: Vec<String> = firsts
        .iter()
        .filter(|((_, _, what), (f, _))| *f <= GOLDEN_WORD_CHAPTER_FIVE && !standing(what))
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    assert_eq!(
        under,
        vec![
            "847 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }".to_string()
        ],
        "what parts at or under the word moved"
    );
    // **Every round of the capture agrees whole** (items 542 and 549):
    // all 506 live rounds of both ships, on every `AMMO` field this crate
    // carries, `v1z` to the last printed digit. Before 549 the first shot
    // row was `1/6 ammo[0]` on 742, the round the dump held alone.
    let first_ammo = firsts
        .iter()
        .filter(|((_, _, what), _)| what.starts_with("ammo["))
        .map(|(_, (f, _))| *f)
        .min();
    assert_eq!(
        first_ammo, None,
        "a round of the capture parts: first ammo row on {first_ammo:?}"
    );
    assert_eq!(ammo_ours, 506, "this crate's rounds are not run127's 506");
    let absent = usize::from((FIRST..LAST).contains(&RUN127_NO_BLOCK));
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "run127's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
}

/// Chapter four's two captures, stood up and staged to a block: the dump
/// and trace of `run`, the harness built from its own start block, and the
/// script. The two widenings below share it; it is the body
/// `chapter_five_s_word_frame_is_widened_whole` has inline.
struct Staged4 {
    loaded: crate::load::Loaded,
    built: Built,
    ix: crate::capture::indexed::IndexedCapture,
    script: Script,
}

fn stage_chapter_four(run: &str) -> Option<Staged4> {
    stage_script(run, "chapter4")
}

/// [`stage_chapter_four`]'s body for any script under
/// `tools/gamelog/golden/`, by stem — chapter seven's two widenings stage
/// the chapter and its control through it (item 578).
fn stage_script(run: &str, stem: &str) -> Option<Staged4> {
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
    let built = stand_up(&loaded, &log, &refs, &trace);
    let ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    Some(Staged4 {
        loaded,
        built,
        ix,
        script: script_named(stem),
    })
}

/// **Chapter four's two captures are one game** (item 552). run132 and
/// run133 are the same seed and the same script, windowed on different
/// frames at different detail, and each carries the trace of the whole
/// run whatever its window. So the draw stream must be the same on every
/// frame of the two, and that is what makes the border's window and the
/// bleed's two views of one game rather than two games.
#[test]
fn chapter_four_s_two_captures_are_one_game() {
    let (Some((_, a)), Some((_, b))) = (golden("ch4b"), golden("ch4u")) else {
        eprintln!("skipping: chapter four needs both run132 and run133 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let last = |t: &crate::trace::Trace| t.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last(&ta) >= 1500 && last(&tb) >= 1500,
        "chapter four's traces end at {} and {}; both runs go to 1500",
        last(&ta),
        last(&tb)
    );
    let mut draws = 0usize;
    let mut parted = Vec::new();
    for f in 0..=1500 {
        let (la, lb) = (ta.labels(f), tb.labels(f));
        draws += la.len();
        if la != lb {
            parted.push((f, la.len(), lb.len()));
        }
    }
    eprintln!("chapter four: {draws} draws over 1501 frames in each trace");
    assert!(draws > 0, "chapter four's traces carry no draw");
    assert_eq!(
        parted,
        vec![],
        "run132's and run133's draw streams part: (frame, run132, run133)"
    );
}

/// **Chapter four's border, widened cell for cell** (item 552, run132).
/// Every block of [`WIDENING_CHAPTER_FOUR_BORDER`]: the whole `WORLD`
/// record, all 3,600 cells, compared on `who`, `who2`, `flags`, `blocked`,
/// `solid` and `bad` as `run93_s_block_7932_is_this_crate_s_world_cell_for_cell`
/// compares a cell; and the `BUILDDATA` and `CITIES` records through
/// [`crate::diff::harness::widen_block`], which carries the Temple's own
/// record and Napata's `city_flags` bit by bit. run132 prints no
/// `UNITDATA` in its window, so a unit's rows are absent there rather than
/// divergent and are dropped.
///
/// A cell's key is its index, `y · 60 + x`, and its first parting block
/// is kept with the value diff beside it. The owner counts go beside the
/// keys, block by block, both sides, because the original spreads each
/// lever's change over several blocks (`GameDaemon::check_borders`, 256
/// cells a frame) and this crate recomputes wholesale.
#[test]
fn chapter_four_s_border_is_widened_cell_for_cell() {
    use std::collections::BTreeMap;
    const FIRST: i64 = WIDENING_CHAPTER_FOUR_BORDER.0;
    const LAST: i64 = WIDENING_CHAPTER_FOUR_BORDER.1;
    let Some(mut s) = stage_chapter_four("ch4b") else {
        return;
    };
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut blocks, mut cells_compared) = (0usize, 0usize);
    let mut counts: Vec<(i64, [i64; 2], [i64; 2], usize)> = Vec::new();
    let mut applied = crate::golden::Applied::default();
    for f in 0..LAST - 1 {
        let did = s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        applied.merge(&did);
        s.built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        let mut unit_rows: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut unit_rows);
        for (k, v) in unit_rows {
            // No `UNITDATA` in this window: a unit is "extra" on every
            // block and says nothing. Buildings are `o` 2000 and up.
            if k.2 == "extra" && k.1 < 2000 {
                continue;
            }
            firsts.entry(k).or_insert(v);
        }
        let raw = s.ix.read_frame(at).unwrap();
        let parsed = Log::parse(&raw);
        let Some((_, block)) = parsed.frames().into_iter().find(|(k, _)| *k == n) else {
            continue;
        };
        let Some(world) = block.kid("WORLD") else {
            continue;
        };
        let fields = world.fields().to_vec();
        let theirs = crate::gamelog::world_cells(&fields);
        assert_eq!(
            theirs.len(),
            3_600,
            "run132's block {n} is not a whole WORLD scan"
        );
        let w = &s.built.sim.world;
        let code = |o: sim::world::Owner| match o {
            sim::world::Owner::Player(p) => i64::from(p),
            sim::world::Owner::Ambiguous => -2,
            sim::world::Owner::None => -1,
        };
        let (mut ours_n, mut theirs_n) = ([0i64; 2], [0i64; 2]);
        let mut owners_part = 0usize;
        for y in 0..w.height() {
            for x in 0..w.width() {
                let c = sim::world::Cell::new(x, y);
                let i = (y * w.width() + x) as usize;
                let t = &theirs[i];
                let d = w.cell_data(c);
                let (who, who2) = (code(w.owner(c)), code(w.second(c)));
                for p in 0..2 {
                    ours_n[p] += i64::from(who == p as i64);
                    theirs_n[p] += i64::from(t.who == p as i64);
                }
                cells_compared += 1;
                owners_part += usize::from(who != t.who);
                for (name, mine, dumped) in [
                    ("who", who, t.who),
                    ("who2", who2, t.who2),
                    ("flags", i64::from(d.flags), t.flags),
                    ("blocked", i64::from(d.blocked), t.blocked),
                    ("solid", i64::from(d.solid), t.solid),
                    ("bad", i64::from(d.bad), t.bad),
                ] {
                    if mine != dumped {
                        firsts
                            .entry((-3, i as i64, format!("cell({x},{y}).{name}")))
                            .or_insert((n, format!("ours {mine} theirs {dumped}")));
                    }
                }
            }
        }
        if counts
            .last()
            .is_none_or(|l| (l.1, l.2, l.3) != (ours_n, theirs_n, owners_part))
        {
            counts.push((n, ours_n, theirs_n, owners_part));
        }
    }
    eprintln!(
        "chapter four staged: {} line(s) ran, {} unit(s), {} building(s)",
        applied.ran, applied.units, applied.buildings
    );
    for ((word, why), k) in &applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    for (n, o, t, k) in &counts {
        eprintln!("  block {n}: owner 0/1 cells ours {o:?} theirs {t:?}, {k} owners part");
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if rows.len() <= 12 {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "ch4 border widening: {blocks} blocks [{FIRST}, {LAST}), {cells_compared} cells, \
         {} keys parted",
        firsts.len()
    );
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize,
        "run132's dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    // **The three levers, each settled cell for cell** (item 552). The
    // original's owner-0 count is 266, then 296 from block 310 (the
    // Temple), 327 from 411 (Religion, temple level 2) and 445 from 511
    // (Civic 3). The count is §8's falsifier and it did not fire on any
    // lever; this crate reaches each figure on every cell once the sweep
    // is done. Each of the item's three fixes was found by this test's
    // counts before it had assertions: the interpreter's old
    // `place_building` arm left an unstarted site (266 throughout), the
    // temple level was a constant 1 (327 never reached), and
    // `set_leader_epoch` skipped `gain_tech`'s tail (445 never reached).
    // Made to fail once with the last reverted: the settled list stops at
    // 411.
    let settled: Vec<(i64, i64)> = counts
        .iter()
        .filter(|c| c.3 == 0)
        .map(|c| (c.0, c.2[0]))
        .collect();
    assert_eq!(
        settled,
        vec![(295, 266), (310, 296), (411, 327), (511, 445)],
        "the blocks on which every owner agrees, with the original's count"
    );
    assert!(
        counts.iter().all(|c| c.2[1] == 261 && c.1[1] == 261),
        "player 1's 261 cells moved on one side"
    );
    // **What parts is the sweep, and only the sweep.** The original spreads
    // each lever over the blocks `GameDaemon::check_borders` takes at 256
    // cells a frame; this crate recomputes wholesale on the line's own
    // frame (`docs/ATTRITION.md`, "Territory"). So a `who` row may part
    // inside the three windows and nowhere else, and every one has closed
    // by the next settled block above.
    let sweeping =
        |f: i64| (301..310).contains(&f) || (401..411).contains(&f) || (501..511).contains(&f);
    let cell_rows: Vec<String> = firsts
        .iter()
        .filter(|((w, _, what), (f, _))| *w == -3 && !(what.ends_with(".who") && sweeping(*f)))
        .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
        .collect();
    assert_eq!(
        cell_rows,
        Vec::<String>::new(),
        "a cell parts outside the sweep: the Temple's footprint, `who2`, or \
         a steady-state owner"
    );
    // The records beside the cells: the 14 standing rows of run132's first
    // block — `0/2000`'s AI half of Napata's record, which `ai off` leaves
    // this crate holding at zero, and London's `filled`/`land` one apart —
    // and nothing else. The Temple's `BUILDDATA` and Napata's temple bit
    // (`city_flags[0x80]`, 0 → 1 on block 301) agree on every block.
    let other: Vec<String> = firsts
        .iter()
        .filter(|((w, _, _), (f, _))| *w != -3 && *f != FIRST)
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    assert_eq!(other, Vec::<String>::new(), "a building or city row parts");
    // **run132's first parting is 301**, the Temple's frame, and it is the
    // sweep's: 30 cells this crate owns a sweep early. The chapter's word,
    // 1277, is in run133's window; this is the border window's own.
    let first_parting = firsts
        .iter()
        .filter(|(_, (f, _))| *f != FIRST)
        .map(|(_, (f, _))| *f)
        .min();
    assert_eq!(first_parting, Some(301), "run132's first parting moved");
    assert_eq!(
        firsts.iter().filter(|(_, (f, _))| *f == FIRST).count(),
        14,
        "the standing rows on run132's first block moved"
    );
}

/// **Chapter four, walked** — the border and the bleed (`docs/GOLDEN.md`
/// §8, item 552, run133). Eight staged lines: `!ai off`, the Temple, the
/// two techs and the civic level, the squad, the scout and the wagon.
#[test]
fn chapter_four_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch4u", 4, 8, 1500) else {
        return;
    };
    assert!(
        w.word >= GOLDEN_WORD_CHAPTER_FOUR,
        "chapter four's golden word fell to {} from {GOLDEN_WORD_CHAPTER_FOUR}",
        w.word
    );
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_FOUR,
        "chapter four's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §8"
    );
    eprintln!(
        "chapter four: sequence {}, values {:?}",
        w.sequence, w.value
    );
}

/// **Chapter four's word, widened whole, both directions** (item 552,
/// `docs/DECISIONS.md` 43). Every record run133 carries on every block of
/// [`WIDENING_CHAPTER_FOUR`]: [`crate::diff::harness::widen_block`] on
/// every unit and figure — which compares the namesake's two fields,
/// `attrition` and `unit_masks2`'s supply mark, since this item — and the
/// leader record whole for both players at `LEADERS=2`. Each key's first
/// parting block is kept with the value diff beside it.
///
/// **The bleed, printed both sides.** For each of player 1's staged units
/// (the three hoplite figures, the scout and the wagon) a line on every
/// block its `attrition`, `damage`, `damage_frac`, supply mark or cell
/// owner changes on either side.
#[test]
fn chapter_four_s_word_frame_is_widened_whole() {
    use std::collections::BTreeMap;
    const FIRST: i64 = WIDENING_CHAPTER_FOUR.0;
    const LAST: i64 = WIDENING_CHAPTER_FOUR.1;
    let Some(mut s) = stage_chapter_four("ch4u") else {
        return;
    };
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut missing = std::collections::BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows) = (0usize, 0usize, 0usize);
    let mut last_seen: BTreeMap<(i64, i64), String> = BTreeMap::new();
    for f in 0..LAST - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut firsts);
        rows += k;
        let raw = s.ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&s.loaded, &s.built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    firsts
                        .entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **Both sides printed once on the word's two blocks** (DECISIONS
        // 43), for player 1's staged units: the record the draw is spent
        // in, before any quiet row is trusted.
        if (GOLDEN_WORD_CHAPTER_FOUR..=GOLDEN_WORD_CHAPTER_FOUR + 1).contains(&n)
            || (1416..=1417).contains(&n)
        {
            for them in frame.units.iter().filter(|u| u.who == 1 && u.o >= 6) {
                let mine = u8::try_from(them.who)
                    .ok()
                    .zip(i16::try_from(them.o).ok())
                    .and_then(|(w, o)| s.built.sim.unit_by_o(w, o))
                    .map(|u| &s.built.sim.units[u]);
                eprintln!("  block {n} 1/{} theirs {them:?}", them.o);
                match mine {
                    Some(u) => eprintln!(
                        "  block {n} 1/{} ours pos {:?} heading {:?} orders {:?} guys {:?}",
                        them.o, u.pos, u.movement.heading, u.orders, u.guys
                    ),
                    None => eprintln!("  block {n} 1/{} ours: absent", them.o),
                }
            }
        }
        // **The wagon's first route, on the block it is planned** (item
        // 569): the first parting of `1/10` in run133, and the frame this
        // chapter's word 1416 traces back to. The Supply Wagon is born on
        // 1101 and sent after its army's first member (`docs/ARMY.md`
        // §4.3), and its `find_wpath` from cell (6, 44) to (13, 37) is the
        // first of a supply unit anywhere in the corpus. The original's
        // route, in world cells, top first: one step **south-east** to
        // (7, 45), then the diagonal, then north along x = 12 — every cell
        // of it clear of the `0x200` flag, which an army pays `base << 5`
        // for (`docs/PATHFINDER.md` §5). Printed both sides by cell.
        //
        // **And this crate's is the same route since item 569**: the
        // wagon's `find_wpath` takes the army mode through `is_supply`'s
        // arm (`docs/PATHFINDER.md` §25). Without that arm it went (7, 43)
        // north-east through the `0x200` cells, one leg shorter, and
        // walked apart from the original's from 1102 on.
        if n == 1101 {
            let cells = |v: Vec<(i64, i64)>| -> Vec<(i64, i64)> {
                v.into_iter()
                    .rev()
                    .map(|(x, y)| (x / 768, y / 768))
                    .collect()
            };
            let theirs = cells(
                frame
                    .units
                    .iter()
                    .find(|u| u.who == 1 && u.o == 10)
                    .expect("the original's wagon is born on 1101")
                    .path
                    .iter()
                    .map(|p| p.to)
                    .collect(),
            );
            let u = s
                .built
                .sim
                .unit_by_o(1, 10)
                .expect("this crate's wagon on 1101");
            let ours = cells(
                s.built.sim.units[u]
                    .path
                    .iter()
                    .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
                    .collect(),
            );
            eprintln!("  route 1101 1/10 theirs {theirs:?}");
            eprintln!("  route 1101 1/10 ours   {ours:?}");
            assert_eq!(
                theirs,
                vec![
                    (7, 45),
                    (8, 44),
                    (9, 43),
                    (10, 42),
                    (11, 41),
                    (12, 40),
                    (12, 39),
                    (12, 38),
                    (13, 37)
                ],
                "run133's wagon route on 1101"
            );
            assert_eq!(ours, theirs, "this crate's wagon route on 1101");
        }
        // **The escort's own row on the block it is issued** (item 567,
        // `docs/ORDERS.md` §24), read off both sides by name rather than
        // through `compare`'s labels: each hoplite's `GUARDORDER` is on the
        // wagon `1/10`, and its offset and both counters are the
        // original's — (0, 264), (144, 264), (−144, 264), `idle` and
        // `retry` 0. The post, which is the wagon's position plus the
        // offset, differed until item 569 walked the wagon on the
        // original's line (`docs/PATHFINDER.md` §25); it agrees now.
        if n == 1277 {
            for (o, dx) in [(6, 0), (7, 144), (8, -144)] {
                let them = frame
                    .units
                    .iter()
                    .find(|u| u.who == 1 && u.o == o)
                    .and_then(|u| u.orders.iter().find(|x| x.index == 12))
                    .expect("the original's escort holds a GUARDORDER on 1277");
                let theirs = (
                    them.ox,
                    them.guard_dx,
                    them.guard_dy,
                    them.guard_idle,
                    them.guard_retry,
                );
                assert_eq!(theirs, (Some(10), Some(dx), Some(264), Some(0), Some(0)));
                // **And the post itself since item 569**: the wagon walks
                // the original's line, so the wagon's position plus the
                // offset is the original's point on all three figures.
                let post = them.guard_x.zip(them.guard_y);
                let u = s.built.sim.unit_by_o(1, i16::try_from(o).unwrap()).unwrap();
                let g = s.built.sim.units[u]
                    .orders
                    .iter()
                    .find_map(|x| match x.body {
                        sim::orders::Body::Guard(g) => Some(g),
                        _ => None,
                    })
                    .expect("this crate's escort holds a GUARD on 1277");
                let ours = (
                    s.built.unit_ids(g.target).map(|(_, o)| o),
                    Some(i64::from(g.dx)),
                    Some(i64::from(g.dy)),
                    Some(i64::from(g.idle)),
                    Some(i64::from(g.retry)),
                );
                assert_eq!(ours, theirs, "1/{o}'s guard row");
                assert_eq!(
                    Some((i64::from(g.guard.x), i64::from(g.guard.y))),
                    post,
                    "1/{o}'s post on 1277"
                );
            }
        }
        // The bleed's own timeline: player 1's staged units, `o` 6 and up.
        let w = &s.built.sim.world;
        for them in frame.units.iter().filter(|u| u.who == 1 && u.o >= 6) {
            let mine = u8::try_from(them.who)
                .ok()
                .zip(i16::try_from(them.o).ok())
                .and_then(|(w, o)| s.built.sim.unit_by_o(w, o))
                .map(|u| &s.built.sim.units[u]);
            let cell = |x: i64, y: i64| {
                let c = sim::Pos::new(x as i32, y as i32).cell();
                w.owner(c).player().map_or(-1, i64::from)
            };
            let theirs_row = format!(
                "attr {:?} dmg {:?}+{:?}/16 shelter {:?} on {}",
                them.attrition,
                them.damage,
                them.damage_frac,
                them.unit_masks2.map(|m| i64::from(m & 0x4_0000 != 0)),
                cell(them.pos.x, them.pos.y)
            );
            let ours_row = mine.map_or("absent".to_string(), |u| {
                format!(
                    "attr {} hp {}/{} +{}/16 shelter {} on {}",
                    u.attrition,
                    u.health,
                    u.max_health,
                    u.damage_frac,
                    i64::from(u.sheltered),
                    w.owner_at(u.pos).player().map_or(-1, i64::from)
                )
            });
            let row = format!("theirs {theirs_row} | ours {ours_row}");
            if last_seen.get(&(them.who, them.o)) != Some(&row) {
                eprintln!("  bleed {n} 1/{} {row}", them.o);
                last_seen.insert((them.who, them.o), row);
            }
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if rows.len() <= 12 || (GOLDEN_WORD_CHAPTER_FOUR..=GOLDEN_WORD_CHAPTER_FOUR + 1).contains(f)
        {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "ch4 widening: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {leader_rows} leader rows, {} keys parted; {} leader keys not printed \
         at LEADERS=2",
        firsts.len(),
        missing.len()
    );
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize - 1,
        "run133's dump no longer carries every frame of [{FIRST}, {LAST}) \
         but 1500, the block its `!quit` replaces"
    );
    assert_eq!(
        leader_rows,
        2 * 88 * blocks,
        "the leader rows LEADERS=2 prints (88 a player) are not compared on \
         every block"
    );
    // **The standing rows of run133's first block**, chapter five's three
    // families and nothing of this chapter's: `build:extra` (the end
    // detail prints no `BUILDDATA`), the unmodelled `form`, and two
    // `filled_gather_slots`.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    assert!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 27,
        "the standing rows on run133's first block moved: {at_floor:?}"
    );
    // **The bleed agrees, tick for tick, on every block of run133** (item
    // 552, item 567). Every hoplite figure's period, every 6/16 and every
    // whole point of `damage` from block 601 to 1500, the scout's and the
    // wagon's zero, and the supply mark. Item 552 left the first row of
    // the namesake's record on 1338, `1/7 attrition: ours 0 theirs 48`,
    // after this crate had walked `1/7` onto unowned ground; with the
    // escort built (item 567) the squad stays beside its wagon as the
    // original's does, and the row is gone. It took two fixes: `Leader::calc_attrition` wired to the tech
    // tree (the period was 0 on every figure, `docs/ATTRITION.md`
    // "Strength") and `curr_uber_size` counted rather than stored (each
    // tick took 16/16, not 6/16). Made to fail once with the second
    // reverted: `1/8 damage_frac` parts on 617.
    let bleed = |what: &str| {
        matches!(
            what,
            "attrition" | "sheltered" | "damage_frac" | "hits_left" | "myhits"
        ) || what.starts_with("hits:")
    };
    let first_bleed = firsts
        .iter()
        .filter(|((_, _, what), _)| bleed(what))
        .map(|((w, o, what), (f, row))| (*f, format!("{w}/{o} {what}: {row}")))
        .min();
    assert_eq!(
        first_bleed, None,
        "the namesake's record parts somewhere new"
    );
    // **What parts under the word**, none of it a draw, and the word is
    // the capture's end: the scout's explore-order `facing` (the declared
    // non-scoring formation mirror, parked 275, as chapter five's `1/0` on
    // 847); the squad's group id (`1020001` against `1026401`, a
    // numbering, on 1021); and the scout's second figure three units off
    // on 1172. Pinned by key and block so none of them can stand in for
    // anything else.
    //
    // **Item 569 took eighty-nine rows off this list.** Twenty were the
    // wagon's first route on 1101, one leg short, and its walk from 1102.
    // The other sixty-nine were the escort's posts from 1277: the post is
    // the wagon's position plus the slot offset (`docs/ORDERS.md` §24.6),
    // so it could not agree until the wagon's walk did. Both were
    // `find_wpath`'s missing `is_supply` arm (`docs/PATHFINDER.md` §25).
    let under: Vec<String> = firsts
        .iter()
        .filter(|((_, _, what), (f, _))| {
            *f > FIRST && *f < GOLDEN_WORD_CHAPTER_FOUR && !standing(what)
        })
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    let mut want: Vec<String> = [
        "767 1/9 order:move.facing",
        "1021 1/6 order:group.id",
        "1021 1/7 order:group.id",
        "1021 1/8 order:group.id",
        "1172 1/9 g.x[1]",
        "1172 1/9 g.y[1]",
        "1173 1/9 g.angle[1]",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let mut under_sorted = under.clone();
    under_sorted.sort();
    want.sort();
    assert_eq!(under_sorted, want, "what parts under the word moved");
    // **The move's block, 1416** (item 569; its delta is in
    // `GOLDEN_WORD_CHAPTER_FOUR`'s comment). Item 567 left two rows here:
    // the wagon's `pause`, 15 in the original and 0 here, and `1/8`'s half
    // step. The pause is `do_attack_to_pause`'s (`docs/ORDERS.md` §24.9):
    // on (1415 + 10) % 15 == 0 the wagon has the escort's captain within
    // `0x600`, and it waits. Now every record agrees on 1416 and on every
    // block above the scout's 1173, to the end of the capture.
    let above: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > 1173)
        .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
        .collect();
    assert_eq!(
        above,
        Vec::<String>::new(),
        "a row parts above 1173, where run133 agrees to its end"
    );
}

/// **Chapter seven's pair is one game until `ai off` is first read**
/// (item 578, `docs/GOLDEN.md` §11). run141 is `chapter7.cmd` and run142
/// `chapter7_control.cmd`, the same file less `0 !ai off`: the same seed,
/// the same lobby and every other line on the same frame. The flag's first
/// reader is `Leader::production_ai` (`docs/INPUT.md` §11.4), so the two
/// draw streams must agree through frame 0 and part on frame 1. Both
/// traces cover all 1201 frames whatever the dump's window.
#[test]
fn chapter_seven_s_pair_is_one_game_until_the_gate() {
    let (Some((_, a)), Some((_, b))) = (golden("ch7"), golden("ch7c")) else {
        eprintln!("skipping: chapter seven needs both run141 and run142 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let last = |t: &crate::trace::Trace| t.frames.last().map_or(0, |(f, _)| *f);
    assert!(
        last(&ta) >= 1200 && last(&tb) >= 1200,
        "chapter seven's traces end at {} and {}; both runs go to 1200",
        last(&ta),
        last(&tb)
    );
    let parted = (0..=1200).find(|&f| ta.labels(f) != tb.labels(f));
    let agreed: usize = (0..parted.unwrap_or(1201))
        .map(|f| ta.labels(f).len())
        .sum();
    let (da, db): (usize, usize) = (0..=1200)
        .map(|f| (ta.labels(f).len(), tb.labels(f).len()))
        .fold((0, 0), |(x, y), (p, q)| (x + p, y + q));
    eprintln!(
        "chapter seven: run141 {da} draws, run142 {db}; {agreed} identical \
         before the parting at {parted:?}"
    );
    if let Some(f) = parted {
        eprintln!(
            "  frame {f}: run141 {} draws, run142 {}",
            ta.labels(f).len(),
            tb.labels(f).len()
        );
    }
    assert_eq!(
        parted,
        Some(1),
        "the pair parts where `ai off` is first read"
    );
    assert_eq!(agreed, 120, "frame 0's draws, identical in the pair");
}

/// **What `ai off` takes away from a human's civilians: nothing** (item
/// 578, run141 and run142; `docs/INPUT.md` §11.9). The five civilians
/// `0/6..0/10` — Citizen, Caravan, Merchant, Scholar, Fur Trapper, born on
/// blocks 611, 616, 621, 626 and 631 — hold the same order kind on every
/// block of both captures. The citizen takes a `GATHERORDER` on 763 with
/// `idle` 12, which is `Unit::think_peasant`'s human wait (item 494): the
/// arm sits above `Unit::think`'s `ai off` block (`think@005f6e40:154`–
/// `158`), and for who=0 that block is entered either way, because the
/// human carries `leader_flags & 4`. The other four take no order at all
/// in either capture — including the caravan, whose `think_caravan` is
/// also above the block and finds no trade city.
///
/// So `docs/GOLDEN.md` §11's first falsifier fires as written (an order on
/// the five in the AI-off run) and its second does not (the control's
/// citizen acts): the gate does what §11.4's corrected reading says and
/// §11's premise — that the cheat silences a human's civilians — is wrong.
#[test]
fn chapter_seven_s_civilians_act_alike_with_the_ai_off_and_on() {
    use std::collections::BTreeMap;
    let (Some((a, _)), Some((b, _))) = (golden("ch7"), golden("ch7c")) else {
        eprintln!("skipping: chapter seven needs both run141 and run142 (docs/RUNS.md)");
        return;
    };
    // (block, o) → the order list's kinds, newest first as the log writes
    // them, and `idle`, for player 0's `o` 6..=10.
    type Kinds = BTreeMap<(i64, i64), (Vec<String>, Option<i64>)>;
    let kinds = |p: &str| -> Kinds {
        let mut ix = crate::capture::indexed::IndexedCapture::open(p).unwrap();
        let mut out = Kinds::new();
        for at in 0..ix.frames().len() {
            let frame = ix.frame_state(at).unwrap();
            for u in frame
                .units
                .iter()
                .filter(|u| u.who == 0 && (6..=10).contains(&u.o))
            {
                out.insert(
                    (frame.n, u.o),
                    (u.orders.iter().map(|o| o.kind.clone()).collect(), u.idle),
                );
            }
        }
        out
    };
    let (ka, kb) = (kinds(&a), kinds(&b));
    let born = |k: &Kinds, o: i64| k.keys().find(|(_, x)| *x == o).map(|(n, _)| *n);
    for (o, n) in [(6, 611), (7, 616), (8, 621), (9, 626), (10, 631)] {
        assert_eq!(born(&ka, o), Some(n), "run141's 0/{o} is born on {n}");
        assert_eq!(born(&kb, o), Some(n), "run142's 0/{o} is born on {n}");
    }
    let only = |k: &Kinds| -> Vec<(i64, i64, String)> {
        k.iter()
            .flat_map(|(&(n, o), (kinds, _))| kinds.iter().map(move |k| (n, o, k.clone())))
            .collect()
    };
    let (oa, ob) = (only(&ka), only(&kb));
    let first = |v: &[(i64, i64, String)]| v.first().cloned();
    eprintln!(
        "chapter seven: {} ordered unit-blocks in run141, {} in run142; first {:?}",
        oa.len(),
        ob.len(),
        first(&oa)
    );
    // Only the citizen ever holds an order: its gather, and the gather's
    // own transit legs (a `MOVEORDER` pushed over it, 764 onward).
    let gathering = |k: &str| k == "GATHERORDER" || k == "MOVEORDER";
    for (run, v) in [("run141", &oa), ("run142", &ob)] {
        assert!(
            v.iter().all(|(_, o, k)| *o == 6 && gathering(k)),
            "{run}: an order on a civilian other than the citizen's gather: {:?}",
            v.iter().find(|(_, o, k)| *o != 6 || !gathering(k))
        );
        assert_eq!(
            first(v).map(|(n, o, _)| (n, o)),
            Some((763, 6)),
            "{run}: the citizen's first order"
        );
    }
    assert_eq!(
        ka.get(&(763, 6)).and_then(|x| x.1),
        Some(12),
        "the human's idle wait"
    );
    assert_eq!(
        ka, kb,
        "the five's orders differ between the AI-off chapter and its control"
    );
}

/// **Chapter seven, walked** — the civilians (`docs/GOLDEN.md` §11, item
/// 578, run141). Seven staged lines: `!ai off`, the library, and the five
/// `add`s. The premise the chapter was designed on did not survive
/// (`docs/INPUT.md` §11.9); what it measures is a human's idle civilians,
/// and the word is the trace's end.
#[test]
fn chapter_seven_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch7", 7, 7, 1200) else {
        return;
    };
    assert_eq!(
        w.word, GOLDEN_WORD_CHAPTER_SEVEN,
        "chapter seven's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §11"
    );
    assert_eq!(
        (w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_SEVEN, None),
        "chapter seven's label order or value word parts under its end"
    );
}

/// **Chapter seven's control, walked to the same end** (item 578, run142:
/// `chapter7_control.cmd`, six staged lines, the Leader AI on). The
/// control's claim — that the same five act exactly as with the cheat on —
/// is on file only if this crate agrees with it as far as with the
/// chapter, so it is pinned to the same word.
#[test]
fn chapter_seven_s_control_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7c", "chapter7_control", 7, 6, 1200) else {
        return;
    };
    assert_eq!(
        (w.word, w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_SEVEN, GOLDEN_WORD_CHAPTER_SEVEN, None),
        "chapter seven's control parts under its end"
    );
}

/// **Chapter seven-b's pair is one game until `ai off` is first read**
/// (item 628): run156 is `chapter7b.cmd` and run157
/// `chapter7b_control.cmd`, the same file less `0 !ai off`. As chapter
/// seven's pair, the draw streams agree through frame 0 and part on
/// frame 1, where `Leader::production_ai` first reads the flag.
#[test]
fn chapter_seven_b_s_pair_is_one_game_until_the_gate() {
    let (Some((_, a)), Some((_, b))) = (golden("ch7b"), golden("ch7bc")) else {
        eprintln!("skipping: chapter seven-b needs both run156 and run157 (docs/RUNS.md)");
        return;
    };
    let read = |p: &str| {
        crate::trace::Trace::read(std::path::Path::new(p))
            .expect("a finalized golden trace")
            .expect("missing RONT header")
    };
    let (ta, tb) = (read(&a), read(&b));
    let parted = (0..=1200).find(|&f| ta.labels(f) != tb.labels(f));
    let agreed: usize = (0..parted.unwrap_or(1201))
        .map(|f| ta.labels(f).len())
        .sum();
    assert_eq!(
        parted,
        Some(1),
        "the pair parts where `ai off` is first read"
    );
    assert_eq!(agreed, 120, "frame 0's draws, identical in the pair");
    eprintln!(
        "chapter seven-b: frame 1 is {} draws in run156 and {} in run157",
        ta.labels(1).len(),
        tb.labels(1).len()
    );
}

/// **What `ai off` takes from a computer's civilians in `Unit::think`:
/// nothing, and it could not** (item 628, run156 and run157;
/// `docs/INPUT.md` §11.10). Every one of the five carries `unit_masks &
/// 0x40000` from its birth — `Unit::init@00612100:585` sets it for a leader
/// whose `flags & 0xc` is not 4, and who=1's is `0x13` — so the cheat's
/// block in `Unit::think` (`:206`) is entered and its one exit (`:264`)
/// never taken. The citizen takes its `GATHERORDER` on its birth block
/// under `!ai off`, which is `docs/GOLDEN.md` §11's first falsifier; the
/// control's citizen takes the same order on the same block, which is the
/// second's not firing.
///
/// The five are found by their birth blocks, never by slot: the AI trains
/// three citizens in the control before 605, so they are `1/9..1/13` there
/// and `1/6..1/10` in run156.
#[test]
fn chapter_seven_b_s_civilians_act_alike_with_the_ai_off_and_on() {
    use std::collections::BTreeMap;
    let (Some((a, _)), Some((b, _))) = (golden("ch7b"), golden("ch7bc")) else {
        eprintln!("skipping: chapter seven-b needs both run156 and run157 (docs/RUNS.md)");
        return;
    };
    // birth block → (guy type, masks at birth, first block of each order kind)
    type Five = BTreeMap<i64, (Option<i64>, i64, BTreeMap<String, i64>)>;
    let five = |p: &str| -> Five {
        let mut ix = crate::capture::indexed::IndexedCapture::open(p).unwrap();
        let mut born: BTreeMap<i64, i64> = BTreeMap::new();
        let mut out = Five::new();
        for at in 0..ix.frames().len() {
            let frame = ix.frame_state(at).unwrap();
            if frame.n < 605 {
                continue;
            }
            for u in frame.units.iter().filter(|u| u.who == 1) {
                let birth = *born.entry(u.o).or_insert(frame.n);
                if ![611, 616, 621, 626, 631].contains(&birth) {
                    continue;
                }
                let e = out.entry(birth).or_insert_with(|| {
                    (
                        u.guys.first().and_then(|g| g.kind),
                        u.unit_masks.unwrap_or(0),
                        BTreeMap::new(),
                    )
                });
                for o in &u.orders {
                    e.2.entry(o.kind.clone()).or_insert(frame.n);
                }
            }
        }
        out
    };
    let (fa, fb) = (five(&a), five(&b));
    for (run, f) in [("run156", &fa), ("run157", &fb)] {
        let types: Vec<(i64, Option<i64>)> = f.iter().map(|(n, e)| (*n, e.0)).collect();
        assert_eq!(
            types,
            [
                (611, Some(50)),
                (616, Some(59)),
                (621, Some(61)),
                (626, Some(52)),
                (631, Some(400))
            ],
            "{run}: the five's births and types"
        );
        assert!(
            f.values().all(|e| e.1 & 0x40000 != 0),
            "{run}: a civilian of who=1 without `unit_masks & 0x40000`"
        );
        let firsts = |n: i64| -> Vec<(String, i64)> {
            f[&n].2.iter().map(|(k, v)| (k.clone(), *v)).collect()
        };
        assert_eq!(
            firsts(611).first(),
            Some(&("GATHERORDER".to_string(), 611)),
            "{run}: the citizen gathers on its birth block"
        );
        assert!(firsts(616).is_empty(), "{run}: the caravan takes an order");
        assert!(firsts(626).is_empty(), "{run}: the scholar takes an order");
        assert_eq!(
            firsts(631),
            [
                ("CASTORDER".to_string(), 1151),
                ("MOVEORDER".to_string(), 631)
            ],
            "{run}: the fur trapper's orders"
        );
    }
    // The merchant is the one of the five whose orders differ in time:
    // it casts on 900 under `!ai off` and on 887 without it.
    let cast = |f: &Five| f[&621].2.get("CASTORDER").copied();
    assert_eq!((cast(&fa), cast(&fb)), (Some(900), Some(887)));
}

/// **Chapter seven-b, walked** — the computer's civilians under the cheat
/// (`docs/GOLDEN.md` §11, item 628, run156). Seven staged lines: `!ai off`,
/// `library who=1 2`, and the five `add`s for who=1 beside London.
#[test]
fn chapter_seven_b_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7b", "chapter7b", 7, 7, 1200) else {
        return;
    };
    // Closed at the trace's end since item 629: no value parts either
    // (`GOLDEN_WORD_CHAPTER_SEVEN_B`).
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_SEVEN_B,
            GOLDEN_WORD_CHAPTER_SEVEN_B,
            None
        ),
        "chapter seven-b's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §11"
    );
}

/// **Chapter seven-b's control, walked to the same end** (item 628,
/// run157: `chapter7b_control.cmd`, six staged lines, the Leader AI on).
#[test]
fn chapter_seven_b_s_control_holds_to_the_golden_word() {
    let Some(w) = walk_script("ch7bc", "chapter7b_control", 7, 6, 1200) else {
        return;
    };
    // Closed at the trace's end since item 647: no value parts either
    // (`GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL`).
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
            GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL,
            None
        ),
        "chapter seven-b's control's word moved"
    );
}

/// One `GOOD` record of a frame block, in the block's order — the order
/// the start block's list, and so this crate's `World::goods`, has
/// (`diff::setup`). `Good::log_data@0066e610` writes the type's name as a
/// bare line, `ever_seen`, then the `SubObject` base.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct FrameGood {
    name: String,
    ever_seen: i64,
    flags: i64,
    o: i64,
    x: i64,
    y: i64,
}

fn frame_goods(raw: &str) -> Vec<FrameGood> {
    let mut out: Vec<FrameGood> = Vec::new();
    let mut open = false;
    for line in raw.lines() {
        let s = line.trim();
        if s == "BEGIN GOOD" {
            out.push(FrameGood::default());
            open = true;
            continue;
        }
        if !open {
            continue;
        }
        if s.starts_with("BEGIN ") && s != "BEGIN SUBOBJECT" {
            open = false;
            continue;
        }
        let g = out.last_mut().expect("an open GOOD");
        let mut kv = s.split_whitespace();
        match (kv.next(), kv.next()) {
            (Some(k), None) if g.name.is_empty() && !k.is_empty() => g.name = k.to_string(),
            (Some("ever_seen"), Some(v)) => g.ever_seen = v.parse().unwrap(),
            (Some("flags"), Some(v)) => g.flags = v.parse().unwrap(),
            (Some("o"), Some(v)) => g.o = v.parse().unwrap(),
            (Some("x_internal"), Some(v)) => g.x = v.parse().unwrap(),
            (Some("y_internal"), Some(v)) => g.y = v.parse().unwrap(),
            _ => {}
        }
    }
    out
}

/// **One golden capture of the civilians' chapters, widened whole**
/// (items 578 and 628): every record on every block of `[first, last)` —
/// [`crate::diff::harness::widen_block`] on every unit, figure, building
/// and city, the leader record for both players at `LEADERS=2`, and the
/// per-frame `GOOD` list keyed on `o`. The civilians of `who` (`o >= 6`)
/// are printed both sides on the blocks of `print`. Returns each parted
/// key's first block and row; `None` when the capture is not on disk.
///
/// `ammo` adds the **`AMMO` record, both directions**, for a capture that
/// dumps it (chapter six, item 648): a live round (`flags & 2`) keyed on
/// its shooter and pool slot, as chapter five's widening keys it, and a
/// round either side holds alone is a row. The civilians' captures do not
/// dump `AMMO`, so for them it is off and a round of this crate's would
/// be compared against nothing.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn widen_civilians(
    run: &str,
    stem: &str,
    (first, last): (i64, i64),
    no_block: i64,
    who: i64,
    print: (i64, i64),
    ammo: bool,
) -> Option<std::collections::BTreeMap<(i64, i64, String), (i64, String)>> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut s = stage_script(run, stem)?;
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows, mut good_rows) = (0usize, 0, 0, 0);
    let mut unread_goods = 0usize;
    let mut ammo_rows = 0usize;
    let mut no_goods = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        crate::diff::harness::debug_leader(&s.built, n);
        crate::diff::harness::debug_builds(&s.built, n);
        crate::diff::harness::debug_watch(&s.built, n);
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut firsts);
        rows += k;
        let raw = s.ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&s.loaded, &s.built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    firsts
                        .entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **The `GOOD` list, both directions**, keyed on the good's own
        // `o`, which is its index in this crate's `World::goods`
        // (`diff::setup` installs the start block's list in order).
        // **The dump cannot print `o 0`'s header**: every block writes
        // the first good's `SubObject` fields straight after the last
        // `GUY` of the list before it, with no `BEGIN GOOD`, no name and
        // no `ever_seen` — run141's line 583513, and the same on every
        // frame. So `o 0` is counted as unreadable, not compared.
        //
        // A capture taken without `GOODS` in its `end:` set (chapter six)
        // prints no list at all, and a block of that kind is counted, not
        // compared: this crate's goods would be rows against nothing.
        let theirs: BTreeMap<i64, FrameGood> =
            frame_goods(&raw).into_iter().map(|g| (g.o, g)).collect();
        let ours = s.built.sim.world.goods();
        if theirs.is_empty() {
            no_goods += 1;
        }
        for i in 0..ours
            .len()
            .max(theirs.keys().max().map_or(0, |&o| o as usize + 1))
            * usize::from(!theirs.is_empty())
        {
            let (a, g) = match (theirs.get(&(i as i64)), ours.get(i)) {
                (Some(a), Some(g)) => (a, g),
                (None, Some(_)) if i == 0 => {
                    unread_goods += 1;
                    continue;
                }
                (a, _) => {
                    let side = if a.is_some() {
                        "the dump"
                    } else {
                        "this crate"
                    };
                    firsts
                        .entry((255, i as i64, "good".to_string()))
                        .or_insert((n, format!("{side} holds it alone")));
                    continue;
                }
            };
            let name = s
                .loaded
                .good_tree
                .iter()
                .position(|&t| t == g.ty)
                .map_or("?", |r| s.loaded.good_names[r].as_str());
            let seen = (0..2u8)
                .filter(|&w| s.built.sim.good_ever_seen(g.pos, w))
                .fold(0i64, |m, w| m | (1 << w));
            for (what, mine, dumped) in [
                ("x", i64::from(g.pos.x), a.x),
                ("y", i64::from(g.pos.y), a.y),
                ("alive", i64::from(g.alive), a.flags & 1),
                ("ever_seen", seen, a.ever_seen),
                ("type", i64::from(name == a.name), 1),
            ] {
                good_rows += 1;
                if mine != dumped {
                    firsts
                        .entry((255, i as i64, format!("good:{what}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped} ({})", a.name)));
                }
            }
        }
        if ammo {
            let theirs: BTreeSet<(i64, i64, i64)> = super::ammo::blocks(&raw)
                .into_iter()
                .filter(|(a, _)| a.flags & 2 != 0)
                .map(|(a, _)| (a.who, a.o, a.index))
                .collect();
            let ours: BTreeSet<(i64, i64, i64)> = s
                .built
                .sim
                .projectiles
                .iter()
                .filter_map(|p| match p.shooter {
                    sim::combat::Obj::Unit(u) => {
                        let un = &s.built.sim.units[u];
                        Some((i64::from(un.owner), i64::from(un.index), i64::from(p.slot)))
                    }
                    sim::combat::Obj::Building(_) => None,
                })
                .collect();
            ammo_rows += theirs.len().max(ours.len());
            for &(w, o, slot) in theirs.symmetric_difference(&ours) {
                let side = if theirs.contains(&(w, o, slot)) {
                    "the dump"
                } else {
                    "this crate"
                };
                firsts
                    .entry((w, o, format!("ammo[{slot}]")))
                    .or_insert((n, format!("{side} holds it alone")));
            }
        }
        // **Both sides printed once** on the citizen's two blocks, for
        // the five: the record a draw would be spent in, before any
        // quiet row is trusted.
        if (print.0..=print.1).contains(&n) {
            for them in frame.units.iter().filter(|u| u.who == who && u.o >= 6) {
                let mine = i16::try_from(them.o)
                    .ok()
                    .and_then(|o| s.built.sim.unit_by_o(who as u8, o))
                    .map(|u| &s.built.sim.units[u]);
                eprintln!("  {run} block {n} {who}/{} theirs {them:?}", them.o);
                match mine {
                    Some(u) => eprintln!(
                        "  {run} block {n} {who}/{} ours pos {:?} idle {} orders {:?} guys {:?}",
                        them.o, u.pos, u.idle, u.orders, u.guys
                    ),
                    None => eprintln!("  {run} block {n} {who}/{} ours: absent", them.o),
                }
            }
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if rows.len() <= 12 {
            for r in rows {
                eprintln!("  {run} f{f} {r}");
            }
        } else {
            eprintln!("  {run} f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "widening {run}: {blocks} blocks [{first}, {last}), {rows} record rows, \
         {leader_rows} leader rows, {good_rows} good rows ({unread_goods} unreadable), \
         {ammo_rows} rounds, {} keys parted; {} leader keys not printed at LEADERS=2",
        firsts.len(),
        missing.len()
    );
    let absent = usize::from((first..last).contains(&no_block));
    assert_eq!(
        blocks,
        (last - first) as usize - absent,
        "{run}'s dump no longer carries every frame of [{first}, {last})"
    );
    assert_eq!(
        leader_rows,
        2 * 88 * blocks,
        "{run}: the leader rows LEADERS=2 prints are not compared on every block"
    );
    assert!(
        good_rows > 0 || no_goods == blocks,
        "{run}: the GOOD list is printed and not read"
    );
    Some(firsts)
}

/// **Chapter seven's word, widened whole, both directions, on both
/// captures** (item 578, `docs/DECISIONS.md` 43). Every record run141 and
/// run142 carry on every block of [`WIDENING_CHAPTER_SEVEN`]:
/// [`crate::diff::harness::widen_block`] on every unit, figure, building
/// and city; the leader record whole for both players at `LEADERS=2`,
/// whose goods block — `resources`, `income`, `filled_gather_slots` — is
/// what says the citizen's gather delivered (the per-frame `GOOD` list is
/// the map's rares, oil and fish, not a stockpile); and that **`GOOD`
/// list**, which no other widening reads: each good's type, position,
/// `flags & 1` and `ever_seen` bits against this crate's
/// [`sim::world::Good`] and `good_ever_seen` for both players.
///
/// The five civilians are printed once, both sides, on 763 and 764, the
/// blocks the citizen's gather is issued and walked on.
///
/// Made to fail once with player 1's `ever_seen` bit dropped from this
/// crate's side: the `GOOD` rows part on the first block and the test
/// refuses. Its first run found its own instrument wrong, too — the dump
/// never prints `o 0`'s header, so an index-keyed comparison was one good
/// out on every block; it is keyed on `o` now.
#[test]
fn chapter_seven_s_word_frame_is_widened_whole() {
    const FIRST: i64 = WIDENING_CHAPTER_SEVEN.0;
    const LAST: i64 = WIDENING_CHAPTER_SEVEN.1;
    /// The window is 605..1199 and the `!quit` block is 1201.
    const NO_BLOCK: i64 = 1200;
    let mut summaries = Vec::new();
    for (run, stem) in [("ch7", "chapter7"), ("ch7c", "chapter7_control")] {
        let Some(firsts) =
            widen_civilians(run, stem, (FIRST, LAST), NO_BLOCK, 0, (763, 764), false)
        else {
            return;
        };
        summaries.push((run, firsts));
    }
    // **The standing rows of each capture's first block**, none of them
    // this chapter's: the unmodelled `form` on every unit, two
    // `filled_gather_slots` (chapter four's and five's families), and the
    // two capitals' `CITIES` rows, which chapter four's bleed window did
    // not print. The control adds its computer's scout `1/0`, whose group
    // and explore-order `facing` part from the first block there (parked
    // 275's mirror), and three more `form`s. Until item 644 two leader
    // `bucket`s stood here too, who=0's knowledge and metal at 0 against
    // 100: `library who=0 2` skipped `gain_tech`'s tail, and the Classical
    // age's starting grant is in it (29 → 27, 34 → 32).
    let standing = |what: &str| {
        what == "form"
            || what.starts_with("leader:filled_gather_slots")
            || what.starts_with("city:")
    };
    let control_standing = |what: &str| what == "group" || what == "order:move.facing";
    for ((run, firsts), (floor, under_want)) in summaries
        .iter()
        .zip([(27usize, &[847i64][..]), (32usize, &[][..])])
    {
        let at_floor: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == FIRST)
            .map(|((_, _, what), _)| what)
            .collect();
        assert!(
            at_floor
                .iter()
                .all(|w| standing(w) || (*run == "ch7c" && control_standing(w)))
                && at_floor.len() == floor,
            "{run}: the standing rows on the first block moved ({}): {at_floor:?}",
            at_floor.len()
        );
        // **What parts under the word**, none of it a draw, pinned by key
        // and block. The **Nubian hit points**: the caravan `0/7`, the
        // merchant `0/8` and the fur trapper `0/10` are born with 135, 135
        // and 180 in the original against this crate's 90, 90 and 120 —
        // `NUBIAN_HIT_POINTS`, "50% more" on merchants, caravans and
        // markets (`rules.xml`), which `Unit::update_hits@0060e930` reads
        // and this crate does not apply. Player 0 is Nubia (its capital is
        // Napata). The citizen's and the scholar's agree. And run141 alone
        // carries the scout `1/0`'s `facing` from 847, as chapter five's
        // does; in run142 it stands from the first block.
        let mut got: Vec<String> = firsts
            .iter()
            .filter(|((_, _, what), (f, _))| *f > FIRST && !standing(what))
            .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
            .collect();
        let mut want: Vec<String> = [(616, 7), (621, 8), (631, 10)]
            .iter()
            .flat_map(|(f, o)| {
                ["hits:myhits", "hits_left", "myhits"]
                    .iter()
                    .map(move |k| format!("{f} 0/{o} {k}"))
            })
            .chain(
                under_want
                    .iter()
                    .map(|f| format!("{f} 1/0 order:move.facing")),
            )
            .collect();
        got.sort();
        want.sort();
        assert_eq!(got, want, "{run}: what parts under the word moved");
        // **The `GOOD` list agrees whole** on every block, `ever_seen`
        // included, bar the one good the dump cannot print.
        assert!(
            !firsts
                .keys()
                .any(|(w, _, what)| *w == 255 || what.starts_with("good")),
            "{run}: a good parts"
        );
    }
}

/// **Chapter six's word, widened whole, both directions** (items 648, 650
/// and 652; since 652, run168 whole). Every
/// record run168 carries on every block of [`WIDENING_CHAPTER_SIX`], by
/// [`widen_civilians`] with the `AMMO` record on: every unit and figure,
/// both leaders at `LEADERS=2`, and every live round either side holds.
/// Every record is printed both sides on the word's two blocks. run168
/// dumps no `BUILDS` and no `GOODS`: the buildings stand as `build:extra`
/// rows on the first block, and the good list is counted, not compared.
///
/// The `AMMO` arm was made to fail once, on run127's 139 rounds with this
/// crate's pool slot shifted by one: eight `ammo[·]` rows parted.
#[test]
fn chapter_six_s_word_frame_is_widened_whole() {
    let Some(firsts) = widen_civilians(
        "ch6",
        "chapter6",
        WIDENING_CHAPTER_SIX,
        900,
        1,
        (GOLDEN_WORD_CHAPTER_SIX, GOLDEN_WORD_CHAPTER_SIX + 1),
        true,
    ) else {
        return;
    };
    for ((w, o, what), (f, row)) in &firsts {
        eprintln!("  ch6 f{f} {w}/{o} {what}: {row}");
    }
    // **The standing rows of the first block**: the unmodelled `form`
    // on all ten start units, two `filled_gather_slots`, and the fourteen
    // start buildings as `build:extra`, which run168 cannot print.
    let standing = |what: &str| {
        what == "form" || what.starts_with("leader:filled_gather_slots") || what == "build:extra"
    };
    let floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_SIX.0)
        .map(|((_, _, what), _)| what)
        .collect();
    assert!(
        floor.iter().all(|w| standing(w)) && floor.len() == 26,
        "ch6: the standing rows on the first block moved ({}): {floor:?}",
        floor.len()
    );
    // **What parts under the word**, pinned by block and key (`docs/GOLDEN.md`
    // §10). Each aircraft's `form` on its birth block, the standing family,
    // and nothing else on any block of run168. **Item 652 staged `bird`**
    // at the channel's measured cursor, (0, 6), and the 701 rows went: the
    // AI scout `1/0`'s move order and path, which parted because
    // `think_scout`'s roll came one draw early without the bird's birth
    // draw. The window is run168 whole now, 605 to its last block, 899,
    // and no round is in either air.
    let mut got: Vec<String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f > WIDENING_CHAPTER_SIX.0)
        .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
        .collect();
    got.sort();
    let want = vec!["611 0/6 form".to_string(), "616 1/6 form".to_string()];
    assert_eq!(got, want, "ch6: what parts under the word moved");
}

/// **Chapter seven-b's words, widened whole, both directions, on both
/// captures** (item 628). Every record run156 and run157 carry on every
/// block of [`WIDENING_CHAPTER_SEVEN_B`], by [`widen_civilians`].
#[test]
fn chapter_seven_b_s_word_frame_is_widened_whole() {
    for (run, stem, window, print) in [
        ("ch7b", "chapter7b", WIDENING_CHAPTER_SEVEN_B, (1199, 1200)),
        (
            "ch7bc",
            "chapter7b_control",
            WIDENING_CHAPTER_SEVEN_B_CONTROL,
            (1199, 1200),
        ),
    ] {
        let Some(firsts) = widen_civilians(run, stem, window, 1200, 1, print, false) else {
            return;
        };
        for ((w, o, what), (f, row)) in &firsts {
            eprintln!("  {run} f{f} {w}/{o} {what}: {row}");
        }
        // **The standing rows of the first block**, chapter seven's
        // families: the unmodelled `form`, two `filled_gather_slots` and
        // the two capitals' `CITIES` rows. The control adds its scout
        // `1/0`'s `group` and explore-order `facing`, as chapter seven's
        // control does. **Item 644 took seven** (34 → 27, 39 → 32): who=1's
        // knowledge and metal `bucket`s, 0 against 100, and its five
        // `resource_cap`s, 1392 against 2992. `library who=1 2` now raises
        // the levels through `gain_tech`'s tail, which pays the Classical
        // age's starting grant, and the caps read the Commerce level live,
        // as `calc_resource_caps` does every frame (parked 633).
        let standing = |what: &str| {
            what == "form"
                || what.starts_with("leader:filled_gather_slots")
                || what.starts_with("city:")
        };
        let control = |what: &str| what == "group" || what == "order:move.facing";
        let at_floor: Vec<&String> = firsts
            .iter()
            .filter(|(_, (f, _))| *f == window.0)
            .map(|((_, _, what), _)| what)
            .collect();
        let floor = if run == "ch7b" { 27 } else { 32 };
        assert!(
            at_floor
                .iter()
                .all(|w| standing(w) || (run == "ch7bc" && control(w)))
                && at_floor.len() == floor,
            "{run}: the standing rows on the first block moved ({}): {at_floor:?}",
            at_floor.len()
        );
        // **What parts under each word**, pinned by block and key, none of
        // it a mechanism (`docs/GOLDEN.md` §11). run156: the scout `1/0`'s
        // `facing` from 847, as in chapter seven (parked 635). Until item
        // 629 the merchant `1/8` stood 24 units off the original's when its
        // cast ended on 1070, and the fur trapper `1/10`'s move was handed
        // another `dest_y` on 1091, 14408 against 14804, whose turn was the
        // word 1148: 22 rows, gone with `cast_unpack`'s merchant arm (the
        // move's value diff; its delta is the constant's). run157, **the
        // control's block, item 632** (the delta is in
        // `GOLDEN_WORD_CHAPTER_SEVEN_B_CONTROL`'s comment): `1/1` keeps its
        // `BUILDORDER` behind the goody box's walk from 990, and only the
        // group id stands there, 64 against 65, the two-slot permutation
        // the scout's standing `group` row began (`docs/GROUPS.md` §24).
        // Past it the scout `1/0`'s explore path parts from 1077, value
        // only. **Item 644's block** (the delta is in the constant's
        // comment): who=1's food bucket from 1018 and `1/7`'s thirteen
        // rows on 1177 are gone. The food was a goody pile paid to metal
        // here, and `1/7` is now sent toward the Woodcutter's Camp the
        // script places and destroys on 1176 and keeps its walk and
        // `BUILDORDER` to the word, as the original's does: on 1177 both
        // sides hold explore-to (40248, 23160) and the `BUILDORDER` on
        // `1/2009`, at (40271, 18360). Nothing new parts on 1177..1188.
        // **Item 647's block** (the delta is in the constant's comment):
        // the scout `1/0`'s eighteen rows are gone — its path from 1077
        // (47 nodes against 48, `path[41..46].to`), its position, figures
        // and `dest_y` from 1116, and its `dest`/`dest_x` from 1137. The
        // path was built across the Small City `1/2007`'s footprint, which
        // this crate's who=1 had not seen: `Wall::start` now writes the
        // owner's bit over it (`docs/SCOUT.md` §14). The window is the
        // capture whole, and only the group id on 990 stands.
        let mut got: Vec<String> = firsts
            .iter()
            .filter(|((_, _, what), (f, _))| *f > window.0 && what != "form")
            .map(|((w, o, what), (f, _))| format!("{f} {w}/{o} {what}"))
            .collect();
        got.sort();
        let want: &[&str] = if run == "ch7b" {
            &["847 1/0 order:move.facing"]
        } else {
            &["990 1/1 group"]
        };
        assert_eq!(got, want, "{run}: what parts under the word moved");
        assert!(
            !firsts
                .keys()
                .any(|(w, _, what)| *w == 255 || what.starts_with("good")),
            "{run}: a good parts"
        );
    }
}

/// **Chapter three, walked** — the mounted and siege lines (`docs/GOLDEN.md`
/// §7, item 587, run145). Six staged lines: `!ai off`, `age who=N 2` for
/// both players (the Classical age), `add 3 chariot`, the hoplite squad and
/// `add catapult`.
///
/// **What the capture established before this walk ran**, each written
/// into `chapter3.cmd`'s header first (`docs/RUNS.md`, run145): eight `INFO
/// cmd` records each returning 1; 297 blocks, 605..899 with no gap; three
/// separate one-unit Chariots `0/6..0/8` on 611 — §7's leading-count
/// falsifier does not fire — the hoplites `1/6..1/8` on 616 and the
/// catapult `0/9` on 621. The other two falsifiers **could not fire**: the
/// catapult is born packed and never launches (its unpack, `spell 652`,
/// starts on 696), and the chariots shoot the hoplites dead without moving.
///
/// The word was the catapult's, 621, until item 590: this crate put the
/// packed `0/9` into `Unit::fight` the frame after its birth. With
/// `Unit::think_attack`'s packed arm (`docs/COMBAT.md` §51) the catapult
/// holds no order until its unpack on 696, as the dump's does, and the
/// word is **633**: the chariot `0/8`'s first attack frame, where the
/// original spends one `Unit::fight+0x9b0` and two `Guy::set_anim+0xf2f`
/// and this crate a second re-search — run146's word, frame and shape.
#[test]
fn chapter_three_holds_to_the_golden_word() {
    let Some(w) = walk_chapter("ch3", 3, 6, 900) else {
        return;
    };
    assert_eq!(
        (w.word, w.sequence, w.value),
        (GOLDEN_WORD_CHAPTER_THREE, GOLDEN_WORD_CHAPTER_THREE, None),
        "chapter three's golden word moved; re-pin it here and say so in \
         docs/GOLDEN.md §7"
    );
}

/// **Chapter three's word, widened whole, both directions** (item 587,
/// `docs/DECISIONS.md` 43). Every record run145 carries on every block of
/// [`WIDENING_CHAPTER_THREE`], the whole capture:
/// [`crate::diff::harness::widen_block`] on every unit, figure, building
/// and city; the leader record whole for both players at `LEADERS=2`; the
/// **`AMMO` record** shot by shot, both directions, as chapter five's
/// widening reads it; and the **`GUY` record** at `GUYS=2` — each figure's
/// position and facing — which `compare` does not carry.
///
/// The word's two blocks print the word's unit once, both sides, before
/// any quiet row is trusted: the chariot `0/8` on 633 and 634 since item
/// 590, the catapult `0/9` on 621 and 622 before it.
#[allow(
    non_snake_case,
    reason = "the window's names as the other widenings spell them"
)]
fn widen_chapter_three(
    run: &str,
    stem: &str,
    (first, last): (i64, i64),
    word: i64,
    catapult: i64,
    whole: &[i64],
) -> Option<ChapterThreeWidening> {
    use std::collections::{BTreeMap, BTreeSet};
    let (FIRST, LAST, WORD) = (first, last, word);
    // The window's one absent block: the capture's last frame, whose
    // `!quit` block is written one past it.
    let no_block = LAST - 1;
    let mut s = stage_script(run, stem)?;
    let players = s.built.sim.players.len();
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut at_word: Vec<String> = Vec::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let (mut blocks, mut rows, mut leader_rows, mut guy_rows) = (0usize, 0usize, 0usize, 0usize);
    let (mut ammo_theirs, mut ammo_ours) = (0usize, 0usize);
    let mut attack_rows = 0usize;
    for f in 0..LAST - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < FIRST {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        blocks += 1;
        // Every row this block parts on, first or not: merged into
        // `firsts` at the block's end, and kept whole on the word's two
        // blocks (item 603), where a key that parted earlier — the
        // release point `ammo[k].sx`, first on 651 — is still the frame's.
        let mut blk: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
        let (_, k) = crate::diff::harness::widen_block(&s.built, &frame, players, n, &mut blk);
        rows += k;
        let raw = s.ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        for who in 0..2usize {
            let Some(block) = flog.leader_block(n, who as i64) else {
                continue;
            };
            let t = crate::diff::leader::theirs(&block);
            for (k, v) in crate::diff::leader::rows(&s.loaded, &s.built, who) {
                let Some(&y) = t.get(&k) else {
                    missing.insert(k);
                    continue;
                };
                leader_rows += 1;
                if v != y {
                    blk.entry((who as i64, -1, format!("leader:{k}")))
                        .or_insert((n, format!("ours {v} theirs {y}")));
                }
            }
        }
        // **The `AMMO` record, both directions**, keyed on the shooter and
        // the pool slot (chapter five's reading, `docs/COMBAT.md` §46.4).
        let theirs: BTreeMap<(i64, i64, i64), super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0)
            .map(|(a, _)| ((a.who, a.o, a.index), a))
            .collect();
        let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = s
            .built
            .sim
            .projectiles
            .iter()
            .filter_map(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => {
                    let un = &s.built.sim.units[u];
                    Some((
                        (i64::from(un.owner), i64::from(un.index), i64::from(p.slot)),
                        *p,
                    ))
                }
                sim::combat::Obj::Building(_) => None,
            })
            .collect();
        ammo_theirs += theirs.len();
        ammo_ours += ours.len();
        // **Both sides' rounds, printed on the blocks kept whole** (item
        // 602): the launch point, the flags, the target and the flight,
        // before any quiet row is trusted.
        if whole.contains(&n) {
            for (k, a) in &theirs {
                eprintln!("  {run} block {n} theirs {k:?} {a:?}");
            }
            for (k, p) in &ours {
                eprintln!("  {run} block {n} ours   {k:?} {p:?}");
                if let sim::combat::Obj::Unit(u) = p.shooter {
                    let un = &s.built.sim.units[u];
                    eprintln!(
                        "  {run} block {n} ours   {k:?} facing {:?} turret {:?}",
                        un.movement.facing, un.guys[0].turret
                    );
                }
            }
        }
        let keys: BTreeSet<_> = theirs.keys().chain(ours.keys()).copied().collect();
        for key @ (who, o, slot) in keys {
            let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                let side = if theirs.contains_key(&key) {
                    "the dump"
                } else {
                    "this crate"
                };
                blk.entry((who, o, format!("ammo[{slot}]")))
                    .or_insert((n, format!("{side} holds it alone")));
                continue;
            };
            let target = p.target.map_or((-1, -1), |t| match t {
                sim::combat::Obj::Unit(u) => {
                    let tu = &s.built.sim.units[u];
                    (i64::from(tu.owner), i64::from(tu.index))
                }
                sim::combat::Obj::Building(_) => (-2, -2),
            });
            for (name, mine, dumped) in [
                ("cur_time", i64::from(p.cur_time), a.cur_time),
                ("total_time", i64::from(p.total_time), a.total_time),
                ("sx", i64::from(p.launch.x), a.sx),
                ("sy", i64::from(p.launch.y), a.sy),
                ("ex", i64::from(p.landing.x), a.ex),
                ("ey", i64::from(p.landing.y), a.ey),
                ("whom", target.0, a.whom),
                ("ox", target.1, a.ox),
                ("accuracy", i64::from(p.accuracy), a.accuracy),
                ("splash_area", i64::from(p.splash_area), a.splash_area),
                ("num_guys", i64::from(p.num_guys), a.num_guys),
                ("sz", i64::from(p.sz), a.sz),
                ("ez", i64::from(p.ez), a.ez),
                ("angle", i64::from(p.angle.0), a.angle),
                ("v1z", super::ammo::tests::printed(p.v1z), a.v1z),
            ]
            .into_iter()
            .chain(ammo_flag_rows(p, a))
            {
                if mine != dumped {
                    blk.entry((who, o, format!("ammo[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
        // **The `GUY` record at `GUYS=2`**: each figure's position and
        // facing, on every staged unit and every other the block prints.
        // `compare` carries no guy row; chapter two's widening is where
        // this reading comes from.
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = s.built.sim.unit_by_o(who, o) else {
                continue;
            };
            let un = &s.built.sim.units[u];
            // **The unit's own `z`** (`ObjectData +0xc`, printed
            // `z_internal`), item 601: `get_damage`'s height step reads it
            // on both sides of every target search, and `compare` carries
            // x and y only. `docs/COMBAT.md` §46.2 swept it on chapter two;
            // this is the same reading on chapter three's every block.
            let z = i64::from(s.built.sim.world.tile_z(un.pos.tile()));
            if z != them.pos.z {
                blk.entry((them.who, them.o, "z".to_string()))
                    .or_insert((n, format!("ours {z} theirs {}", them.pos.z)));
            }
            if them.guys.len() != un.guys.len() {
                blk.entry((them.who, them.o, "guys:count".to_string()))
                    .or_insert((
                        n,
                        format!("ours {} theirs {}", un.guys.len(), them.guys.len()),
                    ));
            }
            for (k, g) in them.guys.iter().enumerate() {
                let Some(og) = un.guys.get(k).copied() else {
                    continue;
                };
                let (body, facing) = match og.follow {
                    Some(b) => (b.body, b.facing),
                    None => (un.movement.body, un.movement.facing),
                };
                // **What the figure is aimed at** (`GuyData +0x8e`/`+0x9f`,
                // printed `ox`/`whom`), item 595: `Unit::set_attack` writes
                // it for the unit's own `guy_mark` figures and not for its
                // crew, so a chariot's horse stays at `−1` while its archer
                // names the target. Position and facing alone read the two
                // figures as one.
                let aim = match og.aim {
                    None => (-1, -1),
                    Some(sim::combat::Obj::Unit(t)) => {
                        let tu = &s.built.sim.units[t];
                        (i64::from(tu.owner), i64::from(tu.index))
                    }
                    Some(sim::combat::Obj::Building(b)) => s.built.build_ids(b).unwrap_or((-2, -2)),
                };
                for (name, mine, dumped) in [
                    ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                    ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                    ("g.angle", i64::from(facing.0), g.angle),
                    ("g.whom", aim.0, g.whom),
                    ("g.ox", aim.1, g.ox),
                ] {
                    let Some(dumped) = dumped else { continue };
                    guy_rows += 1;
                    if mine != dumped {
                        blk.entry((them.who, them.o, format!("{name}[{k}]")))
                            .or_insert((n, format!("ours {mine} theirs {dumped}")));
                    }
                }
            }
            // **The `ATTACKORDER`'s own row** (item 595): `mandatory`,
            // `defensive`, `in_range`, `ever_in_range` and `new_ord`, on
            // the head order when both sides hold an attack there. The
            // order comparison reads the kind and the target and none of
            // these, so "in range on the same frame" was a quiet field
            // until this row (`docs/COMBAT.md` §44.2.1).
            // both sides: a head order on one side only, or of another
            // kind, is a real disagreement and is not quiet — it is
            // `widen_block`'s own `order:length`/`order:kind` row on this
            // block; this row reads only the fields past the kind.
            if let (Some(od), Some(front)) = (them.orders_front_first().next(), un.orders.front())
                && let sim::orders::Body::Attack(a) = front.body
                && od.index == i64::from(sim::orders::index::ATTACK)
            {
                for (name, mine, dumped) in [
                    ("mandatory", i64::from(un.combat.mandatory), od.mandatory),
                    ("defensive", i64::from(a.defensive), od.defensive),
                    ("in_range", i64::from(a.in_range), od.in_range),
                    (
                        "ever_in_range",
                        i64::from(a.ever_in_range),
                        od.ever_in_range,
                    ),
                    ("new_ord", i64::from(a.new_ord), od.new_ord),
                ] {
                    let Some(dumped) = dumped else { continue };
                    attack_rows += 1;
                    if mine != dumped {
                        blk.entry((them.who, them.o, format!("attack.{name}")))
                            .or_insert((n, format!("ours {mine} theirs {dumped}")));
                    }
                }
            }
        }
        // **Both sides printed once on the word's two blocks**, for the
        // catapult: the record the draw is spent in.
        if (WORD..=WORD + 1).contains(&n) {
            for them in frame.units.iter().filter(|u| u.who == 0 && u.o == catapult) {
                eprintln!("  {run} block {n} 0/{catapult} theirs {them:?}");
                let o = i16::try_from(catapult).expect("an o");
                match s.built.sim.unit_by_o(0, o).map(|u| &s.built.sim.units[u]) {
                    Some(u) => eprintln!(
                        "  {run} block {n} 0/{catapult} ours pos {:?} idle {} packed {} orders {:?} guys {:?}",
                        u.pos, u.idle, u.combat.packed, u.orders, u.guys
                    ),
                    None => eprintln!("  {run} block {n} 0/{catapult} ours: absent"),
                }
            }
        }
        if whole.contains(&n) {
            for ((w, o, what), (_, row)) in &blk {
                at_word.push(format!("{n} {w}/{o} {what}: {row}"));
            }
        }
        for (k, v) in blk {
            firsts.entry(k).or_insert(v);
        }
    }
    let mut by_block: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for ((w, o, what), (f, row)) in &firsts {
        by_block
            .entry(*f)
            .or_default()
            .push(format!("{w}/{o} {what}: {row}"));
    }
    for (f, rows) in &by_block {
        if *f <= WORD + 2 || rows.len() <= 6 {
            for r in rows {
                eprintln!("  f{f} {r}");
            }
        } else {
            eprintln!("  f{f}: {} keys, first {}", rows.len(), rows[0]);
        }
    }
    eprintln!(
        "ch3 widening {run}: {blocks} blocks [{FIRST}, {LAST}), {rows} record rows, \
         {leader_rows} leader rows, {guy_rows} guy rows, {attack_rows} attack rows, ammo {ammo_theirs} theirs / \
         {ammo_ours} ours, {} keys parted; {} leader keys not printed at LEADERS=2",
        firsts.len(),
        missing.len()
    );
    let absent = usize::from((FIRST..LAST).contains(&no_block));
    assert_eq!(
        blocks,
        (LAST - FIRST) as usize - absent,
        "{run}'s dump no longer carries every frame of [{FIRST}, {LAST})"
    );
    assert_eq!(
        leader_rows,
        2 * 88 * blocks,
        "the leader rows LEADERS=2 prints are not compared on every block"
    );
    assert!(guy_rows > 0, "the GUY record is not read");
    assert!(attack_rows > 0, "the ATTACKORDER row is not read");
    Some(ChapterThreeWidening {
        firsts,
        at_word,
        ammo_theirs,
        ammo_ours,
    })
}

/// What [`widen_chapter_three`] found: every key's first parting block
/// with the value diff beside it, and the `AMMO` tallies both sides.
struct ChapterThreeWidening {
    firsts: std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
    /// Every row the blocks kept whole part on, first or not, as
    /// `"block who/o key: ours … theirs …"`: the word's two (item 603), and
    /// the first rounds' launch blocks 651 and 652 (item 602).
    at_word: Vec<String>,
    ammo_theirs: usize,
    ammo_ours: usize,
}

/// **`AmmoData`'s flag byte and its roll byte, as the dump prints them**
/// (item 602). `+0x4 flags` carries `Ammo::init`'s bit 4 (the round rolls
/// on past a target that is gone, `docs/COMBAT.md` §42.2) and
/// `Ammo::inc_time`'s bit 8 (it has lost that target and flies on);
/// `+0x5 rolling` is a separate byte whose only writers are `Ammo::init`,
/// which zeroes it on every round (`0067bbf0:830`), and
/// `Ammo::init_crash`, which gives a falling aircraft a random roll. Until
/// this item the widenings compared this crate's bit 4 against the byte,
/// and every rolling round read "ours 1 theirs 0" there: a row that named
/// the wrong field, not a parting. This crate carries no roll byte and no
/// crash round, so its side of `rolling` is the zero `init` writes.
fn ammo_flag_rows(
    p: &sim::combat::Projectile,
    a: &super::ammo::Ammo,
) -> [(&'static str, i64, i64); 3] {
    [
        ("flags&4", if p.rolling { 4 } else { 0 }, a.flags & 4),
        ("flags&8", if p.missed { 8 } else { 0 }, a.flags & 8),
        ("rolling", 0, a.rolling),
    ]
}

/// **Chapter three's word, widened whole, both directions** (item 587,
/// `docs/DECISIONS.md` 43), on run145. The walk is [`widen_chapter_three`],
/// which run146 shares.
#[test]
fn chapter_three_s_word_frame_is_widened_whole() {
    const FIRST: i64 = WIDENING_CHAPTER_THREE.0;
    const WORD: i64 = GOLDEN_WORD_CHAPTER_THREE;
    let Some(w) = widen_chapter_three(
        "ch3",
        "chapter3",
        WIDENING_CHAPTER_THREE,
        WORD,
        8,
        &[651, 652, 753, 754, WORD, WORD + 1],
    ) else {
        return;
    };
    let (firsts, ammo_theirs, ammo_ours) = (w.firsts, w.ammo_theirs, w.ammo_ours);
    // **Anti-vacuity for the `AMMO` record**: run145's live rounds are all
    // read — the chariots' from 651 — and this crate's side is not empty.
    assert_eq!(ammo_theirs, 89, "run145's live rounds are not all read");
    assert!(ammo_ours > 0, "this crate fired no round in the window");
    // **The standing families on the capture's first block**, none of
    // them the chapter's: the unmodelled `form` (also every staged unit's
    // birth row), `build:extra` (the end detail prints no `BUILDDATA`)
    // and player 0's two `filled_gather_slots`. Until item 644 four more
    // stood here, `leader:bucket` 3 and 4 for both players, 0 against the
    // dump's 100: `age who=N 2` raised the ages without `gain_tech`'s
    // tail, and the Classical age's starting grant is in the tail
    // (`Sim::set_leader_levels`). 30 → 26.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor: Vec<&String> = firsts
        .iter()
        .filter(|(_, (f, _))| *f == FIRST)
        .map(|((_, _, what), _)| what)
        .collect();
    assert!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 26,
        "the standing rows on run145's first block moved: {at_floor:?}"
    );
    // **What parts first, and what parts on the word's own two blocks**
    // (item 601; the delta is in `GOLDEN_WORD_CHAPTER_THREE`'s comment).
    // 635 is quiet now: `0/6` takes `1/8`, as the dump's does, once the
    // flank reduction reads the target's mask. The first value parting is
    // the chariots' first rounds on 651, which leave from each unit's own
    // square and height where the dump's leave from the archer's release
    // node (`docs/COMBAT.md` §52.4, item 602's 651 on run146). `rolling`
    // parts with them. No key parts first on the word's blocks (item
    // 603): `0/8` holds its heading on 685 as the dump's does, its pivot
    // bearing from its node, and `1/4`'s move offsets, which followed
    // 684's draws, agree until 804.
    let rows_on = |lo: i64, hi: i64| -> Vec<String> {
        firsts
            .iter()
            .filter(|((_, _, what), (f, _))| (lo..=hi).contains(f) && !standing(what))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect()
    };
    let first = firsts
        .iter()
        .filter(|((_, _, what), _)| !standing(what))
        .map(|(_, (f, _))| *f)
        .min();
    let whole_on = |lo: i64, hi: i64| -> Vec<&String> {
        w.at_word
            .iter()
            .filter(|r| {
                r.split(' ')
                    .next()
                    .and_then(|b| b.parse::<i64>().ok())
                    .is_some_and(|b| (lo..=hi).contains(&b))
                    && !r
                        .split(": ")
                        .next()
                        .is_some_and(|k| standing(k.rsplit(' ').next().unwrap_or("")))
            })
            .collect()
    };
    // **Item 602: the word is the capture's end.** With the release
    // through the pivot turret (`docs/COMBAT.md` §55) the chariots' rounds
    // leave from the dump's point, and 651–652 agree on every record and
    // figure. What parts past the standing families, every row first or
    // not on the blocks kept whole, is three families of the `AMMO`
    // record, and no draw follows from any of them to 900:
    //
    // - a round's target, cleared here on its target's death (678, 705,
    //   757) where the original's `Ammo::check_hit` rewrites it on the
    //   round's due frame;
    // - the pool slot a round takes (703, 729, 730, 754), one apart;
    // - `0/8`'s launch point on 753, one unit in `sx` and `sy`, after its
    //   turn to 61.7° on 711: the round needs a turret step of 1°, where
    //   this crate's is 0° (`des` 0.06° from the node). Its 729 round,
    //   under the pool-slot row, is the same: `946, 8156` against
    //   `947, 8158`. `GUYS=2` prints no turret (§55.5).
    assert_eq!(first, Some(678), "run145's first value parting moved");
    assert_eq!(
        rows_on(FIRST + 1, WORD + 1),
        vec![
            "705 0/6 ammo[1].ox: ours -1 theirs 7",
            "705 0/6 ammo[1].whom: ours -1 theirs 1",
            "730 0/6 ammo[2]: this crate holds it alone",
            "678 0/6 ammo[2].ox: ours -1 theirs 8",
            "678 0/6 ammo[2].whom: ours -1 theirs 1",
            "730 0/6 ammo[5]: the dump holds it alone",
            "754 0/7 ammo[1]: this crate holds it alone",
            "703 0/7 ammo[2]: this crate holds it alone",
            "703 0/7 ammo[3]: the dump holds it alone",
            "754 0/7 ammo[4]: the dump holds it alone",
            "753 0/8 ammo[0].angle: ours 740622336 theirs 739377152",
            "757 0/8 ammo[0].ox: ours -1 theirs 6",
            "753 0/8 ammo[0].sx: ours 940 theirs 941",
            "753 0/8 ammo[0].sy: ours 8154 theirs 8155",
            "757 0/8 ammo[0].whom: ours -1 theirs 1",
            "729 0/8 ammo[1]: this crate holds it alone",
            "678 0/8 ammo[1].ox: ours -1 theirs 8",
            "678 0/8 ammo[1].whom: ours -1 theirs 1",
            "729 0/8 ammo[4]: the dump holds it alone",
        ],
        "a row past run145's first rounds moved"
    );
    assert_eq!(
        whole_on(651, 652),
        Vec::<&String>::new(),
        "the first rounds' launch blocks part again"
    );
    assert_eq!(
        whole_on(753, 754),
        vec![
            "753 0/8 ammo[0].angle: ours 740622336 theirs 739377152",
            "753 0/8 ammo[0].sx: ours 940 theirs 941",
            "753 0/8 ammo[0].sy: ours 8154 theirs 8155",
            "754 0/7 ammo[1]: this crate holds it alone",
            "754 0/7 ammo[4]: the dump holds it alone",
            "754 0/8 ammo[0].angle: ours 740622336 theirs 739377152",
            "754 0/8 ammo[0].sx: ours 940 theirs 941",
            "754 0/8 ammo[0].sy: ours 8154 theirs 8155",
        ],
        "the turned chariot's round moved"
    );
    assert_eq!(
        whole_on(WORD, WORD + 1),
        Vec::<&String>::new(),
        "the capture's last block parts"
    );
    let catapult: Vec<String> = firsts
        .iter()
        .filter(|((w, o, what), _)| (*w, *o) == (0, 9) && !standing(what))
        .map(|((_, _, what), (f, row))| format!("{f} {what}: {row}"))
        .collect();
    assert!(
        catapult.is_empty(),
        "the catapult parts from the dump again: {catapult:?}"
    );
}

/// `(block, idle, packed, head order's OrderIndex, its spell)`.
type CatapultRow = (i64, i64, i64, i64, i64);

/// The packed catapult's life on the blocks that decide it, both sides:
/// `(block, idle, packed, head order's OrderIndex, its spell)`, `-1` for
/// no order and no spell.
fn catapult_rows(run: &str, stem: &str, o: i16, at: &[i64]) -> Option<Vec<[CatapultRow; 2]>> {
    let mut s = stage_script(run, stem)?;
    let last = *at.iter().max().expect("a block");
    let mut out = Vec::new();
    for f in 0..last {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if !at.contains(&n) {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == i64::from(o))
            .expect("the catapult in the dump");
        let head = them.orders_front_first().next();
        let theirs = (
            n,
            them.idle.expect("idle"),
            i64::from(them.unit_masks.expect("unit_masks") & 0x8_0000 != 0),
            head.map_or(-1, |h| h.index),
            head.and_then(|h| h.cast_spell).unwrap_or(-1),
        );
        let u = &s.built.sim.units[s.built.sim.unit_by_o(0, o).expect("our catapult")];
        let front = u.orders.front();
        let ours = (
            n,
            i64::from(u.idle),
            i64::from(u.combat.packed),
            front.map_or(-1, |h| i64::from(h.index())),
            front.map_or(-1, |h| match h.body {
                sim::orders::Body::Cast(c) => i64::from(c.spell),
                _ => -1,
            }),
        );
        eprintln!("  {run} 0/{o} block {n}: theirs {theirs:?} ours {ours:?}");
        out.push([theirs, ours]);
    }
    Some(out)
}

/// **The packed catapult unpacks on the original's frame, in both
/// captures** (item 590, `docs/COMBAT.md` §51) — the value diff beside
/// the word's move from 621 to 633. `Unit::think_attack`'s packed arm
/// returns before the target search, so the catapult holds no order at
/// all until the first auto-attack frame with `idle ≥ 7`, and that
/// order is the unpack, `spell 652` (`0x28c`), put at the head.
///
/// run145's `0/9` casts at `idle 7` on 696. run146's `0/6` reaches
/// `idle 7` on 683 and does **not** cast there: 683 is a 16-phase frame
/// (the `idle` bump) and not a 32-phase one (the auto-attack arm), so
/// its cast is 699 at `idle 8`. Both unpack 80 frames later. Every row
/// is the dump's value and ours, so the pin is the original's and a
/// quiet reader cannot pass it.
#[test]
fn chapter_three_s_catapult_unpacks_on_the_dump_s_frame() {
    const CAST: i64 = sim::orders::index::CAST_SPELL as i64;
    for (run, stem, o, want) in [
        (
            "ch3",
            "chapter3",
            9_i16,
            vec![
                (621, 1, 1, -1, -1),
                (622, 2, 1, -1, -1),
                (695, 6, 1, -1, -1),
                (696, 7, 1, CAST, 652),
                (697, 0, 1, CAST, 652),
                (775, 0, 1, CAST, 652),
                (776, 0, 0, -1, -1),
                (777, 1, 0, -1, -1),
            ],
        ),
        (
            "ch3b",
            "chapter3b",
            6,
            vec![
                (606, 1, 1, -1, -1),
                (683, 7, 1, -1, -1),
                (698, 7, 1, -1, -1),
                (699, 8, 1, CAST, 652),
                (700, 0, 1, CAST, 652),
                (778, 0, 1, CAST, 652),
                (779, 0, 0, -1, -1),
            ],
        ),
    ] {
        let at: Vec<i64> = want.iter().map(|r| r.0).collect();
        let Some(rows) = catapult_rows(run, stem, o, &at) else {
            return;
        };
        let theirs: Vec<_> = rows.iter().map(|r| r[0]).collect();
        let ours: Vec<_> = rows.iter().map(|r| r[1]).collect();
        assert_eq!(
            theirs, want,
            "{run}: the dump's catapult is not the one pinned"
        );
        assert_eq!(
            ours, theirs,
            "{run}: this crate's catapult parts from the dump's"
        );
    }
}

/// **The unpacked catapult sees its hoplites on the dump's frame** (item
/// 616, `docs/COMBAT.md` §56) — the value diff beside the restage's move
/// from 780 to 782. `SpellType::cast_unpack` lights the whole disc at the
/// new line of sight, so run146's `0/6`, unpacked on 779, searches on 780
/// at `idle 1` and takes arena A's hoplite seven tiles off. Before the
/// call, its search refused all three in `valid_target`'s fog test.
///
/// Each row is `(block, idle, head order's OrderIndex, target's x, y)`,
/// `-1` for none. The target is compared by **where it stands**, because
/// its `o` is not an identity here (parked 617: the dump's `1/11` is this
/// crate's `1/8`, the same hoplite at (2472, 8136)). The dump's rows are
/// pinned from the dump, so a quiet reader cannot pass.
#[test]
fn chapter_three_s_unpacked_catapult_sees_its_hoplites() {
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let want = [(779, 0, -1, -1, -1), (780, 1, ATTACK, 2472, 8136)];
    let mut rows = Vec::new();
    for f in 0..780 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 779 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let head = them.orders_front_first().next();
        let at = head
            .and_then(|h| Some((h.whom?, h.ox?)))
            .and_then(|(w, o)| {
                frame
                    .units
                    .iter()
                    .find(|u| u.who == w && u.o == o)
                    .map(|t| (t.pos.x, t.pos.y))
            });
        let theirs = (
            n,
            them.idle.expect("idle"),
            head.map_or(-1, |h| h.index),
            at.map_or(-1, |p| p.0),
            at.map_or(-1, |p| p.1),
        );
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let at = match u.combat.target {
            Some(sim::combat::Obj::Unit(t)) => Some(sim.units[t].pos),
            _ => None,
        };
        let ours = (
            n,
            i64::from(u.idle),
            u.orders.front().map_or(-1, |h| i64::from(h.index())),
            at.map_or(-1, |p| i64::from(p.x)),
            at.map_or(-1, |p| i64::from(p.y)),
        );
        eprintln!("  ch3b 0/6 block {n}: theirs {theirs:?} ours {ours:?}");
        rows.push((theirs, ours));
    }
    let theirs: Vec<_> = rows.iter().map(|r| r.0).collect();
    let ours: Vec<_> = rows.iter().map(|r| r.1).collect();
    assert_eq!(theirs, want, "the dump's catapult is not the one pinned");
    assert_eq!(ours, theirs, "this crate's catapult parts from the dump's");
}

/// One unit's order list as the dump holds it, front first, for the
/// ground order's floor: `(OrderIndex, att_x, att_y, accuracy,
/// attack_unit, flags)` for an `ATTACKGROUNDORDER`, and `(OrderIndex,
/// in_range, new_ord, -, -, flags)` for anything else.
fn ground_rows(u: &crate::gamelog::UnitDump) -> Vec<[i64; 6]> {
    u.orders_front_first()
        .map(|o| match o.ag_att_x {
            Some(x) => [
                o.index,
                x,
                o.ag_att_y.unwrap_or(-1),
                o.ag_accuracy.unwrap_or(-1),
                o.ag_attack_unit.unwrap_or(-1),
                o.flags,
            ],
            None => [
                o.index,
                o.in_range.unwrap_or(-1),
                o.new_ord.unwrap_or(-1),
                -1,
                -1,
                o.flags,
            ],
        })
        .collect()
}

/// **The ground order's life, as the dump prints it** (item 621,
/// `docs/COMBAT.md` §57) — the floor, read before any of it was built.
///
/// - **run146**, the restage's catapult `0/6`: the attack arrives on 780
///   (`in_range 0`). On 781 an `ATTACKGROUNDORDER` sits over it at the
///   hoplite's point, **already fired** (`attack_unit 1`, `flags 0x80`),
///   with the attack beneath now `in_range 1` and still `new_ord 1`. The
///   reload reads 83 and runs down one a block. On 864, the ready block,
///   **both** orders are gone. No other unit of the capture ever holds one.
/// - **run44**, where three siege engines of both players carry one: on
///   each first block the order's point is **the position of the target
///   the attack beneath names**, on that block, and not its cell. `1/6`'s
///   (39421, 40762) is off every cell centre.
#[test]
fn the_ground_order_s_life_is_the_dump_s() {
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    const GROUND: i64 = 23;
    let Some((dump, _)) = golden("ch3b") else {
        eprintln!("skipping: no golden capture ch3b (docs/RUNS.md run146)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let mut holders = std::collections::BTreeSet::new();
    let mut life = Vec::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            if u.orders.iter().any(|o| o.index == GROUND) {
                holders.insert((u.who, u.o));
            }
        }
        if (779..=866).contains(&n) {
            let u = f
                .units
                .iter()
                .find(|u| u.who == 0 && u.o == 6)
                .expect("the catapult");
            life.push((n, ground_rows(u), u.recharging.expect("recharging")));
        }
    }
    assert_eq!(
        holders.into_iter().collect::<Vec<_>>(),
        vec![(0, 6)],
        "run146: only the catapult ever holds a ground order"
    );
    for (n, rows, rech) in &life {
        let want: (Vec<[i64; 6]>, i64) = match n {
            779 => (vec![], 0),
            780 => (vec![[ATTACK, 0, 1, -1, -1, 0]], 0),
            781..=863 => (
                vec![[GROUND, 2472, 8136, 0, 1, -128], [ATTACK, 1, 1, -1, -1, 0]],
                864 - n,
            ),
            _ => (vec![], 0),
        };
        assert_eq!((rows.clone(), *rech), want, "run146 block {n}");
    }

    let Some(r44) = crate::testenv::dump("gamelog-run44-islands-turners.txt") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&r44).unwrap();
    let mut firsts = std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        if !(240..=410).contains(&n) {
            continue;
        }
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            let mut front = u.orders_front_first();
            let (Some(head), Some(under)) = (front.next(), front.next()) else {
                continue;
            };
            if head.index != GROUND || firsts.contains_key(&(u.who, u.o)) {
                continue;
            }
            let (w, o) = (under.whom.unwrap(), under.ox.unwrap());
            let t = f
                .units
                .iter()
                .find(|t| t.who == w && t.o == o)
                .expect("the attack's target");
            firsts.insert(
                (u.who, u.o),
                (
                    n,
                    under.index,
                    head.ag_att_x.unwrap(),
                    head.ag_att_y.unwrap(),
                    t.pos.x,
                    t.pos.y,
                ),
            );
        }
    }
    let firsts: Vec<_> = firsts.into_iter().collect();
    assert_eq!(
        firsts,
        vec![
            ((0, 15), (324, ATTACK, 39816, 39720, 39816, 39720)),
            ((1, 6), (247, ATTACK, 39421, 40762, 39421, 40762)),
            ((1, 7), (248, ATTACK, 40152, 41016, 40152, 41016)),
        ],
        "run44: a ground order's point is its attack's target where it stands"
    );
}

/// **A turning siege engine's crew, as the dump prints it** (item 625,
/// `docs/COMBAT.md` §58) — the floor, read before anything was built.
/// run146 prints no figure clock (`GUYS=2`); run44 does (`GUYS=4`), and
/// its human catapult `0/15` is run146's type (265) turning in place on
/// the push of its ground order, 324.
///
/// - **The crew step.** On 324 both crew figures have moved (`last_speed`
///   10 and 20) and stand on their new destinations at the block's end:
///   the tracked arm of `Guy::move` ran and snapped them, which kills the
///   readings "the crew never enter `Guy::move`" and "its tracked arm
///   refuses a step because `x == des_x`" on their first frame.
/// - **The crew mirror.** From 324 to 330 every figure plays `TURN_LEFT`
///   (21), the crew's clock is figure 0's, and the crew's `end_time`
///   stays 79 — the idle's length from before the push, where figure 0's
///   reads 30. Their `last_time` stays −1 throughout, which a step of
///   their own clock would have overwritten. So no `set_anim` reached the
///   crew and their clock never stepped: `Guy::inc_time`'s mirror copied
///   figure 0's slot and time, and nothing else.
/// - **And across the whole of run44, no figure of an unpacked packer
///   that stepped is ever on the walk category.** 114 records, all on a
///   turn slot. That is `Guy::move:176–179`'s gate, from the listing
///   (`5d9565`–`5d9581`): the moving arm asks for the walk only for a
///   non-plane whose type does not pack or is packed.
#[test]
fn a_turning_catapult_s_crew_mirror_and_never_walk() {
    let Some(r44) = crate::testenv::dump("gamelog-run44-islands-turners.txt") else {
        eprintln!("skipping: no run44 (docs/RUNS.md run44)");
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&r44).unwrap();
    let mut turn = Vec::new();
    let mut stepped: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        for u in &f.units {
            let packer = matches!(u.guys.first().and_then(|g| g.kind), Some(265 | 266));
            if !packer || u.unit_masks.is_some_and(|m| m & 0x80000 != 0) {
                continue;
            }
            for g in u.guys.iter().skip(1) {
                if g.last_speed.is_some_and(|s| s != 0) {
                    *stepped.entry(g.cur_anim.unwrap()).or_default() += 1;
                }
            }
            if u.who == 0 && u.o == 15 && (323..=330).contains(&n) {
                let lead = u.guys[0];
                let crew: Vec<_> = u.guys[1..]
                    .iter()
                    .map(|g| {
                        (
                            g.cur_anim.unwrap(),
                            g.cur_time.unwrap(),
                            g.end_time.unwrap(),
                            g.last_time.unwrap(),
                            g.pos.unwrap().x == g.des.unwrap().x
                                && g.pos.unwrap().y == g.des.unwrap().y,
                            g.last_speed.unwrap() != 0,
                        )
                    })
                    .collect();
                turn.push((n, lead.cur_anim.unwrap(), lead.cur_time.unwrap(), crew));
            }
        }
    }
    for (n, anim, time, crew) in &turn {
        let want = if *n == 323 {
            // The idle before the push: the crew on their slots, still.
            (0, 1, vec![(0, 1, 79, -1, true, false); 2])
        } else {
            let t = n - 323;
            (21, t, vec![(21, t, 79, -1, true, true); 2])
        };
        assert_eq!((*anim, *time, crew.clone()), want, "run44 0/15 block {n}");
    }
    assert_eq!(turn.len(), 8, "run44 carries 323–330");
    assert!(
        stepped
            .keys()
            .all(|a| sim::anim::category(*a as i8) != sim::anim::category(sim::anim::WALK)),
        "run44: an unpacked packer's stepping crew played the walk: {stepped:?}"
    );
    assert_eq!(
        stepped.into_iter().collect::<Vec<_>>(),
        vec![(21, 92), (22, 22)],
        "run44: the stepping crew of an unpacked packer"
    );
}

/// **The restage's catapult fires on the ground** (item 621,
/// `docs/COMBAT.md` §57) — the value diff beside the build, both sides
/// on every block of the order's life. Each block is `0/6`'s order list
/// front first in [`ground_rows`]' form and its reload, 779 to 864: the
/// push and the shot on 781, the hold, and both orders gone on 864. And
/// the round, on the block it first prints: from where, to where, how
/// high, and at no object.
///
/// It stops at 864 on purpose. On 865 this crate's catapult takes a
/// fresh attack and on 866 a chase, where the dump's holds nothing until
/// 868: the search after the reload, past the word and not this order's.
#[test]
fn chapter_three_s_catapult_fires_on_the_ground() {
    use sim::orders::Body;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut parted = Vec::new();
    let mut round = None;
    for f in 0..864 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 779 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let theirs = (ground_rows(them), them.recharging.expect("recharging"));
        let sim = &s.built.sim;
        let i = sim.unit_by_o(0, 6).expect("our catapult");
        let u = &sim.units[i];
        let rows: Vec<[i64; 6]> = u
            .orders
            .iter()
            .map(|o| {
                let flags = i64::from(o.flags as i8);
                match o.body {
                    Body::AttackGround(g) => [
                        i64::from(o.index()),
                        i64::from(g.at.x),
                        i64::from(g.at.y),
                        i64::from(g.sea),
                        i64::from(g.attack_unit),
                        flags,
                    ],
                    Body::Attack(a) => [
                        i64::from(o.index()),
                        i64::from(a.in_range),
                        i64::from(a.new_ord),
                        -1,
                        -1,
                        flags,
                    ],
                    _ => [i64::from(o.index()), -1, -1, -1, -1, flags],
                }
            })
            .collect();
        let ours = (rows, i64::from(u.combat.recharging));
        if ours != theirs {
            parted.push((n, ours, theirs));
        }
        if round.is_none() {
            let text = s.ix.read_frame(ix).unwrap();
            if let Some((a, _)) = crate::diff::ammo::blocks(&text)
                .into_iter()
                .find(|(a, _)| a.who == 0 && a.o == 6)
            {
                let mine = sim
                    .projectiles
                    .iter()
                    .find(|p| p.shooter == sim::combat::Obj::Unit(i))
                    .map(|p| {
                        (
                            p.target.is_some(),
                            p.launch.x,
                            p.launch.y,
                            p.sz,
                            p.landing.x,
                            p.landing.y,
                            p.ez,
                            p.accuracy,
                            p.total_time,
                        )
                    });
                round = Some((
                    n,
                    mine,
                    (
                        a.ox != -1,
                        a.sx as i32,
                        a.sy as i32,
                        a.sz as i32,
                        a.ex as i32,
                        a.ey as i32,
                        a.ez as i32,
                        a.accuracy as i32,
                        a.total_time as i32,
                    ),
                ));
            }
        }
    }
    assert!(parted.is_empty(), "0/6's ground order parts: {parted:?}");
    let (n, mine, theirs) = round.expect("the dump's catapult fires");
    eprintln!("  ch3b 0/6 round on {n}: ours {mine:?} theirs {theirs:?}");
    assert_eq!(n, 798, "the dump's round prints on 798");
    assert_eq!(
        theirs,
        (false, 855, 7990, 496, 2413, 8276, 188, 5, 33),
        "the dump's round is not the one pinned"
    );
    // What the order decides agrees: the release block, no object, the
    // ground's `ez` at the point, the accuracy and the flight. **And the
    // landing** (item 625): the point plus the scatter's two draws, taken
    // on 798, now that the stream holds to 792 — (2413, 8276) on both
    // sides, where ours read (2655, 8142) while the crew's walk parted
    // the stream on 782. One field parts, and it is not the order's: the
    // **launch** is the unit's own square here and the release node in
    // the dump, because `sim::launch` has no node for the catapult's
    // piece (§22's seam).
    let mine = mine.expect("this crate's catapult fires");
    assert_eq!(
        (mine.0, mine.4, mine.5, mine.6, mine.7, mine.8),
        (theirs.0, theirs.4, theirs.5, theirs.6, theirs.7, theirs.8),
        "this crate's round parts from the dump's on what the order decides"
    );
    assert_eq!(
        (mine.1, mine.2, mine.3),
        (888, 7992, 251),
        "the launch moved; re-pin it against the dump's"
    );
}

/// **The restage's crew mirror the catapult's turn** (item 625,
/// `docs/COMBAT.md` §58) — the value diff beside the word's move, 782 →
/// 792. On the push, 781, figure 0 turns in place and the crew are pulled
/// round to their rotated slots. Each block 781–785, ours against the
/// dump's: the crew stand where the dump's do (`g.x`, `g.y`, which
/// `GUYS=2` prints), and — the half `GUYS=2` does not print, read off
/// run44's clock (`a_turning_catapult_s_crew_mirror_and_never_walk`) —
/// they play figure 0's `TURN_LEFT` on figure 0's clock rather than a
/// walk of their own.
///
/// Made to fail first: with `Guy::move`'s packer gate off, ours' crew
/// play `CHAR_WALK` on 781.
#[test]
fn chapter_three_s_crew_mirror_the_turn() {
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut rows = Vec::new();
    for f in 0..785 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 781 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let lead = u.guys[0];
        for (g, theirs) in u.guys.iter().zip(&them.guys).skip(1) {
            let at = g.follow.expect("a tracked crew figure").body.pos;
            let tp = theirs.pos.expect("the dump's figure");
            rows.push((
                n,
                i64::from(g.anim),
                i64::from(g.anim) == i64::from(lead.anim) && g.cur_time == lead.cur_time,
                (i64::from(at.x), i64::from(at.y)) == (tp.x, tp.y),
            ));
        }
    }
    let want: Vec<_> = (781..=785)
        .flat_map(|n| [(n, 21, true, true), (n, 21, true, true)])
        .collect();
    assert_eq!(
        rows, want,
        "0/6's crew on the turn: (block, slot, mirrored, on the dump's point)"
    );
}

/// **Chapter three's restage, walked** — `chapter3b.cmd`, run146 (item
/// 587, `docs/GOLDEN.md` §7): the same three unit types in two arenas, so
/// that §7's minimum-range and speed falsifiers can fire. Seven staged
/// lines. Neither fires: the unpacked catapult launches once at eight
/// tiles and then refuses the hoplites inside three for 173 blocks, and
/// the chasing chariots walk at up to 33 units a block against the
/// hoplites' 28–29.
///
/// The word is the chase's first frame: on 633 the original's chariot
/// `0/8` spends one `Unit::fight+0x9b0` and then starts its walk, two
/// `Guy::set_anim+0xf2f` draws for its two figures; this crate spends a
/// second re-search and starts the walk on 634.
#[test]
fn chapter_three_s_restage_holds_to_its_word() {
    let Some(w) = walk_script("ch3b", "chapter3b", 3, 7, 1000) else {
        return;
    };
    // **Item 627: closed.** Word, sequence and values hold to the
    // capture's end, 1000, as run145's do to 900.
    assert_eq!(
        (w.word, w.sequence, w.value),
        (
            GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
            GOLDEN_WORD_CHAPTER_THREE_RESTAGE,
            None
        ),
        "run146's word moved; re-pin it here and say so in docs/GOLDEN.md §7"
    );
}

/// **run146's word, widened whole, both directions** (item 587,
/// `docs/DECISIONS.md` 43): [`widen_chapter_three`] over
/// [`WIDENING_CHAPTER_THREE_RESTAGE`], the whole capture, printing the
/// catapult `0/6` both sides on the word's two blocks.
#[test]
fn chapter_three_s_restage_is_widened_whole() {
    const WORD: i64 = GOLDEN_WORD_CHAPTER_THREE_RESTAGE;
    let Some(w) = widen_chapter_three(
        "ch3b",
        "chapter3b",
        WIDENING_CHAPTER_THREE_RESTAGE,
        WORD,
        6,
        &[651, 652, 780, 781, 865, 866, 868, 869, WORD, WORD + 1],
    ) else {
        return;
    };
    let firsts = w.firsts;
    assert_eq!(w.ammo_theirs, 89, "run146's live rounds are not all read");
    assert!(w.ammo_ours > 0, "this crate fired no round in the window");
    // The standing families on the first block are run145's, the same 26.
    let standing = |what: &str| {
        what == "form" || what == "build:extra" || what.starts_with("leader:filled_gather_slots")
    };
    let at_floor = firsts
        .iter()
        .filter(|(_, (f, _))| *f == WIDENING_CHAPTER_THREE_RESTAGE.0)
        .map(|((_, _, what), _)| what)
        .collect::<Vec<_>>();
    assert!(
        at_floor.iter().all(|w| standing(w)) && at_floor.len() == 26,
        "the standing rows on run146's first block moved: {at_floor:?}"
    );
    // **What parts first, and on the word's own two blocks** (item 595;
    // the delta is in `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment). The
    // first row past the standing families is `0/8`'s first arrow on 651,
    // which now leaves on the dump's frame from the wrong point: the
    // unit's own square and height, where the dump's leaves from the
    // archer's release node. **Nothing parts on the word's blocks**, 664
    // and 665: the draw is the crew figure's swing, and `GUYS=2` prints no
    // figure's clock. That is a blind row, named so that an empty list is
    // not read as agreement.
    let rows_on = |lo: i64, hi: i64| -> Vec<String> {
        firsts
            .iter()
            .filter(|((_, _, what), (f, _))| (lo..=hi).contains(f) && !standing(what))
            .map(|((w, o, what), (f, row))| format!("{f} {w}/{o} {what}: {row}"))
            .collect()
    };
    let first = firsts
        .iter()
        .filter(|((_, _, what), _)| !standing(what))
        .map(|(_, (f, _))| *f)
        .min();
    // **Item 602: 664 → 780.** The crew swings with its leader
    // (`Guy::inc_time`'s foot, `docs/ANIM.md` §5.2) and the rounds leave
    // through the turret (`docs/COMBAT.md` §55). 651–652 agree whole. The
    // first parting is a round's target cleared on its target's death
    // (736), as on run145; on 771 the dump culls the dead hoplites'
    // `DEATH_OBJS` as arena A's are born (`docs/COMBAT.md` §42.5's cull,
    // which this crate does not do), so their `o`s link differently.
    //
    // **Item 616: 780 → 782** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). `cast_unpack` lights the whole disc (`docs/COMBAT.md` §56),
    // so the catapult `0/6` takes an attack on 780 and turns on 781 as the
    // dump's does: no angle or figure-position row parts on 781 now. Its
    // 780 `order:target` row is 617's numbering, not a choice: ours `1/8`
    // and the dump's `1/11` are the same hoplite at (2472, 8136)
    // (`chapter_three_s_unpacked_catapult_sees_its_hoplites`). What parts
    // on 781 is the siege arm this crate does not carry: the original
    // pushes an `ATTACKGROUNDORDER` (index 23) over the attack at the
    // target's point, its figure keeps `ox −1`, and its reload reads 83
    // where ours reads 82 (`docs/COMBAT.md` §56.3).
    //
    // **Item 617: 792 → 865** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). `find_free` skips a held dead number (`docs/COMBAT.md` §59),
    // so arena A's hoplites are `1/9`–`1/11` on both sides and every
    // `extra`/`unlinked` row and the catapult's `order:target` are gone.
    // What the numbering had hidden, in frame order past 736: the
    // catapult's round takes pool slot 0 here and 1 in the dump on 798
    // (run145's pool-slot family, §55.5); the scout `1/0`'s fresh
    // `EXPLORETOORDER` move reads `facing 1` here and 0 in the dump on
    // 847, no draw; and on 865–866 the catapult's fresh attack after the
    // reload (621's park), which was the word.
    //
    // **Item 627: 865 → 1000, closed** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). An unpacked packer's search must reach what it takes, and
    // a packer re-searches before it chases (`docs/COMBAT.md` §60), so
    // every `0/6` row on 865–866 is gone and nothing new parts to the
    // capture's end. What stands, past the first rounds, is three
    // families and no draw: a round's target (736), a round's pool slot
    // (798) and the scout's move facing (847).
    assert_eq!(first, Some(736), "run146's first value parting moved");
    assert_eq!(
        rows_on(WIDENING_CHAPTER_THREE_RESTAGE.0 + 1, WORD + 1),
        vec![
            "798 0/6 ammo[0]: this crate holds it alone",
            "798 0/6 ammo[1]: the dump holds it alone",
            "736 0/7 ammo[0].ox: ours -1 theirs 8",
            "736 0/7 ammo[0].whom: ours -1 theirs 1",
            "847 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
        ],
        "a row past run146's first rounds moved"
    );
    let whole: Vec<&String> = w
        .at_word
        .iter()
        .filter(|r| {
            !r.split(": ")
                .next()
                .is_some_and(|k| standing(k.rsplit(' ').next().unwrap_or("")))
        })
        .collect();
    // The blocks kept whole are the move's, 780–781, and the word's.
    // **Item 621**: the siege arm is carried, so every `0/6` row the
    // ground order made on 781–783 — the order list's length, the
    // unspellable kind, the figure's `ox`/`whom`, the reload's 83 — is
    // gone. What stands is 617's numbering on the attack beneath the
    // ground order, the same hoplite by position.
    //
    // **Item 625: 782 → 792**: the crew mirror the turn
    // (`docs/COMBAT.md` §58), and the word's blocks were 792–793, where
    // nothing parted but 617's numbering.
    //
    // **Item 617: 792 → 865** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). The numbering is the dump's (`docs/COMBAT.md` §59), so
    // 780–781 are quiet whole: the catapult's attack names `1/11` on both
    // sides. The word's blocks, 865–866, are the catapult's fresh attack
    // after its reload, with its chase on 866 (621's park, §57.6): one
    // order here and none in the dump, then its turn and its crew. The
    // scout `1/0`'s `facing` row has stood since 847 and spends no draw.
    //
    // **Item 627: 865 → 1000** (the delta is in
    // `GOLDEN_WORD_CHAPTER_THREE_RESTAGE`'s comment; this is the word's
    // block). The blocks kept whole are 865–866, the old word's, and
    // 868–869, the catapult's first retaliation and the block it drops
    // it, as well as the capture's `!quit` block, 1001. Every `0/6` row
    // is gone from them; the scout's `facing` row stands on each until it
    // ends, and 1001 is quiet.
    assert_eq!(
        whole,
        vec![
            "865 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "866 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "868 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
            "869 1/0 order:move.facing: Move { field: \"facing\", ours: 1, theirs: 0 }",
        ],
        "a row on run146's whole blocks moved"
    );
}

/// **run146's object numbers, both directions** (item 617): every birth
/// and death of a unit, every `DEATH_OBJS` record's arrival and leaving,
/// and every round's pool slot, over the whole capture. The widening links
/// a unit on `(who, o)`, so a unit numbered differently is `extra` on one
/// side and `unlinked` on the other and none of its fields is compared;
/// this links a birth on its frame, owner and point instead, and says
/// which numbers the two sides handed out.
///
/// A row is one event. `birth` pairs on `(frame, who, x, y)`; `death` on
/// the unit's number; `death_obj` on `(who, o, first_frame)`; `ammo` on
/// `(frame, shooter, slot)`. Anything on one side only is its own row.
#[test]
fn chapter_three_s_restage_numbers_its_objects() {
    use std::collections::{BTreeMap, BTreeSet};
    let (first, last) = WIDENING_CHAPTER_THREE_RESTAGE;
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    type Units = BTreeMap<(i64, i64), (i64, i64)>;
    let (mut ours_prev, mut theirs_prev): (Units, Units) = Default::default();
    let (mut ours_do, mut theirs_do) = (BTreeSet::new(), BTreeSet::new());
    let (mut ours_ammo, mut theirs_ammo) = (BTreeSet::new(), BTreeSet::new());
    let mut rows: Vec<String> = Vec::new();
    let mut events = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let sim = &s.built.sim;
        let ours: Units = sim
            .units
            .iter()
            .filter(|u| u.alive() && u.owner < 8)
            .map(|u| {
                (
                    (i64::from(u.owner), i64::from(u.index)),
                    (i64::from(u.pos.x), i64::from(u.pos.y)),
                )
            })
            .collect();
        let theirs: Units = frame
            .units
            .iter()
            .filter(|u| (0..8).contains(&u.who))
            .map(|u| ((u.who, u.o), (u.pos.x, u.pos.y)))
            .collect();
        let raw = s.ix.read_frame(at).unwrap();
        let o_do: BTreeSet<(i64, i64, i64)> = sim
            .deaths
            .iter()
            .map(|d| (i64::from(d.who), i64::from(d.o), d.first_frame))
            .collect();
        let t_do: BTreeSet<(i64, i64, i64)> = frame
            .deaths
            .iter()
            .filter(|d| d.valid == Some(1))
            .filter_map(|d| Some((d.who?, d.o?, d.first_frame?)))
            .collect();
        let o_ammo: BTreeSet<(i64, i64, i64)> = sim
            .projectiles
            .iter()
            .filter_map(|p| match p.shooter {
                sim::combat::Obj::Unit(u) => {
                    let un = &sim.units[u];
                    Some((i64::from(un.owner), i64::from(un.index), i64::from(p.slot)))
                }
                sim::combat::Obj::Building(_) => None,
            })
            .collect();
        let t_ammo: BTreeSet<(i64, i64, i64)> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0 && a.who < 8)
            .map(|(a, _)| (a.who, a.o, a.index))
            .collect();
        // The window's first block seeds both sides: block 1 before it
        // prints no unit, so everything on 605 would read as born.
        if n > first {
            // Births, paired on the frame, the owner and the point.
            let born = |now: &Units, prev: &Units| -> BTreeMap<(i64, i64, i64), i64> {
                now.iter()
                    .filter(|(k, _)| !prev.contains_key(k))
                    .map(|(&(who, o), &(x, y))| ((who, x, y), o))
                    .collect()
            };
            let (ob, tb) = (born(&ours, &ours_prev), born(&theirs, &theirs_prev));
            for k in ob.keys().chain(tb.keys()).collect::<BTreeSet<_>>() {
                events += 1;
                let (who, x, y) = *k;
                match (ob.get(k), tb.get(k)) {
                    (Some(a), Some(b)) if a == b => {}
                    (a, b) => rows.push(format!(
                        "{n} birth {who} at ({x}, {y}): ours {a:?} theirs {b:?}"
                    )),
                }
            }
            // Deaths, on the number.
            let died = |now: &Units, prev: &Units| -> BTreeSet<(i64, i64)> {
                prev.keys()
                    .filter(|k| !now.contains_key(k))
                    .copied()
                    .collect()
            };
            let (od, td) = (died(&ours, &ours_prev), died(&theirs, &theirs_prev));
            for k in od.symmetric_difference(&td) {
                rows.push(format!(
                    "{n} death {}/{}: {} alone",
                    k.0,
                    k.1,
                    if od.contains(k) { "ours" } else { "the dump's" }
                ));
            }
            events += od.union(&td).count();
            // The death-object list and the rounds, arrivals and leavings.
            for (what, o_prev, t_prev, o_now, t_now) in [
                ("death_obj", &ours_do, &theirs_do, &o_do, &t_do),
                ("ammo", &ours_ammo, &theirs_ammo, &o_ammo, &t_ammo),
            ] {
                for (dir, o_set, t_set) in [
                    ("arrives", o_now - o_prev, t_now - t_prev),
                    ("leaves", o_prev - o_now, t_prev - t_now),
                ] {
                    events += o_set.union(&t_set).count();
                    for k in o_set.symmetric_difference(&t_set) {
                        rows.push(format!(
                            "{n} {what} {dir} {k:?}: {} alone",
                            if o_set.contains(k) {
                                "ours"
                            } else {
                                "the dump's"
                            }
                        ));
                    }
                }
            }
        }
        (ours_prev, theirs_prev) = (ours, theirs);
        (ours_do, theirs_do) = (o_do, t_do);
        (ours_ammo, theirs_ammo) = (o_ammo, t_ammo);
    }
    for r in &rows {
        eprintln!("  numbering: {r}");
    }
    assert!(events > 0, "run146's numbering walk read no event");
    // **The floor, before any reading (item 617)**, and what it was when
    // it was written: every death agrees
    // (`1/6` on 680, `1/7` on 728, `1/8` on 736), and so does the whole
    // death-object list: the three records arrive on those blocks on both
    // sides and **none leaves** before the capture ends. So nothing is
    // culled on 771 or anywhere else in the window. The first parting is
    // the births on 771: arena A's three hoplites, born on the same
    // points, take the dump's 9–11 and ours' 6–8, the numbers of the dead
    // whose death objects both sides still hold. **Since item 617's
    // build** (`docs/COMBAT.md` §59) those three rows are gone: a dead
    // number is not handed out while its death object holds it.
    //
    // The rounds' pool slot is a second family, run145's (`docs/COMBAT.md`
    // §55.5): the catapult's one round takes slot 1 in the dump and 0
    // here on 798. **Since item 627** (`docs/COMBAT.md` §60) ours no
    // longer fires again on 960: the catapult's fresh attack after its
    // reload, and the chase that carried it out of its minimum, are gone.
    assert_eq!(
        rows,
        vec![
            "798 ammo arrives (0, 6, 0): ours alone",
            "798 ammo arrives (0, 6, 1): the dump's alone",
            "830 ammo leaves (0, 6, 0): ours alone",
            "830 ammo leaves (0, 6, 1): the dump's alone",
        ],
        "run146's numbering moved"
    );
}

/// **The value diff on the frame 617 moved** (`docs/COMBAT.md` §59): arena
/// A's three hoplites, `1/9`–`1/11` on both sides, on 790–793. Each row is
/// `(block, o, x, y, idle, the order list's indices)`, the list sorted,
/// because the dump prints it from the rotated head. The dump's reach `idle 4`
/// on 790, 791 and 792 (`(frame + o) & 15`), and take their attack on the
/// catapult with it. Ours had numbered them 6–8, so they reached it on
/// 793–795.
///
/// Made to fail first: with `Objects::find_free`'s hold test off, ours
/// hold no `1/9`–`1/11`.
#[test]
fn chapter_three_s_arena_a_hoplites_take_the_dump_s_numbers() {
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let (mut ours, mut theirs) = (Vec::new(), Vec::new());
    for f in 0..793 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 790 {
            continue;
        }
        let at = s.ix.frames().iter().position(|x| x.number == n).unwrap();
        let frame = s.ix.frame_state(at).unwrap();
        for o in 9..=11 {
            if let Some(u) = frame.units.iter().find(|u| u.who == 1 && u.o == o) {
                theirs.push((n, o, u.pos.x, u.pos.y, u.idle.unwrap_or(-1), {
                    let mut k: Vec<i64> = u.orders.iter().map(|d| d.index).collect();
                    k.sort_unstable();
                    k
                }));
            }
            let sim = &s.built.sim;
            if let Some(i) = sim.unit_by_o(1, o as i16) {
                let u = &sim.units[i];
                ours.push((
                    n,
                    o,
                    i64::from(u.pos.x),
                    i64::from(u.pos.y),
                    i64::from(u.idle),
                    {
                        let mut k: Vec<i64> =
                            u.orders.iter().map(|d| i64::from(d.index())).collect();
                        k.sort_unstable();
                        k
                    },
                ));
            }
        }
    }
    for r in &theirs {
        eprintln!("  theirs {r:?}");
    }
    assert_eq!(theirs.len(), 12, "the dump's arena-A hoplites");
    assert_eq!(ours, theirs, "arena A's hoplites on 790-793");
}

/// **§7's three falsifiers, as the dumps print them** (item 587). Each was
/// a `check:` in a `.cmd` header before its run; this is the same reading
/// made an assertion, so a re-take that changed any of them would fail
/// here rather than in a journal.
///
/// - **The leading count**, both captures: `add 3 chariot` is three
///   separate one-unit Chariots (`o_up`/`o_down` −1, type 195) on 611.
/// - **The minimum range**, run146: the catapult `0/6` launches exactly
///   one round, and on its launch block no live hoplite of arena A stands
///   within `3 × 192 − 6` of it — while on 173 blocks one does, with the
///   catapult alive, which is what makes the refusal a measurement.
/// - **The speed**, run146: a chariot walks on three or more blocks, and
///   its largest one-block step is longer than any of arena B's hoplites'
///   (compared squared: this reads the dump, and does no float arithmetic).
#[test]
fn chapter_three_s_falsifiers_are_the_dump_s() {
    use std::collections::BTreeMap;
    let alive = |u: &crate::gamelog::UnitDump| u.myhits.is_some_and(|h| h > 1);
    for run in ["ch3", "ch3b"] {
        let Some((dump, _)) = golden(run) else {
            eprintln!("skipping: no golden capture {run} (docs/RUNS.md run145, run146)");
            return;
        };
        let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
        let at = ix.frames().iter().position(|x| x.number == 611).unwrap();
        let f = ix.frame_state(at).unwrap();
        let chariots: Vec<_> = f
            .units
            .iter()
            .filter(|u| u.who == 0 && u.guys.first().and_then(|g| g.kind) == Some(195))
            .map(|u| (u.o, u.o_up, u.o_down))
            .collect();
        assert_eq!(
            chariots.len(),
            3,
            "{run}: `add 3 chariot` is not three units"
        );
        assert!(
            chariots.iter().all(|c| c.1 == Some(-1) && c.2 == Some(-1)),
            "{run}: the chariots are threaded as a squad: {chariots:?}"
        );
    }
    let Some((dump, _)) = golden("ch3b") else {
        return;
    };
    let mut ix = crate::capture::indexed::IndexedCapture::open(&dump).unwrap();
    let d2 =
        |a: &crate::gamelog::Pos, b: &crate::gamelog::Pos| (a.x - b.x).pow(2) + (a.y - b.y).pow(2);
    const DEAD_ZONE: i64 = 3 * 192 - 6;
    let (mut launches, mut close_launches, mut inside) = (0usize, 0usize, 0usize);
    let mut last: BTreeMap<(i64, i64), crate::gamelog::Pos> = BTreeMap::new();
    let mut steps: BTreeMap<(i64, i64), (usize, i64)> = BTreeMap::new();
    for at in 0..ix.frames().len() {
        let n = ix.frames()[at].number;
        let f = ix.frame_state(at).unwrap();
        let raw = ix.read_frame(at).unwrap();
        let live: BTreeMap<(i64, i64), &crate::gamelog::UnitDump> = f
            .units
            .iter()
            .filter(|u| alive(u))
            .map(|u| ((u.who, u.o), u))
            .collect();
        let nearest = live.get(&(0, 6)).and_then(|c| {
            (9..=11)
                .filter_map(|o| live.get(&(1, o)))
                .map(|h| d2(&c.pos, &h.pos))
                .min()
        });
        if nearest.is_some_and(|d| d < DEAD_ZONE * DEAD_ZONE) {
            inside += 1;
        }
        for (a, _) in super::ammo::blocks(&raw) {
            if (a.who, a.o, a.cur_time) == (0, 6, 1) {
                launches += 1;
                if nearest.is_none_or(|d| d < DEAD_ZONE * DEAD_ZONE) {
                    close_launches += 1;
                }
            }
        }
        for (k, u) in &live {
            if let Some(p) = last.get(k) {
                let s = d2(p, &u.pos);
                if s > 0 {
                    let e = steps.entry(*k).or_insert((0, 0));
                    e.0 += 1;
                    e.1 = e.1.max(s);
                }
            }
        }
        last = live.iter().map(|(k, u)| (*k, u.pos)).collect();
        let _ = n;
    }
    eprintln!(
        "run146: {launches} catapult launch(es), {close_launches} inside the dead zone, \
         {inside} blocks a hoplite stands inside it; steps {steps:?}"
    );
    assert_eq!(
        (launches, close_launches, inside),
        (1, 0, 173),
        "§7's minimum-range falsifier: the catapult's launches, those inside \
         three tiles, and the blocks that make the refusal a measurement"
    );
    let chariot = (7..=9)
        .filter_map(|o| steps.get(&(0, o)))
        .filter(|s| s.0 >= 3)
        .map(|s| s.1)
        .max()
        .expect("no chariot walks three blocks in run146");
    let hoplite = (6..=8)
        .filter_map(|o| steps.get(&(1, o)))
        .map(|s| s.1)
        .max()
        .expect("arena B's hoplites never walk");
    assert!(
        chariot > hoplite,
        "§7's speed falsifier fires: a chariot's longest step² {chariot} is not \
         longer than a hoplite's {hoplite}"
    );
}

/// **The restage's catapult after its reload** (item 627,
/// `docs/COMBAT.md` §60) — the floor, then the value diff beside the
/// word's move, 865 → 1000. Each block 858–880, both sides: `0/6`'s order
/// list front first (kind, target, `in_range`, `new_ord`, signed flags,
/// or the ground order's point), its `idle`, its reload, its point and
/// its `orders_x/y`; and arena A's three hoplites' points.
///
/// What the dump says, and what the floor read before anything was
/// built: the ground order and the attack beneath die on 864 on both
/// sides; the hoplites stand still inside the catapult's 570 minimum
/// throughout; the dump's catapult then holds nothing until the block
/// after each hit (868, 870, 871), holds that one attack for one block,
/// and never moves. Ours took an attack from its idle search on 864 and
/// chased away for its range from 865.
///
/// Made to fail first: with `find_nearby_target`'s range gate read back
/// as "takes anything", 865 parts on the order list.
#[test]
fn chapter_three_s_catapult_after_its_reload() {
    use sim::orders::Body;
    type Row = (Vec<[i64; 6]>, i64, i64, (i64, i64), (i64, i64));
    let Some(mut s) = stage_script("ch3b", "chapter3b") else {
        return;
    };
    let mut parted = Vec::new();
    let mut theirs_shape = Vec::new();
    for f in 0..880 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < 858 {
            continue;
        }
        let ix =
            s.ix.frames()
                .iter()
                .position(|x| x.number == n)
                .expect("the block");
        let frame = s.ix.frame_state(ix).unwrap();
        let them = frame
            .units
            .iter()
            .find(|u| u.who == 0 && u.o == 6)
            .expect("the catapult in the dump");
        let theirs: Row = (
            them.orders_front_first()
                .map(|o| match o.ag_att_x {
                    Some(x) => [o.index, x, o.ag_att_y.unwrap_or(-1), -1, -1, o.flags],
                    None => [
                        o.index,
                        o.whom.unwrap_or(-1),
                        o.ox.unwrap_or(-1),
                        o.in_range.unwrap_or(-1),
                        o.new_ord.unwrap_or(-1),
                        o.flags,
                    ],
                })
                .collect(),
            them.idle.expect("idle"),
            them.recharging.expect("recharging"),
            (them.pos.x, them.pos.y),
            (
                them.orders_x.expect("orders_x"),
                them.orders_y.expect("orders_y"),
            ),
        );
        let sim = &s.built.sim;
        let u = &sim.units[sim.unit_by_o(0, 6).expect("our catapult")];
        let (tw, to) = match u.combat.target {
            Some(sim::combat::Obj::Unit(t)) => {
                (i64::from(sim.units[t].owner), i64::from(sim.units[t].index))
            }
            _ => (-1, -1),
        };
        let ours: Row = (
            u.orders
                .iter()
                .map(|o| {
                    let flags = i64::from(o.flags as i8);
                    match o.body {
                        Body::AttackGround(g) => [
                            i64::from(o.index()),
                            i64::from(g.at.x),
                            i64::from(g.at.y),
                            -1,
                            -1,
                            flags,
                        ],
                        Body::Attack(a) => [
                            i64::from(o.index()),
                            tw,
                            to,
                            i64::from(a.in_range),
                            i64::from(a.new_ord),
                            flags,
                        ],
                        _ => [i64::from(o.index()), -1, -1, -1, -1, flags],
                    }
                })
                .collect(),
            i64::from(u.idle),
            i64::from(u.combat.recharging),
            (i64::from(u.pos.x), i64::from(u.pos.y)),
            (i64::from(u.orders_pos.x), i64::from(u.orders_pos.y)),
        );
        for o in 9..=11i16 {
            let t = frame
                .units
                .iter()
                .find(|x| x.who == 1 && i64::from(o) == x.o)
                .expect("a hoplite in the dump");
            let k = sim.unit_by_o(1, o).expect("our hoplite");
            let p = sim.units[k].pos;
            if (i64::from(p.x), i64::from(p.y)) != (t.pos.x, t.pos.y) {
                parted.push(format!(
                    "{n} 1/{o} at ({}, {}) against ({}, {})",
                    p.x, p.y, t.pos.x, t.pos.y
                ));
            }
        }
        if ours != theirs {
            parted.push(format!("{n} 0/6 ours {ours:?} theirs {theirs:?}"));
        }
        theirs_shape.push((
            n,
            theirs
                .0
                .iter()
                .map(|r| (r[0], r[1], r[2]))
                .collect::<Vec<_>>(),
        ));
    }
    // The dump's own shape, pinned so the value diff above cannot agree
    // with a different one: the ground order over the attack to 863,
    // nothing 864–867, one attack on the hitter the block after each hit.
    const ATTACK: i64 = sim::orders::index::ATTACK as i64;
    let attacks: Vec<_> = theirs_shape
        .iter()
        .filter(|(n, v)| *n >= 864 && !v.is_empty())
        .cloned()
        .collect();
    assert_eq!(
        attacks,
        vec![
            (868, vec![(ATTACK, 1, 9)]),
            (870, vec![(ATTACK, 1, 11)]),
            (871, vec![(ATTACK, 1, 10)]),
        ],
        "the dump's catapult after its reload"
    );
    assert!(
        parted.is_empty(),
        "run146's catapult after its reload: {parted:#?}"
    );
}

/// **Chapter seven-b's control: the scout's search on 1076, priced step
/// for step against the original's** (item 647, run157). The capture's
/// `rontrace.cfg` proxies `PathFinder::astar_path` and `calc_cost` over
/// the whole run, so the search that built `1/0`'s explore path (block
/// 1077, 48 nodes against this crate's 47) is on disk call for call.
/// Every step both sides price is compared on its whole argument list
/// (`docs/PATHFINDER.md` §17's key); a key only one side priced is the
/// search parting, and is counted, not compared.
#[test]
fn chapter_seven_b_s_control_scout_search_is_priced_as_the_original() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some((dump, tracepath)) = golden("ch7bc") else {
        eprintln!("skipping: no golden capture ch7bc (see docs/RUNS.md)");
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
    let mut script = script_named("chapter7b_control");
    const SEARCH: i64 = 1076;
    while built.sim.frame < SEARCH {
        script.stage(built.sim.frame, &mut built, &loaded);
        built.tick();
    }
    script.stage(built.sim.frame, &mut built, &loaded);
    built.sim.trace_costs = true;
    built.tick();
    let ours = std::mem::take(&mut built.sim.cost_marks);
    let theirs = trace.calls_in(SEARCH, crate::trace::call_site::CALC_COST);
    let theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> = theirs
        .iter()
        .map(|c| (c.cost_key().expect("a calc_cost call"), c.ret))
        .collect();
    let mut shared = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for m in &ours {
        let Some(&t) = theirs_by_key.get(&m.key()) else {
            continue;
        };
        shared += 1;
        if t != m.cost {
            wrong.push(format!(
                "({},{})->({},{}) depth {}: ours {} theirs {t}",
                m.from.0 / m.step,
                m.from.1 / m.step,
                m.to.0 / m.step,
                m.to.1 / m.step,
                m.depth,
                m.cost,
            ));
        }
    }
    eprintln!(
        "run157 f{SEARCH}: ours {} steps, theirs {}, {shared} shared, {} priced apart",
        ours.len(),
        theirs.len(),
        wrong.len()
    );
    for w in &wrong {
        eprintln!("  {w}");
    }
    // **The floor, before the fix** (item 647, `33952d3`): 969 steps here
    // against 945, 587 shared, and nine priced apart, every one into cell
    // (48,31) or (47,31), the south half of the Small City `1/2007`'s
    // footprint (started on 1069, `mylos 0`). The original priced them
    // seen — 128 base, 20 × 9 of blocked tiles, − 4 own ground, + 8 on a
    // diagonal, so 304 and 312 — and this crate as unseen scouting ground,
    // 1 and 9 (`docs/PATHFINDER.md` §5). `Wall::start`'s write of the
    // owner's bit over the footprint is what lights them (`docs/SCOUT.md`
    // §14): with it the two searches are one, step for step.
    assert_eq!(
        (ours.len(), theirs.len(), shared),
        (945, 945, 945),
        "run157 f{SEARCH}: the search's steps moved"
    );
    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "run157 f{SEARCH}: steps priced apart"
    );
}
