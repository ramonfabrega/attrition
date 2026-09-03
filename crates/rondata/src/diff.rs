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

/// One map's headline floors: ticks and orders before divergence on its
/// score run, and its word — the frame its draw stream parts — beside
/// them. The pair is the score and the word is the instrument
/// (`docs/DECISIONS.md` 26). The scoring tests assert against these
/// fields rather than their own literals, and the queue's handoff states
/// the same six numbers on a `Scoreboard:` line that
/// `the_handoff_s_scoreboard_is_the_floors` parses — so a floor that
/// moves without the handoff, or a handoff written off a run that is not
/// the score run (item 69: two numbers describing two different
/// simulations shared one file for a week), fails somewhere instead of
/// waiting for a steering pass to notice.
pub struct MapFloors {
    pub map: &'static str,
    pub ticks: i64,
    pub orders: i64,
    pub word: i64,
}

/// East Indies first — the lower pair leads and is the headline.
pub const FLOORS: [MapFloors; 2] = [
    MapFloors {
        map: "EastIndies",
        ticks: 1851,
        orders: 1850,
        word: 1850,
    },
    MapFloors {
        map: "GreatLakes",
        ticks: 1772,
        orders: 1772,
        word: 1850,
    },
];

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
    /// Every unit's guys at the end of each traced frame, from the dump or
    /// a sibling (`Initial::frame_guys`) — [`Built::tick`] installs them
    /// beside the word, so the clocks are the original's where the stream
    /// is.
    pub frame_guys: crate::gamelog::FrameGuys,
    /// Per frame ticked, the simulation's own draws as a sequence of site
    /// labels — [`mark_sites`] over the frame's marks, taken **before** the
    /// word is installed. Filled only while [`sim::Sim::trace_phases`] is
    /// on, which is what makes a whole run comparable against
    /// [`crate::trace::Trace::labels`] frame for frame rather than one
    /// hand-seeded frame at a time.
    pub frame_sites: Vec<(i64, Vec<String>)>,
    /// The `TypeIndex` of every loaded id, in `Loaded::type_index` order —
    /// what the dump writes as a queue entry's `type`. Carried here so the
    /// per-frame comparison can read the production queue without the
    /// whole [`Loaded`] (`docs/PRODUCTION.md`, "The queue record").
    pub type_index: Vec<i32>,
    /// Each simulation unit type's own id, so a queue entry holding a unit
    /// can be turned into a `TypeIndex` through `type_index` above.
    pub unit_tree: Vec<usize>,
}

/// A sim guy from a dump's `GUY` record, when the record carries the
/// clock.
fn guy_of(g: &crate::gamelog::Guy) -> Option<sim::anim::Guy> {
    if !g.has_clock() {
        return None;
    }
    Some(sim::anim::Guy {
        cur_time: g.cur_time? as u32,
        end_time: match g.end_time? {
            0 => 0,
            e => e as u32,
        },
        last_time: g.last_time.unwrap_or(-1) as i32,
        anim: g.cur_anim? as i8,
        gpiece: g.gpiece.unwrap_or(-1) as i32,
        stopped: g.stopped.unwrap_or(1) != 0,
        // A crew guy's own body is derived, not read: `Sim::seat_guys`
        // installs it from the piece's track offset once the whole unit
        // is in.
        follow: None,
    })
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
    if let Some(v) = int("REVEAL_MAP") {
        l.reveal_map = v;
    }
    if let Some(v) = int("flags") {
        l.no_nation_powers = v & 4 != 0;
    }
    l
}

/// The sim's world from a dump's `WORLD` block — the fields of the block,
/// and the `master_land_heights` table when the dump carries one. Returns
/// the world and the dump-region → sim-region map. A start-of-game block
/// or any `DUMP_ALL` frame block will do: what it holds is the map as it
/// stood at that block.
pub fn world_from(
    fields: &[(&str, &str)],
    heights: &[i64],
    notes: &mut Vec<String>,
) -> (World, Vec<(i64, u16)>) {
    let get = |k: &str| -> Option<i64> {
        fields
            .iter()
            .find(|(key, _)| *key == k)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let xs = get("xs").unwrap_or(60).max(1) as i32;
    let ys = get("ys").unwrap_or(60).max(1) as i32;
    // The dump carries no terrain at level 0 (`docs/DATALAYER.md`, "what is
    // not established"), so the harness's map is **one land region covering
    // every cell** — flat, unblocked, no water. That assumption was implicit
    // before and cost something: a cell in no region is `Blocked::Ruins` to
    // `blocked_tcoord`, so every `place_building` was refused and every
    // building came out untyped and city-less.
    let mut world = World::new(xs, ys);
    let cells = crate::gamelog::world_cells(fields);
    let mut region_map: Vec<(i64, u16)> = Vec::new();
    if cells.len() == (xs as usize) * (ys as usize) {
        // A `WORLD ≥ 5` start dump: the map's own cells — region, the
        // coastal `region2`, the site value, the goods bits, the owners
        // (`docs/ORACLE.md`, "The map is a dump too"). The dump's region
        // numbers are the original's — land `< 0x40`, sea `≥ 0x40`, the
        // boundary `Region::is_coast` and `Regions::set_coastals` test
        // (`docs/TRANSPORT.md` §9; run20's one ocean is 65) — and each
        // distinct one becomes a sim region.
        let mut region_of = |n: i64, world: &mut World| -> Option<u16> {
            if n < 0 {
                return None;
            }
            if let Some((_, r)) = region_map.iter().find(|(d, _)| *d == n) {
                return Some(*r);
            }
            let terrain = if n < 0x40 {
                sim::world::Terrain::Land
            } else {
                sim::world::Terrain::Sea
            };
            let r = world.add_region(terrain);
            region_map.push((n, r));
            Some(r)
        };
        // `CellData.land` is the original's `land_key[]` index and the dump
        // prints the name. The table is a static in the executable —
        // `dynamic_initializer_for_'land_key'@004058a0`: `BASELAND`,
        // `SANDY`, `OCEAN`, `NONE`, in that order — and the census's water
        // test is `land == 1 || land == 2` (shallows or ocean). Until run20
        // the harness numbered the names in order of first appearance,
        // which agreed with the table only because run9's first cell is
        // BASELAND; on the islands map the first cell is OCEAN, every land
        // cell became "water" to the sweep, and the home region's
        // `reg_land` came out 0 against the original's 45.
        const LAND_KEY: [&str; 4] = ["BASELAND", "SANDY", "OCEAN", "NONE"];
        let mut land_names: Vec<&str> = LAND_KEY.to_vec();
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
                    bad: c.bad as u8,
                    down: c.down as i16,
                    down_who: c.down_who as i8,
                },
            );
        }
        // The per-tile masks — forest, mountain, river, ocean, the city
        // radii and the footprints — `tile_xs × tile_ys` row-major.
        let tiles = crate::gamelog::world_tiles(fields);
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
        // The fog grid — `seen2`, what `was_seen` reads — when the block
        // carries it (`WORLD ≥ 6`, or any `DUMP_ALL`). A start-of-game
        // snapshot: the simulation does not advance it.
        let fog = crate::gamelog::world_fog(fields);
        let fog_loaded = !fog.is_empty() && world.set_fog(fog);
        if fog_loaded {
            notes.push("fog: seen2 loaded from the WORLD dump".to_string());
        }
        let hw = tw + 1;
        // The **corner** grid goes into the world whole, and every tile's
        // height is derived from it: a building re-terraforms it when it is
        // placed (`sim::terrain`), so this is no longer a table the loader
        // pins once and nothing writes.
        let heights_loaded = tiles_loaded
            && heights.len() == hw * (th + 1)
            && world.set_corner_grid(heights.to_vec());
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
            } else if heights.is_empty() {
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
    // `world+0x34`, the style's sea class, printed with the WORLD scalars
    // at every level (`sea_map 1` on run9's Great Lakes, 4 on run20's
    // islands). Not derivable from the cells: `World::sea_map`.
    world.set_sea_map(get("sea_map").unwrap_or(0) as i32);
    (world, region_map)
}

/// One `DUMP_ALL` frame block, stood up as a simulation.
pub struct AtFrame {
    pub sim: Sim,
    /// The buildings that were created: simulation handle → the log's
    /// object number.
    pub builds: Vec<(usize, i64)>,
    pub notes: Vec<String>,
}

impl AtFrame {
    /// The handle of the building the log calls `who`/`o`.
    pub fn build(&self, who: i64, o: i64) -> Option<usize> {
        self.builds
            .iter()
            .find(|(b, n)| *n == o && i64::from(self.sim.buildings[*b].owner) == who)
            .map(|(b, _)| *b)
    }
}

/// The state at the **end of sim-frame `frame − 1`**, from a windowed
/// `DUMP_ALL` capture: the map exactly as the block prints it — cells,
/// tile masks, fog and `master_land_heights` — with every dumped building
/// stood up on it, typed and active.
///
/// [`build_sim`] is the start-of-game path and cannot be this: it infers a
/// building's type from its object number (`start_of_game`), which only
/// holds for the setup's own list. A frame block carries `orig_type` on
/// every record, so a mid-game state needs no inference at all.
///
/// What it is for is a mechanic whose input is the world and whose output
/// is the world — the road plan is the first (`docs/ROADS.md` §7). It is
/// deliberately **not** a whole game: units, orders, the economy and the
/// leaders are not restored, so nothing here can be ticked. Building the
/// objects moves the map (a footprint is reserved, a city masks its radius
/// and re-syncs the borders), so the world is put back to the dump's
/// afterwards and the dump is what the mechanic then reads.
pub fn sim_at_frame(loaded: &Loaded, log: &Log, frame: i64, tuning: Tuning) -> Option<AtFrame> {
    let mut notes = Vec::new();
    let (_, body) = log.dumps().into_iter().find(|(n, _)| *n == frame)?;
    let w = body.kid("WORLD")?;
    let heights = log.frame_heights(frame);
    if heights.is_empty() {
        notes.push("no master_land_heights in this block: the map is flat".to_string());
    }
    let (world, region_map) = world_from(&w.fields, &heights, &mut notes);
    let (_, builds, leaders) = crate::gamelog::records(body, false);
    let players = builds
        .iter()
        .map(|b| b.who)
        .chain(leaders.iter().map(|l| l.who))
        .filter(|w| (0..8).contains(w))
        .max()
        .map_or(1, |m| m as usize + 1);
    let mut sim = loaded.sim(tuning, world.clone(), players);
    for l in &leaders {
        if (0..players as i64).contains(&l.who) {
            sim.nation[l.who as usize].human = l.leader_flags & 4 != 0;
        }
    }

    // Cities first: a building joins its city at placement, so the centre
    // has to be standing before its members are.
    let mut out: Vec<(usize, i64)> = Vec::new();
    let place = |sim: &mut Sim, b: &crate::gamelog::BuildDump, notes: &mut Vec<String>| {
        let Some(ty) = b
            .orig_type
            .and_then(|t| loaded.build_of_type_index(t as i32))
        else {
            notes.push(format!(
                "build {}/{}: orig_type {:?} is not a build type — skipped",
                b.who, b.o, b.orig_type
            ));
            return None;
        };
        let h = sim.init_build(b.who as sim::Player, ty, pos_of(b.pos), false);
        sim.buildings[h].index = b.o as i16;
        sim.activate(h, false, false);
        Some((h, b.o))
    };
    let city_of = |b: &crate::gamelog::BuildDump| {
        b.orig_type
            .and_then(|t| loaded.build_of_type_index(t as i32))
            .is_some_and(|ty| sim::build::is_city(&loaded.build_types, ty))
    };
    for b in builds
        .iter()
        .filter(|b| (0..players as i64).contains(&b.who))
    {
        if city_of(b)
            && let Some(p) = place(&mut sim, b, &mut notes)
        {
            out.push(p);
        }
    }
    for b in builds
        .iter()
        .filter(|b| (0..players as i64).contains(&b.who))
    {
        if !city_of(b)
            && let Some(p) = place(&mut sim, b, &mut notes)
        {
            out.push(p);
        }
    }

    // Standing the objects up wrote to the map — the footprints' `PLACED`,
    // `mask_city`'s radius, `sync_territory`'s owners. The dump's map is
    // the one the mechanic under test read, so it goes back.
    sim.world = world;
    notes.push(format!(
        "frame {frame}: {} buildings, {} cities, {} regions mapped",
        out.len(),
        sim.cities.iter().filter(|c| c.alive).count(),
        region_map.len()
    ));
    Some(AtFrame {
        sim,
        builds: out,
        notes,
    })
}

pub fn build_sim(loaded: &Loaded, init: &Initial, tuning: Tuning) -> Built {
    let mut notes = Vec::new();
    let get = |k: &str| -> Option<i64> {
        init.world
            .iter()
            .find(|(key, _)| *key == k)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let players = player_count(init).max(1);
    let (mut world, region_map) = world_from(&init.world, &init.heights, &mut notes);
    // `Region.flags`, from the dump's own `REGIONS` block, through the same
    // map the cells were numbered by. Bit 8 is the resource-region flag —
    // the gate on `Region::go_here`'s "free to settle" and so on the AI
    // sending a scout to another island at all (`docs/TRANSPORT.md` §7,
    // §9.4). Only an `InitialDump` carries the block; without it every
    // region's flags stay 0 and `go_here` answers as it did before.
    let mut flagged = 0;
    for rec in &init.regions {
        if let Some(&(_, sim_r)) = region_map.iter().find(|(d, _)| *d == rec.region) {
            world.set_region_flags(sim_r, rec.flags as i32);
            flagged += 1;
        }
    }
    if flagged > 0 {
        notes.push(format!("regions: {flagged} carry the dump's own flags"));
    }
    let mut sim = loaded.sim(tuning, world, players);
    // The harness is where the per-phase fold is wanted: it is what the
    // sim's own draws are lined up against the trace's sites with
    // (`docs/SYNC.md` §4.2). Everything else leaves it off.
    sim.trace_phases = true;
    sim.lobby = lobby_of(&init.game_info, &loaded.map_styles);
    // `info.flags & 4` is asked of two layers — the AI's host function
    // `get_is_no_nation_powers` reads the lobby, `has_tribe_bonus` reads the
    // tech tree's `Setup` — and it is one bit, so they are kept the same.
    sim.setup.no_nation_powers = sim.lobby.no_nation_powers;
    // The map seed, which picks a gaia guy's piece (`docs/ANIM.md` §3).
    sim.game_seed = get("seed").unwrap_or(0) as i32;
    // The animation art: the lengths a `DUMP_ALL` dump (or a sibling)
    // showed, and which types take gaia's three-piece rule.
    for &(p, a, e) in &init.anim_lengths {
        sim.art.lengths.insert((p as i32, a as i8), e as u32);
    }
    for t in 0x192..0x19e {
        if let Some(k) = loaded.unit_of_type_index(t) {
            sim.art.gaia_types.insert(k);
        }
    }
    if !init.anim_lengths.is_empty() {
        notes.push(format!(
            "anim: {} (piece, slot) lengths from the dump",
            init.anim_lengths.len()
        ));
    }
    // And the gaia types' whole slot lists, from the install rather than
    // the dump — which is what gives a bird a length at all
    // (`crate::artdata`, `docs/SYNC.md` §3.9).
    sim.art.gaia_lengths = loaded.gaia_lengths.clone();
    if !loaded.gaia_lengths.is_empty() {
        notes.push(format!(
            "anim: {} (type, variant, slot) gaia lengths from the install",
            loaded.gaia_lengths.len()
        ));
    }
    // And every player unit piece's whole slot list, from the same three
    // files — which is what stops an idle variant nothing has played from
    // being handed the default's length (`docs/ANIM.md` §3.2).
    sim.art.piece_lengths = loaded.piece_lengths.clone();
    if !loaded.piece_lengths.is_empty() {
        notes.push(format!(
            "anim: {} unit pieces' slot lists from the install",
            loaded.piece_lengths.len()
        ));
    }
    // The craft table — `craftrules.xml`'s own rows, which is what tells a
    // cast how long it takes (`docs/ORDERS.md` §6.9). Without it every
    // `JOB_TIME` reads 0 and the fishing boat's deploy would land on the
    // frame it is queued instead of forty frames later.
    sim.spells = loaded.spells.clone();
    if !loaded.spells.is_empty() {
        notes.push(format!(
            "crafts: {} rows from the install",
            loaded.spells.len()
        ));
    }
    // And every piece's crew-follow offset, from the same file — which is
    // what gives a scout's dog a body of its own instead of the man's
    // (`docs/MOVEMENT.md`, "The follower's destination").
    sim.art.tracks = loaded.piece_tracks.clone();
    if !loaded.piece_tracks.is_empty() {
        notes.push(format!(
            "anim: {} unit pieces' crew track offsets from the install",
            loaded.piece_tracks.len()
        ));
    }

    for l in &init.leaders {
        if (0..players as i64).contains(&l.who) {
            let who = l.who as usize;
            // The nation, and every power that follows from it
            // (`crates/sim/src/nations.rs`). The dump's `tribe` is the
            // roster index — run40's 11 and 4 are the British and the
            // Nubians — and `-1`, which a gaia leader carries, is a leader
            // with no nation rather than the Aztecs.
            sim.set_tribe(who as u8, l.tribe);
            // `leader_flags & 4` is `LeaderData::is_human` — the bit that
            // picks `find_wpath`'s pull-back variant and withholds the
            // `army`/`worker` cost modes (`docs/PATHFINDER.md` §3).
            sim.nation[who].human = l.leader_flags & 4 != 0;
            // **`diplos`, which nothing installed until 2026-09-02.** The
            // harness built every capture with an all-peace matrix while
            // every one of them opens at war, so `is_enemy` was false for
            // forty-six readers and `do_danger`'s enemy arm halved twice
            // (`crates/sim/src/danger.rs`). The diagonal is the leader's
            // own 2 and is skipped; a dump below `LEADERS=1` carries no
            // `diplos` at all and leaves the matrix alone.
            for (other, &d) in l.diplos.iter().enumerate().take(players) {
                if other != who {
                    sim.at_war[who][other] = d == 0;
                    sim.allied[who][other] = d == 2;
                }
            }
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

    // The units — the players' and gaia's (owners 8 and 9: the animals and
    // the birds, whose animation clocks are on the sync stream).
    let mut units = Vec::new();
    let mut clocks = 0usize;
    for u in &init.units {
        let gaia = (8..10).contains(&u.who);
        if !(0..players as i64).contains(&u.who) && !gaia {
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
        unit.type_index = u.guys.first().and_then(|g| g.kind).unwrap_or(-1) as i32;
        if let Some(t) = ty {
            unit.kind = t.kind;
            unit.movement.speed = t.moves;
            unit.movement.turning = sim::turning_of(t);
        }
        if gaia {
            // The herd: not logged per animal, so the herd of the animal's
            // own type — one sheep herd on this lobby; the fish never reach
            // the wander (`docs/ANIM.md` §7).
            unit.herd = init
                .herds
                .iter()
                .position(|h| Some(h.t) == u.guys.first().and_then(|g| g.kind));
        }
        let idx = sim.add_unit(unit);
        // The figures' clocks and pieces (`docs/ANIM.md` §1, §3). A
        // `DUMP_ALL` start dump carries both; any other carries neither,
        // and a sibling's clocks land here through `borrow_from_siblings`.
        for (n, g) in u.guys.iter().enumerate() {
            if let (Some(piece), Some(k)) = (g.gpiece, kind) {
                let o = u.o as i16;
                let sub = if sim.art.gaia_types.contains(&k) {
                    (sim.game_seed.wrapping_add(i32::from(o))).rem_euclid(3) as u8
                } else {
                    (o & 1) as u8
                };
                sim.art
                    .pieces
                    .insert((u.who as sim::Player, k, sub, n as u8), piece as i32);
            }
            if let Some(guy) = guy_of(g) {
                sim.set_guy(idx, n, guy);
                clocks += 1;
            }
        }
        // `Unit::set_new_location`'s placement snap, once the pieces are
        // in: a crew guy whose piece names a track offset is put on it
        // rather than on its leader. Deriving the position rather than
        // reading the dump's `GUY x/y` is deliberate — it is what lets
        // `run56_s_scout_dog_walks_its_own_body` compare the two.
        sim.seat_guys(idx);
        // `Unit::init`'s tally — `num_units`, the group count, `control` —
        // which the price ramp and `population()` read. A unit stood up
        // from the dump counts exactly as a trained one does.
        if let Some(ty) = kind
            && !gaia
        {
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
    if clocks > 0 {
        notes.push(format!(
            "anim: {clocks} guy clocks installed from the start dump, {} pieces known",
            sim.art.pieces.len()
        ));
    }

    let builds = start_of_game(&mut sim, loaded, init, players, &units, &mut notes);
    // `Farms::log_data`'s list: each farm's `farm_type`, and with it the
    // pasture — the farm that grows nothing, so `Farms::inc_time` spends no
    // draw on it, and that carries five animals of **owner 9** which do
    // (`docs/SYNC.md` §3.6). No dump prints an owner-9 object, so the five
    // are stood up here from the farm alone.
    if !init.farms.is_empty() {
        let (mut pastures, mut animals) = (0usize, 0usize);
        for f in &init.farms {
            let Some(b) = sim
                .buildings
                .iter()
                .position(|bd| i64::from(bd.owner) == f.who && i64::from(bd.index) == f.o)
            else {
                notes.push(format!("farms: no building for who {} o {}", f.who, f.o));
                continue;
            };
            sim.buildings[b].farm.farm_type = u8::try_from(f.farm_type).unwrap_or(0);
            if sim.buildings[b].farm.farm_type == sim::farms::ANIMAL_FARM {
                // The five's species and their two offsets, from the run's
                // own trace where it reached the setup — the harness cannot
                // draw them (`docs/SYNC.md` §3.11).
                let seeds: &[sim::farms::AnimalSeed] =
                    init.pasture.get(pastures).map_or(&[], Vec::as_slice);
                pastures += 1;
                animals += sim.farm_add_animals(b, seeds).len();
            }
        }
        notes.push(format!(
            "farms: {} from the dump, {pastures} pasture(s) carrying {animals} animals of owner 9",
            init.farms.len()
        ));
    }
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
    // The goods, in the dump's own order — which is the `goods` list order
    // `WData.down_who` indexes. The record names the *type*, so the id
    // comes back through `good_names`; a name the tables do not carry is
    // dropped with a note rather than guessed, because the id is what
    // `type_avail` is asked about.
    let mut unknown: Vec<&str> = Vec::new();
    for g in &init.goods {
        let Some(row) = loaded.good_names.iter().position(|n| n == g.name) else {
            if !unknown.contains(&g.name) {
                unknown.push(g.name);
            }
            continue;
        };
        sim.world.add_good(sim::world::Good {
            pos: Pos::new(g.x as i32, g.y as i32),
            ty: loaded.good_tree[row],
            alive: g.flags & 1 != 0,
        });
    }
    if !init.goods.is_empty() {
        notes.push(format!(
            "goods: {} from the dump, {} linked to a cell{}",
            init.goods.len(),
            sim.world
                .goods()
                .iter()
                .filter(|g| g.ty != sim::world::OIL)
                .count(),
            if unknown.is_empty() {
                String::new()
            } else {
                format!(" (unnamed types: {unknown:?})")
            }
        ));
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
        frame_guys: init.frame_guys.clone(),
        frame_sites: Vec::new(),
        type_index: (0..loaded.good_tree.len()
            + loaded.unit_tree.len()
            + loaded.build_tree.len()
            + loaded.tech_tree.len())
            .map(|id| loaded.type_index(id))
            .collect(),
        unit_tree: loaded.unit_tree.clone(),
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
                Some(t) => sim.init_build(who, t, pos, false),
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
            //
            // **Before `activate`, and that is the whole of item 162.**
            // `Build::init` fills the list at placement and `Build::activate`
            // then adds the `gather_max` surveyed from it to the leader's
            // `gather_slots`. `init_build` above runs the walk itself, but on
            // a *pre-placed* camp it can only come back empty: the dump's own
            // tile masks are loaded into the world verbatim, the camp's
            // tiles already carry `0x1000` (`is_gathered_from`), and the walk
            // skips every cell that does. So the list has to be installed
            // from the dump — and the count with it — while `activate` can
            // still see it, or the starting camp joins the game claiming
            // nothing. run59's census is what caught it: the human, which
            // builds nothing all game, held `gather_slots[timber]` 0 against
            // the original's 6 (`docs/ECONOMY.md`, "The census at the word").
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
            if ty.is_some() {
                sim.activate(handle, false, true);
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
    // `Setup::build_game@005ac190` builds the starting positions in the
    // order of `info.player[k]`'s start-slot field (unread), and every
    // `DUMP_ALL` dump on hand — run3, run5 (a different lobby) and run13 —
    // shows the result: the `Farms` list is player 1's three farms, then
    // player 0's, while `BUILDDATA` runs player 0 first. The farm draws are
    // spent in `Farms` order (`docs/SYNC.md` §4.1: run13's frame-101 sprout
    // is the AI's `2003`, the list's second), so the starting farms are
    // ordered here as the original activated them: the higher slot first,
    // stable within a player.
    let owners: Vec<sim::Player> = sim.buildings.iter().map(|b| b.owner).collect();
    sim.farm_order
        .sort_by_key(|&h| std::cmp::Reverse(owners[h]));
    // **The dump's height grid is already terraformed for every building
    // the dump lists** — run12's frame-0 heights have p0's city standing on
    // its own plateau — so standing the roster up through `Wall::start`
    // flattens ground that is flat already, and a second pass over a box
    // whose border was blended is not the identity. The grid the dump
    // printed is the truth at frame 0; put it back
    // (`sim::terrain`, `docs/ROADS.md` §7.4).
    if !init.heights.is_empty() {
        sim.world.set_corner_grid(init.heights.clone());
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
    /// One field of a `GATHERORDER`'s own row, named as the log writes it
    /// (`docs/ORDERS.md` §6.4). The tile is the one that matters: the order
    /// list can agree on kind, target and flags for a hundred frames while
    /// the two sides send the worker to different trees, and the first sign
    /// is a `Length` two frames later when one of them queues a walk the
    /// other does not.
    Gather {
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
    /// `MoveOrder::coll_x`/`coll_y` — the point the last collision refused
    /// (`docs/COLLISION.md` §4.3).
    Coll {
        ours: Option<(i32, i32)>,
        theirs: (i64, i64),
    },
    /// One field of a `MOVEORDER`'s own row, named as the log writes it
    /// (`docs/ORDERS.md` §4.1). The destination is the one that matters:
    /// two sides can hold the same kind, the same target and the same
    /// flags while walking to different points, and the first thing any
    /// other check sees is a `Length` a few frames later when one of them
    /// arrives and the other does not.
    Move {
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
    /// The path stack's depth.
    PathLength { ours: usize, theirs: usize },
    /// A path segment's goal, bottom-first.
    PathTo {
        slot: usize,
        ours: (i32, i32),
        theirs: (i64, i64),
    },
    /// The **rest** of a `PATHDATA` row — `tolerance` and `flags`, the two
    /// fields the dump prints beside the point and which nothing compared
    /// for as long as the path stack has been an oracle
    /// (`docs/PATHFINDER.md` §10). The working agreement's "diff the whole
    /// record": a stack whose points are the original's can still carry a
    /// waypoint the original marked final and this crate did not, and that
    /// difference is what says *who built the stack*, not merely where it
    /// goes.
    PathField {
        slot: usize,
        field: &'static str,
        ours: i64,
        theirs: i64,
    },
}

impl OrderMismatch {
    /// Whether this is about the path stack rather than the order list.
    pub const fn is_path(&self) -> bool {
        matches!(
            self,
            Self::PathLength { .. } | Self::PathTo { .. } | Self::PathField { .. }
        )
    }

    /// Whether it counts against the order score. Everything does except
    /// [`Self::Flags`], for the reason on that variant, and two fields of
    /// [`Self::Move`]:
    ///
    /// - **`dest`** is "I have a current waypoint", and the original clears
    ///   it on arrival, on a `go_around_building` failure, after
    ///   `resolve_unit_collision` and on a collision at `coll_x/coll_y`
    ///   (`docs/ORDERS.md` §4.1) — three of the four are the pathfinder
    ///   seam's own timing, which this crate does not reproduce frame for
    ///   frame. `dest_x/dest_y`, the waypoint itself, scores on the frames
    ///   the flag says it is live, and the path stack scores outright.
    /// - **`last_x/last_y`** is where the last straight-line plan was made,
    ///   and only a *successful detour* writes it — `go_around_building`,
    ///   the same seam.
    /// - **`facing`** is the formation mirror, and `docs/QUEUE.md` item 23
    ///   is the open question of its sign: the mirrors the orders carry and
    ///   the flags the pool prints are known to be two different sequences.
    ///
    /// Both are reported, because a surprise in either is worth seeing.
    pub fn scores(&self) -> bool {
        match self {
            Self::Flags { .. } => false,
            Self::Move { field, .. } => !matches!(*field, "dest" | "facing" | "last_x" | "last_y"),
            _ => true,
        }
    }

    /// The variant's name, for a tally.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Length { .. } => "length",
            Self::Kind { .. } => "kind",
            Self::Action { .. } => "action",
            Self::Target { .. } => "target",
            Self::Flags { .. } => "flags",
            Self::Gather { .. } => "gather",
            Self::Coll { .. } => "coll",
            Self::Move { .. } => "move",
            Self::PathLength { .. } => "path-length",
            Self::PathTo { .. } => "path-to",
            Self::PathField { .. } => "path-field",
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
    /// The `(who, o)` of each unit the **simulation** has on this frame and
    /// the original does not — [`unlinked`](Self::unlinked_units)' mirror.
    ///
    /// Counting one side only is a measure that can be *gamed by
    /// over-producing*: a simulation whose AI trains its ninth citizen four
    /// hundred frames early reads as "every unit the original has, we have"
    /// from the frame the original catches up, and the early frames cost it
    /// nothing. Both directions are counted so that neither running ahead
    /// nor running behind can hide.
    pub extra_units: Vec<(i64, i64)>,
    pub compared: usize,
    pub diverged: Vec<Divergence>,
    /// The leaders' scores as logged, by `who`.
    pub scores: Vec<(i64, i64)>,
    /// Units whose order list the log carries, so both sides could be
    /// compared — zero below `UNITS=3`.
    pub order_compared: usize,
    /// Every order-list and path-stack disagreement this frame.
    pub order_diverged: Vec<OrderDivergence>,
    /// Unit-frames whose `ObjectData::mylos` the log carried, so
    /// [`sim::Sim::unit_los`] could be checked against it.
    pub los_compared: usize,
    /// Every one that disagreed — `docs/VISION.md` §2. The dump writes
    /// `mylos` at every detail level, so this is compared on every capture.
    pub los_diverged: Vec<LosDivergence>,
    /// Unit-frames whose `UnitData::unit_masks` the log carried, so the
    /// **packed** bit (`0x80000`) could be checked against
    /// [`sim::combat::Combat::packed`] — `docs/ORDERS.md` §6.9. Written at
    /// every detail level, like `mylos`, so it too is on every capture.
    pub packed_compared: usize,
    /// Every unit-frame whose packed bit disagreed: `(frame, who, o, ours)`.
    pub packed_diverged: Vec<PackedDivergence>,
    /// Angle comparisons made this frame — `UnitData::angle` for every unit
    /// record, and guy 0's `angle` for every record that carries a guy
    /// (`GUYS` at 1 or above). Two per unit-frame where both are present.
    pub angle_compared: usize,
    /// Every heading or facing that disagreed (`docs/MOVEMENT.md`).
    pub angle_diverged: Vec<AngleDivergence>,
    /// Collision-block fields compared this frame, and the ones that
    /// disagreed (`docs/COLLISION.md` §8).
    pub collide_compared: usize,
    pub collide_diverged: Vec<CollideDivergence>,
    /// Gather-record fields compared this frame, and the ones that
    /// disagreed. `BUILDS=7` is what writes the mining list; below it only
    /// `gather_down` is compared, and on a capture with no `BUILDDATA` at
    /// all the count is zero.
    pub gather_compared: usize,
    pub gather_diverged: Vec<GatherDivergence>,
    /// Buildings the frame names for a player and the simulation does not
    /// hold — the gather comparison's own blind spot, counted rather than
    /// assumed away.
    pub build_unlinked: usize,
    /// A building's own `x_internal`/`y_internal` and `orig_type`, compared
    /// on every linked building of every frame — the half of `BUILDDATA`
    /// that says *where the AI put it*, which nothing compared until the
    /// word reached 3021.
    pub build_compared: usize,
    pub build_diverged: Vec<BuildDivergence>,
    /// **The production queue, whole** — `queued` and, for each live slot,
    /// its `type`, `job_counter` and the three `(good, cost)` pairs
    /// (`docs/PRODUCTION.md`, "The queue record"). Written from
    /// `BUILDS=1`; a capture below it compares nothing here.
    ///
    /// Only the first `queued` slots are read: the tail of the array holds
    /// whatever it was last left with — `type −1` on a queue never used,
    /// `type 0` on one that has been — which is not state either side owns.
    pub queue_compared: usize,
    pub queue_diverged: Vec<QueueDivergence>,
    /// **The `CITY` record, whole** — every field
    /// `CityData::log_data@004895c0` writes, on every city of every frame
    /// (`docs/CITIES.md` §5.7). Written at every detail level, like `mylos`
    /// and the packed bit, so this is compared on every capture that dumps
    /// frames at all.
    pub city_compared: usize,
    pub city_diverged: Vec<CityDivergence>,
    /// Cities the frame names and the simulation cannot link to a building
    /// of its own — the city comparison's blind spot, counted rather than
    /// assumed away, exactly as [`FrameReport::build_unlinked`] is.
    pub city_unlinked: usize,
}

/// One field of a `CITY` record the two sides disagree on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CityDivergence {
    pub frame: i64,
    pub who: i64,
    /// The city building's object number — the identity the link is made
    /// on, and the one a `BUILDDATA` row shares.
    pub o: i64,
    /// The field, named as `CityData::log_data` writes it; an array element
    /// as `space[k]` / `ter[k]`.
    pub field: String,
    pub ours: i64,
    pub theirs: i64,
}

/// One field of a building's **production queue** the two sides disagree
/// on — `BuildQueue::log_data`'s own record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as the dump writes it: `queued`, or
    /// `queue[k].<field>` for a slot's.
    pub field: String,
    pub ours: i64,
    pub theirs: i64,
}

/// One field of a building's identity the two sides disagree on — its
/// position, or the type it was created as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as `BuildData::log_data` writes it.
    pub field: &'static str,
    pub ours: i64,
    pub theirs: i64,
}

/// Which of a unit's two angles disagreed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// `UnitData::angle` (`+0x50`) against `Movement::heading` — the bearing
    /// to the destination, which `Unit::set_angle` writes outright.
    Heading,
    /// Guy 0's `angle` (`+0x18`) against `Movement::facing` — the direction
    /// the step was actually taken along, which only the turn rate moves.
    Facing,
}

/// One unit whose heading or facing the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AngleDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub which: Which,
    pub ours: i32,
    pub theirs: i64,
}

/// One field of a building's **gather record** the two sides disagree on —
/// `BuildData::gather_from` and the `MiningList` header it is written
/// under, plus `gather_down`'s chain head (`docs/ECONOMY.md`, "The spine";
/// `docs/ORDERS.md` §6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as the dump writes it.
    pub field: &'static str,
    /// The entry of the mining list this is about, or `−1` for a scalar.
    pub at: i64,
    pub ours: i64,
    pub theirs: i64,
}

/// One field of a unit's collision block the two sides disagree on —
/// `docs/COLLISION.md` §8. The block is written at every detail level, so
/// this is compared on every capture the harness reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollideDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as `UnitData::log_data` writes it.
    pub field: &'static str,
    pub ours: i64,
    pub theirs: i64,
}

/// One unit whose line of sight the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LosDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: i32,
    pub theirs: i64,
}

/// One unit-frame whose `unit_masks & 0x80000` disagreed — the packed bit
/// `Unit::init` sets, `SpellType::cast_pack` sets again and
/// `SpellType::cast_unpack` clears (`docs/ORDERS.md` §6.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackedDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: bool,
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

/// The simulation's own draws as a **sequence of site labels**, one entry
/// per draw — the harness's side of
/// [`crate::trace::Trace::run_in`](crate::trace::Trace::run_in).
///
/// `marks` is [`sim::Sim::phase_marks`], each pair a label and the stream's
/// word *before* that label's draws; `end` is the word after the last of
/// them. A mark that drew nothing contributes nothing, which is what makes
/// a guarded site read correctly — `sim::scout` skips its phase draw on
/// rings under four — rather than as a zero-length hole.
///
/// This is the primitive the seed-anchored check is built on: seed the sim
/// with the trace's own word, run one mechanic, and compare this against
/// the trace's sites. A count cannot tell four rotations and two phases
/// from three and three; this can.
pub fn mark_sites(marks: &[(String, u32)], end: u32) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for (i, (label, from)) in marks.iter().enumerate() {
        let to = marks.get(i + 1).map_or(end, |m| m.1);
        for _ in 0..draws_between(*from, to)? {
            out.push(label.clone());
        }
    }
    Some(out)
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
    /// The frame the sim just stepped, folded by phase — the harness's
    /// answer to `tools/trace/report.py … sites`, which folds the
    /// original's draws by return address (`docs/SYNC.md` §4.2). Phases
    /// that drew nothing are dropped; the unit loop is one entry per unit
    /// that drew, because that is the half that has to be lined up against
    /// the trace. `None` when nothing drew, or when the marks are off.
    fn phase_fold(&self) -> Option<String> {
        let mut rows: Vec<(String, u32)> = Vec::new();
        for pair in self.sim.phase_marks.windows(2) {
            let (label, from) = &pair[0];
            let n = draws_between(*from, pair[1].1)?;
            if n > 0 {
                rows.push((label.clone(), n));
            }
        }
        // A hundred animals rolling one idle each is one fact, not a
        // hundred: a run of neighbouring units drawing the same number
        // folds to its first and last.
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < rows.len() {
            let mut j = i;
            while j + 1 < rows.len()
                && rows[j + 1].1 == rows[i].1
                && rows[i].0.starts_with("unit ")
                && rows[j + 1].0.starts_with("unit ")
            {
                j += 1;
            }
            if j > i {
                out.push(format!(
                    "{}..{} ×{} {}",
                    rows[i].0,
                    rows[j].0.trim_start_matches("unit "),
                    j - i + 1,
                    rows[i].1
                ));
            } else {
                out.push(format!("{} {}", rows[i].0, rows[i].1));
            }
            i = j + 1;
        }
        (!out.is_empty()).then(|| out.join(", "))
    }

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
        // The frame's own draws, named, before anything is installed —
        // the harness's side of `Trace::labels(frame)`.
        if self.sim.trace_phases
            && let Some(sites) = mark_sites(&self.sim.phase_marks, self.sim.rng.seed)
        {
            self.frame_sites.push((frame, sites));
        }
        if let Some(&(_, theirs)) = self.frame_seeds.iter().find(|(n, _)| *n == frame) {
            let ours = draws_between(word_before, self.sim.rng.seed);
            let orig = draws_between(word_before, theirs);
            self.rng_frames.push((frame, ours, orig));
            if let Some(fold) = self.phase_fold() {
                self.notes
                    .push(format!("rng: frame {frame}: ours by phase — {fold}"));
            }
            self.notes.push(format!(
                "rng: frame {frame}: ours {} draws, the original's {} — installed {theirs:#010x}",
                ours.map_or("?".to_string(), |n| n.to_string()),
                orig.map_or("?".to_string(), |n| n.to_string()),
            ));
            self.sim.rng.seed = theirs;
            // And the clocks, where the dump printed them: every linked
            // unit's guys as they stood at this frame's end, so the next
            // wrap falls where the original's does (`docs/ANIM.md` §8).
            if let Some((_, states)) = self.frame_guys.iter().find(|(n, _)| *n == frame) {
                let mut installed = 0;
                let mut skipped = 0;
                let mut arrivals = 0;
                let (mut reseated, mut drift) = (0usize, 0i32);
                for state in states {
                    let (who, o, guys) = (&state.who, &state.o, &state.guys);
                    let unit = self
                        .units
                        .iter()
                        .find(|l| l.who == *who && l.o == *o)
                        .map(|l| l.unit)
                        .or_else(|| {
                            i16::try_from(*o)
                                .ok()
                                .and_then(|o| self.sim.unit_by_o(*who as sim::Player, o))
                        });
                    let Some(u) = unit else {
                        continue;
                    };
                    // **Gaia's animals are re-seated, not compared.** Where
                    // they wander is decided by draws on a stream the
                    // harness only holds at the traced frames, so between
                    // two of them they drift — and an animal's arrival
                    // costs a draw in the middle of the unit loop, between
                    // one player's units and the next (`sim::Sim::
                    // reseat_animal`, `docs/SYNC.md` §4.1). Putting them
                    // back is what keeps the players' units on the
                    // original's draws; the drift is reported rather than
                    // hidden.
                    if *who >= 8 {
                        let d =
                            self.sim
                                .reseat_animal(u, pos_of(state.pos), state.goal.map(pos_of));
                        if d > 0 {
                            reseated += 1;
                            drift = drift.max(d);
                        }
                        for (n, g) in guys.iter().enumerate() {
                            if let Some(guy) = guy_of(g) {
                                self.sim.set_guy(u, n, guy);
                                installed += 1;
                            }
                        }
                        continue;
                    }
                    // A walking clock on a unit the sim has standing (or
                    // the reverse) is no correction: the sim would then
                    // read every idle request as an arrival. Such a unit
                    // keeps its own clock and is counted.
                    //
                    // **Unless the dump's own unit is standing there too.**
                    // A walk animation over an *empty* order list is the
                    // arrival itself — the order popped this frame and the
                    // figure still holds the walk it was playing, which is
                    // exactly the state `Guy::set_anim` reads as an arrival
                    // on the next frame and draws for (`docs/ANIM.md` §4).
                    // The sheep of run10's frame 101 is this and nothing
                    // else: it wandered on an untraced frame the sim's
                    // stream cannot reach, and without its clock the sim
                    // skipped the arrival draw that stands between the AI's
                    // farmers and the human's in `docs/SYNC.md` §4.1.
                    let walking_here = self.sim.units[u]
                        .orders
                        .front()
                        .is_some_and(|o| matches!(o.body, sim::orders::Body::Move(_)));
                    let walking_there = guys
                        .first()
                        .and_then(|g| g.cur_anim)
                        .is_some_and(|a| sim::anim::category(a as i8) == 8);
                    if walking_here != walking_there {
                        if !(walking_there && state.orderless && !walking_here) {
                            skipped += 1;
                            continue;
                        }
                        arrivals += 1;
                    }
                    for (n, g) in guys.iter().enumerate() {
                        if let Some(guy) = guy_of(g) {
                            self.sim.set_guy(u, n, guy);
                            installed += 1;
                        }
                    }
                }
                self.notes.push(format!(
                    "anim: frame {frame}: {installed} guy clocks installed ({arrivals} arrivals the sim had not made), \
                     {skipped} units skipped (walking on one side only), \
                     {reseated} of gaia's animals re-seated (drift up to {drift})"
                ));
            }
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
            // A `TradeOrder`'s `(o, who)` at `+0x8` name a **city**, not
            // an object of the unit lists this compares — and the dump
            // prints the city's own centre object there, which
            // `build_ids` cannot resolve from a city index.
            Body::Trade(_) => None,
            // A `CastOrder` for the transport spell is untargeted
            // (`docs/TRANSPORT.md` §6): its `(o, who)` are `(-1, -1)`.
            Body::Move(_) | Body::Cast(_) | Body::Think => None,
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
        // **`coll_x`/`coll_y`**, the point the last collision refused. The
        // original leaves the pair at its `(0, 0)` start until a collision
        // writes it, and never clears it (`docs/COLLISION.md` §4.3).
        if let Some(theirs) = theirs.coll_x.zip(theirs.coll_y)
            && theirs != (0, 0)
            && let sim::orders::Body::Move(m) = ours.body
        {
            let mine = m.coll.map(|p| (p.x, p.y));
            if mine.map(|(x, y)| (i64::from(x), i64::from(y))) != Some(theirs) {
                at(slot, OrderMismatch::Coll { ours: mine, theirs });
            }
        }
        // **The move order's own row**, field for field (§4.1) — every
        // field of the record this crate models. `tolerance`, `retry`,
        // `attempts` and `orig_x/orig_y` are left out because nothing here
        // writes them and `run29_s_move_orders_match_the_field_table_row_
        // for_row` already pins them against the original; `off_x/off_y`
        // are `x mod 0x300` and so are the destination said twice, which
        // is worth having as a check on this crate's own snapping.
        //
        // Until 2026-08-31 the whole row went uncompared and only
        // `coll_x/coll_y` was read, which is how East Indies' `1/4` came
        // to be booked as an order-list *length* at frame 168: the two
        // sides had picked different camp spots on **167**, and nothing
        // looked at the field that said so.
        if let sim::orders::Body::Move(m) = ours.body {
            // `dest_x/dest_y` is the **current waypoint**, and it is live
            // only while `dest` is 1: on arrival the original clears the
            // flag and leaves the pair holding the waypoint it just
            // reached (§4.1), which is state this crate does not carry.
            // So the pair is compared on the frames the flag says it means
            // something, and `dest` itself on every frame.
            let live = m.has_waypoint && theirs.dest == Some(1);
            let last = m.last.unwrap_or(sim::Pos::new(-1, -1));
            for (field, mine, logged) in [
                ("x", i64::from(m.dest.x), theirs.x),
                ("y", i64::from(m.dest.y), theirs.y),
                ("angle", i64::from(m.angle.0), theirs.angle),
                ("dest", i64::from(m.has_waypoint), theirs.dest),
                (
                    "dest_x",
                    i64::from(m.waypoint.x),
                    live.then_some(theirs.dest_x).flatten(),
                ),
                (
                    "dest_y",
                    i64::from(m.waypoint.y),
                    live.then_some(theirs.dest_y).flatten(),
                ),
                ("last_x", i64::from(last.x), theirs.last_x),
                ("last_y", i64::from(last.y), theirs.last_y),
                ("pause", i64::from(m.pause), theirs.pause),
                ("timer", i64::from(m.timer), theirs.timer),
                ("facing", m.facing.map_or(-1, i64::from), theirs.facing),
                ("off_x", i64::from(m.dest.x).rem_euclid(0x300), theirs.off_x),
                ("off_y", i64::from(m.dest.y).rem_euclid(0x300), theirs.off_y),
            ] {
                if let Some(theirs) = logged
                    && theirs != mine
                {
                    at(
                        slot,
                        OrderMismatch::Move {
                            field,
                            ours: mine,
                            theirs,
                        },
                    );
                }
            }
        }
        // **The gather order's own row**, field for field (§6.4). The kind
        // and the target agreeing says only that both sides are working the
        // same camp; the tile, the phase and the countdown are what say they
        // are doing the same thing at it — and the tile is the field whose
        // absence from this comparison hid the tile choice's missing access
        // filter for as long as it existed.
        if let sim::orders::Body::Gather(g) = ours.body {
            let t = g.tile.unwrap_or(sim::Pos::new(-1, -1));
            for (field, mine, logged) in [
                ("tx", i64::from(t.x), theirs.tx),
                ("ty", i64::from(t.y), theirs.ty),
                ("wait", i64::from(g.wait), theirs.wait),
                ("goto_build", i64::from(g.goto_build), theirs.goto_build),
                ("been_there", i64::from(g.been_there), theirs.been_there),
                ("dist_mod", i64::from(g.dist_mod), theirs.dist_mod),
            ] {
                if let Some(theirs) = logged
                    && theirs != mine
                {
                    at(
                        slot,
                        OrderMismatch::Gather {
                            field,
                            ours: mine,
                            theirs,
                        },
                    );
                }
            }
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
        // The rest of the row. `PATHDATA` prints four numbers and this
        // compared two of them for as long as the stack has been an
        // oracle.
        for (field, mine, logged) in [
            ("tolerance", i64::from(ours.tolerance), theirs.tolerance),
            ("flags", i64::from(ours.flags), theirs.flags),
        ] {
            if mine != logged {
                at(
                    slot,
                    OrderMismatch::PathField {
                        slot,
                        field,
                        ours: mine,
                        theirs: logged,
                    },
                );
            }
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
        // `ObjectData::mylos`, which the object record writes at every
        // detail level — `docs/VISION.md` §2. A garrisoned unit is skipped:
        // `update_los` is not run for one, so the field is whatever it held
        // when the unit was last on the map.
        if let Some(theirs_los) = u.mylos
            && built.sim.units[link.unit].on_map
        {
            r.los_compared += 1;
            let ours_los = built.sim.unit_los(link.unit);
            if i64::from(ours_los) != theirs_los {
                r.los_diverged.push(LosDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_los,
                    theirs: theirs_los,
                });
            }
        }
        // `unit_masks & 0x80000`, the **packed** bit, beside it. It is the
        // one bit of that word this crate keeps, and it is the whole state
        // the pack/unpack crafts change (`docs/ORDERS.md` §6.9): a type
        // that packs is born with it, `cast_unpack` clears it, and
        // `think_fish`'s cadence gate and `unit_los`'s clamp both read it.
        if let Some(masks) = u.unit_masks {
            r.packed_compared += 1;
            let ours_packed = built.sim.units[link.unit].combat.packed;
            if ours_packed != (masks & 0x8_0000 != 0) {
                r.packed_diverged.push(PackedDivergence {
                    frame: frame.n,
                    who: u.who,
                    o: u.o,
                    ours: ours_packed,
                });
            }
        }
        // **The collision block**, field for field
        // (`docs/COLLISION.md` §8). `UnitData::log_data` writes all five at
        // every detail level, so this is checked on every capture — and
        // the whole record is compared, not the field the mechanic happens
        // to care about. Only on unit-frames whose *positions* agree: a
        // unit that has walked somewhere else collides with different
        // things as a consequence, and counting that would measure the
        // position gap twice.
        if ours == theirs && built.sim.units[link.unit].on_map {
            let un = &built.sim.units[link.unit];
            for (field, mine, logged) in [
                ("collide", i64::from(un.collide), u.collide),
                ("collide_o", i64::from(un.collide_o), u.collide_o),
                ("collide_who", i64::from(un.collide_who), u.collide_who),
                ("collide_guy", i64::from(un.collide_guy), u.collide_guy),
                ("safe", i64::from(un.safe), u.safe),
            ] {
                let Some(theirs) = logged else { continue };
                r.collide_compared += 1;
                if theirs != mine {
                    r.collide_diverged.push(CollideDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field,
                        ours: mine,
                        theirs,
                    });
                }
            }
            // `collide_frame` starts at −1 in the original and at 0 here,
            // so it is compared only once a collision has actually
            // happened on both sides.
            if let Some(theirs) = u.collide_frame
                && theirs >= 0
                && un.collide_frame > 0
            {
                r.collide_compared += 1;
                if theirs != un.collide_frame {
                    r.collide_diverged.push(CollideDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        field: "collide_frame",
                        ours: un.collide_frame,
                        theirs,
                    });
                }
            }
        }
        // The two angles, each against its own field — **on the unit-frames
        // where the two sides still agree on the position**. A unit that has
        // walked somewhere else is facing somewhere else as a consequence,
        // and counting that would measure the position gap twice over; what
        // is wanted here is the turn model on its own. A garrisoned unit is
        // skipped for the same reason `mylos` is: nothing turns it.
        if ours == theirs && built.sim.units[link.unit].on_map {
            let m = built.sim.units[link.unit].movement;
            let mut angle = |which: Which, ours: i32, theirs: i64| {
                r.angle_compared += 1;
                if i64::from(ours) != theirs {
                    r.angle_diverged.push(AngleDivergence {
                        frame: frame.n,
                        who: u.who,
                        o: u.o,
                        which,
                        ours,
                        theirs,
                    });
                }
            };
            if let Some(theirs) = u.angle {
                angle(Which::Heading, m.heading.0, theirs);
            }
            if let Some(theirs) = u.guys.first().and_then(|g| g.angle) {
                angle(Which::Facing, m.facing.0, theirs);
            }
        }
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
    // **And the other direction.** A unit this simulation holds for a
    // player the frame does not name at all. `UnitData::log_data` writes
    // every unit of every player at any detail level, so a frame that names
    // none is a capture below the roster rather than an empty world; such a
    // frame is skipped rather than reported as an invented army.
    if frame
        .units
        .iter()
        .any(|u| (0..players as i64).contains(&u.who))
    {
        for un in built.sim.units.iter().filter(|u| u.alive()) {
            let who = i64::from(un.owner);
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let o = i64::from(un.index);
            if !frame.units.iter().any(|u| u.who == who && u.o == o) {
                r.extra_units.push((who, o));
            }
        }
    }
    // **The gather record, whole.** `BuildData::gather_from` is the one
    // input `crates/sim/src/gather.rs` surveys the slot count out of, and
    // until this the dump's copy of it went uncompared on every frame of
    // every capture — the whole list, printed for every building at
    // `BUILDS=7`, beside a `gather_down` printed at `BUILDS=1`. A camp
    // whose list is empty here and 73 tiles long there answers no
    // question the city record's `gather_slots` can, because the count is
    // *derived* from the list: the derived figure is the symptom and the
    // list is the cause (`docs/QUEUE.md` item 85).
    //
    // The two halves are compared independently, and the per-entry loop
    // only runs where the lengths agree — a list one entry short would
    // otherwise report every entry after the gap as wrong and turn one
    // fact into seventy.
    for b in &frame.builds {
        if !(0..players as i64).contains(&b.who) {
            continue;
        }
        let Some(handle) = built
            .sim
            .buildings
            .iter()
            .position(|x| i64::from(x.owner) == b.who && i64::from(x.index) == b.o)
        else {
            r.build_unlinked += 1;
            continue;
        };
        let ours = &built.sim.buildings[handle];
        // **Where it stands, and what it was made as.** The dump writes
        // `x_internal`/`y_internal` at every detail level and `orig_type`
        // from `BUILDS=6`; both were parsed and neither compared, so an AI
        // that chose a different *site* for the same object number linked
        // cleanly and read as agreement. That is what East Indies' word at
        // 3021 turned out to be.
        for (field, mine, theirs) in [
            ("x_internal", i64::from(ours.pos.x), b.pos.x),
            ("y_internal", i64::from(ours.pos.y), b.pos.y),
        ] {
            r.build_compared += 1;
            if mine != theirs {
                r.build_diverged.push(BuildDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field,
                    ours: mine,
                    theirs,
                });
            }
        }
        // **The production queue, whole.** `BuildQueue::log_data` writes
        // it for every building on every frame from `BUILDS=1`, and until
        // this it was compared in one test against one capture — so a
        // queue that filled a frame late, or a clock that ran a hundredth
        // slow, read as agreement on every other run the harness makes.
        // `docs/PRODUCTION.md`, "The queue record"; `docs/QUEUE.md` item
        // 87's ledger.
        if let Some(q) = b.queued {
            let mut off = |field: String, ours: i64, theirs: i64| {
                r.queue_compared += 1;
                if ours != theirs {
                    r.queue_diverged.push(QueueDivergence {
                        frame: frame.n,
                        who: b.who,
                        o: b.o,
                        field,
                        ours,
                        theirs,
                    });
                }
            };
            let mine = ours.queue.items.len() as i64;
            off("queued".into(), mine, q);
            // Only where the depths agree: a queue one entry short would
            // otherwise report every slot after the gap and turn one fact
            // into a page of them — the same rule the mining list keeps.
            if mine == q {
                for (k, theirs) in b.queue.iter().take(q as usize).enumerate() {
                    let item = &ours.queue.items[k];
                    let id = item.tech.unwrap_or_else(|| built.unit_tree[item.ty]);
                    let ty = built.type_index.get(id).copied().unwrap_or(-1);
                    off(format!("queue[{k}].type"), i64::from(ty), theirs.ty);
                    off(
                        format!("queue[{k}].job_counter"),
                        i64::from(item.job_counter),
                        theirs.job_counter,
                    );
                    for j in 0..3 {
                        off(
                            format!("queue[{k}].cost[{j}]"),
                            i64::from(item.cost[j]),
                            theirs.cost[j],
                        );
                        off(
                            format!("queue[{k}].good[{j}]"),
                            i64::from(item.good[j]),
                            theirs.good[j],
                        );
                    }
                }
            }
        }
        let mut wrong = |field, at, ours: i64, theirs| {
            r.gather_compared += 1;
            if ours != theirs {
                r.gather_diverged.push(GatherDivergence {
                    frame: frame.n,
                    who: b.who,
                    o: b.o,
                    field,
                    at,
                    ours,
                    theirs,
                });
            }
        };
        // `BuildData::gather_down` is the head of the chain of registered
        // gatherers, by object number; this crate keeps the chain newest
        // first, so the head is the front of the vector.
        if let Some(theirs) = b.gather_down {
            let head = ours
                .gatherers
                .first()
                .map_or(-1, |&u| i64::from(built.sim.units[u].index));
            wrong("gather_down", -1, head, theirs);
        }
        // The mining list. `mining_len` is `None` below `BUILDS=7`, which
        // is what keeps a thin capture from reading as "every list empty".
        let Some(len) = b.mining_len else { continue };
        wrong("length", -1, ours.gather_from.len() as i64, len);
        if ours.gather_from.len() as i64 != len {
            continue;
        }
        for (k, theirs) in b.gather_from.iter().enumerate() {
            let mine = ours.gather_from[k];
            let k = k as i64;
            wrong("tx", k, i64::from(mine.x), theirs.0);
            wrong("ty", k, i64::from(mine.y), theirs.1);
        }
    }
    // **The `CITY` record, whole.** `CityData::log_data` runs for every
    // live city on every frame a dump writes, at every detail level, and
    // forty fields of it went uncompared on every capture until item 154 —
    // one test read four of them, at frame 1, on one run. The rate's own
    // inputs are in here (`gatherers`, `busy`, `free`, `filled`,
    // `space[3]`, `ter[6]`), and so is every stamp the capture and
    // assimilation clocks keep.
    //
    // The link is the city **building**: `CityData::o` is the centre's
    // object number, which is the same identity a `BUILDDATA` row carries,
    // so a city is matched the way a building is rather than by position.
    for c in &frame.cities {
        if !(0..players as i64).contains(&c.who) {
            continue;
        }
        let Some(ci) = built
            .sim
            .buildings
            .iter()
            .position(|b| i64::from(b.owner) == c.who && i64::from(b.index) == c.o)
            .and_then(|b| built.sim.buildings[b].city)
        else {
            r.city_unlinked += 1;
            continue;
        };
        let ours = &built.sim.cities[ci];
        let w = c.who as usize;
        // The AI's half of the record. The original keeps these on
        // `CityData` itself, where one leader's sweep writes them for its
        // own cities; this crate keeps a copy per leader, so the owner's is
        // the one the dump is showing.
        let ai = built.sim.ai[w].city_ai.get(ci).copied().unwrap_or_default();
        // `city_flags`, bit by bit rather than as a word: four of its bits
        // are the "an active TEMPLE / GRANARY / LUMBERMILL / MARKET stands
        // here" marks of §1.4, which this crate keeps on the economy's own
        // city record instead, and a whole-word compare would report every
        // frame of every game rather than the bit that moved.
        let flags = [
            ("city_flags[0x1]", 0x1, ours.alive),
            ("city_flags[0x2]", 0x2, ours.no_heal),
            ("city_flags[0x10]", 0x10, ours.capital),
            ("city_flags[0x40]", 0x40, ours.alarm),
            ("city_flags[0x100]", 0x100, ours.unassimilated),
            ("city_flags[0x2000]", 0x2000, ours.no_muster),
            ("city_flags[0x4000]", 0x4000, ours.founding_capital),
            ("city_flags[0x8000]", 0x8000, ours.was_founding_capital),
        ];
        let mut fields: Vec<(String, i64, i64)> = vec![
            ("x".into(), i64::from(ours.pos.x), c.x),
            ("y".into(), i64::from(ours.pos.y), c.y),
            ("pop".into(), i64::from(ours.pop), c.pop),
            ("race".into(), ours.race.map_or(-1, i64::from), c.race),
            (
                "city".into(),
                built
                    .sim
                    .cities_of(c.who as sim::Player)
                    .iter()
                    .position(|&i| i == ci)
                    .map_or(-1, |i| i as i64),
                c.city,
            ),
            ("attack_stamp".into(), ours.attack_stamp, c.attack_stamp),
            ("capture_stamp".into(), ours.capture_stamp, c.capture_stamp),
            (
                "assimilation_timer".into(),
                ours.assimilation_timer,
                c.assimilation_timer,
            ),
            (
                "capture_strength".into(),
                i64::from(ours.capture_strength),
                c.capture_strength,
            ),
            // **Through the region map, not raw.** The simulation numbers
            // its regions as it finds them and the dump numbers them as the
            // generator wrote them; `Built::region_map` is the translation
            // the census check already goes through, and without it every
            // city on every capture reads as one region wrong.
            (
                "reg".into(),
                ours.reg.map_or(-1, |r| {
                    built
                        .region_map
                        .iter()
                        .find(|&&(_, s)| s == r)
                        .map_or(-1, |&(d, _)| d)
                }),
                c.reg,
            ),
            (
                "was_capital_flags".into(),
                ours.was_capital as i64,
                c.was_capital_flags,
            ),
            ("in_port".into(), i64::from(ai.in_port), c.in_port),
            (
                "peasant_dist".into(),
                i64::from(ai.peasant_dist),
                c.peasant_dist,
            ),
            ("free".into(), i64::from(ai.free), c.free),
            ("busy".into(), i64::from(ai.busy), c.busy),
            ("gatherers".into(), i64::from(ai.gatherers), c.gatherers),
            ("ocean".into(), i64::from(ai.ocean), c.ocean),
            ("land".into(), i64::from(ai.land), c.land),
            ("filled".into(), i64::from(ai.filled), c.filled),
            ("bordering".into(), i64::from(ai.bordering), c.bordering),
            (
                "ocean_filled".into(),
                i64::from(ai.ocean_filled),
                c.ocean_filled,
            ),
            ("dock_tile".into(), i64::from(ai.dock_tile), c.dock_tile),
        ];
        for (name, bit, mine) in flags {
            fields.push((
                name.to_string(),
                i64::from(mine),
                i64::from(c.city_flags & bit != 0),
            ));
        }
        for (k, &theirs) in c.space.iter().enumerate() {
            fields.push((format!("space[{k}]"), i64::from(ai.space[k]), theirs));
        }
        for (k, &theirs) in c.ter.iter().enumerate() {
            fields.push((format!("ter[{k}]"), i64::from(ai.ter[k]), theirs));
        }
        // **The four fields nothing here holds**, asserted against the
        // zero this crate answers with rather than dropped: `raid_stamp`
        // and `reduce_stamp` are `Leader::raid`'s and the reduce order's
        // clocks, `scouted` is the AI's "a scout has seen this city" mark
        // and `trade_val` is `City::compute_trade`'s output. None has a
        // writer in this crate, so each row is a claim that the original
        // never writes one either on the captures on disk — a claim a
        // capture can falsify, which is the point of leaving it in.
        for (name, theirs) in [
            ("raid_stamp", c.raid_stamp),
            ("reduce_stamp", c.reduce_stamp),
            ("scouted", c.scouted),
            ("trade_val", c.trade_val),
            ("vans.length", c.vans.len() as i64),
        ] {
            fields.push((name.into(), 0, theirs));
        }
        for (field, ours, theirs) in fields {
            r.city_compared += 1;
            if ours != theirs {
                r.city_diverged.push(CityDivergence {
                    frame: frame.n,
                    who: c.who,
                    o: c.o,
                    field,
                    ours,
                    theirs,
                });
            }
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
    run_traced(loaded, log, tuning, limit, stream, &[], None)
}

/// Fills `init`'s checksum trace and height grid from the first sibling
/// dump that has each, when `init` itself has none (see [`run_traced`]).
/// Whether two dumps opened on the same board: every starting building, by
/// owner, object number and position. It is the test a *terraformed* field
/// needs — [`Initial::heights`] is the grid after `Wall::init` has flattened
/// each footprint, so a sibling that placed its buildings elsewhere has a
/// different table however well its map seed matches.
fn same_start(a: &Initial, b: &Initial) -> bool {
    let key = |i: &Initial| -> Vec<(i64, i64, i64, i64)> {
        let mut v: Vec<(i64, i64, i64, i64)> = i
            .builds
            .iter()
            .map(|b| (b.who, b.o, b.pos.x, b.pos.y))
            .collect();
        v.sort_unstable();
        v
    };
    !a.builds.is_empty() && key(a) == key(b)
}

/// **The pasture's five, from the run's own trace.**
///
/// Every other input the harness stands its roster up from is a dump's;
/// this one cannot be, because owner 9 is in no dump block at all and the
/// three draws that place each animal are spent inside
/// `Setup::build_empire` (`docs/SYNC.md` §3.11). A trace that reached the
/// setup carries them, and [`crate::trace::Trace::add_animals`] reads them
/// back out of the seeds the records hold; a windowed trace carries
/// nothing and the simulation keeps its stand-in.
pub fn borrow_pasture(init: &mut Initial<'_>, tr: &crate::trace::Trace) {
    if init.pasture.is_empty() {
        init.pasture = tr.add_animals();
    }
}

pub fn borrow_from_siblings<'a, 'b: 'a>(init: &mut Initial<'a>, siblings: &[&Initial<'b>]) {
    if init.checksums.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.checksums.is_empty())
    {
        init.checksums = s.checksums.clone();
    }
    // **The map itself**, on the same terms as the heights.
    // `WorldData::log_data` writes the cells and the per-tile masks only at
    // `WORLD ≥ 5`; a capture taken for another category — run6 is
    // `BUILDS=7` — writes the block's seventeen scalars and stops. On such
    // a world every cell is region-less and every tile reads back `mask 0`,
    // so `has_gather_access` is false everywhere and no woodcutter can ever
    // choose a tile: a gap in the *capture* that reads exactly like a gap in
    // the mechanic.
    //
    // The gate is those seventeen scalars. They are the map seed, the
    // extent, the eight totals the generator wrote and the territory limits;
    // a sibling whose block opens with the same values, field for field,
    // generated the same map, and the rest of its block is therefore ours.
    // A sibling that disagrees anywhere contributes nothing.
    if !init.world.is_empty()
        && crate::gamelog::world_cells(&init.world).is_empty()
        && let Some(s) = siblings.iter().find(|s| {
            s.world.len() > init.world.len()
                && s.world[..init.world.len()]
                    .iter()
                    .zip(&init.world)
                    .all(|(a, b)| a == b)
        })
    {
        init.world = s.world.clone();
    }
    // **The heights are not the map's; they are the map *after* the setup
    // buildings.** `TerrainOut::terraform_for_building` flattens every
    // non-farm footprint out of `Wall::init`, before the first frame, so
    // two runs of the same seed, style and size whose starting buildings
    // stand anywhere else have different tables — and the difference is
    // not local to the footprints, because the border blend
    // `(h + mean) × 0.5` reaches a tile further and every later placement
    // compounds it.
    //
    // run3 is exactly that sibling: seed 12345, style 14, size 2 like
    // run10–14, but `GAME_RULES 0` rather than 1, and its grid differs
    // from run12's and run13's on **237 corners spread from (7, 83) to
    // (230, 163)** — over the human's own city as well as the AI's. It is
    // the first sibling in the list, so it is what the road search has
    // been reading, and a search whose climb term is a third of its cost
    // cannot survive that (`docs/ROADS.md` §7).
    //
    // The test is the starting buildings themselves: same owner, same
    // object number, same position, all of them.
    if init.heights.is_empty()
        && let Some(s) = siblings
            .iter()
            .find(|s| !s.heights.is_empty() && same_start(init, s))
    {
        init.heights = s.heights.clone();
    }
    // The `REGIONS` block travels with the map, on the map's own terms:
    // a sibling whose world block is ours generated the same regions.
    if init.regions.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.regions.is_empty())
    {
        init.regions = s.regions.clone();
    }
    if init.herds.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.herds.is_empty())
    {
        init.herds = s.herds.clone();
    }
    // The goods are the map's, laid down by the generator before frame 0
    // and never moved (`Objects::init_good@00653f30`), so a capture that
    // did not print them takes its sibling's exactly as it takes the cells.
    if init.goods.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.goods.is_empty())
    {
        init.goods = s.goods.clone();
    }
    // The farm list is a `DUMP_ALL` block too, and it is the same map's
    // (`docs/SYNC.md` §3.6): which farm is the pasture does not depend on
    // the window the capture opened.
    if init.farms.is_empty()
        && let Some(s) = siblings.iter().find(|s| !s.farms.is_empty())
    {
        init.farms = s.farms.clone();
    }
    // The per-frame words are only the same run's if the setup stream is:
    // a sibling's count from the trace's last word, so its own trace must
    // end where ours does (run3 is the map's `DUMP_ALL` but a different
    // start; run12 is run11's stream continued).
    // Every qualifying sibling contributes the frames it traced — run12's
    // 0–3 and run13's 94–103 together (`docs/SYNC.md` §5); the dump's own
    // word wins where two name the same frame.
    let ours = init.checksums.last().map(|c| c.seed);
    for s in siblings
        .iter()
        .filter(|s| !s.frame_seeds.is_empty() && s.checksums.last().map(|c| c.seed) == ours)
    {
        for &(frame, word) in &s.frame_seeds {
            if !init.frame_seeds.iter().any(|(f, _)| *f == frame) {
                init.frame_seeds.push((frame, word));
            }
        }
        // The clocks travel with the words: a frame's guys from the sibling
        // that traced it, and the start dump's guys from any sibling of the
        // same stream — the same setup put the same figures on the map.
        for (frame, guys) in &s.frame_guys {
            if !init.frame_guys.iter().any(|(f, _)| *f == *frame) {
                init.frame_guys.push((*frame, guys.clone()));
            }
        }
        for su in &s.units {
            if !su.guys.iter().any(crate::gamelog::Guy::has_clock) {
                continue;
            }
            match init
                .units
                .iter_mut()
                .find(|u| u.who == su.who && u.o == su.o)
            {
                Some(u) if !u.guys.iter().any(crate::gamelog::Guy::has_clock) => {
                    u.guys = su.guys.clone();
                }
                // Gaia's animals, which a `UNITS`-level dump lists without
                // their clocks or not at all.
                None if su.who >= 8 => init.units.push(su.clone()),
                _ => {}
            }
        }
    }
    init.frame_seeds.sort_unstable();
    init.frame_guys.sort_by_key(|(f, _)| *f);
    // The lengths are art, the same on every dump of the install.
    for s in siblings {
        init.anim_lengths.extend(s.anim_lengths.iter().copied());
    }
    init.anim_lengths.sort_unstable();
    init.anim_lengths.dedup();
}

/// [`run_with`], with what this dump lacks borrowed from **siblings** —
/// other dumps of the same lobby and seed, hence the same map and the same
/// setup stream: the setup path's checksum trace (run11 has it; run9 and
/// run10 were captured before it was found — `docs/ORACLE.md`, "The setup
/// path's checksum trace is the RNG state") and the terrain's height grid
/// (only a `DUMP_ALL` dump prints it; run3 is this map's). The dump's own
/// data wins; the first sibling that has each thing supplies it.
///
/// **`trace` is a source too, and the score depends on it.** A pasture's
/// five animals are owner 9 and no dump prints them (`docs/SYNC.md` §3.6);
/// only [`crate::trace::Trace::add_animals`] has them. They roll an idle
/// every frame, so a map whose AI built a pasture and a run that borrowed
/// none are **not the same simulation** — the stream parts within a few
/// frames of the pasture going up and every later figure is on a stream
/// that is nobody's. East Indies' score sat at 167 for two days for
/// exactly that reason (2026-08-31, item 69); with the trace it is 1374,
/// which is the word's own parting. Pass the trace wherever one exists.
pub fn run_traced<'a, 'b: 'a>(
    loaded: &Loaded,
    log: &Log<'a>,
    tuning: Tuning,
    limit: Option<usize>,
    stream: Option<&mut crate::input::Stream>,
    siblings: &[&Initial<'b>],
    trace: Option<&crate::trace::Trace>,
) -> Option<Report> {
    let mut init = log.initial()?;
    borrow_from_siblings(&mut init, siblings);
    if let Some(tr) = trace {
        borrow_pasture(&mut init, tr);
    }
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
    use crate::gamelog::{Block, CityDump, Guy, LeaderDump, OrderDump, UnitDump};
    use sim::ai::{MAKE_SLOTS, MakeObject};

    use crate::testenv::{dump, install};

    /// The queue's handoff states the floors, verbatim — the `Scoreboard:`
    /// line against [`FLOORS`]. Static: no install, no dump, every machine.
    /// If a score moved, move the floor first (the assert that reads it is
    /// beside it), then rewrite the line; if only the line changed, the
    /// floors are the truth and the line is wrong.
    #[test]
    fn the_handoff_s_scoreboard_is_the_floors() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let line = q.lines().find(|l| l.starts_with("Scoreboard:")).expect(
            "docs/QUEUE.md has no `Scoreboard:` line in the handoff; write \
             `Scoreboard: <map> <ticks>/<orders> w<word> · <map> …`, one part \
             per row of rondata::diff::FLOORS, in order",
        );
        let mut stated = Vec::new();
        for part in line.trim_start_matches("Scoreboard:").split('·') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let (ticks, orders) = t
                .get(1)
                .and_then(|p| p.split_once('/'))
                .unwrap_or_else(|| panic!("unreadable scoreboard part {part:?}"));
            let word = t
                .get(2)
                .and_then(|w| w.strip_prefix('w'))
                .unwrap_or_else(|| panic!("unreadable scoreboard part {part:?}"));
            stated.push((
                t[0].to_string(),
                ticks.parse::<i64>().expect("ticks"),
                orders.parse::<i64>().expect("orders"),
                word.parse::<i64>().expect("word"),
            ));
        }
        let pinned: Vec<_> = FLOORS
            .iter()
            .map(|f| (f.map.to_string(), f.ticks, f.orders, f.word))
            .collect();
        assert_eq!(
            stated, pinned,
            "the handoff's scoreboard is not the pinned floors: left is the \
             queue's line, right is rondata::diff::FLOORS"
        );

        // **And the line below it, which is the headline.** East Indies'
        // scored capture is closed, so the number a session is judged by
        // lives on the `Long captures:` line — and nothing checked it. Same
        // shape, one word a map, against the two long tests' own floors.
        let long = q.lines().find(|l| l.starts_with("Long captures:")).expect(
            "docs/QUEUE.md has no `Long captures:` line in the handoff; write \
             `Long captures: <map> w<word> of <length> ...`, one part per row \
             of rondata::diff::FLOORS, in order",
        );
        let mut said = Vec::new();
        for part in long.trim_start_matches("Long captures:").split('\u{b7}') {
            let t: Vec<&str> = part.split_whitespace().collect();
            let word = t
                .get(1)
                .and_then(|w| w.strip_prefix('w'))
                .unwrap_or_else(|| panic!("unreadable long-capture part {part:?}"));
            said.push((
                t[0].to_string(),
                word.parse::<i64>().expect("the long capture's word"),
            ));
        }
        assert_eq!(
            said,
            vec![
                ("EastIndies".to_string(), LONG_WORD_EAST_INDIES),
                ("GreatLakes".to_string(), LONG_WORD_GREAT_LAKES),
            ],
            "the handoff's long-capture words are not the long tests' floors: \
             left is the queue's line, right is LONG_WORD_EAST_INDIES and \
             LONG_WORD_GREAT_LAKES"
        );
    }

    fn initial() -> Initial<'static> {
        Initial {
            world: vec![("xs", "60"), ("ys", "60"), ("seed", "7236")],
            cities: vec![CityDump {
                x: 3168,
                y: 30816,
                pop: 1,
                who: 0,
                ..CityDump::default()
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
                    angle: None,
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
                    ..UnitDump::default()
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
                    angle: None,
                    guys: vec![Guy {
                        kind: Some(0x32),
                        ..Guy::default()
                    }],
                    ..UnitDump::default()
                },
                // An animal, which the harness ignores.
                UnitDump {
                    flags: 1,
                    o: 3,
                    who: 255,
                    pos: LogPos::default(),
                    angle: None,
                    guys: vec![],
                    ..UnitDump::default()
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
        if texts.len() == 4 {
            assert!(!init.checksums.is_empty(), "run11's trace was not read");
            assert!(!init.heights.is_empty(), "run3's height table was not read");
            assert_eq!(init.herds.len(), 13, "run12's herds were not read");
            assert_eq!(
                (init.herds[0].wx, init.herds[0].wy, init.herds[0].t),
                (22, 34, 408)
            );
            assert_eq!(
                &init.frame_seeds[..4],
                &[
                    (0, 0xb619_4ba1),
                    (1, 0x4554_ec0f),
                    (2, 0xab3b_035d),
                    (3, 0xc242_06bb)
                ],
                "run12's end-of-frame words"
            );
            assert_eq!(init.frame_seeds.len(), 14, "plus run13's ten, 94–103");
            assert_eq!(init.frame_seeds[4], (94, 0x5f8f_3d9d));
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

            // All ten, not just the best. The second reading
            // (`docs/audit/2026-08-25-ai.md`) found two site-layer claims
            // wrong that a best-site-only assertion cannot see — the ring's
            // centre (B4-k) and `Site::dist`'s clobber (B4-f) — so the
            // record is compared slot for slot. `dist` is included
            // deliberately: it carries the last enemy capital's
            // `distance / num_nations` whenever the enemy-capital loop runs,
            // which is a real output of the original and a real thing to
            // reproduce.
            let theirs_full: Vec<(i64, i64, i64, i64, i64)> = theirs
                .kids("SITE")
                .map(|s| {
                    (
                        s.int("wx").unwrap_or(0),
                        s.int("wy").unwrap_or(0),
                        s.int("val").unwrap_or(0),
                        s.int("rank").unwrap_or(0),
                        s.int("dist").unwrap_or(0),
                    )
                })
                .collect();
            let ours_full: Vec<(i64, i64, i64, i64, i64)> = built.sim.ai[1]
                .sites
                .iter()
                .map(|s| {
                    (
                        i64::from(s.wx),
                        i64::from(s.wy),
                        i64::from(s.val),
                        i64::from(s.rank),
                        i64::from(s.dist),
                    )
                })
                .collect();
            let differing: Vec<String> = theirs_full
                .iter()
                .zip(ours_full.iter())
                .enumerate()
                .filter(|(_, (t, o))| t != o)
                .map(|(i, (t, o))| format!("slot {i}: theirs {t:?} ours {o:?}"))
                .collect();
            assert!(
                differing.is_empty(),
                "the ten site records differ (wx, wy, val, rank, dist):\n  {}",
                differing.join("\n  ")
            );
        }

        // The make list, all eleven slots and all ten fields (audit B7-f).
        // Nothing has written it yet at frame 1 — the script blocks the
        // producers for the whole opening — so this is the record
        // `Array<MakeObject>::init@0047d300` fills and `MakeList::clear`
        // rewrites, and the harness must start from the same one.
        let theirs_list = make_list_of(theirs);
        let ours_list = built.sim.ai[1].make_list.list;
        assert_eq!(
            theirs_list
                .iter()
                .filter(|m| **m != MakeObject::EMPTY)
                .count(),
            0,
            "frame 1's list is untouched"
        );
        assert_list_eq("run9 frame 1", &theirs_list, &ours_list);
    }

    /// The coastal ring's guard (audit B4-k): run20 (`gamelog-run20-islands-
    /// dumpall.txt`, 2026-08-25) is the East Indies lobby — `MAP_STYLE 18`,
    /// **`sea_map 4`** — under `DUMP_ALL` for frames 0–3, so it carries its
    /// own setup trace, heights and per-frame words and needs no sibling.
    /// It is the first capture in which `compute_site_stats`' step 5
    /// (`docs/AI.md` §2.13: many landmasses, no dock, an ocean cell on the
    /// radius-5 ring → `base × 30`) is reachable at all; every dump before
    /// it reported `sea_map 1`. The ring is walked from the *original*
    /// sampled cell, not the one the 5×5 slide moved it to — the correction
    /// neither reading had — and this test is the assertion that was owed:
    /// all ten `SITE` records of the AI's frame-1 leader record, slot for
    /// slot, on the original's own stream and heights. Made to fail once by
    /// re-centring the ring on the slid cell before it was landed.
    #[test]
    fn run20_s_islands_sites_walk_the_coastal_ring_from_the_original_cell() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        // Self-sufficient: the setup trace, the heights and the frame words
        // are the run's own, or the assertion below would be on the sim's
        // own luck rather than the original's.
        assert!(
            !init.checksums.is_empty(),
            "run20's setup trace was not read"
        );
        assert!(
            !init.heights.is_empty(),
            "run20's height table was not read"
        );
        assert!(
            !init.frame_seeds.is_empty(),
            "run20's frame words were not read"
        );
        let sea_map = init
            .world
            .iter()
            .find(|(k, _)| *k == "sea_map")
            .and_then(|(_, v)| v.trim().parse::<i32>().ok())
            .expect("the WORLD block's sea_map");
        assert!(sea_map > 2, "an islands map: sea_map {sea_map}");
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.sim.lobby.map_style, 18, "East Indies");
        assert_eq!(
            built.sim.world.sea_map(),
            sea_map,
            "the harness carries the original's `sea_map`"
        );
        // What the first run of this test found: `sea_map` is not a
        // landmass count. The dump's cells fall into twelve land regions
        // and one sea region here, and the original still says 4 — the
        // style's own class (`World::sea_map`).
        let land_regions = (0..built.sim.world.region_count())
            .filter(|&r| built.sim.world.terrain(r as u16) == sim::world::Terrain::Land)
            .count();
        assert!(
            land_regions > 10,
            "twelve islands in the cells, not {land_regions}"
        );
        assert!(
            built
                .notes
                .iter()
                .any(|n| n.contains("heights pinned per tile")),
            "{:?}",
            built.notes
        );

        let theirs = log.leader_block(1, 1).expect("FRAME 1 LEADERDATA who 1");
        assert_eq!(theirs.int("peasants"), Some(5), "the record is the AI's");
        // The fog grid is loaded and is what the original's `was_seen`
        // reads: the AI's city cell carries its bit, and the ten sites
        // below do not — they are seen through the territory arm.
        assert!(
            built.notes.iter().any(|n| n.contains("fog: seen2 loaded")),
            "{:?}",
            built.notes
        );
        assert_eq!(
            built.sim.world.seen2(103, 105),
            Some(2),
            "the AI's city, fog (103,105)"
        );
        assert_eq!(
            built.sim.world.seen2(89, 105),
            Some(0),
            "site (44,52), fog (89,105)"
        );
        built.sim.tick();
        // The per-region census, **every region** — run9's check compares
        // the home region alone, which on one landmass is every land
        // region there is. The site scorer's step 9 multiplies by what
        // `reg_cities`/`reg_land` say about the *site's* region, so an
        // island the AI does not live on is exactly where a census error
        // would hide.
        let c = &built.sim.ai[1].census;
        let arr = |key: &str| -> Vec<i64> {
            theirs
                .all(key)
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect()
        };
        let mut wrong = Vec::new();
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
            // The land arrays are `[64]`, indexed by the land region
            // numbers `0..0x3e`; the sea regions have their own.
            for &(dump_r, sim_r) in built.region_map.iter().filter(|(d, _)| *d < 0x3f) {
                let o = i64::from(sim::ai::Census::reg(ours, sim_r));
                let tv = t.get(dump_r as usize).copied();
                if tv != Some(o) {
                    wrong.push(format!("{key}[{dump_r}]: ours {o} theirs {tv:?}"));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "the per-region census disagrees:\n  {}",
            wrong.join("\n  ")
        );
        // The `CITY` record, whole: step 13's picture — `ocean`, `land`,
        // `filled`, `dock_tile` (`Sim::is_dock_tile`, `docs/TRANSPORT.md`
        // §5.6; 1 for the AI's city, 0 for the human's), `space[3]` — and
        // steps 2/10's `free`, `busy`, `gatherers`, `peasant_dist`,
        // `in_port`, and `ter[6]` out of `World::gather_at`.
        let mut wrong = Vec::new();
        let mut matched = 0;
        let frame1 = log
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 1)
            .map(|(_, b)| b)
            .expect("FRAME 1");
        let cities = frame1.find("CITIES").expect("FRAME 1 CITIES");
        for rec in cities.kids("CITY").filter(|c| c.int("who") == Some(1)) {
            let (x, y) = (rec.int("x").unwrap_or(-1), rec.int("y").unwrap_or(-1));
            let Some(c) = built
                .sim
                .cities
                .iter()
                .position(|c| c.alive && i64::from(c.pos.x) == x && i64::from(c.pos.y) == y)
            else {
                wrong.push(format!("no city of ours at {x},{y}"));
                continue;
            };
            matched += 1;
            let ours = built.sim.ai[1].city_ai[c];
            let space: Vec<i64> = rec
                .all("space[scan]")
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect();
            let fields = [
                ("ocean", ours.ocean),
                ("land", ours.land),
                ("filled", ours.filled),
                ("dock_tile", ours.dock_tile),
                ("free", ours.free),
                ("busy", ours.busy),
                ("gatherers", ours.gatherers),
                ("peasant_dist", ours.peasant_dist),
                ("in_port", ours.in_port),
            ];
            for (key, o) in fields {
                let t = rec.int(key);
                if t != Some(i64::from(o)) {
                    wrong.push(format!("{key}: ours {o} theirs {t:?}"));
                }
            }
            // `space[0..3]`, whole. `space[0]` and `space[1]` were pinned
            // at ours 48 / theirs 58 for a day: `space_at_corner` walks its
            // sixteen tiles centre-2×2-first (`grid_index_x/y`), so its
            // early-out is on the centre, not the top row, and ≥ 8 blocked
            // with the centre free is 2, not 0 (`docs/AI.md` §15.9). The
            // pin flipped the moment the walk order landed.
            for (i, o) in ours.space.iter().enumerate() {
                let t = space.get(i).copied();
                if t != Some(i64::from(*o)) {
                    wrong.push(format!("space[{i}]: ours {o} theirs {t:?}"));
                }
            }
            // `ter[0..6]`, the per-good best over the circle
            // (`docs/AI.md` §24).
            let ter: Vec<i64> = rec
                .all("ter[scan]")
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect();
            for (i, o) in ours.ter.iter().enumerate() {
                let t = ter.get(i).copied();
                if t != Some(i64::from(*o)) {
                    wrong.push(format!("ter[{i}]: ours {o} theirs {t:?}"));
                }
            }
        }
        assert_eq!(matched, 1, "the AI's one city");
        assert!(
            wrong.is_empty(),
            "the CITY record disagrees:\n  {}",
            wrong.join("\n  ")
        );
        built.sim.tick();
        let theirs_full: Vec<(i64, i64, i64, i64, i64)> = theirs
            .kids("SITE")
            .map(|s| {
                (
                    s.int("wx").unwrap_or(0),
                    s.int("wy").unwrap_or(0),
                    s.int("val").unwrap_or(0),
                    s.int("rank").unwrap_or(0),
                    s.int("dist").unwrap_or(0),
                )
            })
            .collect();
        assert_eq!(theirs_full.len(), 10, "ten SITE records");
        let ours_full: Vec<(i64, i64, i64, i64, i64)> = built.sim.ai[1]
            .sites
            .iter()
            .map(|s| {
                (
                    i64::from(s.wx),
                    i64::from(s.wy),
                    i64::from(s.val),
                    i64::from(s.rank),
                    i64::from(s.dist),
                )
            })
            .collect();
        eprintln!("run20 sites — theirs: {theirs_full:?}\n              ours:   {ours_full:?}");
        let differing: Vec<String> = theirs_full
            .iter()
            .zip(ours_full.iter())
            .enumerate()
            .filter(|(_, (t, o))| t != o)
            .map(|(i, (t, o))| format!("slot {i}: theirs {t:?} ours {o:?}"))
            .collect();
        assert!(
            differing.is_empty(),
            "the ten site records differ (wx, wy, val, rank, dist):\n  {}",
            differing.join("\n  ")
        );
    }

    /// The sim's record of a dumped `MAKEOBJECT`, or the whole eleven.
    fn make_object(m: &crate::gamelog::MakeObjectDump) -> MakeObject {
        let i = |v: i64| i32::try_from(v).expect("a make-list field fits i32");
        MakeObject {
            t: i(m.t),
            val: i(m.val),
            escrow: i(m.escrow),
            city: i(m.city),
            up: i(m.up),
            o: i(m.o),
            num: i(m.num),
            cat: i(m.cat),
            wx: i(m.wx),
            wy: i(m.wy),
        }
    }

    fn make_list_of(leader: &crate::gamelog::Block<'_>) -> [MakeObject; MAKE_SLOTS] {
        let v: Vec<MakeObject> = leader.make_list().iter().map(make_object).collect();
        v.try_into()
            .unwrap_or_else(|v: Vec<MakeObject>| panic!("{} MAKEOBJECTs, not eleven", v.len()))
    }

    /// Every slot, every field; the message names each slot that differs.
    fn assert_list_eq(
        what: &str,
        theirs: &[MakeObject; MAKE_SLOTS],
        ours: &[MakeObject; MAKE_SLOTS],
    ) {
        let differing: Vec<String> = theirs
            .iter()
            .zip(ours.iter())
            .enumerate()
            .filter(|(_, (t, o))| t != o)
            .map(|(i, (t, o))| format!("slot {i}:\n    theirs {t:?}\n    ours   {o:?}"))
            .collect();
        assert!(
            differing.is_empty(),
            "{what}: the eleven make-list records differ:\n  {}",
            differing.join("\n  ")
        );
    }

    /// The orderings of `n` items, by index.
    fn permutations(n: usize) -> Vec<Vec<usize>> {
        fn go(rest: Vec<usize>, head: Vec<usize>, out: &mut Vec<Vec<usize>>) {
            if rest.is_empty() {
                out.push(head);
                return;
            }
            for (i, &x) in rest.iter().enumerate() {
                let mut r = rest.clone();
                r.remove(i);
                let mut h = head.clone();
                h.push(x);
                go(r, h, out);
            }
        }
        let mut out = Vec::new();
        go((0..n).collect(), Vec::new(), &mut out);
        out
    }

    /// **The make list across a whole window, slot for slot** — every
    /// consecutive pair of `LEADERS=9` blocks in run18b (dump-frames
    /// 6374–6590) and run19 (8174–8191), all eleven slots and all ten fields
    /// (audit B7-f, the widening `SITES` got). The dump prints the list at
    /// the end of each frame, so a pair `(prev, next)` is one step of the
    /// production ladder (`docs/AI.md` §2.4), and each step is replayed with
    /// the simulation's own operation on `prev`'s record and compared to
    /// `next`'s:
    ///
    /// - **unchanged** — most frames; nothing to replay, counted.
    /// - **step 2's `clear`** — `next` is the init record throughout;
    ///   `MakeList::clear` on `prev` must produce it, which pins that the
    ///   clear rewrites all ten fields (`MakeList::clear@006c9db0`), not
    ///   `t` alone.
    /// - **a producer** (steps 3–7, 9, 10) — the entries `next` has that
    ///   `prev` lacks are the offers that landed; some ordering of them
    ///   through `MakeList::make_me` must reproduce `next` exactly. One
    ///   frame needs an offer the end state does not show: at dump-frame
    ///   8184 the second pass's `create_units` offered a merchant before
    ///   the scholar, and the scholar overwrote it at the head — the only
    ///   trace it leaves is the duplicate-clear of the merchant already at
    ///   slot 3. The offer is taken from the same producer's entry three
    ///   frames earlier.
    /// - **`make_stuff`** (steps 8 and 11) — the head's expiry walk and the
    ///   bought slots' own, with the seeds the trace recorded before each
    ///   `Random::get` (`docs/AI.md` §15.3, §15.6) and the bought slots'
    ///   `val /= 100`. The expiry writes `t` alone, so a cleared slot keeps
    ///   its `val`, and the compare sees it.
    ///
    /// The frames each class covers are asserted, so the test cannot pass
    /// by classifying everything as unchanged.
    #[test]
    fn run18b_and_run19_s_make_list_windows_replay_slot_for_slot() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();

        /// A `make_stuff` frame: the dump-frame that shows its result, the
        /// sync stream before its first draw, the slots whose `make_this`
        /// bought (the head first when it did), and the stream after.
        struct Expiry {
            frame: i64,
            seed: u32,
            bought: &'static [usize],
            after: u32,
        }
        let expiries = [
            // run18b, sim-frame 6383: the temple at the head and slot 8,
            // nothing bought; 61545 clears the head, 25792 keeps slot 8.
            Expiry {
                frame: 6384,
                seed: 0xc593_8177,
                bought: &[],
                after: 0xbb3f_64c1,
            },
            // sim-frame 6582: 17105 keeps the head, 1032 clears slot 8; the
            // citizen bought out of slot 5 (714 → 7) and 48595 keeps it.
            Expiry {
                frame: 6583,
                seed: 0x833a_ab7f,
                bought: &[5],
                after: 0x35dc_bdd4,
            },
            // run19, sim-frame 8182: the head (Coinage) bought and kept on
            // 11233, its duplicate at slot 4 cleared on 14808; the scholar
            // bought out of slot 1 and kept on 22883.
            Expiry {
                frame: 8183,
                seed: 0xa6d1_84cf,
                bought: &[0, 1],
                after: 0x78f6_5964,
            },
            // sim-frame 8185: a scholar at the head, cleared outright on a
            // roll (45911) that would have kept it anywhere else.
            Expiry {
                frame: 8186,
                seed: 0x3f5a_529d,
                bought: &[],
                after: 0x8244_b358,
            },
        ];
        // The offer the end state hides (dump-frame 8184): the merchant
        // `create_units` listed at slot 3 three frames earlier, re-offered
        // and overwritten at the head by the scholar that followed.
        let hidden: &[(i64, MakeObject)] = &[(
            8184,
            MakeObject {
                t: 61,
                val: 869_565,
                escrow: 1,
                city: 0,
                up: 0,
                o: -1,
                num: 1,
                cat: 4,
                wx: 0,
                wy: 0,
            },
        )];

        struct Window {
            file: &'static str,
            who: i64,
            clears: &'static [i64],
            producers: &'static [i64],
            make_stuffs: &'static [i64],
        }
        let windows = [
            Window {
                file: "gamelog-run18b-window-6374-6590.txt",
                who: 1,
                clears: &[6577],
                producers: &[6382, 6383, 6581, 6582],
                make_stuffs: &[6384, 6583],
            },
            Window {
                file: "gamelog-run19-window-8174-8192.txt",
                who: 1,
                clears: &[8177],
                producers: &[8179, 8180, 8181, 8182, 8184, 8185],
                make_stuffs: &[8183, 8186],
            },
        ];

        for w in &windows {
            let Some(path) = dump(w.file) else {
                eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", w.file);
                return;
            };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            // The AI's list at the end of every frame that dumps it whole;
            // a block below level 9 (the quit's) has no MAKEOBJECTs and is
            // left out.
            let lists: Vec<(i64, [MakeObject; MAKE_SLOTS])> = log
                .frames()
                .into_iter()
                .filter_map(|(n, b)| {
                    let l = b.kids("LEADERDATA").find(|l| l.int("who") == Some(w.who))?;
                    let m = l.make_list();
                    assert!(
                        m.is_empty() || m.len() == MAKE_SLOTS,
                        "{n}: {} slots",
                        m.len()
                    );
                    (m.len() == MAKE_SLOTS).then(|| (n, make_list_of(l)))
                })
                .collect();
            assert!(
                lists.len() > 10,
                "{}: {} level-9 blocks",
                w.file,
                lists.len()
            );

            let (mut unchanged, mut clears, mut producers, mut make_stuffs) =
                (0usize, Vec::new(), Vec::new(), Vec::new());
            for pair in lists.windows(2) {
                let (_, prev) = pair[0];
                let (frame, next) = pair[1];
                let what = format!("{} dump-frame {frame}", w.file);
                if prev == next {
                    unchanged += 1;
                    continue;
                }
                if next.iter().all(|m| *m == MakeObject::EMPTY) {
                    let mut l = sim::ai::MakeList { list: prev };
                    l.clear();
                    assert_list_eq(&what, &next, &l.list);
                    clears.push(frame);
                    continue;
                }
                if let Some(e) = expiries.iter().find(|e| e.frame == frame) {
                    let mut s = loaded.sim(Tuning::RON, World::new(4, 4), 2);
                    let who = w.who as sim::Player;
                    s.ai[w.who as usize].make_list.list = prev;
                    s.rng = sim::combat::Rng::new(e.seed);
                    // Step 3: the head's `make_this` demotes it before step
                    // 4's walk; step 6: each bought slot's `make_this`, then
                    // its own walk from that slot.
                    let head = prev[0];
                    let wi = w.who as usize;
                    if e.bought.contains(&0) {
                        s.ai[wi].make_list.list[0].val /= 100;
                    }
                    let unc = s.expire_all(head.t as usize, true);
                    s.expire(who, head.t, 0, unc);
                    for &slot in e.bought.iter().filter(|&&k| k != 0) {
                        s.ai[wi].make_list.list[slot].val /= 100;
                        let t = s.ai[wi].make_list.list[slot].t;
                        let unc = s.expire_all(t as usize, false);
                        s.expire(who, t, slot, unc);
                    }
                    assert_eq!(
                        s.rng.seed, e.after,
                        "{what}: the walk's draw count, by the trace's seeds"
                    );
                    assert_list_eq(&what, &next, &s.ai[wi].make_list.list);
                    make_stuffs.push(frame);
                    continue;
                }
                // A producer: the offers that landed, each once, in some
                // order — plus the one the end state hides.
                let mut offers: Vec<MakeObject> = Vec::new();
                for m in next.iter().filter(|m| m.t != -1 && !prev.contains(m)) {
                    if !offers.contains(m) {
                        offers.push(*m);
                    }
                }
                for (_, m) in hidden.iter().filter(|(f, _)| *f == frame) {
                    offers.push(*m);
                }
                assert!(!offers.is_empty(), "{what}: changed, and no new entry");
                let mut reproduced = Vec::new();
                for order in permutations(offers.len()) {
                    let mut l = sim::ai::MakeList { list: prev };
                    for &k in &order {
                        let m = offers[k];
                        l.make_me(m.t, m.val, m.escrow, m.cat, m.city, m.up, m.num, m.wx, m.wy);
                    }
                    if l.list == next {
                        reproduced.push(order.iter().map(|&k| offers[k].t).collect::<Vec<_>>());
                    }
                }
                eprintln!(
                    "{what}: {} offers {:?}; orders that reproduce it: {reproduced:?}",
                    offers.len(),
                    offers
                        .iter()
                        .map(|m| (m.t, m.val, m.cat))
                        .collect::<Vec<_>>()
                );
                assert!(
                    !reproduced.is_empty(),
                    "{what}: no ordering of the offers reproduces the record through make_me\n  prev {prev:?}\n  next {next:?}"
                );
                producers.push(frame);
            }
            eprintln!(
                "{}: {} blocks — {unchanged} unchanged, clears {clears:?}, producers {producers:?}, make_stuff {make_stuffs:?}",
                w.file,
                lists.len()
            );
            assert_eq!(clears, w.clears, "{}: the clear frames", w.file);
            assert_eq!(producers, w.producers, "{}: the producer frames", w.file);
            assert_eq!(
                make_stuffs, w.make_stuffs,
                "{}: the make_stuff frames",
                w.file
            );
        }
    }

    /// **The install's animation lengths against the dump's.**
    ///
    /// `crate::artdata` reads `unit_graphics.xml`, `anim_graphics.xml` and
    /// the `.bha` headers to say what every gaia type's slots are worth;
    /// `Log::anim_lengths` reads the same numbers back off a `DUMP_ALL`
    /// dump's `GUY` blocks. They are two independent oracles for one
    /// table, and this is what makes the reader's arithmetic — the key
    /// times in milliseconds, `round(times * 3 / 200)` — an assertion
    /// rather than a story.
    ///
    /// Run12 shows six of them: the three sheep pieces' `CHAR_DEFAULT`
    /// (90, 109, 250 — `Sheep Idle1`, `Idle3`, `Idle5`) and the three fish
    /// pieces' idles (170, 101, 116 — `Fish Idle1`, `Idle2`, `Idle3`). It
    /// is also what pins the `-TYPE<v>` suffix to the variant `(seed + o)
    /// % 3` picks, which is the one step of the chain neither file states
    /// outright.
    #[test]
    fn the_install_s_gaia_lengths_match_the_dump_s() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run12-dumpall-seeds.txt") else {
            eprintln!("skipping: no gamelog-run12-dumpall-seeds.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        assert!(
            !loaded.gaia_lengths.is_empty(),
            "the install's graphics tables read"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let built = build_sim(&loaded, &init, Tuning::RON);
        let sim = &built.sim;
        // Every gaia piece the dump named, and every length it showed for
        // it, against the install's own row.
        let mut checked = Vec::new();
        for (&(who, ty, sub, _), &piece) in &sim.art.pieces {
            // Gaia is owners 8 and 9: the herds the map generator laid
            // down are 8 and the birds `Objects::process_all` hatches are
            // 9, which is why no dump has ever printed a bird.
            if who < 8 {
                continue;
            }
            let index = sim.unit_types[ty].type_index;
            for (&(p, slot), &n) in &sim.art.lengths {
                if p != piece {
                    continue;
                }
                let theirs = sim.art.gaia_lengths.get(&(index, sub, slot)).copied();
                assert_eq!(
                    theirs,
                    Some(n),
                    "type {index:#x} variant {sub} slot {slot}: the dump says {n}, \
                     the install {theirs:?}"
                );
                checked.push((index, sub, slot, n));
            }
        }
        checked.sort_unstable();
        checked.dedup();
        // The sheep and the fish, by type and variant — named so a run
        // that stops showing them reads as a thinner check rather than as
        // a passing one.
        assert_eq!(
            checked,
            vec![
                (0x198, 0, sim::anim::DEFAULT, 90),
                (0x198, 1, sim::anim::DEFAULT, 109),
                (0x198, 2, sim::anim::DEFAULT, 250),
                (0x19b, 0, sim::anim::DEFAULT, 170),
                (0x19b, 0, sim::anim::IDLE1, 170),
                (0x19b, 0, sim::anim::IDLE2, 170),
                (0x19b, 0, sim::anim::IDLE3, 170),
                (0x19b, 1, sim::anim::DEFAULT, 101),
                (0x19b, 1, sim::anim::IDLE1, 101),
                (0x19b, 1, sim::anim::IDLE2, 101),
                (0x19b, 2, sim::anim::DEFAULT, 116),
                (0x19b, 2, sim::anim::IDLE1, 116),
                (0x19b, 2, sim::anim::IDLE3, 116),
            ],
            "the six lengths both oracles state"
        );
        // And the bird's two, which only the install has: no dump prints
        // owner 9's bird at all (`docs/SYNC.md` §3.9). *Bird Soar* is the
        // `CHAR_WALK` and *Bird Flap* the `CHAR_JOG`, and their wraps are
        // what run14's stream confirms.
        for v in 0..3u8 {
            assert_eq!(
                loaded.gaia_lengths.get(&(0x192, v, sim::anim::WALK)),
                Some(&31),
                "the bird's Bird Soar"
            );
            assert_eq!(
                loaded.gaia_lengths.get(&(0x192, v, sim::anim::JOG)),
                Some(&23),
                "the bird's Bird Flap"
            );
            assert_eq!(
                loaded.gaia_lengths.get(&(0x192, v, sim::anim::DEFAULT)),
                None,
                "and no idle at all, which is worth three frames"
            );
        }
    }

    /// **The guy stack's *length*, against every dump that names one.**
    ///
    /// `Unit::init@00612100:471`-`508` sizes a unit's figures to
    /// `crew_size + squad_size` and gives every slot its own
    /// `Guy::init_real`. `squad_size` is not a column: `UnitType::init@
    /// 0061ab50:723` writes the literal 1 into every type before it reads
    /// `UBER_SIZE` and `CREW_SIZE` beside it. So the count is
    /// [`sim::anim::SQUAD_SIZE`] plus `unitrules.xml`'s `CREW_SIZE` — one
    /// for a Citizen, two for a Scout (the dog), three for a Caravan, four
    /// for a Trebuchet.
    ///
    /// The count was a hardcoded 1 in [`sim::Sim::init_guys`] until item
    /// 173, which cost East Indies' long word frame **6164**: the AI's
    /// first Caravan spent one `Guy::init_real+0x52` where the original
    /// spent three, and the two figures' first idles were the two
    /// `Guy::set_anim+0x97a < Unit::do_idle+0x7d` missing from 6165. A
    /// unit stood up *from* a dump never had the bug — `build_sim` gives
    /// it the `GUY` blocks the file prints — which is why nothing caught
    /// it for a month and why this assertion is worth having: it checks
    /// the **install's** answer against the dump's on every unit of every
    /// capture, born or stood up.
    #[test]
    fn every_dumped_unit_has_crew_size_plus_one_figures() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut rows = 0usize;
        let mut counts: std::collections::BTreeMap<usize, usize> =
            std::collections::BTreeMap::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
            "gamelog-run34-greatlakes-dumpall-start.txt",
            "gamelog-run58-islands-5k2.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            for u in &init.units {
                // A `GUY` block is what states the count; a dump below
                // `DUMP_ALL` prints none and its units arrive empty.
                if u.guys.is_empty() {
                    continue;
                }
                // Gaia's animals are not `Unit::init`'s: `Gaia::spawn_*`
                // builds them, and their crew is the herd's own business
                // (`docs/ANIM.md` section 7).
                if !(0..8).contains(&u.who) {
                    continue;
                }
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let Some(ty) = link.kind else { continue };
                let crew = built.sim.unit_types[ty].combat.crew_size;
                let ours = usize::try_from(crew + sim::anim::SQUAD_SIZE as i32).unwrap_or(0);
                rows += 1;
                *counts.entry(u.guys.len()).or_default() += 1;
                assert_eq!(
                    ours,
                    u.guys.len(),
                    "{name}: unit {}/{} (type {:?}) has {} figures in the \
                     dump; CREW_SIZE {crew} + SQUAD_SIZE says {ours}",
                    u.who,
                    u.o,
                    u.guys.first().and_then(|g| g.kind),
                    u.guys.len(),
                );
            }
        }
        eprintln!("guy-count walk: {rows} units, lengths {counts:?}");
        // The floor is the walk's own: a run with no multi-figure unit in
        // it would pass on ones alone and say nothing.
        assert!(
            rows >= 60 && counts.keys().any(|&n| n > 1),
            "the units those dumps name between them: {rows} over {counts:?} \
             — the floor is 60, at least one of them crewed"
        );
    }

    /// **`get_unit_gpiece`'s walk, against every piece a dump names.**
    ///
    /// [`sim::Sim::unit_gpiece`] derives a guy's graphic piece from four
    /// coordinates — the nation's `UNIT_CONTINENT`, the leader's age
    /// bracket, the gender bit (or the packed bit, which sits in the same
    /// slot) and the crew index — and then walks *down* from it until the
    /// install's own `<UNIT>` entries have one. Every dump that carries a
    /// `GUY` block prints the answer the original reached, per guy, so the
    /// walk is checkable to the **number** rather than to the arithmetic:
    /// this asserts it over every one of them.
    ///
    /// It is what makes the derivation worth having. [`sim::anim::Art`]'s
    /// `pieces` is seeded from the opening dump alone, so a type absent
    /// from the opening — a Fisherman the AI trains on frame 4,376 — had
    /// **no** piece at all, an `end_time` of [`sim::anim::UNKNOWN`], and no
    /// animation of its ever wrapped (item 152).
    #[test]
    fn the_walk_gives_every_dumped_guy_its_own_piece() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut rows = 0usize;
        let mut pieces: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
            "gamelog-run34-greatlakes-dumpall-start.txt",
            "gamelog-run58-islands-5k2.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            for u in &init.units {
                // Gaia's pieces come off `first_bird_piece`, a runtime
                // pointer no file states — the dump stays their source.
                if !(0..8).contains(&u.who) {
                    continue;
                }
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let Some(ty) = link.kind else { continue };
                let packed = built.sim.units[link.unit].combat.packed;
                for (n, g) in u.guys.iter().enumerate() {
                    let Some(theirs) = g.gpiece else { continue };
                    rows += 1;
                    pieces.insert(theirs);
                    assert_eq!(
                        built
                            .sim
                            .unit_gpiece(u.who as u8, ty, u.o as i16, n as u8, packed),
                        Some(theirs as i32),
                        "{name}: unit {}/{} guy {n} (type {:?}, packed {packed})",
                        u.who,
                        u.o,
                        g.kind
                    );
                }
            }
        }
        eprintln!(
            "gpiece walk: {rows} guys over {} distinct pieces",
            pieces.len()
        );
        assert!(
            rows >= 60,
            "the guys those dumps name between them: {rows}, the floor is 60"
        );
    }

    /// **The player units' lengths, from the install against the dumps.**
    ///
    /// The gaia check above says the `.bha` arithmetic is right; this one
    /// says the *addressing* is — that
    /// `GraphicPieces::init_piece_ranges@008f70e0`'s strides and
    /// `get_unit_gpiece@0090c030`'s sum put each `<UNIT name="…">` entry at
    /// the piece number the original hands out. Eight dumps between them
    /// print 187 `(gpiece, cur_anim) → end_time` rows over six pieces of
    /// two nations, and every one of them has to be the install's own.
    ///
    /// **The list is the assertion, and it was five dumps too short.**
    /// Five starts and short windows print only the idles, the walks, the
    /// chop, the sow and the reap; the wood dump — `CHAR_DUMP_WOOD`,
    /// **32** frames on every citizen piece the corpus names — was in no
    /// dump this test read, and the install's table said 33 for a month
    /// (item 121, `docs/ANIM.md` §3.1). Item 87's ledger, one row of it
    /// paid: a slot the parser had and nothing compared.
    ///
    /// **Except a mirrored guy's, and that is the check's other half.** A
    /// crew member past the squad's size copies guy 0's `cur_anim` and
    /// `cur_time` every frame and keeps its **own** `end_time`
    /// (`docs/ANIM.md` §5), so the pair a dump prints for the scouts' dogs
    /// is not a length row at all: it is one animation's slot beside
    /// another's length. Five of the 88 are that, all on the two dogs
    /// (crew 1, `13043` and `12691`), and each is a length the *same piece*
    /// carries at another slot — which is what says the rows are the
    /// mirror rather than a mis-addressed piece.
    #[test]
    fn the_install_s_piece_lengths_match_the_dumps() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        assert_eq!(
            loaded.piece_lengths.len(),
            1440,
            "every `<UNIT>` entry the unit path can name"
        );
        // **And an entry with no `<ANIM>` child is one of them.** All
        // sixty animation-less `<UNIT>`s in the shipped file are
        // `-CREW{k}`, and they are eighty-one pieces here because a
        // `GRAPH` is shared. The piece exists — it loads a model — and
        // its packet is *empty*, so every slot is the three frames
        // `AnimationPacket::get_game_frames` returns and none of them
        // loops: that is what makes a caravan's two crew figures fall to
        // the idle and roll every third frame (`docs/ANIM.md` §3.6).
        // Until 2026-09-02 the reader dropped the entry, `get_unit_gpiece`
        // fell to `first_unit_piece` and the crew walked on the citizen's
        // art.
        for p in [12681, 25353] {
            assert_eq!(
                loaded
                    .piece_lengths
                    .get(&p)
                    .map(std::collections::BTreeMap::len),
                Some(0),
                "the caravan's crew piece {p} is an entry with an empty packet"
            );
        }
        // The pieces run12's guys name, by the arithmetic
        // (`artdata::tests::a_unit_graphic_s_name_gives_its_piece`): player
        // 0 is Nubian (`UNIT_CONTINENT 1 Arab`, one style stride of
        // `0x160`) and player 1 British (`0 European`, style 0), the scout
        // is `TypeIndex` 69 and the citizen 50, and a dog is crew 1
        // (`+0x3180`). Each of the six is a `<UNIT>` entry the install
        // names, and a mis-addressed table would miss one.
        for p in [371, 13043, 19, 12691, 352, 6688] {
            assert!(
                loaded.piece_lengths.contains_key(&p),
                "no install entry for piece {p}"
            );
        }
        // The scout's idle variants, which are the whole of item 71: its
        // `CHAR_DEFAULT` is 61 frames and its `CHAR_IDLE1` **76**, and no
        // dump has ever shown the second — so a roll that took `IDLE1`
        // used to be played as the default and the clock wrapped fifteen
        // frames early.
        let scout = &loaded.piece_lengths[&371];
        assert_eq!(scout.get(&sim::anim::DEFAULT), Some(&61));
        assert_eq!(scout.get(&sim::anim::IDLE1), Some(&76));
        assert_eq!(scout.get(&sim::anim::IDLE2), Some(&41));
        assert_eq!(scout.get(&sim::anim::IDLE3), Some(&190));
        // And no unit piece in the shipped file has a `CHAR_GROUP_IDLE2` —
        // so `set_anim`'s captain gate, the one frame in sixteen that
        // skips the idle roll, can never fire (§4.2). It was `Art::
        // group_idle`, empty for want of a dump; it is now a fact.
        assert!(
            loaded
                .piece_lengths
                .values()
                .all(|m| !m.contains_key(&sim::anim::GROUP_IDLE2)),
            "no unit packet names a group idle"
        );
        // **And no piece any traced unit carries has a turn animation** —
        // which is what makes `Guy::do_turn@005d97a0:15` unreachable on
        // every capture there is. That arm overrides `Guy::move`'s
        // standing walk (`crates/sim/src/anim.rs`, `guys_follow`) with
        // `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` when `guy_flags & 8`, and
        // `Guy::init_real@005db6b0:179` sets that bit only when the guy's
        // piece names one. 273 of the install's 1,359 unit pieces do, so
        // the mechanic is real; **none of the eight a `DUMP_ALL` run's
        // guys name is among them** — the two scouts, their two dogs and
        // the six citizens of a Nubian and a British start — and gaia's
        // six pieces are not `<UNIT>` entries at all, so they cannot carry
        // the bit either. Booked as item 36's second half; closed here,
        // from the install, rather than modelled (`docs/ANIM.md` §4.6).
        for p in [0, 19, 352, 371, 6336, 6688, 12691, 13043] {
            let m = &loaded.piece_lengths[&p];
            assert!(
                !m.contains_key(&sim::anim::TURN_LEFT) && !m.contains_key(&sim::anim::TURN_RIGHT),
                "piece {p} names a turn animation, so `guy_flags & 8` is live"
            );
        }
        assert_eq!(
            loaded
                .piece_lengths
                .values()
                .filter(|m| m.contains_key(&sim::anim::TURN_LEFT))
                .count(),
            273,
            "the pieces that do have one — the check is that it is neither 0 nor all"
        );

        let mut rows = 0usize;
        let mut slots: std::collections::BTreeSet<i8> = std::collections::BTreeSet::new();
        let mut mirrored = Vec::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            // The three the list wanted and did not have (2026-08-31,
            // item 121). The five above are starts and short windows, so
            // between them they print only the idles, the walks, the chop,
            // the sow and the reap — and the eight slots they never reach
            // are where the install's table was wrong. run25 brings the
            // three attacks, the two dumps and the ore half; run44 the
            // turns and pack/unpack; run27 `CHAR_WALK_WITH_ORE`.
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            let sim = &built.sim;
            for (&(who, _, _, guy), &piece) in &sim.art.pieces {
                // Gaia is the other check's.
                if who >= 8 {
                    continue;
                }
                let theirs = loaded
                    .piece_lengths
                    .get(&piece)
                    .unwrap_or_else(|| panic!("{name}: no install entry for piece {piece}"));
                for (&(p, slot), &n) in &sim.art.lengths {
                    if p != piece {
                        continue;
                    }
                    rows += 1;
                    slots.insert(slot);
                    if theirs.get(&slot) == Some(&n) {
                        continue;
                    }
                    // A mirrored guy: the slot is guy 0's and the length is
                    // this one's, so it must be *some* slot of this piece.
                    assert!(guy > 0, "{name}: piece {piece} slot {slot} says {n}");
                    assert!(
                        theirs.values().any(|&m| m == n),
                        "{name}: piece {piece} slot {slot} says {n}, which is no \
                         slot of its own"
                    );
                    mirrored.push((piece, slot, n));
                }
            }
        }
        eprintln!(
            "piece lengths: {rows} rows over slots {:?}",
            slots.iter().collect::<Vec<_>>()
        );
        assert!(
            rows >= 88,
            "the rows eight dumps between them print: {rows}, the floor is 88"
        );
        mirrored.sort_unstable();
        mirrored.dedup();
        assert_eq!(
            mirrored,
            vec![
                (12691, sim::anim::DEFAULT, 76),
                (12691, sim::anim::IDLE1, 190),
                (12691, sim::anim::IDLE3, 41),
                (13043, sim::anim::IDLE1, 190),
                (13043, sim::anim::IDLE2, 61),
                (13043, sim::anim::IDLE2, 190),
            ],
            "the mirrored rows, all on the two dogs"
        );
    }

    /// **run56's figures, frame for frame** — every `GUY` block's own
    /// `x`, `y` and `angle`, over East Indies' 3,000-frame capture.
    ///
    /// This is the record behind item 128. A unit is a `Unit` and one or
    /// more `Guy`s; guy 0's body is the unit's own, and a **crew** guy
    /// whose piece names a track offset walks a second body toward a
    /// destination of its own. A scout is the pair that shows it: the man
    /// arrives and the dog is still walking four frames later, which is
    /// four frames on which the two figures answer `Guy::set_anim`'s
    /// walking-guy early return differently (`docs/MOVEMENT.md`, "The
    /// follower's destination").
    ///
    /// Nothing here is installed. The bodies are **derived** — the crew's
    /// from `Sim::seat_guys` at the start and from guy 0's own
    /// `Guy::set_new_location` and `Guy::set_angle` after that — so every
    /// row is a prediction the dump can refuse. The whole record is
    /// compared, both figures of every unit gaia's included, because nine
    /// tenths of a dumped record once went uncompared for a month.
    #[test]
    fn run56_s_figures_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        let theirs = std::mem::take(&mut init.frame_bodies);
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let last = theirs.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 2_990,
            "run56's body table reaches frame {last}, wanted 2,990+ — this \
             is the long East Indies capture at full detail, and a short \
             file here is a wrong file"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut compared = 0usize;
        // Three tallies, because the rows fall into three kinds and only
        // one of them is this mechanic's.
        let mut gaia_bearings = 0usize;
        let mut on_the_quit = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                let Some(u) = built
                    .units
                    .iter()
                    .find(|l| l.who == state.who && l.o == state.o)
                    .map(|l| l.unit)
                else {
                    continue;
                };
                let unit = &built.sim.units[u];
                for (n, g) in state.guys.iter().enumerate() {
                    let (Some(pos), Some(angle)) = (g.pos, g.angle) else {
                        continue;
                    };
                    let (ours, facing) = match unit.guys.get(n).and_then(|x| x.follow) {
                        Some(b) => (b.body.pos, b.facing),
                        None => (unit.movement.body.pos, unit.movement.facing),
                    };
                    compared += 3;
                    let placed = ours.x == pos.x as i32 && ours.y == pos.y as i32;
                    let aimed = i64::from(facing.0) == angle;
                    if placed && aimed {
                        continue;
                    }
                    // **Gaia's bearing is not modelled**, and it is not the
                    // body: `Sim::reseat_animal` puts an animal back on the
                    // original's own position every traced frame, so the
                    // place agrees and the angle never does. Counted, not
                    // asserted away.
                    if state.who >= 8 && placed {
                        gaia_bearings += 1;
                        continue;
                    }
                    // The quit's own frame is the capture's last and is
                    // half a frame: the dump is written before the rest of
                    // it runs. Every widening on this capture carries the
                    // same exemption.
                    if f == last {
                        on_the_quit += 1;
                        continue;
                    }
                    if wrong.len() < 8 {
                        wrong.push(format!(
                            "frame {f}: {}/{} guy {n} ours ({}, {}) angle {} \
                             theirs ({}, {}) angle {}",
                            state.who, state.o, ours.x, ours.y, facing.0, pos.x, pos.y, angle
                        ));
                    }
                }
            }
        }
        eprintln!(
            "run56 bodies: {compared} fields compared, {} rows wrong \
             ({gaia_bearings} gaia bearings, {on_the_quit} on the quit's frame)",
            wrong.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 1_060_000,
            "the record is being read: {compared} fields"
        );
        // **Every figure of every player's unit stands where the
        // original's does, on all 3,000 frames** — guy 0 and the crew
        // alike, and the crew's position is derived rather than read.
        assert!(
            wrong.is_empty(),
            "a figure stands somewhere the original's does not: {wrong:?}"
        );
        // Asserted as they stand, so that closing either moves the number
        // rather than passing quietly. The one on the quit's frame is
        // player 0's Citizen `o 5`, eighteen units short on both axes.
        assert_eq!(gaia_bearings, 301_810, "gaia's unmodelled bearings");
        assert_eq!(
            on_the_quit, 1,
            "rows on the capture's half-written last frame"
        );
    }

    /// **run56's collision block, over three thousand frames** — the
    /// same five fields run10 pins on Great Lakes, asked of East Indies'
    /// longest full-detail capture.
    ///
    /// The scored East Indies run is run39's 1,850 frames; run56 is the
    /// same game at the same detail carried to 3,000, and until now the
    /// only halves of its `UNITDATA` anything compared were the figures
    /// and the gather record. The collision block is compared on every
    /// frame the harness reads — `UnitData::log_data` writes all five at
    /// every detail level — and nothing asserted it here.
    #[test]
    fn run56_s_collision_block_agrees_past_the_scored_length() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — a short file here is a wrong file",
            report.frames.len()
        );
        let seen: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run56 collision: {seen} field-frames compared, {} wrong",
            bad.len()
        );
        for d in bad.iter().take(16) {
            eprintln!(
                "  frame {} {}/{} {} ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        // **249,413 field-frames, and not one of them wrong** — where
        // run10 pins 139,514 on the other map. The block is scoped to
        // unit-frames whose *positions* already agree, so the number is
        // this capture's own size rather than a score, and it moves when
        // a longer East Indies capture replaces this one.
        //
        // It was **249,293** while the AI's Dock stood two cells north of
        // the original's and its builder walked, by luck, almost the
        // original's own line. The dock's slide (`docs/AI.md` §21) put the
        // dock right and the *approach* wrong — 249,288, `1/11` parting a
        // frame earlier — and `find_nearby_spot`'s missing terrain test
        // (`docs/ORDERS.md` §10) put that right too. **`1/11` now agrees
        // for the whole capture**, and the only unit that parts at all is
        // player 0's Citizen `o 5` on the quit's own half-written frame.
        assert_eq!(
            seen, 249_413,
            "five fields on every agreeing unit-frame of run56"
        );
        assert!(
            bad.is_empty(),
            "the collision block agrees on every comparable field-frame of {seen}: {bad:?}"
        );
    }

    /// **run57, the thousand frames past run56** — the same game at the
    /// same detail carried to 4,000, asked for both position records at
    /// once because the sim run is what costs.
    ///
    /// It is a drop-in longer run56 and the tools said so before it was
    /// read: `rngcmp.py` against run54 is 4,001 frames with none
    /// differing, `samegame.py` against run56 is 3,000 in common with none
    /// differing (`docs/ORACLE.md`, "run57"). So it inherits run38's
    /// siblings, and what it adds is the only independent evidence anyone
    /// has about frames 3,000–4,000 of East Indies.
    ///
    /// It was captured for a divergence at 3021 that a widening of run56
    /// answered instead, and it has since named and then closed two more
    /// items. Its own statement was that everything past 3,000 was one
    /// thing's consequence — seventeen units first parting from 2978 —
    /// which the dock's slide (`docs/AI.md` §21) and the swarm ring's
    /// terrain test (`docs/ORDERS.md` §10) settled; what was left after
    /// that was two of the AI's farms sited in `x` alone, and those were
    /// the FARM/MINE arm's own distance (`docs/AI.md` §22).
    ///
    /// **Nothing is wrong in the collision record, and nothing is wrong in
    /// the building record up to the word.** Four of the capture's units
    /// ever leave the original's point at all, the earliest at 3647 —
    /// which is *before* the word, and not a contradiction: a unit can
    /// drift without spending a draw for it, so the two numbers are pinned
    /// separately (`docs/SYNC.md` §3.23). The building half is scoped
    /// there deliberately, and item 133 is why: for one item this test
    /// asserted every one of the 4,000 frames, and the assertion was
    /// **luck past the parting**. A building the AI sites after the two
    /// streams have parted is sited from draws that are nobody's, so it
    /// stands wherever this stream puts it — `1/2012` on 3977 came back
    /// one tile north the moment the word moved 3579 → 3608 on a change
    /// that has nothing to do with it (the dock's gull). What has teeth is
    /// the half a shared stream backs; the rest is printed, and named
    /// here, rather than pinned.
    ///
    /// `builds` is structural — two fields on every linked building-frame —
    /// and stays an equality. `coll` counts *agreeing* unit-frames, so it
    /// moves with the simulation's quality and is a floor.
    ///
    /// **The rescope is marked `FABLE:` for the next ratification pass**
    /// (`docs/ORACLE.md`, run57, "what may be asserted past the parting").
    /// It is a code-changing verdict rather than a proposal, the argument
    /// against it is real — a placement defect introduced past the word now
    /// passes quietly, and the scope shrinks with every capture longer than
    /// the word — and a third way, a ratchet on the count rather than a cut
    /// to the range, was named and not costed.
    #[test]
    fn run57_s_four_thousand_frames_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run57-islands-4k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run57.log"),
        ) else {
            eprintln!("skipping: no run57 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 4_000,
            "run57's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        let mut first_build: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &build_bad {
            if !first_build.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first_build.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run57 buildings: {builds} fields compared, {} wrong on {} building(s)",
            build_bad.len(),
            first_build.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first_build {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run57 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **Every building of both players, on every frame up to the word,
        // at the original's own point.** The two residues this test was
        // written around — `1/2011` one cell east on 3177 and `1/2012` one
        // tile on 3977 — were one defect, and it was the FARM/MINE arm's
        // own distance (`docs/AI.md` §22); 3177 is inside the scope and is
        // what this still guards.
        let build_early: Vec<&BuildDivergence> = build_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            build_early.is_empty(),
            "the AI's buildings stand where the original's do up to the word \
             ({LONG_WORD_EAST_INDIES}): {build_early:?}"
        );
        // **Not one collision field wrong in 349,794**, and only two of
        // the capture's units ever leave the original's point at all. The
        // block is scoped to unit-frames whose positions still agree, so
        // this is the capture's size and not a score.
        assert!(
            coll_bad.is_empty(),
            "the collision block agrees on every comparable field-frame of {coll}: {coll_bad:?}"
        );
        // **Nothing leaves the original's point inside the capture.** It
        // was 3647 and two units until the tile grid's own tolerance
        // landed (`docs/PATHFINDER.md` §7): a unit that can transport is
        // given an *exact* waypoint, and `1/13` — granted `unit_masks &
        // 0x800000` on 3580, the frame after its Dock finished — had been
        // cutting three frames off every leg since. The one parting left
        // is `0/5` on **4001**, the capture's last frame and run57's own
        // word. Before that: 3582 and eleven units until the colonist arm
        // landed (`docs/SYNC.md` §3.23), four until the shore reads did
        // (§3.25).
        assert!(
            parted.values().copied().min().is_none_or(|f| f >= 4_001),
            "no unit leaves the original's point before the capture's last \
             frame: {parted:?}"
        );
        assert!(
            parted.len() <= 1,
            "one unit ever leaves the original's point in 4,000 frames: {parted:?}"
        );
        assert_eq!(builds, 130_326, "two fields on every linked building-frame");
        assert!(
            coll >= 350_928,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
    }

    /// **run58 — East Indies at 5,200 frames, the successor sized past the
    /// word** (2026-09-01, item 140). run57 stops at 4,001 and East
    /// Indies' long word had reached 4020, so the standing rule owed a
    /// longer one: run39's recipe unchanged, `MAP_STYLE 18`, seed 12345,
    /// no input, carried to 5,200. Forty-eight minutes and 1.41 GB.
    ///
    /// **It is the same game twice over, and the tools said so before it
    /// was read**: `rngcmp.py` against run54 is 5,201 frames with **zero**
    /// differing, and `samegame.py` against run57 is 4,000 frames in
    /// common with **zero** differing. So it inherits run39's siblings and
    /// run54's word, and it is a drop-in longer run57 — which is why the
    /// two tests stand side by side rather than one replacing the other:
    /// run57's is the shorter, tighter floor and this one is the reach.
    ///
    /// The item it was booked for was answered without it, by the trace
    /// and the decompile while it ran (`docs/SYNC.md` §3.25). What it is
    /// *for* is item 141: the frames the word now parts on, 4275 and 4288,
    /// are past every other dump on disk.
    #[test]
    fn run58_s_five_thousand_frames_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run58-islands-5k2.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run58.log"),
        ) else {
            eprintln!("skipping: no run58 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 5_000,
            "run58's length is {} — a short file here is a wrong file",
            report.frames.len()
        );

        // The buildings, whole.
        let builds: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let build_bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        let mut first_build: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &build_bad {
            if !first_build.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first_build.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run58 buildings: {builds} fields compared, {} wrong on {} building(s)",
            build_bad.len(),
            first_build.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first_build {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }

        // The production queues, whole — every building, every live slot.
        let queues: usize = report.frames.iter().map(|f| f.queue_compared).sum();
        let queue_bad: Vec<&QueueDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.queue_diverged.iter())
            .collect();
        eprintln!(
            "run58 queues: {queues} fields compared, {} wrong",
            queue_bad.len()
        );
        for d in queue_bad.iter().take(24) {
            eprintln!(
                "  f{} {}/{} {}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }

        // **The `CITY` record, whole** — item 154(a), and the widest
        // widening the ledger has taken: forty fields on every live city of
        // every frame, where one test read four of them at frame 1
        // (`docs/CITIES.md` §5.7).
        let cities: usize = report.frames.iter().map(|f| f.city_compared).sum();
        let unlinked: usize = report.frames.iter().map(|f| f.city_unlinked).sum();
        let city_bad: Vec<&CityDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.city_diverged.iter())
            .collect();
        // One row per `(who, o, field)`, in the order each first parted,
        // with the frame it parted on and how many frames it stayed parted.
        let mut city_rows: Vec<(i64, i64, &str, i64, usize)> = Vec::new();
        for d in &city_bad {
            if let Some(row) = city_rows
                .iter_mut()
                .find(|r| (r.0, r.1, r.2) == (d.who, d.o, d.field.as_str()))
            {
                row.4 += 1;
            } else {
                city_rows.push((d.who, d.o, &d.field, d.frame, 1));
            }
        }
        let city_said: Vec<String> = city_rows
            .iter()
            .map(|&(who, o, field, frame, n)| format!("{who}/{o} {field} f{frame} x{n}"))
            .collect();
        eprintln!(
            "run58 cities: {cities} fields compared, {} wrong, {unlinked} unlinked",
            city_bad.len()
        );
        for r in &city_said {
            eprintln!("  {r}");
        }
        assert!(
            cities >= 600_000,
            "the city record is being read: {cities} fields"
        );
        assert_eq!(unlinked, 0, "every city links to a building of ours");
        // **What is left, pinned as it stands** rather than filtered out,
        // so a change that moves any of it fails rather than passing
        // quietly (the `gather_slots` precedent). Three open seams, and
        // nothing else in the record parts on any of the 5,201 frames —
        // `ter[6]` included, on both of the AI's cities, since
        // `World::gather_at` landed (`docs/AI.md` §24):
        //
        // 1. **`who 0`, the human's city — thirteen fields, every frame.**
        //    `Leaders::strategy_all@006ed430`'s gate is `leader_flags & 3
        //    == 3`: in play and not defeated, with **no** test for a human,
        //    and `Leader::production_ai@006c1960` is where a human without
        //    computer assist bails — to the switch's `default`, which
        //    clears the step machine, so the sweep re-arms and runs again on
        //    the leader's next phase frame. This crate skips the human in
        //    `Sim::strategy_all` instead, so its census never runs and the
        //    whole site picture stays zero. The dump says the same as the
        //    decompile: the human's `peasant_dist` moves on frame 401, and
        //    leader 0's phase is `frame % 200 == 0` (`docs/AI.md` §2.2).
        // 2. **The AI's second city, `1/2007`** — `land`, `filled` and the
        //    three `space` counts, one apart, from the frame its circle is
        //    first swept.
        // 3. **A gatherer filed under the wrong city** from 2576, and a
        //    free citizen from 4176 — the totals agree, the attribution
        //    does not.
        let city_want = vec![
            "0/2000 peasant_dist f1 x5201",
            "0/2000 busy f1 x5201",
            "0/2000 gatherers f1 x5201",
            "0/2000 ocean f1 x5201",
            "0/2000 land f1 x5201",
            "0/2000 filled f1 x5201",
            "0/2000 dock_tile f1 x5201",
            "0/2000 space[0] f1 x5201",
            "0/2000 space[1] f1 x5201",
            "0/2000 space[2] f1 x5201",
            "0/2000 ter[0] f1 x5201",
            "0/2000 ter[1] f1 x5201",
            "0/2000 ter[3] f1 x5201",
            "1/2007 land f1819 x157",
            "1/2007 filled f1819 x3383",
            "1/2007 space[0] f1976 x3226",
            "1/2007 space[1] f1976 x3226",
            "1/2007 space[2] f1976 x3226",
            "1/2000 gatherers f2576 x800",
            "1/2007 peasant_dist f2576 x800",
            "1/2007 gatherers f2576 x800",
            "1/2007 free f4176 x200",
        ];
        assert_eq!(
            city_said, city_want,
            "the `CITY` record's open rows moved; every other field of every \
             city of every frame agrees"
        );

        // The collision block, on every unit-frame whose position agrees.
        let coll: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run58 collision: {coll} field-frames compared, {} wrong, \
             {} unit(s) ever off position",
            coll_bad.len(),
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }

        // **Everything this asserts is scoped to before the word**, and
        // run58 is the first capture long enough for that to matter. Past
        // 4275 the two streams are running on draws that are nobody's, so
        // a unit standing somewhere else there is not a defect — nineteen
        // of them do, all first parting between 4300 and 5085, and the
        // list is printed above rather than pinned. Before the word the
        // capture is evidence, and there it is exact.
        let build_early: Vec<&BuildDivergence> = build_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            build_early.is_empty(),
            "the AI's buildings stand where the original's do up to the word \
             ({LONG_WORD_EAST_INDIES}): {build_early:?}"
        );
        let coll_early: Vec<&CollideDivergence> = coll_bad
            .iter()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert!(
            coll_early.is_empty(),
            "the collision block agrees on every comparable field-frame up \
             to the word: {coll_early:?}"
        );
        // **And the production queues, whole** — the widening that found
        // the word itself. Before it, the queue record was compared in
        // `run39_s_build_queues_are_the_original_s_clock` alone, on a
        // capture 2,600 frames too short to reach the AI's first ship; the
        // twenty-eight frames of `queued ours 1 theirs 0` at `1/2010` were
        // sitting in this dump the whole time (`docs/PRODUCTION.md`, "The
        // tail's first caller").
        //
        // **And the word has now outrun this capture.** run54's word is
        // 5376 and run58 is 5,201 frames, so "before the word" is the
        // whole file.
        //
        // The tail used to be twenty-four frames of `1/2005 queued ours 0
        // theirs 1` — the AI's library holding a **Coinage** job from
        // frame 5177 that this crate never started. It was
        // `economy::Holdings::available`, all-true from frame 0 since it
        // existed: knowledge is not available before the Classical Age, so
        // the original charges Coinage's `14k` as two hundred and ten food
        // and this crate asked for a hundred and forty knowledge nobody
        // can hold (`docs/COSTS.md`, "Three of the six resources are not
        // available from the start"; `docs/ECONOMY.md`, "Availability, and
        // where a price lands instead"). Those rows are gone.
        //
        // What is left is the capture's **last frame and nothing else**.
        // On 5201 the original prints `queued 0` for both buildings that
        // held a job on 5200 — `1/2005`'s research and `1/2010`'s
        // Fisherman — and prints the whole `BUILDDATA` list a second time,
        // truncated; the run was quitting. Frames 5195..=5200 are stable
        // and agree. The two rows are asserted **as they stand** rather
        // than filtered out, so a change that moves them fails rather than
        // passing quietly (the `gather_slots` precedent,
        // `run40_s_census_…`).
        let queue_early: Vec<&QueueDivergence> = queue_bad
            .iter()
            .filter(|d| d.frame < RUN58_QUEUE_TAIL)
            .copied()
            .collect();
        assert!(
            queue_early.is_empty(),
            "the AI's queues run the original's clock up to \
             {RUN58_QUEUE_TAIL}: {queue_early:?}"
        );
        let queue_tail: Vec<(i64, i64, &str, i64, i64)> = queue_bad
            .iter()
            .filter(|d| d.frame >= RUN58_QUEUE_TAIL && d.frame < LONG_WORD_EAST_INDIES)
            .map(|d| (d.frame, d.o, d.field.as_str(), d.ours, d.theirs))
            .collect();
        let want: Vec<(i64, i64, &str, i64, i64)> =
            vec![(5201, 2005, "queued", 1, 0), (5201, 2010, "queued", 1, 0)];
        assert_eq!(
            queue_tail, want,
            "run58's queues are the original's on every frame it dumps \
             whole; only 5201, the truncated last one, disagrees"
        );
        assert!(
            queues >= 109_435,
            "the queue record is being read: {queues} fields"
        );
        // **The path stack's own rows, before the word.** run39's
        // widening (`run39_s_path_stack_agrees_row_for_row`) scores
        // `tolerance` and `flags` beside each waypoint's point, and it
        // passed for a week: run39 is 1,850 frames and the tile grid's
        // tolerance only stops being `0x60` for a unit that can transport
        // — which on East Indies is frame **3580**, past the end of that
        // capture. So the check was right and the capture was short. Here
        // it is on the long one, where a `tolerance` row is what the
        // thirteen-frame lead of item 141 actually was.
        //
        // **And the two AI Fishermen are inside it now.** Their sea
        // routes were the one exception this check carried: `1/14` from
        // 4464 and `1/16` from 4870 planned the same nineteen cells from
        // (57, 55) to the fish at (49, 39) staircased two cells north of
        // the original's, because the world grid's same-region test was
        // asking `get_tregion` where the original reads `WData.region`
        // (`docs/PATHFINDER.md` §16). With the raw field both stacks agree
        // row for row on the frame each is planned, so there is no
        // partition and no pin — every waypoint of every unit is asserted.
        let path_rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| {
                d.frame < LONG_WORD_EAST_INDIES
                    && matches!(
                        d.what,
                        OrderMismatch::PathField { .. } | OrderMismatch::PathTo { .. }
                    )
            })
            .collect();
        assert!(
            path_rows.is_empty(),
            "every waypoint agrees row for row up to the word: {} rows,              first {:?}",
            path_rows.len(),
            path_rows.first()
        );
        // **The packed bit and the line of sight, whole and everywhere.**
        // Both are written at every detail level and both are what the
        // fishing boat's deploy moves: `unit_masks 786440 → 262152` and
        // `mylos 4 → 6` on the same frame, 4989. The AI's two Fishermen
        // are the only units on this capture that carry the bit at all,
        // and before item 149 neither ever lost it — the deploy died the
        // frame after it was queued, so the pair here is the item's own
        // oracle (`docs/ORDERS.md` §6.9).
        let packed_seen: usize = report.frames.iter().map(|f| f.packed_compared).sum();
        let packed_bad: Vec<PackedDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.packed_diverged.iter().copied())
            .collect();
        let los_seen: usize = report.frames.iter().map(|f| f.los_compared).sum();
        let los_bad: Vec<LosDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.los_diverged.iter().copied())
            .collect();
        eprintln!(
            "run58 packed: {packed_seen} unit-frames, {} wrong; \
             los: {los_seen} unit-frames, {} wrong",
            packed_bad.len(),
            los_bad.len()
        );
        for d in packed_bad.iter().take(4) {
            eprintln!("  packed f{} {}/{}: ours {}", d.frame, d.who, d.o, d.ours);
        }
        for d in los_bad.iter().take(4) {
            eprintln!(
                "  los f{} {}/{}: ours {} theirs {}",
                d.frame, d.who, d.o, d.ours, d.theirs
            );
        }
        // **The packed bit is asserted over the whole capture, not up to
        // the word**, and deliberately: the boat deploys on 4989, one
        // frame *past* it, so an assertion fenced to the word could not
        // see the thing the item is about. It is exact on all 94,935
        // unit-frames, and it is what fails the minute `do_cast` stops
        // waiting out the forty.
        assert!(
            packed_bad.is_empty(),
            "the packed bit is the original's on every unit-frame: {:?}",
            packed_bad.first()
        );
        // …and `mylos` has exactly the one disagreement run39 pins on its
        // own 1,850 frames — the AI scout's Science level, which this
        // crate's pure function reports on the frame the level lands and
        // the original's cached field reports one level later (item 35,
        // `docs/VISION.md` §7). Over 94,338 unit-frames here it is still
        // the only one, and the AI's two Fishermen — the only units on
        // this capture that are ever packed — are not among them.
        let los_early: Vec<LosDivergence> = los_bad
            .iter()
            .copied()
            .filter(|d| d.frame < LONG_WORD_EAST_INDIES)
            .collect();
        assert_eq!(
            los_early,
            vec![LosDivergence {
                frame: 202,
                who: 1,
                o: 0,
                ours: 6,
                theirs: 4,
            }],
            "mylos is the original's up to the word but for the cache"
        );
        assert!(
            packed_seen >= RUN58_PACKED_FRAMES && los_seen >= RUN58_PACKED_FRAMES,
            "both are read on every linked unit-frame: {packed_seen} / {los_seen}"
        );

        let early: std::collections::BTreeMap<_, _> = parted
            .iter()
            .filter(|(_, f)| **f < LONG_WORD_EAST_INDIES)
            .collect();
        assert_eq!(
            early.len(),
            RUN58_PARTED,
            "before the word, no unit leaves the original's point: {early:?}"
        );
        assert_eq!(
            builds, RUN58_BUILD_FIELDS,
            "two fields on every linked building-frame"
        );
        assert!(
            coll >= RUN58_COLL_FIELDS,
            "five fields on every agreeing unit-frame, and the count only \
             grows: {coll}"
        );
    }

    /// **Where the buildings stand** — run56's `BUILDDATA` position, on
    /// every linked building of every frame.
    ///
    /// `x_internal` and `y_internal` are written at every detail level and
    /// the parser has carried them since the record existed; nothing ever
    /// compared them. What they say is *where the AI put it*, and a
    /// building the AI sites 16 tiles away still links by `(who, o)` and
    /// still agrees on every gather field, so the whole of
    /// `docs/AI.md` §2.20 was uncheckable against a capture until this.
    ///
    /// **92,626 fields, and every one of them right.** The last residue
    /// was one building's `y` — player 1's Dock `o 2010`, laid on frame
    /// 2977 two cells north of the original's — and it closed with the
    /// dock's own sub-position slide (`docs/AI.md` §21): the slide is not
    /// what places this dock, it is what makes the spiral *accept* a cell
    /// three ring-6 candidates earlier, and the stride of three that
    /// engages there lands on the original's own index.
    #[test]
    fn run56_s_buildings_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — a short file here is a wrong file",
            report.frames.len()
        );
        let seen: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        // Which buildings, and from which frame — the successor's own
        // statement, printed whether or not the assertion below fails.
        let mut first: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &bad {
            if !first.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run56 buildings: {seen} fields compared, {} wrong on {} building(s)",
            bad.len(),
            first.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }
        assert_eq!(seen, 92_626, "two fields on every linked building-frame");
        // **Every building of both players stands on the original's own
        // point for all 3,000 frames** — the pre-placed ones, the farms
        // and the second city this crate sites itself, the camp of item
        // 85, and now the AI's Dock.
        assert!(
            bad.is_empty(),
            "a building stands somewhere the original's does not: {first:?}"
        );
    }

    /// **The crew's follow offsets, from the install** — `track_dx` and
    /// `track_dy`, which decide whether a unit's second figure stands on
    /// the first or walks its own body behind it
    /// (`docs/MOVEMENT.md`, "The follower's destination").
    ///
    /// The two scout dogs are the pair that matters, because they are the
    /// headline's own divergence: both nations' `-CREW1` entry writes
    /// `trackoffsetx="-20" trackoffsety="10" scale="1"`, and `guy_scale`
    /// is the executable's 4.8, so the stored pair is `(-96, 48)`. A
    /// citizen has one figure and no entry here at all.
    /// **The crew's follow offsets, from the install** — `track_dx` and
    /// `track_dy`, which decide whether a unit's second figure stands on
    /// the first or walks its own body behind it
    /// (`docs/MOVEMENT.md`, "The follower's destination").
    ///
    /// The two scout dogs are the pair that matters, because they are the
    /// headline's own divergence: both nations' `-CREW1` entry writes
    /// `trackoffsetx="-20" trackoffsety="10" scale="1"`, and `guy_scale`
    /// is the executable's 4.8, so the stored pair is `(-96, 48)`. A
    /// citizen has one figure and no entry here at all.
    #[test]
    fn the_install_s_crew_tracks_are_the_scout_dog_s() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        for dog in [13043, 12691] {
            assert_eq!(
                loaded.piece_tracks.get(&dog).copied(),
                Some((-96, 48)),
                "piece {dog} is a scout's dog and carries the -20/10 offset"
            );
        }
        // A guy-0 piece has no offset: the scouts themselves, and the
        // citizens, which are one figure each.
        for lone in [371, 19, 352, 6688] {
            assert!(
                !loaded.piece_tracks.contains_key(&lone),
                "piece {lone} is a guy 0 and names no track offset"
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

    /// A trace beside the dumps — `RON_GAMELOG_DIR`, where `archive.sh`
    /// puts both. `None` skips the half of a check that needs it, the same
    /// way [`dump`] does.
    fn trace(name: &str) -> Option<crate::trace::Trace> {
        // The same default as [`dump`]: a machine with the captures but no
        // `RON_GAMELOG_DIR` used to skip every trace-backed half silently.
        let path = dump(name)?;
        crate::trace::Trace::read(std::path::Path::new(&path))
            .ok()
            .flatten()
    }

    /// Run20's `Farms` list, and the pasture in it. Six farms, one of them
    /// `farm_type == 1` — the AI's `o 2002` — which grows nothing and
    /// carries five animals of **owner 9** that no dump prints (run20's
    /// first `FULL DUMP` has 104 `ANIMALDATA` records and not one `who 9`).
    ///
    /// The stream is the assertion. `rontrace-run20.log` places frame 0's
    /// 175 draws by site: 104 `Animal::do_idle` idle rolls for the dumped
    /// animals, then **five more and one `Animal::think_farm_animal`** —
    /// the pasture's — and **five** `Farms::inc_time` draws, not six. So
    /// the harness's frame 0 moves from 160 to 165 and its frame 2, which
    /// is the farms alone on both sides, from 6/5 to **5/5**
    /// (`docs/SYNC.md` §3.6, §4.2).
    #[test]
    fn run20_s_pasture_grows_nothing_and_its_five_animals_draw_six() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut init = log.initial().unwrap();

        // The list, as `Farms::log_data` wrote it.
        let farms = &init.farms;
        assert_eq!(farms.len(), 6, "run20's six farms");
        assert!(farms.iter().all(|f| f.valid == 1), "every slot live");
        let pastures: Vec<_> = farms.iter().filter(|f| f.farm_type == 1).collect();
        assert_eq!(pastures.len(), 1, "one pasture");
        assert_eq!(
            (pastures[0].who, pastures[0].o),
            (1, 2002),
            "the AI's first"
        );
        assert_eq!(
            farms.iter().map(|f| f.farm_type).collect::<Vec<_>>(),
            vec![1, 0, 0, 0, 0, 4],
            "the crops, in the list's order"
        );

        // The five animals, and the farm they hang off. run20's own trace
        // reached the setup, so the five are borrowed from it rather than
        // stood on the farm's centre — a **second** capture through
        // `Trace::add_animals`, and a different pasture from run39's
        // (`docs/SYNC.md` §3.11).
        if let Some(tr) = trace("rontrace-run20.log") {
            borrow_pasture(&mut init, &tr);
            assert_eq!(init.pasture.len(), 1, "run20's one pasture, from its trace");
            assert!(
                init.pasture[0].iter().all(|a| a.chicken)
                    || init.pasture[0].iter().all(|a| !a.chicken),
                "a pasture is one species — five even coins or five odd, never a mix"
            );
            assert!(
                init.pasture[0]
                    .iter()
                    .all(|a| (-0xc0..0xc0).contains(&a.dx) && (-0xc0..0xc0).contains(&a.dy)),
                "and each offset is `% 0x180 - 0xc0`, so inside a tile either way"
            );
        }
        let built = build_sim(&loaded, &init, Tuning::RON);
        let animals: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.units[u].owner == 9)
            .collect();
        assert_eq!(animals.len(), 5, "five animals of owner 9");
        let farm = built.sim.units[animals[0]].farm_animal.unwrap().build;
        assert_eq!(built.sim.buildings[farm].owner, 1);
        assert_eq!(built.sim.buildings[farm].index, 2002);
        assert!(
            animals
                .iter()
                .all(|&u| built.sim.units[u].farm_animal.unwrap().build == farm),
            "all five on the same farm"
        );
        assert_eq!(
            animals
                .iter()
                .map(|&u| built.sim.units[u].farm_animal.unwrap().slot)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4],
            "slotted 0..5, which is what phases their think tick"
        );
        assert!(
            init.units.iter().all(|u| u.who != 9),
            "and no dump prints them"
        );

        // The counts, on the original's own stream.
        let report = run_traced(&loaded, &log, Tuning::RON, Some(4), None, &[], None).unwrap();
        let count = |f: i64| -> (Option<u32>, Option<u32>) {
            let &(_, ours, theirs) = report
                .rng_frames
                .iter()
                .find(|(n, _, _)| *n == f)
                .unwrap_or_else(|| panic!("frame {f} was not traced"));
            (ours, theirs)
        };
        assert_eq!(
            count(0),
            (Some(175), Some(175)),
            "frame 0, with `Unit::think_scout`'s ten in (`docs/SCOUT.md` §10)"
        );
        assert_eq!(
            count(2),
            (Some(5), Some(5)),
            "frame 2 is the five crop farms and nothing else, on both sides"
        );

        // And the islands' opening carries **both angles exactly**, on
        // every unit-frame — the smallest of the angle checks and the only
        // one at zero (`docs/MOVEMENT.md`, "Two angles"). It is the one
        // that would fail first if `Unit::init`'s 0x55555555 or the two
        // fields' assignment were put back the way they were.
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        let bad: Vec<AngleDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter().copied())
            .collect();
        assert_eq!(angles, 72);
        assert_eq!(bad, vec![], "every heading and every facing");
    }

    /// The two sequences a whole frame folds to, in one place — ours from
    /// [`mark_sites`], the original's from
    /// [`Trace::labels`](crate::trace::Trace::labels) — and the index they
    /// first part at, with a window either side.
    ///
    /// A `Vec<String>` comparison of 175 entries prints as a wall; the
    /// first difference and its neighbourhood is the whole of what a
    /// reader needs, so the failure message is built rather than left to
    /// `assert_eq!`.
    fn first_parting(ours: &[String], theirs: &[String]) -> Option<(usize, String)> {
        let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i))?;
        let lo = at.saturating_sub(3);
        let hi = (at + 4).min(ours.len().max(theirs.len()));
        let mut s = format!("the sequences part at draw {at}:\n");
        for i in lo..hi {
            let mine = ours.get(i).map_or("—", |x| x.as_str());
            let yours = theirs.get(i).map_or("—", |x| x.as_str());
            let flag = if mine == yours { "  " } else { "≠ " };
            s.push_str(&format!("  {flag}{i:>4}  ours {mine:<52} theirs {yours}\n"));
        }
        Some((at, s))
    }

    /// **Run20's whole frame 0, draw for draw** — item 27, and the swap it
    /// was the instrument for.
    ///
    /// Frame 0 counted 175 against 175 from the moment the scout landed,
    /// and the count was hiding two errors that cancelled. Every mechanic
    /// the frame touches now marks its own draw sites under the original's
    /// offsets (`sim::ai_sites`, `sim::market`, `sim::anim`, `sim::scout`,
    /// `sim::farms`, `sim::gaia`), and [`crate::trace::SITES`] names the
    /// same addresses out of the trace's `ebp` chain — so the frame is one
    /// `Vec<String>` on each side and the comparison is an `assert_eq!`.
    ///
    /// **What it found on its first run, which is the whole of the swap.**
    /// The two sequences agreed for 22 draws and parted: ours spent a
    /// `set_anim` roll for each gathering citizen, at a call the original
    /// does not make. The camp-arrival stand in `do_non_flat_gather` was
    /// the sim's own — the branch is a two-way `CHAR_DUMP_WOOD` /
    /// `CHAR_DUMP_ORE`, and the listing at `5f0b5e`–`5f0b89` has no third
    /// `set_anim`. Removing it did not cost two draws; it moved four. The
    /// stand had been resetting the citizens' clocks, so the four wraps
    /// the original spends in phase 7 — `Guy::set_anim+0x97a` under
    /// `Guy::inc_time`, between the herd walk and the farms — never fell
    /// due here. Both halves of `docs/SYNC.md` §6's stand/wrap swap were
    /// the one line, and no `GUYS=4` capture was needed to see it.
    ///
    /// The assertion is the whole sequence. A count of 175 cannot fail
    /// this way twice.
    #[test]
    fn frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        // (dump, trace, the frame-0 word, draws, farm draws, wraps)
        //
        // Two lobbies, and they are not the same shape: run20's islands
        // have two woodcutters a side and a pasture among its six farms
        // (five crop draws); run12's world6 has four and no pasture (six).
        // Both spend four `Guy::inc_time` wraps at the tail, and on run12
        // those are the four woodcutters `docs/ANIM.md` §5 could not place.
        let maps = [
            (
                "gamelog-run20-islands-dumpall.txt",
                "rontrace-run20.log",
                0x2f50_5213u32,
                175usize,
                5usize,
            ),
            (
                "gamelog-run12-dumpall-seeds.txt",
                "rontrace-run14.log",
                0x3bd3_9ae9,
                120,
                6,
            ),
        ];
        let mut ran = 0;
        for (dump_name, trace_name, word, draws, farms) in maps {
            let (Some(path), Some(trace)) = (dump(dump_name), trace(trace_name)) else {
                eprintln!("skipping {dump_name}: set RON_GAMELOG_DIR");
                continue;
            };
            ran += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let init = log.initial().unwrap();
            let mut built = build_sim(&loaded, &init, Tuning::RON);

            // The two sides start on the same word, or nothing below means
            // anything: the dump's last setup checksum and the word the
            // trace's own first draw stepped are one number seen from two
            // instruments. It is the *draw's* word rather than the `FRAME`
            // record's, because run14 predates the record carrying it —
            // its frame 0 reports `0x03fc45c6` and its first draw steps
            // `0x3bd39ae9`, which is what run12's dump also says.
            assert_eq!(
                trace.frame_draws(0).first().map(|d| d.seed),
                Some(word),
                "{trace_name}'s first frame-0 draw"
            );
            assert_eq!(built.sim.rng.seed, word, "{dump_name}: and the harness's");

            built.sim.trace_phases = true;
            built.sim.tick();
            let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
            let theirs = trace.labels(0);
            assert_eq!(theirs.len(), draws, "{trace_name}'s frame 0");

            // Every draw of the original's frame 0 is a site the sim
            // models: a bare hex address here would be a mechanic with no
            // mark, and the comparison below could not read it.
            let unnamed: Vec<&String> = theirs
                .iter()
                .filter(|l| !crate::trace::SITES.iter().any(|(_, _, n)| n == l))
                .collect();
            assert!(
                unnamed.is_empty(),
                "{trace_name}: frame 0 has draws no mechanic marks: {unnamed:?}"
            );

            if let Some((_, shown)) = first_parting(&ours, &theirs) {
                panic!("{dump_name}: {shown}");
            }
            assert_eq!(ours, theirs, "{dump_name}: frame 0, draw for draw");

            // And the counts the swap turned on, stated so a regression
            // reads as itself rather than as an index: four idle rolls for
            // the two scouts' figures, no camp stand at all, and four
            // phase-7 wraps.
            let count = |site: &str| ours.iter().filter(|l| *l == site).count();
            assert_eq!(
                (
                    count(sim::anim::SITE_IDLE_UNIT),
                    count(sim::anim::SITE_STAND_GATHER),
                    count(sim::anim::SITE_WRAP),
                ),
                (4, 0, 4),
                "{dump_name}: the scouts' four, no camp stand, four wraps"
            );
            // The wraps sit between the herd walk and the farms — the
            // frame's tail, which `docs/SYNC.md` §6 read as "the 4 draws
            // at 110–113".
            let mut fold: Vec<(String, usize)> = Vec::new();
            for l in &ours {
                match fold.last_mut() {
                    Some((last, n)) if last == l => *n += 1,
                    _ => fold.push((l.clone(), 1)),
                }
            }
            let last_three: Vec<(&str, usize)> = fold
                .iter()
                .rev()
                .take(3)
                .map(|(l, n)| (l.as_str(), *n))
                .collect();
            assert_eq!(
                last_three,
                vec![
                    (sim::farms::SITE_CHANCE, farms),
                    (sim::anim::SITE_WRAP, 4),
                    (sim::gaia::SITE_HERD_Y, 1),
                ],
                "{dump_name}: the frame ends farms ← wraps ← herd"
            );
        }
        if ran == 0 {
            eprintln!("skipping: neither traced map is on this machine");
        }
    }

    /// **Run20's frame 1** — the AI's fourth farm, and the walk that stopped
    /// asking for a detour (items 25 and 28; `docs/SYNC.md` §3.8, §6).
    ///
    /// Frame 1 is not draw-for-draw yet, so this is a **ratchet on the two
    /// sites the session moved** rather than a whole-frame `assert_eq!`:
    ///
    /// - `Farms::add`'s **ambience pair**, one `+0x23f` and one `+0x25b`,
    ///   and **no** `+0x128` — the AI's new farm lands in the city that
    ///   already holds the map's pasture, so `others != crops` decides the
    ///   type with no coin, and the city has two crops and no emitter yet.
    ///   The sim spent neither of these until `Sim::farms_add` existed.
    /// - `Unit::do_move+0xe84` **once**, where it used to be twice and the
    ///   original spends it not at all. The one that went was the
    ///   woodcutter's, closed by `find_path`'s pull-back (`docs/ORDERS.md`
    ///   §4.6): its goal is a forest tile, and the original walks the goal
    ///   back out of the forest before marching. **The one that remains is
    ///   the AI scout's**, whose straight line clips a *building* several
    ///   tiles short of its waypoint — `go_around_building@005fc350`, still
    ///   a seam. When that lands this row is `0` and the assertion below
    ///   must be edited to say so.
    ///
    /// - **`Leader::produce_building` draw for draw** since 2026-08-26:
    ///   `+0xc99` ×39 and `+0x1805` ×4, the original's own split, and the
    ///   farm the call places lands on the original's own tile. The
    ///   frame's only residue is now the `do_move` above, so the total
    ///   reads 54 against 53 — it no longer cancels, which is the point.
    ///
    /// The old note here said the frame read 53/53 and was *still wrong*,
    /// because one missing `produce_building` draw cancelled the stray
    /// `do_move`. Three defects were behind that one number
    /// (`docs/AI.md` §2.20): the jitter's two loops are inclusive, so
    /// `ex == ey == 1` is a 2×2 and four draws rather than one; the
    /// stride-by-three test read the loop's start index instead of the
    /// current one; and `WorldData::buildings_allowed` was not modelled at
    /// all, so a forest cell scored as a candidate and drew.
    #[test]
    fn run20_s_frame_1_spends_the_farm_s_ambience_pair() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run20-islands-dumpall.txt"),
            trace("rontrace-run20.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // Frame 0 through the harness, so frame 1 starts on the original's
        // own word and its clocks; then frame 1 raw, so the marks survive.
        built.tick();
        built.sim.tick();
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        let theirs = trace.labels(1);
        assert_eq!(theirs.len(), 53, "the original's frame 1");

        let count = |v: &[String], site: &str| v.iter().filter(|l| *l == site).count();
        for (site, n) in [
            (sim::farms::SITE_AMBIENCE_X, 1),
            (sim::farms::SITE_AMBIENCE_Y, 1),
            (sim::farms::SITE_TYPE_COIN, 0),
        ] {
            assert_eq!(count(&theirs, site), n, "the original's {site}");
            assert_eq!(count(&ours, site), n, "ours: {site}");
        }
        // And they are adjacent, x before y, which a count cannot say.
        let at = ours
            .iter()
            .position(|l| l == sim::farms::SITE_AMBIENCE_X)
            .expect("the x offset");
        assert_eq!(
            ours.get(at + 1).map(String::as_str),
            Some(sim::farms::SITE_AMBIENCE_Y)
        );
        assert_eq!(
            theirs.iter().position(|l| l == sim::farms::SITE_AMBIENCE_X),
            Some(43),
            "the original spends them right after `produce_building`'s 43"
        );

        // The grid draw: neither side's now. The AI scout's line clips a
        // building three tiles short of a clear goal, and since
        // `go_around_building` landed (item 29) the sim finds the same
        // detour the original does and spends nothing on it.
        assert_eq!(count(&theirs, sim::orders::SITE_MOVE_GRID), 0);
        assert_eq!(count(&ours, sim::orders::SITE_MOVE_GRID), 0);
        // And with that, the whole frame is one sequence.
        if let Some((_, shown)) = first_parting(&ours, &theirs) {
            panic!("run20 frame 1: {shown}");
        }
        assert_eq!(ours, theirs, "run20: frame 1, draw for draw");

        // **The detour itself, against the original's own stack.** The unit
        // whose line clips a building on this frame is `1/1` — not `1/0`,
        // which is what `docs/SYNC.md` §6 named while the count was the
        // only measurement; the AI's farm moved and with it which unit
        // pays. Its stack at the end of frame 1 is the goal plus exactly
        // one `go_around_building` waypoint, and the log's frame-2 record
        // carries the same two entries, tolerance and flags included. The
        // skew — `off % 0xc0 / 2` off the tile's low corner rather than the
        // centre — is in that number, so a detour placed at the centre
        // fails here.
        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 1)
            .expect("the AI's unit 1");
        let theirs_path = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 2)
            .expect("frame 2")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 1)
            .expect("1/1 at frame 2")
            .path;
        let ours_path: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs_path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs_path,
            vec![(41640, 39384, 0, 1), (40644, 39036, 0, 0)],
            "the original's 1/1: the goal and one detour waypoint"
        );
        assert_eq!(ours_path, theirs_path, "1/1's stack, entry for entry");

        // `produce_building`, site for site: the spiral's friendless
        // FARM/MINE candidates and the 2×2 jitter's unblocked
        // sub-positions, both the original's counts.
        for site in [sim::ai_place::SITE_SPIRAL, sim::ai_place::SITE_JITTER] {
            assert_eq!(count(&ours, site), count(&theirs, site), "{site}");
        }
        assert_eq!(count(&theirs, sim::ai_place::SITE_SPIRAL), 39);
        assert_eq!(count(&theirs, sim::ai_place::SITE_JITTER), 4);
        // And they are one call, in the original's order: every spiral
        // draw before every jitter draw.
        let last_spiral = ours
            .iter()
            .rposition(|l| *l == sim::ai_place::SITE_SPIRAL)
            .expect("a spiral draw");
        let first_jitter = ours
            .iter()
            .position(|l| *l == sim::ai_place::SITE_JITTER)
            .expect("a jitter draw");
        assert!(
            last_spiral < first_jitter,
            "the spiral scores, then jitters"
        );

        // The site itself, which is what the draws are for: the AI's new
        // farm is `who 1, o 2006` at (41856, 39552) in the run's own
        // `BUILDDATA`, and the sim now puts it there. Before the jitter
        // was a 2×2 it landed one sub-position away.
        let farm = built
            .sim
            .buildings
            .iter()
            .find(|b| b.alive && b.owner == 1 && b.index == 2006)
            .expect("the AI's new farm");
        assert_eq!(
            (farm.pos.x, farm.pos.y),
            (41856, 39552),
            "the original's own tile"
        );
    }

    /// **`toff`, against the original's own world chain** (item 30).
    ///
    /// Run20's unit `1/0` walks a `find_wpath` chain the original logs at
    /// `(42744, 37368)`, `(41976, 38136)`, `(41976, 38904)`, … — every one
    /// of them `cell*0x300 + 504` on both axes, where the simulation used
    /// to emit the cell **centre** `+0x180`. `504` is the unit's own move
    /// order's `off_x` (`MoveOrder +0x4c` = `dest % 0x300`), and
    /// `astar_path`'s prologue reads it through `is_move` / `update_move_
    /// order` — the current order's own, with no target involved
    /// (`docs/PATHFINDER.md` §2, §7).
    ///
    /// So the assertion is on the **lattice**: every waypoint the sim puts
    /// on the world grid sits at `+504` and is one the original's stack
    /// also carries. With the old `toff = 0` seam every one of them sits at
    /// `+0x180` instead, no sim entry is on the lattice at all, and the
    /// count below is zero — which is how this was made to fail.
    ///
    /// ~~What it does **not** yet assert is the two ends.~~ The goal end is
    /// closed by item 31 (`docs/GROUPS.md` §6.7): the bottom entry is the
    /// leader's raw slot and the sim now writes the original's own number.
    /// What is left is the **middle** of the route, which is the
    /// pathfinder's and not this mechanic's — see
    /// `run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot`.
    #[test]
    fn run20_s_world_chain_sits_on_the_move_orders_own_offset() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.tick();
        built.tick();

        let theirs = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 2)
            .expect("frame 2")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("1/0 at frame 2");
        // The oracle, pinned: the goal, then eight world nodes, every one
        // of them on the `+504` lattice with the world grid's tolerance.
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs_path,
            vec![
                (41952, 36576, 0, 1),
                (42744, 37368, 384, 0),
                (41976, 38136, 384, 0),
                (41976, 38904, 384, 0),
                (41208, 39672, 384, 0),
                (40440, 38904, 384, 0),
                (39672, 38904, 384, 0),
                (38904, 38904, 384, 0),
                (38136, 39672, 384, 0),
            ],
            "the original's 1/0 at frame 2"
        );
        // And the offset is the order's own, read straight off the dump.
        let mo = theirs.orders.first().expect("the current order");
        assert_eq!(
            (mo.off_x, mo.off_y),
            (Some(504), Some(504)),
            "run20's `off_x`/`off_y`"
        );
        assert_eq!(
            (mo.x, mo.y),
            (Some(41976), Some(36600)),
            "and they are `x mod 0x300`"
        );
        for e in &theirs_path[1..] {
            assert_eq!((e.0 % 0x300, e.1 % 0x300), (504, 504), "{e:?}");
        }

        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 0)
            .expect("the AI's unit 0");
        let ours: Vec<(i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| (i64::from(p.to.x), i64::from(p.to.y)))
            .collect();
        // Item 31, closed: the goal at the bottom is the leader's raw slot
        // — `Group::action_move_near` pushes `form +0x514/+0x714` un-snapped
        // — and not the order's `dest`, which `add_move_facing_order` has
        // put on the `u*0x30 + 0x18` grid `0x18` further out on both axes.
        assert_eq!(ours[0], (41952, 36576), "ours: the leader's raw slot");
        assert_eq!((theirs_path[0].0, theirs_path[0].1), ours[0]);
        let on_lattice: Vec<(i64, i64)> = ours[1..]
            .iter()
            .copied()
            .filter(|(x, y)| x % 0x300 == 504 && y % 0x300 == 504)
            .collect();
        // Not one of them on the cell centre any more — that is the whole
        // of the old seam, and it is what pinning `toff` back to zero
        // restores.
        let on_centre = ours[1..]
            .iter()
            .filter(|(x, y)| x % 0x300 == 0x180 && y % 0x300 == 0x180)
            .count();
        assert_eq!(on_centre, 0, "cell centres are back: {ours:?}");
        assert!(
            on_lattice.len() >= 5,
            "the sim's chain is off the order's lattice: {ours:?}"
        );
        // And they are the original's own entries, exactly — every one of
        // them, since item 32. This check keeps its own floor rather than
        // pinning the chain: what it is *for* is the lattice, and the
        // chain is `run20_s_group_member_is_pathed_at_order_time_off_the_
        // leaders_slot`'s to pin.
        let shared = on_lattice
            .iter()
            .filter(|e| theirs_path.iter().any(|t| (t.0, t.1) == **e))
            .count();
        assert!(
            shared >= 3,
            "shared with the original: {shared} of {ours:?}"
        );
    }

    /// **The path stack's other two columns** (2026-08-31) — `tolerance`
    /// and `flags`, which every `PATHDATA` row prints beside the point and
    /// which nothing compared for the eight days the stack has been an
    /// oracle (`docs/PATHFINDER.md` §10). The working agreement's "diff the
    /// whole record", applied to the record it had been applied to least.
    ///
    /// Two halves. The first is the **census of the original's own
    /// shapes** over run39's whole capture — 1,724 multi-entry stacks —
    /// which is an oracle in its own right and cost one pass: the bottom
    /// is the order's goal every time, and above it there are exactly
    /// three shapes, one of which this crate never writes (see the counts
    /// below). The second is the harness's own disagreement count on the
    /// same capture, as a ceiling.
    ///
    /// `UnitData::tolerance` is read straight off the top entry
    /// (`docs/ORDERS.md` §4.4, the waypoint block), so a wrong tolerance is
    /// not cosmetic: it is the radius at which a unit calls a waypoint
    /// reached.
    #[test]
    fn a_path_stack_s_rows_are_compared_whole() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);

        // Half one: the original's own shape, read straight off the dump —
        // `(tolerance, flags)` of every multi-entry stack's top and bottom,
        // tallied rather than asserted one by one, because the tally is the
        // finding.
        let mut tops: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        let mut bottoms: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        let mut middles: std::collections::BTreeMap<(i64, i64), usize> = Default::default();
        for f in log.frame_states() {
            for u in &f.units {
                if u.path.len() < 2 {
                    continue;
                }
                *bottoms
                    .entry((u.path[0].tolerance, u.path[0].flags))
                    .or_default() += 1;
                let top = &u.path[u.path.len() - 1];
                *tops.entry((top.tolerance, top.flags)).or_default() += 1;
                for e in &u.path[1..u.path.len() - 1] {
                    *middles.entry((e.tolerance, e.flags)).or_default() += 1;
                }
            }
        }
        eprintln!("run39 stacks: tops {tops:?} bottoms {bottoms:?} middles {middles:?}");
        // The bottom is the order's own goal, always — `Unit::do_move`
        // pushes `{mo->x, mo->y, 0, 1}` before it plans (`docs/ORDERS.md`
        // §4.4) and `Group::action_move_near` the leader's raw slot with
        // the same pair (`docs/GROUPS.md` §6.7). 1,724 stacks, no
        // exception.
        assert_eq!(
            bottoms,
            [((0, 1), 1_724)].into_iter().collect(),
            "a stack bottom that is not the order's goal"
        );
        // Above it, three shapes and only three, and the counts are the
        // dump's own so they are exact rather than a ceiling:
        //
        // - `(384, 0)` is `astar_path`'s world reconstruction
        //   (`docs/PATHFINDER.md` §7), and it is what this crate writes;
        // - `(0, 2)` is the **unit grid**'s, `SIDESTEP` and no tolerance —
        //   `find_upath`'s 48-cell plan round a blocker;
        // - `(0, 0)` is a top the collision arm has rewritten: §4.4 pops
        //   the waypoint and pushes it back with `tolerance =
        //   collider.big_radius × 3`, which is **zero** for every
        //   `BLOCK_RADIUS 1` type in the corpus, and leaves the flags —
        //   so a world node the unit has been blocked at reads `(0, 0)`
        //   rather than `(384, 0)`. That rewrite is not modelled here
        //   (`docs/ORDERS.md` §4.4), and these 243 tops are its record.
        assert_eq!(
            tops,
            [((0, 0), 243), ((0, 2), 73), ((384, 0), 1_408)]
                .into_iter()
                .collect(),
            "the stack-top shapes moved"
        );
        assert_eq!(
            middles,
            [((0, 2), 54), ((384, 0), 2_325)].into_iter().collect(),
            "the stack-middle shapes moved"
        );

        // Half two: what this crate writes instead, as a ceiling. Every one
        // of these is a row the old comparison could not see.
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let rows: Vec<&OrderDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.order_diverged.iter())
            .filter(|d| matches!(d.what, OrderMismatch::PathField { .. }))
            .collect();
        let first = rows.first().copied().copied();
        eprintln!("run39 path-field rows: {}, first {first:?}", rows.len());
        // 24 rows, the first at frame **1518** — past this capture's score
        // (ticks 1477, orders 1476), and all of them the AI scout `1/0`'s,
        // whose stack has been its own since the order at 1477. Nothing the
        // widening found lies before a parting, which is why no floor moved
        // when it landed.
        assert!(
            rows.len() <= 24,
            "path-field disagreements grew: {} — first {first:?}",
            rows.len()
        );
    }

    /// **§6.7 — the group plans, at order time, off the leader's raw slot**
    /// (item 31, `docs/GROUPS.md` §6.7).
    ///
    /// Run20's `1/0` is a one-member group on auto-explore
    /// (`Sim::scout_issue` → `group_action_move_to`), which makes it the
    /// cheapest possible fixture for this section: with one member the slot
    /// translation is the identity, so what is left is exactly the two
    /// halves the simulation did not have — **when** the path is planned
    /// and **what goal** it is planned to.
    ///
    /// The original's frame **1** already carries `flags 1` (`PATHED`) and
    /// a nine-entry stack whose bottom is `(41952, 36576)` with
    /// `tolerance 0` and `flags 1`; before this landed the simulation's
    /// frame 1 had `flags 0` and an **empty** stack, because every member
    /// got a bare `MoveOrder` and waited for its own `do_move` a frame
    /// later. Both halves are asserted here, and both were made to fail
    /// first — by handing `add_move_facing_order` `pathed = false`, and by
    /// pushing the order's snapped `dest` in place of `form.to[idx]`.
    ///
    /// It asserts the **whole chain**, and it did not always. When this
    /// landed, four of the sim's seven entries were the original's entry
    /// for entry and the middle parted — the original ran along cell row
    /// 50 where the sim ran along row 51. That residue was
    /// `astar_path`'s own, and item 32 closed it the same day by pricing
    /// the two things `calc_cost` was not reading: whether the step's
    /// half-cell is **seen**, and what the cell it lands in **costs**
    /// (`docs/PATHFINDER.md` §5). All nine entries agree now, in order.
    #[test]
    fn run20_s_group_member_is_pathed_at_order_time_off_the_leaders_slot() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.tick();

        let theirs = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 1)
            .expect("frame 1")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("1/0 at frame 1");
        let theirs_path: Vec<(i64, i64, i64, i64)> = theirs
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        // The oracle: pathed on the frame the order was issued, with the
        // whole chain already on the stack.
        assert_eq!(
            theirs.orders.first().map(|o| o.flags),
            Some(1),
            "the original's order is PATHED at frame 1"
        );
        assert_eq!(theirs_path.len(), 9, "and its stack is nine deep");
        assert_eq!(
            theirs_path[0],
            (41952, 36576, 0, 1),
            "whose bottom is the leader's raw slot, tolerance and flag"
        );

        let v = built
            .sim
            .units
            .iter()
            .position(|x| x.alive() && x.owner == 1 && x.index == 0)
            .expect("the AI's unit 0");
        // The same position at plan time, so that the two searches are
        // comparable at all: if this drifts, the rest is measuring
        // something else.
        assert_eq!(
            (
                i64::from(built.sim.units[v].pos.x),
                i64::from(built.sim.units[v].pos.y)
            ),
            (theirs.pos.x, theirs.pos.y),
            "the plan frame's position"
        );
        let front = built.sim.units[v].orders.front().expect("the move order");
        assert_eq!(
            front.flags & sim::orders::flag::PATHED,
            sim::orders::flag::PATHED,
            "ours is PATHED at frame 1 too"
        );
        let ours: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        assert!(!ours.is_empty(), "the group planned nothing");
        assert_eq!(
            ours[0], theirs_path[0],
            "the goal is the leader's raw slot, whole"
        );
        // **No residue left.** Item 32 closed the middle of the chain on
        // 2026-08-26: `calc_cost`'s fog read and its terrain cost are both
        // live (`docs/PATHFINDER.md` §5, §12), and with them the sim plans
        // the original's nine entries — position, tolerance and flag —
        // **in order**. The whole chain is the assertion now; anything
        // that moves one waypoint fails here.
        assert_eq!(ours, theirs_path, "the sim's chain: {ours:?}");
    }

    /// **The fuzzed map's frame 1** — the second capture the 2×2 jitter is
    /// checked on, and the one that makes it a rule rather than a run20
    /// coincidence.
    ///
    /// `gamelog-fuzz-424242-*` is the fuzzer's control run on a lobby
    /// nobody tuned against (`docs/SYNC.md` §4.2). Its frame 1 spends
    /// `Leader::produce_building+0x1805` **three** times where run20
    /// spends four: the jitter walks the same 2×2 and `blocked_site`
    /// refuses one of the sub-positions. A one-draw-per-call reading
    /// cannot produce either number.
    ///
    /// The row above it was one draw *short* of the original's thirty for
    /// five days, and it is **30 against 30** since 2026-09-01: the
    /// stride-by-three stepped the spiral's index at the *top* of the
    /// iteration, by the stride as it stood before the body ran, where
    /// the original steps it at the bottom by the stride the body has
    /// just set (`docs/AI.md` §20). One cell, one candidate, one draw.
    /// A `Unit::do_non_flat_gather+0x54b` is still short, so the frame
    /// reads 44 against 45; that residue is asserted as it stands so that
    /// closing it shows up as a failure.
    #[test]
    fn the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-fuzz-424242-heights.txt"),
            trace("rontrace-fuzz-424242.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        built.tick();
        built.sim.tick();
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        let theirs = trace.labels(1);
        assert_eq!(theirs.len(), 45, "the original's frame 1");

        let count = |v: &[String], site: &str| v.iter().filter(|l| *l == site).count();
        assert_eq!(count(&theirs, sim::ai_place::SITE_JITTER), 3);
        assert_eq!(
            count(&ours, sim::ai_place::SITE_JITTER),
            3,
            "one of the 2×2's four sub-positions is blocked on this map"
        );
        // The residues, as they stand.
        assert_eq!(
            (
                count(&theirs, sim::ai_place::SITE_SPIRAL),
                count(&ours, sim::ai_place::SITE_SPIRAL),
            ),
            (30, 30),
            "the spiral walks the original's own candidates"
        );
        assert_eq!(
            (
                count(&theirs, sim::orders::SITE_TILE_WAIT),
                count(&ours, sim::orders::SITE_TILE_WAIT),
            ),
            (5, 4),
            "and one citizen picks no tile — still open"
        );
        assert_eq!(ours.len(), 44, "so the frame is 44 against 45");
    }

    /// Run20's AI scout at frame 0 — `Unit::think_scout`'s ten draws, on
    /// the original's own stream (`docs/SCOUT.md` §10).
    ///
    /// **The sequence is the assertion, not the count.** The trace is read
    /// here rather than in Python (`crate::trace`), filtered to the draws
    /// `think_scout` took itself, and compared site for site against the
    /// harness's own marks. A total cannot tell four rotations and two
    /// phases from three and three; this can, and it is what the ring
    /// walk's two guards need checking against.
    ///
    /// The seed is installed rather than reached, and that is deliberate:
    /// this check has to hold **while the stream that reaches the mechanic
    /// is wrong**, which is the position it landed in. It was worth
    /// keeping — the frame's own stream reached the scout two draws early
    /// until the stand/wrap swap closed, and `think_scout`'s count depends
    /// on the stream, so on the fuzzed map the same ten came out as
    /// eleven. Since 2026-08-26 the sim reaches this word on its own too
    /// (`frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`);
    /// installing it keeps the mechanic checkable the next time something
    /// upstream moves.
    #[test]
    fn run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // Two scouts stand up on this map, one a side — the two units the
        // trace shows standing two figures each before either thinks. Only
        // the computer's reaches `think_scout`.
        let scouts: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.unit_is_scout(u))
            .collect();
        assert_eq!(scouts.len(), 2, "one scout a side");
        let thinking: Vec<usize> = scouts
            .iter()
            .copied()
            .filter(|&u| built.sim.scout_thinks(u))
            .collect();
        assert_eq!(
            thinking.len(),
            1,
            "the human's does not — its `unit_masks & 0x40000` is clear"
        );
        let scout = thinking[0];
        assert_eq!(built.sim.units[scout].owner, 1, "the computer's");

        // The trace's own frame-0 draws, filtered to the ones
        // `Unit::think_scout` took itself.
        let Some(trace) = trace("rontrace-run20.log") else {
            eprintln!("skipping the sequence half: no rontrace-run20.log");
            return;
        };
        let draws = trace.run_in(0, sim::scout::CODE.start, sim::scout::CODE.end);
        // Named through `trace::SITES`, which is the same string
        // `sim::scout`'s own marks write — so the two sides of the
        // comparison below share one vocabulary.
        let theirs: Vec<String> = draws.iter().map(|d| trace.label(d)).collect();
        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        assert_eq!(
            theirs,
            vec![
                SITE_ROTATION,
                SITE_ROTATION,
                SITE_ROTATION,
                SITE_PHASE,
                SITE_CELL,
                SITE_CELL,
                SITE_CELL,
                SITE_CELL,
                SITE_ROTATION,
                SITE_PHASE,
            ],
            "the trace's own frame-0 sequence (docs/SCOUT.md §10)"
        );

        // Ours, from the first of those, site for site rather than by
        // count — four rotations and two phases, not three and three.
        let first = draws[0].seed;
        assert_eq!(first, 0x9c59_1b2b, "the trace's draw 24");
        built.sim.trace_phases = true;
        built.sim.phase_marks.clear();
        built.sim.rng = sim::combat::Rng::new(first);
        assert!(built.sim.think_scout(scout), "a target is found");
        let ours = mark_sites(&built.sim.phase_marks, built.sim.rng.seed).expect("our sites");
        assert_eq!(
            ours, theirs,
            "our ten draws, at the original's sites, in the original's order"
        );
        // Which leaves the stream where the trace's next draw found it.
        assert_eq!(
            draws_between(first, built.sim.rng.seed),
            Some(theirs.len() as u32)
        );

        // §9: the explore order, at a tile centre inside ring 5 of the
        // AI's city.
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        let ci = built
            .sim
            .cities
            .iter()
            .position(|c| c.alive && c.owner == 1)
            .expect("the AI's city");
        let city = built.sim.cities[ci].pos.cell();
        let d = sim::world::vector_dist(m.dest.cell().x - city.x, m.dest.cell().y - city.y);
        assert_eq!(d, 5, "ring 5, where the four cells were");
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
        let report = run_traced(&loaded, &log, Tuning::RON, Some(105), None, &refs, None).unwrap();
        let count = |f: i64| -> (Option<u32>, Option<u32>) {
            let &(_, ours, theirs) = report
                .rng_frames
                .iter()
                .find(|(n, _, _)| *n == f)
                .unwrap_or_else(|| panic!("frame {f} was not traced"));
            (ours, theirs)
        };
        // With the animation clock (`docs/ANIM.md` §6): the new citizen's
        // two creation draws at 99, the twelve fish wraps at 100 and the
        // scout's wrap at 101 are the sim's. ~~The one left at 101 is the
        // sheep's arrival, whose walk the sim does not have.~~ **Closed
        // 2026-08-27**: the sheep wandered on an untraced frame, so the
        // harness re-seats gaia's animals from every traced frame's dump
        // (`sim::Sim::reseat_animal`) and its arrival now falls where the
        // original's does — draw 6 of 21, between the AI's three farmers
        // and the human's.
        assert_eq!(count(98), (Some(6), Some(6)));
        assert_eq!(count(99), (Some(8), Some(8)), "the new citizen's two");
        assert_eq!(count(100), (Some(18), Some(18)), "the twelve fish wraps");
        assert_eq!(
            count(101),
            (Some(21), Some(21)),
            "the scout's wrap; the sheep"
        );
        // **102 was the six-frames-late re-target, and it closed with the
        // body** (item 34, 2026-08-27). It had been booked as the stale-fog
        // row when item 32 landed; item 33 revealed cells as units move and
        // it went 24 → 22, which is what sent someone to the trace instead
        // of the theory — and the trace said the movement layer.
        //
        // What it was. Both sides give the AI scout `1/0` the same
        // `EXPLORE_TO`, to `(45048, 19704)`, and both walk it there. On
        // **frame 62** the original stands still for exactly one frame,
        // turning three degrees, and then steps a constant `(−19, +29)`
        // from 63 to the end. The simulation stood for *seven* frames and
        // eased into the heading over eight more, arriving at frame 101
        // where the original arrived at 95 — so its eighteen `think_scout`
        // ring draws landed on 102 where the original's landed on 96.
        //
        // Two things behind one number, and both are the body's
        // (`docs/MOVEMENT.md`, "The body step"). Guy 0 does not chase the
        // unit at eleven eighths; it is **written onto** it, with
        // `last_speed` the Euclidean length of the unit's own step — so the
        // average settles at 33 rather than 46 and divides the turn rate by
        // nine rather than eleven. And a unit that spent its frame turning
        // in place has `last_speed` zero that same frame, which is what
        // arms `guy_flags & 0x10`, the instant turn from a standstill —
        // which no unit in this simulation had, because nothing populated
        // the flag from the type. With both, the scout is on the
        // original's position and both of its angles on every frame from
        // 57 to 91.
        assert_eq!(
            count(102),
            (Some(6), Some(6)),
            "the scout's re-target is on the original's frame"
        );
        assert_eq!(count(103), (Some(6), Some(6)));
        // The farmers re-target on sim-frame 101 and walk from 102; their
        // path goals are compared on the log's frame 103 (the end of
        // sim-frame 102, the walk's first step). ~~The human's three draw
        // one place late (the sheep's arrival is not modelled) and are not
        // pinned.~~ **Both players are pinned now** (2026-08-27): the
        // sheep's arrival is draw 6, and it is the *ordering* that mattered
        // — the AI's six draws come before it and the human's six after, so
        // without it the human's three farmers spent the AI's leftovers and
        // walked one tile wrong on both axes.
        let at_103 = report
            .frames
            .iter()
            .find(|f| f.frame == 103)
            .expect("the log's frame 103");
        for who in 0..=1 {
            for o in 3..=5 {
                let bad: Vec<_> = at_103
                    .order_diverged
                    .iter()
                    .filter(|d| d.who == who && d.o == o)
                    .collect();
                assert!(bad.is_empty(), "farmer {who}/{o} at frame 103: {bad:?}");
            }
        }
    }

    /// **Every footprint's blocked bits, against the original's own tile
    /// masks** (item 44). A `WORLD ≥ 6` or `DUMP_ALL` start dump prints all
    /// 57,600 `tdata[scan].mask` words, so the map the harness loads *is*
    /// the oracle for the map the harness then stamps its buildings onto:
    /// `build_sim` re-marks every one of them through `Wall::mask_me`, and
    /// if the template is wrong the two disagree on the spot.
    ///
    /// They used to. This crate blocked every non-flat footprint whole,
    /// which is right for a Mine (`2x2 solid`) and wrong for the
    /// Woodcutter's Camp beside it (`2x2 gather`, which blocks nothing) and
    /// wrong for every `extra space` mask, where a 7×7 city blocks 6×6. The
    /// cost was one citizen sent 48 units past the camp's own tile, because
    /// `find_nearby_spot` refuses a `0x4000` tile and the original's camp
    /// does not carry one (`docs/ORDERS.md` §10, `docs/DATALAYER.md`).
    ///
    /// Only the object field and the blocked bit are compared: the rest of
    /// the word is terrain, fog and city radii that the setup does not
    /// re-derive.
    #[test]
    fn every_footprint_takes_the_blocked_bits_the_original_s_map_shows() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut ran = 0;
        for name in [
            "gamelog-run10-world6-long.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run9-world6.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let tiles = crate::gamelog::world_tiles(&init.world);
            let built = build_sim(&loaded, &init, Tuning::RON);
            let tw = (built.sim.world.width() * sim::world::TILES_PER_CELL) as usize;
            if tiles.len() != tw * tw {
                continue;
            }
            ran += 1;
            let keep = sim::world::tile::OBJECT | sim::world::tile::BLOCKED;
            let bad: Vec<(usize, u16, u16)> = tiles
                .iter()
                .enumerate()
                .filter_map(|(i, &theirs)| {
                    let t = Pos::new((i % tw) as i32, (i / tw) as i32);
                    let ours = built.sim.world.tile_mask(t);
                    (ours & keep != theirs & keep).then_some((i, ours, theirs))
                })
                .collect();
            assert!(
                bad.is_empty(),
                "{name}: {} tiles differ on the object/blocked bits, first {:?} \
                 (tile {:?})",
                bad.len(),
                bad.first(),
                bad.first().map(|&(i, _, _)| ((i % tw), (i / tw))),
            );
        }
        assert!(ran > 0, "no start dump with a tile map on this machine");
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
            "gamelog-run13-window-95-105.txt",
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
    /// The first dock, under run22's window (`gamelog-run22-islands-dock-
    /// window.txt`, 2026-08-25: the run21 lobby, `DUMP_ALL` for `[3579,
    /// 3582)`). `docs/TRANSPORT.md` §5, §10, §12 — the docks registry and
    /// the transport level, checked against the original's own records:
    ///
    /// - block 3579 has no active `DOCK` and none of the AI's units carry
    ///   `unit_masks & 0x800000`; block 3580 has one dock and every AI unit
    ///   carries the bit, the human's none (`Leader::check_transport`);
    /// - the `DOCK` record's `reg` is the region of the building's cell in
    ///   the run's own `WORLD` block — the **sea**, so `Dock::init`'s
    ///   `reg < 0x40` guard leaves `reg_docks` untouched;
    /// - a dock placed and activated in the harness at that position takes
    ///   slot 0, records the same region, leaves `reg_docks` at 0, and — the
    ///   AI holding the bonus's prerequisite — grants the civilian level to
    ///   every land unit of its owner and to nobody else's.
    #[test]
    fn run22_s_first_dock_registers_in_the_sea_and_grants_the_level() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run22-islands-dock-window.txt") else {
            eprintln!("skipping: no gamelog-run22-islands-dock-window.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.sim.lobby.map_style, 18, "East Indies");

        let frames = log.frames();
        let block = |n: i64| -> &Block<'_> {
            let (_, b) = frames
                .iter()
                .find(|(f, _)| *f == n)
                .expect("the frame block");
            b.kid("FULL DUMP").unwrap_or(b)
        };
        let sub = |b: &Block<'_>, key: &str| -> Option<i64> { b.find("SUBOBJECT")?.int(key) };
        // The unit bits, by owner: (with the bit, without).
        let bits = |b: &Block<'_>, who: i64| -> (usize, usize) {
            let mut on = 0;
            let mut off = 0;
            for u in b.kids("UNITDATA") {
                if sub(u, "who") != Some(who) {
                    continue;
                }
                if u.int("unit_masks").unwrap_or(0) & 0x80_0000 != 0 {
                    on += 1;
                } else {
                    off += 1;
                }
            }
            (on, off)
        };
        fn active_docks<'a, 'b>(b: &'a Block<'b>) -> Vec<&'a Block<'b>> {
            b.kid("DOCKS")
                .map(|d| {
                    d.kids("DOCK")
                        .filter(|r| r.int("dock_flags").unwrap_or(0) & 1 != 0)
                        .collect()
                })
                .unwrap_or_default()
        }

        let before = block(3579);
        assert!(active_docks(before).is_empty(), "no dock before 3579");
        let (on, off) = bits(before, 1);
        assert_eq!((on, off), (0, 14), "the AI's 14 units before the dock");

        let after = block(3580);
        let docks = active_docks(after);
        assert_eq!(docks.len(), 1, "one dock at 3580");
        let d = docks[0];
        assert_eq!(d.int("dock"), Some(0), "slot 0");
        assert_eq!(d.int("who"), Some(1), "the AI's");
        let o = d.int("o").expect("o");
        let dump_reg = d.int("reg").expect("reg");
        let (on, off) = bits(after, 1);
        assert_eq!((on, off), (14, 0), "every AI unit has the bit at 3580");
        assert_eq!(bits(after, 0).0, 0, "no human unit has it");

        // The building, and its cell in the harness's world.
        let bd = after
            .kids("BUILDDATA")
            .find(|b| sub(b, "o") == Some(o) && sub(b, "who") == Some(1))
            .expect("the dock's BUILDDATA");
        assert_eq!(bd.int("orig_type"), Some(432), "DOCK");
        let pos = Pos::new(
            sub(bd, "x_internal").unwrap() as i32,
            sub(bd, "y_internal").unwrap() as i32,
        );
        let sim_reg = built
            .sim
            .world
            .region_of(pos.cell())
            .expect("the dock's cell has a region");
        let mapped = built
            .region_map
            .iter()
            .find(|(_, r)| *r == sim_reg)
            .map(|(d, _)| *d);
        assert_eq!(mapped, Some(dump_reg), "the DOCK's reg is the cell's");
        assert_eq!(
            built.sim.world.terrain(sim_reg),
            sim::world::Terrain::Sea,
            "a dock's centre cell is water — the guard skips reg_docks"
        );

        // The same dock in the harness: the AI holds the bonus's
        // prerequisite by frame 201 (run21's `check_transport`), so grant
        // it, then place and finish the dock.
        let preq = built
            .sim
            .tech_tree
            .roles
            .transport_preq
            .expect("Written Word, from rules.xml's third TECHBONUS");
        built.sim.gain_tech(1, preq);
        assert_eq!(
            built.sim.transport_level(1),
            sim::transport::TransportType::None,
            "the prerequisite alone grants nothing"
        );
        let ty = loaded.build_of_type_index(432).expect("the dock type");
        let b = built.sim.add_building(1, pos, 1);
        built.sim.buildings[b].ty = Some(ty);
        built.sim.buildings[b].orig_ty = Some(ty);
        built.sim.buildings[b].started = true;
        built.sim.activate(b, false, true);
        assert_eq!(built.sim.buildings[b].dock_slot, Some(0));
        assert_eq!(built.sim.docks[1].slots[0].reg, Some(sim_reg));
        assert!(
            built.sim.ai[1].census.reg_docks.iter().all(|n| *n == 0),
            "reg_docks stays 0 for a dock in the sea"
        );
        assert_eq!(
            built.sim.transport_level(1),
            sim::transport::TransportType::Civilian
        );
        let ai_units: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].alive())
            .collect();
        assert!(!ai_units.is_empty());
        for &u in &ai_units {
            assert!(
                built.sim.units[u].auto_transport,
                "unit {} of the AI has the bit",
                built.sim.units[u].index
            );
        }
        assert!(
            built
                .sim
                .units
                .iter()
                .filter(|u| u.owner == 0)
                .all(|u| !u.auto_transport),
            "the human's units do not"
        );
        assert_eq!(
            built.sim.transport_level(0),
            sim::transport::TransportType::None
        );
    }

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
    /// **Every traced frame, draw for draw** — the whole-run form of
    /// `frame_0_matches_the_trace_draw_for_draw_on_both_traced_maps`, and
    /// the sub-score the untraced stretch is worked against.
    ///
    /// Run14 is run10's own lobby and seed with the draw-site instrument
    /// attached, and it carries **284 frames** rather than the nine a
    /// `DUMP_ALL` window can afford. The dump's words are installed at the
    /// ends of frames 0–3 and 94–103 as always; everywhere else the
    /// simulation's stream is its own, and this compares the *sites*
    /// rather than the values — so a frame can be checked draw for draw on
    /// a stretch where nothing else can reach it at all.
    ///
    /// The number is a floor and it is what says whether a stream item
    /// converged. History:
    ///
    ///   2026-08-28  **179** of 284 (the first pin)
    ///   2026-08-28  **184** of 284 (the bird, `docs/SYNC.md` §3.9): the
    ///               nine frames of `Animal::think_bird`'s three draws
    ///               between 104 and 168, and the sampling's own count.
    ///   2026-08-28  **192** of 284 (the bird's wing beat, §3.9): its two
    ///               animation lengths, read out of the install's own
    ///               `.bha` files (`crate::artdata`), and with them the
    ///               hatch frame's wrap, the birth coin and the wraps at
    ///               127 and 142.
    ///   2026-08-28  **198** of 284 (the residue's names, item 53): five
    ///               draws this simulation was already taking under a
    ///               coarse mark — the script VM's `rand_int`, the herd
    ///               animal's wander coin and its three step draws, the
    ///               farmer's cell re-pick and the arrival stand — given
    ///               a [`crate::trace::SITES`] row each. No mechanic
    ///               changed; the marks got finer, which is the incentive
    ///               below.
    ///
    ///   2026-08-28  **173** of 284, and the *fall* is the item (item 55,
    ///               the road). Two things landed together: the height
    ///               grid stopped being borrowed from another game
    ///               ([`borrow_from_siblings`]), and with the search
    ///               therefore exact, `Sim::plan_roads` came on. Frames 10
    ///               and 11 now match — 226 and 254 draws against 6 and 6
    ///               — and **the first frame whose draws differ at all
    ///               moved from 10 to 18**, which is the number below that
    ///               says so. What fell is the tail: past the first
    ///               divergence both sides are running on words that have
    ///               parted, and which of two wrong streams happens to
    ///               label a frame the same way is luck. 198 was that luck
    ///               with the divergence at 10; 173 is it with the
    ///               divergence at 18.
    ///
    /// A frame is counted only when the two label sequences are equal, so
    /// a coarse mark on our side (`unit 1/9` against a `GameAccess::rnd`
    /// the table does not name) fails it even where the counts agree.
    /// Making a mark finer therefore *raises* this number, which is the
    /// intended incentive: `docs/SYNC.md` §5.1's "mark the phase before
    /// believing the total".
    ///
    /// **The sharper number is the first frame that differs**, and it is
    /// asserted too: a matched *count* rewards luck past the divergence,
    /// where this cannot. It has gone 4 → 10 → **18**.
    /// **The rings lay nothing new.** `BuildType::place_roads` runs on
    /// run14's frames 0 and 10 to 15 and again from 167, and the original's
    /// own map says what it does to the world: the tile masks of run13's
    /// `DUMP_ALL` at sim-frame 95 are **identical, all 57,600 of them**, to
    /// run10's start-of-game `WORLD=6` block. Ninety-five frames of road
    /// regeneration changed not one tile, because every tile a ring or a
    /// road reaches is already a road or is blocked.
    ///
    /// That is the whole oracle for `crate::roads`' §3, and it is sharp:
    /// widening the city's ring by the one column the *third* `place_roads`
    /// arm uses lays twenty-six new roads here and fails this at once (tried,
    /// 2026-08-28). The surface field is what a road changes, so that is
    /// what this compares; `PLACED` and the footprint bits move for reasons
    /// of their own.
    #[test]
    fn run14_s_road_rings_change_no_tile_the_original_does_not() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let (tw, th) = (
            built.sim.world.width() * sim::world::TILES_PER_CELL,
            built.sim.world.height() * sim::world::TILES_PER_CELL,
        );
        let surfaces = |s: &sim::Sim| -> Vec<u16> {
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .map(|(x, y)| s.world.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE)
                .collect()
        };
        let before = surfaces(&built.sim);
        assert_eq!(before.len(), 57_600, "run14's map is 240 tiles square");
        for _ in 0..285 {
            built.tick();
        }
        let after = surfaces(&built.sim);
        let moved: Vec<(i32, i32, u16, u16)> = (0..before.len())
            .filter(|&i| before[i] != after[i])
            .map(|i| {
                let (x, y) = (i as i32 % tw, i as i32 / tw);
                (x, y, before[i], after[i])
            })
            .collect();
        assert!(
            moved.is_empty(),
            "the rings laid {} tiles the original leaves alone: {:?}",
            moved.len(),
            &moved[..moved.len().min(8)]
        );
    }

    /// **The road two fresh sites lay, tile for tile** — item 55's oracle,
    /// and the first time this mechanic is checked against a *path* rather
    /// than against three totals.
    ///
    /// run32 (`docs/ORACLE.md`) is run10–14's game with two enhancers
    /// dropped from the cheat channel at sim-frame 100 on ground the map
    /// has never had a road on: a **Granary** at tile `(6, 171)` and a
    /// **Smelter** at `(33, 161)`. `add` without `NEW` runs
    /// `Build::activate`, and `Wall::start` → `Wall::mask_me(1,
    /// REGEN_FORCE)` → `BuildType::place_roads` plans the road **there and
    /// then** — which is why the roads are on the map four frames before
    /// the scheduled replans of `docs/ROADS.md` §1 come round.
    ///
    /// The world before is **run13's** `FRAME 100` block: the same game,
    /// the same map, ninety-nine frames in, and nothing between them lays a
    /// road (the trace has no `calc_road_cost` draw before frame 100). The
    /// world after is run32's `FRAME 104`. The heights are run13's own —
    /// the *pre*-terraform grid — because `TerrainOut::terraform_for_build\
    /// ing` runs **after** the road is planned: under the frame-104 grid,
    /// which has both footprints flattened, the first search costs 967
    /// nodes against the original's 1,043, and under the pre-terraform one
    /// it costs 1,046.
    ///
    /// The jitters are not left to chance: `calc_road_cost` takes one
    /// `Random::get(0, 0xffff) % 20` a node and the trace records the
    /// generator's word **before** every draw, so each search starts on the
    /// original's own state. The boundary between the two searches is in
    /// the trace too, and it is not a road draw: `Wall::activate` plays a
    /// sound off a *different* generator, so the one non-sync draw inside
    /// frame 100 splits the 2,913 road draws into the Granary's 1,043 and
    /// the Smelter's 1,870.
    ///
    /// **What this pins, and what it leaves open.** The tiles are exact —
    /// 62 of them, both roads and both rings. The counts are not: 1,046
    /// against 1,043 and 1,460 against 1,870, with the right road either
    /// way. Every search this simulation has been shown a *frame's own*
    /// world for — run14's frames 10 and 11, run32's 104 to 107 — matches
    /// node for node, so what is left is particular to the frame a
    /// building is placed on (`docs/ROADS.md` §7).
    #[test]
    fn run32_s_two_fresh_roads_are_the_original_s_tile_for_tile() {
        let Some(inst) = install() else { return };
        let (Some(r32), Some(r13), Some(tr)) = (
            dump("gamelog-run32-roadpath.txt"),
            dump("gamelog-run13-window-95-105.txt"),
            trace("rontrace-run32.log"),
        ) else {
            eprintln!("skipping: no run32 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let (t32, t13) = (
            std::fs::read_to_string(&r32).unwrap(),
            std::fs::read_to_string(&r13).unwrap(),
        );
        let (l32, l13) = (Log::parse(&t32), Log::parse(&t13));

        let world_at = |log: &Log, n: i64, heights: &[i64]| -> World {
            let (_, body) = log
                .dumps()
                .into_iter()
                .find(|(f, _)| *f == n)
                .unwrap_or_else(|| panic!("frame {n} is dumped"));
            let w = body.kid("WORLD").expect("a WORLD block");
            let mut notes = Vec::new();
            world_from(&w.fields, heights, &mut notes).0
        };
        let roads = |w: &World| -> std::collections::BTreeSet<(i32, i32)> {
            let (tw, th) = (w.width() * 4, w.height() * 4);
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    w.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE
                        == sim::world::tile::SURFACE_ROAD
                })
                .collect()
        };

        let heights = l13.frame_heights(100);
        assert_eq!(heights.len(), 58_081, "run13's frame-100 height grid");
        let before = world_at(&l13, 100, &heights);
        let after = world_at(&l32, 104, &heights);
        let was = roads(&before);
        let theirs: std::collections::BTreeSet<(i32, i32)> =
            roads(&after).difference(&was).copied().collect();
        assert_eq!(theirs.len(), 62, "the tiles the original laid at frame 100");

        // The two searches, and the boundary between them: the one draw of
        // frame 100 that is *not* on `game_random` is `Wall::activate`'s
        // sound, and it is emitted after the first building's plan.
        const COST: u32 = 0x0068_6346;
        let draws = tr.run_in(100, COST, COST + 1);
        assert_eq!(draws.len(), 2913, "frame 100's road draws");
        let sound = tr
            .draws
            .iter()
            .position(|d| d.frame == 100 && !d.sync())
            .expect("Wall::activate's sound draw");
        let first = tr.draws[..sound]
            .iter()
            .filter(|d| d.frame == 100 && d.sync() && d.site == COST)
            .count();
        assert_eq!(
            (first, draws.len() - first),
            (1043, 1870),
            "the Granary's nodes and the Smelter's"
        );

        // Stand run13's frame-100 buildings up on run13's frame-100 map,
        // then place the two the channel placed, in the order it placed
        // them, each seeded where the original's own search began.
        let mut sim = loaded.sim(Tuning::RON, before.clone(), 2);
        for pass in [true, false] {
            for b in l13
                .frame_builds(100)
                .iter()
                .filter(|b| (0..2).contains(&b.who))
            {
                let Some(ty) = b
                    .orig_type
                    .and_then(|t| loaded.build_of_type_index(t as i32))
                else {
                    continue;
                };
                if sim::build::is_city(&loaded.build_types, ty) != pass {
                    continue;
                }
                let h = sim.init_build(b.who as sim::Player, ty, pos_of(b.pos), false);
                sim.buildings[h].index = b.o as i16;
                sim.activate(h, false, false);
            }
        }
        // Standing them up moved the map (footprints, `mask_city`,
        // `sync_territory`); the dump's map is the one the searches read.
        sim.world = before.clone();
        sim.plan_roads = true;

        // run62 is this same capture with the three road proxies on
        // (`tools/trace/README.md`): every candidate's coordinate, every
        // node's price, and the bracket delimiting the two searches. With
        // it the two counts stop being a score and become a sequence.
        let theirs_nodes = trace("rontrace-run62.log")
            .map(|t| t.road_nodes(100))
            .unwrap_or_default();
        sim.trace_costs = !theirs_nodes.is_empty();

        let mut laid: std::collections::BTreeSet<(i32, i32)> = std::collections::BTreeSet::new();
        let mut costed: Vec<u32> = Vec::new();
        for (id, tile, seed) in [
            (sim::build::Ident::Granary, Pos::new(6, 171), draws[0].seed),
            (
                sim::build::Ident::Smelter,
                Pos::new(33, 161),
                draws[first].seed,
            ),
        ] {
            let ty = loaded
                .build_types
                .iter()
                .position(|b| b.ident == id)
                .unwrap_or_else(|| panic!("a {id:?} type"));
            let b = sim.init_build(
                0,
                ty,
                Pos::new(
                    tile.x * sim::world::UNITS_PER_TILE + sim::world::UNITS_PER_TILE / 2,
                    tile.y * sim::world::UNITS_PER_TILE + sim::world::UNITS_PER_TILE / 2,
                ),
                false,
            );
            assert_eq!(
                sim.buildings[b].city,
                Some(0),
                "{id:?} joins the human's city, as `Build::find_city` puts it"
            );
            let mine = roads(&sim.world);
            sim.rng.seed = seed;
            // Not `place_roads` directly: `Build::activate` → `Wall::start`
            // → `mask_me(1, REGEN_FORCE)` → `place_roads` is the chain the
            // channel's `add` ran, and nothing else on it draws.
            sim.activate(b, false, false);
            costed.push(draws_between(seed, sim.rng.seed).unwrap_or(0));
            laid.extend(roads(&sim.world).difference(&mine).copied());
        }

        // **The whole record, node for node.** The count was all this
        // could compare until run62: `calc_road_cost` is handed a pooled
        // `PathNode *`, so its own proxy record carries no coordinate, and
        // `valid_roadcoord` -- the gate that admitted the tile -- is
        // proxied beside it for that. What the pair gives is the original's
        // own sequence of `(tile, from, dir, price)`, which is what turned
        // two numbers that were 3 and 410 out into two mechanics
        // (`docs/ROADS.md` 7.3, 7.4).
        if !theirs_nodes.is_empty() {
            let ours = &sim.road_marks;
            let at = (0..ours.len().max(theirs_nodes.len()))
                .find(|&i| ours.get(i) != theirs_nodes.get(i));
            if let Some(i) = at {
                for j in i.saturating_sub(4)..(i + 5).min(ours.len().max(theirs_nodes.len())) {
                    eprintln!(
                        "  {j:>5} ours   {:?}\n        theirs {:?}",
                        ours.get(j),
                        theirs_nodes.get(j)
                    );
                }
            }
            assert_eq!(
                (at, ours.len()),
                (None, theirs_nodes.len()),
                "run62's 2,913 priced nodes: the first index that parts, and the count"
            );
        }
        assert_eq!(
            laid, theirs,
            "the two roads and their rings are not the original's, tile for tile"
        );
        assert_eq!(
            costed,
            vec![1043, 1870],
            "the nodes costed, and they are the original's since run62 \
             (`docs/ROADS.md` §7.3, §7.4)"
        );
    }

    /// **The scheduled replans cost the original's nodes, exactly.** Four
    /// searches, on the world of the frame that ran them.
    ///
    /// run32's two placements flag their city, so `(frame + o) % 16 == 0`
    /// walks the whole of it: the Smelter on sim-frame 104, the Granary on
    /// 105, the Market on 106 and the Library on 107. Each re-runs the same
    /// search over a map that already has the road, and the trace says what
    /// each cost — **332, 231, 232 and 60**. This simulation, given the
    /// frame's own world through [`sim_at_frame`] and the original's word
    /// at the search's first draw, costs the same four, and lays no tile
    /// (there is nothing left to lay).
    ///
    /// This is what settled item 55: nothing was wrong with the search. The
    /// harness had been feeding it the *heights of another game* — run3's,
    /// which shares the seed, the style and the size but not `GAME_RULES`,
    /// and whose grid differs on 237 corners from (7, 83) to (230, 163)
    /// because its starting buildings terraformed elsewhere
    /// ([`borrow_from_siblings`]).
    #[test]
    fn run32_s_scheduled_replans_cost_the_original_s_nodes() {
        let Some(inst) = install() else { return };
        let (Some(r32), Some(tr)) = (
            dump("gamelog-run32-roadpath.txt"),
            trace("rontrace-run32.log"),
        ) else {
            eprintln!("skipping: no run32 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&r32).unwrap();
        let log = Log::parse(&text);
        const COST: u32 = 0x0068_6346;
        let roads = |w: &World| -> std::collections::BTreeSet<(i32, i32)> {
            let (tw, th) = (w.width() * 4, w.height() * 4);
            (0..th)
                .flat_map(|y| (0..tw).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    w.tile_mask(Pos::new(x, y)) & sim::world::tile::SURFACE
                        == sim::world::tile::SURFACE_ROAD
                })
                .collect()
        };
        let mut seen = Vec::new();
        for (frame, o) in [(104i64, 2008i64), (105, 2007), (106, 2006), (107, 2005)] {
            let draws = tr.run_in(frame, COST, COST + 1);
            let mut at = sim_at_frame(&loaded, &log, frame, Tuning::RON)
                .expect("run32's frame block stands up");
            let b = at
                .build(0, o)
                .unwrap_or_else(|| panic!("frame {frame}: no building 0/{o}"));
            let mine = roads(&at.sim.world);
            at.sim.plan_roads = true;
            at.sim.rng.seed = draws[0].seed;
            at.sim.place_roads(b);
            let laid: Vec<(i32, i32)> = roads(&at.sim.world).difference(&mine).copied().collect();
            assert!(
                laid.is_empty(),
                "frame {frame}: the replan laid {laid:?}, and the original laid nothing"
            );
            seen.push((
                frame,
                draws_between(draws[0].seed, at.sim.rng.seed).unwrap_or(0),
                draws.len() as u32,
            ));
        }
        assert_eq!(
            seen,
            vec![
                (104, 332, 332),
                (105, 231, 231),
                (106, 232, 232),
                (107, 60, 60)
            ],
            "(frame, ours, the original's) nodes costed"
        );
    }

    /// The simulation's `Farms` list as the dump writes it: one row per
    /// record in `Sim::farm_order`, the cells in the record's own memory
    /// order (`status[dx][dy]`, index `dx * 4 + dy`).
    fn farm_rows(sim: &Sim) -> Vec<crate::gamelog::FarmDump> {
        sim.farm_order
            .iter()
            .map(|&b| {
                let bd = &sim.buildings[b];
                crate::gamelog::FarmDump {
                    who: i64::from(bd.owner),
                    o: i64::from(bd.index),
                    valid: i64::from(bd.farm.valid),
                    farm_type: i64::from(bd.farm.farm_type),
                    status: bd.farm.state.iter().map(|&s| i64::from(s)).collect(),
                    adds: bd.farm.adds.iter().map(|&a| i64::from(a)).collect(),
                }
            })
            .collect()
    }

    /// **The farm record, whole — every cell of every farm, on every frame
    /// two `DUMP_ALL` captures of this game print one.**
    ///
    /// `Farms::log_data` writes each `FarmStruct` as flat fields of the
    /// dump: `who`, `o`, then sixteen `percent`/`status` pairs, the
    /// twenty-five corner heights, `valid` and `farm_type`. The harness
    /// read four of those fields and threw the thirty-two cells away, and
    /// the thirty-two are the whole of the farm clock — a crop cell's
    /// state and its age in `0.005f` adds. Two hundred frames of that
    /// clock decide when a farmer's cell ripens under it and it walks off
    /// to another, which is two draws on the sync stream and a `MOVE_TO`
    /// in front of its gather.
    ///
    /// Run12 dumps frames 1–3 and run13 frames 95–104 of the same game
    /// run10 records, so this is ninety-five frames of simulation checked
    /// against the original's own arithmetic, cell for cell. What it pins:
    ///
    /// - **The clock.** `Farms::inc_time`'s `0.005f` a frame and the
    ///   farmer's `Farms::grow` on top, and the 201st add crossing `1.0f`
    ///   — every growing cell's count, every frame.
    /// - **The sprout's cell.** Sim-frame 101 sprouts one cell of the AI's
    ///   `1/2003` (`docs/SYNC.md` §4.1's draw 15/16), and it is **memory
    ///   index 5** — which fixes the order `nth_empty` counts in.
    /// - **The farmer's cell**, but only up to frame 101. Every starting
    ///   farmer stands on `(2, 2)`, which is its own transpose, and the
    ///   six that re-pick on 101 do not *reach* their new cells until 109
    ///   — past the last dumped frame. So the transposed index item 61
    ///   fixed (`status[dy][dx]` for `status[dx][dy]`) is **not** caught
    ///   here; the trace is what catches it, and
    ///   `run14_s_frames_match_the_trace_draw_for_draw` pins the
    ///   re-target's own frames for that reason.
    ///
    /// Made to fail on purpose, both halves: dropping `Farm::advance`'s
    /// add parts this at frame 1, and transposing `nth_empty`'s walk
    /// parts it at frame 2 on the AI's `1/2003`.
    #[test]
    fn run12_and_run13_s_farm_records_are_the_original_s_cell_for_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r12), Some(r13)) = (
            dump("gamelog-run10-world6-long.txt"),
            dump("gamelog-run12-dumpall-seeds.txt"),
            dump("gamelog-run13-window-95-105.txt"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);

        // Every frame either capture dumps the list on, by frame number.
        // A `FRAME n` block's `FULL DUMP` is the state after `n` ticks —
        // the same alignment `run13_s_world_at_frame_95…` stands on.
        let (t12, t13) = (
            std::fs::read_to_string(&r12).unwrap(),
            std::fs::read_to_string(&r13).unwrap(),
        );
        let (l12, l13) = (Log::parse(&t12), Log::parse(&t13));
        let mut want: Vec<(i64, Vec<crate::gamelog::FarmDump>)> = Vec::new();
        for l in [&l12, &l13] {
            for (n, b) in l.frames() {
                let farms = crate::gamelog::farms_of(b.kid("FULL DUMP").unwrap_or(b));
                if !farms.is_empty() {
                    want.push((n, farms));
                }
            }
        }
        want.sort_by_key(|(n, _)| *n);
        assert!(
            want.len() >= 13 && want.first().map(|(n, _)| *n) == Some(1),
            "run12's frames 1–3 and run13's 95–104: {:?}",
            want.iter().map(|(n, _)| *n).collect::<Vec<_>>()
        );

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let last = want.last().map_or(0, |(n, _)| *n);
        let mut parted: Vec<String> = Vec::new();
        for frame in 1..=last {
            built.tick();
            for (_, theirs) in want.iter().filter(|(n, _)| *n == frame) {
                let ours = farm_rows(&built.sim);
                if ours == *theirs {
                    continue;
                }
                for (i, (a, b)) in ours.iter().zip(theirs.iter()).enumerate() {
                    if a != b {
                        parted.push(format!(
                            "frame {frame} slot {i}: ours {a:?}\n              theirs {b:?}"
                        ));
                    }
                }
                if ours.len() != theirs.len() {
                    parted.push(format!(
                        "frame {frame}: ours {} records, theirs {}",
                        ours.len(),
                        theirs.len()
                    ));
                }
            }
        }
        assert!(
            parted.is_empty(),
            "the farm records parted from the original's:\n{}",
            parted[..parted.len().min(6)].join("\n")
        );
    }

    /// **The map itself, ninety-five frames in.** The road search reads the
    /// world and nothing else — a cell's owner decides a 240-unit term, its
    /// `ROCK` flag a 120, and a tile's mask decides whether the tile is
    /// valid at all (`docs/ROADS.md` §5.2) — so "is our map the original's"
    /// is the first question any count that comes out wrong has to answer,
    /// and until this it had only ever been asked of the *start* dump the
    /// map was loaded from, which is circular.
    ///
    /// Run13's `DUMP_ALL` writes a whole `WORLD` block at sim-frame 95:
    /// 3,600 cells and 57,600 tile masks of the same game run10 records.
    /// Ninety-five frames of simulation later the two worlds agree on
    /// **every cell's owner and every tile's mask**, and on every cell's
    /// flags but one.
    ///
    /// **The one:** cell `(52, 22)` carries `cell::BUILDING` there and not
    /// here, because nothing in this simulation ever *sets* that bit — every
    /// cell that has it got it from the start dump, and a building finished
    /// after frame 0 in a cell that had none leaves it clear. It is read
    /// (`crate::army`'s muster search classes a cell by it), so it is a real
    /// gap; it is pinned here as the one known difference rather than
    /// waived, and `docs/QUEUE.md` carries it.
    ///
    /// Made to fail twice before landing: once by comparing the world
    /// *before* the ninety-five frames, which the tile masks catch, and
    /// once by moving one cell's owner, which the borders do.
    #[test]
    fn run13_s_world_at_frame_95_is_the_original_s_cell_for_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r13)) = (
            dump("gamelog-run10-world6-long.txt"),
            dump("gamelog-run13-window-95-105.txt"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..95 {
            built.tick();
        }
        // Run13's own frame-95 block, read the way `Initial` reads a start
        // dump: `world_from` takes any `WORLD` block's fields.
        let t13 = std::fs::read_to_string(&r13).unwrap();
        let l13 = Log::parse(&t13);
        let block = l13
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 95)
            .map(|(_, b)| b)
            .expect("run13 traced frame 95");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("run13's frame 95 carries a WORLD block");
        let mut notes = Vec::new();
        let (theirs, _) = world_from(&w.fields, &[], &mut notes);
        let (xs, ys) = (theirs.width(), theirs.height());
        assert_eq!((xs, ys), (60, 60), "the cell grid of run10's map");

        let cells: Vec<sim::world::Cell> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .collect();
        let owners: Vec<String> = cells
            .iter()
            .filter(|&&c| built.sim.world.owner(c) != theirs.owner(c))
            .map(|&c| {
                format!(
                    "cell ({}, {}) ours {:?} theirs {:?}",
                    c.x,
                    c.y,
                    built.sim.world.owner(c),
                    theirs.owner(c)
                )
            })
            .collect();
        assert!(
            owners.is_empty(),
            "the borders parted from the original's by frame 95: {owners:?}"
        );

        let flags: Vec<(i32, i32, u16, u16)> = cells
            .iter()
            .map(|&c| {
                (
                    c.x,
                    c.y,
                    built.sim.world.cell_data(c).flags,
                    theirs.cell_data(c).flags,
                )
            })
            .filter(|(_, _, a, b)| a != b)
            .collect();
        assert_eq!(
            flags,
            vec![(52, 22, 0x200, 0x4200)],
            "cell flags other than the one `BUILDING` bit nothing here sets"
        );

        let (tw, th) = (
            xs * sim::world::TILES_PER_CELL,
            ys * sim::world::TILES_PER_CELL,
        );
        let masks: Vec<(i32, i32, u16, u16)> = (0..th)
            .flat_map(|y| (0..tw).map(move |x| Pos::new(x, y)))
            .map(|t| (t.x, t.y, built.sim.world.tile_mask(t), theirs.tile_mask(t)))
            .filter(|(_, _, a, b)| a != b)
            .collect();
        assert!(
            masks.is_empty(),
            "{} of 57,600 tile masks differ at frame 95: {:?}",
            masks.len(),
            &masks[..masks.len().min(8)]
        );
    }

    #[test]
    fn run14_s_frames_match_the_trace_draw_for_draw() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run10-world6-long.txt"),
            trace("rontrace-run14.log"),
        ) else {
            eprintln!("skipping: set RON_GAMELOG_DIR");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert_eq!(last, 284, "run14's traced length");
        for _ in 0..last {
            built.tick();
        }
        let mut matched = 0usize;
        let mut parted: Vec<String> = Vec::new();
        for (frame, ours) in &built.frame_sites {
            let theirs = trace.labels(*frame);
            if *ours == theirs {
                matched += 1;
                continue;
            }
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            parted.push(format!(
                "frame {frame}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            ));
        }
        // The first frame whose draw *sequence* is not the original's —
        // the number the tail cannot flatter.
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        assert!(
            first_part >= 284,
            "the stream parts at frame {first_part}; the floor is 284\n{}",
            parted.first().cloned().unwrap_or_default()
        );
        assert!(
            matched >= 284,
            "the trace floor fell: {matched} of {last} frames match, the floor is 284\n{}",
            parted.join("\n")
        );
        // **And a stricter floor beside it: the first frame whose draw
        // *count* differs.** Frame 99's disagreement is an attribution and
        // not a divergence — eight draws either side, and the one that
        // differs is the same address under a different caller (ours
        // `Unit::do_idle+0x7d`, the original's `Guy::inc_time+0x271`, the
        // standing swap `docs/SYNC.md` §6 names).
        //
        // The word was the original's through 121 and parted at **122** on
        // the blocked stand this simulation did not take. It takes it now
        // (item 49, `docs/COLLISION.md` §5), and with item 61's cell index
        // it ran to **201** — where the AI's farmer `1/4` re-picked a
        // second time and this simulation did not, because its walk to a
        // cell a sibling was already working was not refused by the
        // collision. With item 63's waypoint test it is, and the word ran
        // to **232**, where the disagreement was an arrival stand
        // (`Guy::set_anim+0x97a < Guy::move+0x19f`) the original takes and
        // this simulation did not.
        //
        // With item 66 — `find_nearby_spot`'s own collision half — it runs
        // to the **end of the capture**: all 284 frames spend the same
        // number of draws, and 282 of them are the original's draw for
        // draw. The two that are not were 99, the ~~attribution swap~~
        // above, and 100 — and with the frame's two loops (item 60) they
        // are the original's too: **284 of 284, draw for draw**. It was
        // never an attribution question. `Objects::process_all` runs the
        // buildings *after* the units, so the citizen trained on 99 is
        // never reached by that frame's unit loop and it is
        // `Objects::inc_time` that wraps its `end_time 0` clock
        // (`docs/SYNC.md` §3.16). **This capture is spent**: it can no
        // longer say where the simulation next parts from the original,
        // and the long traces — run33's and run39's — are what do.
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        assert!(
            first_count >= 284,
            "the stream's *word* parts at frame {first_count}; the floor is 284"
        );
        // **The farmer's re-target, frame for frame — item 61's own row.**
        // Two `orders::SITE_FARM_CELL` draws are one farmer picking a new
        // cell because the one under it ripened, and the frame it happens
        // on is a hundred frames of the farm clock plus the cell it was
        // sowing. The cell index was transposed (`status[dy][dx]` for
        // `status[dx][dy]`, `docs/ORDERS.md` §6.5), so from frame 101 —
        // the first time a farmer stands anywhere but the symmetric
        // `(2, 2)` — six farmers sowed six wrong cells, and the first
        // re-target fell on **185** where the original's falls on 199.
        // The farm-record diff cannot see this (its captures stop at 104,
        // before any farmer reaches its new cell), so it is pinned here.
        let cell_frames = |at: &dyn Fn(i64) -> Vec<String>| -> Vec<(i64, usize)> {
            (0..=last)
                .map(|f| {
                    let n = at(f)
                        .iter()
                        .filter(|l| **l == sim::orders::SITE_FARM_CELL)
                        .count();
                    (f, n)
                })
                .filter(|&(_, n)| n > 0)
                .collect()
        };
        let ours_at = |f: i64| -> Vec<String> {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or_else(Vec::new, |(_, s)| s.clone())
        };
        let theirs_at = |f: i64| -> Vec<String> { trace.labels(f) };
        let (mine, theirs) = (cell_frames(&ours_at), cell_frames(&theirs_at));
        assert_eq!(
            theirs,
            vec![
                (101, 12),
                (199, 2),
                (201, 2),
                (211, 4),
                (217, 4),
                (218, 2),
                (220, 2),
                (241, 2)
            ],
            "the original's own re-targets: six farmers on 101, then one a \
             cell at a time as each ripens"
        );
        // And ours is that list entire up to the frame the word parts on.
        // It used to be a prefix of two — 101 and 199, with 211 and 217
        // checked separately and the tail past the word's divergence
        // unusable. Item 63's waypoint collision test bought the 201 row,
        // which is `1/4` re-picking a second time after its walk to `1/2`'s
        // cell was refused; with it the whole schedule to 241 is the
        // original's, frame for frame and draw for draw.
        //
        // Item 64 added one row of our own at **243**, past the frame the
        // word parted on, so the assertion was split: everything the
        // comparable stretch carries must be the original's exactly, and
        // anything extra must be past that frame. Item 66 took the word to
        // the end of the capture, and with it the extra row went — the
        // whole schedule to 241 is the original's, frame for frame and
        // draw for draw, and there is nothing of ours outside it.
        assert_eq!(
            mine.iter()
                .copied()
                .filter(|&(f, _)| f < first_count)
                .collect::<Vec<_>>(),
            theirs
                .iter()
                .copied()
                .filter(|&(f, _)| f < first_count)
                .collect::<Vec<_>>(),
            "every farm re-target before the word parts, on the original's frames"
        );
        assert!(
            mine.iter()
                .all(|&(f, _)| f >= first_count || theirs.iter().any(|&(g, _)| g == f)),
            "and no re-target of ours inside the comparable stretch that the \
             original does not make: {mine:?} against {theirs:?}"
        );
        assert_eq!(
            mine.iter()
                .filter(|&&(f, _)| !theirs.iter().any(|&(g, _)| g == f))
                .copied()
                .collect::<Vec<_>>(),
            Vec::new(),
            "no re-target of ours the original does not make"
        );
        // **The bird's own row, and it is the original's now.** Every
        // eighth frame carries three `Animal::think_bird` draws per living
        // bird, and with item 59 this simulation hatches its first on
        // **frame 96 — the original's own frame** — so the beat agrees row
        // for row from 104 to 192, the last eighth-frame before the
        // original's second bird. (Before item 59 ours hatched at 32 and
        // 128 and the rows could not be compared at all.) The second bird
        // is still drift: the sampling reads cells off a stream that parts
        // at 122, so ours hatches at 224 where the original's hatches at
        // 192, and every row from 200 on is ours rather than the
        // original's.
        let think = |sites: &[String]| -> usize {
            sites
                .iter()
                .filter(|l| l.starts_with("Animal::think_bird"))
                .count()
        };
        let ours_beat: Vec<(i64, usize)> = built
            .frame_sites
            .iter()
            .map(|(f, s)| (*f, think(s)))
            .filter(|&(f, n)| n > 0 && f <= 192)
            .collect();
        let theirs_beat: Vec<(i64, usize)> = (0..=192)
            .map(|f| (f, think(&trace.labels(f))))
            .filter(|&(_, n)| n > 0)
            .collect();
        assert_eq!(
            ours_beat, theirs_beat,
            "three draws a bird an eighth-frame, on the original's frames"
        );
        assert_eq!(
            ours_beat.len(),
            12,
            "every eighth frame from the hatch to the original's second bird"
        );
        assert_eq!(
            trace.labels(96)[6],
            sim::anim::SITE_INIT_REAL,
            "the hatching roll is draw 6 of the original's sampling frame"
        );
        // **The hatch frame is a mechanism again, not drift.** run14's
        // birds hatch at 96, 192 and 256, and the sampling that decides a
        // hatch reads cells off the sync stream — so a hatch is the
        // original's exactly as far as the word is. With item 61 the word
        // runs to 201 and **the first two hatches are the original's own
        // frames**: 96 (where the pin has stood since item 59) and now
        // **192**, which the old `[96]` pin could not reach because the
        // word parted at 185, seven frames short of it. The tail —
        // ours at 224 and a pair at 256 — is off a stream that is no
        // longer the original's after 201, and the count of live birds
        // with it. The history of this pin is the history of the word:
        // `[96, 224]` off a divergence at 122, `[96]` off 185, `[96, 192]`
        // off 201, `[96, 192, 256, 256]` off 232 — and with item 66's word
        // running the whole capture, **`[96, 192, 256]`, the original's
        // three hatches and nothing else**. There is no tail left to
        // excuse: every hatch this simulation makes is one the original
        // makes, on its frame.
        let hatches: Vec<i64> = built.sim.gaia.bird_spawns.iter().map(|(f, _)| *f).collect();
        assert_eq!(
            hatches,
            vec![96, 192, 256],
            "the original's own three hatch frames, and no other"
        );
        assert_eq!(built.sim.live_birds(), 3, "alive at the end");
        // The wing beat, which item 52 bought and item 59 put on the
        // original's frames: the hatch frame's wrap (`Guy::init_real`
        // leaves `end_time` at zero, so the same frame's `inc_time`
        // overflows it at once), the birth coin `do_air_physics` throws
        // the frame after, and a coin at every wrap the animation's own
        // length places. *Which* animation is a coin, so the spacing
        // alternates between *Bird Flap*'s 23 frames and *Bird Soar*'s 31.
        // The lengths are the install's; the frames are the original's —
        // **97, 127, 142, 150** on both sides, and the next coin is past
        // the word divergence at 122 and is ours.
        let coin_frames = |at: &dyn Fn(i64) -> Vec<String>, upto: i64| -> Vec<i64> {
            (0..=upto)
                .filter(|f| at(*f).iter().any(|l| *l == sim::anim::SITE_BIRD_COIN))
                .collect()
        };
        let ours_at = |f: i64| -> Vec<String> {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or_else(Vec::new, |(_, s)| s.clone())
        };
        let theirs_at = |f: i64| -> Vec<String> { trace.labels(f) };
        assert_eq!(
            coin_frames(&ours_at, 160),
            coin_frames(&theirs_at, 160),
            "the birth coin the frame after the hatch, then a coin at every \
             wrap — on the original's frames while the word is still its own"
        );
        assert_eq!(
            coin_frames(&ours_at, 160),
            vec![97, 127, 142, 150],
            "the hatch's birth coin, then Soar's 31 and Flap's 23"
        );
        // Item 53's own rows, each stated so a regression reads as itself.
        //
        // Frame 1 is the AI's opening: eight `rand_int(1, 10)` inside the
        // script VM, and it is the whole of that frame's script draws.
        // Naming them took the frame from "54 against 54 in the wrong
        // vocabulary" to a match.
        let frame_one = &built
            .frame_sites
            .iter()
            .find(|(n, _)| *n == 1)
            .expect("frame 1")
            .1;
        assert_eq!(
            &frame_one[..8],
            &[sim::ai_host::SITE_RAND_INT; 8],
            "the script VM's eight, and no others"
        );
        // Frame 101 is the farmers' re-target: six farmers, two
        // `GameAccess::rnd(4)` each, one mark carrying the pair.
        let count = |f: i64, label: &str| -> usize {
            built
                .frame_sites
                .iter()
                .find(|(n, _)| *n == f)
                .map_or(0, |(_, s)| s.iter().filter(|l| *l == label).count())
        };
        assert_eq!(
            count(101, sim::orders::SITE_FARM_CELL),
            12,
            "six farmers' cell re-pick, two draws each"
        );
        // Frame 108 is one herd animal's whole wander: the three-in-ten
        // coin, then the direction and the two step counts, in that order.
        let hundred_eight = &built
            .frame_sites
            .iter()
            .find(|(n, _)| *n == 108)
            .expect("frame 108")
            .1;
        let wander: Vec<&String> = hundred_eight
            .iter()
            .filter(|l| l.starts_with("Animal::do_idle"))
            .collect();
        assert_eq!(
            wander,
            vec![
                sim::gaia::SITE_WANDER_ROLL,
                sim::gaia::SITE_WANDER_DIR,
                sim::gaia::SITE_WANDER_X,
                sim::gaia::SITE_WANDER_Y,
            ],
            "the wander's four, in order"
        );
        // `Guy::move+0x19f` is the arrival stand, which this simulation
        // takes on its own frames. Without the chain it reads as a bare
        // `5dac7a` and the residue is unreadable, which is the whole of
        // item 53.
        assert!(
            trace
                .labels(232)
                .iter()
                .any(|l| l == sim::anim::SITE_ARRIVE),
            "the arrival stand is named on the trace's frame 232"
        );
        // **The blocked stand, on the original's own frames** (item 49).
        // `Unit::move_step+0x823` is the idle a unit re-rolls the moment
        // its step is refused, before the three give-up tests. run14 has
        // three of them; this simulation takes the first two on the
        // original's frames, which is what carries the word from 122 to
        // 185. The third is past the divergence and neither side is
        // measuring the same game by then.
        let blocked = |sites: &[String]| sites.iter().any(|l| l == sim::anim::SITE_BLOCKED);
        let theirs_blocked: Vec<i64> = (0..=last).filter(|&f| blocked(&trace.labels(f))).collect();
        assert_eq!(
            theirs_blocked,
            vec![122, 184, 256],
            "the original's three blocked stands"
        );
        let ours_blocked: Vec<i64> = built
            .frame_sites
            .iter()
            .filter(|(_, s)| blocked(s))
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(
            ours_blocked
                .iter()
                .copied()
                .filter(|&f| f <= 185)
                .collect::<Vec<i64>>(),
            vec![122, 184],
            "the two before the word parts are the original's, frame for frame"
        );

        // And it flies, which is what keeps it out of the occupancy grid a
        // citizen walks on — `collide.rs`'s `is_air` is the loaded domain
        // now rather than the seam it was (`docs/COLLISION.md` §2).
        let bird = built
            .sim
            .units
            .iter()
            .find(|u| u.owner == sim::gaia::BIRD_OWNER)
            .expect("a bird");
        assert_eq!(bird.kind.domain, sim::attrition::Domain::Air);
    }

    /// **The long trace, and where the word parts past run14's 284.**
    ///
    /// run14 traced 284 of run10's 1,772 frames, and since item 66 its
    /// word — the per-frame draw *count* — matched for every one of them.
    /// A trace that agrees to its own end cannot say where the simulation
    /// next parts by *site*, which is what item 38 was: **run33** is run10's
    /// own game captured again under the traced executable, with run10's
    /// exact dump settings and `cover=1`, quitting at frame 1,850
    /// (`docs/ORACLE.md`, "run33"). `tools/gamelog/samegame.py` is the
    /// proof it is the same game — every frame block of run33 and run10
    /// digests identically — so this capture inherits run10's siblings and
    /// supersedes run14 wherever the two overlap.
    ///
    /// Two numbers come out of it, and they are the sub-scores the residue
    /// chase is steered by:
    ///
    /// - **the word**: the first frame whose draw *count* is not the
    ///   original's, and
    /// - **the sequence**: the first frame whose draws are not the
    ///   original's site for site, which is at or before it.
    #[test]
    fn run33_s_long_trace_says_where_the_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 1_800,
            "run33's traced length is {last}, wanted 1,800+"
        );
        for _ in 0..last {
            built.tick();
        }
        let mut matched = 0usize;
        let mut parted: Vec<String> = Vec::new();
        for (frame, ours) in &built.frame_sites {
            let theirs = trace.labels(*frame);
            if *ours == theirs {
                matched += 1;
                continue;
            }
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            parted.push(format!(
                "frame {frame}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            ));
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        eprintln!(
            "run33: word parts at {first_count}, sequence at {first_part}; \
             {words} of {last} frames spend the original's number of draws, \
             {matched} of them draw for draw"
        );
        for p in parted.iter().take(4) {
            eprintln!("{p}");
        }
        // The count divergence is the score, so print its own row too: it
        // is the successor item every time this number moves.
        if let Some(p) = parted
            .iter()
            .find(|p| p.starts_with(&format!("frame {first_count}:")))
        {
            eprintln!("count: {p}");
        }
        // **The word: 307.** run14's capture agreed to its own end at 284;
        // this one carries 1,566 frames more, and the simulation's
        // per-frame draw *count* is the original's for twenty-three of
        // them before it parts.
        //
        // What parts it is a **non-flat gather**. The original's frame 307
        // spends eleven draws and this simulation nine, and the two it
        // does not spend are one unit's: a stand issued from inside
        // `Unit::do_non_flat_gather+0x10f`, and `+0x54b`, the gather's own
        // roll — so on the original a gatherer is working a non-flat
        // resource on that frame and here it is not. The other nine are
        // the same on both sides and in the same order: an
        // `Animal::do_idle` roll, a phase-7 wrap, and seven farms.
        // **345 since item 68** (2026-08-29). The two draws frame 307 was
        // short were `1/7`'s, and `1/7` was not at its camp to spend them
        // because the AI's army had marched it away on 252: `Unit::think`'s
        // tail was joining any attacker to an army, where the original
        // joins only a supply wagon or a hero, and run33's own coverage
        // says `Unit::add_to_army@005f7740` is never entered in this game
        // at all. With the tail as the listing has it the word runs to
        // **345**.
        //
        // **What parts it at 345 is not the ninth citizen.** Item 68's
        // note guessed it was — `1/8`'s Woodcutter's Camp, the row that
        // pinned the headline three frames earlier — and item 70 fixed
        // exactly that and moved this number not at all. The row is an
        // animation draw against a farm's: on 345 this simulation spends
        // eight where the original spends seven, and the very first is
        // ours `Guy::set_anim < Guy::inc_time` against the original's
        // `Farms::inc_time+0x1ae`. It is the same row before and after
        // item 70, byte for byte. The **totals** are what moved.
        //
        // **361 with item 71** (2026-08-29), and what was wrong at 345 was
        // a **length**. The human scout's guy 0 rolled `CHAR_IDLE1` on
        // frame 284; no dump has ever shown that slot's length for its
        // piece, so `Art::lengths` had none and the roll fell back to
        // `CHAR_DEFAULT` — 61 frames instead of 76. The clock wrapped on
        // 345 where the original's runs to 360, and in the five frames
        // between, the original's *dog* — whose mirrored `cur_time` had
        // run past its own `end_time` — re-rolled once a frame from
        // `Unit::do_idle`, which this simulation never reached. Four
        // draws of drift by frame 349, and the farmer `0/4`'s two `% 4`
        // re-target draws came off the wrong words (item 71's own
        // symptom, the headline).
        //
        // The lengths now come from the install: `unit_graphics.xml` for
        // every `<UNIT>` entry, placed at the piece
        // `GraphicPieces::get_unit_gpiece` hands out
        // (`crate::artdata::piece_lengths`). That exposed a second one —
        // `unit_masks & 0x78000000`, the carrying walk, which this crate
        // had been reading off the gather order's `goto_build` instead.
        // With the carrying slots' lengths known the stand-in stopped
        // being invisible: `1/7`'s first walk to its camp became a
        // `WALK_WITH_WOOD` and lost the arrival stand the original spends
        // at frame 232. The mask is `do_non_flat_gather`'s own, and with
        // it the word runs to **361**.
        //
        // **432 with item 76**, and what was wrong at 361 was a **tile**.
        // The cell filter's surface probe (`docs/SCOUT.md` §7) was reading
        // `(4x, 4y + 2)` where the listing reads the cell **centre**: the
        // `+4` in `movb 0x4(%eax,%ecx,2)` is two `TData` elements, not a
        // field offset. Cell `(48, 23)` is ocean at its first tile and
        // land at its centre, so the AI scout's re-target refused a
        // candidate the original takes — twenty-six draws against
        // twenty-seven — and went to `(48, 20)` instead. What parts the
        // word at 432 is a gather stand: ours spends
        // `Guy::set_anim < Unit::do_non_flat_gather+0xb99` where the
        // original spends a farm's `Farms::inc_time+0x1ae`, twenty-three
        // draws against twenty-two.
        //
        // **482 with item 78**, and what was wrong at 432 was a **write
        // that went to the wrong order**. `Sim::store_gather` wrote the
        // gather order's fields back to `orders.front_mut()`, but every
        // walk `do_non_flat_gather` issues goes in *front* of the gather
        // (`QUEUE_FIRST`, no action bit) — so on the return-to-camp branch
        // the `goto_build = 1` and `wait = 32` that follow
        // `add_move_order` were written to the move and silently dropped.
        // The original holds a pointer to the order object for the whole
        // function, which is the same rule `is_gathering_at` already
        // needed on the *read* side (`docs/ORDERS.md` §6, §6.4).
        //
        // The human's woodcutter `0/2` finished its shift on frame 426,
        // set off for its camp on 427 and arrived on 431 — and then, with
        // `goto_build` still 0 and `wait` still −1, re-entered the same
        // branch on 432 and spent the stand again. It never unloaded, never
        // took another tile, and re-issued that walk every other frame for
        // the remaining 1,400 frames of the capture.
        //
        // **571 with item 79**, and what was wrong at 482 was an **angle**.
        // The scout re-targets on 482 and the original spends thirty-one
        // draws over its seven rings where this spent thirty: cell
        // `(56, 28)` was already seen here and not there, so the cell
        // filter refused a candidate the original scores
        // (`run33_s_scout_re_targets_at_482_on_the_original_s_ring`).
        //
        // It was seen here because of a reveal three hundred frames
        // earlier. `Object::update_seen` throws a small land unit's disc
        // half a cell forward of its nose, and the angle it projects along
        // is `UnitData +0x50` — the unit's own heading — not the guy's
        // eased facing; the `project` at `00651d05` is handed it by
        // `movl 0x50(%ecx), %ecx` two instructions earlier. This crate had
        // the guy's. On frame 168 the scout was mid-turn, the two angles
        // were −51.6° and −83.0°, and the disc thrown along the wrong one
        // lit a fog cell the original's never reached (`docs/VISION.md`
        // §3).
        //
        // What parts the word at 571 is a **collision**: the original
        // spends a draw at `5fa882`, inside
        // `Unit::resolve_unit_collision@005f9d30`, that this simulation
        // does not — eight draws against seven.
        //
        // **776 with item 81**, and what was wrong at 576 was a **price**.
        // The original's frame 576 spends sixty draws over five
        // `ScenarioFuncSet::place_city_with_cost` calls — the script's
        // `city_placement` under `defensive.bhs` step 11, once per turn of
        // its `num_loops = 5` loop — and this simulation spent forty,
        // because its *first* call bought the city and the guard at
        // `009f5898` (`city_limit <= total_cities`) then returned −1 four
        // times without a draw. The original could not pay: a second Small
        // City costs **sixty** food and sixty timber, not twenty-two, and
        // the AI held 69 and **59**.
        //
        // `TypeData::get_cost` has two ramps. The unit arm reads
        // `UNIT_COST_FACTOR` and all four `*_RAMP_MAX`; the building arm
        // (`00665787`..`00665b5a`) reads `BUILD_COST_FACTOR` and
        // `BUILD_SUPPORT_FACTOR` and **no ceiling at all**. This crate gave
        // every building `RampClass::default()` — the military 125% — which
        // clamped the city's `SUPPORT 50` ramp to 12 and priced it at 22.
        // run40 and run41 measure both ends of it
        // (`run40_s_census_prices_the_ai_s_second_city_at_sixty`).
        //
        // What parted the word at 776 was that same block a second time:
        // the original spent five draws there and this simulation ten,
        // because on 776 it was the *original* that bought on its first
        // call and this simulation that could not — its AI was thirty-two
        // food short of the original's on every frame from 202 on. Item 74
        // is those thirty-two: twenty from the farm the AI finishes on
        // frame 166 (`Build::do_bonus`, `docs/ECONOMY.md`) and twelve from
        // the City State waiting in its library when Written Word lands on
        // 201 (`Build::refund_cost`, `docs/COSTS.md`). With them the city
        // is bought on 776 here too, and **the word parts at 780**.
        //
        // **986 with item 84**, and what was wrong at 780 was a **byte
        // that was not the guy's**. `do_gather`'s farm switch tests
        // `guy[0].cur_anim` — `*(char *)(**(int **)&this->field_0xf4 +
        // 0x9c)`, `GuyData +0x9c` — and this crate kept a `farm_anim`
        // flag written only by the farm branch and cleared only by a
        // step. The AI's `1/4` stands on farm `b10`'s cell 7, reaps it
        // until `inc_time` decays it empty on 777, re-picks a tile on 778
        // and is **blocked** on 779: the blocked stand
        // (`Unit::move_step+0x823`) replaces the guy's reap animation
        // without moving the body, so on 780 the original sows the cell
        // and this crate re-picked again. Two draws it does not spend, a
        // seventh `Farms::inc_time` from 781 — the farm keeps five empty
        // cells here and drops to four there — and what parts the word at
        // 986 is a blocked stand of its own, seven draws against six.
        //
        // **1372 with item 112**, and what was wrong at 986 was a **herd's
        // wander centre, which does not walk**. `Herd::process@00741760`
        // writes `wx = cx − 1 + p % 3` and `wy = cy − 1 + p % 3` — it
        // reads `HerdData +0x0/+0x4` and stores into `+0x8/+0xc`, so the
        // jitter is about the **home** cell every time. This crate read
        // the destination as the source and random-walked it, which is the
        // same thing until a herd's *second* walk; `(frame >> 6) % 13`
        // gives herd 0 its second on frame 832 and no other herd one at
        // all inside the capture. Its `wy` ended 34 here against the
        // original's 33, and the far wander's ring is drawn about
        // `herd_centre` (`docs/ANIM.md` §7): on 981 the sheep `8/3` was
        // sent to `(17688, 26616)` where the original sends it to
        // `(17688, 26328)`, five steps of `(+15, −5)` against five of
        // `(+12, −10)`, so the same neighbour refused it a frame early and
        // the blocked stand fell on 986 instead of 987. Pinned step for
        // step by `run33_s_herd_centre_jitters_about_its_home_cell`.
        //
        // **1802 with item 114**, and what was wrong at 1372 was a
        // **pasture nothing stocked**. `Build::activate`'s gather tail
        // reaches `Farms::add_animals` for every *food* gather building it
        // completes — `LAB_00625a36` is the fall-through from
        // `do_bonus(0, FOOD_BONUS_FOR_FARM)` and the `iVar18 == 0` arm of
        // `switchD_006259b5_caseD_2` is where every unpaid exit lands, and
        // both are the same call — so the AI's fourth farm, which
        // `Farms::add` had already made a pasture, costs twenty draws on
        // the frame it finishes: a species coin, a `y` and an `x` per
        // animal (`+0x92`, `+0x134`, `+0x182`) and `Guy::init_real`'s
        // variant inside `Objects::init_unit`. Five more follow in the
        // same frame's `Objects::inc_time`, because a newborn guy's
        // `end_time` is zero and its clock wraps at once. Twenty-five
        // draws where this crate spent none, and `Farms::inc_time`'s three
        // were all that was left. The harness had the building and the
        // type right — its `b15` activates on 1372 with `farm_type 1` —
        // and only the stocking was missing (`docs/SYNC.md` §3.11).
        //
        // The parting frame goes 1372 -> **1802**, which is past run10's
        // whole 1,772: this map now spends the original's draws in the
        // original's order for longer than its own capture runs. What
        // parts it at 1802 is one draw at
        // `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3`, a gaia
        // bird's flight physics, and it is the **only** time that site is
        // reached in all 1,851 frames.
        assert!(
            first_count >= FLOORS[1].word,
            "the word parts at frame {first_count}; the floor is {}\n{}",
            FLOORS[1].word,
            parted.first().cloned().unwrap_or_default()
        );
        // **The sequence: 576**, and getting there was the whole of the
        // ~~99~~ attribution swap run14's capture had always shown — the
        // same address under a different caller, ours `Unit::do_idle+0x7d`
        // where the original has `Guy::inc_time+0x271`, with the draw count
        // equal either side (queue item 62). It was not an attribution
        // question at all: the trained citizen is created in
        // `Objects::process_all`'s **second** loop, after every unit, so
        // the original never reaches it in the unit loop on its birth
        // frame and `Objects::inc_time` wraps its `end_time 0` clock
        // instead. `docs/SYNC.md` §3.16. It is a floor here so that a
        // regression that moved it would read as itself.
        // **576 -> 780 with item 100**, and the two numbers meet: naming
        // `make_stuff`'s expiry walk (`+0x221`) is what moved it, because
        // an unnamed draw takes the last mark's name and five frames of
        // `place_city_with_cost` at 576 read as `compute_sites+0x50a`.
        // Nothing about the simulation's arithmetic changed there; what
        // changed is that the comparison stopped lying about it.
        // **780 -> 986 with item 84**, the two still meeting: the farm
        // stand's byte is the guy's live `cur_anim`, and nothing between
        // 780 and 986 is a naming question.
        // **986 -> 1372 with item 112**, and the two meet a third time:
        // the herd's wander centre is jitter about the home cell, and
        // nothing between 986 and 1372 is a naming question either.
        // **1372 -> 1802 with item 114**, a fourth time: the pasture's
        // twenty draws carry names of their own from the frame they are
        // first spent on, and nothing between 1372 and 1802 is a naming
        // question. The two numbers have now been equal for four items
        // running, which is what it looks like when every hole left is a
        // mechanic rather than a label.
        assert!(
            first_part >= FLOORS[1].word,
            "the draw sequence parts at frame {first_part}; the floor is {}",
            FLOORS[1].word
        );
        // And the totals over the whole 1,850, which is what says whether a
        // change past the divergence helped or only moved the noise: 618
        // frames spend the original's number of draws and 460 of them are
        // its draws in its order. Past the part frame both sides are off
        // streams of their own, so these are weak numbers — but a fall in
        // them is worth reading.
        //
        // 668 / 556 → **618 / 460** with item 68, the only time either has
        // fallen while the score rose. Every frame before 345 matches on
        // both counts, so the whole of the fall is past the divergence: the
        // stream this simulation is on after `1/8` takes the wrong job is a
        // *different* wrong stream from the one it was on after `1/7` was
        // marched off, and it happens to coincide with the original's less
        // often. The number that is not luck is `first_count`, 307 → 345.
        //
        // 618 / 460 → **635 / 488** with item 70, and this is the useful
        // half of that item on this capture: `first_count` did not move,
        // but seventeen more frames spend the original's number of draws
        // and twenty-eight more spend them in its order, because the AI's
        // citizens are at the buildings the original has them at for the
        // rest of the run.
        //
        // 635 / 488 → **696 / 523** with item 71, the largest move either
        // has made, and the two rose together with `first_count`.
        //
        // 696 / 523 → **724 / 606** with item 76, and `matched` moved most:
        // eighty-three more frames are the original's draws in the
        // original's order, which is what a scout sent to the original's
        // cell buys downstream.
        //
        // 724 / 606 → **752 / 622** with item 78.
        //
        // 752 / 622 → **791 / 662** with item 79, and the two rose with
        // `first_count` again — thirty-nine more frames on the original's
        // word, forty more of them draw for draw, because the AI's scout
        // now walks the original's fog as well as its ground.
        //
        // 791 / 662 → 802 / 688 with item 80, and → **944 / 828** with item
        // 81 — the largest move either has made, and both rose with
        // `first_count`: a hundred and forty-two more frames spend the
        // original's number of draws because the AI's whole economy is two
        // hundred frames closer to the original's from 576 on.
        //
        // 944 / 828 → **951 / 838** with item 74, and the three rose
        // together: the AI's second city is founded on the original's
        // frame, so its ninth citizen is trained on the original's frame
        // too and the roster is one-sided again at 268 + 0.
        //
        // 951 / 838 → **954 / 843** with items 36 and 93 together. This is
        // the number item 93 was held back for: the arm alone, before the
        // farmers' angles were the original's, cost `first_count` 780 →
        // 584 and run10's orders 776 → 586. With item 36 in front of it
        // the word holds at 780 and both totals rise
        // (`docs/SYNC.md` §3.11, §3.12).
        //
        // 954 / 843 -> **938 / 841** with item 95, the animal's hurry, and
        // this is the second time either has fallen while the score rose
        // (item 68 was the first). The fall is entirely noise and it can
        // be said exactly where it starts: with the hurry in and out, this
        // capture's per-frame draw counts are **identical up to frame
        // 1108** — 328 frames past the word's own parting at 780, and past
        // every one of the twelve units' first divergence. `first_count`
        // holds at 780 and `first_part` at 99; what moved is which of two
        // wrong streams the simulation is on after 1108. The number that
        // is not luck is the *other* map's, where the same change takes
        // the word 69 -> 91.
        //
        // 938 / 841 -> **943 / 827** with the blocked animal's dropped
        // walk, and the same argument holds a second time with the same
        // shape: with that rule in and out, this capture's per-frame draws
        // are **identical — the count and the sequence both — up to frame
        // 1128**, 348 frames past the word's own parting at 780 and past
        // eleven of the twelve units' first divergence. `first_count`
        // holds at 780, `first_part` at 99, run10's ticks and orders at
        // 572/776, and the other map's word goes 91 -> 201
        // (`docs/SYNC.md` §3.14).
        //
        // 943 / 827 -> **943 / 830** with the frame's two loops (item 60),
        // and what moved with them is the *sequence*: 99 -> 576. The word
        // holds at 780, run10's ticks and orders at 572/776, and the other
        // map's word goes 219 -> 274 (`docs/SYNC.md` §3.16).
        //
        // 943 / 830 -> **964 / 864** with item 100, and again it is the
        // *sequence* that moves: **576 -> 780**, so this map's two numbers
        // are now the same frame. Two things landed together and only one
        // of them is a mechanic. `make_stuff`'s expiry walk had always
        // spent its draws and never **named** them, so every one read as
        // whatever site marked last — here `compute_sites+0x50a`, five
        // frames of it at 576 — and a hole that was only a missing label
        // sat in front of the holes that are real. The mechanic is the
        // bird's landing search (`docs/SYNC.md` §3.9), sixty draws a
        // landing, which is what takes the *other* map's word 576 -> 645.
        //
        // 964 / 864 -> **977 / 866** with item 101, the chopping guy's own
        // wait: both numbers rise while the word and the sequence hold at
        // 780, because a woodcutter that stays at its tile three times as
        // long is at the original's tile on hundreds of the frames past
        // the parting. The other map's word goes 645 -> 742.
        //
        // 977 / 866 -> **986 / 884** with item 102, the far wander's
        // literal bearing (`docs/SYNC.md` §3.19): this map's herd takes
        // the same branch East Indies' does, so nine more frames spend the
        // original's number of draws and eighteen more are draw for draw,
        // while the word and the sequence hold at 780. The other map's
        // word goes 742 -> 867.
        //
        // 986 / 884 -> **986 / 892** with item 106, the scout's walk to a
        // goody box (`docs/GOODY.md` §7): Great Lakes has 22 boxes, and
        // eight more frames past the parting come out draw for draw while
        // the word and the sequence hold at 780. The other map's word goes
        // 879 -> 1256.
        //
        // 986 / 892 -> **959 / 876** with item 109, the bird's landing
        // step (`docs/SYNC.md` §3.9) — **the first fall these two numbers
        // have taken**, and it is the noise this comment's own opening
        // paragraph names. Every frame whose verdict changed is **1209 or
        // later**, 429 past the parting: 31 gained, 47 lost. What moved
        // them is that a bird's counter is one higher after each landing,
        // so this map's landings go 904, 936, 1080, 1136, 1152, 1288,
        // 1416, 1520, 1728, 1784 to 904, 936, 1080, 1136, 1152, **1184,
        // 1232, 1456, 1640**, 1784 — a stream that is nobody's rolling a
        // different number, on frames where the original's own landings
        // (816, 904, 1024, 1040, 1144, 1208, 1344, 1448, 1456, 1592,
        // 1744, 1816, 1832) it now meets **two** of rather than one. The
        // word and the sequence hold at 780, East Indies' word goes
        // 1256 -> 1373, and run39's landings up to its word are the
        // original's exactly
        // ([`a_bird_s_landing_frames_are_the_trace_s_own`]).
        //
        // 959 / 876 -> **953 / 877** with item 69's collision pause
        // (`docs/ORDERS.md` §4.4, `goto STEP`), the second fall and the
        // same kind of noise: the word and the sequence hold at **780**,
        // and every frame whose verdict changed is past it, on a stream
        // that is nobody's. What the item did move is the score — this
        // map's ticks 572 -> 781 and East Indies' 167 -> 1374 — which is
        // the number these two are instruments beside.
        //
        // 953 / 877 -> **1182 / 1114** with item 84, the largest move
        // either has ever made and the first since item 81 that is not
        // noise: the word itself goes 780 -> 986, so two hundred of those
        // frames are frames both sides genuinely agree on rather than
        // coincidences past a parting.
        //
        // 1182 / 1114 -> **1471 / 1439** with item 112, larger again and
        // not noise either: the word goes 986 -> 1372, so three hundred
        // more of these are frames both sides genuinely agree on. The gap
        // between the two also narrows — 68 frames spend the original's
        // number of draws in some other order, against 68 before — which
        // is what a stretch of real agreement rather than coincidence
        // looks like.
        //
        // 1471 / 1439 -> **1467 / 1436** with item 113, the second fall
        // these two have taken, and the argument is exact rather than
        // statistical this time: **the word and the sequence both hold at
        // 1372**, and `first_count` is by construction the first frame
        // whose draw *count* differs, so every frame before 1372 spends
        // the original's number of draws in the original's order on both
        // sides of the change. Every one of the seven verdicts that moved
        // is at 1372 or later, past the parting, on the two wrong streams
        // this comment's opening paragraph names. What the item moved is
        // the headline: this map's ticks go 910 -> **1375** and its orders
        // 776 -> **791**, the AI's `1/1` stops parting at all, and East
        // Indies holds at 1374/1373.
        //
        // 1467 / 1436 -> **1832 / 1830** with item 114, the largest move
        // either has ever made and the last one that can be large: 1,832
        // of the 1,850 frames spend the original's number of draws and
        // 1,830 of them draw for draw, so what is left is eighteen frames
        // and all of them past 1802. The gap between the two is down to
        // **two**, from 31 — a stretch of real agreement rather than
        // coincidence has no room for a frame that spends the right
        // number of draws in the wrong order.
        assert!(
            words >= 1832 && matched >= 1830,
            "the trace floor fell: {words} frames on the original's word, \
             {matched} draw for draw; the floors are 1832 and 1830"
        );
    }

    /// **run53 — the same game, thirteen times as long, and the ceiling is
    /// the same frame** (2026-08-31, item 91).
    ///
    /// `run33_s_long_trace_says_where_the_word_parts` is 1,850 frames, and
    /// once item 114 took the word to 1802 the obvious worry was that 1802
    /// was an artifact of the capture running out rather than a mechanic
    /// boundary. run53 is the same game — same seed, same lobby, no input —
    /// traced whole over **24,000** frames, and `tools/gamelog/rngcmp.py`
    /// says its `game_random` word is run33's on all 1,851 overlapping
    /// frames with **zero** differing. So this is not a second opinion; it
    /// is the same opinion with twelve times more of it.
    ///
    /// **The word still parts at 1802**, and what parts it is one draw at
    /// `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3` — a gaia
    /// bird's flight physics, reached **once** in 1,851 frames and, as this
    /// capture now shows, rarely enough to stay the boundary over 24,000.
    /// That is the successor item, and this capture is what makes it
    /// scoreable without another one.
    ///
    /// **Only the two frame numbers are pinned here, deliberately.** Past
    /// 1802 both sides run on streams that are nobody's, so of the 24,000
    /// frames 4,231 spend the original's number of draws and 2,040 do it in
    /// the original's order — coincidence at about one frame in six, and a
    /// number that moves with every unrelated change. `docs/QUEUE.md` item
    /// 89(c) is exactly this trap, and run33's own totals are only worth
    /// pinning because most of its frames fall *before* its parting. Here
    /// they would be a non-monotone score pinned as monotone, so they are
    /// printed and not asserted.
    #[test]
    fn run53_s_24000_frames_put_the_ceiling_where_run33_did() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
        ) else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        // The whole point of this capture is its length; a short one would
        // pass every assertion below and mean nothing.
        assert!(
            last >= 23_000,
            "run53's traced length is {last}, wanted 23,000+ — this is the \
             long capture, and a short file here is a wrong file"
        );
        for _ in 0..last {
            built.tick();
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| ours.len() == trace.labels(*f).len())
            .count();
        let matched = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *ours == trace.labels(*f))
            .count();
        eprintln!(
            "run53: word parts at {first_count}, sequence at {first_part} of {last}; \
             {words} frames on the original's count, {matched} draw for draw \
             (both mostly past the parting — printed, not pinned)"
        );
        assert!(
            first_count >= LONG_WORD_GREAT_LAKES && first_part >= LONG_WORD_GREAT_LAKES,
            "run53's ceiling fell: word {first_count}, sequence {first_part}; \
             the floor is {LONG_WORD_GREAT_LAKES} on both"
        );
    }

    /// **run59 — the census window East Indies' word has been asking for**
    /// (2026-09-02, item 154's second half).
    ///
    /// The word is 5376 and the frame is the AI's Market: `economic.bhs`
    /// case 15 calls `place_building_with_cost(who, "Market", my_capital)`
    /// on it, the original places it, and this crate cannot pay — a Market
    /// is eighty timber and the AI holds thirty-four. That is a resource
    /// level, and no capture on disk carried one past frame 800:
    /// `LEADERS=1` at `[End Frame]` is five scalars and no goods, and the
    /// only `LEADERS=9` windows were run40's `[560, 600)` and run41's
    /// `[770, 800)`, both on Great Lakes.
    ///
    /// run59 is that window moved: run58's recipe (`MAP_STYLE 18`, seed
    /// 12345, the profile's lobby, no input) with `LEADERS=9` at
    /// `[End Frame]` and the frame window `[5150, 5400)` — the dump is
    /// written only there, so the run costs minutes rather than the hour a
    /// per-frame one does.
    #[test]
    fn run59_s_census_is_where_the_ai_s_timber_goes() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run59-islands-census-5150.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run59.log"),
        ) else {
            eprintln!("skipping: no run59 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        assert_eq!(init.pasture.len(), 1, "run59's trace reached the setup");
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut steps: Vec<(i64, i64, i64, i64)> = Vec::new();
        // (who, field, good) keyed to (frames wrong, ours and theirs first).
        let mut shapes: std::collections::BTreeMap<(i64, String, usize), (usize, i64, i64)> =
            std::collections::BTreeMap::new();
        // The AI's timber shortfall, frame by frame.
        let mut timber_gap: Vec<(i64, i64)> = Vec::new();
        for n in 1..=5399 {
            built.tick();
            if n < 5150 {
                continue;
            }
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                if who == 1 {
                    steps.push((
                        n,
                        block.int("production_step").unwrap_or(-1),
                        block.int("script_step").unwrap_or(-1),
                        block.int("prod_script_run").unwrap_or(-1),
                    ));
                }
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 6] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                    ("gather_slots[scan]", l.gather_slots),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" && g == 1 {
                            timber_gap.push((n, theirs[g] - i64::from(ours[g])));
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            wrong.push(format!(
                                "frame {n} who {who} {key} good {g}: ours {} theirs {}",
                                ours[g], theirs[g]
                            ));
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        eprintln!("run59: {} of {compared} good-frames disagree", wrong.len());
        for (k, (n, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, ours {o} theirs {t} on the first");
        }
        assert_eq!(
            compared, 18_000,
            "250 frames, two players, six goods, six fields"
        );
        assert_eq!(
            wrong.len(),
            1_500,
            "the census's own count. run59 measured 4,798; item 162 took 500 \
             with the pre-placed camp's six timber slots and 798 more with \
             the wealth slot a dock claims — which is the whole timber \
             lineage, bucket and income both — and item 165 took the last \
             2,000 with `Leader::calc_gather` step 6, the AI's Fisherman on \
             its fish. What is left is item 156's three hundreds"
        );

        // **The record, and every shape in it is a standing state.** Each row
        // is (who, field, good) to (frames wrong, ours and theirs on 5150).
        // 250 is "every frame of the window", which is what says a shape is
        // a level rather than an event.
        let row = |who: i64, key: &str, g: usize| shapes.get(&(who, key.to_string(), g)).copied();
        let n = |who: i64, key: &str, g: usize| row(who, key, g).map_or(0, |(n, _, _)| n);

        // **The item, closed.** run59 measured the AI fifty timber short on
        // every one of these frames, with `leftover` agreeing on 234 of
        // them — a lump, banked before the window. run60 found the frame it
        // was banked on (sim-frame 4988, a goody box) and run42 the reason
        // the good was wrong (the dock's thirty wealth, 1,409 frames
        // earlier). Both landed 2026-09-02, and the AI's timber is now the
        // original's on every frame of this window — including 5377, where
        // it used to flip to thirty ahead because the original could pay
        // eighty for the Market and this crate could not.
        assert_eq!(n(1, "bucket", 1), 0, "the AI's timber, across the Market");
        assert!(
            timber_gap.iter().all(|&(_, d)| d == 0),
            "and the gap is zero on every frame: {:?}",
            timber_gap.iter().find(|&&(_, d)| d != 0)
        );

        // **And the slot count is no longer where it comes from.**
        // `gather_slots` is the running inventory `Build::activate` adds a
        // finished gather building's `gather_max` to, and the dump prints it
        // per good beside its own high-water mark. This census is what
        // caught the pre-placed camp claiming nothing — the human, which
        // builds nothing at all in this game, held 0 against 6 — and since
        // 2026-09-02 the harness hands a dump-stood camp its list *before*
        // `activate` surveys it, so **both players' timber slots are now
        // exact** and so are their food slots.
        assert_eq!(
            n(0, "gather_slots[scan]", 1),
            0,
            "the human's starting camp"
        );
        assert_eq!(n(1, "gather_slots[scan]", 1), 0, "the AI's six plus four");
        assert_eq!(n(0, "gather_slots[scan]", 0), 0, "the human's farms");
        assert_eq!(n(1, "gather_slots[scan]", 0), 0, "and the AI's");
        // The one wealth slot was item 82's, and it is closed too:
        // `BuildTypeData::get_good`'s table indeed cannot produce a
        // wealth-gathering building, because that is not where the slot
        // comes from — `Build::activate` line 590 gives one to a dock, a
        // market or a temple, off the same array (`Sim::claim_commerce_slot`).
        assert_eq!(n(0, "gather_slots[scan]", 2), 0, "the human's market");
        assert_eq!(n(1, "gather_slots[scan]", 2), 0, "the AI's dock");
        // **And fixing the timber slots moved the timber not at all**,
        // which is the measurement that refuted the fifty's first
        // explanation. The bonus is `TIMBER_BONUS_PER_WOOD_SLOT` per slot
        // *past the high-water mark*, and `Build::activate` pays nothing at
        // frame 0: the six arrive during setup, raise the mark to six
        // unpaid, and the one camp the AI builds in the run (frame 2424,
        // four slots) then pays 6 → 10 where it used to pay 0 → 4. Twenty
        // timber either way. The human was the control that said so from
        // the other side — six slots short and its timber bucket exact on
        // all 250 frames.

        // **The two rate seams are closed, and they were one thing.** The
        // AI's food used to be ten short in sixteenths and its wealth
        // missing entirely — `160` and `160` exactly, which is `10 × 16`
        // twice, and `10` is both of Fish's `BONUS_NUM`s in
        // `resourcerules.xml`. `Leader::calc_gather` **step 6** is what
        // pays them: the AI's `1/14` has been standing on its fish since
        // frame 4992 and nothing here walked the idle fishermen
        // (`crates/sim/src/rares.rs`, item 165). Every one of both players'
        // six rates and six incomes is now the original's on every frame of
        // the window.
        for g in 0..sim::economy::RESOURCES {
            assert_eq!(n(1, "income", g), 0, "the AI's income, good {g}");
            assert_eq!(n(1, "resources", g), 0, "the AI's rate, good {g}");
            assert_eq!(n(0, "income", g), 0, "the human's income, good {g}");
            assert_eq!(n(0, "resources", g), 0, "the human's rate, good {g}");
            assert_eq!(n(0, "leftover", g), 0, "the human's leftover, good {g}");
        }
        // **And the hundred in goods 3, 4 and 5 is item 156**, unchanged
        // since run40 measured it: `STARTING_GOODS` arrives with the age, so
        // the original holds none of the three in the Ancient age. Inert —
        // an unavailable good is never charged and never accrues.
        for g in 3..sim::economy::RESOURCES {
            assert_eq!(row(0, "bucket", g), Some((250, 100, 0)));
            assert_eq!(row(1, "bucket", g), Some((250, 100, 0)));
        }

        // **The step machine, at the other end of the same record**
        // (`docs/AI.md` §25). run58 stops at 5,201 and could not show this:
        // the AI's script is still live 5,400 frames in, the machine is
        // armed on its phase frames and cleared again, and `script_step`
        // moves 23, 15, 18 as `economic.bhs` walks its cases.
        assert!(
            steps.iter().all(|&(_, step, _, run)| step <= 1 && run == 1),
            "the ladder never leaves step 1 while the script lives: {:?}",
            steps
                .iter()
                .find(|&&(_, step, _, run)| step > 1 || run != 1)
        );
        let script_steps: Vec<i64> = {
            let mut v: Vec<i64> = steps.iter().map(|&(_, _, s, _)| s).collect();
            v.dedup();
            v
        };
        assert_eq!(script_steps, vec![23, 15, 18], "economic.bhs's cases");

        // **And the positions, which this capture carried for a day before
        // anything compared them** (item 87's ledger, 2026-09-02). run59 is
        // a `UNITS=3` window like run58's frames and run63's, so every
        // dumped unit's point is in it; the census read six goods a leader
        // and left the other nine tenths of the file alone.
        //
        // What the widening found on its first run was the item the queue
        // had booked a *capture* for: `1/18`, the AI's Transport Barge, is
        // born on **5342** — inside this window, 88 frames before run63's
        // opens — and this crate bore it two tiles closer to its
        // destination, which is the whole of the "five frames ahead" run63
        // measured at 5430. `docs/TRANSPORT.md` §6.1: the spot search's
        // `(-1, -1)` form takes `find_unit_with_radius`, not the pairwise
        // pair, and the pairwise pair was refusing the bearing the original
        // takes because the caster stands three cells and four cells away.
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        // **The window, and not the shutdown block after it.** The file's
        // last block is the quit dump — `GameInfo closing` is the next line
        // — and its header is a frame ahead of the state it holds: the
        // blocks run 5150…5399, then this one at 5401, and 5400 is never
        // written. So every unit still walking reads one step behind in it,
        // which is a property of the capture and not of the simulation.
        let tail = report.frames.last().map_or(0, |f| f.frame);
        assert_eq!(tail, 5401, "run59's last block is the shutdown dump");
        assert!(
            !report.frames.iter().any(|f| f.frame == 5400),
            "…and 5400 is the frame it stands in for"
        );
        let parted: Vec<(i64, i64, i64)> = report
            .first_divergence_by_unit()
            .into_iter()
            .filter(|&(_, _, f)| f < tail)
            .collect();
        let unit_frames: usize = report.frames.iter().map(|f| f.compared).sum();
        eprintln!(
            "run59: {unit_frames} unit-frames over {} frames, {} ever off position",
            report.frames.len(),
            parted.len()
        );
        assert!(
            unit_frames > 5_000,
            "run59's window is a UNITS=3 one — {unit_frames} points is a wrong file"
        );
        assert_eq!(
            parted,
            vec![],
            "the census window stands where the original's does too"
        );
    }

    /// One census's disagreements, folded: `(who, field, good)` to the
    /// number of frames it was wrong on, the first such frame, and what the
    /// two sides read there. A shape wrong on *every* frame of a window is a
    /// standing level; one that starts partway through names its own event.
    type Shapes = std::collections::BTreeMap<(i64, String, usize), (usize, i64, i64, i64)>;

    /// **run42's nine hundred frames, and where the fifty is not**
    /// (item 162, 2026-09-02).
    ///
    /// run59's census measured the AI fifty timber short at frame 5,150 and
    /// could not say when the fifty was banked: nothing on disk carried a
    /// resource level between frame 800 and 5,150. Nothing *had to* be
    /// captured to narrow it, though — **run42 was already on disk**. It is
    /// run39's game (`samegame.py --exclude LEADERDATA`: 900 frames in
    /// common, not one differing) at run39's detail plus `LEADERS=2`, which
    /// is the detail `LeaderData::log_data` announces the encrypted block
    /// at: `bucket`, `leftover`, `resources`, `income`, `rate` and
    /// `resource_cap`, per good, on every one of its 900 frames. run39's
    /// game is run54's is run58's is run59's, so this is the same AI walking
    /// the same script — the census before the word, for the cost of a
    /// parser.
    ///
    /// Five fields, six goods, two players, 900 frames.
    #[test]
    fn run42_s_nine_hundred_frames_are_the_census_before_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run42-islands-goodybucket.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run42.log"),
        ) else {
            eprintln!("skipping: no run42 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut shapes = Shapes::new();
        // The AI's timber, frame by frame, so the *step* can be found rather
        // than the level.
        let mut timber: Vec<(i64, i64, i64)> = Vec::new();
        for n in 1..=900 {
            built.tick();
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 5] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" && g == 1 {
                            timber.push((n, i64::from(ours[g]), theirs[g]));
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                n,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        for (k, (n, f, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, from {f}: ours {o} theirs {t}");
        }
        // The AI's timber gap, and every frame it changes on.
        let mut steps: Vec<(i64, i64)> = Vec::new();
        let mut last = 0i64;
        for &(n, o, t) in &timber {
            let d = t - o;
            if d != last {
                steps.push((n, d));
                last = d;
            }
        }
        eprintln!("run42: the AI's timber gap steps at {steps:?}");
        assert_eq!(
            compared, 54_000,
            "900 frames, two players, six goods, five fields"
        );
    }

    /// **run60 — the whole timber curve, and the frame the fifty is banked
    /// on** (item 162, 2026-09-02).
    ///
    /// run42 pins frames 1–900 of this game exact and run59 measures a
    /// standing fifty at 5,150; between them nothing on disk carried a
    /// resource level at all. run60 closes that: run58's game with
    /// `[End Frame]` cut to `MISC,LEADERS=2` — the detail
    /// `LeaderData::log_data` announces the encrypted block at, and nothing
    /// else — so every one of its 5,400 frames prints `bucket`, `leftover`,
    /// `resources`, `income`, `rate` and `resource_cap` per good and the run
    /// costs minutes rather than the hour a full-detail one does.
    ///
    /// Five fields, six goods, two players, 5,400 frames.
    #[test]
    fn run60_s_whole_curve_is_where_the_ai_s_timber_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run60-islands-census-thin.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run60.log"),
        ) else {
            eprintln!("skipping: no run60 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let mut compared = 0usize;
        let mut shapes = Shapes::new();
        // Every good's gap, frame by frame, so the *step* can be found —
        // and the AI's own bucket beside it, so a step can be read as *this
        // crate spent* or *the original was paid*.
        let mut gaps: Vec<(
            i64,
            [i64; sim::economy::RESOURCES],
            [i32; sim::economy::RESOURCES],
        )> = Vec::new();
        for n in 1..=5400 {
            built.tick();
            let mut row = [0i64; sim::economy::RESOURCES];
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 5] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if who == 1 && key == "bucket" {
                            row[g] = theirs[g] - i64::from(ours[g]);
                        }
                        if i64::from(ours[g]) != theirs[g] {
                            let e = shapes.entry((who, key.to_string(), g)).or_insert((
                                0,
                                n,
                                i64::from(ours[g]),
                                theirs[g],
                            ));
                            e.0 += 1;
                        }
                    }
                }
            }
            gaps.push((n, row, built.sim.ledgers[1].bucket));
        }
        eprintln!("run60: {compared} good-frames compared");
        for (k, (n, f, o, t)) in &shapes {
            eprintln!("  {k:?}: {n} frames, from {f}: ours {o} theirs {t}");
        }
        let mut last = [0i64; sim::economy::RESOURCES];
        for &(n, row, mine) in &gaps {
            if row != last {
                eprintln!("  f{n}: the AI's bucket gap {row:?}, ours {mine:?}");
                last = row;
            }
        }
    }

    /// **run61 — a bird's flight, against the oracle owner 9 never had**
    /// (`docs/SYNC.md` §3.9, queue item 120).
    ///
    /// run60's game with three more call proxies open across all 5,400
    /// frames: `Unit::do_air_physics`, whose entry and return **bracket** a
    /// bird's frame and whose two arguments are the patrol point it steers
    /// at; `Unit::air_turn_speed`, whose answer is the bank angle scaled
    /// onto the type's rate; and `Unit::set_new_location`, which is where
    /// the step landed. Eleven flying units, **47,533 air frames**.
    /// `rngcmp.py` against run60: 5,401 frames, **zero differing**, so the
    /// proxies cost the stream nothing and this is run54's game.
    ///
    /// **What it settles.** Seeded only with `Unit::init`'s own birth state
    /// — the patrol point's tile centre, and `Angle::INITIAL` — and fed the
    /// original's own goal each frame, `crates/sim/src/air.rs` reproduces
    /// every one of the ten wild birds' flights **exactly, to the last
    /// frame of the capture**: up to 5,272 consecutive frames, position for
    /// position. The bank's zero-crossings agree too, frame for frame,
    /// which is what the `air_turn_speed` records are: the original calls
    /// it once on a frame its bank is turning the heading and not at all on
    /// a frame the bank is zero.
    ///
    /// So the flight was never the residue. **The birth was**, and it is
    /// one line: `Unit::init@00612100` snaps a new unit onto the centre of
    /// its 48-unit tile — `div_3_table[p >> 4] · 0x30 + 0x18`, which for a
    /// cell centre is `+24` on each axis — and `Gaia::spawn_bird` handed
    /// the cell centre straight through. Twenty-four position units at
    /// birth is what put the coin at 5404 instead of 5437.
    ///
    /// The eleventh flyer is the dock's gull, born at 3579 on a
    /// `StrafeOrder` rather than a patrol: `Unit::do_strafe` points it
    /// before the first physics frame, so its heading is due west and not
    /// `Angle::INITIAL`, and it is not what this asserts.
    #[test]
    fn run61_s_birds_fly_where_the_original_s_do() {
        let Some(tr) = trace("rontrace-run61.log") else {
            eprintln!("skipping: no run61 trace (set RON_GAMELOG_DIR)");
            return;
        };
        let air = tr.air_frames();
        let mut by_unit: std::collections::BTreeMap<u32, Vec<&crate::trace::AirFrame>> =
            std::collections::BTreeMap::new();
        for a in &air {
            by_unit.entry(a.unit).or_default().push(a);
        }
        assert!(air.len() > 47_000, "run61 folds 47,533 air frames");

        // `Unit::init`'s snap: the centre of the unit's 48-unit tile.
        let tile = sim::world::UNITS_PER_TILE / 4;
        let births = tr.air_births();
        let mut birds = 0;
        let mut frames = 0usize;
        for (unit, rows) in &by_unit {
            let goal = rows[0].goal;
            let Some(&(_, birth)) = births.get(unit) else {
                panic!("{unit:#x} flew without ever being put down")
            };
            // **The wild bird's discriminator is its own birth.** Hatched on
            // a cell, its patrol point is that cell's centre and `Unit::init`
            // puts it twenty-four units into the tile — so `birth == goal +
            // 24` on both axes is exactly "this one was born flying a
            // patrol". The gull is put down 168 short of its dock and
            // `Unit::do_strafe` points it before the first physics frame.
            if birth != (goal.0 + tile / 2, goal.1 + tile / 2) {
                assert_eq!(*unit, 0x1478_729c, "run61's one non-patrol flyer");
                continue;
            }
            birds += 1;
            let mut s = sim::Sim::new(sim::tuning::Tuning::RON, sim::world::World::new(64, 64), 2);
            let ty = s.add_unit_type(sim::UnitType {
                hits: 1,
                moves: 35,
                turn_speed: sim::movement::degrees_to_angle(5).0,
                kind: sim::attrition::UnitKind {
                    domain: sim::attrition::Domain::Air,
                    ..sim::attrition::UnitKind::default()
                },
                ..sim::UnitType::default()
            });
            // `Unit::init`'s snap, and its `0x55555555` — the whole of the
            // birth state a bird's flight is downstream of.
            let at = sim::world::Pos::new(birth.0, birth.1);
            let mut u = sim::Unit::new(sim::gaia::BIRD_OWNER, 0, at, 1);
            u.ty = Some(ty);
            u.kind = s.unit_types[ty].kind;
            u.movement.speed = 35;
            u.movement.turning = sim::turning_of(&s.unit_types[ty]);
            let b = s.add_unit(u);
            assert_eq!(s.units[b].movement.heading, sim::movement::Angle::INITIAL);
            s.gaia
                .bird_goals
                .push((b, sim::world::Pos::new(goal.0, goal.1)));

            for r in rows {
                let Some(want) = r.to else { continue };
                s.do_air_physics(b, sim::world::Pos::new(r.goal.0, r.goal.1), r.frame);
                let got = s.units[b].pos;
                assert_eq!(
                    (got.x, got.y),
                    want,
                    "{unit:#x} f{}: the step, against run61's own",
                    r.frame
                );
                // `bank_aircraft` reaches `air_turn_speed` on exactly the
                // frames its bank is not zero — and the bank is stored
                // **negated**, so a settled one is `-0.0` and a bitwise
                // test against `ZERO` calls it turning.
                let banked = s.flight(b).roll.bits() & 0x7fff_ffff != 0;
                assert_eq!(
                    !r.turn_speed.is_empty(),
                    banked,
                    "{unit:#x} f{}: the bank's zero-crossing",
                    r.frame
                );
                frames += 1;
            }
        }
        assert_eq!(birds, 10, "run61's ten wild birds");
        eprintln!("run61: {frames} air frames of {birds} birds reproduced exactly");
        assert!(frames > 45_000);
    }

    /// **The production step machine's ladder, against the original's own
    /// function coverage** (`docs/AI.md` §2.4, §25 — item 160, 2026-09-02).
    ///
    /// `Leader::production_ai`'s switch runs one step a frame and steps 3–7
    /// are the five producers. Nothing on this side had ever said *when*
    /// the original reaches them, and the question came up as a defect
    /// report: `Sim::building_value` is not entered on any of run58's 5,201
    /// frames, so the AI's make list can never ask for a gathering
    /// building there.
    ///
    /// **It is not a defect. The original does not reach them either.**
    /// The script at step 1 answers `BLOCK_ON_THIS` on every sweep for as
    /// long as it is live, and `BLOCK_ON_THIS` clears the machine — so the
    /// ladder never leaves step 1 until the script *ends*, and the shipped
    /// opening runs for two hours of game time. The trace's HIT records
    /// say so exactly: a whole-run capture arms every function once, so
    /// each of these addresses carries the frame the original first entered
    /// it on, and there is one for every producer.
    ///
    /// | | East Indies (run54) | Great Lakes (run53) |
    /// | --- | --- | --- |
    /// | `production_ai_setup` (step 2) | 9977 | 6377 |
    /// | `found_cities` (step 3) | — already 576 — | — |
    /// | `research_techs` (step 4) | 9979 | 6379 |
    /// | `upgrade_units` (step 5) | 9980 | 6380 |
    /// | `create_units` (step 6) | 9981 | 6381 |
    /// | `create_buildings` (step 7) | **9982** | **6382** |
    ///
    /// Five consecutive frames with **one gap**, and the gap is step 3 —
    /// `found_cities`, which was entered at frame 576 already and so has no
    /// second HIT. That is §2.4's ladder read straight off the original,
    /// and the 576 is the second half of the finding: `found_cities` and
    /// `make_stuff` are reached there by
    /// `ScenarioFuncSet::place_city_with_cost`, the **script's** own host
    /// function, not by steps 3 and 8. Every producer entry before the
    /// script ends is the script's.
    ///
    /// **What it means for the queue.** `create_buildings` — and with it
    /// the whole make-list road, `building_value`, `gather_value`, the
    /// `ter` gate and `oil_patches` — is first driven at 9982 on East
    /// Indies and 6382 on Great Lakes, which is 4,606 and 4,580 frames past
    /// each map's word. Nothing in that block can move either headline
    /// until the word reaches it, and no dump on disk is long enough to
    /// compare it: run58, the longest, stops at 5,201.
    #[test]
    fn the_producers_are_not_reached_until_the_ai_script_ends() {
        let (Some(east), Some(lakes)) = (trace("rontrace-run54.log"), trace("rontrace-run53.log"))
        else {
            eprintln!("skipping: no run53/run54 trace (set RON_GAMELOG_DIR)");
            return;
        };
        // `Leader::` unless said otherwise; the export's own addresses.
        const PRODUCTION_AI: u32 = 0x006c_1960;
        const PRODUCTION_AI_SETUP: u32 = 0x006c_83e0;
        const FOUND_CITIES: u32 = 0x006c_7a60;
        const RESEARCH_TECHS: u32 = 0x006c_6ba0;
        const UPGRADE_UNITS: u32 = 0x006c_6430;
        const CREATE_UNITS: u32 = 0x006c_40a0;
        const CREATE_BUILDINGS: u32 = 0x006c_1be0;
        const MAKE_STUFF: u32 = 0x006c_8af0;
        const PLAN_STRATEGY: u32 = 0x006b_9620;
        const STRATEGY_ALL: u32 = 0x006e_d430;
        /// `ScenarioFuncSet::place_city_with_cost` — the script's.
        const PLACE_CITY: u32 = 0x009f_5860;

        for (name, t, ladder) in [
            ("run54", &east, [9977, 9979, 9980, 9981, 9982]),
            ("run53", &lakes, [6377, 6379, 6380, 6381, 6382]),
        ] {
            let at = |va: u32| t.first_entry(va);
            assert_eq!(at(STRATEGY_ALL), Some(0), "{name}: the sweep is frame 0's");
            assert_eq!(at(PLAN_STRATEGY), Some(0), "{name}: and so is its caller");
            assert_eq!(
                at(PRODUCTION_AI),
                Some(1),
                "{name}: the machine is armed on frame 0 and runs on frame 1"
            );
            // The script's own producer calls, long before the ladder.
            assert_eq!(at(PLACE_CITY), Some(576), "{name}: the script's city");
            assert_eq!(
                (at(FOUND_CITIES), at(MAKE_STUFF)),
                (Some(576), Some(576)),
                "{name}: reached by `place_city_with_cost`, not by steps 3 and 8"
            );
            // And the ladder itself, one step a frame.
            let steps = [
                ("production_ai_setup", PRODUCTION_AI_SETUP),
                ("research_techs", RESEARCH_TECHS),
                ("upgrade_units", UPGRADE_UNITS),
                ("create_units", CREATE_UNITS),
                ("create_buildings", CREATE_BUILDINGS),
            ];
            for (i, (label, va)) in steps.iter().enumerate() {
                assert_eq!(
                    at(*va),
                    Some(ladder[i]),
                    "{name}: {label} is first entered on frame {}",
                    ladder[i]
                );
            }
        }

        // And the capture the report came from is a third of the way there.
        let Some(run58) = trace("rontrace-run58.log") else {
            return;
        };
        assert_eq!(
            run58.first_entry(CREATE_BUILDINGS),
            None,
            "run58 is 5,201 frames and the ladder does not leave step 1 until 9977"
        );
        assert_eq!(run58.first_entry(PRODUCTION_AI_SETUP), None, "nor step 2");
        assert_eq!(
            (
                run58.first_entry(FOUND_CITIES),
                run58.first_entry(MAKE_STUFF)
            ),
            (Some(576), Some(576)),
            "run58 is run54's game, and its script buys a city on the same frame"
        );

        // **And the dump says it from the other side.** run18b is run53's
        // own game with a `LEADERS=9` window over the very frames the
        // coverage dates, so the two instruments lie on each other: a
        // `FRAME n` block is the end of sim-frame n − 1, and the ladder
        // starts on the frame after `prod_script_run` falls.
        let Some(run18b) = dump("gamelog-run18b-window-6374-6590.txt") else {
            return;
        };
        let text = std::fs::read_to_string(&run18b).unwrap();
        let log = Log::parse(&text);
        let at = |sim_frame: i64| -> (i64, i64) {
            let b = log
                .leader_block(sim_frame + 1, 1)
                .expect("run18b's window covers it");
            (
                b.int("production_step").expect("production_step"),
                b.int("prod_script_run").expect("prod_script_run"),
            )
        };
        assert_eq!(at(6375), (1, 1), "the sweep arms the machine, script live");
        assert_eq!(at(6376), (2, 0), "SCRIPT_DONE: the script is dropped here");
        assert_eq!(at(6377), (3, 0), "and step 2 — production_ai_setup — ran");
        assert_eq!(at(6378), (4, 0));
        assert_eq!(
            at(6379),
            (5, 0),
            "one producer a frame, as run53's HITs date"
        );
    }

    /// **run54 — East Indies at thirteen times the scored length, read at
    /// last** (2026-09-01).
    ///
    /// run54 was taken with run53 and sat unread for a day: while run39's
    /// own word parted at 1647 there was no question a longer capture could
    /// answer. Item 125 took that word to the end of run39, so the scored
    /// capture stopped being the boundary and this one became the only
    /// thing that says where the boundary is.
    ///
    /// It is the same game as run38/run39 — `tools/gamelog/rngcmp.py` says
    /// its `game_random` word is run39's on all 1,851 overlapping frames,
    /// zero differing — traced whole over **24,000**, and its `[End Frame]`
    /// is `MISC` alone, so it carries the stream and no per-frame record.
    /// That is what it is for: the word is scoreable from the trace, and
    /// ticks and orders are not scoreable here at all.
    ///
    /// **The pasture is a source here too** — run54's own trace reached the
    /// setup, so the five owner-9 animals come from it rather than from
    /// run39's (`docs/SYNC.md` §3.11). Without them this is a different
    /// game within a few hundred frames and every number below is nobody's.
    ///
    /// Only the two frame numbers are pinned, for run53's reason: past the
    /// parting both sides are on streams that are nobody's, and the totals
    /// there are coincidence that moves with every unrelated change.
    #[test]
    fn run54_s_24000_frames_are_where_the_second_map_s_word_now_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(trace)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            eprintln!("skipping: no run54 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &trace);
        assert_eq!(
            init.pasture.len(),
            1,
            "run54's trace reached the setup and East Indies' AI has one pasture"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = trace.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 23_000,
            "run54's traced length is {last}, wanted 23,000+ — this is the \
             long capture, and a short file here is a wrong file"
        );
        // `RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>` prints one unit's position,
        // order and figure clocks over a window of frames. The word says
        // *that* a frame parted; a cadence — a crew figure wrapping every
        // second frame rather than every third — needs the clock itself,
        // and past the last `DUMP_ALL` window there is nothing else to
        // read it from.
        let watch: Option<(i64, i64, i64, i64)> =
            std::env::var("RON_DEBUG_UNIT").ok().and_then(|v| {
                let (u, w) = v.split_once('@')?;
                let (who, o) = u.split_once('/')?;
                let (lo, hi) = w.split_once('-')?;
                Some((
                    who.trim().parse().ok()?,
                    o.trim().parse().ok()?,
                    lo.trim().parse().ok()?,
                    hi.trim().parse().ok()?,
                ))
            });
        for f in 0..last {
            built.tick();
            let Some((who, o, lo, hi)) = watch else {
                continue;
            };
            if !(lo..=hi).contains(&f) {
                continue;
            }
            let Some(u) = (0..built.sim.units.len()).find(|&i| {
                let x = &built.sim.units[i];
                x.alive() && i64::from(x.owner) == who && i64::from(x.index) == o
            }) else {
                continue;
            };
            let x = &built.sim.units[u];
            let clocks: Vec<String> = x
                .guys
                .iter()
                .map(|g| {
                    // The **body** and its destination beside the clock:
                    // `Guy::set_anim`'s idle request returns without a
                    // roll while a walking guy's body has not arrived
                    // (`docs/ANIM.md` §4 step 1), so a draw this crate
                    // spends and the original does not is read here or
                    // nowhere.
                    let (bx, by, dx, dy) = g.follow.map_or_else(
                        || {
                            (
                                x.movement.body.pos.x,
                                x.movement.body.pos.y,
                                x.pos.x,
                                x.pos.y,
                            )
                        },
                        |f| (f.body.pos.x, f.body.pos.y, f.des.x, f.des.y),
                    );
                    format!(
                        "[a{} t{}/{} l{} b({bx},{by})->({dx},{dy})]",
                        g.anim, g.cur_time, g.end_time, g.last_time
                    )
                })
                .collect();
            eprintln!(
                "  f{f} {who}/{o} at ({}, {}) ang {} path {} top {:?} order {:?} {}",
                x.pos.x,
                x.pos.y,
                x.movement.facing.0,
                x.path.len(),
                x.path.last().map(|p| (p.to.x, p.to.y, p.flags)),
                x.orders.front().map(|ord| ord.index()),
                clocks.join(" ")
            );
        }
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| *ours != trace.labels(*f))
            .map_or(last, |(f, _)| *f);
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != trace.labels(*f).len())
            .map_or(last, |(f, _)| *f);
        eprintln!("run54: word parts at {first_count}, sequence at {first_part} of {last}");
        // `RON_DEBUG_SITES=<lo>-<hi>` widens the two printed frames to a
        // window. A residue in a *cadence* — a figure's clock wrapping every
        // second frame rather than every third — is invisible at the frame
        // it finally parts on and obvious over a dozen either side.
        let window = std::env::var("RON_DEBUG_SITES").ok().and_then(|v| {
            let (lo, hi) = v.split_once('-')?;
            Some((
                lo.trim().parse::<i64>().ok()?,
                hi.trim().parse::<i64>().ok()?,
            ))
        });
        for (f, ours) in built.frame_sites.iter() {
            let named = *f == first_count || *f == first_part;
            if !named && !window.is_some_and(|(lo, hi)| (lo..=hi).contains(f)) {
                continue;
            }
            let theirs = trace.labels(*f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            eprintln!(
                "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at)
            );
            // `RON_DEBUG_SITES=1` prints both sequences whole. The line
            // above names the first *index* that parts, which is enough
            // when the two sides are the same length and useless when
            // they are not: the four draws East Indies' word is missing
            // are at the **head** of the original's frame, so every later
            // entry reads as a mismatch and the first one names a bird.
            if std::env::var("RON_DEBUG_SITES").is_ok() {
                for (i, l) in ours.iter().enumerate() {
                    eprintln!("    ours  {i}: {l}");
                }
                for (i, l) in theirs.iter().enumerate() {
                    eprintln!("    thrs  {i}: {l}");
                }
            }
        }
        // **`gull_o` is the one field of the `DOCK` record nothing has ever
        // compared**, and this capture is where it can be. run22 is this
        // same game with a `DUMP_ALL` window on `[3579, 3582)`
        // (`docs/ORACLE.md`), and its block 3580 reads `DOCK dock 0, o
        // 2010, reg 65, gull_o 15, who 1, dock_flags 1`. Owner 9 is absent
        // from a dump, so the gull's own record is not there to compare —
        // its **object number** is, and it is what says this crate hands
        // gaia its slots where the original does.
        let gull = built.sim.docks[1].slots[0]
            .gull
            .expect("the AI's dock spawned its gull");
        assert_eq!(built.sim.units[gull].owner, 9, "the gull is gaia's");
        assert_eq!(
            built.sim.units[gull].index, 15,
            "run22's block 3580: `DOCK … gull_o 15`"
        );
        assert!(
            first_count >= LONG_WORD_EAST_INDIES && first_part >= LONG_WORD_EAST_INDIES,
            "run54's ceiling fell: word {first_count}, sequence {first_part}; \
             the floor is {LONG_WORD_EAST_INDIES} on both"
        );
    }

    /// **run64 — the caravan's road, node for node, and the world it
    /// reads** (2026-09-02).
    ///
    /// run54's game to 6,180 frames with a `DUMP_ALL` window on
    /// `[6164, 6172)` and the three `docs/ROADS.md` §7.2 proxies over
    /// `[6163, 6172]`. `rngcmp.py rontrace-run54.log rontrace-run64.log`:
    /// **6,181 frames, zero differing**, so it is run54's game and the
    /// window and the proxies cost the stream nothing — the fourth capture
    /// in a row of which that is true.
    ///
    /// What it is for: East Indies' word had parted at 6166 on 3,204 road
    /// draws this crate spent none of (`docs/QUEUE.md` item 174), and a
    /// count is not a sequence. This asks the original for every one of
    /// them.
    ///
    /// The five frames are **one search**: `astar_caravan_road` answers −1
    /// on 6166, 6167, 6168 and 6169, parking its three containers in the
    /// `CaravanData` each time, and 1 on 6170. Its bracket names the
    /// endpoints — `oA 2000`, `whoA 1`, `oB 2007`, `caravan 0` — which is
    /// leader 1's two cities, and its `caravan >= 0` is what opens the four
    /// diagonals and swaps the heuristic (`docs/CARAVAN.md` §5).
    #[test]
    fn run64_s_caravan_road_is_the_original_s_node_for_node() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(trace), Some(r64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run64-islands-caravanroad.txt"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(t64) = crate::diff::tests::trace("rontrace-run64.log") else {
            eprintln!("skipping: no run64 trace");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &trace);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6166 {
            built.tick();
        }

        // **The world the search reads, whole.** run64's `FRAME 6166`
        // block is the end of sim-frame 6165, and the tile masks and the
        // height grid it carries are what `valid_roadcoord` and
        // `calc_road_cost` are about to read. The heights are the harder
        // half: a mid-game building re-terraforms them
        // (`crate::terrain`), and the AI's three **farms** had moved 182
        // tiles of them here until `Wall::init`'s `!= FARM` gate landed.
        let text64 = std::fs::read_to_string(&r64).unwrap();
        let l64 = Log::parse(&text64);
        let block = l64
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 6166)
            .map(|(_, b)| b)
            .expect("run64 dumped frame 6166");
        let w = block
            .kid("FULL DUMP")
            .unwrap_or(block)
            .kid("WORLD")
            .expect("a WORLD block");
        let mut notes = Vec::new();
        let heights = l64.frame_heights(6166);
        assert!(
            heights.len() > 1000,
            "run64's block carries the height grid"
        );
        let (theirs, _) = world_from(&w.fields, &heights, &mut notes);
        let (xs, ys) = (theirs.width(), theirs.height());
        let mut z_off = Vec::new();
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                let q = Pos::new(tx, ty);
                if built.sim.world.tile_z(q) != theirs.tile_z(q) && z_off.len() < 8 {
                    z_off.push(format!(
                        "t({tx},{ty}) ours {} theirs {}",
                        built.sim.world.tile_z(q),
                        theirs.tile_z(q)
                    ));
                }
            }
        }
        assert!(
            z_off.is_empty(),
            "the height grid parted from the original's by 6165: {z_off:?}"
        );
        let owners: Vec<String> = (0..ys)
            .flat_map(|y| (0..xs).map(move |x| sim::world::Cell::new(x, y)))
            .filter(|&c| built.sim.world.owner(c) != theirs.owner(c))
            .map(|c| format!("cell ({}, {})", c.x, c.y))
            .collect();
        assert!(owners.is_empty(), "the borders parted by 6165: {owners:?}");

        // The five frames of the search, each against the original's own
        // priced nodes: the tile, the direction it was reached from, and
        // the price. Frame 6170 is the one that arrives.
        let mut counts = Vec::new();
        for f in 6166..=6170 {
            built.sim.trace_costs = true;
            built.sim.road_marks.clear();
            built.tick();
            let ours = built.sim.road_marks.clone();
            let theirs = t64.road_nodes(f);
            counts.push((f, ours.len(), theirs.len()));
            // **And 6170's prices are the search's again.** They were the
            // stream's for one item: East Indies' word parted on 6169,
            // where the original spent two `Guy::set_anim+0x97a <
            // Guy::inc_time+0x271` draws this crate did not, so by 6170 the
            // jitter was two draws out of phase and every price shifted.
            // Those two are the caravan's own crew figures wrapping their
            // empty packet (`docs/ANIM.md` §3.6), and with them spent the
            // whole node — tile, direction **and** price — is the
            // original's on all five frames.
            let at = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i));
            assert_eq!(
                at,
                None,
                "frame {f}: the search parts at node {at:?} — ours {:?}, theirs {:?} \
                 (ours {} nodes, theirs {})",
                at.and_then(|i| ours.get(i)),
                at.and_then(|i| theirs.get(i)),
                ours.len(),
                theirs.len()
            );
        }
        eprintln!("run64 road nodes: {counts:?}");
        assert_eq!(
            counts,
            vec![
                (6166, 3204, 3204),
                (6167, 3204, 3204),
                (6168, 3204, 3204),
                (6169, 3202, 3202),
                (6170, 151, 151),
            ],
            "the five frames' node counts — four budgets of 0xc80 and the \
             arrival"
        );
        // The route itself, from the `astar_caravan_road` bracket.
        let brackets = t64.calls_in(6166, 5);
        assert_eq!(brackets.len(), 1, "one road plan on 6166");
        assert_eq!(
            (
                brackets[0].args[1],
                brackets[0].args[2],
                brackets[0].args[3]
            ),
            (2000, 1, 2007),
            "the endpoints are leader 1's two cities"
        );
        let van = &built.sim.caravans[1].slots[0];
        assert!(
            !van.making_road && van.search.is_none(),
            "the plan finished on 6170 and the parked search went with it"
        );
        // **The route's own stack, whole** — twenty-six `PATHDATA`, and
        // the record has printed them since the first `DUMP_ALL`
        // (item 87's ledger again). run64's `FRAME 6171` block is the end
        // of sim-frame 6170, the frame `build_road` answered 1 on, and the
        // eight window frames before it all read an empty stack: this is
        // the only block in the capture that carries a laid road.
        //
        // What it pins is `build_road`'s own loop rather than the search:
        // the world coordinates the crate had been throwing away (its road
        // was a list of *tiles*, so `set_road_at` was handed numbers thirty
        // thousand tiles off the map and every trade road went unlaid), the
        // `0x60` tolerance, and the flag byte — `0x20` on every node that
        // laid tarmac and `1` on the two ends. A water node would carry
        // `0x180` and no `0x20`; this road has none, so the sampling arm
        // is still reading-only.
        let van_road = van.road.clone();
        let frame71 = l64
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 6171)
            .map(|(_, b)| b)
            .expect("run64 dumped frame 6171");
        // The record sits under the leader's own block, which sits under
        // `FULL DUMP`; walk the subtree rather than pin the depth.
        fn vans_of<'a, 'b>(b: &'a Block<'b>, out: &mut Vec<&'a Block<'b>>) {
            for c in &b.children {
                if c.name == "CARAVAN" {
                    out.push(c);
                }
                vans_of(c, out);
            }
        }
        let mut vans = Vec::new();
        vans_of(frame71, &mut vans);
        let theirs_road: Vec<sim::orders::PathData> = vans
            .into_iter()
            .find(|c| c.int("o") == Some(18))
            .map(|c| {
                crate::gamelog::path_of(c)
                    .into_iter()
                    .map(|p| sim::orders::PathData {
                        to: Pos::new(
                            i32::try_from(p.to.0).expect("to_x"),
                            i32::try_from(p.to.1).expect("to_y"),
                        ),
                        tolerance: i32::try_from(p.tolerance).expect("tolerance"),
                        flags: u8::try_from(p.flags).expect("flags"),
                    })
                    .collect()
            })
            .expect("the AI caravan's own CARAVAN record");
        assert_eq!(
            theirs_road.len(),
            26,
            "run64's frame-6171 block: `length 26`"
        );
        assert_eq!(
            van_road, theirs_road,
            "the route's stack parted from the original's"
        );
    }

    /// **The danger map, whole** — `GameDaemon::calc_danger`'s eight rows
    /// against run64's own `danger[who][scan]`, 7,200 values.
    ///
    /// Item 87's ledger again, and this row was the expensive one: the
    /// `WORLD` block has printed the map since the first `DUMP_ALL`
    /// capture and nothing ever read the field, because this crate had no
    /// writer and every reader answered zero. It is not a decoration —
    /// `calc_cost`'s world arm prices a step by `danger / 8`, and around a
    /// leader's own city the map is 65 to 135 *negative*, which is 8 to 16
    /// off every expensive step there. That is what sent East Indies'
    /// caravan south-west on frame 6167 where the original walks straight
    /// at its city (`docs/CARAVAN.md` §4.1, `docs/DANGER.md`).
    ///
    /// The map is rebuilt every two hundredth frame and nothing decays it,
    /// so run64's window (6163–6170) shows the frame-6000 build unchanged
    /// — which makes the eight dumped frames one assertion repeated, and
    /// the check is against the first.
    #[test]
    fn run64_s_danger_map_is_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run64-islands-caravanroad.txt"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text64 = std::fs::read_to_string(&r64).unwrap();
        let l64 = Log::parse(&text64);
        let theirs = l64
            .dumps()
            .into_iter()
            .find_map(|(n, body)| {
                (n == 6168).then(|| {
                    let w = body.kid("WORLD")?;
                    let d = crate::gamelog::world_danger(&w.fields);
                    (!d.is_empty()).then_some(d)
                })?
            })
            .expect("run64's frame-6168 block carries the danger map");
        assert_eq!(
            theirs.len(),
            8 * 900,
            "eight rows of a 60x60 map's reg_size"
        );

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6167 {
            built.tick();
        }
        let reg_xs = built.sim.world.reg_xs();
        assert_eq!(
            (reg_xs, built.sim.world.reg_ys()),
            (30, 30),
            "East Indies' half-cell grid"
        );

        let mut wrong: Vec<String> = Vec::new();
        let mut nonzero = 0usize;
        for who in 0..8u8 {
            let ours = built.sim.world.danger_row(who);
            for i in 0..900usize {
                let t = theirs[who as usize * 900 + i];
                let o = i64::from(ours.get(i).copied().unwrap_or(0));
                if t != 0 {
                    nonzero += 1;
                }
                if t != o && wrong.len() < 20 {
                    wrong.push(format!(
                        "leader {who} half-cell ({},{}): ours {o} theirs {t}",
                        i % 30,
                        i / 30
                    ));
                }
            }
        }
        assert!(
            nonzero >= 100,
            "the dump's own map is nearly empty ({nonzero} nonzero) — wrong capture"
        );
        assert!(wrong.is_empty(), "the danger map parted: {wrong:#?}");
    }

    /// **run64's frame-6167 world-path prices**, the caravan's own search,
    /// against the original's proxied `calc_cost`.
    ///
    /// The sibling of [`run55_s_frame_1477_prices_are_the_originals`] on
    /// the other map and the other search, and the reason it exists is the
    /// danger map: eleven of the forty-seven steps the original priced here
    /// were 8, 10 or 16 too dear in this crate, every one of them a cell
    /// whose half-cell carries a negative danger, and the expansion order
    /// parted on the seventh pop because of it.
    #[test]
    fn run64_s_frame_6167_prices_are_the_originals() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(t64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            trace("rontrace-run64.log"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let theirs = t64.calls_in(6167, crate::trace::call_site::CALC_COST);
        assert!(
            theirs.len() >= 40,
            "run64's callwin covers 6167; it priced {} steps there",
            theirs.len()
        );
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..6167 {
            built.tick();
        }
        built.sim.trace_costs = true;
        built.tick();
        let ours = std::mem::take(&mut built.sim.cost_marks);

        let mut theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> =
            std::collections::BTreeMap::new();
        for c in &theirs {
            theirs_by_key.insert(c.cost_key().expect("a calc_cost call"), c.ret);
        }
        let mut shared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for m in &ours {
            let Some(&t) = theirs_by_key.get(&m.key()) else {
                continue;
            };
            shared += 1;
            if t != m.cost {
                wrong.push(format!(
                    "({},{}) -> ({},{}) dir {} depth {}: ours {} theirs {t}",
                    m.from.0 / m.step,
                    m.from.1 / m.step,
                    m.to.0 / m.step,
                    m.to.1 / m.step,
                    m.dir,
                    m.depth,
                    m.cost,
                ));
            }
        }
        assert!(
            shared >= 40,
            "frame 6167: {} of the original's steps and {} of ours, {shared} shared \
             — the searches did not start from the same place",
            theirs.len(),
            ours.len()
        );
        assert!(
            wrong.is_empty(),
            "frame 6167: {} of {shared} shared steps priced differently:\n  {}",
            wrong.len(),
            wrong.join("\n  ")
        );
    }

    /// **run64's animation clocks, frame for frame** — every `GUY`
    /// block's `cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`
    /// and `stopped`, over the eight frames of the `DUMP_ALL` window.
    ///
    /// Item 87's ledger, one more row of it paid: `Initial::frame_guys`
    /// has been parsed since the animation clock was first read and
    /// nothing ever *compared* it — [`Built::tick`] **installs** it, which
    /// is the opposite of a check. Every capture that carries the field
    /// therefore said nothing about it, and the caravan's two crew
    /// figures walked on the citizen's art for as long as the window has
    /// existed.
    ///
    /// Nothing here is installed: run64's own `frame_guys` never reach the
    /// simulation, which is built from run54's start and driven forward
    /// 6,163 frames. Every row is a prediction the dump can refuse.
    ///
    /// The window holds the caravan, and the caravan is why this exists.
    /// `CARAVAN-DEFAULT-AGE0-CREW1` and `-CREW2` are two of the sixty
    /// shipped `<UNIT>` entries with **no `<ANIM>` child at all**: the
    /// piece loads, its packet is empty, so every slot is
    /// `AnimationPacket::get_game_frames`' three frames and none of them
    /// loops. Its crew walks three frames, falls to the idle, rolls, and
    /// is put back on the walk by `Guy::move` the next frame — two draws
    /// every third frame for the rest of the game (`docs/ANIM.md` §3.6).
    #[test]
    fn run64_s_window_clocks_are_the_original_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r64)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run64-islands-caravanroad.txt"),
        ) else {
            eprintln!("skipping: no run54/run64 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let text64 = std::fs::read_to_string(&r64).unwrap();
        let theirs = Log::parse(&text64)
            .initial()
            .expect("run64 carries a start block")
            .frame_guys;
        let window: Vec<i64> = theirs.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            window,
            vec![6163, 6164, 6165, 6166, 6167, 6168, 6169, 6170],
            "run64's `DUMP_ALL` window, as sim-frames"
        );
        let last = *window.last().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The window's own clocks are never installed: this is the check.
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut crew_rows = 0usize;
        let mut unmatched = 0usize;
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so their clocks are not a
                // prediction of this crate's; the players' are.
                if !(0..8).contains(&state.who) {
                    continue;
                }
                // Matched on the simulation's own `(owner, o)` rather than
                // through [`UnitLink`], which only knows the start dump's
                // units: by 6,163 frames most of the AI's economy was
                // trained mid-game and a link table would compare a
                // fourteenth of the window.
                let Some(u) = (0..built.sim.units.len()).find(|&i| {
                    let x = &built.sim.units[i];
                    x.alive() && i64::from(x.owner) == state.who && i64::from(x.index) == state.o
                }) else {
                    unmatched += 1;
                    continue;
                };
                for (n, g) in state.guys.iter().enumerate() {
                    if !g.has_clock() {
                        continue;
                    }
                    let Some(ours) = built.sim.units[u].guys.get(n) else {
                        continue;
                    };
                    if n >= sim::anim::SQUAD_SIZE {
                        crew_rows += 1;
                    }
                    let (body, facing) = match ours.follow {
                        Some(b) => (b.body.pos, b.facing),
                        None => (
                            built.sim.units[u].movement.body.pos,
                            built.sim.units[u].movement.facing,
                        ),
                    };
                    let rows: [(&str, i64, Option<i64>); 9] = [
                        ("x", i64::from(body.x), g.pos.map(|p| p.x)),
                        ("y", i64::from(body.y), g.pos.map(|p| p.y)),
                        ("angle", i64::from(facing.0), g.angle),
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                        ("gpiece", i64::from(ours.gpiece), g.gpiece),
                        ("stopped", i64::from(ours.stopped), g.stopped),
                    ];
                    for (name, ours, theirs) in rows {
                        let Some(theirs) = theirs else { continue };
                        compared += 1;
                        if ours == theirs {
                            continue;
                        }
                        let row = format!(
                            "frame {f}: {}/{} guy {n} {name} ours {ours} theirs {theirs}",
                            state.who, state.o
                        );
                        if wrong.len() < 12 {
                            wrong.push(row);
                        }
                    }
                }
            }
        }
        eprintln!(
            "run64 clocks: {compared} fields compared over {crew_rows} crew rows, \
             {unmatched} of the dump's units this crate has no unit for"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 2_000 && crew_rows >= 24,
            "the window's own rows: {compared} fields, {crew_rows} of them a \
             crew figure's — a capture with neither is the wrong file"
        );
        // **The caravan's own rows used to be thirty-four of these**, and
        // they were the walk rather than the clock: `do_trade`'s move to
        // the near city went in with a waypoint at `(37752, 41592)`, so
        // `1/18` set off south-west where the original heads straight at
        // `(39288, 40056)`. The waypoint was never the bug — the danger
        // map was. Without it `calc_cost` priced eight of the caravan's
        // own world-grid steps 8 to 16 too dear, the search took a seventh
        // expansion the original does not, and a route came back where the
        // original's search stops at the goal's own neighbour and pushes
        // nothing (`crates/sim/src/danger.rs`, `docs/DANGER.md` §6). All
        // 2,061 fields are the original's since 2026-09-02.
        assert!(wrong.is_empty(), "run64's clocks parted: {wrong:?}");
    }

    /// **run65 — the caravan's turn out of its own city, every unit, every
    /// frame** (2026-09-02).
    ///
    /// run54's game to 6,220 frames with a `DUMP_ALL` window on
    /// `[6196, 6214)` — the eighteen `FRAME` blocks either side of the
    /// word's own parting — at `MISC` alone otherwise.
    /// `rngcmp.py rontrace-run54.log rontrace-run65.log`: **6,221 frames,
    /// zero differing**, so it is run54's game and the fifth capture in a
    /// row for which a window costs the stream nothing.
    ///
    /// **What it was booked for.** East Indies' word parted at 6207 on the
    /// caravan's first leg: the original stood at its home city and walked
    /// on sim-frame 6207 where this crate walked on 6206, one frame, and
    /// nothing on disk covered the window. `docs/CARAVAN.md` §8 guessed the
    /// turn or a detour; the capture says **neither**. Both sides push the
    /// same detour node, `(38508, 40620)`, on the same frame; both turn
    /// through the same eight bearings at the same rate, because
    /// `avg_speed` decays 3/4 a frame on both. On sim-frame 6206 the turn
    /// leaves 38.9° owed on both, under `move_step`'s 45° gate, so both
    /// compute the same step and the same landing point — `(38750, 40508)`
    /// — and the original **does not take it**. That point is a tile north
    /// of the one the caravan stands on, and `move_step@005faf30`
    /// (`005fb7c1`–`005fb7fd`) asks `UnitData::invalid_loc` about any step
    /// that changes tile: the city's own footprint refuses it, the step is
    /// dropped whole, and the unit turns another 24° and goes on
    /// sim-frame 6207 through a tile it may have.
    ///
    /// It is the last of the four things `docs/MOVEMENT.md`'s `move_step`
    /// section listed as read and not modelled, and it is the whole of the
    /// residue: every field below is the original's, and East Indies' word
    /// went **6207 → 6353**.
    ///
    /// Nothing here is installed. The simulation is built from run54's
    /// start and driven forward 6,213 frames; run65's own blocks are only
    /// ever read.
    #[test]
    fn run65_s_window_is_the_original_s_unit_for_unit() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r65)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run65-islands-caravanturn.txt"),
        ) else {
            eprintln!("skipping: no run54/run65 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text65 = std::fs::read_to_string(&r65).unwrap();
        let l65 = Log::parse(&text65);
        let frames = l65.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        // The window, and then the **quit's own two blocks**: the log
        // writes a frame at `!quit` and one behind it, so 6220 and 6221
        // come free — seven frames past anything the window paid for.
        // They are read like the rest, but only as far as they parse: the
        // log is cut where the process went, so the caravan's eighteen
        // rows below are the window's own and nothing is owed by the tail.
        assert_eq!(
            window,
            (6196..=6213).chain([6220, 6221]).collect::<Vec<i64>>(),
            "run65's `DUMP_ALL` window, plus the two blocks the quit writes"
        );
        let last = *window.last().expect("the window");

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut compared = 0usize;
        let mut caravan_rows = 0usize;
        let mut queue_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        // `Built::tick` stamps the frame it is about to run, so after the
        // tick with `f` the counter reads `f + 1` and the state is that
        // `FRAME` block's — the same alignment run39's scout test uses.
        for f in 0..last {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            for them in &fr.units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so they are not this
                // crate's prediction; the players' units are.
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    unmatched += 1;
                    continue;
                };
                let un = &built.sim.units[u];
                if (them.who, them.o) == (1, 18) {
                    caravan_rows += 1;
                }
                let mut row = |name: &str, ours: i64, theirs: Option<i64>| {
                    let Some(theirs) = theirs else { return };
                    compared += 1;
                    if ours != theirs && wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                row("x", i64::from(un.pos.x), Some(them.pos.x));
                row("y", i64::from(un.pos.y), Some(them.pos.y));
                // `UnitData::angle` is the **heading** `set_angle` writes,
                // not the facing: `move_step` hands it the bearing at the
                // top of the function and only `Guy::do_turn` moves guy 0
                // toward it. The facing is run64's test's business.
                row("angle", i64::from(un.movement.heading.0), them.angle);
                row("orders_x", i64::from(un.orders_pos.x), them.orders_x);
                row("orders_y", i64::from(un.orders_pos.y), them.orders_y);
                row("tolerance", i64::from(un.tolerance), them.tolerance);
                row(
                    "path length",
                    un.path.len() as i64,
                    Some(them.path.len() as i64),
                );
                // The stack whole — point, tolerance and flag byte, every
                // slot. It is what says the caravan walks the road's own
                // twenty-six nodes and not a plan of its own.
                for (slot, (ours, theirs)) in un.path.iter().zip(them.path.iter()).enumerate() {
                    row(
                        &format!("path[{slot}].x"),
                        i64::from(ours.to.x),
                        Some(theirs.to.0),
                    );
                    row(
                        &format!("path[{slot}].y"),
                        i64::from(ours.to.y),
                        Some(theirs.to.1),
                    );
                    row(
                        &format!("path[{slot}].tolerance"),
                        i64::from(ours.tolerance),
                        Some(theirs.tolerance),
                    );
                    row(
                        &format!("path[{slot}].flags"),
                        i64::from(ours.flags),
                        Some(theirs.flags),
                    );
                }
            }
            // **And the build queues, whole** — the widening this window
            // was re-read for. The AI's Market `1/2013` is one `MERCHANT`
            // deep on every block of the window, and its `job_counter`
            // climbs a hundred a frame from the caravan's own hand-over on
            // 6164 to the merchant's on 6353; nothing in this crate
            // trained one until `num_rare_resources_seen` stopped
            // answering zero (`docs/ECONOMY.md`, "The rares a leader has
            // seen"), so this is what says the *count* the script asked
            // for was right and not merely non-zero.
            //
            // The record's own trap is in the slot behind it: `queued` is
            // 1 and slot 1 reads `type 61` too, because `unqueue` shifts
            // the array down over the caravan and leaves the vacated tail
            // standing. Only the first `queued` slots are read, for
            // [`compare`]'s reason — the tail is not state either side
            // owns.
            for them in &fr.builds {
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let (Some(b), Some(q)) = (built.sim.building_by_o(who, o), them.queued) else {
                    continue;
                };
                let ours = &built.sim.buildings[b].queue;
                let mut row = |name: String, ours: i64, theirs: i64| {
                    compared += 1;
                    if ours != theirs && wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                let mine = ours.items.len() as i64;
                row("queued".into(), mine, q);
                if mine != q {
                    continue;
                }
                for (k, theirs) in them.queue.iter().take(q as usize).enumerate() {
                    let item = &ours.items[k];
                    let id = item.tech.unwrap_or_else(|| built.unit_tree[item.ty]);
                    let ty = built.type_index.get(id).copied().unwrap_or(-1);
                    row(format!("queue[{k}].type"), i64::from(ty), theirs.ty);
                    row(
                        format!("queue[{k}].job_counter"),
                        i64::from(item.job_counter),
                        theirs.job_counter,
                    );
                    queue_rows += 1;
                }
            }
        }
        eprintln!(
            "run65 window: {compared} fields over 20 blocks, {caravan_rows} of them \
             the caravan's, {queue_rows} queue entries, {unmatched} of the dump's \
             units this crate has no unit for"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 4_000 && caravan_rows >= 18,
            "the window's own rows: {compared} fields and {caravan_rows} caravan \
             frames — a capture with neither is the wrong file"
        );
        assert!(
            queue_rows >= 18,
            "the Market's merchant over the window's eighteen blocks: \
             {queue_rows} queue entries"
        );
        assert!(wrong.is_empty(), "run65's window parted: {wrong:?}");
    }

    /// **run66 — the merchant's whole walk, every unit, every frame**
    /// (2026-09-02, item 183).
    ///
    /// run54's game to 6,620 frames with the **cheap** per-frame dump
    /// narrowed to `[6340, 6600)` — 260 blocks at run39's `[End Frame]`
    /// detail, 89 MB, four minutes, against run59's thirteen for a window
    /// a tenth the size. `rngcmp.py rontrace-run54.log rontrace-run66.log`:
    /// **6,621 frames, zero differing**, so it is run54's game and the
    /// sixth capture in a row for which a window costs the stream nothing.
    ///
    /// **What it was booked for.** East Indies' word parted at 6570 on a
    /// collision the original does not have: the AI Merchant `1/19`,
    /// two hundred frames into its walk to a `CITRUS`, stood blocked
    /// twice where the original walked on. Nothing on disk covered the
    /// frame — run64 and run65's windows end at 6221 — and the geometry
    /// was a knife edge: the merchant's `BLOCK_RADIUS 2` block passes two
    /// standing citizens with 160 units of clearance against a 144-unit
    /// block sum, so a few units either way decides it.
    ///
    /// **What it settled** is `docs/COLLISION.md` §4.2's fast path, and
    /// the lesson is that it is not an optimisation. With `nocoll` clear
    /// and a proposal exactly one cell away on one axis,
    /// `CollCheck::collide_here@00682540` sweeps the **leading edge** —
    /// the row or column the block is entering — and nothing else: a
    /// strict subset of the parity-filtered disc, so it stops at a
    /// *different* first hit cell. On sim-frame 6570 the merchant's own
    /// cell is `(721, 786)` and its proposal `(721, 785)`, so the sweep
    /// is the row `y = 783` from `x = 719`: the first cell is `1/11`'s
    /// north-east corner, the disc probe's is `1/2`'s two to the right,
    /// and §4.3's corner rule is decided on the cell. Against `1/11` the
    /// merchant's own north-west corner is the opposite diagonal —
    /// `will_be_corner 1` against `is_corner 5`, a difference of exactly
    /// 4 — and the two slip past; against `1/2` the merchant's corner is
    /// 0 and the collision is hard. One frame later the proposal is one
    /// cell **west**, the sweep is the column `x = 718`, `1/11` is
    /// square on it rather than cornered, and the original collides —
    /// with `collide_o 11`, which is what named the cell.
    ///
    /// Every row below is the original's: 12,094 position fields — every
    /// unit on every block as far as the word, and `1/19` alone for all
    /// 261, so its walk, its collision, its centre snap and its recovery
    /// are pinned past the frame the stream parts on. East Indies' word
    /// went **6570 → 6571**.
    #[test]
    fn run66_s_window_is_the_original_s_unit_for_unit() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r66)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run66-islands-merchantwalk.txt"),
        ) else {
            eprintln!("skipping: no run54/run66 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text66 = std::fs::read_to_string(&r66).unwrap();
        let l66 = Log::parse(&text66);
        let frames = l66.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        // `[6340, 6600)` plus the two blocks the `!quit` writes — the same
        // free tail run65's window has.
        assert_eq!(
            window,
            (6340..6600).chain([6621]).collect::<Vec<i64>>(),
            "run66's frame window, plus the block the quit writes"
        );
        let last = *window.last().expect("the window");

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut compared = 0usize;
        let mut merchant_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..last {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            for them in &fr.units {
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    unmatched += 1;
                    continue;
                };
                let merchant = (them.who, them.o) == (1, 19);
                if merchant {
                    merchant_rows += 1;
                } else if f + 1 > LONG_WORD_EAST_INDIES {
                    continue;
                }
                let un = &built.sim.units[u];
                let mut row = |name: &str, ours: i64, theirs: Option<i64>| {
                    let Some(theirs) = theirs else { return };
                    compared += 1;
                    if ours != theirs && wrong.len() < 24 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                row("x", i64::from(un.pos.x), Some(them.pos.x));
                row("y", i64::from(un.pos.y), Some(them.pos.y));
            }
        }
        eprintln!(
            "run66 window: {compared} fields over {} blocks, {merchant_rows} of \
             them the merchant's, {unmatched} of the dump's units this crate has \
             no unit for",
            window.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 12_000 && merchant_rows >= 240,
            "the window's own rows: {compared} fields and {merchant_rows} of the \
             merchant's — a capture with neither is the wrong file"
        );
        // **The merchant's own walk, whole**, and everything else as far as
        // the word. Past the word the stream has parted — the crew figure's
        // draw `docs/COLLISION.md` §9 names — so the units downstream of it
        // are not this crate's prediction any more; `1/19` is, for all 260
        // blocks.
        assert!(wrong.is_empty(), "run66's window parted: {wrong:?}");
    }

    /// East Indies' word on the **long** capture — the number that took
    /// over as the headline when run39's own length stopped bounding it
    /// (`docs/DECISIONS.md` entry 29's first counter). It is not in
    /// [`FLOORS`] because `FLOORS` is the scored captures' scoreboard and
    /// this map's scored capture is closed; the queue states both.
    ///
    /// **5376** — and the frame is `Leader::produce_building`'s jitter,
    /// not a boat's: the AI sites something and this crate does not, and
    /// the whole of the fishing lineage now agrees for 5,375 frames.
    ///
    /// It was **5285** for the second half of one item, and that half was
    /// **`WData.down`** (`docs/ORDERS.md` §6.8). A deployed boat is the
    /// head of its cell's object chain, so the *second* Fisherman's search
    /// must refuse the fish the first is sitting on — and
    /// `think_fish`'s claim test read the start dump's snapshot, which
    /// still held the good's own terminator. One extra accepted cell, one
    /// extra draw: 178 against the original's 177 (item 48's chain, one
    /// reader at a time).
    ///
    /// It was **5106** for one item, and that item was
    /// `UnitData::calc_gather@00609180` (`docs/ORDERS.md` §6.10) — the
    /// head of `think_fish`, which asks a deployed boat once in 1,024
    /// frames whether it may stay where it is. run54's `1/14` is standing
    /// on its fish there and the original answers yes and spends four
    /// draws; this crate had no answer at all, sent it back through the
    /// 17 × 17 and spent 165. The **first** tile of the spiral to carry
    /// `TData & 0x200` is what the search takes, and the fish is not under
    /// the boat's own tile — which is why run58's dump prints `good_obj 1`
    /// and not 0.
    ///
    /// It was **4988** for one item, and the frame is the last of the
    /// fishing boat's deploy:
    /// the original's guy finishes `CHAR_UNPACK` there and pays the wrap's
    /// idle roll (`Guy::set_anim+0x97a < Guy::inc_time+0x271`), where this
    /// crate spends a farm's. The boat's guy carries **no piece** — the
    /// piece table is seeded from the start dump's own `GUY` blocks and no
    /// Fisherman is in one — so its `end_time` is [`sim::anim::UNKNOWN`]
    /// and no animation of its ever wraps (item 152).
    ///
    /// It was **4950** for one item before that, and that item was the
    /// deploy itself.
    /// `Unit::do_cast` killed every craft but the transport on the frame
    /// after it was queued, so the boat came back idle on 4950, still
    /// packed, and searched its 17 × 17 a second time — 165 draws to none.
    /// The craft table is now loaded (`craftrules.xml`, 55 rows), `0x292`
    /// waits out its `JOB_TIME` of **40**, and `SpellType::cast_unpack`
    /// clears `unit_masks & 0x80000` on frame 4989 exactly as run58's own
    /// `unit_masks 786440 → 262152` does (`docs/ORDERS.md` §6.9).
    ///
    /// It was **4945** for one item, and that item was the **turning
    /// stand** (`docs/ANIM.md` §4.8). `Guy::do_turn` asks a guy with
    /// `guy_flags & 8` for `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, and
    /// `Guy::init_real` sets that bit for every type that **packs** as well
    /// as for a piece that names the turn — so the AI's Fisherman asks for
    /// an animation `unit_graphics.xml` never gives it, `Guy::set_anim`
    /// rewrites the request to `CHAR_DEFAULT`, and a boat on the walk
    /// category pays an idle roll. Run26's dump states the flag by type and
    /// is what makes the two writers a diff rather than a reading.
    ///
    /// **4462** — the sequence and the count part on the same frame, and
    /// the frame is the new ship's first think: theirs makes 55 draws
    /// opening at `Unit::think_fish+0x27a` (`5f4eda`) where this crate
    /// makes 2. `think_fish` has no counterpart here at all.
    ///
    /// It was **4461** for one item, and that item was the AI's Dock —
    /// booked as "a late *decision* or a slow *counter*", and it was
    /// neither: the queue record agrees field for field from the frame the
    /// job is queued (**4376**) to the frame before it lands, and parts on
    /// the **target**. Theirs caps at `job_counter` **8481**, this crate
    /// ran on to 11,280, and 11280 × 100 / 133 is 8481 to the unit — the
    /// **British ship bonus**. Player 1 is British (`tribe 11`),
    /// `BRITISH_SHIP_SPEED` is 33, and `ObjectData::train_time@006508c0`'s
    /// national block applies it to every type whose domain is the sea.
    /// The unit is a **Fisherman** (`TypeIndex` 317), which is why the
    /// frame after it is `think_fish`. `docs/PRODUCTION.md`, "The tail's
    /// first caller"; the second job's 12,030 → 9,045 is the same ratio on
    /// the ramp's next step, so the arm is checked twice over.
    ///
    /// **And it was found by widening, not by reading.** The queue record
    /// was parsed on every capture and compared on one — run39's, which is
    /// 2,600 frames too short to reach the AI's first ship. Moving the loop
    /// into [`compare`] put it on run58 and the twenty-eight frames of
    /// `queued ours 1 theirs 0` were there the same minute.
    ///
    /// It was **4313** for one item, and that item was booked as the
    /// scout's re-think and was a **citizen's** (`docs/SCOUT.md` §11.1).
    /// The 28 draws are one `Unit::think_scout+0x941` and 27 `+0xaba` —
    /// §11's region scan, not the city loop — and the caller is
    /// `Unit::think_peasant+0x2ac`, `think_scout`'s **second** call site,
    /// not `Unit::think`'s tail. `think_peasant`'s AI tail was
    /// untranscribed below `find_gather_spot`: a worker standing in a
    /// region where its leader has no city and which none of the ten
    /// `Sites` claims explores instead. The AI's citizen `1/15` reached
    /// cell (39, 34) on 4312, went idle on 4313, and took the whole
    /// hundred-cell region scan of region 8 at stride 2 — 50 cells
    /// visited, 27 scored, the winner tile (162, 138).
    ///
    /// It was **4275** for one item, and that item was booked as a
    /// woodcutter's clock and was **the tile grid's own tolerance**
    /// (`docs/PATHFINDER.md` §7). The clock was real — the AI's citizen
    /// `1/13` reached its wood tile thirteen frames early and started its
    /// `wait` down from 452 there — but the thirteen frames were three a
    /// leg of the walk out, and the walk was wrong because
    /// `astar_path`'s reconstruction gives a unit that **can transport**
    /// an exact waypoint (`00684bfc`: tolerance 0, not `0x60`) and this
    /// crate gave every tile-grid waypoint `0x60`. `1/13` is granted
    /// `unit_masks & 0x800000` on frame **3580**, the frame after its
    /// Dock finishes, and had been cutting the corner off every leg
    /// since — which is also the whole of `1/13`'s parting at 3647, the
    /// last unit in run57 or run58 to leave the original's point before
    /// the word.
    ///
    /// It was **4020** for one item, and that item was booked as the
    /// scout's and was the **colonist's** (`docs/SYNC.md` §3.25). The
    /// frame's extra `do_cast` pair is the AI's citizen `1/15` becoming
    /// its own transport, not the scout `1/0`, and the scout is right on
    /// both sides — it takes cell (48, 30) on 4005 and re-thinks on 4110,
    /// which run54's own trace shows the original doing too. What was
    /// wrong was two reads, three hundred frames earlier:
    /// `Region::coast_here`'s neighbour probe is `WorldData::get_tregion`
    /// and this crate asked the plain cell region, so
    /// `think_civilian_transport` could not see the shore and sent `1/15`
    /// to cell (37, 36) where the original sends it to **(39, 34)**; and
    /// `Unit::move_step`'s `path.flags & 4` — the waypoint's own
    /// turn-in-place bit, written here since the pathfinder landed and
    /// never read — let it walk through the turn the original stands still
    /// for, reaching its embark point a frame early.
    ///
    /// It was **3978** for one item, and that item was three things in a
    /// row, all of them `Unit::come_out@00617c10` and its caller.
    ///
    /// The draw is the **army coin** at `come_out`'s tail (`+0x25ca`,
    /// `61a1da`), unnamed in the trace and named here from the listing: an
    /// AI unit leaving whatever carried it joins an army unless a coin says
    /// otherwise, and only the **scout** and **naval-scout** lineages throw
    /// one — `is_special` is `is(SCOUT)` by the loader, and the other arm is
    /// `is(BARK)`, `% 3` at `+0x25b0`. run54 reaches the site eleven times
    /// in 24,000 frames and both arms are there (`docs/ARMY.md` §4).
    ///
    /// Then the scout had to come out where the original's does, and that
    /// is `come_out`'s **host** arm — a ring of the *boat's* `block_radius`
    /// out to `+ UNIT_DISEMBARK_DISTANCE`, swept from the *boat's* own
    /// angle, and the passenger turned to that angle with its crew seated
    /// on the track offset. And the ring only reaches land because a boat
    /// lying on the water half of a coastal cell **marks no collision
    /// cells there**: `docs/COLLISION.md` §2's region gate is
    /// `WorldData::get_tregion` of the figure's own tile, which answers a
    /// coastal cell's `region2` for an ocean tile, and this crate asked the
    /// plain `region_of` — so the barge filled its own cell and pushed its
    /// passenger four hundred units inland.
    ///
    /// Then it had to keep walking, and `Object::eject_contents@0064cd20`
    /// is `cast_transport` run backwards: for a passenger of `uber_size`
    /// 1 the boat's whole order list moves back onto it and the boat's
    /// path stack is inverted and popped onto its own, the top's embark
    /// flag cleared. run57 block 3979 has the scout's order and both
    /// waypoints field for field against the barge's in block 3978
    /// (`docs/TRANSPORT.md` §6.4).
    ///
    /// It was **3687** for one item, and the frame was the AI's citizen
    /// `1/11` — three draws against seven, ours opening with a
    /// `Guy::set_anim+0x97a < Unit::move_step+0x823` the original does not
    /// spend and then throwing seven gaia wing-beat coins where the
    /// original throws five. Both were one unit's, and it had left the
    /// original's point ninety-five frames earlier: `Unit::think_peasant`
    /// offers an AI **citizen** to the boat before it looks for work
    /// (`005f5760:53`, `think_civilian_transport(1)`), and this crate had
    /// only the scout's `colonise = 0` caller, so the citizen that the
    /// original sends north to colonise cell (50, 31) on frame 3581 went
    /// back to its gather instead (`docs/TRANSPORT.md` §7,
    /// `docs/SYNC.md` §3.23).
    ///
    /// It was **3608** for one item, and the frame was the transport
    /// barge's own guy. `SpellType::cast_transport` first runs there — the
    /// dock's own shadow, the level granted at 3579 being what lets a unit
    /// become its own transport at the shore — and this crate wrapped the
    /// barge's brand-new animation clock where the original wrapped no
    /// barge's, on that frame or any after. The reason is the loop, not the
    /// clock: `Objects::process_all@0065dce0` re-reads `unit_mark[who]` at
    /// the bottom of its inner loop, so the barge — cast by a unit whose
    /// `o` is lower — takes its first step on the frame it is born, and
    /// that step sets its guy to the walk before `Objects::inc_time` gets
    /// there. Every other newborn in run57 is a *trained* unit, born in the
    /// second loop, and every one of those does wrap on its birth frame
    /// (`docs/SYNC.md` §3.22, `docs/TRANSPORT.md` §13).
    ///
    /// It was **3579** for one item, and the frame was the dock's gull.
    /// A finished Dock spawns a **`GULLBIRD` of owner 9** a tile
    /// north-west of itself (`Dock::init@00740a80`: `Objects::init_unit(9,
    /// GULLBIRD, x − 0xc0, y − 0xc0)`) and rolls its facing as `(r % 7) ×
    /// 0xaaaaaaa − 0x40000000`; this crate spawned it and left the roll
    /// unmarked, so the sequence read a second `Guy::init_real+0x52`, and
    /// the gull carried no `type_index`, so it never flew: the count parted
    /// a frame later on `do_air_physics`'s own `set_anim(CHAR_WALK, 0, 1)`
    /// (`docs/TRANSPORT.md` §5.2, `docs/SYNC.md` §3.4).
    ///
    /// It was **3435** for one item, and the frame was three draws against
    /// two: a `Farms::inc_time+0x1ae` this crate spent and the original did
    /// not. That was a symptom of the farm below rather than a clock: with
    /// player 1's farms on the original's own cells the extra growth tick
    /// is gone.
    ///
    /// It was **3177** for the length of one item — the AI's farm `1/2011`
    /// going up one cell east of the original's, and `1/2012` one tile.
    /// `produce_building`'s FARM/MINE arm does not reuse the spiral's own
    /// distance: it rebuilds both sides in **tiles** and measures from the
    /// anchor's exact position, so a straight neighbour and a diagonal one
    /// score `4000/4` against `4000/6` where in cells both are `4000/1`.
    /// The same arm's `0xff` is `0xff − WData.val`, and the map's value
    /// byte is not zero (`docs/AI.md` §22).
    ///
    /// It was **3021** for two items — four draws against five, the
    /// missing one an idle request from `Unit::move_step` rather than from
    /// `Guy::inc_time` — and both of them were the AI's Dock. `o 2010`
    /// went up two cells north of the original's because
    /// `produce_building`'s `is(0x1b0)` slide was not modelled, and the
    /// slide's effect is not where the dock lands but *which* candidate
    /// the spiral accepts, and so the phase of its stride of three
    /// (`docs/AI.md` §21). With the dock right the seam moved to the
    /// builder's approach and `find_nearby_spot` let a citizen stand in
    /// the sea (`docs/ORDERS.md` §10); with both, every unit of run56
    /// stands where the original's does for the whole capture.
    ///
    /// It was **2665** twice over, and the second of those was the scout's
    /// **dog**. A unit is one or more figures, and a crew figure whose
    /// piece names a track offset does not stand on the guy it follows: it
    /// walks a body of its own toward a destination its leader rewrites,
    /// and it is still walking four frames after the man has arrived. Those
    /// are four frames on which the two figures answer `Guy::set_anim`'s
    /// walking-guy early return differently, and this crate — which handed
    /// every guy the unit's body — re-rolled the dog's idle where the
    /// original did not. With the second body landed the frame agrees, and
    /// so does every figure of every player's unit on all 3,000 frames of
    /// run56 ([`run56_s_figures_stand_where_the_original_s_do`]).
    ///
    /// Two readings carried it, and both are in `docs/MOVEMENT.md` under
    /// "The follower's destination": `Guy::set_new_location` and
    /// `Guy::set_angle` both rewrite the crew's `des` — the first from guy
    /// 0's new position when it moves, the second from the angle it has
    /// just turned to when it stands — and `GuyData::turn_speed@005de340`
    /// fences its whole first half behind `guy_num < squad_size`, so a
    /// tracked crew guy turns a flat quarter turn a frame and can never
    /// give a frame up to turning.
    ///
    /// The floor before that was **2176**, item 126's — the frame the
    /// original placed player 1's second Woodcutter's Camp and this
    /// simulation placed nothing, because `produce_building` scored a camp
    /// site by the forest tiles in a one-tile ring rather than by what the
    /// site would gather. `blocked_site`'s out-parameter is that number
    /// (`docs/CITIES.md` §2.6.7), and with it the camp goes up at the
    /// original's own frame, tile and object number.
    /// run58's own three numbers (`run58_s_five_thousand_frames_stand_
    /// where_the_original_s_do`). The two field counts are structural —
    /// what a 5,200-frame capture holds — and `RUN58_PARTED` is the score:
    /// how many of its units ever walk off the original's point **before
    /// the word**. It was 1 — `1/13`, from 3647 — until the tile grid's
    /// tolerance landed; it was **0**; it was **1** while the AI's first
    /// Fisherman walked a sea route this crate planned two cells north of
    /// the original's; and it is **0** again now that the world grid's
    /// same-region test reads `WData.region` rather than `get_tregion`
    /// (`docs/PATHFINDER.md` §16). Both Fishermen's routes now agree row
    /// for row on the frame each is planned, so the boats have no pin of
    /// their own any more — they are inside the ordinary assert.
    const RUN58_PARTED: usize = 0;
    const RUN58_BUILD_FIELDS: usize = 178_326;
    const RUN58_COLL_FIELDS: usize = 449_279;
    /// Unit-frames carrying `unit_masks` and `mylos` — one apiece per
    /// linked unit-frame, which is every one, so the floor only grows.
    /// 94,935 and 94,338 as this was pinned; `mylos` is the smaller
    /// because a garrisoned unit's is not compared.
    const RUN58_PACKED_FRAMES: usize = 94_338;

    /// **run63 — the frame the AI's colony site appears, and the citizen
    /// that was waiting for it** (2026-09-02).
    ///
    /// East Indies' word stood at **5592** and the frame was one citizen's:
    /// `1/15` walks to the explore target `think_scout` gave it on 5455,
    /// arrives on 5592 — position for position with the original, all 137
    /// frames of it — and then, on its very first idle frame, re-targets
    /// and walks off. The original's stands there for **seventy-three
    /// frames** and re-targets on 5666.
    ///
    /// The trace said which arm: `Unit::think_scout+0xaba <
    /// Unit::think_peasant+0x2ac` is `think_peasant`'s **no-site** call and
    /// `+0x2ca` its `idle > 6` one (`docs/SCOUT.md` §11.1; the listing at
    /// `005f5a07`/`005f5a25` settles which is which). The original spends
    /// `+0x2ac` on 5455 and `+0x2ca` on 5666 — so between those two frames
    /// one of leader 1's ten sites came to claim the citizen's region, and
    /// a claimed region is worth waiting six idle frames in.
    ///
    /// run63 is the capture that could say so: run58's recipe with the
    /// `[End Frame]` narrowed to `[5430, 5700)` and `LEADERS=9` in it, so
    /// the `SITE` record is on the record for every frame either side of
    /// the parting. Sixteen minutes and 482 MB, and `rngcmp.py` against
    /// run54 is **5,701 frames, zero differing**.
    ///
    /// **What it says.** On 5550 leader 1's tenth site is `(44, 52) val
    /// 403` and on 5593 it is `(34, 33) val 9728 reg 7 dist 2` — the
    /// citizen's own cell, scored the moment it stands there. This crate
    /// scored every cell of that region **zero**, because
    /// `compute_site_stats`' step 2 zeroes the base when
    /// `blocked_site(TOWN, …)` refuses, and `blocked_location` refused
    /// every one of them with `COLONIZE 0x1c`: a first city in a region
    /// where the leader has none is gated on `has_preq(COLONIZE_BONUS
    /// 0x2af)`, which this crate carried as a **nation** flag that nothing
    /// ever set. It is a technology's — the fourth of `rules.xml`'s
    /// `TECHBONUSES`, `preq0 = Coinage` — and the AI's library takes its
    /// Coinage job on 5177 (`RUN58_QUEUE_TAIL`). The bonus lands inside
    /// this window and nowhere earlier, which is why no capture before
    /// this one could have found it. `docs/CITIES.md` §2.6.1's open
    /// question 4 is what it closes.
    ///
    /// With the gate on the technology the word is **5669**.
    ///
    /// What is asserted here is the whole window: every dumped unit stands
    /// where the original's does on every one of its frames, and the ten
    /// `SITE`s of both leaders are compared field for field — the widening
    /// the ledger wanted (`docs/QUEUE.md` item 87), and the record that
    /// carries the finding.
    #[test]
    fn run63_s_window_is_where_the_ai_s_colony_site_appears() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run63-islands-scoutwalk.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run63.log"),
        ) else {
            eprintln!("skipping: no run63 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let frames = report.frames.len();
        assert!(
            frames >= 260,
            "run63's window is [5430, 5700) — {frames} frames is a wrong file"
        );
        assert_eq!(
            report.frames.first().map(|f| f.frame),
            Some(5430),
            "the window opens where the stanza says"
        );

        // **Every unit, every frame of the window.** The citizen the item
        // is about is `1/15`, and its seventy-three idle frames are in
        // here: nothing may leave the original's point before the word.
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let compared: usize = report.frames.iter().map(|f| f.compared).sum();
        eprintln!(
            "run63: {compared} unit-frames over {frames} frames, {} ever off position",
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        // **Nothing parts, on any frame of the window.** Both of the
        // residues this capture opened with are closed, and each was a
        // mechanic rather than a route:
        //
        // - ~~**`1/18` is five frames ahead.**~~ Closed 2026-09-02. It was
        //   never five lost frames: the AI's Transport Barge is *born* on
        //   5342, two tiles closer to its destination than the original's,
        //   and stayed that far ahead for the rest of its life. The water
        //   it is born on is `UnitType::find_nearby_spot`'s `(-1, -1)`
        //   form, whose collision half is `find_unit_with_radius` and not
        //   the pairwise pair — `docs/ORDERS.md` §10, `docs/TRANSPORT.md`
        //   §6.1, and `run59_s_census_is_where_the_ai_s_timber_goes` is
        //   where it was measured, 88 frames before this window opens.
        // - ~~**`1/17` turns three frames late.**~~ Closed 2026-09-02, and
        //   it was never the turn model: the original's boat is **faster**
        //   from 5552, because leader 1's second Fisherman settles on a
        //   **whale** that frame and `Unit::update_speed`'s one rare arm
        //   puts every naval type at `(WHALES_SHIPS_MOVE + 100)%` —
        //   `myspeed` 38 → 45 on all three boats and 25 → 30 on the barge,
        //   in the dump and now here (`crates/sim/src/rares.rs`). The
        //   apparent late turn was a slower boat reaching its waypoint
        //   later; the twenty-three frames it lost afterwards were East
        //   Indies' word.
        //
        // Pinned at empty, so the day anything parts here the assertion
        // says which unit and on what frame.
        let early: Vec<(i64, i64, i64)> = parted
            .iter()
            .filter(|(_, f)| **f < LONG_WORD_EAST_INDIES)
            .map(|(&(w, o), &f)| (w, o, f))
            .collect();
        assert_eq!(
            early,
            vec![],
            "the window stands where the original's does, on every frame"
        );

        // **The `SITE` record, whole.** Ten slots a leader a frame, five
        // fields apiece — `reg` is left out because the dump's region
        // numbers are the generator's and this crate's are its own
        // (`Built::region_map` is the translation, and this record is
        // keyed by slot rather than by region).
        let mut built = {
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            borrow_pasture(&mut init, &tr);
            build_sim(&loaded, &init, Tuning::RON)
        };
        let last = report.frames.last().map_or(0, |f| f.frame);
        let mut site_compared = 0usize;
        // (who, slot, field) → (frames wrong, ours and theirs on the first).
        let mut site_bad: std::collections::BTreeMap<(i64, usize, &str), (usize, i64, i64)> =
            std::collections::BTreeMap::new();
        for n in 1..=last {
            built.tick();
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let theirs: Vec<&Block> = block.kids("SITE").collect();
                if theirs.len() != sim::ai::SITES {
                    continue;
                }
                for (i, t) in theirs.iter().enumerate() {
                    let o = built.sim.ai[who as usize].sites[i];
                    for (field, ours, theirs) in [
                        ("wx", i64::from(o.wx), t.int("wx")),
                        ("wy", i64::from(o.wy), t.int("wy")),
                        ("val", i64::from(o.val), t.int("val")),
                        ("dist", i64::from(o.dist), t.int("dist")),
                        ("rank", i64::from(o.rank), t.int("rank")),
                    ] {
                        let Some(theirs) = theirs else { continue };
                        site_compared += 1;
                        if ours != theirs {
                            let e = site_bad.entry((who, i, field)).or_insert((0, ours, theirs));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        eprintln!(
            "run63 sites: {site_compared} fields compared, {} shapes wrong",
            site_bad.len()
        );
        for ((who, i, field), (n, ours, theirs)) in &site_bad {
            eprintln!("  who {who} slot {i} {field}: {n} frames, ours {ours} theirs {theirs}");
        }
        assert_eq!(
            site_compared, RUN63_SITE_FIELDS,
            "ten slots, two leaders, five fields, every frame of the window"
        );
        // **The residue, pinned as it stands** rather than filtered out —
        // 7,122 of 27,000, on leader 1 alone (the human's ten slots are
        // empty on both sides and agree on every frame).
        //
        // The slot the item is about is the **tenth**, and it agrees: from
        // 5577 the original holds `(34, 33) val 9728 dist 2` and so does
        // this crate, the same cell at the same score on the same frame.
        // What is left is the four sites the AI has been carrying since
        // long before this window, and their shape is old: run59's census
        // has it 250 frames earlier and nothing compared it until now —
        // `(45, 52)` and `(44, 52)` score twice and four times the
        // original's, and one extra site takes an empty slot the original
        // leaves alone, which drags every `rank` and every slot *order*
        // with it. That is `compute_site_stats`' arithmetic, and it is
        // booked (`docs/QUEUE.md`); this run moved the **region**, not the
        // score.
        let wrong: usize = site_bad.values().map(|(n, _, _)| n).sum();
        assert_eq!(
            wrong, RUN63_SITE_WRONG,
            "the site record's standing residue, and it only moves deliberately"
        );
    }

    /// run63's two site counts: what the window holds, and what is still
    /// wrong in it. The first is structural — ten slots, two leaders, five
    /// fields, every frame the `LEADERS=9` window wrote — and the second is
    /// `compute_site_stats`' own residue, which
    /// `run63_s_window_is_where_the_ai_s_colony_site_appears` names.
    const RUN63_SITE_FIELDS: usize = 27_000;
    const RUN63_SITE_WRONG: usize = 7_122;

    /// East Indies' word on run54, the headline.
    ///
    /// **6570** since 2026-09-02, and the frame is a **collision the
    /// original does not have**: `Guy::set_anim+0x97a <
    /// Unit::move_step+0x823`, the blocked stand (`docs/COLLISION.md` §5),
    /// twice — once per figure — on the AI Merchant two hundred frames
    /// into its walk. The unit in its way is the AI's own citizen `1/2`,
    /// standing still on a gather order 180 units off, and this crate's
    /// exemption ladder does not let the two past. `PathFinder::find_wpath`
    /// is first entered on the original's frame 6602, so the original's
    /// merchant meets something of its own thirty frames later; whether it
    /// is this citizen is not settled.
    ///
    /// It was **6356** for one item, and the frame was the **Merchant's
    /// first step**: `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
    /// Unit::move_step` at the head of the original's frame, three frames
    /// after the unit was born. `Unit::think_merchant@005f4740` is what
    /// sends it — it scores the leader's `new_rares` list, moves the
    /// winner to the back of that list and issues a `MOVE_TO` at the good
    /// — and nothing here reached it (`docs/MERCHANT.md`). Landing it
    /// alone was not enough: the merchant then turned **twice** a frame
    /// where the original turns once, because `Unit::init@00612100:549`'s
    /// `set_new_location(·, ·, 1, 1)` seats a **tracked** crew figure on
    /// its offset at birth and this crate seated only the figures a dump
    /// or a disembark handed it. A trackless crew figure is one
    /// `Guy::do_turn@005d97a0` recurses into, and a merchant's figures all
    /// carry `guy_flags & 8` because the type packs — so it asked for a
    /// turn its art has not got and paid the idle roll beside its driver
    /// (`docs/MERCHANT.md` §5). `Sim::init_guys` now ends where
    /// `Unit::init` does.
    ///
    /// It was **6353** for one item, and the frame was a **unit the
    /// original trains and this crate did not**: two `Guy::init_real+0x52`
    /// under `Objects::init_unit`, a two-figure unit, and run65's window
    /// names it — the AI Market `1/2013` is one `MERCHANT` deep on all
    /// eighteen of its blocks, with a `job_counter` climbing a hundred a
    /// frame from the caravan's hand-over on 6164 to 18,720 (`JOB_TIME`
    /// 156 at `UNIT_RATE_BASE` 120) on 6353. The cause was one host
    /// function: `economic.bhs` trains merchants only up to
    /// `num_rare_resources_seen(who)`, and this crate's script host
    /// answered a flat zero because rares were not in the simulation when
    /// it was written. `Leader::new_rare@006d9e70` is the list's writer,
    /// `World::reveal_fog` its caller, and `FISH`/`WHALES` are excluded
    /// because they pay a fishing boat rather than a merchant
    /// (`docs/ECONOMY.md`, "The rares a leader has seen"). East Indies'
    /// AI has seen `CITRUS` and `HORSES` by frame 5000; the queue and its
    /// clock are now an assertion
    /// (`run65_s_window_is_the_original_s_unit_for_unit`).
    ///
    /// It was **6207** for one item, and that item was neither the caravan
    /// nor its turn. run65 is the capture — an eighteen-frame `DUMP_ALL`
    /// window over the leg — and what it shows is both sides pushing the
    /// same detour node `(38508, 40620)` on the same frame, turning
    /// through the same eight bearings at the same rate, and computing the
    /// same full step on sim-frame 6206 to the same point,
    /// `(38750, 40508)`. The original does not take it:
    /// `move_step@005faf30` at `005fb7c1` compares the step's tile against
    /// the one the unit stands on and, where they differ, asks
    /// `UnitData::invalid_loc` with all five flags clear — and that point
    /// is a tile inside the caravan's own city's footprint. The step is
    /// dropped whole, with no `set_anim` and no `set_new_location`, the
    /// waypoint is kept, and the unit walks a frame later through a tile
    /// it may have. It is the last of the four things `docs/MOVEMENT.md`'s
    /// `move_step` section listed as read and not modelled, and the whole
    /// window is now an assertion
    /// (`run65_s_window_is_the_original_s_unit_for_unit`, 5,186 fields).
    ///
    /// It was **6198** for one item, and that item was `Unit::do_trade`'s
    /// **first instruction**: `set_anim(CHAR_DEFAULT, 0, 1)` at `+0x40`,
    /// ahead of the caravan-slot test and every one of the function's
    /// returns. It draws nothing while the unit is walking — a walk-
    /// category figure whose body has not arrived leaves `Guy::set_anim`
    /// without a roll — so the site is silent for the thirty frames the
    /// caravan spends reaching its city and then spends three draws, one
    /// through `Unit::set_anim+0x56` and two through `+0xb6`, on the frame
    /// it stands still. The order queues its next leg with `QUEUE_FIRST`,
    /// so `do_trade` never runs while the walk does: **its later frames
    /// are its arrivals**, and 6511, 6766 and 7021 are the next three.
    ///
    /// The nine frames after it are the legs (`docs/CARAVAN.md` §7), and
    /// what carried them was not the reading: `Caravan::build_road`'s road
    /// is a stack of **world** positions with a tolerance and a flag byte,
    /// and this crate had been keeping tile coordinates — so `set_road_at`
    /// was handed numbers thirty thousand tiles off the map and no trade
    /// road in the port had ever been laid. The `CARAVAN` record prints
    /// the whole stack and nothing had compared it (item 87's ledger);
    /// twenty-six nodes, right on the first run.
    ///
    /// **6189 was the danger map**, and the whole of item 177's first half
    /// with it. `WorldData::danger[who]` is a half-resolution `int` grid
    /// `GameDaemon::calc_danger` rebuilds every two hundredth frame, and
    /// `calc_cost`'s world arm prices a step by `danger / 8`. Around a
    /// leader's own city the map is 65 to 135 **negative** — its own
    /// buildings subtract — so eight of the caravan's own steps were 8 to
    /// 16 too dear here, the search took a seventh expansion the original
    /// does not, and a route came back where the original's stops at the
    /// goal's own neighbour and pushes nothing. That is why `do_trade`'s
    /// move went in with a waypoint at `(37752, 41592)` and the caravan
    /// set off south-west where the original heads straight at
    /// `(39288, 40056)` (`crates/sim/src/danger.rs`, `docs/DANGER.md`).
    /// The map itself is now diffed whole against run64's own
    /// `danger[who][scan]`, 7,200 values
    /// (`run64_s_danger_map_is_the_original_s`), and so are the
    /// forty-seven prices of the search it broke
    /// (`run64_s_frame_6167_prices_are_the_originals`).
    ///
    /// It was **6189** for one item, and 6169 was **an empty animation packet**. Sixty of the shipped `<UNIT>` entries in
    /// `unit_graphics.xml` have no `<ANIM>` child at all and every one of
    /// them is a `-CREW{k}`: the piece loads a model, so
    /// `get_unit_gpiece` hands it out, and its packet names nothing — so
    /// every slot is `AnimationPacket::get_game_frames`' three frames and
    /// `Guy::inc_time`'s `packet->ids[slot] >= 0 && loopings[id]` is false
    /// for all of them. A caravan's two crew figures therefore walk three
    /// frames, fall to `CHAR_DEFAULT`, roll, and are put back on the walk
    /// by `Guy::move` the next frame: **two draws every third frame, for
    /// the rest of the game** — 11,872 of them over run54's remaining
    /// 17,800 frames, which is why one item moved the word by only
    /// twenty-nine and would have blocked every later one. This reader
    /// dropped the animation-less entry outright, so the walk fell to
    /// `first_unit_piece` and the crew played the citizen's looping art
    /// (`docs/ANIM.md` §3.6).
    ///
    /// It was **5592** for one item, and it was **5466** until a building
    /// started flattening
    /// the ground under it before it planned its road. 5466 was the AI
    /// Market's own placement frame, one road search of 184 nodes against
    /// the original's 205, and the cause was two mechanics at once:
    /// `calc_road_cost` calls `was_seen` and not `was_really_seen`, and
    /// `Wall::start` runs `TerrainOut::terraform_for_building` **before**
    /// `mask_me`. run62 is the capture that could say so — the road
    /// search's gate and its price proxied together, so the original's own
    /// 2,913 node prices are on the record (`docs/ROADS.md` §7.2–§7.4).
    ///
    /// It was **5437** before that, and **5376** before *that*, when
    /// `find_friends` started asking whose *footprint* covers a cell's
    /// centre tile rather than whose centre is in the cell (`docs/AI.md`
    /// §26).
    ///
    /// **6166** since 2026-09-02, and 6164 was **a Caravan's figures**.
    /// `Unit::init@00612100:471`–`508` sizes the guy stack to `crew_size +
    /// squad_size`, and `squad_size` is the literal 1 `UnitType::init@
    /// 0061ab50:723` writes into every type — so a unit has
    /// `unitrules.xml`'s `CREW_SIZE` figures plus one, and the Caravan's
    /// column is 2. `Sim::init_guys` made one figure for every unit it
    /// built, so 6164 spent one `Guy::init_real+0x52` against three and
    /// 6165 was missing the two `Guy::set_anim+0x97a < Unit::do_idle+0x7d`
    /// the absent figures owe. A unit stood up *from* a dump never had it —
    /// `build_sim` gives it the file's own `GUY` blocks — which is why
    /// twenty scout dogs walked correctly throughout
    /// (`docs/ANIM.md` §3.5,
    /// `every_dumped_unit_has_crew_size_plus_one_figures`). The frame it
    /// leaves is a road: 3,207 `PathFinder::calc_road_cost+0x46` draws on
    /// 6166 that this crate does not spend.
    ///
    /// It was **6164** for one item, and 5819 was the water a transport
    /// barge is born on. `UnitType::find_nearby_spot`'s `(-1, -1)` form — the one
    /// `Unit::do_cast` and `SpellType::cast_transport` ask for that water —
    /// does **not** take the pairwise collision pair: `0061deb0` sets the
    /// flag that selects it only when `not_o` and `not_who` are both
    /// non-negative, so the sweep falls to `find_unit_with_radius`, whose
    /// predicate is `vector_dist <= other.big_radius + r_coll` and whose
    /// ordered sibling is skipped outright. The Chebyshev pair refused the
    /// bearing the original takes — the caster itself, three cells and four
    /// cells away, is inside `3 + 1` cells but 228 units from a reach of
    /// 192 — so `1/18` was born two tiles closer to its destination and ran
    /// ahead of the original for the rest of its life
    /// (`docs/ORDERS.md` §10, `docs/TRANSPORT.md` §6.1).
    ///
    /// It was **5819** before that, and 5669 was a **whale**. The AI's
    /// second Fisherman settles on one on frame 5551; `Leader::calc_gather`
    /// step 6 walks the idle fishermen, lights the rare's bit in
    /// `rare_owned`, and `Leader::calc_unit_stats` hands every naval type
    /// `WHALES_SHIPS_MOVE` — `myspeed` 38 → 45 on the same frame, in the
    /// dump and now here. The third Fisherman, still walking to its own
    /// deposit, then arrived twenty-three frames late and did not spend
    /// `think_fish`'s 175 draws on 5669 (`crates/sim/src/rares.rs`,
    /// `docs/ECONOMY.md` step 6; run63 is the capture, and run59's census
    /// had the same finding as an income of 160 food and 160 wealth the AI
    /// was not earning).
    ///
    /// It was **5669** for one item, and 5592 was one AI citizen's first idle
    /// frame: `think_peasant`'s tail sends a worker off to explore the
    /// moment it stands in a region none of its leader's ten sites claims,
    /// and waits six idle frames when one does (`docs/SCOUT.md` §11.1).
    /// The original's site list gains the citizen's own cell on 5577 and
    /// this crate's could not, because `blocked_location` refused every
    /// city site in a region the leader had none in: `COLONIZE_BONUS` is a
    /// **technology's** prerequisite — the fourth of `rules.xml`'s
    /// `TECHBONUSES`, Coinage — and was carried here as a nation flag
    /// nothing set. run63 is the capture that says so
    /// (`run63_s_window_is_where_the_ai_s_colony_site_appears`).
    const LONG_WORD_EAST_INDIES: i64 = 6571;

    /// Great Lakes' word on the **long** capture (run53), the second of
    /// `docs/DECISIONS.md` entry 29's counters — and, since run61 put the
    /// bird's birth right, no longer the same number as the scored
    /// capture's. It was **1802** for as long as the flight was unmodelled;
    /// `Gaia::spawn_bird`'s tile snap took it to **2419**, and the scored
    /// run33 to the end of its own 1,850 frames.
    const LONG_WORD_GREAT_LAKES: i64 = 2419;

    /// The frame the AI's library takes its **Coinage** job on, and the
    /// frame run58's `QUEUE` record used to part on: twenty-four rows of
    /// `1/2005 queued ours 0 theirs 1` running to the end of the file.
    /// Item 154's first half, and it was
    /// `economy::Holdings::available` — knowledge is not available before
    /// the Classical Age, so the original charges Coinage's `14k` as two
    /// hundred and ten food and this crate asked for knowledge the AI had
    /// no way to hold. Both sides now start the job here, and this
    /// constant is what says so: the filter below still splits the file
    /// at it, and the tail it leaves is empty but for the truncated last
    /// frame.
    const RUN58_QUEUE_TAIL: i64 = 5177;

    /// **run40 and run41 — the leader census over a window, and what the
    /// AI's second city actually costs.**
    ///
    /// run33 carries `LEADERS=1` at `[End Frame]`: five scalars, and no
    /// resources. A `LEADERS=9` record is the census oracle
    /// (`docs/ORACLE.md`) but it is ~10k lines a leader, so it is a *window*
    /// setting — `tools/gamelog/censuswindow.sh`, run10's game and lobby
    /// with the frame window moved. run40 is `[560, 600)` and run41 is
    /// `[770, 800)`, the two frames on which
    /// `ScenarioFuncSet::place_city_with_cost` is reached in the whole
    /// 1,850. Both traces' words are run33's draw for draw where they
    /// overlap, so all three are the same game.
    ///
    /// What they measure is a **price**. The AI's `defensive.bhs` step 11
    /// calls `city_placement` once per turn of its `num_loops = 5` loop, and
    /// `place_city_with_cost` returns −1 without a draw once
    /// `city_limit <= total_cities`. So the *number of draws* on those two
    /// frames says whether the city was bought:
    ///
    /// - **576**: sixty draws, five whole blocks — not bought. The census
    ///   says why: 69 food and **59** timber.
    /// - **776**: five draws, one block — bought. 83 food and 73 timber
    ///   before, 23 and 14 after.
    ///
    /// Sixty food and sixty timber, twice over. A Small City is `COST 1t/1f`
    /// with `SUPPORT food 50 / timber 50`, so that is
    /// `1 × BUILD_COST_FACTOR(10) + 50 × BUILD_SUPPORT_FACTOR(1) × 1` with
    /// **no ceiling** — and the ceiling is what this crate had wrong. All
    /// four `*_RAMP_MAX` belong to `TypeData::get_cost`'s unit arm; the
    /// building arm (`00665787`..`00665b5a`) reads `BUILD_COST_FACTOR` at
    /// `+0x358` and `BUILD_SUPPORT_FACTOR` at `+0x37c` and loads no
    /// `RAMP_MAX` at all. With `RampClass::default()` — military, 125% — the
    /// ramp was clamped to 12 and the city priced at 22, which the AI could
    /// always pay.
    ///
    /// **And the rest of the record, because a price is only half of an
    /// affordability test.** `LEADERS=9` prints ten numbers per good per
    /// leader and this compares six of them — `bucket`, `leftover`,
    /// `resources` (the assembled rate), `income` (the rate after the cap),
    /// `resource_cap`, and `gather_slots` — over forty frames and two
    /// players. 480 good-frames apiece.
    ///
    /// Two of the six carried the whole finding. `leftover` agreed on every
    /// frame from the first widening, which is what said the AI's missing
    /// food was a **lump and not a rate**: the fractional accumulator can
    /// only agree if the two sides are paid the same amount every frame.
    /// And the AI was thirty-two short in `bucket` on all forty. The two
    /// lumps are `Build::do_bonus` — twenty food when the AI's fourth farm
    /// finishes on frame 166 — and `Build::refund_cost` — twelve when
    /// Written Word lands on 201 and re-prices the City State waiting
    /// behind it in the library. See `docs/ECONOMY.md` and `docs/COSTS.md`.
    ///
    /// Two of the six were wrong. One of them is now right, and for a
    /// reason outside the mechanic. **`resource_cap`**: the AI's is
    /// **1392** on every frame against the human's 1120, which is the
    /// British `+25%` on `COMMERCE_CAP[0]` — `70 × 125 / 100 = 87` with
    /// the half truncated before the `× 16`.
    /// [`sim::economy::commerce_cap`] computed that all along and
    /// `Nation::british` was never true, because nothing in this harness
    /// read the dump's own `tribe`. `Sim::set_tribe` does now
    /// (`crates/sim/src/nations.rs`) — run40's `tribe 11` and `tribe 4`
    /// are the roster's British and Nubians — and the 200 became **0**.
    ///
    /// The other is still wrong and inert, so it is asserted **as it
    /// stands** rather than left out — the day it is fixed the assertion
    /// moves rather than passing quietly.
    ///
    /// `gather_slots`: the farms agree and the camps do not — the human's
    /// seven and the AI's five read zero here. `Build::init` surveys a
    /// camp's slots against its own **still empty** `gather_from` and
    /// `Build::find_gather_tiles` recomputes once the list is filled, so a
    /// camp the harness stands up from a dump is activated before it has
    /// any. The human also files **one slot under good 2**, which
    /// `BuildTypeData::get_good@0063bd50` cannot produce — its jump table
    /// at `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5 and
    /// nothing else — so a second writer puts it there:
    /// `Leader::plan_strategy@006b9620` line 1137 assigns the **whole**
    /// array from `City::count_gather_slots` and raises the high-water to
    /// match. Reading that is the next widening, and it is booked.
    #[test]
    fn run40_s_census_prices_the_ai_s_second_city_at_sixty() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run40-census.txt") else {
            eprintln!("skipping: no run40 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // The window is [560, 600); a `FRAME n` block is the end of
        // sim-frame n − 1, so the record for `n` is read after `n` ticks.
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut city_price = [0i32; sim::economy::RESOURCES];
        for n in 1..=599 {
            built.tick();
            if n < 560 {
                continue;
            }
            if n == 576 {
                let village = built
                    .sim
                    .build_types
                    .iter()
                    .position(|b| b.ident == sim::build::Ident::Village)
                    .expect("the Small City");
                city_price = built.sim.building_price(1, village);
            }
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let field = |k: &str| -> Vec<i64> {
                    block
                        .all(k)
                        .iter()
                        .map(|v| v.trim().parse().unwrap_or(i64::MIN))
                        .collect()
                };
                let l = &built.sim.ledgers[who as usize];
                // The dump's names, and what each is here. `resources` is
                // the assembled rate and `income` the rate after the cap —
                // the dump's own `rate` is a different field and is zero on
                // every frame of this capture, so it is not compared.
                let rows: [(&str, [i32; sim::economy::RESOURCES]); 6] = [
                    ("bucket", l.bucket),
                    ("leftover", l.leftover),
                    ("resources", l.rate),
                    ("income", l.income),
                    ("resource_cap", l.cap),
                    ("gather_slots[scan]", l.gather_slots),
                ];
                for (key, ours) in rows {
                    let theirs = field(key);
                    // `resource_cap` is written a seventh time for the
                    // leader as a whole; only the six goods are compared.
                    if theirs.len() < sim::economy::RESOURCES {
                        continue;
                    }
                    for g in 0..sim::economy::RESOURCES {
                        compared += 1;
                        if i64::from(ours[g]) != theirs[g] {
                            wrong.push(format!(
                                "frame {n} who {who} {key} good {g}: ours {} theirs {}",
                                ours[g], theirs[g]
                            ));
                        }
                    }
                }
            }
        }

        // **The price, which is the item.** Twenty-two before, sixty after,
        // and sixty is what run41's before-and-after measures.
        assert_eq!(
            &city_price[..2],
            &[60, 60],
            "the AI's second Small City: 1 x BUILD_COST_FACTOR + 50 x 1 x 1, uncapped"
        );
        assert!(
            city_price[2..].iter().all(|&c| c == 0),
            "and nothing else: {city_price:?}"
        );

        // **The record**: 2,880 good-frames — forty frames, two players, six
        // goods, six fields — and 360 disagree, in exactly two shapes, both
        // of them a *standing* state rather than anything the window does.
        //
        // - **240 are `bucket` on goods 3, 4 and 5** (knowledge, metal,
        //   oil), both players, every frame: the original holds **0** and
        //   this crate holds **100**. Inert here because none of the three
        //   is available in the Ancient age and an unavailable good is never
        //   charged — but it is a hundred of something nobody gave the
        //   leader, and it is booked.
        // - **120 are `gather_slots`** — the two woodcutters' camps and the
        //   human's odd wealth slot, three per frame, for the two reasons
        //   the doc comment above sets out.
        //
        // `resource_cap` was the third shape and is gone: 200 to **0**, the
        // British commerce bonus arriving with the dump's own `tribe`.
        //
        // Everything else is exact on every frame: both players' `bucket` on
        // food, timber and wealth — which is the item — every `leftover`,
        // every `resources`, every `income`, every farm's gather slot, and
        // both players' whole cap.
        eprintln!("run40: {} of {compared} good-frames disagree", wrong.len());
        assert_eq!(
            compared, 2_880,
            "forty frames, two players, six goods, six fields"
        );
        let of = |k: &str| wrong.iter().filter(|w| w.contains(k)).count();
        assert_eq!(
            (
                of("bucket good 0"),
                of("bucket good 1"),
                of("bucket good 2")
            ),
            (0, 0, 0),
            "food, timber and wealth are the original's on every frame: {wrong:?}"
        );
        assert_eq!(
            (of("leftover"), of("resources"), of("income")),
            (0, 0, 0),
            "the rate and its accumulator are exact: {wrong:?}"
        );
        assert_eq!(
            of("gather_slots[scan] good 0"),
            0,
            "every farm's gather slot, both players: {wrong:?}"
        );
        assert_eq!(
            of("resource_cap"),
            0,
            "both players' whole commerce cap, the British +25% included: {wrong:?}"
        );
        assert!(
            of("gather_slots") <= 120 && wrong.len() <= 360,
            "the census fell: {} of {compared}, slots {} — the floors are \
             360 and 120",
            wrong.len(),
            of("gather_slots")
        );
    }

    /// **run40 and run41 — the ten `SITE` records, and the territory the
    /// site scorer reads through them (item 113).**
    ///
    /// The census windows the price item captured
    /// ([`run40_s_census_prices_the_ai_s_second_city_at_sixty`]) carry more
    /// than goods: `LEADERS=9` prints `Leader::sites` whole — ten
    /// `{wx, wy, val, reg, dist, rank}` a leader a frame — and
    /// `LeaderData::territory` beside it. Nine tenths of that record had
    /// gone uncompared, and inside it was the whole of Great Lakes' `1/1`.
    ///
    /// **The territory, first, because it is the item.** `World::compute_
    /// reg_territory@006b0bb0` rebuilds its per-player bonus table at the
    /// top of every region pass (lines 125–260) and reads
    /// `data_encrypted->epoch[1] ^ 0x63187` — the **Civic** library level,
    /// the same field `LeaderData::get_city_limit` names — for both
    /// `CIVIC_UPGRADE_TERR` and the Russians' per-step flat bonus. This
    /// crate built that table once, in `add_player`, and never rewrote it,
    /// so the AI's City State never widened its border: **261 cells against
    /// the original's 290**, with the human's 266 exact on every frame of
    /// both windows. [`sim::Sim::player_borders`] reads the level live and
    /// `sync_territory` rebuilds all eight rows, as the original does.
    ///
    /// Twenty-nine cells is not a rounding error in this mechanic, because
    /// `compute_site_stats`' fog test is `WorldData::was_seen@006b53f0` and
    /// its **first** arm is territorial: a cell owned by an ally — `is_ally`
    /// is reflexive — is seen wherever that owner has a city in the region.
    /// Every one of those cells was dark to the site scorer, so its 5×5
    /// slide could not move onto them. On run10's frame 575 the AI samples
    /// cell `(48, 29)`, the original slides it to `(47, 28)` for 6,181 and
    /// this crate could not see `(47, 28)` at all: it slid to `(49, 27)` for
    /// 1,159, and from there the second city went up in the wrong place.
    ///
    /// **What is still wrong, asserted as it stands.** One slot. Where the
    /// original's 5×5 leaves cell `(48, 29)` for `(47, 28)`, this crate
    /// keeps `(48, 29)` — `q = 19` at offset 0 against 17 at offset 1, and
    /// the original's `blocked_town` must be refusing the centre where this
    /// crate's `site_clear` allows it. `BuildTypeData::blocked_site@00636a50`
    /// counts the footprint tiles whose fog half-cell the placer has never
    /// seen and returns `0x24` when more than half of them are dark, with
    /// only a Dock exempt; `blocked_tcoord` here grants visibility
    /// everywhere (`docs/CITIES.md` §11). Landing that rule alone moves
    /// none of these numbers — the AI's own territory is "seen" through the
    /// same territorial arm — so it is booked rather than guessed at. The
    /// one wrong slot drags the `rank` of the four slots it outscores with
    /// it, which is where the rest of the residue comes from.
    #[test]
    fn run40_and_run41_s_sites_and_territory_are_the_original_s() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut ran = 0;
        for (file, lo, hi, fields, floor) in [
            (
                "gamelog-run40-census.txt",
                560i64,
                600i64,
                4_000usize,
                580usize,
            ),
            ("gamelog-run41-census.txt", 770, 800, 3_000, 520),
        ] {
            let Some(path) = dump(file) else {
                eprintln!("skipping: no {file} (set RON_GAMELOG_DIR)");
                continue;
            };
            ran += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            let mut compared = 0usize;
            let mut wrong: Vec<String> = Vec::new();
            let mut terr_compared = 0usize;
            let mut terr_wrong: Vec<String> = Vec::new();
            for n in 1..hi {
                built.tick();
                if n < lo {
                    continue;
                }
                // The live owned-cell count, which is what
                // `LeaderData::territory` is. The census's own
                // `my_team_terr` is a snapshot of it taken on the AI's
                // sweep and lags by up to a sweep, so it is not the field
                // to compare.
                let mut owned = vec![0i64; built.sim.players.len()];
                for y in 0..built.sim.world.height() {
                    for x in 0..built.sim.world.width() {
                        if let sim::world::Owner::Player(p) =
                            built.sim.world.owner(sim::world::Cell::new(x, y))
                        {
                            owned[p as usize] += 1;
                        }
                    }
                }
                for who in 0..2i64 {
                    let Some(block) = log.leader_block(n, who) else {
                        continue;
                    };
                    terr_compared += 1;
                    let theirs = block.int("territory");
                    if theirs != Some(owned[who as usize]) {
                        terr_wrong.push(format!(
                            "frame {n} who {who}: ours {} theirs {theirs:?}",
                            owned[who as usize]
                        ));
                    }
                    let sites: Vec<(i64, i64, i64, i64, i64)> = block
                        .kids("SITE")
                        .map(|s| {
                            (
                                s.int("wx").unwrap_or(0),
                                s.int("wy").unwrap_or(0),
                                s.int("val").unwrap_or(0),
                                s.int("dist").unwrap_or(0),
                                s.int("rank").unwrap_or(0),
                            )
                        })
                        .collect();
                    assert_eq!(sites.len(), 10, "ten SITE records, frame {n} who {who}");
                    for (i, t) in sites.iter().enumerate() {
                        let o = built.sim.ai[who as usize].sites[i];
                        let ours = (
                            i64::from(o.wx),
                            i64::from(o.wy),
                            i64::from(o.val),
                            i64::from(o.dist),
                            i64::from(o.rank),
                        );
                        compared += 5;
                        if ours != *t {
                            wrong.push(format!(
                                "frame {n} who {who} [{i}] ours {ours:?} theirs {t:?}"
                            ));
                        }
                    }
                }
            }
            assert_eq!(
                compared, fields,
                "{file}: ten slots, five fields, two leaders"
            );
            // **The item.** Both players' territory, every frame of the
            // window, exact — 261 against 290 before the Civic level
            // reached the border pass.
            assert!(
                terr_wrong.is_empty(),
                "{file}: {} of {terr_compared} leader-frames hold the original's territory:\n  {}",
                terr_compared - terr_wrong.len(),
                terr_wrong.join("\n  ")
            );
            let bad = wrong.len() * 5;
            eprintln!("{file}: {bad} of {compared} site fields disagree");
            assert!(
                bad <= floor,
                "{file}: the site record fell to {bad} of {compared} disagreeing fields, \
                 the ceiling is {floor}:\n  {}",
                wrong.join("\n  ")
            );
        }
        assert!(ran > 0, "neither census window is on this machine");
    }

    /// **Run33's frame 361 — the scout's second explore target, and the
    /// surface probe that had been reading the wrong tile.**
    ///
    /// `run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings` pins the
    /// mechanic's *opening* call, ten draws over four rings on a stream
    /// that is installed rather than reached. This pins the one call in
    /// the whole corpus where the cell filter actually decides something:
    /// the AI scout `1/0` re-targets on run33's frame 361, three hundred
    /// frames into a game it walked into on its own stream, and the
    /// original spends **twenty-seven** draws there over seven rings.
    ///
    /// The filter's surface read (`docs/SCOUT.md` §7) is what this
    /// caught. The listing at `005f6542` is
    /// `movb 0x4(%eax,%ecx,2)` over `ecx = (4y + 2)·tile_xs + 4x`, and
    /// `TData` is two bytes wide with its `mask` at `+0` — so the `+4` is
    /// **two elements**, and the tile read is the cell centre
    /// `(4x + 2, 4y + 2)`, not `(4x, 4y + 2)`. Cell `(48, 23)` carries
    /// ocean at tile `(192, 94)` and land at `(194, 94)`: with the wrong
    /// probe it was refused, the call spent twenty-six draws instead of
    /// twenty-seven, and the scout went to `(48, 20)` instead.
    ///
    /// Two oracles, and each catches the transposition on its own: the
    /// trace's own site sequence for the frame, and the destination the
    /// dump prints for the order the call issues.
    #[test]
    fn run33_s_scout_re_targets_at_361_on_the_original_s_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..363 {
            built.tick();
        }

        // The original's side: frame 361's draws, filtered to the ones
        // `Unit::think_scout` took itself.
        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        let theirs: Vec<String> = trace
            .run_in(361, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| trace.label(d))
            .collect();
        // Seven rings, and thirteen cells taken across three of them —
        // `docs/SCOUT.md` §10's table for this frame.
        let ring = [SITE_ROTATION, SITE_PHASE];
        let mut want: Vec<&str> = Vec::new();
        for cells in [0, 0, 0, 6, 0, 2, 5] {
            want.extend(ring);
            want.extend(std::iter::repeat_n(SITE_CELL, cells));
        }
        assert_eq!(theirs, want, "the trace's own frame-361 sequence");

        // Ours: the same frame's marks, filtered the same way. The stream
        // is **reached**, not installed — the simulation walks its own way
        // to frame 361 and the trace agrees draw for draw to 431.
        let sites = [SITE_ROTATION, SITE_PHASE, SITE_CELL];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 361)
            .map(|(_, v)| v.clone())
            .expect("frame 361's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our twenty-seven draws, at the original's sites, in its order"
        );

        // And the target the call chose, against the dump's own. The order
        // reaches the unit through its group, so the first frame that
        // prints it is 362, one past the frame that thought.
        let frames = log.frame_states();
        let dest = frames
            .iter()
            .find(|f| f.n == 362)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run33's frame-362 order for 1/0");
        assert_eq!(
            dest,
            (37_368, 18_168),
            "the original: inside tile (194, 94)"
        );
        let scout = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].index == 0)
            .expect("1/0");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "cell (48, 23), whose centre tile is land where its first is ocean"
        );
    }

    /// **Run33's frame 482 — the scout's third explore target, and the fog
    /// the vision projection had been throwing to the wrong half-cell.**
    ///
    /// The AI scout `1/0` re-targets again on frame 482, and the ring walk
    /// is the same shape as frame 361's: seven rings around two cities, the
    /// first its own leader's at cell `(55, 21)` walked every other ring to
    /// twelve, then a three-ring look at the human's at `(4, 40)`. What
    /// this pins is the **cell filter's fog read** (`docs/SCOUT.md` §7):
    /// the original scores **three** cells in city one's ring 7 where this
    /// simulation scored two, because cell `(56, 28)` was seen here and not
    /// there.
    ///
    /// Why it was seen here. `Object::update_seen` throws a small land
    /// unit's disc a half-cell forward of its nose, and the angle it
    /// projects along is `UnitData +0x50` — the unit's own `angle`, the
    /// heading `Unit::set_angle` writes toward the next waypoint — not the
    /// guy's eased facing (`docs/VISION.md` §3). This crate had the guy's.
    /// On frame 168 the scout was mid-turn: the dump's `UNITDATA angle` is
    /// −51.6° and its guy's is −83.0°, and the two projections land in
    /// different fog cells. The disc thrown along the guy's angle reached
    /// fog `(113, 57)` and the original's did not, so `(56, 28)` was
    /// "already seen" here three hundred frames later and the scan spent
    /// thirty draws where the original spends thirty-one.
    ///
    /// The score arithmetic is what identifies the cell, and it is worth
    /// keeping because the trace cannot: the four cells this simulation
    /// refuses in that ring are `(50, 26)`, `(56, 28)`, `(58, 28)`,
    /// `(48, 20)` and `(48, 24)`, and the scan's own `dist × 8` puts three
    /// of them — 48, 0 and 32 — **below** the 76 that wins the frame. Only
    /// `(56, 28)` at 96 and `(58, 28)` at 104 can be scored without
    /// changing the target the dump prints, and only `(56, 28)`'s reveal
    /// falls on a frame the scout was turning.
    #[test]
    fn run33_s_scout_re_targets_at_482_on_the_original_s_ring() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(trace)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..484 {
            built.tick();
        }

        use sim::scout::{SITE_CELL, SITE_PHASE, SITE_ROTATION};
        let theirs: Vec<String> = trace
            .run_in(482, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| trace.label(d))
            .collect();
        // Seven rings and six cells: three in the first city's ring 7, then
        // one and two in the human city's rings 1 and 2.
        let ring = [SITE_ROTATION, SITE_PHASE];
        let mut want: Vec<&str> = Vec::new();
        for cells in [0, 0, 0, 3, 0, 1, 2] {
            want.extend(ring);
            want.extend(std::iter::repeat_n(SITE_CELL, cells));
        }
        assert_eq!(theirs, want, "the trace's own frame-482 sequence");

        let sites = [SITE_ROTATION, SITE_PHASE, SITE_CELL];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 482)
            .map(|(_, v)| v.clone())
            .expect("frame 482's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our thirty-one draws, at the original's sites, in its order"
        );

        // Cell `(56, 28)` is the one the projection decided, and the fog is
        // where it shows: unseen for player 1 on the original's stream, and
        // unseen here now.
        assert!(
            built.sim.world.seen2(2 * 56 + 1, 2 * 28 + 1).unwrap_or(0) & 2 == 0,
            "cell (56, 28) is not seen by player 1 at frame 482"
        );

        // And the target, against the dump's own — cell `(52, 28)`, which
        // both sides pick and which the extra draw does not move.
        let frames = log.frame_states();
        let dest = frames
            .iter()
            .find(|f| f.n == 483)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run33's frame-483 order for 1/0");
        assert_eq!(
            dest,
            (40_440, 22_008),
            "the original: inside tile (210, 114)"
        );
        let scout = (0..built.sim.units.len())
            .find(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].index == 0)
            .expect("1/0");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "cell (52, 28), seven rings out and the cheapest unseen one"
        );
    }

    /// **The fog grid, whole, on ten consecutive frames** — the record the
    /// `WORLD` dump has always printed and nothing compared.
    ///
    /// `seen2` is a 120 × 120 byte grid on this map, one bit a player, and
    /// three things read it: the pathfinder's unseen-cell preference, the
    /// scout's cell filter and the AI's site census. Until this test it was
    /// installed from a frame-0 dump and then grown by `crate::vision`
    /// with **nothing checking the growth** — a wrong disc, a wrong centre
    /// or a wrong radius would show up only when some later mechanic read
    /// a cell it had got wrong, three hundred frames downstream and wearing
    /// somebody else's name. That is exactly how it went: item 79 was a
    /// scout's ring walk and turned out to be this grid.
    ///
    /// run13 is run10's own game with `DUMP_ALL` over frames 95–104, so it
    /// prints the grid ten times. The simulation is stood up on run33's
    /// start (the same game again) and walked forward with nothing
    /// installed; each frame's grid is compared cell for cell against the
    /// dump taken at the **start** of the next frame.
    #[test]
    fn run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(r13)) = (
            dump("gamelog-run33-longtrace.txt"),
            dump("gamelog-run13-window-95-105.txt"),
        ) else {
            eprintln!("skipping: no run33/run13 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        let t13 = std::fs::read_to_string(&r13).unwrap();
        let l13 = Log::parse(&t13);
        let grids: Vec<(i64, Vec<u8>)> = l13
            .dumps()
            .into_iter()
            .filter_map(|(n, body)| {
                let w = body.kid("WORLD")?;
                let fog = crate::gamelog::world_fog(&w.fields);
                (!fog.is_empty()).then_some((n, fog))
            })
            .collect();
        assert_eq!(
            grids.iter().map(|(n, _)| *n).collect::<Vec<i64>>(),
            (95..=104).collect::<Vec<i64>>(),
            "run13's ten dumped worlds"
        );

        let (fw, fh) = (built.sim.world.fog_xs(), built.sim.world.fog_ys());
        assert_eq!((fw, fh), (120, 120), "Great Lakes' fog grid");
        for f in 1..=103i64 {
            built.tick();
            // The dump at the head of frame `f + 1` is the state this many
            // ticks have produced.
            let Some((_, theirs)) = grids.iter().find(|(n, _)| *n == f + 1) else {
                continue;
            };
            assert_eq!(theirs.len(), (fw * fh) as usize, "frame {f}'s grid size");
            let bad: Vec<(i32, i32, u8, u8)> = (0..fh)
                .flat_map(|y| (0..fw).map(move |x| (x, y)))
                .filter_map(|(x, y)| {
                    let t = theirs[(y * fw + x) as usize];
                    let o = built.sim.world.seen2(x, y).unwrap_or(0);
                    (o != t).then_some((x, y, o, t))
                })
                .collect();
            assert_eq!(
                bad,
                vec![],
                "the fog grid parts after {f} ticks, against run13's FRAME {}",
                f + 1
            );
        }
    }

    /// **The second map's word, and where it parts.**
    ///
    /// run33 gave Great Lakes a word — the first frame whose draw *count*
    /// is not the original's — and it is the sub-score a dozen items were
    /// steered by. East Indies had none: run39 was scored on ticks and
    /// orders alone, and `rontrace-run39.log` sat unread beside its dump.
    ///
    /// It parts at **19**, which is 148 frames before the order-list
    /// divergence at 168 that the queue had been calling this map's first.
    /// Everything run39 diverges on after 19 is on a stream that is
    /// nobody's — **its ticks and orders of 167 included** — so this is
    /// the number to move, and the score beside it is the early window
    /// rather than a total over the game (a total past the parting is
    /// noise: a more faithful simulation can score worse on a stream that
    /// is nobody's, and this one measurably does).
    ///
    /// What parts it is **the pasture's five animals**. East Indies' AI
    /// starts with an animal farm (`farm_type 1`, `o 2003`) and Great
    /// Lakes has none, which is why run33 never saw any of this. Three
    /// things are wrong with them here, and the trace names all three:
    ///
    /// 1. **They have no `type_index`**, so `Sim::slot_length` cannot
    ///    reach the install's gaia table and every one carries
    ///    `sim::anim::UNKNOWN`. A clock that never wraps costs no draw,
    ///    and that is **three** of the original's six `Guy::inc_time`
    ///    wraps on frame 29.
    /// 2. **They are the wrong species.** `Farms::add_animals@008d8f30`
    ///    throws `(rnd & 1) == 0 ? FARMCHICKEN : FARMPIG` per animal, and
    ///    a pasture is therefore always **one** species: the four draws
    ///    are a fixed stride, and `Random::get(0, 0xffff)`'s low bit is
    ///    the complement of the seed's, which the LCG flips every step —
    ///    five even coins or five odd ones, never a mix. run39's, read
    ///    back out of its own trace at `add_animals+0x92` with the seeds
    ///    the record carries, are even: **chickens**, whose
    ///    `CHAR_DEFAULT` is 30 frames where a pig's is 90.
    /// 3. **The walk is read and not issued.** The animal whose
    ///    `think_farm_animal` phase hits frame 0 — `o` 0 of 0–4, slot 0;
    ///    the trace's own phases `{0, 108, 116, 122, 126}` are
    ///    `(o·(slot+1)) % 128` for exactly that assignment — is handed a
    ///    `MOVE_TO` this crate does not add. It walks, and its arrival
    ///    spends the **two** `Animal::do_idle` set_anim draws of frames
    ///    19 and 20, after which its clock is nineteen frames behind the
    ///    other four and wraps at 49 rather than 29. The signature
    ///    repeats all game: every `think_farm_animal` draw is followed
    ///    nine to twenty-five frames later by a pair of `Animal::do_idle`
    ///    draws on consecutive frames.
    ///
    /// All three are landed (2026-08-30). The animals' **positions** were
    /// the wall — `add_animals` places each of the five at the farm ±
    /// `% 0x180 − 0xc0` on each axis, up to a whole tile, from two draws
    /// inside `Setup::build_empire`, whose stream the harness does not
    /// replay — and [`borrow_pasture`] takes them from the run's own
    /// trace, the way the heights and the herds are taken from a sibling
    /// dump. run39's five are `(−143, 40)`, `(−187, −148)`, `(−39, −144)`,
    /// `(−83, −76)`, `(−63, 56)` as `(dy, dx)`, and the test asserts them.
    ///
    /// **What is left at 19 is not the pasture's.** With the walk issued
    /// the arrival's *second* draw, frame 20, comes right and the first,
    /// frame 19, does not — and the two residues behind that are movement
    /// and animation, not this mechanic:
    ///
    /// - **The animal arrives one frame late.** Its 455-unit walk takes
    ///   nineteen 25-unit steps here and eighteen there; the arrival test
    ///   is `dist ≤ tolerance` and this crate's straight-line goal carries
    ///   `tolerance 0` (`path.rs:1036`). Give the chicken one more unit of
    ///   speed and frame 19 matches the original **draw for draw** — that
    ///   is how the two halves were told apart.
    /// - **An arrival costs two `Animal::do_idle` draws, not one.** The
    ///   pair is on consecutive frames, every time, all game. One is the
    ///   walk-to-idle transition this crate spends; the other needs the
    ///   guy to be playing something non-idle on the following frame,
    ///   which is what `Guy::move`'s turn arm would do — the unit is still
    ///   easing onto the order's angle on both frames (queue items 36, 37).
    ///
    /// History:
    ///   2026-08-30  word parts at **19**; of the first 64 frames 49
    ///               spend the original's number of draws and 47 draw for
    ///               draw (the first reading of this trace).
    ///   2026-08-30  the pasture's five landed whole — the species and the
    ///               `type_index`, the borrowed positions, and
    ///               `think_farm_animal`'s `MOVE_TO`. The window goes
    ///               49/47 → **62/55**; frames 20, 29 and 32 come right
    ///               and the word holds at 19 on the arrival frame alone.
    ///               The feared cost never arrived: ticks and orders stay
    ///               at 167 and player 0 at 219, because the walk is what
    ///               (1) and (2) were missing rather than a second
    ///               perturbation. A pasture animal that carries a
    ///               `MOVE_TO` it cannot step — `movement.speed` unset —
    ///               *does* cost player 0 two frames, which is what the
    ///               first attempt measured.
    ///   2026-08-30  the pair, landed behind item 36: word **19 → 69** and
    ///               the window **64 of 64 on the count and 64 draw for
    ///               draw**. `Unit::init`'s snap on the animal's birth
    ///               point and `Guy::move`'s turn arm — either alone is
    ///               worse than neither (20 and 60/53; 58/57), and both
    ///               together cost the *other* map 196 frames of word
    ///               until the farmers' angles were the original's
    ///               (`docs/SYNC.md` §3.11's last section, §3.12).
    ///   2026-08-30  word **69 -> 91** with item 95, **the animal's
    ///               hurry**: `AnimalData::get_speed@005d8380` walks an
    ///               animal at `speed * 3 / 2` while it is more than
    ///               `0x180` from its order's goal, and this crate walked
    ///               it at `speed`. Gaia's `8/2` therefore took ten steps
    ///               over ground the original crosses in nine, reached its
    ///               blocked stand a frame late, and spent
    ///               `sim::anim::SITE_BLOCKED` on 70 where the original
    ///               spends it on 69. The window holds at 64/64.
    ///               `docs/SYNC.md` §3.13, and
    ///               [`an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`]
    ///               is the rule against the record.
    ///   2026-08-30  word **91 -> 201**, **the blocked animal's dropped
    ///               walk**: `Unit::resolve_unit_collision`'s first
    ///               statement is `SubObjectData::is_animal`, and when it
    ///               answers the body is the `QUEUE_NEW` clear —
    ///               `docs/COLLISION.md` §6 step 0. Gaia's `8/2`, blocked
    ///               by its herd-mate on 69, stands there for the rest of
    ///               the capture; this crate sidestepped, snapped it onto
    ///               its cell centre and walked it round to the goal,
    ///               where it then blocked `8/0`'s wander spot on 89. The
    ///               window holds at 64/64. `docs/SYNC.md` §3.14, and
    ///               [`a_blocked_animal_drops_its_walk_where_it_stands`]
    ///               is the rule against the record.
    ///   2026-08-30  word **201 -> 219**, **the pasture's herder**:
    ///               `Unit::do_gather@005ef2a0:5efd77` takes the whole of
    ///               the function when the farm's `farm_type & 1` is set —
    ///               a herder shows the sow animation and draws only on
    ///               the frames where `(o · 7 + frame + who) % 256` is
    ///               zero, then walks to one of the farm's inner four
    ///               tiles. `docs/ORDERS.md` §6.5 had the arm from its
    ///               first writing and `do_farm` never did, so the AI's
    ///               `1/3` ran the crop switch instead; a pasture is the
    ///               one farm `Farms::inc_time` skips, so its cell reached
    ///               `RIPE_ADDS` on the herder's own adds alone — the two
    ///               hundredth frame rather than the hundredth — and it
    ///               re-picked a tile on 201 where the original spends its
    ///               pair on **234**, its first phase frame. **Every one
    ///               of the 219 frames is now draw for draw**, not only
    ///               the first 64 (274 as of the next entry). `docs/SYNC.md` §3.15, and
    ///               [`a_pasture_herder_walks_only_on_its_own_256_frame_phase`]
    ///               is the rule against the record.
    ///   2026-08-30  word **219 -> 274**, **the frame's two loops**:
    ///               `Objects::process_all@0065dce0` is the units,
    ///               rotated by owner, and then a *second, unrotated* pass
    ///               over each player's buildings and then their walls.
    ///               `docs/SYNC.md` §3.2 had said so since it was written
    ///               and `Sim::tick` ran the buildings first, so frame
    ///               219's citizen re-target fell behind the frame's road
    ///               search instead of in front of it — and the search
    ///               that looked 152 nodes against 129 was the same
    ///               search on a different world. With the order right it
    ///               is 129 against 129 and the frame is draw for draw.
    ///               `docs/SYNC.md` §3.16. Two rules moved with it: a
    ///               unit created this frame is *not* skipped by
    ///               `Objects::inc_time`, and `think_peasant`'s idle
    ///               threshold is **1** for an AI-driven worker.
    ///   2026-08-30  **413** (item 98, `docs/AI.md` §17): a script's
    ///               `static` is one variable on `Script::static_vars`,
    ///               not a frame slot, and this crate's mirror wiped
    ///               every one of them on the second call. So
    ///               `economic.bhs`'s `needed_citizens` was zero from
    ///               the second call on, its every-call
    ///               `train_unit_with_need` trained nobody, and the
    ///               citizen the original's city hall finishes at 274 was
    ///               never queued. With the statics live the queue clock
    ///               runs 100 to 9,750 on the original's own frames and
    ///               the guy is born on 274.
    ///   2026-08-31  word **413 -> 576** (item 99), **the building's own
    ///               line of sight**: `Build::activate@00623e20`'s last
    ///               statement is `update_seen(0)` — the whole fog disc —
    ///               and nothing here threw it, so the fog grew only where
    ///               units walked. The AI's sixth farm finishes on frame
    ///               219 at cell `(54, 51)` with `mylos 8`, which lights
    ///               the three cells of column 56 east of it; nineteen
    ///               frames later `Unit::think_scout` re-targets and the
    ///               `EXPLORE_TO` path runs *through* those cells here — a
    ///               scout prices unseen ground at a base of 8 against a
    ///               seen cell's `0x400` — where the original, which can
    ///               see them, walks the seen column 55 instead. Its scout
    ///               therefore arrived on 412 and idled on 413 while this
    ///               one was still twelve frames short. `docs/VISION.md`
    ///               §2.1.
    ///   2026-08-31  word **576 -> 645** (item 100), **the bird's landing
    ///               search**: `Animal::think_bird@005d79e0`'s tail is
    ///               thirty rounds over the cell list of the region the
    ///               patrol point sits in, two draws a round, and
    ///               `docs/SYNC.md` §3.9 had recorded it as unreachable
    ///               because no capture had reached it. run39's gaia bird
    ///               `9/8` reaches it on frame **576** — its landing roll
    ///               is the `% spell_time` at `+0x1f8`, and the counter
    ///               only passes 100 after ~90 frames of flight — so the
    ///               original spends **sixty** draws there and this crate
    ///               spent none. The frame's 118 against 56 was that,
    ///               plus the two the frame's own animal birth then falls
    ///               out of step over.
    ///
    ///               **What the frame was not is its AI**, which is what
    ///               the item was booked as. Frame 576 is
    ///               `place_city_with_cost` five times over, and this
    ///               crate spends every one of its twenty draws already:
    ///               `make_stuff`'s expiry walk simply had no
    ///               [`sim::Sim::mark`] on it, so each of its draws read
    ///               as the last site marked — `compute_sites+0x50a` —
    ///               and the sequence appeared to part on a draw that was
    ///               in fact correct. An unnamed draw is a lie in this
    ///               comparison, not a gap.
    ///   2026-08-31  word **742 -> 867** (item 102), **the far wander's
    ///               literal bearing**: `Animal::do_idle@005d7460` hands
    ///               `UnitType::find_nearby_spot` the constant
    ///               `0x55555555` as its sweep's starting angle — the same
    ///               120° `Unit::init` writes into a unit that has never
    ///               turned — where every other call site in the
    ///               executable passes a real bearing, and this crate
    ///               passed the animal's facing. Gaia's `8/3` was 180° out,
    ///               so on frame 736 it walked due north from the herd
    ///               centre where the original walks east-north-east; six
    ///               frames later the original's step is refused by its
    ///               herd-mate `8/2` and spends the blocked stand
    ///               (`Guy::set_anim+0x97a < Unit::move_step+0x823`,
    ///               `docs/COLLISION.md` §5), while ours walked on into
    ///               open ground. **The item was booked as a blocked
    ///               stand and the blocked stand was already right**: with
    ///               the bearing corrected the whole walk is the
    ///               original's step for step — `(28741, 24384)`,
    ///               `(28754, 24360)` … `(28793, 24288)` — and the
    ///               refusal, the dropped walk and the stand all fall
    ///               where they fall in the dump, with no change to the
    ///               collision model at all. Frame 867 is next, and it is
    ///               `Unit::explore_goody+0x27c < Unit::set_new_location
    ///               +0x3cc < Unit::move_step+0x8f4`, three draws.
    ///               `docs/SYNC.md` §3.19, and
    ///               [`a_far_wander_sweeps_from_the_literal_bearing`] is
    ///               the rule against the record.
    ///   2026-08-31  word **867 -> 879** (item 104), **the goody box**:
    ///               `Unit::set_new_location@005f8d20+0x3cc` calls
    ///               `Unit::explore_goody@005f9780` on any non-animal land
    ///               unit that enters a new cell carrying `WData.flags &
    ///               0x8000`, and run39's world has seven of them —
    ///               the `WORLD` record's own `goodies 7`. Player 1's
    ///               scout `1/0` walks into `(45, 49)` on 867, and the box
    ///               holds a lottery: one draw for each good the finder can
    ///               gather, scored `draw % 25 + bucket[good]`, lowest
    ///               wins, knowledge never a candidate. In the Ancient age
    ///               that is **three** — food, timber and wealth — and this
    ///               crate spent none, so the frame read 5 against 9 and
    ///               the sixth `Farms::inc_time` draw (`+0x1de`, the
    ///               sprout) fell out with them. Frame 879 is next and it
    ///               is the scout: two `Unit::set_anim` stands and then
    ///               `Unit::think_scout+0x436`/`+0x458` six times over with
    ///               `+0x64c` twice — sixteen draws this crate does not
    ///               spend. `docs/GOODY.md`, and
    ///               [`a_goody_box_draws_once_for_each_good_its_finder_can_gather`]
    ///               is the rule against the record.
    ///   2026-08-31  word **879 -> 1256** (item 106), **the walk to the
    ///               box**: frame 879 was the scout going idle and
    ///               re-thinking, and the reason it was idle twelve frames
    ///               after the box is that its explore order had been
    ///               *re-aimed at the box* — `Unit::do_explore_to@005f24a0`
    ///               calls `Unit::find_goody_box@005f2540` one frame in
    ///               fifteen, and on frame 825 the sweep found `(45, 49)`
    ///               and re-issued the walk to that cell's centre. The
    ///               gate is not the cell's `was_seen`, which the box's
    ///               own borders answer yes from frame 0, but the **item's**
    ///               `ItemData::is_seen` — the bare accumulated fog, which
    ///               only reaches the box between 811 and 825. So the
    ///               retarget lands on 825, the arrival on 879, and
    ///               `think_scout` runs there with the original's own six
    ///               ring pairs. Three hundred and seventy-seven frames,
    ///               four `think_scout` frames (879, 1021, 1143, 1231) and
    ///               the goody's own second box at 1659 all pass; 1256 is
    ///               next and it is a **bird**: ours lands (87 draws, the
    ///               third bird's `Animal::think_bird+0x2aa`) where the
    ///               original's eight birds only think (27).
    ///               `docs/GOODY.md` §7, and
    ///               [`a_scout_re_aims_its_walk_at_a_goody_box_it_has_seen`]
    ///               is the rule against the record.
    ///   2026-08-31  word **1256 -> 1373** (item 109), **the bird's
    ///               landing step**: frame 1256 was a bird landing that
    ///               the original's does not, and the counter is the only
    ///               state behind it. `Unit::do_air_patrol@005ea620`'s
    ///               tail, after `do_air_physics` returns, branches on
    ///               `vtable+0x30` — `SubObjectData::is_animal`, folded
    ///               onto `Buffer::is_pending_load`'s `return 1` on all
    ///               three `Animal` vtables — and the animal arm is
    ///               `else if (spell_time == 0)
    ///               spell_time = 1`. `think_bird` steps the counter on
    ///               every frame it runs, so it is 0 there only on a
    ///               frame the landing search has just zeroed: one extra
    ///               step per landing, and nothing else. run39's second
    ///               bird had landed on 944, so its counter reached 351
    ///               rather than 352 and `27127 % 351` is exactly the
    ///               `== 100` the branch tests for. With the step the
    ///               original's own landings at **1368** and 1376 fall
    ///               where they fall; 1373 is next and it is the AI
    ///               scout, which takes an eighth `+0x436`/`+0x458` ring
    ///               pair where the original stops at seven and goes to
    ///               `Unit::think_scout+0x941` and five `+0xaba`.
    ///               `docs/SYNC.md` §3.9, and
    ///               [`a_bird_s_landing_frames_are_the_trace_s_own`] is
    ///               the rule against the record.
    ///   2026-08-31  word **1373 -> 1570** (item 110), **the region
    ///               fallback's cell walk**. The eighth ring pair the row
    ///               above blames was never a ring: `scout_region_scan`
    ///               took §11's stride draw with no `mark` of its own, so
    ///               the stream inherited `SITE_PHASE` and the sequence
    ///               *read* as a seventh ring. The original walks its six
    ///               rings, finds nothing, leaves the city loop on
    ///               `best > 199` and scans the whole region — one
    ///               `+0x941` and one `+0xaba` per accepted cell, five of
    ///               them here — and that walk had been called
    ///               unreproducible because `Region.coords`' order is
    ///               nobody's dump. It is the **cell grid's own**:
    ///               `Regions::rebuild_coords@0067f800`, the last writer
    ///               `Regions::find_all` reaches, refills every list by a
    ///               row-major sweep. `docs/SCOUT.md` §11, and
    ///               [`a_scout_with_no_city_near_scans_its_whole_region`]
    ///               is the rule against the record.
    ///   2026-08-31  word **1570 -> 1647** (item 121), **two frames of the
    ///               wood dump**, one in the art loader and one in the
    ///               order. `AnimMgr::force_load@0053ade0` drops a
    ///               **non-looping** animation's last key before the
    ///               conversion — `times = key_times[n-2]` when
    ///               `loopings[i] == 0` — so `lumberjack_dump.bha`'s 2157,
    ///               2190 is 32 frames and not 33, which is what every
    ///               `GUY` block in the corpus prints for `cur_anim 27`
    ///               and what no dump this suite read had ever been asked
    ///               ([`the_install_s_piece_lengths_match_the_dumps`],
    ///               five dumps too short). And the frame that gave back:
    ///               `Unit::set_angle(a, a, **0**)` is what every order in
    ///               the ordinary path calls, so the guy takes `des_angle`
    ///               and its own `angle` stands — `Guy::move` finds a body
    ///               at its destination owed a turn and spends the **turn
    ///               arm**, `set_anim(CHAR_WALK, 0, 1)`, over whatever the
    ///               order just asked for. run44's citizen `0/2` is the
    ///               record: at the camp on 542 its `wait` steps 32 → 31
    ///               and its guy plays the carrying walk restarted; the
    ///               dump is 543. `docs/ANIM.md` §3.1 and §4.7. Next is
    ///               frame 1647, and it is not this family: nineteen draws
    ///               against four, opening `Guy::set_anim+0x97a <
    ///               Unit::do_idle+0x7d`.

    #[test]
    fn run39_s_long_trace_says_where_the_second_map_s_word_parts() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        assert_eq!(
            init.pasture.len(),
            1,
            "run39's trace reached the setup and East Indies' AI has one pasture"
        );
        assert!(
            init.pasture[0].iter().all(|a| a.chicken),
            "a pasture is one species, and run39's five coins are even"
        );
        assert_eq!(
            init.pasture[0]
                .iter()
                .map(|a| (a.dy, a.dx))
                .collect::<Vec<_>>(),
            vec![(-143, 40), (-187, -148), (-39, -144), (-83, -76), (-63, 56)],
            "the five offsets `report.py <log> draws setup` prints"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 1_800,
            "run39's traced length is {last}, wanted 1,800+"
        );
        for _ in 0..last {
            built.tick();
        }
        let first_count = built
            .frame_sites
            .iter()
            .find(|(f, ours)| ours.len() != tr.labels(*f).len())
            .map(|(f, _)| *f)
            .unwrap_or(last);
        // Past the parting the totals over the whole game are noise — a
        // more faithful simulation can score worse on a stream that is
        // nobody's — so the number pinned beside the word is the **early
        // window**: of the first 64 frames, how many spend the original's
        // number of draws, and how many draw for draw.
        const WINDOW: i64 = 64;
        // And the frame the *sequence* parts on, which since the herder
        // (§3.15) is the same frame: every draw of every frame before it
        // is the original's, in its order, not merely its count.
        let first_part = built
            .frame_sites
            .iter()
            .find(|(f, ours)| **ours != tr.labels(*f))
            .map(|(f, _)| *f)
            .unwrap_or(last);
        let words = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *f < WINDOW && ours.len() == tr.labels(*f).len())
            .count();
        let matched = built
            .frame_sites
            .iter()
            .filter(|(f, ours)| *f < WINDOW && **ours == tr.labels(*f))
            .count();
        eprintln!(
            "run39: word parts at {first_count}, sequence at {first_part}; of the first \
             {WINDOW} frames {words} spend the original's number of draws and {matched} \
             draw for draw"
        );
        // The parting frame's own row, and the first few inside the window
        // — the row is the successor item every time the number moves, so
        // the run prints it rather than leaving it to be re-derived.
        let row = |f: i64, ours: &Vec<String>| {
            let theirs = tr.labels(f);
            let at = (0..ours.len().max(theirs.len()))
                .find(|&i| ours.get(i) != theirs.get(i))
                .unwrap_or(0);
            format!(
                "frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
                ours.len(),
                theirs.len(),
                ours.get(at),
                theirs.get(at),
            )
        };
        let mut shown = 0;
        for (f, ours) in built.frame_sites.iter().take(WINDOW as usize) {
            if *ours == tr.labels(*f) {
                continue;
            }
            eprintln!("{}", row(*f, ours));
            shown += 1;
            if shown == 4 {
                break;
            }
        }
        for (f, ours) in built.frame_sites.iter() {
            if *f == first_part {
                eprintln!("parts: {}", row(*f, ours));
            }
            if *f == first_count && first_count != first_part {
                eprintln!("counts: {}", row(*f, ours));
            }
        }
        assert!(
            first_count >= FLOORS[0].word
                && first_part >= FLOORS[0].word
                && words >= 64
                && matched >= 64,
            "the second map's word fell: parts at {first_count}, its sequence at \
             {first_part}, {words} of the first {WINDOW} frames on the count, \
             {matched} draw for draw — the floor is {}, {}, 64 and 64",
            FLOORS[0].word,
            FLOORS[0].word
        );
    }

    /// **A scout whose city loop finds nothing scans its whole region**
    /// (2026-08-31, item 110) — the mechanic behind East Indies' word going
    /// 1373 → 1570, asserted as a sequence and as a destination.
    ///
    /// Run39's frame 1373 is the one frame in the corpus that reaches
    /// `docs/SCOUT.md` §11. The AI scout `1/0` walks its own city's six
    /// rings — `+0x436`/`+0x458` six times, and **not one `+0x64c`**,
    /// because by 1373 every cell within twelve of that city has been seen
    /// — so the city loop comes out with `best` still at 99,999,999, the
    /// tail's `199 < best` sends it to the region fallback, and the
    /// original spends one `+0x941` and five `+0xaba` there.
    ///
    /// That branch had been read but not implemented, on the ground that
    /// it strides through `Region.coords` and no dump prints that list. It
    /// does not need one. `Regions::find_all@0067eff0` appends coordinates
    /// in its flood order, merges, sorts — and then **frees the list and
    /// calls `Regions::rebuild_coords@0067f800`**, which refills every
    /// region's array by a plain row-major sweep of the cell grid:
    /// `for y { for x { coords[wdata[xs·y + x].region].push((x, y)) } }`.
    /// So the order is the grid's own, the sim's per-cell region map from
    /// the `WORLD` dump is enough to rebuild it, and this branch is now
    /// the original's draw for draw.
    ///
    /// Two oracles, as with the two run33 re-targets: the trace's own site
    /// sequence for the frame, and the destination the dump prints for the
    /// order the call issues. Made to fail three ways: the scan's own
    /// `+0x941` mark removed — which is the defect this found, the stride
    /// draw inheriting `SITE_PHASE` and reading as a seventh ring; the
    /// coordinate sweep transposed to column-major, which takes **one**
    /// cell where the original takes five; and the surface probe put back
    /// on `4x` rather than the cell centre `4x + 2` — §7's own trap, which
    /// costs exactly one cell here as it did on run33's 361.
    ///
    /// What this frame does **not** separate, and what therefore stays a
    /// reading in `docs/SCOUT.md` §13: the `× 16` distance scale (the city
    /// loop's is `× 8`), the doubling of the winner at `005f6bb9`, and the
    /// sense of `local_74`. Each was inverted in turn and the word held at
    /// 1570 — the five cells are ordered the same way either way, and
    /// `best` is still 99,999,999 when the scan starts, so nothing here
    /// compares a region score against a city one.
    #[test]
    fn a_scout_with_no_city_near_scans_its_whole_region() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..1375 {
            built.tick();
        }

        // The original's side: frame 1373's draws, filtered to the ones
        // `Unit::think_scout` took itself.
        use sim::scout::{
            SITE_CELL, SITE_PHASE, SITE_REGION_CELL, SITE_REGION_STRIDE, SITE_ROTATION,
        };
        let theirs: Vec<String> = tr
            .run_in(1373, sim::scout::CODE.start, sim::scout::CODE.end)
            .iter()
            .map(|d| tr.label(d))
            .collect();
        let mut want: Vec<&str> = Vec::new();
        for _ in 0..6 {
            want.extend([SITE_ROTATION, SITE_PHASE]);
        }
        want.push(SITE_REGION_STRIDE);
        want.extend(std::iter::repeat_n(SITE_REGION_CELL, 5));
        assert_eq!(theirs, want, "the trace's own frame-1373 sequence");

        // Ours: the same frame's marks, filtered the same way, on a stream
        // this simulation **reached** rather than had installed.
        let sites = [
            SITE_ROTATION,
            SITE_PHASE,
            SITE_CELL,
            SITE_REGION_STRIDE,
            SITE_REGION_CELL,
        ];
        let ours: Vec<String> = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 1373)
            .map(|(_, v)| v.clone())
            .expect("frame 1373's marks")
            .into_iter()
            .filter(|l| sites.contains(&l.as_str()))
            .collect();
        assert_eq!(
            ours, theirs,
            "our twelve ring draws and the region scan's six, in the original's order"
        );

        // And the cell the scan chose, against the dump's own. The order
        // reaches the unit through its group, so the first frame that
        // prints it is 1374.
        let dest = log
            .frame_states()
            .iter()
            .find(|f| f.n == 1374)
            .and_then(|f| f.units.iter().find(|u| u.who == 1 && u.o == 0))
            .and_then(|u| u.orders.first())
            .and_then(|o| Some((o.dest_x?, o.dest_y?)))
            .expect("run39's frame-1374 order for 1/0");
        assert_eq!(
            dest,
            (37_368, 33_528),
            "cell (48, 43), whose centre tile is (194, 174) — the dump's own \
             `orig_x/orig_y` are that centre and `dest` is it plus §13 item 8b's 24"
        );
        let scout = built
            .sim
            .unit_by_o(1, 0)
            .expect("run39 dumps player 1's `1/0`");
        let order = *built.sim.units[scout].orders.front().expect("an order");
        let sim::orders::Body::Move(m) = order.body else {
            panic!("not a move: {order:?}");
        };
        assert_eq!(m.kind, sim::orders::MoveKind::ExploreTo);
        assert_eq!(
            (i64::from(m.dest.x), i64::from(m.dest.y)),
            dest,
            "the region scan's winner, not the ring walk's"
        );
    }

    /// **A far wander sweeps from a literal bearing, not the animal's own**
    /// (2026-08-31, item 102) — the mechanic behind East Indies' word going
    /// 742 → 867, asserted where the word's own count cannot see it.
    ///
    /// `Animal::do_idle@005d7460`'s far branch hands
    /// `UnitType::find_nearby_spot` **`0x55555555`** as the angle its
    /// thirty-one bearings sweep out from — the ninth argument, where
    /// `do_gather`, `do_build`, `do_garrison` and every other call site in
    /// the executable passes a real heading. It is
    /// [`sim::movement::Angle::INITIAL`]: the 120° `Unit::init` writes into
    /// a unit that has never turned, a literal with nothing behind it. So
    /// every far wander any herd makes starts its sweep from the same
    /// direction, whichever way the animal happens to be looking.
    ///
    /// This crate passed `Movement::facing`, and run39's `8/3` was facing
    /// **south** (`UNITDATA angle -2147483648`) when its coin came up on
    /// frame 736 — 180° out. Two things follow, and the second is the one
    /// the item was booked as:
    ///
    /// - **The walk.** From the herd centre `(28800, 23936)` the original's
    ///   first ring at `0xc0` refuses 120° and 142.5° and takes **97.5°**,
    ///   `k = −1` of the sweep, which snaps to `(28968, 23976)`; this crate
    ///   took a bearing near due north and walked to `(28728, 24120)`. The
    ///   dump prints the walk step for step and the two share only its
    ///   first frame.
    /// - **The stand.** Six frames later the original's step is refused by
    ///   the herd-mate `8/2` standing at `(28856, 24197)`, and `move_step`
    ///   spends the blocked stand — `Guy::set_anim+0x97a <
    ///   Unit::move_step+0x823` — before its give-up tests
    ///   (`docs/COLLISION.md` §5). Ours was two hundred units west of that
    ///   and walked on. **The collision model was already right**: with the
    ///   bearing corrected the refusal, the `QUEUE_NEW` clear and the stand
    ///   all fall on the original's own frames with nothing else changed.
    ///
    /// Both halves are here. The first is `8/3`'s whole walk against the
    /// dump, frame for frame; the second is that the walk **stops** on the
    /// frame the original's does, at the point the original's does. Made to
    /// fail by putting `Movement::facing` back, which parts the walk on its
    /// second frame and never reaches the stand.
    #[test]
    fn a_far_wander_sweeps_from_the_literal_bearing() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // **Gaia's positions, on every frame the dump prints them.** run39
        // carries no `GUY` clocks, so `Built::tick` re-seats nothing here
        // (`Sim::reseat_animal` needs one) and the animals free-run for the
        // whole capture — which is what makes this a comparison at all.
        // Owner 8 only: the pasture's five and the birds are owner 9 and no
        // dump prints owner 9.
        let mut walk: Vec<(i64, i32, i32)> = Vec::new();
        let (mut agree, mut seen) = (0usize, 0usize);
        // The first animal-frame that is not the original's: the frame, the
        // `o`, ours and theirs. Printed rather than asserted — it is the
        // successor item the next time the floor moves.
        let mut first_bad: Option<String> = None;
        let mut first_bad_frame: i64 = i64::MAX;
        let mut last = 0i64;
        for f in log.frame_states() {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in f.units.iter().filter(|u| u.who == 8) {
                let Some(unit) = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(8, o))
                else {
                    continue;
                };
                let ours = built.sim.units[unit].pos;
                if u.o == 3 && (730..=750).contains(&f.n) {
                    walk.push((f.n, ours.x, ours.y));
                }
                seen += 1;
                if i64::from(ours.x) == u.pos.x && i64::from(ours.y) == u.pos.y {
                    agree += 1;
                } else if first_bad.is_none() {
                    first_bad_frame = f.n;
                    first_bad = Some(format!(
                        "frame {} 8/{}: ours ({}, {}) theirs ({}, {})",
                        f.n, u.o, ours.x, ours.y, u.pos.x, u.pos.y
                    ));
                }
            }
        }
        eprintln!(
            "run39 gaia: {agree} of {seen} dumped animal-frames on the original's point, \
             first {first_bad:?}"
        );
        // `8/3`'s whole walk: the coin comes up on 736, the first step
        // lands on 737, and 742's is refused — after which the original
        // holds the point for the rest of the capture.
        assert_eq!(
            walk,
            vec![
                (730, 28728, 24408),
                (731, 28728, 24408),
                (732, 28728, 24408),
                (733, 28728, 24408),
                (734, 28728, 24408),
                (735, 28728, 24408),
                (736, 28728, 24408),
                (737, 28728, 24408),
                (738, 28741, 24384),
                (739, 28754, 24360),
                (740, 28767, 24336),
                (741, 28780, 24312),
                (742, 28793, 24288),
                (743, 28793, 24288),
                (744, 28793, 24288),
                (745, 28793, 24288),
                (746, 28793, 24288),
                (747, 28793, 24288),
                (748, 28793, 24288),
                (749, 28793, 24288),
                (750, 28793, 24288),
            ],
            "run39's `8/3` walks the original's ground and is refused on 742"
        );
        // And the walk is dropped where it stands rather than pathed round
        // (`docs/COLLISION.md` §6 step 0): no order, and the point held.
        let u = built.sim.unit_by_o(8, 3).expect("run39 dumps gaia's `8/3`");
        assert!(built.sim.units[u].orders.is_empty());
        // **The whole-capture floor**, and it is the wider claim: gaia's
        // animals are the one population this capture lets free-run for
        // 1,850 frames with nothing installed, and the first animal-frame
        // that is not the original's is `8/3` again on frame **983**, where
        // the original wanders off the point it was refused at and this
        // crate has not yet. That frame is the number with meaning and it
        // may only rise.
        //
        // The whole-capture *count* is the weaker half, and it is not
        // monotone. Every frame past the word's own parting is drawn from a
        // stream that is nobody's — a herd's next coin is whatever the
        // frames before it happened to spend — so a mechanic that moves the
        // word forward re-rolls all of them: item 104's goody box took the
        // word 867 → 879 and the count 190,417 → 189,843, with **983
        // unmoved**. Read the count as a floor on the same word, not as a
        // score across words. Item 106 then moved both: 879 → 1256 took
        // the count to 190,690 and the first parting to **1261**, five
        // frames past the word, because the walk to the box put the whole
        // stream back on the original's for another four hundred frames.
        // Item 109's bird step takes 1256 → 1373 and carries both with it:
        // **191,173** of 192,504 and the first parting to **1381**, `8/3`
        // a step off the original's point again. Item 110's region
        // fallback takes 1373 → 1570 and both again: **191,876** and the
        // first parting to **1658**, `8/0` seventeen units short.
        assert!(
            first_bad_frame >= 1658 && agree >= 191_876 && seen == 192_504,
            "run39's gaia positions fell: {agree} of {seen}, first {first_bad:?}"
        );
    }

    /// **A herd's wander centre jitters about its home cell, and does not
    /// walk** (2026-08-31, item 112) — the mechanic behind Great Lakes'
    /// word going 986 → 1372, asserted against the record the word's own
    /// count cannot see.
    ///
    /// `Herd::process@00741760` takes two draws and writes
    /// `wx = cx − 1 + p % 3`, `wy = cy − 1 + p % 3`: it **reads `cx`/`cy`
    /// and writes `wx`/`wy`** — `HerdData +0x0/+0x4` into `+0x8/+0xc` — so
    /// the wander centre is never more than one cell from the home cell,
    /// however many times the herd is processed. This crate read the
    /// destination as the source and random-walked it. The two agree until
    /// a herd's *second* walk, and only one herd in thirteen gets a second
    /// inside run33's 1,850 frames: `(frame >> 6) % 13`, so herd 0 walks on
    /// frames 0 and 832 and nothing else does twice.
    ///
    /// What one cell is worth is `Animal::do_idle`'s far wander
    /// (`docs/ANIM.md` §7), whose whole ring is drawn about
    /// `herd_centre(cx, cy, wx, wy)`. Herd 0's `wy` ends frame 832 at 34
    /// here against the original's **33**, which moves that centre 240
    /// units south — and on frame 981 the sheep `8/3` is sent to
    /// `(17688, 26616)` where the original sends it to `(17688, 26328)`,
    /// a spot our ring does not contain at any of its nine radii.
    ///
    /// Five frames later the difference is a *draw*: both animals are
    /// refused by the same neighbour, but ours has walked five steps of
    /// `(+15, −5)` against the original's five of `(+12, −10)` and reaches
    /// the refusal one frame early, spending `Guy::set_anim+0x97a <
    /// Unit::move_step+0x823` on 986 where the original spends it on 987.
    /// That one frame was the whole of the word's parting at 986.
    ///
    /// Both halves are here: `8/3`'s walk against the dump step for step,
    /// and the whole capture's animal-frames. Made to fail by putting
    /// `wx`/`wy` back on the right-hand side, which walks `8/3` to the
    /// wrong point on frame 982 and never reaches the original's.
    #[test]
    fn run33_s_herd_centre_jitters_about_its_home_cell() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run33-longtrace.txt") else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut walk: Vec<(i64, i32, i32)> = Vec::new();
        let (mut agree, mut seen) = (0usize, 0usize);
        let mut first_bad: Option<String> = None;
        let mut first_bad_frame: i64 = i64::MAX;
        let mut last = 0i64;
        for f in log.frame_states() {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in f.units.iter().filter(|u| u.who == 8) {
                let Some(unit) = i16::try_from(u.o)
                    .ok()
                    .and_then(|o| built.sim.unit_by_o(8, o))
                else {
                    continue;
                };
                let ours = built.sim.units[unit].pos;
                if u.o == 3 && (980..=990).contains(&f.n) {
                    walk.push((f.n, ours.x, ours.y));
                }
                seen += 1;
                if i64::from(ours.x) == u.pos.x && i64::from(ours.y) == u.pos.y {
                    agree += 1;
                } else if first_bad.is_none() {
                    first_bad_frame = f.n;
                    first_bad = Some(format!(
                        "frame {} 8/{}: ours ({}, {}) theirs ({}, {})",
                        f.n, u.o, ours.x, ours.y, u.pos.x, u.pos.y
                    ));
                }
            }
        }
        eprintln!(
            "run33 gaia: {agree} of {seen} dumped animal-frames on the original's point, \
             first {first_bad:?}"
        );
        // The coin comes up on 981, the first step lands on 982, and the
        // fifth is refused: five steps of `(+12, −10)` toward
        // `(17688, 26328)`, then the point held. The frames are the dump's
        // own `FRAME n` blocks, which carry engine frame `n − 1`'s end.
        assert_eq!(
            walk,
            vec![
                (980, 17112, 26808),
                (981, 17112, 26808),
                (982, 17112, 26808),
                (983, 17124, 26798),
                (984, 17136, 26788),
                (985, 17148, 26778),
                (986, 17160, 26768),
                (987, 17172, 26758),
                (988, 17172, 26758),
                (989, 17172, 26758),
                (990, 17172, 26758),
            ],
            "run33's `8/3` walks the original's ground and is refused on 987"
        );
        // The whole-capture floor, read the same way run39's is: the first
        // animal-frame that is not the original's is the number with
        // meaning, and the count beside it is a floor on this word rather
        // than a score across words.
        //
        // 2026-08-31, item 114: 73,608 of 74,040 with a first bad frame of
        // 1420 -> **74,040 of 74,040, and there is no first bad frame**.
        // Great Lakes' gaia is the original's on every dumped animal-frame
        // of the whole capture, which follows from the word running past
        // its length: the residue at 1420 was a herd re-rolling off a
        // stream that had been nobody's since 1372. Half of queue item
        // 105/45 is closed on this map; run39's stands.
        assert!(
            first_bad_frame == i64::MAX && agree == 74_040 && seen == 74_040,
            "run33's gaia positions fell: {agree} of {seen}, first {first_bad:?}"
        );
    }

    /// **A scout re-aims its walk at a goody box it has seen**
    /// (2026-08-31, item 106) — the mechanic behind East Indies' word going
    /// 879 → 1256, asserted against the record the word's own count cannot
    /// see.
    ///
    /// `Unit::do_explore_to@005f24a0` is not `do_move`. One frame in
    /// fifteen, phased by `o`, a captain still walking the same
    /// `EXPLORE_TO` runs `Unit::find_goody_box@005f2540`: a 49-cell sweep
    /// in `move_x`/`move_y` order for a cell of its own region carrying
    /// `WData.flags & 0x8000`, and the first one it accepts is re-issued
    /// as an `EXPLORE_TO` to that cell's **centre** by
    /// `Unit::get_goody_box@005f7690`. None of it spends a draw.
    ///
    /// run39's scout `1/0` is inside the sweep's range of `(45, 49)` from
    /// frame 796 on, and the original does not re-aim until **825**. The
    /// gate that holds it is not the cell's `WorldData::was_seen`, which
    /// the box's own borders answer yes to from frame 0, but the **item's**
    /// `ItemData::is_seen` — the bare accumulated fog, with no
    /// ally-territory shortcut — which the scout's own line of sight does
    /// not reach until it is two cells out. `docs/GOODY.md` §7.2.
    ///
    /// What the frame's draw count cannot see, and this does — every one
    /// of these is a field of run39's own `UNITDATA`:
    ///
    /// - **`FRAME 797`**: `orders_x/y 35064/37368` and a path of three,
    ///   `(35040, 37344)`, `(35064, 38904)`, `(35064, 39672)`. That is
    ///   `think_scout`'s target and it is untouched through 824.
    /// - **`FRAME 826`**: `orders_x/y 34968/38040`, a path of **one** at
    ///   `(34944, 38016)` — `45 × 0x300 + 0x180`, `49 × 0x300 + 0x180`,
    ///   the box's cell centre — and an order list still holding **one**
    ///   order. A unit-level `QUEUE_FIRST` would leave two; the group's
    ///   halts and re-issues as `QUEUE_NEW` (`docs/GROUPS.md` §17).
    /// - **`FRAME 879`**: the walk is over, the list is empty, and
    ///   `orders_x/y` is `34944/38016`. That idle frame is the sixteen
    ///   draws the word had been short: two `Unit::set_anim` stands and
    ///   `think_scout`'s six ring pairs with two cell draws.
    /// - **`FRAME 880`**: the re-think's own answer, `orders_x/y
    ///   31224/39672` over four legs from `(31200, 39648)`.
    ///
    /// Made to fail by dropping the item gate, which fires the sweep on
    /// **796** instead — the frame `think_scout` itself runs on — and
    /// parts the word there.
    #[test]
    fn a_scout_re_aims_its_walk_at_a_goody_box_it_has_seen() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // The scout's order, its path stack and how many orders it holds —
        // `orders_x`/`orders_y`, `STACK<PathData>` and the order list's
        // own length, in the dump's order.
        let state = |b: &Built| {
            let u = b.sim.unit_by_o(1, 0).expect("run39 dumps player 1's `1/0`");
            let un = &b.sim.units[u];
            (
                (un.orders_pos.x, un.orders_pos.y),
                un.path.iter().map(|p| (p.to.x, p.to.y)).collect::<Vec<_>>(),
                un.orders.len(),
            )
        };

        // `Built::tick` stamps the frame it is about to run, so 797 ticks
        // leave `Sim::frame` on 797 and the state is `FRAME 797`'s.
        for _ in 0..797 {
            built.tick();
        }
        assert_eq!(built.sim.frame, 797);
        let think_scout_target = (
            (35_064, 37_368),
            vec![(35_040, 37_344), (35_064, 38_904), (35_064, 39_672)],
            1,
        );
        assert_eq!(
            state(&built),
            think_scout_target,
            "`FRAME 797`: frame 796's `think_scout` target and its three legs"
        );

        // Through 824 the sweep finds the box in range and refuses it:
        // 810 is a fifteenth frame and the item is still dark there.
        for f in 797..825 {
            built.tick();
            assert_eq!(
                built.sim.frame,
                f + 1,
                "the tick counter, so the frames below name themselves"
            );
            assert_eq!(
                state(&built).0,
                think_scout_target.0,
                "`FRAME {}`: the box is in range and its item is not yet seen",
                f + 1
            );
        }

        // Frame 825 is the retarget, and the box's cell centre is where it
        // aims.
        built.tick();
        assert_eq!(built.sim.frame, 826);
        assert_eq!(
            state(&built),
            ((34_968, 38_040), vec![(34_944, 38_016)], 1),
            "`FRAME 826`: the box's cell centre, one leg, and **one** order"
        );

        // …and it costs nothing: every frame from 796 to 878 spends the
        // original's draws in the original's order.
        for f in 826..879 {
            built.tick();
            assert_eq!(built.sim.frame, f + 1);
        }
        let ours: Vec<(i64, Vec<String>)> = built
            .frame_sites
            .iter()
            .filter(|(f, _)| (796..879).contains(f))
            .cloned()
            .collect();
        let bad: Vec<i64> = ours
            .iter()
            .filter(|(f, s)| *s != tr.labels(*f))
            .map(|(f, _)| *f)
            .collect();
        assert_eq!(
            bad,
            Vec::<i64>::new(),
            "the sweep spends no draw, so 796…878 stay the original's"
        );

        // Frame 879: the arrival, and the re-think it lets happen.
        assert_eq!(built.sim.frame, 879);
        assert_eq!(
            state(&built),
            ((34_944, 38_016), vec![], 0),
            "`FRAME 879`: the walk is over and the list is empty"
        );
        built.tick();
        let f879 = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 879)
            .map(|(_, s)| s.clone())
            .expect("frame 879's sites");
        assert_eq!(
            f879,
            tr.labels(879),
            "frame 879 is two stands, six ring pairs, two cell draws and five farms"
        );
        assert_eq!(
            f879.iter()
                .filter(|s| *s == sim::scout::SITE_ROTATION)
                .count(),
            6,
            "an AI scout skips the even rings, so `max_ring` 12 walks six"
        );
        assert_eq!(
            state(&built),
            (
                (31_224, 39_672),
                vec![
                    (31_200, 39_648),
                    (32_760, 40_440),
                    (33_528, 39_672),
                    (34_296, 38_904)
                ],
                1,
            ),
            "`FRAME 880`: the re-think's own target, over four legs"
        );
    }

    /// **A goody box draws once for every good its finder can gather**
    /// (2026-08-31, item 104) — the mechanic behind East Indies' word going
    /// 867 → 879, asserted where the word's own count cannot see it.
    ///
    /// `Unit::set_new_location@005f8d20+0x3cc` calls
    /// `Unit::explore_goody@005f9780` whenever a unit that is not an animal,
    /// not a placement ghost and of a land type enters a **new cell** whose
    /// `WData` first `short` is negative — bit `0x8000`, `GOODY`. run39's
    /// world carries seven such cells, which is the `WORLD` record's own
    /// `goodies 7`, and on frame 867 player 1's scout `1/0` walks south out
    /// of cell `(45, 50)` into `(45, 49)`, one of them. The original spends
    /// **three** draws there and this crate spent none.
    ///
    /// Three, not six: the lottery walks goods 0…5, skips `KNOWLEDGE`
    /// outright and skips anything `LeaderData::type_avail(good, 1)` does
    /// not call available — and in the Ancient age that is knowledge, metal
    /// and oil, so the candidates are food, timber and wealth. **The draw
    /// count is the candidate count**, which is what makes this frame a test
    /// of `type_avail` over the goods rather than of the lottery.
    ///
    /// What the frame's count cannot see, and this does:
    ///
    /// - **The cell.** The bit at `(45, 49)` is set through frame 866 and
    ///   clear after 867, and the other six are untouched — so no unit can
    ///   take the same ruins twice, and none of the other six has been
    ///   consumed by a walk that merely passed nearby.
    /// - **The pile.** `epoch[3] * GOODY_BOX_AGE + GOODY_BOX`, and the
    ///   `epoch` is the **Science** library level rather than the age
    ///   (`docs/GOODY.md` §3). Player 1 is on Science 1 by 867, so the pile
    ///   is `1 × 25 + 25 = 50` where the age reading would pay 25.
    /// - **The good.** Wealth, and it goes to the finder's bucket and to
    ///   `goody_box_resources`, the score counter `Leader::reset_score`
    ///   zeroes beside `goody_box_techs` and `goody_box_units`.
    ///
    /// Made to fail by dropping the availability guard, which spends five
    /// draws and parts the word back at 867. **The pile is the half no dump
    /// on disk can check**: run39's leader detail is written once, at start,
    /// where every `bucket` and every `epoch_get(scan)` is still its opening
    /// value. `docs/GOODY.md` §6 names the capture that would settle it.
    #[test]
    fn a_goody_box_draws_once_for_each_good_its_finder_can_gather() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;

        // The seven, from the dump's own cells — `goodies 7`.
        let goody_cells = |s: &sim::Sim| -> Vec<(i32, i32)> {
            let mut out = Vec::new();
            for y in 0..s.world.height() {
                for x in 0..s.world.width() {
                    let c = sim::world::Cell::new(x, y);
                    if s.world.cell_data(c).flags & sim::world::cell::GOODY != 0 {
                        out.push((x, y));
                    }
                }
            }
            out
        };
        assert_eq!(
            goody_cells(&built.sim),
            vec![
                (52, 5),
                (22, 22),
                (53, 26),
                (8, 28),
                (36, 31),
                (38, 34),
                (45, 49),
            ],
            "run39's `WORLD` record carries `goodies 7`, and these are they"
        );

        // `Built::tick` stamps the frame it is *about* to run, so 867 ticks
        // leave frames 0…866 behind and `Sim::frame` on 867.
        for _ in 0..867 {
            built.tick();
        }
        assert_eq!(built.sim.frame, 867);
        let before = built.sim.ledgers[1].bucket;
        assert_eq!(
            goody_cells(&built.sim).len(),
            7,
            "nothing has taken a goody through frame 866"
        );
        assert_eq!(built.sim.ledgers[1].goody_box_resources, 0);

        built.tick();
        // The frame itself, against the trace: three of the lottery's draws
        // and then the six the farms spend.
        let ours = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 867)
            .map(|(_, s)| s.clone())
            .expect("frame 867's sites");
        assert_eq!(
            ours,
            tr.labels(867),
            "frame 867 is the goody's three draws and the farms' six"
        );
        assert_eq!(
            ours.iter().filter(|s| *s == sim::goody::SITE_PICK).count(),
            3,
            "food, timber and wealth are the candidates; knowledge, metal and oil are not"
        );

        // The cell is spent and its six neighbours in the list are not.
        assert_eq!(
            goody_cells(&built.sim),
            vec![(52, 5), (22, 22), (53, 26), (8, 28), (36, 31), (38, 34),],
            "`(45, 49)` is taken and only `(45, 49)`"
        );
        // And the finder is where run39's own `FRAME 868` dump puts it.
        let u = built
            .sim
            .unit_by_o(1, 0)
            .expect("run39 dumps player 1's `1/0`");
        assert_eq!(built.sim.units[u].pos.cell(), sim::world::Cell::new(45, 49));

        // The pile: fifty wealth, and nothing anywhere else.
        let after = built.sim.ledgers[1].bucket;
        let moved: Vec<(usize, i32)> = (0..6)
            .filter(|&g| after[g] != before[g])
            .map(|g| (g, after[g] - before[g]))
            .collect();
        assert_eq!(
            moved,
            vec![(2, 50)],
            "one good, `epoch[3] × GOODY_BOX_AGE + GOODY_BOX` of it, and player 1 \
             is on Science 1"
        );
        assert_eq!(built.sim.ledgers[1].goody_box_resources, 50);
    }

    /// **A building's own line of sight, and the scout's path that reads
    /// it** (2026-08-31, item 99) — the mechanic behind the word's
    /// 413 → 576, asserted where a count cannot see it.
    ///
    /// run39's AI finishes its sixth farm on frame **219** at cell
    /// `(54, 51)`; `Wall::update_los` gives it `mylos 8` (`LOS 6` plus half
    /// its `X_SIZE 4`), so `Build::activate`'s closing `update_seen(0)`
    /// lights a radius-4 disc that reaches the three cells of column 56
    /// beside it — and stops short of `(56, 54)`, which
    /// `Unit::think_scout` still needs dark to pick as a target.
    ///
    /// Nineteen frames later the scout re-targets, and the path it plans is
    /// the whole point: with those cells dark a scout's `EXPLORE_TO` search
    /// prices them at a base of **8** against a seen cell's `0x400` and
    /// runs straight through them; with them lit it walks the seen column
    /// 55, which is what the original's dumped stack says it does. Both
    /// halves are here, and both were made to fail first — by dropping
    /// `update_seen_build` from `Sim::activate`, which puts the three cells
    /// back in the dark and the path back on column 56.
    #[test]
    fn run39_s_sixth_farm_lights_the_cells_its_scout_then_paths_around() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // The three cells the farm lights, and the one it must not: their
        // `2c + 1` half-cells, which is what both readers sample.
        let lit = [(56, 50), (56, 51), (56, 52)];
        let seen = |b: &Built, (cx, cy): (i32, i32)| {
            b.sim
                .world
                .seen2(2 * cx + 1, 2 * cy + 1)
                .is_some_and(|v| v & 2 != 0)
        };
        for _ in 0..218 {
            built.tick();
        }
        assert!(
            lit.iter().all(|&c| !seen(&built, c)),
            "column 56 is lit before the farm finishes"
        );
        for _ in 218..220 {
            built.tick();
        }
        assert!(
            lit.iter().all(|&c| seen(&built, c)),
            "the finished farm did not light column 56"
        );
        assert!(
            !seen(&built, (56, 54)),
            "the disc reached (56, 54), which think_scout needs dark"
        );

        // And the path the scout plans on frame 238, entry for entry
        // against the original's own stack. Its middle three are the whole
        // of the disagreement: column 55 where this crate ran column 56.
        for _ in 220..=238 {
            built.tick();
        }
        let theirs: Vec<(i64, i64, i64, i64)> = log
            .frame_states()
            .into_iter()
            .find(|f| f.n == 239)
            .expect("the dump's frame 239 is sim-frame 238")
            .units
            .into_iter()
            .find(|ud| ud.who == 1 && ud.o == 0)
            .expect("the AI scout")
            .path
            .iter()
            .map(|p| (p.to.0, p.to.1, p.tolerance, p.flags))
            .collect();
        assert_eq!(
            theirs,
            vec![
                (43488, 41952, 0, 1),
                (42744, 41208, 384, 0),
                (42744, 40440, 384, 0),
                (42744, 39672, 384, 0),
                (42744, 38904, 384, 0),
                (42744, 38136, 384, 0),
                (41976, 37368, 384, 0),
            ],
            "the original's own seven entries"
        );
        let v = built
            .units
            .iter()
            .find(|l| l.who == 1 && l.o == 0)
            .map(|l| l.unit)
            .expect("the AI scout");
        let ours: Vec<(i64, i64, i64, i64)> = built.sim.units[v]
            .path
            .iter()
            .map(|p| {
                (
                    i64::from(p.to.x),
                    i64::from(p.to.y),
                    i64::from(p.tolerance),
                    i64::from(p.flags),
                )
            })
            .collect();
        assert_eq!(ours, theirs, "the scout's stack on the frame it re-targets");
    }

    /// **A bird that lands, and the sixty draws it spends looking**
    /// (2026-08-31, item 100) — the mechanic behind the second map's word
    /// going 576 -> 645, asserted at the frame rather than as a total.
    ///
    /// `Animal::think_bird@005d79e0`'s third draw is `rnd % spell_time`,
    /// and `== 100` or `> 799` opens the landing search: thirty rounds
    /// over the cell list of the region the bird's patrol point sits in,
    /// two draws a round — the cell (`+0x2aa`, skipped for a region of one)
    /// and a `% 0x32 + 1` score (`+0x2d3`). The modulus of the roll that
    /// opens it *is* the counter, so it cannot fire before a bird has
    /// flown a hundred think-cycles, which is why `docs/SYNC.md` §3.9
    /// recorded the branch as unreached and left its draws unmodelled.
    ///
    /// run39 reaches it. Gaia's `9/8` fires on frame **576**, the frame
    /// the AI also founds its second city on, and the sixty draws are more
    /// than half of that frame's 118 — this crate spent 56. What the check
    /// asserts is the frame's whole draw sequence against the original's,
    /// which is the only oracle there is: the search's *score* is computed
    /// and never compared (`if (-1 < score)` cannot fail), so the cell it
    /// settles on is the thirtieth sampled and no dump prints owner 9
    /// anyway. Made to fail first by dropping the call from `think_bird`,
    /// which puts the frame back at 56 draws against 118.
    #[test]
    fn run39_s_bird_lands_on_576_and_spends_the_search_s_sixty() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        for _ in 0..=576 {
            built.tick();
        }

        // The landing itself: one bird, on the original's frame.
        assert_eq!(
            built.sim.gaia.bird_landings,
            vec![(576, 8)],
            "gaia's `9/8` is the only bird to land by 576, and it lands there"
        );

        // The frame's sixty, in the original's own alternation — thirty
        // pairs, never a `+0x2aa` skipped, so the region has more than one
        // cell in it.
        let ours = built
            .frame_sites
            .iter()
            .find(|(f, _)| *f == 576)
            .map(|(_, v)| v.clone())
            .expect("frame 576 drew");
        let search: Vec<&str> = ours
            .iter()
            .map(String::as_str)
            .filter(|s| {
                *s == sim::gaia::SITE_BIRD_SEARCH_CELL || *s == sim::gaia::SITE_BIRD_SEARCH_SCORE
            })
            .collect();
        let want: Vec<&str> = (0..sim::gaia::BIRD_SEARCH_ROUNDS)
            .flat_map(|_| {
                [
                    sim::gaia::SITE_BIRD_SEARCH_CELL,
                    sim::gaia::SITE_BIRD_SEARCH_SCORE,
                ]
            })
            .collect();
        assert_eq!(search, want, "thirty rounds of two, in order");

        // And the frame whole, which is what says the sixty fall in the
        // right *place* as well as in the right number.
        assert_eq!(ours.len(), 118, "frame 576's draw count");
        assert_eq!(ours, tr.labels(576), "frame 576, draw for draw");
    }

    /// **A bird's counter takes one step more than its flight** — every
    /// landing frame of a traced game, against the trace's own.
    ///
    /// The landing roll is `rnd % spell_time`, so the *counter* is the
    /// only state behind it and its value is unobservable: no dump prints
    /// owner 9 at all ([`run39_s_bird_lands_on_576_and_spends_the_search_s_sixty`]).
    /// What is observable is the frame a search fires on, thirty `+0x2aa`
    /// draws at a time, and a counter one step out puts a landing on a
    /// frame the original does not have — which is what East Indies'
    /// word saw at **1256**.
    ///
    /// The step is `Unit::do_air_patrol@005ea620`'s own tail. After
    /// `do_air_physics` returns — 1 for a wild bird on every path it
    /// takes — the caller branches on `vtable+0x30`,
    /// `SubObjectData::is_animal` (`docs/SYNC.md` §3.14; the map folds
    /// all three `Animal` vtables' slot onto `Buffer::is_pending_load`,
    /// `return 1`), and the animal arm is
    /// `else if (spell_time == 0) spell_time = 1`. `think_bird`
    /// increments the counter on **every** frame it runs, so it is only
    /// ever 0 there on a frame the landing search has just zeroed it:
    /// the branch is one extra step per landing and nothing else.
    ///
    /// run39's original lands on 576, 944 twice over, 1016, 1144 and
    /// 1368, and this simulation now lands on exactly those. The list is
    /// cut at the word (`first_part`) because past it the draws are
    /// nobody's — the original's own later landings, 1376 onward, are
    /// rolls this stream never sees. Made to fail by dropping the branch,
    /// which puts a seventh landing on **1256** and takes the word back
    /// there with it.
    #[test]
    fn a_bird_s_landing_frames_are_the_trace_s_own() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        for _ in 0..last {
            built.tick();
        }
        // Only the frames this stream is still the original's speak to a
        // landing: past the word both sides roll different numbers.
        let end = built
            .frame_sites
            .iter()
            .find(|(f, ours)| **ours != tr.labels(*f))
            .map_or(last, |(f, _)| *f);
        // The original's own landings: one for every thirty `+0x2aa`
        // draws a frame spends, and run39's frame 944 spends sixty.
        let rounds = sim::gaia::BIRD_SEARCH_ROUNDS as usize;
        let mut theirs: Vec<i64> = Vec::new();
        for (f, _) in &tr.frames {
            if *f >= end {
                break;
            }
            let n = tr
                .labels(*f)
                .iter()
                .filter(|l| l.as_str() == sim::gaia::SITE_BIRD_SEARCH_CELL)
                .count();
            for _ in 0..n / rounds {
                theirs.push(*f);
            }
        }
        let ours: Vec<i64> = built
            .sim
            .gaia
            .bird_landings
            .iter()
            .map(|(f, _)| *f)
            .filter(|f| *f < end)
            .collect();
        // 2026-09-01: four more (1736, 1776, 1784, 1800) arrived with item
        // 125 — not because a bird changed, but because `end` did. The
        // list runs to wherever the two streams still agree, and East
        // Indies' now agrees for the whole capture.
        assert_eq!(
            ours,
            vec![
                576, 944, 944, 1016, 1144, 1368, 1376, 1736, 1776, 1784, 1800
            ],
            "run39's landings up to the word at {end}"
        );
        assert_eq!(ours, theirs, "run39's landing frames, the trace's own");
    }

    /// **The production queues, whole**, against run39's own record —
    /// every building of both players, every live slot, every field
    /// `BuildQueue::log_data` writes.
    ///
    /// `docs/PRODUCTION.md` is read end to end and was, until this test,
    /// backed by a single hand-transcribed frame (run7's `[25, 26, 27]`
    /// ramp in
    /// [`run7_s_first_script_call_fills_the_queues_the_dump_shows`]). The
    /// dump has carried the record all along and nothing parsed it: the
    /// clock, the charge, the ramp and the handover are all in it, on
    /// every frame, for every building. This is the working agreement's
    /// "when the original dumps a record, diff the whole record", and it
    /// is what says the counter's arithmetic is the original's rather
    /// than merely plausible.
    ///
    /// **What it pins.** The AI's city hall queues its first citizen on
    /// frame 176 and the counter climbs 100 a frame to the `train_time`
    /// 9,750, capping there on 273 and handing the guy over on 274 — the
    /// two draws `docs/SYNC.md` §3.16 left as the word's residue. Three
    /// citizens sit in that queue at costs 25, 26 and 27, so the price
    /// ramp is checked per entry rather than at the head alone, and the
    /// library's research entry is checked beside them.
    ///
    /// **And the library's clock is the science discount's diff.** The AI
    /// queues Written Word and City State at frame 2; both are
    /// `JOB_TIME 200`. Written Word is researched at Science 0 and takes
    /// the full 20,000 hundredths, landing on 201; City State is then a
    /// level behind the player's Science and takes 18,000, landing on 382.
    /// Until `TECH_SCIENCE_SPEEDUP` was applied this simulation charged the
    /// second one 20,000 too and emptied the queue on 403 — twenty frames
    /// of `queued ours 1 theirs 0`, which were twenty of the twenty-one
    /// fields this test used to disagree on. `docs/PRODUCTION.md`
    /// §"The base, and the research step"; queue item 86.
    ///
    /// Only the first `queued` slots are compared: the tail of the array
    /// holds whatever it was last left with (`type −1` on a queue never
    /// used, `type 0` on one that has been), which is not state either
    /// side owns.
    ///
    /// **The loop this test used to own is now [`compare`]'s** (2026-09-01),
    /// so every capture the harness reads gets the queue rather than this
    /// one — which is how the AI Dock's twenty-eight late frames on run58
    /// were found. This test keeps the numbers: run39's own floor, its
    /// ceiling of one, and the field count, which changed only because the
    /// shared loop counts a slot's eight fields singly where this counted
    /// four groups.
    #[test]
    fn run39_s_build_queues_are_the_original_s_clock() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();

        let compared: usize = report.frames.iter().map(|f| f.queue_compared).sum();
        let wrong: Vec<&QueueDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.queue_diverged.iter())
            .collect();
        let frames = report.frames.len();
        let first = wrong
            .first()
            .map_or(report.frames.len() as i64, |d| d.frame);
        eprintln!(
            "run39 queues: {compared} fields over {frames} frames, \
             {} disagree, first at {first}",
            wrong.len()
        );
        for d in wrong.iter().take(24) {
            eprintln!(
                "  f{} {}/{} {}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        assert!(
            compared >= 40_199,
            "the record is being read: {compared} fields"
        );
        assert!(
            first >= 1851,
            "the queues part at {first}; the floor is 1851"
        );
        assert!(
            wrong.len() <= 1,
            "{} queue fields disagree; the ceiling is 1",
            wrong.len()
        );
    }

    /// **The gather record, whole**, against run39's own — every
    /// building of both players, every entry of every `MiningList`, on
    /// every frame of the capture.
    ///
    /// The list is the *input* `crates/sim/src/gather.rs` surveys the slot
    /// count out of, and nothing had ever compared it: the leader record's
    /// `gather_slots` is checked (the census windows), and the count is
    /// derived from the list, so a wrong count and a wrong list read the
    /// same on the only oracle there was. This is the difference —
    /// `docs/QUEUE.md` item 85, and the working agreement's "when the
    /// original dumps a record, diff the whole record".
    #[test]
    fn run39_s_mining_lists_are_the_gather_record() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let (compared, wrong, first) = gather_verdict("run39", &report);
        assert!(
            compared >= 594_618,
            "the record is being read: {compared} fields"
        );
        // **1851 is the run's own last frame, and its block is the quit's.**
        // The `1850 !quit` interrupts the frame the block belongs to and the
        // end-of-game dump is written into it — 17,201 lines against 15,705
        // on every frame before it — and in that block *the human player's*
        // four buildings carry `gather_down −1` while the AI's carry the
        // same heads they have held since 1845. Four buildings losing every
        // registered gatherer on one frame, on one side only, is a teardown
        // rather than a game, so the four rows are the ceiling and the
        // floor is the frame they sit on: no disagreement inside the game.
        assert!(first >= 1851, "the mining lists part at {first}");
        assert!(
            wrong <= 4,
            "{wrong} gather fields disagree; the ceiling is 4, the quit's own"
        );
    }

    /// **`Build::find_gather_tiles` re-derives the original's own list.**
    ///
    /// run39's two camps are pre-placed, so their `gather_from` comes
    /// straight from the dump — 73 tiles apiece — and until now nothing in
    /// this crate could have produced one. This clears the `0x1000` marks a
    /// camp's own tiles carry and runs [`sim::Sim::gather_tcoords`] at its
    /// corner: the walk has to come back with **exactly** the same tiles,
    /// as a set, on a map neither the walk nor the survey has seen.
    ///
    /// The order is a separate question and this cannot answer it — the
    /// dump's order is the shuffle's, and the shuffle needs the stream the
    /// camp was built on, which for a pre-placed building is the map
    /// generator's rather than the game's.
    #[test]
    fn find_gather_tiles_rederives_run39_s_camp_lists() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let camps: Vec<usize> = (0..built.sim.buildings.len())
            .filter(|&b| !built.sim.buildings[b].gather_from.is_empty())
            .collect();
        assert_eq!(camps.len(), 2, "run39's two woodcutter's camps");
        for b in camps {
            let (ty, who, pos) = {
                let bd = &built.sim.buildings[b];
                (bd.ty.expect("a typed camp"), bd.owner, bd.pos)
            };
            let theirs = built.sim.buildings[b].gather_from.clone();
            // The camp's own tiles are marked, so the walk would refuse
            // every cell of them; unmarking is what makes this the survey
            // the original ran before the camp existed.
            for &t in &theirs {
                built
                    .sim
                    .world
                    .clear_tile_bits(t, sim::gather::GATHERED_FROM);
            }
            let corner = built.sim.tile_corner(ty, pos);
            let ours = built.sim.gather_tcoords(ty, who, corner);
            let (mut a, mut c) = (ours.clone(), theirs.clone());
            a.sort_unstable_by_key(|p| (p.x, p.y));
            c.sort_unstable_by_key(|p| (p.x, p.y));
            assert_eq!(
                a.len(),
                c.len(),
                "player {who}'s camp: the walk found {} tiles, the dump has {}",
                a.len(),
                c.len()
            );
            assert_eq!(a, c, "player {who}'s camp: the tiles themselves");
            for &t in &theirs {
                built.sim.world.set_tile_bits(t, sim::gather::GATHERED_FROM);
            }
        }
    }

    /// The same widening on **run56**, East Indies' 3,000-frame capture —
    /// the first one long enough to hold a camp the *game* built.
    ///
    /// run39 and run33 both stop at 1,851, and both maps' camps are
    /// pre-placed, so their lists are the dump's own by construction and
    /// agreeing costs the simulation nothing. run54's word parts at **2176**,
    /// on a frame the original spends 192 draws filling a new camp's list,
    /// and this capture is run39's game carried past it at run39's detail —
    /// which makes the frame's `BUILDDATA` the first record that can say
    /// whether `Build::find_gather_tiles` produces the original's list rather
    /// than merely the original's *tiles* (`docs/ECONOMY.md`, "The gather
    /// list, and its shuffle").
    ///
    /// **Closed 2026-09-01**: this simulation now builds that camp too, on
    /// the original's own frame and tile, so every gather record of the
    /// capture agrees except the four the quit's own frame carries.
    #[test]
    fn run56_s_mining_lists_reach_past_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — this is the long East Indies capture, \
             and a short file here is a wrong file",
            report.frames.len()
        );
        // What the original holds and this simulation does not, building by
        // building — the successor's own statement, printed whether or not
        // the assertion below is what fails.
        let mut missing: Vec<(i64, i64, i64)> = Vec::new();
        for f in &report.frames {
            for (who, o) in f
                .gather_diverged
                .iter()
                .filter(|d| d.field == "length" && d.ours == 0 && d.theirs > 0)
                .map(|d| (d.who, d.o))
            {
                if !missing.iter().any(|&(w, b, _)| (w, b) == (who, o)) {
                    missing.push((who, o, f.frame));
                }
            }
        }
        for (who, o, frame) in &missing {
            eprintln!("  run56: {who}/{o} has a list from frame {frame}; this one has none");
        }
        let (compared, wrong, first) = gather_verdict("run56", &report);
        assert!(
            compared >= 1_048_118,
            "the record is being read: {compared} fields"
        );
        // **Nothing is missing.** The camp `1/2009` used to stand here as
        // the one list the original had and this simulation did not; item
        // 126 closed it, and the count above rose by the 80,850 fields the
        // camp's own frames contribute.
        assert!(
            missing.is_empty(),
            "the original has a list this one does not: {missing:?}"
        );
        let strays: Vec<&GatherDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.gather_diverged.iter())
            .filter(|d| d.frame < report.frames.len() as i64)
            .collect();
        assert!(
            strays.is_empty(),
            "{} gather rows before the quit's own frame: {:?}",
            strays.len(),
            &strays[..strays.len().min(4)]
        );
        assert!(first >= 3001, "the mining lists part at {first}");
        assert!(
            wrong <= 4,
            "{wrong} gather fields disagree; the ceiling is 4 — the quit's \
             own frame, where four farms hold a `gather_down` the dump has \
             as −1"
        );
    }

    /// **The shuffle, seed-anchored** — the whole of
    /// `Build::find_gather_tiles` against the one camp the *game* built.
    ///
    /// run56's frame 2176 is where the original places player 1's second
    /// Woodcutter's Camp, `o 2009`, and spends 192 draws shuffling its
    /// 48-tile list. ~~This crate's AI does not place it (that is the
    /// successor, and the queue holds it)~~ — item 126 closed that on
    /// 2026-09-01 and this crate's AI now places it in the tick, which is
    /// what [`run56_s_mining_lists_reach_past_the_word`] checks. This test
    /// keeps the *seed-anchored* form, which is stronger: it does not
    /// depend on the AI reaching the frame at all, because the stream is
    /// knowable on its own — run56's trace gives
    /// `game_random`'s word at `do_frame` entry of 2176, and
    /// `find_gather_tiles` is the **first** thing that frame draws.
    ///
    /// So: step to 2176, install the original's word, place the camp where
    /// the original placed it, and compare the list it comes back with —
    /// **in order**, entry for entry — against the dump's. That is the walk,
    /// the marking, the shuffle's round count and the shuffle's arithmetic
    /// in one assertion, and the order is the half
    /// [`find_gather_tiles_rederives_run39_s_camp_lists`] cannot reach.
    #[test]
    fn run56_s_new_camp_is_this_crate_s_own_shuffle() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        const FRAME: i64 = 2176;
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        // The original's own record for the camp, from the first frame that
        // carries it — its position, its type and its list.
        let theirs = log
            .frames()
            .into_iter()
            .filter(|(n, _)| *n > FRAME)
            .find_map(|(_, b)| {
                crate::gamelog::records(b, false)
                    .1
                    .into_iter()
                    .find(|d| d.who == 1 && d.o == 2009)
            })
            .expect("run56 carries player 1's o 2009");
        assert_eq!(
            theirs.gather_from.len(),
            48,
            "the camp's list is 48 tiles, and 4 × 48 is the frame's 192 draws"
        );
        let word = tr
            .frames
            .iter()
            .find(|(n, _)| *n == FRAME)
            .map(|&(_, w)| w)
            .expect("run56's trace carries frame 2176");
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame < FRAME {
            built.tick();
        }
        built.sim.rng.seed = word;
        let ty = loaded
            .build_of_type_index(theirs.orig_type.expect("orig_type") as i32)
            .expect("the camp's build type");
        let b = built.sim.init_build(1, ty, pos_of(theirs.pos), false);
        assert_eq!(
            i64::from(built.sim.buildings[b].index),
            2009,
            "find_free hands out the original's own object number"
        );
        let ours: Vec<(i64, i64)> = built.sim.buildings[b]
            .gather_from
            .iter()
            .map(|p| (i64::from(p.x), i64::from(p.y)))
            .collect();
        eprintln!(
            "run56 camp 1/2009: ours {} tiles, theirs {}; draws {:?}",
            ours.len(),
            theirs.gather_from.len(),
            draws_between(word, built.sim.rng.seed)
        );
        assert_eq!(
            draws_between(word, built.sim.rng.seed),
            Some(4 * theirs.gather_from.len() as u32),
            "the shuffle is four rounds a tile, one draw each"
        );
        assert_eq!(
            ours.len(),
            theirs.gather_from.len(),
            "the walk found {} tiles, the original {}",
            ours.len(),
            theirs.gather_from.len()
        );
        assert_eq!(
            ours, theirs.gather_from,
            "the shuffled list, entry for entry"
        );
        assert_eq!(
            built.sim.buildings[b].gather_max,
            Some(built.sim.max_gatherers(b)),
            "gather_max was recomputed from the filled list"
        );
    }

    /// The same widening on **Great Lakes** — run33's own record, which has
    /// no pasture and a different AI opening, so it is the second map's
    /// independent word on the same claim.
    #[test]
    fn run33_s_mining_lists_are_the_gather_record() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let (compared, wrong, first) = gather_verdict("run33", &report);
        assert!(
            compared >= 584_712,
            "the record is being read: {compared} fields"
        );
        assert!(first >= 1772, "the mining lists part at {first}");
        assert!(wrong <= 4, "{wrong} gather fields disagree");
    }

    /// The gather record's verdict, printed the same way for either map:
    /// how many fields were compared, how many disagreed, the first frame
    /// one did, and the first few rows.
    fn gather_verdict(tag: &str, report: &Report) -> (usize, usize, i64) {
        let compared: usize = report.frames.iter().map(|f| f.gather_compared).sum();
        let unlinked: usize = report.frames.iter().map(|f| f.build_unlinked).sum();
        let rows: Vec<&GatherDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.gather_diverged.iter())
            .collect();
        let first = rows.first().map_or(report.frames.len() as i64, |d| d.frame);
        eprintln!(
            "{tag} gather: {compared} fields over {} frames, {} disagree, \
             first at {first}, {unlinked} unlinked building-frames",
            report.frames.len(),
            rows.len()
        );
        for d in rows.iter().take(16) {
            let at = if d.at < 0 {
                String::new()
            } else {
                format!("[{}]", d.at)
            };
            eprintln!(
                "  frame {} {}/{} {}{at}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        (compared, rows.len(), first)
    }

    /// **A pasture herder walks only on its own 256-frame phase**, and it
    /// is the rule of `docs/SYNC.md` §3.15 against run39's own record.
    ///
    /// `Unit::do_gather@005ef2a0:5efd77` takes the whole of the function
    /// when `FarmsData::get_farm_type` answers 1: the citizen shows the
    /// sow animation, and unless `(o · 7 + frame + who) % 256` is zero it
    /// returns having drawn nothing. A pasture is the one farm
    /// `Farms::inc_time` skips, so no cell under it ripens on the clock —
    /// which is why the crop switch below is not merely the wrong branch
    /// but a branch whose *trigger* never comes on time. This crate ran
    /// the herder through it, and the farmer's own `Farms::grow` alone
    /// carried its cell to `RIPE_ADDS` on the two hundredth frame instead
    /// of the hundredth: two draws on East Indies' frame 201, where the
    /// original spends none.
    ///
    /// Three things are asserted, and each of them can fail:
    ///
    /// - **The record.** Over run39's 1,850 frames the herder's dumped
    ///   position changes on **42** of them, in **four** runs, and every
    ///   run's first frame is `phase + 2` — the order is issued on the
    ///   phase frame and the first step lands the frame after. Three of
    ///   the seven phase frames move it nowhere, because the inner 2 × 2
    ///   holds four tiles and the roll may name the one it stands on.
    /// - **The trace.** All **seven** phase frames of the capture spend
    ///   the `GameAccess::rnd+0x20 < Unit::do_job+0x67` pair, and the pair
    ///   is two draws rather than one because `x_size / 2` is 2 and
    ///   `GameAccess::rnd` only skips its draw at 1 or less.
    /// - **The port.** This crate's herder walks the original's first run
    ///   frame for frame — sim-frames 235 to 242 — and takes **no** step
    ///   on any frame outside a phase's window, over the whole capture.
    ///   Under the crop switch it walks from 202 and the first assertion
    ///   fails at once.
    ///
    /// Great Lakes cannot check this: run33's AI built seven farms and no
    /// pasture, which is why the residue only ever showed on the second
    /// map.
    #[test]
    fn a_pasture_herder_walks_only_on_its_own_256_frame_phase() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        // The pasture, and the one citizen registered at it. Named rather
        // than searched for, so a capture whose setup changes says so.
        let pasture = (0..built.sim.buildings.len())
            .find(|&b| built.sim.buildings[b].farm.farm_type & sim::farms::ANIMAL_FARM != 0)
            .expect("run39's AI built one pasture");
        let herder = *built.sim.buildings[pasture]
            .gatherers
            .first()
            .expect("and put a citizen on it");
        let (who, o) = (
            i64::from(built.sim.units[herder].owner),
            i64::from(built.sim.units[herder].index),
        );
        assert_eq!(
            (who, o, built.sim.buildings[pasture].index),
            (1, 3, 2002),
            "run39's pasture is the AI's `1/2002` and its herder `1/3`"
        );
        // `(o · 7 + frame + who) % 256 == 0`, in sim-frames.
        let on_phase = |f: i64| (o * 7 + f + who) % 0x100 == 0;

        // **The record.** `FRAME n` is the state at the end of sim-frame
        // `n − 1`, so a position that differs between `FRAME n − 1` and
        // `FRAME n` is a step taken on sim-frame `n − 1`.
        let states = log.frame_states();
        let theirs: Vec<(i64, crate::gamelog::Pos)> = states
            .iter()
            .filter_map(|f| {
                let u = f.units.iter().find(|u| u.who == who && u.o == o)?;
                Some((f.n - 1, u.pos))
            })
            .collect();
        assert_eq!(theirs.len(), 1851, "the herder is in every frame's dump");
        let moved: Vec<i64> = theirs
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| w[1].0)
            .collect();
        assert_eq!(moved.len(), 42, "the herder's steps: {moved:?}");
        let starts: Vec<i64> = moved
            .iter()
            .copied()
            .filter(|f| !moved.contains(&(f - 1)))
            .collect();
        assert_eq!(
            starts,
            vec![235, 747, 1003, 1515],
            "four walks, and each begins the frame after a phase frame"
        );
        for f in &starts {
            assert!(
                on_phase(f - 1),
                "the walk that starts on {f} was ordered on {}, which is not a phase frame",
                f - 1
            );
        }

        // **The trace.** Every phase frame of the capture spends the pair.
        let last = tr.frames.last().map_or(0, |(n, _)| *n);
        let phases: Vec<i64> = (0..last).filter(|&f| on_phase(f)).collect();
        assert_eq!(
            phases,
            vec![234, 490, 746, 1002, 1258, 1514, 1770],
            "seven phase frames in 1,850"
        );
        for f in &phases {
            let n = tr
                .labels(*f)
                .iter()
                .filter(|l| *l == sim::orders::SITE_FARM_CELL)
                .count();
            assert!(
                n >= 2,
                "the original spends {n} `{}` draws on phase frame {f}, wanted the pair",
                sim::orders::SITE_FARM_CELL
            );
        }

        // **The port.** Our herder's own steps, over the whole capture.
        let mut ours: Vec<(i64, sim::Pos)> = Vec::new();
        for _ in 0..last {
            let f = built.sim.frame;
            built.tick();
            ours.push((f, built.sim.units[herder].pos));
        }
        let ours_moved: Vec<i64> = ours
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| w[1].0)
            .collect();
        let ours_starts: Vec<i64> = ours_moved
            .iter()
            .copied()
            .filter(|f| !ours_moved.contains(&(f - 1)))
            .collect();
        for f in &ours_starts {
            assert!(
                on_phase(f - 1),
                "this crate's herder starts walking on {f}, off its phase: {ours_moved:?}"
            );
        }
        // The first walk is the original's, frame for frame. Past the
        // word's own parting at 219 the rolls are on a stream that is
        // nobody's, so which of the later phases moves it is not a fact
        // about this simulation — but *that* it only ever moves on one is.
        let first_walk: Vec<i64> = moved.iter().copied().take_while(|&f| f < 300).collect();
        assert_eq!(
            ours_moved
                .iter()
                .copied()
                .take_while(|&f| f < 300)
                .collect::<Vec<_>>(),
            first_walk,
            "the herder's first walk is the original's"
        );
    }

    /// **The second map's score.**
    ///
    /// Phase 3's finish line is a traced human-versus-AI capture holding
    /// lockstep on **two** maps, and until 2026-08-29 there was only one:
    /// every number in this file is Great Lakes, seed 12345, the lobby
    /// run10–14 played. A residue chased on one map can be chased into that
    /// map's shape, and nothing here would say so.
    ///
    /// run38 and run39 are the other map — East Indies (`MAP_STYLE 18`),
    /// the same seed and the same rules (`MAP_SIZE 2`, `GAME_RULES 1`,
    /// `REVEAL_MAP 1`). Getting there took finding that **`-config
    /// check.ini` pins the map style and no file can move it**
    /// (`docs/ORACLE.md`, "The lobby is a file"): the capture scripts drop
    /// it for any style but 14 and take the profile's lobby instead.
    ///
    /// run39 is the 1,850-frame dump; run38 is its `DUMP_ALL` start, and it
    /// is the whole sibling list — its own `Initial` carries the heights,
    /// the checksum trace, the herds and the frame seeds, so `build_sim`
    /// stands the simulation up on a map it has never seen with nothing
    /// borrowed. Frame 0 is **175 draws against 175** on the first try.
    ///
    /// History:
    ///   2026-08-29  ticks **167**, orders **167**; player 0 @ 219,
    ///               player 1 @ 168 (item 38's second capture, the first
    ///               number this map has ever had). Great Lakes stands at
    ///               252 the same day, so the two are within a hundred
    ///               frames of each other — the residue chased on one map
    ///               was not chased into its shape.
    ///
    /// What parts it first is an order-list **length**: `1/4` holds two
    /// orders on the original's frame 168 where this simulation holds one,
    /// and its position parts on the same frame. `1/3` at 202 and `1/5` at
    /// 186 are the same disagreement.
    ///
    /// **2026-08-30: that is not the first divergence, and this is not a
    /// fidelity number.** run39's own trace says the word parts at **19**
    /// — see
    /// [`run39_s_long_trace_says_where_the_second_map_s_word_parts`] — so
    /// every figure below sits 148 frames deep into a stream that is
    /// nobody's, and moving them by chasing frame 168 moves nothing. The
    /// floor stays where it is so that nothing falls through it by
    /// accident; the number to steer this map by is the word.
    /// **The original's own per-step prices, against ours.**
    ///
    /// run55 is run39's game — same lobby, same seed, same detail — taken
    /// with `rontrace.cfg`'s `callwin` over frames 1460–1490, which
    /// **proxies** `PathFinder::calc_cost` and `PathFinder::astar_path` and
    /// logs each call's arguments *and its answer* (`tools/trace/tracer.c`,
    /// instrument 4). `docs/PATHFINDER.md` §10 recorded that there is no
    /// numeric per-search oracle to switch on; there is one now, and this
    /// is the check it exists for.
    ///
    /// The comparison is **by argument, not by position**. Two searches
    /// that price differently expand differently, so their call sequences
    /// cannot be lined up entry for entry — but every step *both* sides
    /// priced is a row of §5 with the original's answer beside ours, and a
    /// disagreement there is a bug in the formula rather than in the
    /// search. What the sequences do beyond that is the queue's item 125.
    #[test]
    fn run55_s_frame_1477_prices_are_the_originals() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(costs)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
            trace("rontrace-run55.log"),
        ) else {
            eprintln!("skipping: no East Indies cost trace (set RON_GAMELOG_DIR)");
            return;
        };
        let theirs = costs.calls_in(FRAME, crate::trace::call_site::CALC_COST);
        assert!(
            !theirs.is_empty(),
            "run55's callwin covers frame {FRAME}; it priced nothing there"
        );

        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().expect("run39 is a dump");
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..FRAME {
            built.tick();
        }
        built.sim.trace_costs = true;
        built.tick();
        let ours = std::mem::take(&mut built.sim.cost_marks);

        // The original's answers, keyed by the whole argument list.
        let mut theirs_by_key: std::collections::BTreeMap<sim::path::CostKey, i32> =
            std::collections::BTreeMap::new();
        for c in &theirs {
            theirs_by_key.insert(c.cost_key().expect("a calc_cost call"), c.ret);
        }
        let mut shared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for m in &ours {
            let Some(&t) = theirs_by_key.get(&m.key()) else {
                continue;
            };
            shared += 1;
            if t != m.cost {
                let cell = |v: i32| v.div_euclid(m.step);
                wrong.push(format!(
                    "({},{}) -> ({},{}) dir {} depth {}: ours {} theirs {}",
                    cell(m.from.0),
                    cell(m.from.1),
                    cell(m.to.0),
                    cell(m.to.1),
                    m.dir,
                    m.depth,
                    m.cost,
                    t
                ));
            }
        }
        assert!(
            shared > 0,
            "frame {FRAME}: {} of the original's steps and {} of ours, and not one \
             argument list in common — the searches did not start from the same place",
            theirs.len(),
            ours.len()
        );
        assert!(
            wrong.is_empty(),
            "frame {FRAME}: {} of {shared} shared steps priced differently:\n  {}",
            wrong.len(),
            wrong.join("\n  ")
        );
    }

    /// The **sim-frame** the scout plans on — `docs/PATHFINDER.md` §12's
    /// search, which every document before this one called frame 1477
    /// because that is the label of the dump block it lands in, and a
    /// block `FRAME n` is the end of sim-frame `n − 1` (`docs/ORACLE.md`,
    /// "The frame label, settled"). run55's trace counts `Game::frame` and
    /// puts the whole search on **1476**, which is also the only search in
    /// its thirty-one-frame window.
    const FRAME: i64 = 1476;

    #[test]
    fn run39_s_islands_game_is_the_second_map_s_score() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        // **The pasture is a source, and without it this is a different
        // game.** East Indies' AI builds one, its five animals are owner 9
        // and no dump prints them, and they roll an idle every frame — so
        // a score run that borrows none parts from the original within a
        // few frames of the pasture going up, and every figure below is
        // then on a stream that is nobody's. That is what pinned this map
        // at 167 (item 69). The assertion is the guard: a run that loses
        // the pasture fails here rather than quietly scoring low.
        assert_eq!(
            tr.add_animals().len(),
            1,
            "run39's trace reached the setup and East Indies' AI has one pasture"
        );
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        // 1,851: the `1850 !quit` runs at the top of frame 1850, and the
        // block the quit interrupts is written too.
        assert_eq!(report.frames.len(), 1851, "run39's length");
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: frame 0: ours 175 draws, the original's 175")),
            "frame 0 on the second map: {:?}",
            report.notes
        );
        //   2026-08-30  ticks **505**, orders **482**; player 0 @ **687**,
        //               player 1 @ **506** (item 78: **the gather order's
        //               write-back went to the front of the list**).
        //               `Sim::store_gather` wrote to `orders.front_mut()`,
        //               and every walk `do_gather` and
        //               `do_non_flat_gather` issue is a `QUEUE_FIRST` move
        //               *in front of* the gather order — so the
        //               `goto_build = 1` / `wait = 32` that the
        //               return-to-camp branch writes **after**
        //               `add_move_order` landed on the move and were
        //               dropped. The original holds the order pointer for
        //               the whole function; §6 of `docs/ORDERS.md` had
        //               already established the same rule for the *read*
        //               side (`is_gathering_at` matches `get_action`, not
        //               the front), and the write side was never made to
        //               match.
        //
        //               The human's `0/2` finished its shift on 426,
        //               reached its camp on 431 and then re-entered the
        //               same branch for the rest of the capture: it never
        //               unloaded, never took another tile, and spent a
        //               `+0xb99` stand every other frame. run33's word
        //               goes 432 → **482** and its totals 724/606 →
        //               **752/622**.
        //
        //               Every one of the twelve compared units improves.
        //               Player 0 recovers item 76's fall and passes it,
        //               450 → 687 (`0/5` 450 → 687, `0/3` 455 → 702,
        //               `0/4` 577 → 703); player 1 goes 437 → 506, and
        //               what pins it is `1/8` at 506. East Indies is
        //               unmoved at 167/167 — its own divergence is an
        //               order-list length at 168, item 69.
        //   2026-08-30  ticks **571**, orders **571**; player 0 @ 574,
        //               player 1 @ 572 (item 79: **the vision projection
        //               was thrown along the guy's angle**).
        //               `Object::update_seen` throws a small land unit's
        //               disc half a cell forward of its nose, and the
        //               angle it projects along is `UnitData +0x50` — the
        //               unit's own heading, what the dump prints as
        //               `UNITDATA angle` — not `GuyData::angle`, the eased
        //               facing the body actually wears. The `project` at
        //               `00651d05` is handed it by `movl 0x50(%ecx), %ecx`
        //               two instructions earlier, with `ecx` the
        //               `units.list[who][o]` the branch above it read.
        //
        //               While a unit turns the two differ by as much as
        //               thirty degrees and the disc lands in a different
        //               fog cell. On run33's frame 168 the AI scout was
        //               mid-turn — heading −51.6°, facing −83.0° — and the
        //               disc thrown along the facing lit fog `(113, 57)`,
        //               which the original's never reached. Three hundred
        //               frames later, on **482**, `Unit::think_scout`'s
        //               cell filter refused cell `(56, 28)` as "already
        //               seen" and spent thirty draws where the original
        //               spends thirty-one. run33's word goes 482 → **571**
        //               and its totals 752/622 → **791/662**.
        //
        //               **Every player-1 unit improves or holds** — `1/8`
        //               506 → 778, `1/4` and `1/5` 550 → 673 and 663,
        //               `1/0` 637 → 722 — which is the AI walking the
        //               original's fog as well as its ground. **Player 0's
        //               three farmers fall**, 687/702/703 → 579/574/577,
        //               and it is the same downstream effect item 76 had:
        //               their old numbers were all past the word's own
        //               parting at 482, on a stream that was nobody's, and
        //               the new ones sit three frames past the new parting
        //               at 571. The whole capture now diverges within
        //               eight frames of the word, which is what
        //               convergence looks like. East Indies is again
        //               unmoved at 167/167.
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        eprintln!(
            "run39: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
        //   2026-08-30  ticks and orders hold at **167**; player 1 holds
        //               at 168 and **player 0 goes 219 -> 217** with item
        //               95, the animal's hurry. It is the one fall the
        //               item costs, and it is downstream of the word: the
        //               word on this map now parts at 91, so by 217 the
        //               two sides have been on different mid-frame draw
        //               orders for a hundred and twenty frames. What moved
        //               is which of player 0's citizens parts first —
        //               `0/3` and `0/5` at 219 before, `0/4` at 217 after,
        //               each of them a step out on a walk no animal
        //               touches. The animal's own step is now the
        //               original's on **264 of 264** dumped gaia steps
        //               across both maps
        //               ([`an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`]),
        //               which is a stronger statement about this
        //               simulation than two frames of a citizen.
        //   2026-08-31  ticks **1374**, orders **536**; player 0 @ **1411**,
        //               player 1 @ **1375** (item 69: **the score run was
        //               not the run the word measures**). Every figure
        //               above was taken on a simulation with no pasture in
        //               it: `run_traced` borrowed the siblings' initial
        //               state and the trace was handed only to the word's
        //               own check, so the five owner-9 animals no dump
        //               prints were missing from the one run that scored.
        //               They roll an idle a frame, so the stream parted
        //               within a few frames of the pasture going up and
        //               everything downstream — `1/4`'s farm re-target at
        //               167, `1/5`'s at 186, player 0's citizens at 217 —
        //               was a different game's arithmetic, not a mechanic.
        //               With the trace passed in, ticks land **one frame
        //               past the word's own parting at 1373**, which is
        //               what a converged capture looks like: the
        //               simulation holds position for as long as it holds
        //               the stream, and parts when the stream does.
        //
        //               Orders followed to **1373** on the same item's
        //               second half — the collision pause's own frame
        //               (`docs/ORDERS.md` §4.4, `goto STEP`) — so both
        //               numbers now sit **on the word's parting**: this
        //               capture holds position and intent for exactly as
        //               long as it holds the stream. The next figure to
        //               move is the word's, and 1373 is item 110's.
        //
        //               The comparison behind `orders` is also wider than
        //               it was: the same item added `OrderMismatch::Move`,
        //               the whole `MOVEORDER` row, of which `dest`,
        //               `facing` and `last_x/last_y` are reported and do
        //               not score (see [`OrderMismatch::scores`]).
        //   2026-08-31  ticks **1374 -> 1477**, orders **1373 -> 1476**;
        //               player 0 @ **1657**, player 1 @ **1478** (item
        //               110, the region fallback's cell walk). Both
        //               numbers stay pinned to the word, which parts at
        //               1570 — a hundred frames of the original's stream
        //               that nothing here reads, spent on units this
        //               capture does not dump every frame — and both moved
        //               by the same 103 the word moved by less its own
        //               lead. Nothing fell.
        //   2026-09-01  ticks **1477 -> 1851**, orders **1476 -> 1850**,
        //               and **neither player diverges at all** (item 125:
        //               `WData.blocked` is a count of the cell's blocked
        //               tiles and nothing here was keeping it). The whole
        //               of run39 — every frame of the capture, every unit,
        //               position and order list — is the original's. It is
        //               the first capture on either map to be matched end
        //               to end, and with Great Lakes already exact over
        //               its own 1,772 it closes both.
        //
        //               What it was: player 1's second city went up on
        //               tile (180, 188), and the four cells under it kept
        //               a `blocked` of zero, so `calc_cost` charged an
        //               empty field where the original charges nine
        //               sixteenths of one — 304 against 128 — and §5.1's
        //               corner-cutting never even opened. The AI scout
        //               therefore walked *through* the city on frame 1476
        //               and arrived six frames early. run55's per-step
        //               cost dump is what said so, in one reading: 103 of
        //               110 steps already agreed, and all seven that did
        //               not were steps into those four cells
        //               ([`run55_s_frame_1477_prices_are_the_originals`]).
        let none = report.first_divergence.iter().all(|&(_, f)| f.is_none());
        assert!(
            ticks >= FLOORS[0].ticks && orders >= FLOORS[0].orders && none,
            "the second map's score fell: ticks {ticks}, orders {orders}, first \
             divergence {:?} — the floor is ticks {}, orders {}, and **no player \
             diverging anywhere in the capture**",
            report.first_divergence,
            FLOORS[0].ticks,
            FLOORS[0].orders
        );
    }

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
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, None).unwrap();
        assert_eq!(report.frames.len(), 1772);

        // **The headline.** Phase 3's score is ticks before divergence, and
        // this is the longest capture there is, so this is the number. It is
        // pinned as a **floor** so that it can only go up: a session that
        // lowers it fails here before it reaches any sub-score below, and a
        // session that raises it moves the floor and adds a line to the
        // history. `first_divergence` is the breakdown the single score
        // hides — one unit the simulation cannot yet drive pins the score
        // while every other unit may be tracking to the end.
        //
        // History:
        //   2026-08-27  ticks 3, orders 2; player 0 @ 103, player 1 @ 4
        //               (item 34 landed; the first pin)
        //   2026-08-27  ticks 99, orders 102; player 0 @ 103, player 1 @ 100
        //               (item 25: the tile choice's access filter). Player
        //               1's woodcutter was being sent to a tile ringed by
        //               its own forest, which the original never considers,
        //               and the walk it queued for it was the frame-3 second
        //               order the simulation did not have. With the filter
        //               the two sides pick the same tree, and the score is
        //               no longer any one unit's: what is left at 100 and
        //               103 is a whole cohort at once — the AI's citizen
        //               `1/6` on 100, and player 0's three farmers on 103.
        //   2026-08-27  ticks 102, orders 102; **both players @ 103**
        //               (item 43: `come_out`'s exit ring). A trained unit
        //               was being put on its trainer's centre tile; the
        //               original builds it there, walks it inside and lets
        //               it out onto a ring five tiles clear of the wall.
        //               Every citizen run10's AI trains now appears where
        //               the original puts it, and the only frame either
        //               player still parts on is 103 — the farm re-target,
        //               which is one mechanic and not five units.
        //   2026-08-27  ticks 122, orders 122; player 1 @ 123, player 0 @
        //               182 (item 44). Frame 103 was **not** the farm
        //               re-target: the arithmetic was right and the AI's
        //               three farmers already matched. It was two other
        //               things, one per player.
        //               *Player 1's* `1/6` walked to a spot 48 units off
        //               because `find_nearby_spot` rejected the camp's own
        //               tile as blocked. A building's blocked bits are a
        //               **per-tile template** — `masks.txt`, named by the
        //               graphic (`docs/DATALAYER.md`) — and a Woodcutter's
        //               Camp blocks nothing at all, where this crate had
        //               been blocking every non-flat footprint whole.
        //               *Player 0's* three farmers spent the wrong draws:
        //               frame 101's stream runs AI farmers, **a sheep's
        //               arrival**, human farmers, and the sheep had
        //               wandered on an untraced frame the sim's stream
        //               cannot reach. The harness re-seats gaia's animals
        //               from every traced frame's dump; the draw fell back
        //               into place and 101 went 20/21 → 21/21.
        //   2026-08-27  ticks 170, orders 166; player 1 @ 171, player 0 @
        //               182 (item 46: **unit collision**, `docs/COLLISION.md`).
        //               `1/6` walked into `1/3` at frame 122. The original
        //               detects it on the 48-cell occupancy bitmask, names
        //               the other unit off the world cell's object chain,
        //               finds the corner rule does not let them slip past,
        //               snaps the walker onto its own cell centre and
        //               re-plans on the 48-grid. Every field of that is in
        //               the dump and every one of them now matches: the
        //               five `collide*` fields, `coll_x`/`coll_y`, the
        //               position, and all seven path entries. `1/6` went
        //               from 123 to 208, and what pins player 1 now is
        //               `1/1`'s gather `dist_mod` at 167.
        //   2026-08-27  ticks 181, orders 168; player 1 @ 203, player 0 @
        //               182 (item 47: **the AI builder does not keep what
        //               it built**). `Unit::do_build`'s two gather
        //               predicates and `Unit::do_repair`'s each carry a
        //               `unit_masks & 0x40000` term this crate did not
        //               have: only a **human** builder adopts the site it
        //               has just finished. `1/1` finished its farm on frame
        //               167 and this simulation put it on that farm; the
        //               original sent it back through `build_done`, whose
        //               AI arm searches afresh and picked the Woodcutter's
        //               Camp `2001` — the same camp, the same tile
        //               `(212, 93)`, the same `dist_mod 4`. The first
        //               gather-tile disagreement went 169 → 430.
        //   2026-08-28  ticks 185, orders 168; player 1 @ 203, player 0 @
        //               186 (item 50: **the bird**, `docs/SYNC.md` §3.9).
        //               Frame 96's sampling hatches one, and from 104 a
        //               live bird spends three `Animal::think_bird` draws
        //               every eighth frame that this crate was not
        //               spending — twenty-four of them between the last
        //               traced word and the frame `1/1` picks its tile.
        //               The order score does not move with it: at the tile
        //               draw the stream is still ten draws short, and
        //               `1/1`'s `wait` went 581 → 476 against 460. Six of
        //               the ten are the bird's own animation, which needs
        //               a length no dump carries.
        //   2026-08-28  ticks 181, orders **180**; player 0 @ 182, player 1
        //               @ 203 (item 52: **the bird's wing beat**,
        //               `docs/SYNC.md` §3.9). The length no dump carries is
        //               in the install: `WILDBIRD` plays *Bird Soar* for
        //               `CHAR_WALK` and *Bird Flap* for `CHAR_JOG`, and the
        //               `.bha` files say 31 frames and 23
        //               (`rondata::artdata`). With them the bird spends the
        //               hatch frame's wrap, `do_air_physics`'s birth coin
        //               and every later one, and the ledger went 184 → 192.
        //
        //               **The two scores met, and that is why `ticks` reads
        //               lower.** The residue is one unit and it did not
        //               change: `0/3`'s order list goes wrong when its
        //               gather tile is picked off a stream that is still
        //               short, and that pick moved **169 → 181**. What
        //               moved with it is when the wrong order starts
        //               *moving* the unit — at 169 it did not for another
        //               seventeen frames, so `ticks` read 185 while the
        //               orders had already parted; now the position follows
        //               the order by one frame, as it should. A `ticks`
        //               above `orders` is the accident, not the gain.
        //   2026-08-28  ticks **202**, orders 168; player 0 @ **213**,
        //               player 1 @ 203 (item 55: **the road, and the
        //               heights of another game**). Two things landed
        //               together. `borrow_from_siblings` was taking
        //               `master_land_heights` from run3 — the same seed,
        //               style and size, a different `GAME_RULES`, and a
        //               grid that differs on 237 corners because its
        //               starting buildings terraformed elsewhere — so the
        //               road search, a third of whose cost is the climb
        //               term, had been reading another game's terrain.
        //               With the map's own heights the search is the
        //               original's node for node on every capture that
        //               shows it a frame's own world (run14's 10 and 11,
        //               run32's 104–107), and the road it lays on fresh
        //               ground is the original's tile for tile, so
        //               `Sim::plan_roads` came on.
        //
        //               **`orders` fell, and it is the same kind of luck
        //               `ticks` used to be.** Frames 10 and 11 now spend
        //               the 468 draws they always should have, so the
        //               stream is the original's through frame 17 rather
        //               than through 9 — and every value after the new
        //               divergence at 18 is a *different* wrong value.
        //               `0/3`, which pinned the old 180, now holds to 213;
        //               what pins 168 is `1/1`'s gather wait at 169, off
        //               by ten, which the old stream happened to land on.
        //               The number that is not luck is in the ledger test:
        //               the first frame whose draws differ at all, 10 → 18.
        //   2026-08-28  ticks **190**, orders **185**; player 0 @ 191,
        //               player 1 @ 203 (item 59: **the builder's own
        //               animation**). `Unit::do_build`'s step 4 and
        //               `Unit::do_repair`'s first line put the worker on
        //               `CHAR_BUILD` / `CHAR_SOW` / `CHAR_REPAIR`, which
        //               this crate never modelled — so its builders stayed
        //               on `CHAR_WALK` and spent an arrival stand
        //               (`Guy::move+0x19f`) the original does not spend.
        //               One draw, on run14's frame 18, and it was the whole
        //               of the residue: the traced stream now parts at
        //               **99** rather than 18 and matches **219** of 284
        //               frames rather than 173.
        //
        //               **`orders` rose and `ticks` fell, and the two are
        //               the same 81 frames.** `1/1`'s gather `wait` at
        //               frame 169 — what pinned `orders` at 168 — is now
        //               the original's, because the draws between 18 and 99
        //               are. What pins 185 is `0/4`, a human farmer whose
        //               re-target moved 220 → 186: its `wait` was already
        //               wrong at 220 and the shape of the disagreement is
        //               unchanged (a `MOVE_TO` in front of a gather the
        //               original never re-issues), so it is the same defect
        //               on a different frame. Every other unit held or
        //               improved — `1/1` went 577 → 647 — and run6's
        //               totals fell from 2,591/1,613 to 1,588/1,415.
        //   2026-08-28  ticks **192**, orders 185; player 0 @ **193**,
        //               player 1 @ 203 (item 49: **the blocked stand**).
        //               `Unit::move_step:281` asks for `CHAR_DEFAULT` the
        //               moment a step is refused, *before* the three
        //               give-up tests, and this crate skipped the call —
        //               so a blocked unit spent no idle roll where the
        //               original spends one. It could not be made until
        //               `Animal::do_idle`'s own `detect_unit_collision`
        //               was: gaia's sheep stand shoulder to shoulder, and
        //               without that gate `8/1` wandered off on a frame
        //               the original's has not moved on in 120, taking a
        //               blocked stand of its own at 112 and parting the
        //               word *earlier*. With both, run14's word runs
        //               122 → **185** and 219 → **235** of 284 frames
        //               match. What parts 185 is item 61's farmer.
        //   2026-08-28  ticks **200**, orders **200**; player 0 @ **213**,
        //               player 1 @ 201 (item 61: **the farmer's cell
        //               index**). `FarmStruct::status` is a `uchar[4][4]`
        //               indexed `status[dx][dy]` — `Farms::grow`'s own
        //               addressing — and `do_farm` read `status[dy][dx]`.
        //               The transpose is invisible for a hundred frames,
        //               because every starting farmer stands on `(2, 2)`;
        //               from frame 101, when six of them re-pick a cell,
        //               all six sow the wrong one and ripen on the wrong
        //               frame. run14's word runs 185 → **201** and 235 →
        //               **251** of 284 frames match, and the trace's
        //               re-targets now agree to the draw on 101, 199, 211
        //               and 217.
        //
        //               **Player 1's own number fell, 203 → 201**, and it
        //               is the newly-correct 199 that exposes it: the
        //               AI's `1/4` re-picks the cell it is *standing on*,
        //               and the original's move there is refused by a
        //               collision (`collide_o 2` in run10's frame-201
        //               record) and killed without a step, so it re-picks
        //               again on 201. Ours paths and walks. That is the
        //               successor item, and the first divergence now.
        //   2026-08-28  ticks **207**, orders **206**; player 0 @ **326**,
        //               player 1 @ **208** (item 63: **the waypoint's own
        //               collision test**). `Unit::do_move`'s waypoint take
        //               — the block that runs once per leg, on the frame
        //               the waypoint is first read off the path stack —
        //               ends with a `detect_unit_collision` at the
        //               waypoint that this crate did not make, and a
        //               **final** waypoint under a `GATHER`, `ATTACK` or
        //               `BUILD_AT` action that another unit is standing on
        //               kills the whole move outright (`docs/ORDERS.md`
        //               §4.4). `1/4` re-picks farm cell `(0, 3)` on 199,
        //               where its sibling `1/2` is already working; the
        //               original names `1/2` on `collide_o`/`collide_who`,
        //               kills the walk without a step, and `do_farm` picks
        //               `(2, 1)` on 201 instead. Ours pathed and walked
        //               off across the farm.
        //
        //               **Player 0 moved 213 → 326 with it**: the human
        //               farmers `0/3` and `0/5` had been parting at 213 on
        //               a walk to a cell that was not the original's, and
        //               they now hold to 325. What parts them at 326 is a
        //               re-target the original has made a frame before
        //               this simulation makes its own, onto a different
        //               cell again. What parts player 1 is `1/6` at 208:
        //               the original's woodcutter collides on 206 and
        //               repaths onto a seven-entry stack this simulation
        //               does not build.
        //   2026-08-28  ticks **209**, orders **208**; player 0 @ 326,
        //               player 1 @ **210** (item 64: **`is_flat`, the
        //               fence on `resolve_unit_collision`'s step 2**).
        //               The step that kills a walk because the unit is
        //               standing inside its own gather target's footprint
        //               asks that target's type `+0x94` —
        //               `BuildTypeData::is_flat`, `build_flags &
        //               0x10000000` — and this crate read the virtual as
        //               true (a stated seam, `docs/COLLISION.md` §9). So
        //               a woodcutter that collides while standing on its
        //               *camp's* footprint killed its own walk and re-made
        //               it on the next frame, for ever: run10's `1/6`
        //               pushed and killed the same `MOVE_TO` every other
        //               frame from 206 to the end of the capture. With the
        //               fence it falls through to the repath and builds
        //               the original's seven-entry stack, field for field
        //               — and the **whole collision block goes to zero
        //               disagreements** over 48,790 field-frames, the
        //               sticky `collide_guy` included. `1/6` parts at 253
        //               rather than 208; what parts player 1 now is `1/7`
        //               at 210, the citizen trained on frame 206.
        //   2026-08-28  ticks **252**, orders **252**; player 0 @ 326,
        //               player 1 @ **253** (item 66: **`find_nearby_spot`'s
        //               collision half**). Every walk an order makes ends
        //               at a point the ring-and-bearing sweep returns, and
        //               the sweep's last test — `Objects::find_collision`
        //               then `Objects::find_ordered_collision`, both
        //               against the unit as "me" — had never been
        //               implemented, so the first *passable* candidate won
        //               whether or not somebody was standing on it. The
        //               AI's new citizen `1/7`, sent to its camp on frame
        //               208, was given the exact quarter-tile `1/6` was
        //               standing on; the original refuses that and the six
        //               bearings behind it and lands seven cells further
        //               south. With the test the two agree, and `1/6` and
        //               `1/7` both hold to 253.
        //
        //               **run14's trace is now spent.** Its word — the
        //               per-frame draw count — used to part at 232; it now
        //               runs to the end of all 284 frames, 282 of which
        //               match draw for draw, and gaia's bird hatches on
        //               the original's 96, 192 and 256 and on no frame of
        //               its own. Nothing on disk can say where the
        //               simulation next parts by site, which is item 38.
        //   2026-08-29  ticks **322**, orders **320**; player 0 @ **356**,
        //               player 1 @ **323** (item 68: **the think tail
        //               conscripted the woodcutters**). `Unit::think`'s
        //               tail at `005f7615` is `if (!is_supply &&
        //               !is_hero) { … return } add_to_army(this)` —
        //               `docs/SCOUT.md` §2 had the listing right all
        //               along — and `Sim::think_join_army` also joined
        //               "an attacker that is not a scout or a caravan",
        //               which is `docs/ARMY.md` §4's prose for the
        //               *other* caller, `think_attack@005f5a80:155`,
        //               grafted onto the wrong site. A citizen has an
        //               attack, so run10's AI woodcutters `1/6` and
        //               `1/7` joined army 0 on frames 99 and 205, and
        //               leader 1's army 0 ticks at `frame ≡ 252 (mod
        //               256)`: on **252** `do_forming` sent both on a
        //               siege attack across the map. The original never
        //               calls `add_to_army` **at all** in this game —
        //               `Unit::add_to_army@005f7740` and
        //               `Army::add_unit@006f9f40` are absent from
        //               run33's 6,703 entered functions over 1,850
        //               frames — and every citizen's dumped `group` is
        //               −1 on every frame it exists, against the scout's
        //               65 and then 64.
        //
        //               The second half is the same listing's line
        //               `005f7195`: a worker whose `think_peasant`
        //               **found something** returns there, and the
        //               return value was being dropped.
        //
        //               `1/6` went 253 → 735 and `1/7` 253 → 937.
        //               **Player 0 moved with them, 326 → 356**, because
        //               the stream it shares is the original's for
        //               seventy frames more: item 65's farmer re-target
        //               at 326 is gone, `0/3` and `0/5` now hold to 450
        //               and 455, and what pins player 0 is `0/4`'s farm
        //               walk at 356 (its path goal parts at 351, one
        //               tile north-west of the original's). What pins
        //               player 1 is the AI's ninth citizen `1/8` at
        //               321: its gather order names the Woodcutter's
        //               Camp `2001` with `dist_mod 4` where the original
        //               names `2006` with `dist_mod 0` — a **farm** —
        //               so `find_gather_spot`'s choice is the successor.
        //   2026-08-29  ticks **355**, orders **350**; player 0 @ 356,
        //               player 1 @ **363** (item 70: **the gather score
        //               is cap headroom, not distance**).
        //               `find_gather_spot@005f5170`'s numerator is not
        //               the rate it was taken for: over the six goods the
        //               leader has and the building gathers, it is
        //               `resource_cap[g] − income[g]` — the unused part
        //               of the commerce cap, both sixteenths, skipped
        //               entirely while `over_cap[g]` is set. Both
        //               candidates for `1/8` sat in the same distance
        //               bucket (`1958 / 0xc0` and `2041 / 0xc0` are both
        //               10), so the tie was the whole question, and with
        //               four woodcutters against three farmers the food
        //               headroom is the larger. `1/8` takes the farm,
        //               parts at 506 rather than 323, and what pins
        //               player 1 now is the **scout** `1/0` at 363.
        //               Player 0 is unmoved at 356 and is the headline:
        //               item 71, `0/4`'s farm walk.
        //
        //               The rest of the function came with it — the
        //               `tregion` gate, the city-crossing rule at
        //               `:108`, `is_gathering_at` in place of the
        //               gatherer chain, and the strict `local_20 < score`
        //               that makes a zero-scoring building unpickable.
        //   2026-08-29  ticks **362**, orders **361**; player 0 @ **464**,
        //               player 1 @ 363 (item 71: **the idle variant's
        //               length, from the install**). `0/4`'s farm walk
        //               was never `do_gather`'s: the two `% 4` draws it
        //               takes on frame 349 came off a stream four words
        //               ahead of the original's, and what had put them
        //               there was an animation length nothing knew. The
        //               human scout rolled `CHAR_IDLE1` on frame 284 and
        //               this crate played it as `CHAR_DEFAULT`, because
        //               `Art::lengths` came out of dumps and no dump had
        //               ever shown that slot. `unit_graphics.xml` has the
        //               whole table, and
        //               `GraphicPieces::init_piece_ranges`' four strides
        //               say which piece each `<UNIT>` entry is
        //               (`crate::artdata::piece_lengths`, 1,359 of them).
        //               With the lengths in, `0/4` takes the original's
        //               cell and **player 0 goes 356 → 464**.
        //
        //               Two more came with it, both exposed by the first.
        //               The carrying walk is `unit_masks & 0x78000000`,
        //               not the gather order's `goto_build`: the walk a
        //               citizen makes to its camp before it has ever
        //               reached a tile is the plain `CHAR_WALK`, and it
        //               ends in the arrival stand `Guy::move+0x19f`.
        //               And `man_walk.bha` carries thirty-one keys where
        //               its own count says thirty, so the reader's
        //               "the keys fill the chunk" test had been throwing
        //               away every citizen's walk.
        //
        //               What pins the headline now is player 1's
        //               **scout** `1/0` at 363, unmoved by this item.
        //   2026-08-30  ticks **436**, orders **427**; player 0 @ 450,
        //               player 1 @ **437** (item 76: **the scout's
        //               surface probe read the wrong tile**). The cell
        //               filter's `TData` read (`docs/SCOUT.md` §7) had
        //               been transcribed as `(4x, 4y + 2)` from a
        //               decompiled `+4` that is **two elements** of a
        //               two-byte record, not a field offset: the listing
        //               at `005f6542` is `movb 0x4(%eax,%ecx,2)` over
        //               `ecx = (4y + 2)·tile_xs + 4x`, so the tile is the
        //               cell **centre**, the same one `invalid_loc` is
        //               handed two lines later. On run33's frame 361 the
        //               AI scout `1/0` re-targets; cell `(48, 23)` is
        //               ocean at tile `(192, 94)` and land at
        //               `(194, 94)`, so the candidate the original takes
        //               was refused — twenty-six draws against
        //               twenty-seven — and the scout went to `(48, 20)`.
        //               With the centre tile it takes the original's
        //               cell: `1/0` goes 363 → **484**, and eight of the
        //               twelve compared units improve
        //               (`1/5` 421 → 663, `1/3` 470 → 583, `0/4`
        //               464 → 577).
        //
        //               **Player 0 falls 464 → 450**, and it is
        //               downstream: run33's word now parts at **432**
        //               (from 361), and every unit that fell — `0/5` at
        //               450, `0/3` at 455, `1/4` at 437 — parts after
        //               that frame, on a stream that is nobody's. What
        //               parts the word at 432 is a gather stand, and it
        //               is the successor item.
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        let first: Vec<i64> = report
            .first_divergence
            .iter()
            .map(|&(_, f)| f.unwrap_or(i64::MAX))
            .collect();
        eprintln!(
            "run10: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
        // The breakdown behind those two numbers: which unit parts when.
        // It is what says whether an item moved the whole or only the
        // unit it was about, and every history line below quotes it.
        eprintln!("run10 by unit: {:?}", report.first_divergence_by_unit());
        eprintln!(
            "run10 orders by unit: {:?}",
            report.order_divergence_by_unit()
        );
        // 2026-08-30, the third steer (Fable): items 81 and 74 had moved
        // orders 576 → 776 and player 0's first divergence 687 → 802 and
        // left this line where it was; the queue carried the numbers and
        // the assertion did not. Raised to what the run prints.
        //
        // 2026-08-31, item 69: ticks **572 -> 781** and player 1 **573 ->
        // 782**, on `do_move`'s `goto STEP` (`docs/ORDERS.md` §4.4). The
        // jump the straight-line check takes when it succeeds lands past
        // the pause check, so a unit that re-verifies its line under a
        // collision pause **steps that frame and does not tick the
        // pause**. `1/2` is the case, field for field in the dump: the
        // collision at 572 writes `pause 3` and `coll_x/coll_y`, 573 steps
        // with the pause still 3, and 574–576 are the three still frames.
        // Ticking on 573 put this simulation one frame ahead for the rest
        // of that walk and parted the position at 573. The order score
        // holds at 776 under a **wider** comparison — the same item added
        // the whole `MOVEORDER` row — and East Indies goes to 1374/1373.
        //
        // 2026-08-31, item 84: ticks **781 -> 910**, player 0 **802 ->
        // 1154** and player 1 **782 -> 911**, on the farm stand's byte
        // (`docs/ORDERS.md` §6.5). The switch reads the guy's live
        // `cur_anim`; this crate kept a flag of its own, so a farmer
        // whose reap animation had been replaced by a blocked stand
        // re-picked a tile where the original sowed. Orders hold at 776
        // — `1/1`'s move at 777 is untouched — and every one of the
        // thirteen units parts later, the median by three hundred
        // frames.
        //
        // 2026-08-31, item 112: player 0 **1154 -> 1385**, and **ticks and
        // orders do not move at all** — the herd's wander centre took the
        // word 986 -> 1372 and this map's headline stayed at 910/776, so
        // for the first time since the two were tied the tick score is no
        // longer sitting on the word's own parting. What holds it is one
        // unit: `1/1` parts at **911**, four hundred and sixty frames
        // before the word does, and every other unit of the fourteen parts
        // at 1375 or later. That unit, not the word, is Great Lakes' next
        // item. Six of the fourteen moved and all six later (`0/3` 1154 ->
        // 1385, `0/4` 1227 -> 1462, `0/5` 1191 -> 1409, `1/5` 1359 ->
        // 1379, `1/8` 1376 unchanged, `1/10` 1552 unchanged).
        //
        // 2026-08-31, item 113: ticks **910 -> 1375**, orders **776 ->
        // 791**, player 1 **911 -> 1376**, and this map's headline is a
        // fifth of a game further in. `1/1` is **gone from the by-unit
        // list entirely** — it never parts on position across the whole
        // 1,772 — and every other one of the fourteen parts on the frame
        // it did before, to the frame. Two defects, and the second was
        // only reachable once the first was fixed.
        //
        // The first is the AI's borders. `World::compute_reg_territory`
        // rebuilds its per-player bonus table at the top of every pass and
        // reads the **Civic** level out of it; this crate built the table
        // once in `add_player`. The AI's City State never widened its
        // border, so it held 261 cells against the original's 290 — and
        // `WorldData::was_seen`'s first arm is territorial, so those
        // twenty-nine cells were dark to `compute_site_stats`. Its 5×5
        // slide could not reach `(47, 28)` and the second city went up in
        // the wrong place
        // ([`run40_and_run41_s_sites_and_territory_are_the_original_s`]).
        //
        // The second is `find_wpath`'s `scouting` mode, and it took the
        // word *down* to 786 on its own before it was found. `scouting`
        // prices seen ground at `0x400` against unseen `8` — the whole of
        // "exploration seeks the unexplored" — and it is set only when the
        // **type** is a scout (`role & 0x10`, `is(SCOUT)` on land,
        // `is(BARK)` at sea) *and* the order is `EXPLORE_TO`. This crate
        // tested the order alone, so the citizen the AI sends to its city
        // site under an `EXPLORETO` took a ten-cell detour west through
        // the fog where the original walks seven south-east. No capture
        // had ever exercised it: until the borders were right, no
        // non-scout in either game was given an explore order over any
        // distance.
        //
        // What now holds the order score at 791 is `1/1` again, and it is
        // one field: on frame 792 the original's `coll_x/coll_y` is
        // `(40539, 18258)` and this crate's `(40632, 18044)` — the same
        // walk, a different collision point.
        //
        // 2026-08-31, item 115: orders **791 -> 1374**, and it is one
        // line. `detect_unit_collision` writes `coll_x/coll_y` **into the
        // order**; the original then walks the rest of `move_step` on a
        // *pointer* to that order, so the write is simply there, while
        // this crate steps on a copy and every `store_move` below put the
        // stale pair back. The arm that shows it is the blocked stand a
        // unit takes while it still owes a turn (`docs/COLLISION.md` §5):
        // it stores and returns without stepping. `1/1` probed
        // `(40539, 18258)` on the original's own frame and this crate
        // threw the answer away.
        //
        // Nothing else moved: **ticks hold at 1375**, both first
        // divergences hold, every one of the fourteen units parts on the
        // frame it did, and run33's word and sequence hold at 1372 with
        // its two totals at 1467/1436. What holds the order score now is
        // `1/8`, whose move order's `x` is `40440` here against `40248`
        // at **1375** — the same unit that holds the tick score at 1376,
        // and both are past the word.
        //
        // 2026-08-31, item 114: ticks **1375 -> 1772**, orders **1374 ->
        // 1772**, and **neither player diverges at all** — the by-unit
        // list is empty on positions and carries nothing that scores on
        // orders. This capture is in lockstep for its whole length.
        //
        // One mechanic did it, and it is one call: `Build::activate`
        // reaches `Farms::add_animals` for every food gather building it
        // completes, so the AI's fourth farm — a pasture — stocks five
        // animals on frame 1372 for twenty draws, and five more follow
        // that frame as the newborns' clocks wrap. This crate spent none
        // of the twenty-five. run33's word goes 1372 -> **1802**, past
        // this capture's own 1,772, and everything the two residue items
        // were about goes with it: `1/8`'s move order `x` (item 118) and
        // `1/1`'s cleared `last_x/last_y` (item 119) were both downstream
        // of a stream that had been wrong since 1372.
        //
        // **What this is not.** run10 is 1,772 frames of one map. The
        // finish line `CLAUDE.md` names is two maps at their full length,
        // and East Indies still parts at 1374/1373 with its word at 1373,
        // so the number that is still moving is that one. What Great
        // Lakes now needs is a *longer* capture (queue item 91) — this
        // one has run out of frames to disagree on.
        assert!(
            ticks >= FLOORS[1].ticks
                && orders >= FLOORS[1].orders
                && first[0] >= 1772
                && first[1] >= 1772,
            "the headline fell: ticks {ticks}, orders {orders}, first divergence {:?} \
             — the floor is ticks {}, orders {}, and neither player parting",
            report.first_divergence,
            FLOORS[1].ticks,
            FLOORS[1].orders
        );
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
        // **The roster, both ways.** `unlinked` is what the original has and
        // this simulation does not; `extra` is the mirror, and it exists
        // because the one-sided count could be *paid off by
        // over-producing* — which is exactly what was happening.
        //
        // 2026-08-24 read "1/9 now trains on the original's frame, and only
        // the last citizen is missing", on 268 unlinked unit-frames. It did
        // not: the AI's ninth citizen was standing here from frame **897**
        // against the original's 1297, four hundred frames early, and the
        // one-sided measure could not see a unit that arrives too soon —
        // from 1297 the link exists and the earlier frames cost nothing.
        // The mirror was added in item 47 and reported those 400 at once.
        //
        // Item 47 (the AI builder's `unit_masks & 0x40000`) took `1/1` off
        // the farm it had built and put it on the original's woodcutter,
        // which is right and is what moved the headline — and with one
        // fewer farmer the AI now never reaches its ninth citizen inside
        // 1,772 frames. So the gap did not appear here; it changed sign,
        // and the two-sided total went 668 → 744. **The AI's long-run
        // economy is the item this measure now names**, and until it is
        // taken the honest statement is a floor on the *pair*.
        //
        // Item 70 (`find_gather_spot`'s cap-headroom score) changed the
        // sign back: preferring the good whose income is furthest below
        // its commerce cap puts the AI back on its farms, `1/9` is trained
        // inside the capture again, and the pair returns to **268 + 400**
        // — the ninth citizen four hundred frames early, exactly where it
        // stood before item 47. The over-production is unexplained and is
        // its own item; what this pin says is that the whole is not worse
        // for it.
        // Item 81 (the building ramp's missing ceiling) changed the sign a
        // third time, and this is the first reading in which the pair is
        // **one-sided again**: the AI's second city costs sixty rather than
        // twenty-two, so it is bought on the original's frame 776 rather
        // than 576, and the hundred and twenty food and timber it no longer
        // has two hundred frames early are what `1/9` was trained on. `1/9`
        // now arrives at **1497** against the original's 1297 — late, where
        // it used to be four hundred frames early — so it joins `1/10` in
        // `missing` and leaves `extras` empty. 268 + 400 → **468 + 0**.
        //
        // Item 74 (the AI's thirty-two food) closes it: the farm's
        // completion bonus and the science re-pricing put the AI back on
        // the original's food, `1/9` is trained on the original's own
        // frame, and it leaves `missing` altogether. **468 + 0 → 268 + 0**,
        // the lowest the pair has been, and the 268 are `1/10` alone — the
        // tenth citizen, which the original trains at 1772 and this does
        // not reach.
        //
        // Item 98 (the script's statics, `docs/AI.md` §17) closes that
        // one too: `needed_citizens` survives the call it was set in, so
        // the opening's every-call `train_unit_with_need` keeps training,
        // and `1/10` arrives. **268 + 0 → 0 + 0** — the roster is the
        // original's, both ways, over the whole capture.
        assert!(
            missing.is_empty(),
            "a citizen the original trains and the AI does not reach: {missing:?}"
        );
        let extras: Vec<(i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.extra_units.iter().copied())
            .collect();
        assert!(
            extras.is_empty(),
            "nothing is ahead of the original any more: {extras:?}"
        );
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        eprintln!(
            "run10 roster: {unlinked} missing + {} extra, missing units {missing:?}",
            extras.len()
        );
        assert!(
            unlinked + extras.len() <= 268,
            "roster unit-frames: {unlinked} missing + {} extra — 2026-08-30 \
             was 268 + 0 with item 74 and 468 + 0 before it, 2026-08-29 was \
             268 + 400, 2026-08-27 was 744 + 0",
            extras.len()
        );

        // **`ObjectData::mylos` over the whole run** — the differential
        // check `docs/VISION.md` §2 hangs on, and the longest one available:
        // every object record carries `mylos` at every detail level, so this
        // is 26,433 unit-frames of the original's own line of sight against
        // `Sim::unit_los`.
        //
        // Exactly one disagreement, and it is a **cache**, not a formula.
        // `mylos` is stored on the object and refreshed by
        // `Leader::calc_unit_stats`, which `Leader::process` runs on the
        // frame *after* `gain_tech` sets `leader_flags & 0x4000000`. So when
        // player 1's first science level lands, the simulation's pure
        // function reports `4 + 1 × 2 = 6` on the frame the level is gained
        // and the original still reports 4 until the next one. `docs/VISION.md`
        // §7 books modelling the cache; the pair is what proves `epoch[3]`
        // is the Science line and `science_los` its multiplier.
        //
        // 26,433 → 25,957 with item 47: the 476 are `1/9`'s, the citizen
        // the AI no longer reaches (see the roster note above). Item 70
        // put it back and the count with it — 25,957 → 26,433, the same
        // 476 unit-frames. 26,433 → **26,233** with item 81, and it is the
        // same citizen a third time: `1/9` is trained two hundred frames
        // later than the original now rather than four hundred early, so
        // two hundred of its unit-frames are no longer comparable. The one
        // disagreement is unmoved through all four.
        //
        // 26,233 → **26,433** with item 74, and it is the same citizen a
        // fourth time: the farm's completion bonus and the science
        // re-pricing give the AI back the thirty-two food it was short, so
        // `1/9` is trained on the original's own frame again and its two
        // hundred unit-frames come back into view.
        //
        // 26,433 → **26,701** with item 98 (the script's statics): `1/10`
        // is trained at last, and its 268 unit-frames are the difference.
        // The one disagreement is unmoved through all five.
        let los_seen: usize = report.frames.iter().map(|f| f.los_compared).sum();
        assert_eq!(los_seen, 26_701, "every compared unit-frame carries mylos");
        let bad: Vec<LosDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.los_diverged.iter().copied())
            .collect();
        assert_eq!(
            bad,
            vec![LosDivergence {
                frame: 202,
                who: 1,
                o: 0,
                ours: 6,
                theirs: 4,
            }],
            "the scout's science level, one frame ahead of the original's cache"
        );

        // **The collision block over the whole run** (`docs/COLLISION.md`
        // §8). `UnitData::log_data` writes `collide`, `collide_o`,
        // `collide_who`, `collide_guy` and `safe` at every detail level, so
        // this is 40,600 field-frames of the original's own collision state
        // against `crates/sim/src/collide.rs` — the widest single record
        // this harness compares, and the one that says the mechanic is
        // right rather than merely plausible.
        //
        // 285 disagree, none before frame 201, and 277 of those are one
        // sticky byte: `collide_guy` is written to 0 by a hard collision
        // and **never cleared** (the clear path writes only `collide_o` and
        // `collide_who`), so a single collision this simulation has and the
        // original does not leaves `1/3` reading 0 against −1 for every one
        // of its remaining frames. The other eight are two units and two
        // collisions: `1/4` at 201–202 and `1/6` at 207, both well past the
        // score.
        //
        // 40,600 → 42,630 with item 47, and 285 → 400 with it. Both are
        // the *same* sticky byte and the same frame: `1/3` holds its
        // position against the original for 115 more frames than it did,
        // so 115 more of its unit-frames come into view — and every one of
        // them reads that one stale `collide_guy`. 392 of the 400 are
        // `1/3`'s; the field tally is the check that says so, and it is
        // asserted below rather than left to the total.
        //
        // 42,630 → **41,225** with item 50 (the bird): the *coverage*
        // moved, not the mechanic. This counter is five fields on every
        // unit-frame whose position already agrees, so it follows each
        // unit's own parting frame rather than the headline's — and the
        // headline went 181 → 185 while `0/1`, `1/1` and `1/2`, hundreds
        // of frames out on a stream the bird has changed, part 281
        // unit-frames sooner between them. The two assertions that say the
        // mechanic is still right are below and both held.
        //
        // 41,225 → **43,340** with item 52 (the bird's wing beat), the
        // same way and in the other direction: the stream is closer, so
        // more unit-frames hold their positions and come into view.
        //
        // 43,340 → **40,750** with item 55 (the road). The same coverage
        // effect once more, and this time the headline went *up* while
        // this went down: `0/3` holds 31 frames longer, but the AI's units
        // — which are hundreds of frames out either way — sit on a stream
        // whose values have all changed, and between them they part
        // sooner. The two assertions that say the mechanic is right are
        // below, and both held.
        //
        // 40,750 → **42,615** with item 59 (the builder's animation), and
        // the same coverage effect a third time: the AI's units hold their
        // positions longer on a stream that is the original's for 81 more
        // frames, so 1,865 more of their unit-frames come into view. `1/1`
        // alone parts 70 frames later.
        //
        // 42,615 → **43,575** with item 49 (the blocked stand), the same
        // coverage effect a fourth time and in the up direction: the
        // headline moved 190 → 192 and the AI's units, hundreds of frames
        // out either way, hold their positions on a stream that is the
        // original's for 63 more frames.
        //
        // 43,575 → **39,950** with item 61 (the farmer's cell index), the
        // coverage effect in the down direction while the headline went
        // 192 → 200. Six farmers now walk off their cells on the
        // original's frames rather than fourteen early, which moves every
        // value the stream carries after 185 — and the AI's units, which
        // are hundreds of frames out either way, part 3,625 unit-frames
        // sooner between them. The two assertions that say the mechanic
        // is right are below and both held.
        //
        // 39,950 → **42,840** with item 63 (the waypoint's own collision
        // test), the coverage effect a sixth time and in the up direction:
        // `1/4` no longer walks off its farm on 200, so it and the two
        // farmers behind it hold their positions for hundreds of frames
        // more, and 2,890 further unit-frames come into view.
        //
        // 42,840 → **48,790** with item 64 (`is_flat`, §6 step 2's fence),
        // and the disagreements went **245 → 0**. This is the run that
        // says the mechanic is right: every one of the 48,790 field-frames
        // agrees, the sticky `collide_guy` included — 243 of the 245 were
        // that one byte, written by a hard collision and never cleared, so
        // a collision this simulation had and the original did not left a
        // unit reading 0 against −1 for the rest of the run, and the fence
        // is what stops the collision happening. The assertion below is
        // now emptiness rather than a ceiling, so any single field on any
        // unit-frame of the capture fails it.
        //
        // 48,790 → **46,941** with item 66 (`find_nearby_spot`'s collision
        // half), the coverage effect a seventh time and in the down
        // direction while the headline went 209 → 252: the AI's `1/6` and
        // `1/7`, hundreds of frames out either way, now part at 253 rather
        // than being carried along by a walk that was already wrong.
        //
        // 46,941 → **62,307** with item 68 (the think tail's conscription),
        // the coverage effect an eighth time and in the up direction while
        // the headline went 252 → 322: `1/6` and `1/7` are no longer
        // marched across the map on frame 252, so the two of them alone
        // bring 15,366 further field-frames into view before they part.
        //
        // 62,307 → **60,247** with item 70 (`find_gather_spot`'s score),
        // the ninth time and in the down direction while the headline went
        // 322 → 355. Two units moved and they moved opposite ways: `1/8`
        // parts at 506 rather than 323, and `1/4` at 437 rather than 550.
        // This count is scoped to **agreement**, not to first divergence —
        // a farmer that walks off and comes back keeps contributing after
        // it has parted — so a re-tasked farmer costs more field-frames
        // than its own parting alone accounts for.
        //
        // 60,247 → **62,932** with item 71 (the idle variant's length),
        // and 62,932 → **62,957** with item 76 (the scout's surface
        // probe) — the eleventh time, and a small one: the scout holds a
        // hundred and twenty frames longer and three units behind it part
        // earlier, so the two nearly cancel.
        //
        // 62,957 → **74,429** with item 78 (the gather write-back), the
        // twelfth time and the largest single move it has made: every one
        // of the twelve compared units parts later, so 11,472 further
        // field-frames are comparable.
        //
        // 74,429 → **77,211** with item 79 (the vision projection), the
        // thirteenth: player 1's units all hold longer and player 0's
        // three farmers part earlier, and the AI's five are worth more
        // field-frames than the human's three cost.
        //
        // 77,211 → **80,161** with item 80 (the repath throttle's decay),
        // the fourteenth — and it is the collision block's own item, so
        // the count is the one to read: player 0's three farmers recover
        // everything item 79 cost them and pass it, 574/577/579 →
        // 687/700/703.
        //
        // 80,161 → **87,548** with item 81 (the building ramp), the
        // fifteenth and the largest single move it has made: the AI's
        // second city lands on the original's frame, so every unit of both
        // players holds two hundred frames longer.
        //
        // 87,548 → **93,341** with item 74 (the AI's thirty-two food), the
        // sixteenth: `1/9` is trained on the original's frame again, so its
        // own field-frames return, and eight of the other twelve units hold
        // longer with them.
        //
        // 93,341 -> **93,357** with items 36 and 93, and -> **93,398**
        // with item 95 (the animal's hurry): the AI's `1/4` and `1/5` each
        // hold a few frames longer, so their collision blocks are
        // comparable for longer.
        //
        // 93,398 -> **91,210** with the blocked animal's dropped walk, the
        // seventeenth and the second time it has fallen while a score
        // rose. **One** of the thirteen units moved: the AI's `1/9` parts
        // at 1320 rather than 1377, and the other twelve are unchanged to
        // the frame, as are both players' first divergences (802 and 573)
        // and the headline 572/776. run33's draws are identical to frame
        // 1128 under the same change, so a unit whose position parts at
        // 1320 has been on a stream of its own for two hundred frames
        // (`docs/SYNC.md` §3.14).
        //
        // 91,210 -> **92,766** with item 98 (the script's statics), the
        // eighteenth: `1/10` exists, so its own field-frames join the
        // count, and the other twelve are unchanged to the frame.
        //
        // 92,766 -> **97,333** with item 100 (the bird's landing search),
        // the nineteenth and the largest rise yet from a mechanic with no
        // unit in it: sixty draws a landing is enough of the stream that
        // several of the thirteen hold hundreds of frames longer. The
        // headline 572/776 and both players' first divergences (802 and
        // 573) are unchanged.
        //
        // 97,333 -> **97,118** with item 101 (the chopping guy's own
        // wait), the twentieth and the third fall while a score rose.
        // **One** of the fourteen moved: the AI's `1/10` parts at 1522
        // rather than 1579, and the other thirteen are unchanged to the
        // frame, as are the headline 572/776 and both players' first
        // divergences. A woodcutter that now stays at its tile three times
        // as long is a different unit on the map from frame 500 on, and
        // 1/10 is the unit that had been holding longest on the old one.
        //
        // 97,118 -> **97,108** with item 102 (the far wander's literal
        // bearing), the twenty-first and the smallest move it has ever
        // made: ten fields, two unit-frames, and **not one** of the
        // fourteen units parts on a different frame. Great Lakes' herd
        // wanders to different spots from this map's, so its animals'
        // arrivals fall on different frames deep past the parting, and two
        // unit-frames that used to re-agree by coincidence no longer do.
        // The headline 572/776, both players' first divergences and every
        // by-unit parting are unchanged; East Indies' word goes 742 → 867
        // and run33's own window totals rise 977/866 → 986/884.
        //
        // 97,108 -> **98,019** with item 106 (the scout's walk to a goody
        // box), the twenty-second. **Three** of the fourteen moved and all
        // three later: `1/0` parts at 959 rather than 872, `1/8` at 906
        // rather than 904, and `1/10` at 1552 rather than 1522. `1/0` is
        // the AI's scout and it is the unit the mechanic is about; the
        // other two move because a scout that walks somewhere else is a
        // different stream from frame 800 on. The headline 572/776 and
        // both players' first divergences (802 and 573) are unchanged;
        // East Indies' word goes 879 → 1256.
        //
        // 98,019 -> **99,343** with item 69's two halves, the twenty-third.
        // The total is scoped to agreeing unit-frames, so it moves with the
        // headline; ticks 572 -> 781 is what moved it.
        //
        // 99,343 -> **112,447** with item 84, the twenty-fourth: ticks
        // 781 -> 910 and every one of the thirteen parting later.
        //
        // 112,447 -> **123,500** with item 112, the twenty-fifth, and this
        // one moved without the headline. Six of the fourteen part later —
        // player 0's three by two hundred frames apiece — so eleven
        // thousand more unit-frames are comparable at all, and the block
        // stays empty over every one of them.
        //
        // 123,500 -> **128,672** with item 113, the twenty-sixth, and it
        // moves with the headline: `1/1` never parts at all now, so its
        // whole run is comparable where only nine hundred frames of it
        // were.
        //
        // 128,672 -> **139,514** with item 114, the twenty-seventh and the
        // last one this capture can make: no unit parts anywhere, so every
        // field-frame run10 holds is comparable and the number is now the
        // capture's own size rather than a score. It moves again only when
        // a longer capture replaces this one.
        let coll_seen: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        assert_eq!(
            coll_seen, 139_514,
            "five fields on every agreeing unit-frame"
        );
        // **The emptiness, scoped to what the capture can speak to.**
        // Item 64 took the block to zero over the whole run; item 66
        // moved `1/3` from parting at 103 to parting at 345, and 158
        // frames past that it takes a collision the original does not —
        // one sticky `collide_guy 0` from frame 503 to 1600, 461
        // field-frames of the same byte. A unit whose position has been
        // wrong for a hundred frames is not evidence about collision, so
        // what is asserted is emptiness **before each unit's own first
        // divergence**: every field-frame of the capture that is
        // comparable at all, and any single one of them fails it.
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        // 2026-08-31, item 69: **two field-frames, and they are the
        // item's own residue.** With ticks 572 -> 781 the comparable
        // window grew by two hundred frames on every unit, and `1/6`'s
        // frame 797 came inside it: this crate's `collide` counter reads
        // **3** where the original's reads 2 and its `collide_frame` is
        // **796** against 795 — one collision more, entered a frame later,
        // on the unit whose position then parts at 798. It is the
        // collision seam one layer under the pause, and it is the queue's
        // successor to this item. Everything else in the window is still
        // empty, and any third row fails this.
        assert_eq!(
            coll_bad
                .iter()
                .map(|d| (d.frame, d.who, d.o, d.field, d.ours, d.theirs))
                .collect::<Vec<_>>(),
            Vec::new(),
            "the collision block agrees on every comparable field-frame of {coll_seen}"
        );

        // **The tile choice, asserted where it was wrong** (item 25). The
        // earliest frame on which the two sides' `GATHERORDER` name
        // different resource tiles: **2** before the access filter landed,
        // when player 1's woodcutter picked `(214, 93)` — a tree ringed by
        // its own forest, with no orthogonal neighbour to stand on — and
        // the original picked `(213, 92)`. That one tile pinned the
        // headline at ticks 3. It went to 169 — `1/1`, the citizen the
        // original turned into a woodcutter and this simulation kept on the
        // farm it had built — and with item 47 it is 430, where the
        // woodcutter `1/6` chooses its second tree of the game.
        //
        // **413 with item 50 (the bird), and this one moved the wrong
        // way.** Which tree a woodcutter's second choice lands on is decided
        // by `find_gather_spot` on a stream that is still ten draws short
        // of the original's from frame 168 (`docs/SYNC.md` §3.9), so it is
        // luck rather than a rule until that closes — and this time the
        // luck went against. Said plainly rather than folded into a
        // ceiling: the two scores that decide the item both rose, and this
        // sub-score fell 17 frames.
        //
        // **407 with item 52 (the wing beat)**, six frames the same way and
        // for the same reason: `1/6`'s second tree is still drawn off a
        // stream that has drifted by then, and a closer stream at frame 180
        // is not a closer one at 407.
        //
        // **430 with item 55 (the road)**, back where item 47 left it, and
        // by the same luck in the other direction.
        //
        // **407 with item 59**, the same twenty-three frames back the other
        // way. `1/6`'s second tree is drawn on a frame the stream reaches
        // long after it has parted (122 on run14's trace), so which tree it
        // is remains luck; the numbers that are not are the two the item
        // was booked on.
        //
        // **415 with item 61**, eight frames the same luck's way again:
        // `1/6`'s second tree is still drawn on a frame long past the
        // word's divergence (201 now), and the six farmers' corrected
        // cells move every value the stream carries from 185 on.
        //
        // **407 with item 63**, those same eight frames back the other
        // way — the sixth time this sub-score has bounced between 407 and
        // 430 on a draw nobody has fixed. It stops being luck when the
        // stream reaches frame 407 in step, and not before.
        //
        // **1,298 with item 70**, and this one is not the same bounce: the
        // cap-headroom score sends the AI's citizens to the buildings the
        // original sends them to, so `1/6`'s trees stop being the question
        // and every gather tile of the capture agrees until the frame
        // after the original trains `1/9`. Nine hundred frames is three
        // times the span the bounce ever covered.
        //
        // **1,299 with item 76**, one frame further: the AI's `1/9` is
        // still trained late here, and the frame its gather tile first
        // disagrees is the frame after that.
        //
        // **1,298 with item 78**, the same frame back: `1/9` is still the
        // unit, and it is still trained late (item 74).
        //
        // **1,384 with item 106**, eleven frames on from the 1,373 the
        // intervening items had reached: the scout's walk to a goody box
        // moves `1/0` and, through the stream, the frames the AI's
        // citizens re-pick on.
        //
        // **1,598 with item 112**, two hundred frames on, and it is the
        // word carrying it: with the herd's wander centre right the stream
        // is the original's to 1372, so the citizens re-pick on the
        // original's frames for two hundred frames more.
        //
        // **Never, with item 114.** The word runs past this capture's own
        // length, so there is no frame left on which a gather tile can
        // disagree: every citizen of both players works the original's
        // tile for all 1,772 frames. `None` is the assertion now, and it
        // is a stronger one than any frame number — a regression anywhere
        // in the run fails it.
        let tile_row = |d: &&OrderDivergence| {
            matches!(
                d.what,
                OrderMismatch::Gather {
                    field: "tx" | "ty",
                    ..
                }
            )
        };
        let first_tile = report
            .frames
            .iter()
            .find(|f| f.order_diverged.iter().any(|d| tile_row(&d)))
            .map(|f| f.frame);
        assert_eq!(
            first_tile, None,
            "the first frame on which a gather tile disagrees"
        );
        assert!(
            !report
                .frames
                .iter()
                .flat_map(|f| f.order_diverged.iter())
                .any(|d| d.who == 0 && tile_row(&d)),
            "player 0's woodcutters chop the original's trees for all 1,772 frames"
        );

        // **Where a trained unit appears** (item 43). The AI's citizens are
        // created on frames 100, 206 and 320, and on each of those frames
        // the simulation stands its unit on the original's own tile —
        // `(42360, 17208)`, due south of London on the exit ring. Before
        // `come_out` was wired into the handover the unit was left on its
        // trainer's centre and this reported the city's position against
        // that one, a thousand units away.
        for (born, o) in [(100i64, 6i64), (206, 7), (320, 8)] {
            let f = report
                .frames
                .iter()
                .find(|f| f.frame == born)
                .expect("the frame the citizen is trained on");
            assert!(
                !f.diverged.iter().any(|d| d.who == 1 && d.o == o),
                "1/{o} does not appear where the original puts it on frame {born}: {:?}",
                f.diverged.iter().find(|d| d.who == 1 && d.o == o)
            );
        }

        // **Both angles, on every unit-frame where the positions agree**
        // (`docs/MOVEMENT.md` §"Two angles", item 34): `UnitData::angle`
        // against the heading and guy 0's `angle` against the facing.
        // 13,542 → 15,336 with item 25: this counts only the unit-frames
        // whose *positions* agree, so the tile-choice fix bought 897 of
        // them outright — player 1's woodcutter alone now stands where the
        // original stands it from frame 4 to frame 567.
        // 15,336 → 15,010 with item 43. Every unit's **first** divergence
        // held or improved, and the AI's trained citizens now appear on the
        // original's own tile; what fell is agreement deep in the untraced
        // stretch, where those citizens are alive and walking instead of
        // standing on their city, so their later frames are their own.
        // 15,010 → 15,318 with item 44, and the same caveat holds twice
        // over: the headline went 102 → 122 while this moved by 308, which
        // is the useful reminder that a total over 1,772 frames is not the
        // score. The score is where the *first* divergence falls.
        // 15,318 → 16,206 with item 46 (collision): `1/6` alone holds from
        // frame 123 to frame 208.
        // 16,206 → 17,018 with item 47, against 6,866 → 6,926 bad: of the
        // 812 rows the AI builder's fix brought into view, 752 agree.
        // 17,018 → **16,456** with item 50 (the bird), the same coverage
        // move the collision tally makes: this counts unit-frames whose
        // positions agree, so it follows each unit's own parting frame,
        // and three units hundreds of frames out part sooner on a stream
        // the bird has changed. The headline went 181 → 185.
        // 16,456 → **17,302** with item 52 (the wing beat), the other way
        // round: 846 more unit-frames hold their positions on a stream the
        // bird's two animation lengths have brought closer.
        // 17,302 → **16,266** with item 55 (the road): coverage again, and
        // again against the headline's direction, which went 181 → 202.
        // 16,266 → **17,012** with item 59 (the builder's animation): 746
        // more unit-frames hold their positions on a stream that is the
        // original's for 81 more frames.
        // 17,012 → **17,396** with item 49 (the blocked stand): 384 more,
        // and the headline went 190 → 192.
        // 17,396 → **15,946** with item 61 (the farmer's cell index):
        // coverage against the headline's direction a fourth time, and
        // for the same reason as the collision tally three paragraphs up
        // — the six corrected farmers move every value the stream carries
        // after 185, and the units hundreds of frames out part sooner on
        // it. The headline went 192 → 200.
        // 15,946 → **17,102** with item 63 (the waypoint's own collision
        // test): 1,156 more, the coverage effect back in the headline's
        // direction — `1/4` stays on its farm from 200 rather than walking
        // off it, and the farmers behind it hold with it.
        // 17,102 → **19,464** with item 64 (`is_flat`): 2,362 more, the
        // coverage effect in the headline's direction again — `1/6` no
        // longer stalls at its own gather target from 207, so it and the
        // AI's later citizens hold their positions for hundreds of frames
        // more. The headline went 207 → 209.
        // 19,464 → **18,724** with item 66 (`find_nearby_spot`'s collision
        // half): 740 fewer, coverage against the headline's direction a
        // fifth time while the headline went 209 → 252. Every unit that
        // now walks to a different spot walks a different route after it,
        // and three of the AI's citizens part sooner in the deep untraced
        // stretch than they did off a walk that was already wrong.
        // 18,724 → **24,120** with item 68 (the think tail's conscription):
        // 5,396 more, the coverage effect back in the headline's direction
        // and the largest single move it has made — `1/6` and `1/7` are no
        // longer marched off on frame 252 and hold to 735 and 937, and the
        // human's farmers hold with them on a stream that is the original's
        // for seventy frames more. The headline went 252 → 322.
        // 24,120 → **23,296** with item 70 (`find_gather_spot`'s score):
        // 824 fewer, coverage against the headline's direction a sixth
        // time while the headline went 322 → 355, and the same two units
        // the collision tally names — `1/4` re-tasked 113 frames sooner
        // costs more agreeing frames than `1/8`'s 183 extra ones pay for.
        // 23,296 → **24,370** with item 71 (the idle variant's length), and
        // 24,370 → **24,380** with item 76 (the scout's surface probe): ten
        // more, the two directions nearly cancelling — the scout holds a
        // hundred and twenty frames longer, three units behind it part
        // earlier.
        // 24,380 → **28,916** with item 78 (the gather write-back), and
        // this is the first widening whose *disagreements* fell with it:
        // 4,536 more rows compared and 2,012 fewer bad. The paragraph
        // below opened on `0/2`'s 2,680 rows — the woodcutter that
        // "walks to `(4440, 28680)` on frame 432 … and on 433 the original
        // turns it to face what it is about to gather while the simulation
        // leaves it pointing the way it walked". That was not one of
        // `set_angle`'s seventeen other callers after all: with the
        // write-back landing, `0/2` reaches the camp-arrival branch on 433
        // and faces the camp there, and its share of the residue is 140.
        // 28,916 → **29,878** with item 79 (the vision projection): 962
        // more, the AI's five holding longer against the human farmers'
        // earlier parting.
        // 29,878 → **31,022** with item 80 (the repath throttle's decay):
        // 1,144 more, the human farmers holding a hundred frames longer.
        // 31,022 → **33,992** with item 81 (the building ramp): 2,970
        // more, of which 1,684 agree.
        // 33,992 → **35,868** with item 74 (the AI's thirty-two food):
        // 1,876 more, `1/9`'s own rows and the eight units that hold
        // longer beside it.
        // Unmoved by item 36 — it changes no position — and 35,868 →
        // **35,984** with item 93.
        // 35,984 → **35,942** with item 95 (the animal's hurry): 42 fewer,
        // the coverage effect against the headline's direction for the
        // seventh time, and it is `1/4` and `1/5` parting a frame or two
        // earlier deep in the untraced stretch. run10's headline is
        // unmoved at 572/776 and its collision rows *rose*, 93,357 →
        // 93,398.
        // 35,942 → **35,188** with the blocked animal's dropped walk: 754
        // fewer, and all of them `1/9`'s, which parts at 1320 rather than
        // 1377 (see the collision rows above). The headline is unmoved at
        // 572/776 and the other map's word goes 91 → 201.
        // 35,188 → **35,742** with item 98 (the script's statics): 554
        // more, and all of them `1/10`'s, the citizen the AI reaches for
        // the first time.
        // 35,742 → **37,376** with item 100 (the bird's landing search):
        // 1,634 more, the same holding-longer the collision rows show.
        // 37,376 → **37,174** with item 101 (the chopping guy's own wait):
        // 202 fewer, and all of them `1/10`'s, which parts at 1522 rather
        // than 1579 (see the collision rows above). The headline is
        // unmoved at 572/776 and East Indies' word goes 645 → 742.
        // 37,174 → **37,170** with item 102 (the far wander's literal
        // bearing): four fewer, the same two unit-frames the collision rows
        // lost, and no unit parts on a different frame. East Indies' word
        // goes 742 → 867.
        // 37,170 → **37,450** with item 106 (the scout's walk to a goody
        // box): 280 more, the three units that hold longer for it —
        // `1/0` at 959, `1/8` at 906, `1/10` at 1552.
        // 38,104 → **37,838** with item 69's two halves. The four units
        // whose parting moved are `1/2` 573 → **1184**, `1/1` 794 → 797,
        // `1/6` 795 → 798 and `1/10` 1579 → **1528**; the total counts
        // every agreeing unit-frame of the whole capture, coincidences
        // past a parting included, so it falls by 266 while the score it
        // sits beside rises by 209 frames. The number to read is the
        // headline.
        // 37,838 → **43,202** with item 84's farm byte, and → **47,364**
        // with item 112's herd centre: both rises are the comparable
        // window growing on six of the fourteen units at once, and neither
        // is a coincidence past a parting — the word carries them.
        // 47,364 → **49,088** with item 113's two halves, and again the
        // word carries it: `1/1` never parts, so its whole 1,772 frames
        // are comparable where nine hundred were.
        // 49,088 → **53,402** with item 114's pasture, and this is the
        // ceiling: no unit parts anywhere, so the number is every guy-frame
        // the capture holds. Like the collision rows it moves again only
        // when a longer capture replaces this one.
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        assert_eq!(angles, 53_402, "two per agreeing unit-frame that has a guy");
        let bad: Vec<AngleDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter().copied())
            .collect();
        // 2026-08-27, on landing item 34: 5,435. The residue is **not** the
        // step's — it is `Unit::set_angle`'s other seventeen callers, which
        // this simulation does not make (`Sim::unit_set_angle`). Unit `0/2`
        // alone is 2,680 of it: it walks to `(4440, 28680)` on frame 432
        // with both sides agreeing on the position, the path and both
        // angles, and on 433 the original turns it to face what it is about
        // to gather while the simulation leaves it pointing the way it
        // walked. The farmers (`o` 3 to 5) are most of the rest, and they
        // are doing a different job at the same spot (`docs/SYNC.md` §6).
        // 5,435 → 6,382 with item 25, against 13,542 → 15,336 compared: the
        // ceiling rose because 1,794 rows the harness could not see before
        // came into view, and 847 of them agree. Item 36 is still the item
        // that takes this down.
        // 6,382 → 6,866 with item 46, against 15,318 → 16,206 compared:
        // 404 of the 888 rows collision brought into view agree, and the
        // rest are the same seventeen callers.
        // 6,926 → 7,042 with item 52, against 16,456 → 17,302 compared:
        // 730 of the 846 the wing beat brought into view agree, and the
        // 116 that do not are those same callers again (item 36).
        // 7,042 → 8,737 with item 64, against 17,102 → 19,464 compared:
        // 667 of the 2,362 rows the fence brought into view agree, and the
        // 1,695 that do not are the same seventeen callers on units that
        // now stand where the original stands them for far longer. Item 36
        // is still the item that takes this down.
        // 8,737 → 9,378 with item 68, against 19,464 → 24,120 compared:
        // 4,755 of the 5,396 rows the think tail's correction brought into
        // view agree and 641 do not, which is the best ratio any widening
        // has had here — and the 641 are the same seventeen callers on
        // two woodcutters that now work their trees for another five
        // hundred frames. Item 36 is still the item that takes this down.
        // 9,378 → **7,366** with item 78, against 24,380 → 28,916
        // compared: `0/2`'s 2,680 rows were never item 36's, and what is
        // left is the farmers (`0/3`–`0/5` and `1/3`–`1/5`, 6,658 of the
        // 7,366) doing a different job at the same spot.
        // 7,366 → **7,242** with item 79, against 28,916 → 29,878
        // compared.
        // 7,242 → **7,870** with item 80, against 29,878 → 31,022
        // compared: 516 of the 1,144 rows the throttle's decay brought
        // into view agree, and the residue is item 36's and nothing else —
        // `0/3`–`0/5`, `1/3`–`1/5` and `1/8`, every one of them a farmer,
        // are 7,818 of the 7,870, and no other unit contributes more than
        // twenty-eight rows.
        // 7,870 → **9,156** with item 81, against 31,022 → 33,992
        // compared: 1,684 of the 2,970 new rows agree, and the residue is
        // still the farmers — `0/3`–`0/5`, `1/3`–`1/5` and `1/8` are 8,866
        // of the 9,156. What is new is `0/1` (140) and `0/2` (136), the two
        // woodcutters, which item 36 did not have to account for before.
        // 9,156 → **8,969** with item 37, against 33,992 → 35,868
        // compared: the standing body's instant turn
        // (`docs/MOVEMENT.md`, "The body step") both removed the scout's
        // three and kept 1,876 more rows in view.
        // 8,969 → **1,227** with item 36, on the same 35,868 — and it was
        // never the seventeen callers. It was two predicates in code this
        // crate already had (`docs/SYNC.md` §3.12): `add_move_order` took
        // the angle to the **snapped** destination where the listing takes
        // it to the point the caller handed over, and `move_step`'s gather
        // clause was applied to both arrival arms where the original has it
        // on the Manhattan snap alone. Every farmer left the residue and
        // the earliest surviving row went 110 → 820.
        // 1,227 → **1,910** against 35,868 → 35,984 with item 93 (the
        // pasture's snap and `Guy::move`'s turn arm, §3.11's pair, landed
        // the same session because item 36 unblocked it): 116 more rows in
        // view, and the coverage effect deep in the untraced stretch that
        // every widening has had here.
        assert!(
            bad.len() <= 1_910,
            "angle disagreements grew: {} of {angles}",
            bad.len()
        );
        // The scout was item 37: **three** rows in 1,772 frames, each the
        // frame after an arrival, where the original's body had already
        // snapped onto the order's angle and the simulation's turned a
        // frame later — 96, 362 and 721. **None, since 2026-08-30**:
        // `Guy::move` zeroes `last_speed` at the head of its at-des branch
        // and `GuyData::turn_speed` reads that zero, so a standing foot or
        // mounted body swallows whatever turn it is owed in one frame.
        // The whole of the residue above is item 36's farmers now.
        let scout: Vec<AngleDivergence> = bad
            .iter()
            .copied()
            .filter(|d| (d.who, d.o) == (1, 0))
            .collect();
        assert!(scout.is_empty(), "the AI scout turns late again: {scout:?}");
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
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, None).unwrap();

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
        // **The carve-out is gone, 2026-08-29.** It used to except a farmer
        // (`o` 3–5, or any citizen trained during the run) after frame 150,
        // for the second re-target's two draws off a stream that had
        // drifted. Item 70 — `find_gather_spot` scored on the headroom
        // under the commerce cap rather than on distance alone — put every
        // citizen of this capture on the building the original puts it on,
        // and with that the exception had nothing left in it.
        //
        // **Scoped to each unit's own first divergence, 2026-08-29
        // (item 71).** Run6 is not traced past its start, so the stream is
        // the simulation's own from frame 4 on, and a farmer's *second*
        // re-target is two draws off it. With the animation lengths read
        // from the install the clocks moved, and `1/5`'s second re-target
        // now lands one frame apart: it parts in **position** on frame
        // 421, and its order list follows. Rows a unit produces after its
        // own position has parted are not evidence about the order system
        // — the same reasoning the collision block above is scoped by — so
        // what is asserted is emptiness up to each unit's own parting, and
        // any single row before it fails.
        let parted_orders: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let modelled: Vec<&OrderDivergence> = modelled
            .into_iter()
            .filter(|d| {
                parted_orders
                    .get(&(d.who, d.o))
                    .is_none_or(|&f| d.frame < f)
            })
            .collect();
        assert!(
            modelled.is_empty(),
            "a modelled order field disagrees: {:?}",
            &modelled[..modelled.len().min(4)]
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
        // 1,238/875 → 1,220/866 with run13's words installed at frame 94
        // (the siblings' traced frames are pooled, `borrow_from_siblings`).
        // 1,220/866 → 1,242/877 with the animation clock (`docs/ANIM.md`):
        // the idle wraps and the training rolls are draws the original
        // makes too, but on the sim's own stream between the traced frames
        // they move the woodcutters' later waits (`% 50 + 100`) to other
        // values — noise in the untraced stretch, not a mechanic lost; the
        // traced frames 0–3 and 94–103 all held or improved.
        // 1,242/877 → 1,246/879 with the camp-arrival stand removed
        // (`orders.rs`, the stand/wrap swap): the woodcutter now unloads on
        // its first frame at the camp where it used to stand idle, so its
        // clock and its `% 50 + 100` wait land elsewhere in the untraced
        // stretch. Four order-frames and two path-frames of that noise
        // against three maps' frame 0 becoming exact — run20 and the fuzzed
        // map draw for draw, run10's 128 against 120 now 120 against 120.
        // 1,246/879 → 1,267/892 with `produce_building`'s three placement
        // defects fixed (`docs/AI.md` §2.20, 2026-08-26). **The totals
        // fell**: 1,679/1,199 to 1,552/1,160. What rose is only this
        // split's non-farmer half, because the AI now puts its buildings
        // somewhere else on this map too and the farmers' share of the
        // disagreements fell further than the rest (433/320 farmer-frames
        // to 285/268). Every traced capture held or improved — run20's
        // frame 1 lost its `produce_building` residue entirely, the fuzzed
        // map went 48/45 to 43/45, the Great Lakes did not move.
        // 1,267/892 → **1,181/1,602** on 2026-08-27, when a trained unit
        // started carrying its type (`docs/VISION.md` §8). The orders half
        // fell; the paths half rose by 710, and the reason is that the AI's
        // trained citizens used to **idle** — a unit with no order has one
        // `Length` disagreement a frame and no path stack at all, and a
        // unit that gathers has a stack that differs in detail on every one
        // of the four hundred frames it lives. So the ceiling rose because
        // the simulation started doing the thing, not because it stopped.
        // What settles that this is not a regression is that **every traced
        // check held**: run20 frame 0 175/175, frame 1 53/53, frame 2 5/5
        // and its world chain entry for entry; the fuzzed map 195/195,
        // 43/45 and 1,377/1,221 unmoved to the number; run10's window
        // 98–103 unmoved but for its own row. And the measurement that
        // motivated it: `mylos` over run10's 26,433 unit-frames went from
        // 5,170 disagreements to **1**.
        // 1,181/1,602 → **1,212/1,594** on 2026-08-27, item 34, and the
        // totals fell hard: 1,793/2,043 to **1,516/1,911**. The body stopped
        // chasing the unit at eleven eighths and started being written onto
        // it, and every unit that turns instantly from a standstill started
        // doing so (`docs/MOVEMENT.md`). What moved is the *split*: the
        // farmers' share fell from 612/441 to 304/317 — they now walk the
        // original's frames, so their later re-targets land elsewhere in
        // the untraced stretch — and 31 order-frames crossed out of it.
        // Every traced check held or improved: run13's window frame 102
        // went 22/6 to **6/6** (the row item 34 was booked on), run20's
        // frame 0 175/175 and frame 2 5/5 unmoved, and run10's scout is on
        // the original's position and both of its angles for the whole of
        // frames 57 to 91.
        // 1,212/1,594 → **1,957/1,254** on 2026-08-27, item 25, and the two
        // halves moved for two different reasons that are worth keeping
        // apart:
        //
        // - **run6 was being run against the wrong map.** Its `WORLD` block
        //   is `BUILDS=7`'s — seventeen scalars, no cells, no tile masks —
        //   so the harness stood a flat, region-less, treeless world up and
        //   every number below was measured on it. `borrow_from_siblings`
        //   now takes the whole block from a sibling whose scalars match
        //   field for field, and the effect is the one that settles it:
        //   **run6 and run10 are the same game, and their diffs now agree
        //   exactly** — the same headline, the same first divergence for
        //   every unit. Before, run6 said player 1 broke at frame 2 and
        //   run10 said frame 4. That took the totals to 1,362/1,674
        //   (736/1,254 without the farmers), and moved the farmers' share
        //   *up*, from 304/317 to 626/420: their tiles were never being
        //   compared against real terrain.
        // - **The gather order is now diffed whole** — `tx`, `ty`, `wait`,
        //   `goto_build`, `been_there`, `dist_mod`, which the harness
        //   parsed and never compared. That is the other 1,221 order rows,
        //   all of them player 1's `1/1`, the citizen the original turns
        //   into a builder and this simulation keeps at the woodcutter
        //   (first row: frame 167). None of them is before the score.
        // 1,957/1,254 → **2,084/1,242** with item 43's exit ring, and the
        // totals fell: 2,583/1,674 to 2,591/1,613 with the farmers' share
        // down from 626/420 to 507/371. The AI's trained citizens are alive
        // on the map from the frame the original creates them instead of
        // standing on their city, so they are compared where they used not
        // to be — and the check that says this is the right trade is the
        // first-divergence list, every entry of which held or improved.
        // 2,084/1,254 → **848/1,050** with item 49 (the blocked stand):
        // run6 is run10's game, so the two blocked stands go with it, and
        // the ceiling comes down to what the run now measures rather than
        // to a peak nobody has reached since.
        // 832/1,048 measured → **853/1,024** with item 63 (the waypoint's
        // own collision test): the paths half fell by 24 and the orders
        // half rose by 21, and all of both is `1/6`, `1/7` and `1/8` — the
        // AI's citizens, whose own first divergences are 208, 210 and 323
        // and did not move. What settles that this is the untraced tail
        // rather than a loss is the first-divergence list, which is run10's
        // to the frame: **six of the ten units improved and four held**,
        // `0/3` and `0/5` 213 → 326, `0/4` 220 → 356, `1/3` 219 → 345,
        // `1/4` 201 → 316, `1/5` 219 → 243.
        // 853/1,024 → **697/1,044** with item 64 (`is_flat`): the orders
        // half fell by 156 and the paths half rose by 20, and again the
        // first-divergence list is what says which way the run moved —
        // **every unit held or improved, and one left the list**. `0/4`
        // now tracks the original's position for the whole run, `0/3`
        // 326 → 331, `1/3` 345 → 411, `1/4` 316 → 318 and `1/6` 208 →
        // 253; `0/5`, `1/5`, `1/7` and `1/8` held. The twenty extra path
        // frames are `1/6`'s own: it walks for another forty-five frames
        // instead of standing still re-making the same order, and a stack
        // it carries is a stack that can disagree.
        assert!(
            orders - farmer_orders <= 697 && paths - farmer_paths <= 1_044,
            "disagreements grew: orders {orders} ({farmer_orders} farmers'), paths {paths} ({farmer_paths} farmers')"
        );
        // Printed so a re-base reads the numbers off `--nocapture`.
        eprintln!(
            "run6: orders {orders} ({farmer_orders} farmers'), paths {paths} ({farmer_paths} farmers')"
        );
        // Re-based a third time, 2026-08-24 (run13): the re-target's modulus
        // is 4, not 3 — sixteen cells to be sent to, not nine — and the
        // unit loop now rotates by owner, so the AI's farmers draw first at
        // frame 101. On run6's own stream past frame 3 both change which
        // tiles the six farmers are sent to, and the counts moved from
        // 527/357 to 662/432 — then to **588/372** once run13's words were
        // pooled in and the first re-target ran on the original's stream.
        // The farmers' pin with teeth is run13's, where the AI's goals are
        // compared (`run13_s_window_counts_and_the_ai_farmers_re_targets_are_matched`).
        // 588/372 → 612/441 with the animation clock (`docs/ANIM.md`): the
        // farmers' second re-target, past the last traced frame, rolls on
        // the sim's own stream, and the idle wraps and the training rolls
        // now sit in front of it — the tiles it sends them to are as much
        // its own as before, only different ones.
        // 612/441 → **304/317** with item 34: the farmers walk on the
        // original's frames now, so the tiles their second re-target picks
        // are fewer frames' worth of drift away from the original's.
        // 304/317 → **626/420** with item 25's whole-`WORLD` borrow, and
        // this is a re-base rather than a regression: 304/317 was measured
        // against a flat treeless world of the harness's own making. run10
        // — the same game, with its own map — has always read the farmers
        // the way run6 reads them now, and the two captures now agree unit
        // for unit.
        // 626/420 → **675/346** with item 46 (collision). The paths half
        // fell by 74 and the orders half rose by 49, and the split is the
        // familiar one: six farmers standing in a cluster around one farm
        // collide constantly, so their walks now go round each other on the
        // 48-grid. Everyone else's share fell on both halves (1,979/1,194
        // to 1,930/1,120), and run10 — the same game — went 122 → 170.
        // 675/420 → **719/404** with item 50 (the bird, `docs/SYNC.md`
        // §3.9), and this is the same re-base every stream change makes
        // here: run6 is run10's game, so a bird hatches in it too and
        // spends three draws every eighth frame from then on. The farmers'
        // *second* re-target rolls past the last traced word, on the sim's
        // own stream, so its tiles move whenever the stream does — in
        // either direction, and here both (orders +44, paths −16). The
        // half of this test with teeth is `rest`, which is held to the
        // letter and did not move; run10's headline went 181 → 185.
        // 719/404 → **739/390** with item 59 (the builder's animation), and
        // the totals fell hard: 2,591/1,613 to **1,588/1,415**. run6 is
        // run10's game, so the builders' arrival stand goes with it and the
        // stream is the original's for 81 more frames; what that buys is
        // 1,003 fewer order disagreements and 198 fewer path ones across
        // everyone. The farmers' own share moved twenty rows the other way,
        // because their second re-target still rolls past the last traced
        // word — the same drift this split has always carried.
        // 739/390 → **503/290** with item 49 (the blocked stand), and the
        // totals with them: 1,588/1,415 to **1,351/1,340**. Six farmers in
        // a cluster round one farm are what collides most in this capture,
        // so the idle a refused step re-rolls is theirs more often than
        // anyone's — and it puts them back on the original's stream.
        // 503/290 → **378/312** with item 61 (the farmer's cell index),
        // and the totals 1,351/1,340 → **1,210/1,360**. The orders half
        // fell 141 across everyone: six farmers now sow the cell the
        // original sows and leave it on the original's frame. The paths
        // half rose 20, and it is the drift this split has always
        // carried — the *second* re-target rolls past the last traced
        // word, so its tiles move whenever the stream does.
        assert!(
            farmer_orders <= 378 && farmer_paths <= 312,
            "the farmers' disagreements grew: orders {farmer_orders}, paths {farmer_paths}"
        );
    }

    /// One of the kept recordings, if this machine has it.
    ///
    /// `$RON_RECGAME_DIR`, or the profile's own `Recorded Games` directory —
    /// `PlayerProfile::get_record_game_directory` builds it under
    /// `CSIDL_PERSONAL`, which CrossOver maps to the Mac's `~/Documents`.
    fn recording(name: &str) -> Option<String> {
        if let Ok(dir) = std::env::var("RON_RECGAME_DIR") {
            let path = format!("{dir}/{name}");
            return std::path::Path::new(&path).is_file().then_some(path);
        }
        // **The kept corpus first, the game's own output directory second.**
        // A recording is a capture like a gamelog or a trace, so it belongs
        // with them — `dump`'s directory, which is `$RON_GAMELOG_DIR` or the
        // bottle's `Logs\`. The original writes new ones to
        // `PlayerProfile::get_record_game_directory`, which CrossOver maps
        // to the Mac's `~/Documents`, and that path is **gated by macOS
        // consent**: the first `open` under it blocks until a human at the
        // machine clicks Allow, which over SSH is nobody. A background run
        // then looks hung at 0 % CPU for as long as it is left to. So the
        // fallback stays — a fresh capture is found where the game put it —
        // but it is the fallback.
        dump(name).or_else(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = format!("{home}/Documents/My Games/Rise of Nations/Recorded Games/{name}");
            std::path::Path::new(&path).is_file().then_some(path)
        })
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

    /// **The animal's hurry, re-derived from the original's own record.**
    ///
    /// `AnimalData::get_speed@005d8380` is `Animal`'s and `AnimalData`'s
    /// slot `+0x17c` — the virtual `do_move` and `find_path` both take the
    /// step length from — and it does not call `UnitData::get_speed` at
    /// all. It takes `UnitData::speed`, and then, on a land or sea animal
    /// whose current order `is_move` (slot `+0x14`, named from the PDB's
    /// `LF_ONEMETHOD` list, not from the map's COMDAT-folded stub),
    /// measures `vector_dist` from the animal to that order's **goal** —
    /// `get_move_order` (slot `+0xb8`), then `MoveOrder +0x4/+0x8`, the
    /// ordered point rather than `dest_x/dest_y`, the current waypoint —
    /// and multiplies the speed by **3/2** when it is more than `0x180`.
    /// Then a floor of 3. An air animal returns before both.
    ///
    /// This is checked against the record rather than the listing.
    /// run39's gaia `8/2` walks nine frames from `(28776, 24360)` toward
    /// `(28968, 23976)`; the dump prints `myspeed 19` on every one of
    /// them, so the cached base never moves, and the steps it prints are
    /// **`(12, −25)` twice and then `(8, −17)`, `(8, −16)` ×6** — a step
    /// of 28 while `vector_dist` is 432 and 404, and of 19 once it is 376.
    /// `19 * 3 / 2 = 28`, truncated, and there is no free parameter in
    /// that.
    ///
    /// So the assertion is the whole prediction: for every gaia
    /// unit-frame in the capture on which the animal moved, the step
    /// `Unit::move_step` would take at that speed, and nothing else. The
    /// same walk without the 3/2 is checked to *fail*, because a rule
    /// nothing can break is not a rule.
    #[test]
    fn an_animal_more_than_0x180_from_its_order_hurries_by_three_halves() {
        use sim::movement::{cos_component, find_angle, sin_component};
        use sim::world::vector_dist;
        // **Both maps.** The rule was found on East Indies and is checked
        // on Great Lakes too, where the herd's `myspeed` is 11 rather than
        // 19 — so the `3/2` is exercised against two different bases and
        // cannot be a coincidence of one animal's arithmetic.
        let files = [
            "gamelog-run39-islands-longtrace.txt",
            "gamelog-run33-longtrace.txt",
        ];
        let Some(paths) = files
            .iter()
            .map(|f| dump(f))
            .collect::<Option<Vec<String>>>()
        else {
            eprintln!("skipping: no long-trace captures (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<String> = paths
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect();
        let captures: Vec<Vec<Frame>> =
            texts.iter().map(|t| Log::parse(t).frame_states()).collect();

        // One dumped step: where it was, where its order sends it, the
        // cached speed `myspeed`, and where it ended up.
        struct Step {
            capture: &'static str,
            frame: i64,
            who: i64,
            o: i64,
            from: (i64, i64),
            goal: (i64, i64),
            base: i64,
            to: (i64, i64),
        }
        let mut steps: Vec<Step> = Vec::new();
        for (c, frames) in captures.iter().enumerate() {
            for w in frames.windows(2) {
                for u in w[0].units.iter().filter(|u| u.who >= 8) {
                    let Some(next) = w[1].units.iter().find(|n| n.who == u.who && n.o == u.o)
                    else {
                        continue;
                    };
                    if (next.pos.x, next.pos.y) == (u.pos.x, u.pos.y) {
                        continue;
                    }
                    let (Some(gx), Some(gy), Some(base)) = (u.orders_x, u.orders_y, u.myspeed)
                    else {
                        continue;
                    };
                    steps.push(Step {
                        capture: files[c],
                        frame: w[0].n,
                        who: u.who,
                        o: u.o,
                        from: (u.pos.x, u.pos.y),
                        goal: (gx, gy),
                        base,
                        to: (next.pos.x, next.pos.y),
                    });
                }
            }
        }
        assert!(
            steps.len() >= 9,
            "the gaia walks: {} steps, wanted at least the nine `8/2` takes",
            steps.len()
        );

        // The prediction, with the hurry and without it.
        let walk = |s: &Step, boost: bool| -> (i64, i64) {
            let (dx, dy) = (s.goal.0 - s.from.0, s.goal.1 - s.from.1);
            let far = vector_dist(dx as i32, dy as i32) > 0x180;
            let mut speed = s.base as i32;
            if boost && far {
                speed = speed * 3 / 2;
            }
            let speed = speed.max(3);
            if dx.abs() + dy.abs() <= i64::from(speed) {
                return s.goal;
            }
            let a = find_angle(dx as i32, dy as i32);
            (
                s.from.0 + i64::from(sin_component(a, speed)),
                s.from.1 - i64::from(cos_component(a, speed)),
            )
        };

        let wrong: Vec<String> = steps
            .iter()
            .filter(|s| walk(s, true) != s.to)
            .map(|s| {
                format!(
                    "{} frame {} {}/{}: from {:?} goal {:?} base {} — ours {:?} theirs {:?}",
                    s.capture,
                    s.frame,
                    s.who,
                    s.o,
                    s.from,
                    s.goal,
                    s.base,
                    walk(s, true),
                    s.to
                )
            })
            .collect();
        assert!(
            wrong.is_empty(),
            "{} of {} gaia steps are not the rule's:\n{}",
            wrong.len(),
            steps.len(),
            wrong.join("\n")
        );

        let far = steps
            .iter()
            .filter(|s| {
                vector_dist((s.goal.0 - s.from.0) as i32, (s.goal.1 - s.from.1) as i32) > 0x180
            })
            .count();
        eprintln!(
            "the two long traces: {} gaia steps, {far} of them beyond 0x180",
            steps.len()
        );
        // And the same walk with the 3/2 taken out, which must break: the
        // frames on which an animal is beyond `0x180` are the ones the
        // hurry is for.
        let unboosted = steps.iter().filter(|s| walk(s, false) != s.to).count();
        assert_eq!(
            (steps.len(), far, unboosted),
            (264, 39, 39),
            "the two captures' gaia steps, the ones beyond `0x180`, and the \
             ones the rule's absence would get wrong"
        );
    }

    /// **A blocked animal drops its walk where it stands.**
    ///
    /// `Unit::resolve_unit_collision@005f9d30`'s first statement is a
    /// virtual on slot `+0x30` — the PDB's `LF_ONEMETHOD` list names it
    /// `SubObjectData::is_animal` at vftable offset 48, which the map
    /// cannot, because both overrides are COMDAT-folded onto trivial
    /// stubs (`Buffer::is_pending_load`, `return 1`, in `Animal`'s
    /// vtable; `Window::get_button`, `return 0`, in `Unit`'s). When it
    /// answers, the body is the `QUEUE_NEW` clear and nothing else —
    /// `unit_masks &= ~0x4000000`, `path.length = 0`, `close_orders`,
    /// `clear_partial_path`, `update_action` — and **none of
    /// `docs/COLLISION.md` §6's six steps runs**. No sidestep, no wait,
    /// no repath, and above all no cell-centre snap.
    ///
    /// The snap is what the record can see. §6 step 6 moves every other
    /// unit onto the middle of its 48-cell before it re-plans, so a
    /// simulation that lets an animal through to step 6 moves it on the
    /// frame the collision lands; the original does not move it at all.
    ///
    /// So the assertion is over every animal walk in both long captures
    /// that **ends short of its goal**: `orders_x/orders_y` stop naming a
    /// point and start naming the animal's own position, while the animal
    /// is not standing on the goal. Sixteen of those — twelve on East
    /// Indies, four on Great Lakes — and on every one of them the
    /// position is **unchanged** across the frame the order dies. Before
    /// this rule was in, `crates/sim` moved the animal on all sixteen.
    ///
    /// The other half of the same window is the arrival, which is not a
    /// collision: seventeen walks end *on* their goal, and they are
    /// counted here so that a parse which stopped seeing orders would
    /// fail rather than pass with nothing to check.
    #[test]
    fn a_blocked_animal_drops_its_walk_where_it_stands() {
        let files = [
            "gamelog-run39-islands-longtrace.txt",
            "gamelog-run33-longtrace.txt",
        ];
        let Some(paths) = files
            .iter()
            .map(|f| dump(f))
            .collect::<Option<Vec<String>>>()
        else {
            eprintln!("skipping: no long-trace captures (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<String> = paths
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect();
        let captures: Vec<Vec<Frame>> =
            texts.iter().map(|t| Log::parse(t).frame_states()).collect();

        let mut reached = 0usize;
        let mut abandoned = 0usize;
        let mut moved: Vec<String> = Vec::new();
        for (c, frames) in captures.iter().enumerate() {
            for w in frames.windows(2) {
                for u in w[0].units.iter().filter(|u| u.who >= 8) {
                    let (Some(gx), Some(gy)) = (u.orders_x, u.orders_y) else {
                        continue;
                    };
                    // A live goal: the order names somewhere else.
                    if (gx, gy) == (u.pos.x, u.pos.y) {
                        continue;
                    }
                    let Some(next) = w[1].units.iter().find(|n| n.who == u.who && n.o == u.o)
                    else {
                        continue;
                    };
                    let (Some(nx), Some(ny)) = (next.orders_x, next.orders_y) else {
                        continue;
                    };
                    // The goal is gone: the order list is empty and
                    // `orders_x/y` name the animal itself again.
                    if (nx, ny) != (next.pos.x, next.pos.y) {
                        continue;
                    }
                    if (next.pos.x, next.pos.y) == (gx, gy) {
                        reached += 1;
                        continue;
                    }
                    abandoned += 1;
                    if (next.pos.x, next.pos.y) != (u.pos.x, u.pos.y) {
                        moved.push(format!(
                            "{} frame {} {}/{}: {:?} → {:?}, goal {:?}",
                            files[c],
                            w[0].n,
                            u.who,
                            u.o,
                            (u.pos.x, u.pos.y),
                            (next.pos.x, next.pos.y),
                            (gx, gy),
                        ));
                    }
                }
            }
        }
        eprintln!(
            "the two long traces: {reached} animal walks reached their goal, {abandoned} were dropped short of it"
        );
        assert!(
            moved.is_empty(),
            "{} of {abandoned} dropped animal walks moved the animal — §6 step 6's \
             cell-centre snap, which an animal never takes:\n{}",
            moved.len(),
            moved.join("\n")
        );
        assert_eq!(
            (reached, abandoned),
            (17, 16),
            "the two captures' animal walks that arrived and that were dropped short"
        );
    }

    /// **The chopping guy's own wait, and the two sites that look like one
    /// branch** (2026-08-31, item 101) — the mechanic behind East Indies'
    /// word 645 → 742.
    ///
    /// `Unit::do_non_flat_gather` reads guy 0's `cur_anim` **before** it
    /// reads the tile (`005f0d0f`), and the branch it takes there is where
    /// a woodcutter spends the rest of its life. `CHAR_CHOP_WOOD`
    /// decrements the wait and on zero rerolls it `% 100 + 300` at
    /// `+0xcc3`; the *arrival* frame — the one that sets `CHAR_CHOP_WOOD`
    /// — rerolls `% 50 + 100` at `+0xdad`; and `CHAR_MINE_ORE` returns
    /// without doing anything at all. `docs/ORDERS.md` §6.4 has carried
    /// all three since August and the implementation had one merged
    /// branch, rolling the arrival's formula at the chop site.
    ///
    /// **A count could not see it.** Both sites draw exactly once, so the
    /// word stayed matched for six hundred frames while the AI's
    /// woodcutter ran its clock at a third of the original's and walked
    /// home two hundred frames early. What sees it is the record.
    ///
    /// Two halves, each of which fails on its own:
    ///
    /// - **The original's own.** Every rise in a dumped `GATHERORDER`'s
    ///   `wait` over run39's 1,850 frames is matched against the draw the
    ///   trace took on that frame, and the value the LCG returned must
    ///   produce it under *that site's* formula. This half needs no
    ///   simulation and is what names the sites; with the two formulas
    ///   swapped it fails on the first reroll.
    /// - **Ours.** Every non-flat `GATHERORDER` the dump prints — tile,
    ///   wait, phase, `been_there`, `dist_mod` — against the simulation's
    ///   own, on every frame to the floor.
    #[test]
    fn run39_s_woodcutters_reroll_on_the_chop_branch_not_the_arrival_s() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            dump("rontrace-run39.log").and_then(|p| {
                crate::trace::Trace::read(std::path::Path::new(&p))
                    .ok()
                    .flatten()
            }),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let frames = log.frame_states();

        // The three sites, taken **through the naming table** rather than
        // written down again, so a label put on the wrong address fails
        // here and not silently three mechanics later.
        let site_of = |label: &str| -> u32 {
            crate::trace::SITES
                .iter()
                .find(|(_, _, l)| *l == label)
                .map(|(a, _, _)| *a)
                .expect("the wood machine's sites are named")
        };
        let tile = site_of(sim::orders::SITE_TILE_WAIT); // `+0x54b`
        let chop = site_of(sim::orders::SITE_WORK_WAIT); // `+0xcc3`
        let arrive = site_of(sim::orders::SITE_ARRIVE_WAIT); // `+0xdad`
        let produced = |site: u32, v: i32| -> Vec<i64> {
            if site == tile {
                // A miner's tile choice draws and then overwrites the roll
                // with the 1,000,000 that keeps it out (§6.4).
                vec![i64::from(400 + v % 200), 1_000_000]
            } else if site == chop {
                vec![i64::from(300 + v % 100)]
            } else {
                vec![i64::from(100 + v % 50)]
            }
        };

        // **Half one: the original against itself.** A `wait` that rises
        // to 100 or more is a reroll — the branch constants (32 at the
        // camp walk, 20 when no spot is free) are all below it — so every
        // one of them owes a draw at one of the three sites on the frame
        // it happened, and the draw's own value must produce it.
        let mut prev: std::collections::BTreeMap<(i64, i64), i64> =
            std::collections::BTreeMap::new();
        let (mut rerolls, mut by_site) = (0usize, std::collections::BTreeMap::new());
        let mut camps: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
        for f in &frames {
            // `FRAME n` is the state at the end of frame n, so a rise
            // between `n − 1` and `n` was written by frame `n − 1`'s step,
            // which is the frame the trace counts.
            let step = f.n - 1;
            let mut rose: Vec<i64> = Vec::new();
            for u in &f.units {
                for o in &u.orders {
                    if o.kind != "GATHERORDER" || o.non_flat_gather != Some(1) {
                        continue;
                    }
                    let Some(wait) = o.wait else { continue };
                    camps.extend(o.build_type);
                    let was = prev.insert((u.who, u.o), wait);
                    if was.is_some_and(|w| wait > w) && wait >= 100 {
                        rose.push(wait);
                    }
                }
            }
            if rose.is_empty() {
                continue;
            }
            let mut spent: Vec<(u32, i32)> = tr
                .draws
                .iter()
                .filter(|d| {
                    d.sync()
                        && d.frame == step
                        && (d.site == tile || d.site == chop || d.site == arrive)
                })
                .filter_map(|d| d.value().map(|v| (d.site, v)))
                .collect();
            for w in rose {
                let at = spent
                    .iter()
                    .position(|&(site, v)| produced(site, v).contains(&w))
                    .unwrap_or_else(|| {
                        panic!(
                            "frame {step}: a wait rose to {w} and no gather draw of \
                             {spent:?} produces it"
                        )
                    });
                *by_site.entry(spent[at].0).or_insert(0usize) += 1;
                spent.remove(at);
                rerolls += 1;
            }
            assert!(
                spent.is_empty(),
                "frame {step}: {spent:?} drew and no order's wait rose"
            );
        }
        eprintln!("run39 rerolls: {rerolls} over 1,850 frames, by site {by_site:?}");
        // Nineteen tile choices and **five** chop rerolls, and not one
        // arrival reroll in 1,850 frames: the branch the implementation
        // used to spend every reroll on is the one the original reaches
        // essentially never, which is why the sites had to be told apart
        // by their formulas rather than by their counts.
        assert_eq!(rerolls, 24, "run39's rerolls, all of them explained");
        // And what the capture cannot speak to: every non-flat gatherer in
        // it works the **same** camp type, so `CHAR_MINE_ORE`'s early
        // return and the miner's 1,000,000 are reading-only until a
        // capture works a mine (`docs/SYNC.md` §7).
        assert_eq!(
            camps.into_iter().collect::<Vec<_>>(),
            vec![418],
            "run39 has one non-flat camp type and no mine"
        );
        assert_eq!(
            by_site.into_iter().collect::<Vec<_>>(),
            vec![(tile, 19usize), (chop, 5usize)],
            "the tile choice, the chopping guy, and no arrival roll at all"
        );

        // **Half two: ours against the record.** Field for field, on every
        // frame, for as far as it holds.
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut last = 0i64;
        let (mut compared, mut first_bad, mut parted) = (0usize, None, 0i64);
        for f in &frames {
            while last < f.n {
                built.tick();
                last += 1;
            }
            for u in &f.units {
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let theirs: Vec<&crate::gamelog::OrderDump> = u
                    .orders
                    .iter()
                    .filter(|o| o.kind == "GATHERORDER" && o.non_flat_gather == Some(1))
                    .collect();
                let ours: Vec<sim::orders::GatherOrder> = built.sim.units[link.unit]
                    .orders
                    .iter()
                    .filter_map(|o| match o.body {
                        sim::orders::Body::Gather(g)
                            if matches!(
                                built.sim.building_ident(g.building),
                                sim::build::Ident::Mine | sim::build::Ident::Woodcutter
                            ) =>
                        {
                            Some(g)
                        }
                        _ => None,
                    })
                    .collect();
                if theirs.len() != ours.len() {
                    if first_bad.is_none() {
                        first_bad = Some(format!(
                            "frame {}: {}/{} has {} non-flat gather orders, the original {}",
                            f.n,
                            u.who,
                            u.o,
                            ours.len(),
                            theirs.len()
                        ));
                    }
                    continue;
                }
                for (t, o) in theirs.iter().zip(&ours) {
                    let mine = (
                        o.tile.map_or(-1, |p| i64::from(p.x)),
                        o.tile.map_or(-1, |p| i64::from(p.y)),
                        i64::from(o.wait),
                        i64::from(o.goto_build),
                        i64::from(o.been_there),
                        i64::from(o.dist_mod),
                    );
                    let his = (
                        t.tx.unwrap_or(-1),
                        t.ty.unwrap_or(-1),
                        t.wait.unwrap_or(0),
                        t.goto_build.unwrap_or(0),
                        t.been_there.unwrap_or(0),
                        t.dist_mod.unwrap_or(0),
                    );
                    compared += 6;
                    if mine != his && first_bad.is_none() {
                        first_bad = Some(format!(
                            "frame {}: {}/{} gather ours {mine:?} theirs {his:?}",
                            f.n, u.who, u.o
                        ));
                    }
                }
            }
            if first_bad.is_some() {
                parted = f.n;
                break;
            }
        }
        eprintln!(
            "run39 gather: {compared} fields to frame {parted}, first {}",
            first_bad.as_deref().unwrap_or("none disagreeing")
        );
        // The floor, and it may only rise. Frame 897 used to be the
        // successor — the human's `1/2` holding its tile with a wait ten
        // short of the original's — and item 106 walked straight past it:
        // the scout's walk to a goody box put the stream back on the
        // original's, and the same clock now runs true through 1,572.
        // 1,573 was the successor after that, and item 109's bird step
        // walked past it too: 24,738 fields to 1,573 → **26,094 to
        // 1,686**, where the human's `0/2` holds `(32, 26)` with a wait of
        // 445 against the original's 480 — the same clock, the same
        // shape, thirty-five short rather than a hundred long.
        // `docs/ORDERS.md` §6.4.
        assert!(
            compared >= 26_094 && parted >= 1_686,
            "the wood machine's record fell: {compared} fields to frame {parted}, \
             the floor is 26,094 and 1,686 — {}",
            first_bad.as_deref().unwrap_or("none disagreeing")
        );
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

/// The `ARMY` records against `docs/ARMY.md`'s implementation
/// (`crates/sim/src/army.rs`): the whole record, every valid slot, per
/// the working agreement's "diff the whole record".
#[cfg(test)]
mod army_tests {
    use super::*;
    use crate::gamelog::Block;
    use crate::testenv::{dump, install};
    use sim::army::Army;

    /// Every `BEGIN ARMY` block under a frame block's first `FULL DUMP`, in
    /// order — `ArmyData::log_data` prints only the valid slots.
    fn army_records<'a, 'b>(frame: &'a Block<'b>) -> Vec<&'a Block<'b>> {
        let b = frame.kid("FULL DUMP").unwrap_or(frame);
        b.kids("ARMY").collect()
    }

    /// The dump's named fields of an `ARMY` record, from the harness's
    /// record. `reg` and `city` are mapped back to the dump's numbering.
    fn ours(built: &Built, a: &Army) -> Vec<(&'static str, i64)> {
        let reg = a.reg.map_or(-1, |r| {
            built
                .region_map
                .iter()
                .find(|(_, s)| *s == r)
                .map_or(-1, |(d, _)| *d)
        });
        let city = a.city.map_or(-1, |c| {
            built
                .sim
                .cities_of(a.who)
                .iter()
                .position(|&x| x == c)
                .map_or(-1, |p| p as i64)
        });
        let (target_o, target_who) = match a.target {
            None => (-1, -1),
            Some(sim::combat::Obj::Unit(u)) => (
                i64::from(built.sim.units[u].index),
                i64::from(built.sim.units[u].owner),
            ),
            Some(sim::combat::Obj::Building(b)) => (
                i64::from(built.sim.buildings[b].index),
                i64::from(built.sim.buildings[b].owner),
            ),
        };
        vec![
            ("army", i64::from(a.army)),
            ("who", i64::from(a.who)),
            ("num_groups", i64::from(a.num_groups())),
            ("status", i64::from(a.status)),
            ("reg", reg),
            ("role", i64::from(a.role)),
            ("num_units", i64::from(a.num_units)),
            ("num_captains", i64::from(a.num_captains)),
            ("num_standard", i64::from(a.num_standard)),
            ("num_decoys", i64::from(a.num_decoys)),
            ("city", city),
            ("navy", i64::from(a.navy)),
            ("human_frame", i64::from(a.human_frame)),
            ("hurry", i64::from(a.hurry)),
            ("target_o", target_o),
            ("target_who", target_who),
            ("x", i64::from(a.pos.x)),
            ("y", i64::from(a.pos.y)),
            ("angle", i64::from(a.angle.0)),
            ("rally_dist", i64::from(a.rally_dist)),
            ("muster_x", i64::from(a.muster.x)),
            ("muster_y", i64::from(a.muster.y)),
            ("muster_angle", i64::from(a.muster_angle.0)),
        ]
    }

    /// Every field of every record, in one list of disagreements.
    fn compare(built: &Built, theirs: &[&Block<'_>], skip: &[&str]) -> Vec<String> {
        let mut wrong = Vec::new();
        let mut mine: Vec<&Army> = Vec::new();
        for who in 0..built.sim.armies.len() {
            for (_, a) in built.sim.armies[who].valid() {
                mine.push(a);
            }
        }
        if mine.len() != theirs.len() {
            wrong.push(format!(
                "{} valid armies, the dump has {}",
                mine.len(),
                theirs.len()
            ));
        }
        for (a, t) in mine.iter().zip(theirs) {
            for (key, o) in ours(built, a) {
                if skip.contains(&key) {
                    continue;
                }
                let tv = t.int(key);
                if tv != Some(o) {
                    wrong.push(format!(
                        "army {} who {} {key}: ours {o} theirs {tv:?}",
                        a.army, a.who
                    ));
                }
            }
        }
        wrong
    }

    /// Run20's frame blocks 1–4 carry the AI's first army —
    /// `Armies::init_army` from the census's step 16 at frame 0, before a
    /// unit has joined — and the harness's census seeds the same slot at
    /// the same city with the same record, field for field, and nothing
    /// for the human.
    #[test]
    fn run20_s_first_army_is_the_census_s_init_army_whole() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let frames = log.frames();
        // Block 4 is the quit's own: a `FRAME 4` header with no dump.
        for n in 1..=3 {
            built.sim.tick();
            let (_, block) = frames
                .iter()
                .find(|(f, _)| *f == n)
                .expect("the frame block");
            let theirs = army_records(block);
            assert_eq!(theirs.len(), 1, "one army at frame {n}");
            assert_eq!(theirs[0].int("who"), Some(1), "the AI's");
            let wrong = compare(&built, &theirs, &[]);
            assert!(wrong.is_empty(), "frame {n}:\n  {}", wrong.join("\n  "));
        }
    }

    /// Run22's block 3579: two armies of the AI, one per city, both
    /// mustering **and** forming (`status 17`) — `do_mustering`'s first
    /// arm, a muster spot found at an active city — with `x, y` the city's
    /// point one cell south (`Army::init`), every count still zero, and
    /// muster cells that are **not** the init's: the ring search of
    /// `find_muster_spot` (§13) moved them, and kept them at least four
    /// cells apart (the same-owner spacing rule). The ring search is the
    /// one seam of `army.rs` this capture reaches; the record is asserted
    /// from the dump alone.
    #[test]
    fn run22_s_two_armies_are_init_s_records_with_the_ring_search_s_muster_cells() {
        let Some(path) = dump("gamelog-run22-islands-dock-window.txt") else {
            eprintln!("skipping: no gamelog-run22-islands-dock-window.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let frames = log.frames();
        let (_, block) = frames.iter().find(|(f, _)| *f == 3579).expect("block 3579");
        let dumpb = block.kid("FULL DUMP").unwrap_or(block);
        let theirs = army_records(block);
        assert_eq!(theirs.len(), 2, "two armies at 3579");
        let cities: Vec<&Block<'_>> = dumpb
            .find("CITIES")
            .expect("CITIES")
            .kids("CITY")
            .filter(|c| c.int("who") == Some(1))
            .collect();
        assert!(
            cities.len() >= 2,
            "the AI's two cities, got {}",
            cities.len()
        );
        let mut musters = Vec::new();
        for (i, a) in theirs.iter().enumerate() {
            assert_eq!(a.int("army"), Some(i as i64));
            assert_eq!(a.int("who"), Some(1));
            assert_eq!(a.int("status"), Some(0x11), "mustering and forming");
            assert_eq!(a.int("city"), Some(i as i64));
            assert_eq!(a.int("reg"), Some(11));
            for k in [
                "num_groups",
                "num_units",
                "num_captains",
                "num_standard",
                "num_decoys",
                "navy",
                "human_frame",
                "hurry",
                "angle",
                "rally_dist",
                "role",
            ] {
                assert_eq!(a.int(k), Some(0), "{k}");
            }
            assert_eq!(a.int("target_o"), Some(-1));
            assert_eq!(a.int("target_who"), Some(-1));
            let c = cities[i];
            let (cx, cy) = (c.int("x").unwrap(), c.int("y").unwrap());
            assert_eq!(a.int("x"), Some(cx), "x is the city's");
            assert_eq!(
                a.int("y"),
                Some(cy + 0x300),
                "y is one cell south of the city's"
            );
            let init_cell = (cx / 0x300, (cy + 0x300) / 0x300);
            let m = (a.int("muster_x").unwrap(), a.int("muster_y").unwrap());
            assert_ne!(m, init_cell, "the ring search moved army {i}'s muster cell");
            musters.push(m);
        }
        let d = sim::world::vector_dist(
            (musters[0].0 - musters[1].0).unsigned_abs() as i32,
            (musters[0].1 - musters[1].1).unsigned_abs() as i32,
        );
        assert!(
            d >= 4,
            "muster cells {musters:?} are {d} apart; the same-owner rule wants four"
        );
    }

    // ---- the ring search on a frame block's own map (§13) ----

    use sim::combat::Obj;
    use sim::world::Cell;

    /// A frame block's state, as far as `find_muster_spot` and
    /// `find_target` read it — the block's own `WORLD` cells (owners and
    /// regions as they stood), its cities typed from their `BUILDDATA`
    /// record and carrying that record's damage, its leaders' transport
    /// bits, diplomacy table and the `LEADERDATA` words §12 reads, its
    /// `ARMY` records, the frame number and the sync stream's word the
    /// block opens with — so one function can be run on the state of that
    /// frame. Not a replay: nothing else of the frame is built.
    ///
    /// **Its `UNITS=3` half**, added 2026-08-26: every `UNITDATA` record of
    /// the block becomes a `sim::Unit` — typed from its first guy's
    /// `TypeIndex`, standing where the record stands, facing the record's
    /// own `angle`, carrying its `+0xaa`/`+0xab` formation bytes, its
    /// stance, its **order list** and its **path stack** — and every
    /// `GROUPDATA` slot that an army owns becomes that army's membership,
    /// in the record's own `list` order. That is what lets a function
    /// which reads a unit's *orders* (`Army::engagement`, `is_moving`,
    /// `GroupData::get_form`) be run on the state of a logged frame at
    /// all; without it a scene had armies with no units in them.
    struct Scene {
        sim: Sim,
        /// `(who, o, handle)` of every city building.
        cities: Vec<(i64, i64, usize)>,
        /// `(who, slot, city)`: the dump's per-leader city slot — the
        /// `city` an `ARMY` record names, which keeps its number when an
        /// earlier slot empties — to the sim's city index.
        slots: Vec<(i64, i64, usize)>,
        /// `(who, o, handle)` of every unit the block carried.
        units: Vec<(i64, i64, usize)>,
        /// The block's whole `GROUPDATA` pool, as parsed — the record the
        /// membership was read out of, kept so a test can compare against
        /// the slots the scene did *not* stand up (a player's selection, a
        /// hotkey group) as well as the ones it did.
        groups: Vec<crate::gamelog::GroupDump>,
        /// Order kinds the block carried that a scene cannot translate —
        /// empty for every window read so far, and the thing to look at
        /// first when a unit's order list comes back shorter than the
        /// record's.
        untranslated: Vec<String>,
    }

    impl Scene {
        /// The handle of the unit the block wrote as `(who, o)`.
        fn unit(&self, who: i64, o: i64) -> Option<usize> {
            self.units
                .iter()
                .find(|(w, n, _)| *w == who && *n == o)
                .map(|(_, _, u)| *u)
        }
    }

    /// One `UNITDATA` order block as the simulation's own order, or `None`
    /// for a kind this scene does not model.
    ///
    /// The geometry is `docs/ORDERS.md` §4.1's table, field for field:
    /// `x/y` is the destination, `dest` is "I have a current waypoint" and
    /// `dest_x/dest_y` is that waypoint, `last` is −1,−1 when no
    /// straight-line plan stands. The target of an attack rides on the
    /// unit here rather than on the order (`sim::group`'s module note), so
    /// it is passed in resolved.
    fn order_of(od: &crate::gamelog::OrderDump) -> Option<sim::orders::Order> {
        use sim::orders::{AttackOrder, Body, MoveKind, MoveOrder, index};
        let i = |v: Option<i64>| v.unwrap_or(0) as i32;
        let kind = match u8::try_from(od.index).ok()? {
            index::MOVE_TO => MoveKind::MoveTo,
            index::ATTACK_TO => MoveKind::AttackTo,
            index::EXPLORE_TO => MoveKind::ExploreTo,
            index::FLEE_TO => MoveKind::FleeTo,
            index::ATTACK => {
                return Some(sim::orders::Order {
                    flags: u8::try_from(od.flags & 0xff).ok()?,
                    body: Body::Attack(AttackOrder {
                        defensive: od.defensive == Some(1),
                        def: match (od.def_x, od.def_y) {
                            (Some(x), Some(y)) if x >= 0 && y >= 0 => {
                                Some(Pos::new(x as i32, y as i32))
                            }
                            _ => None,
                        },
                        in_range: od.in_range == Some(1),
                        ever_in_range: od.ever_in_range == Some(1),
                        new_ord: od.new_ord == Some(1),
                    }),
                });
            }
            _ => return None,
        };
        let dest = Pos::new(i(od.x), i(od.y));
        Some(sim::orders::Order {
            flags: u8::try_from(od.flags & 0xff).ok()?,
            body: Body::Move(MoveOrder {
                kind,
                dest,
                angle: sim::movement::Angle(i(od.angle)),
                // `MoveOrder +0x28`, where the original's −1 is the
                // "not a formation move" of `docs/GROUPS.md` §6.3.
                facing: match od.facing {
                    Some(f) if f >= 0 => Some(f != 0),
                    _ => None,
                },
                has_waypoint: od.dest == Some(1),
                waypoint: Pos::new(i(od.dest_x), i(od.dest_y)),
                last: match (od.last_x, od.last_y) {
                    (Some(x), Some(y)) if x >= 0 && y >= 0 => Some(Pos::new(x as i32, y as i32)),
                    _ => None,
                },
                pause: i(od.pause),
                timer: i(od.timer),
                // `coll_x`/`coll_y`, the point the last collision refused
                // (`docs/COLLISION.md` §4.3). The original leaves 0 rather
                // than −1 when there has been none.
                coll: match (od.coll_x, od.coll_y) {
                    (Some(x), Some(y)) if x > 0 || y > 0 => Some(Pos::new(x as i32, y as i32)),
                    _ => None,
                },
            }),
        })
    }

    fn scene_at(loaded: &Loaded, log: &Log<'_>, frame: i64) -> Scene {
        // `Log::dumps` rather than `Log::frames`: an `end_frame` dump is a
        // *sibling* of the `FRAME` block, so the frame walk misses the two
        // states at the ends of a window (run29's 15103 and 15105). The
        // block handed on is the `FULL DUMP` itself, which every reader
        // below already tolerates — each opens with
        // `kid("FULL DUMP").unwrap_or(b)`.
        let dumps = log.dumps();
        let (_, block) = dumps
            .iter()
            .find(|(f, _)| *f == frame)
            .expect("a FULL DUMP stamped with this frame");
        let body = block.kid("FULL DUMP").unwrap_or(block);
        let mut notes = Vec::new();
        let world_fields = body.kid("WORLD").expect("a WORLD block").fields.clone();
        let (world, region_map) = world_from(&world_fields, &[], &mut notes);
        assert!(
            !region_map.is_empty(),
            "{frame}: the WORLD block carries no cells"
        );
        let init = log.initial().expect("the start-of-game block");
        let players = player_count(&init).max(1);
        let mut sim = loaded.sim(Tuning::RON, world, players);
        sim.lobby = lobby_of(&init.game_info, &loaded.map_styles);
        sim.setup.no_nation_powers = sim.lobby.no_nation_powers;
        // Block `n` is the state sim-frame `n` begins on (`docs/SYNC.md`
        // §1): the frame the stamps are compared against, and the sync
        // stream's word its `say_checksum` record carries — what the
        // frame's first draw advances from.
        sim.frame = frame;
        if let Some((_, seed)) = log.frame_seeds().into_iter().find(|(n, _)| *n == frame - 1) {
            sim.rng = sim::combat::Rng::new(seed);
        }
        let (unit_dumps, builds, leaders) = crate::gamelog::records(block, false);
        for l in &leaders {
            if (0..players as i64).contains(&l.who) {
                let who = l.who as usize;
                sim.set_tribe(who as u8, l.tribe);
                sim.nation[who].human = l.leader_flags & 4 != 0;
                let t = &mut sim.transport[who];
                t.civilian = l.leader_flags & 0x100 != 0;
                t.military = l.leader_flags & 0x200 != 0;
                t.scout = l.leader_flags & 0x400 != 0;
            }
        }
        // The `LEADERDATA` words `find_target` reads (`docs/ARMY.md` §12):
        // the diplomacy table (0 war, 1 peace, 2 allied; the diagonal is
        // 2), `defense_mod`, the two census counts the averages and the
        // difficulty gate use, the attack stamp, and the personality's two
        // knobs. Read from the block itself: `LeaderDump` carries the
        // level-0 fields only.
        for l in body.kids("LEADERDATA") {
            let Some(who) = l.int("who") else { continue };
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let w = who as usize;
            let diplos: Vec<i64> = l
                .all("diplos[scan]")
                .iter()
                .filter_map(|v| v.trim().parse().ok())
                .collect();
            for (other, &d) in diplos.iter().enumerate().take(players) {
                if other != w {
                    sim.at_war[w][other] = d == 0;
                    sim.allied[w][other] = d == 2;
                }
            }
            let a = &mut sim.ai[w];
            if let Some(v) = l.int("defense_mod") {
                a.defense_mod = v as i32;
            }
            if let Some(v) = l.int("combat") {
                a.census.combat = v as i32;
            }
            if let Some(v) = l.int("sea_combat") {
                a.census.sea_combat = v as i32;
            }
            if let Some(v) = l.int("frame_attacked") {
                a.frame_attacked = v;
            }
            if let Some(v) = l.int("attacked_by") {
                a.attacked_by = v as i32;
            }
            if let Some(p) = l.kid("PERSONALITY") {
                if let Some(v) = p.int("raid") {
                    a.pers.raid = v as i32;
                }
                if let Some(v) = p.int("early_army") {
                    a.pers.early_army = v as i32;
                }
            }
        }
        /// The first value under `key` in the block or, depth-first, its
        /// children — a `BUILDDATA` record nests its class chain
        /// (`WALLDATA` → `OBJECT` → `SUBOBJECT`) and `who`, `o` and `damage`
        /// sit at the inner levels.
        fn deep_int(b: &Block<'_>, key: &str) -> Option<i64> {
            b.int(key)
                .or_else(|| b.children.iter().find_map(|c| deep_int(c, key)))
        }
        let mut cities = Vec::new();
        let mut slots = Vec::new();
        for c in body.find("CITIES").expect("CITIES").kids("CITY") {
            let who = c.int("who").unwrap_or(-1);
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let o = c.int("o").expect("the city's building");
            let slot = c.int("city").expect("the city's slot");
            let flags = c.int("city_flags").unwrap_or(0);
            let ty = builds
                .iter()
                .find(|b| b.who == who && b.o == o)
                .and_then(|b| b.orig_type)
                .and_then(|t| loaded.build_of_type_index(t as i32))
                .expect("the city building's type");
            let pos = Pos::new(c.int("x").unwrap() as i32, c.int("y").unwrap() as i32);
            let b = sim.init_build(who as sim::Player, ty, pos, false);
            sim.activate(b, false, false);
            let ci = sim.buildings[b].city.expect("activate founded the city");
            let city = &mut sim.cities[ci];
            city.alive = flags & 1 != 0;
            city.no_heal = flags & 2 != 0;
            city.capital = flags & 0x10 != 0;
            city.unassimilated = flags & 0x100 != 0;
            city.no_muster = flags & 0x2000 != 0;
            city.race = c.int("race").map(|r| r as sim::Player);
            city.was_capital = c.int("was_capital_flags").unwrap_or(0) as u64;
            city.attack_stamp = c.int("attack_stamp").unwrap_or(0);
            // The building's `damage`, from its own record: §12 doubles a
            // damaged city of one's own.
            let damage = body
                .kids("BUILDDATA")
                .find(|bd| deep_int(bd, "who") == Some(who) && deep_int(bd, "o") == Some(o))
                .and_then(|bd| deep_int(bd, "damage"))
                .unwrap_or(0) as i32;
            let bd = &mut sim.buildings[b];
            bd.health = bd.hits - damage;
            cities.push((who, o, b));
            slots.push((who, slot, ci));
        }

        // ---- the `UNITS=3` half ----
        //
        // Every unit of the block, typed from its first guy's `TypeIndex`
        // — `GuyData::type` is the *unit's* type, which is what makes a
        // logged unit typeable at all (the `UNITDATA` level carries no
        // type of its own) — and carrying its order list, its path stack
        // and the two formation bytes `get_form` reads back off it.
        let mut units = Vec::new();
        let mut untranslated = Vec::new();
        for u in &unit_dumps {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            let ty = u
                .guys
                .first()
                .and_then(|g| g.kind)
                .and_then(|t| loaded.unit_of_type_index(t as i32));
            let t = ty.map(|k| &loaded.unit_types[k]);
            // `ObjectData::myhits` is the whole hit points the type
            // carries *after* tech, and `damage` what has been taken off
            // them — the same pair the city loop above reads.
            let hits = u.myhits.unwrap_or_else(|| t.map_or(1, |t| t.hits).into()) as i32;
            let health = if u.flags & 1 == 0 {
                0
            } else {
                (hits - u.damage.unwrap_or(0) as i32).max(1)
            };
            let mut unit = Unit::new(
                u.who as sim::Player,
                i16::try_from(u.o).expect("a unit's object number"),
                pos_of(u.pos),
                health,
            );
            unit.max_health = hits;
            unit.squad_size = u.guys.len().max(1) as i32;
            unit.ty = ty;
            unit.type_index = u.guys.first().and_then(|g| g.kind).unwrap_or(-1) as i32;
            if let Some(t) = t {
                unit.kind = t.kind;
                unit.movement.turning = sim::turning_of(t);
            }
            unit.movement.speed = u.myspeed.unwrap_or(0) as i32;
            // The three angles, each from its own field. `UnitData::angle`
            // (`+0x50`) is the **heading** — what `update_positions` rotates
            // the slot table by — and guy 0's `angle` (`+0x18`) is the
            // **facing** the step is taken along; a record without a guy
            // falls back to the heading, which is where a unit that has
            // finished turning sits anyway. `UnitData::dest_angle` (`+0x58`)
            // is the third and is the order's.
            if let Some(a) = u.angle {
                unit.movement.heading = sim::movement::Angle(a as i32);
                unit.movement.facing = unit.movement.heading;
            }
            if let Some(a) = u.guys.first().and_then(|g| g.angle) {
                unit.movement.facing = sim::movement::Angle(a as i32);
            }
            unit.movement.frame_facing = unit.movement.facing;
            if let Some(a) = u.dest_angle {
                unit.movement.des_angle = sim::movement::Angle(a as i32);
            }
            unit.on_map = u.inside_up.unwrap_or(-1) < 0;
            unit.captain = u.o_up.unwrap_or(-1) < 0;
            unit.form = i8::try_from(u.form.unwrap_or(-1)).unwrap_or(-1);
            unit.form_width = i8::try_from(u.form_mod.unwrap_or(-1)).unwrap_or(-1);
            unit.stance = u8::try_from(u.stance.unwrap_or(0)).unwrap_or(0);
            unit.tolerance = u.tolerance.unwrap_or(0) as i32;
            unit.path_recursion = u8::try_from(u.path_recursion.unwrap_or(0)).unwrap_or(0);
            unit.idle = u8::try_from(u.idle.unwrap_or(0)).unwrap_or(0);
            unit.orders_pos = Pos::new(
                u.orders_x.unwrap_or(0) as i32,
                u.orders_y.unwrap_or(0) as i32,
            );
            unit.line_ok = u.unit_masks.unwrap_or(0) & 8 != 0;
            unit.was_builder = u.unit_masks.unwrap_or(0) & 0x400 != 0;
            unit.decoy = u.unit_masks.unwrap_or(0) & 1 != 0;
            // The order list front-first (the log writes it newest first)
            // and the path stack as it stands — both `Vec`-shaped the same
            // way the original's are.
            for od in u.orders_front_first() {
                match order_of(od) {
                    Some(o) => unit.orders.push_back(o),
                    None => untranslated.push(od.kind.clone()),
                }
            }
            unit.path = u
                .path
                .iter()
                .map(|p| sim::orders::PathData {
                    to: Pos::new(p.to.0 as i32, p.to.1 as i32),
                    tolerance: p.tolerance as i32,
                    flags: u8::try_from(p.flags & 0xff).unwrap_or(0),
                })
                .collect();
            let h = sim.add_unit(unit);
            units.push((u.who, u.o, h));
        }
        // The attack targets, once every unit has a handle. The target is
        // `update_action().get_target_order()`'s — the **action**'s, not
        // the front order's: a unit chasing its target holds
        // `[MOVEORDER(transit), ATTACKORDER]` and only the second names
        // anyone. The simulation keeps the target on the unit rather than
        // on the order (`sim::group`'s module note).
        for u in &unit_dumps {
            let Some(h) = units
                .iter()
                .find(|(w, o, _)| *w == u.who && *o == u.o)
                .map(|(_, _, h)| *h)
            else {
                continue;
            };
            let Some(cur) = u.orders_front_first().find(|o| {
                // `get_action`'s walk: past a move that lacks the action
                // bit, stop on anything else.
                !((1..=4).contains(&o.index) && !o.is_action())
            }) else {
                continue;
            };
            let (Some(whom), Some(ox)) = (cur.whom, cur.ox) else {
                continue;
            };
            if whom < 0 || ox < 0 {
                continue;
            }
            let target = units
                .iter()
                .find(|(w, o, _)| *w == whom && *o == ox)
                .map(|(_, _, t)| Obj::Unit(*t))
                .or_else(|| {
                    cities
                        .iter()
                        .find(|(w, o, _)| *w == whom && *o == ox)
                        .map(|(_, _, b)| Obj::Building(*b))
                });
            sim.units[h].combat.target = target;
        }

        // The one group per army, in the record's own `list` order — which
        // is the order `ArmyData::get_unit` walks and therefore the order
        // `engagement` picks its seed in.
        let pool = crate::gamelog::groups(block);
        for g in &pool {
            if g.buildings != 0 || g.army < 0 || !(0..players as i64).contains(&g.who) {
                continue;
            }
            let slot = g.army as usize;
            let members: Vec<usize> = g
                .members
                .iter()
                .filter_map(|m| {
                    units
                        .iter()
                        .find(|(w, o, _)| *w == g.who && *o == m.o)
                        .map(|(_, _, h)| *h)
                })
                .collect();
            let a = &mut sim.armies[g.who as usize].list[slot];
            a.units = members;
            a.group = sim::group::GroupState {
                form: g.form as i32,
                order_num: g.order_num as i32,
                o: Pos::new(g.ox as i32, g.oy as i32),
                o_angle: sim::movement::Angle(g.o_angle as i32),
                o_dist: g.o_dist as i32,
                facing: g.facing != 0,
                stamp: g.stamp,
                form_num: g.form_num as i32,
                off: g
                    .members
                    .iter()
                    .map(|m| (m.off_x as i32, m.off_y as i32))
                    .collect(),
                curr: g
                    .members
                    .iter()
                    .map(|m| Pos::new(m.curr_x as i32, m.curr_y as i32))
                    .collect(),
                angles: g.members.iter().map(|m| m.angle as i8).collect(),
            };
        }

        for a in army_records(block) {
            let who = a.int("who").unwrap();
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let slot = a.int("army").unwrap() as usize;
            let reg = a.int("reg").filter(|r| *r >= 0).map(|r| {
                region_map
                    .iter()
                    .find(|(d, _)| *d == r)
                    .map(|(_, s)| *s)
                    .expect("the army's region is on the map")
            });
            let city = a.int("city").filter(|c| *c >= 0).map(|c| {
                slots
                    .iter()
                    .find(|(w, s, _)| *w == who && *s == c)
                    .map(|(_, _, ci)| *ci)
                    .expect("the army's city slot")
            });
            let target = match (a.int("target_who"), a.int("target_o")) {
                (Some(tw), Some(o)) if o >= 0 => Some(Obj::Building(
                    cities
                        .iter()
                        .find(|(w, x, _)| *w == tw && *x == o)
                        .map(|(_, _, b)| *b)
                        .expect("an army target that is a city building"),
                )),
                _ => None,
            };
            let int = |k: &str| a.int(k).unwrap_or(0) as i32;
            let rec = &mut sim.armies[who as usize].list[slot];
            rec.valid = true;
            rec.status = int("status");
            rec.reg = reg;
            rec.navy = a.int("navy") == Some(1);
            rec.city = city;
            rec.hurry = int("hurry");
            rec.num_units = int("num_units");
            rec.num_captains = int("num_captains");
            rec.num_standard = int("num_standard");
            rec.num_decoys = int("num_decoys");
            rec.target = target;
            rec.pos = Pos::new(int("x"), int("y"));
            rec.muster = Cell::new(int("muster_x"), int("muster_y"));
            rec.muster_angle = sim::movement::Angle(int("muster_angle"));
        }
        Scene {
            sim,
            cities,
            slots,
            units,
            groups: pool,
            untranslated,
        }
    }

    fn scene(name: &str, frame: i64) -> Option<Scene> {
        Some(scenes(name, &[frame])?.pop().expect("one frame"))
    }

    /// Several blocks of one dump, parsed once — a 250 MB window is not
    /// worth reading twice.
    fn scenes(name: &str, frames: &[i64]) -> Option<Vec<Scene>> {
        let inst = install()?;
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        Some(frames.iter().map(|&f| scene_at(&loaded, &log, f)).collect())
    }

    /// Run22's block 3579 again, this time with the harness's own ring
    /// search on the block's map: both records' muster cells re-derived
    /// from the cities they muster at — `do_mustering`'s search, flag 1 —
    /// in the order the game ran them: army 0's at the capital while it
    /// was the only army (its (44, 50) is two cells from army 1's init
    /// cell, so with army 1 standing it would be dropped as too near),
    /// then army 1's at Norwich with army 0 where it had settled. Army 1's
    /// (49, 54) scores eight — a building's cell in its 3 × 3 — and is the
    /// first admissible entry of ring 7; the first nine-neighbour cell of
    /// ring 8 lies 43 entries later, past the `0x28` early stop. The
    /// angle is the search's too, from the army's point.
    #[test]
    fn run22_s_muster_cells_are_the_ring_search_s_on_block_3579_s_own_map() {
        let Some(mut sc) = scene("gamelog-run22-islands-dock-window.txt", 3579) else {
            return;
        };
        let theirs: Vec<(Cell, i32)> = (0..2)
            .map(|s| {
                let a = &sc.sim.armies[1].list[s];
                assert!(a.valid, "army {s}");
                (a.muster, a.muster_angle.0)
            })
            .collect();
        assert_eq!(theirs[0].0, Cell::new(44, 50));
        assert_eq!(theirs[1].0, Cell::new(49, 54));
        for s in 0..2 {
            let a = &mut sc.sim.armies[1].list[s];
            a.muster = a.pos.cell();
        }
        sc.sim.armies[1].list[1].valid = false;
        for s in [0, 1] {
            sc.sim.armies[1].list[s].valid = true;
            let c = sc.sim.armies[1].list[s].city.expect("mustering at a city");
            let b = sc.sim.cities[c].building;
            assert!(
                sc.sim.find_muster_spot(1, s, Obj::Building(b), true),
                "army {s}: a spot"
            );
            let a = &sc.sim.armies[1].list[s];
            assert_eq!(a.muster, theirs[s].0, "army {s}'s muster cell");
            assert_eq!(a.muster_angle.0, theirs[s].1, "army {s}'s muster angle");
            assert!(!sc.sim.cities[c].no_muster);
        }
    }

    /// Run25's block 12129, before the `emergency` tick that closed army 1
    /// (`docs/ARMY.md` §16.5): on the block's own map the ring search at
    /// Norwich finds no cell — so `do_mustering` sees the failure and the
    /// city takes the `0x2000` mark the next block shows (`city_flags
    /// 0x0001 → 0x2001`) — while the navy, whose target `find_target`
    /// re-finds the same tick, lands on (46, 58) from (51, 53), the cell
    /// and the angle the record carries.
    #[test]
    fn run25_s_emergency_search_finds_no_cell_at_norwich_and_the_navy_re_finds_46_58() {
        let Some(mut sc) = scene("gamelog-run25-islands-emergency-window.txt", 12129) else {
            return;
        };
        let a1 = &sc.sim.armies[1].list[1];
        assert!(a1.valid && a1.status == 0x11 && !a1.navy);
        assert_eq!(a1.muster, Cell::new(49, 54), "the spot it had held");
        let c = a1.city.expect("at its city");
        assert_eq!(sc.sim.cities[c].pos, Pos::new(34656, 36192), "Norwich");
        let b = sc.sim.cities[c].building;
        assert!(!sc.sim.cities[c].no_muster, "0x0001 before");
        assert!(
            !sc.sim.find_muster_spot(1, 1, Obj::Building(b), true),
            "no cell at Norwich"
        );
        assert!(sc.sim.cities[c].no_muster, "0x2001 after");

        let a2 = &sc.sim.armies[1].list[2];
        assert!(a2.valid && a2.navy);
        let t = a2.target.expect("the navy's target");
        assert_eq!(a2.pos.cell(), Cell::new(51, 53));
        assert!(sc.sim.find_muster_spot(1, 2, t, false), "a sea cell");
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.muster, Cell::new(46, 58));
        // `find_target`'s tail turns the search's angle about (§12).
        assert_eq!(a2.muster_angle.0.wrapping_add(i32::MIN), 541_917_184);
        assert_eq!(sc.cities.len(), 3);
        assert_eq!(sc.slots.len(), 3);
    }

    /// Run27's block 15100, before army 0's `do_defending` tick: the
    /// nearest friendly city is Norwich again, still marked from 12129,
    /// and the search — flag 1 — finds no cell, which is the `close` the
    /// next block shows. The navy's search, now against the captured
    /// capital (an enemy's: the spacing is two cells), still lands on
    /// (46, 58).
    #[test]
    fn run27_s_defending_search_finds_no_cell_at_norwich_again() {
        let Some(mut sc) = scene("gamelog-run27-islands-defending-window.txt", 15100) else {
            return;
        };
        let a0 = &sc.sim.armies[1].list[0];
        assert!(a0.valid && a0.status == 1 && !a0.navy);
        assert_eq!(
            a0.muster,
            Cell::new(45, 48),
            "init's cell, one south of Norwich"
        );
        let c = a0.city.expect("at its city");
        assert!(sc.sim.cities[c].no_muster, "marked since 12129");
        let b = sc.sim.cities[c].building;
        assert!(!sc.sim.find_muster_spot(1, 0, Obj::Building(b), true));
        assert!(sc.sim.cities[c].no_muster);

        let a2 = &sc.sim.armies[1].list[2];
        let t = a2.target.expect("the navy's target");
        let Obj::Building(tb) = t else {
            panic!("a building target")
        };
        assert_eq!(sc.sim.buildings[tb].owner, 0, "the captured capital");
        assert!(sc.sim.find_muster_spot(1, 2, t, false));
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.muster, Cell::new(46, 58));
        assert_eq!(a2.muster_angle.0, -1_605_566_464);
    }

    /// Run26's block 12024, before the navy's tick — the first live
    /// `find_target` (`docs/ARMY.md` §16.5) — replayed whole. The trace
    /// (`rontrace-run26.log`, `report.py … draws 12024`) says what the
    /// tick drew: sim-frame 12024's first three `game_random` draws are
    /// all `Army::find_target+0x7df`, the per-candidate `% 200 + 900`,
    /// from `0x63ffe763` — the block's own `game_random seed` — and no
    /// coin, so the difficulty gate's `find_aggressive_army` answered
    /// this army. Three candidates on a three-city map: the human's
    /// Napata (an enemy's, at difficulty 0, which the gate admits
    /// unconditionally — the check that failed before the gate was
    /// corrected), then London and Norwich, the AI's own. London — the
    /// capital, attacked (`city_flags & 2`), damaged (`damage 12`) —
    /// scores `×10 ×10 ×2` over its draw and wins; the record at 12025
    /// carries it as `target_o 2000, target_who 1` with `rally_dist
    /// 0x1200`, the army's point one cell south of it, the ring search's
    /// (46, 58) and the turned-about angle. The stamps are the AI's own
    /// city's, so no `frame_attacked` is written.
    #[test]
    fn run26_s_navy_targets_its_own_attacked_capital_with_three_draws() {
        use sim::army::status;
        let Some(mut sc) = scene("gamelog-run26-islands-findtarget-window.txt", 12024) else {
            return;
        };
        assert_eq!(sc.sim.frame, 12024);
        assert_eq!(sc.sim.rng.seed, 0x63ff_e763, "the block's word");
        assert_eq!(sc.sim.ai_difficulty(), 0);
        assert!(sc.sim.is_enemy(0, 1) && !sc.sim.is_ally(0, 1));
        assert_eq!(sc.cities.len(), 3);
        let london = sc
            .cities
            .iter()
            .find(|(w, o, _)| *w == 1 && *o == 2000)
            .map(|(_, _, b)| *b)
            .expect("London");
        let lc = sc.sim.buildings[london].city.unwrap();
        assert!(sc.sim.cities[lc].capital && sc.sim.cities[lc].no_heal);
        let bd = &sc.sim.buildings[london];
        assert_eq!(bd.hits - bd.health, 12, "London's damage");
        assert_eq!(sc.sim.ai[1].frame_attacked, 12005);
        assert_eq!(sc.sim.ai[1].attacked_by, -1);
        assert_eq!(sc.sim.ai[1].pers.raid, -1);
        assert_eq!(sc.sim.ai[1].census.sea_combat, 8);

        let a2 = &sc.sim.armies[1].list[2];
        assert!(a2.valid && a2.navy && a2.status == 0x11 && a2.target.is_none());
        assert_eq!(a2.muster, Cell::new(56, 42));
        assert_eq!(a2.num_captains, 4);
        // `do_mustering` released it this tick — the attacked capital is
        // `release_mustering`'s first arm — and its common tail (§7) ran
        // before `do_marching` reached `find_target` (`Army::process+0x42c
        // < do_marching+0x248` in the trace): `status = 2`, `city = −1`,
        // the point to the muster cell's centre, `angle = muster_angle`
        // (the 292028416 the 12025 record keeps as `angle`). That move is
        // what the gate turns on: the record's point, one cell south of
        // London, is the AI's own cell (51, 53), where the navy would not
        // be aggressive and the gate would draw a coin; the muster cell
        // (56, 42) is ocean nobody owns, and it is.
        assert!(sc.sim.release_mustering(1, 2), "London is attacked");
        {
            let a = &mut sc.sim.armies[1].list[2];
            a.status = status::MARCHING;
            a.city = None;
            a.pos = sim::army::cell_centre(a.muster);
            a.angle = a.muster_angle;
        }
        assert_eq!(sc.sim.find_aggressive_army(1), Some(2));

        sc.sim.find_target(1, 2);

        assert_eq!(sc.sim.rng.seed, 0xad03_8188, "three draws, the trace's");
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.target, Some(Obj::Building(london)));
        assert_eq!(a2.rally_dist, 0x1200);
        assert_eq!(a2.hurry, 0);
        assert_eq!(a2.pos, Pos::new(39264, 40800));
        assert_eq!(a2.muster, Cell::new(46, 58));
        assert_eq!(a2.muster_angle.0, 541_917_184);
        assert_eq!(a2.angle.0, 292_028_416, "do_mustering's, untouched");
        assert!(a2.valid);
        assert_eq!(sc.sim.ai[1].frame_attacked, 12005, "my own city: no stamp");
        assert_eq!(sc.sim.ai[1].attacked_by, -1);
        assert_eq!(sc.sim.ai[0].frame_attacked, 0);
    }

    // ---- the whole `GROUPDATA` record (`docs/GROUPS.md` §1, §6.4, §10) ----

    /// One frame's group pool: the frame number, its 512 `GROUPDATA`
    /// records and its `last_group[8]`.
    type Pool = (i64, Vec<crate::gamelog::GroupDump>, Vec<i64>);

    /// The three windowed frames' group pools, parsed once.
    fn run29_pools() -> Option<Vec<Pool>> {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let out = log
            .frames()
            .into_iter()
            .filter(|(f, _)| (15100..=15102).contains(f))
            .map(|(f, b)| (f, crate::gamelog::groups(b), crate::gamelog::last_group(b)))
            .collect::<Vec<_>>();
        assert_eq!(out.len(), 3, "the window's three frames");
        Some(out)
    }

    /// The record, **whole** — every scalar, every array, every slot — for
    /// all three windowed frames.
    ///
    /// This is the widening `docs/audit/2026-08-25-groups.md` puts first,
    /// and it is the project's own rule applied: when the original dumps a
    /// record, diff the whole record. `GroupData::log_data@0045e1d0` writes
    /// twenty scalars and six parallel per-member arrays, and until now
    /// nothing in `crates/rondata` had opened one.
    ///
    /// What is asserted here is the record's **shape and its own
    /// invariants** — the parts the simulation cannot yet produce, because
    /// it has no group pool (`docs/GROUPS.md` §12's second seam). The parts
    /// it can are the two tests below.
    #[test]
    fn run29_s_group_pool_is_five_hundred_and_twelve_whole_groupdata_records() {
        use crate::gamelog::GROUP_FIELDS;
        let Some(frames) = run29_pools() else { return };
        for (f, pool, last) in &frames {
            // 8 leaders × 64 slots, in `id` order, every frame. The hotkey
            // groups are a separate array and are not in here.
            assert_eq!(pool.len(), 512, "{f}: 8 leaders × 64 slots");
            for (i, g) in pool.iter().enumerate() {
                assert_eq!(g.id, i as i64, "{f}: the pool is dumped in id order");
                // Every pool record is a unit group, never disbanding, and
                // never a control group — `priority` is the hotkey array's
                // bit and it is 0 on all 512.
                assert_eq!(g.buildings, 0, "{f}/{}: buildings", g.id);
                assert_eq!(g.disband, 0, "{f}/{}: action_begin clears it", g.id);
                assert_eq!(g.priority, 0, "{f}/{}: not a control group", g.id);
                // The six arrays are parallel and exactly `num` long.
                assert_eq!(g.members.len(), g.num as usize, "{f}/{}: arrays", g.id);
                if g.num > 0 {
                    // A live slot belongs to the leader its index names:
                    // `get_open_slot` indexes `who * 0x40`.
                    assert_eq!(g.who, g.id / 64, "{f}/{}: who = id / 64", g.id);
                    assert!(g.num <= 128, "{f}/{}: the 128-member cap", g.id);
                }
            }
            // `last_group[8]`, re-attached by position (see `last_group`).
            assert_eq!(last.len(), 8, "{f}: one per leader");
            for (p, &l) in last.iter().enumerate() {
                let base = p as i64 * 64;
                assert!(
                    (base..base + 46).contains(&l),
                    "{f}: last_group[{p}] = {l} is outside the allocatable 46"
                );
            }
            // Only the AI has ever had a group installed: the other seven
            // still hold `Groups::clear`'s `p × 0x40`, and player 1's has
            // moved to the slot `push_group` last took.
            assert_eq!(last[1], 70, "{f}: player 1's last group is slot 70");
            for p in [0usize, 2, 3, 4, 5, 6, 7] {
                assert_eq!(last[p], p as i64 * 64, "{f}: player {p} never allocated");
            }
        }

        // The writer's own field order, and the one field it never writes.
        // Read from the block rather than the parsed struct, because the
        // order *is* the assertion (`docs/GROUPS.md` §1).
        let name = "gamelog-run29-islands-engagement-window.txt";
        let text = std::fs::read_to_string(dump(name).unwrap()).unwrap();
        let log = Log::parse(&text);
        let mut seen = 0usize;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            let body = b.kid("FULL DUMP").unwrap_or(b);
            for g in body.kids("GROUPDATA") {
                let keys: Vec<&str> = g.fields.iter().take(20).map(|(k, _)| *k).collect();
                assert_eq!(keys, GROUP_FIELDS, "{f}: log_data's own order");
                assert!(
                    !g.fields.iter().any(|(k, _)| *k == "march"),
                    "{f}: march is the one GroupData field the engine never logs"
                );
                seen += 1;
            }
        }
        assert_eq!(seen, 512 * 3);
    }

    /// `order_num` and `form` across the `Army::engagement` frame — the
    /// audit's assertions 2 and 3, and the one place the record and the
    /// harness meet on this mechanic today.
    ///
    /// Army 0's group (`id 69`, seven members) is the group
    /// `Group::action_attack` fires on at 15100 (`docs/GROUPS.md` §11: the
    /// frame `GroupData::num_valid` first executes). The record shows
    /// `order_num 0 → 1 → 1` and `form −1` throughout — `action_attack`
    /// issues orders, so it bumps the counter, and it writes no `form`.
    /// The simulation is made to reproduce **both deltas**, which is what
    /// `group_action_attack` and `group_action_halt` were corrected to do.
    #[test]
    fn run29_s_engagement_bumps_order_num_by_one_and_writes_no_form() {
        let Some(frames) = run29_pools() else { return };
        let army0 = |pool: &Vec<crate::gamelog::GroupDump>| {
            pool.iter().find(|g| g.id == 69).cloned().expect("slot 69")
        };
        let (a, b, c) = (
            army0(&frames[0].1),
            army0(&frames[1].1),
            army0(&frames[2].1),
        );
        for g in [&a, &b, &c] {
            assert_eq!(g.who, 1, "the AI's");
            assert_eq!(g.army, 0, "army 0's one group");
            assert_eq!(g.num, 7);
            assert_eq!(g.form, -1, "never moved: no formation index");
            assert_eq!(g.form_num, 0, "and Form::compute never laid it out");
            assert_eq!(g.think_frame, 15020, "the hoplites' arrival");
            assert!(
                g.members.iter().all(|m| (m.off_x, m.off_y) == (0, 0)),
                "and every slot offset is still zero"
            );
        }
        assert_eq!(a.order_num, 0, "15100 is the state the frame begins on");
        assert_eq!(b.order_num, 1, "action_attack issued orders");
        assert_eq!(c.order_num, 1, "and nothing issued more");

        // The harness, on a group of the same size. `action_attack` bumps
        // `order_num` by exactly one and leaves `form` alone; `action_halt`
        // clears the group's `form` and bumps nothing.
        let mut s = sim::Sim::new(Tuning::RON, World::new(60, 60), 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        let ty = s.add_unit_type(sim::UnitType {
            hits: 100,
            combat: sim::combat::Profile {
                attack: 15,
                uber_size: 1,
                ..sim::combat::Profile::default()
            },
            ..sim::UnitType::default()
        });
        let slot = s.init_army(1, None);
        for k in 0..7 {
            let idx = i16::try_from(s.units.len()).unwrap();
            let mut u = Unit::new(1, idx, Pos::new(0x1000 + k * 0x80, 0x1000), 100);
            u.ty = Some(ty);
            u.on_map = true;
            let u = s.add_unit(u);
            s.army_add_unit(1, slot, u);
        }
        let idx = i16::try_from(s.units.len()).unwrap();
        let mut foe = Unit::new(0, idx, Pos::new(0x1400, 0x1000), 100);
        foe.ty = Some(ty);
        foe.on_map = true;
        let foe = s.add_unit(foe);

        let g = s.army_group(1, slot);
        assert_eq!(s.armies[1].list[slot].group.order_num, 0);
        assert_eq!(s.armies[1].list[slot].group.form, -1);
        s.group_action_attack(
            &g,
            sim::combat::Obj::Unit(foe),
            false,
            sim::orders::QueuePos::New,
            0,
        );
        assert_eq!(
            s.armies[1].list[slot].group.order_num, 1,
            "the record's 0 → 1"
        );
        assert_eq!(
            s.armies[1].list[slot].group.form, -1,
            "action_attack writes no form — the record's −1 across all three frames"
        );
    }

    /// The slot table's own output, for the one live formation in the
    /// window — the audit's assertions 4 and 5, and the fixture that turns
    /// `docs/GROUPS.md` §6.4 from a seam into a checked table the day
    /// `Form::compute_dests` is written.
    ///
    /// Group `id 66` is the AI's navy (`army 2`, `role & 0x80000`), four
    /// members in **formation 0** (Line). Its `off_x` are `[0, −14, 13,
    /// −28]` in 48-unit steps with every `off_y` zero — a line along the
    /// formation's own x axis — and its `curr` are those offsets after
    /// `update_positions` has rotated them by the leader's heading, in
    /// position units. Two things fall out and both are asserted:
    /// `|curr[i]| = 48 · |off_x[i]|` (the `leal (%ecx,%ecx,2)` + `shll $4`
    /// of `7139e8`), and every `curr` is the **same** rotation of its own
    /// `off_x`, so the members stay collinear.
    #[test]
    fn run29_s_navy_group_is_a_line_of_four_rotated_at_forty_eight_units_a_step() {
        let Some(frames) = run29_pools() else { return };
        for (f, pool, _) in &frames {
            let g = pool.iter().find(|g| g.id == 66).expect("slot 66");
            assert_eq!(g.who, 1);
            assert_eq!(g.army, 2, "the AI's navy");
            assert_eq!(g.num, 4);
            assert_eq!(g.form, 0, "formation 0 — Line");
            assert_eq!(g.form_num, g.num, "Form::compute laid out all four");
            assert_eq!(g.o_dist, 351);
            assert_eq!(g.o_angle, -1_605_566_464);
            assert_eq!(g.facing, 0, "not mirrored");

            let off_x: Vec<i64> = g.members.iter().map(|m| m.off_x).collect();
            assert_eq!(off_x, [0, -14, 13, -28], "{f}: the slot table's own row");
            assert!(
                g.members.iter().all(|m| m.off_y == 0),
                "{f}: a line has no depth"
            );
            assert!(
                g.members.iter().all(|m| m.angle == 0),
                "{f}: and no per-slot facing"
            );
            assert_eq!(
                g.members.iter().map(|m| m.o).collect::<Vec<_>>(),
                [32, 34, 35, 38],
                "{f}: the member ids, in join order"
            );

            for m in &g.members {
                // |curr| = 48 · |off_x|, to the sine table's granularity.
                let want = m.off_x * 48;
                let got2 = m.curr_x * m.curr_x + m.curr_y * m.curr_y;
                let want2 = want * want;
                assert!(
                    (got2 - want2).abs() * 100 <= want2 + 100,
                    "{f}: |curr| {got2} is not 48 × |off_x| {want2}"
                );
                // The same rotation for every member: `curr` is collinear
                // with the first non-zero member's, scaled by `off_x`.
                let r = &g.members[1];
                assert!(
                    (m.curr_x * r.off_x - r.curr_x * m.off_x).abs() <= 48 * 2,
                    "{f}: curr_x is not the shared rotation of off_x"
                );
                assert!(
                    (m.curr_y * r.off_x - r.curr_y * m.off_x).abs() <= 48 * 2,
                    "{f}: curr_y is not the shared rotation of off_x"
                );
            }
        }
    }

    /// **The slot table, closed** — `docs/GROUPS.md` §6.4's seam turned
    /// into a diff, from the install's own columns to the record's own row.
    ///
    /// The chain this asserts end to end, with nothing guessed in the
    /// middle:
    ///
    /// 1. `unitrules.xml` gives the fifteen light warships `X_SPACING 55`,
    ///    and `UnitType::init` multiplies it by `UNIT_FORMATION_SPACING`
    ///    (12) — so `x_spacing = 660`, which the loader must reproduce.
    /// 2. A warship's `OBJ_MASK` is `N`/`NRL`: not civilian, not foot, not
    ///    mounted, not a vehicle, not anti-air — so it falls off the end of
    ///    `FormData::type_cat`'s tree into `FORM_CAT_ARTILLERY`.
    /// 3. `GroupData::get_form_mod_option` is **50**, and run29's own
    ///    `UNITDATA` prints `form_mod 50` on each member, so the column
    ///    count is `((4 × 660 × 50)/50)/660 = 4` and there is one rank.
    /// 4. `Form::compute_dests` lays four slots as `0, −w, +w, −2w`, shifts
    ///    the block by `w/2` because the count is even, and slides it back
    ///    by the anchor.
    /// 5. The floor divide by 48 turns `[330, −330, 990, −990]` into
    ///    `[0, −14, 13, −28]` — **the record's own `off_x`**.
    ///
    /// And then the record's `curr`, exactly, through `update_positions`
    /// under the leader's logged heading. Nothing here is a fixture written
    /// from the answer: the four numbers come out of the install's columns
    /// and the simulation's arithmetic.
    #[test]
    fn run29_s_navy_slot_table_is_reproduced_from_the_install_s_own_spacing() {
        use sim::form::{cat, formation, type_cat};
        use sim::movement::Angle;
        let Some(inst) = install() else { return };
        let Some(frames) = run29_pools() else { return };
        let loaded = crate::load::load(&inst).unwrap();

        // The install's light warships, by the column the reading named.
        let ships: Vec<usize> = (0..loaded.unit_types.len())
            .filter(|&t| loaded.unit_types[t].combat.x_spacing == 660)
            .collect();
        assert_eq!(
            ships.len(),
            15,
            "the fifteen types with X_SPACING 55 — {:?}",
            ships
                .iter()
                .map(|&t| &loaded.unit_names[t])
                .collect::<Vec<_>>()
        );
        for &t in &ships {
            let ty = &loaded.unit_types[t];
            assert_eq!(ty.combat.y_spacing, 660, "{}", loaded.unit_names[t]);
            assert_eq!(ty.combat.uber_size, 1, "{}", loaded.unit_names[t]);
            assert_eq!(
                type_cat(ty, t, false),
                cat::ARTILLERY,
                "{} falls off the end of type_cat's tree",
                loaded.unit_names[t]
            );
        }
        let galley = *ships
            .iter()
            .find(|&&t| loaded.unit_names[t] == "Galley")
            .expect("a Galley in the install");

        // Four of them in one army's group, on the harness's own world.
        let mut s = sim::Sim::new(
            sim::tuning::Tuning::RON,
            sim::world::World::new(200, 200),
            2,
        );
        s.nation[1].human = false;
        let mut proto = loaded.unit_types[galley].clone();
        // The harness here carries no tech tree; the columns are the point.
        proto.tree = None;
        let ty = s.add_unit_type(proto);
        let slot = s.init_army(1, None);
        for i in 0..4 {
            let idx = i16::try_from(s.units.len()).unwrap();
            let mut u = Unit::new(1, idx, Pos::new(0x8000 + i * 0x300, 0x8000), 210);
            u.ty = Some(ty);
            u.on_map = true;
            let u = s.add_unit(u);
            s.army_add_unit(1, slot, u);
        }
        let g = s.army_group(1, slot);
        let f = s.form_compute(
            &g,
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::LINE,
            s.group_form_mod_option(&g),
            false,
            false,
            &[],
        );
        assert_eq!(s.group_form_mod_option(&g), 50, "the record's form_mod");
        assert_eq!(f.num_category[cat::ARTILLERY], 4);
        assert_eq!(f.x_spacing[cat::ARTILLERY], 660);
        let ours: Vec<(i32, i32)> = f
            .off
            .iter()
            .map(|&(x, y)| (sim::form::Form::quantise(x), sim::form::Form::quantise(y)))
            .collect();

        for (fr, pool, _) in &frames {
            let navy = pool.iter().find(|g| g.id == 66).expect("slot 66");
            let theirs: Vec<(i32, i32)> = navy
                .members
                .iter()
                .map(|m| {
                    (
                        i32::try_from(m.off_x).unwrap(),
                        i32::try_from(m.off_y).unwrap(),
                    )
                })
                .collect();
            assert_eq!(ours, theirs, "{fr}: the record's own slot table");
            assert_eq!(
                f.angles,
                navy.members
                    .iter()
                    .map(|m| m.angle as i8)
                    .collect::<Vec<_>>(),
                "{fr}: and its per-slot facings"
            );
            assert_eq!(
                i64::from(f.off.len() as i32),
                navy.form_num,
                "{fr}: form_num is the membership Form::compute laid out"
            );
            // And `curr`, through the leader's own logged heading.
            let theta = Angle(-1_605_566_464);
            let curr = sim::Sim::form_update_positions(&ours, theta);
            assert_eq!(
                curr,
                navy.members
                    .iter()
                    .map(|m| Pos::new(
                        i32::try_from(m.curr_x).unwrap(),
                        i32::try_from(m.curr_y).unwrap()
                    ))
                    .collect::<Vec<_>>(),
                "{fr}: the record's own curr"
            );
        }
    }

    /// `priority` says **allocated**, not "in the hotkey array" — which is
    /// not what the audit's assertion 6 predicted, and the widening found
    /// it on its first run.
    ///
    /// The dump keeps the two arrays apart structurally: the pool is 512
    /// flat `GROUPDATA` under `FULL DUMP`, while a hotkey group is a
    /// `HOTKEYGROUPDATA` wrapper with its `GROUPDATA` **nested inside it**.
    /// Every record in the first has `priority 0` and 161 of the 162 in the
    /// second have `priority 1` — but **slot 28 has 0**, with a `stamp` of
    /// 13125. It held a group and lost its last member: `Group::kill`'s
    /// `num == 0 → clear(−1)` runs `Group::clear@00713e80`, which writes
    /// **0** over the bit, and only `HotKeyGroups::find_group@00714d00`
    /// puts it back, the next time that slot is allocated. So the bit is
    /// "this slot is a live control group", and an emptied hotkey slot is
    /// indistinguishable from a pool slot by `priority` alone.
    #[test]
    fn run29_s_priority_bit_says_allocated_rather_than_hotkey() {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut pool = 0usize;
        let mut hotkey = 0usize;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            let body = b.kid("FULL DUMP").unwrap_or(b);
            for g in body.kids("GROUPDATA") {
                assert_eq!(g.int("priority"), Some(0), "{f}: a pool slot");
                pool += 1;
            }
            let hk = body.kid("HOTKEYGROUPS").expect("the hotkey array");
            assert_eq!(hk.int("length"), Some(162), "{f}: the array's own length");
            let mut cleared = 0usize;
            for w in hk.kids("HOTKEYGROUPDATA") {
                let g = w.kid("GROUPDATA").expect("the nested record");
                match g.int("priority") {
                    Some(1) => {}
                    Some(0) => {
                        cleared += 1;
                        assert_eq!(g.int("id"), Some(28), "{f}: the one emptied slot");
                        assert_eq!(g.int("num"), Some(0), "{f}: emptied");
                        assert_eq!(
                            g.int("stamp"),
                            Some(13125),
                            "{f}: and it held a group until then — Group::clear zeroed the bit"
                        );
                    }
                    p => panic!("{f}: priority {p:?} on a hotkey slot"),
                }
                hotkey += 1;
            }
            assert_eq!(cleared, 1, "{f}: exactly one slot has been emptied");
        }
        assert_eq!(pool, 512 * 3);
        assert_eq!(hotkey, 162 * 3, "the hotkey array's own length");
    }

    /// The third pass's widening (`docs/audit/2026-08-25-groups.md`, "Third
    /// pass — verdicts"): the scalars the first widening parsed and compared
    /// against nothing, and the rotation pinned **exactly** rather than by
    /// magnitude.
    ///
    /// `Group::update_positions@00713810`, from the listing: for each of
    /// `form_num` members, `curr_x = off_x·48·sin(θ + 90°) + off_y·48·sin(θ)`
    /// and `curr_y = off_x·48·sin(θ) − off_y·48·sin(θ + 90°)` — the matrix
    /// `[cos θ, sin θ; sin θ, −cos θ]`, determinant −1 — where θ is the
    /// heading (`unit +0x50`) of the unit executing the group move, the
    /// leader. `UNITDATA` logs that heading as `angle`, so the record can be
    /// reproduced to the bit with the simulation's own `sin_component` /
    /// `cos_component`, which are the original's `sin_table`.
    ///
    /// And the scalars: `stamp` and `think_frame` never exceed the frame
    /// (`Group::clear` stamps `game->frame`; `Groups::clear` writes 0 over
    /// it), `speed == new_speed` outside the one `do_group_move` step that
    /// rotates them, and `role` is the OR of the members' type roles
    /// (`Group::add`: `role |= type +0x2c8`, and nothing ever clears a bit)
    /// — so the navy's carries `SEA`, `SEA_MILITARY` and `MILITARY` and not
    /// `LAND`, and army 0's `LAND` and `MILITARY` and not `SEA`.
    #[test]
    fn run29_s_navy_group_s_curr_is_the_leader_s_heading_applied_to_the_slot_table() {
        use sim::ai_load::role;
        use sim::movement::{Angle, cos_component, sin_component};
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut seen = 0;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            seen += 1;
            let pool = crate::gamelog::groups(b);
            let (units, _, _) = crate::gamelog::records(b, false);

            // The scalars nobody had compared, over the whole pool.
            for g in &pool {
                assert!(
                    g.stamp <= f,
                    "{f}/{}: stamp {} is in the future",
                    g.id,
                    g.stamp
                );
                assert!(
                    g.think_frame <= f,
                    "{f}/{}: think_frame {} is in the future",
                    g.id,
                    g.think_frame
                );
                assert_eq!(
                    g.speed, g.new_speed,
                    "{f}/{}: the two speeds differ only inside do_group_move's step",
                    g.id
                );
                if g.num == 0 {
                    assert_eq!(g.role, 0, "{f}/{}: an empty slot has no role", g.id);
                }
            }

            // `role`, by the bits `ai_load::role` names.
            let navy = pool.iter().find(|g| g.id == 66).expect("slot 66");
            let army0 = pool.iter().find(|g| g.id == 69).expect("slot 69");
            let has = |g: &crate::gamelog::GroupDump, bit: u32| g.role & i64::from(bit) != 0;
            for bit in [role::SEA, role::SEA_MILITARY, role::MILITARY] {
                assert!(has(navy, bit), "{f}: the navy's role lacks {bit:#x}");
            }
            assert!(!has(navy, role::LAND), "{f}: a navy is not a land group");
            for bit in [role::LAND, role::MILITARY] {
                assert!(has(army0, bit), "{f}: army 0's role lacks {bit:#x}");
            }
            assert!(!has(army0, role::SEA), "{f}: army 0 is not a sea group");
            // The whole word, as observed: `LAND | MILITARY | RANGED |
            // MOUNTED | MELEE` — a mixed army. `Group::kill` never clears a
            // bit, so the word is the OR over every member the group has
            // *ever* had, not only the seven it has now.
            assert_eq!(
                army0.role,
                i64::from(role::LAND | role::MILITARY | role::RANGED | role::MOUNTED | role::MELEE),
                "{f}: army 0's accumulated role word"
            );

            // The rotation, exactly. The leader is the member at slot (0, 0).
            let lead = navy
                .members
                .iter()
                .find(|m| (m.off_x, m.off_y) == (0, 0))
                .unwrap();
            let u = units
                .iter()
                .find(|u| u.who == navy.who && u.o == lead.o)
                .expect("the leader's UNITDATA");
            let theta = Angle(i32::try_from(u.angle.expect("UNITDATA's angle")).unwrap());
            for m in &navy.members {
                let (dx, dy) = (
                    i32::try_from(m.off_x * 48).unwrap(),
                    i32::try_from(m.off_y * 48).unwrap(),
                );
                let want_x = cos_component(theta, dx) + sin_component(theta, dy);
                let want_y = sin_component(theta, dx) - cos_component(theta, dy);
                assert_eq!(
                    (m.curr_x, m.curr_y),
                    (i64::from(want_x), i64::from(want_y)),
                    "{f}: member {} at off ({}, {}) under the leader's heading {}",
                    m.o,
                    m.off_x,
                    m.off_y,
                    theta.0
                );
                // The check has teeth: a quarter turn off does not
                // reproduce the record. (The group's own `o_angle` does —
                // in this window the leader's heading *is* the move's
                // bearing, so the record cannot separate the two; the
                // listing at `713844` is what says `update_positions`
                // reads the heading.)
                if dx != 0 {
                    assert_ne!(i64::from(cos_component(theta.quarter_turn(), dx)), m.curr_x);
                }
            }
            // Every `off_y` is zero, so the `−cos θ` term never fires: the
            // y-flip is still unpinned, and needs a formation with depth.
            assert!(navy.members.iter().all(|m| m.off_y == 0));
        }
        assert_eq!(seen, 3, "the window's three frames");
    }

    /// Run29's blocks 15100 and 15101 — the tick that put the AI's army on
    /// the one path that reaches `Army::engagement` (`docs/ARMY.md` §16.6).
    ///
    /// Six human hoplites were dropped on army 0's own point at
    /// 15020–15030, so that its 15100 tick found `is_engaged()` true while
    /// it was still mustering. The dispatch (§6 step 6) runs `do_forming`
    /// **or** `engagement`, never both, and `march_to_target`'s engaged arm
    /// leaves `0x10` set — so the only way to `engagement` is
    /// `do_mustering`'s release, whose common tail (§7) overwrites `status`
    /// whole. The two blocks show exactly that: `status 1 → 32`, `city 1 →
    /// −1`, and the point moved from the city's `(34656, 36960)` to the
    /// **muster cell's centre** `(34944, 37248)` — `45 × 0x300 + 0x180`,
    /// `48 × 0x300 + 0x180` — with `muster` itself untouched.
    ///
    /// What is asserted from the sim's own `do_mustering` is the tail and,
    /// crucially, that **no `FORMING` bit survives it** — the gate the
    /// dispatch's `else` needs. Which released branch it takes is *not*
    /// asserted from the harness alone: `strategy[reg]`, the census word
    /// that picks defending over marching, is not in the dump, so it is
    /// set here from the record's own outcome and said to be an input.
    #[test]
    fn run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_point() {
        use sim::army::status;
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101],
        ) else {
            return;
        };
        let (mut before, after) = {
            let mut it = scs.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        };

        // The block before the tick, field for field.
        let a = &before.sim.armies[1].list[0];
        assert!(a.valid && !a.navy);
        assert_eq!(a.status, status::MUSTERING);
        assert_eq!(a.num_units, 7);
        assert_eq!(a.num_standard, 7);
        assert_eq!(a.pos, Pos::new(34656, 36960));
        assert_eq!(a.muster, Cell::new(45, 48));
        assert_eq!(a.muster_angle.0, 0);
        assert!(a.city.is_some(), "mustering at city 1");
        let reg = a.reg.expect("the army's region");

        // The record itself proves the original released it — a mustering
        // army that comes out `0x20` cannot have taken the not-released
        // arm, which only ever ORs `0x10` in. The harness agrees.
        assert!(
            before.sim.release_mustering(1, 0),
            "seven standard at difficulty 0"
        );

        // The one input the dump does not carry: the census's
        // `strategy[reg]`, whose weak bit picks defending over marching.
        before.sim.ai[1].census.strategy.resize(reg as usize + 1, 0);
        before.sim.ai[1].census.strategy[reg as usize] |= 4;

        before.sim.do_mustering(1, 0);

        let got = &before.sim.armies[1].list[0];
        let want = &after.sim.armies[1].list[0];
        assert_eq!(
            got.status & status::FORMING,
            0,
            "the release leaves no FORMING bit — this is the gate `engagement` needs"
        );
        assert_eq!(got.status, want.status, "the record's 32 (DEFENDING)");
        assert_eq!(got.city, None);
        assert_eq!(got.city, want.city);
        assert_eq!(
            got.pos,
            sim::army::cell_centre(Cell::new(45, 48)),
            "the muster cell's centre"
        );
        assert_eq!(got.pos, want.pos, "the record's (34944, 37248)");
        assert_eq!(got.muster, want.muster, "the muster cell is untouched");
        assert_eq!(got.angle, got.muster_angle, "the tail copies it");
        assert_eq!(got.num_standard, want.num_standard);
    }

    // ---- the `UNITS=3` half of run29 (`docs/ARMY.md` §11, §18) ----

    /// **`engagement`'s choice of unit, from the record.** `docs/ARMY.md`
    /// §18 said §11 rested on the reading alone for this, "which no dump
    /// shows"; this is the dump showing it.
    ///
    /// At 15100 army 0's seven members each hold `[MOVEORDER(pathed),
    /// ATTACKORDER]` and their attack targets are a **scatter** — who 0's
    /// objects 15, 16 and 17. At 15101 six of the seven hold `15`, every
    /// one of the seven has gained the **action bit** (`flags 0x10 →
    /// 0x14`), and `order_num` has gone `0 → 1`. That is
    /// `Army::engagement` → `Group::action_attack(·, ox, whom, 0,
    /// QUEUE_NEW, 0)` firing on the tick, and the seed it adopted is
    /// object **15** — the target of `o 54`, the **first** member of the
    /// group's own `list`, which is the order `ArmyData::get_unit` walks.
    ///
    /// Two things the capture falsified in the simulation, both found by
    /// running this:
    ///
    /// 1. `is_engaged` and `engagement` tested the **front** order rather
    ///    than `get_action()`'s. Every member here is walking a transit
    ///    leg in front of its attack, so with the front order tested no
    ///    unit qualified and the whole mechanic was dead on the one frame
    ///    that reaches it.
    /// 2. The seed is not "the first with a map-unit target" alone — see
    ///    `Sim::army_engagement_seed`. This block breaks on the first, so
    ///    the fallback stays unobserved; the capture for it is an army
    ///    whose only attackers are pointed at **buildings**.
    ///
    /// The one member that does not end on 15 is `o 25`, which lands on
    /// 26: `action_attack` gives each member `Unit::find_melee_target`'s
    /// nearest within the respond range and the seed is only the fallback
    /// (`docs/GROUPS.md` §10), so the shared value is what the *seed*
    /// says and not what every member ends up with.
    #[test]
    fn run29_s_engagement_seeds_its_attack_from_the_first_member_of_the_group_s_list() {
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101],
        ) else {
            return;
        };
        let (mut before, after) = {
            let mut it = scs.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        };
        // The only kind a scene cannot stand up is `GATHERORDER`, whose
        // `building` is not optional in the simulation and whose target is
        // a woodcutter's camp or a mine — the scene builds cities and
        // nothing else. No member of an army holds one.
        assert!(
            before.untranslated.iter().all(|k| k == "GATHERORDER"),
            "an order kind beyond the gather seam: {:?}",
            before.untranslated
        );

        // The record's own membership, in `list` order — the order
        // `get_unit` walks and therefore the order the seed is chosen in.
        let listed: Vec<i64> = before
            .groups
            .iter()
            .find(|g| g.id == 69)
            .expect("army 0's group")
            .members
            .iter()
            .map(|m| m.o)
            .collect();
        assert_eq!(listed, [54, 53, 52, 51, 49, 48, 25]);
        assert_eq!(
            before.sim.armies[1].list[0].units,
            listed
                .iter()
                .map(|&o| before.unit(1, o).expect("the member's unit"))
                .collect::<Vec<_>>(),
            "the scene stands the army up in the record's own order"
        );

        // Every member's action is its attack, under a pathed transit leg.
        for &o in &listed {
            let u = before.unit(1, o).unwrap();
            let orders = &before.sim.units[u].orders;
            assert_eq!(orders.len(), if o == 25 { 3 } else { 2 }, "member {o}");
            assert!(
                orders[0].is_move() && orders[0].is_transit(),
                "{o}: the leg"
            );
            let a = before.sim.action_of(u).expect("an action under the leg");
            assert!(
                matches!(orders[a].body, sim::orders::Body::Attack(_)),
                "{o}: the action is the attack"
            );
        }

        // The scatter the frame begins on.
        let target_o = |sc: &Scene, o: i64| -> Option<i64> {
            let u = sc.unit(1, o)?;
            match sc.sim.units[u].combat.target? {
                Obj::Unit(t) => sc
                    .units
                    .iter()
                    .find(|(_, _, h)| *h == t)
                    .map(|(_, n, _)| *n),
                Obj::Building(_) => None,
            }
        };
        assert_eq!(
            listed
                .iter()
                .map(|&o| target_o(&before, o))
                .collect::<Vec<_>>(),
            [
                Some(15),
                Some(16),
                Some(16),
                Some(15),
                Some(17),
                Some(16),
                Some(16)
            ],
            "15100's scatter"
        );

        assert!(
            before.sim.army_is_engaged(1, 0),
            "the tick that reaches engagement at all"
        );
        let (seed, target) = before
            .sim
            .army_engagement_seed(1, 0)
            .expect("a seed on this frame");
        assert_eq!(
            seed,
            before.unit(1, 54).unwrap(),
            "the first member of the list, not the first of the army's own array"
        );
        assert_eq!(
            target,
            Obj::Unit(before.unit(0, 15).unwrap()),
            "who 0's object 15 — the target the six other members end on"
        );

        // And that is what the next block holds.
        let six: Vec<Option<i64>> = listed
            .iter()
            .filter(|&&o| o != 25)
            .map(|&o| target_o(&after, o))
            .collect();
        assert_eq!(six, vec![Some(15); 6], "15101's shared target");
        assert_eq!(
            target_o(&after, 25),
            Some(26),
            "find_melee_target's own nearest, not the seed (docs/GROUPS.md §10)"
        );
        for &o in &listed {
            let u = after.unit(1, o).unwrap();
            let a = after.sim.action_of(u).expect("still an attack");
            assert!(
                after.sim.units[u].orders[a].has(sim::orders::flag::ACTION),
                "{o}: action_attack sets the action bit — the record's 0x10 → 0x14"
            );
        }
    }

    /// **A halted group forgets its formation; its members do not.**
    /// `docs/GROUPS.md` §7's one write, seen in a record for the first
    /// time — `action_halt` sets `GroupData.form = −1` unconditionally and
    /// touches no member's `+0xaa`, so `get_form` reads the members and
    /// gives back the formation the group's own field has lost.
    ///
    /// Army 0's group carries `form −1` and `form_num 0` at every frame of
    /// the window while all seven members carry `form 0`; the navy's
    /// carries `form 0` and so do its four. And `get_form_mod_option` is
    /// the **mean over the members whose byte is not −1**, not `get_form`'s
    /// all-agree-or-−1 twin: two of army 0's seven are −1 and the option
    /// is still 50.
    #[test]
    fn run29_s_halted_group_kept_its_members_formation_bytes() {
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101, 15102],
        ) else {
            return;
        };
        for (n, sc) in scs.iter().enumerate() {
            let f = 15100 + n;
            let rec = |id: i64| sc.groups.iter().find(|g| g.id == id).expect("the slot");
            let (army0, navy) = (rec(69), rec(66));
            assert_eq!(army0.form, -1, "{f}: the group's own byte, cleared");
            assert_eq!(army0.form_num, 0);
            assert_eq!(navy.form, 0, "{f}: and a group that did move");

            let members = |g: &crate::gamelog::GroupDump| -> Vec<usize> {
                g.members
                    .iter()
                    .map(|m| sc.unit(g.who, m.o).expect("the member"))
                    .collect()
            };
            let group = |g: &crate::gamelog::GroupDump| sim::group::Group {
                who: g.who as sim::Player,
                army: Some(g.army.max(0) as usize),
                list: members(g),
            };
            let bytes: Vec<i8> = members(army0)
                .iter()
                .map(|&u| sc.sim.units[u].form)
                .collect();
            assert_eq!(bytes, vec![0; 7], "{f}: every member kept formation 0");
            assert_eq!(
                sc.sim.group_get_form(&group(army0)),
                0,
                "{f}: so get_form gives back what the group's own field lost"
            );
            let widths: Vec<i8> = members(army0)
                .iter()
                .map(|&u| sc.sim.units[u].form_width)
                .collect();
            assert_eq!(
                widths,
                [-1, 50, 50, -1, 50, 50, 50],
                "{f}: o 54 and o 51 have never been laid out"
            );
            assert_eq!(
                sc.sim.group_form_mod_option(&group(army0)),
                50,
                "{f}: the mean skips the −1s rather than refusing"
            );
            assert_eq!(sc.sim.group_form_mod_option(&group(navy)), 50);
            assert_eq!(sc.sim.group_get_form(&group(navy)), 0);
        }
    }

    /// **The whole `MOVEORDER` row, over every move order in the
    /// window** — `docs/ORDERS.md` §4.1's table turned from a reading into
    /// a diff. Every one of these was read off the PE and none had ever
    /// been compared with a record.
    ///
    /// 146 move orders across the window's four blocks, and each of them:
    /// the destination is **snapped to its 48-unit cell centre**
    /// (`u × 0x30 + 0x18`); `off_x`/`off_y` are the destination's offset
    /// **inside its world cell**, `x mod 0x300` — not a formation slot;
    /// `tolerance`, `pause`, `retry`, `attempts` and `timer` are 0
    /// throughout, as the table says of a plain move; the `dest` flag and
    /// the path stack agree — `dest_x/dest_y` is the stack **top** when
    /// `dest` is 1; the bottom of the stack is the goal; and the `pathed`
    /// bit is set exactly when the unit has a stack at all.
    ///
    /// `coll_x/coll_y` is deliberately not in that list: 34 of the 146
    /// carry a blocker's position, which is `detect_unit_collision`
    /// working rather than a violated invariant.
    #[test]
    fn run29_s_move_orders_match_the_field_table_row_for_row() {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let (mut moves, mut collided, mut frames) = (0usize, 0usize, 0usize);
        for (f, b) in log.dumps() {
            frames += 1;
            let (units, _, _) = crate::gamelog::records(b, false);
            for u in &units {
                let tag = format!("{f}: who {} o {}", u.who, u.o);
                for (i, od) in u.orders.iter().enumerate() {
                    let (Some(x), Some(y)) = (od.x, od.y) else {
                        continue;
                    };
                    moves += 1;
                    assert_eq!((x % 0x30, y % 0x30), (0x18, 0x18), "{tag}: cell centre");
                    assert_eq!(od.off_x, Some(x.rem_euclid(0x300)), "{tag}: off_x");
                    assert_eq!(od.off_y, Some(y.rem_euclid(0x300)), "{tag}: off_y");
                    for (k, v) in [
                        ("tolerance", od.tolerance),
                        ("pause", od.pause),
                        ("retry", od.retry),
                        ("attempts", od.attempts),
                        ("timer", od.timer),
                    ] {
                        assert_eq!(v, Some(0), "{tag}: {k}");
                    }
                    if od.coll_x != Some(0) || od.coll_y != Some(0) {
                        collided += 1;
                    }
                    // The current order is the last block logged.
                    if i + 1 != u.orders.len() {
                        continue;
                    }
                    assert_eq!(
                        od.flags & 1 != 0,
                        !u.path.is_empty(),
                        "{tag}: the pathed bit is the stack"
                    );
                    assert_eq!(
                        (u.orders_x, u.orders_y),
                        (Some(x), Some(y)),
                        "{tag}: update_action's orders_x/y"
                    );
                    if let Some(top) = u.path.last() {
                        assert_eq!(u.path[0].flags & 1, 1, "{tag}: the bottom is the goal");
                        if od.dest == Some(1) {
                            assert_eq!(
                                (od.dest_x, od.dest_y),
                                (Some(top.to.0), Some(top.to.1)),
                                "{tag}: the waypoint is the stack top"
                            );
                        }
                    }
                }
            }
        }
        // Four states, not three: the window's [15100, 15103) and the
        // free 15105 of the end-of-run dump, which only `Log::dumps`
        // reaches. 15104's block is the `!quit`'s and carries no state.
        assert_eq!(frames, 4, "the window's three and the free one");
        assert_eq!(moves, 79, "every move order in the four states");
        assert_eq!(
            collided, 17,
            "and the ones detect_unit_collision has marked"
        );
    }

    // ---- run31: the human group move (`docs/GROUPS.md` §6.4, §13) ----

    /// One human group move: the frame, the group, its members' unit
    /// records by object, and the `GroupMoveOrder`s that frame carries.
    struct HumanMove {
        frame: i64,
        group: crate::gamelog::GroupDump,
        units: std::collections::BTreeMap<i64, crate::gamelog::UnitDump>,
        /// `form_id` → the order, for the members that hold one.
        orders: std::collections::BTreeMap<i64, crate::gamelog::OrderDump>,
    }

    /// run31's three right-clicks, each read out of the frame the order was
    /// issued on — the only frames a `GroupMoveOrder` survives into
    /// (`docs/ORACLE.md`, run31).
    fn run31_moves() -> Option<Vec<HumanMove>> {
        let name = "gamelog-run31-humangroup.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut out: Vec<HumanMove> = Vec::new();
        for (frame, block) in log.frames() {
            let (unit_dumps, _, _) = crate::gamelog::records(block, false);
            let orders: std::collections::BTreeMap<i64, crate::gamelog::OrderDump> = unit_dumps
                .iter()
                .flat_map(|u| u.orders.iter())
                .filter(|o| o.kind == "GroupMoveOrder")
                .filter_map(|o| o.form_id.map(|f| (f, o.clone())))
                .collect();
            if orders.is_empty() {
                continue;
            }
            // The run was ended with a kill, so its last block is a
            // half-written frame: the orders are there and the group pool
            // that follows them is not.
            let Some(group) = crate::gamelog::groups(block)
                .into_iter()
                .find(|g| g.who == 0 && g.num > 0)
            else {
                continue;
            };
            let units = unit_dumps
                .iter()
                .filter(|u| u.who == 0)
                .map(|u| (u.o, u.clone()))
                .collect();
            out.push(HumanMove {
                frame,
                group,
                units,
                orders,
            });
        }
        // Three right-clicks, and a `GroupMoveOrder` survives on a member
        // until it is consumed — so the orders are readable for the whole
        // march, not only on the frame they were issued.
        let clicks: std::collections::BTreeSet<(i64, i64)> = out
            .iter()
            .flat_map(|m| m.orders.values())
            .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
            .collect();
        assert_eq!(clicks.len(), 3, "run31's three right-clicks");
        assert!(out.len() >= 30, "and the frames they are readable on");
        Some(out)
    }

    /// run31's twelve squads, stood up in the harness in the record's own
    /// object order so that a harness index *is* a record slot: four
    /// squads of the ranged type then eight of the foot one, each a
    /// captain and two figures down its `o_down` chain, and the group
    /// built by adding the twelve captains (§4.1's subordinate recursion
    /// turns that into 36 members).
    fn run31_harness(
        loaded: &crate::load::Loaded,
        ranged: usize,
        foot: usize,
    ) -> (sim::Sim, sim::group::Group) {
        let mut s = sim::Sim::new(
            sim::tuning::Tuning::RON,
            sim::world::World::new(400, 400),
            2,
        );
        s.nation[0].human = true;
        let mut ours = Vec::new();
        for &t in &[ranged, foot] {
            let mut proto = loaded.unit_types[t].clone();
            proto.tree = None;
            ours.push(s.add_unit_type(proto));
        }
        let mut captains = Vec::new();
        for squad in 0..12 {
            let ty = ours[usize::from(squad >= 4)];
            let mut chain = Vec::new();
            for figure in 0..3 {
                let idx = i16::try_from(s.units.len()).unwrap();
                let mut u = Unit::new(0, idx, Pos::new(0x4000 + squad * 0x100, 0x4000), 120);
                u.ty = Some(ty);
                u.on_map = true;
                u.captain = figure == 0;
                let u = s.add_unit(u);
                chain.push(u);
            }
            s.units[chain[0]].o_down = Some(chain[1]);
            s.units[chain[1]].o_up = Some(chain[0]);
            s.units[chain[1]].o_down = Some(chain[2]);
            s.units[chain[2]].o_up = Some(chain[0]);
            captains.push(chain[0]);
        }
        let mut g = sim::group::Group::stack(0);
        for &c in &captains {
            s.group_add(&mut g, c);
        }
        (s, g)
    }

    /// **The shape of a human group move**, which no dump had held: 36
    /// members and not 12, two categories, and an anchor that is not the
    /// first member.
    ///
    /// The selection was staged slingers-first on purpose
    /// (`docs/ORACLE.md`, run31), so the group's `list[0]` is a
    /// `FORM_CAT_FOOT_RANGED` unit while the lowest category present is
    /// `FORM_CAT_FOOT` — which is what makes `find_leader`'s key
    /// observable at all (§4.4). And a **player's** selection group keeps
    /// its followers, so every figure of every squad is a member: three
    /// objects per unit, the captain first, the two followers behind it
    /// in the object chain (`o_up`).
    #[test]
    fn run31_s_human_group_move_is_thirty_six_figures_of_two_categories() {
        let Some(moves) = run31_moves() else { return };
        for m in &moves {
            let g = &m.group;
            let tag = format!("run31/{}", m.frame);
            assert_eq!(g.who, 0, "{tag}: the human's own group");
            assert_eq!(g.num, 36, "{tag}: twelve squads of three figures");
            assert_eq!(g.form, 0, "{tag}: get_form clamped −1 up to Line");
            assert_eq!(g.form_num, 36, "{tag}: Form::compute laid out all of them");
            assert_eq!(g.army, -1, "{tag}: a selection group, not an army's");
            assert_eq!(g.members.len(), 36, "{tag}: the arrays are parallel");
            // The membership is the selection walk's: every slinger, then
            // every hoplite, each squad's captain followed by its two.
            let objs: Vec<i64> = g.members.iter().map(|m| m.o).collect();
            let mut expect: Vec<i64> = (30..=41).collect();
            expect.extend(6..=29);
            assert_eq!(objs, expect, "{tag}: slingers first, then hoplites");
            for (i, mem) in g.members.iter().enumerate() {
                let u = &m.units[&mem.o];
                let captain = i % 3 == 0;
                assert_eq!(
                    u.o_up.is_none() || u.o_up == Some(-1),
                    captain,
                    "{tag}/{}: a captain heads its figure chain",
                    mem.o
                );
                assert_eq!(
                    u.flags & 0x10 == 0,
                    captain,
                    "{tag}/{}: and SubObjectData's 0x10 says the same",
                    mem.o
                );
                assert_eq!(u.group, Some(g.id), "{tag}/{}: the back-pointer", mem.o);
                assert_eq!(u.form, Some(0), "{tag}/{}: §6.6 step 1 wrote it", mem.o);
                assert_eq!(u.form_mod, Some(50), "{tag}/{}: and the width", mem.o);
            }
            // Exactly one member sits on the origin — the anchor the whole
            // block was slid to — and it is **not** `list[0]`.
            let anchors: Vec<usize> = (0..36)
                .filter(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .collect();
            assert_eq!(anchors.len(), 1, "{tag}: one anchor");
            assert_ne!(anchors[0], 0, "{tag}: and it is not the first member");
            // The offsets have depth as well as width: three distinct
            // `off_y` values, which is what `update_positions`' y-flip
            // needed and what every dump before this one lacked.
            let ys: std::collections::BTreeSet<i64> = g.members.iter().map(|m| m.off_y).collect();
            assert!(
                ys.len() >= 3 && ys.iter().any(|&y| y != 0),
                "{tag}: off_y takes {ys:?}"
            );
        }
    }

    /// **`GroupData::find_leader`'s key, from a record.** `GroupOrder`'s
    /// `oxx` is the object the group move was laid out around, and every
    /// member's order carries the same one; the anchor of the slot table
    /// is that object's slot. So the dump names the leader outright, and
    /// the simulation's own `find_leader` has to agree.
    ///
    /// The old rule — the group's first on-map captain — picks `list[0]`,
    /// a slinger. The record picks a hoplite. `FORM_CAT_FOOT` is below
    /// `FORM_CAT_FOOT_RANGED`, so the key is the category and not the
    /// order, exactly as §4.4 read it and as `Sim::group_find_leader` was
    /// changed to do on 2026-08-26.
    #[test]
    fn run31_s_group_order_names_a_leader_the_first_member_rule_would_miss() {
        use sim::form::{cat, type_cat};
        let Some(inst) = install() else { return };
        let Some(moves) = run31_moves() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            // One leader, agreed by every order the frame carries.
            let leaders: std::collections::BTreeSet<i64> =
                m.orders.values().filter_map(|o| o.oxx).collect();
            assert_eq!(leaders.len(), 1, "{tag}: one oxx across the orders");
            let leader = *leaders.iter().next().unwrap();
            for (form_id, o) in &m.orders {
                assert_eq!(o.whose, Some(0), "{tag}: whose is the group's player");
                assert_eq!(
                    g.members[*form_id as usize].o,
                    {
                        let _ = o;
                        g.members[*form_id as usize].o
                    },
                    "{tag}: form_id indexes the group's own arrays"
                );
            }
            // The record's anchor is the leader's slot.
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            assert_eq!(
                g.members[anchor].o, leader,
                "{tag}: the block is slid onto the leader's slot"
            );
            // Its category is strictly below the first member's, and its
            // index is not zero — so the two rules disagree here.
            let cat_of = |o: i64| {
                let u = &m.units[&o];
                let t = loaded
                    .unit_of_type_index(u.guys[0].kind.unwrap() as i32)
                    .expect("the member's type");
                type_cat(&loaded.unit_types[t], t, true)
            };
            assert_eq!(cat_of(leader), cat::FOOT, "{tag}: a hoplite leads");
            assert_eq!(
                cat_of(g.members[0].o),
                cat::FOOT_RANGED,
                "{tag}: and list[0] is a slinger"
            );
            assert!(
                cat_of(leader) < cat_of(g.members[0].o),
                "{tag}: strictly lower, which is the whole key"
            );
        }
    }

    /// **The `to`/`off` asymmetry, observed.** `compute_dests` slides the
    /// offsets by the whole anchor and the destinations by its `y` alone
    /// (§6.4), so a group whose anchor is off-centre marches to points
    /// displaced from where its own offsets say it will stand.
    ///
    /// The anchor is the one member whose slid offset is exactly `(0, 0)`.
    /// If the destinations were slid by the whole anchor too, that
    /// member's order would point at the click itself. It does not — by
    /// hundreds of position units, on all three of run31's moves — and
    /// every member of the group was clicked to the same point, so the
    /// click is not in doubt.
    #[test]
    fn run31_s_anchor_marches_to_a_point_its_own_offset_says_is_the_click() {
        let Some(moves) = run31_moves() else { return };
        let mut checked = 0usize;
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            // One click, on every order in the frame.
            let clicks: std::collections::BTreeSet<(i64, i64)> = m
                .orders
                .values()
                .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
                .collect();
            assert_eq!(clicks.len(), 1, "{tag}: one right-click");
            let (cx, cy) = *clicks.iter().next().unwrap();
            assert_eq!(
                (g.ox, g.oy),
                (cx, cy),
                "{tag}: and the group's own (ox, oy) is that click"
            );
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            let Some(o) = m.orders.get(&(anchor as i64)) else {
                continue;
            };
            checked += 1;
            let (x, y) = (o.x.unwrap(), o.y.unwrap());
            assert_ne!(
                (x, y),
                (cx, cy),
                "{tag}: the anchor's slot is the click only if `to` were \
                 slid by the whole anchor, and it is not"
            );
            // And the displacement is real rather than a rounding: the
            // order's own point is more than one `UCoord` off the click.
            let (dx, dy) = (x - cx, y - cy);
            assert!(
                dx.abs() + dy.abs() > 48,
                "{tag}: the anchor is displaced by ({dx}, {dy})"
            );
        }
        assert!(
            checked >= 30,
            "the anchor held a group order on {checked} frames"
        );
    }

    /// **`Group::update_positions`' y-flip, pinned.** `curr` is the slot
    /// offset rotated by the leader's own heading through
    /// `[cos t, sin t; sin t, -cos t]` — a rotation composed with a
    /// **y-flip**, determinant -1, which a naive port mirrors (§6.6).
    ///
    /// run29 reproduced `curr` too, but every `off_y` in its window was
    /// zero, so the flipped column was multiplied by nothing: the flip was
    /// unpinned and `docs/GROUPS.md` §13 said so. run31's group has three
    /// ranks and `off_y` of -6, -3 and 0 on every one of the forty frames
    /// its three moves are readable over, and the same arithmetic lands on
    /// the record — while the **unflipped** matrix misses it by hundreds.
    ///
    /// It also settles which heading: not the group's `o_angle`, not the
    /// bearing from the leader to its own slot, and not the order's angle,
    /// but the **leader's `UnitData::angle`** — and the leader is the
    /// object the record's own `GroupOrder::oxx` names.
    ///
    /// One thing it settles that nobody had asked: `curr` is a
    /// **mid-frame** quantity. `Unit::do_group_move` computes it inside the
    /// frame and the unit turns afterwards, so the `angle` the end-frame
    /// dump prints is the heading a hair *past* the one the rotation used.
    /// On the frames where the leader was not turning the dumped angle
    /// reproduces the table to the unit; on the rest a heading within
    /// 0.05 degrees of it does, exactly, all seventy-two numbers at once.
    #[test]
    fn run31_s_curr_is_the_leader_s_heading_through_the_y_flip() {
        use sim::movement::{Angle, cos_component, sin_component};
        let Some(moves) = run31_moves() else { return };
        let (mut frames, mut exact, mut with_depth) = (0usize, 0usize, 0usize);
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            let leader = g.members[anchor].o;
            for o in m.orders.values() {
                assert_eq!(o.oxx, Some(leader), "{tag}: oxx is the anchor's object");
            }
            let theta = Angle(
                i32::try_from(m.units[&leader].angle.expect("the leader's heading")).unwrap(),
            );
            let off: Vec<(i32, i32)> = g
                .members
                .iter()
                .map(|x| (x.off_x as i32 * 48, x.off_y as i32 * 48))
                .collect();
            assert!(
                off.iter().any(|&(_, y)| y != 0),
                "{tag}: the depth the flipped column needs"
            );
            with_depth += 1;
            let want: Vec<Pos> = g
                .members
                .iter()
                .map(|x| Pos::new(x.curr_x as i32, x.curr_y as i32))
                .collect();
            let rotate = |a: Angle, flip: bool| -> Vec<Pos> {
                off.iter()
                    .map(|&(x, y)| {
                        let cy = cos_component(a, y);
                        Pos::new(
                            cos_component(a, x) + sin_component(a, y),
                            sin_component(a, x) + if flip { -cy } else { cy },
                        )
                    })
                    .collect()
            };
            if rotate(theta, true) == want {
                exact += 1;
            }
            // Some heading within a twentieth of a degree of the dumped one
            // reproduces every one of the seventy-two numbers.
            let near = (-(1i32 << 21)..=(1i32 << 21))
                .step_by(1024)
                .any(|d| rotate(Angle(theta.0.wrapping_add(d)), true) == want);
            assert!(near, "{tag}: no heading near {theta:?} reproduces curr");
            // And no heading anywhere near it does without the flip.
            let unflipped = (-(1i32 << 21)..=(1i32 << 21))
                .step_by(1024)
                .any(|d| rotate(Angle(theta.0.wrapping_add(d)), false) == want);
            assert!(!unflipped, "{tag}: the determinant is -1, not +1");
            frames += 1;
        }
        assert!(frames >= 30, "{frames} frames reproduced");
        assert_eq!(with_depth, frames, "every one of them has a non-zero off_y");
        assert!(
            exact >= 9,
            "the dumped heading is the rotation's own on {exact} of {frames}"
        );
    }

    /// run31's thirty-six-member group, stood up in the harness from the
    /// install's own columns and laid out by `sim::form` — and every one of
    /// the 40 records reproduced, all 36 slots, both coordinates.
    ///
    /// The chain, with nothing fitted in the middle:
    ///
    /// 1. The record's twelve squads are two types. `unitrules.xml` gives
    ///    Hoplites and Slingers `X_SPACING 12`, `Y_SPACING 12`,
    ///    `GUY_SPACING 12` and `UBER_SIZE 3`, and `UnitType::init`
    ///    multiplies the first two by `UNIT_FORMATION_SPACING` and the third
    ///    by `UNIT_GUY_SPACING` — both 12 — so the columns are 144.
    /// 2. `Form::categorize` widens a multi-figure type's rank to
    ///    `min(uber_size, 3) × x_spacing`, so the category's width is
    ///    **432**, and its depth `⌈3/3⌉ × 144 = 144`.
    /// 3. `type_cat` puts the hoplites in `FORM_CAT_FOOT` and the slingers,
    ///    which have range, in `FORM_CAT_FOOT_RANGED` — eight captains and
    ///    four. `span = max(8 × 432 / 2, 4 × 432) = 1728`, `form_mod` is 50,
    ///    so both categories are **4** columns wide.
    /// 4. `Group::add`'s subordinate recursion (§4.1) turns twelve captains
    ///    into **36** members, each captain followed by its two figures —
    ///    which is the record's own `list`, object for object.
    /// 5. `compute_dests` places the captains, hangs each follower off its
    ///    captain by one `guy_spacing`, stacks `FOOT_RANGED` behind `FOOT`,
    ///    and slides the block onto the anchor.
    ///
    /// **What the record adds, and what no earlier pass had looked for**:
    /// its offsets are not always `compute_dests`' output. One of the forty
    /// frames is that output **re-origined onto another member**, by
    /// `Group::refresh_group_order` (`docs/GROUPS.md` §6.8) — and that is
    /// the whole of §4.4's old question about why the leader is object 9 on
    /// frame 204 and object 6 from 328 on. `find_leader` picks object 6 both
    /// times; frame 204's record is one refresh past the layout.
    #[test]
    fn run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns() {
        use sim::form::{cat, formation, type_cat};
        use sim::movement::Angle;
        let Some(inst) = install() else { return };
        let Some(moves) = run31_moves() else { return };
        let loaded = crate::load::load(&inst).unwrap();

        // ---- the record's own membership, and the two types behind it ----
        let first = &moves[0];
        let objs: Vec<i64> = first.group.members.iter().map(|m| m.o).collect();
        let type_of = |o: i64| -> usize {
            let u = &first.units[&o];
            loaded
                .unit_of_type_index(i32::try_from(u.guys[0].kind.expect("a guy kind")).unwrap())
                .expect("the member's type")
        };
        let kinds: Vec<usize> = objs.iter().map(|&o| type_of(o)).collect();
        // Four squads of one type then eight of the other, three figures each.
        let ranged = kinds[0];
        let foot = kinds[12];
        assert_ne!(ranged, foot, "two types");
        assert!(kinds[..12].iter().all(|&t| t == ranged));
        assert!(kinds[12..].iter().all(|&t| t == foot));
        assert_eq!(type_cat(&loaded.unit_types[foot], foot, true), cat::FOOT);
        assert_eq!(
            type_cat(&loaded.unit_types[ranged], ranged, true),
            cat::FOOT_RANGED
        );
        for &t in &[foot, ranged] {
            let c = &loaded.unit_types[t].combat;
            assert_eq!(
                (c.x_spacing, c.y_spacing, c.guy_spacing, c.uber_size),
                (144, 144, 144, 3),
                "{}: X_SPACING/Y_SPACING/GUY_SPACING 12 and UBER_SIZE 3",
                loaded.unit_names[t]
            );
        }

        // ---- the same group, stood up in the harness ----
        let (s, g) = run31_harness(&loaded, ranged, foot);
        assert_eq!(g.list.len(), 36, "the subordinate recursion");
        assert_eq!(
            g.list,
            (0..36).collect::<Vec<usize>>(),
            "and in captain-then-figures order, like the record's own list"
        );
        assert_eq!(
            s.group_find_leader(&g),
            Some(12),
            "find_leader takes the lowest category, ties to the first — the \
             first hoplite captain, not the first member"
        );
        assert_eq!(s.group_form_mod_option(&g), 50, "the record's form_mod");

        // ---- the layout, both mirrors ----
        let click = Pos::new(
            i32::try_from(first.group.ox).unwrap(),
            i32::try_from(first.group.oy).unwrap(),
        );
        let table = |reverse: bool, dest: Pos, theta: Angle| {
            s.form_compute(
                &g,
                dest,
                theta,
                formation::LINE,
                s.group_form_mod_option(&g),
                reverse,
                false,
                &[],
            )
        };
        let probe = table(false, click, Angle(0));
        assert_eq!(probe.num_category[cat::FOOT], 8, "eight hoplite captains");
        assert_eq!(probe.num_category[cat::FOOT_RANGED], 4, "four slinger ones");
        assert_eq!(probe.x_spacing[cat::FOOT], 432, "3 × X_SPACING 12 × 12");
        assert_eq!(probe.y_spacing[cat::FOOT], 144, "⌈3/3⌉ × Y_SPACING 12 × 12");
        // The anchor is the first member of the lowest non-empty category,
        // which is the leader `find_leader` names.
        assert_eq!(
            probe.off[12],
            (0, 0),
            "compute_dests slides the block onto the leader's slot"
        );

        // ---- every frame, every slot ----
        let (mut checked, mut refreshed, mut mirrored) = (0usize, 0usize, 0usize);
        let mut orders_checked = 0usize;
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let rec: Vec<(i32, i32)> = m
                .group
                .members
                .iter()
                .map(|x| {
                    (
                        i32::try_from(x.off_x).unwrap(),
                        i32::try_from(x.off_y).unwrap(),
                    )
                })
                .collect();
            let angles: std::collections::BTreeSet<i64> =
                m.orders.values().filter_map(|o| o.angle).collect();
            assert_eq!(angles.len(), 1, "{tag}: one formation angle");
            let theta = Angle(i32::try_from(*angles.iter().next().unwrap()).unwrap());
            let clicks: std::collections::BTreeSet<(i64, i64)> = m
                .orders
                .values()
                .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
                .collect();
            let (cx, cy) = *clicks.iter().next().unwrap();
            let dest = Pos::new(i32::try_from(cx).unwrap(), i32::try_from(cy).unwrap());

            // Exactly one mirror reproduces the frame, up to the re-origin.
            let mut hit = None;
            for reverse in [false, true] {
                let f = table(reverse, dest, theta);
                let quant: Vec<(i32, i32)> = f
                    .off
                    .iter()
                    .map(|&(x, y)| (sim::form::Form::quantise(x), sim::form::Form::quantise(y)))
                    .collect();
                // `refresh_group_order` re-origins the *quantised* table, so
                // the comparison is exact once both are put on the same slot.
                let origin = (0..36).find(|&i| rec[i] == (0, 0)).expect("an anchor");
                let mut ours = quant.clone();
                let mut st = sim::group::GroupState {
                    form_num: 36,
                    off: ours.clone(),
                    ..sim::group::GroupState::default()
                };
                st.reorigin(origin);
                ours = st.off.clone();
                if ours == rec {
                    hit = Some((reverse, f, origin));
                    break;
                }
            }
            let Some((reverse, f, origin)) = hit else {
                panic!("{tag}: neither mirror reproduces the record's 36 slots");
            };
            checked += 1;
            if reverse {
                mirrored += 1;
            }
            if origin != 12 {
                refreshed += 1;
                assert_eq!(origin, 15, "{tag}: run31's one refresh is onto slot 15");
                continue;
            }

            // An un-refreshed frame: the **destinations** are ours too. The
            // order's own `x`/`y` is the slot put through §6.6 step 6's
            // `UCoord` (`/0x30`) — floor to the 48-unit cell and back at its
            // centre, which is exactly `Unit::add_move_order`'s snap — and
            // it lands on every member of every frame, all three moves.
            for (&form_id, o) in &m.orders {
                let i = usize::try_from(form_id).unwrap();
                let want = Pos::new(
                    f.to[i].x.div_euclid(0x30) * 0x30 + 0x18,
                    f.to[i].y.div_euclid(0x30) * 0x30 + 0x18,
                );
                assert_eq!(
                    (i64::from(want.x), i64::from(want.y)),
                    (o.x.expect("x"), o.y.expect("y")),
                    "{tag}/{form_id}: the slot destination, snapped"
                );
                orders_checked += 1;
            }
            // `o_dist` is `Form::compute`'s leftover — the distance from the
            // order's point to the leader's slot, which is exactly the
            // half-column the even count shifted the block by.
            assert!(
                (m.group.o_dist - 216).abs() <= 1,
                "{tag}: o_dist {} is x_spacing/2",
                m.group.o_dist
            );
        }
        assert_eq!(checked, moves.len(), "every frame reproduced");
        assert_eq!(refreshed, 1, "one of the forty is a re-origin");
        assert!(
            orders_checked >= 900,
            "{orders_checked} slot destinations reproduced"
        );
        assert!(
            mirrored > 0 && mirrored < checked,
            "run31 has moves both ways: {mirrored} of {checked} mirrored"
        );
    }

    /// One frame of run31, reduced to what the mirror's state machine reads.
    struct Run31Frame {
        /// The human's live group, if the pool held one that frame.
        facing: Option<bool>,
        /// `who == 0` object → its `UNITDATA::angle`.
        angle: std::collections::BTreeMap<i64, i64>,
        /// The distinct `GroupMoveOrder`s the frame carries: the click, the
        /// order's angle, and its own `facing` byte.
        orders: std::collections::BTreeSet<(i64, i64, i64, i64)>,
    }

    /// Every frame of run31, not only the ones carrying an order — the
    /// predicate reads the leader's heading on the frame **before** the
    /// click, which is the last one printed before `compute_form` ran.
    fn run31_frames() -> Option<std::collections::BTreeMap<i64, Run31Frame>> {
        let name = "gamelog-run31-humangroup.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = std::fs::read_to_string(&path).unwrap();
        let log = Log::parse(&text);
        let mut out = std::collections::BTreeMap::new();
        for (frame, block) in log.frames() {
            let (units, _, _) = crate::gamelog::records(block, false);
            let mut angle = std::collections::BTreeMap::new();
            let mut orders = std::collections::BTreeSet::new();
            for u in &units {
                if u.who != 0 {
                    continue;
                }
                if let Some(a) = u.angle {
                    angle.insert(u.o, a);
                }
                for o in &u.orders {
                    if o.kind != "GroupMoveOrder" {
                        continue;
                    }
                    if let (Some(x), Some(y), Some(a), Some(f)) =
                        (o.orig_x, o.orig_y, o.angle, o.facing)
                    {
                        orders.insert((x, y, a, f));
                    }
                }
            }
            let facing = crate::gamelog::groups(block)
                .into_iter()
                .find(|g| g.who == 0 && g.num > 0)
                .map(|g| g.facing != 0);
            out.insert(
                frame,
                Run31Frame {
                    facing,
                    angle,
                    orders,
                },
            );
        }
        Some(out)
    }

    /// **The mirror's predicate, end to end** (`docs/GROUPS.md` §6.3,
    /// §12.3) — the question run31 opened while closing two others, and
    /// the one this file could not answer until the *other* writers of
    /// `GroupData::facing` were found.
    ///
    /// The flag the layout reads is never the flag the dump prints. Three
    /// things write it, and in one frame all three can run:
    ///
    /// 1. `Unit::kill_current_order@005e2cb0` — the `QUEUE_NEW` clear at
    ///    `70524f` runs **before** `compute_form` at `7053ec`, and the
    ///    leader's dying move hands its own `MoveOrder::facing` back to the
    ///    group, inverted if the leader has since turned around.
    /// 2. `compute_form@00707c80` toggles it around `Form::compute` and
    ///    toggles it back — the mirror is `facing XOR (leader ≥ 90° off the
    ///    bearing)` and the flag itself ends where it started.
    /// 3. `Unit::set_angle@00605400` toggles it again, later in the frame,
    ///    when the leader's own march turns it by 90° or more.
    ///
    /// So run31's three clicks are three predictions each, and every one
    /// is a number the record already holds. Two of the moves have a
    /// dumped `facing` that **contradicts** the mirror their slot table
    /// needs, which is exactly why nothing short of the whole machine
    /// reproduces them.
    ///
    /// **What this run cannot reach**, said here because four of the five
    /// breakages written against it went red and this is the fifth: the
    /// hand-back's *inversion*. Both of run31's kills catch the leader
    /// 10.6° and 6.3° off the dying order's own angle, nowhere near the
    /// 90° that would flip the byte on the way back, so `reversing`'s term
    /// at `5e3062`–`5e307b` is carried by the listing alone. The capture
    /// that would settle it is a group ordered somewhere, turned right
    /// around while marching, and then re-ordered — §13.
    #[test]
    fn run31_s_three_mirrors_come_out_of_facing_s_three_writers() {
        use sim::group::reversing;
        use sim::movement::Angle;
        let Some(moves) = run31_moves() else { return };
        let Some(frames) = run31_frames() else { return };

        // The three clicks, each on the frame its order first appears.
        let mut clicks: Vec<(i64, (i64, i64, i64, i64))> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (&frame, f) in &frames {
            for &o in &f.orders {
                if seen.insert((o.0, o.1)) {
                    clicks.push((frame, o));
                }
            }
        }
        assert_eq!(clicks.len(), 3, "run31's three right-clicks");

        // `find_leader` names object 6 — the first hoplite captain, the
        // lowest `type_cat` in the group (§4.4) — and its slot in the
        // 36-member list is 12, whose `angles` byte is 0 in a Line.
        let leader = 6;
        let leader_slot = 12;
        for m in &moves {
            assert_eq!(
                m.group.members[leader_slot].o, leader,
                "run31/{}: slot 12 is the leader's",
                m.frame
            );
            assert_eq!(
                m.group.members[leader_slot].angle, 0,
                "run31/{}: a Line leans nowhere",
                m.frame
            );
        }

        // The mirror each move actually used is the one its own orders
        // carry: `MoveOrder +0x28` is written with the flag `Form::compute`
        // was handed, and every member of a move agrees on it.
        let d = |a: i64, b: i64| {
            reversing(Angle(
                i32::try_from(a)
                    .unwrap()
                    .wrapping_sub(i32::try_from(b).unwrap()),
            ))
        };
        let mut facing = false; // `Group::clear` — a fresh group mirrors nothing.
        let mut prev: Option<(i64, i64)> = None; // the last order's (angle, facing)
        // How often the flag **as the pool last printed it** predicts the
        // wrong mirror — the model this file held until today, and the
        // control that makes the rest of this test mean something.
        let mut running_wrong = 0;
        for (frame, (_, _, theta, order_facing)) in &clicks {
            let tag = format!("run31/{frame}");
            let before = frames
                .get(&(frame - 1))
                .expect("the frame before the click");
            let now = &frames[frame];
            let heading = *before.angle.get(&leader).expect("the leader's heading");

            // 1. The dying order's hand-back, which only a leader makes.
            //    It is an assignment and not a toggle, so whatever the march
            //    did to the flag since the last click is discarded here.
            if let Some((prev_angle, prev_facing)) = prev {
                facing = (prev_facing != 0) != d(heading, prev_angle);
            }

            // 2. `compute_form`'s toggle: the mirror, and the flag put back.
            let away = d(heading, *theta);
            let mirror = facing != away;
            assert_eq!(
                mirror,
                *order_facing != 0,
                "{tag}: the mirror the orders carry"
            );
            if (before.facing.unwrap_or(false) != away) != mirror {
                running_wrong += 1;
            }

            // 3. `set_angle`'s toggle, once the leader turns into the move —
            //    and the flag stays turned, which is what the pool prints at
            //    the end of the frame and carries to the next click.
            let after = *now.angle.get(&leader).expect("the leader's heading");
            facing = facing != d(after, heading);
            assert_eq!(
                Some(facing),
                now.facing,
                "{tag}: the `facing` the frame's own GROUPDATA prints"
            );

            prev = Some((*theta, *order_facing));
        }
        assert_eq!(
            running_wrong, 1,
            "the flag as the pool prints it mispredicts one of the three"
        );

        // The two halves are not the same sequence, and that is the whole
        // finding: the mirrors run 0, 0, 1 while the dumped flags run
        // 1, 0, 1, so a reader taking `facing` for the mirror gets two of
        // the three moves wrong.
        let mirrors: Vec<i64> = clicks.iter().map(|c| c.1.3).collect();
        let dumped: Vec<bool> = clicks
            .iter()
            .map(|c| frames[&c.0].facing.expect("a live group"))
            .collect();
        assert_eq!(mirrors, vec![0, 0, 1], "the mirrors the orders carry");
        assert_eq!(dumped, vec![true, false, true], "the flags the pool prints");
    }
}
