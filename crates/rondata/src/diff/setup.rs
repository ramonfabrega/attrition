//! Standing a `sim::Sim` up from a start-of-game dump, and stepping it.

use super::*;

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
#[derive(Clone, Debug)]
pub struct Built {
    /// Lab-only overwrite observation; disabled in ordinary replay.
    pub correction_audit: Option<CorrectionAudit>,
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
pub(crate) fn guy_of(g: &crate::gamelog::Guy) -> Option<sim::anim::Guy> {
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

pub(crate) fn pos_of(p: LogPos) -> Pos {
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
    let (world, region_map) = world_from(&w.fields().to_vec(), &heights, &mut notes);
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

    // **And the starting position is re-laid, now that the nations are
    // known.** `Loaded::sim` calls `Sim::start_techs` for every player as
    // it builds them, which is before this function has read a single
    // `LEADER` record — so every leader's opening tech set was computed
    // for `tribe = 0`, the Aztecs. `Leader::init` sets the nation first
    // and its unit arm is `has_preq && tribe_can_type` (`docs/TECH.md`,
    // "The starting position"), so the bits are a *function* of the
    // nation: the British AI of run53 started owning **Atl-Atls**, the
    // Aztec light-infantry variant, and not **Slingers**, whose
    // `TRIBE_MASK` excludes the Aztecs. Nothing here draws, and the
    // players are still empty — the cities and units below are placed
    // after this — so the honest fix is to lay them down again against
    // the leader's real tribe. The lobby is installed above too, so this
    // also gives `starting_age` the dump's own value rather than the
    // default.
    for who in 0..players {
        sim.start_techs(who as sim::Player);
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
        // `UnitData::stance` (`sim::stance`). A unit stood up from a dump
        // takes the dump's, which is the stronger seed: it carries whatever
        // a player's clicks have done to the byte since the unit was born.
        // Where the dump is thin enough not to print it, the crate computes
        // the born value — `Unit::init`'s, since a **seeded** unit is a
        // starting one and never came out of `Build::train`.
        unit.stance = match u.stance {
            Some(v) => u8::try_from(v).unwrap_or(0),
            None => kind.map_or(0, |t| sim.init_stance(u.who as sim::Player, t)),
        };
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
    // **The reveals the dump's own `seen2` stands in for.** The fog grid
    // is installed rather than swept, so every `reveal_fog` the original
    // had already made before the block was written is missing — and with
    // it every rare already in the leader's `new_rares`. Replayed here,
    // once, after the goods are in and the leaders' `human` bits are set,
    // because `Leader::new_rare` reads both (item 207,
    // `sim::Sim::seed_new_rares_from_fog`).
    sim.seed_new_rares_from_fog();
    let seeded: usize = (0..players).map(|w| sim.ai[w].new_rares.len()).sum();
    if seeded > 0 {
        notes.push(format!(
            "rares: {seeded} already-seen good(s) replayed into new_rares"
        ));
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
        correction_audit: None,
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
pub(crate) const PERSONALITY_BEFORE: (&str, i64) = ("leaders.cpp", 13383);
pub(crate) const PERSONALITY_AFTER: (&str, i64) = ("leaders.cpp", 13457);

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
pub(crate) const FARM: i32 = 417;
pub(crate) const WOODCUTTER: i32 = 418;

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
pub(crate) fn start_of_game(
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

impl Built {
    /// The frame the sim just stepped, folded by phase — the harness's
    /// answer to `tools/trace/report.py … sites`, which folds the
    /// original's draws by return address (`docs/SYNC.md` §4.2). Phases
    /// that drew nothing are dropped; the unit loop is one entry per unit
    /// that drew, because that is the half that has to be lined up against
    /// the trace. `None` when nothing drew, or when the marks are off.
    pub(crate) fn phase_fold(&self) -> Option<String> {
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
            if let Some(audit) = &mut self.correction_audit {
                let before = self.sim.rng.seed;
                audit
                    .seed
                    .observe(frame, before != theirs, || format!("{before} -> {theirs}"));
            }
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
                        if let Some(audit) = &mut self.correction_audit {
                            audit.unlinked_units += 1;
                        }
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
                        let before = self
                            .correction_audit
                            .as_ref()
                            .map(|_| self.sim.units[u].clone());
                        let d =
                            self.sim
                                .reseat_animal(u, pos_of(state.pos), state.goal.map(pos_of));
                        if let (Some(audit), Some(before)) = (&mut self.correction_audit, before) {
                            let after = &self.sim.units[u];
                            audit.gaia_reseat.observe(frame, before != *after, || format!(
                                "unit {who}/{o}: position {:?} -> {:?}; path {:?} -> {:?}; orders {:?} -> {:?}",
                                before.pos, after.pos, before.path, after.path, before.orders, after.orders));
                        }
                        if d > 0 {
                            reseated += 1;
                            drift = drift.max(d);
                        }
                        for (n, g) in guys.iter().enumerate() {
                            if let Some(guy) = guy_of(g) {
                                super::corrections::install_clock(
                                    &mut self.sim,
                                    &mut self.correction_audit,
                                    frame,
                                    u,
                                    n,
                                    guy,
                                );
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
                            if let Some(audit) = &mut self.correction_audit {
                                audit.predicate_skipped_units += 1;
                            }
                            skipped += 1;
                            continue;
                        }
                        arrivals += 1;
                    }
                    for (n, g) in guys.iter().enumerate() {
                        if let Some(guy) = guy_of(g) {
                            super::corrections::install_clock(
                                &mut self.sim,
                                &mut self.correction_audit,
                                frame,
                                u,
                                n,
                                guy,
                            );
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
    pub(crate) fn build_ids(&self, handle: usize) -> Option<(i64, i64)> {
        let o = self
            .builds
            .iter()
            .find(|(h, _)| *h == handle)
            .map(|(_, o)| *o)?;
        Some((i64::from(self.sim.buildings[handle].owner), o))
    }

    /// `(whom, ox)` for a simulation unit handle.
    pub(crate) fn unit_ids(&self, handle: usize) -> Option<(i64, i64)> {
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
    pub(crate) fn target_ids(&self, unit: usize, order: &sim::orders::Order) -> Option<(i64, i64)> {
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

/// Fills `init`'s checksum trace and height grid from the first sibling
/// dump that has each, when `init` itself has none (see [`run_traced`]).
/// Whether two dumps opened on the same board: every starting building, by
/// owner, object number and position. It is the test a *terraformed* field
/// needs — [`Initial::heights`] is the grid after `Wall::init` has flattened
/// each footprint, so a sibling that placed its buildings elsewhere has a
/// different table however well its map seed matches.
pub(crate) fn same_start(a: &Initial, b: &Initial) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;
    use crate::gamelog::LeaderDump;
    use sim::ai::{MAKE_SLOTS, MakeObject};

    use crate::testenv::{dump, install};

    /// **A leader's opening unit set is its own nation's** — the ordering
    /// [`build_sim`] had wrong until 2026-09-04.
    ///
    /// `Loaded::sim` lays the starting position down as it builds the
    /// players, which is before this function has read a `LEADER` record,
    /// so every capture opened with its leaders' unit bits computed for
    /// `tribe = 0`. run53's AI is British (tribe 11) and its light
    /// infantry is the generic `Slingers`, whose `TRIBE_MASK` clears the
    /// Aztec bit; `Atl-Atls` is the Aztec variant and carries mask `0x1`.
    /// Before the fix the British AI owned `Atl-Atls` and not `Slingers`,
    /// which made `Slingers` `RESEARCHABLE` and put it in front of
    /// `Leader::upgrade_units` — two draws on frame 6779
    /// (`docs/TECH.md`, "The starting position is a function of the
    /// nation").
    ///
    /// This asserts the state rather than the frame, because the frame is
    /// four thousand ticks downstream of it and says nothing about why.
    #[test]
    fn a_leader_s_opening_units_are_its_own_nation_s() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        let tree = &built.sim.tech_tree;
        let id = |name: &str| {
            tree.types
                .iter()
                .position(|d| d.name == name)
                .unwrap_or_else(|| panic!("no {name} in the tree"))
        };
        let (slingers, atlatls) = (id("Slingers"), id("Atl-Atls"));
        assert_eq!(tree.types[atlatls].tribe_mask, 1, "Atl-Atls is the Aztecs'");
        assert_eq!(
            tree.types[slingers].tribe_mask & 1,
            0,
            "and Slingers is not"
        );
        // Player 1 is the British AI, player 0 the Nubian human; neither
        // is the Aztecs, so neither owns Atl-Atls and both own Slingers.
        assert_eq!(built.sim.tech[1].tribe, 11, "run53's AI is British");
        for who in 0..2 {
            let p = &built.sim.tech[who];
            assert!(
                p.tech[slingers] && p.tech_at_start[slingers],
                "player {who} (tribe {}) does not start with Slingers",
                p.tribe
            );
            assert!(
                !p.tech[atlatls],
                "player {who} (tribe {}) starts with the Aztecs' Atl-Atls",
                p.tribe
            );
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
        let text = crate::capture::read(&path);
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
        let scalars: [(&str, i32); 15] = [
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
            // `pop` is the city lifecycle's, not the sweep's, and it was
            // identically zero here until 2026-09-04 — `create_units`'
            // `base` and `research_techs`' are `pop × 1000 / city_num` and
            // `pop × 200 / city_num`, so every value either produced was
            // zero (`docs/AI.md` §27). It is in this list now because it
            // is in the record, which is the rule the widening ledger
            // exists for.
            ("pop", c.pop),
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
            ("reg_pop", &c.reg_pop),
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

    fn make_list_of(leader: crate::gamelog::Block<'_>) -> [MakeObject; MAKE_SLOTS] {
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
            let text = crate::capture::read(&path);
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
        let text = crate::capture::read(&path);
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

    /// **`Unit::init`'s stance, re-derived and checked against two maps'
    /// first blocks** — item 190.
    ///
    /// `UnitData::stance` is one byte that means four things, and this crate
    /// wrote a flat 1 into it on every unit it created until 2026-09-03.
    /// `Unit::init@00612100:282–309` asks the *type* which of the four kinds
    /// it carries (`UnitTypeData::get_stance_type@0061d350`) and then reads a
    /// different place for each — the player's options, the leader's flags,
    /// the lobby (`sim::stance`).
    ///
    /// Two captures print the whole table at their start block, and they
    /// agree object for object across two maps and two lobbies:
    ///
    /// | who | type | stance | why |
    /// | --- | --- | --- | --- |
    /// | 0 (human) | citizen, `0x32` | **0** | `Worker` → `leader_options.peasants`, and `init` leaves it 0 |
    /// | 1 (AI) | citizen, `0x32` | **1** | `Worker` → `(starting_resources == 8) + 1`, and the lobby is 1 |
    /// | 0 and 1 | scout, `0x45` | **1** | `Caster` → `!bit4`, and `init` leaves bit 4 clear |
    /// | 8 (gaia) | `407`/`411`/`412` | **0** | `None` → the `default:` arm |
    ///
    /// The human/AI split is the half that is easy to get backwards:
    /// `leader_flags & 4` is the **human** bit and it is the human that takes
    /// the option, so the branch a name would put on the player is the AI's.
    /// The leaders' own flags say so — run69's leader 0 is `176160775`
    /// (`… 111`, bit 2 set) and its leader 1 `176160787` (`… 10011`, clear).
    ///
    /// This asserts the derivation, not the seed: [`build_sim`] takes the
    /// dump's byte where the dump prints one, so a wrong rule cannot hide
    /// behind a right seed.
    #[test]
    fn init_stance_is_the_original_s_on_both_maps_first_blocks() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut checked = 0usize;
        let mut seen_zero = 0usize;
        let mut seen_one = 0usize;
        for name in [
            "gamelog-run68-islands-citizenword.txt",
            "gamelog-run69-greatlakes-3k.txt",
        ] {
            let Some(path) = dump(name) else {
                eprintln!("skipping {name} (set RON_GAMELOG_DIR)");
                continue;
            };
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let init = log.initial().unwrap();
            let built = build_sim(&loaded, &init, Tuning::RON);
            for link in &built.units {
                // Gaia's two bands are `who` 8 and 9; the rule answers 0 for
                // them through the `None` arm, and the dump agrees, but the
                // leader table has no row for them.
                let Some(ty) = link.kind else { continue };
                let Some(them) = init
                    .units
                    .iter()
                    .find(|u| u.who == link.who && u.o == link.o)
                    .and_then(|u| u.stance)
                else {
                    continue;
                };
                let who = u8::try_from(link.who).unwrap_or(0);
                let ours = built.sim.init_stance(who, ty);
                assert_eq!(
                    i64::from(ours),
                    them,
                    "{name}: {}/{} (type_index {}) stance",
                    link.who,
                    link.o,
                    built.sim.unit_types[ty].type_index
                );
                checked += 1;
                match them {
                    0 => seen_zero += 1,
                    1 => seen_one += 1,
                    _ => {}
                }
            }
        }
        if checked == 0 {
            eprintln!("skipping: no capture with a start block");
            return;
        }
        assert!(
            seen_zero > 0 && seen_one > 0,
            "a check that only ever sees one value is not checking the switch \
             ({checked} units, {seen_zero} at 0, {seen_one} at 1)"
        );
    }

    /// The personality block of one leader's start-of-game `LEADERDATA`
    /// (`LEADERS=9` under `[Start Game]`), by field name.
    fn personality_of<'a>(log: &'a Log<'a>, who: i64) -> Vec<(&'a str, i64)> {
        log.game()
            .expect("GAME")
            .kids("LEADERDATA")
            .find(|l| l.int("who") == Some(who))
            .and_then(|l| l.kid("PERSONALITY"))
            .expect("a PERSONALITY block")
            .fields()
            .filter_map(|(k, v)| Some((k, v.trim().parse().ok()?)))
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
        let text = crate::capture::read(&path);
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
        let text = crate::capture::read(&run18b);
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
}
