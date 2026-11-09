use crate::attrition::Domain;
use crate::combat::{self, Obj, mask};
use crate::orders::{Body, Order, StrafeOrder};
use crate::world::Pos;
use crate::{Sim, Unit, UnitType};

fn sim() -> Sim {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    s.at_war[0][1] = true;
    s.at_war[1][0] = true;
    s.rng = combat::Rng::new(12345);
    s
}

fn building(s: &mut Sim, who: crate::Player, at: Pos, prof: combat::Profile) -> usize {
    let b = s.add_building(who, at, 0);
    s.buildings[b].started = true;
    s.buildings[b].active = true;
    s.buildings[b].combat = Some(prof);
    s.buildings[b].health = 1200;
    b
}

fn unit(s: &mut Sim, who: crate::Player, at: Pos, prof: combat::Profile) -> usize {
    let ty = s.add_unit_type(UnitType {
        hits: 300,
        kind: crate::attrition::UnitKind {
            domain: prof.domain,
            ..crate::attrition::UnitKind::default()
        },
        combat: prof,
        ..UnitType::default()
    });
    let index = i16::try_from(s.units.len()).unwrap();
    let mut u = Unit::new(who, index, at, 300);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    s.units[u].kind = s.tables.unit_types[ty].kind;
    u
}

/// run223's Bomber: FLY_HIGH 0, FLY_LOW 10, of the air domain.
fn bomber() -> combat::Profile {
    combat::Profile {
        domain: Domain::Air,
        fly_high: 0,
        fly_low: 10,
        ..combat::Profile::default()
    }
}

fn strafe(target: Option<Obj>, home: Option<usize>, returning: bool) -> Order {
    Order {
        flags: 0,
        body: Body::Strafe(StrafeOrder {
            target,
            mandatory: false,
            home,
            cruising_alt: crate::orders::CRUISING_ALT,
            sharp_turn: 0,
            returning,
            at: None,
        }),
    }
}

/// run404's Radar Air Defense: the tower columns of `harness_tests`'
/// tower, `ANTI_AIR`, range 10, FLY 33/75, and its type's cycle — the
/// Unpack's 20 frames, the Attack1's 10, the one release at frame 2 on
/// node 0, and the measured launch vector.
fn radar(s: &mut Sim, with_cycle: bool) -> usize {
    let b = building(
        s,
        1,
        Pos::new(22272, 16512),
        combat::Profile {
            obj_masks: mask::ANTI_AIR,
            attack: 80,
            recharge: 60,
            max_range: 10,
            to_hit: 400,
            proj_speed: 200,
            ammo_per_att: 1,
            base_arrows: 1,
            most_shots: 4,
            x_size: 2,
            y_size: 2,
            big_radius: 96,
            fly_high: 33,
            fly_low: 75,
            ..combat::Profile::default()
        },
    );
    if with_cycle {
        let mut d = crate::tech::TypeDef::building("Radar Air Defense");
        d.wall_cycle = Some(crate::air::WallCycle {
            wind: 20,
            swing: 10,
            releases: vec![(2, 0, false)],
            launch: Some((136, -136, 227)),
        });
        let id = s.tables.tech_tree.add(d);
        let bt = s.add_build_type(crate::build::BuildType {
            tree: Some(id),
            ..crate::build::BuildType::default()
        });
        s.buildings[b].ty = Some(bt);
    }
    b
}

/// **An anti-air building does not fire from `do_attack`** (item 1112,
/// the call site): `Build::do_attack`'s in-range arm returns before
/// `Object::fire_ammo` for an `ANTI_AIR` building that is neither a
/// Lookout nor an Observation Post, and its head leaves `recharging`
/// alone. The same building without the cycle fires at once.
#[test]
fn an_anti_air_building_takes_its_target_and_does_not_fire_it() {
    for with_cycle in [true, false] {
        let mut s = sim();
        let r = radar(&mut s, with_cycle);
        let plane = unit(&mut s, 0, Pos::new(21600, 16400), bomber());
        s.buildings[r].recharging = 5;
        s.buildings[r].target = Some(Obj::Unit(plane));
        s.process_building_combat(r, 16);
        if with_cycle {
            assert_eq!(s.buildings[r].target, Some(Obj::Unit(plane)));
            assert!(s.combat.projectiles.is_empty(), "no round from do_attack");
            assert_eq!(s.buildings[r].recharging, 5, "no countdown");
        } else {
            assert_eq!(s.buildings[r].recharging, 4, "the countdown");
            s.buildings[r].recharging = 0;
            s.process_building_combat(r, 16);
            assert_eq!(s.combat.projectiles.len(), 1, "fired from do_attack");
        }
    }
}

/// **The Radar Air Defense's cycle** (item 1112, `docs/COMBAT.md`
/// §84): run404's `1/2007` from its acquisition on trace frame 777 —
/// `recharging` 1..19, then −1..−10 and round again, a round on each
/// frame the swing's clock crosses 2 (trace frames 797, 807, …), from
/// (22408, 16376, 235) with `num_guys` 0; with the target gone, −4
/// becomes 18 (block 1021) and counts down to 0.
#[test]
fn an_anti_air_building_winds_up_and_fires_on_its_swing() {
    let mut s = sim();
    let r = radar(&mut s, true);
    let plane = unit(&mut s, 0, Pos::new(21600, 16400), bomber());
    s.walls_inc_time(1);
    assert_eq!(s.buildings[r].recharging, 0, "no target: held at 0");
    s.buildings[r].target = Some(Obj::Unit(plane));
    let mut seen = Vec::new();
    let mut rounds = Vec::new();
    for f in 0..33 {
        s.frame = f;
        let before = s.combat.projectiles.len();
        s.walls_inc_time(1);
        seen.push(s.buildings[r].recharging);
        if s.combat.projectiles.len() > before {
            rounds.push(f);
        }
    }
    assert_eq!(seen[..19], (1..=19).collect::<Vec<i32>>()[..]);
    assert_eq!(seen[19..29], (1..=10).map(|k| -k).collect::<Vec<i32>>()[..]);
    assert_eq!(seen[29..33], [-1, -2, -3, -4]);
    assert_eq!(rounds, [20, 30], "on the swing's frame 2");
    let p = s.combat.projectiles[0];
    assert_eq!(p.launch, Pos::new(22408, 16376));
    assert_eq!(p.num_guys, 0);
    assert_eq!(p.shooter, Obj::Building(r));
    s.buildings[r].target = None;
    s.walls_inc_time(1);
    assert_eq!(s.buildings[r].recharging, 18, "−4 to 19, and down one");
    for _ in 0..30 {
        s.walls_inc_time(1);
    }
    assert_eq!(s.buildings[r].recharging, 0, "down to 0, and held");
    s.walls_inc_time(0);
    assert_eq!(s.buildings[r].recharging, 0, "another leader's pass");
}

/// **An anti-air unit keeps its angle at an aircraft, and only there**
/// (item 1109): the Battery at a Bomber does; at a ground unit it does
/// not, and a shooter without `ANTI_AIR` does not at a Bomber.
#[test]
fn an_anti_air_unit_keeps_its_angle_at_an_aircraft_only() {
    let mut s = sim();
    let aa = combat::Profile {
        obj_masks: mask::ANTI_AIR,
        max_range: 17,
        ..combat::Profile::default()
    };
    let battery = unit(&mut s, 1, Pos::new(21384, 17256), aa);
    let rifle = unit(
        &mut s,
        1,
        Pos::new(21384, 17456),
        combat::Profile::default(),
    );
    let plane = unit(&mut s, 0, Pos::new(22000, 16000), bomber());
    let foot = unit(
        &mut s,
        0,
        Pos::new(22000, 18000),
        combat::Profile::default(),
    );
    assert!(s.anti_air_keeps_angle(battery, Obj::Unit(plane)));
    assert!(!s.anti_air_keeps_angle(battery, Obj::Unit(foot)));
    assert!(!s.anti_air_keeps_angle(rifle, Obj::Unit(plane)));
}

/// **The strike keeps the heading** (item 1109, the call site): a
/// Battery facing away from a Bomber and with no pivots fires without
/// turning, where it turns to a ground target.
#[test]
fn an_anti_air_strike_at_a_plane_does_not_turn_the_unit() {
    let mut s = sim();
    let aa = combat::Profile {
        obj_masks: mask::ANTI_AIR,
        max_range: 17,
        ..combat::Profile::default()
    };
    let battery = unit(&mut s, 1, Pos::new(21384, 17256), aa);
    let plane = unit(&mut s, 0, Pos::new(19000, 17256), bomber());
    let foot = unit(
        &mut s,
        0,
        Pos::new(19000, 17256),
        combat::Profile::default(),
    );
    let east = crate::movement::Angle::EAST;
    s.units[battery].movement.heading = east;
    s.fight_pub(battery, Obj::Unit(plane), 0);
    assert_eq!(s.units[battery].movement.heading, east, "kept at a plane");
    s.fight_pub(battery, Obj::Unit(foot), 1);
    assert_ne!(s.units[battery].movement.heading, east, "turned to a man");
}

/// **`is_flying_low` is a distance to the strike** (item 1102): the
/// emulated original's table — 1 at 2303 and 0 at 2304 on the strafe's
/// arm and on a returning plane's home, 0 on the diagonal (1600, 1600),
/// 0 with `returning` clear, and never inside or for a missile.
#[test]
fn a_plane_flies_low_within_0x900_of_its_strike_or_its_home_returning() {
    let mut s = sim();
    let t = building(
        &mut s,
        1,
        Pos::new(21120, 16512),
        combat::Profile::default(),
    );
    let home = building(
        &mut s,
        0,
        Pos::new(11616, 13920),
        combat::Profile::default(),
    );
    let b = unit(&mut s, 0, Pos::new(21120 - 2303, 16512), bomber());
    s.units[b]
        .orders
        .push_back(strafe(Some(Obj::Building(t)), Some(home), false));
    assert!(s.is_flying_low(b), "2303 off its target");
    assert!(!s.is_flying_high(b));
    s.units[b].pos = Pos::new(21120 - 2304, 16512);
    assert!(!s.is_flying_low(b), "2304 off its target");
    assert!(s.is_flying_high(b));
    s.units[b].pos = Pos::new(21120 - 1600, 16512 - 1600);
    assert!(
        !s.is_flying_low(b),
        "the diagonal (1600, 1600) is past the reach"
    );
    s.buildings[t].alive = false;
    s.units[b].pos = Pos::new(21120, 16512);
    assert!(!s.is_flying_low(b), "a dead target is not a strike");
    s.units[b].orders.clear();
    s.units[b].orders.push_back(strafe(None, Some(home), true));
    s.units[b].pos = Pos::new(11616 + 2303, 13920);
    assert!(s.is_flying_low(b), "returning, 2303 off its home");
    s.units[b].pos = Pos::new(11616 + 2304, 13920);
    assert!(!s.is_flying_low(b), "returning, 2304 off its home");
    s.units[b].orders.clear();
    s.units[b].orders.push_back(strafe(None, Some(home), false));
    s.units[b].pos = Pos::new(11616, 13920);
    assert!(!s.is_flying_low(b), "not returning");
    s.units[b].orders.clear();
    assert!(!s.is_flying_low(b), "no order");
}

/// **An `ANTI_AIR` shooter rolls once, against its own figure by the
/// target's altitude** (item 1102): run under `tools/emu/flak_arm.py`,
/// a Battery (50/90) hits a low Bomber on 89 and misses on 90, and a
/// high one hits on 49 and misses on 50. Here the Radar Air Defense's
/// 33/75 on this crate's stream, one draw a round.
#[test]
fn an_anti_air_round_draws_once_against_its_own_fly_figure() {
    let mut s = sim();
    let t = building(
        &mut s,
        1,
        Pos::new(21120, 16512),
        combat::Profile::default(),
    );
    let radar = building(
        &mut s,
        0,
        Pos::new(22272, 16512),
        combat::Profile {
            obj_masks: mask::ANTI_AIR,
            fly_high: 33,
            fly_low: 75,
            ..combat::Profile::default()
        },
    );
    let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
    s.units[b]
        .orders
        .push_back(strafe(Some(Obj::Building(t)), None, false));
    for (at, figure) in [(21120, 75), (21120 - 5000, 33)] {
        s.units[b].pos = Pos::new(at, 16512);
        let mut probe = s.rng;
        let r = probe.roll() % 100;
        let missed = s.air_round_misses(Obj::Building(radar), Obj::Unit(b));
        assert_eq!(missed, Some(r >= figure), "roll {r} against {figure}");
        assert_eq!(s.rng.seed, probe.seed, "one draw");
    }
    // An ANTI_AIR shooter of the air domain takes no roll.
    let fighter = unit(
        &mut s,
        0,
        Pos::new(20000, 16512),
        combat::Profile {
            obj_masks: mask::ANTI_AIR,
            domain: Domain::Air,
            fly_low: 25,
            ..combat::Profile::default()
        },
    );
    let seed = s.rng.seed;
    assert_eq!(
        s.air_round_misses(Obj::Unit(fighter), Obj::Unit(b)),
        Some(false)
    );
    assert_eq!(s.rng.seed, seed, "no draw");
    // A ground target takes no arm at all.
    assert_eq!(
        s.air_round_misses(Obj::Building(radar), Obj::Building(t)),
        None
    );
}

/// **An `ANTI_AIR` attacker spends the jam roll, and nothing else
/// does** (item 1102): `Unit::fight`'s `GameAccess::rnd(100)` at
/// `5fee89`, run404's 742. No jammer exists here, so it never jams.
#[test]
fn an_anti_air_attacker_rolls_its_radar_and_no_other_does() {
    let mut s = sim();
    let aa = unit(
        &mut s,
        1,
        Pos::new(22392, 16632),
        combat::Profile {
            obj_masks: mask::ANTI_AIR,
            ..combat::Profile::default()
        },
    );
    let inf = unit(
        &mut s,
        1,
        Pos::new(20000, 16632),
        combat::Profile::default(),
    );
    let seed = s.rng.seed;
    assert!(!s.radar_jams(inf));
    assert_eq!(s.rng.seed, seed, "no draw for a type without ANTI_AIR");
    let mut probe = s.rng;
    let _ = probe.roll();
    assert!(!s.radar_jams(aa), "no jammer, no jam");
    assert_eq!(s.rng.seed, probe.seed, "one draw");
}

/// **A plane shot down draws its crash's roll and lays no death
/// object** (item 1102): `kill_guy`'s arm for `CAT` Air; a missile and
/// any other category take `add_death`.
#[test]
fn a_plane_of_cat_air_crashes_and_a_missile_does_not() {
    let mut s = sim();
    let b = unit(&mut s, 0, Pos::new(21120, 16512), bomber());
    let ty = s.units[b].ty.unwrap();
    s.tables.unit_types[ty].cols.cat = super::CAT_AIR;
    let mut probe = s.rng;
    let _ = probe.roll();
    assert!(s.crashes(b));
    assert_eq!(s.rng.seed, probe.seed, "one draw");
    let seed = s.rng.seed;
    let missile = unit(
        &mut s,
        0,
        Pos::new(21120, 16512),
        combat::Profile {
            obj_masks: mask::MISSILE,
            domain: Domain::Air,
            ..combat::Profile::default()
        },
    );
    let mty = s.units[missile].ty.unwrap();
    s.tables.unit_types[mty].cols.cat = super::CAT_AIR;
    assert!(!s.crashes(missile), "a missile takes add_death");
    let foot = unit(
        &mut s,
        0,
        Pos::new(21120, 16512),
        combat::Profile::default(),
    );
    assert!(!s.crashes(foot), "Foot takes add_death");
    assert_eq!(s.rng.seed, seed, "no draw for either");
}

/// **A plane shot down lays its crash round, and the round holds its
/// number** (item 2609, `docs/AI.md` §228): run405's two Bombers, from
/// their own figures — `0/6` at (21949, 18537, 1783) and `0/7` at
/// (22866, 16298, 1624), `moves` 60. The dump's first block prints each
/// landing one bank step on, so (22967, 19002) less (23, 5) and (21881,
/// 15895) less (0, 1).
#[test]
fn a_plane_shot_down_lays_the_crash_round_run405_prints() {
    let mut s = sim();
    for (at, z, facing, want, total, bank) in [
        (
            Pos::new(21949, 18537),
            1783,
            1_369_137_312,
            Pos::new(22944, 18997),
            18,
            (23, 5),
        ),
        (
            Pos::new(22866, 16298),
            1624,
            -806_122_848,
            Pos::new(21881, 15894),
            17,
            (0, 1),
        ),
    ] {
        let b = unit(&mut s, 0, at, bomber());
        let ty = s.units[b].ty.unwrap();
        s.tables.unit_types[ty].cols.cat = super::CAT_AIR;
        s.tables.unit_types[ty].moves = 60;
        s.units[b].movement.body.pos = at;
        s.units[b].movement.facing = crate::movement::Angle(facing);
        s.units[b].airframe.z = z;
        let mut probe = s.rng;
        let _ = probe.roll();
        assert!(s.crashes(b));
        assert_eq!(s.rng.seed, probe.seed, "one draw from the game's stream");
        let p = *s
            .combat
            .projectiles
            .iter()
            .find(|p| p.shooter == Obj::Unit(b))
            .expect("the crash round");
        assert_eq!((p.landing, p.total_time, p.sz), (want, total, z));
        assert_eq!(p.crash.map(|c| c.bank), Some(bank));
        assert_eq!(p.target, None);
        // `Object::die`'s tail: the round's remaining time over the
        // close's thirty.
        s.hold_dead_slot(b);
        assert_eq!(s.units[b].hold_frames, Sim::NUKE_SPREAD + 1 + total);
    }
}

/// **A crash round strikes where it comes down** (item 2677,
/// `docs/AI.md` §230): `init_crash`'s `splash_area` 2 and its cleared
/// `0x10` send it through `Ammo::do_damage`'s splash arm when it lands,
/// and the splash strikes a building that stands — at its ceiling too,
/// with no hit points left (run866's capital `0/2000`, struck on 5205
/// by `1/114`'s crash). Made to fail by the crash arm closing its round
/// without `land`, and by the splash asking `active` of a building.
#[test]
fn a_crash_round_strikes_the_building_it_comes_down_on() {
    for spent in [false, true] {
        let mut s = sim();
        let at = Pos::new(12000, 12000);
        let t = building(
            &mut s,
            0,
            at,
            combat::Profile {
                x_size: 2,
                y_size: 2,
                ..combat::Profile::default()
            },
        );
        if spent {
            // At its ceiling: standing, and no hit points left.
            s.buildings[t].health = 0;
            s.buildings[t].damage = s.buildings[t].hits_now();
        }
        let before = (s.buildings[t].damage, s.buildings[t].hit_frame);
        let b = unit(
            &mut s,
            1,
            at,
            combat::Profile {
                attack: 40,
                ..bomber()
            },
        );
        let ty = s.units[b].ty.unwrap();
        s.tables.unit_types[ty].cols.cat = super::CAT_AIR;
        s.tables.unit_types[ty].moves = 60;
        s.units[b].movement.body.pos = at;
        s.units[b].airframe.z = 300;
        assert!(s.crashes(b));
        assert_eq!(
            s.combat.projectiles[0].splash_area,
            super::CRASH_SPLASH_AREA
        );
        let mut frame = 100;
        while !s.combat.projectiles.is_empty() && frame < 200 {
            s.process_projectiles(frame);
            frame += 1;
        }
        assert!(s.combat.projectiles.is_empty(), "the round closes");
        assert_ne!(
            (s.buildings[t].damage, s.buildings[t].hit_frame),
            before,
            "struck where it came down (at its ceiling: {spent})"
        );
    }
}

/// **A fired round at a plane carries the roll's miss** (item 1102):
/// `Object::fire_ammo`'s round takes `Ammo::init`'s air arm, whose miss
/// is the round's `0x10`, this crate's `harmless` — run404's Battery
/// round of 753 (flags 18, a miss) against a hit.
#[test]
fn a_round_fired_at_a_plane_is_harmless_when_its_roll_misses() {
    let mut s = sim();
    let t = building(
        &mut s,
        1,
        Pos::new(21120, 16512),
        combat::Profile::default(),
    );
    let radar = building(
        &mut s,
        0,
        Pos::new(22272, 16512),
        combat::Profile {
            obj_masks: mask::ANTI_AIR,
            fly_high: 33,
            fly_low: 75,
            max_range: 10 * 192,
            ..combat::Profile::default()
        },
    );
    let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
    s.units[b]
        .orders
        .push_back(strafe(Some(Obj::Building(t)), None, false));
    let (mut hits, mut misses) = (0, 0);
    for seed in 1..60u32 {
        s.rng = combat::Rng::new(seed);
        let mut probe = s.rng;
        let missed = probe.roll() % 100 >= 75;
        s.combat.projectiles.clear();
        s.fire_ammo_pub(
            Obj::Building(radar),
            Obj::Unit(b),
            crate::movement::Angle(0),
            800,
            Pos::new(22272, 16512),
            235,
            0,
            false,
            false,
        );
        assert_eq!(s.combat.projectiles.len(), 1);
        assert_eq!(s.combat.projectiles[0].harmless, missed, "seed {seed}");
        if missed { misses += 1 } else { hits += 1 }
    }
    assert!(
        hits > 0 && misses > 0,
        "both arms reached: {hits} hits, {misses} misses"
    );
}

/// **A shooter that is not `ANTI_AIR` rolls against the target's figure
/// first** (item 1102): the emulator's Infantry (0/33) at a low Bomber
/// — rolls 9 and 32 hit, 9 and 33 miss, a first roll of 10 misses with
/// no second draw.
#[test]
fn any_other_shooter_rolls_the_target_s_figure_before_its_own() {
    let mut s = sim();
    let t = building(
        &mut s,
        1,
        Pos::new(21120, 16512),
        combat::Profile::default(),
    );
    let inf = unit(
        &mut s,
        0,
        Pos::new(20000, 16512),
        combat::Profile {
            fly_low: 33,
            ..combat::Profile::default()
        },
    );
    let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
    s.units[b]
        .orders
        .push_back(strafe(Some(Obj::Building(t)), None, false));
    for seed in 1..400u32 {
        s.rng = combat::Rng::new(seed);
        let mut probe = s.rng;
        let r1 = probe.roll() % 100;
        let want = if r1 >= 10 {
            true
        } else {
            probe.roll() % 100 >= 33
        };
        assert_eq!(s.air_round_misses(Obj::Unit(inf), Obj::Unit(b)), Some(want));
        assert_eq!(
            s.rng.seed, probe.seed,
            "seed {seed}: one draw, or two past the first"
        );
    }
}

/// A Bomber that strikes: run404's `0/7`, with an attack so
/// `get_damage` weighs something.
fn striking_bomber() -> combat::Profile {
    combat::Profile {
        attack: 100,
        max_range: 3,
        min_range: 1,
        cost: 300,
        ..bomber()
    }
}

/// **An armed building's ×5 is a computer attacker's** (item 1131,
/// `compare_target`'s `64f1a2`: `testb $0x4, leader_flags` of the
/// attacker, `jne` past the weight). run404's human `0/7` weighs the
/// Radar Air Defense at a fifth of what a computer's Bomber does, and
/// that is what keeps its strafe on the Barracks on trace tick 818.
#[test]
fn a_human_s_bomber_weighs_an_armed_building_without_the_computer_s_five() {
    let weigh = |human: bool| {
        let mut s = sim();
        s.cities_state.nation[0].human = human;
        let r = radar(&mut s, true);
        if let Some(p) = s.buildings[r].combat.as_mut() {
            p.cost = 250;
        }
        let plane = unit(&mut s, 0, Pos::new(22456, 16287), striking_bomber());
        s.compare_target(Obj::Unit(plane), Obj::Building(r), true, false)
    };
    let (human, computer) = (weigh(true), weigh(false));
    assert!(human > 15, "a weight above the floor: {human}");
    assert!(
        computer >= 4 * human,
        "the computer's ×5: {computer} against {human}"
    );
}

/// **No object stands below zero** (item 1131, `Unit::update_z`'s
/// `pushl $0x1` at `00606598`): a plane over a tile whose height is
/// negative presents `z` 0 to `get_damage`, so the river step (`T.z <
/// 0`, ×2) never doubles a hit on it. run404's `0/7` over the river
/// east of the Barracks took 32 from the Radar where ours dealt 64.
#[test]
fn a_plane_over_a_tile_below_zero_takes_no_river_doubling() {
    let hit = |z: i32| {
        let mut s = sim();
        let r = radar(&mut s, true);
        let at = Pos::new(22512, 16266);
        s.world.set_tile_z(at.tile(), z);
        let plane = unit(&mut s, 0, at, striking_bomber());
        s.do_damage(
            Obj::Building(r),
            Obj::Unit(plane),
            crate::movement::Angle(0),
            true,
            0x100,
            false,
            true,
            800,
        );
        300 - s.units[plane].health
    };
    assert!(hit(0) > 0, "the Radar hurts it");
    assert_eq!(hit(-6), hit(0), "z below zero is z 0");
    let mut w = crate::world::World::new(8, 8);
    let t = Pos::new(3, 3);
    w.set_tile_z(t, -6);
    assert_eq!(w.object_z(t), 0);
    w.set_tile_z(t, 17);
    assert_eq!(w.object_z(t), 17);
    w.set_tile_field(
        t,
        crate::world::tile::SURFACE,
        crate::world::tile::SURFACE_OCEAN,
    );
    assert_eq!(w.object_z(t), 0, "an ocean tile is 0");
}

/// **A building's hit records its own `o`** (item 1131,
/// `ObjectData::get_captain@00472400`): run404's `0/7` reads `damage_o
/// 2007` from the Radar's round landing on 828.
#[test]
fn a_building_s_hit_records_its_own_object_number() {
    let mut s = sim();
    let r = radar(&mut s, true);
    let plane = unit(&mut s, 0, Pos::new(22512, 16266), striking_bomber());
    s.do_damage(
        Obj::Building(r),
        Obj::Unit(plane),
        crate::movement::Angle(0),
        true,
        0x100,
        false,
        false,
        828,
    );
    assert_eq!(
        s.units[plane].combat.damage_o,
        i32::from(s.buildings[r].index)
    );
}

/// **An unordered building re-finds its target on every `do_attack`**
/// (item 1131, `622a3f`): a moving target weighs a quarter, so the
/// Radar leaves it for a still plane beside it on a frame that is not
/// its phase. run404's `1/2007` turns from `0/7` to `0/6` on trace
/// tick 836.
#[test]
fn an_unordered_building_re_finds_its_target_on_every_call() {
    let mut s = sim();
    let r = radar(&mut s, true);
    let a = unit(&mut s, 0, Pos::new(22900, 16000), striking_bomber());
    let b = unit(&mut s, 0, Pos::new(22900, 17000), striking_bomber());
    s.units[a].movement.dest = Some(Pos::new(30000, 16000));
    s.buildings[r].target = Some(Obj::Unit(a));
    let frame = 16;
    assert_ne!(
        (s.buildings[r].phase(frame) + 14) & 0x1f,
        0,
        "not the phase's frame"
    );
    s.process_building_combat(r, frame);
    assert_eq!(s.buildings[r].target, Some(Obj::Unit(b)));
}

/// **A building's target outlives its death** (item 1131): run404's
/// Radar still holds `attack_ox 6` on block 1007, after `0/6` was shot
/// down on 1006, so `Build::process` runs `do_attack` on tick 1007 and
/// its `find_target` turns it to `0/7`.
/// A stockade-shaped building: range 11, a ground unit 13 tiles away.
fn stockade_and_scout(s: &mut Sim) -> (usize, usize) {
    let b = building(
        s,
        1,
        Pos::new(22272, 16512),
        combat::Profile {
            attack: 12,
            recharge: 30,
            max_range: 11,
            to_hit: 110,
            proj_speed: 200,
            ammo_per_att: 1,
            base_arrows: 1,
            most_shots: 2,
            x_size: 2,
            y_size: 2,
            ..combat::Profile::default()
        },
    );
    let sc = unit(
        s,
        0,
        Pos::new(22272 + 13 * 192 + 192 + 72, 16512),
        combat::Profile {
            domain: Domain::Land,
            ..combat::Profile::default()
        },
    );
    (b, sc)
}

/// **`get_building_range`'s British term** (item 1415, `docs/GOLDEN.md`
/// §59): a British owner's Tower-line building reaches `BRITISH_TOWER_RANGE`
/// further; anyone else's, or a non-tower's, does not.
#[test]
fn a_british_tower_line_building_reaches_two_tiles_further() {
    let mut s = sim();
    let (b, _) = stockade_and_scout(&mut s);
    let bt = s.add_build_type(crate::build::BuildType {
        ident: crate::build::Ident::Tower,
        ..crate::build::BuildType::default()
    });
    s.buildings[b].ty = Some(bt);
    let me = Obj::Building(b);
    assert_eq!(s.max_range_of(me), 11, "not British");
    s.cities_state.nation[1].british = true;
    assert_eq!(s.max_range_of(me), 13, "British Tower line");
    let other = s.add_build_type(crate::build::BuildType::default());
    s.buildings[b].ty = Some(other);
    assert_eq!(s.max_range_of(me), 11, "British, not a tower");
}

/// **`max_range`'s Turkish siege term** (item 3380): a Turkish leader's
/// siege unit reaches `TURK_SIEGE_RANGE` further; another nation's, or a
/// Turkish unit that is not siege, does not. `GreatLakesTurkishTier4`'s
/// Trebuchet (16) stands its bombard ring at 19 tiles round Napata.
#[test]
fn a_turkish_siege_unit_reaches_three_tiles_further() {
    let mut s = sim();
    let siege = combat::Profile {
        domain: Domain::Land,
        max_range: 16,
        siege: true,
        ..combat::Profile::default()
    };
    let treb = Obj::Unit(unit(&mut s, 1, Pos::new(8532, 29381), siege));
    let archer = Obj::Unit(unit(
        &mut s,
        1,
        Pos::new(8000, 29000),
        combat::Profile {
            domain: Domain::Land,
            max_range: 16,
            ..combat::Profile::default()
        },
    ));
    assert_eq!(s.max_range_of(treb), 16, "not Turkish");
    s.cities_state.nation[1].turks = true;
    assert_eq!(s.max_range_of(treb), 19, "a Turkish siege unit");
    assert_eq!(s.max_range_of(archer), 16, "Turkish, not siege");
}

/// **`Build::process`'s early trigger** (item 1415): a building with no
/// target runs `do_attack` off its 32-frame phase when `near_o` is in
/// range.
#[test]
fn a_building_with_a_sighting_in_range_attacks_off_its_phase() {
    let mut s = sim();
    let (b, sc) = stockade_and_scout(&mut s);
    s.units[sc].pos = Pos::new(22272 + 8 * 192, 16512);
    let frame = (5 - i64::from(s.buildings[b].index)).rem_euclid(32);
    s.process_building_combat(b, frame);
    assert_eq!(s.buildings[b].target, None, "no sighting, off phase");
    s.buildings[b].near = Some(Obj::Unit(sc));
    s.process_building_combat(b, frame);
    assert_eq!(
        s.buildings[b].target,
        Some(Obj::Unit(sc)),
        "sighting in range"
    );
}

/// **`Build::do_attack`'s `visible` write** (item 1419, `docs/GOLDEN.md`
/// §59, `622b5b`..`622bd0`): a round fired sets the target owner's bit
/// on the building, which is `BuildData::is_seen`'s first arm; the
/// 32-frame phase with the latch down clears it.
#[test]
fn a_building_that_fires_shows_itself_to_the_player_it_shot() {
    let mut s = sim();
    let (b, sc) = stockade_and_scout(&mut s);
    s.units[sc].pos = Pos::new(22272 + 8 * 192, 16512);
    s.buildings[b].near = Some(Obj::Unit(sc));
    let frame = (5 - i64::from(s.buildings[b].index)).rem_euclid(32);
    assert_eq!(s.buildings[b].visible, 0);
    s.process_building_combat(b, frame);
    assert!(!s.combat.projectiles.is_empty(), "it fired");
    assert_eq!(s.buildings[b].visible, 1, "the Scout's owner's bit");
    assert!(s.buildings[b].attacking);
    assert!(s.build_is_seen(b, 0), "seen by who it shot");
    assert!(!s.build_is_seen(b, 2), "and by nobody else");
    // The next phase call, with nothing to shoot: the latch holds the
    // bit and drops; the one after clears it.
    let phase0 = (32 - i64::from(s.buildings[b].index)).rem_euclid(32);
    s.buildings[b].recharging = 0;
    s.buildings[b].target = None;
    s.buildings[b].near = None;
    s.units[sc].pos = Pos::new(0, 0);
    s.process_building_combat(b, phase0);
    assert_eq!(s.buildings[b].visible, 1, "the latch held it");
    s.process_building_combat(b, phase0 + 32);
    assert_eq!(
        s.buildings[b].visible, 0,
        "cleared on the phase, latch down"
    );
}

/// **A building's search must reach what it takes** (item 4004,
/// `docs/GOLDEN.md` §192, `find_nearby_target@00648da0`'s `local_24`):
/// its current target walking out of range is skipped, not kept on its
/// current-target `×2`, so the building takes the one in range and
/// fires on the same call. run1545's capital `0/2000` left `1/14` for
/// `1/12` on game frame 911.
#[test]
fn a_building_s_search_skips_a_unit_out_of_its_range() {
    let mut s = sim();
    let (b, gone) = stockade_and_scout(&mut s);
    let near = unit(
        &mut s,
        0,
        Pos::new(22272 + 8 * 192, 16512),
        combat::Profile {
            domain: Domain::Land,
            ..combat::Profile::default()
        },
    );
    for u in [gone, near] {
        let t = s.units[u].ty.unwrap();
        s.tables.unit_types[t].combat.cost = 20_000;
    }
    // The departing one inside the search's radius and out of range;
    // the near one walks (`/4`); the departing one is the target (`×2`).
    let me = Obj::Building(b);
    let edge = (2000..3000)
        .find(|&dx| {
            s.units[gone].pos = Pos::new(22272 + dx, 16512);
            !s.is_in_range(me, Obj::Unit(gone))
        })
        .unwrap();
    s.units[gone].pos = Pos::new(22272 + edge, 16512);
    assert!(
        s.attack_dist(Obj::Building(b), Obj::Unit(gone)) <= (2 + 2 * 11) * 0x60,
        "a candidate"
    );
    s.units[near].movement.dest = Some(Pos::new(22272 + 9 * 192, 16512));
    s.buildings[b].target = Some(Obj::Unit(gone));
    assert!(!s.is_in_range(me, Obj::Unit(gone)), "the departing unit");
    assert!(s.is_in_range(me, Obj::Unit(near)), "the near one");
    let phase0 = (32 - i64::from(s.buildings[b].index)).rem_euclid(32);
    s.process_building_combat(b, phase0);
    assert_eq!(s.buildings[b].target, Some(Obj::Unit(near)), "taken");
    assert!(
        !s.combat.projectiles.is_empty(),
        "and fired on the same call"
    );
}

#[test]
fn a_building_keeps_a_dead_target_until_do_attack_replaces_it() {
    let mut s = sim();
    let r = radar(&mut s, true);
    let a = unit(&mut s, 0, Pos::new(22900, 16000), striking_bomber());
    let b = unit(&mut s, 0, Pos::new(22900, 17000), striking_bomber());
    s.buildings[r].target = Some(Obj::Unit(a));
    s.units[a].health = 0;
    s.forget(Obj::Unit(a));
    assert_eq!(s.buildings[r].target, Some(Obj::Unit(a)), "kept");
    s.process_building_combat(r, 16);
    assert_eq!(s.buildings[r].target, Some(Obj::Unit(b)), "replaced");
}
