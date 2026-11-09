use super::*;
use crate::combat::Profile;
use crate::world::World;

/// **A follower whose target has died takes its captain's**
/// (`Unit::find_new_target@005ff6a0` → `find_melee_target(−1, NULL, 0,
/// 1, 0)`, whose head is `005ff9c0:32-91`; `docs/COMBAT.md` §79).
/// run347's `1/13` on tick 5066: its target `0/2` dead, it takes its
/// captain `1/12`'s `0/4`, some 3,000 away, where a search would have
/// ranked the nearer `0/5` first. run400's packet entered the function
/// on `1/13` and it added `0/4` from its head, with no search at all.
///
/// A RAID follower out of reach of its captain's target, so `fight`'s
/// own mirror (which needs AGGRESSIVE, DEFENSIVE or the target in
/// range) does not hand it over first. Made to fail by taking the head
/// out of the invalid-target arm: the follower searches and takes the
/// nearer foe.
#[test]
fn a_follower_whose_target_died_takes_its_captain_s() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut Sim, who: Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 100);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.tables.unit_types[ty].kind;
        sim.add_unit(u)
    };
    let c = 30 * 0x300 + 0x198;
    let captain = put(&mut sim, 1, Pos::new(c - 16 * 0xc0, c));
    let me = put(&mut sim, 1, Pos::new(c, c));
    let far = put(&mut sim, 0, Pos::new(c - 24 * 0xc0, c));
    let near = put(&mut sim, 0, Pos::new(c + 8 * 0xc0, c));
    let dead = put(&mut sim, 0, Pos::new(c, c + 6 * 0xc0));
    sim.units[me].captain = false;
    sim.units[me].combat.captain = i32::from(sim.units[captain].index);
    sim.units[me].o_up = Some(captain);
    sim.units[captain].o_down = Some(me);
    sim.set_stance(me, combat::Stance::Raid);
    sim.add_attack_order(captain, Obj::Unit(far), QueuePos::First, false, true);
    sim.add_attack_order(me, Obj::Unit(dead), QueuePos::First, false, true);
    sim.units[dead].health = 0;
    // Off the review's phase, `(frame + o) % 16 != 0`.
    let frame = 17 - i64::from(sim.units[me].index);
    assert!(!sim.is_in_range(Obj::Unit(me), Obj::Unit(far)));
    let mut searched = sim.clone();
    assert_eq!(
        searched.find_melee_target(me, -1),
        Some(Obj::Unit(near)),
        "the search alone ranks the nearer foe first"
    );
    sim.work(me, frame);
    assert_eq!(
        sim.units[me].combat.target,
        Some(Obj::Unit(far)),
        "the follower took its captain's target"
    );
}

/// **A target running from the chaser's heading keeps the chase in
/// reach** (`do_move@005f7b30`, the listing `5f7fbe`–`5f7ff6`;
/// `docs/COMBAT.md` §71). run347's `1/11` walks its chase on frame 4922
/// at (5046, 30225), heading −321454080, after the citizen `0/1` at
/// (4035, 28584) walking east, heading `0x40000000`: `e = 0xd3290000`,
/// `flanking` 2, so the original keeps walking, and ends the chase on
/// 4923, when `0/1` has turned and `flanking` reads 0.
///
/// Three arms, on a frame off the review's phase: a target walking
/// ahead of the chaser, in reach, keeps the chase; a target standing
/// there, or walking toward the chaser, ends it. Made to fail by
/// taking the flank clause out of the kill: the first arm ends its
/// chase.
#[test]
fn a_target_running_from_the_chaser_s_heading_keeps_the_chase() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut Sim, who: Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 100);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.tables.unit_types[ty].kind;
        sim.add_unit(u)
    };
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let foe = put(&mut sim, 0, at);
    let me = put(&mut sim, 1, Pos::new(at.x + 2 * 0xc0, at.y));
    let west = crate::movement::find_angle(-1, 0);
    let east = crate::movement::find_angle(1, 0);
    sim.units[me].movement.heading = west;
    assert!(
        sim.is_in_range_at_margin(Obj::Unit(me), sim.units[me].pos, Obj::Unit(foe), true),
        "the chaser stands inside its reach less the margin"
    );
    // Off the review's phase, `(frame + o) % 16 != 0`.
    let frame = 17 - i64::from(sim.units[me].index);
    let chase = |heading: Option<crate::movement::Angle>| {
        let mut s = sim.clone();
        if let Some(h) = heading {
            s.add_move_order(
                foe,
                Pos::new(at.x - 20 * 0xc0, at.y),
                MoveKind::MoveTo,
                QueuePos::New,
                false,
            );
            s.units[foe].movement.heading = h;
        }
        s.add_attack_order(me, Obj::Unit(foe), QueuePos::First, false, true);
        s.add_move_order(
            me,
            Pos::new(at.x - 10 * 0xc0, at.y),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        assert!(!s.units[me].combat.mandatory);
        s.work(me, frame);
        s.units[me].orders.iter().any(Order::is_move)
    };
    assert!(
        chase(Some(west)),
        "a target running along the chaser's heading ended the chase"
    );
    assert!(!chase(None), "a standing target kept the chase");
    assert!(
        !chase(Some(east)),
        "a target walking toward the chaser kept the chase"
    );
}

/// **A living target out of sight keeps the chase** (`do_move@005f7b30`,
/// the listing `5f7ece`–`5f7f11`; `docs/COMBAT.md` §77). The action
/// block asks of its target only that the slot be live — `o`/`who`
/// non-negative, `flags & 1`, the `uid` — and sends nothing else to the
/// dead target's arm at `5f8221`. run347's `1/10`, `1/11` and `1/18`
/// chase the citizen `0/1` on frame 5075 while player 1 has lost sight
/// of it, each within `0x480` of its walk's goal: the original walks on,
/// and this crate, which asked `valid_target`, popped all three walks.
///
/// Two arms, on a frame off the review's phase and under a fog grid
/// that shows nobody anything: the living target keeps the chase, the
/// dead one loses it. Made to fail by gating the block on
/// `valid_target` again: the first arm pops its walk.
#[test]
fn a_living_target_out_of_sight_keeps_the_chase() {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    assert!(
        sim.world.set_fog(vec![0; 60 * 60 * 4]),
        "a 60 x 60 world's fog"
    );
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 4,
            uber_size: 1,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut Sim, who: Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 100);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.tables.unit_types[ty].kind;
        sim.add_unit(u)
    };
    let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
    let foe = put(&mut sim, 0, Pos::new(at.x - 20 * 0xc0, at.y));
    let me = put(&mut sim, 1, at);
    assert!(
        !sim.target_is_seen(Obj::Unit(me), Obj::Unit(foe)),
        "player 1 cannot see its target"
    );
    assert!(
        !sim.is_in_range(Obj::Unit(me), Obj::Unit(foe)),
        "the target is out of reach"
    );
    // Off the review's phase, `(frame + o) % 16 != 0`.
    let frame = 17 - i64::from(sim.units[me].index);
    let chase = |dead: bool| {
        let mut s = sim.clone();
        s.add_attack_order(me, Obj::Unit(foe), QueuePos::First, false, true);
        // The walk's goal is 3 tiles on, inside `0x480`.
        s.add_move_order(
            me,
            Pos::new(at.x - 4 * 0xc0, at.y),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        if dead {
            s.units[foe].health = 0;
        }
        s.work(me, frame);
        s.units[me].orders.iter().any(Order::is_move)
    };
    assert!(chase(false), "a living target out of sight ended the chase");
    assert!(!chase(true), "a dead target kept the chase");
}
