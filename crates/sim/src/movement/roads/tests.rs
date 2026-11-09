use super::*;
use crate::Tuning;
use crate::build::BuildType;
use crate::world::{Terrain, World};

/// A 20×20-cell land world with a finished 7×7 city of player 0 at
/// `centre` (in tiles) and a 3×3 library type ready to place.
fn town(centre: Pos) -> (Sim, usize, usize, usize) {
    let mut w = World::new(20, 20);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 19));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; crate::economy::RESOURCES];
    }
    // The city's own `7x7 extra space` mask: it stands on seven tiles
    // each way and blocks only the inner five, which is what leaves its
    // ring's outer row and column free for a road (`docs/ROADS.md` §3).
    let mut mask = vec![0u8; 49];
    for v in 1..6 {
        for u in 1..6 {
            mask[v * 7 + u] = 1;
        }
    }
    let city_ty = sim.add_build_type(BuildType {
        ident: Ident::Village,
        flags: flags::parse("ean"),
        x_size: 7,
        y_size: 7,
        job_time: 1,
        hits: 1200,
        block_mask: mask,
        ..BuildType::default()
    });
    let lib_ty = sim.add_build_type(BuildType {
        ident: Ident::Library,
        flags: flags::parse("a"),
        x_size: 3,
        y_size: 3,
        job_time: 1,
        hits: 400,
        ..BuildType::default()
    });
    let city = sim
        .place_building(0, city_ty, centre_of(centre))
        .unwrap_or_else(|e| panic!("the city should place: {e:?}"));
    finish(&mut sim, city);
    (sim, city, city_ty, lib_ty)
}

fn finish(sim: &mut Sim, b: usize) {
    let mut guard = 0;
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
        guard += 1;
        assert!(guard < 10, "a million a frame should finish anything");
    }
    sim.buildings[b].helpers = 0;
}

fn library_at(sim: &mut Sim, lib_ty: usize, t: Pos) -> usize {
    let b = sim
        .place_building(0, lib_ty, centre_of(t))
        .unwrap_or_else(|e| panic!("the library should place: {e:?}"));
    finish(sim, b);
    b
}

fn is_road(sim: &Sim, x: i32, y: i32) -> bool {
    sim.world.tile_mask(Pos::new(x, y)) & tile::SURFACE == tile::SURFACE_ROAD
}

#[test]
fn a_gatherer_neither_rings_nor_roads_and_a_university_does_both() {
    let types = vec![
        BuildType {
            ident: Ident::Farm,
            flags: flags::GATHER,
            ..BuildType::default()
        },
        BuildType {
            ident: Ident::University,
            flags: flags::GATHER,
            ..BuildType::default()
        },
        BuildType {
            ident: Ident::Library,
            ..BuildType::default()
        },
        BuildType {
            ident: Ident::Other,
            flags: flags::NO_CITY,
            ..BuildType::default()
        },
    ];
    assert!(!connects_to_roads(&types, 0), "a farm does not connect");
    assert!(connects_to_roads(&types, 1), "a university does");
    assert!(connects_to_roads(&types, 2), "a library does");
    assert!(!connects_to_roads(&types, 3), "a no-city type does not");
}

/// `Wall::start@0063e810` passes `REGEN_FORCE` to `mask_me`, whose
/// tail is `place_roads` — so the ring and the road are laid the moment
/// a building **starts**, and the flag `City::regen_roads` sets is what
/// makes it happen *again* later. Before run32 nothing here modelled
/// that, because no traced game had placed a non-farm building: the
/// setup's own go up before the first frame and the AI never got past
/// its citizens (`docs/QUEUE.md` item 74).
#[test]
fn a_building_lays_its_road_when_it_starts_not_when_its_flag_comes_round() {
    let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let before = sim.rng.seed;
    // Placed, not yet started: nothing is laid and nothing is drawn.
    let b = sim
        .place_building(0, lib_ty, centre_of(Pos::new(40, 52)))
        .expect("the library places");
    assert!(!sim.buildings[b].started, "a site is not started");
    assert!(!is_road(&sim, 40, 52), "an unstarted site has laid no ring");
    assert_eq!(sim.rng.seed, before, "and taken no draw");
    // Starting it lays the ring and plans the road in one go.
    sim.start_building(b);
    assert!(is_road(&sim, 38, 50), "the ring is down");
    assert!(
        sim.rng.seed != before,
        "and the search has drawn its jitter a node"
    );
    // The road is a *path* between the two rings, not a column: since
    // 2026-09-01 a blocked tile puts `BAD_PATH` on its eight
    // neighbours (`World::set_blocked_at`, `docs/PATHFINDER.md` §5),
    // which costs `weight::GROUND` and pushes the route a tile off the
    // straight line it used to take. What is asserted is that it runs.
    let road: Vec<(i32, i32)> = (44..52)
        .flat_map(|y| (37..44).map(move |x| (x, y)))
        .filter(|&(x, y)| is_road(&sim, x, y))
        .collect();
    assert!(
        road.len() >= 4 && road.iter().any(|&(_, y)| y <= 46),
        "a road runs back towards the city: {road:?}"
    );
}

#[test]
fn the_flag_fires_on_the_frame_its_object_number_picks_out() {
    let (mut sim, city, _, _) = town(Pos::new(40, 40));
    // `Build::activate` has already flagged the city through
    // `City::regen_roads`.
    assert!(sim.buildings[city].regen_roads, "activation flags the city");
    let o = i64::from(sim.buildings[city].index);
    for frame in 0..64 {
        let due = (frame + o).rem_euclid(ROTATION) == 0;
        sim.buildings[city].regen_roads = true;
        sim.regen_roads_due(city, frame);
        assert_eq!(
            sim.buildings[city].regen_roads, !due,
            "frame {frame}: object {o} fires when (frame + o) % 16 == 0"
        );
    }
}

/// `Build::process@0061edf0`'s `is_active` gate — `field_0x8 & 4`,
/// which is `WallData::is_active@00472350` inlined — stands above the
/// replan at the function's tail, so a **site** never replans however
/// its flag was set, and the flag is not cleared either.
///
/// East Indies 8193 is the frame that said so: `1/2015`, a half-built
/// market, spent two hundred `calc_road_cost` draws the original never
/// spends (`docs/ROADS.md` §1.1, item 334).
#[test]
fn a_site_does_not_replan_and_keeps_its_flag_until_it_finishes() {
    let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let b = sim
        .place_building(0, lib_ty, centre_of(Pos::new(40, 52)))
        .expect("the library places");
    sim.start_building(b);
    assert!(!sim.buildings[b].active, "still a site");
    let o = i64::from(sim.buildings[b].index);
    // Its own rotation slot, the frame `regen_roads_due` would fire on.
    let due = (ROTATION - o).rem_euclid(ROTATION);
    sim.buildings[b].regen_roads = true;
    let before = sim.rng.seed;
    sim.process_building_whole(b, due);
    assert_eq!(sim.rng.seed, before, "a site spends no road-cost draw");
    assert!(
        sim.buildings[b].regen_roads,
        "and the flag waits rather than being cleared"
    );
    // Finished, the same slot fires — so it is the gate that held it,
    // not the schedule.
    finish(&mut sim, b);
    sim.buildings[b].regen_roads = true;
    sim.process_building_whole(b, due);
    assert!(
        sim.rng.seed != before,
        "an active building replans on that same frame"
    );
    assert!(!sim.buildings[b].regen_roads, "and clears its flag");
}

#[test]
fn the_ring_is_the_border_of_the_footprint_grown_by_one() {
    let (mut sim, city, _, _) = town(Pos::new(40, 40));
    sim.place_roads(city);
    // A 7×7 city at tile (40,40) has its corner at (37,37); the ring
    // runs from (36,36) to (36+7, 36+7) = (43,43), border only.
    for i in 36..=43 {
        assert!(is_road(&sim, i, 36), "top edge at {i}");
        assert!(is_road(&sim, i, 43), "bottom edge at {i}");
        assert!(is_road(&sim, 36, i), "left edge at {i}");
        assert!(is_road(&sim, 43, i), "right edge at {i}");
    }
    assert!(!is_road(&sim, 40, 40), "the footprint itself is not a road");
    assert!(!is_road(&sim, 35, 35), "nor anything outside the ring");
}

/// **The ring goes with the building** (item 3314, `docs/ROADS.md`
/// §13): `mask_me`'s unmask ends in `place_roads(…, 0, REGEN_SIMPLE)`,
/// so a closed Library's ring is handed back through `set_road_at(…,
/// 0, 0, 0)` — and the road it planned to the centre, which only
/// `param_5 != 0` reaches, stays.
#[test]
fn a_closed_building_takes_its_ring_road_with_it_and_leaves_its_road() {
    let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let b = library_at(&mut sim, lib_ty, Pos::new(40, 52));
    // A 3×3 library at (40,52) has its corner at (39,51): the ring runs
    // (38,50) to (41,53), border only.
    let ring: Vec<(i32, i32)> = (38..=41)
        .flat_map(|x| (50..=53).map(move |y| (x, y)))
        .filter(|&(x, y)| (x == 38 || x == 41 || y == 50 || y == 53) && is_road(&sim, x, y))
        .collect();
    let road: Vec<(i32, i32)> = (44..50)
        .flat_map(|y| (34..47).map(move |x| (x, y)))
        .filter(|&(x, y)| is_road(&sim, x, y))
        .collect();
    // The box's bottom row and right column are the footprint's own,
    // blocked here, so the ring is its top row and left column.
    assert_eq!(ring.len(), 7, "the ring is down: {ring:?}");
    assert!(!road.is_empty(), "and a road runs back towards the city");
    sim.close_building(b, false);
    let left: Vec<_> = ring.iter().filter(|&&(x, y)| is_road(&sim, x, y)).collect();
    assert!(left.is_empty(), "the ring went with it: {left:?} stand");
    let kept: Vec<_> = road
        .iter()
        .filter(|&&(x, y)| !is_road(&sim, x, y))
        .collect();
    assert!(
        kept.is_empty(),
        "and the road to the centre stays: {kept:?} went"
    );
}

#[test]
fn laying_a_road_is_idempotent_and_takes_an_ocean_tile_to_land() {
    let (mut sim, _, _, _) = town(Pos::new(40, 40));
    let t = Pos::new(3, 3);
    sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
    sim.set_road_at(t);
    assert_eq!(sim.world.tile_mask(t), tile::SURFACE_ROAD);
    assert!(sim.world.cell_data(Cell::new(0, 0)).flags & cell::ROAD != 0);
    let before = sim.world.tile_mask(t);
    sim.set_road_at(t);
    assert_eq!(sim.world.tile_mask(t), before, "twice is once");
}

#[test]
fn an_endpoint_is_the_city_s_tile_and_any_other_type_s_far_corner() {
    let (mut sim, city, city_ty, lib_ty) = town(Pos::new(40, 40));
    let lib = library_at(&mut sim, lib_ty, Pos::new(34, 40));
    assert_eq!(
        sim.road_end(city, city_ty),
        Pos::new(40, 40),
        "a city answers its own tile"
    );
    // A 3×3 library at (34,40) has its corner at (33,39); its endpoint
    // is `corner + 3 − 1` = (35,41).
    assert_eq!(sim.road_end(lib, lib_ty), Pos::new(35, 41));
}

#[test]
fn a_road_is_planned_between_two_buildings_and_laid_where_it_is_not_blocked() {
    let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let lib = library_at(&mut sim, lib_ty, Pos::new(33, 40));
    let road = sim.find_road(lib, city);
    assert!(!road.is_empty(), "a road is found on open ground");
    // Every step of the chain is a cardinal move of one tile.
    let tiles: Vec<Pos> = road.iter().map(|p| p.tile()).collect();
    for w in tiles.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert_eq!(
            (a.x - b.x).abs() + (a.y - b.y).abs(),
            1,
            "the road walks cardinals only: {a:?} to {b:?}"
        );
    }
    // It runs between the two endpoints, exclusive of both.
    let s = sim.road_end(lib, lib_ty);
    assert!(!tiles.contains(&s) && !tiles.contains(&Pos::new(40, 40)));
}

/// §8.1: `param_6 < 0` gates the odd wheel indices out, and a caravan
/// does not have it.
#[test]
fn a_caravan_expands_the_diagonals_and_a_building_does_not() {
    let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let lib = library_at(&mut sim, lib_ty, Pos::new(33, 33));
    let dirs = |sim: &mut Sim, caravan: bool| -> Vec<i32> {
        sim.trace_costs = true;
        sim.road_marks.clear();
        let mut st = sim.start_road(lib, city).expect("two endpoints");
        let _ = sim.step_road(&mut st, (lib, city), caravan, false);
        let mut d: Vec<i32> = sim.road_marks.iter().map(|m| m.dir).collect();
        d.sort_unstable();
        d.dedup();
        d
    };
    let building = dirs(&mut sim, false);
    assert!(
        building.iter().all(|d| d % 2 == 0),
        "a building's road is cardinal-only: {building:?}"
    );
    let caravan = dirs(&mut sim, true);
    assert!(
        caravan.iter().any(|d| d % 2 != 0),
        "a caravan's takes the diagonals too: {caravan:?}"
    );
}

/// §8.3: the budget parks the search rather than throwing it away, and
/// the frame after carries on from it. Made to fail by returning
/// [`RoadPlan::None`] instead of `Budget`.
#[test]
fn a_caravan_s_search_stops_at_the_budget_and_the_next_frame_resumes_it() {
    // A world wide enough that 3,200 nodes do not reach the goal.
    let mut w = World::new(60, 60);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    for l in &mut sim.ledgers {
        l.bucket = [10_000; crate::economy::RESOURCES];
    }
    let ty = sim.add_build_type(BuildType {
        ident: Ident::Library,
        flags: flags::parse("a"),
        x_size: 1,
        y_size: 1,
        job_time: 1,
        hits: 100,
        ..BuildType::default()
    });
    let a = sim.init_build(0, ty, centre_of(Pos::new(8, 8)), false);
    let b = sim.init_build(0, ty, centre_of(Pos::new(200, 200)), false);
    sim.trace_costs = true;
    let mut st = sim.start_road(a, b).expect("two endpoints");
    let plan = sim.step_road(&mut st, (a, b), true, false);
    assert_eq!(plan, RoadPlan::Budget, "3,200 nodes do not span the map");
    let first = sim.road_marks.len();
    assert!(
        (WORK_CAP as usize..=WORK_CAP as usize + 8).contains(&first),
        "the budget trips one pop past 0xc80: {first}"
    );
    let seen: std::collections::BTreeSet<(i32, i32)> =
        sim.road_marks.iter().map(|m| m.from).collect();
    sim.road_marks.clear();
    let plan = sim.step_road(&mut st, (a, b), true, false);
    assert_eq!(plan, RoadPlan::Budget, "and the second frame is another");
    let then: std::collections::BTreeSet<(i32, i32)> =
        sim.road_marks.iter().map(|m| m.from).collect();
    assert!(
        sim.road_marks.len() >= WORK_CAP as usize,
        "the resumed frame gets the whole budget again, not the rest of one"
    );
    assert!(
        then.intersection(&seen).count() < then.len(),
        "and it expands nodes the first frame did not"
    );
}

/// **Between two regions, only the step off land pays the sea**
/// (`calc_road_cost@00686300`, `docs/CARAVAN.md` §11, item 1115). With
/// `pathfinder +0x8c` clear, `+0xa8` is added when the node's parent
/// stands on land; a sea tile reached from the sea costs the base and
/// the jitter. With it set (both ends in one region), every sea tile
/// pays `+0xa4`. run413's packet holds East Indies' AI route to its
/// second island node for node with this, and 171 of 1,437 nodes
/// apart without it.
#[test]
fn a_sea_crossing_between_regions_pays_once_at_the_shore() {
    let (mut sim, ..) = town(Pos::new(40, 40));
    for x in 50..60 {
        sim.world
            .set_tile_mask(Pos::new(x, 20), tile::SURFACE_OCEAN);
    }
    let sea = centre_of(Pos::new(52, 20));
    let (off_sea, off_land) = (centre_of(Pos::new(51, 20)), centre_of(Pos::new(49, 20)));
    let price = |from: Pos, avoid_sea: bool| {
        let mut s = sim.clone();
        s.calc_road_cost(sea, from, 0, (0, 0), 2, avoid_sea).0
    };
    assert_eq!(
        price(off_land, false) - price(off_sea, false),
        weight::SEA_ENTER,
        "the shore step pays"
    );
    let jitter = price(off_sea, false) - weight::BASE;
    assert!(
        (0..0x14).contains(&jitter),
        "a sea step is the base and the jitter"
    );
    assert_eq!(
        price(off_sea, true),
        price(off_land, true),
        "one region: every sea tile pays"
    );
    assert_eq!(
        price(off_sea, true) - price(off_sea, false),
        weight::SEA_AVOIDED
    );
}

#[test]
fn the_search_refuses_ocean_and_answers_nothing_across_it() {
    let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
    sim.plan_roads = true;
    let lib = library_at(&mut sim, lib_ty, Pos::new(33, 40));
    for y in 0..80 {
        sim.world
            .set_tile_mask(Pos::new(37, y), tile::SURFACE_OCEAN);
    }
    assert!(
        sim.find_road(lib, city).is_empty(),
        "an unbroken sea wall leaves no road"
    );
}
