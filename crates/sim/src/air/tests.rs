use super::*;

#[test]
fn the_owed_angle_is_a_bitwise_not_and_not_a_negation() {
    // Exactly half a turn: the original's `~d` gives `0x7fffffff` for
    // `0x80000000`... except that the compare is `<`, so half a turn is
    // *not* folded and stays `0x80000000`.
    assert_eq!(owed(Angle(0), Angle(i32::MIN)), 0x8000_0000);
    assert_eq!(owed(Angle(0), Angle(0x4000_0000)), 0x4000_0000);
    // One past half a turn folds, and to one less than half rather than
    // to the exact mirror — `~0x80000001` is `0x7ffffffe`.
    assert_eq!(owed(Angle(0), Angle(i32::MIN + 1)), 0x7fff_fffe);
}

#[test]
fn the_turn_rate_is_the_banks_magnitude_over_fifty_five() {
    // The bird's own: `TURN_SPEED 5` through `degrees_to_angle`, with
    // its low byte cleared.
    let raw = ((crate::movement::degrees_to_angle(5).0 as u32) >> 8) * 256;
    assert_eq!(raw, 0x038e_3800);
    // A level bird turns at the floor.
    assert_eq!(air_turn_speed(raw, Single::ZERO, 1), MIN_TURN);
    // Banked hard the wrong way for the sign asked, it also turns at the
    // floor — the frame a bank reverses is a frame at half a degree.
    assert_eq!(air_turn_speed(raw, F55, 1), MIN_TURN);
    // Banked fully, it turns at the type's whole rate.
    assert_eq!(air_turn_speed(raw, F55.neg(), 1), (raw / 0x37) * 55);
    // And a shallow bank is still floored: five degrees of bank buys
    // less than half a degree of turn.
    assert!(air_turn_speed(raw, F5.neg(), 1) == MIN_TURN);
}

/// A bird on the shipped `WILDBIRD` numbers, put where the test wants
/// it and pointed where the test wants it pointed.
fn bird(at: crate::world::Pos, heading: i32) -> (crate::Sim, usize) {
    let mut s = crate::Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    s.rng = crate::combat::Rng::new(7);
    let ty = s.add_unit_type(crate::UnitType {
        hits: 1,
        moves: 35,
        turn_speed: crate::movement::degrees_to_angle(5).0,
        kind: crate::attrition::UnitKind {
            domain: crate::attrition::Domain::Air,
            ..crate::attrition::UnitKind::default()
        },
        ..crate::UnitType::default()
    });
    let mut u = crate::Unit::new(crate::gaia::BIRD_OWNER, 0, at, 1);
    u.ty = Some(ty);
    u.kind = s.tables.unit_types[ty].kind;
    u.type_index = crate::anim::BIRD_TYPE;
    u.movement.speed = 35;
    u.movement.turning = crate::turning_of(&s.tables.unit_types[ty]);
    u.movement.heading = Angle(heading);
    let b = s.add_unit(u);
    // The figure, because `Guy::move` runs for a bird like anybody
    // else and the arrival stand is its own draw.
    s.init_guys(b, Some(ty));
    s.gaia.gaia.bird_goals.push((b, at));
    (s, b)
}

/// **The figure lags its unit by a step, and that is what the arrival
/// stand reads.** `Unit::do_air_physics` ends in `set_new_location(x,
/// y, 0, 1)` — `param_3` zero (`005e86d0:237`) — so
/// `Unit::set_new_location@005f8d20:158` writes guy 0's `des` to the
/// unit's new point and `:161`'s teleport is skipped: the figure is
/// still on last frame's point when `Guy::process` runs, and takes the
/// **moving** arm. A bird pinned on the boundary, whose step
/// `WorldData::restrict` hands back unchanged, is at its `des` instead
/// and takes the arrival one.
///
/// Made to fail on purpose both ways: with `move_guys` back at `true`
/// the figure never lags and the flying frame pays a stand it should
/// not; with `guys_follow`'s old air-gaia return in place the pinned
/// frame pays none.
#[test]
fn a_bird_s_figure_lags_its_unit_and_stands_only_when_the_step_is_refused() {
    // Pointed due west, sixteen units from the edge, with the patrol
    // point behind it so the heading stands.
    let at = crate::world::Pos::new(16, 20_000);
    let (mut s, b) = bird(at, Angle::WEST.0);
    let goal = crate::world::Pos::new(20_000, 20_000);
    s.gaia.gaia.bird_goals[0].1 = goal;
    s.roster.art.lengths.insert((-1, crate::anim::WALK), 31);
    s.roster.art.lengths.insert((-1, crate::anim::JOG), 23);
    s.roster.art.lengths.insert((-1, crate::anim::DEFAULT), 3);
    s.units[b].guys[0].anim = crate::anim::WALK;
    s.units[b].guys[0].end_time = 31;
    s.units[b].guys[0].stopped = true;
    s.trace.trace_phases = true;

    // Frame one: the step is refused and `restrict` puts the bird on
    // the boundary, which is a move — the figure lags and the moving
    // arm runs, no stand.
    s.do_air_physics(b, goal, 1);
    assert_eq!(s.units[b].pos.x, 0, "restrict puts it on the boundary");
    assert_ne!(
        s.units[b].movement.body.pos, s.units[b].pos,
        "the figure has not been teleported with the unit"
    );
    let before = s.rng.seed;
    s.process_movement(b);
    assert_eq!(s.rng.seed, before, "a bird that moved pays no stand");
    assert!(!s.units[b].guys[0].stopped, "and is marked unstopped");
    assert_eq!(s.units[b].movement.body.pos, s.units[b].pos, "it caught up");

    // Frame two: still pointed off the map and already on the
    // boundary, so `restrict` hands back the point it stands on. The
    // figure is on its `des`, the first visit only marks it stopped.
    s.do_air_physics(b, goal, 2);
    assert_eq!(s.units[b].pos.x, 0, "still pinned");
    let before = s.rng.seed;
    s.process_movement(b);
    assert_eq!(s.rng.seed, before, "the first standing frame only stops it");
    assert!(s.units[b].guys[0].stopped);

    // Frame three: stopped, on `CHAR_WALK`, and at its destination —
    // the stand, one draw, and the slot goes to `CHAR_DEFAULT`, which
    // is what makes the *next* frame's `set_anim(CHAR_WALK)` throw the
    // wing-beat coin instead of taking gaia's early return.
    s.do_air_physics(b, goal, 3);
    let before = s.rng.seed;
    s.trace.phase_marks.clear();
    s.process_movement(b);
    assert_ne!(s.rng.seed, before, "the arrival stand's draw");
    assert_eq!(s.units[b].guys[0].anim, crate::anim::DEFAULT);
    assert!(
        s.trace
            .phase_marks
            .iter()
            .any(|(l, _)| l == crate::anim::SITE_ARRIVE),
        "and it is marked as the arrival stand: {:?}",
        s.trace.phase_marks
    );
}

/// **The edge coin, end to end.** A bird pointed at the western edge
/// spends `Unit::do_air_physics+0x639` exactly once: on the frame its
/// step first leaves the world. `WorldData::restrict` puts it on the
/// boundary, `AirOrder::sharp_turn` takes the coin's `±1`, and while
/// that field stands no further coin is thrown however long the bird
/// stays out — which is what makes run53's coins cluster a hundred
/// frames apart rather than fire every frame.
#[test]
fn a_bird_pushed_off_the_map_spends_one_coin_and_then_circles() {
    // Sixteen position units from the western edge, pointed due west,
    // with the patrol point behind it so the heading stands.
    let at = crate::world::Pos::new(16, 20_000);
    let (mut s, b) = bird(at, Angle::WEST.0);
    let goal = crate::world::Pos::new(20_000, 20_000);
    s.gaia.gaia.bird_goals[0].1 = goal;
    s.trace.trace_phases = true;

    let before = s.rng.seed;
    s.do_air_physics(b, goal, 1);
    assert_eq!(s.units[b].pos.x, 0, "restrict puts it on the boundary");
    assert_ne!(s.rng.seed, before, "the coin is a draw");
    let turn = s.flight(b).turn;
    assert!(turn == 1 || turn == -1, "sharp_turn takes the coin's sign");

    // Held out for another twenty frames: the field stands, and the
    // stream does not move again.
    let held = s.rng.seed;
    for f in 2..22 {
        s.do_air_physics(b, goal, f);
    }
    assert_eq!(s.rng.seed, held, "no second coin while sharp_turn stands");
    assert_eq!(s.flight(b).turn, turn, "and the field is not re-rolled");
}

/// **The bank is what turns the bird, and it ramps.** Ten a frame to
/// fifty-five, and the heading moves by `air_turn_speed` of the bank
/// the *previous* frame left — so the first frame of a turn is at the
/// half-degree floor however much is owed.
#[test]
fn the_bank_ramps_by_ten_and_the_heading_lags_it_by_a_frame() {
    let at = crate::world::Pos::new(20_000, 20_000);
    let (mut s, b) = bird(at, Angle::NORTH.0);
    // Half a turn owed, and far enough that the keep-heading arm does
    // not fire.
    let goal = crate::world::Pos::new(20_000, 30_000);
    s.gaia.gaia.bird_goals[0].1 = goal;

    let raw = ((crate::movement::degrees_to_angle(5).0 as u32) >> 8) * 256;
    let mut heading = s.units[b].movement.heading.0;
    let want: [u32; 6] = [
        0xc120_0000,
        0xc1a0_0000,
        0xc1f0_0000,
        0xc220_0000,
        0xc248_0000,
        0xc25c_0000,
    ];
    for (i, bits) in want.iter().enumerate() {
        // The rate this frame is the bank the last one left.
        let bank = s.flight(b).roll;
        s.do_air_physics(b, goal, i as i64 + 1);
        assert_eq!(
            s.flight(b).roll.bits(),
            *bits,
            "frame {i}: the bank steps by ten toward fifty-five"
        );
        let step = air_turn_speed(raw, bank, 1) as i32;
        assert_eq!(
            s.units[b].movement.heading.0,
            heading.wrapping_add(step),
            "frame {i}: the heading moves by the *previous* bank's rate"
        );
        heading = s.units[b].movement.heading.0;
    }
    // The first of those steps is the floor, and the last is the
    // type's whole rate.
    assert_eq!(air_turn_speed(raw, Single::ZERO, 1), MIN_TURN);
    assert_eq!(air_turn_speed(raw, F55.neg(), 1), (raw / 0x37) * 55);
}

/// **The keep-heading arm**: owing more than forty-five degrees inside
/// a tile and a half of the patrol point, a bird flies straight past it
/// rather than wheeling on the spot. It is what makes the orbit an
/// orbit — without it a bird would circle its point at the turn
/// radius and never reach an edge at all.
#[test]
fn a_bird_over_its_patrol_point_keeps_the_heading_it_has() {
    let at = crate::world::Pos::new(20_000, 20_000);
    let (mut s, b) = bird(at, Angle::NORTH.0);
    // The point a hundred units behind it: half a turn owed, well
    // inside `0x300`.
    let goal = crate::world::Pos::new(20_000, 20_100);
    s.gaia.gaia.bird_goals[0].1 = goal;
    for f in 1..10 {
        s.do_air_physics(b, goal, f);
    }
    assert_eq!(
        s.units[b].movement.heading,
        Angle::NORTH,
        "inside 0x300 with more than 45 degrees owed, the heading stands"
    );
    assert_eq!(s.units[b].pos.x, 20_000, "and the bird flies straight");
    assert!(s.units[b].pos.y < 19_700, "north, at its own speed");
}
