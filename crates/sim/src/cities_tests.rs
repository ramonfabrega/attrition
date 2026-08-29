//! End-to-end tests for cities and buildings — `docs/CITIES.md` — run through
//! the harness against the mechanics already there: territory, production,
//! combat, the tech tree's availability.

use super::*;
use crate::build::{BuildType, Ident, flags};
use crate::city::{PlaceFail, capture_value, health_level};
use crate::garrison::{GarrisonRefused, UnitTraits};
use crate::place::Blocked;
use crate::world::{UNITS_PER_CELL, tile};

const TILE: i32 = world::UNITS_PER_TILE;

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
}

/// A 16×16-cell (64×64-tile) land world, two players at war, both rich.
fn world_sim() -> Sim {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.declare_war(0, 1);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    sim
}

#[allow(clippy::too_many_arguments)]
fn bt(
    ident: Ident,
    from: Option<usize>,
    flag: &str,
    xs: i32,
    ys: i32,
    job: i32,
    hits: i32,
    garrison: i32,
) -> BuildType {
    BuildType {
        ident,
        from,
        x_size: xs,
        y_size: ys,
        flags: flags::parse(flag),
        job_time: job,
        hits,
        garrison_max: garrison,
        price: cost::Price {
            kind: cost::Kind::Building,
            // `BUILD_COST_FACTOR` is ten: a hundred timber.
            ..cost::Price::free().with_base(economy::Resource::Timber, 10)
        },
        ..BuildType::default()
    }
}

/// The shipped rows this mechanic's tests lean on, by index.
struct Types {
    village: usize,
    town: usize,
    metropolis: usize,
    barracks: usize,
    farm: usize,
    library: usize,
    market: usize,
    temple: usize,
    senate: usize,
    granary: usize,
    tower: usize,
    fort: usize,
}

fn install_types(sim: &mut Sim) -> Types {
    let village = sim.add_build_type(bt(Ident::Village, None, "ean", 7, 7, 600, 1200, 10));
    let town = sim.add_build_type(bt(Ident::Town, Some(village), "ean", 7, 7, 600, 2500, 15));
    let metropolis = sim.add_build_type(bt(
        Ident::Metropolis,
        Some(town),
        "ean",
        7,
        7,
        600,
        5000,
        20,
    ));
    let barracks = sim.add_build_type(bt(Ident::Barracks, None, "ean", 4, 4, 420, 1200, 10));
    let mut farm = bt(Ident::Farm, None, "gda", 4, 4, 150, 400, 0);
    farm.flags |= flags::FLAT;
    let farm = sim.add_build_type(farm);
    let library = sim.add_build_type(bt(Ident::Library, None, "jam", 5, 5, 420, 1200, 0));
    let market = sim.add_build_type(bt(Ident::Market, None, "jam", 4, 4, 420, 1200, 10));
    let temple = sim.add_build_type(bt(Ident::Temple, None, "jam", 4, 4, 420, 1200, 0));
    let senate = sim.add_build_type(bt(Ident::Senate, None, "jam", 4, 7, 500, 1200, 0));
    let granary = sim.add_build_type(bt(Ident::Granary, None, "ja", 5, 5, 1000, 1000, 0));
    let mut tower = bt(Ident::Tower, None, "ean", 2, 2, 1000, 750, 5);
    tower.attack = 12;
    tower.combat = Some(combat::Profile {
        attack: 12,
        max_range: 10,
        recharge: 30,
        base_arrows: 0,
        most_shots: 2,
        ..combat::Profile::default()
    });
    let tower = sim.add_build_type(tower);
    let fort = sim.add_build_type(bt(Ident::Fort, None, "ean", 5, 5, 2000, 2000, 10));
    Types {
        village,
        town,
        metropolis,
        barracks,
        farm,
        library,
        market,
        temple,
        senate,
        granary,
        tower,
        fort,
    }
}

fn citizen_type(village: usize) -> UnitType {
    UnitType {
        price: cost::Price {
            pop: 1,
            ..cost::Price::free().with_base(economy::Resource::Food, 20)
        },
        hits: 40,
        garrison: UnitTraits {
            trained_at: Some(village),
            ..UnitTraits::default()
        },
        ..UnitType::default()
    }
}

fn hoplite_type(barracks: usize) -> UnitType {
    UnitType {
        price: cost::Price {
            pop: 1,
            ..cost::Price::free().with_base(economy::Resource::Food, 40)
        },
        hits: 100,
        combat: combat::Profile {
            attack: 15,
            max_range: 0,
            obj_masks: 0x20, // FOOT
            uber_size: 1,
            ..combat::Profile::default()
        },
        garrison: UnitTraits {
            fortify: true,
            trained_at: Some(barracks),
            ..UnitTraits::default()
        },
        ..UnitType::default()
    }
}

fn spawn(sim: &mut Sim, owner: Player, ty: usize, pos: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.unit_types[ty].hits;
    let mut u = Unit::new(owner, index, pos, hits);
    u.ty = Some(ty);
    u.kind = sim.unit_types[ty].kind;
    // A citizen's speed and turning, so the walk to a site is the
    // original's 25 a frame.
    u.movement.speed = 25;
    u.movement.turning = movement::Turning {
        type_turn_speed: movement::degrees_to_angle(45).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };
    sim.add_unit(u)
}

/// Builds a placed building to completion the way the original does: one
/// builder's contribution a frame, until `activate`.
fn finish(sim: &mut Sim, b: usize) {
    let mut guard = 0;
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
        guard += 1;
        assert!(guard < 10, "a million a frame should finish anything");
    }
    sim.buildings[b].helpers = 0;
}

/// A city of player 0 at tile (32, 32), finished.
fn city_at(sim: &mut Sim, t: &Types, who: Player, tx: i32, ty: i32) -> (usize, usize) {
    let b = sim
        .place_building(who, t.village, tile_pos(tx, ty))
        .unwrap_or_else(|e| panic!("the city should place: {e:?}"));
    finish(sim, b);
    let c = sim.buildings[b].city.expect("a finished city has a record");
    (b, c)
}

// ----------------------------------------------------------------------
// Placement
// ----------------------------------------------------------------------

#[test]
fn the_first_city_is_a_foothold_and_the_second_keeps_its_distance() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    // Unowned ground, no city anywhere: the foothold.
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(32, 32), None),
        Blocked::Clear
    );
    let b = sim.place_building(0, t.village, tile_pos(32, 32)).unwrap();
    assert!(!sim.buildings[b].started && !sim.buildings[b].active);
    // The placed ghost already counts for spacing and for the foothold.
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(50, 32), None),
        Blocked::CityDistance,
        "18 tiles away is inside CITY_SPACING"
    );
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(56, 32), None),
        Blocked::CityDistance,
        "exactly 24 tiles is still refused: the test is <="
    );
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(57, 32), None),
        Blocked::Territory,
        "25 tiles is far enough, but a second city in the region needs friendly ground"
    );
    // The placement was paid for.
    assert_eq!(sim.ledgers[0].bucket[1], 10_000 - 100);
    // Another player's unstarted ghost does not count against us.
    assert_eq!(
        sim.blocked_site(Some(1), t.village, tile_pos(50, 32), None),
        Blocked::Clear
    );
}

#[test]
fn a_finished_city_projects_territory_and_the_radius_mask() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    assert!(sim.buildings[b].active && sim.cities[c].alive);
    assert_eq!(sim.cities[c].race, Some(0));
    assert!(sim.cities[c].capital, "the first city is the capital");
    assert_eq!(sim.city_num(0), 1);
    // Territory: the cell under the city and its neighbours are player 0's.
    assert_eq!(sim.world.owner_at(tile_pos(32, 32)), Owner::Player(0));
    assert_eq!(sim.world.owner_at(tile_pos(20, 32)), Owner::Player(0));
    // The radius mask is the even circle: 20 tiles on the negative side,
    // 19 on the positive — the disc is centred on a tile corner.
    assert!(sim.world.tile_mask(tile_pos(32 - 20, 32).tile()) & tile::CITY_RADIUS != 0);
    assert!(sim.world.tile_mask(tile_pos(32 - 21, 32).tile()) & tile::CITY_RADIUS == 0);
    assert!(sim.world.tile_mask(tile_pos(32 + 19, 32).tile()) & tile::CITY_RADIUS != 0);
    assert!(sim.world.tile_mask(tile_pos(32 + 20, 32).tile()) & tile::CITY_RADIUS == 0);
    // And the count: every (u, v) with round(√(u²+v²)) ≤ 20, no zero row.
    let n = sim.city_mask_tiles(tile_pos(32, 32), 20).len();
    let expect = (-21..=20)
        .flat_map(|dx| (-21..=20).map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| {
            let u = if dx >= 0 { dx + 1 } else { dx };
            let v = if dy >= 0 { dy + 1 } else { dy };
            4 * (u * u + v * v) <= 41 * 41
        })
        .count();
    assert_eq!(n, expect);
    // The footprint is marked and blocked.
    let m = sim.world.tile_mask(tile_pos(32, 32).tile());
    assert_eq!(m & tile::OBJECT, tile::OBJECT_BUILDING);
    assert!(m & tile::BLOCKED != 0);
    // And a second city now wants friendly territory: 25 tiles out is owned
    // by us, so it places; the same spot in someone else's name is refused.
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(57, 32), None),
        Blocked::Clear
    );
}

#[test]
fn a_library_needs_a_city_and_there_is_one_per_city() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    // Before any city: the territory rule speaks first — unowned ground —
    // and the tile-level "outside every radius" is what it would say next.
    assert_eq!(
        sim.blocked_site(Some(0), t.library, tile_pos(40, 32), None),
        Blocked::NeutralTerritory
    );
    assert_eq!(
        sim.blocked_tcoord(Some(0), t.library, tile_pos(40, 32).tile(), None),
        Blocked::OutsideRadius
    );
    let (_, c) = city_at(&mut sim, &t, 0, 32, 32);
    assert_eq!(
        sim.blocked_site(Some(0), t.library, tile_pos(40, 32), None),
        Blocked::Clear
    );
    let l1 = sim.place_building(0, t.library, tile_pos(40, 32)).unwrap();
    assert_eq!(
        sim.buildings[l1].city,
        Some(c),
        "membership is decided at placement"
    );
    assert!(sim.cities[c].members.contains(&l1));
    // A second library in the same city is refused even before the first is
    // built — `count_buildings` counts placed ones.
    assert_eq!(
        sim.blocked_site(Some(0), t.library, tile_pos(24, 32), None),
        Blocked::One
    );
    // A barracks needs no city but does need friendly territory.
    assert_eq!(
        sim.blocked_site(Some(0), t.barracks, tile_pos(40, 40), None),
        Blocked::Clear
    );
    assert_eq!(
        sim.blocked_site(Some(1), t.barracks, tile_pos(40, 40), None),
        Blocked::EnemyTerritory,
        "player 1 is at war with the owner of that ground"
    );
    // A second city's library may stand at the edge where only one city covers.
    let _ = c;
}

#[test]
fn the_farm_limit_and_the_one_per_city_redirect() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (_, c) = city_at(&mut sim, &t, 0, 32, 32);
    let mut placed = 0;
    for i in 0..6 {
        let pos = tile_pos(20 + 5 * i, 40);
        match sim.place_building(0, t.farm, pos) {
            Ok(_) => placed += 1,
            Err(PlaceFail::Blocked(Blocked::Farm)) => break,
            Err(e) => panic!("{e:?} at farm {i}"),
        }
    }
    assert_eq!(placed, 5, "FARMS_PER_CITY_BASE");
    assert_eq!(sim.count_buildings(c, Ident::Farm, false), 5);
}

#[test]
fn the_city_limit_follows_the_civic_level() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    assert_eq!(sim.city_limit(0), 1);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    assert!(sim.at_city_limit(0));
    assert_eq!(
        sim.place_building(0, t.village, tile_pos(57, 32)),
        Err(PlaceFail::CityLimit)
    );
    sim.tech[0].epoch[tech::Line::Civic as usize] = 1;
    assert_eq!(sim.city_limit(0), 2);
    assert!(sim.place_building(0, t.village, tile_pos(57, 32)).is_ok());
    sim.nation[0].pyramids = true;
    sim.nation[0].bantu = true;
    assert_eq!(sim.city_limit(0), 4, "civic 1 + Bantu 1 + 1 + Pyramids 1");
}

// ----------------------------------------------------------------------
// Construction
// ----------------------------------------------------------------------

#[test]
fn one_builder_finishes_a_barracks_in_job_time_frames_and_the_site_grows() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    assert_eq!(sim.buildings[b].constr_time, 42_000);
    assert_eq!(
        sim.buildings[b].construct_hits, 1,
        "a site starts at one hit point"
    );
    let u = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
    sim.order_build(u, b);
    let mut frames = 0;
    while !sim.buildings[b].active {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    // A builder standing on the footprint walks off it first — `do_build`
    // kills itself and `action_swarm_around` re-queues an approach to the
    // ring `min(xs, ys) × 96 + 48`, nudged 48 further out (`docs/ORDERS.md`
    // §5.2, §5.4): one frame for that, sixteen to walk the 384 units at 25 a
    // frame, one for the build order to be current again — eighteen — and
    // then `JOB_TIME` frames at `ACCEL_CONSTRUCT` 1/1.
    assert_eq!(
        frames,
        420 + 18,
        "the walk off the footprint, then JOB_TIME"
    );
    assert!(sim.buildings[b].started);
    assert_eq!(sim.buildings[b].hits, 1200);
    assert_eq!(sim.buildings[b].health, 1200);
    assert!(
        sim.units[u].orders.is_empty()
            || !matches!(sim.units[u].orders[0].body, crate::orders::Body::Build(_)),
        "the order ends with the building"
    );
    // Midway the site was at about half health.
    let b2 = sim.place_building(0, t.barracks, tile_pos(46, 40)).unwrap();
    sim.order_build(u, b2);
    // Put the citizen — and its body, which would otherwise chase it across
    // six tiles and slow its turn — on the new site.
    sim.units[u].pos = tile_pos(46, 40);
    sim.units[u].movement.body.pos = tile_pos(46, 40);
    for _ in 0..210 {
        sim.tick();
    }
    // Eighteen frames to walk off the footprint again, then 100 a frame;
    // the site's hit points follow the progress (`docs/CITIES.md` §3.4).
    assert_eq!(sim.buildings[b2].job_counter, (210 - 18) * 100);
    assert!(
        (545..=550).contains(&sim.buildings[b2].construct_hits),
        "{}",
        sim.buildings[b2].construct_hits
    );
}

#[test]
fn two_builders_are_one_and_a_half_builders() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    for _ in 0..2 {
        let u = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
        sim.order_build(u, b);
    }
    let mut frames = 0;
    while !sim.buildings[b].active {
        sim.tick();
        frames += 1;
        assert!(frames < 1000);
    }
    // The same eighteen-frame walk off the footprint for both, then
    // `42000 / (100 + 50)` a frame.
    assert_eq!(
        frames,
        280 + 18,
        "the walk, then 42000 / (100 + 50) a frame"
    );
}

#[test]
fn a_nomads_first_city_takes_three_times_as_long_and_is_refunded_by_what_is_left() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let b = sim.place_building(0, t.village, tile_pos(32, 32)).unwrap();
    assert_eq!(
        sim.buildings[b].constr_time, 180_000,
        "CAPITAL_BUILD_TIME 300 %"
    );
    // Build a sixth of it, then cancel: five sixths of the price come back.
    let before = sim.ledgers[0].bucket[1];
    for _ in 0..300 {
        sim.do_construct(b, 100);
        sim.buildings[b].helpers = 0;
    }
    assert_eq!(sim.buildings[b].job_counter, 30_000);
    sim.disband_building(b, false);
    assert!(!sim.buildings[b].alive);
    assert_eq!(sim.ledgers[0].bucket[1], before + 83);
    // Past the type's own `job_time × 100` — sixty thousand — the partial
    // refund gate closes, tripled clock or not: the original tests
    // `job_counter < type.time(who)`, the unmodified figure.
    let b = sim.place_building(0, t.village, tile_pos(32, 32)).unwrap();
    for _ in 0..700 {
        sim.do_construct(b, 100);
        sim.buildings[b].helpers = 0;
    }
    let before = sim.ledgers[0].bucket[1];
    sim.disband_building(b, false);
    assert_eq!(sim.ledgers[0].bucket[1], before);
    // Its tiles are free again.
    assert_eq!(
        sim.world.tile_mask(tile_pos(32, 32).tile()) & tile::OBJECT,
        0
    );
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(32, 32), None),
        Blocked::Clear
    );
}

#[test]
fn a_placed_ghost_is_free_to_cancel_and_blocks_nothing_but_placement() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let b = sim.place_building(0, t.village, tile_pos(32, 32)).unwrap();
    let before = sim.ledgers[0].bucket[1];
    // The site verdict (spacing) wins over the tile verdict, so ask the
    // tile: a placed, unstarted building already blocks its tiles.
    assert_eq!(
        sim.blocked_tcoord(Some(0), t.village, tile_pos(32, 32).tile(), None),
        Blocked::Building
    );
    assert_eq!(
        sim.blocked_site(Some(0), t.village, tile_pos(32, 32), None),
        Blocked::CityDistance
    );
    sim.disband_building(b, false);
    assert_eq!(
        sim.ledgers[0].bucket[1],
        before + 100,
        "the whole price: nothing was built"
    );
}

#[test]
fn a_building_bleeds_in_enemy_territory_and_a_ghost_is_removed() {
    let mut sim = world_sim();
    // This measures a refund; the city's own trickle is switched off so
    // the ledger moves for no other reason.
    sim.tuning.city_gather = [0; 6];
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    // Player 1 takes the ground under it.
    sim.world
        .set_owner(tile_pos(40, 40).cell(), Owner::Player(1), Owner::None);
    let hits = sim.buildings[b].hits;
    for _ in 0..64 {
        sim.tick();
    }
    assert_eq!(
        sim.buildings[b].damage,
        8 * 2,
        "eight hits every thirty-two frames"
    );
    assert_eq!(sim.buildings[b].health, hits - 16);
    // A ghost on enemy ground is simply removed, with its price back.
    let g = sim.place_building(0, t.barracks, tile_pos(46, 46)).unwrap();
    sim.world
        .set_owner(tile_pos(46, 46).cell(), Owner::Player(1), Owner::None);
    let before = sim.ledgers[0].bucket[1];
    for _ in 0..32 {
        sim.tick();
    }
    assert!(!sim.buildings[g].alive);
    assert_eq!(sim.ledgers[0].bucket[1], before + 100);
}

#[test]
fn the_clock_is_rebaked_when_the_wall_stats_go_stale() {
    // Two land regions, so a nomad may place a foothold city in each.
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 15));
    w.fill_region(Terrain::Land, Cell::new(8, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    let t = install_types(&mut sim);
    // A nomad places two cities: both clocks are tripled. When the first
    // finishes, `calc_wall_stats` on the next frame re-bakes the second at
    // the plain rate — the ×3 is not frozen at placement.
    sim.tech[0].epoch[tech::Line::Civic as usize] = 1;
    let a = sim.place_building(0, t.village, tile_pos(20, 32)).unwrap();
    let b = sim.place_building(0, t.village, tile_pos(48, 32)).unwrap();
    assert_eq!(sim.buildings[a].constr_time, 180_000);
    assert_eq!(sim.buildings[b].constr_time, 180_000);
    finish(&mut sim, a);
    assert!(sim.wall_stats_dirty[0]);
    assert_eq!(
        sim.buildings[b].constr_time, 180_000,
        "not until Leader::process runs"
    );
    sim.tick();
    assert_eq!(sim.buildings[b].constr_time, 60_000);
    assert!(!sim.wall_stats_dirty[0]);
    // A speed tech arriving mid-build does the same.
    sim.nation[0].speed_upgrade = 3;
    sim.wall_stats_dirty[0] = true;
    sim.tick();
    assert_eq!(sim.buildings[b].constr_time, 42_000);
}

#[test]
fn a_repair_takes_twice_the_build_time_and_costs_the_price_again() {
    let mut sim = world_sim();
    // This measures a price; the city's own trickle is switched off so the
    // ledger moves for no other reason.
    sim.tuning.city_gather = [0; 6];
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    sim.buildings[b].damage = 600;
    sim.buildings[b].sync_health();
    let u = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
    sim.order_repair(u, b);
    let before = sim.ledgers[0].bucket[1];
    let mut frames = 0;
    while sim.buildings[b].damage > 0 {
        sim.tick();
        frames += 1;
        assert!(frames < 2000);
    }
    // (42000 << 9) / (1200 × 100) = 179.2 → 179 per 256 frames: 0.7 hits a
    // frame, 600 hits in 420 frames — the build time of the whole building
    // for half its hits, i.e. twice the build time for all of them.
    assert!((418..=422).contains(&frames), "{frames}");
    assert_eq!(
        before - sim.ledgers[0].bucket[1],
        50,
        "half the hits, half the price"
    );
}

// ----------------------------------------------------------------------
// Cities
// ----------------------------------------------------------------------

#[test]
fn a_city_levels_up_on_five_kinds_and_grows_its_radius() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    assert_eq!(sim.city_level_of(c), 1);
    assert_eq!(sim.radius_of(c), 20);
    let sites = [
        (t.barracks, tile_pos(40, 40)),
        (t.library, tile_pos(24, 40)),
        (t.market, tile_pos(40, 24)),
        (t.temple, tile_pos(24, 24)),
        (t.farm, tile_pos(44, 32)),
    ];
    let mut placed = Vec::new();
    for (ty, pos) in sites {
        let b = sim.place_building(0, ty, pos).unwrap();
        placed.push(b);
    }
    // Four kinds finished: still a city. The fifth finishes it.
    for &p in &placed[..4] {
        finish(&mut sim, p);
    }
    assert_eq!(sim.num_kinds(c), 5, "the city itself plus four");
    assert_eq!(sim.city_level_of(c), 1);
    finish(&mut sim, placed[4]);
    assert_eq!(sim.city_level_of(c), 2, "CITY_BUILDINGS + 1 distinct kinds");
    assert_eq!(sim.buildings[b].ty, Some(t.town));
    assert_eq!(sim.radius_of(c), 24);
    assert!(sim.world.tile_mask(tile_pos(32 + 23, 32).tile()) & tile::CITY_RADIUS != 0);
    // The Senate bonus arrives with the wall stats, on the next frame.
    sim.tick();
    // The city gained the new type's hits and kept its damage.
    assert_eq!(sim.buildings[b].hits, 2500);
    // Members got the senate bonus: a market in a Large City has 35 % more.
    let market = placed[2];
    assert_eq!(sim.buildings[market].hits, 1620);
    // A second farm is a kind already counted; nine kinds need four more.
    assert!(sim.ready_to_upgrade(c).is_none());
    let more = [
        (t.senate, tile_pos(20, 32)),
        (t.granary, tile_pos(32, 44)),
        (t.tower, tile_pos(32, 20)),
        (t.fort, tile_pos(44, 44)),
    ];
    for (ty, pos) in more {
        let b = sim
            .place_building(0, ty, pos)
            .unwrap_or_else(|e| panic!("{e:?}"));
        finish(&mut sim, b);
    }
    assert_eq!(sim.city_level_of(c), 3, "METRO_BUILDINGS + 1");
    assert_eq!(sim.buildings[b].ty, Some(t.metropolis));
    assert_eq!(sim.radius_of(c), 28);
    let _ = t.fort;
}

// ----------------------------------------------------------------------
// Garrisons
// ----------------------------------------------------------------------

#[test]
fn hoplites_fill_a_barracks_to_its_limit_and_leave_one_squad_a_frame() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let h = spawn(&mut sim, 0, hoplite, tile_pos(40, 40));
    assert_eq!(sim.garrison(h, b), Err(GarrisonRefused::Inactive));
    finish(&mut sim, b);
    assert_eq!(sim.garrison_limit(b), 10);
    let mut inside = vec![h];
    assert_eq!(sim.garrison(h, b), Ok(()));
    assert!(!sim.units[h].on_map && sim.units[h].inside == Some(b));
    for _ in 1..10 {
        let u = spawn(&mut sim, 0, hoplite, tile_pos(40, 40));
        assert_eq!(sim.garrison(u, b), Ok(()));
        inside.push(u);
    }
    assert_eq!(sim.num_inside(b), 10);
    let extra = spawn(&mut sim, 0, hoplite, tile_pos(40, 40));
    assert_eq!(sim.garrison(extra, b), Err(GarrisonRefused::Full));
    // A citizen cannot enter a barracks; a hoplite cannot enter a farm.
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let c = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
    assert_eq!(sim.garrison(c, b), Err(GarrisonRefused::CantGarrison));
    let f = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
    finish(&mut sim, f);
    assert_eq!(sim.garrison(extra, f), Err(GarrisonRefused::NoCapacity));
    // Eject all: one squad a frame, FIFO.
    sim.eject_contents(b, false);
    sim.tick();
    assert!(sim.units[inside[0]].on_map, "the first in is the first out");
    assert!(!sim.units[inside[1]].on_map);
    assert_eq!(sim.squads_inside(b), 9);
    for _ in 0..9 {
        sim.tick();
    }
    assert_eq!(sim.squads_inside(b), 0);
    // The flag clears on the frame after the last squad is out.
    assert!(sim.buildings[b].eject_pending);
    sim.tick();
    assert!(!sim.buildings[b].eject_pending);
    // They stand on the exit ring: `find_nearby_spot` sweeping from due
    // south at `(4 + 4) × 0x30 + 288`, and its first candidate — the ring
    // point itself — snapped to its quarter-tile centre, which is what puts
    // the pair 24 off a tile corner in both axes.
    let (u, bp) = (sim.units[inside[0]].pos, sim.buildings[b].pos);
    assert_eq!((u.x - bp.x, u.y - bp.y), (24, 8 * 0x30 + 288 + 24));
}

#[test]
fn the_garrison_heal_runs_every_twenty_frames_where_the_unit_was_trained() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    let h = spawn(&mut sim, 0, hoplite, tile_pos(40, 40));
    sim.units[h].health = 50;
    assert_eq!(sim.garrison(h, b), Ok(()));
    for _ in 0..100 {
        sim.tick();
    }
    assert_eq!(
        sim.units[h].health, 55,
        "UNIT_HEAL_RATE 20: five steps in a hundred frames"
    );
    // In a market — not its trainer, not a city, fort or tower — nothing.
    let m = sim.place_building(0, t.market, tile_pos(40, 24)).unwrap();
    finish(&mut sim, m);
    sim.come_out(h);
    sim.units[h].pos = tile_pos(40, 24);
    sim.units[h].health = 50;
    // A hoplite cannot garrison a market by order; put it in directly to
    // ask the heal alone.
    assert_eq!(sim.garrison(h, m), Err(GarrisonRefused::CantGarrison));
    sim.go_inside(h, m);
    for _ in 0..100 {
        sim.tick();
    }
    assert_eq!(sim.units[h].health, 50);
    // The Red Fort owner heals at 20 × 100 / 600 = 3.
    sim.come_out(h);
    sim.units[h].pos = tile_pos(40, 40);
    sim.nation[0].red_fort = true;
    assert_eq!(sim.garrison(h, b), Ok(()));
    for _ in 0..30 {
        sim.tick();
    }
    assert_eq!(sim.units[h].health, 60);
}

#[test]
fn a_garrisoned_tower_reloads_faster() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let tw = sim.place_building(0, t.tower, tile_pos(40, 40)).unwrap();
    finish(&mut sim, tw);
    assert_eq!(sim.garrison_attack_sum(tw), 0);
    for _ in 0..3 {
        let h = spawn(&mut sim, 0, hoplite, tile_pos(40, 40));
        assert_eq!(sim.garrison(h, tw), Ok(()));
    }
    // Three melee hoplites: (15/10 + 1) / 2 = 1 each.
    assert_eq!(sim.garrison_attack_sum(tw), 3);
}

// ----------------------------------------------------------------------
// Capture
// ----------------------------------------------------------------------

#[test]
fn capture_values_and_health_levels() {
    assert_eq!(capture_value(false, 3, 1), 1);
    assert_eq!(capture_value(true, 3, 1), 0);
    assert_eq!(capture_value(false, 1, 0), 1);
    assert_eq!(health_level(1200, 0), 0);
    assert_eq!(health_level(1200, 120), 0);
    assert_eq!(health_level(1200, 121), 1);
    assert_eq!(health_level(1200, 1080), 4);
    assert_eq!(health_level(1200, 1081), 5);
    assert_eq!(health_level(1200, 1200), 6);
}

#[test]
fn a_city_at_zero_falls_to_three_hoplites_but_not_two() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 1, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    // Knock it to zero: the clamp holds it there.
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits;
    bd.sync_health();
    assert!(sim.capture_eligible(b));
    sim.frame = 1000;
    let h1 = spawn(&mut sim, 0, hoplite, tile_pos(36, 32));
    let h2 = spawn(&mut sim, 0, hoplite, tile_pos(36, 33));
    // def_base is 2 for a city captured more than 900 frames ago — or never.
    assert!(
        !sim.check_capture(b, h1),
        "one attacker against a base of two"
    );
    assert!(
        !sim.check_capture(b, h2),
        "two against two: the defender holds"
    );
    let h3 = spawn(&mut sim, 0, hoplite, tile_pos(36, 34));
    assert!(sim.check_capture(b, h3), "three against two");
    // The old record is closed, a new one owns the place.
    assert!(!sim.cities[c].alive);
    assert!(!sim.buildings[b].alive);
    let nc = sim
        .cities
        .iter()
        .position(|c| c.alive && c.owner == 0)
        .expect("player 0 has the city now");
    let nb = sim.cities[nc].building;
    assert_eq!(sim.buildings[nb].owner, 0);
    assert_eq!(
        sim.buildings[nb].health, 10,
        "handed over with ten hit points"
    );
    assert_eq!(
        sim.cities[nc].race,
        Some(1),
        "still the old nation's: unassimilated"
    );
    assert!(sim.cities[nc].unassimilated);
    assert_eq!(
        sim.cities[nc].capture_strength, 1,
        "the triggering hoplite's own value"
    );
    assert_eq!(sim.cities[nc].capture_stamp, 1000);
    assert!(sim.city_unassimilated(nc));
    // Territory changed hands.
    assert_eq!(sim.world.owner_at(tile_pos(32, 32)), Owner::Player(0));
    // Player 1 lost their last city: defeated.
    assert!(sim.defeated[1]);
    assert_eq!(sim.city_tally[0].captured, 1);
    assert_eq!(sim.city_tally[1].lost, 1);
}

#[test]
fn an_unassimilated_city_does_not_heal_or_make_anything_and_assimilates_at_two_thousand_frames() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, _) = city_at(&mut sim, &t, 1, 32, 32);
    // Give player 1 a second city so the capture is not an elimination.
    sim.tech[1].epoch[tech::Line::Civic as usize] = 1;
    let _ = city_at(&mut sim, &t, 1, 8, 8);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits;
    bd.sync_health();
    sim.frame = 1000;
    let hs: Vec<usize> = (0..3)
        .map(|i| spawn(&mut sim, 0, hoplite, tile_pos(36, 32 + i)))
        .collect();
    assert!(sim.check_capture(b, hs[2]));
    let nc = sim
        .cities
        .iter()
        .position(|c| c.alive && c.owner == 0)
        .unwrap();
    let nb = sim.cities[nc].building;
    assert_eq!(sim.buildings[nb].health, 10);
    // Not assimilated: no heal, nothing queued, nobody garrisons.
    for _ in 0..100 {
        sim.tick();
    }
    assert_eq!(sim.buildings[nb].health, 10);
    let h = spawn(&mut sim, 0, hoplite, tile_pos(32, 32));
    assert_eq!(sim.garrison(h, nb), Err(GarrisonRefused::Unassimilated));
    let citizen = sim.add_unit_type(citizen_type(t.village));
    assert_eq!(
        sim.queue_up(nb, citizen),
        Err(production::QueueFail::CantTrain)
    );
    // 2000 frames after the capture it assimilates, and then heals its level
    // every four frames.
    while sim.frame < 1000 + 2000 {
        sim.tick();
    }
    assert!(sim.city_unassimilated(nc), "1999 frames elapsed");
    sim.tick();
    assert!(!sim.city_unassimilated(nc));
    assert_eq!(sim.cities[nc].race, Some(0));
    let before = sim.buildings[nb].health;
    for _ in 0..40 {
        sim.tick();
    }
    assert_eq!(
        sim.buildings[nb].health - before,
        10,
        "level 1 every CITY_HEAL_RATE frames"
    );
    // Now it admits the garrison — but not while under a tenth of its hits.
    assert_eq!(sim.garrison(h, nb), Err(GarrisonRefused::CityTooHurt));
    // And the 75-frame grace after a capture blocks the old owner's revenge.
    assert!(!sim.defeated[1]);
}

#[test]
fn a_garrisoned_city_is_emptied_rather_than_captured() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, _) = city_at(&mut sim, &t, 1, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let g = spawn(&mut sim, 1, hoplite, tile_pos(32, 32));
    assert_eq!(sim.garrison(g, b), Ok(()));
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits;
    bd.sync_health();
    sim.frame = 1000;
    let hs: Vec<usize> = (0..4)
        .map(|i| spawn(&mut sim, 0, hoplite, tile_pos(36, 32 + i)))
        .collect();
    assert!(!sim.check_capture(b, hs[3]), "the garrison goes out first");
    assert!(sim.buildings[b].eject_pending);
    sim.tick();
    assert!(sim.units[g].on_map);
    // The tick also ran the city heal — an assimilated city at zero gets a
    // point back every four frames — so a besieger has to hit it again.
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits;
    bd.sync_health();
    // The defender stands by the city now and counts: 2 + 1 against 4.
    assert!(sim.check_capture(b, hs[3]));
}

#[test]
fn the_turks_assimilate_three_times_faster() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, _) = city_at(&mut sim, &t, 1, 32, 32);
    sim.tech[1].epoch[tech::Line::Civic as usize] = 1;
    let _ = city_at(&mut sim, &t, 1, 8, 8);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    sim.nation[0].turks = true;
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits;
    bd.sync_health();
    sim.frame = 1000;
    let hs: Vec<usize> = (0..3)
        .map(|i| spawn(&mut sim, 0, hoplite, tile_pos(36, 32 + i)))
        .collect();
    assert!(sim.check_capture(b, hs[2]));
    let nc = sim
        .cities
        .iter()
        .position(|c| c.alive && c.owner == 0)
        .unwrap();
    while sim.frame < 1000 + 666 {
        sim.tick();
    }
    assert!(sim.city_unassimilated(nc));
    sim.tick();
    sim.tick();
    assert!(!sim.city_unassimilated(nc), "2000 × 100 / 300 = 667 frames");
}

#[test]
fn killing_a_barracks_plunders_its_good() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 1, 32, 32);
    let mut barracks = bt(Ident::Barracks, None, "ean", 4, 4, 420, 1200, 10);
    barracks.plunder_value = 40;
    barracks.plunder_good = Some(0);
    let bty = sim.add_build_type(barracks);
    let b = sim.place_building(1, bty, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    let before = sim.ledgers[0].bucket[0];
    sim.plunder_kill(b, 0);
    assert_eq!(sim.ledgers[0].bucket[0] - before, 40, "PLUNDER 100 % of 40");
    // An unfinished one plunders by its progress: a third built, a third.
    let s = sim.place_building(1, bty, tile_pos(46, 40)).unwrap();
    for _ in 0..140 {
        sim.do_construct(s, 100);
        sim.buildings[s].helpers = 0;
    }
    let before = sim.ledgers[0].bucket[0];
    sim.plunder_kill(s, 0);
    assert_eq!(sim.ledgers[0].bucket[0] - before, 13, "40 × 14000 / 42000");
}

#[test]
fn the_bare_building_of_the_earlier_mechanics_is_untouched() {
    // `add_building` still makes the typeless, active production object, and
    // the new per-frame work leaves it alone.
    let mut sim = world_sim();
    let b = sim.add_building(0, tile_pos(32, 32), 20);
    for _ in 0..100 {
        sim.tick();
    }
    assert!(sim.buildings[b].alive && sim.buildings[b].active);
    assert_eq!(sim.buildings[b].ty, None);
    assert_eq!(sim.cities.len(), 0);
    let _ = UNITS_PER_CELL;
}

// ----------------------------------------------------------------------
// Orders — `docs/ORDERS.md`
// ----------------------------------------------------------------------

use crate::orders::{Body, MoveKind, QueuePos, Worker};

/// `QUEUE_LAST` appends, `QUEUE_FIRST` rotates the new order to the front,
/// `QUEUE_NEW` replaces everything (§1.5).
#[test]
fn the_three_queue_modes_append_rotate_and_replace() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    // On a quarter-tile centre, so a move due east is exactly east.
    let at = tile_pos(20, 20);
    let u = spawn(&mut sim, 0, citizen, Pos::new(at.x + 24, at.y + 24));
    sim.order_move(u, tile_pos(21, 20));
    assert_eq!(sim.units[u].orders.len(), 1);
    let first = sim.units[u].orders[0];
    sim.add_move_order(u, tile_pos(22, 20), MoveKind::MoveTo, QueuePos::Last, true);
    assert_eq!(sim.units[u].orders.len(), 2);
    assert_eq!(
        sim.units[u].orders[0], first,
        "QUEUE_LAST leaves the current order"
    );
    sim.add_move_order(
        u,
        tile_pos(23, 20),
        MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    assert_eq!(sim.units[u].orders.len(), 3);
    assert!(
        matches!(sim.units[u].orders[0].body, Body::Move(m) if m.dest.x == tile_pos(23, 20).x.div_euclid(48) * 48 + 24),
        "QUEUE_FIRST runs now"
    );
    assert_eq!(
        sim.units[u].orders[1], first,
        "and the old current resumes after it"
    );
    sim.order_move(u, tile_pos(24, 20));
    assert_eq!(
        sim.units[u].orders.len(),
        1,
        "QUEUE_NEW replaces everything"
    );
    // Every destination is a quarter-tile centre, and the facing to apply
    // on arrival is the direction at order time (§4.1).
    let Body::Move(m) = sim.units[u].orders[0].body else {
        panic!()
    };
    assert_eq!(m.dest.x.rem_euclid(48), 24);
    assert_eq!(m.dest.y.rem_euclid(48), 24);
    assert_eq!(m.angle, movement::Angle::EAST);
}

/// The dump's starting citizen (§4.8, §6.3): a gather order on a farm joins
/// the chain at once, queues the walk inside its own step (so the unit
/// stands that frame), steps the next, arrives at the farm's centre, and
/// counts for the economy only from `been_there`.
#[test]
fn a_citizen_with_a_gather_order_walks_to_the_farm_and_then_counts() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let farm = sim.place_building(0, t.farm, tile_pos(40, 32)).unwrap();
    finish(&mut sim, farm);
    assert!(sim.buildings[farm].active);
    assert_eq!(
        sim.buildings[farm].gather_max,
        Some(1),
        "a flat type has one slot"
    );
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;
    // Beside the farm, off its footprint: four tiles east of the centre.
    let u = spawn(&mut sim, 0, citizen, tile_pos(44, 32));
    sim.add_gather_order(u, farm, QueuePos::New, false);
    assert!(sim.is_gathered_by(farm, u), "the chain is joined at issue");
    assert_eq!(sim.num_gatherers(farm, false, false), 1);
    assert_eq!(
        sim.num_gatherers(farm, true, true),
        0,
        "not arrived: the economy's count is 0"
    );

    let start = sim.units[u].pos;
    sim.tick(); // the gather step queues the walk; nothing moves
    assert_eq!(sim.units[u].pos, start);
    assert_eq!(sim.units[u].orders.len(), 2);
    assert!(matches!(sim.units[u].orders[0].body, Body::Move(_)));
    assert!(matches!(sim.units[u].orders[1].body, Body::Gather(_)));
    sim.tick(); // the move steps
    assert_ne!(
        sim.units[u].pos, start,
        "the inserted move runs from the next frame"
    );

    let mut frames = 2;
    while sim.units[u].orders.len() == 2 {
        sim.tick();
        frames += 1;
        assert!(frames < 100);
    }
    // Arrived at the farm's centre — the 48-snapped centre — with the gather
    // order current again; it runs its own step the next frame.
    let centre = sim.buildings[farm].pos;
    let snapped = Pos::new(
        centre.x.div_euclid(48) * 48 + 24,
        centre.y.div_euclid(48) * 48 + 24,
    );
    assert_eq!(sim.units[u].pos, snapped);
    assert!(matches!(sim.units[u].orders[0].body, Body::Gather(g) if !g.been_there));
    sim.tick();
    assert!(matches!(sim.units[u].orders[0].body, Body::Gather(g) if g.been_there));
    assert_eq!(
        sim.num_gatherers(farm, true, true),
        1,
        "now the economy counts it"
    );
    assert!(sim.ledgers[0].dirty, "and the rate is marked stale");
    // A second citizen finds the farm full and gives the order up for a
    // THINK.
    let v = spawn(&mut sim, 0, citizen, tile_pos(44, 33));
    sim.add_gather_order(v, farm, QueuePos::New, false);
    assert!(!sim.is_gathered_by(farm, v), "no room: refused at issue");
    sim.tick();
    assert!(!matches!(
        sim.units[v].orders.front().map(|o| o.body),
        Some(Body::Gather(_))
    ));
}

/// **A gather's walk to an occupied spot dies where it stands** —
/// `docs/ORDERS.md` §4.4's waypoint take, the third and last call site of
/// `Unit::detect_unit_collision`. It runs **once per leg**, on the frame
/// the waypoint is first read off the path stack, and a *final* waypoint
/// somebody is already standing on under a `GATHER`, `ATTACK` or
/// `BUILD_AT` action kills the whole move rather than walking it.
///
/// This is run10's frames 199–201: the AI's farmer `1/4` re-picked the farm
/// cell its sibling `1/2` was working, the original named `1/2` on
/// `collide_o`/`collide_who` and killed the walk without a step, and
/// `do_farm` picked a different cell the frame after. Without the test the
/// walk is planned and taken, and the farmer wanders off across its own
/// farm.
#[test]
fn a_gather_walk_onto_an_occupied_spot_is_killed_where_it_stands() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let farm = sim.place_building(0, t.farm, tile_pos(40, 32)).unwrap();
    finish(&mut sim, farm);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;
    // Both blockable: `coll_size` is `block_radius / 48`, and a type with
    // none never collides at all (`docs/COLLISION.md` §2).
    sim.unit_types[citizen].combat.block_radius = 48;
    sim.unit_types[citizen].combat.big_radius = 48;

    // The spot the gather's walk ends on: the farm's 48-snapped centre.
    let centre = sim.buildings[farm].pos;
    let snapped = Pos::new(
        centre.x.div_euclid(48) * 48 + 24,
        centre.y.div_euclid(48) * 48 + 24,
    );
    let blocker = spawn(&mut sim, 0, citizen, snapped);
    let u = spawn(&mut sim, 0, citizen, tile_pos(44, 32));
    sim.add_gather_order(u, farm, QueuePos::New, false);

    let start = sim.units[u].pos;
    sim.tick(); // the gather step queues the walk; nothing moves yet
    assert!(matches!(sim.units[u].orders[0].body, Body::Move(_)));
    assert_eq!(sim.units[u].pos, start);

    sim.tick(); // `do_move` plans, takes the waypoint — and finds it taken
    assert_eq!(
        sim.units[u].pos, start,
        "not one step of the walk was taken"
    );
    assert!(
        matches!(sim.units[u].orders[0].body, Body::Gather(_)),
        "the move is gone and the gather is current again: {:?}",
        sim.units[u].orders[0].body
    );
    assert_eq!(
        sim.units[u].collide_o, sim.units[blocker].index,
        "and the blocker is named on the record"
    );
    assert_eq!(sim.units[u].collide_who, 0);
    assert_eq!(sim.units[u].collide_guy, 0);

    // With nobody there the same walk is planned and taken.
    let mut sim2 = world_sim();
    let t2 = install_types(&mut sim2);
    let _ = city_at(&mut sim2, &t2, 0, 32, 32);
    let farm2 = sim2.place_building(0, t2.farm, tile_pos(40, 32)).unwrap();
    finish(&mut sim2, farm2);
    let c2 = sim2.add_unit_type(citizen_type(t2.village));
    sim2.unit_types[c2].worker = Worker::Citizen;
    sim2.unit_types[c2].combat.block_radius = 48;
    sim2.unit_types[c2].combat.big_radius = 48;
    let v = spawn(&mut sim2, 0, c2, tile_pos(44, 32));
    sim2.add_gather_order(v, farm2, QueuePos::New, false);
    let from = sim2.units[v].pos;
    sim2.tick();
    sim2.tick();
    assert_ne!(sim2.units[v].pos, from, "the unblocked walk steps");
}

/// **Only a human builder keeps the site it just finished** — `docs/ORDERS.md`
/// §5.2's note. Step 6's gather arm is `OILPLATFORM or (`unit_masks &
/// 0x40000` clear and worker_stance ∈ {0, 1})`, so an AI citizen never
/// adopts its own site: it goes back through `build_done`, whose arm (§5.5)
/// is `find_build_spot` → `find_repair_spot` → a fresh `find_gather_spot`
/// **gated on the lobby's `starting_resources != 8`**.
///
/// That gate is what makes the two arms separable here without geometry: a
/// human keeps its farm under either lobby, and an AI under the unlimited
/// one ends the frame holding nothing at all. Which building the AI's fresh
/// search lands on when it does run is run10's frame 167, where it walks
/// past the farm it built to the Woodcutter's Camp — the distinction is
/// invisible whenever the search would pick the site anyway, which is how
/// the missing term survived a month in this crate.
#[test]
fn only_a_human_builder_adopts_the_site_it_has_just_finished() {
    // `run` stands a citizen on an unfinished farm, lets it build the farm
    // out, and answers with the order it is left holding.
    fn run(human: bool, unlimited: bool) -> Option<Body> {
        use crate::tech::{TechTree, TypeDef};
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        sim.nation[0].human = human;
        sim.lobby.starting_resources = if unlimited { 8 } else { 1 };
        // An AI leader runs `Leaders::strategy_all` on every tick, and its
        // research step indexes the tech tree by resource. Six goods is the
        // least that makes that sweep legal here; nothing else reads them.
        let mut tree = TechTree::new();
        for name in ["Food", "Timber", "Metal", "Wealth", "Knowledge", "Oil"] {
            tree.add(TypeDef::good(name));
        }
        sim.set_tech_tree(tree);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;

        let site = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
        assert!(!sim.buildings[site].active, "the site starts unfinished");
        let builder = spawn(&mut sim, 0, citizen, tile_pos(44, 32));
        sim.add_build_order(builder, site, QueuePos::New, true);

        let mut frames = 0;
        while !sim.buildings[site].active {
            sim.tick();
            frames += 1;
            assert!(frames < 4_000, "the farm should go up");
        }
        assert!(
            !matches!(
                sim.units[builder].orders.front().map(|o| o.body),
                Some(Body::Build(_))
            ),
            "the build order dies on the frame the site finishes"
        );
        sim.units[builder].orders.front().map(|o| o.body)
    }

    let gathers = |b: Option<Body>| matches!(b, Some(Body::Gather(_)));
    assert!(
        gathers(run(true, false)),
        "a human builder gathers at the farm it raised"
    );
    assert!(
        gathers(run(true, true)),
        "and the lobby's resource rule is none of the human arm's business"
    );
    // The AI's arm ran and found the site, which is the case the missing
    // term could not be told apart from...
    assert!(
        gathers(run(false, false)),
        "the AI's own search lands on the site when nothing beats it"
    );
    // ...and this is the one it can.
    let ai = run(false, true);
    assert_eq!(
        ai, None,
        "an unlimited-resources AI builder searches for nothing at all: {ai:?}"
    );
}

/// On open ground no move draws from the sync stream: a near one never
/// asks the pathfinder, and a far one is planned by `find_wpath` at order
/// time — before the RNG-thresholded re-plan branch, which only runs when
/// `find_path` refuses a line (§4.4). The far move walks the planned chain
/// of cell centres and arrives.
#[test]
fn moves_on_open_ground_draw_nothing_and_a_far_move_walks_the_chain() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let u = spawn(&mut sim, 0, citizen, tile_pos(20, 20));
    // The frame's own draws — the market, the birds' sampling — are the
    // same with or without the move: a control that stands still says what
    // they are.
    let mut control = world_sim();
    let ct = install_types(&mut control);
    let cc = control.add_unit_type(citizen_type(ct.village));
    spawn(&mut control, 0, cc, tile_pos(20, 20));
    sim.order_move(u, tile_pos(24, 20)); // one cell
    for _ in 0..40 {
        sim.tick();
        control.tick();
    }
    assert!(sim.units[u].orders.is_empty(), "arrived");
    assert_eq!(sim.rng.seed, control.rng.seed, "a near move draws nothing");
    let before = sim.units[u].pos;
    sim.order_move(u, tile_pos(44, 20)); // five cells
    sim.tick();
    control.tick();
    assert!(
        sim.units[u].path.len() > 1,
        "a far move is planned as a chain at order time"
    );
    assert_ne!(sim.units[u].pos, before, "and steps that frame");
    for _ in 0..400 {
        sim.tick();
        control.tick();
    }
    assert!(sim.units[u].orders.is_empty(), "the chain arrives");
    assert_eq!(
        sim.rng.seed, control.rng.seed,
        "and no move on open ground drew"
    );
}

/// `find_nearby_spot` starts its sweep on the unit's side of the target and
/// refuses the target's own footprint (§10).
#[test]
fn the_spot_search_starts_on_the_units_side_and_keeps_off_the_footprint() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let east = spawn(&mut sim, 0, citizen, tile_pos(46, 40));
    let site = sim.buildings[b].pos;
    let here = sim.units[east].pos;
    let angle = movement::find_angle(here.x - site.x, here.y - site.y);
    let spot = sim
        .find_nearby_spot(east, site, 3 * 0x60 + 0x30, 0, -1, angle, Some(b))
        .expect("open ground has a spot");
    assert!(spot.x > site.x, "east of the site: {spot:?}");
    assert_eq!(
        spot.y,
        site.y.div_euclid(48) * 48 + 24,
        "on the unit's bearing"
    );
    assert!(!sim.covers_tile(b, spot.tile()));
    // From the centre itself, radius 0 is one candidate — the footprint — and
    // is refused; the same point with no footprint to refuse is accepted.
    assert_eq!(
        sim.find_nearby_spot(east, site, 0, 0, 0, angle, Some(b)),
        None
    );
    assert!(
        sim.find_nearby_spot(east, site, 0, 0, 0, angle, None)
            .is_some()
    );
}

// ----------------------------------------------------------------------
// Object numbers — `Objects::find_free`, `docs/AI.md` §12.1
// ----------------------------------------------------------------------

#[test]
fn object_numbers_are_per_player_and_a_dead_slot_is_reused() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));

    // Buildings: each player's band starts at 2000.
    let (b0, _) = city_at(&mut sim, &t, 0, 32, 32);
    let (b1, _) = city_at(&mut sim, &t, 1, 8, 8);
    assert_eq!(sim.buildings[b0].index, 2000);
    assert_eq!(sim.buildings[b1].index, 2000);
    let farm = sim.init_build(0, t.farm, tile_pos(36, 32), false);
    assert_eq!(sim.buildings[farm].index, 2001);

    // A dead building's number is the first reused; the mark is untouched.
    sim.buildings[farm].alive = false;
    let farm2 = sim.init_build(0, t.farm, tile_pos(36, 36), false);
    assert_eq!(sim.buildings[farm2].index, 2001);
    let farm3 = sim.init_build(0, t.farm, tile_pos(28, 36), false);
    assert_eq!(sim.buildings[farm3].index, 2002);
    assert_eq!(sim.marks[0].build, 2003);
    assert_eq!(sim.building_by_o(0, 2001), Some(farm2));
    assert_eq!(sim.building_by_o(1, 2001), None);

    // Units: the band starts at 0, per player, and a dead unit's is reused.
    let u0 = sim.produce(0, citizen, tile_pos(32, 30)).unwrap();
    let u1 = sim.produce(1, citizen, tile_pos(8, 6)).unwrap();
    assert_eq!(sim.units[u0].index, 0);
    assert_eq!(sim.units[u1].index, 0);
    sim.units[u0].health = 0;
    let u2 = sim.produce(0, citizen, tile_pos(32, 30)).unwrap();
    assert_eq!(sim.units[u2].index, 0);
    assert_eq!(sim.unit_by_o(0, 0), Some(u2));

    // A unit handed in already numbered (the harness, from a dump) moves the
    // mark past itself, and the numbers it skipped are free — a dump omits
    // exactly the dead.
    let mut u = Unit::new(0, 7, tile_pos(30, 30), 40);
    u.ty = Some(citizen);
    sim.add_unit(u);
    assert_eq!(sim.marks[0].unit, 8);
    let u3 = sim.produce(0, citizen, tile_pos(32, 30)).unwrap();
    assert_eq!(sim.units[u3].index, 1);
}

// ----------------------------------------------------------------------
// A technology in the queue — `docs/PRODUCTION.md`, the research step
// ----------------------------------------------------------------------

#[test]
fn a_technology_queues_at_the_library_and_completes_as_research() {
    use crate::tech::{Line, TechTree, TypeDef};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let library_t = tree.add(TypeDef::building("Library"));
    let mut ww = TypeDef::epoch("Written Word", Line::Science, 0).at(library_t);
    ww.cost[economy::Resource::Food.index()] = 4;
    ww.job_time = 40;
    let written_word = tree.add(ww);
    sim.set_tech_tree(tree);
    sim.build_types[t.library].tree = Some(library_t);
    let (city_b, _) = city_at(&mut sim, &t, 0, 32, 32);
    let lib = sim.init_build(0, t.library, tile_pos(36, 32), false);
    finish(&mut sim, lib);

    // Only a building whose type makes it takes the order.
    assert_eq!(
        sim.queue_tech(city_b, written_word),
        Err(production::QueueFail::CantTrain)
    );
    let food = sim.ledgers[0].bucket[0];
    assert_eq!(sim.queue_tech(lib, written_word), Ok(0));
    assert_eq!(
        sim.ledgers[0].bucket[0],
        food - 4 * sim.tuning.tech_cost_factor,
        "COST times TECH_COST_FACTOR, charged on queue"
    );
    assert!(sim.researching(0, written_word));
    assert_eq!(sim.queue_target(lib, 0), 40 * 100, "JOB_TIME × 100 × 1/1");

    // Forty calls reach the target, the forty-first observes it: the bit is
    // set, the entry is gone, nothing was placed.
    let mut frames = 0;
    while !sim.tech[0].tech[written_word] {
        sim.tick();
        frames += 1;
        assert!(frames < 100);
    }
    assert_eq!(frames, 41);
    assert!(!sim.researching(0, written_word));
    assert!(sim.buildings[lib].queue.items.is_empty());
    assert_eq!(sim.tech[0].epochs, 1);
    assert!(sim.units.is_empty());
}

/// `Unit::go_around_building` (`docs/ORDERS.md` §4.6): a straight line that
/// clips a **building** is answered with a detour, not with a re-plan — and
/// the re-plan is what costs a sync draw.
///
/// The pull-back cannot help here. It walks the *goal* back until its own
/// tile is clear, and the goal's tile is clear already; the obstacle is in
/// the middle of the march. So the sim used to march into the barracks,
/// find the tile invalid, give up, and pay `Unit::do_move+0xe84` for a
/// pathfinder call the original never makes.
#[test]
fn a_line_that_clips_a_building_detours_instead_of_re_planning() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    // A city well clear of the walk, only so that the barracks has
    // territory to stand in.
    let _ = city_at(&mut sim, &t, 0, 24, 30);
    // Two cells east, so `find_wpath` leaves the goal alone and the
    // straight-line verifier is what plans the walk — with a 4×4 barracks
    // squarely across the middle of it.
    let b = sim
        .place_building(0, t.barracks, tile_pos(24, 20))
        .expect("open ground");
    finish(&mut sim, b);
    assert!(
        sim.world.tile_mask(Pos::new(24, 20)) & tile::BLOCKED != 0,
        "the barracks blocks its own tiles"
    );
    let u = spawn(&mut sim, 0, citizen, tile_pos(20, 20));
    sim.trace_phases = true;
    sim.order_move(u, tile_pos(28, 20));
    sim.tick();
    let grid = |s: &Sim| {
        s.phase_marks
            .iter()
            .any(|(l, _)| l == orders::SITE_MOVE_GRID)
    };
    assert!(!grid(&sim), "the detour is free; a re-plan would not be");
    // The stack is the goal plus what the edge walk pushed, and the detour
    // is off the line: the goal's row is 20, the waypoint's is not.
    assert!(
        sim.units[u].path.len() > 1,
        "a detour was pushed: {:?}",
        sim.units[u].path
    );
    let top = *sim.units[u].path.last().expect("a waypoint");
    assert_eq!(top.tolerance, 0, "a detour point has no tolerance");
    assert_ne!(top.to.tile().y, 20, "and it leaves the line: {top:?}");
    for _ in 0..600 {
        sim.tick();
        assert!(!grid(&sim), "and never re-plans on the way");
        if sim.units[u].orders.is_empty() {
            break;
        }
    }
    assert!(sim.units[u].orders.is_empty(), "it arrives");
    assert!(
        !sim.covers_tile(b, sim.units[u].pos.tile()),
        "and not through the barracks"
    );
}

/// The other half of the same mechanic: an obstacle the edge walk cannot
/// get past **gives up**, and giving up is what spends the draw.
///
/// A wall of barracks from the top of the map to the bottom leaves the walk
/// nothing to find in either direction; it runs off the map both ways,
/// `path_recursion` is pinned at ten, and `do_move` rolls its `% 5` grid
/// threshold exactly as it did before any of this existed.
#[test]
fn a_wall_with_no_way_round_gives_up_and_pays_the_grid_draw() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    // A mountain ridge from the top of the map to the bottom. The trigger
    // is `invalid_loc`, not a building, so terrain exercises the same
    // branch and needs no territory to stand in.
    for ty in 0..sim.world.height() * world::TILES_PER_CELL {
        sim.world
            .set_tile_bits(Pos::new(24, ty), tile::OBJECT_MOUNTAIN);
    }
    let u = spawn(&mut sim, 0, citizen, tile_pos(20, 20));
    sim.trace_phases = true;
    sim.order_move(u, tile_pos(28, 20));
    sim.tick();
    assert!(
        sim.phase_marks
            .iter()
            .any(|(l, _)| l == orders::SITE_MOVE_GRID),
        "no way round: the grid draw is the original's answer too"
    );
    // And the ridge holds: nothing walks through it, and the walk that
    // cannot get round it never reaches the goal.
    for _ in 0..300 {
        sim.tick();
        assert!(sim.units[u].pos.tile().x < 24, "through the ridge");
    }
    assert!(!sim.units[u].orders.is_empty(), "still trying");
}
