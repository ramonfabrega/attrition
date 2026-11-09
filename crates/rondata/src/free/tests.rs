//! The free runner on a whole pair: its scenes against the harness's, the
//! renderer's M counted from them (`docs/RENDERER.md` §5.4), and the
//! renderer's clock.
//!
//! **The pair is the Aztecs at a held Modern age, Great Lakes**
//! (`diff::aztec_modern_pair`, whole at 3389 of 3389). Three things chose it:
//! every capture of its lobby prints `gpiece` — run1428 is a `DUMP_ALL`
//! start, run1429 prints its units' guys every hundred frames
//! (`end:UNITS=3,GUYS=2`) and run1429b is a `GUYS` window over the game's
//! last 95 blocks — so M is checked against every frame the lobby's dumps
//! print a figure on; it is the shortest whole pair with a window on disk;
//! and a held Modern age puts the late pieces in play (tanks, aircraft,
//! the Jaguar Infantry and Sherman rows item 3766 closed the word on).

use super::*;
use crate::diff::aztec_modern_pair::{AZTEC_MODERN, RUN1429B};
use crate::diff::testkit::{Pins, pin_eq, trace};
use crate::gamelog::{Frame, UnitDump};
use crate::load::BASE_BUILDTYPES;
use crate::testenv::{dump, install};
use sim::observe::{ObjId, UnitScene};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The pair's captures on this machine, or `None` (the test then skips, as
/// every dump-backed test does).
struct Captures {
    loaded: Loaded,
    long: PathBuf,
    start: PathBuf,
    trace: crate::trace::Trace,
}

fn captures() -> Option<Captures> {
    let inst = install()?;
    let p = &AZTEC_MODERN;
    let (Some(long), Some(start), Some(trace)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        eprintln!("skipping: no {} captures (set RON_GAMELOG_DIR)", p.name);
        return None;
    };
    Some(Captures {
        loaded: crate::load::load(&inst).unwrap(),
        long: long.into(),
        start: start.into(),
        trace,
    })
}

impl Captures {
    fn runner(&self) -> FreeRunner {
        FreeRunner::from_captures(
            &self.loaded,
            &self.long,
            &[self.start.as_path()],
            Some(&self.trace),
            Tuning::RON,
        )
        .unwrap()
    }
}

/// The first field two scenes part on, for the failure message.
fn first_difference(h: &Scene, f: &Scene) -> String {
    if h.frame != f.frame {
        return format!("frame {} against {}", h.frame, f.frame);
    }
    let units = |s: &Scene| -> BTreeMap<ObjId, UnitScene> {
        s.units.iter().map(|u| (u.id, u.clone())).collect()
    };
    let (hu, fu) = (units(h), units(f));
    for id in hu.keys().chain(fu.keys()) {
        match (hu.get(id), fu.get(id)) {
            (Some(a), Some(b)) if a != b => {
                return format!("unit {}/{}: harness {a:?}\n  runner {b:?}", id.who, id.o);
            }
            (Some(_), None) => return format!("unit {}/{} only in the harness", id.who, id.o),
            (None, Some(_)) => return format!("unit {}/{} only in the runner", id.who, id.o),
            _ => {}
        }
    }
    if h.units != f.units {
        return "the units' order".into();
    }
    if h.buildings != f.buildings {
        return "the buildings".into();
    }
    if h.projectiles != f.projectiles {
        return "the projectiles".into();
    }
    if h.deaths != f.deaths {
        return "the deaths".into();
    }
    "the fog or the counters".into()
}

/// Ticks of the whole pair on which the runner's scene equals the harness's,
/// of the ticks run (item 4020): `FRAME 1` through the closing block 3390.
pub(crate) const SCENE_TICKS_AGREEING: (usize, usize) = (3390, 3390);

/// The capture's logged frames (run1429's `FRAME` records) among them.
pub(crate) const SCENE_LOGGED_FRAMES_AGREEING: (usize, usize) = (35, 35);

/// **The precondition sub-score** (`docs/RENDERER.md` §6 item 1): on the
/// pair, uncorrected from frame 1, the free runner's scene equals the
/// harness's on every tick — every unit's position, tile height, facing and
/// health, every figure's position, `gpiece`, `anim`, `cur_time` and
/// `end_time`, the buildings, rounds, corpses and counters — with zero
/// differences. The harness is `diff::Built::tick` stood up as
/// `diff::run_traced_observed` stands it, from the whole long capture's
/// initial with run1428 and the trace's pasture borrowed, so its installs
/// (the start block's sync word and figure clocks) are live; the runner is
/// built from the setup blocks alone and installs nothing.
#[test]
fn the_free_run_s_scenes_are_the_harness_s() {
    let _pins = Pins::hold();
    let Some(c) = captures() else {
        return;
    };
    let text = crate::capture::read(&c.long);
    let log = Log::parse(&text);
    let start_text = crate::capture::read(&c.start);
    let start_log = Log::parse(&start_text);
    let sibling = start_log.replay_initial().unwrap();
    let mut init = log.replay_initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &c.trace);
    let mut harness = build_sim(&c.loaded, &init, Tuning::RON);
    drop(init);
    let mut free = c.runner();
    let mut logged = Vec::new();
    log.visit_frame_states(None, |f| logged.push(f.n));
    let last = *logged.last().unwrap();
    pin_eq!(
        harness.sim.observe(None),
        free.scene(None),
        "frame 0, before the first tick"
    );
    let (mut ticks, mut agree, mut logged_agree) = (0, 0, 0);
    let mut first = None;
    while free.frame() < last {
        harness.tick();
        free.step();
        let (h, f) = (harness.sim.observe(None), free.scene(None));
        assert_eq!(f.frame, free.frame(), "the scene's frame is the runner's");
        ticks += 1;
        if h == f {
            agree += 1;
            logged_agree += usize::from(logged.contains(&free.frame()));
        } else if first.is_none() {
            first = Some(format!("frame {}: {}", f.frame, first_difference(&h, &f)));
        }
    }
    eprintln!(
        "SCENE {}: {agree} of {ticks} ticks agree, {logged_agree} of {} logged frames; first {first:?}",
        AZTEC_MODERN.name,
        logged.len()
    );
    pin_eq!(first, None::<String>, "the first parting");
    pin_eq!(
        (agree, ticks),
        SCENE_TICKS_AGREEING,
        "ticks agreeing, of ticks run"
    );
    pin_eq!(
        (logged_agree, logged.len()),
        SCENE_LOGGED_FRAMES_AGREEING,
        "logged frames agreeing"
    );
}

/// Every figure a dumped unit holds, `((who, o), guy) → gpiece`.
fn dumped_figures(units: &[UnitDump], out: &mut BTreeMap<(ObjId, usize), i32>) {
    for u in units {
        let id = ObjId {
            who: u.who as sim::Player,
            o: u.o as i16,
        };
        for (n, g) in u.guys.iter().enumerate() {
            if let Some(p) = g.gpiece {
                out.insert((id, g.guy_num.map_or(n, |k| k as usize)), p as i32);
            }
        }
    }
}

/// What one dump of the lobby prints and what the runner holds on the same
/// frames.
#[derive(Debug, Default, PartialEq, Eq)]
struct DumpCheck {
    /// Frames the dump prints a figure on.
    frames: usize,
    /// Figures the dump prints whose unit is on the runner's map.
    figures: usize,
    /// Of them, the figures whose piece the runner's scene holds.
    agree: usize,
    /// Figures of units the scene does not carry (not on the map).
    off_map: usize,
    /// The distinct pieces the dump prints.
    pieces: usize,
    /// The distinct pieces the runner's scenes hold on those frames.
    runner_pieces: usize,
    /// The distinct building types (`orig_type`) the dump holds.
    building_types: usize,
    /// Building records the dump prints, and of them those whose type the
    /// runner's building of the same `(who, o)` holds.
    buildings: usize,
    buildings_agree: usize,
}

/// **M, `docs/RENDERER.md` §5.4's denominator**: the distinct pieces the
/// pair's own run plays, counted from the free runner's scenes on every
/// tick from frame 0 to the closing block, plus the distinct building types
/// the pair's captures hold. Item 4020's count; the renderer's K of M is
/// booked against it.
pub(crate) const RENDER_M: usize = RENDER_M_PIECES + RENDER_M_BUILDING_TYPES;

/// M's pieces: the distinct `gpiece` of every figure of every unit on the
/// map, over the runner's 3,391 scenes, frame 0 to the closing block —
/// `-1`, a figure with no piece, left out (item 4020: 39).
pub(crate) const RENDER_M_PIECES: usize = 39;

/// M's building types: the distinct `orig_type` of the captures' building
/// records, the start's and run1429b's window (run1429 prints none past the
/// start; item 4020: 21 — 5 in run1428's start, 21 in run1429's and in the
/// window).
pub(crate) const RENDER_M_BUILDING_TYPES: usize = 21;

/// The building types the runner's own scenes hold over the whole run —
/// beside M, not in it: §5.4 counts the dumps' (item 4020: 23, every one
/// of the dumps' 21 among them). The dumps print only `orig_type`, so a
/// building's type after an upgrade is the runner's alone.
pub(crate) const RUNNER_BUILDING_TYPES: usize = 23;

/// Of M's pieces, those whose model and every slot's animation decode
/// under `render`'s readers (`artdata::piece_decoding`, item 4019's K), of
/// M's pieces (item 4020: 32 of 39). The seven short: the six gaia pieces
/// 60063–60065 and 60072–60074, which no `<UNIT>` entry names (4019's walk
/// is over `unit_graphics.xml`'s entries), and 38322,
/// `THECITIZEN-DEFAULT-AGE0-CREW3`, an entry with a failing file.
pub(crate) const M_PIECES_DECODING: (usize, usize) = (32, 39);

/// **M is counted from the runner and agrees with every dump of the
/// lobby** (`docs/RENDERER.md` §5.4): on each frame run1428, run1429 or
/// run1429b prints a figure, every figure of a unit the runner has on the
/// map holds the dumped piece, and the distinct pieces both sides hold on
/// those frames are the same set.
#[test]
fn m_is_counted_from_the_free_run_and_agrees_with_the_lobby_s_dumps() {
    let _pins = Pins::hold();
    let Some(c) = captures() else {
        return;
    };
    let Some(window) = dump(RUN1429B) else {
        eprintln!("skipping: no {RUN1429B}");
        return;
    };
    let names = [AZTEC_MODERN.start, AZTEC_MODERN.long.0, RUN1429B];
    let paths = [c.start.clone(), c.long.clone(), PathBuf::from(window)];
    // Per dump: frame → the figures and the buildings it prints.
    type Printed = BTreeMap<i64, (BTreeMap<(ObjId, usize), i32>, Vec<(ObjId, i64)>)>;
    let mut printed: Vec<Printed> = Vec::new();
    for path in &paths {
        let mut src = IndexedCapture::open(path).unwrap();
        let mut by_frame = Printed::new();
        let setup = src.read_replay_setup().unwrap();
        let setup_log = Log::parse(&setup);
        let init = setup_log.replay_initial().unwrap();
        let mut put = |n: i64, units: &[UnitDump], builds: &[crate::gamelog::BuildDump]| {
            let mut figs = BTreeMap::new();
            dumped_figures(units, &mut figs);
            let blds = builds
                .iter()
                .filter_map(|b| {
                    let id = ObjId {
                        who: b.who as sim::Player,
                        o: b.o as i16,
                    };
                    b.orig_type.map(|t| (id, t))
                })
                .collect::<Vec<_>>();
            if !figs.is_empty() || !blds.is_empty() {
                by_frame.insert(n, (figs, blds));
            }
        };
        put(0, &init.units, &init.builds);
        for frame in src.frame_states() {
            let f: Frame = frame.unwrap();
            put(f.n, &f.units, &f.builds);
        }
        printed.push(by_frame);
    }
    let last = AZTEC_MODERN.length + 1;
    let mut checks: Vec<DumpCheck> = printed
        .iter()
        .map(|p| DumpCheck {
            frames: p.values().filter(|(f, _)| !f.is_empty()).count(),
            ..DumpCheck::default()
        })
        .collect();
    let mut runner_on: Vec<BTreeSet<i32>> = vec![BTreeSet::new(); paths.len()];
    let mut dump_pieces: Vec<BTreeSet<i32>> = vec![BTreeSet::new(); paths.len()];
    let mut dump_types: BTreeSet<i64> = BTreeSet::new();
    let (mut pieces, mut runner_types) = (BTreeSet::new(), BTreeSet::new());
    let (mut mismatches, mut building_mismatches) = (Vec::new(), Vec::new());
    let mut see = |s: &Scene| {
        // `-1` is a figure with no piece: nothing to draw.
        for u in &s.units {
            pieces.extend(u.figures.iter().map(|f| f.gpiece).filter(|p| *p >= 0));
        }
        runner_types.extend(
            s.buildings
                .iter()
                .filter_map(|b| b.ty.map(|t| BASE_BUILDTYPES as i64 + t as i64)),
        );
        for (i, p) in printed.iter().enumerate() {
            let Some((figs, blds)) = p.get(&s.frame) else {
                continue;
            };
            dump_types.extend(blds.iter().map(|(_, t)| *t));
            for &(id, t) in blds {
                checks[i].buildings += 1;
                match s.buildings.iter().find(|b| b.id == id) {
                    Some(b) if b.ty.map(|k| BASE_BUILDTYPES as i64 + k as i64) == Some(t) => {
                        checks[i].buildings_agree += 1;
                    }
                    other => {
                        building_mismatches.push((id, t, other.and_then(|b| b.ty)));
                    }
                }
            }
            let ours: BTreeMap<ObjId, &UnitScene> = s.units.iter().map(|u| (u.id, u)).collect();
            for (&(id, n), &piece) in figs {
                let Some(u) = ours.get(&id) else {
                    checks[i].off_map += 1;
                    continue;
                };
                dump_pieces[i].insert(piece);
                checks[i].figures += 1;
                match u.figures.get(n) {
                    Some(f) if f.gpiece == piece => {
                        checks[i].agree += 1;
                        runner_on[i].insert(f.gpiece);
                    }
                    other => mismatches.push(format!(
                        "{} frame {} {}/{} guy {n}: dump {piece}, runner {:?}",
                        names[i],
                        s.frame,
                        id.who,
                        id.o,
                        other.map(|f| f.gpiece)
                    )),
                }
            }
        }
    };
    let mut free = c.runner();
    see(&free.scene(None));
    free.run_to(last, None, &mut see);
    for (i, check) in checks.iter_mut().enumerate() {
        check.pieces = dump_pieces[i].len();
        check.runner_pieces = runner_on[i].len();
        check.building_types = printed[i]
            .values()
            .flat_map(|(_, b)| b.iter().map(|(_, t)| *t))
            .collect::<BTreeSet<_>>()
            .len();
        eprintln!("M {}: {check:?}", names[i]);
    }
    let outside: Vec<_> = dump_pieces
        .iter()
        .flatten()
        .filter(|p| !pieces.contains(p))
        .collect();
    let m = pieces.len() + dump_types.len();
    eprintln!(
        "M {}: {} pieces + {} building types the dumps hold = {m}; runner's building types {}; \
         dumps' types the runner never held {:?}; mismatches {} {:?}",
        AZTEC_MODERN.name,
        pieces.len(),
        dump_types.len(),
        runner_types.len(),
        dump_types.difference(&runner_types).collect::<Vec<_>>(),
        mismatches.len(),
        mismatches.iter().take(8).collect::<Vec<_>>()
    );
    let parting: BTreeSet<_> = building_mismatches.iter().copied().collect();
    let graphs: Vec<String> = install()
        .unwrap()
        .units()
        .unwrap()
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
        .collect();
    let art = crate::artdata::piece_decoding(&install().unwrap(), &graphs);
    let decoding = pieces.intersection(&art.decoded_pieces).count();
    let entries = crate::artdata::unit_art(&install().unwrap(), &graphs);
    let short: Vec<(i32, Option<&str>)> = pieces
        .difference(&art.decoded_pieces)
        .map(|p| {
            let e = entries.iter().find(|e| e.pieces.contains(p));
            (*p, e.map(|e| e.name.as_str()))
        })
        .collect();
    eprintln!(
        "M buildings parting: {} records, {parting:?}; pieces {pieces:?}; \
         of them decoding (artdata::piece_decoding) {decoding}: short {:?}",
        building_mismatches.len(),
        short
    );
    pin_eq!(mismatches.len(), 0, "figures whose piece parts");
    pin_eq!(outside.len(), 0, "dumped pieces the runner never played");
    for (i, check) in checks.iter().enumerate() {
        pin_eq!(
            check.runner_pieces,
            check.pieces,
            "{}: the runner's pieces on the dump's frames",
            names[i]
        );
    }
    pin_eq!(
        checks,
        [
            // run1428's `DUMP_ALL` start and its frame-1 state: every
            // figure on the map, every building.
            DumpCheck {
                frames: 2,
                figures: 136,
                agree: 136,
                off_map: 0,
                pieces: 12,
                runner_pieces: 12,
                building_types: 5,
                buildings: 26,
                buildings_agree: 26,
            },
            // run1429's start block (its stride's guys print no `gpiece`):
            // five figures of units inside, which no scene draws, and the
            // capital `1/2000` (below).
            DumpCheck {
                frames: 1,
                figures: 170,
                agree: 170,
                off_map: 5,
                pieces: 32,
                runner_pieces: 32,
                building_types: 21,
                buildings: 62,
                buildings_agree: 61,
            },
            // run1429b prints no unit's `gpiece`; its buildings, 95 blocks.
            DumpCheck {
                frames: 0,
                figures: 0,
                agree: 0,
                off_map: 0,
                pieces: 0,
                runner_pieces: 0,
                building_types: 21,
                buildings: 4476,
                buildings_agree: 4381,
            },
        ]
        .into_iter()
        .collect::<Vec<_>>(),
        "what each dump of the lobby prints, against the runner"
    );
    // **The one building whose type parts is an upgrade, not a
    // disagreement**: the Aztecs' capital `1/2000`, made as type 0
    // (`orig_type` 414, the only type a dump prints) and type 2 in the
    // runner's scene, which carries the current type (RENDERER §2.3) — 96
    // records, run1429's start and the window's 95 blocks.
    pin_eq!(
        (
            building_mismatches.len(),
            parting.into_iter().collect::<Vec<_>>()
        ),
        (96, vec![(ObjId { who: 1, o: 2000 }, 414, Some(2))]),
        "the building records whose type parts"
    );
    pin_eq!(
        (decoding, pieces.len()),
        M_PIECES_DECODING,
        "M's pieces decoding"
    );
    pin_eq!(pieces.len(), RENDER_M_PIECES, "M's pieces");
    pin_eq!(
        dump_types.len(),
        RENDER_M_BUILDING_TYPES,
        "M's building types"
    );
    pin_eq!(m, RENDER_M, "M");
    pin_eq!(
        runner_types.len(),
        RUNNER_BUILDING_TYPES,
        "the runner's building types"
    );
}

/// A clock reading: `(ms to frame zero, ms to the closing block, peak
/// resident MiB)`.
type Clock = (u128, u128, u64);

/// **The renderer's clock, its first readings** (item 4020; `docs/DECISIONS.md`
/// entry 114 §4): the free runner on the pair, from the test's start —
/// the install's tables loaded, the trace read for the pasture, the setup
/// blocks read and the `Sim` built — to frame zero, and on to frame 3390
/// with every tick's scene taken. **How measured**: `clock_the_free_runner`
/// alone in the release test binary, `/usr/bin/time -l <bin> --exact
/// free::tests::clock_the_free_runner --ignored --test-threads 1`, as
/// `tools/suite_timings.py` reads a test's peak; the median of three on
/// 2026-11-09, on this machine with the decoded-oracle and setup-observation
/// caches warm (431, 426, 433 ms; 952, 951, 956 ms; 341,147,648 bytes peak).
/// The milliseconds are the test's own `Instant`; the peak is the whole
/// process's. A reading, not an assertion: nothing fails on it.
pub(crate) const RUNNER_CLOCK: Clock = (431, 952, 325);

/// **The harness's, beside it**, measured the same way the same day
/// (`clock_the_harness`; 566, 540, 533 ms; 1,110, 1,088, 1,078 ms;
/// 480,034,816 bytes peak): the long capture and run1428 read and parsed
/// whole, then each of run1429's 35 logged frames stepped to and compared.
/// The runner's peak is 133 MiB under it: what reading two dumps whole costs.
pub(crate) const HARNESS_CLOCK: Clock = (540, 1088, 458);

/// One clock reading: milliseconds from the test's start to frame zero (the
/// install's tables loaded, the trace read, the `Sim` built) and to the
/// pair's closing block.
fn clock_line(who: &str, t0: std::time::Instant, zero: std::time::Duration) {
    eprintln!(
        "CLOCK {who}: {} ms to frame zero, {} ms to frame {}",
        zero.as_millis(),
        t0.elapsed().as_millis(),
        AZTEC_MODERN.length + 1
    );
}

/// **The renderer's clock, the free runner's half** (`docs/DECISIONS.md`
/// entry 114 §4): the pair from nothing to frame zero and on to its closing
/// block, every tick's scene taken. Read alone, with its peak resident set,
/// as [`RUNNER_CLOCK`] says, which it prints beside the reading.
#[test]
#[ignore = "a measurement: the renderer's clock, read alone under /usr/bin/time -l"]
fn clock_the_free_runner() {
    let t0 = std::time::Instant::now();
    let Some(c) = captures() else {
        return;
    };
    let mut free = c.runner();
    let zero = t0.elapsed();
    let mut figures = 0;
    free.run_to(AZTEC_MODERN.length + 1, None, |s| {
        figures += s.units.iter().map(|u| u.figures.len()).sum::<usize>();
    });
    clock_line("free runner", t0, zero);
    eprintln!("CLOCK free runner pinned: {RUNNER_CLOCK:?}");
    eprintln!("CLOCK free runner: {figures} figures handed out");
}

/// **The clock's harness half**: the same pair the harness's way — the
/// long capture and its start sibling read whole, then each logged frame
/// stepped to and compared (`diff::Replay::step`'s loop).
#[test]
#[ignore = "a measurement: the renderer's clock, read alone under /usr/bin/time -l"]
fn clock_the_harness() {
    let t0 = std::time::Instant::now();
    let Some(c) = captures() else {
        return;
    };
    let text = crate::capture::read(&c.long);
    let log = Log::parse(&text);
    let start_text = crate::capture::read(&c.start);
    let start_log = Log::parse(&start_text);
    let sibling = start_log.replay_initial().unwrap();
    let mut init = log.replay_initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &c.trace);
    let players = crate::diff::player_count(&init);
    let mut built = build_sim(&c.loaded, &init, Tuning::RON);
    drop(init);
    let zero = t0.elapsed();
    let (mut at, mut diverged) = (0, 0);
    log.visit_frame_states(None, |f| {
        while at < f.n {
            built.tick();
            at += 1;
        }
        diverged += crate::diff::compare(&built, &f, players).diverged.len();
    });
    clock_line("harness", t0, zero);
    eprintln!("CLOCK harness pinned: {HARNESS_CLOCK:?}");
    eprintln!("CLOCK harness: {diverged} unit positions parted");
}
