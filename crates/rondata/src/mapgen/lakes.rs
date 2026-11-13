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
//! [`composed_map`] hands each checkpoint to a callback with the state
//! the trace prints there (the seed, parts 2, 5, 6 and 10–13), so a caller
//! holding a setup trace bisects the composed chain phase by phase.
//!
//! **East Indies and Great Sahara on the same walk** (item 4186): East
//! Indies' phases are Great Lakes' calls with its own mask, file and style
//! id; Great Sahara's whole-land mask runs `Terrain::init` without
//! heights, then the rivers, then the heights, the order `mapgen`'s Great
//! Sahara tests checked. Every start dump of the two styles with a setup
//! trace is the falsifier: the sync word after `Herd::create_units`.

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
    /// Each seat's `LeaderData::get_team` (`PLAYER team`, 8 for none),
    /// which East Meets West's `fill_cont` reads (item 4196).
    pub teams: [Option<i32>; 8],
    /// `GAMEINFO`'s `TEAM_STYLE`: Colonial Powers' outsiders are the
    /// unteamed computers under style 7 (`docs/MAPGEN.md` §32.1).
    pub team_style: i64,
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
    /// `Terrain::init`'s half-land lists, as `check_and_set_halfland` left
    /// them at `Terrain.cpp` 720 (parts 10–13 of every later checkpoint).
    pub halfland: HalfLand,
    /// The drawn tileset's numbers a placed building's ground reads:
    /// `weatherbase` and the `is_combo_acceptable` numbers the half-land
    /// pass ran on (`Terrain::object_placed`, item 4136).
    pub land: sim::world::terrain::TileSetLand,
    /// What `Cliffs::init` did, on a style that runs it (Southwest Mesa,
    /// item 4196): its `mining` lists are what a game lays by queue index
    /// (`Sim::place_cliffs`). `None` on the styles that place no cliff.
    pub cliffs: Option<sim::mapgen::cliffs::CliffsInit>,
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
/// The gate is the sum alone (`init_map_data`'s `T − 1 < 1`), so a style
/// with one entry of chance 100 — SAHARA.xml's `GS_Desert` — still draws
/// (item 4186).
/// The chosen name is taken to be an available tileset: the second draw an
/// unavailable one makes is not walked (parked 3819b).
pub fn tileset_draw(chances: &[(String, i32)], rng: &mut Rng) -> Option<String> {
    let total: i32 = chances.iter().map(|c| c.1).sum();
    if total < 2 {
        return None;
    }
    let r = rng.get(0, 0xffff) % total;
    let mut run = 0;
    chances.iter().find_map(|(name, c)| {
        run += c;
        (r <= run).then(|| name.clone())
    })
}

/// The map styles whose chain is composed from the lobby: the two whose
/// masks are grown as `Regions` by the same growers
/// (`sim::mapgen::map_lakes`), and whose later phases are the same calls
/// in the same order (`docs/MAPGEN.md` §15: every East Indies phase was
/// checked on the passes Great Lakes' had built, with no `sim` change
/// past the mask).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// `MAP_STYLE` 14, `MapGreatLakes::make_continents@00699e40`.
    GreatLakes,
    /// `MAP_STYLE` 18, `MapEastIndies::make_continents@00697540` (item 4186).
    EastIndies,
    /// `MAP_STYLE` 7, `MapGreatSahara::make_continents@0069bf70` (item
    /// 4186): the whole map land, so `Terrain::init`'s half-land gate opens
    /// on no cell and the heights follow the rivers
    /// (`sim::mapgen::map_heights::heights`), as `mapgen`'s Great Sahara
    /// tests walk them from 7805.
    GreatSahara,
    /// `MAP_STYLE` 10, `MapSouthwestMesa::make_continents` (item 4196):
    /// Great Sahara's whole-land mask at another address, its height-free
    /// `Terrain::init` and rivers, then the heights in `Terrain::init`'s
    /// own order and `Cliffs::init` — the chain items 3869–3890 checked
    /// phase by phase on run1493 (`crate::mapgen::mesa`).
    SouthwestMesa,
    /// The styles `mapgen`'s chain walk carries 7805 → 894 on their own
    /// takes (items 3941, 3944 to 3953; `docs/MAPGEN.md` §24 to §34),
    /// composed here by item 4196: Old World (6), Amazon Basin (8) and
    /// Himalayas (9) on Great Sahara's mask listing; African Watering Hole
    /// (11), Mediterranean (12), Australian Outback (13), New World (16),
    /// Colonial Powers (17), East Meets West (19), Sea Power (20) and Nile
    /// Delta (21) on their own.
    OldWorld,
    AmazonBasin,
    Himalayas,
    AfricanWateringHole,
    Mediterranean,
    AustralianOutback,
    NewWorld,
    ColonialPowers,
    EastMeetsWest,
    SeaPower,
    NileDelta,
}

/// Every composed style, `MAP_STYLE` order aside.
const STYLES: [Style; 15] = [
    Style::GreatLakes,
    Style::EastIndies,
    Style::GreatSahara,
    Style::SouthwestMesa,
    Style::OldWorld,
    Style::AmazonBasin,
    Style::Himalayas,
    Style::AfricanWateringHole,
    Style::Mediterranean,
    Style::AustralianOutback,
    Style::NewWorld,
    Style::ColonialPowers,
    Style::EastMeetsWest,
    Style::SeaPower,
    Style::NileDelta,
];

impl Style {
    /// The style a lobby's `MAP_STYLE` names, when its chain is composed.
    pub fn of(map_style: i64) -> Option<Style> {
        STYLES.into_iter().find(|s| i64::from(s.id()) == map_style)
    }

    /// The style's file under `mapstyles/`.
    pub fn file(self) -> &'static str {
        match self {
            Style::GreatLakes => "GREATLAKES.xml",
            Style::EastIndies => "EASTINDIES.xml",
            Style::GreatSahara => "SAHARA.xml",
            Style::SouthwestMesa => "SOUTHWESTMESA.xml",
            Style::OldWorld => "OLDWORLD.xml",
            Style::AmazonBasin => "AMAZON.xml",
            Style::Himalayas => "HIMALAYAS.xml",
            Style::AfricanWateringHole => "AFRICA.xml",
            Style::Mediterranean => "MEDITERRANEAN.xml",
            Style::AustralianOutback => "OUTBACK.xml",
            Style::NewWorld => "NEWWORLD.xml",
            Style::ColonialPowers => "COLONIALPOWERS.xml",
            Style::EastMeetsWest => "EASTWEST.xml",
            Style::SeaPower => "SEAPOWER.xml",
            Style::NileDelta => "NILEDELTA.xml",
        }
    }

    /// `MAP_STYLE`, which `place_all`'s inputs read.
    pub fn id(self) -> i32 {
        match self {
            Style::GreatLakes => 14,
            Style::EastIndies => 18,
            Style::GreatSahara => 7,
            Style::SouthwestMesa => 10,
            Style::OldWorld => 6,
            Style::AmazonBasin => 8,
            Style::Himalayas => 9,
            Style::AfricanWateringHole => 11,
            Style::Mediterranean => 12,
            Style::AustralianOutback => 13,
            Style::NewWorld => 16,
            Style::ColonialPowers => 17,
            Style::EastMeetsWest => 19,
            Style::SeaPower => 20,
            Style::NileDelta => 21,
        }
    }
}

/// **Great Lakes' map, composed** ([`composed_map`] on [`Style::GreatLakes`]).
pub fn great_lakes_map(
    root: &Path,
    lobby: &LakesLobby,
    max_range_349: i32,
    check: impl FnMut(&Checkpoint),
) -> Result<LakesMap, Refusal> {
    composed_map(root, Style::GreatLakes, lobby, max_range_349, check)
}

/// **A style's map, composed** — `Map::load_map_data`, `Map::make`,
/// `Terrain::init` to 894 and `Terrain::mark_forest`, in
/// `Setup::build_game`'s order, on one stream
/// from `lobby.at_716`. `check` is handed every checkpoint the setup trace
/// prints for these phases, in order: 7805, 7807, 7854, 7859, 7861, 7865,
/// 7870, 7927, 7937, 7945, `Terrain.cpp` 720 and 738, `map.cpp` 5012,
/// 5014, 5076, 5116 and 5118 (those `make_rivers` prints), `Terrain.cpp`
/// 894, and `setup.cpp` 884 after `Terrain::mark_forest`.
pub fn composed_map(
    root: &Path,
    which: Style,
    lobby: &LakesLobby,
    max_range_349: i32,
    mut check: impl FnMut(&Checkpoint),
) -> Result<LakesMap, Refusal> {
    let style_file = which.file();
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
    let chances = tile_chances(root, style_file).ok_or(Refusal::Data("the style's tilesets"))?;
    let first = tileset_draw(&default, &mut rng);
    let tileset = tileset_draw(&chances, &mut rng)
        .or(first)
        .ok_or(Refusal::Data("a drawn tileset"))?;
    let base_edge = rng.get(0, 0xffff) % 4;
    let mut grid = MapGrid::new(xs, ys);
    say("map.cpp", 7805, &grid, &rng, empty);
    // §5: the mask.
    let mut d = Draws::new(rng.seed);
    let style = || {
        lake_style(
            root,
            style_file,
            (xs, ys, lobby.nations, lobby.map_size),
            base_edge,
            max_range_349,
        )
        .ok_or(Refusal::Data("the style's mask fields"))
    };
    let made = match which {
        Style::GreatLakes => sim::mapgen::map_lakes::great_lakes_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
        Style::EastIndies => sim::mapgen::map_east_indies::east_indies_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
        // Old World's, Amazon Basin's and Himalayas' listings are Great
        // Sahara's at other addresses (§25.1, §26.1, §27).
        Style::GreatSahara | Style::OldWorld | Style::AmazonBasin | Style::Himalayas => {
            sim::mapgen::great_sahara_make_continents(&mut grid, lobby.nations, &mut d.rng);
            Some(())
        }
        Style::SouthwestMesa => {
            sim::mapgen::southwest_mesa_make_continents(&mut grid, lobby.nations, &mut d.rng);
            Some(())
        }
        Style::AfricanWateringHole => {
            sim::mapgen::map_watering_hole::african_watering_hole_make_continents(
                &mut grid,
                lobby.nations,
                style()?,
                &mut d,
            )
            .map(|_| ())
        }
        Style::Mediterranean => sim::mapgen::map_mediterranean::mediterranean_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
        Style::AustralianOutback => {
            sim::mapgen::map_outback::australian_outback_make_continents(
                &mut grid,
                lobby.nations,
                style()?,
                &mut d,
            );
            Some(())
        }
        Style::NewWorld => sim::mapgen::map_new_world::new_world_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
        Style::ColonialPowers => {
            // The outsiders are unteamed computers under team style 7
            // (§32.1); no lobby on disk is one, and the arm is unread.
            if lobby.team_style == 7 {
                return Err(Refusal::Unported(
                    "Colonial Powers' outsiders under team style 7".into(),
                ));
            }
            sim::mapgen::map_colonial::colonial_powers_make_continents(
                &mut grid,
                lobby.nations,
                0,
                style()?,
                &mut d,
            )
            .map(|_| ())
        }
        Style::EastMeetsWest => {
            // `num_sides` is the nations when no seat has a team (run1524,
            // the default lobby, §24); a team game's sides are unread.
            let none = sim::mapgen::map_east_west::NO_TEAM;
            if lobby.teams.iter().flatten().any(|&t| t != none) {
                return Err(Refusal::Unported(
                    "East Meets West's sides in a team game".into(),
                ));
            }
            sim::mapgen::map_east_west::east_west_make_continents(
                &mut grid,
                lobby.nations,
                lobby.nations,
                &lobby.teams,
                style()?,
                &mut d,
            )
            .map(|_| ())
        }
        Style::SeaPower => sim::mapgen::map_sea_power::sea_power_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
        Style::NileDelta => sim::mapgen::map_nile_delta::nile_delta_make_continents(
            &mut grid,
            lobby.nations,
            style()?,
            &mut d,
        )
        .map(|_| ()),
    };
    made.ok_or(Refusal::Continents)?;
    say("map.cpp", 7807, &grid, &d.rng, empty);
    // §6: the zero-draw passes and `fill_fertile`.
    let fert = fertility_of(root, style_file, &tileset).ok_or(Refusal::Data("its fertility"))?;
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
    let groups = place_all_inputs_of(root, style_file, &tileset, xs, ys, which.id())
        .ok_or(Refusal::Data("the style's place_all inputs"))?;
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
    let rows = resource_inputs(root, style_file, xs, ys, lobby.nations)
        .ok_or(Refusal::Data("the style's resource rows"))?;
    place_resources(&mut grid, &mut d, &rows, lobby.nations);
    say("map.cpp", 7927, &grid, &d.rng, empty);
    // §9: blocking, the regions again, the values.
    let makes = land_makes(root).ok_or(Refusal::Data("rules.xml's LANDS"))?;
    let mut regions = regions_of(&grid);
    adjust_for_blocking(&mut grid, &mut regions, &groups.templates);
    say("map.cpp", 7937, &grid, &d.rng, empty);
    compute_all_vals(&mut grid, &mut d, &makes);
    say("map.cpp", 7945, &grid, &d.rng, empty);
    let combo = sim::mapgen::halfland::Combo {
        coast_base: ts.coast_base,
        num_tiles: [
            Some(ts.base_tiles),
            Some(ts.coastal_tiles),
            Some(ts.ocean_tiles),
            None,
        ],
    };
    let thresh = river_thresh(root, style_file).ok_or(Refusal::Data("RIVER_THRESHOLD"))?;
    let rivers = river_list(root, style_file).ok_or(Refusal::Data("a RIVERLIST"))?;
    let standard = standard_map_size(root).ok_or(Refusal::Data("the Standard map size"))?;
    let phase = Phase {
        thresh,
        city_radius: sim::core::tuning::Tuning::RON.city_center_radius,
        rivers: &rivers,
        standard,
        seed,
        nations: lobby.nations,
    };
    let install = crate::Install::new(root);
    let meshes = crate::mountains::meshes(&install);
    if !matches!(which, Style::GreatLakes | Style::EastIndies) {
        // §10, §15 steps 6 to 8 as `mapgen`'s chain walk takes them on
        // every other style's own take (items 3885 to 3890 on Southwest
        // Mesa, 3941 and 3944 to 3953): `Terrain::init` without heights
        // when no cell opens the half-land gate (734 → 5012, the lists
        // empty); a mask with water opens it (African Watering Hole's
        // lake, §28), and then Great Lakes' path to the beach. The rivers
        // (5012..5118), then the heights on the grid the rivers left when
        // they were not made before the half-land, the troughs with
        // `move_goody`'s good arm, `adjust_for_mountains`, and
        // `Cliffs::init` — inert where `place_all` laid no cliff.
        let sea = super::sea_map(root, style_file).ok_or(Refusal::Data("SEA_MAP"))?;
        let mut probe = grid.clone();
        let (mut coord, halfland, wet) =
            match sim::mapgen::terrain_init::terrain_init_without_heights(&mut probe, sea) {
                Ok((_, coord, half)) => {
                    grid = probe;
                    (coord, half, None)
                }
                Err(sim::mapgen::terrain_init::Unported::HalfLand(_)) => {
                    let mut coord = coord_flags(&grid);
                    let f = sim::mapgen::map_heights::fractals(xs, ys, seed);
                    let mut z = sim::mapgen::map_heights::generate_land(&grid, &coord, &f);
                    sim::mapgen::map_heights::build_coastal_data(&grid, &mut coord, &mut z, seed);
                    let mut depth = 0;
                    let lists = sim::mapgen::terrain_init::check_and_set_halfland_heights(
                        &mut grid, &mut coord, &z, combo, &mut depth,
                    )
                    .map_err(|e| Refusal::Unported(format!("the half-land: {e:?}")))?;
                    sim::mapgen::halfland::create_doobers_flags(&mut grid);
                    sim::mapgen::map_heights::create_doobers(&grid, &mut z);
                    sim::mapgen::halfland::Pass::new(&mut grid, &mut coord, &z, combo)
                        .with_depth(depth)
                        .convert_beach_to_sandy()
                        .map_err(|e| Refusal::Unported(format!("the beach: {e:?}")))?;
                    (coord, lists, Some((z, depth)))
                }
                Err(e) => return Err(Refusal::Unported(format!("Terrain::init: {e:?}"))),
            };
        let lists = halfland
            .parts()
            .ok_or(Refusal::Data("the half-land lists' walk"))?;
        say("Terrain.cpp", 720, &grid, &d.rng, lists);
        say("Terrain.cpp", 738, &grid, &d.rng, lists);
        let mut regions = regions_of(&grid);
        make_rivers(&mut grid, &mut regions, &phase, &mut d, |line, g, d| {
            say("map.cpp", line, g, &d.rng, lists)
        });
        let (mut z, depth) = match wet {
            Some(zd) => zd,
            None => {
                let f = sim::mapgen::map_heights::fractals(xs, ys, seed);
                let mut z = sim::mapgen::map_heights::generate_land(&grid, &coord, &f);
                sim::mapgen::map_heights::build_coastal_data(&grid, &mut coord, &mut z, seed);
                sim::mapgen::map_heights::create_doobers(&grid, &mut z);
                (z, 0)
            }
        };
        sim::mapgen::map_troughs::calculate_all(
            &mut grid,
            &mut coord,
            &mut z,
            ts.riverbase,
            combo,
            depth,
            Some(&rows.goods),
        )
        .map_err(|e| Refusal::Unported(format!("the troughs: {e:?}")))?;
        sim::mapgen::map_heights::adjust_for_mountains(&grid, &mut z, &groups.templates, &meshes);
        let cliffs = sim::mapgen::cliffs::init(
            &mut grid,
            &mut coord,
            &mut z,
            &groups.cliffs,
            &rows.goods,
            combo,
        )
        .map_err(|e| Refusal::Unported(format!("Cliffs::init: {e:?}")))?;
        say("Terrain.cpp", 894, &grid, &d.rng, lists);
        grid.mark_forest();
        say("setup.cpp", 884, &grid, &d.rng, lists);
        return Ok(LakesMap {
            grid,
            z,
            draws: d,
            tileset,
            base_edge,
            coord,
            halfland,
            land: sim::world::terrain::TileSetLand {
                weatherbase: ts.weatherbase as u8,
                combo,
            },
            cliffs: Some(cliffs),
        });
    }
    // §10, §16: `Terrain::init` — the heights to 706, the half-land (720),
    // the doobers (738), the beach (5012).
    let mut coord = coord_flags(&grid);
    let f = sim::mapgen::map_heights::fractals(xs, ys, seed);
    let mut z = sim::mapgen::map_heights::generate_land(&grid, &coord, &f);
    sim::mapgen::map_heights::build_coastal_data(&grid, &mut coord, &mut z, seed);
    let mut depth = 0;
    let lists = sim::mapgen::terrain_init::check_and_set_halfland_heights(
        &mut grid, &mut coord, &z, combo, &mut depth,
    )
    .map_err(|e| Refusal::Unported(format!("the half-land: {e:?}")))?;
    let halfland = lists;
    let lists = halfland
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
    let mut regions = regions_of(&grid);
    make_rivers(&mut grid, &mut regions, &phase, &mut d, |line, g, d| {
        say("map.cpp", line, g, &d.rng, lists)
    });
    // §17: the troughs, then `adjust_for_mountains` (894).
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
        halfland,
        land: sim::world::terrain::TileSetLand {
            weatherbase: ts.weatherbase as u8,
            combo,
        },
        cliffs: None,
    })
}
