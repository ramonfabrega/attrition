use crate::ai_load::uflags;
use crate::combat::{Obj, Profile, Stance};
use crate::orders::{Body, MoveKind, QueuePos, index};
use crate::world::World;
use crate::{Player, Pos, Sim};

/// Two players at war on open ground, and a Dragoon-shaped type: an
/// armed, ranged combat unit whose `FLAGS` carry `v` when `fires`.
fn field(fires: bool) -> (Sim, usize) {
    let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    sim.cities_state.nation[0].human = true;
    sim.cities_state.nation[1].human = true;
    let mut t = crate::UnitType {
        hits: 100,
        combat: Profile {
            attack: 17,
            max_range: 9,
            recharge: 30,
            uber_size: 1,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    };
    if fires {
        t.cols.unit_flags |= uflags::FIRES_ON_MOVE;
    }
    let ty = sim.add_unit_type(t);
    (sim, ty)
}

fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
    let index = i16::try_from(sim.units.len()).unwrap();
    let hits = sim.tables.unit_types[ty].hits;
    let mut u = crate::Unit::new(who, index, p, hits);
    u.ty = Some(ty);
    u.on_map = true;
    u.kind = sim.tables.unit_types[ty].kind;
    let i = sim.add_unit(u);
    sim.units[i].guys.push(crate::anim::Guy::fresh(-1));
    i
}

/// **A `v` type walking past an enemy finds it, then shoots it, and
/// walks on** (`docs/AI.md` §134, run657's `1/80` on 17168 and
/// 17169). The first frame's search writes the pair (`cavarch_o`,
/// `cavarch_who`) and spends no reload; the second, in range, is the
/// strike: the reload is set and the move is still the only order.
/// Without `v` the gate does nothing at all.
///
/// The walk holds the call: without [`Sim::cavarch_head`] in `do_move`
/// run657 reads 209 keys against 154 and run661 574 against 254 (item
/// 1508's M1).
#[test]
fn a_unit_that_fires_on_the_move_finds_shoots_and_walks_on() {
    for fires in [true, false] {
        let (mut sim, ty) = field(fires);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
        sim.add_move_order(
            me,
            Pos::new(9000, 3000),
            MoveKind::AttackTo,
            QueuePos::New,
            false,
        );
        let frame = 64 - i64::from(sim.units[me].index);
        sim.cavarch_head(me, frame);
        if !fires {
            assert_eq!(sim.cavarch_o(me), -1, "no `v`, no search");
            assert_eq!(sim.units[me].near, None);
            continue;
        }
        assert_eq!(
            sim.units[me].cast_target,
            Some(Obj::Unit(foe)),
            "the search"
        );
        assert_eq!(sim.units[me].cavarch_who, 0);
        assert_eq!(
            sim.units[me].combat.recharging, 0,
            "a search does not shoot"
        );
        sim.cavarch_head(me, frame + 1);
        assert_eq!(sim.units[me].combat.recharging, 30, "the strike's reload");
        assert_eq!(sim.cavarch_o(me), i64::from(sim.units[foe].index));
        assert_eq!(sim.units[me].orders.len(), 1, "the walk is untouched");
        assert_eq!(sim.order_type(me), index::ATTACK_TO);
    }
}

/// **A captain re-searches what it holds before it shoots**
/// (`fight@005fd4d0:367-461` on the `param_5` path): holding an unarmed
/// unit in range, it rolls (`% 5`, here a seed that does not suppress)
/// and finds the dearer combat unit beside it; the new pair is written
/// and nothing is shot. A non-captain shoots what it holds.
///
/// Made to fail with the arm removed (item 1508's M6), which no walk
/// holds: run657's and run661's counts stand.
#[test]
fn a_captain_firing_on_the_move_retargets_before_it_shoots() {
    for captain in [true, false] {
        let (mut sim, ty) = field(true);
        let mut cheap = sim.tables.unit_types[ty].clone();
        cheap.combat.combat_role = false;
        cheap.combat.cost = 1;
        let cheap = sim.add_unit_type(cheap);
        sim.tables.unit_types[ty].combat.cost = 1000;
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let held = put(&mut sim, 0, cheap, Pos::new(3000, 3900));
        let dear = put(&mut sim, 0, ty, Pos::new(3200, 3900));
        sim.units[me].combat.captain = if captain {
            i32::from(sim.units[me].index)
        } else {
            -1
        };
        while {
            let mut r = sim.rng;
            r.roll() % 5 == 0
        } {
            sim.rng.seed = sim.rng.seed.wrapping_add(1);
        }
        sim.units[me].cast_target = Some(Obj::Unit(held));
        sim.units[me].cavarch_who = 0;
        sim.cavarch_fight_body(me, 100);
        if captain {
            assert_eq!(
                sim.units[me].cast_target,
                Some(Obj::Unit(dear)),
                "retargeted"
            );
            assert_eq!(sim.units[me].combat.recharging, 0, "and nothing shot");
        } else {
            assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(held)));
            assert_eq!(sim.units[me].combat.recharging, 30, "shot");
        }
    }
}

/// **A city at its ceiling is still borne on** (item 3635, `docs/AI.md`
/// §330): `Guy::set_all_pivots@005d8bc0`'s gone test is the target
/// slot's `flags & 1` (`005d8c69`), never its hit points, so a walking
/// `v` type whose `cavarch_o` holds a city brought to 0 keeps it, and
/// its step walks `CHAR_ATTACKWALK`. run1295's Cavalry Archer `1/124`
/// (TypeIndex 188) holds the Village `0/2000` from its fall on 5963 to
/// 5969; ours dropped it on 5964. A building gone from its slot is
/// dropped (−1) and the walk is `CHAR_WALK`.
#[test]
fn a_city_at_its_ceiling_is_still_borne_on() {
    for alive in [true, false] {
        let (mut sim, ty) = field(true);
        const CAVARCHERS: i32 = 188;
        sim.tables.unit_types[ty].type_index = CAVARCHERS;
        sim.roster
            .art
            .pivots
            .insert(CAVARCHERS, [(4, (-180, 180))].into_iter().collect());
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let b = sim.add_building(0, Pos::new(3000, 4200), 0);
        sim.buildings[b].combat = Some(Profile::default());
        sim.buildings[b].health = 0;
        sim.buildings[b].alive = alive;
        sim.units[me].cast_target = Some(Obj::Building(b));
        let walk = sim.cavarch_step_anim(me);
        if alive {
            assert_eq!(walk, crate::anim::ATTACKWALK, "the turret bears");
            assert_eq!(sim.units[me].cast_target, Some(Obj::Building(b)));
        } else {
            assert_eq!(walk, crate::anim::WALK, "a gone slot is dropped");
            assert_eq!(sim.cavarch_o(me), -1);
        }
    }
}

/// **The fire on the move's bearing is off the unit's heading, not its
/// figure's facing** (`find_nearby_target@00648da0`'s `param_4` arm
/// reads `UnitData +0x50`, `angle`; item 3530, `docs/AI.md` §305). Two
/// like foes at one distance, one on the heading and one on the facing
/// a quarter turn away: the search takes the one on the heading (×4
/// against ×1), whichever way the body points.
#[test]
fn the_fire_on_the_move_bears_off_the_heading_not_the_facing() {
    use crate::movement::find_angle;
    for swap in [false, true] {
        let (mut sim, ty) = field(true);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let east = put(&mut sim, 0, ty, Pos::new(4200, 3000));
        let south = put(&mut sim, 0, ty, Pos::new(3000, 4200));
        sim.add_move_order(
            me,
            Pos::new(9000, 3000),
            MoveKind::AttackTo,
            QueuePos::New,
            false,
        );
        let (to_e, to_s) = (find_angle(1200, 0), find_angle(0, 1200));
        let (heading, facing) = if swap { (to_s, to_e) } else { (to_e, to_s) };
        sim.units[me].movement.heading = heading;
        sim.units[me].movement.facing = facing;
        let frame = 64 - i64::from(sim.units[me].index);
        sim.cavarch_head(me, frame);
        let want = if swap { south } else { east };
        assert_eq!(
            sim.units[me].cast_target,
            Some(Obj::Unit(want)),
            "the foe on the heading"
        );
    }
}

/// **A Bark captain keeps a target that is not combat, and re-searches
/// one that is** (`fight@005fd4d0:394`–`400`'s `local_14`, item 3516):
/// the other way round from any other captain. run1161b's Cruiser
/// `1/25` (`unit_masks2` 4) keeps the citizen `0/5` on 3681 and fires,
/// where a re-search found `0/11`.
#[test]
fn a_bark_captain_firing_on_the_move_keeps_a_target_that_is_not_combat() {
    for held_combat in [false, true] {
        let (mut sim, ty) = field(true);
        let mut cheap = sim.tables.unit_types[ty].clone();
        cheap.combat.combat_role = held_combat;
        cheap.combat.cost = 1;
        let cheap = sim.add_unit_type(cheap);
        sim.tables.unit_types[ty].combat.cost = 1000;
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let held = put(&mut sim, 0, cheap, Pos::new(3000, 3900));
        let dear = put(&mut sim, 0, ty, Pos::new(3200, 3900));
        sim.units[me].unit_masks2 |= crate::combat::umask2::MASK2_BARK;
        sim.units[me].combat.captain = i32::from(sim.units[me].index);
        while {
            let mut r = sim.rng;
            r.roll() % 5 == 0
        } {
            sim.rng.seed = sim.rng.seed.wrapping_add(1);
        }
        sim.units[me].cast_target = Some(Obj::Unit(held));
        sim.units[me].cavarch_who = 0;
        sim.cavarch_fight_body(me, 100);
        if held_combat {
            assert_eq!(
                sim.units[me].cast_target,
                Some(Obj::Unit(dear)),
                "a combat target: re-searched"
            );
            assert_eq!(sim.units[me].combat.recharging, 0, "and nothing shot");
        } else {
            assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(held)), "kept");
            assert_eq!(sim.units[me].combat.recharging, 30, "shot");
        }
    }
}

/// **A formation follower fires on the move too, and keeps its pair
/// where the head drops it** (`do_group_move@005e79a0:392`–`406`, item
/// 3336; run967g's Royal Mameluke `1/68` on 20186). A `v` follower on
/// an attack-move searches as the head does; under `HOLD_FIRE` the
/// head clears `cavarch_o` and the follower skips with it kept.
///
/// The call in `group_move_follower` is the floor's to hold: dropped,
/// the Egyptian word reads 20186 and this test stays green (item
/// 3336's M2).
#[test]
fn a_follower_fires_on_the_move_and_keeps_its_pair_under_hold_fire() {
    let (mut sim, ty) = field(true);
    let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
    let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
    sim.add_move_order(
        me,
        Pos::new(9000, 3000),
        MoveKind::AttackTo,
        QueuePos::New,
        false,
    );
    let frame = 64 - i64::from(sim.units[me].index);
    sim.cavarch_follower(me, frame);
    assert_eq!(
        sim.units[me].cast_target,
        Some(Obj::Unit(foe)),
        "the search"
    );
    sim.units[me].combat.stance = Stance::HoldFire;
    sim.cavarch_follower(me, frame + 1);
    assert_eq!(
        sim.units[me].cast_target,
        Some(Obj::Unit(foe)),
        "a follower under HOLD_FIRE keeps the pair"
    );
    assert_eq!(sim.units[me].combat.recharging, 0, "and does not shoot");
    sim.cavarch_head(me, frame + 1);
    assert_eq!(sim.cavarch_o(me), -1, "the head clears it");
}

/// **A `k` type keeps its throttle under an `ATTACK` action**
/// (`do_move@005f7b30:68-72`, item 3554): a `v` type whose search has
/// fallen to −2 is cleared to −1 by an attack under its move — unless
/// its flags carry `k` (`unit_flags & 0x400`), which goes on to the
/// phase and stays −2 off it. run1181's Mahouts `1/27` and `1/46`
/// (TypeIndex 262, `mjvyk`) read −2 on 4001 under `[AttackTo,
/// Attack(0/10), Move]`; ours read −1, and `1/27` missed its shot on
/// 4038 (`docs/AI.md` §308).
#[test]
fn a_melee_and_ranged_type_keeps_its_wait_under_an_attack_action() {
    for k in [false, true] {
        let (mut sim, ty) = field(true);
        if k {
            sim.tables.unit_types[ty].cols.unit_flags |= uflags::MELEE_AND_RANGED;
        }
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 30000));
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, false, true);
        sim.units[me].cavarch_idle = true;
        let off_phase = 65 - i64::from(sim.units[me].index);
        sim.cavarch_head(me, off_phase);
        let want = if k { -2 } else { -1 };
        assert_eq!(sim.cavarch_o(me), want, "k {k}: the attack's clear");
        sim.cavarch_follower(me, off_phase);
        assert_eq!(sim.cavarch_o(me), want, "k {k}: the follower");
    }
}

/// **A melee-and-ranged type walks past `fight`'s reload gate**
/// (`fight@005fd4d0:106`, item 3568): recharging on a fresh attack out
/// of range, a plain type stands (the reload idle, and no chase) where a
/// `k` type goes on and chases; in range, the `k` type swings and
/// returns before the blow and the reload (`005feecb`). run1189b's
/// Mahout `1/46` (TypeIndex 262, `mjvyk`), `recharging` 35 on 4170, is
/// the witness (`docs/AI.md` §308).
#[test]
fn a_melee_and_ranged_type_walks_past_the_reload_gate() {
    for k in [false, true] {
        let (mut sim, ty) = field(false);
        if k {
            sim.tables.unit_types[ty].cols.unit_flags |= uflags::MELEE_AND_RANGED;
        }
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 30000));
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, false, true);
        sim.units[me].combat.recharging = 20;
        sim.do_attack_fight(me, 100);
        let chased = sim.units[me].orders.len() == 2;
        assert_eq!(chased, k, "k {k}: the chase out of range");
        // In range: no blow, the reload untouched.
        let (mut sim, ty) = field(false);
        if k {
            sim.tables.unit_types[ty].cols.unit_flags |= uflags::MELEE_AND_RANGED;
        }
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 3400));
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, false, true);
        sim.units[me].combat.recharging = 20;
        sim.do_attack_fight(me, 100);
        assert_eq!(sim.units[me].combat.recharging, 20, "k {k}: no reload");
        assert_eq!(sim.units[foe].health, 100, "k {k}: no blow");
        assert_eq!(sim.units[me].orders.len(), 1, "k {k}: no chase in range");
    }
}

/// A melee-and-ranged type as the Mahout is (`RANGE 0-8`, `mjvyk`):
/// melee at its first range, 8 at its second, firing on the move.
fn mahout(k: bool) -> (Sim, usize) {
    let (mut sim, ty) = field(true);
    let t = &mut sim.tables.unit_types[ty];
    t.combat.max_range = 0;
    t.combat.second_max_range = 8;
    if k {
        t.cols.unit_flags |= uflags::MELEE_AND_RANGED;
    }
    (sim, ty)
}

/// **A type whose graphic releases a round lands no melee blow**
/// (`fight@005fd4d0:853`–`854`, item 3570): the melee arm strikes only
/// while `UnitType +0x2cc` stands, and
/// `GraphicEvents::init_unit_events@008e2520:325` zeroes it for a
/// graphic with a `RELEASEEVENT`. Both swing and reload; only the one
/// without a release wounds. run1189b's Mahout `1/46` (TypeIndex 262)
/// swings at `0/2000` on 4240, and the building stands at 108 in
/// theirs where ours struck it to 112 (`docs/AI.md` §308).
#[test]
fn a_melee_swing_whose_graphic_releases_a_round_lands_no_blow() {
    for release in [false, true] {
        let (mut sim, ty) = mahout(false);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 3100));
        if release {
            let piece = sim.units[me].guys[0].gpiece;
            sim.roster.art.releases.insert(
                piece,
                [(
                    crate::anim::ATTACK1,
                    vec![(1, 0, false, false, false, false)],
                )]
                .into_iter()
                .collect(),
            );
        }
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, false, true);
        let struck = sim.do_attack_fight(me, 100);
        assert!(struck, "release {release}: the strike answers 1");
        assert_eq!(
            sim.units[me].combat.recharging, 30,
            "release {release}: the reload"
        );
        let wounded = sim.units[foe].health < 100;
        assert_eq!(wounded, !release, "release {release}: the blow");
    }
}

/// **`do_attack`'s tail runs a `k` type's fire on the move after a
/// strike** (`do_attack@005f1b80:255`–`259`, item 3570): `fight`
/// answered 1, so `cavarch_fight` runs with `recharging` held at 0 —
/// its search, at the second range, writes the pair — and the reload
/// is put back. A type without `k` writes nothing. run1189b's Mahout
/// `1/46` reads `cavarch_o` 2000 on 4241 in theirs, −2 in ours before.
#[test]
fn a_melee_and_ranged_strike_takes_the_fire_on_the_move_s_fight() {
    for k in [false, true] {
        let (mut sim, ty) = mahout(k);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 3100));
        sim.add_attack_order(me, Obj::Unit(foe), QueuePos::New, false, true);
        sim.do_attack(me, 100);
        assert_eq!(
            sim.units[me].combat.recharging, 30,
            "k {k}: the reload stands"
        );
        let want = if k {
            i64::from(sim.units[foe].index)
        } else {
            -1
        };
        assert_eq!(sim.cavarch_o(me), want, "k {k}: the pair");
    }
}

/// **An empty search after an empty one waits for the phase**
/// (`cavarch_fight@005ff4b0`'s tail, `do_move@005f7b30:66`): −1, then
/// −2, and from −2 a search runs only on `(o + frame) % 32 == 0`.
/// The one that does run finds a foe that walked in.
#[test]
fn an_empty_search_twice_waits_for_the_thirty_two_frame_phase() {
    let (mut sim, ty) = field(true);
    let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
    sim.add_move_order(
        me,
        Pos::new(9000, 3000),
        MoveKind::AttackTo,
        QueuePos::New,
        false,
    );
    let o = i64::from(sim.units[me].index);
    let phase = 64 - o;
    sim.cavarch_head(me, phase + 1);
    assert_eq!(sim.cavarch_o(me), -2, "an empty answer after −1 at init");
    let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
    sim.cavarch_head(me, phase + 2);
    assert_eq!(sim.cavarch_o(me), -2, "off the phase, no search");
    sim.cavarch_head(me, phase + 32);
    assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(foe)), "on it");
}

/// **The step walks under `CHAR_ATTACKWALK` while the pivots bear,
/// and drops the target when they do not** (`move_step@005faf30:66-100`).
/// The turret's arc is ±90° off the bow: a foe on the bow bears; one
/// behind does not, and the target goes (−1) with the walk.
#[test]
fn the_step_attack_walks_while_the_turret_bears() {
    for (at, bears) in [(Pos::new(3000, 4200), true), (Pos::new(3000, 1800), false)] {
        let (mut sim, ty) = field(true);
        let ti = sim.tables.unit_types[ty].type_index;
        sim.roster
            .art
            .pivots
            .insert(ti, [(4, (-90, 90))].into_iter().collect());
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, at);
        sim.units[me].movement.facing = crate::movement::find_angle(0, 1);
        sim.units[me].cast_target = Some(Obj::Unit(foe));
        let walk = sim.cavarch_step_anim(me);
        assert_eq!(sim.units[me].guys[0].aim, Some(Obj::Unit(foe)), "aimed");
        if bears {
            assert_eq!(walk, crate::anim::ATTACKWALK);
            assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(foe)));
        } else {
            assert_eq!(walk, crate::anim::WALK);
            assert_eq!(sim.cavarch_o(me), -1, "dropped");
            assert_eq!(sim.units[me].guys[0].turret.des_flags & 1, 1);
        }
    }
}

/// **An empty search sends the turret's `des` back to its rest angle**
/// (`find_melee_target@005ff9c0:245`–`262`, `Guy::reset_pivots(0)`;
/// item 1829, run783's `0/6` on 793): the Dragoon's turret aimed
/// −69° at a Citizen, the captain's re-search finds nothing, and the
/// next block reads `des_turret_angles[0]` 0 with the turret's own
/// angle and the node flags untouched. A type without `v` keeps its
/// aim — the reset is the fire-on-the-move arm's alone.
#[test]
fn an_empty_search_of_a_v_type_resets_the_pivots_and_only_that() {
    for fires in [true, false] {
        let (mut sim, ty) = field(fires);
        let ti = sim.tables.unit_types[ty].type_index;
        sim.roster
            .art
            .pivots
            .insert(ti, [(4, (-150, 150))].into_iter().collect());
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let aim = -821_952_512;
        let t = &mut sim.units[me].guys[0].turret;
        t.angles[0] = aim;
        t.des[0] = aim;
        t.des_flags = 1;
        t.node_flags = 15;
        assert_eq!(sim.find_melee_target_as(me, -1, 0, true), None);
        let t = sim.units[me].guys[0].turret;
        assert_eq!(t.des[0], if fires { 0 } else { aim }, "des");
        assert_eq!((t.angles[0], t.node_flags, t.des_flags), (aim, 15, 1));
    }
}

/// **A request for `CHAR_ATTACKWALK` asks the reload** (`Guy::set_anim@
/// 005da300`'s head, run772's Dragoon on 1021 and 1022, `docs/GOLDEN.md`
/// §65): from another animation, a reload under six frames is cleared and
/// the walk-and-shoot plays; six and over, the walk plays instead. From
/// `CHAR_ATTACKWALK` itself the request leaves the reload alone.
#[test]
fn an_attack_walk_request_clears_a_short_reload_and_refuses_a_long_one() {
    for (rest, from, anim, left) in [
        (4u8, crate::anim::WALK, crate::anim::ATTACKWALK, 0u8),
        (6, crate::anim::WALK, crate::anim::WALK, 6),
        (4, crate::anim::ATTACKWALK, crate::anim::ATTACKWALK, 4),
    ] {
        let (mut sim, ty) = field(true);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        sim.units[me].guys[0].anim = from;
        sim.units[me].combat.recharging = rest;
        sim.guy_set_anim(me, 0, crate::anim::ATTACKWALK, false, true);
        assert_eq!(sim.units[me].guys[0].anim, anim, "from {from}, rest {rest}");
        assert_eq!(sim.units[me].combat.recharging, left);
    }
}

/// **A guard's attack-move leg goes under the attack its look finds**
/// (`find_nearby_target@00648da0:551`, `local_2c`; run657's `1/69` on
/// 17166: `[Attack, Guard]`, path 0). A bare attack-move keeps its
/// place beneath (`QUEUE_FIRST` over it), which is run658's `1/90` on
/// 17040.
///
/// Made to fail with the kill removed from [`Sim::nearby_add`] (item
/// 1508's M2), with run657 174 against 154 and run661 274 against 254.
#[test]
fn a_guards_attack_move_leg_goes_under_the_attack_it_finds() {
    for guarding in [true, false] {
        let (mut sim, ty) = field(false);
        let post = put(&mut sim, 1, ty, Pos::new(3000, 2000));
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
        if guarding {
            sim.add_guard_order(me, post, 0, 372, QueuePos::New);
            sim.add_move_order(
                me,
                Pos::new(3000, 2400),
                MoveKind::AttackTo,
                QueuePos::First,
                false,
            );
        } else {
            sim.add_move_order(
                me,
                Pos::new(3000, 2400),
                MoveKind::AttackTo,
                QueuePos::New,
                false,
            );
        }
        assert_eq!(sim.order_type(me), index::ATTACK_TO);
        sim.nearby_add(me, Obj::Unit(foe));
        let kinds: Vec<_> = sim.units[me].orders.iter().map(|o| o.index()).collect();
        if guarding {
            assert_eq!(kinds, [index::ATTACK, index::GUARD], "the leg is killed");
        } else {
            assert_eq!(kinds, [index::ATTACK, index::ATTACK_TO], "kept beneath");
        }
        assert!(matches!(sim.units[me].orders[0].body, Body::Attack(_)));
    }
}
