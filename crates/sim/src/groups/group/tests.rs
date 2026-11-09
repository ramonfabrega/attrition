use super::*;
use crate::ai_load::role;
use crate::combat;
use crate::{Unit, UnitType};

/// A two-player world with an AI leader 1 at war with the human 0.
fn sim() -> Sim {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    s.nation[0].human = true;
    s.nation[1].human = false;
    s.at_war[0][1] = true;
    s.at_war[1][0] = true;
    s
}

/// Every hand-built combatant carries `role::ROLE_MILITARY`, which is what
/// `UnitTypeData::get_stance_type` keys on — for a type loaded from the
/// install `ai_load` derives it from the same facts the original does.
fn fighter(sim: &mut Sim) -> usize {
    let mut t = UnitType {
        hits: 100,
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    t.cols.role |= role::ROLE_MILITARY;
    sim.add_unit_type(t)
}

/// An air-domain type. Without `unit_flags` bit `f` it is a plane
/// (`UnitData::is_plane`); with it, a helicopter — which is not.
fn flier(sim: &mut Sim, helicopter: bool) -> usize {
    let mut t = UnitType {
        hits: 100,
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            domain: Domain::Air,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    if helicopter {
        t.cols.unit_flags |= uflags::FLAG_HELICOPTER;
    }
    t.cols.role |= role::ROLE_MILITARY;
    sim.add_unit_type(t)
}

/// **A helicopter measures edge to edge; a plane, centre to centre**
/// (`ObjectData::attack_dist@006488f0`'s first arm is the attacker's
/// `is_plane`, vslot `+0xc0`; `docs/AI.md` §227). run866's Attack
/// Helicopter `1/145` takes the human's `0/2000` on 5135 only because
/// the footprint comes off its distance.
#[test]
fn a_helicopter_measures_attack_distance_edge_to_edge() {
    let mut s = sim();
    let heli = flier(&mut s, true);
    let plane = flier(&mut s, false);
    let foot = fighter(&mut s);
    let h = spawn(&mut s, 0, heli, Pos::new(1920, 1920));
    let p = spawn(&mut s, 0, plane, Pos::new(1920, 1920));
    let t = spawn(&mut s, 1, foot, Pos::new(2880, 1920));
    assert_eq!(
        s.attack_dist(Obj::Unit(p), Obj::Unit(t)),
        960,
        "a plane: centre to centre"
    );
    assert_eq!(
        s.attack_dist(Obj::Unit(h), Obj::Unit(t)),
        960 - 2 * 0x18,
        "a helicopter: each extent off"
    );
}

/// who=0's Airbase and who=1's other building, as chapter seventeen
/// stages them (`docs/GOLDEN.md` §25).
fn airbase_and_target(sim: &mut Sim) -> (usize, usize) {
    let airbase = sim.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Airbase,
        ..crate::build::BuildType::default()
    });
    let base = sim.add_building(0, Pos::new(11616, 13920), 0);
    sim.buildings[base].ty = Some(airbase);
    let target = sim.add_building(1, Pos::new(21120, 16512), 0);
    (base, target)
}

/// **A flight home is one `StrafeOrder` with no target, going home**
/// (`Group::action_flight@006fb260`'s `MOVE_TO` arm →
/// `Unit::add_strafe_order(−1, −1, base, who, 1, QUEUE_NEW, 1)`,
/// `docs/ORDERS.md` §32; run223's blocks 642 and 662).
#[test]
fn a_flight_home_is_one_strafe_with_no_target_going_home() {
    let mut s = sim();
    let t = flier(&mut s, false);
    let u = spawn(&mut s, 0, t, Pos::new(11616, 16224));
    let (base, _) = airbase_and_target(&mut s);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
    let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
    assert_eq!(o.len(), 1, "one order");
    assert_eq!(o[0].index(), index::STRAFE);
    assert!(o[0].has(flag::ACTION), "the action bit");
    let Body::Strafe(sf) = o[0].body else {
        panic!("a strafe")
    };
    assert_eq!(sf.target, None);
    assert_eq!(sf.at, None);
    assert!(sf.returning && sf.mandatory);
    assert_eq!(sf.home, Some(base));
    assert_eq!(sf.cruising_alt, 0x640);
}

/// **An aircraft on the ground outside a base takes no strike**: an
/// `ATTACK` flight needs the member "inside" — its `inside_up`, or an
/// air order's home — and `add` placed it outside (run223's block
/// 622, where the original gave the pair nothing).
#[test]
fn an_unbased_aircraft_on_the_ground_takes_no_strike() {
    let mut s = sim();
    let t = flier(&mut s, false);
    let u = spawn(&mut s, 0, t, Pos::new(10104, 16248));
    let (_, target) = airbase_and_target(&mut s);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
    assert!(
        s.units[u].orders.is_empty(),
        "no order: the member is skipped"
    );
}

/// **A strike re-points a flying strafe and adds nothing**: the STRAFE
/// arm writes the target and its point, `returning 0`, and keeps the
/// home (run223's block 665, the order the original then turns into
/// an air patrol over the unseen target's point).
#[test]
fn a_strike_re_points_a_flying_strafe() {
    let mut s = sim();
    let t = flier(&mut s, false);
    let u = spawn(&mut s, 0, t, Pos::new(10104, 16248));
    let (base, target) = airbase_and_target(&mut s);
    let g = group_of(0, &[u]);
    s.group_action_flight(&g, Obj::Building(base), Flight::Home);
    s.group_action_flight(&g, Obj::Building(target), Flight::Strike);
    let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
    assert_eq!(o.len(), 1, "still one order");
    let Body::Strafe(sf) = o[0].body else {
        panic!("a strafe")
    };
    assert_eq!(sf.target, Some(Obj::Building(target)));
    assert_eq!(sf.at, Some(Pos::new(21120, 16512)));
    assert!(!sf.returning && sf.mandatory);
    assert_eq!(sf.home, Some(base));
}

/// A plane on run223's numbers: `MOVES 75`, air domain, its figure
/// seated, pointed where the test wants it. `bomber` puts its type on
/// the `0x130` line.
fn plane(s: &mut Sim, at: Pos, heading: Angle, bomber: bool) -> usize {
    let t = UnitType {
        hits: 100,
        moves: 75,
        turn_speed: crate::movement::degrees_to_angle(10).0,
        kind: crate::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::attrition::UnitKind::default()
        },
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            domain: Domain::Air,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    let ty = s.add_unit_type(t);
    if bomber {
        // After the add, which indexes the leaders' tech bits by it: a
        // hand-built tree holds no `0x130`, and `is` answers the
        // equality alone.
        s.tables.unit_types[ty].tree = Some(0x130);
    }
    let u = spawn(s, 0, ty, at);
    s.units[u].kind = s.tables.unit_types[ty].kind;
    s.units[u].movement.speed = 75;
    s.units[u].movement.turning = crate::turning_of(&s.tables.unit_types[ty]);
    s.units[u].movement.heading = heading;
    s.units[u].movement.facing = heading;
    s.init_guys(u, Some(ty));
    u
}

/// **The Fighter's first frame home is run223's block 642, field for
/// field** (`Unit::do_air_physics@005e86d0`, `docs/ORDERS.md` §33).
/// `check_fuel` aims at the landing point pushed half the distance
/// back along the approach, (11424, 15089); the returning bank wants
/// its double, clamps at 55 and steps 10, and turns twice by the
/// half-degree floor; banked, the plane flies at three quarters, and
/// within `0x600` of its point at half that, 28; the pitch climbs 2 and
/// lifts the figure 2. Made to fail with each arm changed: without the
/// second turn the heading is one floor short, without the halving the
/// step is (48, 27).
#[test]
fn a_fighter_s_first_frame_home_is_run223_s_642() {
    let mut s = sim();
    let (base, _) = airbase_and_target(&mut s);
    let u = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
    let seed = s.rng.seed;
    // Frame 641: `(0 + 641) & 7 != 0`, no redraw.
    assert_eq!(s.plane_air_physics(u, None, 641), crate::air::Flew::On);
    assert_eq!(s.rng.seed, seed, "no draw off the eighth frame");
    assert_eq!(s.units[u].pos, Pos::new(11664, 16262), "run223's 642");
    assert_eq!(s.units[u].movement.heading, Angle(1_419_725_301));
    assert_eq!(s.units[u].path[0].to, Pos::new(11424, 15089));
    let af = s.units[u].airframe;
    assert_eq!(af.bank.bits(), 0x4120_0000, "bank 10, the guy's sign");
    assert_eq!(af.pitch.bits(), 0x4000_0000, "pitch 2");
    assert_eq!((af.z, af.last_z), (2, 0), "the climb's first step");
}

/// **The redraw is a non-bomber's, every eighth frame phased by `o`**
/// (`Unit::do_air_physics+0xba`): `(r % 7 + 13) · 100`, and a Bomber
/// holds `0x640` and throws nothing. Made to fail by dropping the
/// bomber test (a draw appears) and by un-phasing the cadence (the
/// Fighter draws on 648 and not on 647).
#[test]
fn only_a_non_bomber_redraws_its_altitude_and_only_on_its_eighth_frame() {
    let mut s = sim();
    let (base, _) = airbase_and_target(&mut s);
    let b = plane(&mut s, Pos::new(13176, 16248), Angle(0x5555_5555), true);
    let f = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
    assert_eq!((s.units[b].index, s.units[f].index), (0, 1));
    s.group_action_flight(&group_of(0, &[f, b]), Obj::Building(base), Flight::Home);
    let alt = |s: &Sim, u: usize| match s.units[u].orders.front().map(|o| o.body) {
        Some(Body::Strafe(sf)) => sf.cruising_alt,
        _ => panic!("a strafe"),
    };
    // 648: `(0 + 648) & 7 == 0` is the Bomber's, and it draws nothing.
    let seed = s.rng.seed;
    s.plane_air_physics(b, None, 648);
    assert_eq!(s.rng.seed, seed, "a Bomber throws no redraw");
    assert_eq!(alt(&s, b), 0x640);
    // 648 is not the Fighter's (`o` 1): nothing.
    s.plane_air_physics(f, None, 648);
    assert_eq!(s.rng.seed, seed, "off its eighth frame");
    // 647 is: `(1 + 647) & 7 == 0`.
    let mut probe = s.rng;
    let want = (probe.roll() % 7 + 13) * 100;
    s.plane_air_physics(f, None, 647);
    assert_eq!(s.rng, probe, "one draw");
    assert_eq!(alt(&s, f), want);
}

/// **The landing** (`Unit::land_plane@005e9950`): inside a step and a
/// half of the landing point the plane lands — the strafe home
/// cleared, the plane inside its base, off the map, on the point it
/// reached, its attitude kept. Made to fail by testing the landing
/// against a single step (it flies on).
#[test]
fn a_plane_inside_a_step_and_a_half_of_its_point_lands_in_its_base() {
    let mut s = sim();
    let (base, _) = airbase_and_target(&mut s);
    let at = Pos::new(11424, 13920 + 100);
    let u = plane(&mut s, at, Angle(0), false);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
    assert_eq!(s.plane_air_physics(u, None, 721), crate::air::Flew::Done);
    assert!(s.units[u].orders.is_empty(), "the strafe home is gone");
    assert!(s.units[u].path.is_empty());
    assert_eq!(s.units[u].inside, Some(base));
    assert!(!s.units[u].on_map);
    assert_eq!(s.units[u].pos, at, "it keeps its point");
}

/// **End to end: a flight home is flown and lands** — the flight
/// command, then whole frames of `Sim::tick`, through `do_strafe`,
/// the unit loop and the figure. The Fighter comes round from 120°
/// onto the north-bound approach, lands inside its base, spends one
/// redraw on each eighth frame it flew and nothing else, and its
/// figure takes the standing arm on the landing frame alone. Made to
/// fail with the landed plane's figure frame removed from
/// `process_unit` (`stopped` stays false).
#[test]
fn a_flight_home_is_flown_to_its_base_and_lands() {
    let mut s = sim();
    s.trace_phases = true;
    let (base, _) = airbase_and_target(&mut s);
    let u = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
    let mut flown = 0;
    let mut eighths = 0;
    for _ in 0..400 {
        if s.units[u].inside.is_some() {
            break;
        }
        if (i64::from(s.units[u].index) + s.frame) & 7 == 0 {
            eighths += 1;
        }
        s.phase_marks.clear();
        s.tick();
        flown += 1;
        let draws = s
            .phase_marks
            .iter()
            .filter(|(l, _)| l == crate::air::SITE_AIR_ALT)
            .count();
        assert!(draws <= 1, "one redraw a frame at most");
    }
    assert_eq!(s.units[u].inside, Some(base), "landed after {flown} frames");
    assert!(flown > 20, "it flew: {flown} frames");
    assert!(eighths >= 2);
    let g = s.units[u].guys[0];
    assert!(g.stopped, "the landing frame's standing arm");
    assert_eq!(s.units[u].movement.body.last_speed, 0);
    // The inside arm puts `last_z` onto `z` from the next frame on.
    s.tick();
    let af = s.units[u].airframe;
    assert_eq!(af.last_z, af.z);
}

/// A Bomber on run223's numbers (`unitrules.xml`'s `Bomber`): `MOVES
/// 60`, `TURN_SPEED 3`, `RANGE 1-3`, `RECHARGE 30`, on the `0x130`
/// line, its figure seated at `z`, banked `bank` (the guy's sign) and
/// pitched `pitch`, all by their bits.
fn bomber(s: &mut Sim, at: Pos, heading: Angle) -> usize {
    let t = UnitType {
        hits: 300,
        moves: 60,
        turn_speed: crate::movement::degrees_to_angle(3).0,
        kind: crate::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::attrition::UnitKind::default()
        },
        combat: combat::Profile {
            attack: 43,
            uber_size: 1,
            domain: Domain::Air,
            min_range: 1,
            max_range: 3,
            recharge: 30,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    let ty = s.add_unit_type(t);
    s.tables.unit_types[ty].tree = Some(0x130);
    let u = spawn(s, 0, ty, at);
    s.units[u].kind = s.tables.unit_types[ty].kind;
    s.units[u].movement.speed = 60;
    s.units[u].movement.turning = crate::turning_of(&s.tables.unit_types[ty]);
    s.units[u].movement.heading = heading;
    s.units[u].movement.facing = heading;
    s.init_guys(u, Some(ty));
    u
}

/// who=1's building made a target: started, active, a combat profile
/// and run223's Barracks' 1200 hit points.
fn live_barracks(s: &mut Sim, b: usize) {
    let bd = &mut s.buildings[b];
    bd.started = true;
    bd.active = true;
    bd.combat = Some(combat::Profile {
        armor: 1,
        ..combat::Profile::default()
    });
    bd.health = 1200;
}

/// **A strike it may not take is a patrol over the strike's point,
/// flown in the same frame: run223's `0/7`, 665 → 666, field for
/// field** (`Unit::do_strafe@005eab00`, `docs/ORDERS.md` §34). The
/// strike is re-pointed at a target `valid_target` refuses (here: no
/// war; on run223, unseen), so `do_strafe` kills it, adds an
/// `AirPatrolOrder` over `xx/yy` with no action bit, and `work` flies
/// it at once. Within 199 of the ground the patrol's bank wants none,
/// so the 40° it came off the runway with steps to 30; the pitch's
/// rate divides by `0x240 / speed`, 9, and climbs 8 → 10, lifting the
/// figure 12. Made to fail with each arm changed: without `work`'s
/// re-entry the plane stands; without the low-bank arm the heading
/// and the bank part; with the distance as the divisor the pitch
/// holds at 8 and the figure rises 10.
#[test]
fn a_strike_it_may_not_take_is_a_patrol_flown_the_same_frame() {
    let mut s = sim();
    let (base, target) = airbase_and_target(&mut s);
    s.at_war[0][1] = false;
    s.at_war[1][0] = false;
    let u = bomber(&mut s, Pos::new(10263, 16330), Angle(1_341_635_061));
    let g = group_of(0, &[u]);
    s.group_action_flight(&g, Obj::Building(base), Flight::Home);
    s.group_action_flight(&g, Obj::Building(target), Flight::Strike);
    {
        let af = &mut s.units[u].airframe;
        af.bank = crate::single::Single::from_bits(0x4220_0000);
        af.pitch = crate::single::Single::from_bits(0x4100_0000);
        af.z = 24;
    }
    s.work(u, 665);
    let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
    assert_eq!(o.len(), 1, "the patrol alone");
    assert_eq!(o[0].index(), index::AIR_PATROL);
    assert_eq!(o[0].flags, 0, "no action bit: slot +0x2c is xor eax, eax");
    let Body::AirPatrol(p) = o[0].body else {
        panic!("an air patrol")
    };
    assert_eq!(p.live(), [Pos::new(21120, 16512)], "the strike's xx/yy");
    assert_eq!((p.home, p.waypoint, p.returning), (Some(base), 0, false));
    assert_eq!(p.cruising_alt, 0x640);
    assert_eq!(s.units[u].pos, Pos::new(10319, 16351), "run223's 666");
    assert_eq!(s.units[u].movement.heading, Angle(1_315_604_981));
    assert_eq!(s.units[u].path[0].to, Pos::new(21120, 16512));
    let af = s.units[u].airframe;
    assert_eq!(af.bank.bits(), 0x41f0_0000, "bank 40 → 30");
    assert_eq!(af.pitch.bits(), 0x4120_0000, "pitch 8 → 10");
    assert_eq!((af.z, af.last_z), (36, 24));
}

/// **The patrol's search pushes its strike first, and takes no
/// `update_action`** (`Unit::do_air_patrol@005ea620`,
/// `add_strafe_order(…, QUEUE_FIRST, 0)@005e48c0`, `docs/ORDERS.md`
/// §34.5): on the sixteenth frame phased by `o` a Bomber asks
/// `find_new_bomber_target` round the patrol's point, and pushes a
/// strafe with `mandatory 0` and no action bit in front of the patrol;
/// `orders_x/y` keep the point `work` wrote before the step (run223's
/// 777).
/// Made to fail with the generic enqueue's `update_action` put back
/// (`orders_pos` is the new point) and with the phase dropped (the
/// search runs on 783).
#[test]
fn the_patrol_s_search_pushes_its_strike_first_without_update_action() {
    let mut s = sim();
    let (base, target) = airbase_and_target(&mut s);
    live_barracks(&mut s, target);
    let u = bomber(&mut s, Pos::new(18803, 16729), Angle(1_014_693_888));
    s.units[u].airframe.z = 1656;
    s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), false);
    // `o` 0: its sixteenth frames are `frame & 15 == 0`, 784 where
    // run223's `0/8` searched on 776.
    assert_eq!(s.units[u].index, 0);
    s.work(u, 783);
    assert_eq!(s.units[u].orders.len(), 1, "783 is not its sixteenth frame");
    let before = s.units[u].pos;
    s.work(u, 784);
    assert_ne!(s.units[u].pos, before, "it flew first");
    let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
    assert_eq!(o.len(), 2, "the strike in front of the patrol");
    assert_eq!(o[0].index(), index::STRAFE);
    assert_eq!(o[0].flags, 0, "no action bit");
    let Body::Strafe(sf) = o[0].body else {
        panic!("a strafe")
    };
    assert_eq!(sf.target, Some(Obj::Building(target)));
    assert!(!sf.mandatory && !sf.returning);
    assert_eq!(sf.home, Some(base));
    assert_eq!(o[1].index(), index::AIR_PATROL);
    // `work`'s own `update_action` ran before the step, so the point
    // is the frame's first; the push after the step leaves it.
    assert_eq!(
        s.units[u].orders_pos, before,
        "no update_action on QUEUE_FIRST"
    );
    assert_ne!(s.units[u].orders_pos, s.units[u].pos);
}

/// **A Bomber releases in range and within 15° of its nose, once, and
/// reloads `recharge() + 1`** (`do_strafe+0x9d0`, `docs/ORDERS.md`
/// §34.3): `CHAR_ATTACK2` on the figure and `recharging 31`, which is
/// run223's `0/8` on 806; reloading, it flies on without a second.
/// Twenty degrees off the nose, it holds its bomb. Made to fail with
/// the angle test dropped (the off-line plane bombs) and with the
/// `+ 1` dropped (`recharging 30`).
#[test]
fn a_bomber_releases_in_range_and_on_its_nose() {
    let mut s = sim();
    let (base, target) = airbase_and_target(&mut s);
    live_barracks(&mut s, target);
    for (anim, len) in [
        (crate::anim::WALK, 31),
        (crate::anim::SLOG, 31),
        (crate::anim::ATTACK2, 30),
    ] {
        s.art.lengths.insert((-1, anim), len);
    }
    let run = |s: &mut Sim, heading: Angle| {
        let u = bomber(s, Pos::new(21120 - 400, 16512), heading);
        s.units[u].airframe.z = 1632;
        s.add_strafe_order(
            u,
            Some(Obj::Building(target)),
            Some(base),
            true,
            crate::orders::QueuePos::New,
            false,
        );
        s.work(u, 805);
        u
    };
    let u = run(&mut s, Angle::EAST);
    assert_eq!(s.units[u].guys[0].anim, crate::anim::ATTACK2);
    assert_eq!(s.units[u].combat.recharging, 31, "recharge() + 1");
    s.work(u, 806);
    assert_eq!(s.units[u].combat.recharging, 31, "no second release");
    let off = Angle(Angle::EAST.0 + crate::movement::degrees_to_angle(20).0);
    let v = run(&mut s, off);
    assert_eq!(s.units[v].combat.recharging, 0, "20° off: no release");
    assert_ne!(s.units[v].guys[0].anim, crate::anim::ATTACK2);
}

/// **A strafe releases its bomb, and the bomb falls** (item 770,
/// `docs/ORDERS.md` §35). The Bomber's release on its first
/// `CHAR_ATTACK2` frame is a round under the strafe's own target; it
/// leaves `node 0`'s bay 19 under the plane's altitude, lands one tile
/// (192) ahead of the bay along the heading with no draw, and takes
/// the fall's 17 frames from 1613 to the ground — run235's `0/8` on
/// 806, `total_time 17`, `sz 1613`, `ex − sx` 191 at 83°. Made to fail
/// with the strafe dropped from the release gate (no round), the
/// ground's height for the plane's (`sz`), the scatter arm kept for a
/// Bomber (the draws) and the fall for the flight (`total_time`).
#[test]
fn a_strafe_releases_a_bomb_that_falls_a_tile_ahead() {
    let mut s = sim();
    let (base, target) = airbase_and_target(&mut s);
    live_barracks(&mut s, target);
    // run223's Barracks is four tiles square: the bomb lands inside.
    if let Some(c) = s.buildings[target].combat.as_mut() {
        c.x_size = 4;
        c.y_size = 4;
    }
    for (anim, len) in [
        (crate::anim::WALK, 31),
        (crate::anim::SLOG, 31),
        (crate::anim::ATTACK2, 30),
    ] {
        s.art.lengths.insert((-1, anim), len);
    }
    let at = Pos::new(21120 - 400, 16512);
    let u = bomber(&mut s, at, Angle::EAST);
    s.units[u].airframe.z = 1632;
    let piece = s.units[u].guys[0].gpiece;
    s.art.releases.insert(
        piece,
        [(
            crate::anim::ATTACK2,
            vec![
                (1, 0, false, false, false, false),
                (2, 1, false, false, false, false),
                (5, 0, false, false, false, false),
                (8, 1, false, false, false, false),
                (10, 0, false, false, false, false),
                (13, 1, false, false, false, false),
                (16, 0, false, false, false, false),
                (18, 1, false, false, false, false),
                (21, 0, false, false, false, false),
                (24, 1, false, false, false, false),
            ],
        )]
        .into_iter()
        .collect(),
    );
    s.add_strafe_order(
        u,
        Some(Obj::Building(target)),
        Some(base),
        true,
        crate::orders::QueuePos::New,
        false,
    );
    s.work(u, 805);
    assert_eq!(s.units[u].guys[0].anim, crate::anim::ATTACK2);
    let seed = s.rng.seed;
    s.guys_inc_time();
    assert_eq!(s.projectiles.len(), 1, "the first event's round");
    assert_eq!(s.rng.seed, seed, "a bomb spends no draw");
    let p = s.projectiles[0];
    assert_eq!(p.target, Some(Obj::Building(target)));
    let from = s.units[u].pos;
    assert_eq!(
        p.launch,
        crate::launch::launch_point(from, Angle::EAST, piece, crate::anim::ATTACK2, 1, 0)
    );
    // The figure's altitude as the release reads it, after `work`'s
    // climb: not the ground's.
    assert_eq!(
        p.sz,
        s.units[u].airframe.z
            + crate::launch::release_dz(piece, crate::anim::ATTACK2, 1, 0).unwrap_or(0)
    );
    assert!(p.sz > 1500);
    assert_eq!(p.landing, Pos::new(p.launch.x + 192, p.launch.y));
    assert_eq!(p.total_time, combat::fall_time(p.sz, p.ez).max(1));
    assert_eq!(p.total_time, 17);
    assert!(!p.rolling);
    for _ in 0..16 {
        s.process_projectiles(806);
    }
    assert_eq!(s.projectiles.len(), 1, "still falling after 16");
    s.process_projectiles(822);
    assert!(s.projectiles.is_empty(), "down on the seventeenth");
    assert!(
        s.hits.iter().any(|h| h.target == Obj::Building(target)),
        "and it strikes"
    );
}

/// **A plane lights the fog it flies over** — `Unit::set_new_location`'s
/// half-cell test and `update_seen(param_3 == 0)`, the ring pass,
/// which `do_air_physics`' step reaches like any other
/// (`docs/ORDERS.md` §34.5). It is how run223's pair first see the
/// Barracks between their searches on 760 and 776. Made to fail with
/// the reveal dropped from the plane's step (no new cell).
#[test]
fn a_flying_plane_lights_the_fog_it_crosses() {
    let mut s = sim();
    assert!(s.world.set_fog(vec![0; 60 * 60 * 4]));
    let (base, _) = airbase_and_target(&mut s);
    let mut t = UnitType {
        hits: 300,
        moves: 60,
        los: 16,
        ..UnitType::default()
    };
    t.kind.domain = Domain::Air;
    t.combat.domain = Domain::Air;
    let ty = s.add_unit_type(t);
    let u = spawn(&mut s, 0, ty, Pos::new(10000, 16000));
    s.units[u].kind = s.tables.unit_types[ty].kind;
    s.units[u].movement.speed = 60;
    s.units[u].movement.turning = crate::turning_of(&s.tables.unit_types[ty]);
    s.units[u].movement.heading = Angle::EAST;
    s.init_guys(u, Some(ty));
    s.add_air_patrol_order(u, Pos::new(30000, 16000), Some(base), false);
    let lit = |s: &Sim| {
        (0..s.world.fog_ys())
            .flat_map(|y| (0..s.world.fog_xs()).map(move |x| (x, y)))
            .filter(|&(x, y)| s.world.seen(x, y).unwrap_or(0) & 1 != 0)
            .count()
    };
    let before = lit(&s);
    for f in 0..20 {
        s.work(u, 801 + f);
    }
    assert!(s.units[u].pos.x > 11000, "it flew east");
    assert!(lit(&s) > before, "the rim it crossed is lit");
}

fn siege_type(sim: &mut Sim) -> usize {
    let mut t = UnitType {
        hits: 100,
        combat: combat::Profile {
            attack: 40,
            uber_size: 1,
            siege: true,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    t.cols.role |= role::ROLE_MILITARY;
    sim.add_unit_type(t)
}

fn spawn(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.tables.unit_types[ty].hits;
    let mut u = Unit::new(who, index, p, hits);
    u.ty = Some(ty);
    u.on_map = true;
    u.movement.speed = 25;
    sim.add_unit(u)
}

fn group_of(who: Player, list: &[usize]) -> Group {
    Group {
        who,
        army: None,
        pushed: None,
        list: list.to_vec(),
    }
}

/// An armed type in one `type_cat` category, by the mask that puts it
/// there (`docs/GROUPS.md` §6.4, `sim::form::type_cat`).
fn typed(sim: &mut Sim, obj_masks: u32, max_range: i32) -> usize {
    let mut t = UnitType {
        hits: 100,
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            obj_masks,
            max_range,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    t.cols.role |= role::ROLE_MILITARY;
    sim.add_unit_type(t)
}

/// `find_leader`'s key is `FormData::type_cat` and nothing else
/// (`docs/GROUPS.md` §4.4, listing `0070ccb0`): the **strictly**
/// lowest category wins, so a tie goes to the first, and the first
/// qualifying member leads until something lower turns up.
///
/// The old simulation took the first on-map captain outright, which is
/// only right for a group of one category — and every group in every
/// dump before run31 was exactly that, which is why no diff caught it.
/// run31 is the capture and it agrees:
/// `run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns`
/// stands its twelve squads up in the harness and `find_leader` names
/// the first **hoplite** captain, member 12 of 36, where the old rule
/// would have named the slinger at member 0.
#[test]
fn find_leader_takes_the_lowest_type_cat_and_ties_to_the_first() {
    use crate::combat::mask;
    use crate::form::{cat, type_cat};
    let mut s = sim();
    let foot = typed(&mut s, mask::FOOT, 0);
    let archer = typed(&mut s, mask::FOOT, 0x600);
    let mounted = typed(&mut s, mask::MOUNTED, 0);
    let mech = typed(&mut s, mask::VEHICLE, 0);
    let human = s.nation[1].human;
    assert_eq!(
        type_cat(&s.tables.unit_types[mech], mech, human),
        cat::FORM_CAT_MECH
    );
    assert_eq!(
        type_cat(&s.tables.unit_types[mounted], mounted, human),
        cat::FORM_CAT_MOUNTED
    );
    assert_eq!(
        type_cat(&s.tables.unit_types[foot], foot, human),
        cat::FORM_CAT_FOOT
    );
    assert_eq!(
        type_cat(&s.tables.unit_types[archer], archer, human),
        cat::FORM_CAT_FOOT_RANGED
    );

    let at = |s: &mut Sim, t: usize, n: i32| spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
    let (a, b, c) = (
        at(&mut s, archer, 0),
        at(&mut s, foot, 1),
        at(&mut s, mounted, 2),
    );
    let d = at(&mut s, mech, 3);
    // Join order is archer, foot, mounted, mech — increasingly early
    // in the category order, so each addition displaces the last.
    assert_eq!(s.group_find_leader(&group_of(1, &[a])), Some(a));
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b])), Some(b));
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c])), Some(c));
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(d));
    // And the other way round nothing displaces the first.
    assert_eq!(s.group_find_leader(&group_of(1, &[d, c, b, a])), Some(d));

    // A tie goes to the first: `cat < best` is strict.
    let e = at(&mut s, foot, 4);
    assert_eq!(s.group_find_leader(&group_of(1, &[b, e])), Some(b));
    assert_eq!(s.group_find_leader(&group_of(1, &[e, b])), Some(e));

    // Two passes, and only the first tests `is_on_map`: the mech is
    // the lowest category but garrisoned, so the on-map pass takes the
    // mounted unit and the second pass never runs.
    s.units[d].on_map = false;
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(c));
    // With nobody on the map at all the second pass finds it.
    for u in [a, b, c] {
        s.units[u].on_map = false;
    }
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(d));
    // A dead member is never a leader, in either pass.
    for u in [a, b, c, d] {
        s.units[u].health = 0;
    }
    assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), None);
}

/// `Group::add`'s two recursions (§4.1): a **captain** drags its figures
/// in behind it, and a **figure** is refused and its captain taken
/// instead. That is why a player's selection of twelve squads is a
/// group of thirty-six, and it is the shape run31 dumps.
#[test]
fn adding_a_captain_takes_its_figures_and_adding_a_figure_takes_its_captain() {
    let mut s = sim();
    let t = fighter(&mut s);
    let squad = |s: &mut Sim, n: i32| {
        let c = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
        let f1 = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
        let f2 = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
        for f in [f1, f2] {
            s.units[f].captain = false;
            s.units[f].o_up = Some(c);
        }
        s.units[c].o_down = Some(f1);
        s.units[f1].o_down = Some(f2);
        (c, f1, f2)
    };
    let (c0, a0, b0) = squad(&mut s, 0);
    let (c1, a1, b1) = squad(&mut s, 1);

    let mut g = Group::stack(1);
    s.group_add(&mut g, c0);
    s.group_add(&mut g, c1);
    assert_eq!(
        g.list,
        vec![c0, a0, b0, c1, a1, b1],
        "each captain followed by its own figures, in `o_down` order"
    );

    // A figure is never a member in its own right: asking for one adds
    // its captain, and with the captain already in, nothing happens.
    let mut h = Group::stack(1);
    s.group_add(&mut h, b1);
    assert_eq!(h.list, vec![c1, a1, b1], "the figure's whole squad");
    let before = h.list.clone();
    s.group_add(&mut h, a1);
    assert_eq!(h.list, before, "and a second ask is a no-op");

    // A dead figure is left out, and its own subordinate with it —
    // `+0x90 >= 0` is tested on the object, not the chain.
    s.units[a0].health = 0;
    let mut k = Group::stack(1);
    s.group_add(&mut k, c0);
    assert_eq!(k.list, vec![c0], "the chain stops at the dead figure");
}

/// `Group::refresh_group_order` (§6.8) — the second writer of `off`,
/// and the one that makes run31's frame 204 different from its frame
/// 328 without a single number of the layout changing.
///
/// The block is slid so that the taking member sits on the origin, over
/// `form_num` entries; `curr` is then re-rotated by **that** member's
/// heading rather than the old leader's.
#[test]
fn a_refresh_re_origins_the_table_onto_the_member_that_took_it_over() {
    let mut st = GroupState {
        form_num: 3,
        off: vec![(0, 0), (-9, 0), (9, -3), (99, 99)],
        ..GroupState::default()
    };
    st.reorigin(1);
    assert_eq!(
        st.off,
        vec![(9, 0), (0, 0), (18, -3), (99, 99)],
        "slid by member 1's own slot, and only over form_num of them"
    );
    // Idempotent on the member it is already origined at.
    let again = st.off.clone();
    st.reorigin(1);
    assert_eq!(st.off, again);
    // A slot past `form_num` is not an origin the walk can reach, but
    // asking for one still slides the block it does reach.
    st.reorigin(2);
    assert_eq!(st.off, vec![(-9, 3), (-18, 3), (0, 0), (99, 99)]);
}

/// §6.3's **tail negation** — `bVar8` at `707e09` and the loop at
/// `707eb8` — which is not the mirror above it and which this crate
/// went without until item 267.
///
/// With the caller supplying the angle (`set_angle`, which is every
/// army call) `compute_form` measures its own `find_angle(dest −
/// group_loc)` against it, and when the two are `90°` or more apart it
/// negates every member's `off_x/off_y` on **both** the `Form` and the
/// group *after* `Form::compute` has written them — leaving `to`
/// alone. So two groups given the same destination and the same angle
/// from opposite sides march to the **same slots** and hold
/// **opposite offsets**, and the offsets are where each follower
/// stands relative to its leader on every later frame
/// (`Group::update_positions` → `curr` → `do_group_move`).
///
/// Angle 0 is north, which is also a fresh unit's heading, so the
/// `facing` toggle above cannot fire and the tail is measured alone.
///
/// Made to fail on purpose twice: with the negation dropped the two
/// offset tables agree, and with `to` negated alongside them the two
/// destination lists no longer do.
#[test]
fn compute_form_negates_the_offsets_when_the_bearing_opposes_the_angle() {
    const DEST: Pos = Pos::new(0x4000, 0x4000);
    let run = |from: Pos| {
        let mut s = sim();
        // A type with a formation footprint — `fighter`'s spacing is
        // zero, and a table of zeroes negates to itself.
        let t = {
            let mut t = UnitType {
                hits: 100,
                combat: combat::Profile {
                    attack: 15,
                    uber_size: 1,
                    x_spacing: 0xc0,
                    y_spacing: 0x180,
                    ..combat::Profile::default()
                },
                ..UnitType::default()
            };
            t.cols.role |= role::ROLE_MILITARY;
            s.add_unit_type(t)
        };
        let slot = s.init_army(1, None);
        let a = spawn(&mut s, 1, t, from);
        let b = spawn(&mut s, 1, t, Pos::new(from.x + 0x100, from.y));
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            DEST,
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        let off = s.armies[1].list[slot].group.off.clone();
        let dests: Vec<Pos> = [a, b]
            .iter()
            .map(|&u| match s.current_order(u).expect("a move order").body {
                Body::Move(m) => m.dest,
                _ => unreachable!("a group move is a move"),
            })
            .collect();
        (off, dests)
    };
    // South of the destination: the bearing to it is north, which is
    // the angle, so the tail does not fire.
    let (aligned, aligned_dests) = run(Pos::new(DEST.x, DEST.y + 0x2000));
    // North of it: the bearing is due south, 180° from the angle.
    let (opposed, opposed_dests) = run(Pos::new(DEST.x, DEST.y - 0x2000));

    assert!(
        aligned.iter().any(|&o| o != (0, 0)),
        "a table of zeroes negates to itself and says nothing: {aligned:?}"
    );
    assert_eq!(
        aligned_dests, opposed_dests,
        "the tail leaves the destinations alone"
    );
    assert_eq!(
        opposed,
        aligned
            .iter()
            .map(|&(x, y)| (-x, -y))
            .collect::<Vec<(i32, i32)>>(),
        "the offsets are negated: {aligned:?} against {opposed:?}"
    );
}

/// **The group's cap is a one-pass-lagged minimum, not a minimum**
/// (§18), and the lag is the whole of item 515's residue — a
/// `compute_speed`-shaped model walks the original's own raid at 25
/// on frames it walks at 26.
///
/// Made to fail on purpose three ways: with
/// [`Sim::group_leader_report_speed`] writing `speed` from its own
/// argument rather than from the accumulator, the second row is 26
/// and the sixth is 26; with [`Sim::group_report_speed`] dropping
/// its `<` test, the third row is 25 and the fifth is 30; with it
/// writing only `new_speed`, the fourth row is 26.
#[test]
fn the_group_s_cap_lags_one_pass_behind_what_its_members_report() {
    let mut s = sim();
    let t = fighter(&mut s);
    let fast = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let slow = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    s.units[fast].movement.speed = 26;
    s.units[slow].movement.speed = 25;
    let mut g = group_of(1, &[fast, slow]);
    assert!(s.push_group(&mut g, true), "the pair takes a slot");
    let cap = |s: &Sim| s.group_speed_of(fast);

    // A freshly seated group caps nothing until something reports.
    assert_eq!(cap(&s), 0, "zero is `Group::clear`'s, and caps nothing");

    // **A zero cap is a trap, and it is the original's.**
    // `report_speed` only ever *lowers*, so a follower cannot get a
    // group off zero: the first two frames of a march out of a
    // freshly seated group run uncapped whatever the followers say,
    // and it takes the leader's second publication to seed the pair.
    // That is why [`Sim::group_normalize`] priming the group matters
    // at all — without it, Great Lakes' raid walks its first frames
    // at the raider's own 26.
    s.group_leader_report_speed(&g, 26);
    assert_eq!(cap(&s), 0, "the accumulator was empty, so the cap still is");
    s.group_report_speed(&g, 25);
    assert_eq!(
        cap(&s),
        0,
        "25 is not less than 0, so the report is dropped"
    );

    // Frame 2: the leader publishes the accumulator it restarted at
    // its own speed, and only now does the group cap anything.
    s.group_leader_report_speed(&g, 26);
    assert_eq!(cap(&s), 26, "the leader's own pass, one frame late");

    // And now a follower can drive it down, both halves at once.
    s.group_report_speed(&g, 25);
    assert_eq!(cap(&s), 25, "a report lowers");
    s.group_report_speed(&g, 30);
    assert_eq!(cap(&s), 25, "and never raises: 30 is not less than 25");

    // Frame 3: the leader publishes the pass the follower reported
    // into, so the cap **stays** at 25 while the accumulator
    // restarts at 26.
    s.group_leader_report_speed(&g, 26);
    assert_eq!(cap(&s), 25, "the pass the follower reported into");

    // Frame 4, with the follower no longer reporting — it has
    // arrived, or it is fighting rather than marching. **Now** the
    // cap drifts back up, one frame late. This is the row a
    // `compute_speed`-shaped model gets wrong, and it is five frames
    // of Great Lakes' raid.
    s.group_leader_report_speed(&g, 26);
    assert_eq!(cap(&s), 26, "no report, so the leader's own speed again");
}

/// **`Group::normalize`'s tail puts the cap back on its leader**
/// (§18) — the site that primes a group before its first march, and
/// the reason the first frame of Great Lakes' raid is capped rather
/// than running at the raider's own speed.
///
/// The leader is `find_leader`'s, so the **slow** type leads here
/// only because its `type_cat` is lower; made to fail by giving both
/// types the same category, which hands the lead to the first member
/// of the list and the cap to 26.
#[test]
fn normalizing_a_group_puts_its_cap_back_on_its_leader() {
    use crate::combat::mask;
    let mut s = sim();
    // Mounted outranks foot in `type_cat`, so the mounted type
    // leads whichever order the list is in.
    let foot = typed(&mut s, mask::FOOT, 0);
    let horse = typed(&mut s, mask::MOUNTED, 0);
    let fast = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let slow = spawn(&mut s, 1, horse, Pos::new(0x1100, 0x1000));
    s.units[fast].movement.speed = 26;
    s.units[slow].movement.speed = 25;
    let mut g = group_of(1, &[fast, slow]);
    assert!(s.push_group(&mut g, true));
    assert_eq!(
        s.group_find_leader(&g),
        Some(slow),
        "the mounted type leads on `type_cat`, not on list order"
    );
    // Walk the cap up the way a march does, then normalize.
    s.group_leader_report_speed(&g, 26);
    s.group_leader_report_speed(&g, 26);
    assert_eq!(s.group_speed_of(fast), 26);
    s.group_normalize(&mut g);
    assert_eq!(
        s.group_speed_of(fast),
        25,
        "`compute_speed` is the leader's own speed, and it is the slow one"
    );
    assert_eq!(
        s.gstate(&g).map(|st| st.new_speed),
        Some(25),
        "and it writes both halves of the pair"
    );
}

/// **`Groups::process` resets one pool slot a frame, and the slot is
/// the frame mod 64** (§19). This is the writer §18 did not count:
/// Great Lakes' raid is player 1's slot 1, `group 65`, and the frame
/// its cap goes back to the slow squad's 25 is 10241 — `65 mod 64`'s —
/// with nobody in the group able to report.
///
/// Made to fail on purpose three ways: with the cursor one frame late
/// (`frame − 1`) the cap is still 26 after 10241; with the pass
/// removed from nowhere but here, the same; with the reset writing
/// only `speed`, the last row's `new_speed` is 26.
#[test]
fn groups_process_resets_one_pool_slot_a_frame_on_the_frame_mod_64() {
    use crate::combat::mask;
    let mut s = sim();
    let foot = typed(&mut s, mask::FOOT, 0);
    let horse = typed(&mut s, mask::MOUNTED, 0);
    let fast = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let slow = spawn(&mut s, 1, horse, Pos::new(0x1100, 0x1000));
    s.units[fast].movement.speed = 26;
    s.units[slow].movement.speed = 25;
    let mut g = group_of(1, &[fast, slow]);
    assert!(s.push_group(&mut g, true));
    assert_eq!(
        s.pool_group_of(fast),
        65,
        "a player's first push skips `last_group`'s initial slot 0"
    );
    // A march's reports walk the cap up to the leader's own 26 …
    s.group_leader_report_speed(&g, 26);
    s.group_leader_report_speed(&g, 26);
    assert_eq!(s.group_speed_of(fast), 26);
    // … and a pass on any other slot leaves it there.
    s.groups_process(10_240);
    assert_eq!(s.group_speed_of(fast), 26, "10240 is slot 0's frame");
    // 10241 is slot 1's: the cap goes back to `find_leader`'s speed,
    // the slow squad's, both halves.
    s.groups_process(10_241);
    assert_eq!(s.group_speed_of(fast), 25, "10241 is `65 mod 64`'s frame");
    assert_eq!(s.gstate(&g).map(|st| st.new_speed), Some(25));
    // And the next frame for the slot is 64 later.
    s.group_leader_report_speed(&g, 26);
    s.group_leader_report_speed(&g, 26);
    for f in 10_242..10_305 {
        s.groups_process(f);
    }
    assert_eq!(s.group_speed_of(fast), 26, "no reset between");
    s.groups_process(10_305);
    assert_eq!(s.group_speed_of(fast), 25, "and one at 10305");
}

/// **A pushed group's prune takes the per-member arrays with it**
/// (`normalize@00711540`'s inner loop shifts `off`, `curr` and `angles`
/// over the member it drops, item 1776): chapter fifty-three's pool slot
/// 1 on 962 held a member the original had dropped, and its `off`,
/// `curr` and `angle` rows parted one slot out.
///
/// Made to fail with the shift removed: the arrays keep three entries.
#[test]
fn a_pushed_group_s_prune_shifts_its_per_member_arrays() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let mut g = group_of(1, &[a, b, c]);
    assert!(s.push_group(&mut g, true));
    let slot = s.pool_group_of(a);
    let i = s.pushed.len() - 1;
    let st = &mut s.pushed[i].state;
    st.off = vec![(1, 1), (2, 2), (3, 3)];
    st.curr = vec![Pos::new(10, 10), Pos::new(20, 20), Pos::new(30, 30)];
    st.angles = vec![1, 2, 3];
    s.units[b].group_ptr = None;
    s.groups_process(slot);
    let p = &s.pushed[i];
    assert_eq!(p.list, vec![a, c]);
    assert_eq!(p.state.off, vec![(1, 1), (3, 3)]);
    assert_eq!(p.state.curr, vec![Pos::new(10, 10), Pos::new(30, 30)]);
    assert_eq!(p.state.angles, vec![1, 3]);
}

/// **`push_group`'s numbering** (§3.1, §3.2, §19): a group equal to
/// `last_group`'s reuses it, anything else takes the lowest empty slot
/// that is not `last_group` — which is how Great Lakes' scout walks
/// 65 → 64 → 66 across its pushes while one slot is held elsewhere.
///
/// Made to fail on purpose twice: without the `last_group` exclusion
/// the first push is 64; without the equality reuse the second is 64.
#[test]
fn a_push_takes_the_lowest_empty_slot_that_is_not_the_last() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let push = |s: &mut Sim, list: &[usize]| {
        let mut g = group_of(1, list);
        assert!(s.push_group(&mut g, true));
        s.pool_group_of(list[0])
    };
    assert_eq!(push(&mut s, &[a]), 65, "slot 0 is `last_group` at start");
    assert_eq!(push(&mut s, &[a]), 65, "the same group again reuses it");
    assert_eq!(push(&mut s, &[b, c]), 64, "slot 0 is free and not the last");
    assert_eq!(
        push(&mut s, &[a]),
        66,
        "65 is not the last now, but `a` still holds it — and it leaves"
    );
    assert_eq!(push(&mut s, &[b]), 65, "65 emptied when `a` left it");
}

#[test]
fn push_group_refuses_a_singleton_unless_forced() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let mut one = group_of(1, &[a]);
    assert!(
        !s.push_group(&mut one, false),
        "a group of fewer than two takes no slot without force"
    );
    assert!(s.push_group(&mut one, true), "Army::add_unit forces it");
    assert_eq!(one.pushed, Some(0), "and the forced push names its slot");
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    assert!(s.push_group(&mut group_of(1, &[a, b]), false));
}

/// **`get_num` of an empty group zeroes its origin and normalizes
/// nothing; the slot scan calls it on every slot it reaches** (twenty-
/// fourth pass, groups 13 and 14; A9 rows 12 and 15). A slot of one to
/// three members that the scan passes has its speed pair reset to its
/// leader's, and an emptied slot keeps its speed while its `(ox, oy)`
/// goes to zero.
#[test]
fn the_slot_scan_calls_get_num_on_every_slot_it_reaches() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let mut first = group_of(1, &[a, b]);
    assert!(s.push_group(&mut first, true));
    let slot = first.pushed.expect("seated");
    let pool = s.pushed[slot].state.pool;
    // The slot's record goes stale: the scan must price it again.
    s.pushed[slot].state.speed = 99;
    s.pushed[slot].state.new_speed = 99;
    // Slot 0 is `last_group`, so the walk goes past it to this slot and
    // on to the first open one.
    s.last_group[1] = 0;
    let open = s.open_slot(1);
    assert_ne!(Some(open), pool, "a seated slot of two is not open");
    let leader = s.units[a].movement.speed;
    assert_eq!(s.pushed[slot].state.speed, leader, "normalized by get_num");
    // An emptied seat: speed stands, origin zeroed, no normalize.
    s.pushed[slot].list.clear();
    s.pushed[slot].state.speed = 7;
    s.pushed[slot].state.o = Pos::new(5, 5);
    let seat = Seat::Pushed(slot);
    assert_eq!(s.seat_get_num(seat), 0);
    assert_eq!(s.pushed[slot].state.o, Pos::new(0, 0));
    assert_eq!(
        s.pushed[slot].state.speed, 7,
        "no normalize on an empty arm"
    );
}

/// **The cross-region arm re-points its member** (group 15; A9 row 34):
/// a listed member whose `+0x80` named another slot names this group
/// after the arm; an unseated group writes nothing.
#[test]
fn the_cross_region_arm_re_points_a_stale_member() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let mut g = group_of(1, &[a, b]);
    assert!(s.push_group(&mut g, true));
    let pool = s.gstate(&g).and_then(|st| st.pool).expect("seated");
    s.units[b].group_ptr = Some(pool + 1);
    s.repoint_member(&g, b);
    assert_eq!(s.units[b].group_ptr, Some(pool));
    let loose = group_of(1, &[a]);
    s.units[a].group_ptr = Some(40);
    s.repoint_member(&loose, a);
    assert_eq!(s.units[a].group_ptr, Some(40), "no seat, no id");
}

/// **A pushed slot's own state** (item 1457, `docs/GROUPS.md` §36): a
/// stack group's point is `Group::clear`'s (0, 0), which `copy_group`
/// carries into the slot; and every push opens with `equals_group`'s
/// normalize of the last pushed slot, whose tail sets its speed to its
/// leader's. Made to fail by copying the default's (−1, −1), or by
/// skipping the normalize (the speed stays 0).
#[test]
fn a_pushed_slot_starts_at_the_origin_and_the_next_push_prices_it() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x4000, 0x4000));
    let d = spawn(&mut s, 1, t, Pos::new(0x4100, 0x4000));
    let mut first = group_of(1, &[a, b]);
    assert!(s.push_group(&mut first, true));
    let slot = first.pushed.expect("seated");
    assert_eq!(s.pushed[slot].state.o, Pos::new(0, 0));
    assert_eq!(s.pushed[slot].state.speed, 0, "a copy carries no speed");
    let mut second = group_of(1, &[c, d]);
    assert!(s.push_group(&mut second, true));
    let leader = s.units[a].movement.speed;
    assert!(leader > 0);
    assert_eq!(
        s.pushed[slot].state.speed, leader,
        "the second push normalized the first slot"
    );
}

/// **A pushed group is still a group the next frame** — the pool
/// slot item 465 gave `push_group`, and the reason Great Lakes'
/// probe can march in formation at all.
///
/// The probe (`docs/ARMY.md` §12) pushes six of the army's raiders
/// out of army 1 on frame 8186 and the dump carries them on
/// `group 65` for the next two thousand blocks. With one slot and no
/// record the scout's next `go_to` overwrote them — eleven times
/// before the word — so `do_group_move` could not run on them and
/// the whole follower arm was dead code on the only capture that
/// reaches it (`docs/ORDERS.md` §16).
///
/// **Made to fail on purpose**: with `pushed` back to one slot the
/// second push takes the first group's seat and `group_of` answers
/// `None` for `a` and `b`.
#[test]
fn a_pushed_group_outlives_the_next_push() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x4000, 0x4000));
    let mut first = group_of(1, &[a, b]);
    assert!(s.push_group(&mut first, true));
    let mut second = group_of(1, &[c]);
    assert!(s.push_group(&mut second, true));
    assert_eq!(
        (first.pushed, second.pushed),
        (Some(0), Some(1)),
        "two live groups take two slots"
    );
    assert_eq!(
        s.group_of(a).map(|g| g.list),
        Some(vec![a, b]),
        "the first group is still the first group's"
    );
    assert_eq!(
        s.group_of(c).map(|g| g.list),
        Some(vec![c]),
        "and the second is its own"
    );
    // `Groups::get_open_slot` (§3.1, §28.4): a slot whose `get_num`
    // is zero is taken before a new one — but never `last_group`, and
    // the dead group's slot is `last_group` until another push moves
    // it. So the third push takes a fresh slot, and the fourth
    // recycles the dead one.
    s.units[c].health = 0;
    let d = spawn(&mut s, 1, t, Pos::new(0x5000, 0x5000));
    let mut third = group_of(1, &[d]);
    assert!(s.push_group(&mut third, true));
    assert_eq!(third.pushed, Some(2), "`last_group` is not recycled");
    let e = spawn(&mut s, 1, t, Pos::new(0x6000, 0x6000));
    let mut fourth = group_of(1, &[e]);
    assert!(s.push_group(&mut fourth, true));
    assert_eq!(fourth.pushed, Some(1), "the dead group's slot is recycled");
    assert_eq!(s.pushed.len(), 3, "and the pool does not grow");
}

/// **A seated group's move is a `GroupMoveOrder`, and an in-danger
/// member's is not** — §6.6 step 6's gate as the original states it
/// (`705f00`-`705f61`), which has no army test in it at all.
///
/// This crate carried one — `g.army.is_some()` — for four months,
/// and it was standing in for the wrong thing: what the original
/// exempts a `Unit::go_to` group by is `unit_masks & 4`, which
/// `go_to_unit@005f78c0` sets over the whole squad one line above
/// the walk. Both halves are asserted here, because dropping the
/// army test without carrying the bit collapses Great Lakes' word to
/// **6994** — the second squad's own walk to the army, which is a
/// `go_to` (`docs/ORDERS.md` §16).
#[test]
fn a_pushed_group_marches_unless_its_member_is_in_danger() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let mut g = group_of(1, &[a, b]);
    assert!(s.push_group(&mut g, true));
    s.group_action_move_to(
        &g,
        Pos::new(0x8000, 0x8000),
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let grouped = |s: &Sim, u: usize| match s.current_order(u).expect("a move").body {
        Body::Move(m) => m.group.is_some(),
        _ => unreachable!("a move is a move"),
    };
    assert!(
        grouped(&s, a) && grouped(&s, b),
        "a seated group of two on a MOVE_TO marches in formation"
    );
    // And the same group with one member marked: only that member
    // falls back to a plain move.
    s.units[a].in_danger = true;
    s.group_action_move_to(
        &g,
        Pos::new(0x2000, 0x8000),
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    assert_eq!(
        (grouped(&s, a), grouped(&s, b)),
        (false, true),
        "`unit_masks & 4` is a per-member exemption, not the group's"
    );
}

/// **A pushed group takes its members out of the army** — §3.2's
/// third bullet (`docs/ARMY.md` §3.4), which the first reading had
/// right and no implementation carried until item 350.
///
/// Great Lakes is the case: §12's probe pushes two of the army's
/// units, `Group::add` brings both squads, and the original's army
/// issues every later order to the nine that are left. Here the
/// fixture is the same shape at two units — push one of them, and the
/// army is one unit, one captain, one standard.
#[test]
fn a_pushed_group_takes_its_members_out_of_the_army() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    assert_eq!(
        (
            s.armies[1].list[slot].units.clone(),
            s.armies[1].list[slot].num_captains
        ),
        (vec![a, b], 2),
        "the army's one group holds both"
    );
    // `push_group(force = 1)`, the probe's own call.
    assert!(s.push_group(&mut group_of(1, &[a]), true));
    assert_eq!(
        s.armies[1].list[slot].units,
        vec![b],
        "the pushed member is killed out of the army's group"
    );
    assert_eq!(
        (
            s.armies[1].list[slot].num_units,
            s.armies[1].list[slot].num_captains,
            s.armies[1].list[slot].num_standard
        ),
        (1, 1, 1),
        "and `Army::normalize` recounts what is left"
    );
    // A group the army never held leaves it alone.
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    assert!(s.push_group(&mut group_of(1, &[c]), true));
    assert_eq!(
        s.armies[1].list[slot].units,
        vec![b],
        "a non-member's push is not the army's business"
    );
}

/// **A member that stays keeps its own slot** (item 1171,
/// `docs/GROUPS.md` §30.3): `Group::kill@00714110` shifts the four
/// offset arrays and the angle byte down with the list, so the
/// formation a follower reads at its new index is still its own.
///
/// Great Sahara's frame 14586 is the case: six members of army group
/// 64 leave for group 68, and `1/52`, listed eighth and now fifth,
/// walked to the fifth *old* slot — `1/47`'s — and stood.
///
/// **Made to fail on purpose**: with `kill_from_named` back to a
/// retain on the list alone, `b` reads `a`'s offset.
#[test]
fn a_member_leaving_an_army_group_takes_its_slot_with_it() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let slot = s.init_army(1, None);
    for u in [a, b, c] {
        s.army_add_unit(1, slot, u);
    }
    let st = &mut s.armies[1].list[slot].group;
    st.off = vec![(1, 1), (2, 2), (3, 3)];
    st.curr = vec![Pos::new(10, 10), Pos::new(20, 20), Pos::new(30, 30)];
    st.angles = vec![1, 2, 3];
    assert!(s.push_group(&mut group_of(1, &[a]), true));
    let st = &s.armies[1].list[slot].group;
    assert_eq!(s.armies[1].list[slot].units, vec![b, c], "`a` left");
    assert_eq!(
        (st.off.clone(), st.curr.clone(), st.angles.clone()),
        (
            vec![(2, 2), (3, 3)],
            vec![Pos::new(20, 20), Pos::new(30, 30)],
            vec![2, 3]
        ),
        "the arrays shift with the list"
    );
    let g = s.group_of(b).expect("`b` is still the army's");
    assert_eq!(
        s.group_slot_point(&g, b, 0),
        Some(Pos::new(0x1100 + 20, 0x1000 + 20)),
        "and `b`, now first, reads its own slot"
    );
}

/// **A member that takes the formation over turns it by its waypoint**
/// (item 1206, `docs/GROUPS.md` §6.8): `refresh_group_order`'s tail is
/// `update_positions` whole (`713b94: call 0x713810`), so when the
/// taking member's move has a waypoint the table points at it, not
/// along the member's heading. Great Sahara's 17493 is the case: `1/51`
/// leaves army group 64, `1/62` takes it over standing, and the dumped
/// `curr` is `find_angle` to `1/62`'s waypoint, 2.5° off its heading.
///
/// **Made to fail on purpose**: with the tail back to a turn by
/// `movement.heading`, the table is the heading's.
#[test]
fn a_member_taking_the_formation_over_turns_it_by_its_waypoint() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    // run457's 17493, `1/62`: heading −615972864, waypoint (−7, −6)
    // off its own square.
    let here = s.units[b].pos;
    let wp = Pos::new(here.x - 7, here.y - 6);
    s.units[b].movement.heading = Angle(-615_972_864);
    for o in &mut s.units[b].orders {
        if let Some(m) = o.move_mut() {
            m.has_waypoint = true;
            m.waypoint = wp;
        }
    }
    let i = g.list.iter().position(|&u| u == b).expect("b is listed");
    s.armies[1].list[slot].group.off = if i == 0 {
        vec![(0, 0), (21, 0)]
    } else {
        vec![(21, 0), (0, 0)]
    };
    assert!(s.group_refresh_order(&g, b));
    let st = &s.armies[1].list[slot].group;
    let theta = find_angle(-7, -6);
    assert_eq!(theta, Angle(-586_678_272), "the bearing run457 dumps");
    assert_eq!(
        st.curr,
        Sim::form_update_positions(&st.off, theta),
        "the table turns by the waypoint's bearing"
    );
    assert_ne!(
        st.curr,
        Sim::form_update_positions(&st.off, Angle(-615_972_864)),
        "and not by the heading"
    );
}

/// An army of two on land takes **`GroupMoveOrder`s** and a lone one
/// does not (`docs/ORDERS.md` §8.2's gate), and a group with no army
/// takes none at all — this crate's own line, since a player's
/// selection has no persistent group for `do_group_move` to read.
/// **A group move takes the builder's mark off** (item 3710,
/// `Group::action_move_near@00704990`'s `705fe3`, after either arm's
/// order): run1370's Citizen `1/95` left its site for group 80's move
/// between 4001 and 4101 and lost `unit_masks & 0x400`; this crate kept
/// `was_builder` and its idle think on 4841 searched for a build. Both
/// arms: an army's `GroupMoveOrder` and a selection's plain move.
#[test]
fn a_group_move_takes_the_builder_s_mark_off() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    s.units[a].was_builder = true;
    s.units[b].was_builder = true;
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    assert!(s.current_move(a).expect("a move").group.is_some());
    assert!(
        !s.units[a].was_builder && !s.units[b].was_builder,
        "the grouped arm"
    );

    let plain = group_of(1, &[a, b]);
    s.units[a].was_builder = true;
    s.group_action_move_to(
        &plain,
        Pos::new(0x5000, 0x5000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    assert!(s.current_move(a).expect("a move").group.is_none());
    assert!(!s.units[a].was_builder, "the plain arm");
}

#[test]
fn an_army_s_move_is_a_group_order_and_a_selection_s_is_not() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    let leader = s.group_find_leader(&g).expect("a leader");
    for u in [a, b] {
        let m = s.current_move(u).expect("a move order");
        let gm = m.group.expect("a group order");
        assert_eq!(gm.leader, leader, "every member names one leader");
        // who 1's pool starts at 64 and the army sits in its first push's
        // slot, 1, since slot 0 is `last_group`'s (items 1863, 1926).
        assert_eq!(gm.id, group_move_id(64 + 1, s.frame, 0));
        assert!(!gm.in_group, "`GroupMoveOrder::clear` leaves it 0");
    }
    assert_eq!(
        s.current_move(a).expect("a move").group.expect("g").form_id,
        0
    );
    assert_eq!(
        s.current_move(b).expect("a move").group.expect("g").form_id,
        1
    );

    // The same two as a plain selection: no army, so no group order.
    let plain = group_of(1, &[a, b]);
    s.group_action_move_to(
        &plain,
        Pos::new(0x5000, 0x5000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    for u in [a, b] {
        assert!(
            s.current_move(u).expect("a move order").group.is_none(),
            "a selection has no group to hang the order on"
        );
    }

    // And an army of one: `group.num < 2` is an exemption of the
    // original's own.
    let c = spawn(&mut s, 1, t, Pos::new(0x2000, 0x2000));
    let solo = s.init_army(1, None);
    s.army_add_unit(1, solo, c);
    let one = s.army_group(1, solo);
    s.group_action_move_to(
        &one,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    assert!(
        s.current_move(c).expect("a move order").group.is_none(),
        "fewer than two is a plain move"
    );
}

/// **A `QUEUE_LAST` move is laid out from where the leader ends up**
/// (`docs/ORDERS.md` §17.2): `Group::action_move_near` asks
/// `GroupData::get_loc_to` for that one queue position and `get_loc`
/// for every other (`00704990:361`–`364`), and the two answers are the
/// whole formation angle when something is queued ahead.
///
/// The scenario is Great Lakes' probe in miniature: a pair standing at
/// the point they are about to be sent to, holding an attack order on a
/// building far to the west. Read from their feet the delta is zero and
/// the block has no bearing; read from the building it points east, and
/// the order carries that bearing.
///
/// **Made to fail on purpose** by asking `group_loc` for both positions
/// — which is what this crate did until item 471 — whereupon the angle
/// is the zero-delta arm's and the two members lay out on an axis.
#[test]
fn a_queued_group_move_takes_its_bearing_from_where_the_leader_ends_up() {
    let mut s = sim();
    let t = fighter(&mut s);
    s.tables.unit_types[t].combat.x_spacing = 0xc0;
    s.tables.unit_types[t].combat.y_spacing = 0xc0;
    let here = Pos::new(0x8000, 0x4000);
    let a = spawn(&mut s, 1, t, here);
    let b = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4200));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    let leader = s.group_find_leader(&g).expect("a leader");
    // The thing queued ahead, and it is the probe's: a target far to
    // the west, which is where `get_final_loc` says the pair will be.
    let away = Pos::new(0x1000, 0x6000);
    let prey = spawn(&mut s, 0, t, away);
    s.units[leader].combat.target = Some(crate::combat::Obj::Unit(prey));
    s.group_action_attack(&g, crate::combat::Obj::Unit(prey), true, QueuePos::New, 0);
    // …and the walk home behind it, to the leader's **own** point.
    s.group_action_move_to(
        &g,
        here,
        QueuePos::Last,
        false,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let want = find_angle(here.x - away.x, here.y - away.y);
    let m = s
        .units
        .iter()
        .find_map(|u| u.orders.iter().find(|o| o.move_dest().is_some()))
        .and_then(|o| match o.body {
            Body::Move(m) => Some(m),
            _ => None,
        })
        .expect("a group move on the pair");
    assert_eq!(
        m.angle, want,
        "the formation angle is `find_angle` from the queued target, not \
             from the leader's feet"
    );
    assert_ne!(want, Angle(0), "the scenario has to have a bearing at all");
}

/// **`get_loc_to`'s `(ox, oy)` window is measured from the leader's own
/// position** (`GroupData::get_loc_to@0070c5d0`, `0070c6cc`–`0070c702`,
/// item 1528): the answer is the leader's *final* location, replaced by
/// the group's `(ox, oy)` only when the **unit** stands within `0x180`
/// of it. This crate measured from the final point, so a leader still
/// far from the point it was sent to — within the window of its own
/// destination — read `(ox, oy)` back, and a formation re-issued at
/// `QUEUE_LAST` was laid out from a zero delta where the original's
/// pointed from the leader's destination: run668's frame 15377,
/// whose second charge's layout the original reversed.
///
/// **Made to fail on purpose** by measuring from `unit_final_loc`'s
/// answer, as this crate did: the first assertion reads `(ox, oy)`.
#[test]
fn get_loc_to_measures_its_window_from_the_leader_not_from_its_destination() {
    let mut s = sim();
    let t = fighter(&mut s);
    let far = Pos::new(0x8000, 0x4000);
    let a = spawn(&mut s, 1, t, far);
    let b = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4200));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    let leader = s.group_find_leader(&g).expect("a leader");
    let to = Pos::new(0x4000, 0x4000);
    s.group_action_move_to(
        &g,
        to,
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let end = s.unit_final_loc(leader);
    assert_ne!(end, s.units[leader].pos, "the leader has somewhere to go");
    // A point within the window of where the leader is going, and far
    // from where it stands.
    let o = Pos::new(end.x + 100, end.y + 100);
    s.gstate_mut(&g).expect("the army's record").o = o;
    assert_eq!(
        s.group_loc_to(&g),
        Some(end),
        "the leader stands far from (ox, oy): the final location answers"
    );
    // Standing within the window, the leader reads (ox, oy) back.
    s.units[leader].pos = Pos::new(o.x + 10, o.y);
    assert_eq!(s.group_loc_to(&g), Some(o));
}

/// **`finish_insert`'s guard arm** (`Group::finish_insert@0070e620`,
/// case `0xc`, item 1528): a leader's guard order, saved ahead of a
/// `QUEUE_FIRST` halt, is re-issued as a **group** guard on the same
/// unit at `QUEUE_LAST` — every member holds it again at the foot of the
/// stack, behind the new order. This crate stood in for it with a no-op,
/// so an army charged onto a city lost its guard (run668's `1/28`:
/// `[GUARD, GROUP_ATTACK_TO, GROUP_ATTACK_TO]` against ours' two).
///
/// **Made to fail on purpose** by dropping the arm: the stack is the
/// new move alone.
#[test]
fn a_group_queue_first_keeps_the_leader_s_guard_at_the_foot() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let buddy = spawn(&mut s, 1, t, Pos::new(0x1300, 0x1000));
    let slot = s.init_army(1, None);
    for u in [a, b, c] {
        s.army_add_unit(1, slot, u);
    }
    let g = s.army_group(1, slot);
    let leader = s.group_find_leader(&g).expect("a leader");
    // The leader guards a unit outside the group, as run668's 1/62
    // guards 1/95, with the action bit a guard order carries.
    s.add_guard_order(leader, buddy, 0, 0, QueuePos::New);
    assert_eq!(s.order_type(leader), index::GUARD);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::First,
        false,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    let stack: Vec<u8> = s.units[leader].orders.iter().map(|o| o.index()).collect();
    assert_eq!(
        stack,
        [index::GROUP_ATTACK_TO, index::GUARD],
        "the new move at the front, the guard behind it"
    );
    // The members that were not guarding anything are given the
    // leader's guard too, at the foot.
    for u in [a, b, c].into_iter().filter(|&u| u != leader) {
        assert_eq!(
            s.units[u].orders.back().map(|o| o.index()),
            Some(index::GUARD),
            "a guard behind the move"
        );
    }
}

/// **`finish_insert` re-issues the guard over the list the move
/// sorted** (item 3517). The original's `this` is the army's own
/// record: the `QUEUE_FIRST` arm's recursive move runs
/// `Form::categorize`'s `Group::sort` on it, and `finish_insert`'s
/// `action_guard` then lays the escort out over the sorted list. An
/// army that lists a squad's follower ahead of the rest — Great
/// Sahara's army 1/0 listed `1/21` second, the tail of `1/19`'s squad —
/// has the squad re-added at the end, and every escort's guard offset
/// is its slot in that order. So the guard comes out as it does for the
/// same army listed in the sorted order to begin with.
///
/// **Made to fail on purpose** by re-issuing over the caller's copy
/// of the list: the squad takes the first slots, and every offset
/// moves.
#[test]
fn a_queue_first_guard_is_laid_out_over_the_sorted_list() {
    let guards = |sorted: bool| {
        let mut s = sim();
        let t = fighter(&mut s);
        s.tables.unit_types[t].combat.x_spacing = 0xc0;
        s.tables.unit_types[t].combat.y_spacing = 0xc0;
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        let cap = spawn(&mut s, 1, t, Pos::new(0x1300, 0x1000));
        let sub = spawn(&mut s, 1, t, Pos::new(0x1330, 0x1000));
        let ward = spawn(&mut s, 1, t, Pos::new(0x2000, 0x1000));
        s.units[sub].captain = false;
        s.units[sub].o_up = Some(cap);
        s.units[cap].o_down = Some(sub);
        let slot = s.init_army(1, None);
        for u in [a, b, c, cap, sub] {
            s.army_add_unit(1, slot, u);
        }
        // The follower listed second, its captain not at all: the
        // list `Group::sort` finds a stray in.
        s.armies[1].list[slot].units = if sorted {
            vec![a, b, c, cap, sub]
        } else {
            vec![a, sub, b, c]
        };
        let g = s.army_group(1, slot);
        for &m in &g.list {
            s.add_guard_order(m, ward, 0, 0, QueuePos::New);
        }
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::First,
            false,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        [a, b, c, cap, sub].map(|u| {
            s.units[u].orders.iter().find_map(|o| match o.body {
                Body::Guard(x) => Some((x.dx, x.dy)),
                _ => None,
            })
        })
    };
    let sorted = guards(true);
    assert!(sorted.iter().all(Option::is_some), "every member guards");
    assert_eq!(
        guards(false),
        sorted,
        "the escort laid out over the list the move sorted"
    );
}

/// **A move to where the group already stands takes the leader's own
/// heading**, not the record's `o_angle` (`docs/ORDERS.md` §17.4).
///
/// `Group::compute_form`'s zero-delta arm has two halves
/// (`707e3d`–`707e6d`) and this crate had neither: with the location
/// different from the group's own `(ox, oy)` — which is every group that
/// has not already been sent to this exact point — the angle is
/// `leader.angle − ((signed char)angles[slot] << 24)`, and only a group
/// standing where it was last sent falls through to `o_angle`.
///
/// **No capture on disk reaches it**: every `QUEUE_NEW` group move in
/// the scored windows has a non-zero delta, so this is a built
/// scenario and the claim is the listing's.
///
/// **Made to fail on purpose** by dropping the first half — which is
/// what this crate did until item 471 — whereupon the angle is nought
/// on a group whose leader is plainly pointing somewhere.
#[test]
fn a_group_sent_to_its_own_feet_lays_out_on_the_leader_s_heading() {
    let mut s = sim();
    let t = fighter(&mut s);
    s.tables.unit_types[t].combat.x_spacing = 0xc0;
    s.tables.unit_types[t].combat.y_spacing = 0xc0;
    let here = Pos::new(0x8000, 0x4000);
    let a = spawn(&mut s, 1, t, here);
    let b = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4200));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    let leader = s.group_find_leader(&g).expect("a leader");
    let facing = Angle(0x2000_0000);
    s.units[leader].movement.heading = facing;
    s.group_action_move_to(
        &g,
        s.units[leader].pos,
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let m = s.current_move(leader).expect("a move order");
    // The slot byte is nought on a group that has not been laid out,
    // so the arm's whole answer is the heading.
    assert_eq!(
        m.angle, facing,
        "a zero delta takes the leader's own heading less its slot byte"
    );
    assert_ne!(
        m.angle,
        Angle(0),
        "and not the `o_angle` of a group that has never moved"
    );
}

/// **`UnitData::get_final_loc@00608040` climbs from the bottom of the
/// ring** and stops at the first order that names a point — the
/// deepest point on the list (`docs/ORDERS.md` §17.3, overturned by
/// item 2055, `docs/AI.md` §206) — and an order naming none is walked
/// past.
///
/// Three orders on one unit, current first: a think with nothing to
/// say, then an attack, then a move. The answer is the move's
/// destination, because it runs last; put the attack beneath it and the
/// answer is the attack's target.
///
/// ~~Made to fail with a `rev()` on the walk~~ until item 2055: the
/// walk without the `rev()` is the one that fails now, Great Sahara's
/// charge read from the charge's own point rather than the copy beneath.
#[test]
fn get_final_loc_climbs_from_the_bottom_of_the_ring() {
    let mut s = sim();
    let t = fighter(&mut s);
    let u = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4000));
    let prey = spawn(&mut s, 0, t, Pos::new(0x1000, 0x6000));
    assert_eq!(
        s.unit_final_loc(u),
        s.units[u].pos,
        "an empty list answers with the unit's own position"
    );
    let far = Pos::new(0x7000, 0x2000);
    s.add_attack_order(
        u,
        crate::combat::Obj::Unit(prey),
        QueuePos::New,
        true,
        false,
    );
    s.add_move_order(u, far, MoveKind::MoveTo, QueuePos::Last, false);
    // A `Think` in front, which names no point.
    s.units[u].orders.push_front(Order {
        flags: 0,
        body: Body::Think,
    });
    let prey_at = s.units[prey].pos;
    // The adder snaps its destination to the 48-unit cell centre
    // (§4.3), so the order's own `dest` is what `get_final_loc` hands
    // back and `far` is not.
    let snapped = s.units[u]
        .orders
        .iter()
        .find_map(Order::move_dest)
        .expect("the move went on");
    assert_eq!(
        s.unit_final_loc(u),
        snapped,
        "the move is beneath the attack, so its destination is the answer"
    );
    s.units[u].orders.swap(1, 2);
    assert_eq!(
        s.unit_final_loc(u),
        prey_at,
        "with the attack at the bottom, its target is"
    );
    // `flags & 1` alone, and nothing about the map: a dead target is
    // walked past like an order that names nothing.
    s.units[prey].health = 0;
    assert_eq!(
        s.unit_final_loc(u),
        snapped,
        "a dead target is not a point, so the walk climbs on to the move"
    );
}

/// **A guard names its escorted unit's position** (item 2055,
/// `docs/AI.md` §206): a `GuardOrder` is a `TargetOrder`, and
/// `get_final_loc` reads the target object's `+0x10/+0x14`, never the
/// post. run821's packet: Great Sahara's leader, a guard on the MLRS at
/// the bottom of its list, answers the MLRS's point.
#[test]
fn get_final_loc_reads_a_guard_s_target() {
    let mut s = sim();
    let t = fighter(&mut s);
    let u = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4000));
    let ward = spawn(&mut s, 1, t, Pos::new(0x1000, 0x6000));
    s.add_move_order(
        u,
        Pos::new(0x7000, 0x2000),
        MoveKind::MoveTo,
        QueuePos::New,
        false,
    );
    s.add_guard_order(u, ward, 0, 960, QueuePos::Last);
    assert!(
        matches!(
            s.units[u].orders.back().map(|o| o.body),
            Some(Body::Guard(_))
        ),
        "the guard is at the bottom"
    );
    // Moved after the order went on, so the post and the unit part.
    s.units[ward].pos = Pos::new(0x1200, 0x6400);
    assert_eq!(s.unit_final_loc(u), s.units[ward].pos);
}

/// **A follow names its leader's position** (item 2131, `docs/GOLDEN.md`
/// §78): `FollowOrder` is a `TargetOrder` too. run827: `0/6`'s squad,
/// following the Chariot `0/9`, is shift-moved on 660, and the queued
/// `GroupMoveOrder`'s angle is the bearing from the Chariot's point.
#[test]
fn get_final_loc_reads_a_follow_s_leader() {
    let mut s = sim();
    let t = fighter(&mut s);
    let u = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4000));
    let leader = spawn(&mut s, 1, t, Pos::new(0x1000, 0x6000));
    s.add_follow_order(u, leader, QueuePos::New);
    assert_eq!(s.unit_final_loc(u), s.units[leader].pos);
}

/// **The follower tracks the leader's current position, not the
/// destination** (§8.3) — and the leader tracks its own path. That
/// difference is the whole mechanic: one `do_move` a frame, not N.
#[test]
fn a_group_move_follower_walks_to_the_leader_s_position_plus_its_slot() {
    let mut s = sim();
    let t = fighter(&mut s);
    // A real `X_SPACING`, so `Form::compute` lays the two out side by
    // side rather than both on the origin — the shipped data never
    // leaves it at zero and a hand-built type would.
    s.tables.unit_types[t].combat.x_spacing = 0xc0;
    s.tables.unit_types[t].combat.y_spacing = 0xc0;
    // The second stands where the Line's slot for it will be — south
    // of the leader, the move going east — so it walks straight to it
    // rather than crossing behind.
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1200));
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    let leader = s.group_find_leader(&g).expect("a leader");
    let follower = if leader == a { b } else { a };
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x1000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    let i = g.list.iter().position(|&u| u == follower).expect("a slot");
    for _ in 0..4 {
        s.tick();
    }
    let want = s
        .group_slot_point(&g, leader, i)
        .expect("the leader's point plus my offset");
    let m = s.current_move(follower).expect("a move order");
    assert!(
        m.group.is_some_and(|x| x.in_group),
        "a follower on its slot is `in_group`"
    );
    assert_eq!(
        m.waypoint, want,
        "the waypoint is the leader's own position plus the rotated slot"
    );
    assert_ne!(
        m.waypoint, m.dest,
        "and it is not the destination the order carries"
    );
}

/// A squad `cap → mid → tail` whose back-pointers name a pushed slot,
/// joining an army already holding `a` and `b`: Great Lakes 11424's
/// shape (`docs/GROUPS.md` §23).
fn a_squad_joins_a_small_army() -> (Sim, usize, [usize; 5]) {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let cap = spawn(&mut s, 1, t, Pos::new(0x2000, 0x2000));
    let mid = spawn(&mut s, 1, t, Pos::new(0x2030, 0x2000));
    let tail = spawn(&mut s, 1, t, Pos::new(0x2060, 0x2000));
    for (f, up) in [(mid, cap), (tail, mid)] {
        s.units[f].captain = false;
        s.units[f].o_up = Some(up);
        s.units[up].o_down = Some(f);
    }
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    // The squad's `come_out` group: its own slot, which is what its
    // three pointers name when the army's `Group::add` walks them.
    let mut g = Group::stack(1);
    s.group_add(&mut g, cap);
    assert!(s.push_group(&mut g, true));
    s.army_add_unit(1, slot, tail);
    (s, slot, [a, b, cap, mid, tail])
}

/// **`Group::add` normalizes a small group at every step**, so the
/// squad's head and middle are dropped by the recursion that adds the
/// figure after them, and only the tail is left listed — while
/// `Unit::set_group` points all three at the army (`docs/GROUPS.md`
/// §23, `Group::get_num`'s `num < 4` arm). Made to fail on purpose by
/// skipping the per-step `get_num`: the list is then the whole squad.
#[test]
fn a_squad_joining_a_small_seated_group_is_listed_by_its_tail_alone() {
    let (s, slot, [a, b, cap, mid, tail]) = a_squad_joins_a_small_army();
    assert_eq!(s.armies[1].list[slot].units, vec![a, b, tail]);
    let pool = s.armies[1].list[slot].group.pool;
    for u in [cap, mid, tail] {
        assert_eq!(
            s.units[u].group_ptr, pool,
            "set_group points the squad at the army"
        );
    }
    assert_eq!(s.army_of(tail), Some(slot));
    assert_eq!(
        s.army_of(cap),
        None,
        "named but not listed: `Object::get_army` answers no army"
    );
}

/// **`Group::sort` kills a stray follower and re-adds its squad whole**,
/// at the end of the list and naming nothing: the kill clears the
/// pointer that names this group, and the `const` re-add writes none.
/// The tail is then listed with no group, and its first
/// `do_group_move` ungroups it (`docs/GROUPS.md` §23). Made to fail on
/// purpose by skipping the sort.
#[test]
fn the_sort_re_seats_a_stray_follower_s_squad_and_clears_its_pointer() {
    let (mut s, slot, [a, b, cap, mid, tail]) = a_squad_joins_a_small_army();
    let pool = s.armies[1].list[slot].group.pool;
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    assert_eq!(s.armies[1].list[slot].units, vec![a, b, cap, mid, tail]);
    assert_eq!(s.units[tail].group_ptr, None, "the kill cleared it");
    assert_eq!(s.units[cap].group_ptr, pool, "the re-add wrote nothing");
    assert_eq!(s.units[mid].group_ptr, pool);
    assert_eq!(s.army_of(tail), None);
    // And the army's own normalize then drops it, as the pool's
    // per-frame pass does on the slot's frame.
    s.army_normalize(1, slot);
    assert_eq!(s.armies[1].list[slot].units, vec![a, b, cap, mid]);
}

/// `ungroup_move_order` walks **up to the captain and back down every
/// subordinate**, so one member's ungroup is the squad's.
#[test]
fn an_ungroup_converts_the_whole_squad_s_orders_to_plain_moves() {
    let mut s = sim();
    let t = fighter(&mut s);
    let cap = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let sub = spawn(&mut s, 1, t, Pos::new(0x1030, 0x1000));
    let far = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    s.units[sub].captain = false;
    s.units[sub].o_up = Some(cap);
    s.units[cap].o_down = Some(sub);
    let slot = s.init_army(1, None);
    for u in [cap, sub, far] {
        s.army_add_unit(1, slot, u);
    }
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    let id = s
        .current_move(sub)
        .expect("a move")
        .group
        .expect("a group order")
        .id;
    s.ungroup_move_order(sub, id);
    for u in [cap, sub] {
        assert!(
            s.current_move(u).expect("a move").group.is_none(),
            "the captain and its subordinate both ungroup"
        );
    }
    assert!(
        s.current_move(far).expect("a move").group.is_some(),
        "and nobody outside the squad does"
    );
}

/// **The plain move an ungroup makes is put at the *head* of the
/// order list, not left where the group order stood** (item 487,
/// `docs/ORDERS.md` §20).
///
/// `Unit::ungroup_move_order@005fd140`'s `GROUP_MOVE` arm ends with
/// `remove_current` on the order list and then
/// `LinkListBase<UnitOrder *, …>::add@0046d5a0`, which **prepends** —
/// `head_node = new` — so the conversion is also a promotion. The two
/// readings are the same thing for a unit whose group move is already
/// the head order, which is every unit any capture on disk reached
/// until run100's `1/29`: an AI raider carrying an `ATTACK` above its
/// group move, on a farm that died ten frames earlier.
///
/// Made to fail on purpose by restoring the in-place rewrite, which
/// leaves the attack in front and the move buried under it.
#[test]
fn an_ungroup_puts_the_plain_move_at_the_head_of_the_list() {
    let mut s = sim();
    let t = fighter(&mut s);
    let cap = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let sub = spawn(&mut s, 1, t, Pos::new(0x1030, 0x1000));
    let prey = spawn(&mut s, 0, t, Pos::new(0x1400, 0x1000));
    s.units[sub].captain = false;
    s.units[sub].o_up = Some(cap);
    s.units[cap].o_down = Some(sub);
    let slot = s.init_army(1, None);
    for u in [cap, sub] {
        s.army_add_unit(1, slot, u);
    }
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    let id = s
        .current_move(sub)
        .expect("a move")
        .group
        .expect("a group order")
        .id;
    // The attack goes in front of the group move, which is the shape
    // run100's `1/29` is in on block 10240.
    s.add_attack_order(sub, Obj::Unit(prey), QueuePos::First, false, true);
    assert!(
        matches!(
            s.current_order(sub).map(|o| o.body),
            Some(crate::orders::Body::Attack(_))
        ),
        "the attack is the head order before the ungroup"
    );
    s.ungroup_move_order(sub, id);
    assert_eq!(
        s.units[sub]
            .orders
            .iter()
            .map(crate::orders::Order::index)
            .collect::<Vec<_>>(),
        vec![crate::orders::index::MOVE_TO, crate::orders::index::ATTACK],
        "the ungrouped move is the head order and the attack is under it"
    );
    assert!(
        s.current_move(sub).expect("a move").group.is_none(),
        "and it is a plain move"
    );
}

/// `do_group_move`'s window is a **third** of a turn, not the quarter
/// `reversing` uses (`5e8167`).
#[test]
fn the_group_move_window_is_a_third_of_a_turn() {
    use crate::orders::within_third;
    assert!(within_third(Angle(0), Angle(0)));
    assert!(
        within_third(Angle(0x5555_5554), Angle(0)),
        "a hair under 120°"
    );
    assert!(
        !within_third(Angle(0x5555_5555), Angle(0)),
        "exactly 120° is out"
    );
    // The fold is the original's `not`, not a negate, so the far side
    // is one unit wider: `!(-0x55555555) == 0x55555554`, which passes.
    assert!(
        within_third(Angle(-0x5555_5555), Angle(0)),
        "the `not` fold makes the other side inclusive"
    );
    assert!(
        !within_third(Angle(-0x5555_5556), Angle(0)),
        "and a unit past it is out"
    );
    assert!(!within_third(Angle(0), Angle(i32::MIN)), "dead astern");
}

/// **A formation command makes no order of its own** (`docs/GOLDEN.md`
/// §22, `docs/ORDERS.md` §30). A group that stands takes the byte and
/// re-forms on the spot — a move to the leader's own point, **without**
/// the action bit; a group that walks is halted and the leader's move
/// replayed to the same point, **with** it. Neither holds a
/// `CHANGE_FORM`.
/// `-1` and `-3` step `get_form_option` forward (modulo 5) and back
/// (wrapping below zero to 4); `-2` keeps the leader's own byte and
/// sets the group's `form` to `-2` (`form_view.rs`, swept against the
/// original). `finish_insert`'s fourth argument is dead (parked 3372f).
#[test]
fn action_form_steps_the_buttons_and_minus_two_keeps_the_leaders_byte() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
    let g = group_of(0, &[a, b]);
    for u in [a, b] {
        s.units[u].form = 1;
    }
    s.group_action_form(&g, -1, 0, QueuePos::New);
    assert_eq!(s.units[a].form, 2, "forward from the majority button");
    s.group_action_form(&g, -3, 0, QueuePos::New);
    assert_eq!(s.units[a].form, 1, "back from it");
    s.units[a].form = 0;
    s.units[b].form = 0;
    s.group_action_form(&g, -3, 0, QueuePos::New);
    assert_eq!(s.units[a].form, 4, "below zero wraps to 4");
    let leader = s.group_find_leader(&g).expect("a leader");
    let other = if leader == a { b } else { a };
    s.units[leader].form = 3;
    s.units[other].form = 1;
    s.group_action_form(&g, -2, 0, QueuePos::New);
    assert_eq!(s.units[a].form, 3);
    assert_eq!(s.units[b].form, 3, "every member takes the leader's byte");
}

#[test]
fn action_form_writes_the_byte_and_moves_the_group_it_does_not_order_a_form() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
    let g = group_of(0, &[a, b]);

    // Standing: the byte, and one move each at the leader's point.
    s.group_action_form(&g, 2, 0, QueuePos::New);
    let leader = s.group_find_leader(&g).expect("a leader");
    for u in [a, b] {
        assert_eq!(s.units[u].form, 2, "the byte is written");
        assert_eq!(s.units[u].orders.len(), 1, "one move, no FormOrder");
        let o = *s.current_order(u).expect("a move order");
        assert_ne!(o.index(), index::CHANGE_FORM);
        assert!(o.move_dest().is_some());
        assert!(
            !o.has(crate::orders::flag::ACTION),
            "no action bit on the spot"
        );
    }
    assert!(
        s.current_order(leader)
            .and_then(Order::move_dest)
            .is_some_and(
                |d| vector_dist(d.x - s.units[leader].pos.x, d.y - s.units[leader].pos.y) < 0x180
            ),
        "the group re-forms round where its leader stands"
    );

    // Walking: an action-bit move, then Line; the move is replayed.
    let to = Pos::new(0x4000, 0x1000);
    s.group_action_move_to(
        &g,
        to,
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    s.group_action_form(&g, 0, 0, QueuePos::New);
    for u in [a, b] {
        assert_eq!(s.units[u].form, 0);
        assert_eq!(s.units[u].orders.len(), 1, "halted, then one replay");
        let o = *s.current_order(u).expect("the replayed move");
        assert!(
            o.has(crate::orders::flag::ACTION),
            "the replay keeps the action bit"
        );
        let d = o.move_dest().expect("a move");
        assert!(
            vector_dist(d.x - to.x, d.y - to.y) < 0x400,
            "to the same point"
        );
    }
}

#[test]
fn action_move_to_gives_every_member_the_destination() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let g = group_of(1, &[a, b]);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    for u in [a, b] {
        let o = *s.current_order(u).expect("a move order");
        assert_eq!(o.index(), index::ATTACK_TO);
        assert!(o.has(crate::orders::flag::ACTION));
    }
}

#[test]
fn a_hurrying_army_stables_its_siege_and_marches_the_rest() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let siege = siege_type(&mut s);
    let city_pos = Pos::new(0x4000, 0x4000);
    let bldg = s.add_building(1, city_pos, 4);
    let c = s.cities.len();
    s.cities.push(crate::city::City {
        alive: true,
        owner: 1,
        race: Some(1),
        founder: 1,
        building: bldg,
        members: Vec::new(),
        reg: None,
        pos: city_pos,
        capital: true,
        founding_capital: true,
        was_founding_capital: false,
        unassimilated: false,
        no_heal: false,
        attacking: false,
        ever_attacked: false,
        alarm: false,
        no_muster: false,
        was_capital: 0,
        capture_stamp: 0,
        assimilation_timer: 0,
        attack_stamp: 0,
        reduce_stamp: 0,
        raid_stamp: 0,
        capture_strength: 0,
        pop: 1,
        has_citizen: false,
        source: None,
        trade_val: 0,
        traded_with: [0; 8],
    });
    // Object indices deliberately exceed the type table: stabling
    // must query the types, not index that table by an object id.
    for _ in 0..8 {
        spawn(&mut s, 0, foot, Pos::new(0x800, 0x800));
    }
    let bt = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Village,
        ..crate::build::BuildType::default()
    });
    s.buildings[bldg].ty = Some(bt);
    let slot = s.init_army(1, Some(c));
    let f = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
    s.army_add_unit(1, slot, f);
    s.army_add_unit(1, slot, m);

    // No hurry: both march.
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        city_pos,
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    // A land formation of two, so both members hold the grouped
    // spelling of the kind (item 237; `docs/ORDERS.md` §1.2).
    assert_eq!(s.order_type(m), index::GROUP_ATTACK_TO);

    // Hurrying, with a city at the destination: the siege is stabled
    // and the footman still marches.
    s.armies[1].list[slot].hurry = 1;
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        city_pos,
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(
        s.order_type(f),
        index::GROUP_ATTACK_TO,
        "the line still marches, and the group is still two"
    );
    assert!(
        !matches!(s.order_type(m), index::ATTACK_TO | index::GROUP_ATTACK_TO),
        "the siege is sent into the city instead, not at it: {}",
        s.order_type(m)
    );
    s.tables.unit_types[siege].garrison.fortify = true;
    s.stable_in_city(m, c, Angle(0));
    assert_eq!(
        s.order_type(m),
        index::GARRISON,
        "the same objects take a garrison order when their types permit it"
    );
}

#[test]
fn a_shooting_siege_unit_is_left_alone_by_a_move_that_is_not_hurrying() {
    let mut s = sim();
    let siege = siege_type(&mut s);
    let slot = s.init_army(1, None);
    let m = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
    let foe = spawn(&mut s, 0, siege, Pos::new(0x1080, 0x1000));
    s.army_add_unit(1, slot, m);
    s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
    assert_eq!(s.order_type(m), index::ATTACK);
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x8000, 0x8000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(
        s.order_type(m),
        index::ATTACK,
        "a siege unit already shooting keeps its order"
    );
}

/// `Group::action_follow@006fd510` (`docs/ORDERS.md` §28): **every**
/// orderable member gets one `FOLLOW` on the leader with the action
/// bit — run204's squad, three orders on `0/11` on block 642 — and a
/// member that is the leader's own captain of the same player gets
/// none.
#[test]
fn a_follow_is_one_follow_order_a_member_and_none_on_the_leader_itself() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(11640, 10872));
    let b = spawn(&mut s, 0, t, Pos::new(11784, 10872));
    let c = spawn(&mut s, 0, t, Pos::new(11688, 11016));
    let lead = spawn(&mut s, 0, t, Pos::new(11640, 11640));
    let mut g = group_of(0, &[a, b, c, lead]);
    assert!(s.push_group(&mut g, true));
    s.group_action_follow(&g, lead, QueuePos::New);
    for u in [a, b, c] {
        let [o] = s.units[u].orders.iter().copied().collect::<Vec<_>>()[..] else {
            panic!("one order on {u}: {:?}", s.units[u].orders);
        };
        assert_eq!(
            o.body,
            Body::Follow(crate::orders::FollowOrder { target: lead })
        );
        assert!(o.has(flag::ACTION));
    }
    assert!(s.units[lead].orders.is_empty(), "the leader follows no one");
}

/// `Unit::do_follow@005e65d0`'s standoff, on run204's squad: a hoplite
/// (`los` 6, slower than its leader) 768 from a standing chariot is
/// inside `s + 0xc0` = 1,575 and stands. Once the chariot walks, `k`
/// doubles, the threshold is 653, and the hoplite takes a `MOVE_TO` leg
/// without the action bit to the point 461 short of the leader — block
/// 722's (11640, 11160).
#[test]
fn a_follower_stands_inside_its_standoff_and_walks_when_the_leader_does() {
    let mut s = sim();
    let t = fighter(&mut s);
    s.tables.unit_types[t].los = 6;
    let lead = spawn(&mut s, 0, t, Pos::new(11640, 11640));
    let f = spawn(&mut s, 0, t, Pos::new(11640, 10872));
    s.units[lead].movement.speed = 29;
    s.units[f].movement.speed = 23;
    s.add_follow_order(f, lead, QueuePos::New);
    let o = i64::from(s.units[f].index);
    s.work(f, 1 - o);
    assert_eq!(s.units[f].orders.len(), 1, "no leg within the standoff");
    s.add_move_order(
        lead,
        Pos::new(11640, 20000),
        MoveKind::MoveTo,
        QueuePos::New,
        true,
    );
    s.work(f, 2 - o);
    let head = s.units[f].orders[0];
    let Body::Move(m) = head.body else {
        panic!("no leg: {:?}", s.units[f].orders);
    };
    assert_eq!((m.kind, m.dest), (MoveKind::MoveTo, Pos::new(11640, 11160)));
    assert!(!head.has(flag::ACTION));
    assert!(matches!(s.units[f].orders[1].body, Body::Follow(_)));
}

/// A pushed squad of three and its patrol, as `process_group` and
/// `process_patrol` build it (`docs/ORDERS.md` §27).
fn patrolling_squad() -> (Sim, [usize; 3], Pos) {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(10872, 7800));
    let b = spawn(&mut s, 0, t, Pos::new(11016, 7800));
    let c = spawn(&mut s, 0, t, Pos::new(10920, 7944));
    let mut g = group_of(0, &[a, b, c]);
    assert!(s.push_group(&mut g, true));
    let to = Pos::new(11136, 11904);
    s.group_action_patrol(&g, to, QueuePos::New);
    (s, [a, b, c], to)
}

fn patrol(s: &Sim, u: usize) -> Vec<crate::orders::PatrolOrder> {
    s.units[u]
        .orders
        .iter()
        .filter_map(|o| match o.body {
            Body::Patrol(p) => Some(p),
            _ => None,
        })
        .collect()
}

/// `Group::action_patrol@007030c0`: **every** member gets one
/// `GroupPatrolOrder` — no plain `PatrolOrder` exists — from the
/// group's place to the click, both on the 48-unit grid, with one id,
/// one leader and its own index; run184's block 642 has the points.
#[test]
fn a_patrol_is_one_group_patrol_order_a_member_from_the_group_s_place() {
    let (s, [a, b, c], _) = patrolling_squad();
    for (i, u) in [a, b, c].into_iter().enumerate() {
        let p = patrol(&s, u);
        assert_eq!(p.len(), 1, "one patrol on member {i}");
        let p = p[0];
        assert_eq!(s.order_type(u), index::GROUP_PATROL);
        assert_eq!(
            p.points,
            [Pos::new(10872, 7800), Pos::new(11160, 11928)],
            "the leader's place and the click, snapped"
        );
        assert_eq!((p.waypoint, p.leader, p.form_id), (0, a, i));
        assert_eq!(p.id, patrol(&s, a)[0].id);
    }
}

/// **A Helicopter in a patrolled group is ordered like a ground unit**
/// (item 3967, `docs/GOLDEN.md` §173): `Group::action_patrol@007030c0`
/// hands off to the air patrol on a plane leader or `count(COUNT_DOMAIN,
/// 4)` — vslot `+0xc0`, `is_plane` — and a Helicopter is no plane, so a
/// squad with one and a Helicopter alone each take the land arm, every
/// member a `GroupPatrolOrder`. run1538's `0/6`..`0/9` on 622 and `0/10`
/// on 702.
#[test]
fn a_helicopter_in_a_patrolled_group_takes_the_land_arm() {
    let mut s = sim();
    let t = fighter(&mut s);
    let heli = flier(&mut s, true);
    let h = spawn(&mut s, 0, heli, Pos::new(3192, 7032));
    let a = spawn(&mut s, 0, t, Pos::new(10872, 7800));
    let b = spawn(&mut s, 0, t, Pos::new(11016, 7800));
    let mut g = group_of(0, &[h, a, b]);
    assert!(s.push_group(&mut g, true));
    s.group_action_patrol(&g, Pos::new(3456, 11136), QueuePos::New);
    for u in [h, a, b] {
        assert_eq!(patrol(&s, u).len(), 1, "a patrol on {u}");
        assert_eq!(s.order_type(u), index::GROUP_PATROL);
    }
    let lone = spawn(&mut s, 0, heli, Pos::new(5880, 5880));
    let mut g = group_of(0, &[lone]);
    assert!(s.push_group(&mut g, true));
    s.group_action_patrol(&g, Pos::new(11136, 11904), QueuePos::New);
    assert_eq!(patrol(&s, lone).len(), 1, "the lone Helicopter patrols");
}

/// **A cast that is no pack or unpack is not busy** (item 3988,
/// `docs/GOLDEN.md` §181): `UnitData::is_busy@0060a370` asks the head
/// cast's spell type `is_pack`/`is_unpack` (vslots `+0x50`/`+0x54`), so
/// a lone member mid-Bribe takes the patrol over its cast and one
/// mid-Pack is passed over. run1546's Spy `0/6` on 1115.
#[test]
fn a_lone_member_mid_bribe_takes_the_patrol_and_one_mid_pack_does_not() {
    use crate::orders::spell;
    let mut s = sim();
    let t = fighter(&mut s);
    for (sp, patrols) in [(spell::BRIBE, true), (spell::PACK, false)] {
        let u = spawn(&mut s, 0, t, Pos::new(10872, 7800));
        s.add_cast_order_on(u, sp, None, Pos::new(-1, -1), QueuePos::New, true);
        assert_eq!(s.is_busy(u), !patrols, "spell {sp:#x}");
        let mut g = group_of(0, &[u]);
        assert!(s.push_group(&mut g, true));
        s.group_action_patrol(&g, Pos::new(11136, 11904), QueuePos::New);
        assert_eq!(patrol(&s, u).len(), usize::from(patrols), "spell {sp:#x}");
    }
}

/// `Unit::do_patrol@005f1910` on the leader steps the waypoint and
/// hands the group an attack-move `QUEUE_FIRST`; the group's
/// `QUEUE_FIRST` halts the members and `redo_patrol_order@00706d90`
/// rebuilds each patrol behind the leg with the **leader's** step and
/// `form_id` — run184's squad carries `form_id 0` and `waypoint 1` on
/// all three on 642. A follower's `do_patrol` issues nothing.
#[test]
fn the_leader_s_leg_rebuilds_every_patrol_with_its_step() {
    let (mut s, [a, b, c], _) = patrolling_squad();
    let Some(Body::Patrol(p)) = s.current_order(b).map(|o| o.body) else {
        panic!("the follower's head is its patrol");
    };
    s.do_patrol(b, p);
    assert_eq!(
        s.order_type(b),
        index::GROUP_PATROL,
        "a follower only idles"
    );
    let Some(Body::Patrol(p)) = s.current_order(a).map(|o| o.body) else {
        panic!("the leader's head is its patrol");
    };
    s.do_patrol(a, p);
    for u in [a, b, c] {
        assert_eq!(
            s.order_type(u),
            index::GROUP_ATTACK_TO,
            "the leg is at the head"
        );
        let ps = patrol(&s, u);
        assert_eq!(ps.len(), 1, "the patrol is rebuilt once, behind the leg");
        assert_eq!((ps[0].waypoint, ps[0].form_id), (1, 0));
        assert!(s.units[u].orders[1].has(flag::ACTION));
    }
    let Some(Body::Move(m)) = s.current_order(a).map(|o| o.body) else {
        panic!("a move");
    };
    assert_eq!(m.kind, MoveKind::AttackTo);
    assert!(
        !s.current_order(a).unwrap().has(flag::ACTION),
        "the leg carries no action bit: do_patrol passes action 0"
    );
}

#[test]
fn action_halt_clears_the_orders_and_the_mask_exempts_siege() {
    let mut s = sim();
    let siege = siege_type(&mut s);
    let foot = fighter(&mut s);
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[a, m]);
    for u in [a, m] {
        s.add_move_order(
            u,
            Pos::new(0x4000, 0x4000),
            MoveKind::MoveTo,
            QueuePos::New,
            true,
        );
    }
    s.group_action_halt(&g, 4);
    assert!(s.current_order(a).is_none(), "the footman halts");
    assert!(
        s.current_order(m).is_some(),
        "the mask's 4 bit exempts a siege unit"
    );
    s.group_action_halt(&g, 0);
    assert!(s.current_order(m).is_none());
}

/// **`action_halt` skips a member that is dead, off the map or a plane,
/// and the mask's 2 bit exempts a special unit** (item 3379,
/// `0070d0c0`, swept in `sweep::group_action_halt`; the sweep holds the
/// steps, this holds the view over this simulation).
#[test]
fn action_halt_skips_the_dead_the_absent_and_planes_and_the_mask_exempts_specials() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let mut plane = UnitType {
        hits: 100,
        ..UnitType::default()
    };
    plane.combat.domain = Domain::Air;
    let plane = s.add_unit_type(plane);
    let mut scout = UnitType {
        hits: 100,
        ..UnitType::default()
    };
    scout.cols.unit_flags2 |= uflags2::FLAG2_SCOUT;
    let scout = s.add_unit_type(scout);
    let ground = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let dead = spawn(&mut s, 1, foot, Pos::new(0x1100, 0x1000));
    let away = spawn(&mut s, 1, foot, Pos::new(0x1200, 0x1000));
    let flier = spawn(&mut s, 1, plane, Pos::new(0x1300, 0x1000));
    let special = spawn(&mut s, 1, scout, Pos::new(0x1400, 0x1000));
    let all = [ground, dead, away, flier, special];
    for u in all {
        s.add_move_order(
            u,
            Pos::new(0x4000, 0x4000),
            MoveKind::MoveTo,
            QueuePos::New,
            true,
        );
    }
    s.units[dead].health = 0;
    s.units[away].on_map = false;
    let g = group_of(1, &all);
    s.group_action_halt(&g, 2);
    assert!(s.current_order(ground).is_none(), "the footman halts");
    assert!(s.current_order(dead).is_some(), "a dead member is skipped");
    assert!(
        s.current_order(away).is_some(),
        "an absent member is skipped"
    );
    assert!(s.current_order(flier).is_some(), "a plane is skipped");
    assert!(
        s.current_order(special).is_some(),
        "the mask's 2 bit exempts a special unit"
    );
    s.group_action_halt(&g, 0);
    assert!(s.current_order(special).is_none(), "mask 0 halts it");
    assert!(s.current_order(dead).is_some() && s.current_order(away).is_some());
    assert!(s.current_order(flier).is_some());
}

/// **A human's 1, 2 or 5 kills an order it was not given** (item 2165,
/// `docs/GOLDEN.md` §79; `Group::action_stance@0070d440`'s middle arm):
/// a current order without the action bit goes, one with it stays, and a
/// computer's stays whatever it carries.
#[test]
fn a_human_s_stand_ground_kills_an_order_without_the_action_bit() {
    let mut s = sim();
    s.nation[0].human = true;
    s.nation[1].human = false;
    let t = fighter(&mut s);
    let found = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
    let given = spawn(&mut s, 0, t, Pos::new(0x1100, 0x1000));
    let ai = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    for (u, action) in [(found, false), (given, true), (ai, false)] {
        s.add_move_order(
            u,
            Pos::new(0x4000, 0x4000),
            MoveKind::MoveTo,
            QueuePos::New,
            action,
        );
    }
    s.group_action_stance(
        &group_of(0, &[found, given]),
        stance_option(Stance::StandGround),
    );
    s.group_action_stance(&group_of(1, &[ai]), stance_option(Stance::StandGround));
    assert!(s.current_order(found).is_none(), "a found order goes");
    assert!(s.current_order(given).is_some(), "a given one stays");
    assert!(s.current_order(ai).is_some(), "a computer's stays");
    assert_eq!(s.units[found].combat.stance, Stance::StandGround);
}

#[test]
fn action_stance_writes_the_combat_stance_and_cycles_on_a_negative() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[a, b]);
    assert_eq!(s.group_stance_type(&g), StanceType::Combat);
    s.units[a].combat.mandatory = true;
    s.group_action_stance(&g, stance_option(Stance::Raid));
    assert_eq!(s.units[a].combat.stance, Stance::Raid);
    assert_eq!(s.units[b].combat.stance, Stance::Raid);
    assert!(!s.units[a].combat.mandatory, "3 clears mandatory");
    // −1 steps forward from the leader's current option, 3 → 4.
    s.group_action_stance(&g, -1);
    assert_eq!(s.units[a].combat.stance, Stance::Raze);
    // −2 steps back, 4 → 3.
    s.group_action_stance(&g, -2);
    assert_eq!(s.units[a].combat.stance, Stance::Raid);
    // And it wraps at the six combat options.
    s.group_action_stance(&g, stance_option(Stance::HoldFire));
    s.group_action_stance(&g, -1);
    assert_eq!(s.units[a].combat.stance, Stance::Aggressive);
}

/// **A stance press is a think on the next frame** (item 3962,
/// `docs/AI.md` §406): `action_stance` raises `flags & 0x10` on every
/// member, so an idle member off its one-in-sixteen phase passes
/// `Unit::think`'s gate, and the think's exit drops the bit again.
#[test]
fn a_stance_press_lets_an_idle_member_past_the_think_gate() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[a, b]);
    for u in [a, b] {
        s.units[u].idle = 6;
    }
    s.group_action_stance(&g, stance_option(Stance::Defensive));
    assert!(
        s.units[a].cant_reach && s.units[b].cant_reach,
        "every member takes the bit"
    );
    // Off phase: `(frame + o) % 16 != 0`.
    let frame = 17 - i64::from(s.units[a].index);
    s.think(a, frame);
    assert!(!s.units[a].cant_reach, "the think ran, and dropped it");
    // Without the press the gate holds, and the bit stays as it was.
    s.units[b].cant_reach = false;
    let frame = 17 - i64::from(s.units[b].index);
    s.think(b, frame);
    assert!(!s.units[b].cant_reach);
}

/// **A defeated seat's unit leaves `think` at `:98` and keeps the bit**
/// (item 3983, `docs/AI.md` §415): `Leader::defeat` sets
/// `leader_flags2 & 2`, the first test past the gate.
#[test]
fn a_defeated_seat_s_unit_keeps_the_bit_through_think() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    s.units[a].idle = 1;
    s.units[a].cant_reach = true;
    s.defeated[1] = true;
    s.think(a, 17);
    assert!(
        s.units[a].cant_reach,
        "the defeated seat's think returns at :98"
    );
    s.defeated[1] = false;
    s.think(a, 18);
    assert!(!s.units[a].cant_reach, "a live seat's think drops it");
}

#[test]
fn siege_attack_to_is_an_attack_move_when_the_group_has_no_siege() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[a, b]);
    s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
    for u in [a, b] {
        assert_eq!(
            s.order_type(u),
            index::ATTACK_TO,
            "no siege, no wagon, no hero: the whole group attack-moves"
        );
    }
}

#[test]
fn siege_attack_to_sends_the_siege_and_leaves_the_escort_behind() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let siege = siege_type(&mut s);
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[a, m]);
    s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
    assert_eq!(s.order_type(m), index::ATTACK_TO, "the siege advances");
    assert_eq!(s.order_type(a), index::GUARD, "and the escort guards it");
    assert!(
        matches!(s.units[a].orders[0].body, Body::Guard(g) if g.target == m),
        "on the anchor: {:?}",
        s.units[a].orders
    );
}

fn wagon_type(sim: &mut Sim) -> usize {
    sim.add_unit_type(UnitType {
        hits: 100,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::SUPPLY_OR_HERO,
            ..crate::ai_load::UnitCols::default()
        },
        ..UnitType::default()
    })
}

/// Golden chapter four's 1277 in miniature (`docs/ORDERS.md` §24): an
/// AI army with a Supply Wagon and no siege sends the wagon on, and
/// every other member takes a `GUARD` on it, at the head of its list,
/// with its slot's offset from `compute_form` laid out at the wagon.
#[test]
fn a_siegeless_army_s_wagon_is_the_anchor_and_the_rest_guard_it() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, foot, Pos::new(0x1030, 0x1000));
    let c = spawn(&mut s, 1, foot, Pos::new(0x1060, 0x1000));
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1400));
    let g = group_of(1, &[a, b, c, w]);
    s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
    assert_eq!(s.order_type(w), index::ATTACK_TO, "the wagon walks in");
    let mut dy = None;
    for u in [a, b, c] {
        let Some(Body::Guard(o)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("{u} does not guard: {:?}", s.units[u].orders);
        };
        assert_eq!(o.target, w);
        assert_eq!(o.guard, s.units[w].pos, "the post starts at the target");
        assert_eq!((o.idle, o.retry), (0, 0));
        assert!(s.units[u].orders[0].has(flag::ACTION));
        // One rank, behind the phantom artillery rank reserved for
        // the wagon: every figure shares the rank's depth.
        assert_eq!(*dy.get_or_insert(o.dy), o.dy);
    }
    // A second call rewrites the offsets in place rather than
    // stacking a second guard (`update_guard_order`).
    s.group_action_guard(&g, w, QueuePos::New, true);
    assert_eq!(s.units[a].orders.len(), 1);
}

/// **A hero carrying the supply bit is the supply arm's anchor** (item
/// 3682, `docs/AI.md` §346.2): `UnitData::is_supply@0046ce80` is the
/// raw `unit_flags2 & 0x40`, so The Citizen (TY 356, both `0x40` and
/// `0x20`) at the list's head is taken before a Supply Wagon behind it
/// — run1334's merged army 1/0 kept guarding its Citizen `1/17` when
/// group 67's wagons joined on 4348.
#[test]
fn a_hero_with_the_supply_bit_is_the_anchor_before_a_later_wagon() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let citizen = s.add_unit_type(UnitType {
        hits: 100,
        cols: crate::ai_load::UnitCols {
            unit_flags2: uflags2::SUPPLY_OR_HERO | uflags2::FLAG2_GENERAL,
            ..crate::ai_load::UnitCols::default()
        },
        ..UnitType::default()
    });
    let h = spawn(&mut s, 1, citizen, Pos::new(0x1400, 0x1000));
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1400));
    let g = group_of(1, &[h, a, w]);
    s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
    assert_eq!(s.order_type(h), index::ATTACK_TO, "the Citizen walks in");
    for u in [a, w] {
        let Some(Body::Guard(o)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("{u} does not guard: {:?}", s.units[u].orders);
        };
        assert_eq!(o.target, h, "{u} guards the Citizen");
    }
}

/// **The anchor's sub-group lays out on its own record** (§26, item
/// 736): `action_siege_attack_to@0070d830` builds it on the stack with
/// `Group::clear(-1)`, so its `facing` is 0 whatever the parent's is,
/// and what the layout writes is discarded. Great Lakes' Despot took
/// army 3's `facing 1` onto its `ATTACK_TO` on 15351 where the original
/// carries 0, and that mirror came back into `unit_masks & 2` when the
/// move died.
#[test]
fn the_anchor_s_sub_group_lays_out_on_its_own_cleared_facing() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let slot = s.init_army(1, None);
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1000));
    for u in [a, w] {
        s.army_add_unit(1, slot, u);
        s.units[u].movement.set_facing(Angle::NORTH);
    }
    s.armies[1].list[slot].group.facing = true;
    let before = s.armies[1].list[slot].group.clone();
    let g = s.army_group(1, slot);
    s.group_action_siege_attack_to(&g, Pos::new(0x1400, 0x4000), Angle::NORTH);
    assert_eq!(s.order_type(w), index::ATTACK_TO, "the wagon walks in");
    let facing = s.units[w].orders.iter().find_map(|o| match o.body {
        Body::Move(m) => Some(m.facing),
        _ => None,
    });
    assert_eq!(
        facing,
        Some(Some(false)),
        "the stack record's 0, not the army's 1: {:?}",
        s.units[w].orders
    );
    let after = &s.armies[1].list[slot].group;
    assert!(after.facing, "the army's own flag is left alone");
    assert_eq!(
        (after.o, after.o_angle, after.order_num),
        (before.o, before.o_angle, before.order_num),
        "and the sub-group's layout is not written onto the army's record"
    );
}

/// **A member's dying move hands its mirror to the army, not to the
/// stack** (`docs/AI.md` §212, item 2155). The sub-group's layout
/// clears each member's orders first, and `kill_current_order`'s
/// hand-back goes through the unit's `+0x80`, the real group. run820's
/// Supply Wagon `1/205`, its army's leader and its only sub-group
/// member, re-took its march on 4082 facing south of the march's
/// bearing: the dying move handed back 1, and the layout read it off
/// the stack record, so the new `ATTACK_TO` carried 0 where the
/// original's carries 1 — and on 4338 its hand-back cleared the
/// wagon's `unit_masks & 2` there and left it here, which turned the
/// guarding helicopters' posts. Made to fail on purpose by writing the
/// hand-back to the seat's record: `Some(Some(false))`.
#[test]
fn a_member_s_dying_move_hands_its_mirror_to_the_army_not_the_stack() {
    let mut s = sim();
    let wagon = wagon_type(&mut s);
    let slot = s.init_army(1, None);
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1000));
    s.army_add_unit(1, slot, w);
    s.units[w].movement.set_facing(Angle::SOUTH);
    let to = Pos::new(0x1400, 0x4000);
    s.add_move_facing_order(
        w,
        to,
        MoveKind::AttackTo,
        QueuePos::New,
        true,
        Angle::NORTH,
        Some(false),
        false,
    );
    assert!(!s.armies[1].list[slot].group.facing);
    let g = s.army_group(1, slot);
    s.group_action_siege_attack_to(&g, to, Angle::NORTH);
    let facing = s.units[w].orders.iter().find_map(|o| match o.body {
        Body::Move(m) => Some(m.facing),
        _ => None,
    });
    assert_eq!(
        facing,
        Some(Some(true)),
        "the stack's 0 against a leader facing away: {:?}",
        s.units[w].orders
    );
    assert!(
        s.armies[1].list[slot].group.facing,
        "the army's record keeps the dying move's hand-back"
    );
}

/// **A siege unit's slot is copied back into the army's table at its
/// own index** (`action_siege_attack_to@0070d830`, the loop after the
/// sub-group's move; item 1493, `docs/AI.md` §128). The sub-group lays
/// out on its own record; the parent's `off`, `curr` and `angles` at
/// each siege member's *parent* index take that slot, and
/// `Unit::replace_form_id@005fd420` re-points the member's group order
/// at the parent index. Great Sahara at Toughest: `1/141`'s `off` (23,
/// 0) at index 62, past the army's `form_num` 27, which
/// `refresh_group_order` reads. Made to fail on purpose by removing the
/// copy: the tail stays zero and the second bombard's order names its
/// index in the sub-group, 1.
#[test]
fn a_siege_unit_s_slot_is_copied_into_the_army_s_table_at_its_own_index() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let siege = siege_type(&mut s);
    s.tables.unit_types[siege].combat.x_spacing = 0x120;
    s.tables.unit_types[siege].combat.y_spacing = 0x120;
    let slot = s.init_army(1, None);
    let mut all = Vec::new();
    for (i, ty) in [foot, foot, foot, siege, siege].into_iter().enumerate() {
        let u = spawn(&mut s, 1, ty, Pos::new(0x1000 + 0x60 * i as i32, 0x1000));
        s.army_add_unit(1, slot, u);
        all.push(u);
    }
    let g = s.army_group(1, slot);
    s.group_action_siege_attack_to(&g, Pos::new(0x1400, 0x4000), Angle::NORTH);
    let (m1, m2) = (all[3], all[4]);
    let form_id = |u: usize| s.current_move(u).and_then(|m| m.group).map(|gm| gm.form_id);
    assert_eq!(form_id(m1), Some(3), "the leader's index in the army");
    assert_eq!(form_id(m2), Some(4), "and the follower's, not 1");
    let st = &s.armies[1].list[slot].group;
    assert_eq!(
        st.off[3],
        (0, 0),
        "the sub-group's leader sits at its origin"
    );
    assert_ne!(st.off[4], (0, 0), "the second slot: {:?}", st.off);
    assert_ne!(st.curr[4], Pos::new(0, 0), "and its rotated twin");
    // And `update_positions` walks `form_num` slots, so a frame of the
    // leader's rotation leaves the tail's `curr` as the copy made it.
    let before = st.curr[4];
    s.group_update_positions(&g, m1);
    assert_eq!(s.armies[1].list[slot].group.curr[4], before);
}

/// **The anchor's sub-group sorts its own list, not the army's**
/// (`docs/GROUPS.md` §27, East Indies 17402). The army holds a wagon, a
/// fighter and a squad's tail without its captain — the list
/// `Group::sort` re-seats. The siege arm's sub-group is the wagon
/// alone, so its sort is a no-op: the wagon takes a **plain**
/// `ATTACK_TO`, and the army's list is left as it was. Made to fail on
/// purpose by sorting the seat: the army's list comes back re-seated
/// and the wagon walks under a `GROUP_ATTACK_TO` of five.
#[test]
fn the_anchor_s_sub_group_sorts_its_own_list_and_walks_alone() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, foot, Pos::new(0x1100, 0x1000));
    let cap = spawn(&mut s, 1, foot, Pos::new(0x2000, 0x2000));
    let mid = spawn(&mut s, 1, foot, Pos::new(0x2030, 0x2000));
    let tail = spawn(&mut s, 1, foot, Pos::new(0x2060, 0x2000));
    for (f, up) in [(mid, cap), (tail, mid)] {
        s.units[f].captain = false;
        s.units[f].o_up = Some(up);
        s.units[up].o_down = Some(f);
    }
    let slot = s.init_army(1, None);
    s.army_add_unit(1, slot, w);
    s.army_add_unit(1, slot, b);
    let mut sq = Group::stack(1);
    s.group_add(&mut sq, cap);
    assert!(s.push_group(&mut sq, true));
    s.army_add_unit(1, slot, tail);
    assert_eq!(
        s.armies[1].list[slot].units,
        vec![w, b, tail],
        "the stray tail"
    );
    let g = s.army_group(1, slot);
    s.group_action_siege_attack_to(&g, Pos::new(0x1400, 0x4000), Angle::NORTH);
    let m = s.current_move(w).expect("the wagon walks in");
    assert_eq!(m.kind, MoveKind::AttackTo);
    assert!(
        m.group.is_none(),
        "a sub-group of one lays out a plain move: {:?}",
        m.group
    );
    assert_eq!(
        s.armies[1].list[slot].units,
        vec![w, b, tail],
        "and the army's list is not the one sorted"
    );
}

/// `Unit::do_attack_to_pause@005f22a0` (`docs/ORDERS.md` §24.9),
/// golden chapter four's 1415 in miniature: a Supply Wagon on an
/// attack-move, seated in a group, looks on its own fifteen-frame phase
/// at the group's armed captains within `0x600` of it, and waits
/// fifteen frames when no fewer than half of them are fighting — which
/// **one** near captain that is only walking satisfies, `1 / 2` being
/// 0. Two near and neither fighting do not. Nor does a captain out of
/// reach.
#[test]
fn an_unarmed_attack_mover_waits_on_its_phase_for_a_captain_at_its_heels() {
    let case = |near: &[i32]| -> i32 {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let at = Pos::new(0x2000, 0x2000);
        let w = spawn(&mut s, 1, wagon, at);
        let mut list = vec![w];
        for &dx in near {
            let a = spawn(&mut s, 1, foot, Pos::new(at.x + dx, at.y));
            s.add_move_order(
                a,
                Pos::new(0x6000, 0x2000),
                MoveKind::AttackTo,
                QueuePos::New,
                true,
            );
            list.push(a);
        }
        let mut g = group_of(1, &list);
        assert!(s.push_group(&mut g, true));
        s.add_move_order(
            w,
            Pos::new(0x6000, 0x2000),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        let dest = s.current_move(w).expect("the wagon's attack-move").dest;
        let o = i64::from(s.units[w].index);
        // Off the phase first: nothing looks.
        s.do_attack_to_tail(w, 16 - o, dest);
        assert_eq!(s.current_move(w).unwrap().pause, 0, "off the phase");
        s.do_attack_to_tail(w, 15 - o, dest);
        s.current_move(w).unwrap().pause
    };
    assert_eq!(case(&[0x300]), 15, "one walking captain near: 0 >= 1 / 2");
    assert_eq!(case(&[0x300, -0x300]), 0, "two near, none fighting");
    assert_eq!(case(&[0x700]), 0, "out of reach: 0x700 > 0x600");
}

/// **`pUVar1 == param_1`** (`Unit::do_attack_to@005f2320`, `docs/AI.md`
/// §135): the look runs only while the order it was dispatched for
/// still heads the list. A guard's leg whose `timer` runs out on the
/// look's phase is killed (`do_move@005f7b30:184`), and `work` has the
/// guard lay a **new** leg to the same post. That leg is not the order
/// `do_attack_to` was called for, so the unarmed wagon does not pause
/// on it — French East Indies' President `1/67` on 17243, whose post
/// had not moved. The control, a leg with time left, pauses fifteen.
#[test]
fn a_guard_s_leg_that_timed_out_is_a_new_order_and_takes_no_pause() {
    let pause = |timed_out: bool| -> i32 {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let w = spawn(&mut s, 1, wagon, Pos::new(8211, 32341));
        s.units[w].movement.heading = Angle(535_429_120);
        s.add_move_order(
            w,
            Pos::new(20000, 20000),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        let at = Pos::new(13438, 26412);
        let g = spawn(&mut s, 1, wagon, at);
        let a = spawn(&mut s, 1, foot, Pos::new(at.x + 0x300, at.y));
        s.add_move_order(
            a,
            Pos::new(0x6000, 0x2000),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        let mut grp = group_of(1, &[g, a]);
        assert!(s.push_group(&mut grp, true));
        s.add_guard_order(g, w, 0, 264, QueuePos::New);
        let o = i64::from(s.units[g].index);
        s.work(g, 1 - o);
        let leg = s.current_move(g).expect("the guard's leg");
        assert_eq!((leg.kind, leg.pause), (MoveKind::AttackTo, 0));
        if timed_out {
            s.units[g].orders[0].move_mut().unwrap().timer = 1;
        }
        // The look's phase.
        s.work(g, 15 - o);
        let m = s.current_move(g).expect("a leg again");
        assert_eq!((m.kind, m.dest), (MoveKind::AttackTo, leg.dest));
        m.pause
    };
    assert_eq!(pause(false), 15, "the same leg: the wagon waits");
    assert_eq!(pause(true), 0, "a new leg to the same post: no look");
}

/// `do_guard`'s reposition: a guard off its post beside a moving
/// target is given an `ATTACK_TO` leg to the post, at the head,
/// without the action bit, stepped the same frame — so its `timer`,
/// thirty frames a world cell of Manhattan distance, reads one less.
///
/// **Every number is run133's** (`docs/ORDERS.md` §24): the three
/// hoplites' and the wagon's block-1276 positions and the wagon's
/// heading, the offsets `action_guard` gave them, and the posts,
/// angle and timers the original printed on block 1277. It is the
/// arithmetic of the post — `sinx`/`cosx` of the target's heading on
/// `dy` and `dx`, the 48-unit snap, the world-cell timer — checked
/// against the original's own answer on the original's own input.
#[test]
fn a_guard_off_its_post_walks_to_it_on_a_timed_leg_the_same_frame() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let w = spawn(&mut s, 1, wagon, Pos::new(8211, 32341));
    s.units[w].movement.heading = Angle(535_429_120);
    s.add_move_order(
        w,
        Pos::new(20000, 20000),
        MoveKind::AttackTo,
        QueuePos::New,
        true,
    );
    let cases = [
        // (block-1276 position, dx, the post, the timer on 1277)
        ((13438, 26412), 0, (8376, 32136), 419),
        ((13430, 26268), 144, (8520, 32232), 389),
        ((13445, 26555), -144, (8280, 32040), 419),
    ];
    for ((x, y), dx, (px, py), timer) in cases {
        let a = spawn(&mut s, 1, foot, Pos::new(x, y));
        s.add_guard_order(a, w, dx, 264, QueuePos::New);
        // A frame off both sixteen-frame phases and the review's.
        let o = i64::from(s.units[a].index);
        s.work(a, 1 - o);
        let post = Pos::new(px, py);
        let head = s.units[a].orders[0];
        let Body::Move(m) = head.body else {
            panic!("no leg: {:?}", s.units[a].orders);
        };
        assert_eq!((m.kind, m.dest), (MoveKind::AttackTo, post), "dx {dx}");
        assert_eq!(m.angle, Angle(535_429_120), "the moving target's heading");
        assert!(!head.has(flag::ACTION));
        assert_eq!(m.timer, timer, "dx {dx}");
        let Body::Guard(g) = s.units[a].orders[1].body else {
            panic!("the guard is not under the leg");
        };
        assert_eq!((g.guard, g.idle, g.retry), (post, 0, 0));
    }
}

/// **The target's own mirror, `unit_masks & 2`** (`docs/GROUPS.md`
/// §25): `do_guard` negates `dx` at `5e5fed` when the guarded unit
/// carries it. **Every number is run196's**: The Despot `1/79` on
/// block 15094 at (42855, 22630), heading −14221312 and `unit_masks`
/// 0x4000A, and the three Longbowmen's `action_guard` offsets. The posts
/// are the ones the original printed on block 15095. `1/78`'s 22392
/// comes from the heading's `sinx` on the negated `dx`. Without the
/// mirror, `1/77` and `1/78` trade posts, which is what this crate
/// printed before item 711.
#[test]
fn a_guard_s_offset_is_mirrored_by_its_target_s_own_flag() {
    let posts = |mirror: bool| {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let w = spawn(&mut s, 1, wagon, Pos::new(42855, 22630));
        s.units[w].movement.heading = Angle(-14_221_312);
        s.units[w].movement.mirror = mirror;
        s.add_move_order(
            w,
            Pos::new(39480, 20184),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        [0, -144, 144].map(|dx| {
            let a = spawn(&mut s, 1, foot, Pos::new(42648, 20472));
            s.add_guard_order(a, w, dx, 264, QueuePos::New);
            let o = i64::from(s.units[a].index);
            s.work(a, 1 - o);
            let Body::Guard(g) = s.units[a].orders[1].body else {
                panic!("no leg: {:?}", s.units[a].orders);
            };
            (g.guard.x, g.guard.y)
        })
    };
    assert_eq!(
        posts(true),
        [(42840, 22344), (42984, 22344), (42696, 22392)],
        "run196's 1/76, 1/77 and 1/78 on block 15095"
    );
    assert_eq!(
        posts(false),
        [(42840, 22344), (42696, 22392), (42984, 22344)],
        "unmirrored, the flanks trade posts"
    );
}

/// `unit_masks & 2`'s two writers (`docs/GROUPS.md` §25), for a unit
/// that leads no group. `Unit::set_angle@00605400` flips it on a turn
/// `reversing` admits (`605424`) and leaves it on a smaller one, in
/// every port of `set_angle`. `Unit::kill_current_order`'s move branch
/// writes the handed-back mirror into it (`5e3087`/`5e308d`). Both
/// writes come **before** the leader test, so a lone unit takes them.
#[test]
fn a_unit_s_own_mirror_flips_on_a_reversing_turn_leader_or_not() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    s.units[a].movement.heading = Angle(0x5555_5555);
    assert!(
        !s.units[a].movement.mirror,
        "born clear, as run192's 0x40000"
    );
    // run192's Despot on 14985: 0x55555555 → −756678656, reversing.
    s.unit_set_angle(a, Angle(-756_678_656));
    assert!(s.units[a].movement.mirror, "a reversing turn sets it");
    s.unit_set_angle(a, Angle(-756_678_656 + 0x1000_0000));
    assert!(s.units[a].movement.mirror, "a smaller one leaves it");
    s.units[a].movement.set_heading(Angle(0x4000_0000));
    assert!(
        !s.units[a].movement.mirror,
        "the ordinary path flips it too"
    );
    s.hand_back_facing(a, true, Angle(0x4000_0000));
    assert!(
        s.units[a].movement.mirror,
        "a dying move hands its mirror back"
    );
    s.hand_back_facing(a, true, Angle(-0x4000_0000));
    assert!(
        !s.units[a].movement.mirror,
        "inverted when the unit has turned"
    );
}

/// On its post beside a target that stands, a guard faces **outward**
/// — `find_angle(post − target)` — and counts `idle`; a dead target
/// ends the order.
#[test]
fn a_guard_on_its_post_faces_out_and_a_dead_target_ends_it() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1000, 0x1000));
    s.units[w].movement.heading = Angle(0);
    // `dy` 264 at heading 0 is 264 south of the target; its cell centre.
    let post = Pos::new(0x1000 / 48 * 48 + 24, (0x1000 - 264) / 48 * 48 + 24);
    let a = spawn(&mut s, 1, foot, post);
    s.add_guard_order(a, w, 0, 264, QueuePos::New);
    let frame = 1; // `o + frame` off both sixteen-frame phases
    s.work(a, frame);
    assert_eq!(s.order_type(a), index::GUARD, "no leg: already on the post");
    let Body::Guard(g) = s.units[a].orders[0].body else {
        unreachable!()
    };
    assert_eq!((g.guard, g.idle), (post, 1));
    assert_eq!(
        s.units[a].movement.heading,
        find_angle(post.x - 0x1000, post.y - 0x1000)
    );
    s.units[w].health = 0;
    s.work(a, frame + 1);
    assert_eq!(s.order_type(a), index::NONE);
}

/// The review's `GUARD` arm (`Unit::work@0060d4a0`): on the
/// sixty-four-frame phase the reposition leg is dropped and `do_guard`
/// issues a fresh one the same frame; off it the leg runs on.
#[test]
fn the_review_drops_a_guard_s_leg_on_the_sixty_four_frame_phase() {
    let mut s = sim();
    let foot = fighter(&mut s);
    let wagon = wagon_type(&mut s);
    let a = spawn(&mut s, 1, foot, Pos::new(0x3000, 0x3000));
    let w = spawn(&mut s, 1, wagon, Pos::new(0x1000, 0x1000));
    s.add_move_order(
        w,
        Pos::new(20000, 20000),
        MoveKind::AttackTo,
        QueuePos::New,
        true,
    );
    s.add_guard_order(a, w, 0, 264, QueuePos::New);
    let o = i64::from(s.units[a].index);
    s.work(a, 1 - o);
    let timer = |s: &Sim| match s.units[a].orders[0].body {
        Body::Move(m) => m.timer,
        _ => panic!("no leg"),
    };
    let first = timer(&s);
    s.work(a, 2 - o);
    assert_eq!(timer(&s), first - 1, "off the phase the leg runs on");
    s.work(a, 64 - o);
    assert_eq!(s.units[a].orders.len(), 2, "one leg over the guard");
    assert!(
        timer(&s) > first - 2,
        "on it the leg is re-issued: {}",
        timer(&s)
    );
}

// ------------------------------------------------------------------
// The second reading's corrections (`docs/audit/2026-08-25-groups.md`)
// ------------------------------------------------------------------

#[test]
fn halt_clears_the_group_s_form_and_leaves_every_member_s_own() {
    // `action_halt@0070d0c0:29`: `GroupData.form = −1` is written once,
    // on the **group**, before any member is examined. No unit's `+0xaa`
    // is touched — and `get_form` reads the unit bytes, so writing them
    // would make the group's next move inherit −1 instead of the form
    // the members still carry.
    let mut s = sim();
    let t = fighter(&mut s);
    let slot = s.init_army(1, None);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(s.armies[1].list[slot].group.form, 0, "the move settled 0");
    assert_eq!(s.group_get_form(&g), 0, "and wrote it to both members");

    s.group_action_halt(&g, 0);
    assert_eq!(
        s.armies[1].list[slot].group.form, -1,
        "the halt clears the group's own form"
    );
    assert_eq!(
        s.group_get_form(&g),
        0,
        "and leaves every member's +0xaa where it was"
    );
}

#[test]
fn stance_cycles_from_the_modal_option_not_the_leader_s() {
    // `action_stance@0070d440:38` seeds the cycle with
    // `get_stance_option(NULL)` — the modal option over the members —
    // not with the leader's own stance.
    let mut s = sim();
    let t = fighter(&mut s);
    let lead = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
    let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let g = group_of(1, &[lead, b, c]);
    s.units[lead].combat.stance = Stance::Raid; // option 3
    s.units[b].combat.stance = Stance::Aggressive; // option 0
    s.units[c].combat.stance = Stance::Aggressive; // option 0
    s.group_action_stance(&g, -1);
    assert_eq!(
        s.units[lead].combat.stance,
        Stance::Defensive,
        "the modal option is 0, so −1 steps to 1 — not to 4 from the leader's 3"
    );
}

#[test]
fn stance_skips_a_plane_but_not_a_helicopter() {
    // `action_stance@0070d440:95`: vslot `+0xc0` is `is_plane`, so a
    // plane never receives a stance. A helicopter carries `unit_flags`
    // bit `f` and is not a plane, so it does.
    let mut s = sim();
    let plane = flier(&mut s, false);
    let heli = flier(&mut s, true);
    let foot = fighter(&mut s);
    let f = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
    let p = spawn(&mut s, 1, plane, Pos::new(0x1100, 0x1000));
    let h = spawn(&mut s, 1, heli, Pos::new(0x1200, 0x1000));
    let g = group_of(1, &[f, p, h]);
    s.group_action_stance(&g, stance_option(Stance::Raze));
    assert_eq!(s.units[f].combat.stance, Stance::Raze);
    assert_eq!(
        s.units[p].combat.stance,
        Stance::Aggressive,
        "a plane is skipped"
    );
    assert_eq!(
        s.units[h].combat.stance,
        Stance::Raze,
        "a helicopter is not a plane"
    );
}

#[test]
fn a_helicopter_takes_a_group_move_where_a_plane_does_not() {
    // The same `is_plane` split at every action's inner loop
    // (`group_member_orderable`).
    let mut s = sim();
    let plane = flier(&mut s, false);
    let heli = flier(&mut s, true);
    let p = spawn(&mut s, 1, plane, Pos::new(0x1000, 0x1000));
    let h = spawn(&mut s, 1, heli, Pos::new(0x1100, 0x1000));
    let g = group_of(1, &[p, h]);
    s.group_action_move_to(
        &g,
        Pos::new(0x4000, 0x4000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        true,
    );
    assert_eq!(s.order_type(p), index::NONE, "a plane takes no group move");
    assert_eq!(s.order_type(h), index::MOVE_TO, "a helicopter does");
}

#[test]
fn attack_keeps_a_lone_shot_and_a_march_into_range_and_re_orders_the_rest() {
    // `action_attack@00712490:390`–`424`: the "already attacking" skip
    // is conditional on two sub-arms. `orderlist.count == 1` keeps it;
    // so does a current *move* order whose destination puts the target
    // in range. Anything else falls through and is re-ordered.
    let mut s = sim();
    let t = fighter(&mut s);
    let foe = spawn(&mut s, 0, t, Pos::new(0x9000, 0x1000));
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1400));
    let c = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1800));
    for u in [a, b, c] {
        s.add_attack_order(u, Obj::Unit(foe), QueuePos::New, true, true);
        assert!(!s.is_in_range(Obj::Unit(u), Obj::Unit(foe)));
    }
    // `b` is walking somewhere that leaves the target out of range;
    // `c`'s leg ends beside it. Both are transit moves, so the *action*
    // under them is still the attack.
    s.add_move_order(
        b,
        Pos::new(0x2000, 0x1400),
        MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    s.add_move_order(
        c,
        Pos::new(0x8fc0, 0x1000),
        MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    assert_eq!(s.units[b].orders.len(), 2);
    assert_eq!(s.units[c].orders.len(), 2);

    let g = group_of(1, &[a, b, c]);
    s.group_action_attack(&g, Obj::Unit(foe), true, QueuePos::New, 0);
    assert_eq!(
        s.units[a].orders.len(),
        1,
        "a lone out-of-range mandatory attack on the same target is kept"
    );
    assert_eq!(
        s.units[b].orders.len(),
        1,
        "a march that does not reach into range falls through and is replaced"
    );
    assert_eq!(
        s.units[c].orders.len(),
        2,
        "a march that ends in range of the target is kept"
    );
}

#[test]
fn the_siege_anchor_scores_only_members_that_are_on_the_map() {
    // `action_siege_attack_to@0070d830:194` gates every summed member on
    // `is_valid_unit() && is_on_map()`.
    let mut s = sim();
    let siege = siege_type(&mut s);
    let foot = fighter(&mut s);
    let near = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
    let far = spawn(&mut s, 1, siege, Pos::new(0x8000, 0x1000));
    // A garrisoned crowd out by `far`: off the map, so it must not pull
    // the anchor over to `far`.
    let mut hidden = Vec::new();
    for k in 0..6 {
        let u = spawn(&mut s, 1, foot, Pos::new(0x8000 + k * 0x40, 0x1000));
        s.units[u].on_map = false;
        hidden.push(u);
    }
    let escort = spawn(&mut s, 1, foot, Pos::new(0x1040, 0x1000));
    let mut list = vec![near, far, escort];
    list.extend_from_slice(&hidden);
    let g = group_of(1, &list);
    let sub = group_of(1, &[near, far]);
    assert_eq!(
        s.siege_anchor(&g, &sub),
        Some(near),
        "the off-map members do not count towards the score"
    );
}

#[test]
fn a_hurrying_army_with_no_city_clears_and_marches_its_shooting_siege() {
    // §6.5's clear/order asymmetry: the `QUEUE_NEW` clear gates on
    // `hurry` alone (`70524f`), the order loop on `hurry && a city was
    // found` (`7054c7`). But the order loop re-reads `order_type()`
    // **after** the clear has emptied the list (`Unit::close_orders`
    // kills every order; `UnitData::order_type` returns `NONE` on an
    // empty list), so the "shooting siege" arm no longer matches and
    // the unit takes the move like everyone else. The third pass
    // overturned the second's "issues it nothing"
    // (`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts").
    let mut s = sim();
    let siege = siege_type(&mut s);
    let slot = s.init_army(1, None);
    let m = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
    let foe = spawn(&mut s, 0, siege, Pos::new(0x1080, 0x1000));
    s.army_add_unit(1, slot, m);
    s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
    s.armies[1].list[slot].hurry = 1; // hurrying, and there is no city
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x8000, 0x8000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(
        s.order_type(m),
        index::ATTACK_TO,
        "the clear ran, and then the order loop no longer saw an ATTACK"
    );
    assert_eq!(s.units[m].orders.len(), 1);

    // Any other queue position skips the clear loop, so the unit is
    // still shooting when the order loop looks, and is left alone.
    s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
    assert_eq!(s.order_type(m), index::ATTACK);
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x8000, 0x8000),
        QueuePos::Last,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(s.order_type(m), index::ATTACK, "QUEUE_LAST clears nothing");
    assert_eq!(s.units[m].orders.len(), 1, "and appends nothing either");
}

#[test]
fn the_stance_type_is_decided_by_the_military_role_first() {
    // `UnitTypeData::get_stance_type@0061d350`, with the PDB's
    // `StanceTypes` (`STANCE_COMBAT = 0`, `WORKER = 1`, `CASTER = 2`,
    // `PACKER = 3`, `NONE = −1`): `role & MILITARY` → PACKER if
    // `unit_flags2 & 4`, else COMBAT; then the four citizen ids →
    // WORKER; then `(unit_flags2 & 6) == 2` → CASTER; else NONE. The
    // order of the tests is the point — a military caster is COMBAT,
    // a civilian packer is NONE, and "has an attack" decides nothing.
    let mut s = sim();
    let base = |role: u32, flags2: u32, packs: bool, attack: i32| UnitType {
        hits: 100,
        combat: combat::Profile {
            attack,
            packs,
            uber_size: 1,
            ..combat::Profile::default()
        },
        cols: crate::ai_load::UnitCols {
            role,
            unit_flags2: flags2,
            ..crate::ai_load::UnitCols::default()
        },
        ..UnitType::default()
    };
    let cases = [
        (
            base(0, 0, false, 15),
            StanceType::None,
            "an attack alone is not military",
        ),
        (
            base(role::ROLE_MILITARY, uflags2::CASTER, false, 15),
            StanceType::Combat,
            "a military caster is combat",
        ),
        (
            base(role::ROLE_MILITARY, 0, true, 15),
            StanceType::Packer,
            "a military packer packs",
        ),
        (
            base(0, 0, true, 0),
            StanceType::None,
            "a civilian packer has no stance",
        ),
        (
            base(role::ROLE_CITIZEN, 0, false, 0),
            StanceType::Worker,
            "a citizen works",
        ),
        (
            base(0, uflags2::CASTER, false, 0),
            StanceType::Caster,
            "a civilian caster casts",
        ),
        (
            base(0, uflags2::CASTER | uflags2::PACKS, true, 0),
            StanceType::None,
            "a caster that packs is neither",
        ),
    ];
    for (ty, want, why) in cases {
        let t = s.add_unit_type(ty);
        let u = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        assert_eq!(s.unit_stance_type(u), want, "{why}");
    }
}

#[test]
fn action_attack_without_mandatory_lets_each_member_take_what_is_nearest() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let far = spawn(&mut s, 0, t, Pos::new(0x9000, 0x1000));
    let near = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
    let g = group_of(1, &[a]);
    s.group_action_attack(&g, Obj::Unit(far), false, QueuePos::New, 0);
    assert_eq!(
        s.units[a].combat.target,
        Some(Obj::Unit(near)),
        "mandatory 0 retargets to whatever is nearest within the respond range"
    );
    s.group_action_attack(&g, Obj::Unit(far), true, QueuePos::New, 0);
    assert_eq!(
        s.units[a].combat.target,
        Some(Obj::Unit(far)),
        "mandatory 1 keeps the given target"
    );
}

/// **A member sent at a `Build` takes it, with no search** (item 1028,
/// `docs/COMBAT.md` §68.2): `Group::action_attack`'s `mandatory == 0`
/// retarget is under the target's vslot `+0x20` answering 0, so only a
/// unit or a wall target is re-searched; a `Build` is added to every
/// member through `action_attack+0xc44`. run369's packet: the look on
/// Great Lakes 4605 hands the human's city to group 65 and all eighteen
/// members take it, with the human's costlier camps nearer.
///
/// Made to fail by searching a `Build` target again (the word 2 takes
/// the costlier camp beside the member).
#[test]
fn a_group_sent_at_a_build_takes_it_without_a_search() {
    let mut s = sim();
    let t = fighter(&mut s);
    let bt = s.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let put = |s: &mut Sim, at: Pos, cost: i32| {
        let b = s.add_building(0, at, 0);
        s.buildings[b].ty = Some(bt);
        s.buildings[b].hits = 800;
        s.buildings[b].health = 800;
        s.buildings[b].combat = Some(combat::Profile {
            cost,
            ..combat::Profile::default()
        });
        b
    };
    let city = put(&mut s, Pos::new(0x2000, 0x1000), 0);
    let camp = put(&mut s, Pos::new(0x1400, 0x1000), 5000);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let g = group_of(1, &[a]);
    s.group_action_attack(&g, Obj::Building(city), false, QueuePos::New, 0);
    assert_eq!(
        s.units[a].combat.target,
        Some(Obj::Building(city)),
        "a member sent at a Build searched and took the camp {camp} beside it"
    );
}

/// **A raider wants the economy** (item 1028, `docs/COMBAT.md` §68.1):
/// `Object::compare_target`'s RAID arm. For a computer's raider a
/// peasant is worth `+900,000` and anything neither a peasant, a
/// caravan nor a combat unit a tenth; for a human's the peasant is worth
/// `+9,000,000`; an active building is worth `/ 20` of its cost and
/// none of its threat. Great Lakes 4688's `1/24` (stance 3) keeps the
/// citizen `0/3` at 9800 over the scout `0/0` at 15 (run368's packet).
///
/// Made to fail by answering `raiding` false (every assertion), by the
/// AI raider's weight `9,000,000` (the second), and by the building's
/// formula in place of `/ 20` (the last).
#[test]
fn a_raider_weighs_a_peasant_over_a_scout_and_a_building_at_a_twentieth() {
    let mut s = sim();
    let t = fighter(&mut s);
    let citizen = s.add_unit_type(UnitType {
        hits: 50,
        worker: crate::orders::Worker::Citizen,
        combat: combat::Profile {
            cost: 50,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    });
    let scout = s.add_unit_type(UnitType {
        hits: 50,
        combat: combat::Profile {
            cost: 50,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    });
    let bt = s.add_build_type(crate::build::BuildType {
        x_size: 2,
        y_size: 2,
        ..crate::build::BuildType::default()
    });
    let tower = s.add_building(0, Pos::new(0x1800, 0x1000), 0);
    s.buildings[tower].ty = Some(bt);
    s.buildings[tower].hits = 800;
    s.buildings[tower].health = 800;
    s.buildings[tower].combat = Some(combat::Profile {
        cost: 400,
        attack: 40,
        ..combat::Profile::default()
    });
    let ai = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let human = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1400));
    let p = spawn(&mut s, 0, citizen, Pos::new(0x1200, 0x1000));
    let sc = spawn(&mut s, 0, scout, Pos::new(0x1200, 0x1200));
    let foe = spawn(&mut s, 1, citizen, Pos::new(0x1200, 0x1400));
    let value = |s: &Sim, a: usize, o: Obj| s.compare_target(Obj::Unit(a), o, true, false);
    let calm = (
        value(&s, ai, Obj::Unit(p)),
        value(&s, ai, Obj::Unit(sc)),
        value(&s, ai, Obj::Building(tower)),
    );
    s.units[ai].combat.stance = combat::Stance::Raid;
    s.units[human].combat.stance = combat::Stance::Raid;
    let raid = (
        value(&s, ai, Obj::Unit(p)),
        value(&s, ai, Obj::Unit(sc)),
        value(&s, ai, Obj::Building(tower)),
    );
    // Out of reach and not raiding, `/5`: 100,000 + 400 over five.
    assert_eq!(calm, (201, 201, 1600), "the same three, not raiding");
    assert_eq!(
        raid,
        (9004, 15, 15),
        "a computer's raider: 900,000 + 400, 400 / 10 and the floor, 4 × 400 / 20 × 2 and the floor"
    );
    assert_eq!(
        value(&s, human, Obj::Unit(foe)),
        90_004,
        "a human's raider: 9,000,000 + 400"
    );
}

/// **A building's `targeted` count decays** (item 1028, `docs/COMBAT.md`
/// §68.3): `Wall::process@00640450`'s first statement quarters it,
/// signed toward zero, on every frame whose low three bits are the
/// owner's player number, beside `check_ever_seen`. run368's packet
/// holds the human's city at 0 on logger frame 4688, where this crate's
/// count, never decayed, read 31.
///
/// Made to fail by skipping the decay.
#[test]
fn a_building_s_targeted_count_is_quartered_on_its_owner_s_eighth_frame() {
    let mut s = sim();
    let b = s.add_building(0, Pos::new(0x1000, 0x1000), 0);
    s.buildings[b].targeted = 31;
    s.process_building(b, 4681);
    assert_eq!(s.buildings[b].targeted, 31, "4681 is not who 0's frame");
    s.process_building(b, 4688);
    assert_eq!(s.buildings[b].targeted, 7, "4688 & 7 == 0: 31 / 4");
}

/// **An army group's attack-move looks, and hands what it finds to
/// the whole group** (item 1012, `docs/COMBAT.md` §66):
/// `Unit::do_group_attack_to@005e74e0`'s fifteen-frame look, then
/// `find_nearby_target`'s add arm — a `GROUP_ATTACK_TO` member of a
/// group that `is_attacking_to` calls `Group::action_attack(…,
/// QUEUE_FIRST, 4)`, so every member attacks and the attack-move comes
/// back under each. With one member off the attack-move the group is
/// not attacking-to, and the looker takes the find alone.
///
/// Made to fail by skipping the look (the first assertion), and by
/// answering `group_is_attacking_to` true (the last).
#[test]
fn an_army_group_s_look_hands_its_find_to_the_group() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
    let mut g = Group::stack(1);
    s.group_add(&mut g, a);
    s.group_add(&mut g, b);
    assert!(s.push_group(&mut g, true));
    let g = s.group_of(a).expect("seated");
    s.group_action_move_to(
        &g,
        Pos::new(0x6000, 0x1000),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(s.order_type(a), index::GROUP_ATTACK_TO);
    let foe = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1400));
    let Some(Order {
        body: Body::Move(m),
        ..
    }) = s.current_order(a).copied()
    else {
        panic!("a group move");
    };
    let id = m.group.expect("a group order").id;
    let frame = 15 * 300 - i64::from(s.units[a].index);
    let mut solo = s.clone();
    s.do_group_attack_to_tail(a, frame, id);
    for u in [a, b] {
        assert_eq!(s.order_type(u), index::ATTACK, "member {u} was not sent");
        assert_eq!(s.units[u].combat.target, Some(Obj::Unit(foe)));
        assert!(
            s.units[u]
                .orders
                .iter()
                .any(|o| o.index() == index::GROUP_ATTACK_TO),
            "member {u}'s attack-move did not come back under its attack"
        );
    }
    // One member walking a plain move: the group is not attacking-to.
    solo.add_move_order(
        b,
        Pos::new(0x1200, 0x3000),
        MoveKind::MoveTo,
        QueuePos::New,
        false,
    );
    solo.do_group_attack_to_tail(a, frame, id);
    assert_eq!(solo.order_type(a), index::ATTACK);
    assert_eq!(
        solo.order_type(b),
        index::MOVE_TO,
        "the find went to the group though a member was off the attack-move"
    );
}

/// **A fresh pool slot is stamped with the frame it was pushed on**,
/// and an equal group pushed again keeps its record's stamp:
/// `Groups::copy_group@006fa690` writes `+0x14 = game->frame`, and
/// `push_group` calls it only on the slot `get_open_slot` hands back
/// (`docs/GOLDEN.md` §23, run215's `stamp 621`).
#[test]
fn a_pushed_group_is_stamped_with_the_frame_it_took_its_slot() {
    let mut s = sim();
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
    let b = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
    s.frame = 621;
    let mut g = Group::stack(0);
    s.group_add(&mut g, a);
    s.group_add(&mut g, b);
    assert!(s.push_group(&mut g, true));
    let slot = s.units[a].group_ptr.expect("a slot");
    assert_eq!(s.pool_state(0, slot).map(|st| st.stamp), Some(621));
    s.frame = 701;
    let mut again = Group::stack(0);
    s.group_add(&mut again, a);
    s.group_add(&mut again, b);
    assert!(s.push_group(&mut again, true));
    assert_eq!(
        s.pool_state(0, slot).map(|st| st.stamp),
        Some(621),
        "an equal group is not copied, so its stamp stays"
    );
}

/// `reversing@0092cf20` is inclusive at **both** ends (§6.3): the
/// unsigned compares are `d < 0x40000000` → no and `d < 0xc0000001` →
/// yes, so exactly 90° reverses and so does exactly 270°.
#[test]
fn reversing_takes_both_bounds() {
    assert!(!reversing(Angle(0x3fff_ffff)), "a hair under 90°");
    assert!(reversing(Angle(0x4000_0000)), "exactly 90°");
    assert!(reversing(Angle(-0x4000_0000)), "exactly 270°, i.e. −90°");
    assert!(
        !reversing(Angle(-0x3fff_ffff)),
        "a hair under 270° from the other side"
    );
    assert!(reversing(Angle(i32::MIN)), "dead astern");
}

/// **The mirror flag's three writers, in one march** (§6.3).
///
/// The record's own version of this is
/// `run31_s_three_mirrors_come_out_of_facing_s_three_writers` in
/// `rondata::diff`; this is the same machine driven by the simulation
/// rather than replayed out of a dump, and it is what proves the
/// writers are wired to the places that call them.
#[test]
fn a_group_s_mirror_is_the_last_layout_s_and_not_the_march_s() {
    /// Degrees as the engine's binary angle, exactly.
    fn deg(n: i64) -> Angle {
        Angle((n * (1 << 32) / 360) as i32)
    }
    let mut s = sim();
    let t = fighter(&mut s);
    let slot = s.init_army(1, None);
    let a = spawn(&mut s, 1, t, Pos::new(0x4000, 0x4000));
    let b = spawn(&mut s, 1, t, Pos::new(0x4100, 0x4000));
    s.army_add_unit(1, slot, a);
    s.army_add_unit(1, slot, b);
    let g = s.army_group(1, slot);
    assert_eq!(
        s.group_find_leader(&g),
        Some(a),
        "the first of one category"
    );
    assert!(!s.armies[1].list[slot].group.facing, "Group::clear");
    // Both face north to start. `Unit::init` would leave them on
    // `Angle::INITIAL` — 120°, which is inside the first order's window
    // too, but the march below is written in round numbers from north.
    for u in [a, b] {
        s.units[u].movement.set_facing(Angle::NORTH);
    }

    // A move at 80°. The leader faces north, which is inside the 90°
    // window, so the block is **not** mirrored — and the order carries
    // that answer out with it.
    let first = deg(80);
    let order_facing = |s: &Sim, u: usize| match s.current_order(u).map(|o| o.body) {
        Some(crate::orders::Body::Move(m)) => m.facing,
        _ => panic!("the unit took a move"),
    };
    s.group_action_move_to(
        &g,
        Pos::new(0x5000, 0x5000),
        QueuePos::New,
        true,
        first,
        MoveKind::MoveTo,
        false,
    );
    assert_eq!(order_facing(&s, a), Some(false), "an unmirrored block");
    assert!(
        !s.armies[1].list[slot].group.facing,
        "and the toggle around Form::compute put the flag back"
    );

    // The march. The leader overshoots to 100° — a 100° `set_angle`,
    // so the group's flag flips — and settles back onto 80°, which is
    // only 20° and flips nothing. The group now reads `facing 1` while
    // its live layout is unmirrored, which is precisely run31's
    // frame 327 and the state that made §6.3 look wrong.
    s.unit_set_angle(a, deg(100));
    assert!(
        s.armies[1].list[slot].group.facing,
        "set_angle toggles the leader's group"
    );
    s.unit_set_angle(a, first);
    s.unit_set_angle(b, deg(-100));
    assert!(
        s.armies[1].list[slot].group.facing,
        "a small turn does not, and a follower's never does"
    );

    // A second `QUEUE_NEW` move, at 150°. Taken at face value the
    // flag would mirror this block; the clear runs first, the leader's
    // dying order hands back the mirror **it** was laid out with, and
    // the answer is the unmirrored one again.
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x5000, 0x6000),
        QueuePos::New,
        true,
        deg(150),
        MoveKind::MoveTo,
        false,
    );
    assert_eq!(
        order_facing(&s, a),
        Some(false),
        "the hand-back discards the march's flag and restores the \
             last layout's — reading `facing` straight would mirror here"
    );
    assert!(
        !s.armies[1].list[slot].group.facing,
        "and the march's 1 is gone: the hand-back **assigns**, so a \
             record dumped after this click shows the last layout's flag \
             until the leader turns again"
    );
}

/// **A flight home counts the planes homed on the base, flying too**
/// (item 3991, `docs/GOLDEN.md` §184): `Group::action_flight@006fb260`'s
/// `MOVE_TO` gate asks `ObjectData::can_carry@006483c0` of each member,
/// whose room is `num_aircraft_here(0) < 10` — every plane whose home
/// is the base, standing in it or not. Ten planes ordered home together
/// fill it with none inside, so an eleventh is refused, while one of the
/// ten, its own home, is still pointed home. Made to fail with the room
/// counting the garrison (the eleventh takes a strafe).
#[test]
fn a_flight_home_is_refused_by_ten_homed_planes_none_of_them_inside() {
    let mut s = sim();
    let (base, _) = airbase_and_target(&mut s);
    let t = flier(&mut s, false);
    let ten: Vec<usize> = (0..10)
        .map(|i| spawn(&mut s, 0, t, Pos::new(11616 + 96 * i, 16224)))
        .collect();
    s.group_action_flight(&group_of(0, &ten), Obj::Building(base), Flight::Home);
    assert!(ten.iter().all(|&u| s.home_base(u) == Some(base)));
    assert!(s.buildings[base].garrison.is_empty(), "none inside");
    assert_eq!(s.aircraft_homed(base), 10);
    let late = spawn(&mut s, 0, t, Pos::new(10104, 16248));
    s.group_action_flight(
        &group_of(0, &[late, ten[0]]),
        Obj::Building(base),
        Flight::Home,
    );
    assert!(s.units[late].orders.is_empty(), "the eleventh is refused");
    assert!(
        matches!(s.units[ten[0]].orders.front().map(|o| o.body),
            Some(Body::Strafe(sf)) if sf.home == Some(base) && sf.returning),
        "its own home still takes it"
    );
}

/// **A strike on a mixed group splits** (item 3991, `docs/GOLDEN.md`
/// §184; `Group::action_flight@006fb260`, `6fb3f9`..`6fb53f`): the
/// members that are not planes (`UnitData::is_plane`, so a helicopter
/// among them) take `action_attack(o, whom, mandatory 1, QUEUE_NEW)` on
/// the target, and the planes are flown alone — the one inside its base
/// takes its strike. Made to fail with the split dropped (the ground
/// member and the helicopter take nothing).
#[test]
fn a_strike_on_a_mixed_group_sends_the_rest_to_attack() {
    let mut s = sim();
    let (base, _) = airbase_and_target(&mut s);
    let pt = flier(&mut s, false);
    let ht = flier(&mut s, true);
    let ft = fighter(&mut s);
    let foe = spawn(&mut s, 1, ft, Pos::new(15360, 16000));
    let p = spawn(&mut s, 0, pt, Pos::new(11616, 13920));
    s.units[p].on_map = false;
    s.units[p].inside = Some(base);
    s.buildings[base].garrison.push(p);
    let h = spawn(&mut s, 0, ht, Pos::new(14000, 16000));
    let g = spawn(&mut s, 0, ft, Pos::new(14400, 16000));
    s.group_action_flight(&group_of(0, &[p, h, g]), Obj::Unit(foe), Flight::Strike);
    for u in [h, g] {
        let o: Vec<u8> = s.units[u].orders.iter().map(|o| o.index()).collect();
        assert_eq!(o, [index::ATTACK], "{u}: one attack");
        assert_eq!(s.units[u].combat.target, Some(Obj::Unit(foe)));
    }
    assert!(!s.units[p].orders.is_empty(), "the plane takes its strike");
}
