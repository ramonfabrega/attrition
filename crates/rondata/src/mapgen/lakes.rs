//! **Great Lakes' chain end to end, from the lobby** (item 4047;
//! `docs/MAPGEN.md` §15, §16, §17; `docs/START.md` §2 step 4). Every phase
//! of the chain is ported and checked at its own checkpoint by `mapgen`'s
//! tests, each driven from a traced word: the seed at `map.cpp` 7805, and
//! the start dump's `World.seed`. This module runs the same phases on
//! **one** stream from `setup.cpp` 716 — the word `Setup::build_game` leaves
//! after `world->seed` — with `World.seed` the builder's own, so nothing of
//! a dump enters: `Map::load_map_data`'s two tileset draws and `Map::make`'s
//! `base_edge` draw (§4) are walked forward rather than stepped back from
//! 7805, and every later phase takes the word the one before it left.
//!
//! [`great_lakes_map`] hands each checkpoint to a callback with the state
//! the trace prints there (the seed, parts 2, 5, 6 and 10–13), so a caller
//! holding a setup trace bisects the composed chain phase by phase.

use super::{fertility_of, mesa::regions_of};
use super::{
    lake_style, land_makes, place_all_inputs_of, resource_inputs, river_list, river_thresh,
    standard_map_size, tile_chances, tileset_groups,
};
use sim::combat::Rng;
use sim::mapgen::map_resources::place_resources;
use sim::mapgen::map_rivers::{Phase, make_rivers};
use sim::mapgen::map_values::{adjust_for_blocking, compute_all_vals};
use sim::mapgen::terrain_groups::{
    Draws, change_bases, check_player_forest, nubify_forest, place_all,
};
use sim::mapgen::terrain_init::{HalfLand, coord_flags};
use sim::mapgen::{Fractal, MapGrid, Regions, fertile_partitions};
use std::path::Path;

/// What the chain reads of a lobby, and nothing of a dump: the stream's
/// word at `setup.cpp` 716 (after `world->seed`'s draw), the grid's extent,
/// the nation count `Map::make` takes, `MAP_SIZE` (the overage allowance)
/// and `World.seed` (every `Fractal`'s).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LakesLobby {
    pub at_716: u32,
    pub xs: i32,
    pub ys: i32,
    pub nations: i32,
    pub map_size: i64,
    pub seed: i32,
}

/// One checkpoint's state, as the setup trace prints it: the file and
/// line, `game_random`'s word, and parts 2 (the start arrays), 5 (the
/// cells), 6 (the tiles) and 10–13 (the half-land lists).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    pub file: &'static str,
    pub line: i64,
    pub seed: u32,
    pub part2: u32,
    pub part5: u32,
    pub part6: u32,
    pub lists: [u32; 4],
}

/// The chain's answer at `Terrain.cpp` 894: the grid, the corner heights,
/// the stream (its log every draw from 716 on), the drawn tileset and
/// `base_edge`, and the `CoordInfo` flags.
pub struct LakesMap {
    pub grid: MapGrid,
    pub z: Vec<sim::num::single::Single>,
    pub draws: Draws,
    pub tileset: String,
    pub base_edge: i32,
    pub coord: Vec<u16>,
}

/// Why the chain stopped short.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A table the chain reads is missing from the install.
    Data(&'static str),
    /// `make_continents` found the lobby past its allowance.
    Continents,
    /// A later phase refused an arm it does not port.
    Unported(String),
}

/// **`Map::init_map_data@0069f250`'s draw, forward** (`docs/MAPGEN.md` §4):
/// when the file's `TILECHANCE` chances sum to at least 2, one
/// `Random::get(0, 0xffff)`, and the first entry whose running total is at
/// least `r % T` (the test is `≤`). `None` when the file draws nothing.
/// The chosen name is taken to be an available tileset: the second draw an
/// unavailable one makes is not walked (parked 3819b).
pub fn tileset_draw(chances: &[(String, i32)], rng: &mut Rng) -> Option<String> {
    let total: i32 = chances.iter().map(|c| c.1).sum();
    if chances.len() < 2 || total < 2 {
        return None;
    }
    let r = rng.get(0, 0xffff) % total;
    let mut run = 0;
    chances.iter().find_map(|(name, c)| {
        run += c;
        (r <= run).then(|| name.clone())
    })
}

const STYLE: &str = "GREATLAKES.xml";

/// **Great Lakes' map, composed** — `Map::load_map_data`, `Map::make`,
/// `Terrain::init` to 894 and `Terrain::mark_forest`, in
/// `Setup::build_game`'s order, on one stream
/// from `lobby.at_716`. `check` is handed every checkpoint the setup trace
/// prints for these phases, in order: 7805, 7807, 7854, 7859, 7861, 7865,
/// 7870, 7927, 7937, 7945, `Terrain.cpp` 720 and 738, `map.cpp` 5012,
/// 5014, 5076, 5116 and 5118, `Terrain.cpp` 894, and `setup.cpp` 884
/// after `Terrain::mark_forest`.
pub fn great_lakes_map(
    root: &Path,
    lobby: &LakesLobby,
    max_range_349: i32,
    mut check: impl FnMut(&Checkpoint),
) -> Result<LakesMap, Refusal> {
    let (xs, ys) = (lobby.xs, lobby.ys);
    let seed = lobby.seed as u32;
    let empty = HalfLand::default()
        .parts()
        .ok_or(Refusal::Data("the half-land walk"))?;
    let mut say = |file: &'static str, line: i64, g: &MapGrid, rng: &Rng, lists: [u32; 4]| {
        check(&Checkpoint {
            file,
            line,
            seed: rng.seed,
            part2: g.start_part(),
            part5: g.cell_part(),
            part6: g.tile_part(),
            lists,
        });
    };
    // §4: `load_map_data` (default.xml, then the style's file), then
    // `base_edge = r % 4` (default.xml's −1; the style leaves it).
    let mut rng = Rng::new(lobby.at_716);
    let default = tile_chances(root, "default.xml").ok_or(Refusal::Data("default.xml"))?;
    let chances = tile_chances(root, STYLE).ok_or(Refusal::Data(STYLE))?;
    let first = tileset_draw(&default, &mut rng);
    let tileset = tileset_draw(&chances, &mut rng)
        .or(first)
        .ok_or(Refusal::Data("a drawn tileset"))?;
    let base_edge = rng.get(0, 0xffff) % 4;
    let mut grid = MapGrid::new(xs, ys);
    say("map.cpp", 7805, &grid, &rng, empty);
    // §5: the mask.
    let style = lake_style(
        root,
        STYLE,
        (xs, ys, lobby.nations, lobby.map_size),
        base_edge,
        max_range_349,
    )
    .ok_or(Refusal::Data("Great Lakes' style"))?;
    let mut d = Draws::new(rng.seed);
    sim::mapgen::map_lakes::great_lakes_make_continents(&mut grid, lobby.nations, style, &mut d)
        .ok_or(Refusal::Continents)?;
    say("map.cpp", 7807, &grid, &d.rng, empty);
    // §6: the zero-draw passes and `fill_fertile`.
    let fert = fertility_of(root, STYLE, &tileset).ok_or(Refusal::Data("its fertility"))?;
    let mut regions = Regions::default();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    grid.fix_diag_land();
    grid.make_coastlines();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    let fractal = Fractal::init(xs, ys, fert.clump_factor, 0, seed);
    grid.fill_fertile(&fractal, &fertile_partitions(&fert.baseland));
    say("map.cpp", 7854, &grid, &d.rng, empty);
    // §7: the groups and their tail.
    let groups = place_all_inputs_of(root, STYLE, &tileset, xs, ys, 14)
        .ok_or(Refusal::Data("Great Lakes' place_all inputs"))?;
    place_all(&mut grid, &mut d, &groups);
    say("map.cpp", 7859, &grid, &d.rng, empty);
    let ts = tileset_groups(root, &tileset).ok_or(Refusal::Data("the tileset's groups"))?;
    check_player_forest(&mut grid, sim::core::tuning::Tuning::RON.city_center_radius);
    say("map.cpp", 7861, &grid, &d.rng, empty);
    nubify_forest(&mut grid, &mut d);
    say("map.cpp", 7865, &grid, &d.rng, empty);
    change_bases(&mut grid, &mut d, &ts);
    say("map.cpp", 7870, &grid, &d.rng, empty);
    // §8: resources.
    let rows = resource_inputs(root, STYLE, xs, ys, lobby.nations)
        .ok_or(Refusal::Data("Great Lakes' rows"))?;
    place_resources(&mut grid, &mut d, &rows, lobby.nations);
    say("map.cpp", 7927, &grid, &d.rng, empty);
    // §9: blocking, the regions again, the values.
    let makes = land_makes(root).ok_or(Refusal::Data("rules.xml's LANDS"))?;
    let mut regions = regions_of(&grid);
    adjust_for_blocking(&mut grid, &mut regions, &groups.templates);
    say("map.cpp", 7937, &grid, &d.rng, empty);
    compute_all_vals(&mut grid, &mut d, &makes);
    say("map.cpp", 7945, &grid, &d.rng, empty);
    // §10, §16: `Terrain::init` — the heights to 706, the half-land (720),
    // the doobers (738), the beach (5012).
    let mut coord = coord_flags(&grid);
    let f = sim::mapgen::map_heights::fractals(xs, ys, seed);
    let mut z = sim::mapgen::map_heights::generate_land(&grid, &coord, &f);
    sim::mapgen::map_heights::build_coastal_data(&grid, &mut coord, &mut z, seed);
    let combo = sim::mapgen::halfland::Combo {
        coast_base: ts.coast_base,
        num_tiles: [
            Some(ts.base_tiles),
            Some(ts.coastal_tiles),
            Some(ts.ocean_tiles),
            None,
        ],
    };
    let mut depth = 0;
    let lists = sim::mapgen::terrain_init::check_and_set_halfland_heights(
        &mut grid, &mut coord, &z, combo, &mut depth,
    )
    .map_err(|e| Refusal::Unported(format!("the half-land: {e:?}")))?;
    let lists = lists
        .parts()
        .ok_or(Refusal::Data("the half-land lists' walk"))?;
    say("Terrain.cpp", 720, &grid, &d.rng, lists);
    sim::mapgen::halfland::create_doobers_flags(&mut grid);
    sim::mapgen::map_heights::create_doobers(&grid, &mut z);
    say("Terrain.cpp", 738, &grid, &d.rng, lists);
    sim::mapgen::halfland::Pass::new(&mut grid, &mut coord, &z, combo)
        .with_depth(depth)
        .convert_beach_to_sandy()
        .map_err(|e| Refusal::Unported(format!("the beach: {e:?}")))?;
    // §15 step 8: the rivers, 5012..5118 (`make_rivers` prints each).
    let thresh = river_thresh(root, STYLE).ok_or(Refusal::Data("RIVER_THRESHOLD"))?;
    let rivers = river_list(root, STYLE).ok_or(Refusal::Data("a RIVERLIST"))?;
    let standard = standard_map_size(root).ok_or(Refusal::Data("the Standard map size"))?;
    let mut regions = regions_of(&grid);
    let phase = Phase {
        thresh,
        city_radius: sim::core::tuning::Tuning::RON.city_center_radius,
        rivers: &rivers,
        standard,
        seed,
        nations: lobby.nations,
    };
    make_rivers(&mut grid, &mut regions, &phase, &mut d, |line, g, d| {
        say("map.cpp", line, g, &d.rng, lists)
    });
    // §17: the troughs, then `adjust_for_mountains` (894).
    let install = crate::Install::new(root);
    let meshes = crate::mountains::meshes(&install);
    sim::mapgen::map_troughs::calculate_all(
        &mut grid,
        &mut coord,
        &mut z,
        ts.riverbase,
        combo,
        depth,
        None,
    )
    .map_err(|e| Refusal::Unported(format!("the troughs: {e:?}")))?;
    sim::mapgen::map_heights::adjust_for_mountains(&grid, &mut z, &groups.templates, &meshes);
    if !sim::mapgen::map_heights::slope_tiles(&grid).is_empty() {
        return Err(Refusal::Unported(
            "a cell flagged 0x40 reaches Cliffs::init's slope arm".into(),
        ));
    }
    say("Terrain.cpp", 894, &grid, &d.rng, lists);
    // `Setup::build_game`'s `Terrain::mark_forest`, after `Terrain::init`
    // returns (setup.cpp 759, 766 move no world part), checked at 884.
    grid.mark_forest();
    say("setup.cpp", 884, &grid, &d.rng, lists);
    Ok(LakesMap {
        grid,
        z,
        draws: d,
        tileset,
        base_edge,
        coord,
    })
}
