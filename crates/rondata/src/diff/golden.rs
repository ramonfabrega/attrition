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

pub(super) mod observations;
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
    let init = log.initial().expect("the golden dump has a BEGIN GAME");
    stand_up_initial(loaded, init, refs, trace)
}

fn stand_up_initial<'a>(
    loaded: &crate::load::Loaded,
    mut init: Initial<'a>,
    refs: &[&Initial<'a>],
    trace: &crate::trace::Trace,
) -> Built {
    let own_seeds = init.frame_seeds.clone();
    let own_guys = init.frame_guys.clone();
    borrow_from_siblings(&mut init, refs);
    init.frame_seeds = own_seeds;
    init.frame_guys = own_guys;
    borrow_pasture(&mut init, trace);
    build_sim(loaded, &init, Tuning::RON)
}

fn siblings(texts: &[crate::capture::Text]) -> Vec<Log<'_>> {
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
    let (mut built, _) = build_golden(run, &dump, &loaded, &trace, true)?;
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
        // The pump first: the trace's frame opens at `do_frame`'s entry,
        // after it (item 1167).
        applied.merge(&script.pump(built.sim.frame, &mut built));
        words.push((built.sim.frame, built.sim.rng.seed));
        let did = script.stage(built.sim.frame, &mut built, &loaded);
        applied.merge(&did);
        let f = built.sim.frame;
        let seed = built.sim.rng.seed;
        built.tick();
        if sites.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
            // The frame's opening word, so each roll can be named (922).
            eprintln!("  f{f} seed {seed:#010x}");
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
    // Where each `add` landed and what each issuer line reached (Loops
    // 2970 and 2654), so a staging is read off the walk, not a probe.
    for line in applied.landed.iter().chain(&applied.reached) {
        eprintln!("  {line}");
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

// Keep case bodies in chapter modules for incremental compiler reuse. These
// wrappers preserve every original libtest name and fixture-audit thread label.
// Shared helpers and comparison instrumentation remain in this crate.
// See docs/journal/2026-10-16-replay-boundary.md.
mod cases_01_09;
mod cases_10_19;
mod cases_20_29;
mod cases_30_39;
mod cases_40_49;
mod cases_50_59;
mod cases_60_69;
mod cases_70_79;
mod cases_80_89;
mod cases_setup;

#[test]
fn indexed_near_reads_preserve_duplicates_missing_fields_and_trailing_records() {
    cases_setup::indexed_near_reads_preserve_duplicates_missing_fields_and_trailing_records();
}

/// **The setup is shared, and this is what says so.** The golden capture
/// carries no `game_random` record of its own, so `borrow_from_siblings`
/// hands it run11's setup trace whole. That is only legitimate if the two
/// runs' setups really are one — and the run's own instrument settles it:
/// the borrowed trace's last word must be the word the golden run's trace
/// records at its first frame-0 draw.
#[test]
fn the_golden_record_s_setup_is_run11_s() {
    cases_setup::the_golden_record_s_setup_is_run11_s();
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
    cases_01_09::chapter_one_holds_to_the_golden_word();
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
    cases_01_09::chapter_two_holds_to_the_golden_word();
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
    cases_01_09::chapter_five_holds_to_the_golden_word();
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
    cases_01_09::chapter_six_holds_to_the_golden_word();
}

/// **Chapter six-b, pinned** — the air line from a base (`docs/GOLDEN.md`
/// §10, item 651, run175). Eight staged lines: chapter six's six and an
/// `add airbase` for each side on 606 and 608, before the aircraft.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run175): ten `INFO cmd` records each returning 1; 647 blocks; run168's
/// game to frame 611 by `rngcmp.py`; both Airbases finished on 607 and
/// 609; the aircraft on run168's seats. **The first falsifier fired, on
/// the target arm**: each aircraft takes an `ATTACKORDER` on the enemy
/// Airbase on its birth block and walks to it on the ground, `air_alt` 0;
/// `inside_up` stays −1, so the base link is dead as read.
///
/// `GOLDEN_WORD_CHAPTER_SIX_B` carries what stands at the word.
#[test]
fn chapter_six_b_holds_to_the_golden_word() {
    cases_01_09::chapter_six_b_holds_to_the_golden_word();
}

/// **Chapter eleven, pinned** — the guard line, an issuer the AI never
/// uses from a command (`docs/GOLDEN.md` §19, item 696, run190). Eight
/// staged lines: `!ai off`, a Chariot on 610, a Supply Wagon on 612, a
/// Hoplite squad on 614, two `@guard` issuer lines on 620 and 640 (the
/// chariot on the wagon, the squad on the building `0/2001`), a `@move` of
/// the wagon on 720, and who=1's Chariot on 1000.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run190): all three issues appended (`INFO 17`, refusal 0) and processed
/// on the next frame; one `GUARDORDER` on the wagon at (0, 372) on 622;
/// the post re-read as the wagon walked; the guard's attack above its
/// guard on 1011; no order on the building's squad. No falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_ELEVEN` carries what stands at the word.
#[test]
fn chapter_eleven_holds_to_the_golden_word() {
    cases_10_19::chapter_eleven_holds_to_the_golden_word();
}

/// **Chapter twelve, pinned** — the follow line, an issuer the AI never
/// uses (`docs/GOLDEN.md` §20, item 714, run204). Eleven staged lines:
/// `!ai off`, a Chariot on 610 and its leader, a Supply Wagon, on 612; a
/// Hoplite squad on 614 and its leader, a Chariot, on 616; two `@follow`
/// issuer lines on 620 and 640; and four `@move`s, walking each leader on
/// 700 and 720 and turning it on 880 and 960.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run204): both follows appended (`INFO 17`, refusal 0) and processed on
/// the next frame; one `FOLLOWORDER` a unit on its leader on 622 and 642;
/// the chariot trailing in one-tile legs from 711, the hoplites from 722.
/// No falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_TWELVE` carries what stands at the word.
#[test]
fn chapter_twelve_holds_to_the_golden_word() {
    cases_10_19::chapter_twelve_holds_to_the_golden_word();
}

/// **Chapter thirteen, pinned** — the garrison line, an issuer the AI
/// never uses from a command (`docs/GOLDEN.md` §21, item 718, run208).
/// Seven staged lines: `!ai off`, a Barracks on 606, a Chariot on 610 and
/// a Hoplite squad on 614; two `@garrison` issuer lines on 620 and 640,
/// both into the Barracks; and an `@eject` on 900.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run208): all three commands appended (`INFO 17`, refusal 0) and
/// processed on the next frame; one `GARRISONORDER` a unit under its own
/// plain leg on 622 and 642; the chariot inside on 699, the squad whole on
/// 761; the chariot out on 902 and the squad on 903. No falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_THIRTEEN` carries what stands at the word.
#[test]
fn chapter_thirteen_holds_to_the_golden_word() {
    cases_10_19::chapter_thirteen_holds_to_the_golden_word();
}

/// **Chapter fourteen, pinned** — the formation line, an issuer the AI
/// never uses (`docs/GOLDEN.md` §22, item 723, run210). Seven staged
/// lines: `!ai off`, two Hoplite squads on 610 and 612 and a Slinger squad
/// on 614; an `@form` issuer line (Envelop) on 620, an `@move` on 700, and
/// an `@form` (Line) on 740.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run210): all three commands processed on the next frame; no
/// `FORMORDER` anywhere; `form 2` on all nine on 622, where each holds a
/// plain move to its Envelop slot round the leader, no action bit; on 742
/// the walking group halted and its move replayed in Line under a new id.
///
/// `GOLDEN_WORD_CHAPTER_FOURTEEN` carries what stands at the word.
#[test]
fn chapter_fourteen_holds_to_the_golden_word() {
    cases_10_19::chapter_fourteen_holds_to_the_golden_word();
}

/// **Chapter fifteen, pinned** — the group attack, an issuer the AI
/// rarely takes whole (`docs/GOLDEN.md` §23, item 731, run215). Seven
/// staged lines: `!ai off`, two Hoplite squads on 610 and 612 and a who=1
/// Chariot on 614; a right-click `@move` on 620, an `@attack` on 734 and
/// an `@amove` on 860.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run215): all three commands processed on the next frame; six
/// `ATTACKORDER`s, `mandatory 1`, the action bit, on 736, and no
/// `GroupAttackOrder` anywhere; the target last prints on 808; six
/// `GROUPATTACKTOORDER`s on 862, standing on their points from 1080.
///
/// `GOLDEN_WORD_CHAPTER_FIFTEEN` carries what stands at the word.
#[test]
fn chapter_fifteen_holds_to_the_golden_word() {
    cases_10_19::chapter_fifteen_holds_to_the_golden_word();
}

/// **Chapter sixteen, pinned** — explore and flee, the move issuer's
/// trailing selector (`docs/GOLDEN.md` §24, item 738, run219). Seven
/// staged lines: `!ai off`, a Chariot on 610, a Hoplite squad on 612,
/// and four issuer lines — `@explore` on 620 and 640, `@flee` on 900 and
/// 1000 — which the harness runs one frame later through
/// `input::group_move_to` with `orders` 3 and 4.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run219): one `EXPLORETOORDER` or `FLEETOORDER` a member and no group
/// order; box legs on 685 and 804, boxes opened on 730 and 855; the
/// re-issue behind a box leg goes to the click. No falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_SIXTEEN` carries what stands at the word.
#[test]
fn chapter_sixteen_holds_to_the_golden_word() {
    cases_10_19::chapter_sixteen_holds_to_the_golden_word();
}

/// **Chapter seventeen, pinned** — the flight line, an issuer the AI
/// rarely takes (`docs/GOLDEN.md` §25, item 746, run223). Eleven staged
/// lines: `!ai off`, `library who=0 6`, who=0's Airbase on 606, a
/// Fighter on 610 and a Bomber pair on 612 and 614, who=1's Barracks on
/// 616; and four issuer lines through `issue_flight@00941d40` — `@strike`
/// on 620 and 664, `@flight` on 640 and 660.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run223): the strike on the ground took no order; each flight home is
/// one `STRAFEORDER`, `returning 1`, home `0/2007`; the flying pair's
/// strike became an `AIRPATROLORDER` over the unseen Barracks' point; the
/// Fighter inside its base on 722.
///
/// **Item 759 flew the flight home** (`docs/ORDERS.md` §33): the word
/// walks 642 → 805, the first bomb. **Item 763 flew the pair's patrol**
/// (§34): 805 → 821, the bomb's damage. `GOLDEN_WORD_CHAPTER_SEVENTEEN`
/// carries what stands at the word.
#[test]
fn chapter_seventeen_holds_to_the_golden_word() {
    cases_10_19::chapter_seventeen_holds_to_the_golden_word();
}

/// **Chapter eighteen, pinned** — the build line, an issuer the AI takes
/// through its own planner (`docs/GOLDEN.md` §26, item 779, run241).
/// Eight staged lines: `!ai off`, `library who=0 2`, four citizens of
/// who=0 on 610–616, and two issuer lines through `issue_build@00941c30`
/// — `@build` on 620 (`0/6`, a Barracks) and 640 (`0/7`–`0/9`, a Siege
/// Factory).
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run241): each citizen a `MOVEORDER` and a `BUILDORDER` (flags 4) on its
/// processed block, both sites paid; the Siege Factory finished on 948
/// and the Barracks on 1141; `0/8`'s `find_build_spot` help on 1097.
///
/// `GOLDEN_WORD_CHAPTER_EIGHTEEN` carries what stands at the word.
#[test]
fn chapter_eighteen_holds_to_the_golden_word() {
    cases_10_19::chapter_eighteen_holds_to_the_golden_word();
}

/// **Chapter nineteen, pinned** — the cast line, a Spy's Informer on an
/// enemy building (`docs/GOLDEN.md` §27, item 790, run245). Six staged
/// lines: `!ai off`, `library` for both sides, a who=1 Barracks on 606, a
/// who=0 Spy on 610, and one issuer line through `issue_spell@00941b80` —
/// `@spell` on 620, the Informer (639) on `1/2006`.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run245): a `CASTORDER` (flags 4, `paid` 1) and a `MOVEORDER` to
/// (14232, 15528) on 622, `mana_burn` +500; the Spy in range on 756 and the
/// Barracks `infiltrated` on 795.
///
/// `GOLDEN_WORD_CHAPTER_NINETEEN` carries what stands at the word.
#[test]
fn chapter_nineteen_holds_to_the_golden_word() {
    cases_10_19::chapter_nineteen_holds_to_the_golden_word();
}

/// **Chapter twenty, pinned** — the board line: the transport toggle and
/// the move it gates (`docs/GOLDEN.md` §28, item 803, run249). Eleven
/// staged lines: `!ai off`, `library who=0 1`, a who=0 Dock on 604, two
/// Chariots on 610 and 612, and six issuer lines — `@settransport` 0 on
/// `0/7` on 620, both Chariots moved onto lake 70 on 640 and 642,
/// `@settransport` 1 on `0/7` on 800, its move again on 820, and the barge
/// `0/8` to the east bank on 900.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run249): `0/7`'s bit cleared on 622 and set on 802, no order laid by
/// either; `0/6`'s Transport `CASTORDER` (650, flags 0) on 703 and barge
/// `0/8` on 704; `0/7`'s stack empty at the shore on 719; its cast on 829
/// and barge `0/9` on 830; `0/6` ashore on 1160, `0/8` gone. No
/// `BOARDORDER` or `AWAITBOARDORDER` on any block.
///
/// `GOLDEN_WORD_CHAPTER_TWENTY` carries what stands at the word.
#[test]
fn chapter_twenty_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_holds_to_the_golden_word();
}

/// **Chapter twenty-one, pinned** — the repair line: a player's
/// right-click on a damaged building (`docs/GOLDEN.md` §29, item 813,
/// run255). Twelve staged lines: `!ai off`, `library` 2 and 3, a who=0
/// Barracks on 606, who=1 Bowmen on 610, `peace 1` on 760, four citizens
/// on 770–776, and two issuer lines — `@repair` on `0/6` on 780 and on
/// `0/7 0/8 0/9` on 800.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run255): `process_swarm_around 2007 0 2 13` on 781 and 801; each
/// citizen a `MOVEORDER` to its own ring spot with a `REPAIRORDER` (flags
/// 4) behind it; the Barracks 3 → 1 → 0 on 930–931 under `0/6`; the
/// trio's orders dying on arrival on 968, 969 and 1023.
///
/// `GOLDEN_WORD_CHAPTER_TWENTY_ONE` carries what stands at the word.
#[test]
fn chapter_twenty_one_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_one_holds_to_the_golden_word();
}

/// **Chapter twenty-two, pinned** — the launch line: a strike from inside
/// a base (`docs/GOLDEN.md` §31, item 836, run265). Twelve staged lines:
/// chapter seventeen's eleven and `@strike` on the Fighter `0/6`, inside
/// `0/2007`, on 766.
#[test]
fn chapter_twenty_two_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_two_holds_to_the_golden_word();
}

/// **Chapter twenty-three, pinned** — the repeat line: an Airbase's repeat
/// toggled off between two landings (`docs/GOLDEN.md` §32, item 867,
/// run281). Thirteen staged lines: chapter twenty-two's twelve and
/// `@buildmask` on the Airbase `0/2007` on 1440.
#[test]
fn chapter_twenty_three_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_three_holds_to_the_golden_word();
}

/// **Chapter twenty-four, pinned** — the queue line: a Barracks' infinite
/// queue toggled on between two finishes (`docs/GOLDEN.md` §33, item 877,
/// run285). Eight staged lines: chapter thirteen's four to 614, two
/// `@queueup`s on 620 and 640, and two `@buildmask` 0x40 on 900 and 1300.
#[test]
fn chapter_twenty_four_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_four_holds_to_the_golden_word();
}

/// **Chapter twenty-five, pinned** — the cancel line: a Barracks' queue
/// cancelled inside a run, across two types, on an infinite queue and
/// from the end (`docs/GOLDEN.md` §34, item 884, run292). Thirteen staged
/// lines: chapter thirteen's four to 614, three `@queueup`s, four
/// `@unqueue`s and one `@buildmask` 0x40.
#[test]
fn chapter_twenty_five_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_five_holds_to_the_golden_word();
}

#[test]
fn chapter_twenty_six_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_six_holds_to_the_golden_word();
}

/// **Chapter twenty-seven, pinned** — the upgrade line: a unit upgrade
/// through the player's command and the finish that converts the line
/// (`docs/GOLDEN.md` §36, item 901, run300). Twelve staged lines: `!ai
/// off`, the two cheats, chapter thirteen's cast, the Slingers and five
/// `@queueup`s.
#[test]
fn chapter_twenty_seven_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_seven_holds_to_the_golden_word();
}

/// **run255's repair, unit for unit, both directions** (item 813,
/// `docs/GOLDEN.md` §29). On every block of
/// [`WIDENING_CHAPTER_TWENTY_ONE`], every unit of either player: the
/// builder bit, `unit_masks & 0x400`, which `add_repair_order@005e4ff0`
/// sets and `widen_block` does not read (it compares other bits of the
/// word), against this crate's `was_builder`; and every `REPAIRORDER`'s
/// flags and target, beside a count of them. The Barracks' `damage`,
/// its target and the buckets the price comes out of are the widening's;
/// its `helpers` is 0 on every block of both sides, the step's own
/// counter, and stays unread.
#[test]
fn run255_s_repair_is_the_original_s_unit_for_unit() {
    cases_setup::run255_s_repair_is_the_original_s_unit_for_unit();
}

/// **run235's bombs, record for record** (item 770, `docs/ORDERS.md` §35).
/// run235 is run223's game again — the same seed, script and detail line —
/// with `AMMO=5`, to 1100: every bomb the pair drop on the Barracks is a
/// dumped round. Every live round of either side on every block of
/// [`RUN235_BOMBS`], keyed on its shooter and pool slot, both directions,
/// and every field this crate carries: the release point and height, the
/// landing point and height, the fall time, the target, the flags.
#[test]
fn run235_s_bombs_are_the_original_s_record_for_record() {
    cases_setup::run235_s_bombs_are_the_original_s_record_for_record();
}

/// Every live round of either side on every block of `window`, keyed on
/// its shooter and pool slot, both directions, and every `AMMO` field this
/// crate carries — run235's walk ([`run235_s_bombs_are_the_original_s_record_for_record`])
/// over any `AMMO=5` capture. Returns each key's first parting with the
/// value diff beside it, and the rounds read on each side.
#[allow(clippy::type_complexity)]
fn rounds_record_for_record(
    name: &str,
    script: &str,
    window: (i64, i64),
    show: &[i64],
) -> Option<(
    std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
    usize,
    usize,
)> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut s = stage_script(name, script)?;
    let (first, last) = window;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let (mut theirs_n, mut ours_n) = (0usize, 0usize);
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let theirs: BTreeMap<(i64, i64, i64), super::ammo::Ammo> = super::ammo::blocks(&raw)
            .into_iter()
            .filter(|(a, _)| a.flags & 2 != 0)
            .map(|(a, _)| ((a.who, a.o, a.index), a))
            .collect();
        let sim = &s.built.sim;
        let ident = |o: sim::combat::Obj| match o {
            sim::combat::Obj::Unit(u) => {
                (i64::from(sim.units[u].owner), i64::from(sim.units[u].index))
            }
            sim::combat::Obj::Building(b) => (
                i64::from(sim.buildings[b].owner),
                i64::from(sim.buildings[b].index),
            ),
        };
        let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = sim
            .projectiles
            .iter()
            .map(|p| {
                let (w, o) = ident(p.shooter);
                ((w, o, i64::from(p.slot)), *p)
            })
            .collect();
        theirs_n += theirs.len();
        ours_n += ours.len();
        if show.contains(&n) {
            for (k, a) in &theirs {
                eprintln!("  {name} block {n} theirs {k:?} {a:?}");
            }
            for (k, p) in &ours {
                eprintln!("  {name} block {n} ours   {k:?} {p:?}");
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
            let target = p.target.map_or((-1, -1), ident);
            for (field, mine, dumped) in [
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
                    firsts
                        .entry((who, o, format!("ammo[{slot}].{field}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
    }
    Some((firsts, theirs_n, ours_n))
}

/// **run265's rounds, record for record** (item 853, `docs/ORDERS.md`
/// §39.3): every round the Fighter `0/6` strafes the Barracks with, from
/// its first release on 923 to the capture's end.
#[test]
fn run265_s_rounds_are_the_original_s_record_for_record() {
    cases_setup::run265_s_rounds_are_the_original_s_record_for_record();
}

// **What stands on run265's rounds** (item 853). The Fighter `0/6`'s
// every round agrees whole: 36 rounds, two guns, the walked landing, the
// flight time. Two families of the Bombers' stand, both seams this crate
// names elsewhere:
//
// - a bomb's `accuracy`, 0 here against the slot's last round's there:
//   the bomb's arm writes none (`docs/ORDERS.md` §35.3's SEAM), and since
//   the Fighter's rounds use the pool, the recycled value shows. Only a
//   unit target's hit test reads it, and a bomb's is a building;
// - on 1080 the rounds in flight at the Barracks' death keep it there
//   and lose it here — run235's standing family.
const WANT_RUN265_ROUNDS: &[&str] = &[
    "1000 0/7 ammo[7].accuracy",
    "1003 0/7 ammo[6].accuracy",
    "1061 0/8 ammo[5].accuracy",
    "1064 0/8 ammo[6].accuracy",
    "1069 0/8 ammo[4].accuracy",
    "1072 0/8 ammo[7].accuracy",
    "1080 0/8 ammo[0].ox",
    "1080 0/8 ammo[0].whom",
    "1080 0/8 ammo[1].ox",
    "1080 0/8 ammo[1].whom",
    "1080 0/8 ammo[2].ox",
    "1080 0/8 ammo[2].whom",
    "1080 0/8 ammo[3].ox",
    "1080 0/8 ammo[3].whom",
    "1080 0/8 ammo[4].ox",
    "1080 0/8 ammo[4].whom",
    "1080 0/8 ammo[7].ox",
    "1080 0/8 ammo[7].whom",
    "935 0/8 ammo[2].accuracy",
    "936 0/8 ammo[3].accuracy",
    "939 0/8 ammo[0].accuracy",
    "942 0/8 ammo[1].accuracy",
    "980 0/7 ammo[0].accuracy",
    "981 0/7 ammo[1].accuracy",
    "984 0/7 ammo[2].accuracy",
    "987 0/7 ammo[3].accuracy",
    "995 0/7 ammo[5].accuracy",
];

/// **Chapter ten, pinned** — the patrol line, an issuer the AI never uses
/// (`docs/GOLDEN.md` §18, item 693, run184). Five staged lines: `!ai
/// off`, a Chariot on 610, a Hoplite squad on 612, and two `@patrol`
/// issuer lines on 620 and 640, which the harness runs one frame later.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run184): both issues appended (`INFO 17`, refusal 0, 15 bytes each) and
/// processed on the next frame; one `GroupPatrolOrder` a unit under the
/// leader's attack-move leg; the chariot turning on 761, 903, 1043 and
/// 1185, the squad on 812, 985 and 1155. No falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_TEN` carries what stands at the word.
#[test]
fn chapter_ten_holds_to_the_golden_word() {
    cases_10_19::chapter_ten_holds_to_the_golden_word();
}

/// **Chapter nine, pinned** — the move line, the first issuer chapter
/// (`docs/GOLDEN.md` §17, item 676, run180). Five staged lines: `!ai off`,
/// a Chariot on 610, a Hoplite squad on 612, and two `@move` issuer lines
/// on 620 and 640, which the harness runs one frame later, as the turn
/// pump does.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run180): both issues appended (`INFO 17`, refusal 0) and processed on
/// the next frame; the chariot a plain `MOVEORDER` with an 11-entry plan,
/// the squad three `GroupMoveOrder`s under one id; all four arrive. No
/// falsifier fired.
///
/// `GOLDEN_WORD_CHAPTER_NINE` carries what stands at the word.
#[test]
fn chapter_nine_holds_to_the_golden_word() {
    cases_01_09::chapter_nine_holds_to_the_golden_word();
}

/// **Chapter eight, pinned** — the commanders and a declared war
/// (`docs/GOLDEN.md` §12, item 660, run171). Ten staged lines: `!ai off`,
/// `library 2` for both players, a hoplite squad and a General for who=0, a
/// hoplite squad and a Spy for who=1, then `peace 1`, `war 1` and `ally 1`
/// on 700, 800 and 900.
///
/// **What the capture established before this walk ran** (`docs/RUNS.md`,
/// run171): twelve `INFO cmd` records each returning 1; the diplomacy row
/// moving on 701, 801 and 901, as `chapter8.cmd` predicted; and **`ally`
/// ending the game** — both leaders carry the victory bit on 901, the dump
/// closes after it and the trace's last frame is 900, because
/// `Leader::set_diplo@006ec6a0`'s count of unallied leaders walks
/// `leaders.list[0..8]` and gaia's leaders are 8 and 9.
///
/// `GOLDEN_WORD_CHAPTER_EIGHT` carries what stands at the word.
#[test]
fn chapter_eight_holds_to_the_golden_word() {
    cases_01_09::chapter_eight_holds_to_the_golden_word();
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
    cases_01_09::chapter_two_s_squads_stand_where_the_dump_stands_them();
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
    cases_01_09::chapter_one_s_two_squads_are_seated_where_the_dump_says();
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
    cases_01_09::chapter_one_s_captains_reach_each_other_and_nobody_else();
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
    cases_01_09::chapter_one_s_captain_picks_the_one_it_can_reach();
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
    cases_01_09::chapter_one_s_hit_is_answered_by_the_victim_s_captain();
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
    cases_01_09::chapter_one_s_squad_is_handed_its_captain_s_target();
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
    cases_01_09::chapter_one_s_word_frame_is_widened_whole();
}

/// `near_o`/`near_who` for every `UNITDATA` of one block's raw text, keyed
/// on `(who, o)`. No parser carries the pair (`ch2_dump_near` reads it the
/// same way from a whole file).
fn raw_near(block: &str) -> std::collections::BTreeMap<(i64, i64), (i64, i64)> {
    if std::env::var_os("RON_DISABLE_GOLDEN_READERS").is_some() {
        return raw_near_legacy(block);
    }
    let mut out = std::collections::BTreeMap::new();
    let mut cur: Option<[Option<i64>; 4]> = None;
    let flush = |cur: &mut Option<[Option<i64>; 4]>,
                 out: &mut std::collections::BTreeMap<(i64, i64), (i64, i64)>| {
        if let Some([Some(who), Some(o), Some(no), Some(nw)]) = cur.take() {
            out.insert((who, o), (no, nw));
        }
    };
    // Preserve the raw reader's lexical boundary and first valid value rule,
    // including its treatment of nested fields. Only these four keys escape.
    for line in block.lines() {
        let s = line.trim();
        if s == "BEGIN UNITDATA" {
            flush(&mut cur, &mut out);
            cur = Some([None; 4]);
            continue;
        }
        let Some(r) = cur.as_mut() else { continue };
        let Some((k, v)) = s.rsplit_once(' ') else {
            continue;
        };
        let at = match k {
            "who" => 0,
            "o" => 1,
            "near_o" => 2,
            "near_who" => 3,
            _ => continue,
        };
        if r[at].is_none() {
            r[at] = v.parse().ok();
        }
    }
    flush(&mut cur, &mut out);
    if std::env::var_os("RON_VERIFY_GOLDEN_READERS").is_some() {
        assert_eq!(out, raw_near_legacy(block), "complete near map changed");
    }
    out
}

// Independent pre-optimization reader retained for the explicit migration audit.
fn raw_near_legacy(block: &str) -> std::collections::BTreeMap<(i64, i64), (i64, i64)> {
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

#[test]
fn near_projection_preserves_the_raw_reader_s_lexical_rules() {
    cases_setup::near_projection_preserves_the_raw_reader_s_lexical_rules();
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
    cases_setup::refusing_the_borrowed_frame_stream_does_not_move_great_lakes_word();
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
    cases_01_09::chapter_two_s_first_attack_orders_are_the_dump_s();
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
    cases_01_09::chapter_two_s_hoplite_captain_refused_a_cell_three_searches_reached();
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
    cases_01_09::chapter_two_s_word_frame_is_widened_whole();
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
    cases_01_09::chapter_two_s_near_o_is_the_dump_s_on_every_unit_frame();
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
///
/// **Empty again on item 1089** (`docs/COMBAT.md` §79): both bowmen are
/// followers, and `fight`'s invalid-target arm now takes
/// `find_melee_target`'s squad head, which hands a follower its
/// captain's target with no search. The search was what wrote this
/// crate's `-1`; the original never reached it.
const NEAR_PARTED: &[((i64, i64), i64)] = &[];
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
/// **And by item 1089**, 17113 → 17219 agreeing: total again.
const NEAR_TALLY: (usize, usize, usize) = (17219, 859, 17219);

/// One frame's `near_o`/`near_who` per unit, read out of the raw dump
/// text — [`ch2_dump_units`]'s sibling, for the pair that reader takes
/// only `near_o` of.
fn indexed_ch2_near(
    source: &mut crate::capture::indexed::IndexedCapture,
    frame: i64,
) -> std::collections::BTreeMap<(i64, i64), (i64, i64)> {
    let Some(at) = source
        .frames()
        .iter()
        .position(|range| range.number == frame)
    else {
        return std::collections::BTreeMap::new();
    };
    // Preserve the original reader's first matching frame and all trailing
    // siblings up to the next frame, including the final frame's footer.
    let text = source.read_frame_and_siblings(at).unwrap();
    ch2_dump_near(&text, frame)
}

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
    cases_01_09::chapter_two_s_hit_points_are_the_dump_s_on_every_unit_frame();
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
    cases_01_09::chapter_two_s_frozen_frames_are_the_dump_s_on_every_unit_frame();
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
    cases_01_09::chapter_two_s_visible_byte_is_the_dump_s_on_every_unit_frame();
}

/// Chapter two stood up and not ticked — the dump's path, its text, and
/// the world the siblings' grid makes — for the tests that read the
/// ground rather than the game.
fn chapter_two_stood_up() -> Option<(String, crate::capture::Text, Built)> {
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
    cases_setup::every_object_s_z_is_the_ground_it_stands_on();
}

/// **Chapter two's shots, `v1z` to the last printed digit**
/// (`docs/COMBAT.md` §46.1) — run109's test on the golden record's own
/// 373 `AMMO` blocks, bowmen and slingers both. The rolled `0/8` shot is
/// among them, printed `18.134823` from 677 to 685; its `sz`, `ez` and
/// flight time are integers this crate reproduces, and
/// [`sim::combat::arc_v1z`] over them is that single exactly.
#[test]
fn chapter_two_s_v1z_is_arc_v1z_to_the_last_digit() {
    cases_01_09::chapter_two_s_v1z_is_arc_v1z_to_the_last_digit();
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
    cases_setup::every_rolled_shot_comes_down_where_the_original_s_does();
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
    cases_01_09::chapter_five_s_word_frame_is_widened_whole();
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
    let (built, ix) = build_golden(run, &dump, &loaded, &trace, false)?;
    Some(Staged4 {
        loaded,
        built,
        ix,
        script: script_named(stem),
    })
}

/// Shared bounded construction; draw-word walks require a complete own start
/// block before sibling borrowing, just as their whole-text path did.
fn build_golden(
    run: &str,
    dump: &str,
    loaded: &crate::load::Loaded,
    trace: &crate::trace::Trace,
    require_start: bool,
) -> Option<(Built, crate::capture::indexed::IndexedCapture)> {
    let mut ix = crate::capture::indexed::IndexedCapture::open(dump).unwrap();
    let built = with_sibling_initials(|refs| {
        if refs.is_empty() {
            eprintln!("skipping: no sibling dumps");
            return None;
        }
        ix.with_replay_initial(|init| {
            if require_start {
                require_golden_start(run, &init);
            }
            if std::env::var_os("RON_VERIFY_GOLDEN_SETUP").is_some() {
                let text = crate::capture::read(dump);
                let log = Log::parse(&text);
                let mut expected = log.initial().expect("the golden dump has a BEGIN GAME");
                // Audit bodies are not consumed by build_sim; the bounded
                // replay reader intentionally retains every other field.
                expected.frame_bodies.clear();
                assert!(
                    init == expected,
                    "{run}: bounded setup changed an Initial field"
                );
                eprintln!(
                    "{run}: bounded setup matches every retained Initial field; test={}",
                    std::thread::current().name().unwrap_or("unnamed")
                );
            }
            Some(stand_up_initial(loaded, init, refs, trace))
        })
        .unwrap()
    })?;
    Some((built, ix))
}

fn require_golden_start(run: &str, init: &Initial<'_>) {
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
    assert!(
        !init.leaders.is_empty() && !init.units.is_empty(),
        "golden capture {run} has {} leader(s) and {} unit(s) in its \
         `[Start Game]` block: the capture was taken without a `--detail \
         start:…` line and there is nothing to stand up. Re-take it with \
         run105's set — `--detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,\
         UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1`",
        init.leaders.len(),
        init.units.len()
    );
}

/// **Chapter four's two captures are one game** (item 552). run132 and
/// run133 are the same seed and the same script, windowed on different
/// frames at different detail, and each carries the trace of the whole
/// run whatever its window. So the draw stream must be the same on every
/// frame of the two, and that is what makes the border's window and the
/// bleed's two views of one game rather than two games.
#[test]
fn chapter_four_s_two_captures_are_one_game() {
    cases_01_09::chapter_four_s_two_captures_are_one_game();
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
    cases_01_09::chapter_four_s_border_is_widened_cell_for_cell();
}

/// **Chapter four, walked** — the border and the bleed (`docs/GOLDEN.md`
/// §8, item 552, run133). Eight staged lines: `!ai off`, the Temple, the
/// two techs and the civic level, the squad, the scout and the wagon.
#[test]
fn chapter_four_holds_to_the_golden_word() {
    cases_01_09::chapter_four_holds_to_the_golden_word();
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
    cases_01_09::chapter_four_s_word_frame_is_widened_whole();
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
    cases_01_09::chapter_seven_s_pair_is_one_game_until_the_gate();
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
    cases_01_09::chapter_seven_s_civilians_act_alike_with_the_ai_off_and_on();
}

/// **Chapter seven, walked** — the civilians (`docs/GOLDEN.md` §11, item
/// 578, run141). Seven staged lines: `!ai off`, the library, and the five
/// `add`s. The premise the chapter was designed on did not survive
/// (`docs/INPUT.md` §11.9); what it measures is a human's idle civilians,
/// and the word is the trace's end.
#[test]
fn chapter_seven_holds_to_the_golden_word() {
    cases_01_09::chapter_seven_holds_to_the_golden_word();
}

/// **Chapter seven's control, walked to the same end** (item 578, run142:
/// `chapter7_control.cmd`, six staged lines, the Leader AI on). The
/// control's claim — that the same five act exactly as with the cheat on —
/// is on file only if this crate agrees with it as far as with the
/// chapter, so it is pinned to the same word.
#[test]
fn chapter_seven_s_control_holds_to_the_golden_word() {
    cases_01_09::chapter_seven_s_control_holds_to_the_golden_word();
}

/// **Chapter seven-b's pair is one game until `ai off` is first read**
/// (item 628): run156 is `chapter7b.cmd` and run157
/// `chapter7b_control.cmd`, the same file less `0 !ai off`. As chapter
/// seven's pair, the draw streams agree through frame 0 and part on
/// frame 1, where `Leader::production_ai` first reads the flag.
#[test]
fn chapter_seven_b_s_pair_is_one_game_until_the_gate() {
    cases_01_09::chapter_seven_b_s_pair_is_one_game_until_the_gate();
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
    cases_01_09::chapter_seven_b_s_civilians_act_alike_with_the_ai_off_and_on();
}

/// **Chapter seven-b, walked** — the computer's civilians under the cheat
/// (`docs/GOLDEN.md` §11, item 628, run156). Seven staged lines: `!ai off`,
/// `library who=1 2`, and the five `add`s for who=1 beside London.
#[test]
fn chapter_seven_b_holds_to_the_golden_word() {
    cases_01_09::chapter_seven_b_holds_to_the_golden_word();
}

/// **Chapter seven-b's control, walked to the same end** (item 628,
/// run157: `chapter7b_control.cmd`, six staged lines, the Leader AI on).
#[test]
fn chapter_seven_b_s_control_holds_to_the_golden_word() {
    cases_01_09::chapter_seven_b_s_control_holds_to_the_golden_word();
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

/// **An anti-air building's cycle rows** (item 1112, `docs/COMBAT.md`
/// §84): each `BUILDDATA`'s `recharging`, `attack_ox` and `attack_whom`,
/// keyed on its `SUBOBJECT`'s `(who, o)`. The widening compares them on
/// the buildings whose type carries the `Wall::inc_time` cycle, and the
/// coverage driver runs this reader beside the harness's own.
pub(super) fn cycle_rows(frame: crate::gamelog::Block<'_>) -> Vec<((i64, i64), [i64; 3])> {
    frame
        .kids("BUILDDATA")
        .filter_map(|b| {
            let sub = b.find("SUBOBJECT")?;
            Some((
                (sub.int("who")?, sub.int("o")?),
                [
                    b.int("recharging")?,
                    b.int("attack_ox")?,
                    b.int("attack_whom")?,
                ],
            ))
        })
        .collect()
}

/// **A building's `visible` byte** (item 1423, `docs/GOLDEN.md` §61):
/// `ObjectData::visible`, the mask `Build::do_attack@006228f0` sets after a
/// round and clears on its 32-frame phase with the latch down. Keyed on the
/// `SUBOBJECT`'s `(who, o)`; read off the `WALLDATA`'s `OBJECT`, where no
/// parser carried it until this item (the coverage pin listed it unread).
pub(super) fn visible_rows(frame: crate::gamelog::Block<'_>) -> Vec<((i64, i64), i64)> {
    frame
        .kids("BUILDDATA")
        .filter_map(|b| {
            let obj = b.find("OBJECT")?;
            let sub = obj.find("SUBOBJECT")?;
            Some(((sub.int("who")?, sub.int("o")?), obj.int("visible")?))
        })
        .collect()
}

/// **A pivot figure's turret, every figure, both directions** (item
/// 1117, `docs/COMBAT.md` §55.3): `GUY`'s `turret_angles[4]`,
/// `des_turret_angles[4]`, `node_flags` and `des_node_flags` against
/// `sim::anim::Turret`, on every figure of every unit both sides hold.
/// `widen_block` compares the rest of the record; these four were parsed
/// by nothing until this item (parked 1119), and a pivot piece's release
/// waits on `node_flags`. A figure whose dump does not print them — a
/// capture below `GUYS=4` — is counted, not compared. Returns the rows
/// compared.
fn widen_turrets(
    built: &Built,
    frame: &crate::gamelog::Frame,
    players: usize,
    n: i64,
    firsts: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
) -> usize {
    use crate::diff::compared;
    let mut rows = 0;
    for them in &frame.units {
        if !(0..players as i64).contains(&them.who) {
            continue;
        }
        let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
            continue;
        };
        let Some(u) = built.sim.unit_by_o(w, o) else {
            continue;
        };
        for (k, g) in them.guys.iter().enumerate() {
            let Some(t) = built.sim.units[u].guys.get(k).map(|x| x.turret) else {
                continue;
            };
            let mut here: Vec<(String, i64, i64)> = Vec::new();
            if let Some(v) = g.node_flags {
                compared::note("Guy", &["node_flags"]);
                here.push((format!("g.node_flags[{k}]"), i64::from(t.node_flags), v));
            }
            if let Some(v) = g.des_node_flags {
                compared::note("Guy", &["des_node_flags"]);
                here.push((format!("g.des_node_flags[{k}]"), i64::from(t.des_flags), v));
            }
            if let Some(a) = g.turret_angles {
                compared::note("Guy", &["turret_angles"]);
                for (j, v) in a.into_iter().enumerate() {
                    here.push((format!("g.turret{j}[{k}]"), i64::from(t.angles[j]), v));
                }
            }
            if let Some(a) = g.des_turret_angles {
                compared::note("Guy", &["des_turret_angles"]);
                for (j, v) in a.into_iter().enumerate() {
                    here.push((format!("g.des_turret{j}[{k}]"), i64::from(t.des[j]), v));
                }
            }
            for (what, ours, theirs) in here {
                rows += 1;
                if ours != theirs {
                    firsts
                        .entry((them.who, them.o, what))
                        .or_insert((n, format!("ours {ours} theirs {theirs}")));
                }
            }
        }
    }
    rows
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
    leader_keys: usize,
) -> Option<std::collections::BTreeMap<(i64, i64, String), (i64, String)>> {
    let walk = |reuse| {
        widen_civilians_walk(
            run,
            stem,
            (first, last),
            no_block,
            who,
            print,
            ammo,
            leader_keys,
            reuse,
            None,
        )
    };
    let report = walk(true)?;
    if std::env::var_os("RON_VERIFY_GOLDEN_FRAME_REUSE").is_some() {
        let legacy = walk(false).expect("the same golden fixtures remain available");
        assert_eq!(
            report, legacy,
            "{run}: frame reuse changed the complete golden report"
        );
        eprintln!("{run}: frame reuse matches the complete legacy golden report");
    }
    Some(report.firsts)
}

/// One staged simulation produces both reports for the same run, window and
/// player. The opt-in audit still executes the two independent walks.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn widen_civilians_and_pool(
    run: &str,
    stem: &str,
    window: (i64, i64),
    no_block: i64,
    who: i64,
    print: (i64, i64),
    ammo: bool,
    leader_keys: usize,
) -> Option<(
    std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
    std::collections::BTreeMap<(i64, String), (i64, String)>,
)> {
    let mut pool = PoolWidening {
        who,
        ..Default::default()
    };
    let report = widen_civilians_walk(
        run,
        stem,
        window,
        no_block,
        who,
        print,
        ammo,
        leader_keys,
        true,
        Some(&mut pool),
    )?;
    pool.finish(run);
    if std::env::var_os("RON_VERIFY_SHARED_GOLDEN").is_some()
        || std::env::var_os("RON_VERIFY_GOLDEN_FRAME_REUSE").is_some()
    {
        let legacy = widen_civilians_walk(
            run,
            stem,
            window,
            no_block,
            who,
            print,
            ammo,
            leader_keys,
            false,
            None,
        )
        .expect("the same golden fixtures remain available");
        let legacy_pool = widen_pool(run, stem, window, who)
            .expect("the same golden pool fixture remains available");
        assert_eq!(
            report, legacy,
            "{run}: shared execution changed the complete civilian report"
        );
        assert_eq!(
            pool, legacy_pool,
            "{run}: shared execution changed the complete pool report"
        );
        eprintln!("{run}: shared execution matches both complete independent golden reports");
        if std::env::var_os("RON_VERIFY_GOLDEN_FRAME_REUSE").is_some() {
            eprintln!("{run}: frame reuse matches the complete legacy golden report");
        }
    }
    Some((report.firsts, pool.firsts))
}

/// All widening results, including counts that the callers do not return.
/// The opt-in legacy audit compares these before returning the original map.
#[derive(Debug, PartialEq, Eq)]
struct CivilianWidening {
    firsts: std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
    missing: std::collections::BTreeSet<String>,
    // blocks, record rows, leader rows, goods, unread goods, ammo, no-goods blocks
    counts: [usize; 7],
}

#[allow(clippy::too_many_arguments)]
fn widen_civilians_walk(
    run: &str,
    stem: &str,
    (first, last): (i64, i64),
    no_block: i64,
    who: i64,
    print: (i64, i64),
    ammo: bool,
    leader_keys: usize,
    reuse: bool,
    mut pool: Option<&mut PoolWidening>,
) -> Option<CivilianWidening> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut s = stage_script(run, stem)?;
    let mut store = super::oracle::Store::new(
        reuse
            && std::env::var_os("RON_DISABLE_GOLDEN_ORACLE").is_none()
            && std::env::var_os("RON_DISABLE_GOLDEN_READERS").is_none(),
    )
    .with_civilian(ammo);
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
        crate::diff::harness::debug_ammo(&s.built, n);
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        // One slice and eager tree serve both typed and auxiliary readers.
        // The legacy branch exists only for the opt-in differential audit.
        let cached = store.enabled().then(|| store.read(&mut s.ix, at));
        let auxiliary = cached
            .as_ref()
            .map(|v| v.civilian.as_ref().expect("civilian mode"));
        let reused_raw = (reuse && cached.is_none()).then(|| s.ix.read_frame(at).unwrap());
        let reused_log = reused_raw.as_deref().map(Log::parse_eager);
        let fresh_frame;
        let frame = if let Some(value) = &cached {
            &value.frame
        } else {
            fresh_frame = if let Some(log) = &reused_log {
                crate::capture::indexed::IndexedCapture::frame_from_log(log, n).unwrap()
            } else {
                s.ix.frame_state(at).unwrap()
            };
            &fresh_frame
        };
        blocks += 1;
        let (_, k) = crate::diff::harness::widen_block(&s.built, frame, players, n, &mut firsts);
        rows += k;
        rows += widen_turrets(&s.built, frame, players, n, &mut firsts);
        // `RON_ROW_WALK=who/o,…` — every key that parts on this block for
        // the units named, not only its first parting: a key whose first
        // parting is a birth seat hides everything after it (parked 1046,
        // item 1048).
        if let Ok(spec) = std::env::var("RON_ROW_WALK") {
            let mut here = BTreeMap::new();
            crate::diff::harness::widen_block(&s.built, frame, players, n, &mut here);
            widen_turrets(&s.built, frame, players, n, &mut here);
            for ((w, o, what), (_, row)) in &here {
                if spec.split(',').any(|u| u.trim() == format!("{w}/{o}")) {
                    eprintln!("  walk f{n} {w}/{o} {what}: {row}");
                }
            }
        }
        let legacy_raw = (!reuse).then(|| s.ix.read_frame(at).unwrap());
        let raw = reused_raw.as_deref().or(legacy_raw.as_deref());
        // **`near_o`/`near_who`, every unit, both directions** (item
        // 707): the last search's footprint (`sim::Unit::near`,
        // `docs/COMBAT.md` §37.1), which no parser carries, so the
        // coverage pin listed it as unread on every window here. Chapter
        // eleven's guard is the witness: its `near_o` goes 6 → −1 on
        // the block its attack ends, the one record that says a search
        // ran there and saw nothing.
        let fresh_near;
        let near = if let Some(value) = auxiliary {
            &value.near
        } else {
            fresh_near = raw_near(raw.unwrap());
            &fresh_near
        };
        for them in &frame.units {
            if !(0..players as i64).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let (Some(u), Some(&(no, nw))) =
                (s.built.sim.unit_by_o(w, o), near.get(&(them.who, them.o)))
            else {
                continue;
            };
            let ours = match s.built.sim.units[u].near {
                Some(sim::combat::Obj::Unit(x)) => (
                    i64::from(s.built.sim.units[x].index),
                    i64::from(s.built.sim.units[x].owner),
                ),
                Some(sim::combat::Obj::Building(b)) => (
                    i64::from(s.built.sim.buildings[b].index),
                    i64::from(s.built.sim.buildings[b].owner),
                ),
                None => (-1, -1),
            };
            rows += 1;
            if ours != (no, nw) {
                firsts
                    .entry((them.who, them.o, "near".into()))
                    .or_insert((n, format!("ours {ours:?} theirs {:?}", (no, nw))));
            }
        }
        let legacy_log = (!reuse).then(|| Log::parse(raw.unwrap()));
        let flog = reused_log.as_ref().or(legacy_log.as_ref());
        if let Some(pool) = pool.as_deref_mut() {
            assert_eq!(
                n, s.built.sim.frame,
                "{run}: the pool and civilian clocks differ"
            );
            if let Some(value) = &cached {
                pool.observe_groups(run, &s.built, &value.groups, n);
            } else {
                pool.observe(run, &s.built, flog.unwrap(), n);
            }
        }
        // **An anti-air building's cycle, both directions** (item 1112,
        // `docs/COMBAT.md` §84): `recharging` and `attack_ox/attack_whom`
        // on every building whose type carries the `Wall::inc_time`
        // cycle — no other reader parses the three, and the cycle is
        // nothing but them.
        let fresh_cycles;
        let cycles = if let Some(value) = auxiliary {
            &value.cycles
        } else {
            fresh_cycles = flog
                .unwrap()
                .frames()
                .into_iter()
                .flat_map(|(_, fb)| cycle_rows(fb))
                .collect();
            &fresh_cycles
        };
        {
            for &((w, o), [rech, ox, whom]) in cycles {
                let Some(b) =
                    s.built.sim.buildings.iter().position(|b| {
                        b.alive && i64::from(b.owner) == w && i64::from(b.index) == o
                    })
                else {
                    continue;
                };
                let bd = &s.built.sim.buildings[b];
                let cycles = bd
                    .ty
                    .and_then(|t| s.built.sim.build_types[t].tree)
                    .is_some_and(|id| s.built.sim.tech_tree.types[id].wall_cycle.is_some());
                if !cycles {
                    continue;
                }
                let (to, tw) = match bd.target {
                    Some(sim::combat::Obj::Unit(x)) => (
                        i64::from(s.built.sim.units[x].index),
                        i64::from(s.built.sim.units[x].owner),
                    ),
                    Some(sim::combat::Obj::Building(x)) => (
                        i64::from(s.built.sim.buildings[x].index),
                        i64::from(s.built.sim.buildings[x].owner),
                    ),
                    None => (-1, -1),
                };
                for (what, ours, theirs) in [
                    ("recharging", i64::from(bd.recharging), rech),
                    ("attack_ox", to, ox),
                    ("attack_whom", tw, whom),
                ] {
                    rows += 1;
                    if ours != theirs {
                        firsts
                            .entry((w, o, format!("build:{what}")))
                            .or_insert((n, format!("ours {ours} theirs {theirs}")));
                    }
                }
            }
        }
        // **A building's `visible` byte, every building** (item 1423,
        // `docs/GOLDEN.md` §61): the mask its own round sets and its phase
        // clears, held until now by a unit test alone.
        let fresh_visible;
        let visible = if let Some(value) = auxiliary {
            &value.visible
        } else {
            fresh_visible = flog
                .unwrap()
                .frames()
                .into_iter()
                .flat_map(|(_, fb)| visible_rows(fb))
                .collect();
            &fresh_visible
        };
        {
            for &((w, o), theirs) in visible {
                let Some(b) =
                    s.built.sim.buildings.iter().position(|b| {
                        b.alive && i64::from(b.owner) == w && i64::from(b.index) == o
                    })
                else {
                    continue;
                };
                rows += 1;
                // A nuke's launch sets the silo's byte to `0xff`
                // (`Build::do_missile_launch@00622670`), which the dump
                // prints as −1; that writer lives in `Nukes::shown`.
                let ours = match s.built.sim.nukes.visible_of(b) {
                    0 => i64::from(s.built.sim.buildings[b].visible),
                    v => v,
                };
                if ours != theirs {
                    firsts
                        .entry((w, o, "build:visible".into()))
                        .or_insert((n, format!("ours {ours} theirs {theirs}")));
                }
            }
        }
        // **The fog plane, every cell** (item 1497, `docs/GOLDEN.md` §62):
        // `update_local_seen_build`'s mask — `ever_seen | visible | owner`
        // — is held by a unit test alone until a window prints `WORLD=6`
        // per frame (run770); the other chapters print none and skip.
        let fresh_worlds;
        let worlds = if let Some(value) = auxiliary {
            &value.worlds
        } else {
            fresh_worlds = observations::worlds(flog.unwrap());
            &fresh_worlds
        };
        for world in worlds {
            let theirs = &world.fog;
            if theirs.is_empty() {
                continue;
            }
            let w = &s.built.sim.world;
            let fw = w.fog_xs();
            let mut bad = 0usize;
            let mut first = String::new();
            for (i, &t) in theirs.iter().enumerate() {
                rows += 1;
                let (x, y) = (i as i32 % fw, i as i32 / fw);
                let ours = w.seen2(x, y).unwrap_or(0);
                if ours != t {
                    if bad == 0 {
                        first = format!("cell ({x}, {y}) ours {ours} theirs {t}");
                    }
                    bad += 1;
                }
            }
            if bad > 0 {
                firsts
                    .entry((0, -1, "world:seen2".into()))
                    .or_insert((n, format!("{bad} cells, first {first}")));
            }
            // **And the detect plane beside it** (item 2755, `docs/GOLDEN.md`
            // §102): `seen3`, the inner disc of every `DETECT` object, which
            // `is_detected` reads for a stealth unit (parked 2727).
            let theirs3 = &world.detect;
            let mut bad3 = 0usize;
            let mut first3 = String::new();
            for (i, &t) in theirs3.iter().enumerate() {
                rows += 1;
                let (x, y) = (i as i32 % fw, i as i32 / fw);
                let ours = w.seen3(x, y);
                if ours != t {
                    if bad3 == 0 {
                        first3 = format!("cell ({x}, {y}) ours {ours} theirs {t}");
                    }
                    bad3 += 1;
                }
            }
            // **And the danger planes** (`calc_danger`, every 200th frame,
            // which `is_seen`'s stealth arm weighs): every viewer's row.
            let dg = &world.danger;
            if !dg.is_empty() {
                let reg = dg.len() / 8;
                let mut badd = 0usize;
                let mut firstd = String::new();
                for (i, &t) in dg.iter().enumerate() {
                    rows += 1;
                    let (who, c) = (i / reg, i % reg);
                    let row = w.danger_row(who as u8);
                    let ours = i64::from(row.get(c).copied().unwrap_or(0));
                    if ours != t {
                        if badd == 0 {
                            firstd = format!("who {who} cell {c} ours {ours} theirs {t}");
                        }
                        badd += 1;
                    }
                }
                if badd > 0 {
                    firsts
                        .entry((0, -1, "world:danger".into()))
                        .or_insert((n, format!("{badd} cells, first {firstd}")));
                }
            }
            if bad3 > 0 {
                firsts
                    .entry((0, -1, "world:seen3".into()))
                    .or_insert((n, format!("{bad3} cells, first {first3}")));
            }
        }
        for who in 0..2usize {
            let fresh_leader;
            let t = if let Some(value) = &cached {
                let Some(t) = &value.leaders[who] else {
                    continue;
                };
                t
            } else {
                let Some(block) = flog.unwrap().leader_block(n, who as i64) else {
                    continue;
                };
                fresh_leader = crate::diff::leader::theirs(&block);
                &fresh_leader
            };
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
        let fresh_goods;
        let theirs = if let Some(value) = auxiliary {
            &value.goods
        } else {
            fresh_goods = frame_goods(raw.unwrap())
                .into_iter()
                .map(|g| (g.o, g))
                .collect::<BTreeMap<_, _>>();
            &fresh_goods
        };
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
            // **Every field of the round, not only its presence** (item
            // 1257): chapter forty-four's first round flew 40 frames here
            // against the original's 39, from a launch point 155 units
            // short, and this walk said the two rounds agreed on every
            // block to the word because it compared which rounds were
            // held and nothing else.
            let fresh_ammo;
            let theirs = if let Some(value) = auxiliary {
                value.ammo.as_ref().expect("ammo mode")
            } else {
                fresh_ammo = observations::ammo(raw.unwrap());
                &fresh_ammo
            };
            let sim = &s.built.sim;
            let ours: BTreeMap<(i64, i64, i64), sim::combat::Projectile> = sim
                .projectiles
                .iter()
                .map(|p| {
                    // A building's round too (item 1112): the dump's
                    // `who`/`o` are the building's.
                    let (w, o) = obj_ident(sim, p.shooter);
                    ((w, o, i64::from(p.slot)), *p)
                })
                .collect();
            ammo_rows += theirs.len().max(ours.len());
            let keys: BTreeSet<_> = theirs.keys().chain(ours.keys()).copied().collect();
            for key @ (w, o, slot) in keys {
                let (Some(a), Some(p)) = (theirs.get(&key), ours.get(&key)) else {
                    let side = if theirs.contains_key(&key) {
                        "the dump"
                    } else {
                        "this crate"
                    };
                    firsts
                        .entry((w, o, format!("ammo[{slot}]")))
                        .or_insert((n, format!("{side} holds it alone")));
                    continue;
                };
                for (name, mine, dumped) in ammo_value_rows(sim, p, a) {
                    if mine != dumped {
                        firsts
                            .entry((w, o, format!("ammo[{slot}].{name}")))
                            .or_insert((n, format!("ours {mine} theirs {dumped}")));
                    }
                }
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
         {ammo_rows} rounds, {} keys parted; {} leader keys the dump does not print",
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
        2 * leader_keys * blocks,
        "{run}: the leader rows the dump prints are not compared on every block \
         ({leader_keys} a leader: {LEADERS_TWO_KEYS} at LEADERS=2)"
    );
    assert!(
        good_rows > 0 || no_goods == blocks,
        "{run}: the GOOD list is printed and not read"
    );
    Some(CivilianWidening {
        firsts,
        missing,
        counts: [
            blocks,
            rows,
            leader_rows,
            good_rows,
            unread_goods,
            ammo_rows,
            no_goods,
        ],
    })
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
    cases_01_09::chapter_seven_s_word_frame_is_widened_whole();
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
    cases_01_09::chapter_six_s_word_frame_is_widened_whole();
}

/// **Chapter six-b's word, widened whole, both directions** (item 651).
/// Every record run175 carries on every block of
/// [`WIDENING_CHAPTER_SIX_B`], by [`widen_civilians`] with the `AMMO`
/// record on: every unit and figure, every building — the two Airbases
/// among them, since run175 prints `BUILDS=7` — both leaders at
/// `LEADERS=2`, and every live round.
#[test]
fn chapter_six_b_s_word_frame_is_widened_whole() {
    cases_01_09::chapter_six_b_s_word_frame_is_widened_whole();
}

/// **Chapter eleven's word, widened whole, both directions** (item 696).
/// Every record run190 carries on every block of
/// [`WIDENING_CHAPTER_ELEVEN`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` under its player guard, the wagon `0/7`,
/// the squad `0/8`–`0/10` told to guard a building, and who=1's `1/6`
/// among them — every building, both leaders at `LEADERS=2`. run190 dumps
/// no `AMMO`.
#[test]
fn chapter_eleven_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_eleven_s_word_frame_is_widened_whole();
}

/// **Chapter twelve's word, widened whole, both directions** (item 714).
/// Every record run204 carries on every block of
/// [`WIDENING_CHAPTER_TWELVE`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` following the Supply Wagon `0/7`, the squad
/// `0/8`–`0/10` following the Chariot `0/11` — every building, both
/// leaders at `LEADERS=2`. run204 dumps no `AMMO`.
#[test]
fn chapter_twelve_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_twelve_s_word_frame_is_widened_whole();
}

/// **Chapter thirteen's word, widened whole, both directions** (item
/// 718). Every record run208 carries on every block of
/// [`WIDENING_CHAPTER_THIRTEEN`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` and the squad `0/7`–`0/9` under their
/// player garrisons — every building, the Barracks `0/2007` at `BUILDS=7`
/// among them, both leaders at `LEADERS=2`. run208 dumps no `AMMO`.
#[test]
fn chapter_thirteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_thirteen_s_word_frame_is_widened_whole();
}

/// **Chapter fourteen's word, widened whole, both directions** (item
/// 723). Every record run210 carries on every block of
/// [`WIDENING_CHAPTER_FOURTEEN`], by [`widen_civilians`]: every unit and
/// figure — the three squads `0/6`–`0/14` under their player formation
/// and move — every building, both leaders at `LEADERS=2`. run210 dumps no
/// `AMMO`, and its `GROUPDATA` pool did not come out (`docs/RUNS.md`).
#[test]
fn chapter_fourteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_fourteen_s_word_frame_is_widened_whole();
}

/// **One golden capture's `GROUPDATA` pool, widened whole, both
/// directions** (item 731): on every block of `[first, last)`, each of
/// `who`'s 64 slots that either side holds — the dump's record by its
/// `id − who·64`, this crate's by [`sim::Sim::pool_state`] and
/// [`sim::Sim::pool_list`] — compared on what the slot holds: the member
/// list, the scalars this crate carries (`num`, `form`, `order_num`,
/// `ox`/`oy`, `o_dist`, `o_angle`, `facing`, `form_num`, `speed`,
/// `new_speed`, `stamp`) and each member's `off`, `curr` and `angle`.
/// A slot one side holds alone is a `held` row. The report retains each
/// parted key's first block and row, keyed `(slot, key)`, and all counters.
/// `id` is not compared: this crate numbers a pushed group
/// by its seat (parked 689). `army`, `ox`'s AI fields `priority`, `role`,
/// `think_frame`, `buildings` and `disband` are the AI's and a player's
/// group writes none of them.
#[derive(Debug, Default, PartialEq, Eq)]
struct PoolWidening {
    who: i64,
    firsts: std::collections::BTreeMap<(i64, String), (i64, String)>,
    blocks: usize,
    records: usize,
    rows: usize,
}

impl PoolWidening {
    fn observe(&mut self, run: &str, built: &Built, flog: &Log<'_>, n: i64) {
        let Some((_, block)) = flog.frames().into_iter().find(|(f, _)| *f == n) else {
            return;
        };
        let pool = crate::gamelog::groups(block);
        self.observe_groups(run, built, &pool, n);
    }

    fn observe_groups(
        &mut self,
        run: &str,
        built: &Built,
        pool: &[crate::gamelog::GroupDump],
        n: i64,
    ) {
        assert_eq!(pool.len(), 512, "{run} block {n}: the pool is 512 records");
        self.blocks += 1;
        let w = sim::Player::try_from(self.who).unwrap();
        for slot in 0..64u8 {
            let id = self.who * 64 + i64::from(slot);
            let theirs = pool.iter().find(|g| g.id == id).filter(|g| g.num > 0);
            let mut list = built.sim.pool_list(w, slot);
            let mut ours = built.sim.pool_state(w, slot);
            // A building group (`process_group`'s push at a command on a
            // building, item 882): its buildings are the list.
            if list.is_empty()
                && ours.is_none()
                && let Some((st, b)) = built.sim.pool_building_group(w, slot)
            {
                list = b;
                ours = Some(st);
            }
            if theirs.is_none() && list.is_empty() {
                continue;
            }
            self.records += 1;
            let slot = i64::from(slot);
            let mut row = |key: String, o: String, t: String| {
                self.rows += 1;
                if o != t {
                    self.firsts
                        .entry((slot, key))
                        .or_insert((n, format!("ours {o} theirs {t}")));
                }
            };
            let (Some(t), Some(o)) = (theirs, ours) else {
                row(
                    "held".into(),
                    format!("{list:?}"),
                    format!(
                        "{:?}",
                        theirs.map(|t| t.members.iter().map(|m| m.o).collect::<Vec<_>>())
                    ),
                );
                continue;
            };
            let tl: Vec<i64> = t.members.iter().map(|m| m.o).take(t.num as usize).collect();
            let ol: Vec<i64> = list.iter().map(|&x| i64::from(x)).collect();
            row("list".into(), format!("{ol:?}"), format!("{tl:?}"));
            for (k, ov, tv) in [
                ("num", ol.len() as i64, t.num),
                ("form", i64::from(o.form), t.form),
                ("order_num", i64::from(o.order_num), t.order_num),
                ("ox", i64::from(o.o.x), t.ox),
                ("oy", i64::from(o.o.y), t.oy),
                ("o_dist", i64::from(o.o_dist), t.o_dist),
                ("o_angle", i64::from(o.o_angle.0), t.o_angle),
                ("facing", i64::from(o.facing), t.facing),
                ("form_num", i64::from(o.form_num), t.form_num),
                ("speed", i64::from(o.speed), t.speed),
                ("new_speed", i64::from(o.new_speed), t.new_speed),
                ("stamp", o.stamp, t.stamp),
                ("think_frame", o.opportunity, t.think_frame),
            ] {
                row(k.into(), ov.to_string(), tv.to_string());
            }
            let slots = (o.form_num.max(0) as usize).max(t.form_num.max(0) as usize);
            for i in 0..slots {
                let tm = t.members.get(i);
                row(
                    format!("off[{i}]"),
                    format!("{:?}", o.off.get(i)),
                    format!("{:?}", tm.map(|m| (m.off_x, m.off_y))),
                );
                row(
                    format!("curr[{i}]"),
                    format!("{:?}", o.curr.get(i).map(|p| (p.x, p.y))),
                    format!("{:?}", tm.map(|m| (m.curr_x, m.curr_y))),
                );
                row(
                    format!("angle[{i}]"),
                    format!("{:?}", o.angles.get(i)),
                    format!("{:?}", tm.map(|m| m.angle)),
                );
            }
        }
    }

    fn finish(&self, run: &str) {
        assert!(
            self.blocks > 0,
            "{run}: no block of the pool window was read"
        );
        eprintln!(
            "{run} pool: {} block(s), {} slot record(s), {} row(s), {} key(s) part",
            self.blocks,
            self.records,
            self.rows,
            self.firsts.len()
        );
    }
}

fn widen_pool(run: &str, stem: &str, (first, last): (i64, i64), who: i64) -> Option<PoolWidening> {
    let mut s = stage_script(run, stem)?;
    let mut report = PoolWidening {
        who,
        ..Default::default()
    };
    for _ in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = s.built.sim.frame;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let raw = s.ix.read_frame(at).unwrap();
        let flog = Log::parse(&raw);
        report.observe(run, &s.built, &flog, n);
    }
    report.finish(run);
    Some(report)
}

/// **Chapter fifteen's word, widened whole, both directions** (item
/// 731). Every record run215 carries on every block of
/// [`WIDENING_CHAPTER_FIFTEEN`], by [`widen_civilians`]: every unit and
/// figure — the two squads `0/6`–`0/11` under their player move, attack
/// and attack-move, and the Chariot `1/6` — every building, both leaders
/// at `LEADERS=2`; and who=0's `GROUPDATA` pool by [`widen_pool`], the
/// first golden capture whose pool printed (`GUYS=4`, `docs/RUNS.md`
/// run215). run215 dumps no `AMMO` and no `DEATHS`.
#[test]
fn chapter_fifteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_fifteen_s_word_frame_is_widened_whole();
}

/// **Chapter sixteen's word, widened whole, both directions** (item
/// 738). Every record run219 carries on every block of
/// [`WIDENING_CHAPTER_SIXTEEN`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` and the squad `0/7`–`0/9` under their
/// explores and their box legs — every building, both leaders at
/// `LEADERS=2`; and who=0's `GROUPDATA` pool by [`widen_pool`], where each
/// box leg's pushed group sits. run219 dumps no `AMMO` and no `DEATHS`.
#[test]
fn chapter_sixteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_sixteen_s_word_frame_is_widened_whole();
}

/// **Chapter nineteen's word, widened whole, both directions** (item
/// 790). Every record run245 carries on every block of
/// [`WIDENING_CHAPTER_NINETEEN`], by [`widen_civilians`]: every unit and
/// figure — the Spy `0/6` — every building, the staged Barracks `1/2006`
/// among them, both leaders at `LEADERS=2`; and who=0's `GROUPDATA` pool
/// by [`widen_pool`], where the command's pushed selection sits. run245
/// dumps no `AMMO` and no `DEATHS`.
#[test]
fn chapter_nineteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_nineteen_s_word_frame_is_widened_whole();
}

/// **Chapter twenty's word, widened whole, both directions** (item 803).
/// Every record run249 carries on every block of
/// [`WIDENING_CHAPTER_TWENTY`], by [`widen_civilians`]: every unit and
/// figure — the Chariots `0/6` and `0/7` and the barges `0/8` and `0/9`
/// while they live — every building, the staged Dock among them, both
/// leaders at `LEADERS=2`, whose `leader_flags` carry the transport level;
/// and who=0's `GROUPDATA` pool by [`widen_pool`], where each command's
/// pushed selection sits. run249 dumps no `AMMO` and no `DEATHS`.
#[test]
fn chapter_twenty_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_s_word_frame_is_widened_whole();
}

/// **Chapter twenty-one's word, widened whole, both directions** (item
/// 813). Every record run255 carries on every block of
/// [`WIDENING_CHAPTER_TWENTY_ONE`], by [`widen_civilians`]: every unit and
/// figure — the Bowmen `1/6`..`1/8` and the citizens `0/6`..`0/9` among
/// them — every building, the staged Barracks among them, both leaders at
/// `LEADERS=2`, and every live arrow's `AMMO` record; and who=0's
/// `GROUPDATA` pool by [`widen_pool`], where each command's pushed
/// selection sits.
#[test]
fn chapter_twenty_one_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_one_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run255, the draw word closed at 1300.
// Nothing of the repair line parts: both swarms, the ring spots, the
// `REPAIRORDER`s, the Barracks' `damage` and `helpers`, the price and the
// three late orders dying agree on every block; and since item 824 so
// does everything after it, the idle citizens' gathers and their walks.
// - **The births' `form`** and chapter two's `order:target` on the
//   Bowmen's attack (635), the standing families.
// - **The peace's attack-move**: the Bowmen's `group.id` on 1021, 1020001
//   here against 1026401 there, chapter eight's family at the same frame.
// ~~The idle citizens' gathers: `group` on 1083, 1129 and 1130, and
// `0/7`'s approach one tile west on 1131, the word at 1141~~: a human's
// found gather drops its group (`think_peasant@005f5760`, `5f590c`), so
// `find_ordered_collision`'s own-group arm no longer refuses `0/7` the
// spot `0/8` was sent to (item 824, `docs/GOLDEN.md` §29).
const WANT_CH21: &[&str] = &[
    // Item 1502: an attack's `order:target` on a building the start dump did
    // not link is named by `(owner, index)` (parked 681; `docs/AI.md`
    // §131.4); these rows agree and leave: 635 1/6 order:target, 635 1/7
    // order:target, 635 1/8 order:target.
];
// **What parts in the pool**: the pushed selections' `ox`/`oy` on 782 and
// 802 (chapter seventeen's family) and slot 1's `speed`/`new_speed` on 802
// (chapter twenty's, the 642 move's), both before the repair. ~~The idle
// gathers' groups past the word~~ (1086, 1090, 1153, 1217): item 824.
const WANT_CH21_POOL: &[&str] = &[];

// **What parts under the word** on run249, the draw word closed at 1300.
// - The births' `form`, the standing family: the Chariots and the barges.
// - ~~**The passenger's figures on boarding**: `avg_speed` 20 here against
//   15 there on `0/6` (704) and 18 against 13 on `0/7` (830) — the
//   original ages it once more on the boarding frame (27 → 20 → 15, a
//   quarter a frame) before the figures stand still inside; the order of
//   `Guy::move` against `go_inside` in the frame, not this item's module.~~
//   Item 850: `Unit::process` runs `Guy::process` after the work whether
//   or not the work put the unit inside (`docs/ANIM.md` §15). What is left
//   is `0/6` on 1160, its disembark: ours seats its figures at 0, the
//   original keeps the 15.
// - **The disembarked figure**: on 1162 `0/6`'s animation, `cur_anim` 7
//   here against 8 and `end_time` 89 against 22; on 1165 its facing,
//   −1581692416 here against −1518063274 (`crate::anim` and
//   `crate::movement`, fenced to this item).
// ~~On 1160 `0/6`'s `orders_x/y` and `dest_angle`~~: `eject_contents`'s
// `update_action` is step 1's, before the orders come back (item 803).
const WANT_CH20: &[&str] = &[
    // Item 850: a unit that goes inside in its own work takes its figures'
    // frame (`docs/ANIM.md` §15). The Chariots' `g.avg_speed`, 20 against
    // 15 (`0/6`, from 704) and 18 against 13 (`0/7`, from 830), stood from
    // each one's boarding frame; `0/7`'s agrees now, and `0/6`'s parted on
    // its disembark, 1160, ours 0 against 15, and its turn after — until
    // item 1164 put it ashore at the average it boarded with
    // (`docs/TRANSPORT.md` §6.4): ten rows, all agreeing.
    "704 0/8 form",
    "830 0/9 form",
];
// **What parts in the pool**: the 620 toggle's pushed selection's
// `ox`/`oy`, (0, 0) there and (−1, −1) here, chapter nineteen's standing
// family; and the 642 move's group, slot 0, whose `speed`/`new_speed` are
// 30 there and 0 here on 644 (`crate::group`'s, fenced to this item).
const WANT_CH20_POOL: &[&str] = &[];

/// The first occurrence of every integer key in each `UNITDATA` and
/// `BUILDDATA` record of one block, keyed on `(kind, who, o)` — the raw
/// read [`raw_near`] makes, for fields no parser carries.
fn raw_records(
    block: &str,
) -> std::collections::BTreeMap<(&'static str, i64, i64), std::collections::BTreeMap<String, i64>> {
    use std::collections::BTreeMap;
    let mut out = BTreeMap::new();
    let mut cur: Option<(&'static str, BTreeMap<String, i64>)> = None;
    let flush =
        |cur: &mut Option<(&'static str, BTreeMap<String, i64>)>,
         out: &mut BTreeMap<(&'static str, i64, i64), BTreeMap<String, i64>>| {
            if let Some((kind, r)) = cur.take()
                && let (Some(&who), Some(&o)) = (r.get("who"), r.get("o"))
            {
                out.insert((kind, who, o), r);
            }
        };
    for line in block.lines() {
        let s = line.trim();
        let top = match s {
            "BEGIN UNITDATA" => Some("UNITDATA"),
            "BEGIN BUILDDATA" => Some("BUILDDATA"),
            _ => None,
        };
        if let Some(kind) = top {
            flush(&mut cur, &mut out);
            cur = Some((kind, BTreeMap::new()));
            continue;
        }
        if s.starts_with("BEGIN ") {
            continue;
        }
        let Some((_, r)) = cur.as_mut() else { continue };
        if let Some((k, v)) = s.rsplit_once(' ')
            && let Ok(n) = v.parse::<i64>()
        {
            r.entry(k.to_string()).or_insert(n);
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// **run245's cast, field for field, both directions** (item 790,
/// `docs/GOLDEN.md` §27). The fields the cast writes that no parser
/// carries, read raw from every block of [`WIDENING_CHAPTER_NINETEEN`]:
/// on every unit of either player, `mana_burn`, `spell_time`, the target
/// `cavarch_o`/`cavarch_who`, the started bit `unit_masks & 0x20000` and
/// `visible`; on every building, `infiltrated`; and on every `CASTORDER`
/// either side holds, its target, its point, its craft and `paid`.
///
/// Not compared: `cavarch_uid` and the order's `uid`, since this crate
/// carries no uid; the unit's `flags & 0x80`, which it does not carry.
#[test]
fn run245_s_cast_is_the_original_s_field_for_field() {
    cases_setup::run245_s_cast_is_the_original_s_field_for_field();
}

/// **run265's launch and tank, field for field, both directions** (item
/// 836, `docs/GOLDEN.md` §31, `docs/ORDERS.md` §38). The fields the launch
/// line writes that no parser carries, read raw from every block of
/// [`WIDENING_CHAPTER_TWENTY_TWO`]: on every unit of either player, the
/// tank `mana_burn` and whether it stands inside (`inside_up ≥ 0`); on
/// every building, `launch_frames`. Everything agrees to the word and
/// past it, 39,793 rows. ~~What parts is the Fighter's flight home, four
/// blocks early from the climb's drift at 798 (the word's own cause).~~
/// Nothing parts since item 842 built the strafer's half altitude. `mana_burn` and `launch_frames` stay
/// pinned unread in `diff::coverage` for the widenings, which the long
/// captures share.
#[test]
fn run265_s_launch_is_the_original_s_field_for_field() {
    cases_setup::run265_s_launch_is_the_original_s_field_for_field();
}

/// **run249's transport, unit for unit, both directions** (item 803,
/// `docs/GOLDEN.md` §28). The state the toggle writes and the board reads,
/// which [`crate::diff::harness::widen_block`] does not compare — it reads
/// `unit_masks` for the packed and half-step bits only, and the long
/// captures' widenings share it — read raw from every block of
/// [`WIDENING_CHAPTER_TWENTY`]: on every unit of either player the
/// auto-transport bit, `unit_masks & 0x800000`, against
/// [`sim::Unit::auto_transport`]; on each leader the transport level,
/// `leader_flags & 0x700`, against [`sim::Sim::transport`]; on every
/// `CASTORDER` either side holds its craft and its `UNITORDER` flags; and
/// no `BOARDORDER` or `AWAITBOARDORDER` on the original's side, which this
/// crate cannot hold.
#[test]
fn run249_s_transport_is_the_original_s_unit_for_unit() {
    cases_setup::run249_s_transport_is_the_original_s_unit_for_unit();
}

// **What parts under the word**, by block and key (`docs/GOLDEN.md` §27).
// ~~The first pin, word 669, open~~ (605, 672), the harness skipping
// `@spell`: the Spy's stack on 622, its first step on 623. **The spell
// command entered, 1100, closed** (605, 1101): past the births only the
// Spy's birth `form` stands. Its figure's `stopped` on 756 went with
// `Guy::set_angle`'s third argument (`crate::cast`).
const WANT_CH19: &[&str] = &[];
// **What parts in the pool**: the pushed selection's `ox`/`oy`, (0, 0)
// there and (−1, −1) here, chapter seventeen's and eighteen's standing
// family: a point no craft reads.
const WANT_CH19_POOL: &[&str] = &[];

/// **Chapter eighteen's word, widened whole, both directions** (item
/// 779). Every record run241 carries on every block of
/// [`WIDENING_CHAPTER_EIGHTEEN`], by [`widen_civilians`]: every unit and
/// figure — the lone builder `0/6` and the three `0/7`–`0/9` — every
/// building, the two staged sites among them, both leaders at
/// `LEADERS=2`, whose buckets carry the price; and who=0's `GROUPDATA`
/// pool by [`widen_pool`], where each command's pushed selection sits.
/// run241 dumps no `AMMO` and no `DEATHS`.
#[test]
fn chapter_eighteen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_eighteen_s_word_frame_is_widened_whole();
}

/// **Chapter seventeen's word, widened whole, both directions** (item
/// 746). Every record run223 carries on every block of
/// [`WIDENING_CHAPTER_SEVENTEEN`], by [`widen_civilians`]: every unit and
/// figure — the Fighter `0/6` on its strafe home, the Bomber pair `0/7`,
/// `0/8` on their pads — every building, the two staged ones among them,
/// both leaders at `LEADERS=2`; and who=0's `GROUPDATA` pool by
/// [`widen_pool`], where each command's pushed selection sits. run223
/// dumps no `AMMO` and no `DEATHS`.
/// **run265 whole, both directions** (item 836, `docs/GOLDEN.md` §31):
/// every dumped record of either player on every block of
/// [`WIDENING_CHAPTER_TWENTY_TWO`], and who=0's pool.
#[test]
fn chapter_twenty_two_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_two_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run265, chapter twenty-two. ~~The
// first pin, word 778: `0/6`'s stack on 768 (no strike here against the
// original's `STRAFEORDER` laid inside) and the launch's 13 keys on 778.~~
// **The launch line built** (item 836, `docs/ORDERS.md` §38): 768 and 778
// agree whole — the strike laid inside and waiting on the tank, the EXIT
// onto (11424, 13920) with its figure on the ground, `last_bank` and
// `last_pitch` 0.0, `avg_speed` kept at 25, and the first step. **Word
// 923, open.** The first value parting is **798**, the climb: `0/6`'s
// `pitch` 40.0 here against 38.0 there (bits 1109393408 / 1108869120),
// `last_speed` 38 against 40, its point a unit or two off; the heading
// parts on 813 (1390007340 / 1391742680), and on 924 the original's
// Fighter is in its attack (`cur_anim` 12, `recharging` 31) where ours is
// not. From 924 the one-draw shift in the citizens' and scouts' legs.
// The births' `form` and chapter one's clocks stand as in chapter
// seventeen (611–655), and the pair's tank rows are gone with the tank.
// **Item 842, closed at 1500** (`docs/ORDERS.md` §39): the Fighter strafes
// (`w`), so its strike wants half its `cruising_alt` and 798 agrees
// (`pitch` 38.0 both sides), and its round is exact, so 923 spends no
// scatter. 315 rows → 15. What stands: the births' (611–655); ~~the
// Barracks' `damage` on 928, 571 + 8/16 here against 572 + 0, the
// second gun and the landing's walk~~ (item 853, §39.3: `FIGHTER` fires
// both guns and `Ammo::init` walks a strafer's landing; 928's `damage`
// is 572 + 0 on both sides, and every round is the dump's,
// [`run265_s_rounds_are_the_original_s_record_for_record`]; 15 → 13);
// ~~and the landed patrols on 1385 and 1489, one order there and none
// here~~ (item 854, `docs/ORDERS.md` §40: the Airbase's `build_masks &
// 0x80`, set by `Build::init`, keeps a landing plane's order with the
// action bit cleared; on 1385 `0/6` and on 1489 `0/7` each hold one
// `AIRPATROLORDER`, flags 0, `oxx` 2007, point (21120, 16512),
// `returning` 0, `cruising_alt` 1400 and 1600, on both sides; 13 → 9).
const WANT_CH22: &[&str] = &[];

// **What parts in the pool** on run265: chapter seventeen's ten, the
// pushed flight groups' point (parked 760) and item 738's `get_num` speeds;
// the Fighter's selection on 767 agrees.
const WANT_CH22_POOL: &[&str] = &[];

/// **run281 whole, both directions** (item 867, `docs/GOLDEN.md` §32):
/// every dumped record of either player on every block of
/// [`WIDENING_CHAPTER_TWENTY_THREE`], and who=0's pool.
#[test]
fn chapter_twenty_three_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_three_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run281, chapter twenty-three. ~~The
// first walk, with the `@buildmask` line skipped: the toggle on 1442,
// `0/2007`'s `build:repeat_air` ours 1 theirs 0 (`build_masks` 4232 →
// 4104 there), and the two landings off the non-repeating base, `0/7` on
// 1489 and `0/8` on 1513, each with its `AIRPATROLORDER` kept here against
// none there.~~ **The command entered** (item 867,
// [`crate::input::group_buildmask`] → `Sim::action_buildmask`): 1442,
// 1489 and 1513 agree, 14 → 9 rows. What stands is chapter twenty-two's:
// the births' and chapter one's clocks (611–655). 1585 agrees on both
// walks: this crate's `do_launch` kills `0/6`'s unflagged patrol, and
// under the cleared bit so does the original's.
const WANT_CH23: &[&str] = &[];

// **What parts in the pool** on run281: chapter twenty-two's ten. ~~And the
// building group `[2007]` the original's `process_group` pushes into
// who=0's slot 0 on 1442 (`buildings 1`, stamp 1441), which this crate
// does not seat.~~ Seated since item 882
// ([`crate::input::group_buildmask`] → `Sim::push_command_buildings`),
// and slot 0 agrees on 1442 whole: its `ox`/`oy` first part on 642.
const WANT_CH23_POOL: &[&str] = &[];

/// **run285 whole, both directions** (item 877, `docs/GOLDEN.md` §33):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_four_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_four_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run285, chapter twenty-four. ~~The
// first walk, with both `@queueup` lines skipped and `@buildmask` 0x40
// writing nothing: on 622 the Hoplites' queue-up, `0/2007`'s `queued`
// and the food and timber it paid; on 642 the Bowmen's wealth; on 856 the
// Hoplites the original trained.~~ **The commands entered** (item 877,
// [`crate::input::group_queue_up`] → `Sim::action_queue_up`, the 0x40
// toggle and `do_queue`'s re-queue): 622, 642 and 856's `unlinked` agree,
// and the three trained squads are born on 856, 1060 and 1272 on both
// sides. ~~What stands is chapter thirteen's births' `form` (611, 615), and
// on each trained squad, `Unit::come_out@00617c10`'s human arm: … `group`
// (the pool slot: 0, 2, 1), `form` 0, and the followers' `orders_x/y`.~~
// **The push built** (item 882, `docs/GOLDEN.md` §33): `come_out` pushes
// every trained squad — `618900`..`6189aa` test the captain, a building
// host and `uber_size` (`+0x308`) > 1, and no owner — and the command's
// `process_group` seats the Barracks' group, so the squads' `group` reads
// 0, 2 and 1 on both sides. The listing killed the "human arm". What
// stands is **`Unit::init@00612100`'s**, a birth mechanism and not the
// push (parked 646): `form` 0 on every non-citizen it makes (611, 615,
// and the three squads), ours −1; and the followers' `orders_x/y`, the
// tile-centred point it inits a unit at, (2712, 14232) there against the
// Barracks' point (2688, 14208) here — kept until the follower's own next
// `work` on the block after, where both sides agree. The captain's is
// rewritten by `come_out`'s `update_action`. **Readers**: `form` by the
// next `Group::action_move_near`/`action_form` layout (none in run285),
// `orders_x` by `Unit::check_target_path@005e22d0` (nothing targets
// these squads).
const WANT_CH24: &[&str] = &[];

// **What parts in the pool** on run285. ~~All pushes this crate does not
// seat: the building group `[2007]` in slot 1 on 622 and slot 3 on 1302,
// the squads in slot 0 on 856 and slot 2 on 1060.~~ **Both pushes built**
// (item 882): the building group `[2007]` (`process_group`, at each
// command) and each trained squad (`come_out`) are seated, and every
// slot holds the original's list, `num`, `stamp`, `speed` and
// `new_speed` on every block — the third squad takes slot 1 back on 1272
// because `Groups::get_open_slot@006fa460` counts the building group's
// slot as open, and the second press's group lands in slot 3. What stands
// is each slot's `ox`/`oy` on its first seat, ours −1 and theirs 0:
// `push_group`'s record `o`, which this crate seats as −1 (the pool
// record's, att-880's fence). Its reader is a layout's `o` — none here.
const WANT_CH24_POOL: &[&str] = &[];

/// **run292 whole, both directions** (item 884, `docs/GOLDEN.md` §34):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_five_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_five_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run292, chapter twenty-five. ~~The
// first walk, with the four `@unqueue` lines skipped: 702's `queued` and
// food and timber, 842's `build:infinite_queue`, and on 856 the Hoplites
// this crate trained from the uncancelled head.~~ **The cancel entered**
// (item 884, [`crate::input::unqueue`] → `Sim::action_unqueue`): every
// arm's block agrees — 702, 762, 842, 965 and 1002 — and the two trained
// squads are born on 965 and 1216 on both sides. What stands is
// **`Unit::init@00612100`'s** (parked 646), chapter twenty-four's shape
// exactly: `form` 0 on every non-citizen it makes (611, 615, and each
// squad), ours −1; and the followers' `orders_x/y`, the tile-centred
// point (2712, 14232) there against the Barracks' (2688, 14208) here.
// **Readers**: `form` by the next `Group::action_move_near`/`action_form`
// layout (none in run292), `orders_x` by `Unit::check_target_path@
// 005e22d0` (nothing targets these squads).
const WANT_CH25: &[&str] = &[];

// **What parts in the pool** on run292. ~~The first walk: 622's record
// `o` and the squad this crate trained alone on 856.~~ **The cancel
// entered**: every slot's list, `num` and `stamp` agree on every block,
// and no cancel seats or re-stamps a slot on either side (falsifier 5).
// What stands: each slot's `ox`/`oy` on its first seat (622, 965, 1216),
// ours −1 and theirs 0 — `push_group`'s record `o`, chapter twenty-four's
// standing row (parked 887, the pool lane's; its reader is a layout's `o`,
// and nothing is laid out). And **982 slot 0's `speed`/`new_speed`**, ours
// 0 and theirs 26: the 981 `@queueup`'s building-group push runs
// `Groups::get_open_slot@006fa460`, which calls vslot 8 —
// `Group::get_num_cap@007145c0` — on each slot whose stamp is not newer
// than the best so far, and `get_num_cap` `normalize`s a group of fewer
// than four: the Bowmen squad (num 3, stamp 964) takes its leader's 26.
// This crate's `open_slot` has no such side effect (the pool lane's code,
// not built here). Its reader is `UnitData::get_speed@00608720`'s group
// cap on a moving member, and the squad does not move in run292.
const WANT_CH25_POOL: &[&str] = &[];

/// **run296 whole, both directions** (item 883, `docs/GOLDEN.md` §35):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_six_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_six_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run296, chapter twenty-six. ~~The
// first walk, with a technology's `@queueup` skipped: every research's
// entry, price and finish — 622's `queued` and food, 652's timber and
// wealth, 822's `epochs`, `epoch[0]` and `discovered`, 1023's `epoch[3]`,
// 1242's `epoch[2]`, and 1243's commerce cap, 1120 against 1600.~~ **The
// research entered** (item 883, [`crate::input::group_queue_up`] →
// `Sim::action_queue_research`): every research's block agrees — 622,
// 642, 652, 822, 852, 1023, 1042, 1062 and 1242, the counters and the cap
// with them. What stands is **`Unit::init@00612100`'s** (parked 646),
// chapter twenty-five's shape: `form` on every non-citizen it makes (611,
// 615, the Hoplites on 1106) and the followers' tile-centred `orders_x/y`.
// ~~And **1023 `0/0` `mylos`**, ours 6 and theirs 4~~: Written Word's
// `SCIENCE_LOS`, which this crate computed on read and the original caches
// in `Unit::update_los@0060e4d0`, refreshed a block after the gain. It
// agrees since item 1354 made `mylos` the cache (`docs/VISION.md` §2).
const WANT_CH26: &[&str] = &[];

// **What parts in the pool** on run296. ~~The first walk: the Library's
// building group `[2005]` in slot 1 from 622 there and not here, so the
// Barracks' group took slot 1 on 872 here and slot 0 there.~~ **The
// research entered**: the Library's group is seated by its first command,
// and every slot's list, `num` and `stamp` agree on every block. What
// stands is each first seat's `ox`/`oy`, ours −1 and theirs 0 —
// `push_group`'s record `o` (parked 887; its reader is a layout's `o`, and
// nothing is laid out).
const WANT_CH26_POOL: &[&str] = &[];

/// **run300 whole, both directions** (item 901, `docs/GOLDEN.md` §36):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_seven_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_seven_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run300, chapter twenty-seven. ~~The
// first walk over (605, 1105): 922, `0/2007`'s `queue[0].type`, ours 82
// and theirs 83 — `Leader::gain_tech@006dcb60`'s queue loop re-targets the
// Slingers entry in place there, and this crate had no queue loop; then
// 1103, the Slinger squad it trained on 1102 and `queued` 1 against 2.~~
// **The queue loop built** (item 901, `Sim::retarget_queued_to`): run300
// whole agrees on every queue, bucket, counter and unit type — 622, 642,
// 652, 922, 1002, 1012 and both births. What stands is
// **`Unit::init@00612100`'s** (parked 646), chapter twenty-six's shape:
// `form` on every non-citizen it makes (611, 615, 617, and the squads on
// 1111 and 1307) and the followers' tile-centred `orders_x/y`.
const WANT_CH27: &[&str] = &[];

// **What parts in the pool** on run300. ~~The first walk: the extra
// Slinger squad's seat on 1103.~~ **The queue loop built**: what stands is
// each first seat's `ox`/`oy`, ours −1 and theirs 0 — `push_group`'s
// record `o` (parked 887) — the Barracks' building group on 622 and the
// first Javelineers squad on 1111.
const WANT_CH27_POOL: &[&str] = &[];

/// **Chapter twenty-eight, pinned** — two buildings under one command
/// (`docs/GOLDEN.md` §37, item 888, run304). Eleven staged lines: `!ai
/// off`, chapter thirteen's cast with a second Barracks, three
/// `@queueup`s on one or both, two `@buildmask`s, and the two refused
/// presses.
#[test]
fn chapter_twenty_eight_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_eight_holds_to_the_golden_word();
}

/// **run304 whole, both directions** (item 888, `docs/GOLDEN.md` §37):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_eight_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_eight_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run304, chapter twenty-eight. ~~The
// first walk, the two-building seat not built: 825's `group`, the Bowmen
// `0/10`..`0/12` in slot 1 here and 0 there, and 876's Hoplites in 0
// against 1.~~ **The seat built** (item 888, `Sim::push_buildings_group`):
// every `group` agrees, as every queue, bucket and `build_masks` did on
// the first walk. What stands is `Unit::init@00612100`'s (parked 646):
// `form` on every birth and the followers' tile-centred `orders_x/y`.
const WANT_CH28: &[&str] = &[];

// **What parts in the pool** on run304. ~~The first walk: 642, slot 0
// `held`, theirs `[2007, 2008]` and ours empty, and every seat after it
// one slot off.~~ **The seat built**: every slot's list, `num`, `stamp`,
// `speed` and `new_speed` agree on every block. What stands is each first
// seat's `ox`/`oy`, ours −1 and theirs 0 — `push_group`'s record `o`
// (parked 887).
const WANT_CH28_POOL: &[&str] = &[];

/// **Chapter twenty-nine, pinned** — the repeat launch (`docs/GOLDEN.md`
/// §38, item 915, run308). Thirteen staged lines: chapter twenty-two's
/// twelve and `@queueup` of a Biplane at the Airbase `0/2007` on 1540.
#[test]
fn chapter_twenty_nine_holds_to_the_golden_word() {
    cases_20_29::chapter_twenty_nine_holds_to_the_golden_word();
}

/// **run308 whole, both directions** (item 915, `docs/GOLDEN.md` §38):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_twenty_nine_s_word_frame_is_widened_whole() {
    cases_20_29::chapter_twenty_nine_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run308, chapter twenty-nine. ~~The first
// walk, `Build::train`'s `CARRY_AIR` arm not built: on 1746 the Biplane
// `0/9` out on the EXIT's point here, idling, and inside `0/2007` there
// (`inside` −1 against 2007, `idle` 1 against 0); its idle's draw parted
// the stream on 1745's tick, and 262 rows stood.~~ **The arm built** (item
// 915): `inside`, `idle`, the angles, `z` and every row after 1746 agree,
// 262 → 17. What stands is chapter twenty-two's (611–655) and the
// Biplane's birth on 1746: `form` and its point — ours the base's own
// (11616, 13920), theirs the quarter-tile centre (11640, 13944) —
// `Unit::init@00612100`'s seat (parked 646), the point's one writer; the
// plane never leaves the base, so nothing reads it on the run.
const WANT_CH29: &[&str] = &[];

// **What parts in the pool** on run308: chapter twenty-two's ten. The
// press's building group `[2007]` is seated by both sides on 1542.
const WANT_CH29_POOL: &[&str] = &[];

#[test]
fn chapter_seventeen_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_seventeen_s_word_frame_is_widened_whole();
}

/// **run223's air orders, read whole** (item 746, `docs/ORDERS.md` §11.1
/// and §32) — the first `STRAFEORDER` and the first player's
/// `AIRPATROLORDER` on disk, each with its `AIRORDER` base, pinned as the
/// parser reads them. On 642 the Fighter's flight home: target −1,
/// `mandatory 1`, home `0/2007`, `cruising_alt` 1600, `sharp_turn` and
/// `old` 0, `returning 1`, `xx/yy −1`. On 666 a bomber's strike turned
/// patrol: the Barracks' point in both arrays, the same home, `returning
/// 0`. A second reader beside `compare_orders`, so a parser change that
/// drops a field fails here as well as there.
#[test]
fn run223_s_air_orders_are_read_whole() {
    cases_setup::run223_s_air_orders_are_read_whole();
}

/// **Chapter ten's word, widened whole, both directions** (item 693).
/// Every record run184 carries on every block of
/// [`WIDENING_CHAPTER_TEN`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` and the squad `0/7`–`0/9` under their
/// player patrols among them — every building, both leaders at
/// `LEADERS=2`. run184 dumps no `AMMO`.
#[test]
fn chapter_ten_s_word_frame_is_widened_whole() {
    cases_10_19::chapter_ten_s_word_frame_is_widened_whole();
}

/// **Chapter nine's word, widened whole, both directions** (item 676).
/// Every record run180 carries on every block of
/// [`WIDENING_CHAPTER_NINE`], by [`widen_civilians`]: every unit and
/// figure — the Chariot `0/6` and the squad `0/7`–`0/9` under their
/// player orders among them — every building, both leaders at
/// `LEADERS=2`. run180 dumps no `AMMO`. The four staged units are printed
/// both sides on the word's blocks.
#[test]
fn chapter_nine_s_word_frame_is_widened_whole() {
    cases_01_09::chapter_nine_s_word_frame_is_widened_whole();
}

/// **Chapter eight's word, widened whole, both directions** (item 660).
/// Every record run171 carries on every block of
/// [`WIDENING_CHAPTER_EIGHT`], by [`widen_civilians`] with the `AMMO`
/// record on: every unit and figure, both leaders at `LEADERS=5` — whose
/// `diplos[·]` row is what this chapter reads — and every live round. The
/// eight staged units are printed both sides on the word's two blocks.
#[test]
fn chapter_eight_s_word_frame_is_widened_whole() {
    cases_01_09::chapter_eight_s_word_frame_is_widened_whole();
}

/// **Chapter seven-b's words, widened whole, both directions, on both
/// captures** (item 628). Every record run156 and run157 carry on every
/// block of [`WIDENING_CHAPTER_SEVEN_B`], by [`widen_civilians`].
#[test]
fn chapter_seven_b_s_word_frame_is_widened_whole() {
    cases_01_09::chapter_seven_b_s_word_frame_is_widened_whole();
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
    cases_01_09::chapter_three_holds_to_the_golden_word();
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
        rows += widen_turrets(&s.built, &frame, players, n, &mut blk);
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
            let z = i64::from(s.built.sim.world.object_z(un.pos.tile()));
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
        2 * LEADERS_TWO_KEYS * blocks,
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
/// the wrong field, not a parting. A crash round carries its roll and its
/// bank pair since item 2609 (`docs/AI.md` §228), printed in millionths as
/// the dump's singles are; every other round's side is the zero `init`
/// writes.
/// An object's `(who, o)` as the dump prints it: a unit's owner and
/// index, or a building's.
pub(super) fn obj_ident(sim: &sim::Sim, o: sim::combat::Obj) -> (i64, i64) {
    match o {
        sim::combat::Obj::Unit(u) => (i64::from(sim.units[u].owner), i64::from(sim.units[u].index)),
        sim::combat::Obj::Building(b) => (
            i64::from(sim.buildings[b].owner),
            i64::from(sim.buildings[b].index),
        ),
    }
}

/// **Every field of `AmmoData::log_data` this crate carries**, ours
/// against the dump's, for one round held on both sides: the clock, the
/// launch and landing, the target, the accuracy, the splash, the heights,
/// the bearing, the arc and the flag bits. `traj`, `dx`, the roll and
/// bank angles, `gpiece` and `graph_index` are not carried.
pub(super) fn ammo_value_rows(
    sim: &sim::Sim,
    p: &sim::combat::Projectile,
    a: &super::ammo::Ammo,
) -> Vec<(&'static str, i64, i64)> {
    let target = p.target.map_or((-1, -1), |t| obj_ident(sim, t));
    [
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
    .collect()
}

fn ammo_flag_rows(
    p: &sim::combat::Projectile,
    a: &super::ammo::Ammo,
) -> [(&'static str, i64, i64); 6] {
    let (roll, bank) = p.crash.map_or((0, (0, 0)), |c| (c.rolling, c.bank));
    [
        ("flags&4", if p.rolling { 4 } else { 0 }, a.flags & 4),
        ("flags&8", if p.missed { 8 } else { 0 }, a.flags & 8),
        // `Ammo::init`'s bit `0x10`, a `do_damage="0"` round (item 853,
        // `docs/ORDERS.md` §39.5).
        ("flags&16", if p.harmless { 16 } else { 0 }, a.flags & 16),
        ("rolling", i64::from(roll), a.rolling),
        ("bank_dx", i64::from(bank.0) * 1_000_000, a.bank_dx),
        ("bank_dy", i64::from(bank.1) * 1_000_000, a.bank_dy),
    ]
}

/// **Chapter three's word, widened whole, both directions** (item 587,
/// `docs/DECISIONS.md` 43), on run145. The walk is [`widen_chapter_three`],
/// which run146 shares.
#[test]
fn chapter_three_s_word_frame_is_widened_whole() {
    cases_01_09::chapter_three_s_word_frame_is_widened_whole();
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
    cases_01_09::chapter_three_s_catapult_unpacks_on_the_dump_s_frame();
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
    cases_01_09::chapter_three_s_unpacked_catapult_sees_its_hoplites();
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
    cases_setup::the_ground_order_s_life_is_the_dump_s();
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
    cases_setup::a_turning_catapult_s_crew_mirror_and_never_walk();
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
    cases_01_09::chapter_three_s_catapult_fires_on_the_ground();
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
    cases_01_09::chapter_three_s_crew_mirror_the_turn();
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
    cases_01_09::chapter_three_s_restage_holds_to_its_word();
}

/// **run146's word, widened whole, both directions** (item 587,
/// `docs/DECISIONS.md` 43): [`widen_chapter_three`] over
/// [`WIDENING_CHAPTER_THREE_RESTAGE`], the whole capture, printing the
/// catapult `0/6` both sides on the word's two blocks.
#[test]
fn chapter_three_s_restage_is_widened_whole() {
    cases_01_09::chapter_three_s_restage_is_widened_whole();
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
    cases_01_09::chapter_three_s_restage_numbers_its_objects();
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
    cases_01_09::chapter_three_s_arena_a_hoplites_take_the_dump_s_numbers();
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
    cases_01_09::chapter_three_s_falsifiers_are_the_dump_s();
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
    cases_01_09::chapter_three_s_catapult_after_its_reload();
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
    cases_01_09::chapter_seven_b_s_control_scout_search_is_priced_as_the_original();
}

/// **Chapter thirty, pinned** — the gather point (`docs/GOLDEN.md` §39,
/// item 928, run312). Fourteen staged lines: chapter twenty-eight's cast,
/// five `@gatherpoint` and three `@queueup` lines.
#[test]
fn chapter_thirty_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_holds_to_the_golden_word();
}

/// **run312 whole, both directions** (item 928, `docs/GOLDEN.md` §39):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run312, chapter thirty. **The first
// walk**, the gather point not modelled (`@gatherpoint` skipped): the
// lists part on 618, 652 and 702 (`build:gather_len`, ours 0 against 1);
// the Citizen `0/10` on 760, out south with no order here and north-east
// with a `MOVEORDER` there; the first Hoplites on 856; 2008's Hoplites out
// here and inside there on 953; the Bowmen on 1060; and from the word,
// 822, the stream's cascade over the other citizens and the AI's units.
// **The gather point built** (item 928, `docs/PRODUCTION.md` "The gather
// point"): 696 → 105 rows, the word closed at 1450. What stands: the
// births' `form` (parked 646); 953, 2008's Hoplites seated at the
// building's own point here and at it plus (24, 24) there (`Unit::init`'s
// seat, parked 646); 856, the group move's id, ours `64 +` a pushed index
// and theirs `who·64 +` the slot (parked 676's first); ~~and 1060..1079, the
// Bowmen's exit: the bearing agrees and the ring's point does not, ours
// (3336, 14376) against (3384, 14232), and everything after it on the walk
// to 2008 (named, not built).~~ **The re-seat built** (item 945,
// `docs/PRODUCTION.md` "The gather point"): `come_out`'s routing sweeps the
// captain round its trainer again from the bearing to the building at the
// point, and `0/17` stands at (3384, 14232) on 1060 as there. 105 → 35
// rows: the 70 of 1060..1079 go, and the Bowmen's three `form` rows stay
// (parked 646).
const WANT_CH30: &[&str] = &[];

// **What parts in the pool** on run312. ~~The first walk: each press's
// building group, seated there by `process_group` and skipped here (618
// `[2007]`, 652 `[2000]`, 702 `[2008]`), and every seat after it one
// stamp or one slot off; the first Hoplites' group laid out and moving on
// 856 there.~~ **The command built** (item 928): every slot's list, stamp,
// layout and speeds agree; what stands is the first seats' `ox`/`oy`,
// ours −1 against 0 (parked 887).
const WANT_CH30_POOL: &[&str] = &[];

/// **Chapter thirty-one, pinned** — the gather point's other arms
/// (`docs/GOLDEN.md` §40, item 955, run338). Sixteen staged lines: chapter
/// thirty's cast and a second Chariot, six `@gatherpoint`, one
/// `@gatherpointadd`, three `@queueup` and one `@build`.
#[test]
fn chapter_thirty_one_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_one_holds_to_the_golden_word();
}

/// **run338 whole, both directions** (item 955, `docs/GOLDEN.md` §40):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_one_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_one_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run338, chapter thirty-one. **The
// first walk** (item 955, before any build): 811 rows. **The first
// parting's field list**, 718, `0/11` alone: `pos`, `g.x/y[0]`,
// `g.des_x/y[0]`, `dest_angle` and the move's `angle` (ours (4104, 31032),
// theirs (3960, 31368): the lone re-seat, which counts the seeker there),
// and `form` (parked 646). Then 824 (`0/12`, the same re-seat, and its
// stack: ours a `MOVEORDER` ahead of the `GATHERORDER`), 858 (`0/13`, the
// third re-seat, which this crate lacks, and the group's target), 938
// (`0/16`, as `0/12`), 953 (`0/17`..`0/19`, one order here and two there),
// and from the word, 740, `0/11` idle here and walking there, and the
// stream's cascade. **The arms built** (item 955, `docs/PRODUCTION.md`
// "The gather point"): 811 → 15 rows, the word closed at 1400. What
// stands: the births' `form` (parked 646); and the group moves' ids, ours
// `64 +` a pushed index and theirs `who·64 +` the slot (parked 676's
// first). 1144, the builder `0/11`'s `group` (ours 1 against −1 as its
// finished `BuildOrder` gave way to a `GATHERORDER`), agrees since item
// 1167: the builder's gather clears `+0x80` (`docs/GOLDEN.md` §49). 15 →
// 14 rows.
const WANT_CH31: &[&str] = &[];

// **What parts in the pool** on run338: the first seats' `ox`/`oy`
// (parked 887). ~~And the two Hoplite groups laid out from their own
// captains' points and orders~~: 13 → 4 rows with the arms built.
const WANT_CH31_POOL: &[&str] = &[];

/// **Chapter thirty-two, pinned** — an Airbase's gather point
/// (`docs/GOLDEN.md` §41, item 947, run344). Sixteen staged lines:
/// chapter twenty-nine's thirteen and three `@gatherpoint` lines on the
/// Airbase, one an append and one the Clear.
#[test]
fn chapter_thirty_two_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_two_holds_to_the_golden_word();
}

/// **run344 whole, both directions** (item 947, `docs/GOLDEN.md` §41):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_two_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_two_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run344, chapter thirty-two. **The
// first walk** (item 947, before any build): 364 rows. Chapter
// twenty-two's standing nine (611..655); **the first parting's field
// list**, 1602: `0/6`'s `order:flags` and action bit (ours 0, theirs 4),
// `cruising_alt` (ours 1700, its own; theirs 1600, the strafe home's
// carried), `x_pos[0]`/`y_pos[0]` (ours (21120, 16512), theirs P1) and
// `path[0].to`, and `0/7`'s and `0/8`'s flags, action bit and point — the
// hangar loop of `Build::add_gather_point@00622e70`, their one writer on
// the block; then 0/6's flight from 1603, 1702's arrays, 1741's waypoint,
// the Biplane from 1746 (inside with no order here, on its patrol there,
// out on 1747), and from the word, 1751, the stream's cascade. **The arm
// built** (item 947, `docs/PRODUCTION.md` "The gather point"): the hangar
// loop, `clear_gather`'s half, `train`'s `CARRY_AIR` arm and the patrol's
// points and waypoint; 364 → 17, the word closed at 2360. What stands is
// chapter twenty-nine's: chapter twenty-two's nine, and the Biplane's
// birth on 1746, `form` and its seat — ours the base's own (11616,
// 13920), theirs (11640, 13944) — `Unit::init@00612100`'s (parked 646).
// It leaves on 1747 on the EXIT, where both sides agree.
const WANT_CH32: &[&str] = &[];

// **What parts in the pool** on run344: chapter twenty-two's ten, as
// chapter twenty-nine's.
const WANT_CH32_POOL: &[&str] = &[];

/// **Chapter thirty-three, pinned** — an Airbase's launch issuers
/// (`docs/GOLDEN.md` §42, item 976, run358). Twenty-two staged lines:
/// chapter thirty-two's sixteen, an enemy Barracks, `@launchstrike`,
/// `@launchpatrol`, `@launchpatrolall`, and a `@gatherpoint` of action 3
/// with a `@gatherpointadd` behind it.
#[test]
fn chapter_thirty_three_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_three_holds_to_the_golden_word();
}

/// **run358 whole, both directions** (item 976, `docs/GOLDEN.md` §42):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_three_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_three_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run358, chapter thirty-three. **The
// first walk** (item 976, before any build): 389 rows. Chapter
// thirty-two's standing seventeen (611..655, and the Biplane's birth on
// 1746); **the first parting's field list**, 2262, `0/8` alone: its
// stack (`orders.len`, `order:length`, ours 0, theirs 1 — the strike on
// `1/2006`), `inside` (ours 2007, theirs −1), `pos`, `heading`,
// `dest_angle`, `orders_x/y`, `path:length`, and its figure's point,
// height, bank, pitch and clock. **Their writers** (889): the stack's is
// `action_launch_flight`'s strike from inside the base (`add_strafe_order`)
// alone; the rest are `Object::do_launch`'s `come_out` on the same block,
// the EXIT at the Airbase, which the strike is what admits. Then 2282
// (`0/9`'s patrol), 2297 (`0/7`'s and `0/6`'s), 2307 (every plane's
// strike on `0/1`), 2337 (the patrols over P3), and each flight after.
// **The arms built** (item 976, `docs/PRODUCTION.md` "The launch
// commands"): 389 → 18. What stands is chapter thirty-two's seventeen and
// `0/6`'s `mirror` on 2314, its EXIT under the escort (ours 0, theirs 1),
// with the pool's slot 1 `facing` on the same block: its one other writer,
// `Unit::kill_current_order@005e2cb0`'s move branch, and the order the
// EXIT kills, are not read (the EXIT's own `set_angle(0)` turns 6° and
// cannot flip it).
const WANT_CH33: &[&str] = &["2314 0/6 mirror"];

// **What parts in the pool** on run358: chapter twenty-two's ten, and
// slot 1's `facing` on 2314.
const WANT_CH33_POOL: &[&str] = &["2314 slot 1 facing"];

/// **Chapter thirty-four, pinned** — the launch commands' other arms
/// (`docs/GOLDEN.md` §43, item 1009, run362). Twenty-seven staged lines:
/// chapter thirty-three's seventeen to its Barracks, a second Airbase,
/// ctrl and alt on both launch commands, `@launchmove`, the list `[P1, A3,
/// P2]`, the Clear, and alt at the second base.
#[test]
fn chapter_thirty_four_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_four_holds_to_the_golden_word();
}

/// **Chapter thirty-five, pinned** — the Helicopter's and missiles'
/// launch arms (`docs/GOLDEN.md` §44, item 1019, run371). Thirty staged
/// lines: chapter thirty-four's eighteen to its second Airbase, the
/// resources, a Missile Silo, two V2 Rockets and two Helicopters trained
/// with and without a point, and both launch commands at each base.
#[test]
fn chapter_thirty_five_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_five_holds_to_the_golden_word();
}

/// **run371 whole, both directions** (item 1019, `docs/GOLDEN.md` §44):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_five_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_five_s_word_frame_is_widened_whole();
}

/// **Chapter thirty-six, pinned** — the missile's other arms
/// (`docs/GOLDEN.md` §45, item 1078, run390). Forty-two staged lines:
/// chapter thirty-five's thirty, two silos with a V2 each, two who=1
/// Barracks and a spotter, three strikes and the shield's tech line.
#[test]
fn chapter_thirty_six_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_six_holds_to_the_golden_word();
}

/// **run390 whole, both directions** (item 1078, `docs/GOLDEN.md` §45):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_six_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_six_s_word_frame_is_widened_whole();
}

/// **Chapter thirty-seven, pinned** — the nuke (`docs/GOLDEN.md` §46,
/// item 1091, run397). Fourteen staged lines on the golden start.
#[test]
fn chapter_thirty_seven_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_seven_holds_to_the_golden_word();
}

/// **run397 whole, both directions** (item 1091, `docs/GOLDEN.md` §46):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_seven_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_seven_s_word_frame_is_widened_whole();
}

/// Chapter thirty-seven's widening rows (item 1091): run397 whole, the
/// nuke built. The four probes' and the nuke's `form` (0 against −1) and
/// the nuke's seat inside its silo on 3022 (+24, +24: parked 646's
/// family); `0/6`'s and `0/7`'s death objects, which this capture's end
/// detail does not print (no `DEATHS`), on the blocks after the ring
/// kills them on both sides; and two citizens' animation clocks on
/// 617..655, a residue outside the nuke that moves no draw.
const WANT_CH37: &[&str] = &["3218 0/6 death:extra", "3227 0/7 death:extra"];

/// Chapter thirty-seven's pool rows (item 1091): the research's building
/// group on 617, `ox`/`oy` 0 against −1.
const WANT_CH37_POOL: &[&str] = &[];

/// **Chapter thirty-eight, pinned** — the air line under fire
/// (`docs/GOLDEN.md` §47, item 1102, run404). Eleven staged lines on the
/// golden start.
#[test]
fn chapter_thirty_eight_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_eight_holds_to_the_golden_word();
}

/// **Chapter thirty-nine, pinned** — the spell issuer's untargeted crafts
/// (`docs/GOLDEN.md` §48, item 1111, run422). Fourteen staged lines on the
/// golden start.
#[test]
fn chapter_thirty_nine_holds_to_the_golden_word() {
    cases_30_39::chapter_thirty_nine_holds_to_the_golden_word();
}

/// **Chapter forty, pinned** — the casts' other arms (`docs/GOLDEN.md`
/// §49, item 1167, run430). Twenty-one staged lines on the golden start.
#[test]
fn chapter_forty_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_holds_to_the_golden_word();
}

/// **Chapter forty-one, closed** — CENSUS row 7's last seven
/// (`docs/GOLDEN.md` §50, items 1182 and 1200, run437): the computer's
/// sortie and the build-site spill, and the Biplane's strafes on who=0's
/// Hoplites to the chapter's end. Fifteen staged lines on the golden
/// start.
#[test]
fn chapter_forty_one_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_one_holds_to_the_golden_word();
}

/// **Chapter forty-two** — a ring of who=1's Barracks and four walkers
/// inside it (`docs/GOLDEN.md` §51, item 1209, run460):
/// `Unit::resolve_block`'s arms. Eighteen staged lines on the golden
/// start.
#[test]
fn chapter_forty_two_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_two_holds_to_the_golden_word();
}

/// **Chapter forty-three** — the landing's arms no walk held
/// (`docs/GOLDEN.md` §52, item 1223, run466): a barge put ashore standing
/// on its figure's sixty-fourth frame, and two Hoplite barges whose
/// passengers are a squad's. Thirteen staged lines on the golden start.
#[test]
fn chapter_forty_three_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_three_holds_to_the_golden_word();
}

/// **Chapter forty-four** — the idle search's head on a computer's fleet
/// (`docs/GOLDEN.md` §53, item 1254, run484): a computer's Bomb Vessel
/// takes a human's Barracks across the regions and out of its range, and
/// a human's Bomb Vessel refuses a computer's. Seven staged lines on the
/// golden start.
#[test]
fn chapter_forty_four_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_four_holds_to_the_golden_word();
}

/// **Chapter forty-five** — `get_cost`'s library-line tail on the golden
/// start (`docs/GOLDEN.md` §54, item 1268, run492): a military epoch
/// under Despotism, a civic epoch with Dye held by a human's Merchant,
/// and a commerce epoch with both held. Seven staged lines.
#[test]
fn chapter_forty_five_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_five_holds_to_the_golden_word();
}

/// **Chapter forty-six** — three arms a unit test alone held
/// (`docs/GOLDEN.md` §55, item 1278, run496): chapter forty-three's cast
/// with its barge landing the squad (`Sim::disembark_squad`), and who=1's
/// City training a Citizen under a two-point gather list onto its
/// Woodcutter's Camp (`all_gathering`'s prune). Eighteen staged lines.
#[test]
fn chapter_forty_six_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_six_holds_to_the_golden_word();
}

/// **Chapter forty-seven** — three arms the widenings held and no walk
/// (`docs/GOLDEN.md` §56, item 1310, run514): a Tower site standing when
/// its leader gains the Keep, a squad put ashore facing away from its
/// barge, and an age's snap on a unit mid-move. Thirteen staged lines on
/// the golden start.
#[test]
fn chapter_forty_seven_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_seven_holds_to_the_golden_word();
}

/// **Chapter forty-eight** — three arms the unit tests or a widening held
/// and no walk (`docs/GOLDEN.md` §57, item 1358, run551): a who=1 Tower
/// site standing on who=0's land, struck by a squad and bombed while its
/// builders work. Seventeen staged lines on the golden start.
#[test]
fn chapter_forty_eight_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_eight_holds_to_the_golden_word();
}

/// **Chapter forty-nine** — three arms the unit tests hold and no walk
/// (`docs/GOLDEN.md` §58, item 1393, run576): a Tower's arrow past its
/// owner's second age, and `get_cost`'s bump loop's `JUMP` arm (a Catapult
/// trained under a queued Bombard). Fifteen staged lines on the golden
/// start.
#[test]
fn chapter_forty_nine_holds_to_the_golden_word() {
    cases_40_49::chapter_forty_nine_holds_to_the_golden_word();
}

/// **run576 whole, both directions** (item 1393, `docs/GOLDEN.md` §58):
/// every dumped record on every block, the leaders at `LEADERS=5`, the
/// rounds at `AMMO=5`, and who=0's `GROUPDATA` pool.
#[test]
fn chapter_forty_nine_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_nine_s_word_frame_is_widened_whole();
}

const WANT_CH49: &[&str] = &[
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "943 0/7 death:extra",
];

const WANT_CH49_POOL: &[&str] = &[];

/// **Chapter fifty** — three arms the unit tests hold and no walk
/// (`docs/GOLDEN.md` §59, item 1404, run577): a Stockade's arrow at the
/// first age, and `Group::target_opportunity`'s cooldown and member arm.
#[test]
fn chapter_fifty_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_holds_to_the_golden_word();
}

/// **run577 whole, both directions** (item 1404, `docs/GOLDEN.md` §59).
#[test]
fn chapter_fifty_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_s_word_frame_is_widened_whole();
}

/// **Chapter fifty-one** — chapter fifty's script a frame earlier (item
/// 1423, `docs/GOLDEN.md` §61, run582): the group's cooldown, and a
/// building's `visible` clear gate on 778.
#[test]
fn chapter_fifty_one_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_one_holds_to_the_golden_word();
}

/// **run582 whole, both directions** (item 1423, `docs/GOLDEN.md` §61).
#[test]
fn chapter_fifty_one_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_one_s_word_frame_is_widened_whole();
}

/// **Chapter fifty-three** (item 1707, `docs/GOLDEN.md` §63, run771): the
/// member `find_melee_target` arm, and the cavalry archer's M5 and M6.
#[test]
fn chapter_fifty_three_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_three_holds_to_the_golden_word();
}

/// **run771 whole, both directions** (item 1707, `docs/GOLDEN.md` §63).
#[test]
fn chapter_fifty_three_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_three_s_word_frame_is_widened_whole();
}

const WANT_CH53: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "943 0/6 death:extra",
];

const WANT_CH53_POOL: &[&str] = &[];

/// **Chapter fifty-five** (item 1798, `docs/GOLDEN.md` §67, run783): a captain
/// Dragoon past a Citizen (M6) and a second Dragoon's stationary rounds at a
/// Stockade (the anim-12 `RELEASES` row).
#[test]
fn chapter_fifty_five_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_five_holds_to_the_golden_word();
}

/// **run783 whole, both directions** (item 1798, `docs/GOLDEN.md` §67).
#[test]
fn chapter_fifty_five_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_five_s_word_frame_is_widened_whole();
}

const WANT_CH55: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

const WANT_CH55_POOL: &[&str] = &[];

/// **Chapter fifty-six** (item 1829, `docs/GOLDEN.md` §68, run786): a captain
/// Dragoon past a Citizen and a Hoplite squad added on 763 (M6's other arm).
#[test]
fn chapter_fifty_six_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_six_holds_to_the_golden_word();
}

/// **run786 whole, both directions** (item 1829, `docs/GOLDEN.md` §68).
#[test]
fn chapter_fifty_six_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_six_s_word_frame_is_widened_whole();
}

const WANT_CH56: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "874 1/8 death:extra",
    "900 1/9 death:extra",
    "961 1/7 death:extra",
];

const WANT_CH56_POOL: &[&str] = &[];

/// **Chapter fifty-seven** (item 1863, `docs/GOLDEN.md` §69, run791): a captain
/// Dragoon past a Citizen and a Hoplite squad added on 763 (M6's other arm).
#[test]
fn chapter_fifty_seven_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_seven_holds_to_the_golden_word();
}

/// **run791 whole, both directions** (item 1863, `docs/GOLDEN.md` §69).
#[test]
fn chapter_fifty_seven_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_seven_s_word_frame_is_widened_whole();
}

const WANT_CH57: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// **Chapter fifty-nine** (item 1960, `docs/GOLDEN.md` §71, run801): a Hoplite
/// squad guarding a Citizen between two Stockades; its members' guard search
/// goes through `find_melee_target`'s squad head (parked 1887).
#[test]
fn chapter_fifty_nine_holds_to_the_golden_word() {
    cases_50_59::chapter_fifty_nine_holds_to_the_golden_word();
}

/// **run801 whole, both directions** (item 1960, `docs/GOLDEN.md` §71).
#[test]
fn chapter_fifty_nine_s_word_frame_is_widened_whole() {
    cases_50_59::chapter_fifty_nine_s_word_frame_is_widened_whole();
}

const WANT_CH59: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// Empty since item 2002 (`docs/GOLDEN.md` §73): slot 1's `speed`/`new_speed`
/// 0 against 25 on 625 left when `find_nearby_target`'s add normalizes the
/// searcher's group.
const WANT_CH59_POOL: &[&str] = &[];

/// **Chapter sixty-one** (item 2032, `docs/GOLDEN.md` §74, run814): two
/// guarding Hoplite squads; a follower's chase on its captain's Stockade
/// refused (`find_new_target`) and a follower's target refused by the leash,
/// each through `find_melee_target`'s squad head (parked 1887).
#[test]
fn chapter_sixty_one_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_one_holds_to_the_golden_word();
}

/// **run814 whole, both directions** (item 2032, `docs/GOLDEN.md` §74;
/// through its end since item 2099, §76).
#[test]
fn chapter_sixty_one_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_one_s_word_frame_is_widened_whole();
}

// **What parts through the word** on run814 (item 2032, `docs/GOLDEN.md`
// §74; item 2099, §76): the standing `form` rows (605) and the
// `death:extra` instrument on `0/7` (774) and `0/6` (931), parked 1830's
// family. Scene two's captain `0/10` on 779–780 left with `fight`'s
// `poor_target` arm (item 2099): the chapter is closed at 1000.
const WANT_CH61: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "774 0/7 death:extra",
    "931 0/6 death:extra",
];

// Empty since item 2099: the word's slot 0 `facing` on 780 left with
// `0/10`'s attack.
const WANT_CH61_POOL: &[&str] = &[];

/// **Chapter sixty-two** (item 2066, `docs/GOLDEN.md` §75, run818): two
/// guarding Hoplite squads, who=1's squads walked past them and a third added
/// inside the leash; a follower's guard refusal under its squad head
/// (parked 2033).
#[test]
fn chapter_sixty_two_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_two_holds_to_the_golden_word();
}

/// **run818 from 605 through the word, both directions** (item 2066,
/// `docs/GOLDEN.md` §75).
#[test]
fn chapter_sixty_two_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_two_s_word_frame_is_widened_whole();
}

// **What parts through the word** on run818, whole since item 2117
// (`docs/GOLDEN.md` §75, §77): the standing `form` rows (605); both leaders'
// `treaties[·]` on 757, the first blow, 2 here and 3 there (chapter eight's
// standing family, `docs/GOLDEN.md` §12); `1/8`'s first wound on 841, 13 here
// and 9 there (`hits_left` 107 against 111; parked 2068); and `1/14` on
// 901–903, the third squad's last man: its walk to the Scout has the
// original's order point and slot (46056, `off` 744/72) and a path whose
// waypoints stand 120 east of the original's from the second on, so its
// first step and heading part (parked 2118). `1/9`'s rows on 702–703 left
// with item 2117's `QUEUE_FIRST`.
const WANT_CH62: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "757 0/-1 leader:treaties[1]",
    "757 1/-1 leader:treaties[0]",
    "841 1/8 damage_frac",
    "841 1/8 hits:damage",
    "841 1/8 hits:damage_frac",
    "841 1/8 hits_left",
    "901 1/14 g.angle[0]",
    "901 1/14 g.des_angle[0]",
    "901 1/14 g.des_x[0]",
    "901 1/14 g.des_y[0]",
    "901 1/14 g.x[0]",
    "901 1/14 g.y[0]",
    "901 1/14 heading",
    "901 1/14 order:move.dest_x",
    "901 1/14 path[10].to",
    "901 1/14 path[11].to",
    "901 1/14 path[12].to",
    "901 1/14 path[13].to",
    "901 1/14 path[14].to",
    "901 1/14 path[15].to",
    "901 1/14 path[16].to",
    "901 1/14 path[17].to",
    "901 1/14 path[18].to",
    "901 1/14 path[19].to",
    "901 1/14 path[1].to",
    "901 1/14 path[20].to",
    "901 1/14 path[21].to",
    "901 1/14 path[22].to",
    "901 1/14 path[23].to",
    "901 1/14 path[24].to",
    "901 1/14 path[25].to",
    "901 1/14 path[26].to",
    "901 1/14 path[27].to",
    "901 1/14 path[28].to",
    "901 1/14 path[29].to",
    "901 1/14 path[2].to",
    "901 1/14 path[30].to",
    "901 1/14 path[31].to",
    "901 1/14 path[32].to",
    "901 1/14 path[33].to",
    "901 1/14 path[34].to",
    "901 1/14 path[35].to",
    "901 1/14 path[36].to",
    "901 1/14 path[37].to",
    "901 1/14 path[38].to",
    "901 1/14 path[39].to",
    "901 1/14 path[3].to",
    "901 1/14 path[40].to",
    "901 1/14 path[41].to",
    "901 1/14 path[42].to",
    "901 1/14 path[4].to",
    "901 1/14 path[5].to",
    "901 1/14 path[6].to",
    "901 1/14 path[7].to",
    "901 1/14 path[8].to",
    "901 1/14 path[9].to",
    "901 1/14 pos",
    "902 1/14 dest_angle",
    "902 1/14 orders_x",
    "902 1/14 orders_y",
    "903 1/14 angle:Facing",
    "903 1/14 angle:Heading",
];

// Empty: no slot of the pool parts through the word.
const WANT_CH62_POOL: &[&str] = &[];

/// **Chapter sixty-three** (item 2131, `docs/GOLDEN.md` §78, run827): two
/// Hoplite squads follow a Chariot and are shift-moved — a `QUEUE_LAST`
/// move's formation read from `get_loc_to`, which reads `get_final_loc`'s
/// `Follow` arm (parked 2058).
#[test]
fn chapter_sixty_three_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_three_holds_to_the_golden_word();
}

/// **run827 whole, both directions** (item 2131, `docs/GOLDEN.md` §78).
#[test]
fn chapter_sixty_three_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_three_s_word_frame_is_widened_whole();
}

// **What parts through the word** on run827 (item 2131, `docs/GOLDEN.md`
// §78): the eleven standing `form` rows (605) alone. Without the `Follow`
// arm of `unit_final_loc`, the two shift-moves' queued orders part on 662
// and 712 — `angle`, the slot points and the group's record.
const WANT_CH63: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

const WANT_CH63_POOL: &[&str] = &[];

/// **Chapter sixty-four** (item 2165, `docs/GOLDEN.md` §79, run835): two
/// human Barks, a computer's, a Trireme and a fishing boat, and a player's
/// stance button twice — `Unit::init`'s Bark arm (`unit_masks2 4`, a
/// computer's RAID), `fight`'s re-search for a unit with that bit (parked
/// 2100), and `Group::action_stance`'s human arm.
#[test]
fn chapter_sixty_four_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_four_holds_to_the_golden_word();
}

/// **run835 to the word, both directions** (item 2165, `docs/GOLDEN.md` §79).
#[test]
fn chapter_sixty_four_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_four_s_word_frame_is_widened_whole();
}

// **What parts through run835** (items 2165, 2205, 2304; `docs/GOLDEN.md`
// §79, §80, §83): the eleven standing `form` rows (605), and block 800,
// where this crate prints `0/6`'s death and the Trireme `1/6`'s round loses
// its target while the dump prints no death record for the sunk Bark. The
// Barks' rounds (638 on, parked 2206) and the Trireme's third key (634, a
// unit out) leave from the packet's release rows since item 2304: 140 → 14.
const WANT_CH64: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "800 0/6 death:extra",
    "800 1/6 ammo[1].ox",
    "800 1/6 ammo[1].whom",
];

const WANT_CH64_POOL: &[&str] = &[];

/// **Chapter sixty-five** (item 2235, `docs/GOLDEN.md` §81, run843): a
/// computer's second city, `add small_city`, with a Tower and a Barracks; a
/// human Hoplite squad on the Tower and a Catapult that engages the city
/// centre on its own — `Object::do_damage`'s city alarm, its non-capital
/// heavy arm, the `/10` ceiling and the siege arm (parked 2110).
#[test]
fn chapter_sixty_five_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_five_holds_to_the_golden_word();
}

/// **run843 whole, both directions** (item 2235, `docs/GOLDEN.md` §81). No
/// pool: the capture prints no `GROUPS`.
#[test]
fn chapter_sixty_five_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_five_s_word_frame_is_widened_whole();
}

// **What parts through run843** (items 2235, 2265, 2304; `docs/GOLDEN.md`
// §81, §82, §83): the eleven standing `form` rows (605) alone. Item 2235's
// 777 wake went with 2265's packer stance, and the Catapult's first round's
// launch point on 818 (parked 2266) with item 2304's release row: 15 → 11.
const WANT_CH65: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// **Chapter sixty-seven** (item 2331, `docs/GOLDEN.md` §84, run845): a
/// Light Horse at a Stable each side of Monarchy and Hoplites at a
/// Barracks behind them — `train_time`'s and `get_cost`'s Monarchy arms,
/// and the trainer by equality.
#[test]
fn chapter_sixty_seven_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_seven_holds_to_the_golden_word();
}

/// **run845 whole, both directions** (item 2331, `docs/GOLDEN.md` §84). No
/// pool: the capture prints no `GROUPS`.
#[test]
fn chapter_sixty_seven_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_seven_s_word_frame_is_widened_whole();
}

// **What parts through run845** (item 2331, `docs/GOLDEN.md` §84): the
// eleven standing `form` rows (600) alone. Without `get_cost`'s Monarchy arm
// the second Light Horse's queue entry parts on 804 (61/41 against 45/30);
// without `train_time`'s, its birth on 978.
const WANT_CH67: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-one** (item 2436, `docs/GOLDEN.md` §88, run858): a
/// Senate researching Despotism under the Generals' and Supply upgrades —
/// The Despot is born at 659 hits.
#[test]
fn chapter_seventy_one_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_one_holds_to_the_golden_word();
}

/// **run858 whole, both directions** (item 2436, `docs/GOLDEN.md` §88).
#[test]
fn chapter_seventy_one_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_one_s_word_frame_is_widened_whole();
}

// **What parts through run858** (item 2436, `docs/GOLDEN.md` §88): the
// eleven standing `form` rows and item 2384's two `aggression` rows (600)
// alone; The Despot's birth on 1013 agrees whole, 659 hits.
const WANT_CH71: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-two** (item 2448, `docs/GOLDEN.md` §89, run859): a
/// Senate that trained The Despot researches Monarchy at the Gunpowder
/// Age — the tail's `set_type` arm on a human.
#[test]
fn chapter_seventy_two_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_two_holds_to_the_golden_word();
}

/// **run859 whole, both directions** (item 2448, `docs/GOLDEN.md` §89).
#[test]
fn chapter_seventy_two_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_two_s_word_frame_is_widened_whole();
}

const WANT_CH72: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-three** (item 2474, `docs/GOLDEN.md` §90, run860): a
/// Senate that trained The Despot researches Monarchy at the Gunpowder
/// Age — the tail's `set_type` arm on a human.
#[test]
fn chapter_seventy_three_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_three_holds_to_the_golden_word();
}

/// **run860 whole, both directions** (item 2474, `docs/GOLDEN.md` §90).
#[test]
fn chapter_seventy_three_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_three_s_word_frame_is_widened_whole();
}

const WANT_CH73: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-four** (item 2500, `docs/GOLDEN.md` §91, run864): the
/// console's `finish` on a human's Library — `Console::hurry_action`'s queue
/// arm pays the head its train time.
#[test]
fn chapter_seventy_four_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_four_holds_to_the_golden_word();
}

/// **run864 whole, both directions** (item 2500, `docs/GOLDEN.md` §91).
#[test]
fn chapter_seventy_four_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_four_s_word_frame_is_widened_whole();
}

const WANT_CH74: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-five** (item 2519, `docs/GOLDEN.md` §92, run868): the
/// console's `finish` behind a population-capped unit head —
/// `get_next_non_unit`'s entry takes the done head's train time.
#[test]
fn chapter_seventy_five_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_five_holds_to_the_golden_word();
}

/// **run868 whole, both directions** (item 2519, `docs/GOLDEN.md` §92).
#[test]
fn chapter_seventy_five_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_five_s_word_frame_is_widened_whole();
}

const WANT_CH75: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-six** (item 2572, `docs/GOLDEN.md` §95, run871): the
/// President's train arm held on a human — Hoplites at a Barracks beside the
/// Senate's President, against the same batch before he is born.
#[test]
fn chapter_seventy_six_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_six_holds_to_the_golden_word();
}

/// **run871 whole, both directions** (item 2572, `docs/GOLDEN.md` §95).
#[test]
fn chapter_seventy_six_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_six_s_word_frame_is_widened_whole();
}

const WANT_CH76: &[&str] = &[
    "1013 0/-1 leader:resource_cap[0:food]",
    "1013 0/-1 leader:resource_cap[1:timber]",
    "1013 0/-1 leader:resource_cap[2:wealth]",
    "1013 0/-1 leader:resource_cap[4:metal]",
    "1013 0/-1 leader:resource_cap[5:oil]",
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-seven** (item 2598, `docs/GOLDEN.md` §96, run872): the
/// Porcelain Tower's ship arm held on a human — a Galley at a Dock before the
/// Tower and after it.
#[test]
fn chapter_seventy_seven_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_seven_holds_to_the_golden_word();
}

/// **run872 whole, both directions** (item 2598, `docs/GOLDEN.md` §96).
#[test]
fn chapter_seventy_seven_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_seven_s_word_frame_is_widened_whole();
}

const WANT_CH77: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **Chapter seventy-eight** (item 2627, `docs/GOLDEN.md` §97, run874): the
/// Porcelain Tower's ship arm held on a human — a Galley at a Dock before the
/// Tower and after it.
#[test]
fn chapter_seventy_eight_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_eight_holds_to_the_golden_word();
}

/// **run874 whole, both directions** (item 2627, `docs/GOLDEN.md` §97).
#[test]
fn chapter_seventy_eight_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_eight_s_word_frame_is_widened_whole();
}

/// **Chapter seventy-nine** (item 2649, `docs/GOLDEN.md` §98, run875): a
/// Biplane flown from an Airbase, patrolling and then striking a Barracks.
#[test]
fn chapter_seventy_nine_holds_to_the_golden_word() {
    cases_70_79::chapter_seventy_nine_holds_to_the_golden_word();
}

/// **run875 whole, both directions** (item 2649, `docs/GOLDEN.md` §98).
#[test]
fn chapter_seventy_nine_s_word_frame_is_widened_whole() {
    cases_70_79::chapter_seventy_nine_s_word_frame_is_widened_whole();
}

/// **run875's start-of-game `seen3` plane, cell for cell** (item 2755,
/// `docs/GOLDEN.md` §102, parked 2727): the first `WORLD` block prints
/// `seen3[scan]`, the inner disc of every `DETECT` object, and nothing read
/// it. The start dump is the only block of the capture that prints the plane;
/// ours is the staged sim before its first tick.
#[test]
fn run875_start_seen3_plane_is_compared_cell_for_cell() {
    cases_setup::run875_start_seen3_plane_is_compared_cell_for_cell();
}

/// **Chapter eighty's staging, read off this crate** (item 2755,
/// `docs/GOLDEN.md` §102): an Airbase `1/2006` (190, 60) lights, never
/// detecting, the three sites 14 tiles from it — an idle Partisan `0/6`
/// (`FLAGS s`), a Partisan `0/7` a Scout's disc reaches on 1197, and three
/// Hoplites, the control. At the 1000 pass the first two weigh half
/// (`unit_stealth_hides`), at the 1200 pass the second whole, the Hoplites
/// whole throughout.
#[test]
fn chapter_eighty_staging_reaches_the_stealth_arm() {
    cases_80_89::chapter_eighty_staging_reaches_the_stealth_arm();
}

/// **Chapter eighty** (item 2755, `docs/GOLDEN.md` §102, run877): the stealth
/// arm of `is_seen` under a computer's `calc_danger`.
#[test]
fn chapter_eighty_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_holds_to_the_golden_word();
}

/// **Chapter eighty-five** (item 2832, `docs/GOLDEN.md` §108, run880): the
/// cloaked Spy's Bribe, a who=1 Scout in sight and a control out of it.
#[test]
fn chapter_eighty_five_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_five_holds_to_the_golden_word();
}

/// **run880 whole, both directions** (item 2832, `docs/GOLDEN.md` §108).
#[test]
fn chapter_eighty_five_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_five_s_window_is_widened_whole();
}

/// **Chapter eighty-six** (item 2876, `docs/GOLDEN.md` §109, run884): the
/// cloaked Spy's Bribe, the control alone on a Hoplite that stays.
#[test]
fn chapter_eighty_six_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_six_holds_to_the_golden_word();
}

/// **run884 whole, both directions** (item 2876, `docs/GOLDEN.md` §109).
#[test]
fn chapter_eighty_six_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_six_s_window_is_widened_whole();
}

/// **Chapter eighty-seven's second arm** (item 2895, `docs/GOLDEN.md` §111, run885):
/// a Bribe on a target outside the caster's owner's sight.
#[test]
fn chapter_eighty_seven_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_seven_holds_to_the_golden_word();
}

/// **run885 whole, both directions** (item 2895, `docs/GOLDEN.md` §111).
#[test]
fn chapter_eighty_seven_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_seven_s_window_is_widened_whole();
}

/// What parts in run885's window, blocks 1100–1500.
const WANT_CH87: &[&str] = &[
    "1100 0/0 form",
    "1100 0/1 form",
    "1100 0/2 form",
    "1100 0/2000 build:extra",
    "1100 0/2001 build:extra",
    "1100 0/2002 build:extra",
    "1100 0/2003 build:extra",
    "1100 0/2004 build:extra",
    "1100 0/2005 build:extra",
    "1100 0/2006 build:extra",
    "1100 0/3 form",
    "1100 0/4 form",
    "1100 0/5 form",
    "1100 1/1 form",
    "1100 1/2 form",
    "1100 1/2000 build:extra",
    "1100 1/2001 build:extra",
    "1100 1/2002 build:extra",
    "1100 1/2003 build:extra",
    "1100 1/2004 build:extra",
    "1100 1/2005 build:extra",
    "1100 1/3 form",
    "1100 1/4 form",
    "1100 1/5 form",
];

/// **Chapter eighty-eight** (item 2915, `docs/GOLDEN.md` §112, run889): a Bribe whose
/// target turns invalid on the approach.
#[test]
fn chapter_eighty_eight_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_eight_holds_to_the_golden_word();
}

/// **run889 whole, both directions** (item 2915, `docs/GOLDEN.md` §112).
#[test]
fn chapter_eighty_eight_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_eight_s_window_is_widened_whole();
}

/// **Chapter eighty-nine** (item 2930, `docs/GOLDEN.md` §113, run890): a Bribe whose
/// target is outside the caster's owner's sight when the Spy reaches range.
#[test]
fn chapter_eighty_nine_holds_to_the_golden_word() {
    cases_80_89::chapter_eighty_nine_holds_to_the_golden_word();
}

/// **run890 whole, both directions** (item 2930, `docs/GOLDEN.md` §113).
#[test]
fn chapter_eighty_nine_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_nine_s_window_is_widened_whole();
}

/// **Chapter ninety** (item 2949, `docs/GOLDEN.md` §114, run892): an Informer
/// cast lost mid-approach on a building, unseen at range arrival on a unit,
/// and refused on a stealth-hidden Spy.
#[test]
fn chapter_ninety_holds_to_the_golden_word() {
    cases_80_89::chapter_ninety_holds_to_the_golden_word();
}

/// **run892 whole, both directions** (item 2949, `docs/GOLDEN.md` §114).
#[test]
fn chapter_ninety_s_window_is_widened_whole() {
    cases_80_89::chapter_ninety_s_window_is_widened_whole();
}

/// What parts in run892's window, blocks 605–1200.
const WANT_CH90: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// **Chapter ninety-one** (item 2971, `docs/GOLDEN.md` §115, run894): the
/// Sniper and Counterintelligence's target arms.
#[test]
fn chapter_ninety_one_holds_to_the_golden_word() {
    cases_80_89::chapter_ninety_one_holds_to_the_golden_word();
}

/// **run894 whole, both directions** (item 2971, `docs/GOLDEN.md` §115).
#[test]
fn chapter_ninety_one_s_window_is_widened_whole() {
    cases_80_89::chapter_ninety_one_s_window_is_widened_whole();
}

/// What parts in run894's window, blocks 605–1200.
const WANT_CH91: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// What parts in run890's window, blocks 1100–1500.
const WANT_CH89: &[&str] = &[
    "1100 0/0 form",
    "1100 0/1 form",
    "1100 0/2 form",
    "1100 0/2000 build:extra",
    "1100 0/2001 build:extra",
    "1100 0/2002 build:extra",
    "1100 0/2003 build:extra",
    "1100 0/2004 build:extra",
    "1100 0/2005 build:extra",
    "1100 0/2006 build:extra",
    "1100 0/3 form",
    "1100 0/4 form",
    "1100 0/5 form",
    "1100 1/1 form",
    "1100 1/2 form",
    "1100 1/2000 build:extra",
    "1100 1/2001 build:extra",
    "1100 1/2002 build:extra",
    "1100 1/2003 build:extra",
    "1100 1/2004 build:extra",
    "1100 1/2005 build:extra",
    "1100 1/3 form",
    "1100 1/4 form",
    "1100 1/5 form",
];

/// What parts in run889's window, blocks 1100–1500.
const WANT_CH88: &[&str] = &[
    "1100 0/0 form",
    "1100 0/1 form",
    "1100 0/2 form",
    "1100 0/2000 build:extra",
    "1100 0/2001 build:extra",
    "1100 0/2002 build:extra",
    "1100 0/2003 build:extra",
    "1100 0/2004 build:extra",
    "1100 0/2005 build:extra",
    "1100 0/2006 build:extra",
    "1100 0/3 form",
    "1100 0/4 form",
    "1100 0/5 form",
    "1100 1/1 form",
    "1100 1/2 form",
    "1100 1/2000 build:extra",
    "1100 1/2001 build:extra",
    "1100 1/2002 build:extra",
    "1100 1/2003 build:extra",
    "1100 1/2004 build:extra",
    "1100 1/2005 build:extra",
    "1100 1/3 form",
    "1100 1/4 form",
    "1100 1/5 form",
];

/// What parts in run884's window, blocks 1100–1299.
///
/// Only the standing rows of 1100 (`form`, the extra buildings this crate holds alone), the same as chapter eighty-five's. **Nothing
/// the Bribe touches parts**: the cast from 1114, the swap on 1212, the new squad `0/7`..`0/9`, the closed `1/6` and the Spy's idle agree on every block.
const WANT_CH86: &[&str] = &[
    "1100 0/0 form",
    "1100 0/1 form",
    "1100 0/2 form",
    "1100 0/2000 build:extra",
    "1100 0/2001 build:extra",
    "1100 0/2002 build:extra",
    "1100 0/2003 build:extra",
    "1100 0/2004 build:extra",
    "1100 0/2005 build:extra",
    "1100 0/2006 build:extra",
    "1100 0/3 form",
    "1100 0/4 form",
    "1100 0/5 form",
    "1100 1/1 form",
    "1100 1/2 form",
    "1100 1/2000 build:extra",
    "1100 1/2001 build:extra",
    "1100 1/2002 build:extra",
    "1100 1/2003 build:extra",
    "1100 1/2004 build:extra",
    "1100 1/2005 build:extra",
    "1100 1/3 form",
    "1100 1/4 form",
    "1100 1/5 form",
];

/// What parts in run880's window, blocks 1100–1218.
///
/// The standing `form` rows and the added Scout's first disc (`1103 world:seen2`, 7 cells, the first (59, 54) ours 3 theirs 1).
/// **The control Spy `0/7`'s chase of a walking target no longer parts** (item 2881, 19 rows struck): the original's
/// sixteen-frame review (`Unit::work`'s `(frame + o) & 0xf` gate, `check_target_path`'s cast arm) `repath`s the approach on 1177
/// because the target's head order is a move, and `do_cast` re-aims it the same frame; this crate carries the arm.
/// **Nothing the Bribe's conversion touches parts**: the swapped squad `0/8`..`0/10`, its owner, `unit_masks` and the closed `1/6`
/// agree on every block.
const WANT_CH85: &[&str] = &[
    "1100 0/0 form",
    "1100 0/1 form",
    "1100 0/2 form",
    "1100 0/3 form",
    "1100 0/4 form",
    "1100 0/5 form",
    "1100 1/1 form",
    "1100 1/2 form",
    "1100 1/3 form",
    "1100 1/4 form",
    "1100 1/5 form",
    "1103 0/-1 world:seen2",
];

/// **run877 whole, both directions** (item 2755, `docs/GOLDEN.md` §102).
#[test]
fn chapter_eighty_s_window_is_widened_whole() {
    cases_80_89::chapter_eighty_s_window_is_widened_whole();
}

/// **run878 whole, both directions** (item 2785, `docs/GOLDEN.md` §103).
#[test]
fn chapter_eighty_w_window_is_widened_whole() {
    cases_80_89::chapter_eighty_w_window_is_widened_whole();
}

/// What parts on run878's blocks 998–1001: the standing `form` rows alone
/// (the `0/7` blown bit is carried, item 2785).
const WANT_CH80W: &[&str] = &[
    "998 0/0 form",
    "998 0/1 form",
    "998 0/2 form",
    "998 0/3 form",
    "998 0/4 form",
    "998 0/5 form",
    "998 1/1 form",
    "998 1/2 form",
    "998 1/3 form",
    "998 1/4 form",
    "998 1/5 form",
];

/// What parts on run877's blocks 1198–1203 (block 1204 is not in the dump). **Item 2785 closes the chapter**: the
/// standing `form` rows alone remain. Before it, **the first**, `1198 0/-1 world:danger`, was the word: who 1's danger at half-cell (25, 7), the
/// second Partisan's, is 120 theirs and 60 ours on the first dumped block —
/// the plane the 1000 pass wrote, before the Scout stood there. The 1200
/// pass, where the Scout's disc covers it, agrees. The rest are the added
/// Scout's own walk from 1199 and the `seen3` cells its path leaves.
const WANT_CH80: &[&str] = &[
    "1198 0/0 form",
    "1198 0/1 form",
    "1198 0/2 form",
    "1198 0/3 form",
    "1198 0/4 form",
    "1198 0/5 form",
    "1198 1/1 form",
    "1198 1/2 form",
    "1198 1/3 form",
    "1198 1/4 form",
    "1198 1/5 form",
];

const WANT_CH79: &[&str] = &[
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

const WANT_CH78: &[&str] = &[
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
    // An `add`ed Helicopter stands at height 30 the frame after, 631 in ours.
    "611 0/6 g.last_z[0]",
    "611 0/6 g.z[0]",
];

/// **Chapter sixty-eight** (item 2357, `docs/GOLDEN.md` §85, run852): a
/// Light Horse at a Stable under Monarchy and a live military discount,
/// and a Catapult at a Siege Factory each side of Socialism — `get_cost`'s
/// three truncations in order, and `train_time`'s Socialism arm.
#[test]
fn chapter_sixty_eight_holds_to_the_golden_word() {
    cases_60_69::chapter_sixty_eight_holds_to_the_golden_word();
}

/// **run852 whole, both directions** (item 2357, `docs/GOLDEN.md` §85). No
/// pool: the capture prints no `GROUPS`.
#[test]
fn chapter_sixty_eight_s_word_frame_is_widened_whole() {
    cases_60_69::chapter_sixty_eight_s_word_frame_is_widened_whole();
}

// **What parts through run852** (item 2357, `docs/GOLDEN.md` §85): the
// eleven standing `form` rows (600) alone. Without `get_cost`'s Socialism
// arm Catapult B's queue entry parts on 804 (81/81 against 60/60).
const WANT_CH68: &[&str] = &[
    // Item 2384: the two `aggression` rows, ours 0 theirs 1 (parked 2385).
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// **run770: chapter fifty-one's script with the fog printed** (item 1497,
/// `docs/GOLDEN.md` §62) — `WORLD=6` per frame over 690..714, which holds the
/// Stockade's first round on 698. The walk compares every fog cell.
#[test]
fn chapter_fifty_two_s_fog_plane_agrees_through_the_first_round() {
    cases_50_59::chapter_fifty_two_s_fog_plane_agrees_through_the_first_round();
}

/// **run804: chapter fifty-nine's script with the fog plane printed**, 605..645
/// (item 1988, `docs/GOLDEN.md` §72): the Stockade added on 606 is met by
/// player 0 at once in the original.
#[test]
fn chapter_sixty_s_fog_plane() {
    cases_60_69::chapter_sixty_s_fog_plane();
}

const WANT_CH51: &[&str] = &[
    "1054 0/7 death:extra",
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "750 0/6 death:extra",
    "900 0/9 death:extra",
    "982 0/8 death:extra",
];

// Item 1776: a pushed group's prune shifts the arrays with the list, so the
// 770 slot 1 rows agree and leave. Item 2002 (`docs/GOLDEN.md` §73): the
// dead member's rows (907, 1003: `list`, `num`, `off`/`curr`/`angle`) agree
// and leave too — a living member's search normalizes the slot and prunes
// it (`find_nearby_target`'s add).
const WANT_CH51_POOL: &[&str] = &[];

const WANT_CH50: &[&str] = &[
    "1063 0/7 death:extra",
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "751 0/6 death:extra",
    "908 0/9 death:extra",
    "986 0/8 death:extra",
];

// Item 1776: as chapter fifty-one's pool (770 slot 1 agrees and leaves).
// Item 2002: as chapter fifty-one's, the dead member's 932 and 996 rows leave.
const WANT_CH50_POOL: &[&str] = &[];

/// **run436: a command for another seat is dropped at the pump** (item
/// 1182, `docs/GOLDEN.md` §50). The first take of chapter forty-one wrote
/// `be 0` on the issuing frame: the DLL issued both who=1 commands (its
/// INFO records carry them, no refusal), and `CommandManager::
/// process_turn` stamped the package with `console->play` — back to 0 —
/// as the pump walked it, so `CommandPackage::process_group` dropped
/// both groups. The Biplane `1/6` holds no order on any block to 1600
/// and the Citizen `1/7` never repairs. Staged here with run436's own
/// lines; made to fail with [`crate::golden::Script::pump`]'s seat test
/// removed.
#[test]
fn run436_s_other_seat_s_commands_are_dropped_at_the_pump() {
    cases_setup::run436_s_other_seat_s_commands_are_dropped_at_the_pump();
}

/// **An issuer line for another seat is refused where it is written**
/// (`tracer.c`'s refusal 1, item 1182): with no `be`, run436's flight
/// for who=1 is refused on its own frame and never reaches the pump. Made
/// to fail with [`crate::golden::Script::apply`]'s seat test removed —
/// the pump's drop then takes the line instead, under its own reason.
#[test]
fn an_issuer_line_for_another_seat_is_refused_as_the_dll_refuses_it() {
    cases_setup::an_issuer_line_for_another_seat_is_refused_as_the_dll_refuses_it();
}

/// **run437 whole, both directions** (item 1182, `docs/GOLDEN.md` §50):
/// every dumped record on every block of the capture. No pool: the
/// capture prints no `GROUPS`, which its verifier refuses when no block
/// prints one (parked 735).
#[test]
fn chapter_forty_one_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_one_s_word_frame_is_widened_whole();
}

/// **run460 whole, both directions** (item 1209, `docs/GOLDEN.md` §51):
/// every dumped record on every block of the capture, the leaders at
/// `LEADERS=5`, whose record prints `agendas[·]`. No pool: the capture
/// prints no `GROUPS` (parked 735).
#[test]
fn chapter_forty_two_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_two_s_word_frame_is_widened_whole();
}

/// **run484 whole, both directions** (item 1254, `docs/GOLDEN.md` §53):
/// every dumped record on every block of the capture, the rounds, and the
/// leaders at `LEADERS=5`. No pool: the capture prints no `GROUPS`
/// (parked 735).
#[test]
fn chapter_forty_four_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_four_s_word_frame_is_widened_whole();
}

/// **run492 whole, both directions** (item 1268, `docs/GOLDEN.md` §54):
/// every dumped record on every block of the capture, the leaders at
/// `LEADERS=5`. No pool: the capture prints no `GROUPS` (parked 735).
#[test]
fn chapter_forty_five_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_five_s_word_frame_is_widened_whole();
}

/// Chapter forty-five's widening rows: run492 whole, 600 to 1650 (item
/// 1268; `docs/GOLDEN.md` §54). The standing families only — the birth
/// `form` (parked 1169; the Merchant `0/6`'s on 603), the first block's
/// census (parked 1183) and `filled_gather_slots`. The three prices agree
/// on who=0's food and timber (blocks 802, 804, 806). Before the item's
/// two Nubian arms, `0/6`'s hits stood from 603 (ours 90, theirs 135) and
/// the Dye's pay from 769 (`income[2:wealth]` 320 against 400,
/// `resources[3:knowledge]` 160 against 240).
const WANT_CH45: &[&str] = &[
    "600 0/-1 leader:aggression[1]",
    "600 0/0 form",
    "600 0/1 form",
    "600 0/2 form",
    "600 0/3 form",
    "600 0/4 form",
    "600 0/5 form",
    "600 1/-1 leader:aggression[0]",
    "600 1/1 form",
    "600 1/2 form",
    "600 1/3 form",
    "600 1/4 form",
    "600 1/5 form",
];

/// Chapter forty-four's widening rows: run484 whole, 605 to 1450 (item
/// 1257; `docs/GOLDEN.md` §53). The standing families only — the birth
/// `form` (parked 1169), the first block's census (parked 1183) and
/// `filled_gather_slots`, the Bomb Vessel's attack `order:target`, which
/// this crate keeps on the unit — and chapter eight's `treaties[·]` 3 on
/// the first blow, on 671, where the first round now lands on both sides.
/// Item 1254's rows past 631 went with the release: `1/6`'s turret on
/// 631, the round and `0/2007`'s `damage` on 671, and the cascade from
/// 686. The rounds are compared field by field (item 1257), and all 381
/// agree.
const WANT_CH44: &[&str] = &[
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// **run496 whole, both directions** (item 1278, `docs/GOLDEN.md` §55):
/// every dumped record on every block, the leaders at `LEADERS=5`, and
/// who=0's `GROUPDATA` pool.
#[test]
fn chapter_forty_six_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_six_s_word_frame_is_widened_whole();
}

/// Chapter forty-six's widening rows (items 1278, 1291): run496 whole,
/// 605 to 2200 (`docs/GOLDEN.md` §55). **The chapter closes at 2200**:
/// the squad's landing agrees since item 1291 (a captain's `come_out`
/// takes its squad, so `eject_contents` runs the squad arm once).
///
/// - **Before 1902, the standing families only** (44 rows): the birth
///   `form` (parked 1169; the trained Citizen W `1/6`'s on 726 and X
///   `1/7`'s on 1401), the first block's census (parked 1183) and
///   `filled_gather_slots`, and chapter forty-three's rows under their
///   own frames — the pushed squad's `order:group.id` on 1302, the
///   barge's `path_recursion` on 1357, the treaty and raid stamps of X's
///   staging, the Chariot's turret on 1479, and X's `DEATH` record on
///   1487. `all_gathering`'s prune on 1049 agrees: `1/2`'s
///   `order:gather.wait` −1 and `1/2001`'s `gather_down` 2 on block 1050.
/// - **From 1902, nothing.** Item 1278 pinned 305 rows here and 6 pool
///   rows on slot 3 (ours stacked three moves on `0/7`, two on `0/8` and
///   one on `0/9`); all agree.
const WANT_CH46: &[&str] = &[
    "1404 0/-1 leader:treaties[1]",
    "1404 1/-1 leader:treaties[0]",
    "1487 1/7 death:extra",
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "704 0/10 form",
];

/// Chapter forty-six's pool rows (item 1278): run496 whole, all the
/// word's — the squad's slot 3 on 1902 (`order_num` ours 3 against 1, and
/// the members' formation offsets mirrored) and its `speed` on 1922.
const WANT_CH46_POOL: &[&str] = &[];

/// **run514 whole, both directions** (item 1310, `docs/GOLDEN.md` §56):
/// every dumped record on every block, the leaders at `LEADERS=5`, and
/// who=0's `GROUPDATA` pool.
#[test]
fn chapter_forty_seven_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_seven_s_word_frame_is_widened_whole();
}

/// **run551 whole, both directions** (item 1358, `docs/GOLDEN.md` §57):
/// every dumped record on every block, the leaders at `LEADERS=5`, the
/// rounds at `AMMO=5`, and who=0's `GROUPDATA` pool.
#[test]
fn chapter_forty_eight_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_eight_s_word_frame_is_widened_whole();
}

/// Chapter forty-eight's widening rows (items 1358, 1375, 1380): run551
/// whole, 605 to 1750 (`docs/GOLDEN.md` §57). **The word is 1750, the
/// trace's end** (item 1380; 1481 before it), and these are pinned as
/// they stand, 55 rows (398 before it). **Where it moved, 1481 → 1750**
/// (item 1380): `1/2006`'s `ammo[0].total_time` (1451), the Scout's
/// wound and its cascade of 1457..1733, and `ammo[3]`'s landing on 1482
/// are gone — a Tower in the first three ages flies its arrow at
/// `unit_move_speed × 0x5a` (`BuildData::get_shot@0062dd90` is 0), and
/// a building's shot takes `Ammo::init`'s lead on a moving target. What
/// stands is the Bomber's six rounds on 1468..1482 (`accuracy`, `ex`,
/// `sx`, and the original's `ox`/`whom` kept after the hit where ours
/// forgets them), the city and form rows from 605, `raid_stamp` on 1490,
/// the three Pikemen's `order:target` on 901 and the two `treaties` on
/// 935 — none of which parts a draw.
///
/// **The arms, read off the dump by hand**: block 683, the site's first
/// hit on who=0's land, `damage` 0 → 8 and `job_counter` 11149 → 11332,
/// one frame's work (attrition costs no progress); blocks 1094..1099, the
/// first bombs, `damage` 178 → 242 and `job_counter` climbing by the
/// builders' quarter rate (an aircraft's blow costs none); block 983, the
/// original's fourth blow, 104/15 → 113/4 and 450 of progress (the carry).
const WANT_CH48: &[&str] = &[
    "1468 0/6 ammo[0].accuracy",
    "1468 0/6 ammo[0].ex",
    "1468 0/6 ammo[0].sx",
    "1474 0/6 ammo[6].ex",
    "1474 0/6 ammo[6].ez",
    "1474 0/6 ammo[6].sx",
    "1474 0/6 ammo[6].v1z",
    "1479 0/6 ammo[2].ex",
    "1479 0/6 ammo[2].sx",
    "1482 0/6 ammo[0].ox",
    "1482 0/6 ammo[0].whom",
    "1482 0/6 ammo[1].ox",
    "1482 0/6 ammo[1].whom",
    "1482 0/6 ammo[2].ox",
    "1482 0/6 ammo[2].whom",
    "1482 0/6 ammo[5].ox",
    "1482 0/6 ammo[5].whom",
    "1482 0/6 ammo[6].ox",
    "1482 0/6 ammo[6].whom",
    "1482 0/6 ammo[7].ox",
    "1482 0/6 ammo[7].whom",
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// Chapter forty-eight's pool rows (item 1358): who=0's slots, the
/// standing `ox`/`oy` family (`-1` against `0`) on 642, 904 and 952, and
/// slot 0's `speed`/`new_speed` on 952.
const WANT_CH48_POOL: &[&str] = &[];

/// Chapter forty-seven's widening rows (item 1310): run514 whole, 605 to
/// 1400 (`docs/GOLDEN.md` §56). **The chapter opens with a parting at
/// 712**, now 838 (item 1323), and these are pinned as they stand, 539
/// rows:
///
/// - the standing families on 605 (parked 1169, 1183) and the births'
///   `form` (607, 609, 681, 708); the squad's `order:group.id` on 614;
/// - **680**, who=0's three builders walking off under `@move`: ours'
///   `order:kind` 19 against 1 on `0/9`, and who=1's squad on 681 takes
///   `0/9` for its target where the original takes `0/11`, whose walks
///   part from 685;
/// - **706**, the site's first blow: `0/2008`'s `damage` 3 against 4 and
///   `job_counter` 9900 against 9850;
/// - ~~**713**, the word's block: who=1's `1/6` `hits:damage` 8 against 0,
///   the site's round (ours alone) landed on 712; from 744, the
///   cascade~~ — closed by item 1323 (a site does not shoot), 517 → 539
///   rows, every one from 713 to 837 gone and none opened before the
///   word;
/// - **838**, the word's walk-back: `0/9`'s `orders.len` ours 1 against
///   0, `orders_x/orders_y` (3864, 36888) against (3840, 36864); from
///   839, the cascade.
///
/// **The original converts the site**, read off the dump by hand:
/// `0/2008`'s `gpiece` 51364 → 51365, `myhits` 750 → 1000 and
/// `construct_hits` 71 → 96 on block 677. The widening reads neither a
/// building's `gpiece` nor its `myhits` (the `UNREAD` pin) and not its
/// `construct_hits` (the compared pin): what it holds of the arm is the
/// site's death — the original's on 901, ours on 965, and with the site
/// left a Tower (1264's mutation B) ours on 901 and 480 rows.
///
/// **Item 1350: 449 → 40 rows, the chapter closed.** A site holds half
/// its armour and loses `lost × 50` of progress (`docs/COMBAT.md` §4.2):
/// the site's 706 rows and its death row on 902 close, and so does every
/// row of the cascade after 904. What stands is the families of 605,
/// the squad's `order:group.id` on 614, who=1's squad's `order:target` on
/// 682 (ours none against the site), the treaties on 706, the barge's
/// birth `form` on 708, and two rows past the site's death on 1001 and
/// 1021.
const WANT_CH47: &[&str] = &[
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "708 0/12 form",
];

/// Chapter forty-seven's pool rows (item 1310): run514 whole — slot 0,
/// the builders' `@build` group on 622..624 (`ox`/`oy`, `facing`) and
/// their `@move` group on 680 (`form`, the offsets).
const WANT_CH47_POOL: &[&str] = &[];

/// **run466 whole, both directions** (items 1223, 1235, 1248,
/// `docs/GOLDEN.md` §52): every dumped record on every block, the leaders
/// at `LEADERS=5`, and who=0's `GROUPDATA` pool.
#[test]
fn chapter_forty_three_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_three_s_word_frame_is_widened_whole();
}

/// Chapter forty-three's widening rows (items 1223, 1235, 1248): run466
/// whole, the chapter **closed** at 2200 (`docs/GOLDEN.md` §52). Every row
/// is a standing family — the birth `form` (parked 1169), the first
/// block's census (parked 1183) and `filled_gather_slots`, the pushed
/// squad's `order:group.id` on 1302, the barge's `path_recursion` on
/// 1357, the treaty and raid stamps of the who=1 Citizen's staging, the
/// Chariot's turret on 1479 — and the Citizen's `DEATH` record on 1488,
/// which the capture does not print at its end detail. Item 1248 took the
/// dead Citizen off its gather list the frame it died (`gather_down` 2 on
/// 1488) and with it every row from 1553; and the 1900 line's refusal
/// (the DLL's refusal 3) every row from 1902.
const WANT_CH43: &[&str] = &[
    "1404 0/-1 leader:treaties[1]",
    "1404 1/-1 leader:treaties[0]",
    "1488 1/6 death:extra",
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "704 0/10 form",
];

/// Chapter forty-three's pool rows (items 1223, 1235, 1248): none, run466
/// whole. Item 1223's six on the squad's slot 2 on block 1357 were the
/// members this crate left ashore; boarded whole, the slot agrees, and so
/// do the two after it. The squad's slot 2 holds `[7, 8, 9]` to the end
/// once the 1900 line is refused (item 1248).
const WANT_CH43_POOL: &[&str] = &[];

/// Chapter forty-two's widening rows (item 1209): run460 whole, the
/// chapter **closed** at 1160 (`docs/GOLDEN.md` §51). Every row is a
/// standing family: the birth `form` (parked 1169) on every unit, the
/// first block's `filled_gather_slots`, the human City's census (parked
/// 1183), the Knight's attack `order:target`, which this crate keeps on
/// the unit, and chapter eight's `treaties[·]` 3 on the first blow (623).
/// `agendas[·]` is compared on every block and agrees: who=0's slot for
/// who=1 turns 2 on 812.
const WANT_CH42: &[&str] = &[
    "605 0/-1 leader:aggression[1]",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/-1 leader:aggression[0]",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// Chapter forty-one's widening rows (items 1182, 1200): run437 whole,
/// the chapter **closed** at 1770 (`docs/GOLDEN.md` §50). What stands is
/// chapter forty's families — the birth `form` (parked 1169) on every
/// unit, the first block's `filled_gather_slots`, the human City's census
/// (parked 1183), the trireme's attack `order:target`, which this crate
/// keeps on the unit rather than the order — and three of the chapter's
/// own: `1/2000`'s `raid_stamp` on 671 (closed by item 2235, a citizen's gather); the
/// `group` numbers from 902, which this crate hands out in another order;
/// and the death-object cull (`docs/COMBAT.md` §59.7), which this crate
/// does not have — the original clears `0/7`'s record 643 frames after
/// its death and `0/8`'s 656 after, and here both stay.
const WANT_CH41: &[&str] = &[
    // Item 1502: an attack's `order:target` on a building the start dump did
    // not link is named by `(owner, index)` (parked 681; `docs/AI.md`
    // §131.4); these rows agree and leave: 632 0/6 order:target.
    "1622 0/7 death:extra",
    "1725 0/8 death:extra",
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
    "902 1/6 group",
    "902 1/7 group",
    "968 1/0 group",
    // **The rounds' fields, compared from item 1257**: the Biplane
    // `1/6`'s rounds keep no target here (`whom`/`ox` −1) where the
    // original's name the Citizen it strafes (979, 1069, 1615). No draw.
    "979 1/6 ammo[0].ox",
    "979 1/6 ammo[0].whom",
    "979 1/6 ammo[2].ox",
    "979 1/6 ammo[2].whom",
    "1069 1/6 ammo[3].ox",
    "1069 1/6 ammo[3].whom",
    "1615 1/6 ammo[1].ox",
    "1615 1/6 ammo[1].whom",
];

/// **run430 whole, both directions** (item 1167, `docs/GOLDEN.md` §49):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_forty_s_word_frame_is_widened_whole() {
    cases_40_49::chapter_forty_s_word_frame_is_widened_whole();
}

/// Chapter forty's widening rows (item 1167): run430 whole, the casts'
/// other arms built. Three standing families and nothing of the
/// chapter's: `Unit::init`'s birth `form` (9 for a Citizen, 0 else,
/// against this crate's −1; parked 1169) on every unit of the start and
/// every one the script adds; the first block's `filled_gather_slots`;
/// and the human City's census — `CITIES=5` is new to a golden widening,
/// and this crate keeps `busy`, `filled`, `gatherers`, `land`, `space`,
/// `ter`, `peasant_dist` and `free` for a computer's cities only
/// (`crate::ai`), so `0/2000` reads 0 on each (`free` from 801, the first
/// block it is not 0 there) and the computer's `1/2000` is one cell off
/// on `filled` and `land` from the first block.
const WANT_CH40: &[&str] = &[
    "605 0/0 form",
    "605 0/1 form",
    "605 0/2 form",
    "605 0/3 form",
    "605 0/4 form",
    "605 0/5 form",
    "605 1/1 form",
    "605 1/2 form",
    "605 1/3 form",
    "605 1/4 form",
    "605 1/5 form",
];

/// Chapter forty's pool rows (item 1167): each press's pushed selection's
/// `ox`/`oy` (0 against −1, chapters nineteen's, thirty-four's and
/// thirty-nine's family); the `speed`/`new_speed` refreshes the original
/// makes off its cursor's frame (chapter thirty-nine's parked pair); and
/// slot 4's `form` ~~on 702, D's group, which takes the Citizen's birth
/// `form` 9 (parked 1169)~~ — agreeing from 702 since item 1330 bore the
/// Citizen in form 9, and parting on 802, where the `@spell`'s group reads
/// −1 in the original and 9 here.
const WANT_CH40_POOL: &[&str] = &[
    "802 slot 4 form",
    "882 slot 4 new_speed",
    "882 slot 4 speed",
];

/// **run422 whole, both directions** (item 1111, `docs/GOLDEN.md` §48):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_nine_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_nine_s_word_frame_is_widened_whole();
}

/// Chapter thirty-nine's widening rows (item 1111): run422 whole, the
/// three casts built. What stands is `Unit::init`'s birth `form` (9 for a
/// Citizen, 0 else, against this crate's −1) on every unit the script adds
/// and on the six decoys of 821 — the standing row of every chapter; a cast
/// writes the byte, and from each conversion on it agrees.
const WANT_CH39: &[&str] = &[];

/// Chapter thirty-nine's pool rows (item 1111): each press's pushed
/// selection's `ox`/`oy` (0 against −1, chapters nineteen's and
/// thirty-four's family), and two `speed`/`new_speed` refreshes the
/// original makes off its cursor's frame — slot 0 (B, a Militia since
/// 646) at 24 on 662, slot 2 (C, a Citizen since 666) at 25 on 702 —
/// where this crate's cursor reaches them on 704 and 706.
const WANT_CH39_POOL: &[&str] = &[];

/// **run422's casts, field for field, both directions** (item 1111,
/// `docs/GOLDEN.md` §48). The fields the three untargeted crafts write that
/// no parser carries (`diff::coverage` pins them unread for the widenings),
/// read raw from every block of [`WIDENING_CHAPTER_THIRTY_NINE`] on every
/// unit of either player: `rare` (the former type To Arms keeps and
/// Civilian reads), `mana_burn` (the General's 1,000 and each decoy's
/// age), `spell_time`, `cavarch_o`/`cavarch_who` (−1/−1 after an
/// untargeted press), the decoy bit `unit_masks & 1` and the started bit
/// `& 0x20000`; and on every `CASTORDER` either side holds, its target,
/// its point, its craft and `paid`. Nothing parts.
#[allow(clippy::type_complexity)]
fn casts_field_for_field(
    run: &str,
    stem: &str,
    (first, last): (i64, i64),
) -> Option<(
    usize,
    usize,
    std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
)> {
    use std::collections::BTreeMap;
    let mut s = stage_script(run, stem)?;
    let players = s.built.sim.players.len() as i64;
    let mut firsts: BTreeMap<(i64, i64, String), (i64, String)> = BTreeMap::new();
    let mut rows = 0usize;
    let mut decoys = 0usize;
    for f in 0..last - 1 {
        s.script.stage(s.built.sim.frame, &mut s.built, &s.loaded);
        s.built.tick();
        let n = f + 1;
        if n < first {
            continue;
        }
        let Some(at) = s.ix.frames().iter().position(|x| x.number == n) else {
            continue;
        };
        let frame = s.ix.frame_state(at).unwrap();
        let raw = s.ix.read_frame(at).unwrap();
        let recs = raw_records(&raw);
        let sim = &s.built.sim;
        let target_o = |o: Option<sim::combat::Obj>| match o {
            Some(sim::combat::Obj::Unit(x)) => i64::from(sim.units[x].index),
            Some(sim::combat::Obj::Building(b)) => i64::from(sim.buildings[b].index),
            None => -1,
        };
        let mut alone: Vec<(i64, i64, &str)> = Vec::new();
        let mut note = |who: i64, o: i64, what: &str, ours: i64, theirs: Option<i64>| {
            let Some(theirs) = theirs else {
                return;
            };
            rows += 1;
            if ours != theirs {
                firsts
                    .entry((who, o, what.to_string()))
                    .or_insert((n, format!("ours {ours} theirs {theirs}")));
            }
        };
        for them in &frame.units {
            if !(0..players).contains(&them.who) {
                continue;
            }
            let (Ok(w), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                continue;
            };
            let Some(u) = sim.unit_by_o(w, o) else {
                continue;
            };
            let un = &sim.units[u];
            let r = recs.get(&("UNITDATA", them.who, them.o));
            let get = |k: &str| r.and_then(|r| r.get(k).copied());
            note(them.who, them.o, "rare", i64::from(un.rare), get("rare"));
            note(
                them.who,
                them.o,
                "mana_burn",
                i64::from(un.mana_burn),
                get("mana_burn"),
            );
            note(
                them.who,
                them.o,
                "spell_time",
                i64::from(un.spell_time),
                get("spell_time"),
            );
            note(
                them.who,
                them.o,
                "cavarch_o",
                sim.cavarch_o(u),
                get("cavarch_o"),
            );
            note(
                them.who,
                them.o,
                "cavarch_who",
                i64::from(un.cavarch_who),
                get("cavarch_who"),
            );
            let masks = them.unit_masks;
            note(
                them.who,
                them.o,
                "decoy",
                i64::from(un.decoy),
                masks.map(|m| m & 1),
            );
            if un.decoy {
                decoys += 1;
            }
            note(
                them.who,
                them.o,
                "casting",
                i64::from(un.casting),
                masks.map(|m| i64::from(m & 0x20000 != 0)),
            );
            let ours_cast = un.orders.iter().find_map(|x| match x.body {
                sim::orders::Body::Cast(c) => Some(c),
                _ => None,
            });
            let theirs_cast = them.orders.iter().find(|x| x.kind == "CASTORDER");
            match (ours_cast, theirs_cast) {
                (Some(c), Some(t)) => {
                    note(them.who, them.o, "cast.ox", target_o(c.target), t.ox);
                    note(them.who, them.o, "cast.x", i64::from(c.at.x), t.x);
                    note(them.who, them.o, "cast.y", i64::from(c.at.y), t.y);
                    note(
                        them.who,
                        them.o,
                        "cast.spell",
                        i64::from(c.spell),
                        t.cast_spell,
                    );
                    note(
                        them.who,
                        them.o,
                        "cast.paid",
                        i64::from(c.paid),
                        t.cast_paid,
                    );
                }
                (None, None) => {}
                (mine, _) => alone.push((
                    them.who,
                    them.o,
                    if mine.is_some() {
                        "this crate"
                    } else {
                        "the dump"
                    },
                )),
            }
        }
        for (who, o, side) in alone {
            firsts
                .entry((who, o, "cast".into()))
                .or_insert((n, format!("{side} holds it alone")));
        }
    }
    Some((rows, decoys, firsts))
}

/// run422's casts through [`casts_field_for_field`] (item 1111): nothing
/// parts.
#[test]
fn run422_s_casts_are_the_original_s_field_for_field() {
    cases_setup::run422_s_casts_are_the_original_s_field_for_field();
}

/// **run430's casts, field for field, both directions** (item 1167,
/// `docs/GOLDEN.md` §49), through [`casts_field_for_field`] over
/// [`WIDENING_CHAPTER_FORTY`]: D's refused press on who=1's land and its
/// two wounded conversions, the three Militia's Civilians ahead of a
/// repair, a build and a gather (each `CASTORDER`'s point, craft and
/// `paid`), and M4's `rare` through the all-clear. Nothing parts.
#[test]
fn run430_s_casts_are_the_original_s_field_for_field() {
    cases_setup::run430_s_casts_are_the_original_s_field_for_field();
}

/// **run404 whole, both directions** (item 1102, `docs/GOLDEN.md` §47):
/// every dumped record on every block of the capture. run404 printed no
/// pool (`DEATHS=1` beside `GROUPS=1`), so there is none to compare.
#[test]
fn chapter_thirty_eight_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_eight_s_word_frame_is_widened_whole();
}

/// **The word's block, the Battery's turret** (item 1117, `docs/COMBAT.md`
/// §85): run404's block 780, the frame after the word. The Battery `1/9`
/// is on `CHAR_ATTACK2`'s frame 4, the frame of its node-0 release, and
/// its turret 0 is still short of its aim — `node_flags` 14 (bit 0 clear:
/// turrets 1–3 sit on their zero aims), `des_node_flags` 1 (one
/// `<RESTRICTION>` row, node 4). Both sides' four fields, read off the
/// block and off the figure, and the dump's own values pinned so the test
/// cannot pass on an empty record.
#[test]
fn chapter_thirty_eight_s_battery_is_short_of_its_aim_on_the_word_s_block() {
    cases_30_39::chapter_thirty_eight_s_battery_is_short_of_its_aim_on_the_word_s_block();
}

/// **The word's squad, run404 whole** (item 1113, `docs/GROUPS.md` §34):
/// the Infantry `1/6` and its followers `1/7` and `1/8` on the army's
/// `ATTACKTO` of 764.
///
/// - **Block 765, the scatter**: each member's order point, both sides,
///   and the dump's own values pinned — `1/7` (38568, 13368) and `1/8`
///   (38712, 13176), where ours stood a cell past each (13416; 38760,
///   13224) until `Form::compute_dests`' modern-infantry scatter.
/// - **Every block, the pack**: the current move's `retry` and
///   `attempts`, both sides, on every block both hold one — two fields the
///   compared pin leaves to run29's field table on the move row. run404
///   packs twenty times, each on its unit's own phase `(o · 0x11 + f) &
///   0x7f == 0`: `1/7` first on 777 (block 778), `1/6` on 794, `1/8` on
///   888, `retry` 22 and `attempts` −3 on the block after each.
#[test]
fn chapter_thirty_eight_s_squad_stands_on_its_points_and_packs_on_its_phase() {
    cases_30_39::chapter_thirty_eight_s_squad_stands_on_its_points_and_packs_on_its_phase();
}

/// Chapter thirty-eight's widening rows (items 1102, 1109, 1112, 1117,
/// 1113, 1131): run404 whole, closed at 1762. The standing rows of every
/// golden start (`form`, `g.gpiece`, the gather slots, the idle clocks);
/// the Radar Air Defense's `constr_time` (30000 against 22556); the
/// Battery's aim (743), its bits (801) and its node-1 round's launch point
/// (784, `0/7`'s hit a frame late, parked 1125); the two crash rounds' `splash_area` (1007, 1020:
/// the pool slot's leftover, which `Ammo::init_crash` never writes — item 2609 laid the rounds,
/// and every other field of both agrees to their landing, `docs/AI.md` §228; the Battery round
/// on 1015 the dump held alone was the slot the crash took); the Barracks' and the
/// Radar's damage fractions on 1036; and the Battery's group id on 1277.
/// Item 1131 took 455 → 36: every Bomber row, the Radar's target and the
/// strikes' re-targets agree. Item 1200 took 36 → 30: the Battery's own
/// wounds from 940 agree once a unit type's `guy_radius` is its
/// `block_radius` (`UnitType::init@0061ab50:753`), the splash's inner
/// radius the loader had left at 0. Item 2677 took six more: `init_crash`
/// does write `splash_area` (`+0x30 = 2`), and the two crash rounds strike
/// where they come down (`docs/AI.md` §230), so both rounds' `splash_area`
/// and the Barracks' and the Radar's `damage` and `damage_frac` on 1036
/// agree.
const WANT_CH38: &[&str] = &[
    "619 1/2007 build:constr_time",
    "743 1/9 g.des_turret0[0]",
    "743 1/9 g.turret0[0]",
    "754 1/9 ammo[0].angle",
    "754 1/9 ammo[0].ez",
    "754 1/9 ammo[0].sx",
    "754 1/9 ammo[0].sy",
    "754 1/9 ammo[0].sz",
    "754 1/9 ammo[0].v1z",
    "756 1/9 ammo[1].angle",
    "756 1/9 ammo[1].ex",
    "756 1/9 ammo[1].ey",
    "756 1/9 ammo[1].ez",
    "756 1/9 ammo[1].sx",
    "756 1/9 ammo[1].sy",
    "756 1/9 ammo[1].sz",
    "756 1/9 ammo[1].total_time",
    "756 1/9 ammo[1].v1z",
    "761 1/9 ammo[1]",
    "783 1/9 ammo[0].ex",
    "783 1/9 ammo[0].ey",
    "783 1/9 ammo[0].total_time",
    "784 0/7 damage_frac",
    "784 0/7 damage_frame",
    "784 0/7 hits:damage",
    "784 0/7 hits:damage_frac",
    "784 0/7 hits_left",
    "784 1/9 ammo[0]",
    "786 0/7 ammo[0].accuracy",
    "787 0/7 ammo[1].accuracy",
    "798 1/2007 ammo[6].ez",
    "798 1/2007 ammo[6].v1z",
    "801 1/9 g.node_flags[0]",
    "806 0/7 ammo[6].accuracy",
    "808 1/2007 ammo[2].ez",
    "808 1/2007 ammo[2].v1z",
    "828 1/2007 ammo[0].ez",
    "828 1/2007 ammo[0].v1z",
    "832 0/6 ammo[0].accuracy",
    "833 0/6 ammo[1].accuracy",
    "836 0/6 ammo[2].accuracy",
    "838 1/2007 ammo[4].ez",
    "838 1/2007 ammo[4].v1z",
    "838 1/9 ammo[3].angle",
    "838 1/9 ammo[3].ez",
    "838 1/9 ammo[3].sx",
    "838 1/9 ammo[3].sy",
    "838 1/9 ammo[3].sz",
    "838 1/9 ammo[3].v1z",
    "840 1/9 ammo[6].angle",
    "840 1/9 ammo[6].ez",
    "840 1/9 ammo[6].sx",
    "840 1/9 ammo[6].sy",
    "840 1/9 ammo[6].sz",
    "840 1/9 ammo[6].v1z",
    "844 0/6 ammo[3].accuracy",
    "847 0/6 ammo[4].accuracy",
    "858 1/9 ammo[5].angle",
    "858 1/9 ammo[5].ez",
    "858 1/9 ammo[5].sx",
    "858 1/9 ammo[5].sy",
    "858 1/9 ammo[5].sz",
    "858 1/9 ammo[5].v1z",
    "888 1/2007 ammo[1].ez",
    "888 1/2007 ammo[1].v1z",
    "919 0/7 ammo[3].accuracy",
    "922 0/7 ammo[4].accuracy",
    "924 0/7 ammo[2].accuracy",
    "927 0/7 ammo[5].accuracy",
    "930 0/7 ammo[7].accuracy",
    "964 1/9 ammo[4].angle",
    "964 1/9 ammo[4].ez",
    "964 1/9 ammo[4].sx",
    "964 1/9 ammo[4].sy",
    "964 1/9 ammo[4].sz",
    "964 1/9 ammo[4].v1z",
    "968 1/2007 ammo[5].ez",
    "968 1/2007 ammo[5].v1z",
    "969 0/6 ammo[6].accuracy",
    "975 0/6 ammo[5].accuracy",
    "980 0/6 ammo[7].accuracy",
    "989 1/9 ammo[2].angle",
    "989 1/9 ammo[2].ez",
    "989 1/9 ammo[2].sx",
    "989 1/9 ammo[2].sy",
    "989 1/9 ammo[2].sz",
    "989 1/9 ammo[2].v1z",
];

/// Chapter thirty-six's widening rows (item 1078): run390 whole. Chapter
/// thirty-five's forty-two, the spotter's and the two V2s' `form` (0
/// against −1), and the V2s' seats inside their silos on 2926 and 2931
/// (+24, +24: parked 646's family, run371's `0/10` again).
const WANT_CH36: &[&str] = &[
    "1000 0/7 ammo[7].accuracy",
    "1003 0/7 ammo[6].accuracy",
    "1061 0/8 ammo[5].accuracy",
    "1064 0/8 ammo[6].accuracy",
    "1069 0/8 ammo[4].accuracy",
    "1072 0/8 ammo[7].accuracy",
    "1080 0/8 ammo[0].ox",
    "1080 0/8 ammo[0].whom",
    "1080 0/8 ammo[1].ox",
    "1080 0/8 ammo[1].whom",
    "1080 0/8 ammo[2].ox",
    "1080 0/8 ammo[2].whom",
    "1080 0/8 ammo[3].ox",
    "1080 0/8 ammo[3].whom",
    "1080 0/8 ammo[4].ox",
    "1080 0/8 ammo[4].whom",
    "1080 0/8 ammo[7].ox",
    "1080 0/8 ammo[7].whom",
    "2446 0/11 mirror",
    "2659 0/12 g.last_z[0]",
    "2659 0/12 g.z[0]",
    "2675 0/12 dest_angle",
    "935 0/8 ammo[2].accuracy",
    "936 0/8 ammo[3].accuracy",
    "939 0/8 ammo[0].accuracy",
    "942 0/8 ammo[1].accuracy",
    "980 0/7 ammo[0].accuracy",
    "981 0/7 ammo[1].accuracy",
    "984 0/7 ammo[2].accuracy",
    "987 0/7 ammo[3].accuracy",
    "995 0/7 ammo[5].accuracy",
];

/// Chapter thirty-six's pool rows (item 1078).
const WANT_CH36_POOL: &[&str] = &[];

/// **The nuke's launch and round, field for field, both directions**
/// (item 1091, `docs/PRODUCTION.md` "The nuke (item 1091)"). On every
/// block of run397 from the strike to five past the landing,
/// [`NUKE_LAUNCH`]: the silo `0/2007`'s `recharging`, `launch_frames`,
/// `launching` and **`visible`** (−1 from 3082: the nuke arm's `+0x40`),
/// read raw; the nuke `0/10` on the map or gone; and every round it
/// fired, every field this crate carries — its landing on the target's
/// point, no scatter.
#[test]
fn chapter_thirty_seven_s_nuke_is_launched_and_fired_field_for_field() {
    cases_30_39::chapter_thirty_seven_s_nuke_is_launched_and_fired_field_for_field();
}

/// Chapter thirty-seven's launch rows (item 1091): none.
const WANT_CH37_NUKE: &[&str] = &[];

/// `chapter_thirty_seven_s_nuke_is_launched_and_fired_field_for_field`'s
/// window: the strike's block to five past the landing.
const NUKE_LAUNCH: (i64, i64) = (3050, 3206);

/// **The V2's launch, field for field, both directions** (item 1050,
/// `docs/PRODUCTION.md` "The missile's launch and round"). On every block
/// of run371 from the silo's strike to two blocks past the round's
/// release, [`V2_LAUNCH`]:
/// - the silo `0/2009`'s `recharging`, `launch_frames` and `launching`
///   (its length and first missile), read raw — no parser carries them;
/// - the V2 `0/10` on the map or gone, either side;
/// - every round `0/10` fired, keyed on its pool slot, every field this
///   crate carries: the release point and height, the landing point and
///   height, the time, the accuracy, the angle, `v1z`, the flags.
///
/// The order the V2 holds, 2672..2701, is the general widening's
/// (`crate::diff::order`'s air attack row).
#[test]
fn chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field() {
    cases_30_39::chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field();
}

/// **The V2's blast, field for field, both directions** (item 1077,
/// `docs/PRODUCTION.md` "The missile's launch and round", `docs/COMBAT.md`
/// §73). On every block of [`V2_BLAST`], run371's last two before the
/// round comes down on 2820 to two after:
/// - every building either side, on the map or gone, and its `damage`
///   where both hold it — the Barracks `1/2006` the round lands on among
///   them, 0 on 2820 and gone on 2821 in the dump;
/// - every unit either side, on the map or gone;
/// - the V2's round, held or closed.
///
/// The round has no target (`whom −1`); `Ammo::check_hit`'s
/// `find_building_at` on its landing tile makes the Barracks its own
/// target, struck whole rather than as a fringe.
#[test]
fn chapter_thirty_five_s_v2_blast_is_compared_field_for_field() {
    cases_30_39::chapter_thirty_five_s_v2_blast_is_compared_field_for_field();
}

/// What stands of the V2's blast on [`V2_BLAST`] (item 1077): nothing.
const WANT_CH35_BLAST: &[&str] = &[];

/// [`chapter_thirty_five_s_v2_blast_is_compared_field_for_field`]'s
/// window: run371's blocks 2818..2822, the round's landing on 2820 and
/// the Barracks gone on 2821.
const V2_BLAST: (i64, i64) = (2818, 2823);

/// **The missile's launch offset is the install's** (item 1050):
/// `<MISSILEOFFSET x y z>` under `EFFECTS` in `Data/effects_graphics.xml`,
/// which `GraphicEvents::init@008e5390` reads before the first frame, is
/// [`sim::air::MISSILE_OFFSET`]. The constant is a number from the user's
/// files; this is what re-derives it from them.
#[test]
fn the_missile_offset_is_the_install_s() {
    cases_setup::the_missile_offset_is_the_install_s();
}

/// What stands of the V2's launch on [`V2_LAUNCH`] (item 1050): nothing.
const WANT_CH35_V2: &[&str] = &[];

/// [`chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field`]'s
/// window: run371's blocks from the silo's strike (2670, the press) to two
/// past the round's release (2703).
const V2_LAUNCH: (i64, i64) = (2668, 2704);

// **What parts under the word** on run371, chapter thirty-five. **The
// first walk** (item 1019, before any build): 469 rows. Chapter
// thirty-two's standing seventeen (611..655, and the Biplane's birth on
// 1746); the silo's queue on 2227 (one V2 there, two here) and the
// buckets it charged; V2 `0/10`'s birth seat on 2431 (parked 646's
// family); and from 2446 the Helicopter `0/11` out at its birth, where
// here it stands inside — **the first parting's field list**, 2446,
// `0/11` alone, twenty keys: its berth, point, heading, `dest_angle`,
// `orders_x/y`, `idle`, `unit_masks` and its figure's point, height and
// clock. **After the build** (item 1019): 426 rows. The Helicopter's exit
// at its birth and the silo's one missile took the rest; what stands is
// the birth seats (`0/10` on 2431, `0/12` on 2659, parked 646's family),
// `0/11`'s `mirror` on 2446 (chapter thirty-three's unread writer of
// `unit_masks & 2`), and from 2672 the V2's strike and the second
// Helicopter's attack-move, parked with the word. **After item 1048**:
// 403 rows. The Helicopter's flight — the planners' straight path for a
// type that flies like a helicopter, `Unit::work`'s separation of the two
// and the figure's climb — took every `0/11` and `0/12` row on 2675 and
// 2676, and a row walk has `0/12`'s point, figure and height agreeing on
// every block from 2675 to the end; what stands of it is the birth seat's
// bearing (`dest_angle` and `order:move.angle` on 2659, the heading the
// arrival turns to on 2719). The rest from 2702 is the V2's round, which
// the draw stream now reaches (the word, 2701). **After item 1050**: 43
// rows. The V2's launch — its order on 2672, the silo's countdown, its
// step, round and end on 2701 — took `0/10`'s rows on 2672 and 2702, and
// with the draw stream agreeing to the end every row the misaligned
// stream had pushed after 2701 is gone. **The word's block**, 2701 and
// 2702 (the word, closed at 3260): no row; the V2's own record there is
// `chapter_thirty_five_s_v2_is_counted_out_and_fired_field_for_field`'s,
// 220 rows and none parting. What stood of it was the blast: 2821,
// `1/2006` gone there, at 400 damage here. **After item 1077**: 42 rows.
// The round, with no target, makes the Barracks under its landing tile
// its own target (`Ammo::check_hit`'s `find_building_at`), and strikes it
// whole: `1/2006` is gone on 2821 both sides. The blast's own record is
// `chapter_thirty_five_s_v2_blast_is_compared_field_for_field`'s, none
// parting. What stands is the birth seats and the second Helicopter's
// bearing, parked.
const WANT_CH35: &[&str] = &[
    "1000 0/7 ammo[7].accuracy",
    "1003 0/7 ammo[6].accuracy",
    "1061 0/8 ammo[5].accuracy",
    "1064 0/8 ammo[6].accuracy",
    "1069 0/8 ammo[4].accuracy",
    "1072 0/8 ammo[7].accuracy",
    "1080 0/8 ammo[0].ox",
    "1080 0/8 ammo[0].whom",
    "1080 0/8 ammo[1].ox",
    "1080 0/8 ammo[1].whom",
    "1080 0/8 ammo[2].ox",
    "1080 0/8 ammo[2].whom",
    "1080 0/8 ammo[3].ox",
    "1080 0/8 ammo[3].whom",
    "1080 0/8 ammo[4].ox",
    "1080 0/8 ammo[4].whom",
    "1080 0/8 ammo[7].ox",
    "1080 0/8 ammo[7].whom",
    "2446 0/11 mirror",
    "2659 0/12 g.last_z[0]",
    "2659 0/12 g.z[0]",
    "2675 0/12 dest_angle",
    "935 0/8 ammo[2].accuracy",
    "936 0/8 ammo[3].accuracy",
    "939 0/8 ammo[0].accuracy",
    "942 0/8 ammo[1].accuracy",
    "980 0/7 ammo[0].accuracy",
    "981 0/7 ammo[1].accuracy",
    "984 0/7 ammo[2].accuracy",
    "987 0/7 ammo[3].accuracy",
    "995 0/7 ammo[5].accuracy",
];

// **What parts in the pool** on run371: chapter twenty-two's ten, and
// slot 3's `ox`/`oy` on 2227 — the silo's building group, seated by
// `@queueup`, whose action writes no point: (0, 0) there, (−1, −1) here,
// parked 760 and 792's family, as chapter thirty-four's slot 3 on 2602.
const WANT_CH35_POOL: &[&str] = &[];

/// **run362 whole, both directions** (item 1009, `docs/GOLDEN.md` §43):
/// every dumped record on every block of the capture, and the pool.
#[test]
fn chapter_thirty_four_s_word_frame_is_widened_whole() {
    cases_30_39::chapter_thirty_four_s_word_frame_is_widened_whole();
}

// **What parts under the word** on run362, chapter thirty-four. **The
// first walk** (item 1009, before any build): 421 rows. Chapter
// thirty-two's standing seventeen (611..655, and the Biplane's birth on
// 1746); **the first parting's field list**, 2307, `0/7` alone: its
// stack (`orders.len`, `order:length`, ours 0, theirs 1 — the flight home
// to `0/2008`), `inside` (ours 2007, theirs −1), `pos`, `heading`,
// `dest_angle`, `orders_x/y`, `path:length`, and its figure's point,
// height, bank, pitch and clock. **Their writers** (889): the stack's is
// `action_launch_flight`'s `MOVE_TO` arm alone (`action_flight`'s flight
// home, `add_strafe_order`); the rest are `Object::do_launch`'s EXIT on
// the same block, which the order admits. Then 2314 (ours, `0/7` still
// inside, takes ctrl's strike), 2322 (the word), each flight after, and
// who=1's `wealth` on 2719. **The arms built** (item 1009,
// `docs/PRODUCTION.md` "The launch commands"): the `MOVE_TO` arm and the
// base's height clamped at 0 in the approach home (`0/7`'s descent to
// `0/2008`, whose tile is −42: 21 → 17 on 2399). 421 → 17: chapter
// thirty-two's seventeen alone.
const WANT_CH34: &[&str] = &[];

// **What parts in the pool** on run362: chapter twenty-two's ten, and
// slot 3's `ox`/`oy` on 2602 — the building group of `0/2008` seated by
// alt's press, whose action writes no point: (0, 0) there, (−1, −1)
// here, parked 760 and 792's family on a third chapter.
const WANT_CH34_POOL: &[&str] = &[];

/// **A staging walk** (item 1019, `docs/GOLDEN.md` §44): a chapter's
/// candidate script staged on a golden capture's start and ticked past
/// that capture's end, compared with nothing. It prints, on each frame
/// one changes, the aircraft and missiles of every player (their berth,
/// point, tank and order stack), each hangar and each building with a
/// queue or a gather list, and player 0's six buckets.
///
/// `RON_STAGE=<run>:<script>:<to frame>`, where `<script>` is a chapter's
/// stem under `tools/gamelog/golden/` or a path; skipped when unset. Items
/// 976 and 1009 each built this as a scratch prototype and lost it with
/// their lanes; the third reach graduates it here.
#[test]
fn stage_walk() {
    cases_setup::stage_walk();
}
