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
use std::path::{Path, PathBuf};

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
/// `-1`, a figure with no piece, left out (item 4020: 39). Item 4026: **38
/// (−1)** — the read door stopped reporting a unit at no health (the
/// original has taken it out of its object array; `1/68`'s number is a
/// new plane's by 2701), and piece 265 was played only by such a record.
pub(crate) const RENDER_M_PIECES: usize = 38;

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
/// M's pieces (item 4020: 32 of 39; item 4026: 31 of 38, piece 265 left M
/// and it decoded). The seven short were the six gaia pieces
/// 60063–60065 and 60072–60074 and 38322, `THECITIZEN-DEFAULT-AGE0-CREW3`.
/// Item 4028: **38 of 38**, +7. The gaia six resolve through the loader's
/// own name, `{GRAPH}-TYPE{v}` (`artdata::bird_pieces`, from
/// `unit_node_from_gpiece@00901c30`), and decode. 38322's one failing file,
/// `ceo_crew3_idle3.bha` (not in the install), is `CHAR_IDLE2`'s, which the
/// run never plays: condition 1 is read on the captured slots, as §5.4
/// writes it (`PieceDecoding::decodes_on`, [`played_slots`]).
pub(crate) const M_PIECES_DECODING: (usize, usize) = (38, 38);

/// M's building types whose art resolves and decodes (item 4028),
/// `(decoding, M's types, the pieces the dumps print for them, files
/// decoded)`: each `orig_type`'s pieces are the `gpiece`s its records print
/// (`BuildDump::gpiece`), each named and decoded through
/// `artdata::building_decoding`. New, so no delta: **20 of 21**, from 25
/// pieces and 80 files (zbuffer models, addon models and their
/// animations); the one short is the farm (417), whose
/// `FARM-DEFAULT-AGE0` names no model — its look is the `<FARM>` element's,
/// which nothing here reads (`docs/RENDERER.md` §5.4).
pub(crate) const M_BUILDING_TYPES_DECODING: (usize, usize, usize, usize) = (20, 21, 25, 80);

/// The slots each piece is shown in on a scene: §5.4's "captured
/// animation slots" (item 4028), read on the runner's scenes, which equal
/// the harness's on every tick.
fn played_slots(u: &UnitScene, played: &mut BTreeMap<i32, BTreeSet<i8>>) {
    for f in u.figures.iter().filter(|f| f.gpiece >= 0) {
        played.entry(f.gpiece).or_default().insert(f.anim);
    }
}

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
    // Every building type the dumps print, with every `gpiece` a record of
    // that type carries (item 4028): the art the original drew it with.
    let mut type_pieces: BTreeMap<i64, BTreeSet<i32>> = BTreeMap::new();
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
                    let t = b.orig_type?;
                    if let Some(p) = b.gpiece {
                        type_pieces.entry(t).or_default().insert(p as i32);
                    }
                    Some((id, t))
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
    let mut played: BTreeMap<i32, BTreeSet<i8>> = BTreeMap::new();
    let (mut mismatches, mut building_mismatches) = (Vec::new(), Vec::new());
    let mut see = |s: &Scene| {
        // `-1` is a figure with no piece: nothing to draw.
        for u in &s.units {
            pieces.extend(u.figures.iter().map(|f| f.gpiece).filter(|p| *p >= 0));
            played_slots(u, &mut played);
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
    let decoding = pieces
        .iter()
        .filter(|p| art.decodes_on(**p, &played[p]))
        .count();
    let entries = crate::artdata::unit_art(&install().unwrap(), &graphs);
    let short: Vec<(i32, Option<&str>, &BTreeSet<i8>)> = pieces
        .iter()
        .filter(|p| !art.decodes_on(**p, &played[p]))
        .map(|p| {
            let e = entries.iter().find(|e| e.pieces.contains(p));
            (*p, e.map(|e| e.name.as_str()), &played[p])
        })
        .collect();
    // The building half (item 4028): each type's pieces resolved and
    // decoded; a type decodes when every piece the dumps print for it does
    // and names a model — the farm's `<BUILD>` names none (`texture`,
    // `zbuffer` and `anim` empty), and an empty list is not a decode.
    let all_build: BTreeSet<i32> = type_pieces.values().flatten().copied().collect();
    let builds = crate::artdata::building_decoding(&install().unwrap(), &all_build);
    let named = crate::artdata::build_art(&install().unwrap(), &all_build);
    for (t, ps) in &type_pieces {
        for p in ps {
            eprintln!(
                "M building type {t}: piece {p} {:?} → {:?}",
                named[p]
                    .as_ref()
                    .map(|a| (&a.name, &a.zbuffer, a.addons.len())),
                builds[p]
            );
        }
    }
    let types_decoding = dump_types
        .iter()
        .filter(|t| {
            type_pieces
                .get(t)
                .is_some_and(|ps| ps.iter().all(|p| matches!(builds[p], Ok(n) if n > 0)))
        })
        .count();
    let build_files: usize = builds.values().filter_map(|r| r.as_ref().ok()).sum();
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
    let unnamed = builds
        .values()
        .filter(
            |r| matches!(r, Err(f) if matches!(f[..], [crate::artdata::ArtFailure::Unnamed(_)])),
        )
        .count();
    pin_eq!(unnamed, 0, "building pieces that name no `<BUILD>`");
    pin_eq!(
        (
            types_decoding,
            dump_types.len(),
            all_build.len(),
            build_files
        ),
        M_BUILDING_TYPES_DECODING,
        "M's building types decoding, of M's, their pieces and files"
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

/// **The runner with the draw list on** (item 4026, `clock_the_draw_list`,
/// measured as [`RUNNER_CLOCK`] is, the median of three on 2026-11-09:
/// 434, 493, 428 ms; 994, 1,077, 978 ms; 340,541,440 bytes peak; the
/// slowest single list 18 µs). The runner alone, interleaved with it the
/// same hour: 428 ms, 974 ms, 340,721,664 bytes. So the list costs about
/// 20 ms over 3,390 ticks — 6 µs a tick for 109 figures — and no memory.
pub(crate) const DRAW_CLOCK: Clock = (434, 994, 325);

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

/// **The clock with the draw list on** (item 4026): the free runner's
/// reading again, every tick's scene copied into the draw list's input and
/// the list built at the pinned camera — what a client pays before its
/// first pixel. Measured as [`RUNNER_CLOCK`] is, the same day, alone.
#[test]
#[ignore = "a measurement: the renderer's clock, read alone under /usr/bin/time -l"]
fn clock_the_draw_list() {
    let t0 = std::time::Instant::now();
    let Some(c) = captures() else {
        return;
    };
    let mut free = c.runner();
    let terrain = free.terrain();
    let (camera, screen) = pinned_camera(terrain.width, terrain.height);
    let zero = t0.elapsed();
    let (mut drawn, mut worst) = (0, std::time::Duration::ZERO);
    free.run_to(AZTEC_MODERN.length + 1, None, |s| {
        let t = std::time::Instant::now();
        let list = render::draw::draw_list(&placed(s), &[], &camera, &screen, Some(SIZE), None);
        worst = worst.max(t.elapsed());
        drawn += list.len();
    });
    clock_line("draw list", t0, zero);
    eprintln!("CLOCK draw list pinned: {DRAW_CLOCK:?}; slowest list {worst:?}");
    eprintln!("CLOCK draw list: {drawn} figures drawn");
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

// ---- The draw list, and frames in register (item 4026) ----

/// The pinned camera: the game's own as run1484's packet holds it (logger
/// frame 676; `docs/RENDERER.md` §3.4 — the default zoom, `tilt` 60, `dir`
/// 45, the parallel projection) moved to look at the map's centre, at the
/// packet's 1920×1080.
fn pinned_camera(width: i32, height: i32) -> (render::Camera, render::Screen) {
    let cell = sim::world::UNITS_PER_CELL as f32;
    (
        render::Camera {
            zoom_distance: f32::from_bits(0x46a2_f59c),
            camera_scale: 1.0,
            tilt_angle: 60.0,
            dir_angle: 45.0,
            loc_x: width as f32 * cell / 2.0,
            loc_y: height as f32 * cell / 2.0,
            loc_x_adjust: 0.0,
            loc_y_adjust: 0.0,
            loc_z_adjust: 0.0,
            angle: 45.0,
            near_plane: 6589.402,
            far_plane: 36262.105,
            view_ratio: 0.563_020_8,
            parallel: true,
            parallel_distance: f32::from_bits(0x4622_f59c),
        },
        render::Screen {
            scaled_xs: 1920,
            scaled_ys: 1080,
            natural_ratio: 1.0,
            console_y: 0,
        },
    )
}

/// The pinned screen size every point is projected at.
const SIZE: (i32, i32) = (1920, 1080);

/// `camera_oracle.py points`' camera line, in the sweep's field order.
fn state_line(c: &render::Camera, s: &render::Screen) -> String {
    let h = |v: f32| format!("{:08x}", v.to_bits());
    format!(
        "state {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        h(c.zoom_distance),
        h(c.camera_scale),
        h(c.tilt_angle),
        h(c.dir_angle),
        h(c.loc_x),
        h(c.loc_y),
        h(c.loc_x_adjust),
        h(c.loc_y_adjust),
        h(c.loc_z_adjust),
        h(c.angle),
        h(c.near_plane),
        h(c.far_plane),
        h(c.view_ratio),
        u8::from(c.parallel),
        h(c.parallel_distance),
        s.scaled_xs,
        s.scaled_ys,
        h(s.natural_ratio),
        s.console_y
    )
}

/// The original's `MiscAccess::world_to_screen` on every point, under
/// unicorn (`tools/render/camera_oracle.py points`, item 4026), or `None`
/// (said) with no `uv`. Each row: the tag, the pixel.
fn original_pixels(
    exe: &Path,
    camera: &str,
    points: &[(String, [i64; 3])],
) -> Option<BTreeMap<String, (i32, i32)>> {
    let dir = std::env::temp_dir().join(format!("att-draw-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("points.txt");
    let mut text = format!("{camera}\n");
    for (tag, [x, y, z]) in points {
        text += &format!("pt {tag} {x} {y} {z} {} {}\n", SIZE.0, SIZE.1);
    }
    std::fs::write(&file, text).unwrap();
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let out = std::process::Command::new("uv")
        .args(["run", "--locked", "--project"])
        .arg(root.join("tools/emu"))
        .arg("python")
        .arg(root.join("tools/render/camera_oracle.py"))
        .arg(exe)
        .arg("points")
        .arg(&file)
        .output();
    let _ = std::fs::remove_dir_all(&dir);
    let out = match out {
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            return None;
        }
        Ok(o) => o,
    };
    assert!(
        out.status.success(),
        "camera_oracle.py points failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rows = String::from_utf8(out.stdout).unwrap();
    Some(
        rows.lines()
            .filter(|l| l.starts_with("pt "))
            .map(|l| {
                let t: Vec<&str> = l.split_whitespace().collect();
                (
                    t[1].to_string(),
                    (t[8].parse().unwrap(), t[9].parse().unwrap()),
                )
            })
            .collect(),
    )
}

/// **Frames in register** (`docs/RENDERER.md` §5.1, candidate A, kept as a
/// floor): of run1429's logged frames, those on which the free runner's
/// draw list, at the pinned camera, holds exactly the figures the dump
/// prints on the map and puts each within one pixel of the original's own
/// projection of the dumped figure's point (item 4026: 33 of 35; item
/// 4027: **35 of 35**, +2). The two 4026 left out were each a guy 0 a
/// later unit's push had moved its unit off (`collide_frame` the frame
/// before, `collide_o` 75 and 90) without moving the guys, so the figure
/// stood on the unit's point from before the push — guy 0's own
/// `Movement::body`, which `observe` now reads (`docs/RENDERER.md` §3.5):
/// - 2501, `1/9` guy 0 (piece 11264): the dump's figure (39869, 23358,
///   305), the original's pixel (3554, 1827); ours was its unit's (39879,
///   23369, 305), pixel (3554, 1829), and is now (39869, 23358, 305),
///   pixel (3554, 1827).
/// - 3001, `1/67` guy 0 (piece 232): the dump's (37576, 19846, −16), pixel
///   (3746, 1433); ours was (37591, 19892, −14), pixel (3741, 1438), and is
///   now (37576, 19846, −16), pixel (3746, 1433).
pub(crate) const FRAMES_IN_REGISTER: (usize, usize) = (35, 35);

/// Figures in register, of every figure the dump prints on the map on
/// those frames (item 4026: 3840 of 3842; item 4027: 3842 of 3842, +2).
pub(crate) const FIGURES_IN_REGISTER: (usize, usize) = (3842, 3842);

/// **§5.4's K**: of M's pieces, those whose model and slots decode
/// (condition 1, `artdata::piece_decoding`) and whose every logged frame
/// is in register wherever a figure plays it (condition 2), and the two
/// halves apart: (K, decoding, in register, M's pieces). Item 4026: K 29
/// — 31 decode, 36 in register (232 and 11264 out). Item 4027: **K 31**,
/// +2 — 31 decode (the seven short are `M_PIECES_DECODING`'s), 38 of 38
/// in register. M's 21 building types have no decoder and no dumped
/// frame, and count toward no K: **K 31 of M 59**. Item 4028: **K 38**, +7
/// — 38 decode on the slots the run plays (`M_PIECES_DECODING`), 38 of 38
/// in register. The building types now decode, 20 of 21
/// (`M_BUILDING_TYPES_DECODING`), and still count toward no K: the draw
/// list carries no building, so condition 2 is unmet, not vacuous
/// (`docs/RENDERER.md` §5.4): **K 38 of M 59**. Item 4035: renamed from
/// `RENDER_K`, now the pieces' half beside [`RENDER_K_BUILDINGS`]; 38.
pub(crate) const RENDER_K_PIECES: (usize, usize, usize, usize) = (38, 38, 38, 38);

#[test]
fn the_draw_list_s_frames_are_in_register() {
    let _pins = Pins::hold();
    let Some(c) = captures() else {
        return;
    };
    let inst = install().unwrap();
    // The dump's figures on the map, by logged frame: `inside_up` −1 is a
    // unit standing in the world rather than garrisoned or carried.
    let mut src = IndexedCapture::open(&c.long).unwrap();
    let mut dumped: BTreeMap<i64, BTreeMap<(ObjId, usize), [i64; 3]>> = BTreeMap::new();
    for frame in src.frame_states() {
        let f: Frame = frame.unwrap();
        let figs = dumped.entry(f.n).or_default();
        for u in f
            .units
            .iter()
            .filter(|u| u.inside_up.is_none_or(|i| i == -1))
        {
            let id = ObjId {
                who: u.who as sim::Player,
                o: u.o as i16,
            };
            for (k, g) in u.guys.iter().enumerate() {
                if let Some(p) = g.pos {
                    figs.insert((id, g.guy_num.map_or(k, |n| n as usize)), [p.x, p.y, p.z]);
                }
            }
        }
    }
    let mut free = c.runner();
    let terrain = free.terrain();
    let (camera, screen) = pinned_camera(terrain.width, terrain.height);
    let mut pieces: BTreeSet<i32> = BTreeSet::new();
    // Ours on each logged frame: (id, guy) → (point, pixel, piece).
    type Ours = BTreeMap<(ObjId, usize), ([i32; 3], (i32, i32), i32)>;
    let mut ours: BTreeMap<i64, Ours> = BTreeMap::new();
    let mut all_pieces: BTreeMap<(i64, ObjId, usize), i32> = BTreeMap::new();
    let last = AZTEC_MODERN.length + 1;
    let mut played: BTreeMap<i32, BTreeSet<i8>> = BTreeMap::new();
    let mut see = |s: &Scene| {
        for u in &s.units {
            pieces.extend(u.figures.iter().map(|f| f.gpiece).filter(|p| *p >= 0));
            played_slots(u, &mut played);
        }
        if !dumped.contains_key(&s.frame) {
            return;
        }
        let placed = placed(s);
        for p in &placed {
            let id = ObjId { who: p.who, o: p.o };
            all_pieces.insert((s.frame, id, p.figure as usize), p.piece);
        }
        let list = render::draw::draw_list(&placed, &[], &camera, &screen, Some(SIZE), None);
        let at: BTreeMap<_, _> = placed
            .iter()
            .map(|p| ((p.who, p.o, p.figure), [p.x, p.y, p.z]))
            .collect();
        ours.insert(
            s.frame,
            list.iter()
                .filter_map(|d| Some((d, d.figure()?)))
                .map(|(d, figure)| {
                    (
                        (ObjId { who: d.who, o: d.o }, figure as usize),
                        (at[&(d.who, d.o, figure)], (d.anchor.x, d.anchor.y), d.piece),
                    )
                })
                .collect(),
        );
    };
    see(&free.scene(None));
    free.run_to(last, None, &mut see);
    // The reference: the original's projection of every dumped point.
    let tag = |n: i64, id: ObjId, k: usize| format!("{n}/{}/{}/{k}", id.who, id.o);
    let points: Vec<(String, [i64; 3])> = dumped
        .iter()
        .flat_map(|(&n, figs)| figs.iter().map(move |(&(id, k), &p)| (tag(n, id, k), p)))
        .collect();
    let Some(theirs) = original_pixels(
        &inst.root().join("riseofnations.exe"),
        &state_line(&camera, &screen),
        &points,
    ) else {
        return;
    };
    let (mut frames_ok, mut figs_ok, mut figs_all) = (0, 0, 0);
    let mut misses: Vec<String> = Vec::new();
    let mut bad_pieces: BTreeSet<i32> = BTreeSet::new();
    let mut first_out = None;
    for (&n, figs) in &dumped {
        let mine = ours.get(&n).cloned().unwrap_or_default();
        let mut frame_ok = true;
        for (&(id, k), p) in figs {
            figs_all += 1;
            let them = theirs[&tag(n, id, k)];
            match mine.get(&(id, k)) {
                Some((at, px, piece))
                    if (px.0 - them.0).abs() <= 1 && (px.1 - them.1).abs() <= 1 =>
                {
                    let _ = (at, piece);
                    figs_ok += 1;
                }
                other => {
                    frame_ok = false;
                    if let Some(&pc) = all_pieces.get(&(n, id, k)) {
                        bad_pieces.insert(pc);
                    }
                    misses.push(format!(
                        "frame {n} {}/{} guy {k}: dump {p:?} at {them:?}, ours {:?}",
                        id.who,
                        id.o,
                        other.map(|(at, px, pc)| (at, px, pc))
                    ));
                }
            }
        }
        for (&(id, k), (at, px, piece)) in &mine {
            if !figs.contains_key(&(id, k)) {
                frame_ok = false;
                bad_pieces.insert(*piece);
                misses.push(format!(
                    "frame {n} {}/{} guy {k}: drawn by us at {at:?} {px:?} piece {piece}, not in the dump",
                    id.who, id.o
                ));
            }
        }
        if frame_ok {
            frames_ok += 1;
        } else if first_out.is_none() {
            first_out = Some(n);
        }
    }
    let graphs: Vec<String> = inst
        .units()
        .unwrap()
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
        .collect();
    let art = crate::artdata::piece_decoding(&inst, &graphs);
    let decoding: BTreeSet<i32> = pieces
        .iter()
        .filter(|p| art.decodes_on(**p, &played[p]))
        .copied()
        .collect();
    let registered: BTreeSet<i32> = pieces.difference(&bad_pieces).copied().collect();
    let k = decoding.intersection(&registered).count();
    eprintln!(
        "DRAW {}: frames in register {frames_ok} of {}, first out {first_out:?}; figures {figs_ok} of {figs_all}; \
         K {k} of M's {} pieces (decoding {}, in register {}; pieces out {bad_pieces:?})\n  {}",
        AZTEC_MODERN.name,
        dumped.len(),
        pieces.len(),
        decoding.len(),
        registered.len(),
        misses.join("\n  ")
    );
    // Item 4027's value diff: the two figures 4026 left out, both sides.
    for (n, o) in [(2501, 9), (3001, 67)] {
        let id = ObjId { who: 1, o };
        eprintln!(
            "  {n} {}/{} guy 0: dump {:?} at {:?}, ours {:?}",
            id.who,
            id.o,
            dumped.get(&n).and_then(|f| f.get(&(id, 0))),
            theirs.get(&tag(n, id, 0)),
            ours.get(&n).and_then(|f| f.get(&(id, 0)))
        );
    }
    pin_eq!(fog_visible_is_render_s(), true, "the fog's VISIBLE value");
    pin_eq!(
        (frames_ok, dumped.len()),
        FRAMES_IN_REGISTER,
        "frames in register"
    );
    pin_eq!(
        (figs_ok, figs_all),
        FIGURES_IN_REGISTER,
        "figures in register"
    );
    pin_eq!(
        (k, decoding.len(), registered.len(), pieces.len()),
        RENDER_K_PIECES,
        "K, decoding, in register, M's pieces"
    );
}

fn fog_visible_is_render_s() -> bool {
    sim::observe::fog::VISIBLE == render::draw::FOG_VISIBLE
}

// ---- Buildings in the draw list (item 4035) ----

/// **§5.4's K**, both halves: M's pieces' K ([`RENDER_K_PIECES`]) and M's
/// building types' ([`RENDER_K_BUILDINGS`]). Item 4035: the draw list
/// carries buildings, so a building type's condition 2 is live.
pub(crate) const RENDER_K: usize = RENDER_K_PIECES.0 + RENDER_K_BUILDINGS.0;

/// M's building types that count toward K, `(K, decoding, in register,
/// M's types)`: a type (`orig_type`) counts when it decodes (condition 1,
/// [`M_BUILDING_TYPES_DECODING`]) and every record of it the lobby's
/// dumps print is in register (condition 2,
/// [`the_draw_list_s_buildings_are_in_register`]). Item 4035: new —
/// **20 of 21**: all 21 in register, 20 decoding; the farm (417) is short
/// on condition 1 alone ([`M_BUILDING_TYPES_DECODING`]). With the pieces'
/// 38, **K 58 of M 59** (+20).
pub(crate) const RENDER_K_BUILDINGS: (usize, usize, usize, usize) = (20, 20, 21, 21);

/// The dumped frames that print a building — run1428's two, run1429's
/// start block and its closing block 3390, and run1429b's 95 — on which
/// every building record is in register, of them (item 4035: new, 99 of
/// 99).
pub(crate) const BUILDING_FRAMES_IN_REGISTER: (usize, usize) = (99, 99);

/// The building records in register on those frames, of all printed:
/// drawn on both sides or on neither, and where drawn, our anchor within
/// one pixel of the original's projection and our piece the dump's
/// (item 4035: new, 4,564 of 4,564).
pub(crate) const BUILDINGS_IN_REGISTER: (usize, usize) = (4564, 4564);

/// The building records whose piece — `GraphicPieces::get_build_gpiece`'s
/// walk from the door's type, nation and age
/// ([`crate::artdata::BuildLook::piece`]) — is the dump's `gpiece`, of all
/// printed (item 4035: new, 4,564 of 4,564).
pub(crate) const BUILDING_PIECES_AGREEING: (usize, usize) = (4564, 4564);

/// One building record a dump prints, as the reference reads it.
#[derive(Clone, Copy, Debug)]
struct DumpedBuilding {
    pos: [i64; 3],
    orig_type: i64,
    gpiece: i32,
    active: bool,
    job_counter: i64,
    constr_time: i64,
}

/// **Buildings in register** (`docs/RENDERER.md` §3.5, §5.4 condition 2;
/// item 4035). On every frame a dump of the lobby prints buildings on, the
/// draw list's building entries are compared with the original's own
/// projection of each dumped building: `WallOut::draw@00641930`'s point —
/// the dumped `x_internal`, `y_internal`, `z_internal`, moved back
/// `0x60 · halfoffset` on both axes, `halfoffset` the dumped piece's
/// `<BUILD>`'s — through `MiscAccess::world_to_screen` under unicorn. The
/// drawn set is `WallOut::draw`'s predicate on each side's own state (the
/// dumps print no draw): a building is in register when both sides draw it
/// or neither, and, drawn, when its anchor is within one pixel and its
/// piece is the dump's.
#[test]
fn the_draw_list_s_buildings_are_in_register() {
    let _pins = Pins::hold();
    let Some(c) = captures() else {
        return;
    };
    let Some(window) = dump(RUN1429B) else {
        eprintln!("skipping: no {RUN1429B}");
        return;
    };
    let inst = install().unwrap();
    let look = crate::artdata::BuildLook::new(&inst);
    let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
    let rules = inst.buildings().unwrap();
    let build_graphs: Vec<String> = rules
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().to_string())
        .collect();
    let names = [AZTEC_MODERN.start, AZTEC_MODERN.long.0, RUN1429B];
    let paths = [c.start.clone(), c.long.clone(), PathBuf::from(window)];
    // (dump, frame) → the buildings it prints.
    type Printed = BTreeMap<(usize, i64), BTreeMap<ObjId, DumpedBuilding>>;
    let mut printed = Printed::new();
    for (i, path) in paths.iter().enumerate() {
        let mut src = IndexedCapture::open(path).unwrap();
        let setup = src.read_replay_setup().unwrap();
        let setup_log = Log::parse(&setup);
        let init = setup_log.replay_initial().unwrap();
        let mut put = |n: i64, builds: &[crate::gamelog::BuildDump]| {
            let recs: BTreeMap<ObjId, DumpedBuilding> = builds
                .iter()
                .filter_map(|b| {
                    Some((
                        ObjId {
                            who: b.who as sim::Player,
                            o: b.o as i16,
                        },
                        DumpedBuilding {
                            pos: [b.pos.x, b.pos.y, b.pos.z],
                            orig_type: b.orig_type?,
                            gpiece: b.gpiece? as i32,
                            active: b.flags & 4 != 0,
                            job_counter: b.job_counter.unwrap_or(0),
                            constr_time: b.constr_time.unwrap_or(1),
                        },
                    ))
                })
                .collect();
            if !recs.is_empty() {
                printed.insert((i, n), recs);
            }
        };
        put(0, &init.builds);
        for frame in src.frame_states() {
            let f: Frame = frame.unwrap();
            put(f.n, &f.builds);
        }
    }
    let wanted: BTreeSet<i64> = printed.keys().map(|(_, n)| *n).collect();
    let mut free = c.runner();
    let terrain = free.terrain();
    let (camera, screen) = pinned_camera(terrain.width, terrain.height);
    // Ours on each wanted frame: id → (placed, drawn pixel if drawn).
    type Ours = BTreeMap<ObjId, (render::draw::PlacedBuilding, Option<(i32, i32)>)>;
    let mut ours: BTreeMap<i64, Ours> = BTreeMap::new();
    let mut see = |s: &Scene| {
        if !wanted.contains(&s.frame) {
            return;
        }
        let blds = placed_buildings(s, &look, &tribes);
        let list = render::draw::draw_list(&[], &blds, &camera, &screen, Some(SIZE), None);
        let px: BTreeMap<ObjId, (i32, i32)> = list
            .iter()
            .map(|d| (ObjId { who: d.who, o: d.o }, (d.anchor.x, d.anchor.y)))
            .collect();
        ours.insert(
            s.frame,
            blds.iter()
                .map(|b| {
                    let id = ObjId { who: b.who, o: b.o };
                    (id, (*b, px.get(&id).copied()))
                })
                .collect(),
        );
    };
    see(&free.scene(None));
    free.run_to(AZTEC_MODERN.length + 1, None, &mut see);
    // The reference: the dumped building as `WallOut::draw` reads it.
    let theirs_placed = |id: ObjId, d: &DumpedBuilding| {
        let type_index = crate::artdata::build_piece_name(d.gpiece, &build_graphs)
            .map_or(d.orig_type as i32, |(t, _)| t);
        render::draw::PlacedBuilding {
            who: id.who,
            o: id.o,
            x: d.pos[0] as i32,
            y: d.pos[1] as i32,
            z: d.pos[2] as i32,
            type_index,
            orig_type: d.orig_type as i32,
            piece: d.gpiece,
            half_offset: look.half_offset(type_index),
            active: d.active,
            job_counter: d.job_counter as i32,
            construct_time: d.constr_time as i32,
        }
    };
    let tag = |i: usize, n: i64, id: ObjId| format!("b{i}/{n}/{}/{}", id.who, id.o);
    let points: Vec<(String, [i64; 3])> = printed
        .iter()
        .flat_map(|(&(i, n), recs)| {
            recs.iter().filter_map(move |(&id, d)| {
                let p = theirs_placed(id, d);
                let (x, y, z) = p.anchor_point();
                p.drawn()
                    .then(|| (tag(i, n, id), [x as i64, y as i64, z as i64]))
            })
        })
        .collect();
    let Some(theirs) = original_pixels(
        &inst.root().join("riseofnations.exe"),
        &state_line(&camera, &screen),
        &points,
    ) else {
        return;
    };
    let (mut frames_ok, mut recs_ok, mut recs_all) = (0, 0, 0);
    let (mut pieces_ok, mut first_out) = (0, None);
    let mut misses: Vec<String> = Vec::new();
    let mut bad_types: BTreeSet<i64> = BTreeSet::new();
    let mut type_pieces: BTreeMap<i64, BTreeSet<i32>> = BTreeMap::new();
    let mut parted_pieces: BTreeMap<(i64, i32, i32), usize> = BTreeMap::new();
    for (&(i, n), recs) in &printed {
        let mine = ours.get(&n).cloned().unwrap_or_default();
        let mut frame_ok = true;
        for (&id, d) in recs {
            recs_all += 1;
            type_pieces.entry(d.orig_type).or_default().insert(d.gpiece);
            let them = theirs.get(&tag(i, n, id)).copied();
            let ok = match mine.get(&id) {
                Some((b, px)) => {
                    if b.piece == d.gpiece {
                        pieces_ok += 1;
                    } else {
                        *parted_pieces
                            .entry((d.orig_type, d.gpiece, b.piece))
                            .or_default() += 1;
                    }
                    match (px, them) {
                        (Some(p), Some(t)) => {
                            b.piece == d.gpiece && (p.0 - t.0).abs() <= 1 && (p.1 - t.1).abs() <= 1
                        }
                        (None, None) => true,
                        _ => false,
                    }
                }
                None => false,
            };
            if ok {
                recs_ok += 1;
            } else {
                frame_ok = false;
                bad_types.insert(d.orig_type);
                misses.push(format!(
                    "{} frame {n} {}/{} type {}: dump {:?} piece {} at {them:?}, ours {:?}",
                    names[i],
                    id.who,
                    id.o,
                    d.orig_type,
                    d.pos,
                    d.gpiece,
                    mine.get(&id)
                        .map(|(b, px)| ((b.x, b.y, b.z), b.piece, b.half_offset, *px))
                ));
            }
        }
        if frame_ok {
            frames_ok += 1;
        } else if first_out.is_none() {
            first_out = Some((names[i], n));
        }
    }
    // Condition 1, as the M test reads it (item 4028).
    let all_build: BTreeSet<i32> = type_pieces.values().flatten().copied().collect();
    let builds = crate::artdata::building_decoding(&inst, &all_build);
    let decoding: BTreeSet<i64> = type_pieces
        .iter()
        .filter(|(_, ps)| ps.iter().all(|p| matches!(builds[p], Ok(n) if n > 0)))
        .map(|(t, _)| *t)
        .collect();
    let registered: BTreeSet<i64> = type_pieces
        .keys()
        .filter(|t| !bad_types.contains(t))
        .copied()
        .collect();
    let k = decoding.intersection(&registered).count();
    eprintln!(
        "BUILDINGS {}: frames in register {frames_ok} of {}, first out {first_out:?}; records {recs_ok} of {recs_all} \
         ({} drawn by the original's predicate); \
         pieces {pieces_ok} of {recs_all}, parting (type, dump, ours) → records {parted_pieces:?}; \
         K {k} of M's {} building types (decoding {:?}, in register {:?}; out {bad_types:?})\n  {}",
        AZTEC_MODERN.name,
        printed.len(),
        points.len(),
        type_pieces.len(),
        decoding,
        registered,
        misses
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    pin_eq!(
        (frames_ok, printed.len()),
        BUILDING_FRAMES_IN_REGISTER,
        "building frames in register"
    );
    pin_eq!(
        (recs_ok, recs_all),
        BUILDINGS_IN_REGISTER,
        "building records in register"
    );
    pin_eq!(
        (pieces_ok, recs_all),
        BUILDING_PIECES_AGREEING,
        "building pieces agreeing"
    );
    pin_eq!(
        (k, decoding.len(), registered.len(), type_pieces.len()),
        RENDER_K_BUILDINGS,
        "K, decoding, in register, M's building types"
    );
}

// ---- The terrain's binary condition (item 4036) ----

/// **The terrain's binary condition** (`docs/RENDERER.md` §5.4, §3.6): of
/// the frames the lobby's dumps print a height table on, those on which
/// every corner of the free runner's heightfield (`TerrainScene::corners`,
/// `render::terrain::corner_anchors`) at the pinned camera is within one
/// pixel of the original's own projection of the dumped corner
/// (`master_land_heights`, `MiscAccess::world_to_screen` under unicorn).
/// Only run1428's `DUMP_ALL` blocks print the table — the start (frame 0),
/// 1 and 37 (`Log::dump_heights`) — so this is (frames, of the frames with
/// a table). New at item 4036: **3 of 3**. Frame 37's table is not the
/// start's: the AI's first terraform moved 248 corners round (216..221,
/// 68..69), so 58,329 distinct points are projected, not 58,081. Made to
/// fail once by drawing every frame from the frame-0 grid: 2 of 3, frame
/// 37's corner (220, 68) at z 202 and pixel (5546, 1225) against the
/// dump's 175 at (5546, 1230).
pub(crate) const TERRAIN_FRAMES_IN_REGISTER: (usize, usize) = (3, 3);

/// The corners behind it, every corner of every such frame (241 × 241 on
/// the 60-cell map, three frames): new at item 4036; 174,077 under the
/// mutation above.
pub(crate) const TERRAIN_CORNERS_IN_REGISTER: (usize, usize) = (174_243, 174_243);

/// And the heights behind the pixels: the corners whose single prints as
/// the dump's millionths (`Single::prints_as`), the value half of the
/// condition, beside it. New at item 4036.
pub(crate) const TERRAIN_HEIGHTS_AGREEING: (usize, usize) = (174_243, 174_243);

#[test]
fn the_terrain_s_corners_are_in_register() {
    let _pins = Pins::hold();
    let Some(c) = captures() else {
        return;
    };
    let inst = install().unwrap();
    // The dump's height tables, by frame: the start block's is frame 0's.
    let text = crate::capture::read(&c.start);
    let log = Log::parse(&text);
    let mut tables: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let start = log.terrain_heights();
    if !start.is_empty() {
        tables.insert(0, start);
    }
    for (n, h) in log.dump_heights() {
        tables.entry(n).or_insert(h);
    }
    drop(log);
    drop(text);
    let mut free = c.runner();
    let mut ours: BTreeMap<i64, sim::observe::TerrainScene> = BTreeMap::new();
    let last = tables.keys().copied().max().unwrap_or(0);
    if tables.contains_key(&0) {
        ours.insert(0, free.terrain());
    }
    while free.frame() < last {
        free.step();
        if tables.contains_key(&free.frame()) {
            ours.insert(free.frame(), free.terrain());
        }
    }
    let (width, height) = (ours[&0].width, ours[&0].height);
    let (camera, screen) = pinned_camera(width, height);
    let stride = width * 4 + 1;
    let tag = |n: i64, cx: i32, cy: i32| format!("{n}/{cx}/{cy}");
    // The original's corners, deduplicated: the projection depends on the
    // point alone.
    let point =
        |cx: i32, cy: i32, m: i64| [i64::from(cx) * 192, i64::from(cy) * 192, m / 1_000_000];
    let mut unique: BTreeMap<[i64; 3], String> = BTreeMap::new();
    for h in tables.values() {
        for (i, &m) in h.iter().enumerate() {
            let (cx, cy) = ((i as i32) % stride, (i as i32) / stride);
            let p = point(cx, cy, m);
            let k = unique.len();
            unique.entry(p).or_insert_with(|| format!("u{k}"));
        }
    }
    let points: Vec<(String, [i64; 3])> = unique.iter().map(|(p, t)| (t.clone(), *p)).collect();
    let Some(theirs) = original_pixels(
        &inst.root().join("riseofnations.exe"),
        &state_line(&camera, &screen),
        &points,
    ) else {
        return;
    };
    let (mut frames_ok, mut corners_ok, mut corners_all, mut heights_ok) = (0, 0, 0, 0);
    let mut misses: Vec<String> = Vec::new();
    for (&n, h) in &tables {
        let Some(t) = ours.get(&n) else {
            misses.push(format!("frame {n}: no terrain of ours"));
            continue;
        };
        let field = render::terrain::Heightfield {
            width: t.width,
            height: t.height,
            corners: &t.corners,
        };
        let mine = render::terrain::corner_anchors(&field, &camera, &screen, Some(SIZE));
        let mut ok = mine.len() == h.len();
        if !ok {
            misses.push(format!(
                "frame {n}: {} corners of ours, {} dumped",
                mine.len(),
                h.len()
            ));
        }
        for (i, &m) in h.iter().enumerate() {
            corners_all += 1;
            let (cx, cy) = ((i as i32) % stride, (i as i32) / stride);
            let them = theirs[&unique[&point(cx, cy, m)]];
            if t.corners
                .get(i)
                .is_some_and(|&b| sim::num::single::Single(b).prints_as(m))
            {
                heights_ok += 1;
            }
            match mine.get(i) {
                Some((_, at, px)) if (px.x - them.0).abs() <= 1 && (px.y - them.1).abs() <= 1 => {
                    let _ = at;
                    corners_ok += 1;
                }
                other => {
                    ok = false;
                    if misses.len() < 40 {
                        misses.push(format!(
                            "frame {n} corner ({cx}, {cy}): dump {:?} at {them:?}, ours {:?}",
                            point(cx, cy, m),
                            other.map(|(_, at, px)| (at, (px.x, px.y)))
                        ));
                    }
                }
            }
        }
        if ok {
            frames_ok += 1;
        }
    }
    eprintln!(
        "TERRAIN {}: frames {:?} in register {frames_ok} of {}; corners {corners_ok} of {corners_all}; \
         heights {heights_ok}; {} distinct points projected\n  {}",
        AZTEC_MODERN.name,
        tables.keys().collect::<Vec<_>>(),
        tables.len(),
        unique.len(),
        misses.join("\n  ")
    );
    // A corner each side, for the journal's value table.
    for (cx, cy) in [(0, 0), (120, 120), (16, 155), (240, 240)] {
        let i = (cy * stride + cx) as usize;
        let m = tables[&0][i];
        let field = render::terrain::Heightfield {
            width,
            height,
            corners: &ours[&0].corners,
        };
        let px = render::terrain::corner_anchors(&field, &camera, &screen, Some(SIZE))[i].2;
        eprintln!(
            "  corner ({cx}, {cy}): dump {m} → {:?} at {:?}; ours {:?} at ({}, {})",
            point(cx, cy, m),
            theirs[&unique[&point(cx, cy, m)]],
            field.point(cx, cy),
            px.x,
            px.y
        );
    }
    pin_eq!(
        (frames_ok, tables.len()),
        TERRAIN_FRAMES_IN_REGISTER,
        "terrain frames in register"
    );
    pin_eq!(
        (corners_ok, corners_all),
        TERRAIN_CORNERS_IN_REGISTER,
        "terrain corners in register"
    );
    pin_eq!(
        (heights_ok, corners_all),
        TERRAIN_HEIGHTS_AGREEING,
        "terrain heights agreeing"
    );
}
