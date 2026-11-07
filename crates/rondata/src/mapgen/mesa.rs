//! **Southwest Mesa's chain to `Cliffs::init`, out of the tests** (item
//! 3943; `docs/MAPGEN.md` §15, §18.3; parked 3942b). The port's walk from
//! the mask on 7805's seed to `Terrain.cpp` 894, which items 3869–3890
//! checked phase by phase on run1493, is the only source of a Southwest
//! Mesa game's cliffs: no dump prints `cliff_mining_data`. So the harness
//! that stands a Mesa game up (`diff::setup::build_sim`) runs it at the
//! lobby's own seed and lays [`sim::cliffs::CliffsInit::mining`] with
//! `Sim::place_cliffs`, and `mapgen`'s tests walk the same functions.

use super::{
    fertility, land_makes, place_all_inputs, resource_inputs, river_list, river_thresh, sea_map,
    standard_map_size, tileset_groups,
};
use crate::gamelog::Initial;
use sim::combat::Rng;
use sim::map_resources::place_resources;
use sim::map_rivers::{Phase, make_rivers};
use sim::map_values::{adjust_for_blocking, compute_all_vals};
use sim::mapgen::{Fractal, MapGrid, Regions, fertile_partitions};
use sim::terrain_groups::{Draws, change_bases, check_player_forest, nubify_forest, place_all};
use sim::terrain_init::terrain_init_without_heights;
use std::path::Path;

/// What the chain reads of a lobby: the setup trace's seed at `map.cpp`
/// 7805 (the word `Map::make` draws the mask from), the grid's extent,
/// the nation count and `World.seed` (every `Fractal`'s).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MesaLobby {
    pub at_7805: u32,
    pub xs: i32,
    pub ys: i32,
    pub nations: i32,
    pub seed: i32,
}

impl MesaLobby {
    /// From a kept file's [`super::StartGrid`]; `None` when its setup trace
    /// does not reach 7805.
    pub fn of(g: &super::StartGrid) -> Option<Self> {
        Some(Self {
            at_7805: g.at("map.cpp", 7805)?.2,
            xs: g.xs,
            ys: g.ys,
            nations: g.nations,
            seed: g.seed,
        })
    }

    /// From a start dump's [`Initial`], read as [`super::start_grid`] reads
    /// a head: `None` when the dump is not Southwest Mesa (`MAP_STYLE` 10),
    /// carries no setup trace to 7805, or has no extent.
    pub fn of_initial(init: &Initial<'_>) -> Option<Self> {
        let style = init
            .game_info
            .iter()
            .find(|(k, _)| *k == "MAP_STYLE")
            .and_then(|(_, v)| v.trim().parse::<i64>().ok());
        if style != Some(10) {
            return None;
        }
        let at_7805 = init
            .checksums
            .iter()
            .find(|c| c.file == "map.cpp" && c.line == 7805)?
            .seed;
        let world = |k: &str| {
            init.world
                .iter()
                .find(|(f, _)| *f == k)
                .and_then(|(_, v)| v.trim().parse::<i32>().ok())
        };
        let (xs, ys) = (world("xs")?, world("ys")?);
        (xs > 0 && ys > 0).then_some(Self {
            at_7805,
            xs,
            ys,
            nations: init.players.len() as i32,
            seed: world("seed")?,
        })
    }
}

/// The region records as the labelling leaves them: each label's cell
/// count, and the sea counter at its highest label. `adjust_for_blocking`
/// re-runs `find_all` on these; on Great Sahara, with no region
/// released and every cell labelled, it writes nothing.
pub fn regions_of(grid: &MapGrid) -> Regions {
    let mut r = Regions::default();
    for c in &grid.cells {
        r.count[c.region as usize] += 1;
        if c.region >= 0x40 {
            r.sea = r.sea.max(c.region);
        }
    }
    r
}

/// The grid and stream through `place_all` (7859), from the port's 7854
/// grid (3872's walk: the mask on 7805's seed, the zero-draw passes,
/// `fill_fertile` under `SW_desert`).
pub fn southwest_mesa_at_7859(root: &Path, g: &MesaLobby) -> (MapGrid, Draws) {
    let fert = fertility(root, "SOUTHWESTMESA.xml").expect("its fertility");
    let mut grid = MapGrid::new(g.xs, g.ys);
    let mut rng = Rng::new(g.at_7805);
    sim::mapgen::southwest_mesa_make_continents(&mut grid, g.nations, &mut rng);
    let mut regions = Regions::default();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    grid.fix_diag_land();
    grid.make_coastlines();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    let fractal = Fractal::init(g.xs, g.ys, fert.clump_factor, 0, g.seed as u32);
    grid.fill_fertile(&fractal, &fertile_partitions(&fert.baseland));
    let inputs = place_all_inputs(root, "SOUTHWESTMESA.xml", g.xs, g.ys, 10)
        .expect("Southwest Mesa's place_all inputs");
    let mut d = Draws::new(rng.seed);
    place_all(&mut grid, &mut d, &inputs);
    (grid, d)
}

/// The grid and stream at 7927: 3878's chain from
/// [`southwest_mesa_at_7859`] — `check_player_forest`, `nubify_forest`,
/// the base passes and `place_resources` — in its order.
pub fn southwest_mesa_at_7927(root: &Path, g: &MesaLobby) -> (MapGrid, Draws) {
    let (mut grid, mut d) = southwest_mesa_at_7859(root, g);
    let fert = fertility(root, "SOUTHWESTMESA.xml").expect("its fertility");
    let ts = tileset_groups(root, &fert.tileset).expect("SW_desert's groups");
    check_player_forest(&mut grid, sim::tuning::Tuning::RON.city_center_radius);
    nubify_forest(&mut grid, &mut d);
    change_bases(&mut grid, &mut d, &ts);
    let inputs = resource_inputs(root, "SOUTHWESTMESA.xml", g.xs, g.ys, g.nations)
        .expect("Southwest Mesa's rows");
    place_resources(&mut grid, &mut d, &inputs, g.nations);
    (grid, d)
}

/// The grid and stream at 7945: the port's 7927 grid
/// ([`southwest_mesa_at_7927`]), then `adjust_for_blocking` and
/// `compute_all_vals`, as 3882 checked them.
pub fn southwest_mesa_at_7945(root: &Path, g: &MesaLobby) -> (MapGrid, Draws) {
    let (mut grid, mut d) = southwest_mesa_at_7927(root, g);
    let makes = land_makes(root).expect("rules.xml's LANDS");
    let inputs = place_all_inputs(root, "SOUTHWESTMESA.xml", g.xs, g.ys, 10)
        .expect("Southwest Mesa's place_all inputs");
    let mut regions = regions_of(&grid);
    adjust_for_blocking(&mut grid, &mut regions, &inputs.templates);
    compute_all_vals(&mut grid, &mut d, &makes);
    (grid, d)
}

/// [`southwest_mesa_at_894`]'s answer: the grid, the heights, the
/// stream, parts 10–13 and what `Cliffs::init` did.
pub type MesaAt894 = (
    MapGrid,
    Vec<sim::single::Single>,
    Draws,
    [u32; 4],
    sim::cliffs::CliffsInit,
);

/// One Southwest Mesa run carried to `Terrain.cpp` 894 (item 3890):
/// step 8's 5118 grid (`southwest_mesa_s_rivers_are_5012_to_5118`'s
/// walk), then the heights in `Terrain::init`'s order — `generate_land`
/// and the coastal pass (706), the oil cells' terraform (738), the
/// troughs (`RIVERLIST` empty: nothing to calculate) and
/// `adjust_for_mountains` — and `Cliffs::init` (`sim::cliffs`).
pub fn southwest_mesa_at_894(root: &Path, g: &MesaLobby) -> MesaAt894 {
    let install = crate::Install::new(root);
    let meshes = crate::mountains::meshes(&install);
    let sea = sea_map(root, "SOUTHWESTMESA.xml").expect("SOUTHWESTMESA.xml's SEA_MAP");
    let thresh = river_thresh(root, "SOUTHWESTMESA.xml").expect("RIVER_THRESHOLD");
    let rivers = river_list(root, "SOUTHWESTMESA.xml").expect("a RIVERLIST");
    let standard = standard_map_size(root).expect("the Standard map size");
    let fert = fertility(root, "SOUTHWESTMESA.xml").expect("its fertility");
    let ts = tileset_groups(root, &fert.tileset).expect("SW_desert's groups");
    let inputs = place_all_inputs(root, "SOUTHWESTMESA.xml", g.xs, g.ys, 10)
        .expect("Southwest Mesa's place_all inputs");
    let (mut grid, mut d) = southwest_mesa_at_7945(root, g);
    let (_, mut coord, half) = terrain_init_without_heights(&mut grid, sea)
        .unwrap_or_else(|u| panic!("an unported arm: {u:?}"));
    let p = half.parts().expect("empty half-land lists");
    let mut regions = regions_of(&grid);
    let phase = Phase {
        thresh,
        city_radius: sim::tuning::Tuning::RON.city_center_radius,
        rivers: &rivers,
        standard,
        seed: g.seed as u32,
        nations: g.nations,
    };
    make_rivers(&mut grid, &mut regions, &phase, &mut d, |_, _, _| {});
    let f = sim::map_heights::fractals(grid.xs, grid.ys, g.seed as u32);
    let mut z = sim::map_heights::generate_land(&grid, &coord, &f);
    sim::map_heights::build_coastal_data(&grid, &mut coord, &mut z, g.seed as u32);
    sim::map_heights::create_doobers(&grid, &mut z);
    let combo = sim::halfland::Combo {
        coast_base: ts.coast_base,
        num_tiles: [
            Some(ts.base_tiles),
            Some(ts.coastal_tiles),
            Some(ts.ocean_tiles),
            None,
        ],
    };
    let troughs = sim::map_troughs::calculate_all(
        &mut grid,
        &mut coord,
        &mut z,
        ts.riverbase,
        combo,
        0,
        None,
    )
    .unwrap_or_else(|e| panic!("the troughs: {e:?}"));
    assert!(troughs.sections.is_empty(), "a trough on Southwest Mesa");
    sim::map_heights::adjust_for_mountains(&grid, &mut z, &inputs.templates, &meshes);
    let goods = resource_inputs(root, "SOUTHWESTMESA.xml", g.xs, g.ys, g.nations)
        .expect("Southwest Mesa's rows")
        .goods;
    let cliffs = sim::cliffs::init(&mut grid, &mut coord, &mut z, &inputs.cliffs, &goods, combo)
        .unwrap_or_else(|e| panic!("Cliffs::init: {e:?}"));
    (grid, z, d, [p[0], p[1], p[2], p[3]], cliffs)
}
