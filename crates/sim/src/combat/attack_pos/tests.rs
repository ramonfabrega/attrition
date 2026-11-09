use super::*;

/// The octant table of §17.2, read back as a function of where the
/// asker stands. The sides run **anticlockwise from the north-west
/// corner**: odd are corners, even are edge midpoints, and a target
/// at the origin with a 4 × 4 footprint puts the asker's own compass
/// bearing on the side facing it.
///
/// Made to fail on purpose by swapping the `ey < ex` arms: the four
/// diagonals keep their answers and all four axes move, which is why
/// the axes are here as well as the corners.
#[test]
fn the_octant_table_faces_the_asker() {
    let t = Pos::new(10_000, 10_000);
    let (xs, ys) = (4, 4);
    let at = |dx: i32, dy: i32| {
        let from = Pos::new(t.x + dx, t.y + dy);
        let (ex, ey) = (
            (from.x - t.x).abs() - xs * 0x60,
            (from.y - t.y).abs() - ys * 0x60,
        );
        crate::Sim::start_side(from, t, ex, ey)
    };
    // The axes, well outside the footprint: due north is side 2,
    // east 4, south 6, west 8.
    assert_eq!(at(0, -5000), 2, "north");
    assert_eq!(at(5000, 0), 4, "east");
    assert_eq!(at(0, 5000), 6, "south");
    assert_eq!(at(-5000, 0), 8, "west");
    // The diagonals are the corners between them.
    assert_eq!(at(-5000, -5000), 1, "north-west");
    assert_eq!(at(5000, -5000), 3, "north-east");
    assert_eq!(at(5000, 5000), 5, "south-east");
    assert_eq!(at(-5000, 5000), 7, "south-west");
    // Great Lakes' own archers: far east and a little north of the
    // farm, which is the north-east corner.
    assert_eq!(at(34_392, -7_848), 3, "run19's `1/27`");
}

/// The two arms leave the same point in opposite directions, and an
/// edge is entered from the end the arm walks away from (§17.2).
///
/// Made to fail on purpose by giving both arms `arm = 0` in
/// [`crate::Sim::side_base`]: the two arms then start the same edge
/// at the same corner and walk into each other.
#[test]
fn the_two_arms_leave_the_same_edge_at_opposite_ends() {
    let t = Pos::new(10_000, 10_000);
    let (xs, ys) = (384, 384);
    let d = 0x30;
    // Side 2 is the north face. Arm 0 walks the ring anticlockwise
    // and so enters from the east end; arm 1 from the west.
    let a0 = crate::Sim::side_base(2, t, xs, ys, d, 0);
    let a1 = crate::Sim::side_base(2, t, xs, ys, d, 1);
    assert_eq!(a0, Pos::new(t.x + xs, t.y - ys - d));
    assert_eq!(a1, Pos::new(t.x - xs, t.y - ys - d));
    assert_eq!(a0.y, a1.y, "one face, one stand-off");
    // A corner's base point is the corner itself — the arc adds the
    // stand-off — and both arms share it.
    for side in [1, 3, 5, 7] {
        assert_eq!(
            crate::Sim::side_base(side, t, xs, ys, d, 0),
            crate::Sim::side_base(side, t, xs, ys, d, 1),
            "side {side} is a corner"
        );
    }
    assert_eq!(
        crate::Sim::side_base(1, t, xs, ys, d, 0),
        Pos::new(t.x - xs, t.y - ys)
    );
    assert_eq!(
        crate::Sim::side_base(5, t, xs, ys, d, 0),
        Pos::new(t.x + xs, t.y + ys)
    );
}

/// **An edge start is the face's midpoint** (item 1012,
/// `docs/COMBAT.md` §66): the start-side switch at `601e3e` puts both
/// arms on `(t.x + x_size × 0x60 + d, t.y)` for side 4
/// (`601ea0`-`601ef4`), and likewise for 2, 6 and 8; only an arm
/// stepping onto an edge from a corner enters it at an end. Great Lakes
/// 4605's `1/17`, square on the city's east face, stands at (3912,
/// 30840) in run356's block 4606, level with the centre.
///
/// Made to fail by starting an edge at [`crate::Sim::side_base`]'s
/// end: every edge assertion names the corner.
#[test]
fn an_edge_start_is_the_face_s_midpoint() {
    let t = Pos::new(10_000, 10_000);
    let (xs, ys) = (384, 384);
    let d = 0x30;
    let at = |side| crate::Sim::start_base(side, t, xs, ys, d);
    assert_eq!(at(2), Pos::new(t.x, t.y - ys - d), "north");
    assert_eq!(at(4), Pos::new(t.x + xs + d, t.y), "east");
    assert_eq!(at(6), Pos::new(t.x, t.y + ys + d), "south");
    assert_eq!(at(8), Pos::new(t.x - xs - d, t.y), "west");
    for side in [1, 3, 5, 7] {
        assert_eq!(
            at(side),
            crate::Sim::side_base(side, t, xs, ys, d, 0),
            "side {side} is a corner, and starts on it"
        );
    }
}

/// **And the walk begins there** (item 1012): an asker square on a
/// building's east face takes a spot level with the centre, as run356's
/// `1/17` does on block 4606, because both arms leave the face's
/// midpoint and the call's budget ends four candidates later. From the
/// corner end the four candidates are all near the corner.
///
/// Made to fail by starting the ring at [`crate::Sim::side_base`]'s
/// end in [`crate::Sim::find_attack_pos`]: the spot is 504 south.
#[test]
fn a_ring_started_on_an_edge_stands_level_with_the_target() {
    use crate::combat::Profile;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 70,
        combat: Profile {
            attack: 15,
            max_range: 6,
            uber_size: 1,
            block_radius: 48,
            big_radius: 48,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let t = Pos::new(20 * 0x300, 20 * 0x300);
    let city = sim.add_building(0, t, 0);
    sim.buildings[city].hits = 1200;
    sim.buildings[city].health = 1200;
    sim.buildings[city].combat = Some(Profile {
        x_size: 7,
        y_size: 7,
        ..Profile::default()
    });
    let mut u = crate::Unit::new(1, 0, Pos::new(t.x + 0x1800, t.y), 70);
    u.ty = Some(ty);
    u.on_map = true;
    let me = sim.add_unit(u);
    let from = sim.units[me].pos;
    let p = sim
        .find_attack_pos(
            me,
            Obj::Building(city),
            from,
            crate::fight::SITE_ATTACK_POS_FIGHT,
        )
        .expect("a spot on the ring");
    assert!(p.x > t.x, "on the east side: {p:?}");
    assert!(
        (p.y - t.y).abs() <= 0x60,
        "the spot is {} from level with the centre: {p:?}",
        p.y - t.y
    );
}

/// **The asker cannot ride while it searches** (§17.7, item 2845): a
/// land unit carrying `unit_masks & 0x800000` may stand on water as
/// far as `invalid_loc` is concerned, but `find_attack_pos` clears the
/// bit for the call (`601505`) and OR-s it back on every exit. So the
/// ring refuses the water face it would otherwise stand on, and the
/// bit is still set afterwards. The peace pair's Rocket Artillery
/// `1/63` refuses four water candidates of its bombard ring on 6808
/// (run882's packet, `invalid_loc` answering 2 at `607f28`).
///
/// Made to fail on purpose by keeping the bit through the call: the
/// spot is then on the water east of the target.
#[test]
fn an_asker_s_auto_transport_is_held_clear_while_it_searches() {
    use crate::combat::Profile;
    use crate::world::{World, tile};
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 70,
        combat: Profile {
            attack: 15,
            max_range: 6,
            uber_size: 1,
            block_radius: 48,
            big_radius: 48,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let t = Pos::new(20 * 0x300, 20 * 0x300);
    let city = sim.add_building(0, t, 0);
    sim.buildings[city].hits = 1200;
    sim.buildings[city].health = 1200;
    sim.buildings[city].combat = Some(Profile {
        x_size: 7,
        y_size: 7,
        ..Profile::default()
    });
    // Everything east of the footprint's face is open sea.
    let face = t.tile().x + 4;
    for y in 0..sim.world.height() * 4 {
        for x in face..sim.world.width() * 4 {
            sim.world
                .set_tile_field(Pos::new(x, y), tile::SURFACE, tile::SURFACE_OCEAN);
        }
    }
    let mut u = crate::Unit::new(1, 0, Pos::new(t.x + 0x1800, t.y), 70);
    u.ty = Some(ty);
    u.on_map = true;
    u.auto_transport = true;
    let me = sim.add_unit(u);
    let from = Pos::new(t.x + 0x900, t.y);
    let p = sim
        .find_attack_pos(
            me,
            Obj::Building(city),
            from,
            crate::fight::SITE_ATTACK_POS_FIGHT,
        )
        .expect("a spot on the ring");
    assert_ne!(
        sim.world.tile_mask(p.tile()) & tile::SURFACE,
        tile::SURFACE_OCEAN,
        "the ring stands on land while the bit is held clear: {p:?}"
    );
    assert!(
        sim.units[me].auto_transport,
        "the bit is OR-ed back on the way out"
    );
}

/// **The golden record's own melee chase** (§19): chapter one's
/// `0/6`, ordered onto `1/6` at the end of frame 616, asks where to
/// stand on 617 and the original's dump answers `(1080, 8280)` —
/// `orders_x`/`orders_y` at block 618, with the five-node path stack
/// beneath it.
///
/// Every number here is the golden dump's: the six positions at block
/// 617, and the three `orders_x`/`orders_y` that decide the ring's
/// rejections — `0/7` seated on its own point, `1/7` walking to
/// `(1320, 7800)` and `1/8` to `(1176, 8088)`, all three from the
/// original's own block 617 (`docs/RUNS.md` run101–run105).
///
/// **It is the rejections that make the answer, not the distance.**
/// `(1080, 8280)` is the *farthest* of the ring's four corners from
/// the asker; six nearer candidates beat it on `vector_dist` and every
/// one of them is inside two unit cells of an ordered destination.
/// Made to fail on purpose by seating `1/7` and `1/8` on their own
/// points instead of their ordered ones — the call then answers
/// `(1080, 7992)`, two rows up the same column.
#[test]
fn chapter_one_s_melee_chase_stands_where_the_golden_dump_puts_it() {
    use crate::combat::Profile;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    // HOPLITES as the data loads it: `BLOCK_RADIUS 1` (so `+0x248` is
    // 1 and the ring's step is 3), melee, and in its own lineage.
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 120,
        combat: Profile {
            attack: 15,
            uber_size: 1,
            block_radius: 48,
            big_radius: 48,
            combat_role: true,
            hoplites: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut crate::Sim, who: crate::Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 120);
        u.ty = Some(ty);
        u.on_map = true;
        sim.add_unit(u)
    };
    // Block 617's `x_internal`/`y_internal`, in the dump's own order.
    let a6 = put(&mut sim, 0, Pos::new(888, 7800));
    let a7 = put(&mut sim, 0, Pos::new(1032, 7800));
    let a8 = put(&mut sim, 0, Pos::new(936, 7944));
    let b6 = put(&mut sim, 1, Pos::new(1368, 7992));
    let b7 = put(&mut sim, 1, Pos::new(1491, 7973));
    let b8 = put(&mut sim, 1, Pos::new(1388, 8131));
    // And block 617's `orders_x`/`orders_y`.
    sim.units[a6].orders_pos = Pos::new(888, 7800);
    sim.units[a7].orders_pos = Pos::new(1032, 7800);
    sim.units[a8].orders_pos = Pos::new(936, 7944);
    sim.units[b6].orders_pos = Pos::new(1368, 7992);
    sim.units[b7].orders_pos = Pos::new(1320, 7800);
    sim.units[b8].orders_pos = Pos::new(1176, 8088);
    let from = sim.units[a6].pos;
    assert_eq!(
        sim.find_attack_pos(a6, Obj::Unit(b6), from, crate::fight::SITE_ATTACK_POS_FIGHT),
        Some(Pos::new(1080, 8280)),
        "the golden dump's own `orders_x`/`orders_y` for `0/6` at block 618"
    );
}

/// **The golden record's own *ranged* chase** (§32.2): chapter two's
/// slinger `0/9`, ordered onto the hoplite `1/8` at the end of frame
/// 621, asks where to stand and run112's dump answers
/// `(1608, 8184)` — `orders_x`/`orders_y` at block 622, with a
/// six-node path stack beneath it.
///
/// Every number is the dump's: `0/9` at `(888, 8376)` and `1/8` at
/// `(2472, 7944)` are block 622's `x_internal`/`y_internal`, and the
/// answer is that block's `MOVEORDER x`/`y`. The target carries no
/// order at all there (`STACK<TYPE> length 0`), which is what keeps
/// the call off the flanking branch (`Sim::chase_lead`).
///
/// The whole chain is pinned by one point: `stand` is 1008, the band
/// is `[912, 1104]` by `0x60`, the bearing is `find_angle(from −
/// target)` and the first candidate of the **inner** ring is the
/// answer.
///
/// Made to fail on purpose by sweeping from `spot` instead of
/// `spot − 0xc0` — one ring out on the same bearing, and the call
/// then answers `(1416, 8232)`.
#[test]
fn chapter_two_s_ranged_chase_stands_where_the_golden_dump_puts_it() {
    use crate::combat::Profile;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let mut kind = |max_range: i32, hits: i32| {
        sim.add_unit_type(crate::UnitType {
            hits,
            combat: Profile {
                attack: 15,
                max_range,
                uber_size: 1,
                block_radius: 48,
                big_radius: 48,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        })
    };
    let slinger = kind(6, 85);
    let hoplite = kind(0, 120);
    let put = |sim: &mut crate::Sim, who: crate::Player, ty: usize, p: Pos, hits: i32| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        let h = sim.add_unit(u);
        sim.units[h].orders_pos = p;
        h
    };
    let a9 = put(&mut sim, 0, slinger, Pos::new(888, 8376), 85);
    let b8 = put(&mut sim, 1, hoplite, Pos::new(2472, 7944), 120);
    // The stand-off the band is built on, stated so a change to §17.3
    // fails here rather than silently moving the ring.
    assert_eq!(
        sim.attack_dist(Obj::Unit(a9), Obj::Unit(b8)),
        1468,
        "block 622's own geometry"
    );
    let from = sim.units[a9].pos;
    assert_eq!(
        sim.find_attack_pos(a9, Obj::Unit(b8), from, crate::fight::SITE_ATTACK_POS_FIGHT),
        Some(Pos::new(1608, 8184)),
        "run112's own `orders_x`/`orders_y` for `0/9` at block 622"
    );
    // **The far arm takes the same road and a wider stand-off.** Past
    // `(range + 8) × 0xc0` the answer is unconditional — no `is_in_
    // range` test — so a chaser this far out still gets a ring point
    // and not the target's seat, which is what the whole of §32.2
    // exists to stop.
    let far = put(&mut sim, 0, slinger, Pos::new(888, 16_000), 85);
    let p = sim
        .find_attack_pos(
            far,
            Obj::Unit(b8),
            sim.units[far].pos,
            crate::fight::SITE_ATTACK_POS_FIGHT,
        )
        .expect("the far arm sweeps rather than falling through");
    assert_ne!(
        p, sim.units[b8].pos,
        "a chaser must never be sent to the target's own seat"
    );
    assert!(
        !sim.is_in_range_at(Obj::Unit(far), p, Obj::Unit(b8)),
        "the far arm's spot is accepted without the range test (`local_2c`)"
    );
}

/// The ring itself, read back as the seventeen quarter-tile centres
/// `find_open_slots` walks — the south-west corner first, then north
/// up the west side, east along the north, south down the east and
/// west along the south, closing on the corner it started from.
///
/// Made to fail on purpose by walking the ring the other way round —
/// the wrap is not symmetric, and the reversed table answers thirteen
/// slots along the south, east and north sides and never the west.
#[test]
fn the_open_slot_ring_walks_four_sides_and_closes() {
    use crate::combat::Profile;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 120,
        combat: Profile {
            uber_size: 1,
            block_radius: 48,
            big_radius: 48,
            hoplites: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut crate::Sim, who: crate::Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 120);
        u.ty = Some(ty);
        u.on_map = true;
        sim.add_unit(u)
    };
    // The asker stands far away so nothing it carries rejects a slot.
    let asker = put(&mut sim, 0, Pos::new(20_000, 20_000));
    let target = put(&mut sim, 1, Pos::new(1368, 7992));
    sim.units[asker].orders_pos = sim.units[asker].pos;
    sim.units[target].orders_pos = sim.units[target].pos;
    let got = sim.find_open_slots(asker, target);
    let cell = |x: i32, y: i32| Pos::new(x * 0x30 + 0x18, y * 0x30 + 0x18);
    // `1/6` sits in unit cell (28, 166); `r` is 1 + 1 + 4 and the step
    // is 2 × 1 + 1.
    let want: Vec<Pos> = [
        (22, 172),
        (22, 169),
        (22, 166),
        (22, 163),
        (22, 160),
        (25, 160),
        (28, 160),
        (31, 160),
        (34, 160),
        (34, 163),
        (34, 166),
        (34, 169),
        (34, 172),
        (31, 172),
        (28, 172),
        (25, 172),
        (22, 172),
    ]
    .iter()
    .map(|&(x, y)| cell(x, y))
    .collect();
    assert_eq!(got, want, "the ring, corner to corner and back");
}

/// `wrap_side` closes the ring both ways — arm 0 only decrements and
/// arm 1 only increments, so each needs one of them.
#[test]
fn the_ring_closes_both_ways() {
    assert_eq!(wrap_side(0), 8);
    assert_eq!(wrap_side(9), 1);
    for s in 1..=8 {
        assert_eq!(wrap_side(s), s);
    }
}

/// **The unit tail's `return 0`** (item 3520, `602b9f`–`602c1e`):
/// once the sweep has found nothing for a unit target, the call
/// refuses unless the asker's attack is mandatory and the target stands
/// `melee_far` or more off. run1166's packet takes `601604`'s 0 for the
/// Cruiser `1/25`'s non-mandatory attack on the fleeing citizen `0/5`.
#[test]
fn a_unit_target_with_no_spot_is_refused_unless_the_attack_is_mandatory_and_far() {
    use crate::combat::Profile;
    use crate::orders::QueuePos;
    use crate::world::World;
    let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
    sim.leaders.at_war[0][1] = true;
    sim.leaders.at_war[1][0] = true;
    let ty = sim.add_unit_type(crate::UnitType {
        hits: 85,
        combat: Profile {
            attack: 15,
            max_range: 6,
            uber_size: 1,
            block_radius: 48,
            big_radius: 48,
            combat_role: true,
            ..Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |sim: &mut crate::Sim, who: crate::Player, p: Pos| {
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 85);
        u.ty = Some(ty);
        u.on_map = true;
        sim.add_unit(u)
    };
    let me = put(&mut sim, 0, Pos::new(888, 8376));
    let near = put(&mut sim, 1, Pos::new(2472, 7944));
    let far = put(&mut sim, 1, Pos::new(4800, 8376));
    // `melee_far(6)` is 1920: `near` stands 1642 off, `far` 3912.
    for (t, mandatory, refused) in [
        (near, false, true),
        (far, false, true),
        (near, true, true),
        (far, true, false),
    ] {
        sim.units[me].orders.clear();
        sim.add_attack_order(me, Obj::Unit(t), QueuePos::New, mandatory, false);
        sim.units[me].combat.mandatory = mandatory;
        assert_eq!(
            sim.attack_pos_refused(me, Obj::Unit(t)),
            refused,
            "target {t}, mandatory {mandatory}"
        );
    }
}
