use crate::combat::{self, Obj};
use crate::groups::group::{Flight, Group};
use crate::movement::Angle;
use crate::orders::{Body, flag};
use crate::territory::attrition::Domain;
use crate::world::Pos;
use crate::{Player, Sim, Unit, UnitType};

fn sim() -> Sim {
    let mut s = Sim::new(
        crate::core::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    s.cities.nation[0].human = true;
    s.cities.nation[1].human = false;
    s.leaders.at_war[0][1] = true;
    s.leaders.at_war[1][0] = true;
    s.frame = 766;
    s
}

/// who=0's Airbase at run223's point, and who=1's Barracks, finished
/// and seen by who=0 (`ever_seen` 3 from 762).
fn base_and_target(s: &mut Sim) -> (usize, usize) {
    let airbase = s.add_build_type(crate::cities::build::BuildType {
        ident: crate::cities::build::Ident::Airbase,
        ..crate::cities::build::BuildType::default()
    });
    let base = s.add_building(0, Pos::new(11616, 13920), 0);
    s.buildings[base].ty = Some(airbase);
    let target = s.add_building(1, Pos::new(21120, 16512), 0);
    s.buildings[target].started = true;
    s.buildings[target].active = true;
    s.buildings[target].combat = Some(combat::Profile::default());
    s.buildings[target].health = 1200;
    s.buildings[target].ever_seen = 3;
    (base, target)
}

/// A Fighter with the `FIGHTER` row's tank, 400, standing inside `base`.
fn fighter_inside(s: &mut Sim, base: usize, mana: i32) -> usize {
    let t = UnitType {
        hits: 100,
        moves: 75,
        mana,
        turn_speed: crate::movement::degrees_to_angle(10).0,
        kind: crate::territory::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::territory::attrition::UnitKind::default()
        },
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            max_range: 7 * 192,
            domain: Domain::Air,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    let ty = s.add_unit_type(t);
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(0, index, Pos::new(11424, 14005), 100);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    s.units[u].kind = s.tables.unit_types[ty].kind;
    s.units[u].movement.speed = 75;
    s.units[u].movement.turning = crate::turning_of(&s.tables.unit_types[ty]);
    s.init_guys(u, Some(ty));
    s.go_inside(u, base);
    s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
    u
}

/// **A base on ground under 0 is aimed at from 0** (item 1009,
/// `docs/GOLDEN.md` §43): the approach home reads the base's
/// `z_internal`, which `SubObject::init` takes from `find_tcoord_z`
/// with its fourth argument 1 — a negative height answers 0. run362's
/// `0/2008` stands on a tile of −42 and prints `z_internal` 0; `0/7`'s
/// descent to it parts on 2399 without the clamp.
#[test]
fn the_approach_home_reads_a_base_on_low_ground_as_height_zero() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    s.units[u].pos = Pos::new(8936, 16261);
    let t = s.buildings[base].pos.tile();
    s.world.set_tile_z(t, 0);
    let level = s.home_approach(u, base).z;
    s.world.set_tile_z(t, -42);
    assert_eq!(s.home_approach(u, base).z, level, "clamped at 0");
    s.world.set_tile_z(t, 157);
    assert_eq!(
        s.home_approach(u, base).z,
        level + 157,
        "above 0 it is read"
    );
}

fn group_of(who: Player, list: &[usize]) -> Group {
    Group {
        who,
        army: None,
        pushed: None,
        list: list.to_vec(),
    }
}

/// **The tank** (`Unit::process@00610bc0`'s air arm): one a frame on
/// the map while any is left, to the type's `MANA` and no further;
/// inside, `AIR_UNIT_MANA_RECHARGE` a frame, to 0. run265: 1 on 779,
/// 400 on 1178; −2 a block inside. Made to fail with the refill at 1
/// (the third assertion) and with the burn unbounded (the second).
#[test]
fn a_plane_burns_one_a_frame_on_the_map_and_refills_two_inside() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    s.units[u].mana_burn = 0;
    s.recover_mana(u, 779);
    assert_eq!(s.units[u].mana_burn, 1, "one a frame on the map");
    s.units[u].mana_burn = 399;
    s.recover_mana(u, 1178);
    s.recover_mana(u, 1179);
    assert_eq!(
        s.units[u].mana_burn, 400,
        "the tank's own size, and no more"
    );
    s.go_inside(u, base);
    s.units[u].mana_burn = 24;
    s.recover_mana(u, 767);
    assert_eq!(s.units[u].mana_burn, 22, "two a frame inside");
    s.units[u].mana_burn = 1;
    s.recover_mana(u, 768);
    assert_eq!(s.units[u].mana_burn, 0, "and never below 0");
}

/// **The heal inside** (item 3961, `Unit::process@00610bc0`'s air arm,
/// `006110a0`–`611154`): `AIRCRAFT_HEAL_RATE[heal level]` frames apart
/// on the plane's phase, a point back and `healing = 1`; off the phase
/// nothing is written, and on the map `healing` is 0 every frame.
/// run1151c: who=1's planes in their Airbase (heal level 1, 16 frames,
/// the Bomber's two thirds 11) read 1 one block in 11. Made to fail
/// with the phase dropped (the second assertion), with the rate off the
/// table's first entry (the first), and with the map's write gone (the
/// last).
#[test]
fn a_plane_inside_heals_on_its_phase_and_marks_healing_1() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    assert_eq!(s.heal_level(s.units[u].owner), 0, "no HEAL_FASTER held");
    let hurt = s.units[u].max_health - 3;
    s.units[u].health = hurt;
    let o = i64::from(s.units[u].index);
    let due = 20 * 100 - o;
    s.recover_mana(u, due);
    assert_eq!(
        (s.units[u].health, s.units[u].healing),
        (hurt + 1, 1),
        "on the level-0 phase: a point back and the mark"
    );
    s.units[u].healing = 0;
    s.recover_mana(u, due + 1);
    assert_eq!(
        (s.units[u].health, s.units[u].healing),
        (hurt + 1, 0),
        "off it: nothing"
    );
    s.units[u].healing = 7;
    s.come_out(u);
    s.recover_mana(u, due + 2);
    assert_eq!(s.units[u].healing, 0, "on the map: 0");
}

/// **A strike from inside a base** (`Group::action_flight@006fb260`'s
/// inside arm, run265's 768): one `StrafeOrder` on the target, home
/// the base, `mandatory`, the action bit, `returning` 0 — and the
/// plane still inside. Made to fail with the reach against `mana`
/// alone (no order: 9857 > 400).
#[test]
fn a_strike_on_a_plane_inside_its_base_is_a_strafe_home_to_it() {
    let mut s = sim();
    let (base, target) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
    let o = s.units[u].orders.front().copied().expect("a strike");
    assert_eq!(o.flags & flag::ACTION, flag::ACTION);
    let Body::Strafe(sf) = o.body else {
        panic!("a strafe")
    };
    assert_eq!(sf.target, Some(Obj::Building(target)));
    assert_eq!(sf.home, Some(base));
    assert!(sf.mandatory && !sf.returning);
    assert_eq!(sf.at, Some(Pos::new(21120, 16512)));
    assert_eq!(s.units[u].inside, Some(base), "it waits in the base");
}

/// **Out of reach or unseen, a plane still takes the strike; a
/// missile does not** (item 3968, T71-5; the listing's `6fbba8`
/// gives `valid_target`, the shield and the reach to the missile bit
/// alone): `vector_dist(base − target)` 9857 against a tank of 100 at
/// 75 a frame, and a target never seen. Made to fail with the three
/// gates on every plane (the first two assertions) and with none (the
/// last two).
#[test]
fn a_strike_beyond_the_tank_s_reach_binds_a_missile_only() {
    for unseen in [false, true] {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, if unseen { 400 } else { 100 });
        if unseen {
            let n = (s.world.fog_xs() * s.world.fog_ys()) as usize;
            assert!(s.world.set_fog(vec![0; n]), "a fogged world");
            s.buildings[target].ever_seen = 0;
        }
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        let o = s.units[u].orders.front().copied();
        assert!(
            matches!(o.map(|o| o.body), Some(Body::Strafe(sf)) if sf.target == Some(Obj::Building(target))),
            "unseen {unseen}: a plane takes the strike"
        );
        let ty = s.units[u].ty.unwrap();
        s.units[u].orders.clear();
        s.tables.unit_types[ty].combat.obj_masks |= crate::combat::mask::MISSILE;
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        assert!(
            s.units[u].orders.is_empty(),
            "unseen {unseen}: a missile does not"
        );
    }
}

/// **`land_plane` under a repeating base** (`Unit::land_plane
/// @005e9950`'s `005e9a43`, `docs/ORDERS.md` §40): with the home's
/// `build_masks & 0x80` set the plane goes inside with its patrol on
/// the stack and the action bit cleared — run265's Fighter on 1385 and
/// Bomber on 1489 — and with it clear the stack is closed. Made to fail
/// with the keep unconditional (the second arm keeps an order) and
/// with the flag left alone (the first arm's bit).
#[test]
fn a_plane_landing_at_a_repeating_base_keeps_its_order_unflagged() {
    for repeat in [true, false] {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        s.buildings[base].repeat_air = repeat;
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
        s.set_returning(u, true);
        s.land_plane(u);
        assert_eq!(s.units[u].inside, Some(base), "{repeat}: inside");
        if repeat {
            assert_eq!(s.units[u].orders.len(), 1, "the patrol stays");
            let o = s.units[u].orders.front().copied().unwrap();
            assert_eq!(o.flags & flag::ACTION, 0, "its action bit cleared");
            let Body::AirPatrol(p) = o.body else {
                panic!("a patrol")
            };
            assert!(!p.returning, "and `returning` cleared");
        } else {
            assert!(s.units[u].orders.is_empty(), "no repeat: closed");
        }
    }
}

/// **A strafe with no target goes as `land_plane` finds it** (item 3396,
/// `005e9950`): `clear_orders` for a `mandatory` one, which takes the patrol
/// under it too, `kill_current_order` for the rest, which leaves the patrol.
/// Made to fail with the two swapped, and with the path left standing at a
/// base that does not repeat (the last pair of rows).
#[test]
fn a_targetless_strafe_goes_by_its_mandatory_byte() {
    for (mandatory, left) in [(true, 0), (false, 1)] {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        s.buildings[base].repeat_air = true;
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
        s.add_strafe_order(
            u,
            None,
            Some(base),
            mandatory,
            crate::orders::QueuePos::First,
            true,
        );
        assert_eq!(
            s.units[u].orders.len(),
            2,
            "{mandatory}: a strafe on a patrol"
        );
        s.land_plane(u);
        assert_eq!(s.units[u].orders.len(), left, "{mandatory}: what is left");
    }
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
    s.units[u].path.push(crate::orders::PathData {
        to: Pos::new(0, 0),
        tolerance: 0,
        flags: 0,
    });
    s.land_plane(u);
    assert!(s.units[u].path.is_empty(), "no repeat: the path goes");
}

/// **A computer's base sends its plane over a city under attack**
/// (`Object::do_launch@0064f3b0`'s sortie, item 1182, `docs/GOLDEN.md`
/// §50): on `(frame + id) % 32 == 0` a plane inside a computer's base
/// is cleared and patrolled over an enemy city with `city_flags & 3 ==
/// 3`; nothing for a human's base, off the cadence, or over a city not
/// under attack. run437's Biplane on 778. Made to fail with the city's
/// attack bit not read, and with the owner's human bit not read.
#[test]
fn a_computer_s_base_sends_its_plane_over_a_city_under_attack() {
    for (human, under_attack, on_cadence, sorties) in [
        (false, true, true, true),
        (true, true, true, false),
        (false, false, true, false),
        (false, true, false, false),
    ] {
        let mut s = sim();
        s.cities.nation[0].human = human;
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.units[u].orders.clear();
        let pos = s.buildings[target].pos;
        s.cities.cities.push(crate::cities::city::City {
            alive: true,
            owner: 1,
            race: Some(1),
            founder: 1,
            building: target,
            members: Vec::new(),
            reg: None,
            pos,
            capital: true,
            founding_capital: true,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: under_attack,
            attacking: under_attack,
            ever_attacked: under_attack,
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
        let id = i64::from(s.buildings[base].index);
        s.frame = 32 * 30 - id + if on_cadence { 0 } else { 1 };
        s.do_launch(base);
        let patrol = s.units[u].orders.front().and_then(|o| match o.body {
            crate::orders::Body::AirPatrol(p) => Some(p.current()),
            _ => None,
        });
        if sorties {
            assert_eq!(patrol, Some(pos), "no sortie over the city");
        } else {
            assert_eq!(
                patrol, None,
                "human {human}, attacked {under_attack}: a sortie"
            );
        }
    }
}

/// **`do_launch` keeps an unflagged order only under a repeating
/// base** (`Object::do_launch@0064f3b0`, `has_repeat_air() || flags &
/// 4`; `docs/GOLDEN.md` §32): at a full tank the kept patrol is
/// launched under the bit and killed without it, the plane staying
/// inside. Made to fail with the kill unconditional (the first arm)
/// and with the bit read as always set (the second).
#[test]
fn a_full_tank_launches_an_unflagged_patrol_only_under_the_bit() {
    for repeat in [true, false] {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
        s.buildings[base].repeat_air = true;
        s.land_plane(u);
        s.buildings[base].repeat_air = repeat;
        s.units[u].mana_burn = 0;
        s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
        s.do_launch(base);
        if repeat {
            assert_eq!(s.units[u].inside, None, "launched");
            assert_eq!(s.units[u].orders.len(), 1, "on its patrol");
        } else {
            assert_eq!(s.units[u].inside, Some(base), "still inside");
            assert!(s.units[u].orders.is_empty(), "the patrol killed");
        }
    }
}

/// **The walk ends at a launch** (`Object::do_launch@0064f3b0`,
/// `64f806`; `docs/GOLDEN.md` §38, item 915): two planes full under
/// the bit, and one call launches the first and never reaches the
/// second — the next link is the launched plane's own, which
/// `remove_from_inside` has cleared. Under the emulator, with the
/// link left, the second joins `launching`. Made to fail with the
/// walk carried on after the launch.
#[test]
fn a_launch_ends_the_walk_along_the_base_s_chain() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    s.buildings[base].repeat_air = true;
    let mut planes = Vec::new();
    for _ in 0..2 {
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
        s.land_plane(u);
        s.units[u].mana_burn = 0;
        planes.push(u);
    }
    s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
    s.do_launch(base);
    assert_eq!(s.units[planes[0]].inside, None, "the first launched");
    assert_eq!(s.units[planes[1]].inside, Some(base), "the second inside");
    assert!(
        !s.buildings[base].launching.contains(&planes[1]),
        "and never walked: not in `launching`"
    );
    assert_eq!(s.units[planes[1]].orders.len(), 1, "its patrol kept");
}

/// **An aircraft trained at an Airbase stays in it** (`Build::train
/// @0062f9b0`'s `CARRY_AIR` arm, `62fac0`; `docs/GOLDEN.md` §38, item
/// 915): no `come_out`, no order, in the base's chain; the same unit
/// trained at a building without the mask comes out. Made to fail with
/// the arm dropped (the plane on the map).
#[test]
fn an_aircraft_trained_at_an_airbase_stays_inside_with_no_order() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let first = fighter_inside(&mut s, base, 300);
    let ty = s.units[first].ty.unwrap();
    let trained = s.build_train(base, ty).unit;
    assert_eq!(s.units[trained].inside, Some(base), "inside the Airbase");
    assert!(s.units[trained].orders.is_empty(), "with no order");
    assert_eq!(
        s.buildings[base].garrison.last(),
        Some(&trained),
        "at the chain's tail"
    );
    s.units[trained].mana_burn = 0;
    s.do_launch(base);
    assert_eq!(s.units[trained].inside, Some(base), "passed over");
    let barracks = s.add_build_type(crate::cities::build::BuildType {
        ident: crate::cities::build::Ident::Barracks,
        ..crate::cities::build::BuildType::default()
    });
    let b = s.add_building(0, Pos::new(5000, 5000), 0);
    s.buildings[b].ty = Some(barracks);
    let out = s.build_train(b, ty).unit;
    assert_eq!(s.units[out].inside, None, "a Barracks lets it out");
}

/// The front patrol of `u`, which the tests below expect.
fn patrol_of(s: &Sim, u: usize) -> (crate::orders::AirPatrolOrder, u8) {
    match s.units[u].orders.front() {
        Some(o) => match o.body {
            Body::AirPatrol(p) => (p, o.flags),
            ref b => panic!("0/{}: not a patrol: {b:?}", s.units[u].index),
        },
        None => panic!("0/{}: no order", s.units[u].index),
    }
}

/// **An Airbase's gather point re-orders every plane homed there, on
/// every press** (item 947, `Build::add_gather_point@00622e70`'s hangar
/// loop; `docs/GOLDEN.md` §41). `QUEUE_NEW` sends the flying plane home
/// and empties the one inside first, and then each takes a patrol over
/// the list with the action bit; the flying one's heights are the strafe
/// home's. `QUEUE_LAST` rebuilds each from the whole list and carries a
/// patrol's `cruising_alt` and `sharp_turn`. Made to fail with the
/// hangar loop dropped (the first press), with the heights not carried
/// (the second), and with the later points not appended.
#[test]
fn an_airbase_s_gather_point_re_orders_every_plane_homed_there() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let inside = fighter_inside(&mut s, base, 400);
    let flying = fighter_inside(&mut s, base, 400);
    s.come_out(flying);
    s.add_air_patrol_order(inside, Pos::new(21120, 16512), Some(base), false);
    s.add_air_patrol_order(flying, Pos::new(21120, 16512), Some(base), false);
    let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
    let point = |pos| crate::cities::rally::GatherPoint { pos, action: 0 };
    s.add_gather_point(base, point(p1), true);
    for u in [inside, flying] {
        let (p, flags) = patrol_of(&s, u);
        assert_eq!(p.live(), [p1], "0/{}: one patrol over P1", s.units[u].index);
        assert_eq!(flags & flag::ACTION, flag::ACTION, "the action bit");
        assert_eq!((p.cruising_alt, p.sharp_turn), (0x640, 0));
        assert_eq!(s.units[u].orders.len(), 1);
    }
    if let Some(Body::AirPatrol(p)) = s.units[flying].orders.front_mut().map(|o| &mut o.body) {
        p.cruising_alt = 1800;
        p.sharp_turn = -1;
    }
    s.add_gather_point(base, point(p2), false);
    let (p, _) = patrol_of(&s, flying);
    assert_eq!(p.live(), [p1, p2], "the list whole");
    assert_eq!(
        (p.waypoint, p.cruising_alt, p.sharp_turn),
        (0, 1800, -1),
        "carried"
    );
    let (p, _) = patrol_of(&s, inside);
    assert_eq!((p.live(), p.cruising_alt), (&[p1, p2][..], 0x640));
    assert_eq!(s.units[inside].inside, Some(base), "still inside");
}

/// **The Clear at an Airbase** (item 947, `Build::clear_gather@
/// 00623180`'s hangar half): a homed plane on the map is sent home
/// (`add_strafe_order(−1, −1, base, 0, QUEUE_NEW, 0)`), one inside loses
/// its orders and leaves `launching`. Made to fail with the half
/// dropped.
#[test]
fn the_clear_at_an_airbase_sends_a_flying_plane_home_and_empties_one_inside() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let inside = fighter_inside(&mut s, base, 400);
    let flying = fighter_inside(&mut s, base, 400);
    s.come_out(flying);
    // A plane on the map is homed by its air order's home.
    s.add_air_patrol_order(flying, Pos::new(21120, 16512), Some(base), false);
    let p1 = crate::cities::rally::GatherPoint {
        pos: Pos::new(11520, 7680),
        action: 0,
    };
    s.add_gather_point(base, p1, true);
    s.buildings[base].launching.push(inside);
    s.clear_gather(base);
    assert!(s.buildings[base].gather.is_empty());
    assert!(s.units[inside].orders.is_empty(), "no order inside");
    assert!(
        !s.buildings[base].launching.contains(&inside),
        "out of launching"
    );
    let o = s.units[flying].orders.front().copied().expect("an order");
    let Body::Strafe(sf) = o.body else {
        panic!("a strafe home: {:?}", o.body)
    };
    assert_eq!(
        (sf.target, sf.home, sf.returning, sf.mandatory, o.flags),
        (None, Some(base), true, false, 0)
    );
    assert_eq!(s.units[flying].orders.len(), 1);
}

/// **A plane trained under an Airbase's list takes a patrol over it**
/// (item 947, `Build::train@0062f9b0`'s `CARRY_AIR` arm, `62fadf`..
/// `62fbfb`): the first point with the action bit, the rest appended,
/// and it stays inside. Made to fail with the arm dropped.
#[test]
fn a_plane_trained_under_an_airbase_s_list_takes_a_patrol_over_it() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let first = fighter_inside(&mut s, base, 300);
    let ty = s.units[first].ty.unwrap();
    let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
    s.buildings[base].gather = vec![
        crate::cities::rally::GatherPoint { pos: p1, action: 0 },
        crate::cities::rally::GatherPoint { pos: p2, action: 0 },
    ];
    let trained = s.build_train(base, ty).unit;
    assert_eq!(s.units[trained].inside, Some(base), "inside");
    let (p, flags) = patrol_of(&s, trained);
    assert_eq!(p.live(), [p1, p2]);
    assert_eq!(
        (flags & flag::ACTION, p.home, p.waypoint),
        (flag::ACTION, Some(base), 0)
    );
}

/// **The patrol walks its points** (item 947, `Unit::do_air_patrol@
/// 005ea620`): within `0x240` of `points[waypoint]` the waypoint steps
/// on; alone at the last it stays there. Made to fail with the step
/// dropped.
#[test]
fn a_patrol_steps_its_waypoint_on_at_each_point_and_stays_on_the_last() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
    s.add_air_patrol_order(u, p1, Some(base), true);
    if let Some(Body::AirPatrol(p)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
        p.push(p2);
    }
    s.units[u].pos = p1;
    let frame = s.frame;
    s.do_air_patrol(u, frame);
    assert_eq!(patrol_of(&s, u).0.waypoint, 1, "on to P2");
    s.units[u].pos = p2;
    s.do_air_patrol(u, frame + 1);
    assert_eq!(patrol_of(&s, u).0.waypoint, 1, "the last kept");
    assert_eq!(s.units[u].orders.len(), 1, "alone, not killed");
}

/// **A launch sets the patrol's `waypoint` to 0** (item 947,
/// `Object::do_launch@0064f3b0`'s store at `64f6a0`, `+0x3c`, the PDB's
/// `PatrolOrder::waypoint`). Made to fail with the store dropped.
#[test]
fn a_launch_starts_a_patrol_at_its_first_point() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.add_air_patrol_order(u, Pos::new(11520, 7680), Some(base), true);
    if let Some(Body::AirPatrol(p)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
        p.push(Pos::new(5760, 5760));
        p.waypoint = 1;
    }
    s.units[u].mana_burn = 0;
    s.do_launch(base);
    assert_eq!(s.units[u].inside, None, "launched");
    assert_eq!(patrol_of(&s, u).0.waypoint, 0);
}

/// **The repeat button toggles off the first member's state**
/// (`Group::action_buildmask@006fc9a0`, `docs/GOLDEN.md` §32): one base
/// goes 1 → 0 → 1; of two, the first's state decides and a clear
/// clears everything after it; a building that cannot carry aircraft
/// and any mask but `0x80` are passed over. Made to fail with the bit
/// set whatever it held (the toggle), and with `all_set` never cleared
/// (the second pair).
#[test]
fn the_repeat_button_toggles_off_the_first_member() {
    let mut s = sim();
    let (base, target) = base_and_target(&mut s);
    // `Build::init`'s `|= 0x88`, which `add_building` does not run.
    s.buildings[base].repeat_air = true;
    assert_eq!(s.action_buildmask(&[base], 0x80), 1);
    assert!(!s.buildings[base].repeat_air, "4232 -> 4104");
    assert_eq!(s.action_buildmask(&[base], 0x80), 1);
    assert!(s.buildings[base].repeat_air, "4104 -> 4232");
    assert_eq!(s.action_buildmask(&[base], 0x40), 0, "not carried");
    assert_eq!(s.action_buildmask(&[target], 0x80), 0, "no hangar");
    let second = s.add_building(0, Pos::new(13824, 13920), 0);
    s.buildings[second].ty = s.buildings[base].ty;
    for (a, b, want) in [
        (true, false, (false, false)),
        (false, true, (true, false)),
        (false, false, (true, true)),
        (true, true, (false, false)),
    ] {
        s.buildings[base].repeat_air = a;
        s.buildings[second].repeat_air = b;
        s.action_buildmask(&[base, second], 0x80);
        assert_eq!(
            (s.buildings[base].repeat_air, s.buildings[second].repeat_air),
            want,
            "from ({a}, {b})"
        );
    }
}

/// **A strafe with no target goes, and the bit spares what is behind
/// it** (`005e9a43`'s `else if (!bVar2)`): under a repeating base the
/// killed strafe's successor keeps its action bit. Made to fail with
/// the flag cleared after the kill.
#[test]
fn a_killed_strafe_home_leaves_the_order_behind_it_flagged() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    s.buildings[base].repeat_air = true;
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
    s.add_strafe_order(
        u,
        None,
        Some(base),
        false,
        crate::orders::QueuePos::First,
        true,
    );
    assert_eq!(s.units[u].orders.len(), 2);
    s.land_plane(u);
    assert_eq!(s.units[u].orders.len(), 1, "the strafe went");
    let o = s.units[u].orders.front().copied().unwrap();
    assert!(matches!(o.body, Body::AirPatrol(_)));
    assert_eq!(o.flags & flag::ACTION, flag::ACTION, "untouched");
}

/// **The tank gates the launch, and the launch is the EXIT's**
/// (`Object::do_launch@0064f3b0`, `do_spec_anim@005e5880`; run265's
/// 768–778). While `mana_burn` is above 0 the plane stays inside with
/// its strike; on the call it reads 0 it comes out onto the base's
/// point less `0xc0`, turned to 0, its figure on the ground with
/// `last_bank`/`last_pitch` 0, and flies its first step north; the
/// base's `launch_frames` is 0 and `launching` empty. Made to fail
/// without the tank's test (out on the first call) and without the
/// EXIT (the ring's spot, south of the base).
#[test]
fn a_strike_from_inside_waits_for_the_tank_and_leaves_on_the_exit() {
    let mut s = sim();
    let (base, target) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.units[u].mana_burn = 22;
    s.units[u].airframe.pitch = crate::num::single::Single::from_i32(8);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
    for f in 768..778 {
        s.frame = f;
        s.recover_mana(u, f);
        s.do_launch(base);
        assert_eq!(s.units[u].inside, Some(base), "{f}: the tank is not full");
        assert_eq!(
            s.buildings[base].launch_frames,
            super::FRAMES_BETWEEN_LAUNCHES
        );
    }
    s.frame = 778;
    s.recover_mana(u, 778);
    assert_eq!(s.units[u].mana_burn, 0);
    s.do_launch(base);
    assert_eq!(s.units[u].inside, None, "out on the tank's first 0");
    assert!(s.units[u].on_map);
    assert_eq!(s.buildings[base].launch_frames, 0);
    assert!(s.buildings[base].launching.is_empty());
    let at = s.units[u].pos;
    assert_eq!(at.x, 11424, "the base's point less 0xc0");
    assert!(at.y < 13920 && 13920 - at.y <= 75, "one step north: {at:?}");
    let af = s.units[u].airframe;
    assert_eq!(af.last_bank.bits(), 0);
    assert_eq!(af.last_pitch.bits(), 0, "the pitch moved and zeroed");
    assert!(matches!(
        s.units[u].orders.front().map(|o| o.body),
        Some(Body::Strafe(_))
    ));
    let _ = Angle(0);
}

/// A Fighter out of `base` on a strike at `target`, its type flagged
/// `w` when `strafes`, heading straight at the Barracks.
fn striking_fighter(s: &mut Sim, strafes: bool) -> (usize, usize, i32, i32) {
    let (base, target) = base_and_target(s);
    let u = fighter_inside(s, base, 400);
    if strafes {
        let ty = s.units[u].ty.unwrap();
        s.tables.unit_types[ty].cols.unit_flags |= crate::ai::ai_load::uflags::STRAFES;
    }
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
    s.come_out(u);
    let (at, to) = (s.units[u].pos, s.buildings[target].pos);
    let (dx, dy) = (to.x - at.x, to.y - at.y);
    s.units[u].movement.heading = crate::movement::find_angle(dx, dy);
    (u, target, dx, dy)
}

/// **A strafing type on a strike wants half its cruising altitude**
/// (`pitch_aircraft`, `0x5e9001`–`0x5e920b`, `docs/ORDERS.md` §39),
/// and every predicate of the arm gives the whole back: the flag, the
/// target standing, the 90° off the nose. Made to fail with the flag
/// test dropped (the second assertion) and with the angle's bound at
/// 180° (the fourth).
#[test]
fn a_strafing_type_on_a_strike_wants_half_its_cruising_altitude() {
    let mut s = sim();
    let (u, target, dx, dy) = striking_fighter(&mut s, true);
    assert_eq!(
        s.strike_altitude(u, dx, dy, 1600, 0),
        800,
        "half, on a strike"
    );
    let mut plain = sim();
    let (v, _, vdx, vdy) = striking_fighter(&mut plain, false);
    assert_eq!(
        plain.strike_altitude(v, vdx, vdy, 1600, 0),
        1600,
        "no w, no half"
    );
    let h = s.units[u].movement.heading;
    s.units[u].movement.heading = Angle(h.0.wrapping_add(0x4000_0000));
    assert_eq!(
        s.strike_altitude(u, dx, dy, 1600, 0),
        800,
        "90° off is inside"
    );
    s.units[u].movement.heading = Angle(h.0.wrapping_add(0x4000_0000 + 0x100_0000));
    assert_eq!(
        s.strike_altitude(u, dx, dy, 1600, 0),
        1600,
        "past 90°, the whole"
    );
    s.units[u].movement.heading = h;
    s.buildings[target].alive = false;
    assert_eq!(
        s.strike_altitude(u, dx, dy, 1300, 0),
        1300,
        "no target, the whole"
    );
}

/// **A strike on a flying target wants that target's height** (item
/// 3991, `docs/GOLDEN.md` §184; `pitch_aircraft`, `0x5e9114`–`0x5e9206`):
/// the target figure's `z`, plus `min(300, max(0, (0x600 − dist)·5))`
/// when its object number is below the striker's — or, equal, its
/// owner is — and less it otherwise; over the ground ahead. Made to
/// fail with the arm dropped (the whole cruise comes back) and with the
/// comparison's sense turned (the second and third swap).
#[test]
fn a_strike_on_a_flying_target_wants_that_target_s_height() {
    let mut s = sim();
    let (u, _, _, _) = striking_fighter(&mut s, true);
    let ty = s.units[u].ty.unwrap();
    let mut e = Unit::new(1, s.units[u].index, Pos::new(11424, 12000), 100);
    e.ty = Some(ty);
    e.on_map = true;
    let e = s.add_unit(e);
    s.units[e].airframe.z = 700;
    if let Some(Body::Strafe(sf)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
        sf.target = Some(Obj::Unit(e));
    }
    let (dx, dy) = (0, -0x200);
    s.units[u].movement.heading = crate::movement::find_angle(dx, dy);
    // Equal numbers, the target's owner above: less 300, over 40.
    assert_eq!(s.strike_altitude(u, dx, dy, 1600, 40), 700 - 300 - 40);
    // The target's number below the striker's: plus.
    s.units[e].index = s.units[u].index - 1;
    assert_eq!(s.strike_altitude(u, dx, dy, 1600, 40), 700 + 300 - 40);
    // Above: less; and from 0x5a0 off, 0x60 · 5 = 480 caps at 300, 0x600 off gives 0.
    s.units[e].index = s.units[u].index + 1;
    assert_eq!(s.strike_altitude(u, dx, dy, 1600, 0), 400);
    assert_eq!(
        s.strike_altitude(u, 0, -0x600, 1600, 0),
        700,
        "no offset at 0x600"
    );
    assert_eq!(s.strike_altitude(u, 0, -0x5f0, 1600, 0), 700 - 80);
}

/// **A flying target farther than `0xc00` is led** (`do_strafe`,
/// `0x5eb05b`–`0x5eb0a8`; `docs/GOLDEN.md` §184): the strike aims
/// `(dist − 0xc00) / 3` ahead along the target's own heading, and at
/// the target itself from `0xc00` in. 4,096 off gives 341. Made to
/// fail with the divisor at 2 (item 3991's seventh mutation, which only
/// the chapter had caught); the call site in `do_strafe` dropped is the
/// chapter's alone.
#[test]
fn a_flying_target_farther_than_0xc00_is_led_a_third_of_the_excess() {
    let mut s = sim();
    let (u, _, _, _) = striking_fighter(&mut s, true);
    let ty = s.units[u].ty.unwrap();
    s.units[u].pos = Pos::new(8000, 16000);
    let mut e = Unit::new(1, s.units[u].index, Pos::new(8000, 16000 - 0x1000), 100);
    e.ty = Some(ty);
    e.on_map = true;
    let e = s.add_unit(e);
    s.units[e].airframe.z = 700;
    let h = crate::movement::find_angle(1, 0);
    s.units[e].movement.heading = h;
    let at = s.units[e].pos;
    let lead = s.strafe_lead(u, Obj::Unit(e));
    let want = Pos::new(
        at.x + crate::movement::sin_component(h, 341),
        at.y - crate::movement::cos_component(h, 341),
    );
    assert_ne!(want, at, "the heading must move the point");
    assert_eq!(lead, want, "a third of 4,096 − 0xc00 ahead");
    s.units[e].pos = Pos::new(8000, 16000 - 0xc00);
    assert_eq!(s.strafe_lead(u, Obj::Unit(e)), s.units[e].pos, "0xc00 in");
}

/// **run265's 798**: the Fighter at pitch 40 and `z` 677 comes down
/// two, where the whole 1600 over the ground held it at 40 (40.0 here
/// against 38.0 there, before item 842). Flat ground at 0: the target
/// is 800, `((800 − 677) / 25 · 20) / (0x240 / 75)` = 11, 29 under
/// the pitch. Made to fail with `strike_altitude` returning the whole.
#[test]
fn run265_s_climb_comes_down_at_the_strafers_half() {
    let mut s = sim();
    let (u, _, dx, dy) = striking_fighter(&mut s, true);
    s.units[u].airframe.pitch = crate::num::single::Single::from_i32(40);
    s.units[u].airframe.z = 677;
    let mut speed = 75;
    s.pitch_plane(u, dx, dy, 0, &mut speed, false);
    assert_eq!(s.units[u].airframe.pitch.to_i32(), 38);
    assert_eq!(s.units[u].airframe.last_pitch.to_i32(), 40);
    let mut plain = sim();
    let (v, _, vdx, vdy) = striking_fighter(&mut plain, false);
    plain.units[v].airframe.pitch = crate::num::single::Single::from_i32(40);
    plain.units[v].airframe.z = 677;
    let mut speed = 75;
    plain.pitch_plane(v, vdx, vdy, 0, &mut speed, false);
    assert_eq!(plain.units[v].airframe.pitch.to_i32(), 40, "no w, it holds");
}

/// **A plane over a wood holds its fire** (`is_in_range@006486b0`'s
/// world-cell test, `6486b0`..`64875b`, item 1200, `docs/GOLDEN.md`
/// §50): the tile under the point asked from answers no when its
/// surface is forest, and the test asks no domain. Chapter forty-one's
/// Biplane on 839, over `TData` `0x7138`. A wood under the target
/// alone does not. Made to fail with the test dropped.
#[test]
fn a_plane_over_a_wood_holds_its_fire() {
    use crate::world::tile;
    let mut s = sim();
    let (u, target, _, _) = striking_fighter(&mut s, true);
    let to = s.buildings[target].pos;
    s.units[u].pos = Pos::new(to.x - 600, to.y);
    let me = Obj::Unit(u);
    assert!(s.is_in_range(me, Obj::Building(target)), "in reach");
    s.world
        .set_tile_field(to.tile(), tile::SURFACE, tile::SURFACE_FOREST);
    assert!(
        s.is_in_range(me, Obj::Building(target)),
        "a wood under the target"
    );
    let under = s.units[u].pos.tile();
    s.world
        .set_tile_field(under, tile::SURFACE, tile::SURFACE_FOREST);
    assert!(
        !s.is_in_range(me, Obj::Building(target)),
        "a wood under the plane"
    );
}

/// **A dry strike with a patrol behind it dies in the tail** (`do_strafe`
/// `5eb642`..`5eb673`, item 1200): the tank at `mana_burn` = `MANA`
/// turns the strike for home (`check_fuel`'s arm, in the flight), and
/// the tail kills a dry strike and one flying home when an order stands
/// behind it, so the patrol flies alone — chapter forty-one's Biplane
/// on 1108. Made to fail with the tail's early return for a strike
/// flying home back and its dry test dropped.
#[test]
fn a_dry_strike_with_a_patrol_behind_it_dies_in_the_tail() {
    let mut s = sim();
    let (base, target) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.come_out(u);
    let point = s.buildings[target].pos;
    s.add_air_patrol_order(u, point, Some(base), false);
    s.add_strafe_order(
        u,
        Some(Obj::Building(target)),
        Some(base),
        false,
        crate::orders::QueuePos::First,
        false,
    );
    assert_eq!(s.units[u].orders.len(), 2);
    assert!(matches!(
        s.units[u].orders.front().map(|o| o.body),
        Some(Body::Strafe(_))
    ));
    s.units[u].mana_burn = 400;
    s.work(u, 767);
    assert_eq!(s.units[u].orders.len(), 1, "the strike is killed");
    assert!(matches!(
        s.units[u].orders.front().map(|o| o.body),
        Some(Body::AirPatrol(_))
    ));
}

/// **A strafer's round never rolls** (`Ammo::init`, `67c548`..`67c557`,
/// item 1200): the unit-strafer arm jumps past `67c633`, where flag `4`
/// and the `0x4b` over a land unit are set, so its round at a land
/// unit lands where it lands — chapter forty-one's Biplane on 1529,
/// short of `0/9`. A type without `w` rolls. Made to fail with the
/// strafer test dropped from `rolling`.
#[test]
fn a_strafers_round_at_a_land_unit_does_not_roll() {
    for strafes in [true, false] {
        let mut s = sim();
        let (u, _, _, _) = striking_fighter(&mut s, strafes);
        let foot = s.add_unit_type(UnitType {
            hits: 100,
            combat: combat::Profile {
                domain: Domain::Land,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        });
        let index = i16::try_from(s.units.len()).unwrap();
        let mut t = Unit::new(1, index, Pos::new(21120, 16800), 100);
        t.ty = Some(foot);
        t.on_map = true;
        t.kind = s.tables.unit_types[foot].kind;
        let t = s.add_unit(t);
        let at = s.units[u].pos;
        s.fire_ammo_pub(
            Obj::Unit(u),
            Obj::Unit(t),
            Angle(0),
            1529,
            at,
            1231,
            0,
            false,
            false,
        );
        let p = s.combat.projectiles.last().unwrap();
        assert_eq!(p.rolling, !strafes, "strafes {strafes}");
    }
}

/// **A strafer's round is exact** (`Ammo::init@0067bbf0`, `0x67c33a`):
/// no scatter, so neither of the two draws a land shot spends. run265's
/// 923: the original's Fighter plays its attack and spends nothing
/// else. Made to fail with `exact` false at the call.
#[test]
fn a_strafers_round_spends_no_draw() {
    let mut s = sim();
    let (u, target, _, _) = striking_fighter(&mut s, true);
    let ty = s.units[u].ty.unwrap();
    s.tables.unit_types[ty].combat.to_hit = 50;
    s.units[u].kind = s.tables.unit_types[ty].kind;
    let before = s.rng;
    let at = s.units[u].pos;
    let n = s.combat.projectiles.len();
    s.fire_ammo_pub(
        Obj::Unit(u),
        Obj::Building(target),
        Angle(0),
        923,
        at,
        870,
        0,
        false,
        false,
    );
    assert_eq!(s.combat.projectiles.len(), n + 1, "a round");
    assert_eq!(s.rng, before, "and no draw");
}

/// **A strafer's landing walks with its gun** (`Ammo::init`,
/// `0x67c9b2`–`0x67cb1a`, item 853): run265's first pair, released on
/// `cur_time` 1 of 30 at heading 1340473344 on the Barracks at
/// (21120, 16512), lands on (20816, 16440) from node 0 and
/// (20853, 16351) from node 1 — `trunc((1/30 − 0.3f)·6·192)` = −307
/// along the heading, then 48 to either side. Neither a figure out of
/// its attack nor a type that does not strafe is walked. Made to fail
/// with the node's sides swapped, and with the walk returning early.
#[test]
fn a_strafers_landing_walks_with_its_gun() {
    let fire = |s: &mut Sim, u: usize, target: usize, node: i8| {
        let at = s.units[u].pos;
        s.fire_ammo_pub(
            Obj::Unit(u),
            Obj::Building(target),
            Angle(0),
            923,
            at,
            873,
            node,
            false,
            false,
        );
        s.combat.projectiles.last().unwrap().landing
    };
    let mut s = sim();
    let (u, target, _, _) = striking_fighter(&mut s, true);
    s.units[u].movement.heading = Angle(1_340_473_344);
    let g = &mut s.units[u].guys[0];
    (g.anim, g.cur_time, g.end_time) = (crate::units::anim::ATTACK2, 1, 30);
    assert_eq!(fire(&mut s, u, target, 0), Pos::new(20816, 16440), "node 0");
    assert_eq!(fire(&mut s, u, target, 1), Pos::new(20853, 16351), "node 1");
    let (me, it, at) = (
        Obj::Unit(u),
        Some(Obj::Building(target)),
        Pos::new(21120, 16512),
    );
    s.units[u].guys[0].cur_time = 30;
    assert_eq!(
        s.strafe_walk(me, it, at, 0),
        Pos::new(21847, 16865),
        "806 past, at the swing's end"
    );
    s.units[u].guys[0].anim = crate::units::anim::WALK;
    assert_eq!(s.strafe_walk(me, it, at, 0), at, "out of its attack");
    let mut plain = sim();
    let (v, target, _, _) = striking_fighter(&mut plain, false);
    plain.units[v].movement.heading = Angle(1_340_473_344);
    let g = &mut plain.units[v].guys[0];
    (g.anim, g.cur_time, g.end_time) = (crate::units::anim::ATTACK2, 1, 30);
    assert_eq!(
        plain.strafe_walk(Obj::Unit(v), Some(Obj::Building(target)), at, 0),
        at,
        "no w, no walk"
    );
}

/// **An empty base counts nothing** (run265: `launch_frames` 0 from
/// 778 on): the whole of `do_launch` is under `inside_down ≥ 0`. Made
/// to fail with the counter above the test.
#[test]
fn an_empty_hangar_s_launch_counter_stands() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    for _ in 0..20 {
        s.do_launch(base);
    }
    assert_eq!(s.buildings[base].launch_frames, 0);
}

/// **The empty tank turns a strike for home** (`Unit::check_fuel@
/// 005e9be0`'s first arm; run265's 1178, run223's 1212): `returning`
/// set on the order flown. Made to fail without the arm.
#[test]
fn an_empty_tank_turns_a_strike_for_home() {
    let mut s = sim();
    let (base, target) = base_and_target(&mut s);
    let u = fighter_inside(&mut s, base, 400);
    s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
    s.come_out(u);
    s.units[u].mana_burn = 400;
    s.plane_air_physics(u, Some(Pos::new(21120, 16512)), 1178);
    let Some(Body::Strafe(sf)) = s.units[u].orders.front().map(|o| o.body) else {
        panic!("a strafe")
    };
    assert!(sf.returning);
}

/// A Fighter inside `base` on the Biplane line (`is(BIPLANE)`), tank
/// full.
fn biplane_inside(s: &mut Sim, base: usize) -> usize {
    let u = fighter_inside(s, base, 400);
    let ty = s.units[u].ty.unwrap();
    s.tables.unit_types[ty].tree = Some(crate::air::airbase::BIPLANE);
    s.units[u].mana_burn = 0;
    u
}

/// **A blow on a base scrambles its last full Biplane** (item 3990,
/// `Object::attempt_launch@00643a10`; run1548's 795–796: `0/6` takes
/// one `AIRPATROLORDER` at the hoplite's point). Under
/// `FRAMES_BETWEEN_LAUNCHES` nothing; a tank not full is passed over;
/// an occupant with an order and a full tank ends the walk. Made to
/// fail with the launch-frames gate dropped (the first assertion), with
/// the first candidate kept (the second) and with the walk's return
/// dropped (the last).
#[test]
fn a_blow_on_a_base_sends_its_last_full_biplane_at_the_striker() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let a = biplane_inside(&mut s, base);
    let b = biplane_inside(&mut s, base);
    let c = biplane_inside(&mut s, base);
    s.units[c].mana_burn = 1;
    let at = Pos::new(12408, 14712);
    s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES - 1;
    s.attempt_launch(base, at);
    assert!(
        [a, b, c].iter().all(|&u| s.units[u].orders.is_empty()),
        "too soon after a launch"
    );
    s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
    s.attempt_launch(base, at);
    assert!(s.units[a].orders.is_empty(), "not the first full one");
    assert!(s.units[c].orders.is_empty(), "nor the one refuelling");
    let o = s.units[b].orders.front().copied().expect("a patrol");
    assert_eq!(o.flags & flag::ACTION, flag::ACTION, "the action bit");
    let Body::AirPatrol(p) = o.body else {
        panic!("an air patrol")
    };
    assert_eq!((p.points[0], p.home), (at, Some(base)));
    assert_eq!(s.units[b].inside, Some(base), "out on the next launch");
    s.units[a].mana_burn = 0;
    s.units[c].mana_burn = 0;
    s.units[b].orders.clear();
    s.add_air_patrol_order(a, at, Some(base), true);
    s.attempt_launch(base, at);
    assert!(
        s.units[b].orders.is_empty() && s.units[c].orders.is_empty(),
        "a full occupant with an order ends the walk"
    );
}

/// A Fighter out on a patrol from `base`, its tank empty: the flight
/// home.
fn returning_from(s: &mut Sim, base: usize, at: Pos) -> usize {
    let u = fighter_inside(s, base, 400);
    s.come_out(u);
    s.units[u].pos = at;
    s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
    if let Some(Body::AirPatrol(p)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
        p.returning = true;
    }
    s.units[u].mana_burn = 400;
    u
}

fn patrol_home(s: &Sim, u: usize) -> Option<usize> {
    match s.units[u].orders.front().map(|o| o.body) {
        Some(Body::AirPatrol(p)) => p.home,
        _ => None,
    }
}

/// **A plane whose home is razed re-homes to the nearest base, and
/// with none dies** (item 3990, `Unit::check_fuel@005e9be0:60`–`:136`;
/// run1548: the three planes' `oxx` 2008 by 1216, all gone on 1231).
/// Of two live Airbases the nearer is written into the order; a base
/// of the other side is never one. Made to fail with `rehome` answering
/// none (the first assertion), with the first base kept over the
/// nearer (the same), and with `out_of_fuel` dropped (the last).
#[test]
fn a_plane_whose_home_is_razed_flies_to_the_nearest_base_or_dies() {
    let mut s = sim();
    let (base, _) = base_and_target(&mut s);
    let airbase = s.buildings[base].ty;
    let far = s.add_building(0, Pos::new(11616, 30000), 0);
    s.buildings[far].ty = airbase;
    let near = s.add_building(0, Pos::new(11616, 21000), 0);
    s.buildings[near].ty = airbase;
    let theirs = s.add_building(1, Pos::new(11616, 18000), 0);
    s.buildings[theirs].ty = airbase;
    let u = returning_from(&mut s, base, Pos::new(11616, 17000));
    s.buildings[base].alive = false;
    s.plane_air_physics(u, None, 1201);
    assert_eq!(patrol_home(&s, u), Some(near), "the nearest of its own");
    assert!(s.obj_alive(Obj::Unit(u)), "flying on");
    s.buildings[near].alive = false;
    s.buildings[far].alive = false;
    s.plane_air_physics(u, None, 1202);
    assert!(!s.obj_alive(Obj::Unit(u)), "no base: it dies");
}
