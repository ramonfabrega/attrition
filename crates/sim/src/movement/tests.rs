use super::*;

/// **`angle_to_degrees@00a28e00` is not rounding** (`docs/COMBAT.md`
/// §52). Its steps are truncated constants, so the remainder it
/// rounds is a few units long and a half degree comes early:
/// `5_965_232` is `0.4999999…°` and reads 1, where
/// `round(a × 360 / 2³²)` reads 0. Away from the half degrees the two
/// agree, and a full turn less one unit is 360, not 0. The values are
/// the listing's arithmetic, worked outside the crate.
#[test]
fn angle_to_degrees_rounds_a_half_degree_early() {
    let exact = |a: u32| ((u64::from(a) * 360 + (1 << 31)) >> 32) as i32;
    assert_eq!(angle_to_degrees(Angle(5_965_232)), 1);
    assert_eq!(exact(5_965_232), 0);
    assert_eq!(angle_to_degrees(Angle(0x5555_5555)), 120);
    assert_eq!(angle_to_degrees(Angle(0x4000_0000)), 90);
    assert_eq!(angle_to_degrees(Angle(-1)), 360);
    // run145's chariot: its bearing to `1/8` against its facing.
    assert_eq!(
        angle_to_degrees(Angle(991_232_000_i32.wrapping_sub(0x5555_5555))),
        323
    );
    let mut a = 0u32;
    for _ in 0..4096 {
        a = a.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
        let d = angle_to_degrees(Angle(a as i32));
        assert!((d - exact(a)).abs() <= 1, "{a:#x}");
        // Off the half degrees they are the same number.
        let frac = (u64::from(a) * 360) & 0xffff_ffff;
        if !(0x7fff_0000..=0x8000_ffff).contains(&frac) {
            assert_eq!(d, exact(a), "{a:#x}");
        }
    }
}

const T: Tuning = Tuning::RON;

/// A Citizen: 45°, on foot, so it turns instantly from a stop.
const CITIZEN: Turning = Turning {
    type_turn_speed: degrees_to_angle(45).0,
    packed: false,
    instant_from_stop: true,
    wide_limit: false,
};

/// A slow type that does not turn instantly — a 10° siege piece.
const SIEGE: Turning = Turning {
    type_turn_speed: degrees_to_angle(10).0,
    packed: false,
    instant_from_stop: false,
    wide_limit: false,
};

/// **The original on a real frame, three ways.** `tools/recomp/difftest.py
/// frame` calls `GuyData::turn_speed@005de340` on every guy of every
/// active unit of the lab's end-frame snapshot — natively, as lifted C,
/// and under unicorn, on the same memory — and writes each call's inputs
/// and the original's answer to a table; this asserts the port on every
/// row it models. The table is `$RON_TURN_TABLE` (a machine without it
/// says so): `mode type_turn packed squad guy_num track_dx track_dy
/// last_speed avg_speed instant -> answer`. A guy past its squad with no
/// track offset is the one shape the port has no function for; those
/// rows are counted and skipped, not asserted.
#[test]
fn the_original_on_a_real_frame_agrees_on_every_guy() {
    let Ok(path) = std::env::var("RON_TURN_TABLE") else {
        eprintln!("skipping: set RON_TURN_TABLE (tools/recomp/difftest.py frame --table)");
        return;
    };
    let table = std::fs::read_to_string(&path).expect("RON_TURN_TABLE");
    let (mut rows, mut crew_untracked) = (0, 0);
    for line in table.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        assert!(
            f.len() == 13 && f[0] == "turn_speed" && f[11] == "->",
            "{line}"
        );
        let v = |i: usize| f[i].parse::<i64>().unwrap_or_else(|_| panic!("{line}"));
        let (mode, type_turn, packed, squad, guy_num) = (v(1), v(2), v(3), v(4), v(5));
        let (track_dx, track_dy, last_speed, avg_speed, instant) = (v(6), v(7), v(8), v(9), v(10));
        let want = v(12) as i32 as u32;
        let got = if guy_num >= squad {
            if track_dx == 0 && track_dy == 0 {
                crew_untracked += 1;
                continue;
            }
            CREW_TURN_SPEED
        } else {
            let turning = Turning {
                type_turn_speed: type_turn as i32,
                packed: packed != 0,
                instant_from_stop: instant != 0,
                wide_limit: false,
            };
            let mode = if mode == 0 {
                TurnMode::Unit
            } else {
                TurnMode::Body
            };
            turn_speed(&T, &turning, last_speed as i32, avg_speed as i32, mode)
        };
        assert_eq!(got, want, "{line}");
        rows += 1;
    }
    assert!(rows > 0, "{path}: an empty table");
    eprintln!(
        "{rows} rows agree; {crew_untracked} untracked crew rows the port has no function for"
    );
}

#[test]
fn the_axes_are_exact_and_name_the_convention() {
    // North is zero and y increases southward. Everything else in this
    // module is oriented off these four.
    assert_eq!(find_angle(0, -100), Angle::NORTH);
    assert_eq!(find_angle(100, 0), Angle::EAST);
    assert_eq!(find_angle(0, 100), Angle::SOUTH);
    assert_eq!(find_angle(-100, 0), Angle::WEST);
    assert_eq!(Angle::NORTH.quarter_turn(), Angle::EAST);
    assert_eq!(Angle::SOUTH.quarter_turn(), Angle::WEST);
    // And a quarter turn from west wraps back to north.
    assert_eq!(Angle::WEST.quarter_turn(), Angle::NORTH);
}

#[test]
fn degrees_decompose_into_exact_pieces() {
    // The quarter and eighth turns are exact by construction.
    assert_eq!(degrees_to_angle(90), Angle::EAST);
    assert_eq!(degrees_to_angle(45), Angle(0x2000_0000));
    assert_eq!(degrees_to_angle(180), Angle::SOUTH);
    assert_eq!(degrees_to_angle(270), Angle::WEST);
    assert_eq!(degrees_to_angle(360), Angle::NORTH);
    // One degree is the truncated constant.
    assert_eq!(degrees_to_angle(1), Angle(ONE_DEGREE));
    // Five degrees is five of those plus the three-unit correction; twenty
    // is fifteen plus five, and lands one above the step's "slow" line,
    // which is why a 20° type is not slow and a 19° one is.
    assert_eq!(degrees_to_angle(5), Angle(5 * ONE_DEGREE + 3));
    assert_eq!(degrees_to_angle(20), Angle(SLOW_TURN_BELOW + 1));
    assert!(degrees_to_angle(19).0 < SLOW_TURN_BELOW);
    // Non-multiples of the pieces still decompose: 75 = 45 + 30.
    assert_eq!(degrees_to_angle(75), Angle(0x2000_0000 + 0x1555_5555));
}

#[test]
fn the_arctangent_overshoots_at_forty_five_degrees() {
    // The diagonal is where the polynomial is worst: 541,917,184 against a
    // true eighth-turn of 536,870,912, which is 0.94% high. Pinned because
    // a "better" approximation would move every unit in the game.
    assert_eq!(find_angle(100, -100), Angle(541_917_184));
    assert_eq!(0x2000_0000, 536_870_912);
}

#[test]
fn every_angle_is_within_a_percent_of_the_true_bearing() {
    // The only floating point in this crate, and it is in a test: it bounds
    // the original's approximation rather than computing anything.
    let full = 4_294_967_296f64;
    let mut worst = 0f64;
    for dx in -40..=40 {
        for dy in -40..=40 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let got = f64::from(find_angle(dx, dy).0) / full;
            let want = f64::from(dx).atan2(f64::from(-dy)) / std::f64::consts::TAU;
            let mut err = (got - want).abs();
            if err > 0.5 {
                err = 1.0 - err;
            }
            worst = worst.max(err);
        }
    }
    // Under a hundredth of a turn, everywhere.
    assert!(worst < 0.01, "worst error {worst} of a full turn");
}

#[test]
fn the_sine_table_is_a_quarter_wave_that_saturates_one_step_early() {
    assert_eq!(SIN_TABLE[0], 0);
    assert_eq!(SIN_TABLE[255], 65535);
    assert!(SIN_TABLE.windows(2).all(|w| w[0] <= w[1]));
    // The divisor is 255 while the index runs over 256 steps, so the table
    // runs ahead of a true sine. At the midpoint it reads 46482 where a
    // true quarter-wave would read 46340.
    assert_eq!(SIN_TABLE[128], 46482);
}

#[test]
fn arrival_is_a_manhattan_test_not_a_distance() {
    // Along an axis the two agree.
    assert!(arrives(25, 0, 25));
    assert!(!arrives(26, 0, 25));
    // Diagonally they do not: 18,18 is 25 away by any real measure and 36
    // by this one, so the unit does *not* snap — it is further out than an
    // axis approach of the same true distance.
    assert!(!arrives(18, 18, 25));
    assert!(arrives(12, 12, 25));
}

#[test]
fn turning_takes_the_short_way_and_reports_what_is_owed() {
    let rate = 0x0800_0000;
    // A hair off counts as facing, and snaps onto the target.
    let (a, owed) = turn_towards(Angle::NORTH, Angle(0x0100_0000), rate);
    assert_eq!((a, owed), (Angle(0x0100_0000), 0));
    // Within one frame's turn, snap to the target.
    let (a, owed) = turn_towards(Angle::NORTH, Angle(0x0400_0000), rate);
    assert_eq!((a, owed), (Angle(0x0400_0000), 0));
    // Beyond it, turn by the rate and report the rest.
    let (a, owed) = turn_towards(Angle::NORTH, Angle::EAST, rate);
    assert_eq!(a, Angle(0x0800_0000));
    assert_eq!(owed, 0x4000_0000 - rate);
    // The short way round: north to west turns anticlockwise, not three
    // quarters clockwise.
    let (a, _) = turn_towards(Angle::NORTH, Angle::WEST, rate);
    assert_eq!(a, Angle(-0x0800_0000));
    // The instant rate snaps from anywhere, including a full reverse.
    let (a, owed) = turn_towards(Angle::NORTH, Angle::SOUTH, INSTANT_TURN);
    assert_eq!((a, owed), (Angle::SOUTH, 0));
}

#[test]
fn the_turn_rate_is_the_types_angle_with_its_low_byte_cleared() {
    // UNIT_TURN_SPEED is 256, so `(angle >> 8) * 256` is the angle with
    // the low eight bits gone. 45° is already clean; 10° loses 198.
    assert_eq!(T.unit_turn_speed, 256);
    assert_eq!(turn_speed(&T, &CITIZEN, 25, 0, TurnMode::Body), 0x2000_0000);
    assert_eq!(
        turn_speed(&T, &SIEGE, 25, 0, TurnMode::Body),
        (degrees_to_angle(10).0 as u32) & !0xff
    );
    // Packing doubles it.
    let packed = Turning {
        packed: true,
        ..CITIZEN
    };
    assert_eq!(turn_speed(&T, &packed, 25, 0, TurnMode::Body), 0x4000_0000);
}

#[test]
fn a_foot_unit_turns_instantly_from_a_stop_and_a_siege_piece_does_not() {
    // The data file's own comment: "Foot & Mounted units turn instantly
    // from a stopped position". `last_speed == 0` is "stopped".
    assert_eq!(turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit), INSTANT_TURN);
    assert_eq!(turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Body), INSTANT_TURN);
    // Moving, it is back on the rate.
    assert_ne!(
        turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit),
        INSTANT_TURN
    );
    // A type without the flag gets its base from a standstill.
    assert_eq!(
        turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit),
        (degrees_to_angle(10).0 as u32) & !0xff
    );
}

#[test]
fn a_moving_unit_turns_slower_the_faster_it_goes_down_to_a_degree() {
    // Mode 0 divides by `avg_speed / 4 + 1`: a Citizen at full stride
    // turns at 45°/7, a little over six degrees a frame.
    assert_eq!(
        turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit),
        0x2000_0000 / 7
    );
    // The body does not slow down.
    assert_eq!(
        turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Body),
        0x2000_0000
    );
    // The floor is one degree a frame, to the same precision as the base:
    // a 5° type at a Citizen's pace would be 5°/7 and is held at 1°.
    let slow = Turning {
        type_turn_speed: degrees_to_angle(5).0,
        ..SIEGE
    };
    let floor = 256 * 0xb60b;
    assert_eq!(floor, (ONE_DEGREE as u32) & !0xff);
    assert_eq!(turn_speed(&T, &slow, 25, 25, TurnMode::Unit), floor);
}

#[test]
fn the_axis_components_are_whole_steps_and_the_overflow_is_why() {
    // The four cardinals, which are exactly where the table's wrap-around
    // lands. If the interpolation multiply were widened these would all
    // come back as zero and every unit would stand still.
    let d = 1000;
    assert_eq!(sin_component(Angle::NORTH, d), 0);
    assert_eq!(cos_component(Angle::NORTH, d), d);
    assert_eq!(sin_component(Angle::EAST, d), d);
    assert_eq!(cos_component(Angle::EAST, d), 0);
    assert_eq!(sin_component(Angle::SOUTH, d), 0);
    assert_eq!(cos_component(Angle::SOUTH, d), -d);
    assert_eq!(sin_component(Angle::WEST, d), -d);
    assert_eq!(cos_component(Angle::WEST, d), 0);
    // And a zero step is zero regardless of heading.
    assert_eq!(sin_component(Angle::EAST, 0), 0);
}

#[test]
fn the_components_stay_on_the_circle() {
    // Not a proof, a bound: every heading should put the two components
    // within a couple of percent of the step length. The metric here is a
    // true hypotenuse on purpose — this is a test asking "is the trig
    // sane", not simulation arithmetic.
    let d = 10_000f64;
    for k in 0i32..64 {
        let a = Angle(k.wrapping_mul(0x0400_0000));
        let (sx, cy) = (
            f64::from(sin_component(a, 10_000)),
            f64::from(cos_component(a, 10_000)),
        );
        let r = sx.hypot(cy);
        assert!((r - d).abs() / d < 0.02, "heading {k}: radius {r}");
    }
}

#[test]
fn a_unit_walks_east_and_lands_exactly_on_its_destination() {
    // Twelve tiles east at a Citizen's speed. The unit's step is its speed,
    // 25 position units a frame — not 34, which is the body's — so this
    // takes a while and must not drift off the axis or overshoot at the
    // end.
    let start = Pos::new(0, 0);
    let dest = Pos::new(12 * 192, 0);
    let mut pos = start;
    let mut facing = Angle::EAST;
    let mut frames = 0;
    loop {
        let rate = turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit);
        let step = move_step(pos, facing, dest, 25, &CITIZEN, rate, false, false);
        pos = step.pos;
        facing = step.facing;
        frames += 1;
        if step.arrived {
            break;
        }
        assert!(frames < 1000, "never arrived");
    }
    assert_eq!(pos, dest);
    assert_eq!(pos.y, 0, "drifted off the axis");
    // 2304 units at 25 a frame is 92 full steps and a short one, and the
    // short one is the Manhattan snap onto the destination. (An earlier
    // draft of this test expected 68, at the body's 34.)
    assert_eq!(frames, 93);
}

#[test]
fn a_foot_unit_reverses_and_walks_on_the_same_frame() {
    // Facing east, told to go west, stopped: the rate is instant, so it
    // faces west and takes a full step in one frame. (An earlier draft of
    // this module had every unit stand still for the turn; the original
    // reserves that for types without the flag, or already moving.)
    let rate = turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit);
    let s = move_step(
        Pos::new(0, 0),
        Angle::EAST,
        Pos::new(-10_000, 0),
        25,
        &CITIZEN,
        rate,
        false,
        false,
    );
    assert_eq!(s.facing, Angle::WEST);
    assert_eq!(s.owed, 0);
    assert_eq!(s.pos, Pos::new(-25, 0));
}

#[test]
fn a_helicopter_owing_a_quarter_turn_or_less_steps_while_it_turns() {
    // Item 1668: `Unit::move_step`'s `unit_flags & 0x20` arm. A
    // helicopter facing east and told to go north owes 90°, less its
    // frame's turn: inside the quarter it takes the whole step at once,
    // aimed along a second, faster turn; any other type with the same
    // numbers stands and turns, and so does a helicopter owing more.
    let dest = Pos::new(0, -10_000);
    let rate = turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit);
    let fast = turn_speed(&T, &SIEGE, 0, 0, TurnMode::Body);
    let heli = move_step_as(
        Pos::new(0, 0),
        Angle::EAST,
        dest,
        25,
        &SIEGE,
        rate,
        false,
        false,
        Some(fast),
    );
    assert_ne!(heli.pos, Pos::new(0, 0), "it moved");
    let plain = move_step(
        Pos::new(0, 0),
        Angle::EAST,
        dest,
        25,
        &SIEGE,
        rate,
        false,
        false,
    );
    assert_eq!(plain.pos, Pos::new(0, 0), "a siege piece stands to turn");
    assert_eq!(heli.facing, plain.facing, "the guy's own turn is the same");
    // Owing past a quarter turn the arm is not taken.
    let behind = move_step_as(
        Pos::new(0, 0),
        Angle::WEST,
        Pos::new(10_000, 0),
        25,
        &SIEGE,
        0x1000_0000,
        false,
        false,
        Some(fast),
    );
    assert!(behind.turned_in_place.is_some());
}

#[test]
fn a_siege_piece_turns_in_place_and_then_walks_where_it_points() {
    // A 10° type, stopped, facing east and told to go north, with the
    // destination well over two tiles away. Its rate from a standstill is
    // its base, 10° less the low byte; it owes a quarter turn less that
    // each frame, and may not move while owing 45° or more. That is four
    // frames stood still; on the fifth it owes under 45° and walks — at
    // half speed, because it still owes more than 22.5°, and along the
    // facing it actually has, which is some forty degrees east of north.
    let dest = Pos::new(0, -10_000);
    let mut pos = Pos::new(0, 0);
    let mut facing = Angle::EAST;
    let mut stood = 0;
    let s = loop {
        let rate = turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit);
        let s = move_step(pos, facing, dest, 25, &SIEGE, rate, false, false);
        facing = s.facing;
        if s.pos != pos {
            break s;
        }
        pos = s.pos;
        stood += 1;
        assert!(stood < 40, "never got moving");
    };
    assert_eq!(stood, 4);
    assert!(s.owed < FORTY_FIVE && s.owed >= FORTY_FIVE / 2);
    // Half of 25 is 12, split between the two axes along the facing: it
    // walks north *and* east, because the trig uses the facing after the
    // turn and not the heading it wants.
    assert!(s.pos.y < 0, "moved the wrong way");
    assert!(s.pos.x > 0, "walked along the heading, not the facing");
    assert_eq!(s.pos.x.abs() + s.pos.y.abs(), 16, "not a half step");
}

#[test]
fn a_soft_collision_costs_the_next_step_its_half_and_is_spent_doing_it() {
    // `unit_masks & 0x100000`, `move_step@005faf30`'s `005fb1f4`–`005fb219`: the
    // one-shot a **soft** collision leaves behind
    // (`docs/COLLISION.md` §4.3, §7). It is read on the arm that owes
    // *less* than 45° — the arm a marching squad spends every frame on
    // — and reading it is what makes run76's captain step
    // `26, 13, 26, 26, 13` where the same walk unimpeded steps 26 flat.
    let rate = turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit);
    let dest = Pos::new(0, -10_000);
    let plain = move_step(
        Pos::new(0, 0),
        Angle::NORTH,
        dest,
        26,
        &CITIZEN,
        rate,
        false,
        false,
    );
    assert_eq!(plain.owed, 0);
    assert_eq!(plain.pos, Pos::new(0, -26));
    assert!(!plain.half_step_used);
    let soft = move_step(
        Pos::new(0, 0),
        Angle::NORTH,
        dest,
        26,
        &CITIZEN,
        rate,
        false,
        true,
    );
    assert_eq!(soft.pos, Pos::new(0, -13), "the one-shot half step");
    assert!(soft.half_step_used, "and the frame spends it");

    // **The other arm does not spend it.** A turn of 45° or more halves
    // the step on its own, and the original's `else` never touches the
    // bit — so a unit that soft-collided while turning hard still owes
    // itself a half step when the turn is done. The siege piece's fifth
    // frame is that arm (`a_siege_piece_turns_in_place_and_then_walks…`).
    let mut pos = Pos::new(0, 0);
    let mut facing = Angle::EAST;
    let turning = loop {
        let rate = turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit);
        let s = move_step(pos, facing, dest, 25, &SIEGE, rate, false, true);
        facing = s.facing;
        if s.pos != pos {
            break s;
        }
        pos = s.pos;
    };
    assert!(turning.owed >= FORTY_FIVE / 2, "the hard-turn arm");
    assert_eq!(
        turning.pos.x.abs() + turning.pos.y.abs(),
        16,
        "halved once, not twice"
    );
    assert!(!turning.half_step_used, "the hard-turn arm keeps the bit");
}

#[test]
fn close_to_the_destination_any_turn_owed_costs_the_frame() {
    // Within a tile — two for a slow type — the unit will not move while it
    // owes anything at all, even a turn it could otherwise walk through.
    let rate = turn_speed(&T, &SIEGE, 25, 25, TurnMode::Unit);
    // 300 away: inside two tiles for a 10° type, so owing a few degrees is
    // enough to stand.
    let near = move_step(
        Pos::new(0, 0),
        Angle(0x0400_0000),
        Pos::new(0, -300),
        25,
        &SIEGE,
        rate,
        false,
        false,
    );
    assert!(near.owed != 0 && near.owed < FORTY_FIVE);
    assert_eq!(near.pos, Pos::new(0, 0));
    // The same owed turn two tiles out is walked through.
    let far = move_step(
        Pos::new(0, 0),
        Angle(0x0400_0000),
        Pos::new(0, -1000),
        25,
        &SIEGE,
        rate,
        false,
        false,
    );
    assert_eq!(far.owed, near.owed);
    assert_ne!(far.pos, Pos::new(0, 0));

    // **And the waypoint's own `TURN_FIRST` puts the far case back in
    // the near case's arm** — `(manh < slow × 0xc0) || (path.flags & 4)`
    // at `005fb1a5`. The bit is `docs/PATHFINDER.md` §7's transport
    // marker and `shore_flagged`'s, and until 2026-09-01 this crate
    // wrote it and never read it: a citizen crossing the waterline
    // walked through the turn the original stands still for, and
    // reached its embark point a frame early (`docs/SYNC.md` §3.25).
    let told = move_step(
        Pos::new(0, 0),
        Angle(0x0400_0000),
        Pos::new(0, -1000),
        25,
        &SIEGE,
        rate,
        true,
        false,
    );
    assert_eq!(told.owed, near.owed);
    assert_eq!(told.pos, Pos::new(0, 0), "the flag costs the whole frame");
    // With nothing owed the flag costs nothing: it gates the stand, not
    // the step.
    let facing = move_step(
        Pos::new(0, 0),
        Angle::NORTH,
        Pos::new(0, -1000),
        25,
        &SIEGE,
        rate,
        true,
        false,
    );
    assert_eq!(facing.owed, 0);
    assert_ne!(facing.pos, Pos::new(0, 0));
}

#[test]
fn the_clamps_land_the_unit_exactly_and_that_counts_as_arrival() {
    // A diagonal short hop: the true distance is 141 and the step is 149,
    // so a component would carry the unit past the destination on both
    // axes — but the Manhattan sum is 200, so the snap does not fire and
    // the clamps are what has to stop it. They are active because 200 is
    // within two steps.
    let dest = Pos::new(100, -100);
    assert!(!arrives(100, -100, 149));
    let rate = turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit);
    let s = move_step(
        Pos::new(0, 0),
        Angle::NORTH,
        dest,
        149,
        &CITIZEN,
        rate,
        false,
        false,
    );
    // Both clamps fire, so it lands exactly on the destination — and with
    // a zero tolerance that is arrival, even though the snap never said so.
    assert_eq!(s.pos, dest);
    assert!(s.arrived);
    // And **which arm** it arrived on is the difference between facing
    // the order's angle and keeping this step's bearing when a gather
    // order sits beneath (`docs/ORDERS.md` §4.5). This one is the
    // partial step; a shorter hop of the same shape is the snap.
    assert!(!s.snapped, "the partial step, not the Manhattan snap");
    let near = Pos::new(3, -4);
    assert!(arrives(3, -4, 149));
    let s = move_step(
        Pos::new(0, 0),
        Angle::NORTH,
        near,
        149,
        &CITIZEN,
        rate,
        false,
        false,
    );
    assert_eq!(s.pos, near);
    assert!(s.arrived && s.snapped);
}

#[test]
fn a_group_moves_at_its_slowest_and_never_below_the_floor() {
    assert_eq!(group_capped(25, 12), 12);
    assert_eq!(group_capped(12, 25), 12);
    // Zero means the group has no speed set, and caps nothing.
    assert_eq!(group_capped(25, 0), 25);
    // The floor bites after two halvings of an already slow unit.
    assert_eq!(group_capped(2, 0), SPEED_FLOOR);
    assert_eq!(group_capped(25, 1), SPEED_FLOOR);
}

/// **`UnitData::get_speed`'s slow ground** (`0060879b`–`006087d8`):
/// a land unit standing on a tile that carries `0x800` moves at half
/// speed, and only while its own `z_internal` is not above zero.
///
/// Both halves matter and each was made to fail on its own. run75's
/// AI scout `1/0` walks the river bed south of its city from sim-frame
/// 5950 to 6090 at a step of 17 against its `myspeed 34`, and its
/// dumped `z_internal` is 14 on the frame before, 0 for every frame of
/// it, and 17 on the frame it comes out — which is what pins the `z`
/// gate rather than the tile alone.
#[test]
fn a_land_unit_on_a_river_tile_at_or_below_the_waterline_walks_at_half() {
    use crate::world::tile;
    use crate::{Sim, Tuning, Unit, UnitType, World};

    let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
    let ty = s.add_unit_type(UnitType {
        hits: 1,
        moves: 34,
        ..UnitType::default()
    });
    // The scout's own tile on run75's frame 5960, `(36654, 29572)`.
    let at = Pos::new(36654, 29572);
    let t = at.tile();
    let mut u = Unit::new(1, 0, at, 1);
    u.ty = Some(ty);
    let a = s.add_unit(u);
    s.units[a].movement.speed = 34;

    s.world.set_tile_z(t, 0);
    assert_eq!(s.get_speed(a, 0), 34, "plain ground at the waterline");

    s.world.set_tile_bits(t, tile::RIVER);
    assert_eq!(s.get_speed(a, 0), 17, "river, and z is not above zero");

    // The `z` gate: one unit of height above the water and the tile
    // stops slowing it.
    s.world.set_tile_z(t, 1);
    assert_eq!(s.get_speed(a, 0), 34, "z > 0 skips the halving");
    s.world.set_tile_z(t, -1);
    assert_eq!(s.get_speed(a, 0), 17, "and below it does not");

    // The arm is the **land** one whole: a boat over the same tile is
    // not slowed by it.
    s.units[a].kind.domain = crate::territory::attrition::Domain::Sea;
    assert_eq!(s.get_speed(a, 0), 34, "sea takes none of layer three");
    s.units[a].kind.domain = crate::territory::attrition::Domain::Land;

    // The floor is below the halving, not above it.
    s.units[a].movement.speed = 4;
    assert_eq!(s.get_speed(a, 0), SPEED_FLOOR, "4 / 2 is under the floor");
}

/// **`get_speed`'s third argument selects exactly one thing, the
/// group cap** (`docs/GROUPS.md` §18), and the cap has a second gate
/// the flag does not reach: a unit with an **action** order is
/// exempt, which is how Great Lakes' `1/29` walks at 29 in a group
/// whose other two raiders walk at 25.
///
/// Made to fail on purpose three ways: drop the `flag == 0` test and
/// the second row is 25; drop the `action == index::NONE` test and
/// the fourth is 25; take the cap before the order scale instead of
/// after and the fourth is 28.
#[test]
fn the_group_cap_is_flag_zero_s_and_an_action_order_is_exempt() {
    use crate::orders::{AttackOrder, Body, Order};
    use crate::{Sim, Tuning, Unit, UnitType, World};

    use crate::combat::mask;

    let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
    // Two types, so `find_leader`'s answer is `type_cat`'s and not
    // the list's order: mounted outranks foot, and the mounted one
    // is the slow one.
    let armed = |m: u32| {
        let mut t = UnitType {
            hits: 1,
            moves: 26,
            ..UnitType::default()
        };
        t.combat.attack = 15;
        t.combat.obj_masks = m;
        t
    };
    let foot = s.add_unit_type(armed(mask::FOOT));
    let horse = s.add_unit_type(armed(mask::MOUNTED));
    fn make(s: &mut Sim, ty: usize, o: i16, p: Pos, speed: i32) -> usize {
        let mut u = Unit::new(1, o, p, 1);
        u.ty = Some(ty);
        u.on_map = true;
        let h = s.add_unit(u);
        s.units[h].movement.speed = speed;
        h
    }
    // Great Lakes' group 65 in miniature: a 26 and a 25.
    let fast = make(&mut s, foot, 0, Pos::new(0x1000, 0x1000), 26);
    let slow = make(&mut s, horse, 1, Pos::new(0x1100, 0x1000), 25);
    let mut g = crate::groups::group::Group {
        who: 1,
        army: None,
        pushed: None,
        list: vec![fast, slow],
    };
    assert!(s.push_group(&mut g, true));
    s.group_normalize(&mut g);

    assert_eq!(
        s.group_find_leader(&g),
        Some(slow),
        "the mounted type leads on `type_cat`, whatever the list order"
    );
    assert_eq!(
        s.group_speed_of(fast),
        25,
        "the group's cap is its leader's, and the leader is the slow one"
    );
    assert_eq!(s.get_speed(fast, 0), 25, "flag 0 is capped");
    assert_eq!(s.get_speed(fast, 1), 26, "flag 1 is not");

    // An action order exempts it — `get_action` answering non-null is
    // `local_8 != 0`, and the same value carries the order's own
    // scale, so the answer is `26 * 9 / 8` and not 26.
    s.units[fast].orders.push_back(Order {
        flags: 0,
        body: Body::Attack(AttackOrder {
            defensive: false,
            def: None,
            in_range: false,
            ever_in_range: false,
            new_ord: true,
        }),
    });
    assert_eq!(
        s.get_speed(fast, 0),
        29,
        "an action order skips the cap and takes the ATTACK 9/8"
    );

    // And a unit with no group is not capped by anything.
    let lone = make(&mut s, foot, 2, Pos::new(0x2000, 0x1000), 26);
    assert_eq!(s.group_speed_of(lone), 0, "no seat, no cap");
    assert_eq!(s.get_speed(lone, 0), 26);
}

#[test]
fn the_body_lands_on_the_unit_and_records_the_euclidean_step() {
    // The unit stepped 25 east; the body is written straight onto it, and
    // `last_speed` is the distance of the jump — no chase, no 11/8, and
    // no turn on the way.
    let rate = turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Body);
    let b = body_follow(
        Body::at(Pos::new(0, 0)),
        Angle::EAST,
        Pos::new(25, 0),
        Angle::EAST,
        false,
        rate,
    );
    assert_eq!(b.body.pos, Pos::new(25, 0));
    assert_eq!(b.body.last_speed, 25);
    assert_eq!(b.body.avg_speed, 6);
    assert_eq!(b.facing, Angle::EAST);
    // Standing on the unit — which is every frame the unit did not move —
    // the speed is zero and the average decays.
    let idle = body_follow(
        Body {
            pos: Pos::new(25, 0),
            last_speed: 25,
            avg_speed: 25,
        },
        Angle::EAST,
        Pos::new(25, 0),
        Angle::EAST,
        false,
        rate,
    );
    assert_eq!(idle.body.last_speed, 0);
    assert_eq!(idle.body.avg_speed, 18);
}

#[test]
fn the_average_settles_below_the_step_and_that_is_what_divides_the_turn() {
    // run10's AI scout: `MOVES` 34, walking a diagonal, stepping (24, 24)
    // a frame. `vector_dist` calls that 36 — the octagonal measure, not
    // 33.9 — and the quarter-old-quarter-new average truncates its way to
    // **33**, not to 36.
    assert_eq!(vector_dist(24, 24), 36);
    let mut body = Body::at(Pos::new(0, 0));
    for i in 0..40 {
        let to = Pos::new(24 * (i + 1), 24 * (i + 1));
        body = body_follow(body, Angle::EAST, to, Angle::EAST, false, 0).body;
    }
    assert_eq!(body.last_speed, 36);
    assert_eq!(body.avg_speed, 33, "the fixpoint of (3a + 36) / 4");
    // And 33 is what the unit's turn rate is divided by: `33 / 4 + 1` is
    // nine. A Scout's 27 degrees over nine is the 0x222221c the original
    // turned by on run10's frame 62. An eleven-eighths chase would have
    // given `last_speed` 46, an average of 43, and a divisor of eleven.
    let scout = Turning {
        type_turn_speed: degrees_to_angle(27).0,
        ..CITIZEN
    };
    assert_eq!(
        turn_speed(&T, &scout, 36, body.avg_speed, TurnMode::Unit),
        0x0222_221c
    );
}

#[test]
fn an_idle_body_turns_to_its_desired_angle_unless_the_unit_just_turned() {
    let rate = turn_speed(&T, &SIEGE, 0, 25, TurnMode::Body);
    let at = Body::at(Pos::new(0, 0));
    let b = body_follow(at, Angle::NORTH, Pos::new(0, 0), Angle::EAST, false, rate);
    assert_eq!(b.facing, Angle(rate as i32));
    // If the unit step already turned the shared facing this frame, the
    // body leaves it alone.
    let b = body_follow(at, Angle::NORTH, Pos::new(0, 0), Angle::EAST, true, rate);
    assert_eq!(b.facing, Angle::NORTH);
}

/// A scout's dog, in the three positions run56 prints for it.
///
/// The pair is `(-96, 48)` — `unit_graphics.xml`'s `trackoffsetx=-20`
/// and `trackoffsety=10` through the executable's `guy_scale` of 4.8
/// (`rondata::artdata::piece_tracks`). Each row here is a `GUY` block
/// the capture writes, so a rotation with its axes swapped, its signs
/// flipped or its quarter turn dropped fails on one of them.
#[test]
fn a_crew_guy_stands_where_its_leader_s_facing_puts_it() {
    let bound = Pos::new(200 * 768, 200 * 768);
    let dog = (-96, 48);
    // The human scout at frame 0, facing `Unit::init`'s 120°.
    assert_eq!(
        follower_des(Pos::new(5784, 8088), Angle::INITIAL, dog, bound),
        Pos::new(5790, 7979)
    );
    // The AI's, on the frame its man arrives and on the frame after —
    // the same leader position, a different facing, and a destination
    // 150 units away because of it. That second point is where the dog
    // finally arrives, four frames later.
    let arrived = Pos::new(40416, 34272);
    assert_eq!(
        follower_des(arrived, Angle(-424673280), dog, bound),
        Pos::new(40365, 34367)
    );
    assert_eq!(
        follower_des(arrived, Angle(683016192), dog, bound),
        Pos::new(40322, 34217)
    );
    // And the clamp is the world's own edge, in position units: a
    // leader in the corner facing north puts its dog 96 to the west,
    // which is off the map, so the x is pinned at zero and the y —
    // 48 south of the corner, and on the map — is left alone.
    let corner = follower_des(Pos::new(0, 0), Angle::NORTH, dog, Pos::new(768, 768));
    assert_eq!(corner, Pos::new(0, 48));
    let far = follower_des(Pos::new(767, 767), Angle::SOUTH, dog, Pos::new(768, 768));
    assert_eq!(far, Pos::new(767, 719));
}

/// The crew's rate is a quarter turn, and the give-up can never fire.
///
/// `Guy::move` abandons the step when `2 × rate < owed`, and
/// [`turn_towards`] returns `|delta| − rate`; with the rate at
/// [`CREW_TURN_SPEED`] that wants a shortest turn of more than 270°,
/// which does not exist. So a tracked crew guy always steps.
#[test]
fn a_crew_guy_turns_a_quarter_at_a_time_and_never_gives_the_frame_up() {
    let body = Body::at(Pos::new(1000, 1000));
    // Facing north, told to walk due south: a half turn owed, of which
    // a quarter is taken this frame — and the step is taken anyway.
    let (facing, next) = follower_step(
        body,
        Angle::NORTH,
        Pos::new(1000, 2000),
        CREW_TURN_SPEED,
        32,
    );
    assert_eq!(facing, Angle::EAST);
    let next = next.expect("a crew guy never spends the whole frame turning");
    assert_eq!(next.last_speed, 44, "floor(32 * 11 / 8)");
    // Half a turn is the worst case there is, and the doubling that
    // guards the give-up is a 32-bit multiply — so the constant's own
    // arithmetic is what makes the branch dead.
    let (_, owed) = turn_towards(Angle::NORTH, Angle::SOUTH, CREW_TURN_SPEED);
    assert!(CREW_TURN_SPEED.wrapping_mul(2) >= owed);
}

/// The Manhattan snap, and the per-axis clamp that stands in for it
/// when the step is short.
#[test]
fn a_crew_guy_snaps_inside_a_step_and_is_clamped_outside_one() {
    let body = Body::at(Pos::new(1000, 1000));
    // Twelve away with a step of 44: inside, so it lands exactly and
    // reports the octagonal distance rather than the step.
    let (_, next) = follower_step(body, Angle::EAST, Pos::new(1008, 1004), CREW_TURN_SPEED, 32);
    let next = next.unwrap();
    assert_eq!(next.pos, Pos::new(1008, 1004));
    assert_eq!(next.last_speed, vector_dist(8, 4));
    // Far away on x and one unit away on y: the y component is clamped
    // to the remainder so the guy cannot overshoot that axis.
    let (_, next) = follower_step(body, Angle::EAST, Pos::new(9000, 1001), CREW_TURN_SPEED, 32);
    let next = next.unwrap();
    assert_eq!(next.pos.y, 1001);
    assert_eq!(next.last_speed, 44);
}
