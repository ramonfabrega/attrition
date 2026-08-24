//! The gamelog diff: the original's per-frame dump against the simulation.
//!
//! Phase 3's score is ticks before divergence. This is the harness that
//! produces it: [`build_sim`] stands a `sim::Sim` up from a start-of-game
//! dump ([`crate::gamelog::Initial`]) with the loaded types
//! ([`crate::load::Loaded`]), and [`run`] steps it frame by frame against the
//! dump's `FRAME n` blocks, comparing what both sides hold.
//!
//! # What it can see today, and what it cannot
//!
//! At detail level 0 the original writes, per unit per frame, the object base
//! — owner, object number, `x_internal`/`y_internal`/`z_internal` — and per
//! leader its `score` and flags. So the comparison is **positions of units
//! keyed by `(who, o)`**, and the score is reported but not matched (the
//! simulation has no score yet). The per-frame `GUY` blocks are empty, the
//! leaders carry no goods, and buildings are not written per frame under the
//! categories the logged runs enabled; `DUMP_ALL=1` is the untried lever for
//! more (`docs/ORACLE.md`).
//!
//! And the simulation has no AI and sees no orders: in the logged runs the
//! human's units stand still and the AI's move as soon as it decides to, so
//! the honest expectation is that player 0 matches for as long as the human
//! did nothing and the AI player diverges the frame its first order lands.
//! That is still worth running — it proves the ids, the types, the world
//! scale and the step cadence agree end to end — and it is the scaffold the
//! order stream plugs into when the recorded-game container is read.

use crate::gamelog::{Frame, Initial, Log, Pos as LogPos, UnitDump};
use crate::load::Loaded;
use sim::{Pos, Sim, Tuning, Unit, World};

/// What a `(who, o)` unit in the log is in the simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitLink {
    pub who: i64,
    pub o: i64,
    /// The simulation's unit index.
    pub unit: usize,
    /// The unit record, if the `GUY` type named one.
    pub kind: Option<usize>,
}

/// The simulation plus the maps back into the log's ids.
#[derive(Debug)]
pub struct Built {
    pub sim: Sim,
    pub units: Vec<UnitLink>,
    /// The pre-placed buildings: simulation handle → the log's object number.
    pub builds: Vec<(usize, i64)>,
    /// The dump's region numbers → the sim's region ids, when the dump
    /// carried the map (`WORLD ≥ 5`); empty on the flat world.
    pub region_map: Vec<(i64, u16)>,
    /// Anything that could not be carried over, and why.
    pub notes: Vec<String>,
    /// The sync stream's word at the end of each engine frame the dump (or
    /// a sibling) traced — [`Built::tick`] installs them.
    pub frame_seeds: Vec<(i64, u32)>,
    /// Per traced frame ticked so far: the sim's draw count and the
    /// original's.
    pub rng_frames: Vec<(i64, Option<u32>, Option<u32>)>,
}

/// How many player slots the log's leaders occupy: the `who`s below eight.
/// The original keeps eight slots and uses 8 and 9 for the two non-player
/// leaders (`LEADERDATA who 8`, `who 9` in every dump).
pub fn player_count(init: &Initial) -> usize {
    init.leaders
        .iter()
        .filter(|l| (0..8).contains(&l.who))
        .map(|l| l.who as usize + 1)
        .max()
        .unwrap_or(0)
}

fn pos_of(p: LogPos) -> Pos {
    Pos::new(p.x as i32, p.y as i32)
}

/// Stands a simulation up from the initial dump.
///
/// The world is `xs × ys` cells from `WORLD`; every unit the dump lists for
/// a player slot becomes one simulation unit at its object position, typed
/// by its first `GUY`'s `type` (a `TypeIndex`; `0x32` is the Citizen), with
/// its squad size the number of `GUY`s and its health the type's `HITS`.
/// Each player's tribe comes from `LEADERDATA`. The city each player starts
/// with is placed by type at the `CITIES` position when the placement rules
/// allow it, and as an untyped building otherwise — with a note.
/// The lobby, from the dump's `GAMEINFO` block — every `GameInfo` option
/// field is logged under its own upper-case name (`docs/AI.md` §12.1). A
/// field the block lacks keeps the run7 default.
pub fn lobby_of(game_info: &[(&str, &str)], map_styles: &[String]) -> sim::ai::Lobby {
    let mut l = sim::ai::Lobby::default();
    let int = |key: &str| -> Option<i32> {
        game_info
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    if let Some(v) = int("TEAM_STYLE") {
        l.team_style = v;
    }
    if let Some(v) = int("MAP_STYLE") {
        l.map_style = v;
        if let Some(name) = usize::try_from(v).ok().and_then(|i| map_styles.get(i)) {
            l.map_style_name = name.clone();
        }
    }
    if let Some(v) = int("DIFFICULTY") {
        l.difficulty = v;
    }
    if let Some(v) = int("STARTING_TOWN") {
        l.starting_town = v;
    }
    if let Some(v) = int("STARTING_RESOURCES") {
        l.starting_resources = v;
    }
    if let Some(v) = int("STARTING_RESOURCES2") {
        l.starting_resources2 = v;
    }
    if let Some(v) = int("GAME_RULES") {
        l.game_rules = v;
    }
    if let Some(v) = int("TECH_COST") {
        l.tech_cost = v;
    }
    if let Some(v) = int("RUSH_RULES") {
        l.rush_rules = v;
    }
    if let Some(v) = int("ELIMINATION") {
        l.elimination = v;
    }
    if let Some(v) = int("VICTORY") {
        l.victory = v;
    }
    if let Some(v) = int("flags") {
        l.no_nation_powers = v & 4 != 0;
    }
    l
}

pub fn build_sim(loaded: &Loaded, init: &Initial, tuning: Tuning) -> Built {
    let mut notes = Vec::new();
    let get = |k: &str| -> Option<i64> {
        init.world
            .iter()
            .find(|(key, _)| *key == k)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let xs = get("xs").unwrap_or(60).max(1) as i32;
    let ys = get("ys").unwrap_or(60).max(1) as i32;
    let players = player_count(init).max(1);
    // The dump carries no terrain at level 0 (`docs/DATALAYER.md`, "what is
    // not established"), so the harness's map is **one land region covering
    // every cell** — flat, unblocked, no water. That assumption was implicit
    // before and cost something: a cell in no region is `Blocked::Ruins` to
    // `blocked_tcoord`, so every `place_building` was refused and every
    // building came out untyped and city-less.
    let mut world = World::new(xs, ys);
    let cells = crate::gamelog::world_cells(&init.world);
    let mut region_map: Vec<(i64, u16)> = Vec::new();
    if cells.len() == (xs as usize) * (ys as usize) {
        // A `WORLD ≥ 5` start dump: the map's own cells — region, the
        // coastal `region2`, the site value, the goods bits, the owners
        // (`docs/ORACLE.md`, "The map is a dump too"). The dump's region
        // numbers are the original's (`0..0x3e` land, `0x3f..0x7e` sea,
        // `docs/AI.md` §2.3); each distinct one becomes a sim region.
        let mut region_of = |n: i64, world: &mut World| -> Option<u16> {
            if n < 0 {
                return None;
            }
            if let Some((_, r)) = region_map.iter().find(|(d, _)| *d == n) {
                return Some(*r);
            }
            let terrain = if n < 0x3f {
                sim::world::Terrain::Land
            } else {
                sim::world::Terrain::Sea
            };
            let r = world.add_region(terrain);
            region_map.push((n, r));
            Some(r)
        };
        // `CellData.land` is the original's `land_key[]` index; the dump
        // prints the name, so until that table is read the harness numbers
        // the names in order of first appearance and keeps the list.
        let mut land_names: Vec<&str> = Vec::new();
        for (i, c) in cells.iter().enumerate() {
            let cell = sim::world::Cell::new((i % xs as usize) as i32, (i / xs as usize) as i32);
            if let Some(r) = region_of(c.region, &mut world) {
                world.set_region(cell, r);
            }
            let region2 = region_of(c.region2, &mut world);
            let land = match land_names.iter().position(|n| *n == c.land) {
                Some(i) => i,
                None => {
                    land_names.push(c.land);
                    land_names.len() - 1
                }
            } as i8;
            let owner = |w: i64| -> sim::world::Owner {
                match w {
                    -2 => sim::world::Owner::Ambiguous,
                    w if (0..=255).contains(&w) => sim::world::Owner::Player(w as sim::Player),
                    _ => sim::world::Owner::None,
                }
            };
            world.set_owner(cell, owner(c.who), owner(c.who2));
            world.set_cell_data(
                cell,
                sim::world::CellData {
                    flags: c.flags as u16,
                    land,
                    land_sub: c.land_sub as u8,
                    region2,
                    val: c.val as u8,
                    goods: c.goods as u8,
                    blocked: c.blocked as u8,
                    solid: c.solid as i8,
                    down: c.down as i16,
                    down_who: c.down_who as i8,
                },
            );
        }
        // The per-tile masks — forest, mountain, river, ocean, the city
        // radii and the footprints — `tile_xs × tile_ys` row-major.
        let tiles = crate::gamelog::world_tiles(&init.world);
        let tw = (xs * sim::world::TILES_PER_CELL) as usize;
        let th = (ys * sim::world::TILES_PER_CELL) as usize;
        let tiles_loaded = tiles.len() == tw * th;
        if tiles_loaded {
            for (i, m) in tiles.iter().enumerate() {
                let t = Pos::new((i % tw) as i32, (i / tw) as i32);
                world.set_tile_mask(t, *m);
            }
        }
        // The heights — `TerrainOut::find_tcoord_z@008544a0` per tile:
        // `(int)((h[ty+1][tx] + h[ty][tx+1]) × 0.5)` on the corner grid of
        // `4·xs + 1` columns, 0 where the tile's surface bits are ocean
        // (`mask & 0x30 == 0x20`). The mean of two exact millionths,
        // truncated toward zero, is the float's truncation exactly.
        let hw = tw + 1;
        let heights_loaded = tiles_loaded && init.heights.len() == hw * (th + 1);
        if heights_loaded {
            let h = &init.heights;
            for ty in 0..th {
                for tx in 0..tw {
                    let t = Pos::new(tx as i32, ty as i32);
                    let ocean = world.tile_mask(t) & sim::world::tile::SURFACE
                        == sim::world::tile::SURFACE_OCEAN;
                    let z = if ocean {
                        0
                    } else {
                        ((h[(ty + 1) * hw + tx] + h[ty * hw + tx + 1]) / 2_000_000) as i32
                    };
                    world.set_tile_z(t, z);
                }
            }
        }
        notes.push(format!(
            "world: {} cells from the WORLD dump, {} regions, land kinds {:?}, {} tile masks{}, {}",
            cells.len(),
            region_map.len(),
            land_names,
            tiles.len(),
            if tiles_loaded {
                ""
            } else {
                " (not applied: count differs)"
            },
            if heights_loaded {
                "heights pinned per tile"
            } else if init.heights.is_empty() {
                "no height table (flat)"
            } else {
                "height table not applied: size differs"
            }
        ));
    } else {
        // The dump carries no cells (`WORLD < 3`), so the harness's map is
        // **one land region covering every cell** — flat, unblocked, no
        // water. That assumption was implicit before and cost something: a
        // cell in no region is `Blocked::Ruins` to `blocked_tcoord`, so
        // every `place_building` was refused and every building came out
        // untyped and city-less.
        let land = world.add_region(sim::world::Terrain::Land);
        for y in 0..ys {
            for x in 0..xs {
                world.set_region(sim::world::Cell::new(x, y), land);
            }
        }
    }
    let mut sim = loaded.sim(tuning, world, players);
    sim.lobby = lobby_of(&init.game_info, &loaded.map_styles);

    for l in &init.leaders {
        if (0..players as i64).contains(&l.who) {
            let who = l.who as usize;
            sim.tech[who].tribe = l.tribe.max(0) as usize;
            sim.tech[who].power = Some(l.tribe.max(0) as usize);
            // `leader_flags & 4` is `LeaderData::is_human` — the bit that
            // picks `find_wpath`'s pull-back variant and withholds the
            // `army`/`worker` cost modes (`docs/PATHFINDER.md` §3).
            sim.nation[who].human = l.leader_flags & 4 != 0;
        }
    }

    // The opening scripts, compiled against the host's table
    // (`Leaders::init_production_script`), then `Leader::init`'s AI tail for
    // every computer leader: the personality roll and the script choice.
    // The roll draws from the sync stream at the point `Setup::build_game`
    // calls `Leader::init` — after the map maker's and the start
    // permutation's draws (`docs/ORDERS.md` §9.2), which the harness does
    // not model. A dump with the setup path's checksum trace
    // (`docs/ORACLE.md`, "The setup path's checksum trace is the RNG
    // state") carries that state exactly: `leaders.cpp` line 13383 is the
    // checkpoint just before `random_personality` and 13457 the one after
    // the script choice, so each computer leader's roll is seeded from its
    // bracket and checked against the far end. Without the trace the roll
    // draws from the sim's own stream and a test that needs the original's
    // personality sets it from a `LEADERS=9` dump's `PERSONALITY` block
    // (`docs/AI.md` §5).
    if !loaded.scripts.is_empty() {
        let sources: Vec<sim::bhs::Source> = loaded
            .scripts
            .iter()
            .map(|(name, text)| sim::bhs::Source { name, text })
            .collect();
        if let Err(e) = sim.load_scripts(&["economic.bhs", "defensive.bhs"], &sources) {
            notes.push(format!("the opening scripts did not compile: {e}"));
        }
    }
    let mut brackets = personality_brackets(&init.checksums).into_iter();
    for who in 0..players {
        if sim.nation[who].human {
            continue;
        }
        let flags = init
            .players
            .iter()
            .find(|p| {
                p.iter()
                    .any(|(k, v)| *k == "who" && v.trim() == who.to_string())
            })
            .and_then(|p| p.iter().find(|(k, _)| *k == "flags"))
            .and_then(|(_, v)| v.trim().parse::<u32>().ok())
            .unwrap_or(0);
        let bracket = brackets.next();
        if let Some((before, _)) = bracket {
            sim.rng.seed = before;
        }
        sim.init_leader_ai(who as sim::Player, flags);
        if let Some((before, after)) = bracket {
            if sim.rng.seed == after {
                notes.push(format!(
                    "player {who}: personality rolled from the trace's Leader::init state {before:#010x}, landing on {after:#010x} as the original did"
                ));
            } else {
                notes.push(format!(
                    "player {who}: personality rolled from {before:#010x} lands on {:#010x}, the original on {after:#010x} — the draw count differs",
                    sim.rng.seed
                ));
            }
        }
    }

    // The starting city, by type at the logged position.
    let city_ty = loaded.build_named("Small City");
    for c in &init.cities {
        if !(0..players as i64).contains(&c.who) {
            continue;
        }
        let who = c.who as sim::Player;
        let pos = Pos::new(c.x as i32, c.y as i32);
        // `build_cities@005ab910`: `Build::init(who, VILLAGE, o, x, y, 0)`
        // then `Build::activate(0, 0, 0)` — no cost, no placement test, and
        // the starting city is finished and active at frame 0 (§9.2). Going
        // through `place_building` would charge for it and re-run
        // `blocked_site`, neither of which the original does here.
        let placed = match city_ty {
            Some(ty) => {
                let b = sim.init_build(who, ty, pos, false);
                sim.activate(b, false, false);
                Ok(b)
            }
            None => Err("no Small City type".to_string()),
        };
        if let Err(why) = placed {
            let b = sim.add_building(who, pos, 8);
            sim.buildings[b].ty = city_ty;
            notes.push(format!(
                "player {who}: the city at {} {} was added untyped — place_building refused it ({why})",
                c.x, c.y
            ));
        }
    }

    // The units.
    let mut units = Vec::new();
    for u in &init.units {
        if !(0..players as i64).contains(&u.who) {
            continue;
        }
        let kind = u
            .guys
            .first()
            .and_then(|g| g.kind)
            .and_then(|t| loaded.unit_of_type_index(t as i32));
        if u.guys.first().is_some_and(|g| g.kind.is_some()) && kind.is_none() {
            notes.push(format!(
                "unit who {} o {}: GUY type {:?} is not a unit TypeIndex",
                u.who, u.o, u.guys[0].kind
            ));
        }
        let ty = kind.map(|k| &loaded.unit_types[k]);
        let health = ty.map_or(1, |t| t.hits.max(1));
        let mut unit = Unit::new(u.who as sim::Player, u.o as i16, pos_of(u.pos), health);
        unit.squad_size = u.guys.len().max(1) as i32;
        unit.ty = kind;
        if let Some(t) = ty {
            unit.kind = t.kind;
            unit.movement.speed = t.moves;
            unit.movement.turning.type_turn_speed = t.turn_speed;
        }
        let idx = sim.add_unit(unit);
        // `Unit::init`'s tally — `num_units`, the group count, `control` —
        // which the price ramp and `population()` read. A unit stood up
        // from the dump counts exactly as a trained one does.
        if let Some(ty) = kind {
            let who = u.who as usize;
            let group = sim.unit_types[ty].group;
            let pop = sim.unit_types[ty].price.pop;
            let m = &mut sim.muster[who];
            m.by_type[ty] += 1;
            if let Some(g) = group {
                m.by_group[g] += 1;
            }
            m.control += pop;
        }
        units.push(UnitLink {
            who: u.who,
            o: u.o,
            unit: idx,
            kind,
        });
    }

    let builds = start_of_game(&mut sim, loaded, init, players, &units, &mut notes);
    // On the flat fallback world a camp's survey (`gather.rs`) finds no
    // forest and would cap it at zero slots; the map-less dumps (run6, run7)
    // keep the old uncapped stand-in so their pins measure what they did.
    if region_map.is_empty() {
        for b in &mut sim.buildings {
            let flat =
                b.ty.is_some_and(|t| sim.build_types[t].has(sim::build::flags::FLAT));
            if !flat {
                b.gather_max = None;
            }
        }
    }
    // The herds, as the dump's `HERDS` block prints them — their walk every
    // 64 frames is on the sync stream (`docs/SYNC.md` §3.2).
    for h in &init.herds {
        sim.gaia.herds.push(sim::gaia::Herd {
            cx: h.cx as i32,
            cy: h.cy as i32,
            wx: h.wx as i32,
            wy: h.wy as i32,
            kind: h.t as i32,
            alive: h.herd_flags & 1 != 0,
        });
    }
    if !init.herds.is_empty() {
        notes.push(format!("herds: {} from the dump", init.herds.len()));
    }
    // The sync stream entering frame 0: the trace's last record is the end
    // of `Game::init` (`game.cpp` 5024 on this build), after the empires,
    // the herds and the scripts — every draw between `Leader::init` and the
    // first `do_frame` the harness does not model (`build_empire`'s 1,293
    // on run11, `Herd::create_units`' 44). Nothing draws between there and
    // `GameLog::begin_game`, so this is the state the first frame reads.
    if let Some(last) = init.checksums.last() {
        sim.rng.seed = last.seed;
        notes.push(format!(
            "rng: seeded {:#010x} from the trace's last checkpoint ({} {}, CHECKSUM {})",
            last.seed, last.file, last.line, last.n
        ));
    } else {
        notes.push(
            "rng: no checksum trace in the dump; the stream is the sim's own (frame-0 AI draws will not match)"
                .to_string(),
        );
    }
    Built {
        sim,
        units,
        builds,
        region_map,
        notes,
        frame_seeds: init.frame_seeds.clone(),
        rng_frames: Vec::new(),
    }
}

/// `Leader::init`'s source lines around `random_personality` on this build
/// (`leaders.cpp` in the EE-era `rise.pdb`): 13383 (`0x3447`) is the
/// checkpoint just before the roll, 13457 (`0x3491`) the one after the
/// script choice. A human's visit passes both with no draw; a computer
/// leader's moves the seed.
const PERSONALITY_BEFORE: (&str, i64) = ("leaders.cpp", 13383);
const PERSONALITY_AFTER: (&str, i64) = ("leaders.cpp", 13457);

/// Each computer leader's `(before, after)` sync-stream states around its
/// personality roll, in `Setup::build_game`'s visiting order — the
/// consecutive checkpoint pairs at the two lines whose seed changed.
pub fn personality_brackets(checksums: &[crate::gamelog::Checksum<'_>]) -> Vec<(u32, u32)> {
    checksums
        .windows(2)
        .filter(|w| {
            (w[0].file, w[0].line) == PERSONALITY_BEFORE
                && (w[1].file, w[1].line) == PERSONALITY_AFTER
                && w[0].seed != w[1].seed
        })
        .map(|w| (w[0].seed, w[1].seed))
        .collect()
}

/// `TypeIndex` values the start-of-game rule names (`docs/ORDERS.md` §9.2).
const FARM: i32 = 417;
const WOODCUTTER: i32 = 418;

/// The pre-placed buildings and the starting citizens' gather orders, derived
/// from `docs/ORDERS.md` §9.2 and §9.3 — **not** read out of the dump.
///
/// The dump's `BUILDDATA` carries no type at any detail level, so the
/// buildings are typed by `Leader::produce_building`'s order: `2001` is the
/// woodcutter, then the farms, then the rest (a library, towers, a
/// civ-specific building). Only the gather types matter to the simulation —
/// a farm is flat and a woodcutter is not, which is the difference between
/// "walk to the centre and stand" and the whole §6.4 walk-out machine — so
/// everything after the farms is left untyped.
///
/// The citizens are then assigned by §9.3's four-step rule, as the second
/// reading corrected it (`docs/audit/2026-08-21-orders.md` R5 U10, U11):
/// the first `ordered` of them take `2001` **unconditionally**, and the rest
/// scan from `2002` for successive farms.
fn start_of_game(
    sim: &mut Sim,
    loaded: &Loaded,
    init: &Initial,
    players: usize,
    units: &[UnitLink],
    notes: &mut Vec<String>,
) -> Vec<(usize, i64)> {
    let Some(woodcutter) = loaded.build_of_type_index(WOODCUTTER) else {
        notes.push("no WOODCUTTER build type; start-of-game not placed".into());
        return Vec::new();
    };
    let Some(farm) = loaded.build_of_type_index(FARM) else {
        notes.push("no FARM build type; start-of-game not placed".into());
        return Vec::new();
    };

    let mut all_builds: Vec<(usize, i64)> = Vec::new();
    for who in 0..players as sim::Player {
        let w = who as i64;
        // The citizens of this player, by object number — `Setup::build_units`
        // creates them in index order after the scout.
        let mut citizens: Vec<&UnitLink> = units
            .iter()
            .filter(|l| {
                l.who == w
                    && l.kind
                        .is_some_and(|k| sim.unit_types[k].worker != sim::orders::Worker::None)
            })
            .collect();
        citizens.sort_by_key(|l| l.o);
        if citizens.is_empty() {
            continue;
        }

        // `get_starting_citizens`: (n, ordered) is (5, 2) for a Small Town and
        // (10, 5) for a Large Town, and the two lists carry three and five
        // farms. `starting_resources` and the nation powers adjust n, so the
        // match is on the plain counts and anything else falls back.
        let (ordered, farms) = match citizens.len() {
            5 => (2usize, 3usize),
            10 => (5, 5),
            n => {
                notes.push(format!(
                    "player {who}: {n} citizens is neither a Small ({}) nor a Large ({}) Town — \
                     assuming the Small Town list; §9.3's count adjustments are not modelled",
                    5, 10
                ));
                (2, 3)
            }
        };

        // Place the buildings `2001..` in production order.
        let mut sites: Vec<(i64, usize)> = Vec::new();
        for b in init.builds.iter().filter(|b| b.who == w && b.o >= 2001) {
            let idx = (b.o - 2001) as usize;
            let derived = if idx == 0 {
                Some(woodcutter)
            } else if idx <= farms {
                Some(farm)
            } else {
                None
            };
            // **Derive, then read.** At `BUILDS=6` and above the dump carries
            // `orig_type`, so the type no longer has to be inferred from
            // `produce_building`'s order — but §9.2's rule is still a claim
            // worth checking, so the derivation runs anyway and any
            // disagreement is a note. The dump wins where it speaks.
            let logged = b
                .orig_type
                .and_then(|t| loaded.build_of_type_index(t as i32));
            if let (Some(d), Some(l)) = (derived, logged)
                && d != l
            {
                notes.push(format!(
                    "player {who} building {}: §9.2 derives {:?}, the dump's orig_type is {:?}",
                    b.o,
                    loaded.build_names.get(d),
                    loaded.build_names.get(l)
                ));
            }
            let ty = logged.or(derived);
            let pos = Pos::new(b.pos.x as i32, b.pos.y as i32);
            // Every pre-placed building is complete and active at frame 0:
            // `produce_building` at `frame == 0` skips `pay_cost` and the
            // swarm, and is `Objects::init_build(…)` then `activate(0, 1, 0)`
            // (§9.2, R5 C5). `init_build` is what joins the building to the
            // city it stands in (`Wall::find_city`), and that membership is
            // load-bearing: `do_gather` sends a citizen away from a farm
            // outside a city, which is what the order diff caught when the
            // harness went through `place_building` and it was refused.
            let handle = match ty {
                Some(t) => {
                    let h = sim.init_build(who, t, pos, false);
                    sim.activate(h, false, true);
                    h
                }
                None => {
                    let h = sim.add_building(who, pos, 8);
                    sim.buildings[h].active = true;
                    sim.buildings[h].activated = true;
                    sim.buildings[h].started = true;
                    h
                }
            };
            // The mining list, straight from the dump — `BUILDS=7`. It is an
            // input by construction: the original fills it from the terrain
            // (`Build::find_gather_tiles`), which no dump carries, and
            // without it §6.4's machine has nowhere to send a woodcutter's
            // citizen and the citizen never leaves the camp.
            if !b.gather_from.is_empty() {
                sim.buildings[handle].gather_from = b
                    .gather_from
                    .iter()
                    .map(|&(x, y)| Pos::new(x as i32, y as i32))
                    .collect();
                // `Build::find_gather_tiles` line 88: the list is what the
                // slot count is surveyed from (`crates/sim/src/gather.rs`).
                sim.buildings[handle].gather_max = Some(sim.max_gatherers(handle));
            }
            // `find_free` numbers the building as it is placed; the dump's
            // `o` is the original's own numbering of the same placement
            // order, so the two agree unless the order here is wrong.
            if i64::from(sim.buildings[handle].index) != b.o {
                notes.push(format!(
                    "player {who} building {}: find_free numbered it {} — \
                     the buildings were not placed in the original's order",
                    b.o, sim.buildings[handle].index
                ));
            }
            all_builds.push((handle, b.o));
            if ty.is_some() {
                sites.push((b.o, handle));
            }
        }
        if sites.is_empty() {
            continue;
        }

        // §9.3's assignment. `2001` for the first `ordered`; then successive
        // farms from `2002`, the cursor advancing past each one taken.
        let site_at = |o: i64| sites.iter().find(|(n, _)| *n == o).map(|(_, h)| *h);
        let mut cursor = 0i64;
        for (i, link) in citizens.iter().enumerate() {
            let target = if i < ordered {
                // Unconditional — a dead `2001` sends the citizen idle rather
                // than to a farm (R5 U10).
                site_at(2001)
            } else {
                let t = site_at(2002 + cursor);
                if t.is_some() {
                    cursor += 1;
                }
                t
            };
            let Some(b) = target else {
                continue; // `place_unit`, idle — the fallback's last step.
            };
            if !sim.is_gather_type(b) {
                continue;
            }
            sim.add_gather_order(link.unit, b, sim::orders::QueuePos::New, false);
        }
    }
    all_builds
}

/// What two order lists, or two path stacks, disagree about.
///
/// The order list is the unit's *intent*, and it diverges long before a
/// position does — or, worse, never shows in a position at all, which is the
/// whole reason this comparison exists (`check_start_orders`' doc comment
/// says it for the starting orders; this says it for every frame).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderMismatch {
    /// The lists are different lengths.
    Length { ours: usize, theirs: usize },
    /// `get_type()` — the `OrderIndex` of the order in this slot.
    Kind { ours: i64, theirs: i64 },
    /// The action bit, `UnitOrder::flags & 4` (§1.3): an intent rather than
    /// a transit leg. Settled, so it scores.
    Action { ours: bool, theirs: bool },
    /// `TargetOrder`'s `whom`/`ox`: whose object, and which.
    Target {
        ours: Option<(i64, i64)>,
        theirs: Option<(i64, i64)>,
    },
    /// The whole `flags` byte. **Does not score**: `0x8` and `0x10` have no
    /// established reader (`docs/ORDERS.md` §14) and `0x1` (`PATHED`) follows
    /// the path stack, which the pathfinder stub does not reproduce. Reported
    /// because a surprise here is worth seeing.
    Flags { ours: u8, theirs: i64 },
    /// The path stack's depth.
    PathLength { ours: usize, theirs: usize },
    /// A path segment's goal, bottom-first.
    PathTo {
        slot: usize,
        ours: (i32, i32),
        theirs: (i64, i64),
    },
}

impl OrderMismatch {
    /// Whether this is about the path stack rather than the order list.
    pub const fn is_path(&self) -> bool {
        matches!(self, Self::PathLength { .. } | Self::PathTo { .. })
    }

    /// Whether it counts against the order score. Everything does except
    /// [`Self::Flags`], for the reason on that variant.
    pub const fn scores(&self) -> bool {
        !matches!(self, Self::Flags { .. })
    }

    /// The variant's name, for a tally.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Length { .. } => "length",
            Self::Kind { .. } => "kind",
            Self::Action { .. } => "action",
            Self::Target { .. } => "target",
            Self::Flags { .. } => "flags",
            Self::PathLength { .. } => "path-length",
            Self::PathTo { .. } => "path-to",
        }
    }
}

/// One unit's order list or path stack disagreeing on one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// Position in the list, **front first** — slot 0 is the order being
    /// executed. The log writes the list the other way round
    /// ([`UnitDump::orders_front_first`]).
    pub slot: usize,
    pub what: OrderMismatch,
}

/// One unit whose position the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: Pos,
    pub theirs: Pos,
}

/// The outcome of one frame's comparison.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameResult {
    pub frame: i64,
    /// Units the log has for a player and the simulation does not.
    pub unlinked: usize,
    /// The `(who, o)` of each unlinked unit-frame — the units the original
    /// has on this frame that the simulation does not.
    pub unlinked_units: Vec<(i64, i64)>,
    pub compared: usize,
    pub diverged: Vec<Divergence>,
    /// The leaders' scores as logged, by `who`.
    pub scores: Vec<(i64, i64)>,
    /// Units whose order list the log carries, so both sides could be
    /// compared — zero below `UNITS=3`.
    pub order_compared: usize,
    /// Every order-list and path-stack disagreement this frame.
    pub order_diverged: Vec<OrderDivergence>,
}

impl FrameResult {
    /// The order-list disagreements only — what the two sides *intend*.
    pub fn order_only(&self) -> impl Iterator<Item = &OrderDivergence> {
        self.order_diverged.iter().filter(|d| !d.what.is_path())
    }

    /// The path-stack disagreements only — what the pathfinder seam costs.
    pub fn path_only(&self) -> impl Iterator<Item = &OrderDivergence> {
        self.order_diverged.iter().filter(|d| d.what.is_path())
    }
}

/// The whole run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub frames: Vec<FrameResult>,
    /// Per player slot: the first frame a unit of theirs diverged, if any.
    pub first_divergence: Vec<(i64, Option<i64>)>,
    pub notes: Vec<String>,
    /// What the recorded order stream did, when one was fed in — orders
    /// enqueued, and every command that was carried but not acted on.
    pub applied: crate::input::Applied,
    /// Per traced frame: the sim's draw count and the original's, each
    /// `None` when the words are more than [`DRAW_CAP`] apart.
    pub rng_frames: Vec<(i64, Option<u32>, Option<u32>)>,
}

/// The furthest `draws_between` walks.
pub const DRAW_CAP: u32 = 200_000;

/// How many `Random::get` steps take the sync stream from `from` to `to`,
/// if fewer than [`DRAW_CAP`].
pub fn draws_between(from: u32, to: u32) -> Option<u32> {
    let mut r = sim::combat::Rng::new(from);
    for n in 0..=DRAW_CAP {
        if r.seed == to {
            return Some(n);
        }
        r.roll();
    }
    None
}

impl Report {
    /// Ticks before divergence: the last frame on which every compared unit
    /// agreed, or the number of frames if none ever disagreed.
    pub fn ticks_before_divergence(&self) -> i64 {
        self.frames
            .iter()
            .find(|f| !f.diverged.is_empty())
            .map_or(self.frames.len() as i64, |f| f.frame - 1)
    }

    /// The frame each `(who, o)` first disagreed on, for the units that ever
    /// did — the breakdown the single score cannot show.
    ///
    /// The score is a minimum over every unit of both players, so one unit
    /// the simulation cannot yet drive (an AI-ordered scout, a woodcutter
    /// whose tile list needs `BUILDS=7`) pins it at the floor while every
    /// other unit may be tracking perfectly. Reading which units diverge, and
    /// when, is what says whether a mechanic landed.
    /// Whether any frame carried an order list at all — `UNITS=3`. Without
    /// it every order figure below is zero and means nothing.
    pub fn orders_seen(&self) -> bool {
        self.frames.iter().any(|f| f.order_compared > 0)
    }

    /// The order score: the last frame on which every compared unit's order
    /// list **and path stack** agreed.
    ///
    /// It is a second score beside [`Self::ticks_before_divergence`], not a
    /// replacement, and it is the stricter of the two — two simulations can
    /// agree on every position for a whole dump and still have given every
    /// unit the wrong job. Path-stack disagreements **score** since the
    /// pathfinder landed (`docs/PATHFINDER.md` §10): the stack is now a
    /// modelled output, and how many frames it survives is the mechanic's
    /// grade — exactly the acceptance test the brief named in advance.
    pub fn order_ticks_before_divergence(&self) -> i64 {
        self.frames
            .iter()
            .find(|f| f.order_diverged.iter().any(|d| d.what.scores()))
            .map_or(self.frames.len() as i64, |f| f.frame - 1)
    }

    /// The frame each `(who, o)` first disagreed on an order, and what about
    /// — the breakdown, as [`Self::first_divergence_by_unit`] is for
    /// positions. Path-stack entries are included and marked by their
    /// variant.
    pub fn order_divergence_by_unit(&self) -> Vec<(i64, i64, i64, OrderMismatch)> {
        let mut seen: std::collections::BTreeMap<(i64, i64), (i64, OrderMismatch)> =
            std::collections::BTreeMap::new();
        for f in &self.frames {
            for d in &f.order_diverged {
                seen.entry((d.who, d.o)).or_insert((f.frame, d.what));
            }
        }
        seen.into_iter()
            .map(|((w, o), (f, what))| (w, o, f, what))
            .collect()
    }

    pub fn first_divergence_by_unit(&self) -> Vec<(i64, i64, i64)> {
        let mut seen: std::collections::BTreeMap<(i64, i64), i64> =
            std::collections::BTreeMap::new();
        for f in &self.frames {
            for d in &f.diverged {
                seen.entry((d.who, d.o)).or_insert(f.frame);
            }
        }
        seen.into_iter().map(|((w, o), f)| (w, o, f)).collect()
    }
}

impl Built {
    /// One engine frame, the harness's way: `Sim::tick`, then — when the
    /// dump or a sibling traced this frame's end (`docs/SYNC.md` §5) — the
    /// frame's draw count on both sides is recorded and **the original's
    /// word is installed**, so the next frame starts on the true stream.
    /// That is a correction, and it is noted as one; the counts are the
    /// number each modelled draw site moves.
    pub fn tick(&mut self) {
        let frame = self.sim.frame;
        let word_before = self.sim.rng.seed;
        self.sim.tick();
        if let Some(&(_, theirs)) = self.frame_seeds.iter().find(|(n, _)| *n == frame) {
            let ours = draws_between(word_before, self.sim.rng.seed);
            let orig = draws_between(word_before, theirs);
            self.rng_frames.push((frame, ours, orig));
            self.notes.push(format!(
                "rng: frame {frame}: ours {} draws, the original's {} — installed {theirs:#010x}",
                ours.map_or("?".to_string(), |n| n.to_string()),
                orig.map_or("?".to_string(), |n| n.to_string()),
            ));
            self.sim.rng.seed = theirs;
        }
    }

    /// `(whom, ox)` — the log's ids for a simulation building handle, for the
    /// pre-placed buildings the start-of-game rule created. A building the
    /// simulation made itself has no logged id and reads back `None`.
    fn build_ids(&self, handle: usize) -> Option<(i64, i64)> {
        let o = self
            .builds
            .iter()
            .find(|(h, _)| *h == handle)
            .map(|(_, o)| *o)?;
        Some((i64::from(self.sim.buildings[handle].owner), o))
    }

    /// `(whom, ox)` for a simulation unit handle.
    fn unit_ids(&self, handle: usize) -> Option<(i64, i64)> {
        self.units
            .iter()
            .find(|l| l.unit == handle)
            .map(|l| (l.who, l.o))
    }

    /// What a simulation order targets, in the log's ids — `TargetOrder`'s
    /// `whom` and `ox`.
    ///
    /// An attack order is the awkward one: the original's `AttackOrder` is a
    /// `TargetOrder` and carries the target itself, while here the order is
    /// the wrapper and the target lives in `combat::State` (`docs/ORDERS.md`
    /// §13). So the attack case reads the unit's combat target, which is the
    /// same object as long as the order is the current one — and is why the
    /// comparison below only checks a target it can name on both sides.
    fn target_ids(&self, unit: usize, order: &sim::orders::Order) -> Option<(i64, i64)> {
        use sim::orders::Body;
        match order.body {
            Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                self.build_ids(b)
            }
            Body::Gather(g) => self.build_ids(g.building),
            Body::Attack(_) => match self.sim.units[unit].combat.target? {
                sim::combat::Obj::Unit(u) => self.unit_ids(u),
                sim::combat::Obj::Building(b) => self.build_ids(b),
            },
            Body::Move(_) | Body::Think => None,
        }
    }
}

/// Compares one unit's order list and path stack against the logged ones
/// (`docs/ORDERS.md` §11.1).
///
/// Both sides are walked **front first** — the order being executed is slot
/// 0 — which for the log means reversing what it wrote. The path stack needs
/// no reversing: the log writes it bottom first and `Vec<PathData>` is pushed
/// and popped at the end, so both start at the goal.
///
/// A target is only compared when both sides name one: an order kind that
/// carries no target has none, and a simulation building the start-of-game
/// rule did not create has no logged id to be compared against. That keeps
/// the check from manufacturing disagreements out of what it cannot see.
fn compare_orders(
    built: &Built,
    link: &UnitLink,
    them: &UnitDump,
    frame: i64,
) -> Vec<OrderDivergence> {
    let mut out = Vec::new();
    let unit = &built.sim.units[link.unit];
    let mut at = |slot: usize, what: OrderMismatch| {
        out.push(OrderDivergence {
            frame,
            who: link.who,
            o: link.o,
            slot,
            what,
        });
    };

    if unit.orders.len() != them.orders.len() {
        at(
            0,
            OrderMismatch::Length {
                ours: unit.orders.len(),
                theirs: them.orders.len(),
            },
        );
    }
    for (slot, (ours, theirs)) in unit
        .orders
        .iter()
        .zip(them.orders_front_first())
        .enumerate()
    {
        let kind = i64::from(ours.index());
        if kind != theirs.index {
            at(
                slot,
                OrderMismatch::Kind {
                    ours: kind,
                    theirs: theirs.index,
                },
            );
            // The kinds disagree, so the fields under them are not
            // comparable; the rest of this slot would be noise.
            continue;
        }
        let action = ours.has(sim::orders::flag::ACTION);
        if action != theirs.is_action() {
            at(
                slot,
                OrderMismatch::Action {
                    ours: action,
                    theirs: theirs.is_action(),
                },
            );
        }
        if i64::from(ours.flags) != theirs.flags {
            at(
                slot,
                OrderMismatch::Flags {
                    ours: ours.flags,
                    theirs: theirs.flags,
                },
            );
        }
        let mine = built.target_ids(link.unit, ours);
        let logged = theirs.whom.zip(theirs.ox);
        if let (Some(a), Some(b)) = (mine, logged)
            && a != b
        {
            at(
                slot,
                OrderMismatch::Target {
                    ours: mine,
                    theirs: logged,
                },
            );
        }
    }

    if unit.path.len() != them.path.len() {
        at(
            0,
            OrderMismatch::PathLength {
                ours: unit.path.len(),
                theirs: them.path.len(),
            },
        );
    }
    for (slot, (ours, theirs)) in unit.path.iter().zip(them.path.iter()).enumerate() {
        let mine = (ours.to.x, ours.to.y);
        if i64::from(mine.0) != theirs.to.0 || i64::from(mine.1) != theirs.to.1 {
            at(
                slot,
                OrderMismatch::PathTo {
                    slot,
                    ours: mine,
                    theirs: theirs.to,
                },
            );
        }
    }
    out
}

/// Compares one logged frame against the simulation as it stands.
pub fn compare(built: &Built, frame: &Frame, players: usize) -> FrameResult {
    let mut r = FrameResult {
        frame: frame.n,
        ..FrameResult::default()
    };
    // The order list and the path stack are written only at `UNITS=3`. Below
    // it every unit reads back with an empty list, and comparing would say
    // the simulation had invented every order it holds — so the whole check
    // is gated on the frame carrying an order somewhere. The blind spot that
    // leaves is a `UNITS=3` frame in which *no* unit holds an order, which
    // then goes uncompared; it corrects itself on the next frame that does.
    let orders_logged = frame.units.iter().any(|u| !u.orders.is_empty());
    for u in &frame.units {
        if !(0..players as i64).contains(&u.who) {
            continue;
        }
        // A unit the dump started with is in the link table; one trained
        // since is found by its number — `find_free` hands out the same
        // per-player `o` the original did, which is what makes a trained
        // unit comparable at all.
        let trained;
        let link = match built.units.iter().find(|l| l.who == u.who && l.o == u.o) {
            Some(l) => l,
            None => {
                let found = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(u.who as sim::Player, o));
                match found {
                    Some(unit) => {
                        trained = UnitLink {
                            who: u.who,
                            o: u.o,
                            unit,
                            kind: built.sim.units[unit].ty,
                        };
                        &trained
                    }
                    None => {
                        r.unlinked += 1;
                        r.unlinked_units.push((u.who, u.o));
                        continue;
                    }
                }
            }
        };
        let ours = built.sim.units[link.unit].pos;
        let theirs = pos_of(u.pos);
        r.compared += 1;
        if orders_logged {
            r.order_compared += 1;
            r.order_diverged
                .extend(compare_orders(built, link, u, frame.n));
        }
        if ours != theirs {
            r.diverged.push(Divergence {
                frame: frame.n,
                who: u.who,
                o: u.o,
                ours,
                theirs,
            });
        }
    }
    r.scores = frame.leaders.iter().map(|l| (l.who, l.score)).collect();
    r
}

/// Builds the simulation from the log's initial state and steps it through
/// every logged frame, comparing as it goes. `limit` caps the frames.
pub fn run(loaded: &Loaded, log: &Log, tuning: Tuning, limit: Option<usize>) -> Option<Report> {
    run_with(loaded, log, tuning, limit, None)
}

/// The diff, optionally fed the recorded order stream.
///
/// Without a stream this is exactly [`run`]: the simulation stands its
/// roster up from the initial dump and then runs on the engine's own
/// start-of-game logic. With one, each frame's commands are applied
/// immediately before the tick that produces the gamelog's next frame —
/// see [`crate::input`] for the frame convention and for why the AI's units
/// can never be driven this way.
pub fn run_with(
    loaded: &Loaded,
    log: &Log,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
) -> Option<Report> {
    run_traced(loaded, log, tuning, limit, stream, &[])
}

/// Fills `init`'s checksum trace and height grid from the first sibling
/// dump that has each, when `init` itself has none (see [`run_traced`]).
pub fn borrow_from_siblings<'a, 'b: 'a>(init: &mut Initial<'a>, siblings: &[&Initial<'b>]) {
    if init.checksums.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.checksums.is_empty())
    {
        init.checksums = s.checksums.clone();
    }
    if init.heights.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.heights.is_empty())
    {
        init.heights = s.heights.clone();
    }
    if init.herds.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.herds.is_empty())
    {
        init.herds = s.herds.clone();
    }
    // The per-frame words are only the same run's if the setup stream is:
    // a sibling's count from the trace's last word, so its own trace must
    // end where ours does (run3 is the map's `DUMP_ALL` but a different
    // start; run12 is run11's stream continued).
    if init.frame_seeds.is_empty() {
        let ours = init.checksums.last().map(|c| c.seed);
        if let Some(s) = siblings
            .iter()
            .find(|s| !s.frame_seeds.is_empty() && s.checksums.last().map(|c| c.seed) == ours)
        {
            init.frame_seeds = s.frame_seeds.clone();
        }
    }
}

/// [`run_with`], with what this dump lacks borrowed from **siblings** —
/// other dumps of the same lobby and seed, hence the same map and the same
/// setup stream: the setup path's checksum trace (run11 has it; run9 and
/// run10 were captured before it was found — `docs/ORACLE.md`, "The setup
/// path's checksum trace is the RNG state") and the terrain's height grid
/// (only a `DUMP_ALL` dump prints it; run3 is this map's). The dump's own
/// data wins; the first sibling that has each thing supplies it.
pub fn run_traced<'a, 'b: 'a>(
    loaded: &Loaded,
    log: &Log<'a>,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
    siblings: &[&Initial<'b>],
) -> Option<Report> {
    let mut init = log.initial()?;
    borrow_from_siblings(&mut init, siblings);
    let players = player_count(&init);
    let mut built = build_sim(loaded, &init, tuning);
    let mut report = Report {
        notes: std::mem::take(&mut built.notes),
        ..Report::default()
    };
    let mut stream = stream;
    let frames = log.frame_states();
    let mut last = 0i64;
    for f in frames.iter().take(limit.unwrap_or(usize::MAX)) {
        // `FRAME n` is the state at the end of frame n; step up to it.
        while last < f.n {
            // The package for recording frame `last` is processed by the
            // game frame the log then reports as `FRAME last + 1`, which is
            // the tick about to run.
            if let Some(s) = stream.as_deref_mut() {
                let did = s.apply(last as i32, &mut built);
                report.applied.merge(&did);
            }
            built.tick();
            last += 1;
        }
        report.frames.push(compare(&built, f, players));
    }
    report.notes.append(&mut built.notes);
    report.rng_frames = built.rng_frames.clone();
    report.first_divergence = (0..players as i64)
        .map(|who| {
            let first = report
                .frames
                .iter()
                .find(|f| f.diverged.iter().any(|d| d.who == who))
                .map(|f| f.frame);
            (who, first)
        })
        .collect();
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamelog::{CityDump, Guy, LeaderDump, OrderDump, UnitDump};

    fn install() -> Option<crate::Install> {
        let root = std::env::var("RON_INSTALL").ok().or_else(|| {
            let here = env!("CARGO_MANIFEST_DIR");
            Some(format!("{here}/../../game"))
        })?;
        let i = crate::Install::new(root);
        i.looks_valid().then_some(i)
    }

    /// One of the kept dumps, if this machine has it.
    ///
    /// `$RON_GAMELOG_DIR`, or the bottle's `Logs\` — the same default
    /// `tools/gamelog/` uses. The dumps are tens of megabytes and live
    /// outside the repo (`CLAUDE.md`), so a machine without them skips; the
    /// test says so rather than passing quietly, because a check that can
    /// evaporate is how a stale assertion stayed green for a day
    /// (`docs/DATALAYER.md`).
    fn dump(name: &str) -> Option<String> {
        let dir = std::env::var("RON_GAMELOG_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!(
                "{home}/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/\
                 crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"
            )
        });
        let path = format!("{dir}/{name}");
        std::path::Path::new(&path).is_file().then_some(path)
    }

    fn initial() -> Initial<'static> {
        Initial {
            world: vec![("xs", "60"), ("ys", "60"), ("seed", "7236")],
            cities: vec![CityDump {
                x: 3168,
                y: 30816,
                pop: 1,
                who: 0,
            }],
            units: vec![
                UnitDump {
                    flags: 65,
                    o: 0,
                    who: 0,
                    pos: LogPos {
                        x: 4248,
                        y: 32664,
                        z: 528,
                    },
                    guys: vec![
                        Guy {
                            kind: Some(0x32 + 19),
                            ..Guy::default()
                        },
                        Guy {
                            kind: Some(0x32 + 19),
                            ..Guy::default()
                        },
                    ],
                    orders: Vec::new(),
                    path: Vec::new(),
                },
                UnitDump {
                    flags: 1,
                    o: 1,
                    who: 0,
                    pos: LogPos {
                        x: 4248,
                        y: 28680,
                        z: 496,
                    },
                    guys: vec![Guy {
                        kind: Some(0x32),
                        ..Guy::default()
                    }],
                    orders: Vec::new(),
                    path: Vec::new(),
                },
                // An animal, which the harness ignores.
                UnitDump {
                    flags: 1,
                    o: 3,
                    who: 255,
                    pos: LogPos::default(),
                    guys: vec![],
                    orders: Vec::new(),
                    path: Vec::new(),
                },
            ],
            leaders: vec![
                LeaderDump {
                    who: 0,
                    tribe: 11,
                    ..LeaderDump::default()
                },
                LeaderDump {
                    who: 8,
                    ..LeaderDump::default()
                },
            ],
            ..Initial::default()
        }
    }

    #[test]
    fn player_count_is_the_highest_player_slot_plus_one() {
        assert_eq!(player_count(&initial()), 1);
        let mut i = initial();
        i.leaders.push(LeaderDump {
            who: 1,
            ..LeaderDump::default()
        });
        assert_eq!(player_count(&i), 2);
    }

    #[test]
    fn the_simulation_is_built_from_the_dump_and_idle_units_hold() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let init = initial();
        let built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.units.len(), 2);
        assert_eq!(built.units[0].kind, Some(19)); // the Scout
        assert_eq!(built.units[1].kind, Some(0)); // the Citizen
        assert_eq!(built.sim.units[built.units[1].unit].health, 40);
        assert_eq!(built.sim.units[built.units[0].unit].squad_size, 2);
        assert_eq!(built.sim.tech[0].tribe, 11);
        assert_eq!(built.sim.units[built.units[1].unit].movement.speed, 25);
        // Idle units stand still, so a frame that repeats the positions agrees.
        let frame = Frame {
            n: 1,
            units: init.units.clone(),
            ..Frame::default()
        };
        let mut built = built;
        built.sim.tick();
        let r = compare(&built, &frame, 1);
        assert_eq!(r.compared, 2);
        assert!(r.diverged.is_empty());
        // And a moved one is reported.
        let mut moved = frame.clone();
        moved.units[1].pos.x += 10;
        let r = compare(&built, &moved, 1);
        assert_eq!(r.diverged.len(), 1);
        assert_eq!(r.diverged[0].o, 1);
    }

    /// `docs/AI.md` §5's table, first row: the opening script's first call
    /// at game frame 1 — `defensive` steps 6, 7, 9, 10 in one call (Written
    /// Word researched, the fourth farm placed, City State researched, three
    /// citizens queued) and 11 blocking. The dump's `FRAME 2` is the state
    /// after that frame. The AI's personality and script are run8's
    /// `PERSONALITY` block (`LEADERS=9`), since the harness's stream is not
    /// the original's at `Leader::init`.
    #[test]
    fn run7_s_first_script_call_fills_the_queues_the_dump_shows() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run7-ancient-nubian-orders.txt") else {
            eprintln!("skipping: no gamelog-run7-ancient-nubian-orders.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        assert_eq!(loaded.scripts.len(), 3, "the three shipped scripts");
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        // The script's frame-1 coin is thrown on whatever stream frame 0
        // leaves: the siblings supply the true one (run11's setup trace,
        // run12's end-of-frame words), which is the boom order the original
        // took (`docs/SYNC.md` §5).
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert!(built.sim.scripts.is_some(), "the scripts compiled");
        assert!(!built.sim.nation[1].human, "player 1 is the AI");
        let ai = &mut built.sim.ai[1];
        ai.pers = sim::ai::Personality {
            rush: 1,
            cities: 1,
            upgrades: 0,
            arms: -1,
            army: -1,
            army_size: 1,
            raid: 1,
            invade: 1,
            target: 0,
            strategy: 1,
            raze: 0,
            spells: 0,
            forts: -1,
            nukes: 1,
            air: 1,
            naval: 0,
            market: -1,
            scouts: 1,
            civilians: -1,
            early_army: 1,
            friendly_human: 1,
            alliance_human: -1,
            friendly_ai: 1,
            alliance_ai: -1,
        };
        ai.script = Some("defensive".to_string());
        ai.script_live = true;
        ai.script_step = 1;

        // Frame 0 arms the machine; frame 1 is the script's first call.
        built.tick();
        built.tick();
        assert_eq!(built.sim.frame, 2);
        assert!(built.sim.ai[1].script_live, "the script is not done");
        assert_eq!(built.sim.ai[1].script_step, 11, "6 → 7 → 9 → 10 → 11");

        // The city's queue: three citizens, on the ramp the dump shows.
        let city_b = built
            .sim
            .cities
            .iter()
            .find(|c| c.alive && c.owner == 1)
            .map(|c| c.building)
            .expect("the AI's city");
        let citizen = loaded.unit_named("Citizen").expect("a Citizen record");
        let q = &built.sim.buildings[city_b].queue;
        assert_eq!(
            q.items.iter().map(|i| (i.ty, i.tech)).collect::<Vec<_>>(),
            vec![(citizen, None); 3]
        );
        assert_eq!(
            q.items
                .iter()
                .map(|i| i32::from(i.cost[0]))
                .collect::<Vec<_>>(),
            [25, 26, 27],
            "the citizens' food ramp in the dump's FRAME 2"
        );

        // The library's queue: Written Word, then City State.
        let lib = built.sim.first_library(1).expect("the AI's library");
        let tech = |n: &str| loaded.tech_tree[loaded.tech_named(n).unwrap()];
        assert_eq!(
            built.sim.buildings[lib]
                .queue
                .items
                .iter()
                .map(|i| i.tech)
                .collect::<Vec<_>>(),
            [Some(tech("Written Word")), Some(tech("City State"))]
        );

        // The fourth farm: a site placed for the AI this frame, with a
        // citizen ordered onto it. Its tile is the map's to decide
        // (`docs/AI.md` §13); that it exists is the script's.
        let farm = loaded.build_named("Farm").unwrap();
        let sites: Vec<usize> = built
            .sim
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.owner == 1 && b.alive && !b.active && b.ty == Some(farm))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(sites.len(), 1, "one farm site");
        assert_eq!(
            i32::from(built.sim.buildings[sites[0]].index),
            2006,
            "the dump's site number"
        );
        let builder = built.sim.units.iter().any(|u| {
            u.owner == 1
                && u.orders
                    .iter()
                    .any(|o| matches!(o.body, sim::orders::Body::Build(b) if b == sites[0]))
        });
        assert!(builder, "a citizen holds the build order");
    }

    /// The map, from the dump: run9 (`gamelog-run9-world6.txt`, 2026-08-24)
    /// is run7's lobby and seed logged with `WORLD=6` under `[Start Game]`,
    /// so its start block carries every cell's `WData` and every tile's
    /// mask (`docs/ORACLE.md`, "The map is a dump too"). With the cells'
    /// `val` bytes real, the AI's fourth farm (site 2006) lands on the
    /// original's tile and its builder `1/1` — the first divergence on
    /// every earlier dump — tracks the whole run.
    #[test]
    fn run9_s_world_dump_puts_the_ai_s_farm_on_the_original_s_tile() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        assert_eq!(built.sim.world.width(), 60);
        assert_eq!(built.region_map.len(), 8, "the dump's eight regions");
        assert_eq!(built.sim.world.region_count(), 8);
        assert!(
            built.notes.iter().any(|n| n.contains("57600 tile masks")),
            "{:?}",
            built.notes
        );
        // A forest tile and an ocean tile exist, so the layers are there.
        let masks = |bits: u16| {
            (0..240)
                .flat_map(|y| (0..240).map(move |x| Pos::new(x, y)))
                .filter(|&t| built.sim.world.tile_mask(t) & sim::world::tile::SURFACE == bits)
                .count()
        };
        // Counted from the dump's `tdata[scan].mask` histogram: 10,749
        // tiles with the ocean surface, ~1,800 with forest.
        assert_eq!(masks(sim::world::tile::SURFACE_OCEAN), 10749);
        assert!(masks(sim::world::tile::SURFACE_FOREST) > 1500);

        let report = run(&loaded, &log, Tuning::RON, None).unwrap();
        assert_eq!(report.frames.len(), 36);
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        assert_eq!(unlinked, 0);
        let by_unit = report.first_divergence_by_unit();
        assert!(
            !by_unit.iter().any(|&(w, o, _)| w == 1 && o == 1),
            "the farm's builder diverged in position: {by_unit:?}"
        );
        assert!(
            !report
                .order_divergence_by_unit()
                .iter()
                .any(|(w, o, _, _)| *w == 1 && *o == 1),
            "the farm's builder diverged in its orders"
        );
    }

    /// The census against the original's own leader record: run9's `FRAME 1`
    /// `LEADERDATA who 1` is the AI after its frame-0 sweep (`LEADERS=9`,
    /// `docs/ORACLE.md`), and the harness's frame 0 runs the same sweep on
    /// the same map (run9 carries the tiles). Every count the sweep writes
    /// is compared under its own name; the per-region arrays through
    /// `Built.region_map`. What is left out is named: `explored`
    /// (`check_explore`'s visibility recount is a seam), `reg_known_rares`
    /// (rares are not in the simulation), `territory`/`reg_terr` (the
    /// territory pass is `docs/ATTRITION.md`'s and compared elsewhere).
    #[test]
    fn run9_s_frame_1_leader_record_is_the_census_after_the_sweep() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        // Run9 predates the trace and never had the heights; run11 (the
        // trace) and run3 (`DUMP_ALL`, the heights) are the same map.
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut init = log.initial().unwrap();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        borrow_from_siblings(&mut init, &refs);
        // A sibling on disk that yields nothing is a reader bug, not a
        // reason to skip: the assertion below must not evaporate.
        if texts.len() == 3 {
            assert!(!init.checksums.is_empty(), "run11's trace was not read");
            assert!(!init.heights.is_empty(), "run3's height table was not read");
            assert_eq!(init.herds.len(), 13, "run12's herds were not read");
            assert_eq!(
                (init.herds[0].wx, init.herds[0].wy, init.herds[0].t),
                (22, 34, 408)
            );
            assert_eq!(
                init.frame_seeds,
                vec![
                    (0, 0xb619_4ba1),
                    (1, 0x4554_ec0f),
                    (2, 0xab3b_035d),
                    (3, 0xc242_06bb)
                ],
                "run12's end-of-frame words"
            );
        }
        let traced = !init.checksums.is_empty() && !init.heights.is_empty();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let theirs = log.leader_block(1, 1).expect("FRAME 1 LEADERDATA who 1");
        assert_eq!(theirs.int("peasants"), Some(5), "the record is level 9");

        // Frame 0: the sweep, which arms the step machine.
        built.sim.tick();
        let c = &built.sim.ai[1].census;
        let scalars: [(&str, i32); 14] = [
            ("active", c.active),
            ("combat", c.combat),
            ("siege", c.siege),
            ("non_siege", c.non_siege),
            ("defense", c.defense),
            ("attack", c.attack),
            ("naval", c.naval),
            ("peasants", c.peasants),
            ("scholars", c.scholars),
            ("caras", c.caras),
            ("merchants", c.merchants),
            ("free_peasants", c.free_peasants),
            ("gatherers", c.gatherers),
            ("full_cities", c.full_cities),
        ];
        let mut wrong = Vec::new();
        for (name, ours) in scalars {
            let t = theirs.int(name).unwrap_or(i64::MIN);
            if i64::from(ours) != t {
                wrong.push(format!("{name}: ours {ours} theirs {t}"));
            }
        }
        let arr = |key: &str| -> Vec<i64> {
            theirs
                .all(key)
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect()
        };
        for (key, ours) in [
            ("filled_gather_slots[scan]", c.filled_gather_slots),
            ("escrow_rate[scan]", c.escrow_rate),
        ] {
            let t = arr(key);
            let o: Vec<i64> = ours.iter().map(|&v| i64::from(v)).collect();
            if t.len() >= 6 && o != t[..6] {
                wrong.push(format!("{key}: ours {o:?} theirs {:?}", &t[..6]));
            }
        }
        // `gather_slots` sums each finished gather building's `gather_max`:
        // the farms (flat, 1 each) and the woodcutter camp's five, surveyed
        // from its `gather_from` list by `crates/sim/src/gather.rs`.
        let t = arr("gather_slots[scan]");
        assert_eq!(i64::from(c.gather_slots[0]), t[0], "food slots");
        assert_eq!(i64::from(c.gather_slots[1]), t[1], "wood slots");
        // The home region: the dump's region 1 is the sim's region_map entry.
        let home_dump = theirs.int("home_reg").unwrap();
        let (_, home) = built
            .region_map
            .iter()
            .find(|(d, _)| *d == home_dump)
            .copied()
            .expect("the home region is in the map");
        for (key, ours) in [
            ("reg_active", &c.reg_active),
            ("reg_peasants", &c.reg_peasants),
            ("reg_free_peasants", &c.reg_free_peasants),
            ("reg_gatherers", &c.reg_gatherers),
            ("reg_gather_slots", &c.reg_gather_slots),
            ("reg_cities", &c.reg_cities),
            ("reg_land", &c.reg_land),
            ("strategy", &c.strategy),
        ] {
            let t = arr(&format!("{key}[scan]"));
            let o = i64::from(sim::ai::Census::reg(ours, home));
            if t.get(home_dump as usize).copied() != Some(o) {
                wrong.push(format!(
                    "{key}[home]: ours {o} theirs {:?}",
                    t.get(home_dump as usize)
                ));
            }
        }
        assert!(
            wrong.is_empty(),
            "the census disagrees:\n  {}",
            wrong.join("\n  ")
        );

        // The sites (`compute_sites`, `docs/AI.md` §2.7): the record holds
        // ten `SITE`s, taken from the frame-1 record — the state after
        // frame 1, which is when the script first runs (its `city_placement`
        // would force a second pass through `place_city_with_cost`, but
        // only once City State is in, so at frame 1 both sides have the
        // frame-0 sweep's single pass). The sampler's stride is a
        // sync-stream draw; with run11's trace the harness's stream is the
        // original's, and with run3's heights the slide is the original's,
        // so the record's best site is asserted when both are on hand and
        // reported otherwise.
        built.sim.tick();
        let theirs_sites: Vec<(i64, i64, i64, i64)> = theirs
            .kids("SITE")
            .map(|s| {
                (
                    s.int("wx").unwrap_or(0),
                    s.int("wy").unwrap_or(0),
                    s.int("val").unwrap_or(0),
                    s.int("rank").unwrap_or(0),
                )
            })
            .collect();
        let ours_sites: Vec<(i32, i32, i32, i32)> = built.sim.ai[1]
            .sites
            .iter()
            .map(|s| (s.wx, s.wy, s.val, s.rank))
            .collect();
        eprintln!(
            "sites — theirs: {theirs_sites:?}\n        ours:   {ours_sites:?}\n  site_mark theirs {:?} ours {}",
            theirs.int("site_mark"),
            built.sim.ai[1].site_mark
        );
        if traced {
            let best_theirs = theirs_sites
                .iter()
                .max_by_key(|s| s.2)
                .map(|s| (s.0 as i32, s.1 as i32, s.2 as i32))
                .unwrap();
            let best_ours = ours_sites
                .iter()
                .max_by_key(|s| s.2)
                .map(|s| (s.0, s.1, s.2))
                .unwrap();
            assert_eq!(
                best_ours, best_theirs,
                "the best site (wx, wy, val), sampled on the original's stream"
            );
        }
    }

    /// Run12's per-frame words, read straight from the dump (`docs/SYNC.md`
    /// §1): the `end_frame` record inside each `FRAME n` block's `FULL
    /// DUMP`, keyed by the engine frame.
    #[test]
    fn run12_s_end_of_frame_words_are_read() {
        let Some(path) = dump("gamelog-run12-dumpall-seeds.txt") else {
            eprintln!("skipping: no gamelog-run12-dumpall-seeds.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        assert_eq!(
            log.frame_seeds(),
            vec![
                (0, 0xb619_4ba1),
                (1, 0x4554_ec0f),
                (2, 0xab3b_035d),
                (3, 0xc242_06bb)
            ]
        );
        let init = log.initial().unwrap();
        assert_eq!(init.checksums.last().map(|c| c.seed), Some(0x3bd3_9ae9));
        assert_eq!(
            draws_between(0x3bd3_9ae9, 0xb619_4ba1),
            Some(120),
            "frame 0's draws"
        );
        assert_eq!(draws_between(0xb619_4ba1, 0x4554_ec0f), Some(54));
        assert_eq!(draws_between(0x4554_ec0f, 0xab3b_035d), Some(6));
        assert_eq!(draws_between(0xab3b_035d, 0xc242_06bb), Some(6));
    }

    /// Run13's window (`docs/SYNC.md` §4.1, §5): run10 stepped with run11,
    /// run3 and **run13** as the siblings, so run13's end-of-frame words for
    /// sim-frames 94–103 are installed and the per-frame counts compared.
    /// Pinned: the frames the sim matches outright (98, 102, 103 — six
    /// farm draws and no `do_move` draw for the seven walks that start on
    /// 102 and 103), the two the animation clock owes (100: twelve fish
    /// wraps; 101: the sheep's arrival and the scout's wrap on top of the
    /// farmers' twelve and the farms' seven), and — with the unit loop
    /// rotated so the AI's units go first at frame 101 — the AI's three
    /// farmers' re-target goals, which the dump shows at frame 103.
    #[test]
    fn run13_s_window_counts_and_the_ai_farmers_re_targets_are_matched() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: no gamelog-run10-world6-long.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<String> = [
            "gamelog-run11-checksum.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run13-window-95-105.txt",
        ]
        .iter()
        .filter_map(|n| dump(n))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
        if texts.len() != 3 {
            eprintln!("skipping: run11, run3 and run13 are all needed");
            return;
        }
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        assert_eq!(
            inits[2].frame_seeds.first(),
            Some(&(94, 0x5f8f_3d9d)),
            "run13's first word is the end of sim-frame 94"
        );
        assert_eq!(inits[2].frame_seeds.len(), 10);
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, Some(105), None, &refs).unwrap();
        let count = |f: i64| -> (Option<u32>, Option<u32>) {
            let &(_, ours, theirs) = report
                .rng_frames
                .iter()
                .find(|(n, _, _)| *n == f)
                .unwrap_or_else(|| panic!("frame {f} was not traced"));
            (ours, theirs)
        };
        assert_eq!(count(98), (Some(6), Some(6)));
        assert_eq!(count(99), (Some(6), Some(8)), "the new citizen's two");
        assert_eq!(count(100), (Some(6), Some(18)), "the twelve fish wraps");
        assert_eq!(count(101), (Some(19), Some(21)), "the sheep and the scout");
        assert_eq!(count(102), (Some(6), Some(6)));
        assert_eq!(count(103), (Some(6), Some(6)));
        // The AI's farmers re-target on sim-frame 101 and walk from 102;
        // their path goals are compared on the log's frame 103 (the end of
        // sim-frame 102, the walk's first step). The human's three draw one
        // place late (the sheep's arrival is not modelled) and are not
        // pinned.
        let at_103 = report
            .frames
            .iter()
            .find(|f| f.frame == 103)
            .expect("the log's frame 103");
        for o in 3..=5 {
            let bad: Vec<_> = at_103
                .order_diverged
                .iter()
                .filter(|d| d.who == 1 && d.o == o)
                .collect();
            assert!(bad.is_empty(), "AI farmer 1/{o} at frame 103: {bad:?}");
        }
    }

    /// The sibling dumps of the run9/run10/run11 map that carry what the
    /// others lack (`run_traced`): run11 (the setup path's checksum trace),
    /// run3 (`DUMP_ALL` — the terrain heights, the regions' coordinate
    /// lists) and run12 (`DUMP_ALL` with the per-frame sync words and the
    /// herds, `docs/SYNC.md`). Whichever the machine has.
    fn sibling_texts() -> Vec<String> {
        [
            "gamelog-run11-checksum.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run12-dumpall-seeds.txt",
        ]
        .iter()
        .filter_map(|n| dump(n))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect()
    }

    /// The personality block of one leader's start-of-game `LEADERDATA`
    /// (`LEADERS=9` under `[Start Game]`), by field name.
    fn personality_of<'a>(log: &Log<'a>, who: i64) -> Vec<(&'a str, i64)> {
        log.game()
            .expect("GAME")
            .kids("LEADERDATA")
            .find(|l| l.int("who") == Some(who))
            .and_then(|l| l.kid("PERSONALITY"))
            .expect("a PERSONALITY block")
            .fields
            .iter()
            .filter_map(|(k, v)| Some((*k, v.trim().parse().ok()?)))
            // The 24 ints; what follows is the next sibling's fields at the
            // same indent, which the parser records on both candidates.
            .take(24)
            .collect()
    }

    fn personality_fields(p: &sim::ai::Personality) -> Vec<(&'static str, i64)> {
        vec![
            ("rush", p.rush.into()),
            ("cities", p.cities.into()),
            ("upgrades", p.upgrades.into()),
            ("arms", p.arms.into()),
            ("army", p.army.into()),
            ("army_size", p.army_size.into()),
            ("raid", p.raid.into()),
            ("invade", p.invade.into()),
            ("target", p.target.into()),
            ("strategy", p.strategy.into()),
            ("raze", p.raze.into()),
            ("spells", p.spells.into()),
            ("forts", p.forts.into()),
            ("nukes", p.nukes.into()),
            ("air", p.air.into()),
            ("naval", p.naval.into()),
            ("market", p.market.into()),
            ("scouts", p.scouts.into()),
            ("civilians", p.civilians.into()),
            ("early_army", p.early_army.into()),
            ("friendly_human", p.friendly_human.into()),
            ("alliance_human", p.alliance_human.into()),
            ("friendly_ai", p.friendly_ai.into()),
            ("alliance_ai", p.alliance_ai.into()),
        ]
    }

    /// Run11 (`gamelog-run11-checksum.txt`, 2026-08-24, the run9/run10
    /// lobby with `check_all_level=14` and `[Misc Logging] CHECKSUM=2`): the
    /// setup path's own sync trace — 146 `say_checksum` records from
    /// `init_rules_and_teams` to the end of `Game::init`, each with the
    /// sync stream's state (`docs/ORACLE.md`, "The setup path's checksum
    /// trace is the RNG state"). Three pins. The trace starts at the lobby
    /// seed. The AI's `Leader::init` bracket is exactly the personality
    /// roll: `Personality::roll` from its near end reproduces the record's
    /// own `PERSONALITY` block field for field **and** lands on the far
    /// end — twenty draws, the original's count. And the last record is the
    /// state entering frame 0, which `build_sim` now installs.
    #[test]
    fn run11_s_checksum_trace_pins_the_setup_stream() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run11-checksum.txt") else {
            eprintln!("skipping: no gamelog-run11-checksum.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let c = log.checksums();
        assert_eq!(c.len(), 146, "every accepted call site printed its seed");
        assert_eq!(
            (c[0].file, c[0].line, c[0].seed),
            ("game.cpp", 6136, 12345),
            "the trace opens at the lobby seed"
        );
        let brackets = personality_brackets(&c);
        assert_eq!(
            brackets,
            vec![(0x9991b076, 0xf2299eda)],
            "one computer leader, one roll"
        );
        // Twenty draws between the two checkpoints, counted on the LCG.
        let mut r = sim::combat::Rng::new(0x9991b076);
        let mut n = 0;
        while r.seed != 0xf2299eda && n < 1000 {
            r.roll();
            n += 1;
        }
        assert_eq!(
            n, 20,
            "the original's personality roll draws twenty times here"
        );
        assert_eq!(
            c.last().map(|l| (l.file, l.line, l.seed)),
            Some(("game.cpp", 5024, 0x3bd39ae9)),
            "the state entering frame 0"
        );

        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        assert!(
            built
                .notes
                .iter()
                .any(|n| n.contains("landing on 0xf2299eda as the original did")),
            "the roll's draw count matches the trace: {:?}",
            built.notes
        );
        assert_eq!(
            personality_fields(&built.sim.ai[1].pers),
            personality_of(&log, 1),
            "the personality, rolled from the original's own state, is the original's"
        );
        assert_eq!(built.sim.rng.seed, 0x3bd39ae9, "seeded for frame 0");
    }

    /// Income against the original's own ledger: run8's `FRAME 2`
    /// `LEADERDATA who 1` (`LEADERS=9`) prints the encrypted goods block —
    /// `resources` is the assembled rate, in sixteenths — and it reads
    /// `[160, 160, 0, 0, 0, 0]`: `CITY_GATHER × 16` for food and timber and
    /// nothing from the five citizens, all on their chains and none yet
    /// arrived (`Unit::do_gather` sets the dirty flag on the same statement
    /// as `been_there`). By frame 60 it is `[640, 320, …]` — three farmers
    /// and one camp citizen arrived. The rate, not the cap: the AI is
    /// British and its cap carries `BRITISH_COMMERCE`.
    #[test]
    fn run8_s_frame_2_income_is_the_city_and_nothing_the_citizens_have_reached() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run8-personality.txt") else {
            eprintln!("skipping: no gamelog-run8-personality.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        for _ in 0..3 {
            built.sim.tick();
        }
        let theirs = log.leader_block(2, 1).expect("FRAME 2 LEADERDATA who 1");
        let rate: Vec<i64> = theirs
            .all("resources")
            .iter()
            .map(|v| v.trim().parse().unwrap_or(0))
            .collect();
        assert_eq!(&rate[..6], &[160, 160, 0, 0, 0, 0], "the dump's own rate");
        let ours: Vec<i64> = built.sim.ledgers[1]
            .rate
            .iter()
            .map(|&v| i64::from(v))
            .collect();
        assert_eq!(
            ours,
            rate[..6].to_vec(),
            "the AI's five citizens are on their chains and none has arrived: \
             CITY_GATHER x 16 and nothing else"
        );
    }

    /// The long run with the map: run10 (`gamelog-run10-world6-long.txt`,
    /// 1,772 frames, run7's lobby and length, no input, `WORLD=6` at start),
    /// on the original's own sync stream (run11's trace) with the terrain
    /// heights (run3). The three citizens of frame 1 train on the original's
    /// frames (100, 206, 320) and every unit tracks as before; what no
    /// longer matches is the **script's branch**: on the sim's own stream
    /// its eight `rand_int(1, 10)` at frame 1 happened to pick the boom
    /// order (and `1/9` trained at 1297, the original's frame); on the
    /// original's stream, displaced by the ~130 per-frame draws the sim does
    /// not model (units, herds, farms, ammo — `docs/AI.md` §12.1), they pick
    /// the rush order, two farms go down at step 3 and the city is never
    /// placed. So `1/9` (1297) and `1/10` (1505) are the ceiling, 744
    /// unit-frames, and the per-frame draws are what move it. The earlier
    /// 268 was on a stream that was not the original's.
    #[test]
    fn run10_s_opening_trains_the_original_s_citizens_on_its_frames() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: no gamelog-run10-world6-long.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        // The sync stream from run11's trace and the heights from run3 —
        // the same map, seed and lobby.
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs).unwrap();
        assert_eq!(report.frames.len(), 1772);
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: seeded 0x3bd39ae9")),
            "the stream is the original's at frame 0: {:?}",
            report.notes
        );
        // Every unit the original has before frame 1297 exists here too.
        let early_unlinked: usize = report
            .frames
            .iter()
            .filter(|f| f.frame < 1297)
            .map(|f| f.unlinked)
            .sum();
        assert_eq!(
            early_unlinked, 0,
            "a unit the original trained before 1/9 that the simulation did not"
        );
        let mut missing: Vec<(i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.unlinked_units.iter().copied())
            .collect();
        missing.sort_unstable();
        missing.dedup();
        // 2026-08-24, on the true stream through frame 3 (run12's words)
        // and with the market, the farms, the birds and the herds drawing:
        // 1/9 now trains on the original's frame, and only the last citizen
        // is missing.
        assert_eq!(missing, vec![(1, 10)], "the last food-bound citizen, 1/10");
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        assert!(
            unlinked <= 268,
            "unlinked unit-frames: {unlinked} — 2026-08-24 was 268: 1/10 from 1505"
        );
    }

    /// The slot count against the original's own survey: run9's frame-1
    /// `BUILDDATA` gives each camp its `gather_from` list — 82 tiles for
    /// player 0's, 61 for player 1's — and the `LEADERS=9` record gives the
    /// counts those lists produced, `gather_slots[1]` of 7 and 5
    /// (`crates/sim/src/gather.rs`).
    #[test]
    fn run9_s_camps_get_the_original_s_gatherer_counts() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        let sim = &built.sim;
        let mut got: Vec<(sim::Player, usize, i32)> = Vec::new();
        for b in 0..sim.buildings.len() {
            if sim.building_ident(b) == sim::build::Ident::Woodcutter {
                got.push((
                    sim.buildings[b].owner,
                    sim.buildings[b].gather_from.len(),
                    sim.max_gatherers(b),
                ));
            }
        }
        got.sort_unstable();
        assert_eq!(got, vec![(0, 82, 7), (1, 61, 5)], "the camps' slot counts");
    }

    /// The oracle, as a regression guard.
    ///
    /// Every other test in this crate checks the harness against something we
    /// wrote. This one checks it against **the original's own 432 frames**,
    /// and pins the state of the port on 2026-08-21 so that a change which
    /// quietly un-does it fails here rather than in six weeks' reading of a
    /// number nobody remembers.
    ///
    /// The assertions are chosen to be the *invariants*, not the readings: a
    /// field the simulation models must never disagree where it is
    /// comparable, and the one unit the harness can drive end to end must
    /// keep doing so. Counts that are expected to move as mechanics land —
    /// `length`, `kind`, the path-stack pair — are bounded rather than fixed,
    /// so landing the pathfinder does not fail this test, it improves it.
    #[test]
    fn the_original_s_own_run_is_still_matched_frame_for_frame() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run6-ancient-nubian-builds7.txt") else {
            // Not a silent skip: say which file is missing.
            eprintln!(
                "skipping: no gamelog-run6-ancient-nubian-builds7.txt \
                 (set RON_GAMELOG_DIR; docs/ORACLE.md says how to capture one)"
            );
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);

        // The lobby, read from `GAMEINFO`: `docs/INPUT.md` §2's settings,
        // and MAP_STYLE 14 named through rules.xml's `mapstyles` order.
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        assert_eq!(built.sim.lobby.difficulty, 0, "Easiest");
        assert_eq!(built.sim.lobby.starting_town, 2, "Small Town");
        assert_eq!(built.sim.lobby.starting_resources, 1);
        assert_eq!(built.sim.lobby.map_style, 14);
        assert_eq!(built.sim.lobby.map_style_name, "Great Lakes");
        assert_eq!(built.sim.lobby.rush_rules, 0);
        assert_eq!(built.sim.lobby.victory, 0);
        assert!(!built.sim.lobby.no_nation_powers);

        // Run6 is the run7/run9/run10 lobby and seed: the siblings' setup
        // trace and end-of-frame words put the AI's frame-1 coin on the
        // original's stream (`docs/SYNC.md` §5); without them the branch is
        // the sim's own stream's luck.
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs).unwrap();

        assert_eq!(report.frames.len(), 432, "the dump's frame count");
        assert!(report.orders_seen(), "run6 is a UNITS=3 dump");
        // Every unit the game *started* with links. Later frames unlink a
        // growing number, and that is not a fault: over 432 frames both
        // players train units, and the harness stands its roster up from the
        // initial dump and has no production. It is worth pinning as a
        // ceiling, because the day production is wired in it should fall.
        assert_eq!(
            report.frames[0].unlinked, 0,
            "a unit in the first logged frame has no simulation unit"
        );
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        // 2026-08-24: zero. The AI trains the three citizens the original
        // does, on the same frames, and `find_free` numbers them so they
        // link. This is now exact, not a ceiling.
        assert_eq!(
            unlinked, 0,
            "unlinked unit-frames: {unlinked} — a unit the original has that \
             the simulation never trained"
        );

        // The start-of-game rule, derived without reading the log.
        let checks = check_start_orders(
            &build_sim(&loaded, &log.initial().unwrap(), Tuning::RON),
            &log,
        );
        let held: Vec<_> = checks
            .iter()
            .filter(|c| c.ours.is_some() || c.theirs.is_some())
            .collect();
        assert_eq!(held.len(), 10, "ten starting citizens");
        assert!(held.iter().all(|c| c.agrees()), "{held:?}");

        // **The invariant**: every field the simulation actually models
        // agrees wherever both sides name it. A regression in the order
        // system shows up here first, and in nothing else.
        //
        // With one named exception, 2026-08-24: the farmers' re-target
        // (`docs/ORDERS.md` §6.5) picks its tile with two sync-stream
        // draws, and the stream is the original's only for the four frames
        // run12 traced — so the *second* re-target, some hundred frames
        // after the first one's walk, lands a frame or two apart on the two
        // sides (the first, on the log's frame 102, agrees). Those are the
        // three farmers per player, `o` 3–5, after frame 200, and they go
        // away with a longer trace (`docs/SYNC.md` §6). The second cycle
        // begins once the re-sown cell ripens, a hundred-odd frames after
        // the walk — anything past 150 is it; with the unit loop rotated
        // (`docs/SYNC.md` §3.2) the sim's own-stream luck once put a
        // farmer's flags mismatch on frame 200 exactly.
        let modelled: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| {
                matches!(
                    d.what,
                    OrderMismatch::Action { .. } | OrderMismatch::Target { .. }
                ) || matches!(d.what, OrderMismatch::Flags { .. })
            })
            .collect();
        let (farmers_late, rest): (Vec<&OrderDivergence>, Vec<&OrderDivergence>) = modelled
            .iter()
            .partition(|d| d.frame > 150 && (3..=5).contains(&d.o));
        assert!(
            rest.is_empty(),
            "a modelled order field disagrees: {:?}",
            &rest[..rest.len().min(4)]
        );
        assert!(
            !farmers_late.is_empty(),
            "the farmers' second re-target now happens on both sides"
        );

        // **The unit that tracks**: player 0's first woodcutter citizen
        // matched the original's position and its whole order list for the
        // entire run once `gather_from` arrived. If that stops being true,
        // something took the harness backwards.
        let by_unit = report.first_divergence_by_unit();
        assert!(
            !by_unit.iter().any(|&(w, o, _)| w == 0 && o == 1),
            "0/1 diverged in position: {by_unit:?}"
        );
        assert!(
            !report
                .order_divergence_by_unit()
                .iter()
                .any(|&(w, o, _, _)| w == 0 && o == 1),
            "0/1 diverged in its order list"
        );

        // **The pathfinder's pin** (2026-08-23): the second woodcutter's
        // citizen, whose stack was the stub's visible gap, now agrees with
        // the original's path stack for 427 straight frames — its first
        // disagreement of any kind is an order-list Length at frame 428,
        // and only after that fork do its stacks differ. The mismatches
        // that remain are player 1's mirror units, whose straight lines
        // cross forest the harness's flat world does not carry
        // (`docs/PATHFINDER.md` §10) — a world-data gap, not a search gap.
        assert!(
            !report
                .frames
                .iter()
                .filter(|f| f.frame < 428)
                .flat_map(|f| f.order_diverged.iter())
                .any(|d| d.who == 0 && d.o == 2),
            "0/2 disagreed before frame 428"
        );

        // The rest is expected to shrink, never grow. These are ceilings.
        // Re-based 2026-08-24 from 1,279/783: the AI's three trained
        // citizens now link and are compared — unit-frames the harness had
        // never seen before — and they idle where
        // the original sends them to gather, which is the census's job
        // (`docs/AI.md` §12.1).
        let farmer = |d: &&OrderDivergence| (3..=5).contains(&d.o);
        let orders: usize = report.frames.iter().map(|f| f.order_only().count()).sum();
        let paths: usize = report.frames.iter().map(|f| f.path_only().count()).sum();
        let farmer_orders: usize = report
            .frames
            .iter()
            .map(|f| f.order_only().filter(farmer).count())
            .sum();
        let farmer_paths: usize = report
            .frames
            .iter()
            .map(|f| f.path_only().filter(farmer).count())
            .sum();
        // Re-based again 2026-08-24, split: the farmers now re-target
        // (`docs/ORDERS.md` §6.5), and on the sim's own stream past run12's
        // four traced frames their tiles, walks and second re-targets are
        // their own — 527 order and 357 path disagreements that a longer
        // trace removes (`docs/SYNC.md` §6). Everyone else's fell, from
        // 1,784/1,123 to 1,238/875.
        assert!(
            orders - farmer_orders <= 1_238 && paths - farmer_paths <= 875,
            "disagreements grew: orders {orders} ({farmer_orders} farmers'), paths {paths} ({farmer_paths} farmers')"
        );
        // Re-based a third time, 2026-08-24 (run13): the re-target's modulus
        // is 4, not 3 — sixteen cells to be sent to, not nine — and the
        // unit loop now rotates by owner, so the AI's farmers draw first at
        // frame 101. On run6's own stream past frame 3 both change which
        // tiles the six farmers are sent to, and the counts moved from
        // 527/357 to 662/432. The farmers' pin with teeth is run13's, where
        // the frame-101 words are the original's
        // (`run13_s_window_counts_and_the_ai_farmers_re_targets_are_matched`).
        assert!(
            farmer_orders <= 662 && farmer_paths <= 432,
            "the farmers' disagreements grew: orders {farmer_orders}, paths {farmer_paths}"
        );
    }

    /// One of the kept recordings, if this machine has it.
    ///
    /// `$RON_RECGAME_DIR`, or the profile's own `Recorded Games` directory —
    /// `PlayerProfile::get_record_game_directory` builds it under
    /// `CSIDL_PERSONAL`, which CrossOver maps to the Mac's `~/Documents`.
    fn recording(name: &str) -> Option<String> {
        let dir = std::env::var("RON_RECGAME_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/Documents/My Games/Rise of Nations/Recorded Games")
        });
        let path = format!("{dir}/{name}");
        std::path::Path::new(&path).is_file().then_some(path)
    }

    /// **The paired run** (2026-08-24): one game described by both ground
    /// truths at once — the gamelog's per-frame state and the recording's
    /// command stream — which is what `docs/RECGAME.md` §5 left open and
    /// what `docs/DATALAYER.md` called the diff's missing input.
    ///
    /// This is written to fail if the wiring breaks in either direction: if
    /// the pairing is wrong the frame counts stop matching, and if the
    /// stream stops reaching the simulation the scout goes back to holding
    /// no order at all on the frame the original moved it.
    #[test]
    fn the_recorded_order_stream_drives_the_units_it_names() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run7-ancient-nubian-orders.txt") else {
            eprintln!(
                "skipping: no gamelog-run7-ancient-nubian-orders.txt \
                 (set RON_GAMELOG_DIR; docs/ORACLE.md says how to capture one)"
            );
            return;
        };
        let Some(rc) = recording("Playback - 2026.08.24 10'15'53 (Mon).rcx") else {
            eprintln!(
                "skipping: no run7 recording (set RON_RECGAME_DIR; \
                 docs/RECGAME.md says where the game writes them)"
            );
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let data = crate::recgame::decompress(&rc).unwrap();
        let rec = crate::recgame::parse(&data, &rc).unwrap();

        // The pairing itself. A recording holds one package a frame, so a
        // recording and a dump of the *same* run have equal counts; this is
        // the cheapest possible guard against diffing two different games.
        assert_eq!(
            rec.packages.len(),
            log.frame_states().len(),
            "the recording and the dump are not the same run"
        );
        assert_eq!(rec.packages.len(), 1732, "run7's frame count");
        assert_eq!(rec.seed, 12345, "run7's fixed seed");

        // The input, and what of it the harness can act on today. Both are
        // ceilings in opposite directions: the stream never carries fewer
        // commands, and the number it cannot act on only ever falls as
        // mechanics land.
        let mut stream = crate::input::Stream::new(&rec);
        assert_eq!(stream.len(), 46, "run7's input commands");
        let report = run_with(&loaded, &log, Tuning::RON, None, Some(&mut stream)).unwrap();
        assert_eq!(
            report.applied.orders, 9,
            "the nine move orders the stream names"
        );
        assert!(
            report.applied.skipped_total() <= 22,
            "commands the harness ignores grew to {}: this only ever shrinks",
            report.applied.skipped_total()
        );

        // **The check with teeth.** Unit 0/0 is the scout, and the original
        // moves it on frame 477. Un-fed, the harness has no order there at
        // all and the disagreement is `Length { ours: 0, theirs: 1 }`; fed,
        // the order exists and matches in kind, so whatever remains is
        // path-level. If the stream stops arriving, this reverts.
        let scout: Vec<_> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| d.who == 0 && d.o == 0)
            .collect();
        assert!(
            !scout
                .iter()
                .any(|d| matches!(d.what, OrderMismatch::Length { ours: 0, .. })),
            "the scout holds no order where the original moved it: the \
             stream is not reaching the simulation"
        );
    }

    /// A unit holding a build order on the building the log calls `2001`,
    /// with a transit move in front of it — the shape §11.1's worked block
    /// has, and the one that catches a list read the wrong way round.
    fn ordered_citizen(loaded: &crate::load::Loaded) -> (Built, usize, Vec<OrderDump>) {
        let init = initial();
        let mut built = build_sim(loaded, &init, Tuning::RON);
        let u = built.units[1].unit;
        let b = built.sim.add_building(0, Pos::new(4248, 28680), 8);
        built.builds.push((b, 2001));
        built
            .sim
            .add_build_order(u, b, sim::orders::QueuePos::New, true);
        built.sim.add_move_order(
            u,
            Pos::new(4300, 28700),
            sim::orders::MoveKind::ExploreTo,
            sim::orders::QueuePos::First,
            false,
        );
        // Newest first, as `OrderList::log_data` writes it: the build order
        // was given first and prints first, the transit move it is walking
        // now prints last.
        let logged = vec![
            OrderDump {
                index: i64::from(sim::orders::index::BUILD_AT),
                kind: "BUILDORDER".into(),
                flags: 4,
                ox: Some(2001),
                whom: Some(0),
                ..OrderDump::default()
            },
            OrderDump {
                index: i64::from(sim::orders::index::EXPLORE_TO),
                kind: "EXPLORETOORDER".into(),
                flags: built.sim.units[u].orders[0].flags.into(),
                ..OrderDump::default()
            },
        ];
        (built, u, logged)
    }

    #[test]
    fn two_order_lists_that_agree_are_walked_front_first_and_report_nothing() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (built, _, logged) = ordered_citizen(&loaded);
        let them = UnitDump {
            who: 0,
            o: 1,
            orders: logged,
            ..UnitDump::default()
        };
        let d = compare_orders(&built, &built.units[1].clone(), &them, 1);
        assert!(d.is_empty(), "{d:?}");
        // Read the other way round it is two kind disagreements — which is
        // the whole risk in a list the log writes newest first.
        let mut backwards = them.clone();
        backwards.orders.reverse();
        let d = compare_orders(&built, &built.units[1].clone(), &backwards, 1);
        assert_eq!(d.len(), 2);
        assert!(
            d.iter()
                .all(|d| matches!(d.what, OrderMismatch::Kind { .. }))
        );
    }

    #[test]
    fn each_field_of_an_order_is_reported_on_its_own() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (built, _, logged) = ordered_citizen(&loaded);
        let link = built.units[1];
        let one = |orders: Vec<OrderDump>| {
            compare_orders(
                &built,
                &link,
                &UnitDump {
                    who: 0,
                    o: 1,
                    orders,
                    ..UnitDump::default()
                },
                1,
            )
        };

        // A shorter list: the length, and then the slots that do line up.
        let short = one(logged[..1].to_vec());
        assert!(matches!(
            short[0].what,
            OrderMismatch::Length { ours: 2, theirs: 1 }
        ));

        // A different target on the build order — slot 1, front first.
        let mut other = logged.clone();
        other[0].ox = Some(2002);
        let d = one(other);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].slot, 1);
        assert!(matches!(
            d[0].what,
            OrderMismatch::Target {
                ours: Some((0, 2001)),
                theirs: Some((0, 2002))
            }
        ));

        // The action bit alone, which is what says "intent" rather than
        // "transit leg" — and it takes the flags byte with it.
        let mut unset = logged.clone();
        unset[0].flags = 0;
        let d = one(unset);
        assert_eq!(d.len(), 2);
        assert!(matches!(
            d[0].what,
            OrderMismatch::Action {
                ours: true,
                theirs: false
            }
        ));
        assert!(matches!(d[1].what, OrderMismatch::Flags { .. }));
        assert!(!d[1].what.scores(), "a flags byte does not score");
    }

    #[test]
    fn the_path_stack_is_compared_bottom_first() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let (mut built, u, logged) = ordered_citizen(&loaded);
        built.sim.units[u].path = vec![
            sim::orders::PathData {
                to: Pos::new(45024, 19680),
                tolerance: 0,
                flags: 1,
            },
            sim::orders::PathData {
                to: Pos::new(45048, 18168),
                tolerance: 384,
                flags: 0,
            },
        ];
        let link = built.units[1];
        let them = |path: Vec<crate::gamelog::PathDump>| UnitDump {
            who: 0,
            o: 1,
            orders: logged.clone(),
            path,
            ..UnitDump::default()
        };
        // The log writes the goal first and so does the `Vec`, so equal
        // stacks agree without either being reversed.
        let same = vec![
            crate::gamelog::PathDump {
                to: (45024, 19680),
                tolerance: 0,
                flags: 1,
            },
            crate::gamelog::PathDump {
                to: (45048, 18168),
                tolerance: 384,
                flags: 0,
            },
        ];
        assert!(compare_orders(&built, &link, &them(same.clone()), 1).is_empty());
        // The stub's shape: we keep only the goal, the original had waypoints.
        let d = compare_orders(&built, &link, &them(same[..1].to_vec()), 1);
        assert_eq!(d.len(), 1);
        assert!(d[0].what.is_path());
        assert!(matches!(
            d[0].what,
            OrderMismatch::PathLength { ours: 2, theirs: 1 }
        ));
    }
}

/// One starting citizen's derived order against the one the original issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderCheck {
    pub who: i64,
    pub o: i64,
    /// The building object number the simulation's derivation chose.
    pub ours: Option<i64>,
    /// The `ox` of the logged `GATHERORDER`, if the unit is holding one.
    pub theirs: Option<i64>,
    /// The logged current order's `OrderIndex` — `-1` for an empty list.
    pub their_kind: i64,
}

impl OrderCheck {
    pub const fn agrees(&self) -> bool {
        match (self.ours, self.theirs) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }
}

/// **Derive, then read, then compare** — the check the position diff cannot
/// make.
///
/// [`build_sim`] derives every starting citizen's gather order from
/// `docs/ORDERS.md` §9.3's rule *without looking at the log*. This reads what
/// the original actually issued, out of the first logged frame's `UNITS=3`
/// order blocks, and lines the two up by `(who, o)`.
///
/// It is the honest test of that rule, because **positions cannot show it**: a
/// farm's citizen is placed inside the footprint and has already arrived, so
/// it stands still in both simulations for the whole of a short dump, and a
/// woodcutter's cannot walk at all until `gather_from` arrives (which needs
/// `BUILDS=7`, not the `BUILDS=6` the first reading claimed). Two simulations
/// can agree on every position for 47 frames and still have given every
/// citizen the wrong job.
pub fn check_start_orders(built: &Built, log: &Log<'_>) -> Vec<OrderCheck> {
    let states = log.frame_states();
    let Some(first) = states.first() else {
        return Vec::new();
    };
    let logged = &first.units;
    let o_of = |handle: usize| -> Option<i64> {
        built
            .builds
            .iter()
            .find(|(h, _)| *h == handle)
            .map(|(_, o)| *o)
    };
    built
        .units
        .iter()
        .filter_map(|link| {
            let them = logged.iter().find(|u| u.who == link.who && u.o == link.o)?;
            let ours = built.sim.units[link.unit]
                .orders
                .iter()
                .find_map(|o| match o.body {
                    sim::orders::Body::Gather(g) => Some(g.building),
                    _ => None,
                })
                .and_then(o_of);
            let cur = them.current_order();
            Some(OrderCheck {
                who: link.who,
                o: link.o,
                ours,
                theirs: cur.and_then(|c| {
                    (c.index == i64::from(sim::orders::index::GATHER))
                        .then_some(c.ox)
                        .flatten()
                }),
                their_kind: cur.map_or(-1, |c| c.index),
            })
        })
        .collect()
}

/// A freshly built simulation at frame 0, for [`check_start_orders`] — the
/// same construction [`run`] does, before any frame is stepped.
pub fn build_for_check(loaded: &Loaded, log: &Log<'_>, tuning: Tuning) -> Option<Built> {
    let init = log.initial()?;
    Some(build_sim(loaded, &init, tuning))
}
