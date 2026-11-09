use super::*;
use crate::world::World;

/// **A swarmed builder is marked a builder** (item 3140,
/// `Group::action_swarm_around@0070fbe0`'s one arm queues through
/// `Unit::add_build_order@005e5210`, which sets `unit_masks |= 0x400`):
/// `think_peasant`'s build arm reads the bit, and an AI citizen sent
/// by `produce_building`'s `QUEUE_NEW` swarm that comes back idle asks
/// for a site before a gather. Made to fail by taking the mark out of
/// [`Sim::swarm_around`]: both asserts fall.
#[test]
fn a_swarmed_builder_is_marked_a_builder() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let site = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    let mut u = crate::Unit::new(1, 0, Pos::new(0x1400, 0x1000), 10);
    u.ty = Some(citizen);
    u.on_map = true;
    let u = sim.add_unit(u);
    sim.swarm_around(u, site, Body::Build(site), true);
    assert!(sim.units[u].was_builder, "a build swarm marks it");
    sim.units[u].was_builder = false;
    sim.swarm_around(u, site, Body::Repair(site), true);
    assert!(sim.units[u].was_builder, "a repair swarm marks it");
}

/// **A gather order takes the mark off** (item 3710,
/// `Unit::add_gather_order@0061a5c0`'s `61a5f7`, every queue position):
/// the sweep's view clears `+0x68 & 0x400` in `take_order`, and this
/// crate's left `was_builder` standing.
#[test]
fn a_gather_order_takes_the_builder_s_mark_off() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let site = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    let mut u = crate::Unit::new(1, 0, Pos::new(0x1400, 0x1000), 10);
    u.ty = Some(citizen);
    u.on_map = true;
    let u = sim.add_unit(u);
    for pos in [QueuePos::Last, QueuePos::New] {
        sim.units[u].was_builder = true;
        sim.add_gather_order(u, site, pos, true);
        assert!(!sim.units[u].was_builder, "{pos:?}");
    }
}

/// **`find_repair_spot` is a tier arm** (item 3187,
/// `Unit::find_repair_spot@00604320`): a computer below difficulty 2
/// answers 0, and from 2 up it swarms the nearest damaged friendly
/// building not under attack, a `MOVE_TO` approach and a repair order
/// behind it. Made to fail by dropping the difficulty gate (the first
/// assert falls) or the under-attack filter (the last).
#[test]
fn find_repair_spot_is_a_tier_arm_and_takes_the_nearest_damaged_building() {
    let mut world = World::new(60, 60);
    world.fill_region(
        crate::world::Terrain::Land,
        crate::world::Cell::new(0, 0),
        crate::world::Cell::new(59, 59),
    );
    let mut sim = Sim::new(crate::tuning::Tuning::RON, world, 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let near = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    let far = sim.add_building(1, Pos::new(0x1600, 0x1000), 1);
    for b in [near, far] {
        sim.buildings[b].active = true;
        sim.buildings[b].damage = 5;
    }
    let mut u = crate::Unit::new(1, 0, Pos::new(0x1400, 0x1000), 10);
    u.ty = Some(citizen);
    u.on_map = true;
    let u = sim.add_unit(u);
    sim.cities_state.nation[1].human = false;
    sim.lobby.difficulty = 1;
    assert!(!sim.find_repair_spot(u), "tier 1 never repairs");
    sim.lobby.difficulty = 2;
    sim.buildings[far].under_attack = 2;
    sim.buildings[near].under_attack = 2;
    assert!(
        !sim.find_repair_spot(u),
        "a building under attack is skipped"
    );
    sim.buildings[near].under_attack = 0;
    assert!(sim.find_repair_spot(u), "tier 2 repairs");
    assert!(
        sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Repair(b) if b == near))
    );
}

/// **`Build::all_gathering` prunes the chain before it walks it**
/// (`62f573`: `check_gatherers` first; item 1248, `docs/GOLDEN.md`
/// §52). A dead member still on the chain, its walk still at the front
/// of its orders, would answer "not out at its tile" and send the
/// chopping woodcutter's reroll to `% 100 + 300` where the original
/// sets −1 and walks home. Made to fail by taking the prune out: the
/// dead member keeps the answer at false.
#[test]
fn all_gathering_drops_a_dead_member_before_it_reads_the_chain() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let camp = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    sim.buildings[camp].ty = Some(sim.add_build_type(crate::build::BuildType {
        flags: bflags::GATHERER,
        ..crate::build::BuildType::default()
    }));
    let put = |sim: &mut Sim, x: i32| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(1, index, Pos::new(x, 0x1000), 10);
        u.ty = Some(citizen);
        u.on_map = true;
        sim.add_unit(u)
    };
    let chopping = [put(&mut sim, 0x1100), put(&mut sim, 0x1200)];
    for u in chopping {
        sim.add_gather_order(u, camp, QueuePos::New, false);
        if let Some(Body::Gather(g)) = sim.units[u].orders.front_mut().map(|o| &mut o.body) {
            g.goto_build = false;
            g.wait = 100;
        }
    }
    assert!(sim.all_gathering(camp), "both out at their tiles");
    let dead = put(&mut sim, 0x1300);
    sim.add_gather_order(dead, camp, QueuePos::New, false);
    sim.add_move_order(
        dead,
        Pos::new(0x1800, 0x1000),
        MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    sim.units[dead].health = 0;
    assert_eq!(sim.buildings[camp].gatherers[0], dead, "at the head");
    assert!(sim.all_gathering(camp), "the dead member is not read");
    assert!(!sim.is_gathered_by(camp, dead), "it is pruned");
}

/// **And a live member whose action is not the gather** (item 1278,
/// `docs/GOLDEN.md` §55, run496's 1049): `add_gather_order` joins the
/// chain on the call, so a Citizen out of a City under a two-point
/// gather list — `[MOVE_TO waypoint (action), GATHER (action)]` — is
/// at the chain's head while it walks the first leg. `is_gathering_at`
/// reads its action, the move, and `check_gatherers` unlinks it; the
/// choppers are all out, and the answer is yes. Made to fail by taking
/// the prune out: the walker's head is a move, and the answer no.
#[test]
fn all_gathering_drops_a_member_walking_a_waypoint_first() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let camp = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    sim.buildings[camp].ty = Some(sim.add_build_type(crate::build::BuildType {
        flags: bflags::GATHERER,
        ..crate::build::BuildType::default()
    }));
    let put = |sim: &mut Sim, x: i32| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(1, index, Pos::new(x, 0x1000), 10);
        u.ty = Some(citizen);
        u.on_map = true;
        sim.add_unit(u)
    };
    let chopping = put(&mut sim, 0x1100);
    sim.add_gather_order(chopping, camp, QueuePos::New, false);
    if let Some(Body::Gather(g)) = sim.units[chopping].orders.front_mut().map(|o| &mut o.body) {
        g.goto_build = false;
        g.wait = 100;
    }
    let walker = put(&mut sim, 0x1300);
    sim.add_move_order(
        walker,
        Pos::new(0x1800, 0x1000),
        MoveKind::MoveTo,
        QueuePos::New,
        true,
    );
    sim.add_gather_order(walker, camp, QueuePos::Last, true);
    assert_eq!(sim.buildings[camp].gatherers[0], walker, "at the head");
    assert!(
        !sim.is_gathering_at(walker, camp, false),
        "its action is the move"
    );
    assert!(sim.all_gathering(camp), "the walker is not read");
    assert!(!sim.is_gathered_by(camp, walker), "it is pruned");
    assert!(
        sim.units[walker].alive(),
        "alive, and still holding its gather"
    );
}
