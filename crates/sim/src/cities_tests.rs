//! End-to-end tests for cities and buildings — `docs/CITIES.md` — run through
//! the harness against the mechanics already there: territory, production,
//! combat, the tech tree's availability.

use super::*;
use crate::build::{BuildType, Ident, flags};
use crate::city::{PlaceFail, capture_value, health_level};
use crate::combat::Obj;
use crate::garrison::{GarrisonRefused, UnitTraits};
use crate::orders::Coll;
use crate::place::Blocked;
use crate::world::{UNITS_PER_CELL, cell, tile};

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
            // `role & 0x10000`, as every soldier's row carries it: the
            // capture tally counts only an armed combat type (item 1099).
            combat_role: true,
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

/// **A city finished in play counts in its region at once and reaches its
/// ground at the border pass's pace** (item 1106, `docs/AI.md` §84): its
/// `City::init` raises `reg_cities` on the frame it is made, and the fix it
/// raises writes 256 cells a frame, row-major, so its own cell — the region
/// list's 1,221st — is its owner's on the fifth pass, and `territory` is
/// summed again only when the region is done. Before the first frame a fix
/// is setup's and lands whole
/// (`a_finished_city_projects_territory_and_the_radius_mask`). East Indies'
/// `1/14` finished `1/2017` on tick 5517, and the original's colonist gate,
/// which reads that cell, passed on 5521 (run407's packet).
#[test]
fn a_city_finished_in_play_counts_at_once_and_takes_its_ground_at_the_pass_s_pace() {
    let mut w = World::new(40, 40);
    let r = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    let t = install_types(&mut sim);
    sim.ai[0].census.reg_cities = vec![0; sim.world.region_count()];
    sim.in_play = true;
    let here = tile_pos(80, 120);
    assert_eq!(here.cell(), Cell::new(20, 30), "row-major index 1,220");
    let (_, c) = city_at(&mut sim, &t, 0, 80, 120);
    assert!(sim.cities[c].alive);
    assert_eq!(
        sim.ai[0].census.reg_cities[r as usize], 1,
        "City::init counts it"
    );
    assert_eq!(
        sim.world.owner_at(here),
        Owner::None,
        "the fix writes no cell"
    );
    assert_eq!(sim.holdings[0].territory, 0);
    let mut frames = 0;
    while sim.world.owner_at(here) != Owner::Player(0) {
        sim.check_borders();
        frames += 1;
        assert!(frames < 10, "the pass ends");
    }
    assert_eq!(frames, 5, "1,221 cells at 256 a frame");
    assert_eq!(sim.holdings[0].territory, 0, "the region is not done yet");
    while sim.border_pass.is_some() {
        sim.check_borders();
    }
    assert!(sim.holdings[0].territory > 0, "summed when the pass ends");
}

/// **`City::fix_world_vals`: a city wears the map's site values down
/// round it** (`docs/AI.md` §67). The last thing `City::init` does is
/// quarter `WData.val` on every cell of the circle round the centre out to
/// ring `k = (radius + 3) / 4`, and halve it on the three rings beyond.
/// This fixture's radius is 20 tiles, so `k` is 5 and the walk ends with
/// ring 8; a cell past it keeps its value. Great Lakes 14382's Barracks
/// went up on the cell Norwich's founding had halved, and every one of
/// the leader's ten site scores stood at double the original's for want
/// of it.
#[test]
fn a_founded_city_quarters_the_site_values_inside_its_rings_and_halves_three_beyond() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    for y in 0..16 {
        for x in 0..16 {
            let c = Cell::new(x, y);
            let mut d = sim.world.cell_data(c);
            d.val = 200;
            sim.world.set_cell_data(c, d);
        }
    }
    let (_, c) = city_at(&mut sim, &t, 0, 32, 32);
    let centre = sim.cities[c].pos.cell();
    assert_eq!(centre, Cell::new(8, 8));
    assert_eq!(sim.radius_of(c), 20, "the fixture's radius, so k is 5");
    let circle = crate::ai_place::circle();
    let val = |sim: &Sim, i: usize| {
        let at = Cell::new(centre.x + circle.x[i], centre.y + circle.y[i]);
        sim.world.cell_data(at).val
    };
    assert_eq!(val(&sim, 0), 50, "the centre is quartered");
    assert_eq!(
        val(&sim, circle.radius[5] - 1),
        50,
        "ring 5's last is quartered"
    );
    assert_eq!(val(&sim, circle.radius[5]), 100, "ring 6's first is halved");
    assert_eq!(
        val(&sim, circle.radius[7] - 1),
        100,
        "ring 7's last is halved"
    );
    assert_eq!(
        sim.world.cell_data(Cell::new(0, 0)).val,
        200,
        "a corner twelve cells out is past ring 8"
    );
}

/// **`mask_me`'s first write: the `BUILDING` bit on the building's own
/// cell** — `006312a0`, `cells[y / 0x300 * xs + x / 0x300].flags |= 0x4000`
/// when marking and `&= 0xbfff` when unmarking. `docs/CITIES.md` §3.6 named
/// it at the first reading and nothing wrote it until item 352.
///
/// It is a **cell** bit, not a tile one, and it marks one cell per building
/// however large the footprint — which is why `tile::OBJECT_BUILDING` above
/// is not a substitute for it. Its one reader in the export is
/// `Army::find_muster_spot`'s ring score (`docs/ARMY.md` §13), and with the
/// bit missing the AI scored the middle of its own town as open ground:
/// Great Lakes 8442 mustered nine units on cell (50, 27) where the original
/// takes (58, 29), the cell of that ring with the fewest buildings beside
/// it. run34's own start block is the confirmation that the original writes
/// it — thirteen cells carry `0x4000` and each is a starting building's.
#[test]
fn a_building_marks_its_own_cell_and_unmarks_it_when_it_closes() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let centre = tile_pos(32, 32).cell();
    assert_eq!(
        sim.world.cell_data(centre).flags & cell::BUILDING,
        0,
        "nothing has been placed yet"
    );
    let (b, _) = city_at(&mut sim, &t, 0, 32, 32);
    assert_ne!(
        sim.world.cell_data(centre).flags & cell::BUILDING,
        0,
        "a started building marks the cell holding its own position"
    );
    // One cell, not the footprint: the village is four tiles across and
    // the tile mask covers all of them, but only its own cell is flagged.
    assert_eq!(
        sim.world.cell_data(tile_pos(36, 32).cell()).flags & cell::BUILDING,
        0,
        "the neighbouring cell is not marked"
    );
    // And the unmark. `Build::close` calls `mask_me(0)` for a started
    // building, which clears the bit it set.
    sim.close_building(b, false);
    assert_eq!(
        sim.world.cell_data(centre).flags & cell::BUILDING,
        0,
        "closing a started building unmarks its cell"
    );
}

/// Goody ruins are a cell flag; visibility changes the refusal, not its presence.
#[test]
fn goody_ruins_refuse_placement_with_the_players_visibility_verdict() {
    let mut sim = world_sim();
    let types = install_types(&mut sim);
    let at = Pos::new(32, 32);
    let cell = World::cell_of_tile(at);
    let mut data = sim.world.cell_data(cell);
    data.flags |= cell::GOODY;
    sim.world.set_cell_data(cell, data);
    assert!(sim.world.set_fog(vec![0; 32 * 32]));
    let verdict = |s: &Sim| s.blocked_tcoord(Some(0), types.town, at, None);

    sim.lobby.reveal_map = 1;
    assert_eq!(verdict(&sim), Blocked::Seen, "not even territorially seen");
    sim.lobby.reveal_map = 2;
    assert_eq!(
        verdict(&sim),
        Blocked::Unseen,
        "seen without revealing ruins"
    );
    sim.lobby.reveal_map = 3;
    assert_eq!(verdict(&sim), Blocked::Ruins, "fully revealed");
    sim.lobby.reveal_map = 1;
    assert!(sim.world.set_seen2_only(at.x >> 1, at.y >> 1, 1));
    assert_eq!(verdict(&sim), Blocked::Ruins, "actually explored");
    assert_eq!(
        sim.blocked_tcoord(None, types.town, at, None),
        Blocked::Ruins
    );

    data.flags &= !cell::GOODY;
    sim.world.set_cell_data(cell, data);
    assert_eq!(
        verdict(&sim),
        Blocked::Clear,
        "collected ruins cease to block"
    );
}

/// **`blocked_tcoord`'s tile arms, as `0x636db0` has them** (twenty-fourth
/// pass, group 12; A8 rows 40, 42, 43, 44, 52): a `0x200` tile answers RARE,
/// not BUILDING; a mountain is the tile's object bits **or** the cell's
/// `0x10`; a sea type that is no dock is refused a leader with no transport
/// level; a cell with no region is not ruins; and `blocked_site` returns a
/// first refusal of `Seen` before `blocked_location` can replace it.
#[test]
fn blocked_tcoord_s_tile_arms_follow_the_listing() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let at = Pos::new(44, 32);
    let c = World::cell_of_tile(at);
    let ask = |sim: &Sim| sim.blocked_tcoord(None, t.farm, at, None);
    // 44: `TData & 0x200`.
    let plain = sim.world.tile_mask(at);
    sim.world.set_tile_mask(at, plain | tile::AS_BUILDING);
    assert_eq!(ask(&sim), Blocked::Rare, "0x200 is RARE");
    sim.world.set_tile_mask(at, plain);
    // 43: a blocked tile on a mountain-flagged cell.
    sim.world.set_tile_mask(at, plain | tile::BLOCKED);
    assert_eq!(ask(&sim), Blocked::Rare, "blocked, no mountain bit");
    let mut d = sim.world.cell_data(c);
    d.flags |= cell::MOUNTAIN;
    sim.world.set_cell_data(c, d);
    assert_eq!(
        ask(&sim),
        Blocked::Mountain,
        "the cell's 0x10 is a mountain"
    );
    d.flags &= !cell::MOUNTAIN;
    sim.world.set_cell_data(c, d);
    sim.world.set_tile_mask(at, plain);
    // 42: a sea-domain type, no dock, no transport.
    let platform = bt(Ident::Other, None, "gb", 4, 4, 150, 400, 0);
    let platform = sim.add_build_type(platform);
    assert_eq!(
        sim.blocked_tcoord(Some(0), platform, at, None),
        Blocked::CantTransport,
        "no transport level"
    );
    sim.transport[0].civilian = true;
    assert_ne!(
        sim.blocked_tcoord(Some(0), platform, at, None),
        Blocked::CantTransport
    );
    // 52: the first refusal's `Seen` is the site's answer.
    let ruins = Pos::new(60, 60);
    let rc = World::cell_of_tile(ruins);
    let mut d = sim.world.cell_data(rc);
    d.flags |= cell::GOODY;
    sim.world.set_cell_data(rc, d);
    assert!(sim.world.set_fog(vec![0; 32 * 32]));
    sim.lobby.reveal_map = 1;
    let centre = tile_pos(ruins.x, ruins.y);
    let camp = sim.add_build_type(bt(Ident::Woodcutter, None, "gaef", 2, 2, 150, 400, 0));
    let (verdict, _) = sim.blocked_site_slots(Some(0), camp, centre, None);
    assert_eq!(verdict, Blocked::Seen, "not replaced by the location test");
}

/// **`was_seen`'s allied-owner arm tests forts as well as cities** (group 12;
/// A8 row 38, `WorldData::was_seen@006b53f0`: `reg_cities[region]` **or**
/// `reg_forts[region]`), and a cell with no region is not ruins (row 40).
#[test]
fn an_owner_s_fort_makes_its_region_seen_and_a_regionless_cell_is_no_ruin() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let fort = sim
        .place_building(0, t.fort, tile_pos(32, 32))
        .expect("a first fort in the region");
    finish(&mut sim, fort);
    assert!(sim.buildings[fort].active);
    let at = Pos::new(20, 20);
    let c = World::cell_of_tile(at);
    assert!(sim.world.set_fog(vec![0; 32 * 32]));
    sim.lobby.reveal_map = 1;
    sim.world.set_owner(c, Owner::Player(0), Owner::None);
    assert!(
        sim.was_seen_fog(at.x >> 1, at.y >> 1, 0),
        "an ally's (here its own) fort in the region shows its ground"
    );
    sim.close_building(fort, false);
    assert!(
        !sim.was_seen_fog(at.x >> 1, at.y >> 1, 0),
        "and not without"
    );

    // Row 40: no region, no ruins.
    let mut bare = Sim::new(Tuning::RON, World::new(16, 16), 2);
    let farm = bare.add_build_type(bt(Ident::Farm, None, "gda", 4, 4, 150, 400, 0));
    assert_eq!(bare.world.region_of(c), None);
    assert_ne!(bare.blocked_tcoord(None, farm, at, None), Blocked::Ruins);
}

/// **`blocked_tcoord`'s two cell arms** (`docs/CITIES.md` §2.5, item 904,
/// `docs/AI.md` §78). A rock cell refuses every type but the oil pair
/// (`006370b2`), which needs the cell's oil instead (`00637105`); and a
/// flat gather type needs its good on the tile's land (`00637532`: the
/// tile form of `get_land`, then `LandData::get_amount`). East Indies'
/// word 23182 was the first: slot 4's Farm jitter reached a rock cell.
#[test]
fn a_rock_cell_refuses_a_farm_and_a_flat_gather_type_needs_its_good() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut well = bt(Ident::OilWell, None, "gd", 4, 4, 150, 400, 0);
    well.flags |= flags::FLAT;
    let well = sim.add_build_type(well);
    city_at(&mut sim, &t, 0, 32, 32);
    let at = Pos::new(44, 32);
    let c = World::cell_of_tile(at);
    let plain = sim.world.cell_data(c);
    let verdicts =
        |sim: &Sim| [t.farm, t.barracks, well].map(|ty| sim.blocked_tcoord(Some(0), ty, at, None));
    let with = |sim: &mut Sim, flags: u16, land: i8| {
        let mut d = plain;
        d.flags |= flags;
        d.land = land;
        sim.world.set_cell_data(c, d);
    };
    // Plain land grows food and holds no oil.
    assert_eq!(
        verdicts(&sim),
        [Blocked::Clear, Blocked::Clear, Blocked::NoOil]
    );
    // Rock refuses the Farm and the Barracks; oil is what the well asks.
    with(&mut sim, cell::ROCK, 0);
    assert_eq!(
        verdicts(&sim),
        [Blocked::Rock, Blocked::Rock, Blocked::NoOil]
    );
    // Rock and oil, East Indies' cell (47, 50): the rock arm speaks first
    // for the Farm, and the well's land is oil.
    with(&mut sim, cell::ROCK | cell::OIL, 0);
    assert_eq!(
        verdicts(&sim),
        [Blocked::Rock, Blocked::Rock, Blocked::Clear]
    );
    // Oil alone: no rock, but an oil cell grows no food.
    with(&mut sim, cell::OIL, 0);
    assert_eq!(
        verdicts(&sim),
        [Blocked::NoResources, Blocked::Clear, Blocked::Clear]
    );
    // A forest cell's land is Forest whatever its tiles, and sand makes
    // nothing: neither is refused a Barracks.
    with(&mut sim, cell::FOREST, 0);
    assert_eq!(
        verdicts(&sim),
        [Blocked::NoResources, Blocked::Clear, Blocked::NoOil]
    );
    with(&mut sim, 0, 1);
    assert_eq!(
        verdicts(&sim),
        [Blocked::NoResources, Blocked::Clear, Blocked::NoOil]
    );
    // A coast cell is plain land under a land tile.
    with(&mut sim, cell::COAST, 0);
    assert_eq!(
        verdicts(&sim),
        [Blocked::Clear, Blocked::Clear, Blocked::NoOil]
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

/// **A Civic level sweeps a building that straddled the city mask into
/// its city** (item 661, `docs/AI.md` §63). Placement's `get_town` wants
/// every footprint tile inside the mask; `City::find_buildings` wants only
/// the centre tile within the radius, and `Leader::gain_tech`'s Civic arm
/// runs it on every city. A Science level runs no sweep. Great Lakes'
/// who=1 Barracks and Stable stood cityless until block 8734 in the
/// original for exactly this, and their absence from Norwich's chain was
/// 16 of each trade route's 176.
///
/// Made to fail once with the arm's call removed from `Sim::gain_tech`:
/// the Barracks stays cityless.
#[test]
fn a_civic_level_sweeps_a_straddling_building_into_its_city() {
    use crate::tech::{Line, TechTree, TypeDef};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let civic = tree.add(TypeDef::epoch("Civic 1", Line::Civic, 0));
    let science = tree.add(TypeDef::epoch("Science 1", Line::Science, 0));
    sim.set_tech_tree(tree);
    sim.start_techs(0);
    let (_, c) = city_at(&mut sim, &t, 0, 32, 32);
    let r = sim.radius_of(c);
    let centre = sim.cities[c].pos.tile();
    // The first site east of the city that places, joins no city, and
    // whose centre tile the sweep's radius still reaches.
    let b = (33..60)
        .find_map(|tx| {
            let p = tile_pos(tx, 32);
            if sim.blocked_site(Some(0), t.barracks, p, None) != Blocked::Clear
                || sim.get_town(0, t.barracks, p).is_some()
            {
                return None;
            }
            let b = sim.place_building(0, t.barracks, p).ok()?;
            let bt = sim.buildings[b].pos.tile();
            if sim.buildings[b].city.is_none()
                && world::vector_dist(centre.x - bt.x, centre.y - bt.y) <= r
            {
                Some(b)
            } else {
                sim.close_building(b, false);
                None
            }
        })
        .expect("a Barracks site straddling the city mask");
    finish(&mut sim, b);
    assert_eq!(sim.buildings[b].city, None, "placement joins no city");
    sim.gain_tech(0, science);
    assert_eq!(
        sim.buildings[b].city, None,
        "a Science level sweeps nothing"
    );
    let before = sim.num_buildings(c);
    sim.gain_tech(0, civic);
    assert_eq!(sim.buildings[b].city, Some(c), "the Civic level seats it");
    assert!(sim.cities[c].members.contains(&b));
    assert_eq!(
        sim.num_buildings(c),
        before + 1,
        "and a route's worth counts it"
    );
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

/// **A group's `QUEUE_FIRST` keeps a builder's site behind the new walk**
/// (`docs/GROUPS.md` §24, item 632). `Unit::get_goody_box` sends a
/// one-member group to the box at `QUEUE_FIRST`. The group copies the
/// leader's action-flagged `BUILDORDER` aside, halts, walks, and
/// `finish_insert`'s case 6 re-issues the build as
/// `action_swarm_around(…, QUEUE_LAST, BUILD_AT, 1)`: a fresh approach and
/// the order, both behind the walk. run157's `1/1` on 990 is that list
/// (three orders, `orders_x` on the approach). And the citizen carries
/// the group's `form_mod`, 50, but keeps its own `form`: the citizen
/// exemption at `00705749` guards `+0xaa` alone.
///
/// Made to fail once with the `Body::Build` arm removed from the insert:
/// the list comes out holding the walk alone.
#[test]
fn a_group_s_queue_first_keeps_the_build_behind_the_walk() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    sim.nation[0].human = false;
    let mut ct = citizen_type(t.village);
    ct.worker = Worker::Citizen;
    ct.cols.role = crate::ai_load::role::CITIZEN;
    let citizen = sim.add_unit_type(ct);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(30, 44));
    sim.swarm_around(u, b, Body::Build(b), true);
    let approach = |sim: &Sim, i: usize| match sim.units[u].orders[i].body {
        Body::Move(m) => m,
        other => panic!("an approach move, not {other:?}"),
    };
    let first = approach(&sim, 0);
    assert!(matches!(sim.units[u].orders[1].body, Body::Build(x) if x == b));

    let box_at = tile_pos(26, 44);
    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, u);
    assert!(sim.push_group(&mut g, true));
    sim.group_action_move_to(
        &g,
        box_at,
        QueuePos::First,
        false,
        movement::Angle(0),
        MoveKind::ExploreTo,
        false,
    );
    let orders = &sim.units[u].orders;
    assert_eq!(
        orders.len(),
        3,
        "the walk, the approach, the build: {orders:?}"
    );
    let walk = approach(&sim, 0);
    let again = approach(&sim, 1);
    assert_eq!(walk.kind, MoveKind::ExploreTo);
    assert_ne!(walk.dest.cell(), first.dest.cell(), "the box's walk leads");
    assert_eq!(again.kind, MoveKind::ExploreTo, "a computer's approach");
    assert_eq!(again.dest, first.dest, "the same unit, the same ring spot");
    let build = sim.units[u].orders[2];
    assert!(matches!(build.body, Body::Build(x) if x == b));
    assert!(
        build.has(crate::orders::flag::ACTION),
        "the action bit rides"
    );
    assert_eq!(
        sim.units[u].orders_pos, again.dest,
        "`orders_x` is the approach's, as run157 prints 37080 on 990"
    );
    assert_eq!(sim.units[u].form_width, 50, "the width twin is written");
    assert_eq!(sim.units[u].form, -1, "and the citizen's form is not");
}

/// **A computer's builder gets an approach before each build it is
/// given** (item 1532, `docs/AI.md` §144). `Leader::produce_building`
/// sends a citizen whose action is already a `BUILD_AT` through
/// `action_swarm_around(…, QUEUE_LAST, BUILD_AT, 1)`, whose member arm
/// appends an `EXPLORE_TO` to the new site's ring and then the build:
/// run669's `1/9` holds `[Build 2014, ExploreTo, Build 2022]` on block
/// 186.
///
/// Made to fail once with the `QUEUE_LAST` arm as a bare
/// `add_build_order`: the list comes out three orders long.
#[test]
fn a_busy_builder_gets_an_approach_before_its_next_build() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    sim.nation[0].human = false;
    let mut ct = citizen_type(t.village);
    ct.worker = Worker::Citizen;
    ct.cols.role = crate::ai_load::role::CITIZEN;
    let citizen = sim.add_unit_type(ct);
    let first = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let next = sim.place_building(0, t.barracks, tile_pos(24, 40)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(30, 44));
    sim.order_builder(0, u, crate::orders::index::NONE, first);
    let action = sim.action_of(u).map(|i| sim.units[u].orders[i].index());
    assert_eq!(
        action,
        Some(crate::orders::index::BUILD_AT),
        "the builder's action"
    );
    sim.order_builder(0, u, crate::orders::index::BUILD_AT, next);
    let orders = &sim.units[u].orders;
    assert_eq!(
        orders.len(),
        4,
        "approach, build, approach, build: {orders:?}"
    );
    assert!(matches!(orders[1].body, Body::Build(x) if x == first));
    match orders[2].body {
        Body::Move(m) => assert_eq!(m.kind, MoveKind::ExploreTo, "a computer's approach"),
        other => panic!("an approach move, not {other:?}"),
    }
    assert!(matches!(orders[3].body, Body::Build(x) if x == next));
    assert!(
        orders[3].has(crate::orders::flag::ACTION),
        "the action bit rides"
    );
}

/// **A human's drop places and pays for one site and gives each citizen a
/// move, then the build** (item 779, `docs/GOLDEN.md` §26, run241).
/// `Group::action_build@00707510` pays once and `action_swarm_around(site,
/// who, QUEUE_NEW, BUILD_AT, 1)` gives each member its ring spot's
/// approach and a `BUILDORDER` with the action bit behind it; the
/// approach is `local_40`'s class, `MOVE_TO` for a human and `EXPLORE_TO`
/// for a computer. The list each citizen held before is cleared.
///
/// Made to fail once with the approach's kind forced to `EXPLORE_TO`
/// (the human's class), and once with `QUEUE_NEW` passed as `QUEUE_LAST`
/// (the stale order survives).
#[test]
fn a_human_s_build_drop_gives_each_citizen_a_move_then_the_build() {
    use crate::tech::{TechTree, TypeDef};
    for human in [true, false] {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        let mut tree = TechTree::new();
        let barracks_t = tree.add(TypeDef::building("Barracks"));
        sim.set_tech_tree(tree);
        sim.build_types[t.barracks].tree = Some(barracks_t);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        sim.nation[0].human = human;
        let mut ct = citizen_type(t.village);
        ct.worker = Worker::Citizen;
        let citizen = sim.add_unit_type(ct);
        let us: Vec<usize> = [44, 46, 48]
            .iter()
            .map(|&x| spawn(&mut sim, 0, citizen, tile_pos(x, 44)))
            .collect();
        // A stale order the drop must clear.
        sim.add_move_facing_order(
            us[0],
            tile_pos(20, 20),
            MoveKind::MoveTo,
            QueuePos::New,
            true,
            movement::Angle(0),
            None,
            false,
        );
        let price = sim.building_price(0, t.barracks);
        let before = sim.ledgers[0].bucket;
        let mut g = crate::group::Group::stack(0);
        for &u in &us {
            sim.group_add(&mut g, u);
        }
        assert!(sim.push_group(&mut g, true));
        let b = sim
            .group_action_build(&g, tile_pos(40, 40), t.barracks, QueuePos::New)
            .expect("the site is placed");
        for (r, (was, cost)) in before.iter().zip(price).enumerate() {
            assert_eq!(
                sim.ledgers[0].bucket[r],
                was - cost,
                "paid once, resource {r}"
            );
        }
        let want = if human {
            MoveKind::MoveTo
        } else {
            MoveKind::ExploreTo
        };
        for &u in &us {
            let orders = &sim.units[u].orders;
            assert_eq!(orders.len(), 2, "the approach and the build: {orders:?}");
            assert!(
                matches!(orders[0].body, Body::Move(m) if m.kind == want && m.dest != tile_pos(20, 20)),
                "human {human}: the approach is {want:?}: {orders:?}"
            );
            assert!(!orders[0].has(crate::orders::flag::ACTION));
            assert!(matches!(orders[1].body, Body::Build(x) if x == b));
            assert!(orders[1].has(crate::orders::flag::ACTION));
        }
    }
}

/// **A human's one-unit swarm walks under a move** (item 779, run241's
/// `0/8` on 1097). `find_build_spot`'s help and `do_build`'s re-entry are
/// `action_swarm_around` on one citizen, and its approach is `local_40`'s
/// class like the group's: `MOVE_TO` for a human. Made to fail once with
/// the kind fixed at `EXPLORE_TO`, as this crate had it.
#[test]
fn a_human_s_one_unit_swarm_walks_under_a_move() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    for (human, want) in [(true, MoveKind::MoveTo), (false, MoveKind::ExploreTo)] {
        sim.nation[0].human = human;
        let u = spawn(&mut sim, 0, citizen, tile_pos(30, 44));
        sim.swarm_around(u, b, Body::Build(b), false);
        assert!(
            matches!(sim.units[u].orders[0].body, Body::Move(m) if m.kind == want),
            "human {human}: {:?}",
            sim.units[u].orders
        );
    }
}

/// **A closed site's builder walks on until its number is reused** (item
/// 644). `Unit::work@0060d180:319` ends the action on a `uid` mismatch,
/// and a closed building keeps its uid until `Objects::find_free` hands the
/// number to the next one. run157's `1/7` keeps its walk and `BUILDORDER`
/// on the camp the script placed and destroyed on 1176 to the end of the
/// capture; this crate used to drop both on the alive bit.
#[test]
fn a_closed_site_s_builder_walks_on_until_its_number_is_reused() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(30, 44));
    sim.swarm_around(u, b, Body::Build(b), true);
    assert_eq!(sim.units[u].orders.len(), 2, "the approach, then the build");
    sim.close_building(b, false);
    let start = sim.units[u].pos;
    for _ in 0..3 {
        sim.tick();
    }
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Build(x) if x == b)),
        "the build on the closed site stands: {:?}",
        sim.units[u].orders
    );
    assert_ne!(sim.units[u].pos, start, "and the walk under it goes on");
    // The next building takes the dead one's number, and the uid test fails.
    let b2 = sim.place_building(0, t.barracks, tile_pos(46, 40)).unwrap();
    assert_eq!(
        sim.buildings[b2].index, sim.buildings[b].index,
        "the number is reused"
    );
    sim.tick();
    assert!(
        !sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Build(x) if x == b)),
        "a reused number ends it: {:?}",
        sim.units[u].orders
    );
}

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
    // `BUILDINGS_FASTER_1..3`'s prerequisites, as the shipped file's
    // Construction, Architecture and Engineering.
    let mut tree = tech::TechTree::new();
    let ladder = ["Construction", "Architecture", "Engineering"]
        .map(|n| tree.add(tech::TypeDef::plain(n, 0)));
    tree.roles.buildings_speed_preq = ladder.map(Some);
    sim.set_tech_tree(tree);
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
    for x in ladder {
        sim.tech[0].tech[x] = true;
    }
    sim.wall_stats_dirty[0] = true;
    sim.tick();
    assert_eq!(sim.buildings[b].constr_time, 42_000);
}

/// Item 1476 (`docs/AI.md` §121): Construction is `BUILDINGS_FASTER_1`'s
/// and `BUILDINGS_HP_1`'s prerequisite. run636's who=1 gained it on 13782
/// and holds its five unfinished sites at `constr_time` 90000 and its
/// Smelter `1/2047` at `myhits` 1100 — a tenth off the clock, a tenth on
/// the hit points — where this crate held 100000 and 1000.
#[test]
fn construction_takes_a_tenth_off_the_clock_and_adds_a_tenth_to_the_hits() {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    let t = install_types(&mut sim);
    let mut tree = tech::TechTree::new();
    let construction = tree.add(tech::TypeDef::plain("Construction", 0));
    let other = tree.add(tech::TypeDef::plain("Herbal Lore", 0));
    tree.roles.buildings_speed_preq = [Some(construction), None, None];
    tree.roles.buildings_hp_preq = [Some(construction), None, None];
    sim.set_tech_tree(tree);
    sim.tech[0].epoch[tech::Line::Civic as usize] = 1;
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 32)).unwrap();
    sim.tick();
    let (clock, hits) = (sim.buildings[b].constr_time, sim.buildings[b].hits);
    // A tech off both ladders moves neither.
    sim.tech[0].tech[other] = true;
    sim.wall_stats_dirty[0] = true;
    sim.tick();
    assert_eq!(sim.buildings[b].constr_time, clock);
    assert_eq!(sim.buildings[b].hits, hits);
    sim.tech[0].tech[construction] = true;
    sim.wall_stats_dirty[0] = true;
    sim.tick();
    assert_eq!(
        sim.buildings[b].constr_time,
        9 * clock / 10,
        "(10 − 1) × t / 10"
    );
    assert_eq!(
        sim.buildings[b].hits,
        110 * hits / 100,
        "BUILDING_HP_UPGRADE, 10 % a level"
    );
}

/// **The President's clause of `BuildData::construct_time`** (item 1503, Great
/// Sahara at Toughest's Smelter `1/2053`, finished on 14363 at a clock of
/// 75187 — `100000 × 100 / 133`). `has_general(0, 0x163)` finds one of the
/// owner's THEPRESIDENT heroes, on the map and within `get_radius × 0xc0`
/// of the building less its footprint's `(x_size + y_size) × 0x60`.
///
/// Made to fail with the clause back at `president: false` (the clock stays
/// the base), with the owner test dropped (a rival's President shortens it),
/// and with the type test dropped (a Senator does).
#[test]
fn the_president_near_a_site_shortens_its_clock_by_a_quarter() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 32)).unwrap();
    let base = sim.buildings[b].constr_time;
    assert_eq!(sim.construct_time_of(b), base, "no President");
    let hero = |sim: &mut Sim, type_index: i32| {
        sim.add_unit_type(crate::UnitType {
            hits: 100,
            type_index,
            cols: crate::ai_load::UnitCols {
                unit_flags2: crate::ai_load::uflags2::GENERAL,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        })
    };
    let senator = hero(&mut sim, 0x161);
    let president = hero(&mut sim, 0x163);
    let _ = sim.init_unit(0, senator, tile_pos(41, 33));
    assert_eq!(
        sim.construct_time_of(b),
        base,
        "a Senator is not The President"
    );
    let _ = sim.init_unit(1, president, tile_pos(41, 34));
    assert_eq!(sim.construct_time_of(b), base, "a rival's President");
    let far = sim.init_unit(0, president, tile_pos(110, 110));
    assert_eq!(sim.construct_time_of(b), base, "out of reach");
    sim.units[far].pos = tile_pos(41, 34);
    assert_eq!(
        sim.construct_time_of(b),
        base * 100 / (sim.tuning.thepresident_building_speed + 100),
        "THEPRESIDENT_BUILDING_SPEED 33 %"
    );
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

/// `Leader::gain_tech`'s buildings cascade, the city arm (`0x6dec2f`,
/// `docs/AI.md` §98): a city that already holds its kinds while the Large
/// City is still locked levels up **on the gain** of the Large City's
/// prerequisite, not at its next building's completion. East Indies'
/// London and Norwich did so with the Medieval Age in the original and
/// stood Small here until a building finished, which put the AI's `pop` at
/// 3 against 7 and every research offer at three sevenths.
#[test]
fn the_age_the_large_city_needs_levels_a_ready_city_up_on_the_gain() {
    use crate::tech::{TechTree, TypeDef};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let classical = tree.add(TypeDef::age("Classical Age", 0));
    let medieval = tree.add(TypeDef::age("Medieval Age", 1));
    let town = tree.add(TypeDef::building("Large City").needs(0, medieval));
    tree.ages[0] = Some(classical);
    tree.ages[1] = Some(medieval);
    tree.roles.town = Some(town);
    tree.add_tribe(tech::Tribe::default());
    tree.finalize();
    sim.build_types[t.town].tree = Some(town);
    sim.tech_tree = tree;
    for w in 0..sim.tech.len() {
        sim.tech[w] = tech::PlayerTech::new(&sim.tech_tree);
    }
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    let sites = [
        (t.barracks, tile_pos(40, 40)),
        (t.library, tile_pos(24, 40)),
        (t.market, tile_pos(40, 24)),
        (t.temple, tile_pos(24, 24)),
        (t.farm, tile_pos(44, 32)),
    ];
    for (ty, pos) in sites {
        let p = sim.place_building(0, ty, pos).unwrap();
        finish(&mut sim, p);
    }
    assert_eq!(sim.num_kinds(c), 6, "the kinds are there");
    assert_eq!(sim.city_level_of(c), 1, "the Large City is locked");
    assert_eq!(sim.ai[0].census.pop, 1);
    sim.gain_tech(0, classical);
    assert_eq!(
        sim.city_level_of(c),
        1,
        "an age the Large City does not need"
    );
    sim.gain_tech(0, medieval);
    assert_eq!(sim.city_level_of(c), 2, "levelled up on the gain");
    assert_eq!(sim.buildings[b].ty, Some(t.town));
    assert_eq!(sim.ai[0].census.pop, 3, "and `pop` follows it");
}

/// **The Spice rare and the caravans' nation power scale a route** —
/// `Caravan::trade_value@0073d9d0`'s tail, on the leader of the end the
/// original passes in `ecx`/`edx`, which `compute_trade` makes the city's
/// own owner (`docs/CARAVAN.md` §4). Great Sahara's who=1 holds Spice,
/// and its two Villages' routes read 26 where 22 is the bare sum (item
/// 1189). Each city here is a Major City of one finished building, `1 + 4`
/// a side, in opposite corners, and the pair is foreign so each owner's
/// rare is its own: 10, doubled by band 3, half again foreign.
#[test]
fn spice_and_the_caravan_power_scale_a_route_on_the_computing_city_s_owner() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b0, c0) = city_at(&mut sim, &t, 0, 6, 6);
    let (b1, c1) = city_at(&mut sim, &t, 1, 56, 56);
    sim.buildings[b0].ty = Some(t.metropolis);
    sim.buildings[b1].ty = Some(t.metropolis);
    let v = sim.init_caravan(1, 0).expect("a slot");
    let slot = &mut sim.caravans[1].slots[v];
    slot.city_a = Some(c1);
    slot.city_b = Some(c0);
    slot.delivered = true;
    assert_eq!(
        sim.trade_value(c0, c1),
        30,
        "(5 + 5) · (3 + 3) / 3 · 3 / 2, bare"
    );
    sim.compute_trade(c0);
    sim.compute_trade(c1);
    assert_eq!(
        (sim.cities[c0].trade_val, sim.cities[c1].trade_val),
        (240, 240),
        "30 · 16 / 2 at each end"
    );
    // Spice for who=1 alone: its own city's end is worth 30 · 120 / 100.
    sim.ledgers[1].rare |= 1 << (economy::SPICE - economy::BASE_RARE);
    sim.compute_trade(c0);
    sim.compute_trade(c1);
    assert_eq!(
        (sim.cities[c0].trade_val, sim.cities[c1].trade_val),
        (240, 288),
        "the rare is the computing city's owner's"
    );
    // The caravans' power for who=0 alone: 30 · 115 / 100.
    sim.tech[0].power = Some(0x15);
    sim.tech[0].has_city = true;
    sim.compute_trade(c0);
    assert_eq!(sim.cities[c0].trade_val, 34 * 8, "and so is the power");
}

/// **A new route refreshes both its cities' trade** (item 1275):
/// `do_trade@005ed270:393–396` runs `City::compute_trade` on the two ends
/// the first time the order is stepped, after the pair is linked and before
/// `build_road`. The route itself adds nothing — it has not delivered — but
/// every route those cities already hold is re-summed at today's
/// `trade_value`, which a round trip is otherwise the only thing to
/// refresh. Great Sahara at Toughest's `1/52` takes city 2 ↔ 3 on 6587 and
/// city 2's delivered route to city 1 goes from 176 to 184.
#[test]
fn a_new_route_recomputes_the_trade_its_cities_already_hold() {
    use crate::orders::TradeOrder;
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    sim.tech[0].epoch[tech::Line::Civic as usize] = 2;
    let (b0, c0) = city_at(&mut sim, &t, 0, 6, 6);
    let (b1, c1) = city_at(&mut sim, &t, 0, 32, 32);
    let (_, c2) = city_at(&mut sim, &t, 0, 56, 56);
    sim.buildings[b0].ty = Some(t.metropolis);
    sim.buildings[b1].ty = Some(t.metropolis);
    // The old route, city 0 ↔ 1, has delivered and was summed.
    let old = sim.init_caravan(0, 0).expect("a slot");
    let slot = &mut sim.caravans[0].slots[old];
    slot.alive = true;
    slot.linked = true;
    slot.city_a = Some(c0);
    slot.city_b = Some(c1);
    slot.delivered = true;
    sim.compute_trade(c1);
    let before = sim.cities[c1].trade_val;
    assert!(before > 0);
    // City 0 is worth less since: the stored sum is stale.
    sim.buildings[b0].ty = Some(t.village);
    let fresh = sim.trade_value(c1, c0) * 16 / 2;
    assert_ne!(fresh, before, "the partner's value moved");
    assert_eq!(sim.cities[c1].trade_val, before);
    // A caravan at city 1 takes a route to city 2.
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let u = spawn(&mut sim, 0, citizen, tile_pos(34, 34));
    let v = sim.init_caravan(0, u).expect("a slot");
    sim.units[u].caravan = Some(v);
    sim.enqueue_order(
        u,
        crate::orders::Order {
            flags: 0,
            body: Body::Trade(TradeOrder {
                home: c1,
                dest: Some(c2),
                started: false,
                loaded: false,
            }),
        },
        crate::orders::QueuePos::New,
    );
    sim.do_trade(u);
    assert!(sim.caravans[0].slots[v].linked, "the pair is linked");
    assert!(!sim.caravans[0].slots[v].delivered);
    assert_eq!(
        sim.cities[c1].trade_val, fresh,
        "city 1's old route is re-summed at today's value"
    );
}

/// `LeaderData::pop` and `reg_pop` — `CityData::get_pop_value@00738450`,
/// which is **1, 3, 5** and not the level, summed over the leader's live
/// cities. The number is `create_units`' and `research_techs`' whole `base`
/// (`docs/AI.md` §27), and it was zero here until 2026-09-04.
#[test]
fn a_leader_s_pop_is_one_three_five_by_city_level() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    let (b2, _) = city_at(&mut sim, &t, 1, 8, 8);
    let reg = sim
        .world
        .region_of(tile_pos(32, 32).cell())
        .expect("the city stands in a region");
    let pop = |s: &crate::Sim, who: usize| {
        (
            s.ai[who].census.pop,
            crate::ai::Census::reg(&s.ai[who].census.reg_pop, reg),
        )
    };
    assert_eq!(pop(&sim, 0), (1, 1), "a Small City is worth one");
    // Per leader, and the other leader's city is not in this one's count.
    assert_eq!(pop(&sim, 1).0, 1);
    // The Large City is three and the Major City five — `2 · level − 1`.
    sim.buildings[b].ty = Some(t.town);
    sim.sync_pop_cities();
    assert_eq!(pop(&sim, 0), (3, 3), "a Large City is worth three");
    sim.buildings[b].ty = Some(t.metropolis);
    sim.sync_pop_cities();
    assert_eq!(pop(&sim, 0), (5, 5), "a Major City is worth five");
    // A city that dies leaves with its own value, not with one.
    sim.close_building(b, true);
    assert_eq!(pop(&sim, 0), (0, 0));
    assert_eq!(pop(&sim, 1).0, 1, "the other leader is untouched");
    let _ = (b2, c);
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

/// `Unit::come_out@00617c10:535` recurses on `o_down` with the "already the
/// captain" flag, so **every member of a squad runs the exit search for
/// itself**, with the siblings already placed standing in it
/// (`docs/CITIES.md` §6.5.1). This crate placed the whole squad on the
/// captain's spot, which is what stacked run76's three Archers on one point.
///
/// The captain still takes the ring's first candidate — due south at
/// `(4 + 4) * 0x30 + 288`, snapped — which run76 confirms to the unit.
#[test]
fn a_squad_comes_out_one_member_at_a_time_and_no_two_share_a_spot() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut squad = hoplite_type(t.barracks);
    squad.combat.uber_size = 3;
    // `coll_size` is `block_radius / 48`, so a radius of 1 occupies no cell
    // at all and no sibling could ever block another.
    squad.combat.block_radius = 48;
    let squad = sim.add_unit_type(squad);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);

    let produced = sim.build_train(b, squad);
    let members = sim.squad_members(produced.unit);
    assert_eq!(members.len(), 3, "uber_size 3 is three objects");

    let spots: Vec<Pos> = members.iter().map(|&m| sim.units[m].pos).collect();
    for &m in &members {
        assert!(sim.units[m].on_map && sim.units[m].inside.is_none());
    }
    // The whole point: three searches, three answers.
    let mut seen = spots.clone();
    seen.sort_by_key(|p| (p.x, p.y));
    seen.dedup();
    assert_eq!(
        seen.len(),
        3,
        "each member searches for itself, so no two share a spot: {spots:?}"
    );
    // And the captain is still on the ring's own first candidate, due south
    // of the trainer at the inner radius, snapped to its quarter-tile centre.
    let bp = sim.buildings[b].pos;
    let hp = sim.units[produced.unit].pos;
    assert_eq!(
        (hp.x - bp.x, hp.y - bp.y),
        (24, 8 * 0x30 + 288 + 24),
        "the captain keeps the ring's own first candidate"
    );
}

/// **A computer's member comes out with its own orders** (item 1457,
/// `docs/CITIES.md` §6.5.3): each member's `come_out` runs the tail, and
/// for a computer's unit that is not a plane, a Citizen or a Scholar the
/// tail ends in `update_action`, so a member's `orders_x/y` are its new
/// place. A human's members are left as they were trained.
#[test]
fn a_computer_s_squad_members_come_out_with_their_own_orders() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut squad = hoplite_type(t.barracks);
    squad.combat.uber_size = 3;
    squad.combat.block_radius = 48;
    squad.type_index = 0x52;
    let squad = sim.add_unit_type(squad);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    sim.nation[0].human = false;
    let cap = sim.build_train(b, squad).unit;
    for m in sim.squad_members(cap) {
        let u = &sim.units[m];
        assert_eq!(u.orders_pos, u.pos, "1/{}'s orders are its place", u.index);
    }
}

/// **`come_out`'s push** (`618900`..`6189aa`, item 882, `docs/GOLDEN.md`
/// §33): a trained squad — the captain out of a building, its type's
/// `uber_size` over 1 — is `Group::add`ed and `push_group(who, g, 1)`ed,
/// so every member names one pool slot and the slot lists the squad in
/// `captain, o_down…` order. There is no owner test: a computer's squad
/// is pushed exactly as a human's. A one-object type is not pushed.
#[test]
fn a_trained_squad_is_pushed_into_a_pool_slot_of_its_own_and_a_single_unit_is_not() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut squad = hoplite_type(t.barracks);
    squad.combat.uber_size = 3;
    squad.combat.block_radius = 48;
    let squad = sim.add_unit_type(squad);
    let single = sim.add_unit_type(hoplite_type(t.barracks));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);

    let lone = sim.build_train(b, single).unit;
    assert_eq!(sim.pool_group_of(lone), -1, "uber_size 1 takes no slot");

    let cap = sim.build_train(b, squad).unit;
    let members = sim.squad_members(cap);
    assert_eq!(members.len(), 3);
    let g = sim.pool_group_of(cap);
    assert!(g >= 0, "the captain names a slot");
    for &m in &members {
        assert_eq!(sim.pool_group_of(m), g, "every member names the same slot");
    }
    let listed: Vec<i16> = members.iter().map(|&m| sim.units[m].index).collect();
    assert_eq!(
        sim.pool_list(0, g as u8),
        listed,
        "the slot lists the squad"
    );

    // A second squad takes a fresh slot: the first is live and is
    // `last_group`'s.
    let cap2 = sim.build_train(b, squad).unit;
    let g2 = sim.pool_group_of(cap2);
    assert!(g2 >= 0 && g2 != g);
}

/// **A command's building group of two** (item 888, `docs/GOLDEN.md`
/// §37, run304). `process_group` adds every listed building and pushes
/// once: one record lists both, in the command's order; the same group
/// again is `equals_group`'s against `last_group`'s record and seats
/// nothing; the two in the other order are another group, and take the
/// next open slot; a building listed twice is added once.
#[test]
fn a_command_s_two_buildings_take_one_slot_in_its_order_and_an_equal_group_takes_none() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let a = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let b = sim.place_building(0, t.barracks, tile_pos(48, 40)).unwrap();
    finish(&mut sim, a);
    finish(&mut sim, b);
    let (oa, ob) = (sim.buildings[a].index, sim.buildings[b].index);
    let seat = |sim: &crate::Sim, want: &[i16]| {
        (0..64u8).find_map(|s| {
            sim.pool_building_group(0, s)
                .filter(|(_, o)| o.as_slice() == want)
                .map(|(st, _)| (s, st.stamp))
        })
    };

    sim.frame = 641;
    sim.push_command_buildings(0, &[a, b]);
    let (s1, stamp) = seat(&sim, &[oa, ob]).expect("one record lists both, in order");
    assert_eq!(stamp, 641);

    sim.frame = 661;
    sim.push_command_buildings(0, &[a, b]);
    assert_eq!(
        seat(&sim, &[oa, ob]),
        Some((s1, 641)),
        "an equal group seats nothing and keeps its stamp"
    );

    sim.frame = 681;
    sim.push_command_buildings(0, &[b, a]);
    let (s2, stamp) = seat(&sim, &[ob, oa]).expect("the other order is another group");
    assert_ne!(s2, s1);
    assert_eq!(stamp, 681);
    assert_eq!(
        seat(&sim, &[oa, ob]),
        Some((s1, 641)),
        "the first record is kept"
    );

    sim.frame = 701;
    sim.push_command_buildings(0, &[a, a]);
    assert!(
        seat(&sim, &[oa]).is_some(),
        "a building listed twice is added once"
    );
}

/// **`Group::action_queue_up@006fdbb0`'s train arm on two buildings**
/// (item 888, `docs/GOLDEN.md` §37, run304): the members sorted once by
/// `queued`, least first, then `num` passes of one entry a member, the
/// answer of each `queue_up` unread. With money for exactly three entries
/// and the busier listed first, the idle member takes the first and third
/// and the busier the second; laid a member at a time it would take all
/// three, and unsorted the busier would take two.
#[test]
fn a_press_on_two_buildings_lays_one_entry_a_member_a_pass_from_the_least_queued() {
    let food = economy::Resource::Food.index();
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let ty = sim.add_unit_type(hoplite_type(t.barracks));
    sim.muster[0].researched[ty] = true;
    let busy = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let idle = sim.place_building(0, t.barracks, tile_pos(48, 40)).unwrap();
    finish(&mut sim, busy);
    finish(&mut sim, idle);
    // A Barracks' room, twenty (`docs/PRODUCTION.md`, "Queueing charges
    // the price"), so no member is refused for room.
    sim.buildings[busy].queue.capacity = 20;
    sim.buildings[idle].queue.capacity = 20;
    sim.ledgers[0].bucket[food] = 100_000;
    sim.queue_up(busy, ty).unwrap();
    // Exactly three more entries' worth, whatever the ramp.
    let mut probe = sim.clone();
    let before = probe.ledgers[0].bucket[food];
    for _ in 0..3 {
        probe.queue_up(idle, ty).unwrap();
    }
    sim.ledgers[0].bucket[food] = before - probe.ledgers[0].bucket[food];

    assert_eq!(sim.action_queue_up(&[busy, idle], ty, 3), 3);
    assert_eq!(
        sim.buildings[idle].queue.items.len(),
        2,
        "the first and third"
    );
    assert_eq!(
        sim.buildings[busy].queue.items.len(),
        2,
        "its own and the second"
    );
    assert_eq!(sim.ledgers[0].bucket[food], 0);
    let paid = |b: usize, i: usize| sim.buildings[b].queue.items[i].cost[0];
    assert!(paid(idle, 0) <= paid(busy, 1) && paid(busy, 1) <= paid(idle, 1));
}

/// **A player's garrison command, and the building's eject** (item 718,
/// `docs/ORDERS.md` §29, run208). `Group::action_garrison` gives each
/// member that `can_garrison` the building one GARRISON with the action
/// bit, and a member that cannot — a citizen at a barracks — nothing.
/// The first of the squad through the door takes the whole squad in, and
/// `kill_garrison_order` walks the captain's chain, so no member keeps
/// its walk or its GARRISON inside. On the way out a unit keeps its own
/// heading and each member takes its captain's.
#[test]
fn a_player_s_garrison_takes_the_squad_in_whole_and_its_eject_keeps_their_angles() {
    use crate::orders::{Body, QueuePos, flag};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut squad = hoplite_type(t.barracks);
    squad.combat.uber_size = 3;
    squad.combat.block_radius = 48;
    let squad = sim.add_unit_type(squad);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    let produced = sim.build_train(b, squad);
    let cap = produced.unit;
    let members = sim.squad_members(cap);
    assert_eq!(members.len(), 3);
    let c = spawn(&mut sim, 0, citizen, tile_pos(40, 52));

    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, cap);
    sim.group_add(&mut g, c);
    assert!(sim.push_group(&mut g, true));
    sim.group_action_garrison(&g, b, QueuePos::New);
    for &m in &members {
        let o = sim.units[m].orders.front().copied().expect("a GARRISON");
        assert!(matches!(o.body, Body::Garrison { building, search: false } if building == b));
        assert_ne!(
            o.flags & flag::ACTION,
            0,
            "the command's order is an action"
        );
    }
    assert!(
        sim.units[c].orders.is_empty(),
        "a citizen cannot garrison a barracks, so it gets no order"
    );

    for _ in 0..300 {
        if members.iter().all(|&m| sim.units[m].inside == Some(b)) {
            break;
        }
        sim.tick();
    }
    for &m in &members {
        assert_eq!(sim.units[m].inside, Some(b), "the squad goes in whole");
        assert!(
            sim.units[m].orders.is_empty(),
            "no member keeps its walk or its GARRISON inside: {:?}",
            sim.units[m].orders
        );
    }

    let heading = crate::movement::Angle(0x1234_0000);
    for &m in &members {
        sim.units[m].movement.heading = crate::movement::Angle(m as i32);
    }
    sim.units[cap].movement.heading = heading;
    sim.action_eject_all(0, &[b], -1, -1);
    sim.tick();
    for &m in &members {
        assert!(
            sim.units[m].on_map,
            "one squad a frame, and this is the one"
        );
        assert_eq!(
            sim.units[m].movement.heading, heading,
            "the captain keeps its heading, and each member takes it"
        );
    }
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

/// **Only what is armed counts in the capture tally** (item 1099,
/// `docs/CITIES.md` §7.2): `Search::valid_filter(…, 8)` passes an armed
/// combat unit and an armed, finished building that is not a city. A
/// temple, a market, a granary and two citizens round a city at zero
/// defend it with nothing, so
/// three hoplites take it against the base of 2; a tower counts its 7.
#[test]
fn unarmed_buildings_and_citizens_do_not_defend_a_city() {
    let setup = |tower: bool| {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        let (b, _) = city_at(&mut sim, &t, 1, 32, 32);
        sim.tech[1].epoch[tech::Line::Civic as usize] = 1;
        let _ = city_at(&mut sim, &t, 1, 8, 8);
        for (ty_, tx, ty) in [(t.temple, 25, 32), (t.market, 32, 25), (t.granary, 32, 39)] {
            let x = sim
                .place_building(1, ty_, tile_pos(tx, ty))
                .unwrap_or_else(|e| panic!("a building at ({tx}, {ty}): {e:?}"));
            finish(&mut sim, x);
        }
        if tower {
            let x = sim
                .place_building(1, t.tower, tile_pos(39, 36))
                .unwrap_or_else(|e| panic!("the tower: {e:?}"));
            finish(&mut sim, x);
        }
        let citizen = sim.add_unit_type(citizen_type(t.village));
        for i in 0..2 {
            spawn(&mut sim, 1, citizen, tile_pos(28, 28 + i));
        }
        let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
        let bd = &mut sim.buildings[b];
        bd.damage = bd.hits;
        bd.sync_health();
        sim.frame = 1000;
        let hs: Vec<usize> = (0..3)
            .map(|i| spawn(&mut sim, 0, hoplite, tile_pos(36, 32 + i)))
            .collect();
        (sim, b, hs[2])
    };
    let (mut sim, b, h) = setup(false);
    assert!(
        sim.check_capture(b, h),
        "three hoplites against the base of 2: the unarmed buildings and citizens count nothing"
    );
    let (mut sim, b, h) = setup(true);
    assert!(
        !sim.check_capture(b, h),
        "the tower counts 1 + 6, and 2 + 7 holds against 3"
    );
}

/// **`Object::valid_target`'s capture attempt** (`docs/CITIES.md` §7.1's
/// fourth caller, `docs/AI.md` §145, item 1535): asking whether a unit may
/// target a capture-eligible city is itself a `Build::check_capture` with the
/// asker as the captor — and the answer to the question is *no* for a
/// hoplite either way, the city being the capture path's and not the
/// attack's. Three hoplites against the base of 2 take it on the third's
/// question; a decoy's question takes nothing.
#[test]
fn valid_target_on_a_city_at_its_ceiling_is_a_capture_attempt() {
    let setup = |n: i32| {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        let (b, _) = city_at(&mut sim, &t, 1, 32, 32);
        sim.tech[1].epoch[tech::Line::Civic as usize] = 1;
        let _ = city_at(&mut sim, &t, 1, 8, 8);
        let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
        let bd = &mut sim.buildings[b];
        bd.damage = bd.hits;
        bd.sync_health();
        sim.frame = 1000;
        let hs: Vec<usize> = (0..n)
            .map(|i| spawn(&mut sim, 0, hoplite, tile_pos(36, 32 + i)))
            .collect();
        (sim, b, hs)
    };
    let (mut sim, b, hs) = setup(3);
    assert!(sim.capture_eligible(b));
    let city = Obj::Building(b);
    assert!(
        !sim.valid_target(Obj::Unit(hs[2]), city),
        "a hoplite never targets a city at its ceiling"
    );
    assert!(
        sim.cities.iter().any(|c| c.alive && c.owner == 0),
        "…and its question took the city: three against the base of 2"
    );
    let (mut sim, b, hs) = setup(2);
    assert!(!sim.valid_target(Obj::Unit(hs[1]), Obj::Building(b)));
    assert!(
        !sim.cities.iter().any(|c| c.alive && c.owner == 0),
        "two against 2 holds"
    );
    let (mut sim, b, hs) = setup(3);
    sim.units[hs[2]].decoy = true;
    assert!(!sim.valid_target(Obj::Unit(hs[2]), Obj::Building(b)));
    assert!(
        !sim.cities.iter().any(|c| c.alive && c.owner == 0),
        "a decoy captures nothing"
    );
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
    // on arrival is the direction at order time (§4.1) — **to the point the
    // caller named, not to the snap of it** (§4.3). `tile_pos` is a
    // 192-tile centre, which is never a 48-cell centre, so the two differ
    // here by 24 units on each axis and the bearings by 1.7°.
    let Body::Move(m) = sim.units[u].orders[0].body else {
        panic!()
    };
    assert_eq!(m.dest.x.rem_euclid(48), 24);
    assert_eq!(m.dest.y.rem_euclid(48), 24);
    let here = Pos::new(at.x + 24, at.y + 24);
    let to = tile_pos(24, 20);
    assert_eq!(
        m.angle,
        movement::find_angle(to.x - here.x, to.y - here.y),
        "the bearing to the caller's point"
    );
    assert_eq!(
        movement::find_angle(m.dest.x - here.x, m.dest.y - here.y),
        movement::Angle::EAST,
        "and the snapped destination is due east, which the order's angle is not"
    );
    assert_ne!(m.angle, movement::Angle::EAST);
}

/// **The two arrival arms, and the one clause between them** (§4.5).
///
/// `move_step@005faf30` ends a final leg two ways. The **Manhattan snap**
/// (`manh <= step`, the destination written in outright) faces the order's
/// angle when the move was the only order **or the action beneath is a
/// gather** — `005fb562`. The **partial step** (the unit walked its whole
/// step and happened to land on the destination) faces it only when the
/// move was the only order — `005fb4a8`, no gather clause. A farmer's last
/// leg is routinely the second kind: run10's AI farmers land on their cells
/// by a 29-unit Manhattan step at speed 25 whose sine and cosine still
/// reach, so they keep the bearing of that step, and the human's land by a
/// 7-unit snap and take the order's angle. Both are on frames 110 and 116
/// of the same capture, which is what made the pair readable.
///
/// Made to fail by giving `arrive` the gather clause on both arms: the
/// partial-step citizen then faces east with the rest.
#[test]
fn only_the_snap_arm_s_arrival_faces_the_order_s_angle_under_a_gather() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    // The farm is far away: the gather order beneath is the *action*, and
    // nothing it does interferes with the leg under test.
    let farm = sim.place_building(0, t.farm, tile_pos(20, 20)).unwrap();
    finish(&mut sim, farm);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;

    // A 48-snapped destination, and two starts: one 7 units of Manhattan
    // away (the snap takes it) and one 29 (a full 25-unit step lands on it
    // exactly — `sin/cos` of the bearing are −5 and 24).
    let dest = Pos::new(176 * 48 + 24, 130 * 48 + 24);
    let arrive_from = |sim: &mut Sim, d: (i32, i32), gather: bool| -> movement::Angle {
        let from = Pos::new(dest.x - d.0, dest.y - d.1);
        let u = spawn(sim, 0, citizen, from);
        // Settled on the bearing, so the step is taken along it and no
        // frame is spent turning.
        let bearing = movement::find_angle(d.0, d.1);
        sim.units[u].movement.facing = bearing;
        sim.units[u].movement.heading = bearing;
        if gather {
            sim.add_gather_order(u, farm, QueuePos::New, false);
        }
        // The order's own angle is due east, which neither bearing is.
        sim.add_move_facing_order(
            u,
            dest,
            MoveKind::MoveTo,
            QueuePos::First,
            false,
            movement::Angle::EAST,
            None,
            false,
        );
        let mut frames = 0;
        while sim.units[u]
            .orders
            .front()
            .is_some_and(crate::orders::Order::is_move)
        {
            sim.tick();
            frames += 1;
            assert!(frames < 20, "never arrived from {d:?}");
        }
        assert_eq!(sim.units[u].pos, dest, "landed on the destination");
        sim.units[u].movement.heading
    };

    // The snap arm, gather beneath: the order's angle.
    assert_eq!(
        arrive_from(&mut sim, (3, -4), true),
        movement::Angle::EAST,
        "the snap arm takes the order's angle under a gather"
    );
    // The partial arm, gather beneath: the bearing of its own last step.
    assert_eq!(
        arrive_from(&mut sim, (-5, -24), true),
        movement::find_angle(-5, -24),
        "the partial step keeps its own bearing"
    );
    // The same partial arm with the move as the only order: the order's
    // angle after all, which is the clause the gather one sits beside.
    assert_eq!(
        arrive_from(&mut sim, (-5, -24), false),
        movement::Angle::EAST,
        "a lone move takes the order's angle on either arm"
    );
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

/// **The farm switch reads the guy's live `cur_anim`, not a flag of the
/// farm's own** — `docs/ORDERS.md` §6.5. `do_gather@005ef2a0:454` tests
/// `*(char *)(**(int **)&this->field_0xf4 + 0x9c)`, `GuyData +0x9c`, so
/// an empty cell is sown by anyone *not* mid-reap and re-picked by a
/// reaper — and anything at all that plays an animation between two farm
/// frames decides which.
///
/// That is the difference between a byte and a flag, and it is worth two
/// hundred frames of Great Lakes' word (item 84). The AI's farmer stands
/// on a cell it has been reaping, `Farms::inc_time` decays the cell empty
/// under it, it re-picks a tile — and is blocked on the way, so
/// `Unit::move_step+0x823`'s stand replaces the reap **without the body
/// moving a unit**. The next farm frame the original sows. A flag written
/// only by this branch and cleared only by a step re-picks for ever.
///
/// The two sims here differ in one byte and nothing else.
#[test]
fn an_empty_farm_cell_is_sown_unless_the_guy_is_still_reaping() {
    fn farmer_on_a_fresh_farm(anim: i8) -> (Sim, usize, usize) {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let farm = sim.place_building(0, t.farm, tile_pos(40, 32)).unwrap();
        finish(&mut sim, farm);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        // Standing on the farm, so `do_gather` arrives on the first tick
        // and reaches the switch on the same frame.
        let stand = sim.buildings[farm].pos;
        let u = spawn(&mut sim, 0, citizen, stand);
        sim.units[u].guys = vec![crate::anim::Guy::fresh(-1)];
        sim.units[u].guys[0].anim = anim;
        sim.add_gather_order(u, farm, QueuePos::New, false);
        sim.trace_phases = true;
        (sim, u, farm)
    }

    // The re-target's own draws, counted by name: the farm's sprout clock
    // is on the same stream, so a seed is not the instrument here.
    let re_picks = |sim: &Sim| {
        sim.phase_marks
            .iter()
            .filter(|(site, _)| site == orders::SITE_FARM_CELL)
            .count()
    };

    // Every cell of a fresh farm is empty, and the guy is playing nothing.
    let (mut sowing, u, farm) = farmer_on_a_fresh_farm(crate::anim::DEFAULT);
    sowing.tick();
    assert_eq!(
        sowing.buildings[farm]
            .farm
            .state
            .iter()
            .filter(|&&c| c == farms::GROWING)
            .count(),
        1,
        "the cell under the farmer is sown"
    );
    assert_eq!(re_picks(&sowing), 0, "and a sow draws for no new tile");
    assert!(matches!(
        sowing.units[u].orders.front().map(|o| o.body),
        Some(Body::Gather(_))
    ));

    // The same farmer, mid-reap: the cell is left alone and a new tile is
    // drawn for — `GameAccess::rnd(x_size)` then `rnd(y_size)`, one mark.
    let (mut reaping, v, farm2) = farmer_on_a_fresh_farm(crate::anim::REAP);
    reaping.tick();
    assert_eq!(
        reaping.buildings[farm2].farm.state,
        [farms::EMPTY; 16],
        "a reaper sows nothing"
    );
    assert!(
        matches!(
            reaping.units[v].orders.front().map(|o| o.body),
            Some(Body::Move(_))
        ),
        "it walks to the new tile instead: {:?}",
        reaping.units[v].orders.front().map(|o| o.body)
    );
    assert_eq!(re_picks(&reaping), 1, "the pair, and one of it");
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

/// **`Unit::find_build_spot@00603e20`** — where a builder with nothing left
/// to do goes, and the first of `build_done`'s three searches
/// (`docs/ORDERS.md` §5.5).
///
/// Four things at once, because they are one walk: an AI citizen that has
/// just finished a site takes the **next** one rather than gathering at
/// the one it raised; a site out of `UNIT_BUILD_RESPOND_RANGE` is not a
/// candidate; among candidates the **nearest ring** wins a tie on builder
/// count, because the found list is the circle's own order and the
/// min-search is a strict `<` from index 0; and a site that already has
/// builders loses to one that has none however much further out it is.
///
/// Great Lakes' 2803 is the capture behind it: `1/1` finishes building
/// `2010` and walks to `2011`, which is the whole of the map's word going
/// 2808 → 2930.
#[test]
fn an_ai_builder_takes_the_next_site_nearest_first_and_least_crowded() {
    use crate::tech::{TechTree, TypeDef};

    /// An AI player 0 with a city, a citizen type and the six goods its
    /// research sweep indexes.
    fn ground() -> (Sim, Types, usize) {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        sim.nation[0].human = false;
        sim.lobby.starting_resources = 1;
        let mut tree = TechTree::new();
        for name in ["Food", "Timber", "Metal", "Wealth", "Knowledge", "Oil"] {
            tree.add(TypeDef::good(name));
        }
        sim.set_tech_tree(tree);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        (sim, t, citizen)
    }

    // 1. The end-to-end shape: two sites, the builder finishes one and is
    //    left holding a `BUILD_AT` on the other — not a `GATHER` on its
    //    own farm, which is what a **human** builder would take.
    let (mut sim, t, citizen) = ground();
    let mine = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
    let next = sim.place_building(0, t.farm, tile_pos(50, 32)).unwrap();
    let builder = spawn(&mut sim, 0, citizen, tile_pos(44, 32));
    sim.add_build_order(builder, mine, QueuePos::New, true);
    let mut frames = 0;
    while !sim.buildings[mine].active {
        sim.tick();
        frames += 1;
        assert!(frames < 4_000, "the farm should go up");
    }
    let held: Vec<Body> = sim.units[builder].orders.iter().map(|o| o.body).collect();
    assert!(
        held.contains(&Body::Build(next)),
        "the AI's builder walks to the site it has not built: {held:?}"
    );

    // 2. Range. `UNIT_BUILD_RESPOND_RANGE` is 12 tiles, doubled to 24 on
    //    worker stance 1, and the search is over **cells** — so a site 40
    //    tiles away is not a candidate and the citizen finds nothing.
    let (mut sim, t, citizen) = ground();
    let far = sim.place_building(0, t.farm, tile_pos(20, 32)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(60, 32));
    assert!(
        !sim.find_build_spot(u),
        "a site 40 tiles out is past the doubled respond range"
    );
    assert!(sim.units[u].orders.is_empty());
    // And in range it is found.
    let u2 = spawn(&mut sim, 0, citizen, tile_pos(30, 32));
    assert!(sim.find_build_spot(u2));
    assert!(
        sim.units[u2]
            .orders
            .iter()
            .any(|o| o.body == Body::Build(far)),
        "the same site ten tiles out is one"
    );

    // 3. The tie goes to the **nearer** ring: both sites are empty, and
    //    the circle reaches the closer cell first.
    let (mut sim, t, citizen) = ground();
    let close = sim.place_building(0, t.farm, tile_pos(38, 32)).unwrap();
    let further = sim.place_building(0, t.farm, tile_pos(48, 32)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(36, 32));
    assert!(sim.find_build_spot(u));
    let held: Vec<Body> = sim.units[u].orders.iter().map(|o| o.body).collect();
    assert!(
        held.contains(&Body::Build(close)) && !held.contains(&Body::Build(further)),
        "an empty tie goes to the nearer site: {held:?}"
    );

    // 4. …and one builder on the near site is enough to lose it, because
    //    the choice is the count and only then the order.
    let (mut sim, t, citizen) = ground();
    let close = sim.place_building(0, t.farm, tile_pos(38, 32)).unwrap();
    let further = sim.place_building(0, t.farm, tile_pos(48, 32)).unwrap();
    let busy = spawn(&mut sim, 0, citizen, tile_pos(38, 34));
    sim.add_build_order(busy, close, QueuePos::New, false);
    let u = spawn(&mut sim, 0, citizen, tile_pos(36, 32));
    assert!(sim.find_build_spot(u));
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| o.body == Body::Build(further)),
        "the crowded near site loses to the empty far one"
    );

    // 5. `build_masks & 0x20` — a site under attack is no candidate at
    //    all, whoever is free.
    let (mut sim, t, citizen) = ground();
    let hot = sim.place_building(0, t.farm, tile_pos(38, 32)).unwrap();
    sim.buildings[hot].under_attack |= 0x2;
    let u = spawn(&mut sim, 0, citizen, tile_pos(36, 32));
    assert!(
        !sim.find_build_spot(u),
        "a site under attack is not offered to a free builder"
    );
}

/// **`game->total_units` counts owners below nine** (`Unit::init@00612100
/// :379`), and it is what picks `find_units`' walk for the builder tally
/// `find_build_spot` takes (`docs/AI.md` §148). The list walk keeps a
/// builder only within `range` plus its radius; the circle walk keeps every
/// builder whose **cell** is in the ring, and a cell at offset (−4, −5) is
/// in ring 6 while its unit can stand 4,900 units out. run672's `1/8` on
/// 690: 134 counted units against `circle_radius[6]`'s 145, and nineteen
/// owner-9 birds and beasts beside them.
#[test]
fn the_builder_tally_walks_the_lists_while_the_counted_units_are_few() {
    use crate::tech::{TechTree, TypeDef};
    let run = |pad_owner: Player| {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        sim.nation[0].human = false;
        sim.lobby.starting_resources = 1;
        let mut tree = TechTree::new();
        for name in ["Food", "Timber", "Metal", "Wealth", "Knowledge", "Oil"] {
            tree.add(TypeDef::good(name));
        }
        sim.set_tech_tree(tree);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        let near = sim.place_building(0, t.farm, tile_pos(44, 40)).unwrap();
        let far = sim.place_building(0, t.farm, tile_pos(40, 30)).unwrap();
        let on_near = spawn(&mut sim, 0, citizen, tile_pos(44, 42));
        sim.add_build_order(on_near, near, QueuePos::New, false);
        // Cell (6, 5) against the searcher's (10, 10): ring 6 by the
        // circle, 4,916 units out by `vector_dist`.
        let on_far = spawn(&mut sim, 0, citizen, Pos::new(4776, 4076));
        sim.add_build_order(on_far, far, QueuePos::New, false);
        for _ in 0..150 {
            let _ = spawn(&mut sim, pad_owner, citizen, tile_pos(4, 60));
        }
        let u = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
        assert!(sim.find_build_spot(u));
        let held: Vec<Body> = sim.units[u].orders.iter().map(|o| o.body).collect();
        (held, near, far)
    };
    // Three counted units and 150 of owner 9: the lists, so the far site's
    // builder is out of reach and the far site is the emptier one.
    let (held, _, far) = run(9);
    assert!(
        held.contains(&Body::Build(far)),
        "owner 9 is not counted, so the tally walks the lists: {held:?}"
    );
    // 150 of the enemy's instead: counted, so the circle, the far site's
    // builder in ring 6, and the tie goes to the nearer site.
    let (held, near, _) = run(1);
    assert!(
        held.contains(&Body::Build(near)),
        "a player's units are counted, so the tally walks the circle: {held:?}"
    );
}

/// **The circle walk counts a builder by its own tile's region**
/// (`Objects::find_units@0065a620`'s `0x200` gate, `docs/AI.md` §155): a
/// builder standing on the water of a coastal cell — land by the cell's
/// `region`, the sea by its `region2` for an ocean tile — is in the ring
/// and is not counted. run679's `1/24`, carried at sea towards the Mine
/// `1/2028` on 1340, left that site the emptier one, and `1/8` went to it.
/// 150 of the enemy's units keep the tally on the circle (§148).
#[test]
fn the_builder_tally_counts_no_builder_on_a_coastal_cell_s_water() {
    use crate::tech::{TechTree, TypeDef};
    let run = |at_sea: bool| {
        let mut w = World::new(16, 16);
        let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
        let coast = Cell::new(12, 12);
        let mut d = w.cell_data(coast);
        d.flags |= cell::HALFLAND;
        d.region2 = Some(land + 1);
        w.set_cell_data(coast, d);
        for tx in 48..52 {
            for ty in 48..52 {
                w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
            }
        }
        let mut sim = Sim::new(Tuning::RON, w, 2);
        sim.declare_war(0, 1);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; economy::RESOURCES];
        }
        let t = install_types(&mut sim);
        sim.nation[0].human = false;
        sim.lobby.starting_resources = 1;
        let mut tree = TechTree::new();
        for name in ["Food", "Timber", "Metal", "Wealth", "Knowledge", "Oil"] {
            tree.add(TypeDef::good(name));
        }
        sim.set_tech_tree(tree);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        // Cell (9, 10) comes before (11, 10) on the circle (ring 1, `x`
        // ascending), so a tie goes to `first`.
        let first = sim.place_building(0, t.farm, tile_pos(36, 40)).unwrap();
        let second = sim.place_building(0, t.farm, tile_pos(44, 40)).unwrap();
        let on_first = spawn(&mut sim, 0, citizen, tile_pos(36, 42));
        sim.add_build_order(on_first, first, QueuePos::New, false);
        let on_second = spawn(&mut sim, 0, citizen, tile_pos(44, 42));
        sim.add_build_order(on_second, second, QueuePos::New, false);
        // A second builder on `first`: on the coastal cell's water, or on
        // its land beside it.
        let shore = if at_sea {
            tile_pos(49, 49)
        } else {
            tile_pos(46, 46)
        };
        let carried = spawn(&mut sim, 0, citizen, shore);
        sim.add_build_order(carried, first, QueuePos::New, false);
        for _ in 0..150 {
            let _ = spawn(&mut sim, 1, citizen, tile_pos(4, 60));
        }
        let u = spawn(&mut sim, 0, citizen, tile_pos(40, 40));
        assert!(sim.find_build_spot(u));
        let held: Vec<Body> = sim.units[u].orders.iter().map(|o| o.body).collect();
        (held, first, second)
    };
    // At sea: one builder counted on each, and the tie goes to `first`.
    let (held, first, _) = run(true);
    assert!(
        held.contains(&Body::Build(first)),
        "the builder on the water is in another region: {held:?}"
    );
    // On land: two on `first`, so `second` is the emptier.
    let (held, _, second) = run(false);
    assert!(
        held.contains(&Body::Build(second)),
        "the builder on land is counted: {held:?}"
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

/// `find_nearby_spot`'s collision half (§10, `docs/COLLISION.md` §5.2):
/// the sweep refuses a candidate another unit is **standing on** and one
/// another unit has been **ordered to**, and `Coll::None` takes either.
///
/// This is item 66's mechanic on a bench. In run10 the AI's new citizen
/// `1/7` was sent to the exact quarter-tile `1/6` was standing on, because
/// the sweep's last test had never been written.
#[test]
fn the_spot_search_refuses_a_unit_standing_there_and_one_walking_there() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    let citizen = sim.add_unit_type(citizen_type(t.village));
    // `coll_size` is `block_radius / 48`, and a type with none is in
    // neither index — the whole test would pass vacuously.
    sim.unit_types[citizen].combat.block_radius = 48;
    let east = spawn(&mut sim, 0, citizen, tile_pos(46, 40));
    let other = spawn(&mut sim, 0, citizen, tile_pos(20, 20));
    let site = sim.buildings[b].pos;
    let here = sim.units[east].pos;
    let angle = movement::find_angle(here.x - site.x, here.y - site.y);
    let ring = 3 * 0x60 + 0x30;

    let clear = sim
        .find_nearby_spot(east, site, ring, 0, -1, angle, Some(b))
        .expect("open ground has a spot");

    // Standing on it. The next candidate must be somewhere else, and far
    // enough that the two blocks do not overlap: Chebyshev `> 2` unit
    // cells, which is `coll_size 1` twice over.
    sim.set_new_location(other, clear, true);
    let pushed = sim
        .find_nearby_spot(east, site, ring, 0, -1, angle, Some(b))
        .expect("the ring is not full");
    assert_ne!(pushed, clear, "the occupied quarter-tile is refused");
    let (a, c) = (collide::ucell(pushed), collide::ucell(clear));
    assert!(
        (a.x - c.x).abs() > 2 || (a.y - c.y).abs() > 2,
        "and so is every candidate whose block overlaps it: {pushed:?}"
    );
    assert_eq!(
        sim.find_nearby_spot_coll(east, site, ring, 0, -1, angle, Some(b), Coll::None),
        Some(clear),
        "`nocoll` takes it regardless"
    );

    // Walking to it. Two unit cells clear of the spot is far enough that
    // the *position* test passes, and near enough to stay on the chain
    // the nine world cells around the candidate walk.
    let aside = Pos::new(clear.x, clear.y + 3 * 0x30);
    sim.set_new_location(other, aside, true);
    sim.units[other].orders_pos = aside;
    assert_eq!(
        sim.find_nearby_spot(east, site, ring, 0, -1, angle, Some(b)),
        Some(clear),
        "standing aside, it blocks nothing"
    );
    sim.units[other].orders_pos = clear;
    assert_ne!(
        sim.find_nearby_spot(east, site, ring, 0, -1, angle, Some(b)),
        Some(clear),
        "but a spot it has been ordered to is taken"
    );
    assert_eq!(
        sim.find_nearby_spot_coll(east, site, ring, 0, -1, angle, Some(b), Coll::None),
        Some(clear),
        "`nocoll` takes that too"
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

/// **A building's periodic phase is its object number, not its handle**
/// (`Build::process@0061edf0:728`; `docs/CITIES.md` §5.8, item 233).
///
/// This is the assertion that would have caught the defect: the handle and
/// the object number agree in **no** game — `o` is per player and starts at
/// [`crate::BUILD_BASE`] (2000, which is 16 mod 32), the handle is global
/// and starts at 0 — so the whole 32-frame family fired sixteen frames off
/// for player 0's first building and somewhere else again for everyone
/// else's. Both directions are tested on the same building, so a keying
/// that happened to agree on one frame cannot pass.
#[test]
fn a_building_s_periodic_phase_is_its_object_number() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b0, _) = city_at(&mut sim, &t, 0, 32, 32);
    let (b1, _) = city_at(&mut sim, &t, 1, 8, 8);
    // Handle 0 and 1; object number 2000 for both, because the band is per
    // player. Neither handle is congruent to 2000 mod 32.
    assert_eq!((b0, sim.buildings[b0].index), (0, 2000));
    assert_eq!((b1, sim.buildings[b1].index), (1, 2000));
    for b in [b0, b1] {
        assert_ne!(
            (b as i64) % 32,
            i64::from(sim.buildings[b].index) % 32,
            "the fixture must be one where the two keyings disagree",
        );
    }

    // The under-attack decay (§1.3) is the cheapest of the family to
    // observe: `0x3` becomes `0x2` on the phase frame and on no other.
    for b in [b0, b1] {
        let o = i64::from(sim.buildings[b].index);
        let mut fired = Vec::new();
        for frame in 0..64 {
            sim.buildings[b].under_attack = 0x3;
            sim.process_building(b, frame);
            if sim.buildings[b].under_attack != 0x3 {
                fired.push(frame);
            }
        }
        assert_eq!(
            fired,
            (0..64).filter(|f| (f + o) % 32 == 0).collect::<Vec<i64>>(),
            "building {b} (o {o}): the 32-frame work is phased by o, not by \
             the handle",
        );
    }
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

/// **The player's research** (item 883, `docs/GOLDEN.md` §35,
/// `docs/PRODUCTION.md` "The player's research"): `action_queue_up`'s
/// research arm. `researching` refuses the command whole; the first pass
/// offers the job only to an idle member and a refusal goes on, so a
/// research on [an idle building that cannot make it, a busy Library]
/// lands behind the busy one; `can_make` refuses a technology already
/// held; and a cancel clears the gate. Made to fail with each of the gate,
/// the second pass and the held refusal dropped.
#[test]
fn the_player_s_research_lands_once_behind_its_gate() {
    use crate::tech::{Line, TechTree, TypeDef};
    let food = economy::Resource::Food.index();
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let library_t = tree.add(TypeDef::building("Library"));
    let mut ww = TypeDef::epoch("Written Word", Line::Science, 0).at(library_t);
    ww.cost[food] = 4;
    ww.job_time = 40;
    let written_word = tree.add(ww);
    let mut cs = TypeDef::epoch("City State", Line::Civic, 0).at(library_t);
    cs.cost[food] = 6;
    cs.job_time = 40;
    let city_state = tree.add(cs);
    sim.set_tech_tree(tree);
    sim.build_types[t.library].tree = Some(library_t);
    let (city_b, _) = city_at(&mut sim, &t, 0, 32, 32);
    let lib = sim.init_build(0, t.library, tile_pos(36, 32), false);
    finish(&mut sim, lib);
    sim.ledgers[0].bucket[food] = 1_000;
    let factor = sim.tuning.tech_cost_factor;

    assert_eq!(sim.action_queue_research(&[lib], written_word), 1);
    assert_eq!(sim.ledgers[0].bucket[food], 1_000 - 4 * factor);
    assert_eq!(
        sim.action_queue_research(&[lib], written_word),
        0,
        "researching: the command refused whole"
    );
    assert_eq!(sim.buildings[lib].queue.items.len(), 1);
    assert_eq!(sim.ledgers[0].bucket[food], 1_000 - 4 * factor);

    // The city is idle and cannot make it; the Library is busy. The first
    // pass offers it to the city alone, which refuses, and the second
    // lands it behind the Library's head.
    assert_eq!(sim.action_queue_research(&[city_b, lib], city_state), 1);
    let q = &sim.buildings[lib].queue.items;
    assert_eq!(q.len(), 2, "behind the busy member");
    assert_eq!(q[1].tech, Some(city_state));
    assert!(sim.buildings[city_b].queue.items.is_empty());

    while !sim.tech[0].tech[written_word] {
        sim.tick();
    }
    let before = sim.ledgers[0].bucket[food];
    assert_eq!(
        sim.action_queue_research(&[lib], written_word),
        0,
        "held: can_make's has_tech"
    );
    assert_eq!(sim.buildings[lib].queue.items.len(), 1);
    assert_eq!(sim.ledgers[0].bucket[food], before, "nothing charged");

    assert!(sim.cancel(lib, 0).is_some());
    assert!(!sim.researching(0, city_state));
    assert_eq!(
        sim.action_queue_research(&[lib], city_state),
        1,
        "the cancel cleared the gate"
    );
}

/// **run39's library, end to end**: Written Word on 201, City State on 382.
///
/// The two entries the AI queues at frame 2 in both long captures, with the
/// shipped numbers — both `JOB_TIME 200`, City State `12f`. Only slot 0
/// advances (one library city), so they run one after the other, and the pair
/// exercises every side of the science discount at once:
///
/// - **The gate is strict.** Written Word is itself the Science epoch, so
///   while it is being researched `epoch[3]` is still zero against its own
///   level of zero and it takes the full 20,000 hundredths — 200 frames of
///   climbing, observed done on the 201st.
/// - **The refund.** Gaining it re-prices what is still queued *before* the
///   counter it is about to raise, so City State's 120 food comes back to 108
///   (`docs/COSTS.md` §Paying).
/// - **The speedup.** From frame 202 `epoch[3]` is one and City State's own
///   level is zero, so its target is 18,000 rather than 20,000 and it lands on
///   382. Before item 86 this simulation charged it the full 20,000 and landed
///   on 402; those twenty frames were the whole of the twenty-one fields
///   `run39_s_build_queues_are_the_original_s_clock` disagreed on.
///
/// The target is recomputed every frame rather than stored, which is what lets
/// a level arriving mid-queue shorten a job already under way.
#[test]
fn the_science_epoch_shortens_the_entry_behind_it_and_refunds_its_price() {
    use crate::tech::{Line, TechTree, TypeDef};
    let food = economy::Resource::Food.index();
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let library_t = tree.add(TypeDef::building("Library"));
    // `JOB_TIME 200` on both, as `techrules.xml` writes them.
    let mut ww = TypeDef::epoch("Written Word", Line::Science, 0).at(library_t);
    ww.job_time = 200;
    let written_word = tree.add(ww);
    let mut cs = TypeDef::epoch("City State", Line::Commerce, 0).at(library_t);
    cs.job_time = 200;
    cs.cost[food] = 12;
    let city_state = tree.add(cs);
    sim.set_tech_tree(tree);
    sim.build_types[t.library].tree = Some(library_t);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let lib = sim.init_build(0, t.library, tile_pos(36, 32), false);
    finish(&mut sim, lib);
    sim.ledgers[0].bucket[food] = 1_000;

    assert_eq!(sim.queue_tech(lib, written_word), Ok(0));
    assert_eq!(sim.queue_tech(lib, city_state), Ok(1));
    assert_eq!(
        sim.ledgers[0].bucket[food],
        1_000 - 120,
        "12f × TECH_COST_FACTOR, charged on queue with no science discount: \
         City State is an epoch, so its level is its AGE of zero and the \
         player is level with it"
    );
    assert_eq!(sim.buildings[lib].queue.items[1].cost[0], 120);
    // Neither entry is sped up while the player has no Science level: the
    // gate is `level < science`, and both are zero.
    assert_eq!(sim.queue_target(lib, 0), 20_000);
    assert_eq!(sim.queue_target(lib, 1), 20_000);

    let mut frames = 0;
    while !sim.tech[0].tech[written_word] {
        sim.tick();
        frames += 1;
        assert!(frames < 600);
    }
    assert_eq!(frames, 201, "200 frames of climbing, the 201st observes it");
    assert_eq!(sim.tech[0].epoch[Line::Science.index()], 1);
    // The refund landed: the entry carries the re-struck price, and the
    // twelve food that came back with it is
    // `run40_s_census_prices_the_ai_s_second_city_at_sixty`'s. (The ledger is
    // not asserted here because the city beside the library is also earning.)
    assert_eq!(sim.buildings[lib].queue.items[0].cost[0], 108);
    // And the entry behind it is now a level behind the player's Science.
    assert_eq!(sim.queue_target(lib, 0), 18_000);

    while !sim.tech[0].tech[city_state] {
        sim.tick();
        frames += 1;
        assert!(frames < 600);
    }
    assert_eq!(frames, 382, "the original's frame, not 402");
    assert!(sim.buildings[lib].queue.items.is_empty());
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

/// **`find_gather_spot` scores the headroom under the commerce cap, not the
/// distance** — `docs/ORDERS.md` §6.6, and the whole of queue item 70.
///
/// The numerator is `Σ_g resource_cap[g] − income[g]` over the goods the
/// leader has and the building gathers, so two candidates that land in the
/// same distance bucket are separated entirely by which good the player is
/// further from maxing. run10's frame 321 is exactly that shape: the AI's
/// ninth citizen stands 1,958 from a Woodcutter's Camp and 2,041 from a
/// farm, both `/ 0xc0 == 10`, and the original takes the farm.
///
/// Here the geometry is made a dead heat on purpose — one gather building
/// six tiles east, one six tiles west — so nothing but the ledger can
/// decide, and the choice flips when the ledger does.
#[test]
fn a_citizen_gathers_where_the_cap_has_the_most_room_left() {
    fn pick(food_income: i32, timber_income: i32, timber_over_cap: bool) -> Option<Ident> {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        let camp = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 4, 4, 150, 400, 0));
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        city_at(&mut sim, &t, 0, 30, 30);

        // Six tiles either way: `vector_dist` is 1,152 to both, and
        // `1152 / 0xc0 + 2` is 8 for both. The distance term cancels.
        let farm = sim.place_building(0, t.farm, tile_pos(36, 30)).unwrap();
        sim.plant_camp_forest(tile_pos(24, 30));
        let wood = sim.place_building(0, camp, tile_pos(24, 30)).unwrap();
        for b in [farm, wood] {
            finish(&mut sim, b);
            // The camp's own slot count is the map survey (`gather.rs`) and
            // there are no trees here; the search only asks for room.
            sim.buildings[b].gather_max = Some(4);
        }

        let food = economy::Resource::Food.index();
        let timber = economy::Resource::Timber.index();
        let l = &mut sim.ledgers[0];
        l.cap = [70 * economy::RATE_SCALE; economy::RESOURCES];
        l.income = [0; economy::RESOURCES];
        l.income[food] = food_income;
        l.income[timber] = timber_income;
        l.over_cap = [economy::OverCap::Under; economy::RESOURCES];
        if timber_over_cap {
            l.over_cap[timber] = economy::OverCap::At;
        }

        let u = spawn(&mut sim, 0, citizen, tile_pos(30, 30));
        // The range the caller passes; both candidates are well inside it.
        let range = sim.tuning.unit_gather_respond_range * TILE;
        if !sim.find_gather_spot(u, range) {
            return None;
        }
        match sim.units[u].orders.front().map(|o| o.body) {
            Some(Body::Gather(g)) => Some(sim.building_ident(g.building)),
            other => panic!("a gather order was expected, got {other:?}"),
        }
    }

    // Timber further from its cap than food: the camp, which is also the
    // building the walk-order tie would have given anyway.
    assert_eq!(
        pick(600, 100, false),
        Some(Ident::Woodcutter),
        "the good with the most headroom wins"
    );
    // Food further from its cap: the farm, at the same distance. This is
    // the row a distance-only score could never produce.
    assert_eq!(
        pick(100, 600, false),
        Some(Ident::Farm),
        "and it wins from the other side too"
    );
    // A good already over its cap contributes nothing at all — not a
    // smaller number, nothing — so its building drops out of the search
    // even though it is the nearer half of a dead heat.
    assert_eq!(
        pick(600, 100, true),
        Some(Ident::Farm),
        "an over-cap good takes its building out of the running"
    );
    // And with every good at its cap there is no positive score anywhere:
    // `local_20` starts at 0 and the comparison is strict, so the search
    // finds nothing rather than falling back on the nearest.
    assert_eq!(
        pick(70 * economy::RATE_SCALE, 70 * economy::RATE_SCALE, false),
        None,
        "a zero score is not a candidate"
    );
}

/// **A walk goes in front of the order that issued it, and the order is
/// still the order** (`docs/ORDERS.md` §6.4).
///
/// `Unit::do_non_flat_gather` is handed a pointer to the `GatherOrder` and
/// keeps it for the whole function; `add_move_order(…, QUEUE_FIRST, …)`
/// only relinks the list head. So the `goto_build = 1` and `wait = 32` the
/// return-to-camp branch writes *after* it issues the walk land on the
/// gather order that is now second in the list — never on the move.
///
/// Writing the front instead was a silent no-op, and it cost the whole of
/// a woodcutter's working life: `0/2` on run33 finished its shift on frame
/// 426, reached its camp on 431, and with `goto_build` still 0 and `wait`
/// still −1 re-entered the same branch on 432 and every other frame after
/// it for the rest of the capture — a stand and a walk to the spot it was
/// already standing on, never an unload and never another tile. That
/// stand is where run33's word parted.
#[test]
fn the_return_walk_goes_in_front_of_the_gather_order_it_updates() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let camp = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 4, 4, 150, 400, 0));
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;
    city_at(&mut sim, &t, 0, 30, 30);
    sim.plant_camp_forest(tile_pos(24, 30));
    let wood = sim.place_building(0, camp, tile_pos(24, 30)).unwrap();
    finish(&mut sim, wood);
    sim.buildings[wood].gather_max = Some(4);
    // Out at a tree, five tiles from the camp, with the shift just over:
    // `goto_build == 0` and `wait < 0` is "return to the camp".
    let u = spawn(&mut sim, 0, citizen, tile_pos(30, 30));
    sim.add_gather_order(u, wood, orders::QueuePos::Last, false);
    let Some(Body::Gather(g)) = sim.units[u].orders.front_mut().map(|o| &mut o.body) else {
        panic!("the gather order");
    };
    g.goto_build = false;
    g.been_there = true;
    g.wait = -1;
    g.tile = Some(Pos::new(31, 30));

    sim.work(u, 0);

    // The walk is in front, and it is a carrying one.
    assert!(
        sim.units[u]
            .orders
            .front()
            .is_some_and(orders::Order::is_move),
        "the return walk goes in front: {:?}",
        sim.units[u].orders
    );
    assert_eq!(sim.units[u].carry, orders::CARRY_WITH_WOOD);
    // And the two fields behind it are the gather order's, not the move's.
    let g = sim.units[u]
        .orders
        .iter()
        .find_map(|o| match o.body {
            Body::Gather(g) => Some(g),
            _ => None,
        })
        .expect("the gather order is still there");
    assert!(g.goto_build, "goto_build was written to the walk instead");
    assert_eq!(g.wait, 32, "wait was written to the walk instead");
}

/// **A dock, a market or a temple is a wealth gather slot, and the first of
/// them past the mark pays thirty** (`docs/ECONOMY.md`, "The wealth slot,
/// and the thirty it pays" — `Build::activate@00623e20` line 590).
///
/// The block reads and writes `gather_slots[2]` and `gather_slots_high[2]`
/// — `LeaderData +0x8ac` and `+0x8dc`, which are those two array entries
/// and not the separate counters they were once read as. So the shape is
/// the gather block's: claim always, pay only past the high-water mark, and
/// never at frame 0.
#[test]
fn a_dock_a_market_and_a_temple_each_claim_a_wealth_slot() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let dock = sim.add_build_type(bt(Ident::Dock, None, "jam", 4, 4, 420, 1200, 0));
    city_at(&mut sim, &t, 0, 32, 32);
    let w = economy::Resource::Wealth.index();
    sim.ledgers[0].bucket[w] = 0;
    sim.frame = 1;

    // The market is the first: one slot, and thirty wealth for it.
    let m = sim.place_building(0, t.market, tile_pos(20, 20)).unwrap();
    finish(&mut sim, m);
    assert_eq!(sim.ledgers[0].gather_slots[w], 1);
    assert_eq!(sim.ledgers[0].gather_slots_high[w], 1);
    assert_eq!(sim.ledgers[0].bucket[w], 30);

    // A temple and a dock are the same kind, and each is past the mark.
    let te = sim.place_building(0, t.temple, tile_pos(20, 28)).unwrap();
    finish(&mut sim, te);
    // A dock wants water under it, which this bare world has none of, so
    // it is stood up the way the diff harness stands a dump's own
    // buildings: `Build::init` then `Build::activate`.
    let d = sim.init_build(0, dock, tile_pos(44, 20), false);
    sim.activate(d, false, true);
    assert_eq!(sim.ledgers[0].gather_slots[w], 3);
    assert_eq!(sim.ledgers[0].bucket[w], 90);

    // A library is not, and neither is a farm.
    let l = sim.place_building(0, t.library, tile_pos(44, 28)).unwrap();
    finish(&mut sim, l);
    assert_eq!(sim.ledgers[0].gather_slots[w], 3, "a library is not a slot");
    assert_eq!(sim.ledgers[0].bucket[w], 90);

    // `Build::close` gives the slot back; the mark stands, so the rebuild
    // is free of bonus. That is the whole point of the high-water pair.
    sim.disband_building(te, true);
    assert_eq!(sim.ledgers[0].gather_slots[w], 2);
    assert_eq!(sim.ledgers[0].gather_slots_high[w], 3);
    sim.ledgers[0].bucket[w] = 0;
    let te2 = sim.place_building(0, t.temple, tile_pos(20, 28)).unwrap();
    finish(&mut sim, te2);
    assert_eq!(sim.ledgers[0].gather_slots[w], 3);
    assert_eq!(sim.ledgers[0].bucket[w], 0, "a rebuild is not a new slot");
}

/// **And at frame 0 the slot is claimed and nothing is paid** — the same
/// gate the gather bonuses take, which is what keeps a dump's own starting
/// market from paying. run59's human is the record: a Market it was handed
/// at setup, `gather_slots[wealth]` 1, and a wealth bucket that never saw
/// the thirty.
#[test]
fn a_market_stood_up_at_frame_zero_claims_its_slot_and_pays_nothing() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    city_at(&mut sim, &t, 0, 32, 32);
    let w = economy::Resource::Wealth.index();
    sim.ledgers[0].bucket[w] = 0;
    assert_eq!(sim.frame, 0);

    let m = sim.place_building(0, t.market, tile_pos(20, 20)).unwrap();
    finish(&mut sim, m);
    assert_eq!(sim.ledgers[0].gather_slots[w], 1);
    assert_eq!(
        sim.ledgers[0].gather_slots_high[w], 1,
        "the mark still rises"
    );
    assert_eq!(sim.ledgers[0].bucket[w], 0);
}

/// **The carrying walk is `unit_masks & 0x78000000`, not the order.**
///
/// `Guy::set_anim`'s walk arm reads four bits off the unit and nothing else
/// (`005db61f`–`005db665`); `Unit::do_non_flat_gather` is the only writer,
/// and it sets one on each walk it issues. This crate had been deriving the
/// slot from the gather order's `goto_build`, which made the citizen's very
/// first walk to its camp — issued by `find_gather_spot`, before
/// `do_non_flat_gather` has ever run — a `CHAR_WALK_WITH_WOOD`. The
/// original plays that walk as the plain `CHAR_WALK` and takes the arrival
/// stand at the end of it, which is a draw (`docs/ANIM.md` §4.4); run33's
/// `1/7` carries `unit_masks 262146` on the frame it arrives, and the two
/// sides parted on frame 232 over exactly that.
#[test]
fn the_carrying_walk_comes_off_the_mask_and_not_off_goto_build() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let camp = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 4, 4, 150, 400, 0));
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;
    city_at(&mut sim, &t, 0, 30, 30);
    sim.plant_camp_forest(tile_pos(24, 30));
    let wood = sim.place_building(0, camp, tile_pos(24, 30)).unwrap();
    finish(&mut sim, wood);
    sim.buildings[wood].gather_max = Some(4);
    let u = spawn(&mut sim, 0, citizen, tile_pos(30, 30));
    sim.add_gather_order(u, wood, orders::QueuePos::Last, false);

    // The order says it is heading to the building; the mask is clear, so
    // the walk is the plain one and the arrival stand can fire.
    assert!(matches!(
        sim.units[u].orders.front().map(|o| o.body),
        Some(Body::Gather(g)) if g.goto_build
    ));
    assert_eq!(sim.units[u].carry, 0);
    assert_eq!(sim.gather_walk(u), None);
    assert_eq!(sim.walk_for(u), crate::anim::WALK);

    // One bit at a time, in the order the walk arm tests them.
    for (bit, slot) in [
        (orders::CARRY_WITH_WOOD, crate::anim::WALK_WITH_WOOD),
        (orders::CARRY_TO_WOOD, crate::anim::WALK_TO_WOOD),
        (orders::CARRY_WITH_ORE, crate::anim::WALK_WITH_ORE),
        (orders::CARRY_TO_ORE, crate::anim::WALK_TO_ORE),
    ] {
        sim.units[u].carry = bit;
        assert_eq!(sim.gather_walk(u), Some(slot));
    }
    // Two at once — the walk out is only ever cleared by the arrival, so a
    // worker that has been both carries both, and `WALK_TO_WOOD` is asked
    // for first.
    sim.units[u].carry = orders::CARRY_WITH_WOOD | orders::CARRY_TO_WOOD;
    assert_eq!(sim.gather_walk(u), Some(crate::anim::WALK_TO_WOOD));
    sim.units[u].carry = orders::CARRY_WITH_ORE | orders::CARRY_TO_ORE;
    assert_eq!(sim.gather_walk(u), Some(crate::anim::WALK_TO_ORE));

    // And the job's end takes them with it (`kill_current_order:107`).
    sim.units[u].carry = orders::CARRY_WITH_WOOD;
    sim.kill_current_order(u);
    assert_eq!(sim.units[u].carry, 0);
}

/// `Build::activate`'s tail (`docs/ECONOMY.md`, "What a finished gather
/// building pays"): the slots join the leader's count, and only the part of
/// them past the high-water mark is paid for. So the first farm of a game is
/// worth twenty food and a farm rebuilt where one was razed is worth nothing,
/// which is what run40 measures — the AI's fourth farm on frame 166 is the
/// twenty of the thirty-two it was short.
#[test]
fn a_finished_farm_pays_once_and_a_rebuilt_one_pays_nothing() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    // Frame 0 is the setup path and pays nothing whatever is finished on it.
    sim.frame = 1;
    for l in &mut sim.ledgers {
        l.gather_slots = [0; economy::RESOURCES];
        l.gather_slots_high = [0; economy::RESOURCES];
    }
    let food = |s: &Sim| s.ledgers[0].bucket[0];
    let bonus = sim.tuning.food_bonus_for_farm;

    let before = food(&sim);
    let farm = sim.place_building(0, t.farm, tile_pos(40, 32)).unwrap();
    finish(&mut sim, farm);
    assert_eq!(
        sim.ledgers[0].gather_slots[0], 1,
        "a flat type has one slot"
    );
    assert_eq!(sim.ledgers[0].gather_slots_high[0], 1);
    assert_eq!(food(&sim) - before, bonus, "FOOD_BONUS_FOR_FARM, once");

    // A second farm is a second slot, so it pays again.
    let before = food(&sim);
    let second = sim.place_building(0, t.farm, tile_pos(44, 36)).unwrap();
    finish(&mut sim, second);
    assert_eq!(sim.ledgers[0].gather_slots[0], 2);
    assert_eq!(food(&sim) - before, bonus);

    // Raze one and rebuild it: `Build::close` gives the slot back, the
    // high-water does not fall, and the rebuild is free of bonus.
    sim.disband_building(second, true);
    assert_eq!(sim.ledgers[0].gather_slots[0], 1, "the slot goes back");
    let before = food(&sim);
    let third = sim.place_building(0, t.farm, tile_pos(44, 36)).unwrap();
    finish(&mut sim, third);
    assert_eq!(sim.ledgers[0].gather_slots[0], 2);
    assert_eq!(
        sim.ledgers[0].gather_slots_high[0], 2,
        "the mark does not fall"
    );
    assert_eq!(
        food(&sim) - before,
        0,
        "a slot the player has held before pays nothing"
    );
}

/// **A nation power needs "a starting town or a city"** (twenty-fourth
/// pass, group 22): `has_tribe_bonus@006e1370` returns 0 when the lobby's
/// starting town is nomad *and* `city_num == 0`. The lobby's starting town
/// alone opens the gate; in a nomad lobby the first city founded opens it and
/// the last one closed shuts it, each recomputing the cached flags.
#[test]
fn a_nomad_has_no_power_between_its_cities() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    sim.lobby.starting_town = 0;
    sim.sync_setup_from_lobby();
    assert!(!sim.setup.starting_town);
    assert!(!sim.tech[0].has_city, "no city yet");
    sim.set_tribe(0, 11);
    assert!(!sim.nation[0].british, "a nomad with no city: no power");
    let (_, village) = city_at(&mut sim, &t, 0, 32, 32);
    assert!(sim.nation[0].british, "the first city opens the gate");
    sim.close_building(village, false);
    assert!(!sim.nation[0].british, "the last city closed shuts it");
    // And a starting town is the other half: no city, still a power.
    sim.lobby.starting_town = 2;
    sim.sync_setup_from_lobby();
    sim.refresh_nation_powers(0);
    assert!(sim.nation[0].british);
}

/// **A Barracks-trained unit garrisons in a Siege Factory or a Factory**
/// (twenty-fourth pass, group 3): above patch version 3 the `WHERE ==
/// BARRACKS` arm of `can_garrison` admits Barracks, Stable, Auto Plant, Siege
/// Factory and Factory (`61d8d7..61d978`), not the first three only.
#[test]
fn a_barracks_unit_garrisons_in_a_siege_factory_and_a_factory() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let sf = sim.add_build_type(bt(Ident::SiegeFactory, None, "ean", 4, 4, 420, 1200, 10));
    let fac = sim.add_build_type(bt(Ident::Factory, None, "ean", 4, 4, 420, 1200, 10));
    let ut = sim.add_unit_type(hoplite_type(t.barracks));
    assert!(sim.can_garrison(ut, t.barracks));
    assert!(sim.can_garrison(ut, sf), "a Siege Factory takes it");
    assert!(sim.can_garrison(ut, fac), "and so does a Factory");
    assert!(!sim.can_garrison(ut, t.market), "a market still does not");
}

/// **The British arm of `train_time`'s tail** — `docs/PRODUCTION.md`,
/// "The tail's first caller".
///
/// Three tests inside one `has_tribe_bonus(0xb)`, and each is its own
/// `t * 100 / (K + 100)`: the sea domain at `BRITISH_SHIP_SPEED`, the
/// Bowmen root at `BRITISH_ARCHER_SPEED`, the anti-air root at
/// `BRITISH_AA_SPEED`. The shipped file makes the middle one **0**, so the
/// arm is live and inert at once — the assertion below is that a British
/// archer costs exactly what everyone else's does, which is the sort of
/// thing an implementation gets wrong by leaving the test out and looking
/// right.
///
/// The numbers are run58's own: a Fisherman is `JOB_TIME 94`, so the base
/// is 9,400, `UNIT_RATE_BASE` is 120 and the ramp's first step puts a
/// British player's target at **11,280 × 100 / 133 = 8,481** — the value
/// the AI's Dock `1/2010` caps at on frame 4461.
#[test]
fn the_british_ship_bonus_is_a_third_off_the_clock() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    // A Fisherman as the shipped row has it, and two land types beside it
    // that differ only in which lineage bit they carry.
    let mut fisher = citizen_type(t.barracks);
    fisher.times.job_time = 94;
    fisher.combat.domain = attrition::Domain::Sea;
    let fisher = sim.add_unit_type(fisher);
    let mut archer = citizen_type(t.barracks);
    archer.times.job_time = 94;
    archer.archer = true;
    let archer = sim.add_unit_type(archer);
    let mut aa = citizen_type(t.barracks);
    aa.times.job_time = 94;
    aa.anti_air = true;
    let aa = sim.add_unit_type(aa);

    // The nation power needs a city — `has_tribe_bonus`' own gate — and the
    // three types are made at the barracks beside it.
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.init_build(0, t.barracks, tile_pos(38, 32), false);
    finish(&mut sim, b);
    // Train jobs, not research ones: the availability bit is what puts a
    // type on the national side of the tail's partition at all.
    for ty in [fisher, archer, aa] {
        sim.muster[0].researched[ty] = true;
    }
    // Each type is read on its own and put back: the target is recomputed
    // every frame, and the ramp has to see nothing of the type owned.
    let target = |sim: &mut Sim, ty: usize| {
        let slot = sim.queue_up(b, ty).expect("the barracks takes the order");
        let t = sim.queue_target(b, slot);
        sim.cancel(b, slot);
        t
    };

    // No nation: `JOB_TIME × 100`, through `UNIT_RATE_BASE` and a ramp with
    // nothing owned. All three types are the same row but for one bit.
    let plain = target(&mut sim, fisher);
    assert_eq!(plain, 11_280, "94 × 100 × UNIT_RATE_BASE / 100");
    assert_eq!(target(&mut sim, archer), plain);
    assert_eq!(target(&mut sim, aa), plain);

    // `tribe 11` is the British, and it is player 0 that gets it.
    sim.set_tribe(0, 11);
    assert!(
        sim.nation[0].british,
        "the power needs a city, and there is one"
    );
    assert_eq!(
        target(&mut sim, fisher),
        8_481,
        "11280 × 100 / 133 — run58's frame 4461, to the hundredth"
    );
    assert_eq!(
        target(&mut sim, archer),
        plain,
        "BRITISH_ARCHER_SPEED ships as 0: the arm runs and changes nothing"
    );
    assert_eq!(
        target(&mut sim, aa),
        8_481,
        "BRITISH_AA_SPEED is 33 like the ship one"
    );

    // And it is the *nation*, not the player: nobody else is sped up.
    sim.set_tribe(0, 2);
    assert_eq!(
        target(&mut sim, fisher),
        plain,
        "the Inca build ships at cost"
    );
}

/// **The speed-upgrade step of `train_time`'s tail** — `docs/PRODUCTION.md`,
/// "The tail's first caller". `t = (10 − n) × t / 10` (`0065102d`), `n`
/// counted on **one** ladder: the ships' for a sea type, else the troops'
/// for `obj_masks` `F` or `M`, else the vehicles' for `V`, else none.
///
/// The numbers are run547's: who=1's Citizen at `1/2022`, `JOB_TIME 50`
/// and 38 owned, sits at the ramp's ceiling, 18,000; Herbal Lore
/// (`TROOPS_FASTER_1`'s prerequisite) takes it to **16,200**, the counter
/// the original trains `1/89` on at frame 10144.
#[test]
fn a_speed_upgrade_takes_a_tenth_per_level_off_its_own_ladder_only() {
    use crate::combat::mask;
    use crate::tech::{TechTree, TypeDef};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new().with_tuning(&Tuning::RON);
    let herbal = tree.add(TypeDef::plain("Herbal Lore", 0));
    let medicine = tree.add(TypeDef::plain("Medicine", 0));
    let forage = tree.add(TypeDef::plain("Forage", 0));
    tree.roles.troops_speed_preq = [Some(herbal), Some(medicine), None];
    tree.roles.ships_speed_preq = [Some(forage), None, None];
    tree.roles.vehicles_speed_preq = [Some(forage), None, None];
    sim.set_tech_tree(tree);
    let typed = |sim: &mut Sim, masks: u32, sea: bool| {
        let mut u = citizen_type(t.barracks);
        u.times.job_time = 50;
        u.times.job_extra_time = 10;
        u.times.research_premium_time = 2;
        u.combat.obj_masks = masks;
        if sea {
            u.combat.domain = attrition::Domain::Sea;
        }
        sim.add_unit_type(u)
    };
    // The Citizen's `FCWP`, a horseman's `M`, a tank's `V`, a boat that
    // carries `F` as well, and a type with none of the three.
    let citizen = typed(&mut sim, mask::FOOT | mask::CIVILIAN, false);
    let rider = typed(&mut sim, mask::MOUNTED, false);
    let tank = typed(&mut sim, mask::VEHICLE, false);
    let boat = typed(&mut sim, mask::FOOT, true);
    let other = typed(&mut sim, mask::SIEGE, false);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.init_build(0, t.barracks, tile_pos(38, 32), false);
    finish(&mut sim, b);
    let all = [citizen, rider, tank, boat, other];
    for ty in all {
        sim.muster[0].researched[ty] = true;
        sim.muster[0].by_type[ty] = 38;
    }
    let target = |sim: &mut Sim, ty: usize| {
        let slot = sim.queue_up(b, ty).expect("the barracks takes the order");
        let t = sim.queue_target(b, slot);
        sim.cancel(b, slot);
        t
    };
    let targets = |sim: &mut Sim| all.map(|ty| target(sim, ty));

    assert_eq!(
        targets(&mut sim),
        [18_000; 5],
        "the ramp's ceiling, 3 × 6000"
    );
    sim.tech[0].tech[herbal] = true;
    assert_eq!(
        targets(&mut sim),
        [16_200, 16_200, 18_000, 18_000, 18_000],
        "TROOPS_FASTER_1: foot and mounted, and not a boat that is foot too"
    );
    sim.tech[0].tech[medicine] = true;
    assert_eq!(
        targets(&mut sim),
        [14_400, 14_400, 18_000, 18_000, 18_000],
        "every level held is counted"
    );
    sim.tech[0].tech[forage] = true;
    assert_eq!(
        targets(&mut sim),
        [14_400, 14_400, 16_200, 16_200, 18_000],
        "the ships' and the vehicles' ladders"
    );
    // A research job — the availability bit clear — never reaches it: the
    // step sits inside the availability branch.
    sim.muster[0].researched[citizen] = false;
    let held = target(&mut sim, citizen);
    for t in [herbal, medicine, forage] {
        sim.tech[0].tech[t] = false;
    }
    assert_eq!(
        target(&mut sim, citizen),
        held,
        "the research half is untouched"
    );
}

/// **A dock's own warships are born off its bad water** (`docs/ORDERS.md`
/// §25). `BuildType::mask_me`'s `is(DOCK)` arm marks every ocean tile of the
/// footprint grown by three `BAD_PATH`, and a sea type with an attack may
/// neither be placed on (`find_nearby_spot`'s `0x2400` test) nor stand on
/// (`invalid_loc` answers 3) such a tile. A boat with no attack — a
/// fishing boat — is refused neither. East Indies' Trireme `1/32` is the
/// capture: due east of Dock `1/2010`, where the ring's south is the margin.
#[test]
fn a_warship_keeps_off_its_dock_s_margin_and_a_fishing_boat_does_not() {
    let mut sim = world_sim();
    // Ocean from tile column 30 eastward.
    for tx in 30..64 {
        for ty in 0..64 {
            sim.world
                .set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
        }
    }
    let dock_ty = sim.add_build_type(bt(Ident::Dock, None, "ean", 4, 4, 420, 2400, 0));
    let dock = sim.add_building(1, Pos::new(30 * TILE, 32 * TILE), 8);
    sim.buildings[dock].ty = Some(dock_ty);
    assert_eq!(
        sim.tile_corner(dock_ty, sim.buildings[dock].pos),
        Pos::new(28, 30)
    );
    sim.mask_dock_water(dock, true);
    let bad =
        |sim: &Sim, tx: i32, ty: i32| sim.world.tile_mask(Pos::new(tx, ty)) & tile::BAD_PATH != 0;
    // The margin is [corner − 3, corner + size + 3) on both axes, ocean only.
    assert!(bad(&sim, 34, 30) && bad(&sim, 30, 36) && bad(&sim, 30, 27));
    assert!(!bad(&sim, 35, 30) && !bad(&sim, 30, 37) && !bad(&sim, 30, 26));
    assert!(!bad(&sim, 27, 32), "land takes no mark");
    let sea_type = |attack: i32| UnitType {
        hits: 100,
        combat: combat::Profile {
            attack,
            domain: crate::attrition::Domain::Sea,
            block_radius: 48,
            big_radius: 48,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    let trireme = sim.add_unit_type(sea_type(200));
    let fishing = sim.add_unit_type(sea_type(0));
    let centre = sim.buildings[dock].pos;
    let south = crate::movement::Angle(i32::MIN);
    // The fishing boat takes the ring's first bearing, due south and inside
    // the margin; the warship's first free candidate is past it.
    let f = sim
        .find_nearby_spot_type(fishing, centre, 600, 1200, 0, south)
        .expect("a fishing boat finds water");
    assert_eq!(f, Pos::new(centre.x + 24, centre.y + 600 + 24 - 600 % 48));
    assert!(bad(&sim, f.tile().x, f.tile().y));
    let w = sim
        .find_nearby_spot_type(trireme, centre, 600, 1200, 0, south)
        .expect("a warship finds water");
    assert!(
        !bad(&sim, w.tile().x, w.tile().y),
        "the warship's spot {w:?} is in the margin"
    );
    // And standing: the hazard, on the margin and not past it.
    let u = sim.init_unit(1, trireme, w);
    assert_eq!(
        sim.invalid_loc(u, Pos::new(32, 35), false, false, false, false, false),
        3
    );
    assert_eq!(
        sim.invalid_loc(u, Pos::new(36, 32), false, false, false, false, false),
        0
    );
    let b = sim.init_unit(1, fishing, f);
    assert_eq!(
        sim.invalid_loc(b, Pos::new(32, 35), false, false, false, false, false),
        0
    );
    // The unmask clears it.
    sim.mask_dock_water(dock, false);
    assert!(!bad(&sim, 34, 30));
}

/// **A Senate finished in a city of the owner's own race moves the capital
/// there** when the capital holds none (`docs/CITIES.md` §15,
/// `Build::activate@00623e20`'s Senate arm). Great Lakes' Senate `1/2024`
/// moves who=1's capital from London to Norwich on tick 14528, and a
/// second Senate in the old capital does not move it back: the new
/// capital holds one.
///
/// Made to fail once with the arm's call removed: the capital stays put.
#[test]
fn a_senate_moves_the_capital_to_its_city_unless_the_capital_has_one() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (_, a) = city_at(&mut sim, &t, 0, 32, 32);
    sim.tech[0].epoch[tech::Line::Civic as usize] = 1;
    let (_, b) = city_at(&mut sim, &t, 0, 57, 32);
    assert!(sim.cities[a].capital && !sim.cities[b].capital);
    let s = sim
        .place_building(0, t.senate, tile_pos(57, 42))
        .unwrap_or_else(|e| panic!("the senate should place: {e:?}"));
    assert_eq!(sim.buildings[s].city, Some(b));
    finish(&mut sim, s);
    assert!(
        sim.cities[b].capital,
        "the capital moves to the Senate's city"
    );
    assert!(!sim.cities[a].capital, "and leaves the old one");
    assert!(
        sim.cities[a].founding_capital,
        "the founding mark is the first city's for good"
    );
    let s2 = sim
        .place_building(0, t.senate, tile_pos(32, 42))
        .unwrap_or_else(|e| panic!("the second senate should place: {e:?}"));
    finish(&mut sim, s2);
    assert!(
        sim.cities[b].capital && !sim.cities[a].capital,
        "a capital that holds a Senate keeps the flag"
    );
}

/// **The Persians' Market trains a caravan on its fifteen-frame slot**
/// (`Build::process@0061edf0:362–381`, `docs/AI.md` §146): a finished
/// Market of a leader with `has_tribe_bonus(0x17)` trains a `CARA` through
/// `Build::train` — no queue, no price — on `(o + frame) % 15 == 0`, while
/// `caras` is under `get_caravan_limit(1)` (here one: two cities make one
/// pairing). The second slot trains nothing, because the first caravan is
/// already counted; a British Market trains nothing at all. The coverage
/// pair's Persians: the Market `1/2018`, finished on 661, trains `1/15` on
/// 667.
///
/// Made to fail by dropping the tick's call: no caravan is ever born.
#[test]
fn a_persian_market_trains_a_caravan_on_its_slot_up_to_the_limit() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let caravan = sim.add_unit_type(UnitType {
        price: cost::Price {
            pop: 1,
            ..cost::Price::free().with_base(economy::Resource::Wealth, 100)
        },
        hits: 40,
        ..UnitType::default()
    });
    // `init_unit` reads the type's kind off the tree, so the tree reaches
    // `CARA`'s own number.
    while sim.tech_tree.types.len() <= crate::nations::ty::CARA {
        sim.tech_tree
            .add(tech::TypeDef::unit("Caravan", tech::UnitTraits::default()));
    }
    sim.unit_types[caravan].tree = Some(crate::nations::ty::CARA);
    sim.tech[0].epoch[tech::Line::Civic as usize] = 1;
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let _ = city_at(&mut sim, &t, 0, 57, 32);
    let m = sim.init_build(0, t.market, tile_pos(38, 32), false);
    finish(&mut sim, m);
    let caravans = |sim: &Sim| {
        (0..sim.units.len())
            .filter(|&u| sim.units[u].alive() && sim.units[u].ty == Some(caravan))
            .count()
    };
    // The British first: the slot comes and goes, and nothing is born.
    sim.set_tribe(0, 11);
    for _ in 0..15 {
        sim.tick();
    }
    assert_eq!(caravans(&sim), 0, "only the Persians' Market trains one");
    sim.set_tribe(0, 23);
    assert_eq!(sim.caravan_limit(0), 1, "two cities, one pairing");
    let o = i64::from(sim.buildings[m].index);
    let mut born = None;
    for _ in 0..15 {
        let f = sim.frame;
        sim.tick();
        if born.is_none() && caravans(&sim) == 1 {
            born = Some(f);
        }
    }
    let f = born.expect("a Persian Market trains a caravan within fifteen frames");
    assert_eq!((o + f) % 15, 0, "on the Market's own slot");
    assert_eq!(
        sim.ai[0].census.caras, 1,
        "and the caravan is counted at birth"
    );
    for _ in 0..30 {
        sim.tick();
    }
    assert_eq!(
        caravans(&sim),
        1,
        "at the limit, the next slots train nothing"
    );
}

/// **A Senate that finishes a government trains its patriot, once**
/// (`Build::finished@00628490`'s tail, `docs/TECH.md` §"The government
/// patriot"). `get_gov` reads the government bonus's prerequisite,
/// `get_gov_hero` finds the patriot that names it, `Build::train` places it
/// at the Senate with no queue, and `Unit::init` stamps `gov_hero_frame`,
/// which is what stops the second government from training another — the
/// original `set_type`s the standing one instead, a seam here. Great Lakes'
/// Despot `1/79` on tick 14982 and East Indies' Senator `1/60` on 15782.
#[test]
fn a_senate_that_finishes_a_government_trains_its_patriot_once() {
    use crate::tech::{Preq, TechTree, TypeDef, UnitTraits as Traits};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let senate_t = tree.add(TypeDef::building("Senate"));
    let despotism = tree.add(TypeDef::gov("Despotism", 0, 0).at(senate_t));
    let monarchy = tree.add(TypeDef::gov("Monarchy", 1, 0).at(senate_t));
    let patriot = Traits {
        patriot: true,
        hero: true,
        ..Traits::default()
    };
    let despot_t = tree.add(TypeDef::unit("The Despot", patriot).needs(0, despotism));
    let monarch_t = tree.add(TypeDef::unit("The Monarch", patriot).needs(0, monarchy));
    // `get_gov`'s own order: Monarchy is tested ahead of Despotism.
    tree.roles.gov_bonuses = vec![
        ([Preq::Of(monarchy), Preq::None, Preq::None], monarchy),
        ([Preq::Of(despotism), Preq::None, Preq::None], despotism),
    ];
    sim.set_tech_tree(tree);
    sim.build_types[t.senate].tree = Some(senate_t);
    let hero = |tt| UnitType {
        tree: Some(tt),
        hits: 109,
        ..UnitType::default()
    };
    let despot = sim.add_unit_type(hero(despot_t));
    let monarch = sim.add_unit_type(hero(monarch_t));
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let senate = sim.init_build(0, t.senate, tile_pos(36, 32), false);
    finish(&mut sim, senate);
    assert_eq!(sim.tech[0].gov_hero_frame, -1);
    // Off frame 0: a government held at frame 0 counts as its patriot
    // born (`gain_tech:161`), and that is a different rule.
    for _ in 0..5 {
        sim.tick();
    }
    let despots = |sim: &Sim| {
        (0..sim.units.len())
            .filter(|&u| {
                sim.units[u].alive()
                    && matches!(sim.units[u].ty, Some(x) if x == despot || x == monarch)
            })
            .count()
    };

    assert_eq!(sim.queue_tech(senate, despotism), Ok(0));
    let mut frames = 0;
    while !sim.tech[0].tech[despotism] {
        sim.tick();
        frames += 1;
        assert!(frames < 600);
    }
    assert_eq!(despots(&sim), 1, "the Senate trains the Despot");
    assert_eq!(
        sim.tech_tree.get_gov_hero(&sim.setup, &sim.tech[0]),
        Some(despot_t)
    );
    assert!(sim.tech[0].gov_hero_frame >= 0, "and its birth is stamped");

    assert_eq!(sim.queue_tech(senate, monarchy), Ok(0));
    while !sim.tech[0].tech[monarchy] {
        sim.tick();
        frames += 1;
        assert!(frames < 1200);
    }
    assert_eq!(
        despots(&sim),
        1,
        "a patriot already born is not trained again"
    );
    assert_eq!(
        sim.tech_tree.get_gov_hero(&sim.setup, &sim.tech[0]),
        Some(monarch_t),
        "get_gov answers the first bonus in its own order"
    );
    // **The other arm** (item 1416): the standing patriot is `set_type`d
    // to the new government's, where the original's `find_unit` finds it.
    let standing: Vec<_> = (0..sim.units.len())
        .filter(|&u| sim.units[u].alive() && sim.units[u].owner == 0)
        .filter_map(|u| sim.units[u].ty)
        .collect();
    assert_eq!(standing, [monarch], "the Despot became the Monarch");
    sim.tech[0].no_patriots = true;
    assert_eq!(sim.tech_tree.get_gov_hero(&sim.setup, &sim.tech[0]), None);
}

/// **`Wall::process`'s oil-platform arm** (`docs/AI.md` §162, item 1588).
/// On `(frame + o) & 0x7f == 0`, a computer leader's unfinished Oil Platform
/// that no unit of its owner holds as its **action** is disbanded, its price
/// refunded; a citizen with the site queued beneath another build does not
/// keep it. run679's `1/2037` goes on 1419 with `1/10` on `Build 2026` and
/// `Build 2037` two orders down.
#[test]
fn an_oil_platform_site_nobody_is_building_is_disbanded_on_its_128_frame_phase() {
    let mut sim = world_sim();
    sim.tuning.city_gather = [0; 6];
    let t = install_types(&mut sim);
    sim.nation[0].human = false;
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let oil = sim.add_build_type(bt(Ident::OilPlatform, None, "", 2, 2, 2000, 2000, 0));
    // A site placed as a Barracks and then made an Oil Platform: the arm
    // reads the type, and the placement rules for water are not its test.
    let site = sim.place_building(0, t.barracks, tile_pos(44, 32)).unwrap();
    sim.buildings[site].ty = Some(oil);
    let other = sim.place_building(0, t.barracks, tile_pos(44, 44)).unwrap();
    let u = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    let due = |sim: &Sim, b: usize, after: i64| -> i64 {
        (after..)
            .find(|f| sim.buildings[b].phase(*f) & 0x7f == 0)
            .unwrap()
    };

    // 1. The citizen's action is the site: it stands.
    sim.add_build_order(u, site, QueuePos::New, true);
    let f = due(&sim, site, 200);
    sim.process_building(site, f);
    assert!(sim.buildings[site].alive, "its builder holds it");

    // 2. Another build on top: the site is two orders down, and only on the
    //    128-frame phase — not on a 32-frame one — is it disbanded.
    sim.add_build_order(u, other, QueuePos::First, true);
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| o.body == Body::Build(site)),
        "the site is still in the stack"
    );
    let f = due(&sim, site, f + 1);
    sim.process_building(site, f - 32);
    assert!(
        sim.buildings[site].alive,
        "a 32-frame phase is not the arm's"
    );
    let before = sim.ledgers[0].bucket[1];
    assert!(!sim.process_building(site, f), "Wall::process returns");
    assert!(!sim.buildings[site].alive, "nobody's action is the site");
    assert_eq!(
        sim.ledgers[0].bucket[1],
        before + 100,
        "the whole price back"
    );
    assert!(
        sim.buildings[other].alive,
        "a Barracks site is not the arm's"
    );

    // 3. A human's site is never the arm's.
    let site = sim.place_building(0, t.barracks, tile_pos(20, 44)).unwrap();
    sim.buildings[site].ty = Some(oil);
    sim.nation[0].human = true;
    let f = due(&sim, site, f + 1);
    sim.process_building(site, f);
    assert!(sim.buildings[site].alive, "the gate is the owner's");
}

/// **`Wall::process`'s site recruiter** (`docs/AI.md` §69, item 715). On
/// its `(frame + o) & 31` phase an unfinished wonder or fort of a computer
/// leader wants `max(4, helpers)` builders, and while it is short it hands
/// the **nearest** own citizen that is not busy — idle, gathering or on a
/// plain move — `add_build_order(site, QUEUE_NEW, 0)`: a build order with
/// no action bit. Great Lakes' Pyramids `1/2026` does it on 15382 for
/// `1/70`, whose swarm `do_move` had just refused.
#[test]
fn an_unfinished_wonder_calls_in_the_nearest_citizen_that_is_not_busy() {
    use crate::tech::{TechTree, TypeDef};
    struct Ground {
        sim: Sim,
        t: Types,
        citizen: usize,
        wonder: usize,
    }
    fn ground(human: bool) -> Ground {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        sim.nation[0].human = human;
        sim.lobby.starting_resources = 1;
        let mut tree = TechTree::new();
        // The Citizen is the tree's first unit — `types.list[BASE_UNITTYPES]`,
        // what `FILTER_TYPE 0x32` asks a candidate to be.
        let root = tree.add(TypeDef::unit(
            "Citizen",
            crate::tech::UnitTraits {
                free: false,
                jumpable: false,
                unique: false,
                hero: false,
                patriot: false,
                combat: false,
            },
        ));
        for name in ["Food", "Timber", "Metal", "Wealth", "Knowledge", "Oil"] {
            tree.add(TypeDef::good(name));
        }
        sim.set_tech_tree(tree);
        let _ = city_at(&mut sim, &t, 0, 32, 32);
        let citizen = sim.add_unit_type(citizen_type(t.village));
        sim.unit_types[citizen].worker = Worker::Citizen;
        sim.unit_types[citizen].tree = Some(root);
        let mut wonder_type = bt(Ident::Wonder, None, "ean", 4, 4, 2000, 2000, 0);
        // The loader sets the range flag with the ident; the recruiter reads
        // the flag (`BuildData::is_wonder`).
        wonder_type.wonder = true;
        let wonder = sim.add_build_type(wonder_type);
        Ground {
            sim,
            t,
            citizen,
            wonder,
        }
    }
    /// The first frame past `after` on which building `b`'s phase is due.
    fn due(sim: &Sim, b: usize, after: i64) -> i64 {
        (after..)
            .find(|f| sim.buildings[b].phase(*f) & 31 == 0)
            .unwrap()
    }
    let held = |sim: &Sim, u: usize| -> Vec<(Body, u8)> {
        sim.units[u]
            .orders
            .iter()
            .map(|o| (o.body, o.flags))
            .collect()
    };

    // 1. The recruit: the nearer of two idle citizens, on the phase frame
    //    and not a frame before, with `QUEUE_NEW` and no action bit.
    let Ground {
        mut sim,
        citizen,
        wonder,
        ..
    } = ground(false);
    let site = sim.place_building(0, wonder, tile_pos(44, 32)).unwrap();
    assert!(!sim.buildings[site].active);
    let far = spawn(&mut sim, 0, citizen, tile_pos(52, 32));
    let near = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    let f = due(&sim, site, 100);
    sim.process_building(site, f - 1);
    assert!(
        held(&sim, near).is_empty(),
        "off the phase, nobody is called"
    );
    sim.process_building(site, f);
    assert_eq!(
        held(&sim, near),
        [(Body::Build(site), 0)],
        "the nearer is sent"
    );
    assert!(held(&sim, far).is_empty(), "one citizen a phase");
    // The next phase takes the other: `helpers` is still short of four.
    let f2 = due(&sim, site, f + 1);
    sim.process_building(site, f2);
    assert_eq!(held(&sim, far), [(Body::Build(site), 0)]);

    // 2. A gatherer is not busy, and a builder is: `FILTER_NOT_BUSY` is
    //    `action_type` ∈ {NONE, GATHER, MOVE_TO}.
    let Ground {
        mut sim,
        t,
        citizen,
        wonder,
    } = ground(false);
    let site = sim.place_building(0, wonder, tile_pos(44, 32)).unwrap();
    let farm = sim.place_building(0, t.farm, tile_pos(40, 44)).unwrap();
    let builder = spawn(&mut sim, 0, citizen, tile_pos(42, 32));
    sim.add_build_order(builder, farm, QueuePos::New, false);
    let camp = sim.place_building(0, t.farm, tile_pos(46, 40)).unwrap();
    let gatherer = spawn(&mut sim, 0, citizen, tile_pos(48, 32));
    sim.add_gather_order(gatherer, camp, QueuePos::New, false);
    assert!(
        sim.units[gatherer]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Gather(_))),
        "the fixture's gatherer gathers"
    );
    let f = due(&sim, site, 100);
    sim.process_building(site, f);
    assert_eq!(
        held(&sim, gatherer),
        [(Body::Build(site), 0)],
        "the gatherer is taken"
    );
    assert!(
        held(&sim, builder)
            .iter()
            .all(|(b, _)| *b != Body::Build(site)),
        "the nearer builder is busy"
    );

    // 3. Four builders already at it last frame: `helpers` is read before
    //    the reset, and four is enough.
    let Ground {
        mut sim,
        citizen,
        wonder,
        ..
    } = ground(false);
    let site = sim.place_building(0, wonder, tile_pos(44, 32)).unwrap();
    let idle = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    sim.buildings[site].helpers = 4;
    let f = due(&sim, site, 100);
    sim.process_building(site, f);
    assert!(held(&sim, idle).is_empty(), "four helpers want nobody");
    assert_eq!(sim.buildings[site].helpers, 0, "and the reset comes after");

    // 4. Neither a farm site nor a human's wonder recruits.
    let Ground {
        mut sim,
        t,
        citizen,
        ..
    } = ground(false);
    let farm = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
    let idle = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    let f = due(&sim, farm, 100);
    sim.process_building(farm, f);
    assert!(
        held(&sim, idle).is_empty(),
        "a farm is neither wonder nor fort"
    );
    let Ground {
        mut sim,
        citizen,
        wonder,
        ..
    } = ground(true);
    let site = sim.place_building(0, wonder, tile_pos(44, 32)).unwrap();
    let idle = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    let f = due(&sim, site, 100);
    sim.process_building(site, f);
    assert!(held(&sim, idle).is_empty(), "a human's site calls nobody");

    // 5. A fort recruits too, while undamaged.
    let Ground {
        mut sim,
        t,
        citizen,
        ..
    } = ground(false);
    let fort = sim.place_building(0, t.fort, tile_pos(44, 32)).unwrap();
    let idle = spawn(&mut sim, 0, citizen, tile_pos(40, 32));
    sim.buildings[fort].damage = 1;
    let f = due(&sim, fort, 100);
    sim.process_building(fort, f);
    assert!(held(&sim, idle).is_empty(), "a damaged fort calls nobody");
    sim.buildings[fort].damage = 0;
    let f2 = due(&sim, fort, f + 1);
    sim.process_building(fort, f2);
    assert_eq!(held(&sim, idle), [(Body::Build(fort), 0)]);
}

/// A spy's craft table and a unit type that casts it, for `crate::cast`'s
/// tests: the Informer's row as `craftrules.xml` has it (`fbcml`, job 40,
/// ten tiles, `MANA 500`), cast by the returned type.
fn informer_sim() -> (Sim, usize, Types) {
    use crate::tech::{TechTree, TypeDef};
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let mut tree = TechTree::new();
    let spy_t = tree.add(TypeDef::unit("Spy", crate::tech::UnitTraits::default()));
    let barracks_t = tree.add(TypeDef::building("Barracks"));
    sim.set_tech_tree(tree);
    sim.build_types[t.barracks].tree = Some(barracks_t);
    let spy = sim.add_unit_type(UnitType {
        hits: 15,
        mana: 1000,
        tree: Some(spy_t),
        ..UnitType::default()
    });
    let mut rows = vec![crate::orders::SpellType::default(); 55];
    rows[(crate::orders::spell::INFORMER - crate::orders::spell::FIRST) as usize] =
        crate::orders::SpellType {
            job_time: 40,
            flags: 0x1826,
            range: 10 * 192,
            mana: 500,
            from: [Some(spy_t), None],
            ..Default::default()
        };
    sim.spells = rows;
    (sim, spy, t)
}

/// **A player's Informer on an enemy building walks to the ring, then
/// casts on its fortieth frame in range** (item 790, `docs/GOLDEN.md`
/// §27, run245). `Group::action_spell` lays one `CastOrder` with the
/// action bit; `do_cast`'s targeted half pays the mana once and, out of
/// range + radius, queues a `MOVE_TO` ahead of it to a ring spot on the
/// target; in range it holds the cast, the started bit set, and on the
/// fortieth frame the building's `infiltrated` takes the caster's player,
/// the order dies and the bit with it.
///
/// Made to fail once with the approach queued `QUEUE_NEW` (the cast is
/// lost), and once with the range taken whole on a building (the ring is
/// farther out than 1,344).
#[test]
fn a_spy_s_informer_walks_to_the_ring_then_casts_on_its_fortieth_frame() {
    use crate::combat::Obj;
    let (mut sim, spy_ty, t) = informer_sim();
    let _ = city_at(&mut sim, &t, 1, 44, 36);
    let b = sim
        .place_building(1, t.barracks, tile_pos(40, 30))
        .expect("the Barracks places");
    finish(&mut sim, b);
    let spy = spawn(&mut sim, 0, spy_ty, tile_pos(20, 30));
    sim.units[spy].mana_burn = 480;
    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, spy);
    assert!(sim.push_group(&mut g, true));
    let at = sim.buildings[b].pos;
    let laid = sim.group_action_spell(
        &g,
        crate::orders::spell::INFORMER,
        Some(Obj::Building(b)),
        at,
    );
    assert_eq!(laid, 1);
    {
        let o = &sim.units[spy].orders;
        assert_eq!(o.len(), 1, "one cast: {o:?}");
        assert!(
            matches!(o[0].body, Body::Cast(c) if !c.paid && c.target == Some(Obj::Building(b)))
        );
        assert!(o[0].has(crate::orders::flag::ACTION));
    }
    assert_eq!(sim.units[spy].cast_target, Some(Obj::Building(b)));
    sim.tick();
    {
        let o = &sim.units[spy].orders;
        assert_eq!(o.len(), 2, "the approach ahead of the cast: {o:?}");
        let Body::Move(m) = o[0].body else {
            panic!("the head is the approach: {o:?}")
        };
        assert_eq!(m.kind, MoveKind::MoveTo);
        let d = crate::world::vector_dist(m.dest.x - at.x, m.dest.y - at.y);
        assert!(d <= 960 + 384, "the spot is in range + radius: {d}");
        assert!(matches!(o[1].body, Body::Cast(c) if c.paid));
    }
    assert_eq!(sim.units[spy].mana_burn, 480 - 1 + 500, "paid in mana once");
    let mut first_in_range = None;
    for f in 0..600 {
        sim.tick();
        if first_in_range.is_none() && sim.units[spy].casting {
            first_in_range = Some(f);
        }
        if sim.buildings[b].infiltrated != 0 {
            let start = first_in_range.expect("the cast started before it landed");
            assert_eq!(f - start, 39, "the fortieth frame in range casts");
            break;
        }
    }
    assert_eq!(sim.buildings[b].infiltrated, 1, "who=0's informer is in");
    assert!(
        sim.units[spy].orders.is_empty(),
        "the order died with the cast"
    );
    assert!(!sim.units[spy].casting, "and the started bit with it");
    assert!(sim.units[spy].alive(), "the Informer costs the Spy nothing");
}

/// **The mana waits while the cast runs** (item 790, run245's 756–795):
/// `Unit::process` recovers one point a frame only while `unit_masks &
/// 0x2a000` is clear, and a targeted cast's started bit is one of the
/// three. Made to fail once with the recovery ignoring the bit.
#[test]
fn a_caster_s_mana_waits_while_its_cast_has_started() {
    let (mut sim, spy_ty, _) = informer_sim();
    let spy = spawn(&mut sim, 0, spy_ty, tile_pos(20, 30));
    sim.units[spy].mana_burn = 900;
    sim.tick();
    assert_eq!(sim.units[spy].mana_burn, 899, "one a frame when idle");
    sim.units[spy].casting = true;
    sim.tick();
    sim.tick();
    assert_eq!(
        sim.units[spy].mana_burn, 899,
        "none while the cast has started"
    );
    sim.units[spy].casting = false;
    sim.tick();
    assert_eq!(sim.units[spy].mana_burn, 898);
}

/// **A verified line reads the stack again** (`Unit::do_move@005f7b30`'s
/// TAKE, `5f8c3d`–`5f8c5d`; `docs/GROUPS.md` §32). A goal eight cells off
/// is planned on the world grid, and the straight line to the new top clips
/// a barracks: `find_path` pushes a detour and verifies it. The step must
/// walk at the detour — the top after the check — with its tolerance 0,
/// not at the world entry under it with 384. Great Lakes' `1/40` walked at
/// the world entry for seventeen frames without this (item 795).
#[test]
fn a_detour_the_line_check_pushes_is_the_waypoint() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    let _ = city_at(&mut sim, &t, 0, 24, 30);
    let b = sim
        .place_building(0, t.barracks, tile_pos(11, 20))
        .expect("open ground");
    finish(&mut sim, b);
    let u = spawn(&mut sim, 0, citizen, tile_pos(8, 21));
    // The order as the Great Lakes march held it on 19875: `PATHED`, only
    // its `FINAL` entry on the stack, and no waypoint — so the line check
    // on the far goal refuses, the grid roll plans, and TAKE takes the new
    // top (a first `do_move` without `PATHED` plans before the line check
    // and never reaches TAKE).
    let dest = tile_pos(40, 20);
    sim.order_move(u, dest);
    sim.units[u].orders[0].flags |= orders::flag::PATHED;
    sim.units[u].path.push(orders::PathData {
        to: dest,
        tolerance: 0,
        flags: orders::path_flag::FINAL,
    });
    sim.trace_phases = true;
    sim.tick();
    assert!(
        sim.phase_marks
            .iter()
            .any(|(l, _)| l == orders::SITE_MOVE_GRID),
        "the planner ran, through the grid roll"
    );
    let n = sim.units[u].path.len();
    assert!(n > 2, "a world plan: {:?}", sim.units[u].path);
    let top = sim.units[u].path[n - 1];
    let under = sim.units[u].path[n - 2];
    assert_eq!(
        (top.tolerance, under.tolerance),
        (0, 384),
        "a detour over a world entry: {:?}",
        sim.units[u].path
    );
    let Some(orders::Body::Move(m)) = sim.units[u].orders.front().map(|o| o.body) else {
        panic!("still moving");
    };
    assert_eq!(
        (m.waypoint, sim.units[u].tolerance),
        (top.to, 0),
        "the step walks at the detour"
    );
}

/// **The gather point's command** (item 928, `docs/GOLDEN.md` §39,
/// run312): `Group::action_gather_point` gives a training building one
/// point, replaced by the next press and appended under `add_to_end`; the
/// Clear empties it; a click on the building's own footprint is the
/// "inside" point (−1, −1, 0).
#[test]
fn a_rally_press_writes_one_point_replaced_by_the_next_and_the_clear_empties_it() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    let at = |x, y, action| crate::rally::GatherPoint {
        pos: Pos::new(x, y),
        action,
    };
    sim.action_gather_point(0, &[b], Pos::new(1344, 12096), 0, false);
    assert_eq!(sim.buildings[b].gather, vec![at(1344, 12096, 0)]);
    sim.action_gather_point(0, &[b], Pos::new(5000, 6000), 1, false);
    assert_eq!(sim.buildings[b].gather, vec![at(5000, 6000, 1)], "replaced");
    sim.action_gather_point(0, &[b], Pos::new(7000, 8000), 0, true);
    assert_eq!(
        sim.buildings[b].gather.len(),
        2,
        "appended under add_to_end"
    );
    sim.action_gather_point(0, &[b], Pos::new(-1, -1), 0, false);
    assert!(sim.buildings[b].gather.is_empty(), "the Clear");
    let own = sim.buildings[b].pos;
    sim.action_gather_point(0, &[b], own, 1, false);
    assert_eq!(sim.buildings[b].gather, vec![at(-1, -1, 0)], "on itself");
    assert!(sim.gather_inside(b));
}

/// A Barracks and a trained squad of three, for the rally tests below.
fn rally_barracks(sim: &mut Sim) -> (Types, usize, usize) {
    let t = install_types(sim);
    let _ = city_at(sim, &t, 0, 32, 32);
    let mut squad = hoplite_type(t.barracks);
    squad.combat.uber_size = 3;
    squad.combat.block_radius = 48;
    let squad = sim.add_unit_type(squad);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(sim, b);
    (t, b, squad)
}

/// **A squad trained under a ground point** (run312's `0/11`..`0/13` on
/// 856): it leaves on the bearing to the point, not due south, and each
/// member is sent there under one group move, `ATTACK_TO` for an armed
/// unit made at a Barracks. With no point it leaves south with no order.
#[test]
fn a_squad_trained_under_a_ground_point_leaves_toward_it_and_attack_moves_there_as_one_group() {
    let mut sim = world_sim();
    let (_, b, squad) = rally_barracks(&mut sim);
    let home = sim.buildings[b].pos;
    let plain = sim.build_train(b, squad).unit;
    assert!(
        sim.units[plain].pos.y > home.y,
        "no point: out on the south"
    );
    assert!(sim.units[plain].orders.is_empty());

    let point = Pos::new(home.x - 2000, home.y - 2000);
    sim.action_gather_point(0, &[b], point, 0, false);
    let cap = sim.build_train(b, squad).unit;
    let p = sim.units[cap].pos;
    assert!(p.x < home.x && p.y < home.y, "out on the north-west: {p:?}");
    let mut ids = Vec::new();
    for m in sim.squad_members(cap) {
        let Some(crate::orders::Order {
            body: crate::orders::Body::Move(mv),
            ..
        }) = sim.units[m].orders.front()
        else {
            panic!("member {m} has no move: {:?}", sim.units[m].orders);
        };
        assert_eq!(mv.kind, crate::orders::MoveKind::AttackTo);
        ids.push(mv.group.map(|g| g.id));
    }
    assert!(
        ids[0].is_some() && ids.iter().all(|&i| i == ids[0]),
        "one group: {ids:?}"
    );
}

/// **A unit trained under its trainer's own point stays in** (run312's
/// `0/14`..`0/16`, inside `0/2008` from 953).
#[test]
fn a_squad_trained_under_its_trainer_s_own_point_stays_inside() {
    let mut sim = world_sim();
    let (_, b, squad) = rally_barracks(&mut sim);
    let own = sim.buildings[b].pos;
    sim.action_gather_point(0, &[b], own, 1, false);
    let cap = sim.build_train(b, squad).unit;
    assert_eq!(sim.units[cap].inside, Some(b));
    assert!(sim.units[cap].orders.is_empty());
}

/// **A squad trained under a friendly building's point garrisons it**
/// (run312's Bowmen into `0/2008`, a `GARRISONORDER` each on 1060).
#[test]
fn a_squad_trained_under_a_friendly_building_s_point_garrisons_it() {
    let mut sim = world_sim();
    let (t, b, squad) = rally_barracks(&mut sim);
    let other = sim.place_building(0, t.barracks, tile_pos(48, 40)).unwrap();
    finish(&mut sim, other);
    let there = sim.buildings[other].pos;
    sim.action_gather_point(0, &[b], there, 1, false);
    let cap = sim.build_train(b, squad).unit;
    for m in sim.squad_members(cap) {
        assert!(
            matches!(
                sim.units[m].orders.front().map(|o| &o.body),
                Some(crate::orders::Body::Garrison { building, .. }) if *building == other
            ),
            "member {m}: {:?}",
            sim.units[m].orders
        );
    }
}

/// **A lone unit not made at a Barracks, Stable or Dock walks to its
/// point with a plain move** (run312's Citizen `0/10` on 760, a
/// `MOVEORDER` to the Woodcutter's point and no gather order).
#[test]
fn a_lone_unit_trained_under_a_ground_point_walks_there_with_a_plain_move() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (city, _) = city_at(&mut sim, &t, 0, 32, 32);
    let walker = sim.add_unit_type(hoplite_type(t.village));
    let home = sim.buildings[city].pos;
    let point = Pos::new(home.x + 1500, home.y - 1500);
    sim.action_gather_point(0, &[city], point, 0, false);
    let u = sim.build_train(city, walker).unit;
    let Some(crate::orders::Order {
        body: crate::orders::Body::Move(mv),
        ..
    }) = sim.units[u].orders.front()
    else {
        panic!("no move: {:?}", sim.units[u].orders);
    };
    assert_eq!(mv.kind, crate::orders::MoveKind::MoveTo);
    assert!(mv.group.is_none());
    let p = sim.units[u].pos;
    assert!(p.x > home.x && p.y < home.y, "out on the north-east: {p:?}");
}

/// **A City's click on a forest takes its Woodcutter's point** (run312's
/// `0/2000` on 652: (3936, 28128) pressed, (4224, 28608) stored). Only a
/// group of City centres snaps, only within `0x600`, and the action is
/// kept.
#[test]
fn a_city_s_rally_on_a_forest_takes_its_woodcutter_s_point() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (city, _) = city_at(&mut sim, &t, 0, 32, 32);
    let camp_t = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 4, 4, 150, 400, 0));
    // Placed by hand: `place_building` wants forest under a camp.
    let camp = sim.add_building(0, tile_pos(44, 32), 2);
    sim.buildings[camp].ty = Some(camp_t);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 48)).unwrap();
    finish(&mut sim, b);
    let camp_at = sim.buildings[camp].pos;
    let forest = tile_pos(46, 32);
    sim.world.set_tile_mask(forest.tile(), 0x30);
    sim.action_gather_point(0, &[city], forest, 0, false);
    assert_eq!(sim.buildings[city].gather[0].pos, camp_at, "snapped");
    assert_eq!(sim.buildings[city].gather[0].action, 0, "the action kept");
    sim.action_gather_point(0, &[city, b], forest, 0, false);
    assert_eq!(
        sim.buildings[city].gather[0].pos, forest,
        "not every member a City"
    );
    let far = tile_pos(60, 32);
    sim.world.set_tile_mask(far.tile(), 0x30);
    sim.action_gather_point(0, &[city], far, 0, false);
    assert_eq!(
        sim.buildings[city].gather[0].pos, far,
        "no Woodcutter within 0x600"
    );
}

/// **A trained member's turn to its captain's gather bearing flips its
/// mirror** (run312's `0/12` and `0/13` on 856): the member's angle is
/// `Unit::set_angle`'s, whose `reversing` test flips the mirror past 90°;
/// the captain's is a bare store and keeps its own.
#[test]
fn a_member_turned_past_ninety_degrees_to_its_captain_s_bearing_flips_its_mirror() {
    let mut sim = world_sim();
    let (_, b, squad) = rally_barracks(&mut sim);
    let home = sim.buildings[b].pos;
    sim.action_gather_point(0, &[b], Pos::new(home.x - 2000, home.y - 2000), 0, false);
    let cap = sim.build_train(b, squad).unit;
    let before = crate::Movement::at(home).mirror;
    assert_eq!(
        sim.units[cap].movement.mirror, before,
        "the captain keeps its own"
    );
    for m in sim.squad_members(cap).into_iter().filter(|&m| m != cap) {
        assert_ne!(sim.units[m].movement.mirror, before, "member {m} flipped");
    }
}

/// The first candidate of `b`'s land exit ring on `bearing`, snapped to
/// its quarter-tile centre, for the re-seat tests below.
fn ring_first(sim: &Sim, b: usize, bearing: crate::movement::Angle) -> Pos {
    let t = sim.buildings[b].ty.unwrap();
    let (xs, ys) = (sim.build_types[t].x_size, sim.build_types[t].y_size);
    let r = (xs + ys) * 0x30 + sim.tuning.unit_train_distance;
    let home = sim.buildings[b].pos;
    let x = home.x + crate::movement::sin_component(bearing, r);
    let y = home.y - crate::movement::cos_component(bearing, r);
    Pos::new(x.div_euclid(48) * 48 + 24, y.div_euclid(48) * 48 + 24)
}

/// **A squad trained under a building's point is re-seated on the bearing
/// to that building** (item 945, run312's Bowmen `0/17` on 1060):
/// `come_out`'s routing sweeps the captain round its trainer again from
/// `find_angle(building − trainer)`, due east here, and not from the
/// gather block's bearing to the free spot beside it; the members stay
/// where the exit seated them.
#[test]
fn a_squad_trained_under_a_building_s_point_is_re_seated_on_the_bearing_to_that_building() {
    let mut sim = world_sim();
    let (t, b, squad) = rally_barracks(&mut sim);
    let other = sim.place_building(0, t.barracks, tile_pos(48, 40)).unwrap();
    finish(&mut sim, other);
    let there = sim.buildings[other].pos;
    let home = sim.buildings[b].pos;
    sim.action_gather_point(0, &[b], there, 1, false);
    let cap = sim.build_train(b, squad).unit;
    let east = crate::movement::find_angle(there.x - home.x, there.y - home.y);
    let exit = ring_first(&sim, b, sim.units[cap].movement.heading);
    assert_ne!(exit, ring_first(&sim, b, east), "the two bearings differ");
    assert_eq!(sim.units[cap].pos, ring_first(&sim, b, east));
    for m in sim.squad_members(cap).into_iter().filter(|&m| m != cap) {
        let d = crate::world::vector_dist(sim.units[m].pos.x - exit.x, sim.units[m].pos.y - exit.y);
        assert!(d <= 480, "member {m} seated round the exit: {d}");
    }
}

/// **A lone unit's move arm re-seats it again, on the exit's own bearing**
/// (item 945, run312's Citizen `0/10` on 760, its point on the
/// Woodcutter): the building point's re-seat moves it off its exit point,
/// and the lone arm's sweep from `find_angle(spot − trainer)` puts it back.
#[test]
fn a_lone_unit_under_a_building_s_point_is_put_back_on_its_exit_bearing() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (city, _) = city_at(&mut sim, &t, 0, 32, 32);
    let walker = sim.add_unit_type(hoplite_type(t.village));
    let b = sim.place_building(0, t.barracks, tile_pos(40, 32)).unwrap();
    finish(&mut sim, b);
    let there = sim.buildings[b].pos;
    let home = sim.buildings[city].pos;
    sim.action_gather_point(0, &[city], there, 0, false);
    let u = sim.build_train(city, walker).unit;
    let bearing = sim.units[u].movement.heading;
    let east = crate::movement::find_angle(there.x - home.x, there.y - home.y);
    assert_ne!(
        ring_first(&sim, city, bearing),
        ring_first(&sim, city, east),
        "the two bearings differ"
    );
    assert_eq!(sim.units[u].pos, ring_first(&sim, city, bearing));
}

/// **A squad trained under two points walks to the first and attack-moves
/// to the second** (item 955, run338's `0/17`..`0/19` on 953): `come_out`'s
/// routing walks the list from its head, the first point a waypoint sent
/// under `MOVE_TO` and the last under its own kind, both `QUEUE_LAST` and
/// one group; the exit leaves on the bearing to the head.
#[test]
fn a_squad_trained_under_two_points_moves_to_the_first_then_attack_moves_to_the_second() {
    let mut sim = world_sim();
    let (_, b, squad) = rally_barracks(&mut sim);
    let home = sim.buildings[b].pos;
    let first = Pos::new(home.x + 1536, home.y - 1536);
    let second = Pos::new(home.x - 1536, home.y - 3072);
    sim.action_gather_point(0, &[b], first, 0, false);
    sim.action_gather_point(0, &[b], second, 0, true);
    let cap = sim.build_train(b, squad).unit;
    let p = sim.units[cap].pos;
    assert!(p.x > home.x && p.y < home.y, "out toward the head: {p:?}");
    for m in sim.squad_members(cap) {
        let legs: Vec<_> = sim.units[m]
            .orders
            .iter()
            .map(|o| match &o.body {
                crate::orders::Body::Move(mv) => (mv.kind, mv.dest, mv.group.map(|g| g.id)),
                other => panic!("member {m}: {other:?}"),
            })
            .collect();
        assert_eq!(legs.len(), 2, "member {m}: {legs:?}");
        assert_eq!(legs[0].0, crate::orders::MoveKind::MoveTo);
        assert_eq!(legs[1].0, crate::orders::MoveKind::AttackTo);
        assert!(
            crate::world::vector_dist(legs[0].1.x - first.x, legs[0].1.y - first.y) < 480,
            "member {m}'s first leg near the head: {legs:?}"
        );
        assert!(
            crate::world::vector_dist(legs[1].1.x - second.x, legs[1].1.y - second.y) < 480,
            "member {m}'s second leg near the tail: {legs:?}"
        );
    }
}

/// **A squad trained under a point on a unit is re-seated on the bearing
/// to that unit** (item 955, run338's `0/13` on 858, its point on the
/// Chariot `0/10`): with no building at the last point,
/// `find_unit_with_radius` finds the unit whose body covers it, and the
/// routing sweeps the captain round its trainer again from
/// `find_angle(unit − trainer)`; the members stay where the exit seated
/// them.
#[test]
fn a_squad_trained_under_a_point_on_a_unit_is_re_seated_on_the_bearing_to_that_unit() {
    let mut sim = world_sim();
    let (t, b, squad) = rally_barracks(&mut sim);
    let home = sim.buildings[b].pos;
    let there = Pos::new(home.x - 648, home.y + 1272);
    let mut standing = hoplite_type(t.barracks);
    standing.combat.block_radius = 48;
    standing.combat.big_radius = 48;
    let standing = sim.add_unit_type(standing);
    let _ = spawn(&mut sim, 0, standing, there);
    sim.action_gather_point(0, &[b], there, 1, false);
    let cap = sim.build_train(b, squad).unit;
    let toward = crate::movement::find_angle(there.x - home.x, there.y - home.y);
    let exit = ring_first(&sim, b, sim.units[cap].movement.heading);
    assert_ne!(exit, ring_first(&sim, b, toward), "the two bearings differ");
    assert_eq!(sim.units[cap].pos, ring_first(&sim, b, toward));
}

/// **A Citizen trained under its own gather building's point, action 1,
/// is sent to gather there; under its own unfinished building's point, to
/// build it** (item 955, run338's `0/12` on 824 and `0/16` on 938): the
/// routing's citizen arms, each ending it before any move.
#[test]
fn a_citizen_under_a_point_on_its_own_building_gathers_there_or_builds_it() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (city, _) = city_at(&mut sim, &t, 0, 32, 32);
    let mut ct = citizen_type(t.village);
    ct.type_index = 0x32;
    let citizen = sim.add_unit_type(ct);
    let camp_t = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 4, 4, 150, 400, 0));
    let camp = sim.add_building(0, tile_pos(44, 32), 2);
    sim.buildings[camp].ty = Some(camp_t);
    let camp_at = sim.buildings[camp].pos;
    sim.action_gather_point(0, &[city], camp_at, 1, false);
    let u = sim.build_train(city, citizen).unit;
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(&o.body, crate::orders::Body::Gather(g) if g.building == camp)),
        "a gather order on the camp: {:?}",
        sim.units[u].orders
    );
    let site = sim.place_building(0, t.barracks, tile_pos(40, 24)).unwrap();
    assert!(!sim.buildings[site].active);
    let site_at = sim.buildings[site].pos;
    sim.action_gather_point(0, &[city], site_at, 1, false);
    let v = sim.build_train(city, citizen).unit;
    assert!(
        sim.units[v]
            .orders
            .iter()
            .any(|o| matches!(o.body, crate::orders::Body::Build(s) if s == site)),
        "a build order on the site: {:?}",
        sim.units[v].orders
    );
}

/// **A lone unit under a ground point counts itself in its re-seat**
/// (item 955, run338's Citizens `0/11` on 718, `0/12` on 824 and `0/16` on
/// 938): the lone arm sweeps its trainer's ring again from the exit's own
/// bearing, and under `FILTER_ALL` the unit standing on the first
/// candidate is a candidate's blocker like any other, so it moves one
/// candidate on.
#[test]
fn a_lone_unit_under_a_ground_point_is_re_seated_past_the_spot_it_stands_on() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (city, _) = city_at(&mut sim, &t, 0, 32, 32);
    let mut walker = hoplite_type(t.village);
    walker.combat.block_radius = 48;
    walker.combat.big_radius = 24;
    let walker = sim.add_unit_type(walker);
    let home = sim.buildings[city].pos;
    let point = Pos::new(home.x + 1500, home.y + 300);
    sim.action_gather_point(0, &[city], point, 0, false);
    let u = sim.build_train(city, walker).unit;
    let bearing = sim.units[u].movement.heading;
    let exit = ring_first(&sim, city, bearing);
    let p = sim.units[u].pos;
    assert_ne!(p, exit, "counted: not left on the spot it stood on");
    let d = crate::world::vector_dist(p.x - exit.x, p.y - exit.y);
    assert!(
        d <= 480,
        "one candidate on, on the same ring: {p:?} against {exit:?}"
    );
}

/// **A city hit by another player stops healing for two decay ticks**
/// (item 1034, `docs/COMBAT.md` §69): `Object::take_damage`'s `city_flags
/// |= 0xe` on any building of the city, then `Build::process`'s 200-frame
/// decay on the city building — `CITY_ATTACKING` on the first tick,
/// `CITY_UNDER_ATTACK`, the heal's veto, on the second. Its own owner's
/// hit and attrition set nothing. Great Lakes' 4690 was a city that healed
/// off its first wound while the original's held it.
///
/// Made to fail by taking the setter out of `damage_building` (the
/// second assertion), and by clearing `0x2` on the first decay tick.
#[test]
fn a_city_hit_by_another_player_stops_healing_for_two_decay_ticks() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 1, 32, 32);
    let hoplite = sim.add_unit_type(hoplite_type(t.barracks));
    let own = spawn(&mut sim, 1, hoplite, tile_pos(36, 32));
    let foe = spawn(&mut sim, 0, hoplite, tile_pos(36, 33));
    let flags = |s: &Sim| {
        let cd = &s.cities[c];
        (cd.no_heal, cd.attacking, cd.ever_attacked)
    };
    let hit = |s: &mut Sim, by: usize, f: i64| {
        s.do_damage(
            combat::Obj::Unit(by),
            combat::Obj::Building(b),
            movement::Angle(0),
            false,
            256,
            false,
            true,
            f,
        );
    };
    hit(&mut sim, own, 100);
    let eight = combat::Sixteenths { whole: 8, frac: 0 };
    sim.damage_building(b, eight, None, 100, true);
    assert_eq!(
        flags(&sim),
        (false, false, false),
        "its owner and attrition"
    );
    hit(&mut sim, foe, 101);
    assert_eq!(flags(&sim), (true, true, true), "another player's hit: 0xe");
    // The decay ticks on the city building's own phase.
    let o = i64::from(sim.buildings[b].index);
    let first = (1000 - o).rem_euclid(200) + 200;
    let bd = &mut sim.buildings[b];
    bd.damage = 10;
    bd.sync_health();
    for f in 102..first {
        sim.process_building_whole(b, f);
    }
    assert_eq!(sim.buildings[b].damage, 10, "no heal while under attack");
    sim.process_building_whole(b, first);
    assert_eq!(flags(&sim), (true, false, true), "the first tick: 0x4");
    for f in first + 1..first + 200 {
        sim.process_building_whole(b, f);
    }
    assert_eq!(sim.buildings[b].damage, 10, "still no heal");
    sim.process_building_whole(b, first + 200);
    assert_eq!(flags(&sim), (false, false, true), "the second: 0x2");
    assert_eq!(sim.buildings[b].damage, 9, "and the heal, the same frame");
}

/// **A caravan trades across water when it `can_transport`**
/// (`docs/CARAVAN.md` §11, item 1115). `Unit::do_trade@005ed270` asks
/// `can_transport` wherever two regions differ: the caravan's and its home
/// city's, each candidate's and the home's, and the chosen pair's. East
/// Indies' AI caravan `1/33` has London in region 11 and its only free
/// partner, Newcastle, in region 5; this crate refused every cross-region
/// partner, found none, and idled it (`idle` 99) where the original laid a
/// road. Its move to the near city faces the **bearing** there, the
/// `ecx`/`edx` pair `add_move_order@00616ed0` loads.
#[test]
fn a_caravan_trades_across_regions_only_when_it_can_transport() {
    let build = |transport: bool| {
        let mut w = World::new(40, 40);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(17, 39));
        w.fill_region(Terrain::Sea, Cell::new(18, 0), Cell::new(21, 39));
        w.fill_region(Terrain::Land, Cell::new(22, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; economy::RESOURCES];
        }
        let t = install_types(&mut sim);
        sim.tech[1].epoch[tech::Line::Civic as usize] = 1;
        let (_, home) = city_at(&mut sim, &t, 1, 40, 60);
        let (_, far) = city_at(&mut sim, &t, 1, 96, 40);
        assert_ne!(
            sim.world.tregion(sim.cities[home].pos.tile()),
            sim.world.tregion(sim.cities[far].pos.tile()),
            "two regions"
        );
        let ty = sim.add_unit_type(citizen_type(t.village));
        let u = spawn(&mut sim, 1, ty, tile_pos(46, 70));
        sim.units[u].auto_transport = transport;
        let v = sim.init_caravan(1, u).expect("a slot");
        sim.units[u].caravan = Some(v);
        sim.caravans[1].slots[v].unit = Some(u);
        // The human arm's threshold under the default option.
        sim.units[u].idle = 12;
        assert!(sim.think_caravan(u), "the order is given");
        sim.do_trade(u);
        (sim, u, v, home, far)
    };

    let (sim, u, v, ..) = build(false);
    assert_eq!(sim.units[u].idle, 99, "no partner: the order dies");
    assert!(sim.units[u].orders.is_empty());
    assert!(!sim.caravans[1].slots[v].linked);

    let (sim, u, v, home, far) = build(true);
    let van = &sim.caravans[1].slots[v];
    assert!(van.linked, "the route is taken");
    assert_eq!((van.city_a, van.city_b), (Some(home), Some(far)));
    // The search stops at its budget here, so `do_trade`'s failure arm
    // sends the caravan to the near city — `home` — with the bearing to
    // its own point (the order's `dest` is that point snapped).
    assert!(van.making_road, "the plan spans frames");
    let (here, to) = (sim.units[u].pos, sim.cities[home].pos);
    let Some(orders::Body::Move(m)) = sim.units[u].orders.front().map(|o| o.body) else {
        panic!("a move leads: {:?}", sim.units[u].orders);
    };
    assert_eq!(
        m.angle,
        movement::find_angle(to.x - here.x, to.y - here.y),
        "the move faces its bearing"
    );
    assert_ne!(m.angle, movement::find_angle(1, 0), "and not due east");
}

/// The untargeted crafts' fixture (item 1111, `docs/GOLDEN.md` §48): a
/// Citizen (`PEASANTS`, 40 hits, LOS 2), the Militia (50), a General
/// (`MANA 1000`) and an armed foot squad type, and the three rows as
/// `craftrules.xml` has them — To Arms and Civilian with no flags and job
/// 5, Create Decoys `lm`, job 100, `MANA 1000`. Returns the sim and the
/// four unit records, plus the Militia's tree id.
fn untargeted_sim() -> (Sim, [usize; 4], crate::tech::TypeId) {
    use crate::orders::spell;
    use crate::tech::{TechTree, TypeDef, UnitTraits as Traits};
    let mut sim = world_sim();
    let mut tree = TechTree::new();
    let citizen_t = tree.add(TypeDef::unit("Citizen", Traits::default()));
    let militia_t = tree.add(TypeDef::unit("Militia", Traits::default()));
    let general_t = tree.add(TypeDef::unit("General", Traits::default()));
    let foot_t = tree.add(TypeDef::unit("Hoplites", Traits::default()));
    sim.set_tech_tree(tree);
    let ty = |sim: &mut Sim, ti: i32, hits: i32, los: i32, attack: i32, mana: i32, t| {
        let mut u = UnitType {
            hits,
            los,
            mana,
            tree: Some(t),
            type_index: ti,
            ..UnitType::default()
        };
        u.combat.attack = attack;
        sim.add_unit_type(u)
    };
    let citizen = ty(&mut sim, 0x32, 40, 2, 4, 0, citizen_t);
    let militia = ty(&mut sim, 0x42, 50, 6, 10, 0, militia_t);
    let general = ty(&mut sim, 0x36, 109, 8, 0, 1000, general_t);
    let foot = ty(&mut sim, 0x84, 120, 6, 13, 0, foot_t);
    for t in [citizen_t, foot_t, general_t] {
        sim.tech[0].tech[t] = true;
    }
    let mut rows = vec![crate::orders::SpellType::default(); 55];
    let row = |s: i32| (s - spell::FIRST) as usize;
    rows[row(spell::TO_ARMS)] = crate::orders::SpellType {
        job_time: 5,
        from: [Some(citizen_t), None],
        ..Default::default()
    };
    rows[row(spell::CIVILIAN)] = crate::orders::SpellType {
        job_time: 5,
        from: [Some(militia_t), None],
        ..Default::default()
    };
    rows[row(spell::CREATE_DECOY)] = crate::orders::SpellType {
        job_time: 100,
        flags: 0x1800,
        mana: 1000,
        from: [Some(general_t), None],
        ..Default::default()
    };
    sim.spells = rows;
    (sim, [citizen, militia, general, foot], militia_t)
}

/// Press an untargeted craft on one unit, as the button does: `(ox, whom)`
/// `(−1, −1)` and the point `(0, 0)`.
fn press(sim: &mut Sim, u: usize, s: i32) -> usize {
    let mut g = crate::group::Group::stack(sim.units[u].owner);
    sim.group_add(&mut g, u);
    assert!(sim.push_group(&mut g, true));
    sim.group_action_spell(&g, s, None, Pos::new(0, 0))
}

/// **To Arms keeps the damage's fraction, `rare` the former type, and
/// wants the Militia's own bit** (item 1111, `docs/GOLDEN.md` §48; the
/// listing of `cast_to_arms@00670880` and the emulator's sweep: a Citizen
/// at 7 of 40 is a Militia at 8 of 50). The press is refused while the
/// player holds no Militia line (run422's first take: `tech … militia`
/// without `on` changed nothing, and no order was laid), and a Citizen's
/// own hits and LOS are the Militia line's once it does (`Object::
/// update_hits`, `Unit::update_los`).
///
/// Made to fail with the damage carried whole (`set_type`'s rule: 7 of
/// 50), with `rare` left 0, and with the castability's Militia test
/// dropped.
#[test]
fn a_citizen_s_to_arms_keeps_its_damage_fraction_and_its_former_type() {
    use crate::orders::spell;
    let (mut sim, [citizen, militia, _, _], militia_t) = untargeted_sim();
    let c = sim.init_unit(0, citizen, tile_pos(20, 20));
    assert_eq!(press(&mut sim, c, spell::TO_ARMS), 0, "no Militia held");
    assert!(sim.units[c].orders.is_empty());
    assert_eq!((sim.units[c].max_health, sim.unit_los(c)), (40, 2));
    sim.gain_tech(0, militia_t);
    assert_eq!(
        (sim.units[c].max_health, sim.unit_los(c)),
        (50, 4),
        "a Citizen's hits are the Militia's, and it sees two more"
    );
    sim.units[c].health = 50 - 9;
    assert_eq!(press(&mut sim, c, spell::TO_ARMS), 1);
    assert_eq!(sim.units[c].cavarch_who, -1, "an untargeted press");
    for _ in 0..5 {
        sim.tick();
    }
    let u = &sim.units[c];
    assert_eq!(u.ty, Some(militia));
    assert_eq!(u.rare, 0x32, "rare keeps the Citizen");
    assert_eq!(u.form, 0);
    // frac = (9 << 8) / 50 = 46; 50 × 46 / 256 = 8.
    assert_eq!((u.max_health, u.max_health - u.health), (50, 8));
    assert!(u.orders.is_empty(), "the cast is spent");
}

/// **Civilian reads `rare`: a Militia never converted becomes the
/// Citizen and keeps `rare` at `PEASANTS`** (item 1111, `docs/GOLDEN.md`
/// §48; run422's C, `rare` 0 → 50 on 666, and A's, 50 kept on 706).
///
/// Made to fail with `rare` cleared by the cast.
#[test]
fn a_militia_s_civilian_turns_back_to_what_rare_holds() {
    use crate::orders::spell;
    let (mut sim, [citizen, militia, _, _], militia_t) = untargeted_sim();
    sim.gain_tech(0, militia_t);
    let m = sim.init_unit(0, militia, tile_pos(20, 20));
    assert_eq!(sim.units[m].rare, 0);
    sim.units[m].health = 50 - 13;
    assert_eq!(press(&mut sim, m, spell::CIVILIAN), 1);
    for _ in 0..5 {
        sim.tick();
    }
    let u = &sim.units[m];
    assert_eq!(u.ty, Some(citizen));
    assert_eq!((u.rare, u.form), (0x32, 9));
    // frac = (13 << 8) / 50 = 66; the Citizen's hits are the Militia's 50
    // now, so 50 × 66 / 256 = 12.
    assert_eq!((u.max_health, u.max_health - u.health), (50, 12));
}

/// **Create Decoys copies the armed land captains within the General's
/// radius, each figure a decoy of age 0 that sees one tile** (item 1111,
/// `docs/GOLDEN.md` §48; run422's block 821). A squad of the foot type
/// beside the General is copied; a Citizen beside it (a `PEASANTS`) and a
/// squad eleven tiles off (past `GENERAL_RADIUS` 6 × 3/2 = 9) are not. The
/// General pays its 1,000 once, and a decoy's `mana_burn` counts up.
///
/// Made to fail with the radius taken in tiles rather than × 0xc0, and
/// with the decoy's ageing dropped.
#[test]
fn a_general_s_decoys_copy_the_armed_land_captains_near_it() {
    use crate::orders::spell;
    let (mut sim, [citizen, _, general, foot], _) = untargeted_sim();
    let g = sim.init_unit(0, general, tile_pos(30, 30));
    let near = sim.init_unit(0, foot, tile_pos(33, 30));
    let _civ = sim.init_unit(0, citizen, tile_pos(30, 33));
    let far = sim.init_unit(0, foot, tile_pos(41, 30));
    let before = sim.units.len();
    assert_eq!(press(&mut sim, g, spell::CREATE_DECOY), 1);
    sim.tick();
    assert_eq!(
        sim.units[g].mana_burn, 1000,
        "paid once, on the first frame"
    );
    for _ in 0..99 {
        sim.tick();
    }
    let decoys: Vec<usize> = (before..sim.units.len()).collect();
    assert_eq!(decoys.len(), 1, "one copy, of the near squad");
    let d = decoys[0];
    assert_eq!(sim.units[d].ty, sim.units[near].ty);
    assert!(sim.units[d].decoy && !sim.units[near].decoy && !sim.units[far].decoy);
    assert_eq!(sim.unit_los(d), 1);
    let age = sim.units[d].mana_burn;
    sim.tick();
    assert_eq!(sim.units[d].mana_burn, age + 1, "a decoy ages a frame");
}

/// The President's decoy reach includes its extra tile, while the
/// ordinary General retains nine tiles. The boundary is strict (§60).
#[test]
fn a_patriot_s_decoy_cast_uses_its_own_radius() {
    for (caster_type, offset, expected) in [
        (0x36, 1727, 1),
        (0x36, 1728, 0),
        (0x163, 1888, 1),
        (0x163, 1919, 1),
        (0x163, 1920, 0),
    ] {
        let (mut sim, [_, _, general, foot], _) = untargeted_sim();
        sim.unit_types[general].type_index = caster_type;
        let g = sim.init_unit(0, general, tile_pos(30, 30));
        let p = sim.units[g].pos;
        // `init_unit` snaps its birth point, so the distance under test is
        // set after it.
        let f = sim.init_unit(0, foot, Pos::new(p.x + offset, p.y));
        sim.units[f].pos = Pos::new(p.x + offset, p.y);
        let before = sim.units.len();
        sim.cast_create_decoy(g);
        assert_eq!(
            sim.units.len() - before,
            expected,
            "caster {caster_type}, distance {offset}"
        );
    }
}

/// **A decoy closes when its age reaches `(general_upgrade + 2) ×
/// DECOY_TIME / 2`, 2,500 frames, silently** (item 1351, `docs/GOLDEN.md`
/// §48; run535's `1/104`..`1/120` at `mana_burn` 2499 on block 14136 and
/// gone on 14137, `1/93` at 2498 a block later). The copy at 2499 closes on
/// its next frame, the one at 2498 a frame after; the close leaves no
/// death object, and the General and the squad it copied stand.
///
/// Made to fail with the close dropped, and with `<=` taken as `<`.
#[test]
fn a_decoy_closes_at_its_age_and_leaves_no_death() {
    use crate::orders::spell;
    let (mut sim, [_, _, general, foot], _) = untargeted_sim();
    let g = sim.init_unit(0, general, tile_pos(30, 30));
    let near = sim.init_unit(0, foot, tile_pos(33, 30));
    let before = sim.units.len();
    assert_eq!(press(&mut sim, g, spell::CREATE_DECOY), 1);
    for _ in 0..100 {
        sim.tick();
    }
    let copies: Vec<usize> = (before..sim.units.len())
        .filter(|&d| sim.units[d].decoy)
        .collect();
    assert!(!copies.is_empty(), "the near squad is copied");
    let (first, late) = (copies[0], *copies.last().unwrap());
    for &d in &copies {
        sim.units[d].mana_burn = if d == late && late != first {
            2498
        } else {
            2499
        };
    }
    let deaths = sim.deaths.len();
    sim.tick();
    assert!(!sim.units[first].alive(), "2499 + 1 reaches the life");
    if late != first {
        assert!(sim.units[late].alive(), "2498 + 1 does not");
        assert_eq!(sim.units[late].mana_burn, 2499);
        sim.tick();
        assert!(!sim.units[late].alive(), "and a frame later it does");
    }
    assert_eq!(sim.deaths.len(), deaths, "a close of 0 leaves no death");
    assert!(copies.iter().all(|&d| sim.units[d].orders.is_empty()));
    assert!(sim.units[g].alive() && sim.units[near].alive());
}

/// **A computer's General standing in its army orders Create Decoys on
/// the army's spellcaster turn, and the copies join that army** (item
/// 1302; run346's `1/98`, `spell_time` 1 and `mana_burn` 1,000 on block
/// 11539, six squads in its group 70 on 11638). `think_spellcaster`'s hero
/// arm lays the cast at the General's own point, `QUEUE_FIRST`; it refuses
/// while any decoy captain of its player stands, and while the General is
/// moving. The army's standard line does not count a decoy.
///
/// Made to fail with the hero arm back at `return 0`, with the copies left
/// out of the army, and with `num_decoys` not counted.
#[test]
fn an_ai_general_standing_orders_create_decoys_and_its_copies_join_its_army() {
    use crate::orders::{Body, spell};
    let (mut sim, [_, _, general, foot], _) = untargeted_sim();
    sim.unit_types[general].cols.unit_flags2 |=
        crate::ai_load::uflags2::GENERAL | crate::ai_load::uflags2::CASTER;
    sim.nation[0].human = false;
    sim.lobby.difficulty = 5;
    let g = sim.init_unit(0, general, tile_pos(30, 30));
    let near = sim.init_unit(0, foot, tile_pos(33, 30));
    let slot = sim.init_army(0, None);
    sim.army_add_unit(0, slot, near);
    sim.army_add_unit(0, slot, g);
    sim.army_normalize(0, slot);
    assert_eq!(sim.army_of(g), Some(slot));
    assert!(sim.think_spellcaster(g), "the hero arm casts");
    match sim.units[g].orders.front().map(|o| o.body) {
        Some(Body::Cast(c)) => {
            assert_eq!(c.spell, spell::CREATE_DECOY);
            assert_eq!((c.target, c.at), (None, sim.units[g].pos));
        }
        other => panic!("not a cast: {other:?}"),
    }
    let before = sim.units.len();
    sim.cast_create_decoy(g);
    let copy = before;
    assert!(sim.units[copy].decoy);
    assert_eq!(sim.army_of(copy), Some(slot), "the copy joins the army");
    sim.army_normalize(0, slot);
    let a = &sim.armies[0].list[slot];
    assert_eq!((a.num_captains, a.num_decoys), (3, 1));
    assert_eq!(a.num_standard, 1, "the Hoplites alone: the General casts");
    sim.units[g].orders.clear();
    assert!(!sim.think_spellcaster(g), "a decoy captain stands");
    sim.units[copy].health = 0;
    assert!(sim.think_spellcaster(g), "and none does");
    sim.units[g].orders.clear();
    sim.add_move_order(
        g,
        tile_pos(20, 20),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        true,
    );
    assert!(!sim.think_spellcaster(g), "a moving General: the seam");
}

/// **A copy is placed by its General's collision pair, not by the copied
/// type's radius** (item 1302, `docs/AI.md` §99.14): `cast_create_decoy`
/// asks `find_nearby_spot(type, …, FILTER_NOT_ME, the General, 0)`, whose
/// collision half is `find_collision`/`find_ordered_collision` against the
/// General. A type whose own block radius would refuse every spot beside
/// the General is still copied beside it; run346's fourth squad on 11637
/// stands at `k = −3`, where the radius query put it at `k = −2`.
///
/// Made to fail with the spot asked by the type alone.
#[test]
fn a_decoy_is_placed_by_its_general_s_collision_pair() {
    use crate::orders::spell;
    let (mut sim, [_, _, general, foot], _) = untargeted_sim();
    sim.unit_types[foot].combat.block_radius = 0x200;
    let g = sim.init_unit(0, general, tile_pos(30, 30));
    let _near = sim.init_unit(0, foot, tile_pos(33, 30));
    let before = sim.units.len();
    sim.units[g].orders.clear();
    sim.cast_create_decoy(g);
    assert_eq!(sim.units.len(), before + 1, "one copy, beside the General");
    let d = crate::world::vector_dist(
        sim.units[before].pos.x - sim.units[g].pos.x,
        sim.units[before].pos.y - sim.units[g].pos.y,
    );
    assert!(
        (0x150..0x200).contains(&d),
        "on the first ring, snapped: {d}"
    );
    let _ = spell::CREATE_DECOY;
}

/// **A decoy's type change is no count of its leader's** (item 1302,
/// `docs/AI.md` §99.14; both of `Unit::set_type`'s `track_unit_type` calls
/// sit behind `(unit_masks & 1) == 0`). run346's decoy Peltasts took the
/// age's upgrade after 11637, and without the test `num_units` went to −2
/// on run508's block 12577.
///
/// Made to fail with the decoy test dropped.
#[test]
fn a_decoy_s_type_change_is_no_count_of_its_leader_s() {
    let (mut sim, [_, militia, _, foot], _) = untargeted_sim();
    let d = sim.init_unit(0, foot, tile_pos(30, 30));
    sim.track_unit_type(0, foot, -1);
    sim.units[d].decoy = true;
    let before = (sim.muster[0].by_type[foot], sim.muster[0].by_type[militia]);
    sim.unit_set_type(d, militia);
    assert_eq!(sim.units[d].ty, Some(militia));
    assert_eq!(
        (sim.muster[0].by_type[foot], sim.muster[0].by_type[militia]),
        before
    );
}

/// **A paid cast that dies before its craft hands the craft back; one
/// that ran its course does not, and the cast clock restarts under any
/// other order** (item 1302; run346's `1/98`: the army's `ATTACK_TO`
/// replaced its Create Decoys on 11508 at `spell_time` 98, and block 11509
/// has `mana_burn` 903 → 0 and `spell_time` 0 — `kill_current_order`'s
/// `unpay_cast_costs`, and `Unit::work`'s reset).
///
/// Made to fail with the refund dropped, with `paid` left set through the
/// closing kill, and with the clock's reset dropped.
#[test]
fn an_interrupted_cast_hands_its_craft_back_and_its_clock_restarts() {
    use crate::orders::spell;
    let (mut sim, [_, _, general, foot], _) = untargeted_sim();
    let g = sim.init_unit(0, general, tile_pos(30, 30));
    let _near = sim.init_unit(0, foot, tile_pos(33, 30));
    assert_eq!(press(&mut sim, g, spell::CREATE_DECOY), 1);
    for _ in 0..10 {
        sim.tick();
    }
    let (burn, clock) = (sim.units[g].mana_burn, sim.units[g].spell_time);
    assert!(burn > 900 && clock == 10, "{burn} {clock}");
    sim.add_move_order(
        g,
        tile_pos(30, 34),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        true,
    );
    assert_eq!(sim.units[g].mana_burn, 0, "the craft handed back");
    assert_eq!(sim.units[g].spell_time, clock, "until the unit works");
    sim.tick();
    assert_eq!(sim.units[g].spell_time, 0, "`Unit::work` restarts it");
    sim.units[g].orders.clear();
    assert_eq!(press(&mut sim, g, spell::CREATE_DECOY), 1);
    for _ in 0..101 {
        sim.tick();
    }
    assert!(sim.units[g].orders.is_empty(), "the cast is spent");
    assert!(
        sim.units[g].mana_burn > 800,
        "a cast that ran its course keeps its craft spent: {}",
        sim.units[g].mana_burn
    );
}

/// Chapter forty's fixture (item 1167, `docs/GOLDEN.md` §49):
/// [`untargeted_sim`]'s types with a city's buildings beside them, the
/// Citizen trained at the city (so it garrisons one) and the Militia a
/// fortifying type (so it does too). Returns the sim, the build types and
/// the Citizen's and the Militia's records.
fn alarm_sim() -> (Sim, Types, usize, usize) {
    let (mut sim, [citizen, militia, _, _], _) = untargeted_sim();
    let t = install_types(&mut sim);
    sim.unit_types[citizen].garrison.trained_at = Some(t.village);
    sim.unit_types[citizen].price.pop = 1;
    sim.unit_types[militia].garrison.fortify = true;
    sim.unit_types[militia].price.pop = 1;
    (sim, t, citizen, militia)
}

/// **A Militia in a repair swarm is a builder, and takes the Civilian
/// between its approach and the repair** (item 1167, `docs/GOLDEN.md`
/// §49; `Group::action_swarm_around@0070fbe0`'s member arm, run430's M1
/// on block 632: a `MOVEORDER`, a `CASTORDER` of 659 at its own point,
/// `ox` −1, then the `REPAIRORDER` on 2006).
///
/// Made to fail with the member filter back at Citizens alone, and with
/// the cast dropped.
#[test]
fn a_militia_in_a_repair_swarm_takes_the_civilian_between_its_approach_and_the_repair() {
    use crate::orders::{Body, QueuePos, index, spell};
    let (mut sim, t, _, militia) = alarm_sim();
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let b = sim.place_building(0, t.barracks, tile_pos(40, 40)).unwrap();
    finish(&mut sim, b);
    let m = sim.init_unit(0, militia, tile_pos(40, 52));
    let at = sim.units[m].pos;
    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, m);
    assert!(sim.push_group(&mut g, true));
    assert!(sim.group_swarm_around(&g, b, QueuePos::New, index::REPAIR));
    let kinds: Vec<u8> = sim.units[m].orders.iter().map(|o| o.index()).collect();
    assert_eq!(
        kinds,
        vec![index::MOVE_TO, index::CAST_SPELL, index::REPAIR],
        "the approach, the Civilian, the repair"
    );
    match sim.units[m].orders[1].body {
        Body::Cast(c) => {
            assert_eq!(c.spell, spell::CIVILIAN);
            assert_eq!(c.target, None, "untargeted: (−1, −1)");
            assert_eq!(c.at, at, "on the point it stood on at the swarm");
        }
        other => panic!("not a cast: {other:?}"),
    }
}

/// **A Militia's gather puts the Civilian in front of the gather**
/// (item 1167, `docs/GOLDEN.md` §49; `Group::action_gather@00700b90` at
/// `QUEUE_NEW`: `add_gather_order(QUEUE_NEW)`, then `add_cast_order(0x293,
/// QUEUE_FIRST)` — run430's M3 on block 652, a `CASTORDER` at its own
/// point ahead of the `GATHERORDER` on 2001, a Citizen by 657).
///
/// Made to fail with the cast dropped, and with it laid behind the
/// gather.
#[test]
fn a_militia_s_gather_puts_the_civilian_in_front_of_the_gather() {
    use crate::orders::{Body, QueuePos, index, spell};
    let (mut sim, t, citizen, militia) = alarm_sim();
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let farm = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
    finish(&mut sim, farm);
    assert!(sim.is_gather_type(farm));
    let m = sim.init_unit(0, militia, tile_pos(44, 40));
    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, m);
    assert!(sim.push_group(&mut g, true));
    sim.group_gather(&g, farm, QueuePos::New);
    let u = &sim.units[m];
    assert_eq!(u.orders.len(), 2);
    assert!(matches!(u.orders[0].body, Body::Cast(c) if c.spell == spell::CIVILIAN));
    assert_eq!(u.orders[1].index(), index::GATHER);
    for _ in 0..6 {
        sim.tick();
    }
    assert_eq!(sim.units[m].ty, Some(citizen), "a Citizen five frames on");
}

/// **The alarm rings, then sounds the all-clear** (item 1167,
/// `docs/GOLDEN.md` §49; `Group::action_alarm@0070ec30`, run430's blocks
/// 882 and 902: `city_flags` 18449 → 18513 → 18449). The bell sends every
/// Citizen within the city's radius to garrison, a worker's order at
/// `QUEUE_FIRST` with `search` 1, and a Citizen past the radius nowhere;
/// the all-clear kills the walking Citizens' GARRISONs, makes the Militia
/// inside a Citizen (`cast_civilian`: `rare` `PEASANTS`) and puts it out
/// at once. Each press marks the economy dirty.
///
/// Made to fail with the bell's worker arm at `QUEUE_NEW`, with the
/// all-clear's `cast_civilian` dropped, with the dirty bit dropped, and
/// with the eject deferred.
#[test]
fn the_city_s_alarm_rings_then_sounds_the_all_clear() {
    use crate::orders::{Body, QueuePos, index};
    let (mut sim, t, citizen, militia) = alarm_sim();
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    let near = sim.init_unit(0, citizen, tile_pos(38, 32));
    let far = sim.init_unit(0, citizen, tile_pos(60, 60));
    let m = sim.init_unit(0, militia, tile_pos(36, 36));
    sim.go_inside(m, b);
    assert_eq!(sim.units[m].inside, Some(b), "a Militia inside the city");
    // A walk already held: the GARRISON goes in front of it.
    sim.add_move_order(
        near,
        tile_pos(39, 33),
        crate::orders::MoveKind::MoveTo,
        QueuePos::New,
        true,
    );
    sim.ledgers[0].dirty = false;

    sim.action_alarm(0, &[b]);
    assert!(sim.cities[c].alarm, "city_flags | 0x40");
    assert!(sim.ledgers[0].dirty, "the press marks the economy dirty");
    let kinds: Vec<u8> = sim.units[near].orders.iter().map(|o| o.index()).collect();
    assert_eq!(kinds[0], index::GARRISON, "in front: {kinds:?}");
    assert!(
        kinds.contains(&index::MOVE_TO),
        "the walk kept behind: {kinds:?}"
    );
    assert!(
        sim.units[near]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Garrison { building, search: true } if building == b))
    );
    assert!(sim.units[far].orders.is_empty(), "past the radius");

    sim.ledgers[0].dirty = false;
    sim.action_alarm(0, &[b]);
    assert!(!sim.cities[c].alarm, "city_flags & ~0x40");
    assert!(sim.ledgers[0].dirty);
    assert!(
        !sim.units[near]
            .orders
            .iter()
            .any(|o| o.index() == index::GARRISON),
        "the walking Citizen's GARRISON is killed"
    );
    let u = &sim.units[m];
    assert_eq!(u.ty, Some(citizen), "the Militia inside is a Citizen");
    assert_eq!((u.rare, u.form), (0x32, 9));
    assert_eq!(u.inside, None, "and out at once");
    assert!(sim.buildings[b].garrison.is_empty());
}

/// **`come_out`'s tail, as `0x617c10` has it** (twenty-fourth pass, group
/// 15; A9 rows 24 and 29): a computer's captain runs the same wipe a member
/// does — its queued orders go — where a human's keeps them; and the city
/// flag after an exit is cleared for a human's city only, and not while a
/// unit of its owner holds a `GARRISON` on the container.
#[test]
fn a_computer_captain_comes_out_clean_and_the_city_flag_waits_for_a_human() {
    use crate::orders::{Body, MoveKind, QueuePos};
    for human in [false, true] {
        let (mut sim, t, _, militia) = alarm_sim();
        sim.nation[0].human = human;
        let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
        let m = sim.init_unit(0, militia, tile_pos(36, 36));
        sim.go_inside(m, b);
        sim.add_move_order(m, tile_pos(50, 50), MoveKind::MoveTo, QueuePos::New, true);
        sim.cities[c].alarm = true;
        assert!(sim.come_out(m));
        assert_eq!(
            sim.units[m].orders.is_empty(),
            !human,
            "human {human}: a computer's captain wipes its orders"
        );
        assert_eq!(
            sim.cities[c].alarm, !human,
            "human {human}: only a human's city flag clears"
        );
    }
    // A human's city whose last garrison leaves while a citizen is on its
    // way in keeps the flag.
    let (mut sim, t, citizen, militia) = alarm_sim();
    sim.nation[0].human = true;
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    let walker = sim.init_unit(0, citizen, tile_pos(40, 32));
    sim.add_garrison_order(walker, b, false, QueuePos::New, true);
    assert!(
        sim.units[walker]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Garrison { building, .. } if building == b))
    );
    let m = sim.init_unit(0, militia, tile_pos(36, 36));
    sim.go_inside(m, b);
    sim.cities[c].alarm = true;
    assert!(sim.come_out(m));
    assert!(sim.cities[c].alarm, "an inbound GARRISON holds the flag");
}

/// **A city takes its owner's first dead slot, not anyone's** (twenty-fourth
/// pass, group 20; A5 #11): `Cities::init_city` has one list per leader, so a
/// leader's cities sort the same after any capture. Another leader's dead
/// record is left where it is.
#[test]
fn a_new_city_reuses_only_its_owner_s_dead_slot() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let (b0, c0) = city_at(&mut sim, &t, 0, 20, 20);
    let (b1, c1) = city_at(&mut sim, &t, 1, 50, 50);
    assert_ne!(c0, c1);
    sim.close_building(b0, false);
    sim.close_building(b1, false);
    assert!(!sim.cities[c0].alive && !sim.cities[c1].alive);
    let (_, again) = city_at(&mut sim, &t, 1, 50, 20);
    assert_eq!(again, c1, "player 1 takes its own dead slot, not the first");
    let (_, back) = city_at(&mut sim, &t, 0, 20, 50);
    assert_eq!(back, c0, "and player 0 its own");
}

/// **`Build::process`'s order, per building, in object-number order**
/// (twenty-fourth pass, group 18; A8 rows 20, 21, 22 and 27): head, the
/// tower, `do_queue`, the gather re-entries, then the city block and the road
/// replan — one building at a time, each leader's by object number and then
/// the walls, not the `Vec`'s creation order.
#[test]
fn the_building_walk_is_per_building_in_object_order() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    // Created high-numbered first, then a lower number of the same leader,
    // then another leader's.
    let (hi, _) = city_at(&mut sim, &t, 1, 50, 50);
    let (own, _) = city_at(&mut sim, &t, 0, 20, 20);
    let lo = sim.place_building(0, t.farm, tile_pos(28, 20)).unwrap();
    sim.buildings[own].index = 2000;
    sim.buildings[hi].index = 2007;
    sim.buildings[lo].index = 2001;
    sim.building_log.clear();
    sim.tick();
    let steps: Vec<(usize, &str)> = sim
        .building_log
        .iter()
        .copied()
        .filter(|(b, _)| *b == hi || *b == lo)
        .collect();
    // Leader 0's building first, whole, and only then leader 1's.
    let names: Vec<&str> = steps.iter().map(|s| s.1).collect();
    assert_eq!(steps.first().map(|s| s.0), Some(lo), "{steps:?}");
    let first_hi = steps.iter().position(|s| s.0 == hi).expect("walked");
    assert!(steps[..first_hi].iter().all(|s| s.0 == lo), "{steps:?}");
    let of = |b: usize| -> Vec<&str> { steps.iter().filter(|s| s.0 == b).map(|s| s.1).collect() };
    for b in [hi, lo] {
        let seq = of(b);
        let order = ["head", "tower", "queue", "gather", "tail"];
        let mut at = 0;
        for step in &seq {
            let k = order.iter().position(|o| o == step).unwrap();
            assert!(k >= at, "{step} out of order in {seq:?}");
            at = k;
        }
        assert!(seq.contains(&"queue"), "{seq:?}");
        // A site (the farm) goes no further than its queue and gather step;
        // a finished city reaches the tail.
        assert_eq!(seq.contains(&"tail"), b == hi, "{seq:?}");
    }
    let _ = names;
}

/// **A unit comes out with the body's speeds it froze at the door**
/// (item 1167, `docs/GOLDEN.md` §49; run430's `0/5` on 902, out of the
/// City at `avg_speed` 11 from the 15 it went in with). `set_new_location
/// (·, ·, 1, 1)` moves the guys and writes neither speed.
///
/// Made to fail with the body rebuilt at rest.
#[test]
fn a_unit_comes_out_with_the_speeds_it_went_in_with() {
    let (mut sim, t, _, militia) = alarm_sim();
    let (b, _) = city_at(&mut sim, &t, 0, 32, 32);
    let m = sim.init_unit(0, militia, tile_pos(36, 36));
    sim.go_inside(m, b);
    sim.units[m].movement.body.avg_speed = 15;
    sim.units[m].movement.body.last_speed = 3;
    assert!(sim.come_out(m));
    let body = sim.units[m].movement.body;
    assert_eq!((body.avg_speed, body.last_speed), (15, 3));
    assert_eq!(
        body.pos, sim.units[m].pos,
        "and it stands where it came out"
    );
}

/// **A builder that takes the gather of the site it finished drops its
/// group pointer** (item 1167, `docs/GOLDEN.md` §49; `Unit::do_build`'s
/// `LAB_005eed5f`, `+0x80 = −1` at `005eed9c`, and `Unit::build_done`'s
/// human gather at `00603cb5`: run430's `0/9` on 1075, `group` 3 → −1).
///
/// Made to fail with the pointer kept.
#[test]
fn a_builder_that_gathers_at_its_finished_site_drops_its_group_pointer() {
    use crate::orders::{Body, QueuePos, index};
    let (mut sim, t, citizen, _) = alarm_sim();
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    sim.nation[0].human = true;
    let farm = sim.place_building(0, t.farm, tile_pos(44, 32)).unwrap();
    finish(&mut sim, farm);
    let u = sim.init_unit(0, citizen, tile_pos(44, 36));
    sim.add_build_order(u, farm, QueuePos::New, true);
    sim.units[u].group_ptr = Some(3);
    sim.tick();
    assert_eq!(
        sim.units[u].orders.front().map(|o| o.index()),
        Some(index::GATHER)
    );
    assert!(matches!(sim.units[u].orders[0].body, Body::Gather { .. }));
    assert_eq!(sim.units[u].group_ptr, None);
}

/// **To Arms is refused on another player's land and taken on no one's**
/// (item 1167, `docs/GOLDEN.md` §49; `is_castable`'s `0x294` case, the
/// cell under the caster no one's, its own or an ally's: run430's D,
/// refused on 612 at cell (46, 26), who=1's, and converted on 800 at
/// (43, 26), no one's).
///
/// Made to fail with the land test dropped.
#[test]
fn to_arms_is_refused_on_another_player_s_land() {
    use crate::orders::spell;
    let (mut sim, [citizen, militia, _, _], militia_t) = untargeted_sim();
    sim.gain_tech(0, militia_t);
    let c = sim.init_unit(0, citizen, tile_pos(20, 20));
    let cell = sim.units[c].pos.cell();
    sim.world.set_owner(cell, Owner::Player(1), Owner::None);
    assert_eq!(press(&mut sim, c, spell::TO_ARMS), 0, "who=1's land");
    assert!(sim.units[c].orders.is_empty());
    sim.world.set_owner(cell, Owner::None, Owner::None);
    assert_eq!(press(&mut sim, c, spell::TO_ARMS), 1, "no one's");
    for _ in 0..5 {
        sim.tick();
    }
    assert_eq!(sim.units[c].ty, Some(militia));
}

/// **A newborn's `form` is `Unit::init`'s** (item 1330, `docs/GROUPS.md`
/// §24.3): 9 for the four civilian ids (`PEASANTS`..`SCHOLARSKOREAN`) and
/// 0 for every other type, where this crate had −1 for both.
///
/// Made to fail with [`crate::init_form`]'s call removed from
/// `Sim::init_unit`.
#[test]
fn a_citizen_is_born_in_the_mob_and_a_soldier_in_the_line() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut ct = citizen_type(t.village);
    ct.type_index = 0x32;
    let citizen = sim.add_unit_type(ct);
    let mut ht = hoplite_type(t.barracks);
    ht.type_index = 0x52;
    let hoplite = sim.add_unit_type(ht);
    let c = sim.init_unit(0, citizen, tile_pos(40, 40));
    let h = sim.init_unit(0, hoplite, tile_pos(44, 40));
    assert_eq!(sim.units[c].form, 9, "a citizen is born in the mob");
    assert_eq!(sim.units[h].form, 0, "a soldier in the line");
    assert_eq!(sim.units[c].form_width, -1, "and the width twin is −1");
    for k in 0x32..=0x35 {
        assert_eq!(crate::init_form(k), 9, "type {k:#x}");
    }
    assert_eq!(crate::init_form(0x31), 0);
    assert_eq!(crate::init_form(0x36), 0);
}

/// **Three Citizens sent together walk three plain moves** (item 1330,
/// run514's `@move 0 3840 36864 9 10 11` on 678): born in form 9, their
/// group's `get_form` is 9, and form 9 is one of the arms that keep a
/// member off the `GroupMoveOrder` (`705f00`–`705f61`). The Mob lays them
/// out on its rings, so no two share a destination.
///
/// Made to fail with the citizens born −1: one group move, and all three
/// sent to the anchor.
#[test]
fn three_citizens_sent_together_walk_three_plain_moves_to_the_mob_s_rings() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let mut ct = citizen_type(t.village);
    ct.type_index = 0x32;
    ct.combat.x_spacing = 144;
    ct.combat.y_spacing = 144;
    let citizen = sim.add_unit_type(ct);
    let us: Vec<usize> = (0..3)
        .map(|i| sim.init_unit(0, citizen, tile_pos(40 + 2 * i, 44)))
        .collect();
    for &u in &us {
        sim.units[u].on_map = true;
    }
    let mut g = crate::group::Group::stack(0);
    for &u in &us {
        sim.group_add(&mut g, u);
    }
    assert!(sim.push_group(&mut g, true));
    sim.group_action_move_to(
        &g,
        tile_pos(24, 50),
        QueuePos::New,
        false,
        movement::Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let mut dests = Vec::new();
    for &u in &us {
        match sim.units[u].orders.front().map(|o| o.body) {
            Some(Body::Move(m)) => {
                assert_eq!(m.kind, MoveKind::MoveTo);
                assert!(m.group.is_none(), "a plain move, not a group move");
                dests.push(m.dest);
            }
            other => panic!("a move, not {other:?}"),
        }
    }
    dests.sort_by_key(|p| (p.x, p.y));
    dests.dedup();
    assert_eq!(dests.len(), 3, "three points on the Mob's rings: {dests:?}");
}

/// **A builder that leads its group turns the group's mirror as it faces
/// the site** (item 1330): `Unit::do_build@005eebf0:155` turns it through
/// `Unit::set_angle`, whose turn of 90° or more toggles `GroupData::facing`
/// of the group it leads. run514's builders' group reads `facing` 1 from
/// block 624, the frame `0/9` turned 122.5° to its Tower site, and the
/// flag is the mirror of the group's next move.
///
/// Made to fail with `do_build`'s turn through `Movement::set_heading`
/// again.
#[test]
fn a_builder_leading_its_group_turns_the_group_s_mirror_as_it_faces_the_site() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    sim.nation[0].human = true;
    let _ = city_at(&mut sim, &t, 0, 32, 32);
    let citizen = sim.add_unit_type(citizen_type(t.village));
    sim.unit_types[citizen].worker = Worker::Citizen;
    let site = sim.place_building(0, t.farm, tile_pos(44, 20)).unwrap();
    assert!(!sim.buildings[site].active);
    let builder = spawn(&mut sim, 0, citizen, tile_pos(43, 20));
    sim.units[builder].on_map = true;
    let mut g = crate::group::Group::stack(0);
    sim.group_add(&mut g, builder);
    assert!(sim.push_group(&mut g, true));
    let g = sim.group_of(builder).expect("the builder's group");
    let before = sim.gstate(&g).expect("a pushed record").facing;
    sim.units[builder].movement.heading = movement::Angle::WEST;
    sim.add_build_order(builder, site, QueuePos::New, true);
    let mut frames = 0;
    while sim.units[builder].movement.heading == movement::Angle::WEST {
        sim.tick();
        frames += 1;
        assert!(frames < 50, "the builder should face its site");
    }
    let g = sim.group_of(builder).expect("still grouped");
    assert_eq!(
        sim.gstate(&g).expect("a pushed record").facing,
        !before,
        "a turn of 90° or more toggles the leader's group's mirror"
    );
}

/// **The census's civilian counts move the frame a unit is born or
/// closed** (item 1398, `docs/AI.md` §103): `Leader::track_unit_type`
/// adds to `peasants` and `scholars` by the type's predicate, so a
/// citizen born between two sweeps is already in the count `create_units`
/// reads. run571's who=1 read 42 against 43 on 11345.
///
/// Made to fail with `track_civilian_type` dropped.
#[test]
fn a_birth_moves_the_census_s_civilian_counts_before_the_sweep() {
    use crate::orders::Worker;
    let (mut sim, [_, _, _, foot], _) = untargeted_sim();
    sim.unit_types[foot].worker = Worker::Citizen;
    sim.track_unit_type(0, foot, 1);
    assert_eq!(sim.ai[0].census.peasants, 1);
    sim.track_unit_type(0, foot, -1);
    assert_eq!(sim.ai[0].census.peasants, 0);
    sim.unit_types[foot].worker = Worker::Scholar;
    sim.track_unit_type(0, foot, 1);
    assert_eq!(
        (sim.ai[0].census.scholars, sim.ai[0].census.peasants),
        (1, 0)
    );
}

/// **A Citizen is ramped by the Militia line too** (item 1455,
/// `docs/COSTS.md`): `get_cost`'s unit ramp counts, for a Citizen,
/// `support(0x42) + support(0x43) − scholar_militia + support(0x44)` beside
/// its own, and for a Scholar `scholar_militia`. French East Indies' AI had
/// one Militia queued on 7782 and paid 52/53/54 for three Citizens where
/// the crate asked 51/52/53.
#[test]
fn a_citizen_is_ramped_by_the_militia_too() {
    let mut sim = world_sim();
    let ty = |sim: &mut Sim, ti: i32| {
        sim.add_unit_type(UnitType {
            hits: 40,
            type_index: ti,
            ..UnitType::default()
        })
    };
    let citizen = ty(&mut sim, 0x32);
    let scholar = ty(&mut sim, 0x34);
    let militia = ty(&mut sim, 0x42);
    let hoplite = ty(&mut sim, 0x52);
    sim.muster[1].by_type[citizen] = 25;
    sim.muster[1].queued_by_type[citizen] = 6;
    sim.muster[1].queued_by_type[militia] = 1;
    assert_eq!(sim.worker_support(1, citizen), 1, "the queued Militia");
    assert_eq!(sim.worker_support(1, scholar), 0);
    assert_eq!(sim.worker_support(1, hoplite), 0, "no other line");
    assert_eq!(sim.worker_support(0, citizen), 0, "the leader's own");
    // A militia that was a scholar (`former_type`, `UnitData +0x54`) is
    // the Scholar's, and comes off the Citizen's.
    let mut u = Unit::new(1, 9, tile_pos(3, 3), 40);
    u.ty = Some(militia);
    u.type_index = 0x42;
    u.rare = 0x34;
    sim.add_unit(u);
    sim.muster[1].by_type[militia] = 1;
    assert_eq!(sim.worker_support(1, citizen), 1, "1 + 1 − 1");
    assert_eq!(sim.worker_support(1, scholar), 1);
}

/// **A French Siege Factory unit moves a fifth faster** (item 1455,
/// `Unit::update_speed@006055c0`, `docs/MOVEMENT.md`): a type whose
/// `WHERE` is `0x1ae` or `0x1af` takes `FRENCH_SIEGE_MOVE` for a French
/// leader. French East Indies' Supply Wagon `1/72` was born at 30 there,
/// 25 here.
#[test]
fn a_french_siege_factory_unit_moves_a_fifth_faster() {
    use crate::tech::{TechTree, TypeDef, UnitTraits as Traits};
    let mut sim = world_sim();
    let mut tree = TechTree::new();
    while tree.types.len() < 0x1ae {
        tree.add(TypeDef::good("filler"));
    }
    let factory = tree.add(TypeDef::building("Siege Factory"));
    assert_eq!(factory, 0x1ae);
    tree.add(TypeDef::building("Factory"));
    let barracks = tree.add(TypeDef::building("Barracks"));
    let wagon_t = tree.add(TypeDef::unit("Supply Wagon", Traits::default()).at(factory));
    let foot_t = tree.add(TypeDef::unit("Hoplites", Traits::default()).at(barracks));
    sim.set_tech_tree(tree);
    let ty = |sim: &mut Sim, t| {
        sim.add_unit_type(UnitType {
            hits: 40,
            moves: 25,
            tree: Some(t),
            ..UnitType::default()
        })
    };
    let wagon = ty(&mut sim, wagon_t);
    let foot = ty(&mut sim, foot_t);
    assert_eq!(sim.type_speed(1, wagon), 25);
    sim.nation[1].french = true;
    assert_eq!(sim.type_speed(1, wagon), 30, "25 × 120 / 100");
    assert_eq!(sim.type_speed(1, foot), 25, "not the factory line");
}

/// **A French siege unit costs `FRENCH_SIEGE_COST` less** (item 1460,
/// `docs/COSTS.md`, "A French siege unit costs less"): `get_cost`'s nation
/// tail takes 15% off a French leader's unit trained at `0x1ae`/`0x1af`,
/// right after `UNIT_COST_FACTOR`. French East Indies' Trebuchet research
/// is 79 there, 94 here before the arm.
#[test]
fn a_french_siege_factory_unit_costs_fifteen_percent_less() {
    use crate::tech::{TechTree, TypeDef, UnitTraits as Traits};
    let mut sim = world_sim();
    let mut tree = TechTree::new();
    while tree.types.len() < 0x1ae {
        tree.add(TypeDef::good("filler"));
    }
    let factory = tree.add(TypeDef::building("Siege Factory"));
    tree.add(TypeDef::building("Factory"));
    let barracks = tree.add(TypeDef::building("Barracks"));
    let siege_t = tree.add(TypeDef::unit("Trebuchet", Traits::default()).at(factory));
    let foot_t = tree.add(TypeDef::unit("Hoplites", Traits::default()).at(barracks));
    sim.set_tech_tree(tree);
    // The train arm: the types' availability bits are set.
    sim.tech[1].tech[siege_t] = true;
    sim.tech[1].tech[foot_t] = true;
    let ty = |sim: &mut Sim, t| {
        sim.add_unit_type(UnitType {
            price: cost::Price::free().with_base(economy::Resource::Metal, 7),
            tree: Some(t),
            ..UnitType::default()
        })
    };
    let siege = ty(&mut sim, siege_t);
    let foot = ty(&mut sim, foot_t);
    let metal = economy::Resource::Metal.index();
    sim.holdings[1].available[metal] = true;
    let scaled = 7 * sim.tuning.unit_cost_factor;
    assert_eq!(sim.price_of(1, siege)[metal], scaled);
    sim.nation[1].french = true;
    assert_eq!(sim.price_of(1, siege)[metal], scaled * 85 / 100);
    assert_eq!(sim.price_of(1, foot)[metal], scaled, "not the factory line");
}

/// **`path_recursion` survives a recycled object slot** (item 1458,
/// `docs/GROUPS.md` §37.2): `Unit::init` never writes `UnitData +0xaf`,
/// so a unit born into a dead unit's object number keeps its count. A
/// fresh number starts at 0.
#[test]
fn a_recycled_slot_keeps_its_path_recursion() {
    let mut sim = world_sim();
    let mut old = Unit::new(1, 9, tile_pos(3, 3), 40);
    old.path_recursion = 3;
    let o = sim.add_unit(old);
    sim.units[o].health = 0;
    let n = sim.add_unit(Unit::new(1, 9, tile_pos(4, 4), 40));
    assert_eq!(sim.units[n].path_recursion, 3, "the slot's last occupant's");
    let fresh = sim.add_unit(Unit::new(1, 10, tile_pos(5, 5), 40));
    assert_eq!(sim.units[fresh].path_recursion, 0);
}

/// **A city at its ceiling charges the shooter's army and is stamped**
/// (item 1528, `docs/AI.md` §142): `Object::do_damage@0064a480`'s tail.
/// After `Build::check_capture` has said no, a hit by another player's
/// object on an active city building whose `damage` has reached `hits(0)`
/// (1) drags a computer leader's **siege** shooter's army onto the city
/// (`Army::charge`: an `ATTACK_TO` laid at the front, the army's target the
/// city) and (2) writes the city's `reduce_stamp` — always on the hit that
/// brought it to the ceiling, and once in 300 frames after. The charge is
/// not behind the cooldown: run668's second siege shot on 15377 charged
/// again. A shooter that is not siege, or sits in no army, or is a human's,
/// stamps and charges nothing more.
///
/// Made to fail by taking the stamp's `fresh` arm out (the second
/// assertion), the `is_siege_unit` test out (the last) and the whole
/// `city_reduced` call out (the first).
#[test]
fn a_city_at_its_ceiling_charges_the_shooters_army_and_is_stamped() {
    let mut sim = world_sim();
    sim.nation[0].human = true;
    sim.nation[1].human = false;
    let t = install_types(&mut sim);
    let (b, c) = city_at(&mut sim, &t, 0, 32, 32);
    let (_, home) = city_at(&mut sim, &t, 1, 4, 4);
    let mut siege_t = hoplite_type(t.barracks);
    siege_t.combat.siege = true;
    let siege_ty = sim.add_unit_type(siege_t);
    let foot_ty = sim.add_unit_type(hoplite_type(t.barracks));
    let shooter = spawn(&mut sim, 1, siege_ty, tile_pos(36, 32));
    let mate = spawn(&mut sim, 1, foot_ty, tile_pos(37, 32));
    let foot = spawn(&mut sim, 1, foot_ty, tile_pos(36, 33));
    for u in [shooter, mate, foot] {
        sim.units[u].on_map = true;
    }
    let slot = sim.init_army(1, Some(home));
    sim.army_add_unit(1, slot, shooter);
    sim.army_add_unit(1, slot, mate);
    assert_eq!(sim.army_of(shooter), Some(slot), "the fixture's army");
    let hit = |s: &mut Sim, by: usize, f: i64| {
        s.frame = f;
        // A capture is never the answer in this test: the stamp holds it off.
        s.cities[c].capture_stamp = f;
        s.do_damage(
            combat::Obj::Unit(by),
            combat::Obj::Building(b),
            movement::Angle(0),
            false,
            256,
            false,
            true,
            f,
        );
    };
    let charged = |s: &Sim| {
        matches!(
            s.order_type(mate),
            orders::index::ATTACK_TO | orders::index::GROUP_ATTACK_TO
        )
    };
    // Not at the ceiling yet: no stamp, no charge.
    sim.buildings[b].damage = 0;
    sim.buildings[b].sync_health();
    hit(&mut sim, shooter, 500);
    assert!(sim.buildings[b].damage < sim.buildings[b].hits_now());
    assert_eq!((sim.cities[c].reduce_stamp, charged(&sim)), (0, false));
    // The hit that brings it to the ceiling.
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits_now() - 1;
    bd.sync_health();
    hit(&mut sim, shooter, 600);
    assert_eq!(sim.buildings[b].damage, sim.buildings[b].hits_now());
    assert_eq!(sim.cities[c].reduce_stamp, 600, "stamped on the frame");
    assert!(charged(&sim), "the army is charged onto the city");
    assert_eq!(
        sim.armies[1].list[slot].target,
        Some(combat::Obj::Building(b)),
        "the army's target is the city"
    );
    // A second hit inside the 300 frames charges again and does not
    // re-stamp: run668's second shot on 15377.
    sim.clear_orders(mate);
    sim.armies[1].list[slot].target = None;
    hit(&mut sim, shooter, 610);
    assert!(charged(&sim), "the charge is not behind the cooldown");
    assert_eq!(
        sim.cities[c].reduce_stamp, 600,
        "the cooldown holds the stamp"
    );
    // And past it the stamp is written again.
    hit(&mut sim, shooter, 901);
    assert_eq!(sim.cities[c].reduce_stamp, 901, "300 frames on");
    // A city healed back below its ceiling and struck to it again inside
    // the 300 frames is stamped at once: `fresh` (`local_70`) skips the
    // cooldown.
    let bd = &mut sim.buildings[b];
    bd.damage = bd.hits_now() - 1;
    bd.sync_health();
    hit(&mut sim, shooter, 910);
    assert_eq!(
        sim.cities[c].reduce_stamp, 910,
        "a fresh blow ignores the 300"
    );
    // A shooter that is not siege stamps and charges nothing more.
    sim.clear_orders(mate);
    hit(&mut sim, foot, 1300);
    assert_eq!(sim.cities[c].reduce_stamp, 1300, "any shooter stamps");
    assert!(!charged(&sim), "a foot soldier charges no army");
}

/// **The Persians' second capital** (`Build::activate@00623e20`, lines
/// 463–475 and 513–518; `docs/AI.md` §149): a Persian leader's second city
/// is a capital too — `0x10` without the founding capital's `0x4000` —
/// while one own capital stands, and the third is not. Any other nation's
/// second city is none.
#[test]
fn a_persian_second_city_is_a_second_capital_and_the_third_is_not() {
    for (tribe, second) in [(11, false), (23, true)] {
        let mut sim = world_sim();
        let t = install_types(&mut sim);
        sim.set_tribe(0, tribe);
        // Three cities: past the limit a bare tree allows.
        sim.tech[0].epoch[tech::Line::Civic as usize] = 3;
        let (_, c0) = city_at(&mut sim, &t, 0, 8, 8);
        let (_, c1) = city_at(&mut sim, &t, 0, 33, 8);
        let (_, c2) = city_at(&mut sim, &t, 0, 33, 33);
        assert!(sim.cities[c0].capital && sim.cities[c0].founding_capital);
        assert_eq!(
            sim.cities[c1].capital, second,
            "tribe {tribe}: the second city is a capital for the Persians alone"
        );
        assert!(
            !sim.cities[c1].founding_capital,
            "and never the founding one"
        );
        assert!(
            !sim.cities[c2].capital,
            "tribe {tribe}: the third city is none"
        );
    }
}

/// **A gather enhancer stays in the city it is placed for** (item 1565,
/// `docs/AI.md` §156). `Leader::produce_building`'s spiral, for a type
/// `is_gather_enhancer` answers, drops every candidate whose
/// `BuildTypeData::get_town` is not the city the order is for (`006e20eb`:
/// the covering city's building `o` against `near`) — so a Granary or a
/// Refinery for one city is never seated in the next one's catchment, where
/// the Persians' Refinery `1/2045` stood at y 22176, city 1, in ours and at
/// 19872, city 2, in the original.
///
/// Made to fail once with the `get_town` comparison removed: the Granary for
/// the first city stands in the second's.
#[test]
fn an_enhancer_is_seated_in_the_city_it_is_placed_for() {
    let mut sim = world_sim();
    let t = install_types(&mut sim);
    sim.tech[0].epoch[tech::Line::Civic as usize] = 2;
    let (b0, c0) = city_at(&mut sim, &t, 0, 8, 8);
    let (_, c1) = city_at(&mut sim, &t, 0, 33, 8);
    assert_ne!(c0, c1);
    for (near, city) in [(b0, c0), (sim.cities[c1].building, c1)] {
        let before = sim.buildings.len();
        assert!(sim.produce_building(0, t.granary, near, Some(city), false));
        let g = sim.buildings.len() - 1;
        assert!(g >= before, "a Granary was placed");
        assert_eq!(
            sim.buildings[g].city,
            Some(city),
            "the Granary for city {city} at {:?} stands in its own city",
            sim.buildings[g].pos
        );
    }
}
