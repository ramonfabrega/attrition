//! **The draw list across the ages** (item 4166, `docs/RENDERER.md` §3.13):
//! every late-age pair and every battery word stood up by the free runner,
//! the client's draw list taken on each frame the long capture dumps, and
//! every object on it resolved to its own install file — a figure's
//! `<UNIT>` model and the animation of the slot it plays, a building's
//! `<BUILD>` models and animations, a farm's wheat field, a foundation's
//! `<CONSTRUCTION>` sprite, a scaffold's `<SCAFFOLD>` model — decoded under
//! `render`'s readers. Projectiles and corpses are counted beside them:
//! the scene carries no piece for either, so the list cannot draw them,
//! and they stand unresolved under their own names.
//!
//! The value check beside the count is the closing block: every long
//! capture prints its units' guys and its buildings with `gpiece` there
//! (frame `length + 1`), and the free runner's figure and building pieces
//! on that frame are compared record by record with the dump's.

use super::tests::{SIZE, pinned_camera};
use super::*;
use crate::artdata::{ArtFailure, PieceDecoding, UnitArt};
use crate::diff::drawn_games::{BATTERY_WORDS, Game, LATE_AGES};
use crate::diff::testkit::{Pins, pin_eq, trace};
use crate::gamelog::Frame;
use crate::testenv::{dump, install};
use render::draw::Pose;
use sim::observe::ObjId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// What an object of the list is, cheapest to resolve first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Figure,
    Building,
    Field,
    Foundation,
    Scaffold,
    Projectile,
    Wreck,
}

/// One game's census.
#[derive(Clone, Debug, Default)]
pub(crate) struct GameCensus {
    pub name: &'static str,
    /// Dumped frames walked: frame 0 and every `FRAME` the long prints.
    pub frames: usize,
    /// Objects on those frames, and of them those whose file decodes.
    pub objects: usize,
    pub resolved: usize,
    /// Per kind, `(resolved, objects)`.
    pub by_kind: BTreeMap<Kind, (usize, usize)>,
    /// The unresolved, by kind and the reason that names the file.
    pub unresolved: BTreeMap<(Kind, String), usize>,
    /// The closing block: records that print a `gpiece` whose object the
    /// runner holds, and of them those whose piece is ours.
    pub closing: (usize, usize),
    /// The closing records whose piece parts.
    pub closing_partings: Vec<String>,
    /// The closing records by the dump's printed `gpiece`: `(ours, all)`.
    pub closing_by_piece: BTreeMap<i32, (usize, usize)>,
    /// The lobby's M (§5.4, on its dumped frames): figure pieces played
    /// and building types drawn; and the K of it — decoding on the slots
    /// played, and the closing block's piece ours on every record of it.
    pub m: (usize, usize),
    pub k: (usize, usize),
    /// Height reads on inexact corners over the run, and the pin the
    /// game's own tests hold (the largest `testkit::GROUND_INEXACT` row
    /// naming `Game::module`; 0 when none does).
    pub ground_inexact: (u32, u32),
}

/// Each building piece's decode, by piece and tileset.
type BuildingCache = BTreeMap<(i32, String), Result<usize, Vec<ArtFailure>>>;

/// The install's art, decoded once for every game.
struct Art {
    root: std::path::PathBuf,
    pieces: PieceDecoding,
    entries: Vec<UnitArt>,
    construction: Vec<bool>,
    scaffold: Vec<bool>,
    buildings: std::sync::Mutex<BuildingCache>,
}

impl Art {
    fn new(inst: &crate::Install) -> Self {
        let graphs: Vec<String> = inst
            .units()
            .unwrap()
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        let root = inst.root().to_path_buf();
        let read =
            |rel: &str| crate::artdata::resolve_art(&root, rel).and_then(|p| std::fs::read(p).ok());
        let tga = |rel: &str| read(rel).is_some_and(|b| render::tga::decode(&b).is_ok());
        let bh3 = |rel: &str| read(rel).is_some_and(|b| render::bh3::decode(&b).is_ok());
        let bha = |rel: &str| read(rel).is_some_and(|b| render::bha::decode(&b).is_ok());
        let construction = crate::artdata::construction_art(inst)
            .iter()
            .map(|c| tga(&c.texture) && c.model.as_deref().is_none_or(bh3))
            .collect();
        let scaffold = crate::artdata::scaffold_art(inst)
            .iter()
            .map(|s| {
                bh3(&s.model)
                    && tga(&s.texture)
                    && s.anim
                        .as_ref()
                        .is_none_or(|(_, f)| f.as_deref().is_some_and(bha))
            })
            .collect();
        Self {
            pieces: crate::artdata::piece_decoding(inst, &graphs),
            entries: crate::artdata::unit_art(inst, &graphs),
            construction,
            scaffold,
            buildings: Default::default(),
            root,
        }
    }

    /// A figure's piece on the slot it plays: `None` when it decodes, else
    /// what fails, naming the entry and the file.
    fn figure(&self, piece: i32, slot: i8) -> Option<String> {
        if self.pieces.decodes_on(piece, &BTreeSet::from([slot])) {
            return None;
        }
        let entry = self.entries.iter().find(|e| e.pieces.contains(&piece));
        let name = entry.map_or("no <UNIT>", |e| e.name.as_str());
        let why = match self.pieces.by_piece.get(&piece) {
            None => "no entry names it".to_string(),
            Some(f) if !f.model => format!("model {}", entry.map_or("?", |e| e.model.as_str())),
            Some(_) => {
                let file = entry
                    .and_then(|e| e.anims.iter().find(|a| a.0 == slot))
                    .map_or("?".to_string(), |a| {
                        a.2.clone().unwrap_or_else(|| format!("unnamed {}", a.1))
                    });
                format!("slot {slot} {file}")
            }
        };
        Some(format!("piece {piece} {name}: {why}"))
    }

    /// A building piece (§5.4's building condition 1): `None` when every
    /// file it names decodes and it names one.
    fn building(&self, inst: &crate::Install, piece: i32, tileset: &str) -> Option<String> {
        let mut cache = self.buildings.lock().unwrap();
        let r = cache
            .entry((piece, tileset.to_string()))
            .or_insert_with(|| {
                crate::artdata::building_decoding(inst, &BTreeSet::from([piece]), tileset)
                    .remove(&piece)
                    .unwrap_or_else(|| Err(vec![ArtFailure::Unnamed(format!("piece {piece}"))]))
            });
        match r {
            Ok(n) if *n > 0 => None,
            Ok(_) => Some(format!("piece {piece}: names no file")),
            Err(f) => Some(format!("piece {piece}: {f:?}")),
        }
    }
}

/// The tileset a game on `start`'s map draws (item 4037's read, for any
/// map): the style file's `TILECHANCE` draw recovered from the start's
/// `map.cpp` 7805 seed, or the one entry when the style names one.
fn tileset_of(root: &Path, start: &Path, style_file: &str) -> String {
    let chances = crate::mapgen::tile_chances(root, style_file).expect("the style's tilesets");
    if let [(only, _)] = &chances[..] {
        return only.clone();
    }
    let head = crate::mapgen::start_head(start)
        .unwrap()
        .expect("a start head");
    let seed = crate::mapgen::start_grid(&head)
        .at("map.cpp", 7805)
        .expect("the start prints map.cpp 7805")
        .2;
    crate::mapgen::tileset_before(&chances, seed).expect("a drawn tileset")
}

/// The style file of a game's map, by the name its line carries.
fn style_file(name: &str) -> &'static str {
    if name.starts_with("GreatLakes") {
        "GREATLAKES.xml"
    } else if name.starts_with("EastIndies") {
        "EASTINDIES.xml"
    } else if name.starts_with("GreatSahara") {
        "SAHARA.xml"
    } else {
        panic!("{name}: no style file")
    }
}

/// What every walk shares: the install, its tables and its art.
struct Shared {
    inst: crate::Install,
    loaded: Loaded,
    art: Art,
    look: crate::artdata::BuildLook,
    tribes: Vec<crate::TribeDef>,
}

/// Walk one game whose captures are `start`, `long` and `tr`.
fn walk(
    g: &Game,
    start: &str,
    long: &str,
    tr: Option<&crate::trace::Trace>,
    sh: &Shared,
) -> GameCensus {
    let (inst, loaded, art, look, tribes) =
        (&sh.inst, &sh.loaded, &sh.art, &sh.look, &sh.tribes[..]);
    let tileset = tileset_of(art.root.as_path(), Path::new(start), style_file(g.name));
    let mut src = IndexedCapture::open(long).unwrap();
    let mut wanted: BTreeSet<i64> = src.frames().iter().map(|f| f.number).collect();
    wanted.insert(0);
    let last = *wanted.last().unwrap();
    assert_eq!(
        last,
        g.length + 1,
        "{}: the closing block is the frame after the game's last",
        g.name
    );
    // The closing block's printed pieces: figures `(id, guy) → gpiece`,
    // buildings `id → gpiece`.
    let closing: Frame = src.frame_state(src.frames().len() - 1).unwrap();
    let mut figs: BTreeMap<(ObjId, usize), i32> = BTreeMap::new();
    for u in &closing.units {
        let id = ObjId {
            who: u.who as sim::Player,
            o: u.o as i16,
        };
        for (n, guy) in u.guys.iter().enumerate() {
            if let Some(p) = guy.gpiece {
                figs.insert((id, guy.guy_num.map_or(n, |k| k as usize)), p as i32);
            }
        }
    }
    let blds: BTreeMap<ObjId, i32> = closing
        .builds
        .iter()
        .filter_map(|b| {
            Some((
                ObjId {
                    who: b.who as sim::Player,
                    o: b.o as i16,
                },
                b.gpiece? as i32,
            ))
        })
        .collect();
    drop(closing);
    let mut free = FreeRunner::from_captures(
        loaded,
        Path::new(long),
        &[Path::new(start)],
        tr,
        Tuning::RON,
    )
    .unwrap();
    let terrain = free.terrain();
    let (camera, screen) = pinned_camera(terrain.width, terrain.height);
    let mut c = GameCensus {
        name: g.name,
        ..GameCensus::default()
    };
    // §5.4 on this lobby's dumped frames: each figure piece with the slots
    // it is played in, each building type with the pieces it is drawn as.
    let mut played: BTreeMap<i32, BTreeSet<i8>> = BTreeMap::new();
    let mut types: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
    let mut parted_pieces: BTreeSet<i32> = BTreeSet::new();
    let mut parted_types: BTreeSet<i32> = BTreeSet::new();
    let tally = |c: &mut GameCensus, kind: Kind, why: Option<String>| {
        let e = c.by_kind.entry(kind).or_default();
        e.1 += 1;
        c.objects += 1;
        match why {
            None => {
                e.0 += 1;
                c.resolved += 1;
            }
            Some(w) => *c.unresolved.entry((kind, w)).or_default() += 1,
        }
    };
    let mut see = |s: &Scene, c: &mut GameCensus| {
        if !wanted.contains(&s.frame) {
            return;
        }
        c.frames += 1;
        let placed = placed(s);
        let buildings = placed_buildings(s, look, tribes);
        let list = render::draw::draw_list(
            &placed,
            &buildings,
            look.scaffold_timing(),
            &camera,
            &screen,
            Some(SIZE),
            None,
        );
        for d in &list {
            let (kind, why) = match d.pose {
                Pose::Figure { slot, .. } => {
                    played.entry(d.piece).or_default().insert(slot);
                    (Kind::Figure, art.figure(d.piece, slot))
                }
                Pose::Building { orig_type, .. } => {
                    types.entry(orig_type).or_default().insert(d.piece);
                    let why = art
                        .building(inst, d.piece, &tileset)
                        .map(|w| format!("type {orig_type} {w}"));
                    (Kind::Building, why)
                }
                Pose::Field { orig_type } => {
                    let why = crate::artdata::farm_decoding(inst, &tileset)
                        .err()
                        .map(|f| format!("type {orig_type} on {tileset}: {f:?}"));
                    (Kind::Field, why)
                }
                Pose::Foundation { orig_type, slot } => {
                    let ok = art.construction.get(usize::from(slot)) == Some(&true);
                    let why = (!ok).then(|| format!("type {orig_type} slot {slot}"));
                    (Kind::Foundation, why)
                }
                Pose::Scaffold {
                    orig_type, element, ..
                } => {
                    let ok = art.scaffold.get(usize::from(element)) == Some(&true);
                    let why = (!ok).then(|| format!("type {orig_type} element {element}"));
                    (Kind::Scaffold, why)
                }
            };
            tally(c, kind, why);
        }
        for _ in &s.projectiles {
            tally(
                c,
                Kind::Projectile,
                Some("the scene carries no piece".into()),
            );
        }
        for _ in &s.deaths {
            tally(c, Kind::Wreck, Some("the scene carries no piece".into()));
        }
        if s.frame == last {
            let ours: BTreeMap<ObjId, &sim::observe::UnitScene> =
                s.units.iter().map(|u| (u.id, u)).collect();
            for (&(id, n), &piece) in &figs {
                let Some(u) = ours.get(&id) else {
                    continue;
                };
                c.closing.1 += 1;
                let by = c.closing_by_piece.entry(piece).or_default();
                by.1 += 1;
                if u.figures.get(n).is_some_and(|f| f.gpiece == piece) {
                    by.0 += 1;
                }
                match u.figures.get(n) {
                    Some(f) if f.gpiece == piece => c.closing.0 += 1,
                    other => {
                        parted_pieces.insert(piece);
                        let named = |p: i32| {
                            art.entries
                                .iter()
                                .find(|e| e.pieces.contains(&p))
                                .map_or("?", |e| e.name.as_str())
                        };
                        c.closing_partings.push(format!(
                            "frame {last} {}/{} (TypeIndex {:?}) guy {n}: dump {piece} {}, ours {:?}",
                            id.who,
                            id.o,
                            u.ty.map(|t| t as i32 + crate::load::BASE_UNITTYPES),
                            named(piece),
                            other.map(|f| (f.gpiece, named(f.gpiece), f.anim))
                        ));
                    }
                }
            }
            for b in &buildings {
                let id = ObjId { who: b.who, o: b.o };
                let Some(&piece) = blds.get(&id) else {
                    continue;
                };
                c.closing.1 += 1;
                let by = c.closing_by_piece.entry(piece).or_default();
                by.1 += 1;
                by.0 += usize::from(b.piece == piece);
                if b.piece == piece {
                    c.closing.0 += 1;
                } else {
                    parted_types.insert(b.orig_type);
                    c.closing_partings.push(format!(
                        "frame {last} building {}/{} type {}: dump {piece}, ours {}",
                        b.who, b.o, b.orig_type, b.piece
                    ));
                }
            }
        }
    };
    see(&free.scene(None), &mut c);
    while free.frame() < last {
        free.step();
        if wanted.contains(&free.frame()) {
            see(&free.scene(None), &mut c);
        }
    }
    let piece_k = played
        .iter()
        .filter(|(p, slots)| art.pieces.decodes_on(**p, slots) && !parted_pieces.contains(p))
        .count();
    let type_k = types
        .iter()
        .filter(|(t, ps)| {
            !parted_types.contains(t)
                && ps
                    .iter()
                    .all(|p| art.building(inst, *p, &tileset).is_none())
        })
        .count();
    c.m = (played.len(), types.len());
    c.k = (piece_k, type_k);
    let pin = crate::diff::testkit::GROUND_INEXACT
        .iter()
        .filter(|(t, _)| t.contains(g.module))
        .map(|&(_, n)| n)
        .max()
        .unwrap_or(0);
    c.ground_inexact = (free.take_ground_inexact(), pin);
    c
}

/// Every game the census walks, in the lines' order.
fn games() -> impl Iterator<Item = &'static Game> {
    LATE_AGES.iter().chain(BATTERY_WORDS)
}

/// **The census walks every late-age and battery game**: its lists are the
/// handoff's `Late ages:` and `Battery word:` rows of `AI_WORDS`, in order.
#[test]
fn the_census_walks_every_late_age_and_battery_game() {
    // The held-out battery is never read (DECISIONS 61 §6): the battery
    // words are the lobbies promoted out of it, and none may still be in
    // it. Checked first, so a held-out row is named as one.
    let held: BTreeSet<&str> = crate::diff::drawn_games::held_out().collect();
    assert!(!held.is_empty(), "the held-out battery is listed");
    let read: Vec<&str> = games()
        .map(|g| g.name)
        .filter(|n| held.contains(n))
        .collect();
    assert!(
        read.is_empty(),
        "the census walks held-out lobbies: {read:?}"
    );
    for (line, list) in [("Late ages", LATE_AGES), ("Battery word", BATTERY_WORDS)] {
        let rows: Vec<&str> = crate::diff::testkit::AI_WORDS
            .iter()
            .filter(|w| w.line == line)
            .map(|w| w.map)
            .collect();
        let ours: Vec<&str> = list.iter().map(|g| g.name).collect();
        assert_eq!(ours, rows, "the census's {line} list is not AI_WORDS'");
    }
}

/// The walkers: `RON_TEST_CPU`, the slots the isolated runner reserved,
/// else the host's parallelism, at most four (`diff::coverage`'s rule).
fn walkers() -> usize {
    std::env::var("RON_TEST_CPU")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get().min(4)))
}

/// Walk every game on [`walkers`] threads, in the lines' order. Each
/// walker carries the test thread's name, which the fixture audit records
/// as the captures' requester.
fn census() -> Option<Vec<GameCensus>> {
    let inst = install()?;
    let loaded = crate::load::load(&inst).unwrap();
    let art = Art::new(&inst);
    let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
    let tribes = inst.tribe_defs(&inst.rules().unwrap()).unwrap();
    let sh = Shared {
        inst,
        loaded,
        art,
        look,
        tribes,
    };
    let only = std::env::var("RON_AGES_ONLY").ok();
    let list: Vec<&Game> = games()
        .filter(|g| {
            only.as_deref()
                .is_none_or(|o| o.split(',').any(|n| n == g.name))
        })
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut out: Vec<(usize, GameCensus)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..walkers().clamp(1, list.len().max(1)))
            .map(|_| {
                let (list, next, sh) = (&list, &next, &sh);
                let name = std::thread::current()
                    .name()
                    .unwrap_or("unnamed")
                    .to_owned();
                std::thread::Builder::new()
                    .name(name)
                    .spawn_scoped(scope, move || {
                        let mut mine = Vec::new();
                        while let Some(i) =
                            Some(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                                .filter(|&i| i < list.len())
                        {
                            let g = list[i];
                            // Asked for here, one game at a time: a trace
                            // held for every game at once peaked at 13 GiB.
                            let (Some(start), Some(long)) = (dump(g.start), dump(g.long.0)) else {
                                eprintln!("skipping: no {} captures (set RON_GAMELOG_DIR)", g.name);
                                continue;
                            };
                            let tr = trace(g.long.1);
                            let t = std::time::Instant::now();
                            let c = walk(g, &start, &long, tr.as_ref(), sh);
                            drop(tr);
                            eprintln!(
                                "AGES {}: resolved {} of {} on {} frames; closing {} of {}; \
                             K {}+{} of M {}+{}; inexact {:?}; {} ms; {:?}; closing partings {:?}",
                                c.name,
                                c.resolved,
                                c.objects,
                                c.frames,
                                c.closing.0,
                                c.closing.1,
                                c.k.0,
                                c.k.1,
                                c.m.0,
                                c.m.1,
                                c.ground_inexact,
                                t.elapsed().as_millis(),
                                c.by_kind,
                                c.closing_partings
                            );
                            mine.push((i, c));
                        }
                        mine
                    })
                    .expect("a census walker")
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap_or_else(|p| std::panic::resume_unwind(p)))
            .collect()
    });
    out.sort_by_key(|(i, _)| *i);
    Some(out.into_iter().map(|(_, c)| c).collect())
}

/// **Resolved X of Y objects over L lobbies** (item 4166, new):
/// `(resolved, objects, lobbies)` on the 4,608 dumped frames of the 82
/// late-age pairs and the 28 battery words. The objects are every entry of
/// the client's draw list on those frames — 1,167,081 resolve — and the
/// scene's projectiles and corpses, which the list cannot draw. Item 4169:
/// +481 from 1,166,600 — the sawmill's 22 and the smelter's 180, whose
/// unnamed animations name no file (`artdata::building_decoding`), and the
/// Air Defense line's 279, which loads as its `<UNIT>`
/// (`artdata::loads_as_unit`).
pub(crate) const AGES_RESOLVED: (usize, usize, usize) = (1_167_081, 1_171_799, 110);

/// The dumped frames walked: frame 0 and every `FRAME` each long prints.
pub(crate) const AGES_FRAMES: usize = 4_608;

/// **The unresolved, by kind and the file that fails**, objects summed
/// over the lobbies (item 4166, new), in the order RENDERER §3.13 ranks
/// them cheapest first. A type bent to a missing file adds a row here, so
/// the test turns red on it. Item 4169: seven building rows left, 481
/// objects (the sawmill 22, the smelter 98 + 82, the Air Defense line
/// 4 + 28 + 66 + 181); the crews, rounds and corpses stand.
pub(crate) const AGES_UNRESOLVED: &[(Kind, &str, usize)] = &[
    (
        Kind::Figure,
        "piece 38322 THECITIZEN-DEFAULT-AGE0-CREW3: slot 2 .\\art\\ceo_crew3_idle3.bha",
        9,
    ),
    (
        Kind::Figure,
        "piece 38323 THECEO-DEFAULT-AGE0-CREW3: slot 2 .\\art\\ceo_crew3_idle3.bha",
        3,
    ),
    (
        Kind::Figure,
        "piece 39378 THECITIZEN-ASIAN-AGE0-CREW3: slot 2 .\\art\\ceo_crew3_idle3.bha",
        2,
    ),
    (
        Kind::Figure,
        "piece 39379 THECEO-ASIAN-AGE0-CREW3: slot 2 .\\art\\ceo_crew3_idle3.bha",
        1,
    ),
    (Kind::Projectile, "the scene carries no piece", 2133),
    (Kind::Wreck, "the scene carries no piece", 2570),
];

/// **The closing block, record by record** (item 4166, new): of the
/// figures and buildings the 110 closing blocks print a `gpiece` for, whose
/// object the runner holds, those whose piece is ours. The five that part
/// are `GreatLakesAmericanInformation`'s Assault Marines (TypeIndex 122,
/// land) on frame 4321, RENDERER §3.13.
pub(crate) const AGES_CLOSING: (usize, usize) = (34_394, 34_399);

/// **The closing records that print the pieces item 4169 reads**:
/// `(gpiece, ours, printed)` over the 110 closing blocks. A piece the
/// original prints is one its loader loaded — `get_build_gpiece@0090bce0`
/// takes the first piece whose `data_pieces` entry is set, and
/// `get_unit_gpiece@0090c030` the first that exists — so a row here is the
/// dump's own word that the piece loads with what it names.
pub(crate) const AGES_CLOSING_WATCHED: &[(i32, usize, usize)] = &[
    (38322, 10, 10),
    (38323, 9, 9),
    (39378, 1, 1),
    (39379, 3, 3),
    (50803, 10, 10),
    (50804, 11, 11),
    (50805, 19, 19),
    (50963, 5, 5),
    (51092, 5, 5),
    (55219, 2, 2),
];

/// **K of M on each lobby** (§5.4 on the lobby's dumped frames: figure
/// pieces by the slots played and building types by the pieces drawn;
/// a member counts when every file decodes and the closing block's piece
/// is ours on every record of it): the lobbies whose K is M, of all.
/// Item 4169: 98, +15 from 83 — the lobbies short only through the
/// sawmill, the smelter or the Air Defense line.
pub(crate) const AGES_LOBBIES_WHOLE: (usize, usize) = (98, 110);

/// **The draw list across the ages resolves, and the closing block's
/// pieces are ours** (item 4166, `docs/RENDERER.md` §3.13): every
/// late-age pair and battery word walked by the free runner, the client's
/// list taken on each dumped frame and every object resolved to its own
/// install file; about 100 s on four walkers.
#[test]
fn the_draw_list_across_the_ages_resolves() {
    let _pins = Pins::hold();
    let Some(all) = census() else {
        return;
    };
    let mut unresolved: BTreeMap<(Kind, String), usize> = BTreeMap::new();
    let (mut x, mut y, mut frames, mut closing, mut whole) = (0, 0, 0, (0, 0), 0);
    let mut over: Vec<String> = Vec::new();
    let mut by_piece: BTreeMap<i32, (usize, usize)> = BTreeMap::new();
    for c in &all {
        for (p, (a, n)) in &c.closing_by_piece {
            let e = by_piece.entry(*p).or_default();
            e.0 += a;
            e.1 += n;
        }
        x += c.resolved;
        y += c.objects;
        frames += c.frames;
        closing.0 += c.closing.0;
        closing.1 += c.closing.1;
        whole += usize::from(c.k == c.m);
        for (k, n) in &c.unresolved {
            *unresolved.entry(k.clone()).or_default() += n;
        }
        if c.ground_inexact.0 > c.ground_inexact.1 {
            over.push(format!("{}: {:?}", c.name, c.ground_inexact));
        }
    }
    eprintln!(
        "AGES resolved {x} of {y} objects over {} lobbies on {frames} frames; closing {} of {}; \
         K = M on {whole}",
        all.len(),
        closing.0,
        closing.1
    );
    for ((k, w), n) in &unresolved {
        eprintln!("AGES unresolved {k:?} {w}: {n}");
    }
    // Each game reads no more inexact corners than its own tests' pin
    // (`Built`'s guard, which the runner hands its count to).
    pin_eq!(
        over,
        Vec::<String>::new(),
        "games over their inexact-ground pin"
    );
    let mut pinned: Vec<(Kind, String, usize)> = AGES_UNRESOLVED
        .iter()
        .map(|&(k, w, n)| (k, w.to_string(), n))
        .collect();
    pinned.sort();
    let ours: Vec<(Kind, String, usize)> = unresolved
        .into_iter()
        .map(|((k, w), n)| (k, w, n))
        .collect();
    pin_eq!(ours, pinned, "the unresolved, by kind and file");
    pin_eq!(
        (x, y, all.len()),
        AGES_RESOLVED,
        "resolved of objects over lobbies"
    );
    pin_eq!(frames, AGES_FRAMES, "dumped frames walked");
    pin_eq!(closing, AGES_CLOSING, "the closing block's pieces ours");
    pin_eq!(
        (whole, all.len()),
        AGES_LOBBIES_WHOLE,
        "lobbies whose K is M"
    );
    let watched: Vec<(i32, usize, usize)> = AGES_CLOSING_WATCHED
        .iter()
        .map(|&(p, _, _)| {
            let (a, n) = by_piece.get(&p).copied().unwrap_or_default();
            (p, a, n)
        })
        .collect();
    pin_eq!(
        watched,
        AGES_CLOSING_WATCHED.to_vec(),
        "the closing records printing the watched pieces, ours of all"
    );
}
