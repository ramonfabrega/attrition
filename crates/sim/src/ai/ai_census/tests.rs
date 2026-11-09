use super::*;
use crate::build::{BuildType, Ident, flags};
use crate::garrison::UnitTraits;
use crate::orders::{GatherOrder, Order};
use crate::world::{TILES_PER_CELL, UNITS_PER_TILE, World};
use crate::{Tuning, Unit, UnitType, cost, economy};

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(
        tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

fn bt(ident: Ident, from: Option<usize>, flag: &str, xs: i32, ys: i32) -> BuildType {
    BuildType {
        ident,
        from,
        x_size: xs,
        y_size: ys,
        flags: flags::parse(flag),
        job_time: 100,
        hits: 500,
        garrison_max: 10,
        price: cost::Price {
            kind: cost::Kind::Building,
            ..cost::Price::free().with_base(economy::Resource::Timber, 1)
        },
        ..BuildType::default()
    }
}

struct Fix {
    sim: Sim,
    village: usize,
    farm: usize,
    woodcutter: usize,
    university: usize,
    tower: usize,
    fort: usize,
    dock: usize,
    citizen: usize,
    scholar: usize,
    scout: usize,
}

/// A 16×16-cell land world with a sea strip on the right, one AI
/// player and one human, and the handful of types the census reads.
fn fix() -> Fix {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(11, 15));
    w.fill_region(Terrain::Sea, Cell::new(12, 0), Cell::new(15, 15));
    // The sea cells' tiles read as ocean, so `tregion_alt` and the dock
    // scan see water where the regions say water.
    for tx in 48..64 {
        for ty in 0..64 {
            w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
        }
    }
    let mut sim = Sim::new(Tuning::RON, w, 2);
    sim.cities_state.nation[1].human = false;
    // Room for six cities, so the escrow and thin-region tests can found
    // more than one.
    sim.tech[1].epoch[crate::tech::Line::Civic as usize] = 5;
    for l in &mut sim.ledgers {
        l.bucket = [10_000; economy::RESOURCES];
    }
    let village = sim.add_build_type(bt(Ident::Village, None, "ean", 7, 7));
    let mut farm = bt(Ident::Farm, None, "gda", 4, 4);
    farm.flags |= flags::FLAT;
    let farm = sim.add_build_type(farm);
    let woodcutter = sim.add_build_type(bt(Ident::Woodcutter, None, "ga", 3, 3));
    let university = sim.add_build_type(bt(Ident::University, None, "ga", 4, 4));
    let mut tower = bt(Ident::Tower, None, "ean", 2, 2);
    tower.attack = 12;
    let tower = sim.add_build_type(tower);
    let mut fort = bt(Ident::Fort, None, "ean", 5, 5);
    fort.attack = 20;
    let fort = sim.add_build_type(fort);
    let mut dock = bt(Ident::Dock, None, "eab", 4, 4);
    dock.attack = 8;
    let dock = sim.add_build_type(dock);

    let base = UnitType {
        price: cost::Price {
            pop: 1,
            ..cost::Price::free()
        },
        hits: 40,
        garrison: UnitTraits {
            trained_at: Some(village),
            ..UnitTraits::default()
        },
        ..UnitType::default()
    };
    let citizen = sim.add_unit_type(UnitType {
        worker: Worker::Citizen,
        ..base.clone()
    });
    let scholar = sim.add_unit_type(UnitType {
        worker: Worker::Scholar,
        ..base.clone()
    });
    let scout = sim.add_unit_type(base);
    Fix {
        sim,
        village,
        farm,
        woodcutter,
        university,
        tower,
        fort,
        dock,
        citizen,
        scholar,
        scout,
    }
}

/// Step 9 (`docs/AI.md` §55): every good in `new_rares` counts into its
/// cell's region unless the cell is another leader's and that leader is
/// not a mutual ally — and only the census writes the array, while the
/// leader-level sum waits for `calc_gather`'s recompute.
/// **The census's territory is the leader's land count** (item 3516,
/// `LeaderData::territory`, the field `get_team_terr@006d62e0` sums):
/// a sea cell the map's owner layer gives the leader is not in it.
/// run1162d's who=0 census read who=1's team as 462 where the
/// original held 437.
#[test]
fn the_census_territory_is_the_land_count_not_the_owned_cells() {
    let Fix { mut sim, .. } = fix();
    let p = Owner::Player(1);
    sim.world.set_owner(Cell::new(3, 3), p, Owner::None);
    sim.world.set_owner(Cell::new(13, 3), p, Owner::None);
    sim.holdings[1].territory = 1;
    assert_eq!(sim.territory_of(1), 1, "the land cell, not the sea one");
    assert_eq!(sim.team_terr(1), 1);
}

#[test]
fn step_9_counts_the_seen_rares_a_merchant_may_reach() {
    let mut f = fix();
    let s = &mut f.sim;
    let good = |s: &mut Sim, x: i32, y: i32| {
        s.world.add_good(crate::world::Good {
            pos: tile_pos(x * TILES_PER_CELL, y * TILES_PER_CELL),
            ty: 20,
            alive: true,
        })
    };
    let unowned = good(s, 2, 2);
    let mine = good(s, 4, 4);
    let theirs = good(s, 6, 6);
    s.world
        .set_owner(Cell::new(4, 4), Owner::Player(1), Owner::None);
    s.world
        .set_owner(Cell::new(6, 6), Owner::Player(0), Owner::None);
    s.ai[1].new_rares = vec![unowned, mine, theirs];
    let land = s.world.region_of(Cell::new(2, 2)).expect("land") as usize;

    s.census(1);
    assert_eq!(
        s.ai[1].census.reg_known_rares[land], 2,
        "unowned and own ground count; the human's does not"
    );
    assert_eq!(
        s.ai[1].known_rares, 0,
        "the census writes the array, not the sum"
    );
    s.assemble_holdings(1);
    assert_eq!(s.ai[1].known_rares, 2, "calc_gather's recompute sums it");

    // A mutual ally's ground counts; a one-sided alliance does not.
    s.allied[1][0] = true;
    s.census(1);
    assert_eq!(
        s.ai[1].census.reg_known_rares[land], 2,
        "one side is not an ally"
    );
    s.allied[0][1] = true;
    s.census(1);
    assert_eq!(s.ai[1].census.reg_known_rares[land], 3, "both sides are");

    // And the sweep zeroes before it counts, so a list that shrank
    // would not leave a stale count behind.
    s.ai[1].new_rares.clear();
    s.census(1);
    assert_eq!(s.ai[1].census.reg_known_rares[land], 0);
}

/// A border fix between the census and `calc_gather`'s recompute
/// zeroes the array, so the recompute sums 0 and the Merchant arm is
/// shut until the next sweep counts again — East Indies 18182's make
/// list (`docs/AI.md` §76). `Region::fix_borders` →
/// `check_borders` → `compute_reg_territory:76–105`.
#[test]
fn a_border_fix_zeroes_the_rares_until_the_next_census() {
    let mut f = fix();
    let s = &mut f.sim;
    let g = s.world.add_good(crate::world::Good {
        pos: tile_pos(2 * TILES_PER_CELL, 2 * TILES_PER_CELL),
        ty: 20,
        alive: true,
    });
    s.ai[1].new_rares = vec![g];
    let land = s.world.region_of(Cell::new(2, 2)).expect("land") as usize;
    s.in_play = true;
    s.census(1);
    assert_eq!(s.ai[1].census.reg_known_rares[land], 1, "counted");
    s.sync_territory();
    assert_eq!(
        s.ai[1].census.reg_known_rares[land], 1,
        "the fix only marks the regions"
    );
    s.check_borders();
    assert_eq!(
        s.ai[1].census.reg_known_rares[land], 0,
        "the next check_borders zeroes every region"
    );
    s.assemble_holdings(1);
    assert_eq!(s.ai[1].known_rares, 0, "the recompute sums the zero");
    s.census(1);
    s.assemble_holdings(1);
    assert_eq!(s.ai[1].known_rares, 1, "the next sweep counts it again");
}

/// **A setup fix leaves frame 0's census count standing** (item 3170,
/// `docs/AI.md` §250): the original's setup ends in
/// `World::compute_all_territory`, so no region's pass is pending when
/// the first `check_borders` runs. run923's who=1 keeps its start-seen
/// Silk in `reg_known_rares` from block 1.
#[test]
fn a_setup_fix_leaves_frame_0_s_rares_standing() {
    let mut f = fix();
    let s = &mut f.sim;
    let g = s.world.add_good(crate::world::Good {
        pos: tile_pos(2 * TILES_PER_CELL, 2 * TILES_PER_CELL),
        ty: 20,
        alive: true,
    });
    s.ai[1].new_rares = vec![g];
    let land = s.world.region_of(Cell::new(2, 2)).expect("land") as usize;
    assert!(!s.in_play, "the fixture stands before the first frame");
    s.sync_territory();
    s.census(1);
    s.check_borders();
    assert_eq!(
        s.ai[1].census.reg_known_rares[land], 1,
        "setup's fix marked nothing for frame 0's check_borders"
    );
    s.assemble_holdings(1);
    assert_eq!(s.ai[1].known_rares, 1, "and the recompute sums it");
}

fn finish(sim: &mut Sim, b: usize) {
    let mut guard = 0;
    while !sim.buildings[b].active && sim.buildings[b].alive {
        sim.do_construct(b, 1_000_000);
        guard += 1;
        assert!(guard < 10);
    }
    sim.buildings[b].helpers = 0;
}

fn build(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
    let b = sim
        .place_building(who, ty, tile_pos(tx, ty_))
        .unwrap_or_else(|e| panic!("placing at ({tx},{ty_}) should work: {e:?}"));
    finish(sim, b);
    b
}

/// A finished building placed straight into the world, past the
/// placement rules — the only way to stand a dock on this test map.
fn force_build(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
    let b = sim.init_build(who, ty, tile_pos(tx, ty_), false);
    sim.activate(b, false, true);
    b
}

fn spawn(sim: &mut Sim, who: Player, ty: usize, tx: i32, ty_: i32) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.tables.unit_types[ty].hits;
    let mut u = Unit::new(who, index, tile_pos(tx, ty_), hits);
    u.ty = Some(ty);
    sim.add_unit(u)
}

fn gather_order(building: usize) -> Order {
    Order {
        flags: crate::orders::flag::ACTION,
        body: Body::Gather(GatherOrder {
            building,
            tile: None,
            wait: 0,
            goto_build: false,
            dist_mod: 0,
            been_there: true,
        }),
    }
}

/// The AI leader's city, five citizens on farms and woodcutters, one
/// scout — run8's shape. The free/busy split is by the **action**'s
/// kind: no action or EXPLORE_TO is free, everything else is busy, and
/// a GATHER moves the count on to the target building's city.
/// **A citizen on a barge is off the map** (item 592, `docs/AI.md` §58):
/// neither free nor busy nor in `reg_peasants`, and — while the barge's
/// order is a move — an `xport` peasant in the region of the move's
/// point, which is also where the `reg_active` family counts it, because
/// the original keeps that region in the local the unit's own region
/// lives in. run143's 10576 is the case: `1/31` rides `1/36` and the
/// original counts it in `reg_active[5]` and `reg_xport_peasants[5]`.
#[test]
fn a_rider_is_an_xport_peasant_where_its_boat_is_going() {
    let mut f = fix();
    // Two land regions: the boarding shore and the far shore.
    let home = f.sim.world.region_of(Cell::new(2, 2)).unwrap();
    let far = f
        .sim
        .world
        .fill_region(Terrain::Land, Cell::new(0, 10), Cell::new(11, 15));
    let sea = f.sim.world.region_of(Cell::new(13, 4)).unwrap();
    // A city at home, so the citizen ashore is not in transport itself.
    build(&mut f.sim, 1, f.village, 20, 20);
    let mut barge = UnitType {
        hits: 50,
        ..UnitType::default()
    };
    barge.combat.domain = Domain::Sea;
    let barge = f.sim.add_unit_type(barge);
    let boat = spawn(
        &mut f.sim,
        1,
        barge,
        13 * TILES_PER_CELL,
        4 * TILES_PER_CELL,
    );
    let rider = spawn(
        &mut f.sim,
        1,
        f.citizen,
        2 * TILES_PER_CELL,
        2 * TILES_PER_CELL,
    );
    spawn(
        &mut f.sim,
        1,
        f.citizen,
        3 * TILES_PER_CELL,
        3 * TILES_PER_CELL,
    );
    f.sim.units[rider].inside_unit = Some(boat);
    f.sim.units[rider].on_map = false;
    let sail = |dest: Pos| Order {
        flags: crate::orders::flag::ACTION,
        body: Body::Move(crate::orders::MoveOrder {
            kind: crate::orders::MoveKind::MoveTo,
            dest,
            angle: crate::movement::Angle(0),
            facing: None,
            has_waypoint: false,
            waypoint: dest,
            last: None,
            pause: 0,
            retry: 0,
            attempts: 0,
            timer: 0,
            coll: None,
            orig: None,
            group: None,
        }),
    };
    f.sim.units[boat]
        .orders
        .push_back(sail(tile_pos(4 * TILES_PER_CELL, 12 * TILES_PER_CELL)));

    f.sim.census(1);
    let cs = &f.sim.ai[1].census;
    let reg = |v: &[i32], r: u16| Census::reg(v, r);
    assert_eq!(cs.peasants, 2, "the rider is a peasant");
    assert_eq!(cs.free_peasants, 1, "only the one ashore is free");
    assert_eq!(cs.xport_peasants, 1, "the rider is in transport");
    assert_eq!(reg(&cs.reg_xport_peasants, far), 1, "where the boat goes");
    assert_eq!(reg(&cs.reg_peasants, home), 1, "a rider is not on the map");
    assert_eq!(reg(&cs.reg_free_peasants, home), 1);
    assert_eq!(
        (reg(&cs.reg_active, home), reg(&cs.reg_active, far)),
        (1, 1),
        "the rider's `reg_active` is the move point's region"
    );

    // A boat at rest carries no `xport`: the rider counts in the boat's
    // own tile's region, the sea, where a land unit is not active.
    f.sim.units[boat].orders.clear();
    f.sim.census(1);
    let cs = &f.sim.ai[1].census;
    assert_eq!((cs.free_peasants, cs.xport_peasants), (1, 0));
    assert_eq!(reg(&cs.reg_active, far), 0);
    assert_eq!(reg(&cs.reg_active, sea), 0);
    assert_eq!(reg(&cs.reg_active, home), 1, "only the one ashore");
}

#[test]
fn the_free_busy_split_follows_the_action_kind() {
    let mut f = fix();
    let city_b = build(&mut f.sim, 1, f.village, 20, 20);
    let c = f.sim.buildings[city_b].city.expect("a city record");
    let farm_b = build(&mut f.sim, 1, f.farm, 26, 20);
    f.sim.buildings[farm_b].gather_max = Some(3);
    f.sim.plant_camp_forest(tile_pos(20, 26));
    let wood_b = build(&mut f.sim, 1, f.woodcutter, 20, 26);
    f.sim.buildings[wood_b].gather_max = Some(5);

    // Three gathering on the farm, two on the woodcutter.
    for k in 0..3 {
        let u = spawn(&mut f.sim, 1, f.citizen, 26 + k, 21);
        f.sim.units[u].orders.push_back(gather_order(farm_b));
    }
    for k in 0..2 {
        let u = spawn(&mut f.sim, 1, f.citizen, 21 + k, 26);
        f.sim.units[u].orders.push_back(gather_order(wood_b));
    }
    // One idle citizen and a scout that is exploring.
    spawn(&mut f.sim, 1, f.citizen, 21, 21);
    let s = spawn(&mut f.sim, 1, f.scout, 22, 22);
    f.sim.units[s].orders.push_back(Order {
        flags: crate::orders::flag::ACTION,
        body: Body::Move(crate::orders::MoveOrder {
            kind: crate::orders::MoveKind::ExploreTo,
            dest: tile_pos(40, 40),
            angle: crate::movement::Angle(0),
            facing: None,
            has_waypoint: false,
            waypoint: tile_pos(40, 40),
            last: None,
            pause: 0,
            retry: 0,
            attempts: 0,
            timer: 0,
            coll: None,
            orig: None,
            group: None,
        }),
    });

    f.sim.census(1);
    let cs = &f.sim.ai[1].census;
    assert_eq!(cs.peasants, 6, "six citizens, the scout is not one");
    assert_eq!(cs.gatherers, 5);
    assert_eq!(cs.free_peasants, 1, "only the idle citizen is free");
    assert_eq!(cs.active, 7);
    // Five gatherers, all working buildings of this city, so the
    // `busy--`/`busy++` pair puts them all back on the same record.
    let rec = f.sim.ai[1].city_ai[c];
    assert_eq!(rec.free, 1);
    assert_eq!(rec.busy, 5);
    assert_eq!(rec.gatherers, 5);
    assert!(rec.peasant_dist < PEASANT_DIST_NONE, "a citizen was found");
}

/// **A gatherer counts in its building's city, not its nearest**
/// (item 752, `docs/AI.md` §73). `plan_strategy` overwrites the found
/// city with the target building's at `6babfb`, so `gatherers` and
/// `peasant_dist` follow `busy` there. run227's woodcutters `1/50` and
/// `1/61` stand nearer London and work a camp of who=1's second city:
/// the original counts them in the second, 10 and 12 against this
/// crate's 11 and 11, and the crossing rule in `find_gather_spot` read
/// the difference on East Indies 16594.
#[test]
fn a_gatherer_counts_in_its_building_s_city_not_the_nearest() {
    let mut f = fix();
    let near_b = build(&mut f.sim, 1, f.village, 20, 20);
    let near = f.sim.buildings[near_b].city.expect("a city record");
    let far_b = build(&mut f.sim, 1, f.village, 44, 20);
    let far = f.sim.buildings[far_b].city.expect("a city record");
    f.sim.plant_camp_forest(tile_pos(30, 26));
    let wood_b = build(&mut f.sim, 1, f.woodcutter, 30, 26);
    f.sim.buildings[wood_b].gather_max = Some(5);
    f.sim.buildings[wood_b].city = Some(far);
    // Nearer the first city, working the second's camp.
    let u = spawn(&mut f.sim, 1, f.citizen, 24, 22);
    f.sim.units[u].orders.push_back(gather_order(wood_b));

    f.sim.census(1);
    let (n, fa) = (f.sim.ai[1].city_ai[near], f.sim.ai[1].city_ai[far]);
    assert_eq!((n.busy, n.gatherers), (0, 0), "not the nearest city");
    assert_eq!((fa.busy, fa.gatherers), (1, 1), "the building's city");
    assert!(fa.peasant_dist < PEASANT_DIST_NONE, "its distance goes too");
    assert_eq!(n.peasant_dist, PEASANT_DIST_NONE);
}

/// **One field, three writers** (item 3332): the census rewrites
/// `LeaderData +0x8a4` from its walk, and a Farm that finishes before
/// the next census reaches the same field through `Build::activate` —
/// which is what the AI's three-quarters gather gate reads between
/// censuses. An unfinished Farm counts for neither.
#[test]
fn a_farm_finished_between_censuses_reaches_the_gather_gate() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    build(&mut f.sim, 1, f.farm, 26, 20);
    let late = f
        .sim
        .place_building(1, f.farm, tile_pos(26, 24))
        .expect("a second farm site");
    f.sim.census(1);
    let at_census = f.sim.ledgers[1].gather_slots[0];
    assert!(at_census > 0, "the finished farm");
    assert!(!f.sim.buildings[late].active, "the second is a site");

    finish(&mut f.sim, late);
    let gm = f.sim.buildings[late].gather_max.unwrap_or(0);
    assert!(gm > 0);
    assert_eq!(
        f.sim.ledgers[1].gather_slots[0],
        at_census + gm,
        "the gate's field moved with no census"
    );
    f.sim.census(1);
    assert_eq!(
        f.sim.ledgers[1].gather_slots[0],
        at_census + gm,
        "and the census agrees"
    );
}

/// `gather_slots` comes off the buildings, `filled_gather_slots` off
/// the citizens working them, and each is keyed by the *building's*
/// type: farm food, woodcutter timber, university knowledge (which only
/// a garrisoned scholar fills), mine metal, oil.
#[test]
fn the_gather_slot_books_are_kept_by_building_type() {
    let mut f = fix();
    let city_b = build(&mut f.sim, 1, f.village, 20, 20);
    let c = f.sim.buildings[city_b].city.unwrap();
    let farm_b = build(&mut f.sim, 1, f.farm, 26, 20);
    f.sim.buildings[farm_b].gather_max = Some(3);
    f.sim.plant_camp_forest(tile_pos(20, 26));
    let wood_b = build(&mut f.sim, 1, f.woodcutter, 20, 26);
    f.sim.buildings[wood_b].gather_max = Some(5);
    let uni_b = build(&mut f.sim, 1, f.university, 26, 26);
    f.sim.buildings[uni_b].gather_max = Some(4);

    for _ in 0..3 {
        let u = spawn(&mut f.sim, 1, f.citizen, 26, 21);
        f.sim.units[u].orders.push_back(gather_order(farm_b));
    }
    for _ in 0..2 {
        let u = spawn(&mut f.sim, 1, f.citizen, 21, 26);
        f.sim.units[u].orders.push_back(gather_order(wood_b));
    }
    // A scholar inside the university fills the knowledge slot.
    let sch = spawn(&mut f.sim, 1, f.scholar, 26, 26);
    f.sim.units[sch].inside = Some(uni_b);
    f.sim.units[sch].on_map = false;

    f.sim.census(1);
    let slots = f.sim.ledgers[1].gather_slots;
    assert_eq!(slots[0], 3, "the farm");
    assert_eq!(slots[1], 5, "the woodcutter");
    assert_eq!(slots[3], 4, "the university");
    assert_eq!(slots[2], 0, "wealth is never written");
    let cs = &f.sim.ai[1].census;
    assert_eq!(cs.filled_gather_slots[0], 3);
    assert_eq!(cs.filled_gather_slots[1], 2);
    assert_eq!(cs.filled_gather_slots[3], 1, "the garrisoned scholar");
    assert_eq!(cs.scholars, 1);
    // `count_gather_slots` leaves knowledge out of its total, so the
    // region sees 3 + 5 and not 3 + 5 + 4 — run8's `reg_gather_slots 8`.
    let r = f.sim.cities_state.cities[c].reg.unwrap() as usize;
    assert_eq!(cs.reg_gather_slots[r], 8);
}

/// A ship garrisoned in a dock counts in the **sea** region behind the
/// dock, not in the land region the dock stands in; a citizen in the
/// same dock counts in the land region.
#[test]
fn a_garrisoned_unit_takes_its_dock_s_sea_region() {
    let mut f = fix();
    // The dock stands on the land/sea border: cells 11 (land) and 12
    // (sea) meet at tile 48, and its 4×4 footprint reaches tile 48.
    let dock_b = force_build(&mut f.sim, 1, f.dock, 47, 20);
    let land = f.sim.world.region_of(Cell::new(11, 5)).unwrap();
    let sea = f.sim.world.region_of(Cell::new(12, 5)).unwrap();
    assert_ne!(land, sea);

    let ship = spawn(&mut f.sim, 1, f.scout, 47, 20);
    let sty = f.sim.units[ship].ty.unwrap();
    f.sim.tables.unit_types[sty].combat.domain = Domain::Sea;
    f.sim.units[ship].inside = Some(dock_b);
    f.sim.units[ship].on_map = false;

    assert_eq!(f.sim.census_unit_region(ship), Some(sea));

    let cit = spawn(&mut f.sim, 1, f.citizen, 47, 20);
    f.sim.units[cit].inside = Some(dock_b);
    f.sim.units[cit].on_map = false;
    assert_eq!(f.sim.census_unit_region(cit), Some(land));
}

/// A tower counts 1 for defence, a fort 2, and a dock — whose type has
/// a water domain — counts nothing even though it has `ATTACK`.
#[test]
fn defence_counts_towers_and_forts_and_exempts_the_dock() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    build(&mut f.sim, 1, f.tower, 28, 20);
    build(&mut f.sim, 1, f.fort, 20, 28);
    force_build(&mut f.sim, 1, f.dock, 47, 20);
    f.sim.census(1);
    assert_eq!(f.sim.ai[1].census.defense, 3);
    let r = f.sim.world.region_of(Cell::new(5, 5)).unwrap() as usize;
    assert_eq!(f.sim.ai[1].census.reg_defense[r], 3);
}

/// **`defense` between sweeps, and an unfinished site in one** (item
/// 1377): `Build::init` adds 1 for a type with arrows the moment it is
/// placed, `Build::activate` 1 more, `Build::close` takes 1; the sweep
/// recounts and counts the site, because its `is_active` test gates the
/// gather slots alone (`006bb09c` → `006bb15a`). A human's tally is
/// never moved here: this crate does not sweep a human.
#[test]
fn defence_moves_with_a_site_s_life_and_the_sweep_counts_a_site() {
    let mut f = fix();
    f.sim.census(1);
    assert_eq!(f.sim.ai[1].census.defense, 0);
    let site = f.sim.init_build(1, f.tower, tile_pos(28, 20), false);
    assert_eq!(f.sim.ai[1].census.defense, 1, "`Build::init`'s +1");
    f.sim.census(1);
    assert_eq!(
        f.sim.ai[1].census.defense, 1,
        "the sweep counts the unfinished site"
    );
    let r = f.sim.world.region_of(Cell::new(7, 5)).unwrap() as usize;
    assert_eq!(f.sim.ai[1].census.reg_defense[r], 1);
    f.sim.activate(site, false, true);
    assert_eq!(f.sim.ai[1].census.defense, 2, "`Build::activate`'s +1");
    f.sim.close_building(site, true);
    assert_eq!(f.sim.ai[1].census.defense, 1, "`Build::close`'s -1");
    f.sim.init_build(0, f.tower, tile_pos(28, 28), false);
    assert_eq!(f.sim.ai[0].census.defense, 0, "a human's is not moved");
}

/// The maxima only ever climb, and the escrow rate switches on at the
/// *third* city and stays on.
#[test]
fn the_maxima_climb_and_the_escrow_rate_latches() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    for _ in 0..4 {
        spawn(&mut f.sim, 1, f.citizen, 21, 21);
    }
    f.sim.census(1);
    assert_eq!(f.sim.ai[1].census.peasants, 4);
    assert_eq!(f.sim.ai[1].census.peasant_high, 4);
    assert_eq!(f.sim.ai[1].census.escrow_rate, [0; RESOURCES], "one city");

    // Kill three citizens: the count falls, the high-water mark does not.
    for u in 0..f.sim.units.len() {
        if f.sim.units[u].owner == 1 && f.sim.units[u].health > 0 && u < 3 {
            f.sim.units[u].health = 0;
        }
    }
    build(&mut f.sim, 1, f.village, 40, 20);
    build(&mut f.sim, 1, f.village, 20, 40);
    f.sim.census(1);
    assert_eq!(f.sim.ai[1].census.peasants, 1);
    assert_eq!(f.sim.ai[1].census.peasant_high, 4);
    assert_eq!(f.sim.ai[1].census.city_high, 3);
    assert_eq!(
        f.sim.ai[1].census.escrow_rate, [ESCROW_RATE; RESOURCES],
        "three cities is more than two"
    );
}

/// A region nobody has filled is *thin* — bit 1. With a city in a
/// region that is not thin the word starts at 8 instead, which is the
/// original's `else if` arm and not in `docs/AI.md` §2.3.
#[test]
fn the_thin_bit_and_the_eight_that_replaces_it() {
    let mut f = fix();
    // The eight survives step 15's tail only on a map whose sea class
    // is 1 (Great Lakes' — `world+0x34`, `World::sea_map`): class 0,
    // the land maps, clears it; 3 and up set it again for a region
    // with neither `2` nor `4`. Run9's lobby is class 1.
    f.sim.world.set_sea_map(1);
    build(&mut f.sim, 1, f.village, 20, 20);
    f.sim.census(1);
    let r = f.sim.world.region_of(Cell::new(5, 5)).unwrap() as usize;
    // 12 × 16 = 192 cells, one city: 50 < 192, so thin.
    assert_eq!(f.sim.ai[1].census.strategy[r], 1);
    assert_eq!(f.sim.ai[1].census.reg_cities[r], 1);

    // Four cities fill it: 4 × 50 = 200 ≥ 192.
    build(&mut f.sim, 1, f.village, 40, 20);
    build(&mut f.sim, 1, f.village, 20, 40);
    build(&mut f.sim, 1, f.village, 40, 40);
    f.sim.census(1);
    assert_eq!(f.sim.ai[1].census.strategy[r], 8);
}

/// **A human leader takes the sweep, but for its army** (item 1605,
/// `docs/AI.md` §169). `Leaders::strategy_all@006ed430` gates on
/// `leader_flags & 3 == 3` alone, so the human's city carries the
/// circle walk's picture, and `plan_strategy@006b9620:1642` seeds an
/// army only for `(leader_flags & 0xc) != 4` — the same city of a
/// computer leader takes one.
#[test]
fn a_human_s_sweep_fills_its_city_and_seeds_no_army() {
    let mut f = fix();
    let b = build(&mut f.sim, 0, f.village, 20, 20);
    let c = f.sim.buildings[b].city.unwrap();
    f.sim.defeated[1] = true;
    f.sim.frame = 0;
    f.sim.strategy_all();
    assert!(
        f.sim.ai[0].city_ai.get(c).is_some_and(|r| r.land > 0),
        "the human's city is swept"
    );
    assert_eq!(f.sim.armies[0].valid().count(), 0, "and no army is seeded");
    assert_eq!(
        f.sim.ai[0].step,
        crate::ai::Step::Script,
        "the sweep arms the machine"
    );
    f.sim.frame = 1;
    f.sim.strategy_all();
    assert_eq!(
        f.sim.ai[0].step,
        crate::ai::Step::Idle,
        "and `production_ai`'s default disarms it"
    );
    f.sim.frame = 0;
    f.sim.cities_state.nation[0].human = false;
    f.sim.strategy_all();
    assert_eq!(
        f.sim.armies[0].valid().count(),
        1,
        "a computer leader's same city seeds one"
    );
}

/// A city's circle walk fills `land`, `filled` and `space`, and
/// `reg_land` takes the open remainder; `full_cities` counts a city
/// with fewer than two open cells.
#[test]
fn the_site_picture_counts_the_open_cells_of_the_circle() {
    let mut f = fix();
    let b = build(&mut f.sim, 1, f.village, 20, 20);
    let c = f.sim.buildings[b].city.unwrap();
    f.sim.census(1);
    let rec = f.sim.ai[1].city_ai[c];
    let r = f.sim.cities_state.cities[c].reg.unwrap() as usize;
    // The radius is the city's own `get_radius` — 20 tiles for a level
    // one city — so `(20 + 2) / 4 = 5` picks `circle_radius[5]`, and
    // the inner cutoff is `circle_radius[5]` too... one ring in.
    let circle = crate::ai_place::circle();
    assert!(rec.land > 0, "an empty map is all open");
    assert!(
        (rec.land as usize) < circle.radius[5],
        "the walk stops at the inner ring"
    );
    assert_eq!(rec.ocean, 0, "no water in reach");
    assert_eq!(
        f.sim.ai[1].census.reg_land[r],
        rec.land - rec.filled,
        "the region takes the open remainder"
    );
    assert_eq!(f.sim.ai[1].census.full_cities, 0);
}

/// **The circle walk starts at entry 1** (`docs/AI.md` §122, item 1477).
/// `plan_strategy@006b9620`'s loop reads `circle_x + 1` and `circle_y + 1`
/// from `i = 0` and stops on `i + 1 < circle_radius[k + 1]`: the centre's
/// own `(0, 0)` is never visited, and the last entry inside the inner
/// ring — `circle_radius[5] − 1` — is. Run640's packet at frame 12575
/// holds the original's answer for Great Sahara's `1/2031`
/// (`filled` 38, `space` 47/47/34; entry 0 gave 39/46/46/33).
///
/// Here a neighbour owns, in turn, the centre cell and the last inner
/// entry's: the first changes nothing (the centre is not walked), the
/// second takes one cell off `land`.
#[test]
fn the_circle_walk_starts_at_entry_one() {
    let circle = crate::ai_place::circle();
    let inner = circle.radius[5];
    let land_with = |owned: Option<(i32, i32)>| {
        let mut f = fix();
        let b = build(&mut f.sim, 1, f.village, 20, 20);
        let c = f.sim.buildings[b].city.unwrap();
        let centre = f.sim.cities_state.cities[c].pos.cell();
        if let Some((dx, dy)) = owned {
            let cell = Cell::new(centre.x + dx, centre.y + dy);
            f.sim.world.set_owner(cell, Owner::Player(0), Owner::None);
        }
        f.sim.census(1);
        f.sim.ai[1].city_ai[c].land
    };
    let base = land_with(None);
    assert_eq!(
        base as usize,
        inner - 1,
        "entries 1..circle_radius[5], all of them land in the city's own region"
    );
    assert_eq!(
        land_with(Some((0, 0))),
        base,
        "the centre's own cell is never walked"
    );
    let last = (circle.x[inner - 1], circle.y[inner - 1]);
    assert_eq!(
        land_with(Some(last)),
        base - 1,
        "the last entry before the inner ring is walked"
    );
}

/// `check_explore` answers the whole **region** grid — two world cells
/// across each way — because there is no fog here.
#[test]
fn check_explore_counts_the_whole_region_grid() {
    let mut f = fix();
    f.sim.check_explore(1);
    assert_eq!(
        f.sim.ai[1].census.explored,
        8 * 8,
        "a 16x16 cell world is 8x8 region cells"
    );
    assert_eq!(TILES_PER_CELL, 4);
}

/// **The sweep reads `UnitTypeData::role`, over captains** — item
/// 295, and both halves failed on purpose before they landed.
///
/// The role word the producers read (`ai_load::role`) and this crate's
/// own profile bitfield (`combat::role`, `docs/DECISIONS.md` entry 18)
/// collide twice: `1 << 16` is `MILITARY` in one and `CARAVAN` in the
/// other, `1 << 4` is the scout bit and `V2ROCKET`. The sweep read the
/// wrong one, so on run91's Great Lakes the leader's caravan was its
/// only soldier and nine Longbowmen and three Hoplites were civilians.
/// And it walked every live object where the original walks captains
/// (`docs/AI.md` §2.3 step 10), so the twelve of those that stand in
/// four squads counted twelve rather than four.
///
/// Here: a three-figure squad of a military type and a lone caravan.
/// `active` is 2 and not 4, `combat` is 1 and not 3, and the caravan
/// is on neither count. **`attack` is the half that separates the two
/// faults**: under the profile word the caravan took the military
/// branch and `combat` read 1 for the wrong unit, with a zero attack
/// where the soldier's is 15.
#[test]
fn the_census_counts_captains_under_the_original_s_role_word() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    f.sim.ai[1].census.resize(f.sim.world.region_count(), 2);

    // A military type by the original's word, and a caravan by this
    // crate's — whose `1 << 16` the sweep used to read as military.
    let soldier = f.sim.tables.unit_types[f.scout].clone();
    let soldier = f.sim.add_unit_type(soldier);
    f.sim.tables.unit_types[soldier].cols.role = ROLE_MILITARY;
    f.sim.tables.unit_types[soldier].combat.attack = 150;
    let trader = f.sim.tables.unit_types[f.scout].clone();
    let trader = f.sim.add_unit_type(trader);
    f.sim.tables.unit_types[trader].combat.roles = crate::combat::role::IS_CARAVAN;

    let squad: Vec<usize> = (0..3)
        .map(|i| spawn(&mut f.sim, 1, soldier, 20 + i, 22))
        .collect();
    for w in squad.windows(2) {
        f.sim.units[w[1]].captain = false;
        f.sim.units[w[1]].o_up = Some(w[0]);
        f.sim.units[w[0]].o_down = Some(w[1]);
    }
    spawn(&mut f.sim, 1, trader, 24, 22);

    f.sim.census(1);
    let c = &f.sim.ai[1].census;
    assert_eq!(c.active, 2, "the squad's followers are counted as units");
    assert_eq!(c.combat, 1, "one squad is one soldier");
    assert_eq!(c.non_siege, 1);
    assert_eq!(c.attack, 15, "`attack() / 10`, the captain's alone");
}

/// **A decoy is no unit of the census** (item 1302, `docs/AI.md`
/// §99.14; `plan_strategy`'s `testb $0x1, 0x68(%esi)` at `6b9f57`):
/// run346's six decoy squads of 11637 stay off `non_siege`, and
/// `create_units`' military gate on 11780 reads the count without them.
///
/// Made to fail with the decoy test dropped.
#[test]
fn a_decoy_is_no_unit_of_the_census() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    f.sim.ai[1].census.resize(f.sim.world.region_count(), 2);
    let soldier = f.sim.tables.unit_types[f.scout].clone();
    let soldier = f.sim.add_unit_type(soldier);
    f.sim.tables.unit_types[soldier].cols.role = ROLE_MILITARY;
    f.sim.tables.unit_types[soldier].combat.attack = 150;
    spawn(&mut f.sim, 1, soldier, 20, 22);
    let copy = spawn(&mut f.sim, 1, soldier, 22, 22);
    f.sim.units[copy].decoy = true;
    f.sim.census(1);
    let c = &f.sim.ai[1].census;
    assert_eq!((c.active, c.combat, c.non_siege), (1, 1, 1));
}

/// A military unit of mine standing on another leader's land shows up
/// in *their* `invaders[me]`, and the count is cleared each sweep.
#[test]
fn invaders_land_on_the_owner_s_record() {
    let mut f = fix();
    build(&mut f.sim, 1, f.village, 20, 20);
    // Player 0 owns a patch of cells.
    for x in 8..10 {
        for y in 8..10 {
            f.sim
                .world
                .set_owner(Cell::new(x, y), Owner::Player(0), Owner::None);
        }
    }
    let u = spawn(&mut f.sim, 1, f.scout, 34, 34);
    let ty = f.sim.units[u].ty.unwrap();
    f.sim.tables.unit_types[ty].cols.role = ROLE_MILITARY;
    f.sim.ai[0].census.resize(f.sim.world.region_count(), 2);

    f.sim.census(1);
    assert_eq!(f.sim.ai[0].census.invaders[1], 1);
    assert_eq!(f.sim.ai[1].census.combat, 1);

    // Move it home: the next sweep clears the old count.
    f.sim.units[u].pos = tile_pos(20, 22);
    f.sim.census(1);
    assert_eq!(f.sim.ai[0].census.invaders[1], 0);
}
