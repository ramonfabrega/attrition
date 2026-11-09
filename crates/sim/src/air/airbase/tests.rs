use super::*;
use crate::attrition::Domain;
use crate::combat;
use crate::{Unit, UnitType};

fn sim() -> Sim {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    s.nation[0].human = true;
    s.at_war[0][1] = true;
    s.at_war[1][0] = true;
    s
}

/// A plane of `line` (the tree's type id) with a 450 tank, standing in
/// `base` behind those already there.
fn plane(s: &mut Sim, base: usize, line: crate::tech::TypeId) -> usize {
    let t = UnitType {
        hits: 100,
        moves: 75,
        mana: 450,
        kind: crate::attrition::UnitKind {
            domain: Domain::Air,
            ..crate::attrition::UnitKind::default()
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
    // The line only: `air_line_is` reads the id itself before the tree.
    s.unit_types[ty].tree = Some(line);
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(0, index, s.buildings[base].pos, 100);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    s.units[u].kind = s.unit_types[ty].kind;
    s.units[u].movement.speed = 75;
    s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
    s.init_guys(u, Some(ty));
    s.go_inside(u, base);
    u
}

/// Chapter thirty-two's hangar after its Clear: a who=0 Airbase with
/// the Bomber `0/8`, the Bomber `0/7`, the Fighter `0/6` and the
/// Biplane `0/9` in the order they landed, their tanks as given.
fn hangar(burn: [i16; 4]) -> (Sim, usize, [usize; 4]) {
    let mut s = sim();
    let airbase = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Airbase,
        flags: crate::build::flags::TRAINS,
        ..crate::build::BuildType::default()
    });
    let b = s.add_building(0, Pos::new(11616, 13920), 0);
    s.buildings[b].ty = Some(airbase);
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    let planes = [
        plane(&mut s, b, crate::air::BOMBER),
        plane(&mut s, b, crate::air::BOMBER),
        plane(&mut s, b, BIPLANE),
        plane(&mut s, b, BIPLANE),
    ];
    for (i, &u) in planes.iter().enumerate() {
        s.units[u].mana_burn = burn[i];
    }
    (s, b, planes)
}

fn patrol_of(s: &Sim, u: usize) -> Option<(Pos, u8)> {
    match s.units[u].orders.front() {
        Some(o) => match o.body {
            crate::orders::Body::AirPatrol(ref ap) => Some((ap.live()[0], o.flags)),
            _ => None,
        },
        None => None,
    }
}

/// **The launch patrol's choice** (`docs/GOLDEN.md` §42, the
/// emulator's rows): the Biplane line's ÷ 10 puts a fighter ahead of
/// the bombers before it in the chain; a refuelling plane is passed
/// over without shift; shift takes every plane, fuel or none.
#[test]
fn the_launch_patrol_takes_the_first_fuelled_fighter_and_shift_takes_every_plane() {
    let p = Pos::new(11520, 7680);
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
    s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
    assert_eq!(patrol_of(&s, f6), Some((p, crate::orders::flag::ACTION)));
    assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
    s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
    assert_eq!(
        patrol_of(&s, f9).map(|x| x.0),
        Some(p),
        "the fuelled fighter"
    );
    assert!([b8, b7, f6].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, planes) = hangar([40, 12, 0, 4]);
    let shift = Keys {
        shift: true,
        ..Keys::default()
    };
    s.group_action_launch_patrol(&[b], p, QueuePos::Last, shift);
    assert!(planes.iter().all(|&u| patrol_of(&s, u).is_some()));

    let (mut s, b, planes) = hangar([4, 4, 4, 4]);
    s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
    assert!(planes.iter().all(|&u| s.units[u].orders.is_empty()));
}

/// who=1's Barracks, finished and seen by who=0, at `at`.
fn enemy_barracks(s: &mut Sim, at: Pos) -> usize {
    let t = s.add_building(1, at, 0);
    s.buildings[t].started = true;
    s.buildings[t].active = true;
    s.buildings[t].combat = Some(combat::Profile::default());
    s.buildings[t].health = 1200;
    s.buildings[t].ever_seen = 3;
    t
}

/// A who=0 unit standing on the ground at `at`.
fn friend(s: &mut Sim, at: Pos) -> usize {
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(0, index, at, 40);
    u.on_map = true;
    s.add_unit(u)
}

fn strike_of(s: &Sim, u: usize) -> Option<(Option<Obj>, u8, bool)> {
    match s.units[u].orders.front() {
        Some(o) => match o.body {
            crate::orders::Body::Strafe(ref sf) => Some((sf.target, o.flags, sf.mandatory)),
            _ => None,
        },
        None => None,
    }
}

/// **The scramble** (item 3912, `Group::action_scramble@007111c0`,
/// `docs/GOLDEN.md` §159): every full plane inside goes up over the
/// base's own point with the action bit; one still refuelling stays,
/// and a plane on the map is not a passenger. Made to fail with the
/// tank gate dropped and with the point taken off the plane.
#[test]
fn the_scramble_sends_every_full_plane_inside_up_over_its_base() {
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 7, 0, 0]);
    s.come_out(f9);
    let base = s.buildings[b].pos;
    s.group_action_scramble(&[b]);
    for u in [b8, f6] {
        assert_eq!(
            patrol_of(&s, u),
            Some((base, crate::orders::flag::ACTION)),
            "0/{}",
            s.units[u].index
        );
        assert_eq!(s.units[u].orders.len(), 1);
    }
    assert!(s.units[b7].orders.is_empty(), "refuelling: passed over");
    assert!(s.units[f9].orders.is_empty(), "on the map: not inside");
}

/// **The recall on an Airbase** (item 3912, `Group::action_recall@
/// 006fa7e0`'s building arm, `docs/GOLDEN.md` §159): every homed plane
/// in flight — on a strike or a patrol — is sent home, one strafe with
/// no target, `returning`, `mandatory` 0, no action bit, at the strafe's
/// own heights; a plane inside loses its orders; a plane homed at
/// another base is not touched. Made to fail with the arm dropped.
#[test]
fn the_recall_sends_every_plane_homed_at_the_base_home() {
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
    let other = second_base(&mut s, b, Pos::new(8544, 13920));
    let target = enemy_barracks(&mut s, Pos::new(13824, 14976));
    s.come_out(b8);
    s.add_strafe_order(
        b8,
        Some(Obj::Building(target)),
        Some(b),
        true,
        QueuePos::New,
        true,
    );
    s.come_out(f9);
    s.add_air_patrol_order(f9, Pos::new(13440, 9600), Some(b), true);
    s.set_cruising_alt(f9, 1800);
    s.come_out(f6);
    s.add_air_patrol_order(f6, Pos::new(13440, 9600), Some(other), true);
    s.add_air_patrol_order(b7, Pos::new(13440, 9600), Some(b), true);
    s.group_action_recall_buildings(0, &[b]);
    for u in [b8, f9] {
        assert_eq!(
            strike_of(&s, u),
            Some((None, 0, false)),
            "0/{}",
            s.units[u].index
        );
        let a = s.current_air(u).expect("an air order");
        assert_eq!((a.home, a.returning), (Some(b), true));
        assert_eq!(a.cruising_alt, 0x640, "the strafe home's own heights");
        assert_eq!(s.units[u].orders.len(), 1);
    }
    assert!(s.units[b7].orders.is_empty(), "inside: cleared");
    assert!(patrol_of(&s, f6).is_some(), "homed at another base");
}

/// **The launch strike's choice** (`docs/GOLDEN.md` §42, the
/// emulator's rows): on an enemy building the Bomber line's ÷ 4 puts
/// the first Bomber ahead of the fighters; with both Bombers
/// refuelling the first fuelled fighter goes; shift sends all four.
/// One plane takes the strike from inside, the others nothing.
#[test]
fn the_launch_strike_takes_the_first_bomber_on_a_building() {
    let target = Pos::new(13920, 15072);
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
    let t = Obj::Building(enemy_barracks(&mut s, target));
    s.group_action_launch_flight(0, &[b], t, Flight::Strike, Keys::default());
    assert_eq!(
        strike_of(&s, b8),
        Some((Some(t), crate::orders::flag::ACTION, true))
    );
    assert!([b7, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, [b8, b7, f6, f9]) = hangar([40, 40, 0, 0]);
    let t = Obj::Building(enemy_barracks(&mut s, target));
    s.group_action_launch_flight(0, &[b], t, Flight::Strike, Keys::default());
    assert!(strike_of(&s, f6).is_some(), "the first fuelled fighter");
    assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, planes) = hangar([0, 0, 0, 0]);
    let t = Obj::Building(enemy_barracks(&mut s, target));
    let shift = Keys {
        shift: true,
        ..Keys::default()
    };
    s.group_action_launch_flight(0, &[b], t, Flight::Strike, shift);
    assert!(planes.iter().all(|&u| strike_of(&s, u).is_some()));
}

/// **Action 3 at an Airbase, and a point after it** (§42): every homed
/// plane a `mandatory` strike on the friendly unit, flags 0, and the
/// one `action_launch_flight` picks inside — the Biplane line's ÷ 4 on
/// a unit of one's own — re-pointed with the action bit; then a
/// ground point appended is a fresh patrol that ends every strike.
#[test]
fn action_three_at_an_airbase_strikes_every_plane_and_a_later_point_ends_it() {
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
    let c = friend(&mut s, Pos::new(4104, 28392));
    let at = Pos::new(i32::from(s.units[c].index), 0);
    s.action_gather_point(0, &[b], at, 3, false);
    let u = Obj::Unit(c);
    assert_eq!(
        strike_of(&s, f6),
        Some((Some(u), crate::orders::flag::ACTION, true))
    );
    for p in [b8, b7, f9] {
        assert_eq!(strike_of(&s, p), Some((Some(u), 0, true)), "plane {p}");
    }
    let p3 = Pos::new(9600, 7680);
    s.action_gather_point(0, &[b], p3, 0, true);
    assert_eq!(s.buildings[b].gather.len(), 2);
    for p in [b8, b7, f6, f9] {
        assert_eq!(s.units[p].orders.len(), 1, "plane {p}");
        assert_eq!(
            patrol_of(&s, p),
            Some((p3, crate::orders::flag::ACTION)),
            "plane {p}"
        );
    }
}

/// **`[P1, A3, P2]`** (item 1009, `docs/GOLDEN.md` §43, run362 on
/// 2322, 2337 and 2352): a ground point is a patrol for every homed
/// plane; A3 behind it a strike that closes the patrol; and P2 behind
/// that goes into the walk's dead patrol, so the strike stands, alone.
#[test]
fn a_point_behind_an_action_three_point_behind_a_patrol_leaves_the_strike() {
    let (mut s, b, planes) = hangar([0, 0, 0, 0]);
    let c = friend(&mut s, Pos::new(4104, 28392));
    let p1 = Pos::new(11520, 7680);
    s.action_gather_point(0, &[b], p1, 0, false);
    assert!(
        planes
            .iter()
            .all(|&u| patrol_of(&s, u).map(|x| x.0) == Some(p1))
    );
    let a3 = Pos::new(i32::from(s.units[c].index), 0);
    s.action_gather_point(0, &[b], a3, 3, true);
    let p2 = Pos::new(7680, 11520);
    s.action_gather_point(0, &[b], p2, 0, true);
    assert_eq!(s.buildings[b].gather.len(), 3);
    for p in planes {
        assert_eq!(s.units[p].orders.len(), 1, "plane {p}");
        assert_eq!(
            strike_of(&s, p).map(|x| x.0),
            Some(Some(Obj::Unit(c))),
            "plane {p}"
        );
    }
}

/// **The escort** (`Unit::do_strafe@005eab00`'s ally arm, §42): a
/// strike on one's own unit is flown at the unit, its `xx/yy`
/// following it; this crate's `return` for an ally left the plane where
/// it was and its point stale.
#[test]
fn a_strike_on_one_s_own_unit_is_flown_at_the_unit() {
    let (mut s, b, [_, _, f6, _]) = hangar([0, 0, 0, 0]);
    let c = friend(&mut s, Pos::new(4104, 28392));
    s.come_out(f6);
    s.add_strafe_order(f6, Some(Obj::Unit(c)), Some(b), true, QueuePos::New, true);
    s.units[c].pos = Pos::new(4296, 28392);
    // How far the heading is off the bearing to the unit.
    let off = |s: &Sim| {
        let p = s.units[f6].pos;
        let bearing = crate::movement::find_angle(4296 - p.x, 28392 - p.y);
        let d = (s.units[f6].movement.heading.0 as u32).wrapping_sub(bearing.0 as u32);
        if d > 0x8000_0000 { !d } else { d }
    };
    let off0 = off(&s);
    for f in 2308..2368 {
        s.work(f6, f);
    }
    let Some(crate::orders::Order {
        body: crate::orders::Body::Strafe(sf),
        ..
    }) = s.units[f6].orders.front().copied()
    else {
        panic!("the escort stands");
    };
    assert_eq!(sf.at, Some(Pos::new(4296, 28392)), "xx/yy follow the unit");
    assert!(
        off(&s) < off0,
        "turned at it: {:#x} from {off0:#x}",
        off(&s)
    );
}

/// **ctrl and alt** (item 1009, `docs/GOLDEN.md` §43, the emulator's
/// rows): on the patrol ctrl keeps the Bomber line, so the first Bomber
/// goes where the Fighter's ÷ 10 would have won; on the strike alt keeps
/// the Biplane line, so the first fuelled fighter goes where the Bomber
/// line's ÷ 4 would have; ctrl on the strike with only a fighter full
/// sends nothing; ctrl and alt together are alt's.
#[test]
fn ctrl_keeps_the_bombers_and_alt_the_fighters_on_both_launch_commands() {
    let p = Pos::new(13440, 9600);
    let ctrl = Keys {
        ctrl: true,
        ..Keys::default()
    };
    let alt = Keys {
        alt: true,
        ..Keys::default()
    };
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
    s.group_action_launch_patrol(&[b], p, QueuePos::New, ctrl);
    assert_eq!(patrol_of(&s, b8), Some((p, crate::orders::flag::ACTION)));
    assert!([b7, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
    s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys { ctrl: true, ..alt });
    assert!(patrol_of(&s, f6).is_some(), "alt is asked first");
    assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let target = Pos::new(13920, 15072);
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
    let t = Obj::Building(enemy_barracks(&mut s, target));
    s.group_action_launch_flight(0, &[b], t, Flight::Strike, alt);
    assert!(strike_of(&s, f9).is_some(), "the fuelled fighter");
    assert!([b8, b7, f6].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, planes) = hangar([40, 40, 0, 40]);
    let t = Obj::Building(enemy_barracks(&mut s, target));
    s.group_action_launch_flight(0, &[b], t, Flight::Strike, ctrl);
    assert!(planes.iter().all(|&u| s.units[u].orders.is_empty()));
}

/// A second who=0 Airbase at `at`, finished.
fn second_base(s: &mut Sim, first: usize, at: Pos) -> usize {
    let ty = s.buildings[first].ty;
    let b = s.add_building(0, at, 0);
    s.buildings[b].ty = ty;
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    b
}

/// **`Build::finished`'s aircraft arm** (item 1714, `docs/AI.md` §186):
/// an Airbase training a plane answers -1 at ten aircraft homed to it,
/// and not at nine, not for a type that flies like a helicopter, not at
/// another building type.
#[test]
fn an_airbase_with_ten_homed_aircraft_refuses_another() {
    let (mut s, b, planes) = hangar([0, 0, 0, 0]);
    let ty = s.units[planes[0]].ty.unwrap();
    for _ in 0..5 {
        plane(&mut s, b, BIPLANE);
    }
    assert_eq!(s.aircraft_homed(b), 9);
    assert!(!s.airbase_pad_full(b, ty), "nine homed: room");
    plane(&mut s, b, BIPLANE);
    assert_eq!(s.aircraft_homed(b), 10);
    assert!(s.airbase_pad_full(b, ty), "ten homed: full");
    s.unit_types[ty].cols.unit_flags |= crate::ai_load::uflags::FLAG_HELICOPTER;
    assert!(!s.airbase_pad_full(b, ty), "a helicopter's type is exempt");
}

/// **`Build::do_queue`'s second fallback** (item 1905, `docs/AI.md`
/// §198, run790's `1/2011` on 3767): a head the aircraft limit refuses
/// (a plane at ten homed) lets the first Helicopter behind it train;
/// a head the population cap refuses (zero, not negative) does not.
#[test]
fn a_full_airbase_trains_the_helicopter_behind_a_refused_plane() {
    let build = |cap: i32| {
        let (mut s, b, planes) = hangar([0, 0, 0, 0]);
        for _ in 0..6 {
            plane(&mut s, b, BIPLANE);
        }
        let plane_ty = s.units[planes[2]].ty.unwrap();
        let h = plane(&mut s, b, HELICOPTER);
        let heli_ty = s.units[h].ty.unwrap();
        s.unit_types[heli_ty].cols.unit_flags |= crate::ai_load::uflags::FLAG_HELICOPTER;
        // No technology tree in this fixture: the lines are ids a
        // tree would index.
        s.unit_types[heli_ty].tree = None;
        s.unit_types[plane_ty].tree = None;
        assert!(s.airbase_pad_full(b, plane_ty));
        s.muster[0].researched[plane_ty] = true;
        s.muster[0].researched[heli_ty] = true;
        s.muster[0].cap = cap;
        let c = [0; crate::economy::RESOURCES];
        for ty in [plane_ty, heli_ty] {
            s.buildings[b]
                .queue
                .items
                .push(crate::production::Item::queued(ty, &c));
            s.buildings[b].queue.items.last_mut().unwrap().job_counter = 1_000_000;
        }
        (s, b, heli_ty)
    };
    let (mut s, b, heli_ty) = build(1000);
    let out = s.process_queues();
    assert_eq!(
        out.len(),
        1,
        "the helicopter trains behind the refused head"
    );
    assert_eq!(s.buildings[b].queue.items.len(), 1, "the plane stays");
    assert_ne!(s.buildings[b].queue.items[0].ty, heli_ty);
    let (mut s, b, _) = build(-1000);
    assert!(
        s.process_queues().is_empty(),
        "a population refusal is zero"
    );
    assert_eq!(s.buildings[b].queue.items.len(), 2);
}

/// **`MOVE_TO` onto another base** (item 1009, §43): the first fuelled
/// plane of equals — the farthest, strict — flies home to it; a
/// refuelling plane is passed over; shift sends every one; a base
/// with no room takes none — which `group_action_flight`'s own room
/// test would refuse too, so `can_carry` dropped fails nothing here.
#[test]
fn a_right_click_on_another_base_sends_the_first_fuelled_plane_to_it() {
    let to = Pos::new(8544, 14688);
    let (mut s, b, [b8, b7, f6, f9]) = hangar([40, 0, 0, 0]);
    let b2 = second_base(&mut s, b, to);
    let t = Obj::Building(b2);
    s.group_action_launch_flight(0, &[b], t, Flight::Home, Keys::default());
    let home = |s: &Sim, u: usize| match s.units[u].orders.front() {
        Some(o) => match o.body {
            crate::orders::Body::Strafe(ref sf) => Some((sf.home, sf.returning, o.flags)),
            _ => None,
        },
        None => None,
    };
    assert_eq!(
        home(&s, b7),
        Some((Some(b2), true, crate::orders::flag::ACTION)),
        "0/7, the first full"
    );
    assert!([b8, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

    let (mut s, b, planes) = hangar([40, 0, 0, 0]);
    let b2 = second_base(&mut s, b, to);
    let shift = Keys {
        shift: true,
        ..Keys::default()
    };
    s.group_action_launch_flight(0, &[b], Obj::Building(b2), Flight::Home, shift);
    assert!(planes.iter().all(|&u| home(&s, u).is_some()), "shift: all");

    let (mut s, b, planes) = hangar([0, 0, 0, 0]);
    let b2 = second_base(&mut s, b, to);
    for _ in 0..10 {
        let u = plane(&mut s, b2, BIPLANE);
        assert_eq!(s.units[u].inside, Some(b2));
    }
    s.group_action_launch_flight(0, &[b], Obj::Building(b2), Flight::Home, Keys::default());
    assert!(
        planes.iter().all(|&u| s.units[u].orders.is_empty()),
        "no room under num_aircraft_limit"
    );
}

/// **The Helicopter's move** (item 1009, §43, the emulator's row; no
/// capture holds one): a type that flies like a helicopter takes a
/// `MOVE_TO` to the point's cell centre, facing from itself to the
/// point as clicked, with the action bit — and wins over the Bombers
/// by the Helicopter line's ÷ 4.
#[test]
fn a_helicopter_launched_on_the_ground_takes_a_move_to_the_cell_centre() {
    let p = Pos::new(13440, 9600);
    let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 40]);
    let h = plane(&mut s, b, HELICOPTER);
    let t = s.units[h].ty.unwrap();
    s.unit_types[t].cols.unit_flags |= crate::ai_load::uflags::FLAG_HELICOPTER;
    let here = s.units[h].pos;
    s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
    assert!(
        [b8, b7, f6, f9]
            .iter()
            .all(|&u| s.units[u].orders.is_empty())
    );
    let o = s.units[h].orders.front().copied().expect("the move");
    assert_eq!(
        o.flags & crate::orders::flag::ACTION,
        crate::orders::flag::ACTION
    );
    let crate::orders::Body::Move(m) = o.body else {
        panic!("a MOVE_TO, not {:?}", o.body);
    };
    assert_eq!(m.dest, Pos::new(13464, 9624));
    assert_eq!(
        m.angle,
        crate::movement::find_angle(p.x - here.x, p.y - here.y)
    );
}

/// A type of `line` made a Helicopter (`unit_flags & 0x20`) or a
/// missile (`obj_masks & 0x8000000`), its template standing in `b`.
fn air_type(s: &mut Sim, b: usize, line: crate::tech::TypeId, heli: bool, missile: bool) -> usize {
    // The tree holds the line's id, so `init_unit` can read its entry.
    if s.tech_tree.types.len() <= line {
        let unit = crate::tech::TypeDef::new("", crate::tech::Kind::Unit(Default::default()));
        let mut tree = s.tech_tree.clone();
        tree.types.resize(line + 1, unit);
        s.set_tech_tree(tree);
    }
    let p = plane(s, b, line);
    let t = s.units[p].ty.unwrap();
    if heli {
        s.unit_types[t].cols.unit_flags |= crate::ai_load::uflags::FLAG_HELICOPTER;
    }
    if missile {
        s.unit_types[t].combat.obj_masks |= crate::combat::mask::MISSILE;
    }
    t
}

/// **A Helicopter trained with no gather point comes out at once, and
/// a missile stays inside** (item 1019, `Build::train@0062f9b0`'s
/// `62ff59`..`62ff8f`, the emulator's first row): no limit test on the
/// empty list. Made to fail with the arm dropped (the Helicopter
/// inside).
#[test]
fn a_helicopter_trained_with_no_point_comes_out_and_a_missile_stays() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    s.frame = 2445;
    let heli = air_type(&mut s, b, HELICOPTER, true, false);
    let v2 = air_type(&mut s, b, 0x139, false, true);
    let h = s.build_train(b, heli).unit;
    assert_eq!(s.units[h].inside, None, "the Helicopter out");
    assert!(s.units[h].orders.is_empty(), "with no order");
    let m = s.build_train(b, v2).unit;
    assert_eq!(s.units[m].inside, Some(b), "the missile inside");
    assert!(s.units[m].orders.is_empty(), "with no order");
}

/// **Under a point, a missile or a Helicopter reads the first point
/// alone** (item 1019, the block at `62fc47`; the emulator's rows): on
/// the ground a Helicopter takes the move `add_air_patrol_order` gives
/// its line and a missile nothing; on an enemy building each a strike,
/// mandatory, the action bit; on a base of one's own a Helicopter a
/// flight home to it and a missile nothing. Made to fail with
/// [`Sim::train_first_point`] dropped.
#[test]
fn a_missile_or_a_helicopter_under_a_point_reads_the_first_point_alone() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let heli = air_type(&mut s, b, HELICOPTER, true, false);
    let v2 = air_type(&mut s, b, 0x139, false, true);
    let ground = Pos::new(5760, 12288);
    let point = |pos| crate::rally::GatherPoint { pos, action: 0 };
    s.buildings[b].gather = vec![point(ground), point(Pos::new(7680, 11520))];
    let h = s.build_train(b, heli).unit;
    let o = s.units[h].orders.front().copied().expect("a move");
    let crate::orders::Body::Move(mv) = o.body else {
        panic!("an ATTACK_TO, not {:?}", o.body);
    };
    assert_eq!(mv.kind, crate::orders::MoveKind::AttackTo);
    assert_eq!(
        (mv.dest, o.flags & crate::orders::flag::ACTION),
        (Pos::new(5784, 12312), crate::orders::flag::ACTION)
    );
    assert_eq!(s.units[h].orders.len(), 1, "the first point alone");
    let m = s.build_train(b, v2).unit;
    assert!(
        s.units[m].orders.is_empty(),
        "a missile on the ground: nothing"
    );
    // An enemy building on the point: a strike, for either — and a
    // missile's strike is `add_strafe_order`'s head, an air attack on
    // the building's point (item 1050; the emulator hooked the adder
    // and never ran its head).
    let enemy = s.add_building(1, Pos::new(13824, 14976), 0);
    s.buildings[enemy].started = true;
    s.buildings[enemy].active = true;
    s.buildings[enemy].combat = Some(combat::Profile::default());
    s.buildings[enemy].health = 1200;
    s.buildings[enemy].ever_seen = 3;
    s.buildings[b].gather = vec![point(Pos::new(13824, 14976))];
    let u = s.build_train(b, heli).unit;
    let o = s.units[u].orders.front().copied().expect("a strike");
    let crate::orders::Body::Strafe(sf) = o.body else {
        panic!("a STRAFE, not {:?}", o.body);
    };
    assert_eq!(sf.target, Some(Obj::Building(enemy)));
    assert!(sf.mandatory && o.flags & crate::orders::flag::ACTION != 0);
    let u = s.build_train(b, v2).unit;
    let o = s.units[u].orders.front().copied().expect("a strike");
    let crate::orders::Body::AirAttackGround(g) = o.body else {
        panic!("an AIRATTACKGROUNDORDER, not {:?}", o.body);
    };
    assert_eq!(g.at, Pos::new(13824, 14976));
    assert_eq!(g.home, Some(b));
    assert!(o.flags & crate::orders::flag::ACTION != 0);
    // A base of one's own: a Helicopter flies home to it, a missile stays.
    let other = second_base(&mut s, b, Pos::new(8544, 14688));
    s.buildings[other].combat = Some(combat::Profile::default());
    s.buildings[other].health = 1200;
    // The Airbase's own 4-by-4 footprint, which the search reads.
    let t = s.buildings[other].ty.unwrap();
    (s.build_types[t].x_size, s.build_types[t].y_size) = (4, 4);
    s.buildings[b].gather = vec![point(Pos::new(8544, 14688))];
    let u = s.build_train(b, heli).unit;
    let o = s.units[u].orders.front().copied().expect("a flight home");
    let crate::orders::Body::Strafe(sf) = o.body else {
        panic!("a STRAFE home, not {:?}", o.body);
    };
    assert_eq!(
        (sf.target, sf.home, sf.returning),
        (None, Some(other), true)
    );
    let m = s.build_train(b, v2).unit;
    assert!(s.units[m].orders.is_empty(), "a missile to a base: nothing");
}

/// **A Helicopter's exit spends two draws** (item 1019,
/// `Unit::do_spec_anim@005e5880`, `5e59d5`..`5e5a1c`): `x` less `197 −
/// rand % 11`, `y` plus `rand % 11 − 5`, and 200 over the ground. Made
/// to fail with the arm dropped (no draw, the plane's `x − 0xc0`).
#[test]
fn a_helicopter_s_exit_spends_two_draws_and_hovers_200_up() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    s.frame = 2445;
    air_type(&mut s, b, HELICOPTER, true, false);
    let h = *s.buildings[b].garrison.last().unwrap();
    let mut r = s.rng;
    let (dx, dy) = (r.roll() % 11 - 197, r.roll() % 11 - 5);
    assert!(s.come_out(h));
    let at = s.buildings[b].pos;
    assert_eq!(s.units[h].pos, Pos::new(at.x + dx, at.y + dy));
    let ground = s.ground_z(at);
    assert_eq!(s.units[h].airframe.z, ground + 200);
}

/// A Helicopter of its own type out of `b` and on the map at `at`, with
/// its figure on it and no order.
fn heli_at(s: &mut Sim, b: usize, at: Pos) -> usize {
    // The exit's arm, which faces it north, wants a frame past 0.
    s.frame = s.frame.max(1);
    air_type(s, b, HELICOPTER, true, false);
    let h = *s.buildings[b].garrison.last().unwrap();
    assert!(s.come_out(h));
    s.units[h].orders.clear();
    s.set_new_location(h, at, true);
    h
}

/// **A Helicopter's move is one straight waypoint** (item 1048: the
/// planners' `+0x2b4 & 0x20` returns, `find_path@005fb910`'s `5fb96f`,
/// `find_wpath`'s `689110`, `find_tpath`'s `68990d`): run371's `0/12`
/// out at (8351, 14690) on its attack-move to P_h's cell takes the goal
/// alone, no `path_recursion`, and steps 75 straight at it — the dump's
/// (−55, −51). Made to fail with each of the three returns dropped.
#[test]
fn a_helicopter_flies_one_straight_waypoint() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let h = heli_at(&mut s, b, Pos::new(8351, 14690));
    let goal = Pos::new(5784, 12312);
    s.add_air_patrol_order(h, goal, None, true);
    // Faced along the bearing already: this fixture's type turns
    // slowly, where the install's Helicopter took it in one frame.
    let at = s.units[h].pos;
    let bearing = crate::movement::find_angle(goal.x - at.x, goal.y - at.y);
    s.units[h].movement.set_facing(bearing);
    s.frame = 2674;
    s.work(h, 2674);
    let path: Vec<Pos> = s.units[h].path.iter().map(|p| p.to).collect();
    assert_eq!(path, vec![goal], "the goal alone");
    assert_eq!(s.units[h].path_recursion, 0);
    assert_eq!(s.units[h].pos, Pos::new(8296, 14639), "75 straight at it");
}

/// **Two Helicopters are set 48 apart by the one working** (item 1048,
/// `Unit::work@0060d180`'s tail, `60dadd`..`60dc9f`): run371's `0/12`
/// at (8296, 14639) and `0/11` idle at (8355, 14690) on 2674 — the one
/// working moves half the vector, the other the whole of it, each with
/// its figure put on it: (8278, 14624) and (8392, 14721), block 2675's
/// own. Nothing moves while the other is inside. Made to fail with the
/// arm dropped.
#[test]
fn two_helicopters_are_set_48_apart_by_the_one_working() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let near = heli_at(&mut s, b, Pos::new(8355, 14690));
    s.go_inside(near, b);
    let me = heli_at(&mut s, b, Pos::new(8296, 14639));
    s.frame = 2674;
    s.work(me, 2674);
    assert_eq!(s.units[me].pos, Pos::new(8296, 14639), "the other inside");
    assert!(s.come_out(near));
    s.units[near].orders.clear();
    s.set_new_location(near, Pos::new(8355, 14690), true);
    s.work(me, 2674);
    assert_eq!(s.units[me].pos, Pos::new(8278, 14624));
    assert_eq!(s.units[near].pos, Pos::new(8392, 14721));
    for u in [me, near] {
        assert_eq!(s.units[u].movement.body.pos, s.units[u].pos, "put on it");
    }
}

/// **The tile planner leaves a Helicopter's goal as it is** (item 1048,
/// `PathFinder::find_tpath@006897d0`'s `68990d`): a goal two cells off
/// comes back alone, tolerance 0. `find_wpath` answers first on every
/// fresh move, so this arm is reached only by a caller that asks the
/// tile grid directly. Made to fail with the arm dropped.
#[test]
fn the_tile_planner_leaves_a_helicopter_s_goal_as_it_is() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let h = heli_at(&mut s, b, Pos::new(8351, 14690));
    let goal = Pos::new(5784, 12312);
    s.units[h].path.push(crate::orders::PathData {
        to: goal,
        tolerance: 384,
        flags: crate::orders::path_flag::FINAL,
    });
    assert_eq!(s.find_tpath(h), 1);
    let top = s.units[h].path[0];
    assert_eq!((top.to, top.tolerance), (goal, 0));
}

/// **A Helicopter on an air order is not set apart** (item 1048, the
/// separation's `is_air` gate, `UnitOrder` vslot `+0x30` at `60db14`):
/// the pair of run371's 2674, the working one on a strike. No capture
/// has the case. Made to fail with the gate dropped.
#[test]
fn a_helicopter_on_an_air_order_is_not_set_apart() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let near = heli_at(&mut s, b, Pos::new(8355, 14690));
    let me = heli_at(&mut s, b, Pos::new(8296, 14639));
    s.add_strafe_order(
        me,
        None,
        Some(b),
        false,
        crate::orders::QueuePos::New,
        false,
    );
    s.helicopter_spread(me);
    assert_eq!(s.units[me].pos, Pos::new(8296, 14639));
    assert_eq!(s.units[near].pos, Pos::new(8355, 14690));
}

/// **A Helicopter's figure climbs thirty a move toward 1000 over the
/// ground** (item 1048, `Guy::set_new_location@005d86f0`'s
/// `5d880b`..`5d884a`): by a teleport, which leaves `last_z` on it, and
/// by a step of the figure, which leaves `last_z` where it stood —
/// run371's `0/12` on 2677, `z` 308 over `last_z` 278. Made to fail
/// with either call dropped.
#[test]
fn a_helicopter_s_figure_climbs_thirty_a_move() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let at = Pos::new(8351, 14690);
    let h = heli_at(&mut s, b, at);
    let ground = s.ground_z(at);
    s.units[h].airframe.z = ground + 158;
    s.set_new_location(h, at, true);
    assert_eq!(s.units[h].airframe.z, ground + 188);
    assert_eq!(s.units[h].airframe.last_z, ground + 188);
    s.units[h].airframe.z = ground + 990;
    s.set_new_location(h, at, true);
    assert_eq!(s.units[h].airframe.z, ground + 1000, "the cap");
    // A step: the unit ahead of its figure, which follows it.
    s.units[h].airframe.z = ground + 278;
    s.units[h].pos = Pos::new(at.x - 55, at.y - 51);
    s.process_movement(h);
    let z = s.units[h].airframe;
    assert_eq!((z.z, z.last_z), (ground + 308, ground + 278));
}

/// **A Helicopter is born one climb up** (item 3804,
/// `Unit::init@00612100:549`'s `set_new_location(x, y, 1, 1)`, whose
/// guy half is the climb; `docs/GOLDEN.md` §152.3): run1469's Attack
/// Helicopters fly 30 over theirs' first frame and ours flew at 0. A
/// plane is born at 0. Made to fail with the climb dropped.
#[test]
fn a_helicopter_is_born_one_climb_up() {
    let (mut s, b, planes) = hangar([0, 0, 0, 0]);
    let heli = air_type(&mut s, b, HELICOPTER, true, false);
    let at = Pos::new(8351, 14690);
    let ground = s.ground_z(at);
    let want = (ground + 1000).clamp(-30, 30);
    assert_ne!(want, 0, "the fixture's ground leaves a climb");
    let h = s.init_unit(0, heli, at);
    let z = s.units[h].airframe;
    assert_eq!((z.z, z.last_z), (want, want));
    let bomber = s.units[planes[0]].ty.unwrap();
    let p = s.init_unit(0, bomber, at);
    assert_eq!(s.units[p].airframe.z, 0, "a plane");
}

/// **A Missile Silo holds one missile at a time** (item 1019,
/// `Group::action_queue_up@006fdbb0`'s `can_carry(type)` at a silo,
/// `ObjectData::has_nuke@00643d40`): of `@queueup 313 2`, one V2 is
/// queued and paid and the second refused — run371's `queued 1` on
/// 2227. A missile already inside refuses the first as well. Made to
/// fail with the gate dropped (two queued).
#[test]
fn a_missile_silo_queues_one_missile_at_a_time() {
    let (mut s, b, _) = hangar([0, 0, 0, 0]);
    let silo_ty = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::MissileSilo,
        flags: crate::build::flags::TRAINS,
        ..crate::build::BuildType::default()
    });
    let silo = s.add_building(0, Pos::new(9984, 12288), 0);
    s.buildings[silo].ty = Some(silo_ty);
    s.buildings[silo].started = true;
    s.buildings[silo].active = true;
    let v2 = air_type(&mut s, b, 0x139, false, true);
    assert!(s.silo_takes(silo, v2), "an empty silo takes a missile");
    // Through the command: `@queueup 313 2` lays one entry.
    s.muster[0].researched[v2] = true;
    s.buildings[silo].queue.capacity = 20;
    for g in 0..crate::economy::RESOURCES {
        s.ledgers[0].bucket[g] = 100_000;
    }
    assert_eq!(s.action_queue_up(&[silo], v2, 2), 1, "one of two");
    assert_eq!(s.buildings[silo].queue.items.len(), 1);
    s.buildings[silo].queue.items.clear();
    s.buildings[silo].queue.items.push(crate::production::Item {
        job_counter: 0,
        ty: v2,
        tech: None,
        disband: false,
        good: [-1; 3],
        cost: [0; 3],
    });
    assert!(!s.silo_takes(silo, v2), "one queued: the second is refused");
    s.buildings[silo].queue.items.clear();
    let m = s.build_train(silo, v2).unit;
    assert_eq!(s.units[m].inside, Some(silo));
    assert!(!s.silo_takes(silo, v2), "one inside: refused");
    let bomber = air_type(&mut s, b, crate::air::BOMBER, false, false);
    let empty = s.add_building(0, Pos::new(7680, 12288), 0);
    s.buildings[empty].ty = Some(silo_ty);
    s.buildings[empty].active = true;
    assert!(!s.silo_takes(empty, bomber), "not a missile: refused");
}

/// **A silo with a missile inside holds its queue's clock** (item 2412,
/// `Build::do_queue@0061e410`'s second early return): the peace pair's
/// `1/2018` keeps its second ICBM at `job_counter` 0 while the first
/// stands inside (run853, 3296..3552). The clock runs once the silo is
/// empty, and a missile not yet trained (its availability bit clear, a
/// research job) is not held.
#[test]
fn a_silo_with_a_missile_inside_holds_its_queue_clock() {
    let (mut s, silo, m, _) = silo_with_a_v2();
    let v2 = s.units[m].ty.unwrap();
    s.buildings[silo].queue.capacity = 20;
    s.muster[0].researched[v2] = true;
    assert_eq!(s.buildings[silo].garrison, vec![m], "the V2 is inside");
    s.buildings[silo].queue.items.push(crate::production::Item {
        job_counter: 0,
        ty: v2,
        tech: None,
        disband: false,
        good: [-1; 3],
        cost: [0; 3],
    });
    s.process_queues();
    assert_eq!(s.buildings[silo].queue.items[0].job_counter, 0, "held");
    s.muster[0].researched[v2] = false;
    s.process_queues();
    // The fixture's times are short: an entry that runs may finish
    // and leave the queue in the one frame.
    let ran = |s: &Sim| {
        s.buildings[silo]
            .queue
            .items
            .first()
            .is_none_or(|i| i.job_counter > 0)
    };
    assert!(ran(&s), "a research job runs");
    s.muster[0].researched[v2] = true;
    s.buildings[silo].queue.items.clear();
    s.buildings[silo].queue.items.push(crate::production::Item {
        job_counter: 0,
        ty: v2,
        tech: None,
        disband: false,
        good: [-1; 3],
        cost: [0; 3],
    });
    s.process_queues();
    assert!(!ran(&s), "held again");
    s.buildings[silo].garrison.retain(|&u| u != m);
    s.units[m].inside = None;
    assert!(s.buildings[silo].garrison.is_empty());
    s.process_queues();
    assert!(ran(&s), "an empty silo runs its clock");
}

/// run371's silo `0/2009` with a V2 inside — `TO_HIT 300`, `ATTENUATE
/// 3`, `RECHARGE 30` at the silo — and the Barracks `1/2006` it strikes.
fn silo_with_a_v2() -> (Sim, usize, usize, usize) {
    let mut s = sim();
    let silo_ty = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::MissileSilo,
        flags: crate::build::flags::TRAINS,
        ..crate::build::BuildType::default()
    });
    let silo = s.add_building(0, Pos::new(9984, 12288), 0);
    s.buildings[silo].ty = Some(silo_ty);
    s.buildings[silo].started = true;
    s.buildings[silo].active = true;
    s.buildings[silo].combat = Some(combat::Profile {
        recharge: 30,
        ..combat::Profile::default()
    });
    let t = air_type(&mut s, silo, 0x139, false, true);
    s.unit_types[t].combat.to_hit = 300;
    s.unit_types[t].combat.attenuate = 3;
    let v2 = s.units.len() - 1;
    let enemy = enemy_barracks(&mut s, Pos::new(13824, 14976));
    (s, silo, v2, enemy)
}

/// **A missile's strike from its silo is an air attack on the point**
/// (item 1050): the flight command's inside arm takes a missile as it
/// takes a plane, and `add_strafe_order`'s head turns the strafe into
/// `add_air_attack_ground_order` at the target's point, home the silo,
/// with the action bit — run371's `0/10` on 2672. Made to fail with the
/// head dropped (a `StrafeOrder`) and with the inside arm's missile
/// refusal put back (no order).
#[test]
fn a_missile_s_strike_from_its_silo_is_an_air_attack_on_the_point() {
    let (mut s, silo, v2, enemy) = silo_with_a_v2();
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    assert_eq!(s.units[v2].orders.len(), 1, "one order");
    let o = s.units[v2].orders[0];
    assert_eq!(o.index(), crate::orders::index::AIR_ATTACK_GROUND);
    assert_eq!(o.flags, crate::orders::flag::ACTION, "flags 4");
    let crate::orders::Body::AirAttackGround(g) = o.body else {
        panic!("an air attack on the ground")
    };
    assert_eq!(
        g.at,
        Pos::new(13824, 14976),
        "att_x/att_y: the target's point"
    );
    assert_eq!(g.home, Some(silo), "oxx: the silo");
    assert_eq!(g.cruising_alt, crate::orders::CRUISING_ALT);
    assert!(!g.returning);
    assert_eq!(s.units[v2].inside, Some(silo), "the missile waits inside");
}

/// **The shield refuses a missile's order** (item 1078, `action_flight`'s
/// `6fbbd7`..`6fbbfb`): a target owned by a player holding
/// `MISSILE_DEFENSE_BONUS`'s prerequisite takes no missile — run390's
/// V2c `0/15` on 3080, inside with no order on 3082 — and the same
/// press before the shield orders it. Made to fail with the gate
/// dropped (the order laid).
#[test]
fn the_shield_refuses_a_missile_s_order_on_its_holder() {
    let (mut s, silo, v2, enemy) = silo_with_a_v2();
    let mut tree = s.tech_tree.clone();
    let shield = tree.types.len();
    tree.types.push(crate::tech::TypeDef::new(
        "Missile Shield",
        crate::tech::Kind::Final,
    ));
    tree.roles.missile_defense_preq = Some(shield);
    s.set_tech_tree(tree);
    for p in &mut s.tech {
        p.tech.resize(shield + 1, false);
    }
    s.tech[1].tech[shield] = true;
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    assert!(
        s.units[v2].orders.is_empty(),
        "refused: who=1 holds the shield"
    );
    s.tech[1].tech[shield] = false;
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    assert_eq!(s.units[v2].orders.len(), 1, "ordered without it");
}

/// **A nuke's launch shows its silo, and its round does not scatter**
/// (item 1091, `Build::do_missile_launch@00622670`'s nuke arm and
/// `Ammo::init@0067bbf0`'s `67c64b`; `tools/emu/nuke_launch.py`):
/// run397's `0/2007` prints `visible` −1 from 3082 and its round lands
/// on (23040, 34560), the target's point, with no draw. A V2 shows
/// nothing and scatters. Made to fail with the nuke's scatter left the
/// V2's (two draws) and with the silo's `+0x40` dropped.
#[test]
fn a_nuke_s_launch_shows_its_silo_and_its_round_does_not_scatter() {
    let mut spent = Vec::new();
    for nuke in [true, false] {
        let (mut s, silo, m, enemy) = silo_with_a_v2();
        let ty = s.units[m].ty.unwrap();
        if nuke {
            s.unit_types[ty].tree = Some(NUCLEARMISSILE);
        }
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        s.frame = 3051;
        s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
        let mut probe = s.rng;
        for _ in 0..31 {
            s.do_launch(silo);
            s.frame += 1;
        }
        let mut n = 0;
        while probe != s.rng {
            probe.roll();
            n += 1;
            assert!(n < 100, "diverged");
        }
        spent.push(n);
        assert!(!s.units[m].alive(), "fired and gone");
        assert_eq!(s.nukes.shown.contains(&silo), nuke, "the silo shown");
        if nuke {
            assert_eq!(
                s.projectiles[0].landing, s.buildings[enemy].pos,
                "on the point"
            );
        }
    }
    assert_eq!(spent[0] + 2, spent[1], "the V2's two scatter draws alone");
}

/// **A computer's silo strikes an enemy city with its nuke** (item
/// 1591, `Object::do_launch@0064f3b0`'s silo arm, `Sim::silo_strike`):
/// on `(frame + id) % 128 == 0` the nuke at the head of a computer's
/// silo takes `AIR_ATTACK_GROUND` at the city building's point, home
/// none, the action bit set — run710's ICBM `1/42` on 1569, at
/// Napata. Nothing off the 128-frame cadence (on the sortie's 32), for
/// a V2, over a city nobody has seen, over one with an object of the
/// silo's owner within 0x1800, or for a human's silo. Made to fail
/// with the silo's arm dropped (no order), with the cadence read as 32,
/// and with the friendly search dropped.
#[test]
fn a_computer_s_silo_strikes_an_enemy_city_with_its_nuke() {
    for (case, strikes) in [
        ("strikes", true),
        ("off the cadence", false),
        ("a V2", false),
        ("unseen", false),
        ("a friend beside it", false),
        ("human", false),
    ] {
        let (mut s, silo, m, enemy) = silo_with_a_v2();
        s.nation[0].human = case == "human";
        if case != "a V2" {
            let ty = s.units[m].ty.unwrap();
            s.unit_types[ty].tree = Some(NUCLEARMISSILE);
        }
        // Past 0x1800 of the silo itself, an object of its owner's.
        s.buildings[enemy].pos = Pos::new(20160, 20160);
        if case == "unseen" {
            s.buildings[enemy].ever_seen = 0;
        }
        if case == "a friend beside it" {
            let at = s.buildings[enemy].pos;
            friend(&mut s, Pos::new(at.x + 0x1800, at.y));
        }
        let pos = s.buildings[enemy].pos;
        s.cities.push(crate::city::City {
            alive: true,
            owner: 1,
            race: Some(1),
            founder: 1,
            building: enemy,
            members: Vec::new(),
            reg: None,
            pos,
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
        let id = i64::from(s.buildings[silo].index);
        s.frame = 128 * 30 - id + if case == "off the cadence" { 32 } else { 0 };
        s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
        s.do_launch(silo);
        let order = s.units[m].orders.front().copied();
        if strikes {
            let o = order.expect("the nuke is ordered");
            assert_eq!(o.flags, crate::orders::flag::ACTION, "the action bit");
            let crate::orders::Body::AirAttackGround(g) = o.body else {
                panic!("an air attack on the ground")
            };
            assert_eq!(g.at, pos, "the city building's point");
            assert_eq!(g.home, None, "home −1");
            assert_eq!(s.units[m].inside, Some(silo), "it waits inside");
        } else {
            assert_eq!(order, None, "{case}: ordered");
        }
    }
}

/// **A missile that fires leaves its type's count** (item 1078,
/// `Unit::close@0060ee50`'s `track_unit_type(·, −1)`): the next V2 is
/// priced as if none stood — run390's `0/2010` charged 100 and 100 on
/// 2722, where this crate charged 120. Made to fail with the decrement
/// dropped (the count stays 1).
#[test]
fn a_missile_that_fires_leaves_its_type_s_count() {
    let (mut s, silo, v2, enemy) = silo_with_a_v2();
    let ty = s.units[v2].ty.unwrap();
    s.unit_types[ty].price.pop = 1;
    let before = s.muster[0].by_type[ty];
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    s.frame = 2671;
    s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
    for _ in 0..31 {
        s.do_launch(silo);
        s.frame += 1;
    }
    assert!(!s.units[v2].alive(), "fired and gone");
    assert_eq!(s.muster[0].by_type[ty], before - 1, "out of num_units");
}

/// **A second strike pressed while the first counts down finds
/// nothing** (item 1078, `action_launch_flight`'s `6fc681`..`6fc6ef`,
/// `tools/emu/launch_arm.py`): the V2 on its `AIR_ATTACK_GROUND` is
/// passed over, with or without shift, and keeps the first target's
/// point — chapter thirty-six's `0/14` on 3029. Made to fail with the
/// live-order test dropped (the V2 re-pointed to the second target).
#[test]
fn a_second_strike_during_the_countdown_passes_the_missile_over() {
    let (mut s, silo, v2, enemy) = silo_with_a_v2();
    let other = enemy_barracks(&mut s, Pos::new(35712, 18816));
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    s.frame = 2671;
    s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
    s.do_launch(silo);
    assert_eq!(s.buildings[silo].recharging, 30, "counting down");
    for shift in [false, true] {
        let keys = Keys {
            shift,
            ..Keys::default()
        };
        s.group_action_launch_flight(0, &[silo], Obj::Building(other), Flight::Strike, keys);
        assert_eq!(s.units[v2].orders.len(), 1, "one order still");
        let crate::orders::Body::AirAttackGround(g) = s.units[v2].orders[0].body else {
            panic!("the air attack on the ground")
        };
        assert_eq!(
            g.at,
            Pos::new(13824, 14976),
            "the first target's point, shift {shift}"
        );
    }
    assert_eq!(
        s.buildings[silo].launching,
        vec![v2],
        "one entry in launching"
    );
}

/// **The silo counts its missile out, and the missile fires and ends in
/// the same call** (item 1050): the launch sets `recharging` to the
/// silo's `RECHARGE`, 30, with `launch_frames` 0 and the missile in
/// `launching`; each later call takes one off and does nothing else;
/// at 0 the missile comes out on the silo's own point, flies one step,
/// fires its round from its point plus [`crate::air::MISSILE_OFFSET`],
/// and dies with its number held for the round's 120 frames and one.
/// run371: 30 on 2672, 1 on 2701, the round on 2702. Made to fail with
/// the countdown's head dropped (the missile never leaves), the
/// missile's `recharging` dropped (the same), and the silo's point
/// dropped from `come_out` (the ring's spot).
#[test]
fn the_silo_counts_its_missile_out_and_it_fires_on_the_thirtieth() {
    let (mut s, silo, v2, enemy) = silo_with_a_v2();
    s.group_action_launch_flight(
        0,
        &[silo],
        Obj::Building(enemy),
        Flight::Strike,
        Keys::default(),
    );
    s.frame = 2671;
    s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
    s.do_launch(silo);
    assert_eq!(s.buildings[silo].recharging, 30);
    assert_eq!(s.buildings[silo].launch_frames, 0);
    assert_eq!(s.buildings[silo].launching, vec![v2]);
    for k in 1..30 {
        s.frame += 1;
        s.do_launch(silo);
        assert_eq!(s.buildings[silo].recharging, 30 - k);
        assert_eq!(s.buildings[silo].launch_frames, 0, "it stands");
        assert_eq!(s.units[v2].inside, Some(silo));
    }
    assert!(s.projectiles.is_empty());
    s.frame += 1;
    s.do_launch(silo);
    assert_eq!(s.buildings[silo].recharging, 0);
    assert!(s.buildings[silo].launching.is_empty());
    assert!(s.buildings[silo].garrison.is_empty());
    assert!(!s.units[v2].alive(), "the missile ends with its shot");
    let at = s.units[v2].pos;
    let from = s.buildings[silo].pos;
    let step = vector_dist(at.x - from.x, at.y - from.y);
    assert!(
        step > 0 && step <= s.get_speed(v2, 1) + 1,
        "one step from the silo's own point: {at:?}"
    );
    assert_eq!(s.projectiles.len(), 1, "one round");
    let p = s.projectiles[0];
    let (dx, dy, dz) = crate::air::MISSILE_OFFSET;
    assert_eq!(p.launch, Pos::new(at.x + dx, at.y + dy));
    assert_eq!(p.sz, dz, "the ground at 0 and the offset's height");
    assert_eq!(p.total_time, crate::air::MISSILE_FLIGHT);
    assert_eq!(p.target, None, "a round at the ground");
    // 120 frames of flight, one, and `nuke_effect +0x108`'s 30 (item
    // 1594): `Object::die` adds the spread time to every round.
    assert_eq!(s.units[v2].hold_frames, 151, "held for its round");
}

/// **A missile's round, on run371's own numbers** (item 1050,
/// `Ammo::init`'s missile arm): from (10083, 12346) at the point
/// (13824, 14976), accuracy `300 − 3 · (4662 / 192)` = 228, the land
/// formula's 11 **doubled** to 22, and the trace's own frame word
/// `0x14e73b8f` — rolls 62369 and 56984, `% 22 − 11` — lands it on
/// (13834, 14969) after 120 frames, as the dump's `ex`/`ey` read on 2702.
/// Made to fail with the doubling dropped (`s` 11: (13826, 14976)) and
/// with the time the flight formula's.
#[test]
fn a_missile_s_round_is_run371_s_to_the_unit() {
    let (mut s, silo, v2, _) = silo_with_a_v2();
    s.come_out(v2);
    s.units[v2].pos = Pos::new(10083, 12346);
    let _ = silo;
    s.rng.seed = 0x14e7_3b8f;
    s.missile_round(v2, Pos::new(13824, 14976));
    let p = s.projectiles[0];
    assert_eq!(p.launch, Pos::new(9974, 12347), "sx, sy");
    assert_eq!(p.accuracy, 228);
    assert_eq!(p.landing, Pos::new(13834, 14969), "ex, ey");
    assert_eq!(p.total_time, 120);
    assert_eq!(p.angle.0, 1_482_031_104, "the dump's angle");
}
