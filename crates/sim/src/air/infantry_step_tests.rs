use crate::world::Pos;
use crate::{Sim, Unit, UnitType, movement};

fn walker(age: i32, flags: u32) -> (Sim, usize) {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(128, 128),
        2,
    );
    let mut ty = UnitType {
        hits: 50,
        moves: 34,
        ..UnitType::default()
    };
    ty.cols.unit_flags = flags;
    ty.combat.age = age;
    let ty = s.add_unit_type(ty);
    let mut u = Unit::new(1, 0, Pos::new(0x60 * 20 + 0x30, 0x60 * 20 + 0x30), 50);
    u.ty = Some(ty);
    u.on_map = true;
    let u = s.add_unit(u);
    let m = &mut s.units[u].movement;
    m.speed = 34;
    m.turning = movement::Turning {
        type_turn_speed: movement::degrees_to_angle(45).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };
    m.set_facing(movement::Angle::EAST);
    (s, u)
}

fn first_step(age: i32, flags: u32) -> i32 {
    let (mut s, u) = walker(age, flags);
    let from = s.units[u].pos;
    s.order_move(u, Pos::new(from.x + 0x60 * 20, from.y));
    for _ in 0..8 {
        s.tick();
        if s.units[u].pos != from {
            break;
        }
    }
    s.units[u].pos.x - from.x
}

/// **The pack** (item 1113, `Unit::do_move`'s `5f82df`-`5f8390`): once
/// every 128 frames, on its own phase `(o · 0x11 + frame) & 0x7f`, a
/// walking Modern Infantry stops, plays `CHAR_PACK` and waits out the
/// animation with `retry` its `end_time` and `attempts` −3. run404's
/// `1/7` (o 7) packs on 777 and `1/6` (o 6) on 794, 128 apart from
/// nothing but their numbers.
#[test]
fn a_walking_modern_infantry_packs_on_its_own_phase() {
    const PIECE: i32 = 7;
    let packs_beside = |age: i32, flags: u32, general: bool| {
        let (mut s, u) = walker(age, flags);
        s.units[u].index = 7;
        if general {
            // A marching General standing where the walk starts.
            let mut ty = UnitType::default();
            ty.cols.unit_flags2 = crate::ai_load::uflags2::GENERAL;
            let ty = s.add_unit_type(ty);
            let mut g = Unit::new(1, 0, s.units[u].pos, 50);
            g.ty = Some(ty);
            g.on_map = true;
            g.marching = Some(i64::MAX);
            s.add_unit(g);
        }
        s.units[u].guys.push(crate::anim::Guy::fresh(PIECE));
        s.art
            .piece_lengths
            .entry(PIECE)
            .or_default()
            .insert(crate::anim::PACK, 22);
        let from = s.units[u].pos;
        s.order_move(u, Pos::new(from.x + 0x60 * 60, from.y));
        let mut seen = Vec::new();
        for _ in 0..300 {
            let f = s.frame;
            let at = s.units[u].pos;
            s.tick();
            let mo = s.units[u].orders.iter().find_map(|o| match &o.body {
                crate::orders::Body::Move(m) => Some((m.retry, m.attempts)),
                _ => None,
            });
            if s.units[u].guys[0].anim == crate::anim::PACK && mo == Some((22, -3)) {
                seen.push((f, at == s.units[u].pos));
            }
        }
        seen
    };
    let packs = |age: i32, flags: u32| packs_beside(age, flags, false);
    // o 7: `7 · 0x11 + 9 = 128`, so frames 9 and 137 of the walk (it
    // has arrived by 265), and the unit stands on each, `retry` 22
    // frames of `CHAR_PACK`.
    assert_eq!(
        packs(6, 0x100),
        [(9, true), (137, true)],
        "a Modern Infantry packs on its own phase"
    );
    assert!(packs(5, 0x100).is_empty(), "not past age 5: it walks on");
    assert!(packs(6, 0).is_empty(), "not the flag: it walks on");
    // `has_general(0x8000, -1) >= 0` (`5f8326`–`5f8336`, item 1614): a
    // General on the march in range keeps it walking through its phase
    // (`docs/AI.md` §172) — frame 9, a few steps out; by 137 it has
    // walked past the General's nine tiles and packs again.
    assert_eq!(
        packs_beside(6, 0x100, true),
        [(137, true)],
        "a marching General in range: it walks on"
    );
}

#[test]
fn a_modern_infantry_steps_five_quarters_of_its_speed() {
    // The step east is the trig component of the step, one short of it:
    // 42 walks 41 and 34 walks 33.
    assert_eq!(first_step(6, 0x100), 41, "34 x 5 / 4 = 42, truncated");
    assert_eq!(first_step(5, 0x100), 33, "not past age 5: 34");
    assert_eq!(first_step(6, 0), 33, "not the flag: 34");
}
