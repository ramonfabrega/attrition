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
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
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
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
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
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, world, 2);
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
    sim.cities.nation[1].human = false;
    sim.ai.lobby.difficulty = 1;
    assert!(!sim.find_repair_spot(u), "tier 1 never repairs");
    sim.ai.lobby.difficulty = 2;
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
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let camp = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    sim.buildings[camp].ty = Some(sim.add_build_type(crate::cities::build::BuildType {
        flags: bflags::GATHERER,
        ..crate::cities::build::BuildType::default()
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
    assert_eq!(sim.gather_chain(camp)[0], dead, "at the head");
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
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let camp = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    sim.buildings[camp].ty = Some(sim.add_build_type(crate::cities::build::BuildType {
        flags: bflags::GATHERER,
        ..crate::cities::build::BuildType::default()
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
    assert_eq!(sim.gather_chain(camp)[0], walker, "at the head");
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

/// **The farmer's turn east is `Unit::set_angle` whole** (item 4477,
/// `docs/CHECKSUM.md` §23.14): `Unit::do_gather@005ef2a0:382` turns the
/// farmer with `set_angle(this, 0x40000000, ., 0)`, whose reversing arm
/// flips `unit_masks & 2` and toggles `GroupData::facing` when
/// `find_leader` names the unit. run854's Citizen `1/47`, sole member of
/// player 1's record 75, turned east at its farm on 3065 and theirs'
/// record read `facing` 1 on 3301. Made to fail by putting
/// `Movement::set_heading` back in `face_east`: the mirror flips and the
/// group's flag stays clear.
#[test]
fn the_farmer_s_turn_east_toggles_its_group_s_facing() {
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let farm = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    let mut u = crate::Unit::new(1, 0, Pos::new(0x1400, 0x1000), 10);
    u.ty = Some(citizen);
    u.on_map = true;
    let u = sim.add_unit(u);
    let slot = sim.init_army(1, None);
    sim.army_add_unit(1, slot, u);
    sim.units[u]
        .movement
        .set_heading(crate::movement::Angle::WEST);
    let mirror = sim.units[u].movement.mirror;
    assert!(!sim.ai.armies[1].list[slot].group.facing);
    let g = GatherOrder {
        building: farm,
        tile: None,
        wait: 0,
        goto_build: false,
        dist_mod: 0,
        been_there: true,
    };
    SimDoGather {
        s: &mut sim,
        u,
        b: farm,
        g,
        frame: 0,
        drew: false,
    }
    .face_east();
    assert_eq!(
        sim.units[u].movement.heading,
        crate::movement::Angle(0x4000_0000)
    );
    assert_ne!(
        sim.units[u].movement.mirror, mirror,
        "the unit's mirror flips"
    );
    assert!(
        sim.ai.armies[1].list[slot].group.facing,
        "the leader's reversal toggles its group's facing"
    );
}

/// **A gatherer pushed onto a second chain carries the first's walk with
/// it** (item 4505; `Build::add_gatherer@0062f640`'s `0062f6d6`–`0062f6e9`
/// writes the unit's own `UnitData::gather_down` and nothing unlinks it
/// from a chain that still links to it). The 2017 sample's computer 1
/// Citizen `1/11`, behind `1/13` in `#2001`'s chain, was pushed onto
/// `#2009`'s on 4247: its link became `1/8`, `#2009`'s next, and
/// `#2001`'s walk ran on into `#2009`'s chain. Then `#2001`'s prune drops
/// every member not gathering there, the moved unit's link left as it
/// is. Made to fail by keeping a chain per building (the walk of the
/// first chain stays `[a3, a2, a1]`).
#[test]
fn a_gatherer_pushed_onto_a_second_chain_carries_the_first_s_walk() {
    let mut sim = Sim::new(crate::core::tuning::Tuning::RON, World::new(60, 60), 2);
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 10,
        worker: Worker::Citizen,
        ..crate::UnitType::default()
    });
    let gatherer = sim.add_build_type(crate::cities::build::BuildType {
        flags: bflags::GATHERER,
        ..crate::cities::build::BuildType::default()
    });
    let first = sim.add_building(1, Pos::new(0x1000, 0x1000), 1);
    let second = sim.add_building(1, Pos::new(0x2000, 0x1000), 1);
    sim.buildings[first].ty = Some(gatherer);
    sim.buildings[second].ty = Some(gatherer);
    let put = |sim: &mut Sim, x: i32, camp: usize| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(1, index, Pos::new(x, 0x1000), 10);
        u.ty = Some(citizen);
        u.on_map = true;
        let u = sim.add_unit(u);
        sim.add_gather_order(u, camp, QueuePos::New, false);
        u
    };
    let a1 = put(&mut sim, 0x1100, first);
    let a2 = put(&mut sim, 0x1200, first);
    let a3 = put(&mut sim, 0x1300, first);
    let b1 = put(&mut sim, 0x2100, second);
    assert_eq!(sim.gather_chain(first), vec![a3, a2, a1], "newest first");
    // The job changes under it with no kill: the order now names the
    // second camp, and the second camp takes it.
    if let Some(Body::Gather(g)) = sim.units[a2].orders.front_mut().map(|o| &mut o.body) {
        g.building = second;
    }
    assert!(sim.add_gatherer(second, a2), "the second camp takes it");
    assert_eq!(sim.gather_chain(second), vec![a2, b1]);
    assert_eq!(
        sim.units[a2].gather_down,
        Some(b1),
        "its own link is the new chain's"
    );
    assert_eq!(
        sim.gather_chain(first),
        vec![a3, a2, b1],
        "the first chain walks on into the second's"
    );
    sim.check_gatherers(first);
    assert_eq!(sim.gather_chain(first), vec![a3], "the prune drops both");
    assert_eq!(
        sim.gather_chain(second),
        vec![a2, b1],
        "and leaves it whole"
    );
    assert!(!sim.is_gathered_by(first, a1), "a1 is off every chain");
}
