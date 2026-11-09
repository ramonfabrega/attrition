use super::*;
use crate::orders::QueuePos;
use crate::world::{Cell, Terrain, World};
use crate::{Sim, Tuning, Unit, UnitType};

/// A flat land world with two `BLOCK_RADIUS 1` walkers on it —
/// `coll_size 1`, the only value any capture has.
fn pair(a: Pos, b: Pos) -> (Sim, usize, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        moves: 25,
        combat: crate::combat::Profile {
            block_radius: 48,
            big_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let make = |sim: &mut Sim, o: i16, at: Pos| {
        let mut u = Unit::new(0, o, at, 40);
        u.ty = Some(ty);
        let i = sim.add_unit(u);
        sim.units[i].movement.speed = 25;
        sim.units[i].movement.turning = crate::movement::Turning {
            type_turn_speed: crate::movement::degrees_to_angle(45).0,
            packed: false,
            instant_from_stop: true,
            wide_limit: false,
        };
        i
    };
    let x = make(&mut sim, 0, a);
    let y = make(&mut sim, 1, b);
    (sim, x, y)
}

/// **`find_unit_with_radius` asks the disc's cells, not the map**
/// (`ObjectsData::find_unit_with_radius@00659890`, item 3782,
/// `docs/AI.md` §378.2): at `r_coll` 0 the disc is ring 0, the spot's
/// own 768-unit cell, so a unit whose `big_radius` reaches the spot
/// from the next cell is not asked. Ring 1 (`r_coll` 0x180) takes the
/// neighbours in. The Turks' Attack Sub `1/98` on East Indies' 3833
/// was offered (1512, 5640), 237 west of its leader `1/49` at (1747,
/// 5674) and one cell over, and the original took it.
#[test]
fn a_block_zero_query_asks_only_the_spot_s_own_cell() {
    let (mut sim, lead, other) = pair(Pos::new(1747, 5674), Pos::new(9000, 9000));
    let wide = sim.add_unit_type(UnitType {
        combat: crate::combat::Profile {
            block_radius: 48,
            big_radius: 300,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    sim.units[lead].ty = Some(wide);
    let _ = other;
    let west = Pos::new(1512, 5640);
    assert_ne!(
        west.cell(),
        sim.units[lead].pos.cell(),
        "the spot is a cell west"
    );
    assert!(
        !sim.find_unit_with_radius(0, west, None),
        "ring 0 is the spot's own cell: the leader one cell east is not asked"
    );
    assert!(
        sim.find_unit_with_radius(0x180, west, None),
        "ring 1 takes the neighbouring cells, and the leader is within 300"
    );
    assert!(
        sim.find_unit_with_radius(0, Pos::new(1600, 5640), None),
        "in the leader's own cell, 150 away, it is asked and refuses"
    );
}

#[test]
fn a_reused_figure_clears_its_old_body_with_the_new_radius() {
    for radius in [48, 144] {
        let (mut sim, old, _) = pair(Pos::new(3000, 3000), Pos::new(9000, 9000));
        sim.coll_remove(old);
        sim.units[old].health = 0;
        sim.units[old].on_map = false;
        sim.units[old].guys.push(crate::anim::Guy::fresh(-1));
        // A closed occupant keeps a body that differs from its unit point.
        let body = ucell_centre(Pos::new(80, 80));
        sim.units[old].movement.body.pos = body;
        let ghost = Pos::new(82, 80);
        sim.coll.set(ghost.x, ghost.y, true);
        let unit_cell = ucell(sim.units[old].pos);
        sim.coll.set(unit_cell.x, unit_cell.y, true);
        let ty = sim.add_unit_type(UnitType {
            combat: crate::combat::Profile {
                block_radius: radius,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        });
        let destination = ucell_centre(Pos::new(140, 140));
        let mut newborn = Unit::new(0, 0, destination, 40);
        newborn.ty = Some(ty);
        let u = sim.add_unit(newborn);
        assert_eq!(
            sim.coll.get(ghost.x, ghost.y),
            radius == 48,
            "the new radius, not the old radius, controls the clear"
        );
        assert!(
            sim.coll.get(unit_cell.x, unit_cell.y),
            "the retained figure point is used, not the closed unit's point"
        );
        assert!(sim.coll.get(140, 140), "the new disc is painted");
        assert_eq!(sim.units[u].coll_at, Some(destination));
    }
}

/// §2's region gate is `get_tregion` of the *figure's tile*, and for
/// a boat lying on the water half of a coastal cell that is the cell's
/// **`region2`** — which never equals the cell's own `region`, so the
/// boat marks nothing there.
///
/// This is the whole gate: with the plain `region_of` the two agree,
/// every cell of the disc is written, and a `BLOCK_RADIUS 3` barge
/// fills its own world cell — which is what pushed a disembarking
/// passenger four hundred units inland (`docs/TRANSPORT.md` §6.4).
/// Put `tregion` back in `coll_paint` and the second half fails.
/// **Born at the building, then out** (`docs/COLLISION.md` §20, item
/// 1133): Great Sahara's AI camp `1/2001`, scaled down. Two citizens are
/// born on one unit cell and come out to either side; the second's exit
/// clears every cell of the birth disc its own disc does not cover,
/// including the first citizen's, and a unit born after both keeps what
/// it covers. run382's frame 5 turns on the hole: `1/2`'s sidestep
/// probe finds the cell diagonal to `1/1` clear.
/// Loop 2132: a second `coll_add` at a moved point, with the first disc
/// standing, is a stale disc nobody clears — 2088's fixture painted one
/// and its test passed on it while the walk held the step. A debug
/// build refuses it by name; the release harness never calls it so.
/// Made to fail first: without the assertion the second paint is silent.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "coll_remove first, or coll_move (Loop 2132)")]
fn a_second_paint_at_a_moved_point_is_refused_in_a_debug_build() {
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        combat: crate::combat::Profile {
            block_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let mut u = Unit::new(1, 1, Pos::new(4 * 48 + 24, 4 * 48 + 24), 40);
    u.ty = Some(ty);
    let u = sim.add_unit(u);
    // The same point again is idempotent, as `add_to_world` is.
    sim.coll_add(u);
    sim.units[u].pos = Pos::new(6 * 48 + 24, 4 * 48 + 24);
    sim.coll_add(u);
}

#[test]
fn the_second_citizen_out_of_a_camp_leaves_a_hole_in_the_first_s_block() {
    let mut world = World::new(8, 8);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        combat: crate::combat::Profile {
            block_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let centre = |x: i32, y: i32| Pos::new(x * 48 + 24, y * 48 + 24);
    let mut add = |o: i16, at: Pos| {
        let mut u = Unit::new(1, o, at, 40);
        u.ty = Some(ty);
        let u = sim.add_unit(u);
        sim.coll_add(u);
        u
    };
    // `1/1` out on the cell south of the birth, `1/2` three east, and a
    // third unit born after both whose block reaches the birth's top row.
    let a = add(1, centre(4, 5));
    let b = add(2, centre(7, 4));
    let c = add(3, centre(4, 2));
    let birth = centre(4, 4);
    assert!(sim.coll.get(5, 5), "the control: 1/1's block covers (5, 5)");
    sim.birth_come_out(a, birth, &[b, c]);
    sim.birth_come_out(b, birth, &[c]);
    for x in 3..=5 {
        for y in 4..=5 {
            assert!(!sim.coll.get(x, y), "1/2's exit cleared ({x}, {y})");
        }
        assert!(
            sim.coll.get(x, 6),
            "1/1's row out of the birth stands ({x}, 6)"
        );
        assert!(sim.coll.get(x, 3), "the later unit's cell stands ({x}, 3)");
    }
    for x in 6..=8 {
        for y in 3..=5 {
            assert!(sim.coll.get(x, y), "1/2's own block is whole ({x}, {y})");
        }
    }
}

#[test]
fn a_boat_on_a_coastal_cell_s_water_marks_no_collision_cells() {
    let mut world = World::new(8, 8);
    let land = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    // One coastal cell: land by `region`, with `region2` naming a sea
    // region and every tile of it surfaced as ocean — the shape a map
    // maker leaves where a shore cuts a cell.
    let sea = land + 1;
    let c = Cell::new(4, 4);
    let mut d = world.cell_data(c);
    d.flags |= crate::world::cell::HALFLAND;
    d.region2 = Some(sea);
    world.set_cell_data(c, d);
    for tx in 16..20 {
        for ty in 16..20 {
            world.set_tile_field(
                Pos::new(tx, ty),
                crate::world::tile::SURFACE,
                crate::world::tile::SURFACE_OCEAN,
            );
        }
    }
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        combat: crate::combat::Profile {
            block_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let centre = Pos::new(4 * 768 + 384, 4 * 768 + 384);

    // A land unit on a dry cell marks its disc, which is the control:
    // the gate is not simply refusing everything.
    let dry = Pos::new(384, 384);
    let mut a = Unit::new(0, 0, dry, 40);
    a.ty = Some(ty);
    let a = sim.add_unit(a);
    sim.coll_add(a);
    assert!(
        sim.coll.get(ucell(dry).x, ucell(dry).y),
        "a land unit on its own region's cell is in the index"
    );

    // The boat, on the ocean tile of the coastal cell: `get_tregion` is
    // `region2` and the cell's `region` is the land one, so nothing.
    let mut b = Unit::new(0, 1, centre, 40);
    b.ty = Some(ty);
    let b = sim.add_unit(b);
    sim.coll_add(b);
    let uc = ucell(centre);
    for dy in -1..=1 {
        for dx in -1..=1 {
            assert!(
                !sim.coll.get(uc.x + dx, uc.y + dy),
                "the coastal cell's water marks nothing at ({dx}, {dy})"
            );
        }
    }
}

/// **A move inside one world cell is gated by the old tile alone**
/// (`CollCheck::move_unit@00682ad0`, `docs/COLLISION.md` §2.4): a boat
/// born on the land half of a coastal cell and first moved onto its
/// water half, both discs inside the cell, sets its new disc ungated —
/// the block was found by the **old** tile's region. run572's Galleon
/// `1/170` leaves that block set behind it, and `1/109`'s step on tick
/// 16238 reads the ghost as the hit cell.
///
/// Written by making it fail: with the one-block form removed the set
/// pass takes the water tile's `region2` against the cell's `region`
/// and marks nothing, so the cells west of the old disc stay clear.
#[test]
fn a_move_inside_one_world_cell_sets_its_disc_by_the_old_tile_s_region() {
    let mut world = World::new(8, 8);
    let land = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let c = Cell::new(4, 4);
    let mut d = world.cell_data(c);
    d.flags |= crate::world::cell::HALFLAND;
    d.region2 = Some(land + 1);
    world.set_cell_data(c, d);
    // The cell's two western tile columns are ocean, the rest land.
    for tx in 16..18 {
        for ty in 16..20 {
            world.set_tile_field(
                Pos::new(tx, ty),
                crate::world::tile::SURFACE,
                crate::world::tile::SURFACE_OCEAN,
            );
        }
    }
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        combat: crate::combat::Profile {
            block_radius: 3 * 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let at = |x: i32, y: i32| Pos::new(x * 48 + 24, y * 48 + 24);
    let from = at(75, 70);
    let to = at(68, 70);
    let mut b = Unit::new(0, 1, from, 40);
    b.ty = Some(ty);
    let b = sim.add_unit(b);
    sim.coll_add(b);
    assert!(
        sim.coll.get(78, 70),
        "the birth disc stands on the land tile"
    );
    assert!(!sim.coll.get(65, 70), "and does not reach the west");
    sim.coll_move(b, from, to);
    assert!(!sim.coll.get(78, 70), "the old disc's far edge is cleared");
    assert!(
        sim.coll.get(65, 70),
        "the new disc is set whole though its tile is water"
    );
}

/// **A snap onto the point the unit already holds still seats the
/// crew** — `005f8d20` has no early return for an unchanged position,
/// and `crates/sim` had one until item 210 (`docs/MOVEMENT.md`, "Who
/// writes it, and when").
///
/// Written by making it fail: with the `if from == to { return true }`
/// back in, the figure stays where it was dragged to.
#[test]
fn a_snap_to_the_point_already_held_still_seats_the_crew() {
    let (mut sim, a, _b) = pair(Pos::new(0x1800, 0x1800), Pos::new(0x4800, 0x4800));
    // One crew figure with a track offset, seated, and then dragged
    // away — a figure that is still walking after its leader.
    sim.units[a].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
    sim.art.tracks.insert(2, (-48, -192));
    sim.seat_guys(a);
    let seated = sim.units[a].guys[1].follow.expect("a tracked crew guy").des;
    let behind = Pos::new(seated.x - 300, seated.y - 300);
    sim.units[a].guys[1].follow.as_mut().unwrap().body.pos = behind;
    assert_ne!(behind, seated);

    let held = sim.units[a].pos;
    assert!(sim.set_new_location(a, held, true));
    let f = sim.units[a].guys[1].follow.expect("still tracked");
    assert_eq!(f.body.pos, seated, "the figure is put on its offset");
    assert_eq!(f.des, seated, "which is also where it is told to be");
    assert_eq!(
        f.facing, sim.units[a].movement.facing,
        "with its leader's facing written outright"
    );
    assert_eq!(sim.units[a].pos, held, "and the unit has not moved");
}

/// The index is written when a unit is added and moved when it walks —
/// §2, and the reason a fresh scan of the unit list is not the same
/// thing.
#[test]
fn the_index_follows_the_figure_and_is_not_refcounted() {
    let (sim, a, _b) = pair(Pos::new(0x1800, 0x1800), Pos::new(0x4800, 0x4800));
    let c = ucell(sim.units[a].pos);
    for (dx, dy) in spiral(1) {
        assert!(sim.coll.get(c.x + dx, c.y + dy), "the whole 3×3 block");
    }
    // Two cells over: the discs overlap on one column, and the mover
    // clears it on the way out even though the other unit is still
    // standing on it. That is the original's behaviour, kept.
    let d = Pos::new(c.x + 2, c.y);
    let (mut sim2, a2, _b2) = pair(ucell_centre(c), ucell_centre(d));
    assert!(sim2.coll.get(c.x + 1, c.y), "shared by both discs");
    // The figure goes with it (`Guy::set_new_location(guy 0, pos, 1)`),
    // so the disc moves now.
    sim2.set_new_location(a2, ucell_centre(Pos::new(c.x - 4, c.y)), true);
    assert!(
        !sim2.coll.get(c.x + 1, c.y),
        "the mover cleared a cell its neighbour still occupies"
    );
    let _ = &sim;
}

/// §16: **the disc follows guy 0, not the unit.** `CollCheck::move_unit`
/// has one caller, `Guy::set_new_location`, so a unit moved without
/// its figures — a push, `Unit::set_new_location(x, y, 0, 0)` — keeps
/// its bits on the old cell until its own `Guy::process` puts the
/// figure on the new point. Golden chapter eleven's guard, pushed by
/// its wagon on tick 732, is refused its own step on tick 733 by a
/// bit its old disc still holds.
#[test]
fn a_pushed_unit_s_disc_waits_for_its_figure() {
    let (mut sim, a, _b) = pair(
        ucell_centre(Pos::new(20, 20)),
        ucell_centre(Pos::new(30, 30)),
    );
    let old = Pos::new(19, 20);
    let to = ucell_centre(Pos::new(23, 20));
    assert!(sim.coll.get(old.x, old.y), "the disc stands on its cell");
    sim.set_new_location(a, to, false);
    assert!(
        sim.coll.get(old.x, old.y) && !sim.coll.get(24, 20),
        "a push leaves the disc where the figure is"
    );
    assert_eq!(sim.units[a].coll_at, Some(ucell_centre(Pos::new(20, 20))));
    sim.units[a].movement.body.pos = to;
    sim.coll_follow(a);
    assert!(
        !sim.coll.get(old.x, old.y) && sim.coll.get(24, 20),
        "the figure's move takes the disc with it"
    );
    assert_eq!(sim.units[a].coll_at, Some(to));
}

/// **A push before the pushed unit's own process takes its body's
/// turn** (`Guy::process@005e0230:28`, item 3571). `Guy::do_turn`
/// sets `guy_flags & 2` whoever calls it, and only `Guy::process`
/// clears it, after `Guy::move`; so a standing unit another unit
/// pushed earlier in the frame skips `Guy::move:108`'s own turn
/// toward its `des_angle`. East Indies' General `1/138` on 4336: the
/// Catapult `1/124` turns it one half-rate step, and the original's
/// body turns no further where this crate's turned a whole step more.
///
/// Written to fail first: with the mark taken at the head of the
/// unit's process the body turns again.
#[test]
fn a_push_before_the_pushed_unit_s_process_takes_its_body_s_turn() {
    let (mut sim, a, _b) = pair(
        ucell_centre(Pos::new(20, 20)),
        ucell_centre(Pos::new(30, 30)),
    );
    sim.units[a].movement.turning.instant_from_stop = false;
    let start = Angle(0x5555_5555);
    let bearing = Angle(0x0936_0000);
    sim.units[a].movement.facing = start;
    sim.units[a].movement.frame_facing = start;
    sim.units[a].movement.heading = bearing;
    sim.push_turn(a, bearing);
    let pushed = sim.units[a].movement.facing;
    assert_ne!(pushed, start, "the push turned guy 0");
    let mut events = Vec::new();
    let frame = sim.frame;
    sim.process_unit(a, frame, &mut events);
    assert_eq!(
        sim.units[a].movement.facing, pushed,
        "the pushed body took its own turn as well"
    );
    // And the mark is spent: the next frame's body turns.
    let frame = sim.frame;
    sim.process_unit(a, frame, &mut events);
    assert_ne!(
        sim.units[a].movement.facing, pushed,
        "the body never turned once the mark was cleared"
    );
}

/// §4 again, and the half of it that is a **write into the order**:
/// the probe records the point it refused in `coll_x`/`coll_y`, and
/// `move_step`'s own stores must not put the stale pair back.
///
/// The original steps on a pointer to the move order, so the write is
/// simply there for everything downstream; this crate steps on a copy
/// and has to take it back. The arm that shows the difference is the
/// **blocked stand while a turn is still owed** (§5): it stores the
/// move and returns without stepping, so a copy written back over the
/// probe's is the whole of the bug. run10's `1/1` is the case — on the
/// original's frame 792 it stands mid-turn with
/// `coll_x/coll_y (40539, 18258)` and this crate held the pair it had
/// refused fifteen frames earlier.
#[test]
fn a_blocked_stand_keeps_the_point_the_probe_refused() {
    // Facing north, ordered sixty degrees round to the east: one
    // frame's turn leaves fifteen degrees owed, which is under
    // `move_step`'s forty-five and so still takes the step — into the
    // unit standing on the next cell up and to the right.
    let a = Pos::new(30 * 0x30 + 40, 30 * 0x30 + 8);
    let b = ucell_centre(Pos::new(31, 29));
    let (mut sim, x, y) = pair(a, b);
    sim.units[x].movement.turning.instant_from_stop = false;
    sim.units[x].movement.facing = crate::movement::Angle(0);
    sim.order_move(x, Pos::new(a.x + 866, a.y - 500));
    sim.tick();

    assert_eq!(sim.units[x].pos, a, "it stood: the turn was still owed");
    assert_eq!(
        sim.units[x].collide_o, sim.units[y].index,
        "and it named the unit it would have walked into"
    );
    let coll = sim
        .current_move(x)
        .and_then(|m| m.coll)
        .expect("the refused point survives the blocked stand's own store");
    assert_eq!(coll, Pos::new(1497, 1431), "the point the step proposed");
    assert_ne!(ucell(coll), ucell(a), "and it is not the unit's own cell");
}

/// **§2.2 — the sixty-fourth frame puts back what the walker took.**
///
/// The index is not refcounted, so a unit leaving a cell clears the
/// bits another unit is still standing on: `A` stands at unit cell
/// `(10, 10)` with a block of `9..=11` square, `B` steps from
/// `(11, 11)` to `(12, 12)`, and its clear pass erases `(10, 10)`,
/// `(10, 11)` and `(11, 10)` — three corners of a block whose owner
/// never moved. `Guy::process@005e0230` is what puts them back, on the
/// frames where `(frame + o) % 64 == 0`.
///
/// Both halves are asserted, because the first is what makes the
/// second a check rather than a tautology: **the hole is real** on the
/// frame after the step, and **gone** by the repaint. Delete the call
/// in `process_movement` and the second half fails; the run69 diff
/// fails with it, which is where this was found (`docs/PATHFINDER.md`
/// §17).
#[test]
fn the_sixty_fourth_frame_puts_back_the_block_a_walker_cleared() {
    let a = ucell_centre(Pos::new(10, 10));
    let b = ucell_centre(Pos::new(11, 11));
    let (mut sim, x, y) = pair(a, b);
    assert_eq!(sim.units[x].index, 0, "A's object number sets the phase");
    for c in [Pos::new(10, 10), Pos::new(10, 11), Pos::new(11, 10)] {
        assert!(sim.coll.get(c.x, c.y), "A's block starts whole at {c:?}");
    }

    // Off the phase, so the repaint cannot run before the hole is
    // looked at: A repaints on frames divisible by 64.
    while (sim.frame + i64::from(sim.units[x].index)).rem_euclid(64) == 0 {
        sim.tick();
    }
    sim.set_new_location(y, ucell_centre(Pos::new(12, 12)), true);
    let holes = [Pos::new(10, 10), Pos::new(10, 11), Pos::new(11, 10)];
    for c in holes {
        assert!(
            !sim.coll.get(c.x, c.y),
            "B's clear pass took {c:?} out of A's block — the index is not \
                 refcounted, and this is the hole the repaint exists for"
        );
    }

    // A stood still throughout, so its `avg_speed` is zero and the
    // next phase frame re-marks the whole disc.
    for _ in 0..64 {
        sim.tick();
    }
    for c in holes {
        assert!(
            sim.coll.get(c.x, c.y),
            "the sixty-fourth frame put {c:?} back"
        );
    }
}

/// **§2.3 — crossing a world cell puts back what the walker took**,
/// and it is the only healer a unit on the march ever meets.
///
/// `Unit::set_new_location` brackets the coordinate write with
/// `Object::remove_from_world` and `Object::add_to_world` on a
/// **world**-cell change, and the two walk the same disc around the
/// figure's own — still unmoved — point, one clearing and one setting.
/// So the pair sets the whole old block, and `move_unit` then carries a
/// healed disc onto the new cell. §2.2's sixty-fourth frame cannot do
/// this job: it asks `avg_speed == 0`.
///
/// Both halves are asserted, and the second is what makes the first a
/// check: the **same** puncture and the **same** one-cell step leave
/// the hole standing when the step does not cross a world cell. Delete
/// the `coll_add` in [`Sim::set_new_location`] and the first half
/// fails; make it unconditional and the second does.
#[test]
fn a_world_cell_crossing_heals_the_block_a_neighbour_punctured() {
    // `A` at unit cell `(ax, 10)`, `B` at `(ax + 2, 11)`, whose block
    // covers `A`'s eastern column. `B` walks two cells south and its
    // clear pass takes the row `y = 10` with it — `(ax + 1, 10)` is
    // `A`'s, and `A` has not moved.
    let punch = |ax: i32| {
        let (mut sim, a, b) = pair(
            ucell_centre(Pos::new(ax, 10)),
            ucell_centre(Pos::new(ax + 2, 11)),
        );
        assert!(
            sim.coll.get(ax + 1, 10),
            "A's block starts whole at ({}, 10)",
            ax + 1
        );
        sim.set_new_location(b, ucell_centre(Pos::new(ax + 2, 13)), true);
        assert!(
            !sim.coll.get(ax + 1, 10),
            "B's clear pass did not punch ({}, 10) out of A's block",
            ax + 1
        );
        // One cell east. `move_unit` re-sets only the column the block
        // is entering, so nothing but the crossing can put this hole
        // back — it is `A`'s new **centre**.
        sim.set_new_location(a, ucell_centre(Pos::new(ax + 1, 10)), true);
        sim.coll.get(ax + 1, 10)
    };
    // 15 → 16 crosses: sixteen unit cells to a world cell, so the
    // object is re-seated and the disc repainted on the way.
    assert!(
        punch(15),
        "the world-cell crossing did not repaint A's own disc"
    );
    // 5 → 6 does not, and the same hole is still standing.
    assert!(
        !punch(5),
        "a step inside one world cell repainted the disc — the heal is \
             `Unit::set_new_location`'s `local_10`, not every move"
    );
}

/// §4: a unit walking into another one detects it, names it, and
/// records the refused point.
#[test]
fn a_walker_detects_the_unit_in_front_and_names_it() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    let mut hit = None;
    for _ in 0..40 {
        sim.tick();
        if sim.units[x].collide_o >= 0 {
            hit = Some(sim.units[x].collide_o);
            break;
        }
    }
    assert_eq!(hit, Some(sim.units[y].index), "it names the unit in front");
    assert_eq!(sim.units[x].collide_who, 0);
    assert_eq!(sim.units[x].collide_guy, 0);
    assert!(sim.units[x].collide >= 1, "the counter moved");
    assert!(
        sim.current_move(x).and_then(|m| m.coll).is_some(),
        "`coll_x`/`coll_y` carry the refused point"
    );
}

/// §6 step 6: the recovery snaps the walker onto its own cell centre
/// and re-plans on the 48-grid, and the plan goes **round** the unit in
/// the way rather than through it — which is `valid_ucoord`'s half of
/// the mechanic (`docs/PATHFINDER.md` §6).
#[test]
fn the_recovery_snaps_to_the_cell_centre_and_paths_around() {
    let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    sim.order_move(x, goal);
    let mut snapped = false;
    for _ in 0..60 {
        sim.tick();
        if sim.units[x].collide_frame != 0 {
            snapped = true;
            break;
        }
    }
    assert!(snapped, "the walk reached the unit in front");
    assert_eq!(
        sim.units[x].pos,
        ucell_centre(ucell(sim.units[x].pos)),
        "snapped onto its own cell centre"
    );
    assert!(
        sim.units[x].path.len() > 1,
        "the 48-grid plan is on the stack: {:?}",
        sim.units[x].path
    );
    assert!(
        sim.units[x]
            .path
            .iter()
            .any(|p| p.flags & path_flag::SIDESTEP != 0),
        "and its waypoints carry the unit grid's flag"
    );
    // It gets there, and it never walks over the unit standing still.
    let stood = sim.units[y].pos;
    for _ in 0..400 {
        sim.tick();
        assert!(
            (sim.units[x].pos.x - stood.x).abs() > 0x30
                || (sim.units[x].pos.y - stood.y).abs() > 0x30,
            "walked through it at {:?}",
            sim.units[x].pos
        );
    }
    assert!(sim.units[y].pos == stood, "the other one never moved");
}

/// §6 step 0, the twin of the test above: **an animal takes none of
/// it.** `resolve_unit_collision`'s first statement is
/// `SubObjectData::is_animal` (vftable offset 48, from the PDB's
/// `LF_ONEMETHOD` list — the map folds both overrides onto trivial
/// stubs and cannot name it), and when it answers, the body is the
/// `QUEUE_NEW` clear: the order list and the path go, and steps 1–6
/// never run.
///
/// The same walk, the same blocker, the same frame — and where a
/// player's unit snaps onto its cell centre and paths around, gaia's
/// stops dead where it stood, keeps no order, and never moves again.
/// This is run39's `8/2`, whose herd-mate blocks it on frame 69:
/// the original stands it at `(28856, 24197)` for the rest of the
/// capture, and this crate walked it round to the goal
/// (`docs/SYNC.md` §3.14).
#[test]
fn an_animal_drops_its_walk_where_it_stands_and_takes_no_step() {
    let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    // The walker is gaia's; everything else is the player's case.
    sim.units[x].owner = 8;
    sim.order_move(x, goal);
    // `collide_frame` is step 5's write and an animal never reaches
    // it; what names the blocker is `detect_unit_collision`, which
    // runs before `resolve_unit_collision` is called at all.
    let mut blocked = None;
    for _ in 0..60 {
        let before = sim.units[x].pos;
        sim.tick();
        if sim.units[x].collide_o >= 0 {
            blocked = Some(before);
            break;
        }
    }
    let stopped = blocked.expect("the walk reached the unit in front");
    assert_eq!(
        sim.units[x].pos, stopped,
        "an animal does not take §6 step 6's cell-centre snap"
    );
    assert_eq!(
        sim.units[x].collide_frame, 0,
        "and step 5's counter is never reached"
    );
    assert!(
        sim.units[x].orders.is_empty(),
        "the whole order list goes: {:?}",
        sim.units[x].orders
    );
    assert!(sim.units[x].path.is_empty(), "and so does the path");
    // And it stays there: no repath, no second attempt at the goal.
    for _ in 0..200 {
        sim.tick();
        assert_eq!(sim.units[x].pos, stopped, "it walked again");
    }
    assert_ne!(stopped, goal, "the goal was never reached");
    assert_eq!(sim.units[y].pos, b, "the blocker never moved");
}

/// **The other side of the same coin: an animal is a blocker like any
/// other, and a crewed walker pays the stand twice** (2026-09-06,
/// item 241).
///
/// §4.3's chain walk skips itself, non-units and aircraft, and asks
/// nothing about the owner —`detect_unit_collision@00617060:150-172`
/// tests `alive`, vfunc `+0x18`, `domain != 2`, `is_on_map` and
/// `is_here`, and no `who`. [`Sim::chain_hit`]'s `who < 8` fence is
/// **not** on this path: a step probes [`Sim::collide_here`], the
/// occupancy bitmask, which [`Sim::coll_paint`] writes for every unit
/// with a block, gaia's included. So East Indies' `1/20` is blocked
/// by gaia's animal `8/0` on run85's frame 7448 and the crate models
/// that, whatever else it does with the frame
/// (`docs/COLLISION.md` §8.2).
///
/// And it pays it **once per figure**: `Unit::set_anim@00616f40` has
/// two loops, the squad's `0 .. guy_mark` and the crew's `squad_size
/// .. guy_num`, and run85's 7448 spends one draw from each —
/// `Unit::set_anim+0x56` and `+0xb6`, both under
/// [`crate::anim::SITE_BLOCKED`]. A Merchant is `CREW_SIZE 1`, so the
/// frame owes two draws and not one.
#[test]
fn a_gaia_animal_blocks_a_player_s_walker_and_a_crew_pays_the_stand_twice() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    // The **blocker** is gaia's; the walker is a player's, which is
    // run85's 7448 the way round the animal test above is not.
    sim.units[y].owner = 8;
    for slot in 0..4 {
        sim.art.lengths.insert((1, slot), 40);
    }
    // Two figures, the Merchant's shape: `UBER_SIZE 1, CREW_SIZE 1`.
    sim.units[x].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(1)];
    sim.trace_phases = true;
    sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    let mut marks = None;
    for _ in 0..60 {
        sim.tick();
        if sim.units[x].collide_o >= 0 {
            marks = Some(sim.phase_marks.clone());
            break;
        }
    }
    let marks = marks.expect("gaia's animal blocked the player's walker");
    assert_eq!(
        (sim.units[x].collide_o, sim.units[x].collide_who),
        (sim.units[y].index, 8),
        "the blocker named is the animal, owner and all"
    );
    let at = marks
        .iter()
        .position(|(l, _)| l == crate::anim::SITE_BLOCKED)
        .expect("the blocked stand is marked on that frame");
    // How many draws the mark covers: walk the LCG from the word it
    // recorded to the next mark's (or the frame's end).
    let after = marks.get(at + 1).map_or(sim.rng.seed, |(_, w)| *w);
    let mut r = crate::combat::Rng::new(marks[at].1);
    let mut drew = 0;
    while r.seed != after && drew < 8 {
        r.get(0, 0xffff);
        drew += 1;
    }
    assert_eq!(
        (r.seed, drew),
        (after, 2),
        "one draw per figure — `Unit::set_anim`'s two loops, run85's \
             `Unit::set_anim+0x56` and `+0xb6` on 7448"
    );
}

/// **A unit that turns in place on its own waypoint stands when the
/// turn completes** (`docs/AI.md` §130, item 1500).
///
/// `Unit::move_step@005faf30`'s snap arm passes `UVar17` to
/// `Unit::set_anim`, and `UVar17` is `CHAR_DEFAULT` — a draw — when
/// the waypoint offsets were both zero at the top of the function and
/// the order list holds fewer than two orders (`005fb44c`–`005fb474`).
/// French East Indies' General `1/79` turned six frames on
/// (6840, 13704) and drew on 16857, the frame the turn finished. The
/// shape here is that one: a leg whose top waypoint is where the unit
/// stands, and a body facing the other way. With a second order
/// beneath the move the arm keeps the walk and draws nothing.
#[test]
fn a_unit_on_its_own_waypoint_stands_when_its_turn_completes() {
    for beneath in [false, true] {
        let a = Pos::new(31 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let far = Pos::new(31 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let (mut sim, x, _y) = pair(a, Pos::new(9000, 9000));
        sim.trace_phases = true;
        sim.order_move(x, far);
        sim.tick();
        let at = sim.units[x].pos;
        sim.units[x].path.push(crate::orders::PathData {
            to: at,
            tolerance: 0,
            flags: 0,
        });
        if let Some(crate::orders::Order {
            body: crate::orders::Body::Move(m),
            ..
        }) = sim.units[x].orders.back_mut()
        {
            m.waypoint = at;
            m.has_waypoint = true;
        }
        if beneath {
            let under = *sim.units[x].orders.back().expect("the move");
            sim.units[x].orders.push_front(under);
        }
        sim.units[x].movement.facing = crate::movement::Angle(0);
        let stands = |sim: &Sim| {
            sim.phase_marks
                .iter()
                .filter(|(l, _)| l == crate::anim::SITE_SNAP_STAND)
                .count()
        };
        sim.tick();
        assert_eq!(stands(&sim), 0, "the first frame turns in place");
        assert!(
            sim.phase_marks
                .iter()
                .any(|(l, _)| l == crate::anim::SITE_TURN_NEAR),
            "the near turn-in-place arm"
        );
        sim.tick();
        assert_eq!(sim.units[x].pos, at, "the snap takes no step");
        assert_eq!(
            stands(&sim),
            usize::from(!beneath),
            "`UVar17` is the stand with one order, the walk with two"
        );
    }
}

/// **The snap arm has a collision block of its own, and it resolves
/// nothing** (`docs/COLLISION.md` §5.4, item 360).
///
/// `Unit::move_step@005faf30` splits on `param_2 < local_28` and
/// probes on *each* side. The partial step's block is §5 — snap
/// through, wait on an owed turn, `resolve_unit_collision`, widen the
/// tolerance — and this crate spent it for both arms until Great
/// Lakes 9134 said otherwise. The snap's, at `005fb3bd`, clears the
/// collider the probe just named, consumes the waypoint where the
/// unit stands, and joins the accepted step's tail.
///
/// The shape here is 9134's without the capture: a walker one step
/// short of its final waypoint, and a blocker standing on it. The
/// blocker carries a move order of its own so that `do_move`'s own
/// waypoint probe (§5.1) takes neither of its two arms — its first
/// wants an action beneath the move, its second a collider that is
/// **not** moving — and the leg reaches `move_step` intact, which is
/// exactly how a formation hop reaches it.
#[test]
fn a_blocked_snap_stands_and_eats_its_waypoint_without_resolving() {
    // Neighbouring cell centres, 48 apart, and a walker fast enough to
    // cover that in one step — so the **first** proposal the walk
    // makes is already the snap onto its final waypoint, and §5's
    // partial arm is never reached.
    let a = Pos::new(31 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(32 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    for slot in 0..4 {
        sim.art.lengths.insert((1, slot), 40);
    }
    sim.trace_phases = true;
    sim.units[x].movement.speed = 60;
    // The blocker holds a move order it can never advance, so it is
    // "moving" to §5.1 — whose second arm wants a collider that is
    // *not* — and standing still to everyone else.
    sim.units[y].movement.speed = 0;
    sim.order_move(y, Pos::new(39 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    sim.order_move(x, b);
    let mut hit = None;
    for _ in 0..8 {
        let before = sim.units[x].pos;
        sim.tick();
        if sim
            .phase_marks
            .iter()
            .any(|(l, _)| l == crate::anim::SITE_SNAP_BLOCKED)
        {
            hit = Some(before);
            break;
        }
        assert!(
            !sim.phase_marks
                .iter()
                .any(|(l, _)| l == crate::anim::SITE_BLOCKED),
            "the partial step's block fired first: this walk never \
                 reaches the snap"
        );
    }
    let before = hit.expect("the walker snapped into the blocker");
    assert_eq!(
        sim.units[x].pos, before,
        "the snap arm takes no step: the unit stands where it was"
    );
    assert_eq!(
        (sim.units[x].collide_o, sim.units[x].collide_who),
        (-1, -1),
        "`field_0x8a = 0xffff` / `field_0xb3 = 0xff`: the arm clears \
             the collider the probe named"
    );
    // The waypoint is consumed where the unit stands, and this one was
    // the **final** leg, so the pop falls into `move_step`'s own tail:
    // `set_angle`, the pathed bit, `kill_current_order`. A *middle*
    // leg returns 1 instead and the walk resumes next frame from the
    // same place — which is what Great Lakes 9134 is, and what
    // `run53_s_24000_frames_put_the_ceiling_where_run33_did` pins
    // against the original rather than against this crate.
    assert!(
        sim.units[x].path.is_empty() && sim.units[x].orders.is_empty(),
        "the final waypoint is consumed and the order killed: path \
             {:?}, orders {:?}",
        sim.units[x].path,
        sim.units[x].orders
    );
    assert_eq!(
        sim.units[x].collide_frame, 0,
        "`resolve_unit_collision` never ran, so nothing dated the \
             collision — run97's 9135 carries a `collide_frame` nine \
             hundred frames stale for the same reason"
    );
    // `coll_x`/`coll_y` — the point the probe refused, stored into the
    // order — cannot be read here: this waypoint was the final one, so
    // the order it was stored into is gone by the time the tick ends.
    // `great_lakes_9134_is_the_snap_arm_s_blocked_stand` in
    // `rondata::diff` is where that field is pinned, against the
    // original's own `MOVEORDER` record on a middle leg.
}

/// **`collide` chooses the grid the re-plan runs on**
/// (`docs/ORDERS.md` §4.4, `Unit::do_move@005f7b30`'s `field_0x88`
/// test).
///
/// Past the `% 5` roll the original branches: a unit that has *not*
/// been colliding drops its loose near waypoints (tolerance 1 to
/// `0x60`) and plans on the **tile** grid (`find_tpath`, waypoints at
/// `tolerance 0x60`, no flags); one that has keeps them and plans on
/// the **48** grid (`find_upath`, `flags 2`) — the finer one, and the
/// only one that knows units are in the way. The plan laid here is
/// loose (`0x30`) so the two arms part; a tolerance-0 plan survives
/// both.
///
/// The state here is run53's `1/7` on frame 5502, hand-built: a unit
/// standing **on** its own waypoint with the 48-grid plan
/// `resolve_unit_collision` laid still under it. The straight-line
/// check finds a top equal to its position, refuses, and the frame
/// rolls. Taking the tile arm there popped that plan and walked the
/// unit back into the collider it had just recovered from, frame after
/// frame, to the end of the capture (item 204).
#[test]
fn collide_sends_the_re_plan_to_the_unit_grid_not_the_tile_grid() {
    let start = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let plan = |collide: i16, tolerance: i32| {
        let (mut sim, x, _y) = pair(start, Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18));
        sim.order_move(x, goal);
        sim.tick();
        // The 48-grid plan, and the unit sitting on the top of it.
        let here = sim.units[x].pos;
        sim.units[x].path.push(PathData {
            to: Pos::new(here.x - 0x30, here.y),
            tolerance,
            flags: path_flag::SIDESTEP,
        });
        sim.units[x].path.push(PathData {
            to: here,
            tolerance,
            flags: path_flag::SIDESTEP,
        });
        sim.units[x].line_ok = false;
        sim.units[x].collide = collide;
        if let Some(front) = sim.units[x].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.has_waypoint = true;
            m.waypoint = here;
        }
        sim.trace_phases = true;
        sim.phase_marks.clear();
        sim.tick();
        assert!(
            sim.phase_marks
                .iter()
                .any(|(l, _)| l == crate::orders::SITE_MOVE_GRID),
            "a unit standing on its own waypoint rolls, on both arms: {:?}",
            sim.phase_marks
        );
        sim.units[x].path.clone()
    };

    let tiles = plan(0, 0x30);
    let units = plan(1, 0x30);
    assert!(
        tiles.iter().any(|p| p.tolerance == 0x60),
        "the tile grid's waypoints carry its half-tile tolerance: {tiles:?}"
    );
    assert!(
        !tiles.iter().any(|p| p.flags & path_flag::SIDESTEP != 0),
        "and the loose plan under it was dropped: {tiles:?}"
    );
    assert!(
        units.iter().any(|p| p.flags & path_flag::SIDESTEP != 0),
        "a unit that has been colliding keeps it and plans on the 48 \
             grid: {units:?}"
    );
    // **And a tolerance-0 plan is not loose** (item 669): the unwind
    // pops 1 to `0x60`, unsigned (`0x5f8ad2`), so the tile arm keeps
    // an exact point too. A fresh `find_upath` plan is the tolerance-0
    // kind, and until item 669 this test laid one and asserted the
    // tile arm dropped it — that was the misread `< 0x60`.
    let exact = plan(0, 0);
    assert!(
        exact
            .iter()
            .any(|p| p.flags & path_flag::SIDESTEP != 0 && p.tolerance == 0),
        "the tile arm keeps an exact point: {exact:?}"
    );
}

/// §6 step 6's last store is **`dest = 0`, and nothing else** — and
/// what that buys is a frame the original does not spend.
///
/// `005f9d30`'s two closing blocks both clear the order's `+0x10`
/// through `update_order()->get_move_order()`; the only thing the
/// successful one adds is the pause roll. So the recovery does **not**
/// take the waypoint: `do_move`'s own `dest == 0` block does, on the
/// next frame, and that block clears `unit_masks & 8` and runs the
/// leg's arrival test.
///
/// Taking it here instead is the difference between two frames. The
/// top of a fresh `find_upath` plan is the unit's own snapped cell, so
/// a unit handed that waypoint stands *on* it with `dest` set: the
/// arrival test never runs, the straight-line check finds a top equal
/// to its position, and the frame falls through to the grid roll
/// (`SITE_MOVE_GRID`). run53's `1/7` spent that draw on frame 5502 and
/// the original spent nothing (item 204).
#[test]
fn the_recovery_leaves_the_waypoint_for_do_move_to_take() {
    let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, _y) = pair(a, b);
    sim.order_move(x, goal);
    sim.trace_phases = true;
    let mut snapped = false;
    for _ in 0..60 {
        sim.tick();
        if sim.units[x].collide_frame != 0 {
            snapped = true;
            break;
        }
    }
    assert!(snapped, "the walk reached the unit in front");
    assert!(
        sim.units[x].path.len() > 1,
        "the 48-grid plan is on the stack: {:?}",
        sim.units[x].path
    );
    let mo = sim.current_move(x).expect("still a move");
    assert!(
        !mo.has_waypoint,
        "the recovery clears `dest`; it does not take the top: {mo:?}"
    );

    // The next frame takes it — and the leg it takes is the snapped
    // cell the unit is already standing on, so the arrival test pops
    // it and the frame ends there. No grid roll.
    sim.phase_marks.clear();
    sim.tick();
    assert!(
        !sim.phase_marks
            .iter()
            .any(|(m, _)| m == crate::orders::SITE_MOVE_GRID),
        "the frame after the snap spends no grid draw: {:?}",
        sim.phase_marks
    );

    // And it gets where it was going rather than oscillating: the
    // pop-and-re-plan pair this used to make was a two-frame livelock
    // that ran to the end of run53's capture.
    let mut arrived = false;
    for _ in 0..400 {
        sim.tick();
        if sim.units[x].orders.is_empty() {
            arrived = true;
            break;
        }
    }
    assert!(
        arrived,
        "the walk finished; it stalled at {:?} with {:?}",
        sim.units[x].pos, sim.units[x].path
    );
}

/// **Step 5 waits on a transport cast as on a move** (item 1410,
/// `005fa4f4`): `y` stands with a `TRANSPORT` cast at its head and the
/// move under it; `x` walking into it waits where it stands, and
/// repaths for any other spell.
#[test]
fn a_collider_casting_the_transport_spell_is_waited_for() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let run = |spell: i32| {
        let (mut sim, x, y) = pair(a, b);
        sim.order_move(x, goal);
        sim.order_move(y, Pos::new(35 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        sim.tick();
        sim.add_cast_order(y, spell);
        sim.units[x].collide_o = sim.units[y].index;
        sim.units[x].collide_who = 0;
        sim.units[x].waiting_on = false;
        sim.units[y].collide_o = -1;
        sim.units[y].collide_who = -1;
        sim.units[y].waiting_on = false;
        sim.resolve_unit_collision(x);
        sim.units[x].waiting_on
    };
    assert!(
        run(crate::orders::spell::TRANSPORT),
        "waits for the boat's cast"
    );
    assert!(!run(crate::orders::spell::PACK), "another cast is repathed");
}

/// **A walking animal is never waited for** (item 1561, Great Sahara's
/// 1247): step 5's `LeaderData::is_enemy` reads a gaia owner's
/// `diplos[8]` as `treaties[0]`, so the collider is "an enemy" and the
/// walker repaths instead of standing. A walking player-owned unit that
/// is not at war with it is still waited for.
#[test]
fn a_walking_animal_in_the_way_is_not_waited_for() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let run = |owner: u8| {
        let (mut sim, x, y) = pair(a, b);
        sim.order_move(x, goal);
        sim.order_move(y, Pos::new(35 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        sim.tick();
        sim.units[y].owner = owner;
        sim.units[x].collide_o = sim.units[y].index;
        sim.units[x].collide_who = owner as i8;
        sim.units[x].waiting_on = false;
        sim.units[y].collide_o = -1;
        sim.units[y].collide_who = -1;
        sim.units[y].waiting_on = false;
        sim.resolve_unit_collision(x);
        sim.units[x].waiting_on
    };
    assert!(run(1), "a walking neighbour at peace is waited for");
    assert!(!run(8), "a walking animal is repathed around");
}

/// A wait chain reads a retained slot, including a closed transport.
/// Older occupants of the same number must not shadow the newest one.
#[test]
fn a_collision_wait_chain_reads_the_latest_retained_slot() {
    for alive in [false, true] {
        for waiting in [false, true] {
            let a = ucell_centre(Pos::new(30, 30));
            let b = ucell_centre(Pos::new(28, 30));
            let (mut sim, x, y) = pair(a, b);
            sim.order_move(x, ucell_centre(Pos::new(20, 30)));
            sim.order_move(y, ucell_centre(Pos::new(35, 30)));
            sim.tick();
            sim.units[x].path.push(PathData {
                to: ucell_centre(Pos::new(29, 30)),
                tolerance: 0,
                flags: path_flag::SIDESTEP,
            });
            // Two incarnations of object 2: the previous one has the
            // opposite bit, so reading the first match is observable.
            for current in [false, true] {
                let mut z = Unit::new(0, 2, ucell_centre(Pos::new(50, 50)), 40);
                z.health = if current && alive { 40 } else { 0 };
                z.on_map = current && alive;
                z.waiting_on = if current { waiting } else { !waiting };
                sim.add_unit(z);
            }
            sim.repaths[0] = 0;
            sim.units[x].collide = 0;
            sim.units[y].collide = 0;
            sim.units[x].collide_o = sim.units[y].index;
            sim.units[x].collide_who = 0;
            sim.units[x].waiting_on = false;
            sim.units[y].collide_o = 2;
            sim.units[y].collide_who = 0;
            sim.units[y].waiting_on = true;
            sim.resolve_unit_collision(x);
            assert_eq!(
                sim.units[x].waiting_on, !waiting,
                "alive={alive}, waiting={waiting}"
            );
            assert_eq!(
                sim.repaths[0] > 0,
                waiting,
                "recovery must replace a blocked wait"
            );
        }
    }
}

/// §6's tail, the mechanic's **only** draw — and the two guards that
/// silence it.
///
/// A head-on pair: each names the other, so step 5's "wait for it"
/// refuses on `on_me` and step 4's sidestep is out because the path
/// top already carries the sidestep flag. What is left is the repath,
/// and it ends in `Random::get(0, 0xffff) % 9 + 1` written into the
/// order's `pause` — the stagger that keeps the two from stepping off
/// on the same frame and colliding again. One-sided, or against a unit
/// already waiting, nothing is rolled and the stream does not move.
#[test]
fn the_head_on_pair_staggers_itself_and_that_is_the_only_draw() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);

    // The three cases share the setup: `x` walking west into `y`,
    // `y` walking east into `x`, and a sidestep already on the top of
    // `x`'s stack.
    let arrange = |named_back: bool, waiting: bool| {
        let (mut sim, x, y) = pair(a, b);
        sim.order_move(x, goal);
        sim.order_move(y, Pos::new(35 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        sim.tick();
        sim.units[x].path.push(PathData {
            to: Pos::new(29 * 0x30 + 0x18, 30 * 0x30 + 0x18),
            tolerance: 0,
            flags: path_flag::SIDESTEP,
        });
        sim.units[x].collide_o = sim.units[y].index;
        sim.units[x].collide_who = 0;
        let (o, who) = if named_back {
            (sim.units[x].index, 0)
        } else {
            (-1, -1)
        };
        sim.units[y].collide_o = o;
        sim.units[y].collide_who = who;
        sim.units[y].waiting_on = waiting;
        sim.trace_phases = true;
        sim.phase_marks.clear();
        (sim, x)
    };
    // The snap teleports the body, so the recovery re-anims and can
    // roll an idle variant of its own; what the site table cares about
    // is whether the *stagger* is one of the frame's draws.
    let staggered = |sim: &Sim| sim.phase_marks.iter().any(|(m, _)| m == SITE_PAUSE);

    let (mut sim, x) = arrange(true, false);
    sim.resolve_unit_collision(x);
    assert!(
        staggered(&sim),
        "the stagger is a draw: {:?}",
        sim.phase_marks
    );
    let pause = sim.current_move(x).expect("still a move").pause;
    assert!((1..=9).contains(&pause), "`% 9 + 1`, got {pause}");

    // It does not name me back: the repath still happens, and it is
    // silent.
    let (mut sim, x) = arrange(false, false);
    sim.resolve_unit_collision(x);
    assert!(
        !staggered(&sim),
        "one-sided, no draw: {:?}",
        sim.phase_marks
    );
    assert_eq!(sim.current_move(x).expect("still a move").pause, 0);

    // It names me back but is already waiting on somebody: also
    // silent, because it is not going to step off either.
    let (mut sim, x) = arrange(true, true);
    sim.resolve_unit_collision(x);
    assert!(!staggered(&sim), "already waiting: {:?}", sim.phase_marks);
    assert_eq!(sim.current_move(x).expect("still a move").pause, 0);
}

/// §6 step 3, **the enemy ladder**, arm B (item 445): a unit walking
/// under an attack whose target is already in reach, bumped by a
/// *different* enemy, drops the walk where it stands. The attack stays
/// and so does the collider's name, and nothing is incremented: no
/// `collide`, no `collide_frame`, no sidestep, no repath. That is
/// golden chapter one's `0/8` on block 625 field for field
/// (`docs/COMBAT.md` §48), and without the arm this crate sidestepped
/// and walked on.
///
/// With the target out of reach the arm does not fire and the unit
/// falls through to steps 4–6, as the third, unmodelled arm leaves it.
#[test]
fn a_bump_from_another_enemy_drops_the_chase_when_the_target_is_in_reach() {
    let here = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let arrange = |target_at: Pos| {
        let (mut sim, x, t) = pair(here, target_at);
        sim.units[t].owner = 1;
        // The collider: a third unit, the target's player's, standing
        // east of `x`.
        let mut c = Unit::new(1, 7, Pos::new(here.x + 0x30, here.y), 40);
        c.ty = sim.units[t].ty;
        let c = sim.add_unit(c);
        sim.add_attack_order(x, Obj::Unit(t), QueuePos::New, false, false);
        sim.add_move_order(
            x,
            Pos::new(35 * 0x30 + 0x18, 20 * 0x30 + 0x18),
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        sim.units[x].collide_o = sim.units[c].index;
        sim.units[x].collide_who = 1;
        (sim, x)
    };

    // In reach: the move goes, the attack and the collider's name stay.
    let (mut sim, x) = arrange(Pos::new(here.x, here.y + 0x30));
    assert_eq!(sim.units[x].orders.len(), 2);
    sim.resolve_unit_collision(x);
    assert_eq!(sim.units[x].orders.len(), 1, "the chase is dropped");
    assert!(matches!(sim.units[x].orders[0].body, Body::Attack(_)));
    assert_eq!(
        (sim.units[x].collide_o, sim.units[x].collide_who),
        (7, 1),
        "arm B keeps the collider's name"
    );
    assert_eq!(
        (sim.units[x].collide, sim.units[x].collide_frame),
        (0, 0),
        "nothing is counted"
    );
    assert_eq!(sim.units[x].orders_pos, sim.units[x].pos);

    // Out of reach: the ladder passes it on to step 5, which counts
    // the collision (the collider holds no move, so no sidestep).
    let (mut sim, x) = arrange(Pos::new(here.x, here.y + 10 * 0x30));
    sim.resolve_unit_collision(x);
    assert_eq!(sim.units[x].orders.len(), 2, "the chase goes on");
    assert_eq!(sim.units[x].collide, 1, "and step 5 counts it");
}

/// Arm A of the same ladder: the collider **is** the target. A plain
/// order forgets the collider and drops the walk; a group order is
/// left alone with `collide = 1` (`005f9d30:162-174`). Reading only:
/// no capture on disk reaches it.
#[test]
fn a_bump_from_the_target_itself_drops_the_chase_and_forgets_the_collider() {
    let here = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let (mut sim, x, t) = pair(here, Pos::new(here.x + 5 * 0x30, here.y));
    sim.units[t].owner = 1;
    sim.add_attack_order(x, Obj::Unit(t), QueuePos::New, false, false);
    sim.add_move_order(
        x,
        Pos::new(35 * 0x30 + 0x18, 20 * 0x30 + 0x18),
        crate::orders::MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    sim.units[x].collide_o = sim.units[t].index;
    sim.units[x].collide_who = 1;
    sim.resolve_unit_collision(x);
    assert_eq!(sim.units[x].orders.len(), 1);
    assert!(matches!(sim.units[x].orders[0].body, Body::Attack(_)));
    assert_eq!((sim.units[x].collide_o, sim.units[x].collide_who), (-1, -1));
}

/// **Arm C of the enemy ladder** (`005f9d30:189-252`, §14): a captain
/// whose attack's target is out of range, bumped on its walk by an
/// enemy that is not its target, drops the walk and the attack where
/// it stands and takes what it can strike from there
/// (`find_new_target(this, NULL, 1)`). run171's `1/6` on 658: chasing
/// the General, bumped by `0/7`, it attacks `0/7` without a step or a
/// count. At ten retargets on its leader's frame it queues the
/// collider itself, unless it is siege; a follower takes its
/// captain's target instead, when it can strike it from where it
/// stands.
///
/// Made to fail on purpose by taking the arm out: the captain keeps
/// its walk and its attack on the far target and counts the collision.
#[test]
fn a_captain_bumped_by_another_enemy_strikes_what_it_can_reach() {
    let tile = 0xc0;
    let here = Pos::new(0x1000 + 24, 0x1000 + 24);
    let setup = |retargets: i32| {
        let mut sim = Sim::new(Tuning::RON, World::new(60, 60), 2);
        sim.world
            .fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        let ty = sim.add_unit_type(UnitType {
            hits: 100,
            moves: 25,
            combat: crate::combat::Profile {
                attack: 15,
                max_range: 4,
                uber_size: 1,
                block_radius: 48,
                big_radius: 48,
                combat_role: true,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        });
        let put = |sim: &mut Sim, who: Player, at: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = Unit::new(who, index, at, 100);
            u.ty = Some(ty);
            u.on_map = true;
            u.kind = sim.tables.unit_types[ty].kind;
            sim.add_unit(u)
        };
        let me = put(&mut sim, 0, here);
        sim.units[me].captain = true;
        let far = put(&mut sim, 1, Pos::new(here.x - 20 * tile, here.y));
        let bump = put(&mut sim, 1, Pos::new(here.x - tile, here.y));
        sim.add_attack_order(me, Obj::Unit(far), QueuePos::New, false, false);
        sim.add_move_order(
            me,
            Pos::new(here.x - 19 * tile, here.y),
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        sim.units[me].collide_o = sim.units[bump].index;
        sim.units[me].collide_who = 1;
        sim.retargets[0] = retargets;
        (sim, me, far, bump)
    };

    let (mut sim, me, _, bump) = setup(9);
    sim.resolve_unit_collision(me);
    assert_eq!(sim.units[me].orders.len(), 1, "the walk is dropped");
    assert!(matches!(sim.units[me].orders[0].body, Body::Attack(_)));
    assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(bump)));
    assert_eq!(sim.units[me].pos, here, "no snap");
    assert_eq!(sim.units[me].collide, 0, "no count");
    assert_eq!(
        sim.units[me].combat.stance,
        crate::combat::Stance::Aggressive,
        "the stance is put back"
    );

    // At ten the search is shut (`cmp [+0x9f4], 0xa; jge`), and a
    // captain that is not siege queues the collider itself in front.
    let (mut sim, me, _, bump) = setup(10);
    sim.resolve_unit_collision(me);
    assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(bump)));
    assert_eq!(sim.units[me].orders.len(), 1);
    assert_eq!(sim.units[me].collide, 0);
    // A siege captain at ten takes §6 as before.
    let (mut sim, me, far, _) = setup(10);
    let ty = sim.units[me].ty.unwrap();
    sim.tables.unit_types[ty].combat.siege = true;
    sim.resolve_unit_collision(me);
    assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(far)));
    assert_eq!(sim.units[me].collide, 1, "§6 step 5's count");

    // A follower takes its captain's target when it can strike it.
    let (mut sim, cap, _, bump) = setup(0);
    sim.clear_orders(cap);
    sim.add_attack_order(cap, Obj::Unit(bump), QueuePos::New, false, false);
    let index = i16::try_from(sim.units.len()).unwrap();
    let mut f = sim.units[cap].clone();
    f.index = index;
    f.captain = false;
    f.orders.clear();
    f.o_up = Some(cap);
    let f = sim.add_unit(f);
    let far = Obj::Unit(sim.units.len() - 2);
    sim.add_attack_order(f, far, QueuePos::New, false, false);
    sim.add_move_order(
        f,
        Pos::new(here.x - 19 * tile, here.y),
        crate::orders::MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    sim.units[f].collide_o = sim.units[bump].index;
    sim.units[f].collide_who = 1;
    // The bump is the follower's collider but not its target, and
    // the captain's target is the bump: arm A does not fire, arm C's
    // follower branch does.
    sim.resolve_unit_collision(f);
    assert_eq!(sim.units[f].combat.target, Some(Obj::Unit(bump)));
    // `repath` takes the walk and `kill_current_order` the attack on
    // the far target, so the captain's is the only order left.
    assert_eq!(sim.units[f].orders.len(), 1);
    assert!(matches!(sim.units[f].orders[0].body, Body::Attack(_)));
}

/// The throttle of §6 step 6 is a **rate**, not a lifetime count:
/// `GameDaemon::process_all` halves every player's `repaths` at the top
/// of each frame and snaps it to zero under three
/// (`docs/PATHFINDER.md` §8).
///
/// Without the decay the counter only ever climbs, and the fifth
/// recovery a player ever makes puts it past 4 for the rest of the
/// game — where `(o + collide) & 3` throws away three collisions in
/// four. That is what run33's frame 571 was: `1/2` reached the repath
/// with `repaths` stuck at 5 and was thrown away, so the stagger draw
/// the original spends was never spent.
#[test]
fn the_repath_throttle_halves_every_frame_and_clears_under_three() {
    let (mut sim, _x, _y) = pair(
        Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18),
        Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18),
    );
    sim.repaths[0] = 12;
    sim.repaths[1] = 5;
    sim.tick();
    assert_eq!(
        (sim.repaths[0], sim.repaths[1]),
        (6, 0),
        "halved, and 2 < 3"
    );
    sim.tick();
    assert_eq!(sim.repaths[0], 3, "6 → 3, which survives");
    sim.tick();
    assert_eq!(sim.repaths[0], 0, "3 → 1, which does not");
}

/// §4.3's **group** arm: two members of one group pass through each
/// other, and two units that merely share an owner do not.
///
/// This is what keeps a marching squad from standing blocked on its
/// own leader every time the formation swings (`docs/ORDERS.md` §8.3);
/// it was a stated seam until this crate had a group — the army's — to
/// ask about.
#[test]
fn two_members_of_one_group_pass_through_each_other() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    assert!(
        sim.detect_unit_collision(x, into).is_some(),
        "two strangers block"
    );
    // The same two in one army — and so in one group.
    let slot = sim.init_army(0, None);
    sim.army_add_unit(0, slot, x);
    sim.army_add_unit(0, slot, y);
    assert!(
        sim.detect_unit_collision(x, into).is_none(),
        "sharing a group makes it a nudge, not a collision"
    );
    // …unless I am attacking, which is the arm's own first test.
    sim.units[x].orders.push_front(crate::orders::Order {
        flags: crate::orders::flag::ACTION,
        body: crate::orders::Body::Attack(crate::orders::AttackOrder {
            defensive: false,
            def: None,
            in_range: false,
            ever_in_range: false,
            new_ord: true,
        }),
    });
    assert!(
        sim.detect_unit_collision(x, into).is_some(),
        "an attacker gets no exemption"
    );
}

/// §4.3's group arm on a **wreck** (item 3824, `docs/GOLDEN.md`
/// §152.10): a dead group-mate whose type blocks while dead is a nudge,
/// and a dead stranger stays the wall §131 made it. The arm
/// (`detect_unit_collision:308`–`316`) reads the candidate object's
/// `+0x80`, `+0x104` and `+0xdc`; no death writes `+0x80`, and
/// `close_orders` empties `+0xdc`. run1476's `2/22` steps onto the dead
/// Main Battle Tank `2/21`'s cell on 761.
///
/// **Made to fail on purpose**: with the wreck's node answering
/// `groups_shared` false, the second assertion collides, and the third
/// melee's word returns from 822 to 761.
#[test]
fn a_dead_group_mate_is_a_nudge_and_a_dead_stranger_a_wall() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let into = b;
    let wreck = |grouped: bool| {
        let (mut sim, x, y) = pair(a, b);
        let ty = sim.units[y].ty.expect("typed");
        sim.tables.unit_types[ty].cols.unit_flags |= crate::ai_load::uflags::BLOCKS_WHILE_DEAD;
        if grouped {
            let slot = sim.init_army(0, None);
            sim.army_add_unit(0, slot, x);
            sim.army_add_unit(0, slot, y);
        }
        sim.cast_sniper_unit(y, 2);
        assert!(!sim.units[y].alive(), "a wreck");
        assert!(sim.units[y].orders.is_empty(), "its orders closed");
        (sim, x, y)
    };
    let (mut sim, x, y) = wreck(false);
    assert_eq!(
        sim.detect_unit_collision(x, into),
        Some(y),
        "a dead stranger whose type blocks while dead is hard"
    );
    let (mut sim, x, _) = wreck(true);
    assert!(
        sim.detect_unit_collision(x, into).is_none(),
        "a dead group-mate is a nudge, read off its own kept group"
    );
    assert!(sim.units[x].half_step, "and the soft one-shot is raised");
}

/// §4.3's group arm again, from the **other seat**: two members of a
/// **pushed** group pass through each other too
/// (`docs/COLLISION.md` §11).
///
/// The original's test is `this->+0x80 == other->+0x80 != -1`
/// (`detect_unit_collision@00617060:307`) — one `short`, the
/// `Groups::list` slot, and it knows nothing about armies. A group
/// here has one of two seats: an army's, or a
/// [`crate::group::Pushed`] slot, and the sibling above only ever
/// exercised the first. [`Sim::same_group_soft`] read
/// [`Sim::army_of`] for a fortnight after item 465 gave this crate a
/// real pool, so every member of a pushed group was **hard** to
/// every other — which on Great Lakes is the whole raid, and was the
/// headline's word at 10277.
///
/// **Made to fail on purpose**: with `group_of` put back to
/// `army_of`, the second assertion below collides, and Great Lakes'
/// word returns from 10294 to 10277 with block 10278's seventeen
/// rows of `1/40` and `1/41` back on it.
#[test]
fn two_members_of_a_pushed_group_pass_through_each_other() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    assert!(
        sim.detect_unit_collision(x, into).is_some(),
        "two strangers block"
    );
    // The same two out of every army and into a pool slot, which is
    // where `do_group_move`'s probe leaves the AI's raiders.
    let mut g = crate::group::Group::stack(0);
    g.list = vec![x, y];
    assert!(sim.push_group(&mut g, false), "a pair takes a slot");
    assert_eq!(g.army, None, "and it is not an army's group");
    assert_eq!(g.pushed, Some(0), "it is the pool's");
    assert!(
        sim.detect_unit_collision(x, into).is_none(),
        "sharing a pushed group makes it a nudge, not a collision"
    );
    assert!(
        sim.units[x].half_step,
        "and a soft collision raises the one-shot the next step is \
             halved by"
    );
}

/// §4.3's one-shot is the **walk's**, not the soft candidate's: it is
/// set only when the nine-cell sweep ends without a hard hit
/// (`orl $0x100000, 0x68(%ebx)` at `006177f3`, reached by falling
/// out of the loop; a hard hit returns 1 before it). A group-mate
/// seen on the way to a stranger leaves no half step, in either
/// chain order.
///
/// Great Lakes' word 11806 was exactly this: `1/37`'s sweep met a
/// soft group-mate and `1/64` hard on frame 11804, and this crate
/// kept the bit, spent it on 11805 and stepped where the original's
/// `1/37` waited (`docs/COLLISION.md` §12). **Made to fail on
/// purpose**: with `name_collider` setting the bit on any soft
/// candidate, the order that lists the group-mate first raises it.
/// §13's distance, from the listing (`5fabf3`–`5fac63`): the longer
/// leg plus the shorter's square over twice the longer, and past
/// 60,000 on the shorter leg their mean with the longer counted twice.
#[test]
fn a_boat_measures_with_the_listing_s_distance() {
    assert_eq!(boat_dist(0, 0), 0);
    assert_eq!(boat_dist(3, -4), 4 + 9 / 8);
    assert_eq!(boat_dist(-100, 0), 100);
    assert_eq!(boat_dist(0, 50), 50);
    assert_eq!(boat_dist(100_000, 70_000), (70_000 + 200_000) / 2);
}

/// Two ships on open water, `push_size` 192 in two circles, the second
/// standing idle across the first one's bow.
fn fleet(a: Pos, b: Pos) -> (Sim, usize, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
    for tx in 0..40 {
        for ty in 0..40 {
            world.set_tile_field(
                Pos::new(tx, ty),
                crate::world::tile::SURFACE,
                crate::world::tile::SURFACE_OCEAN,
            );
        }
    }
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        moves: 42,
        combat: crate::combat::Profile {
            block_radius: 192,
            big_radius: 192,
            uber_size: 1,
            domain: Domain::Sea,
            push_size: 192,
            push_circles: 2,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let make = |sim: &mut Sim, o: i16, at: Pos| {
        let mut u = Unit::new(0, o, at, 40);
        u.ty = Some(ty);
        u.kind.domain = Domain::Sea;
        let i = sim.add_unit(u);
        sim.units[i].movement.facing = crate::movement::Angle::EAST;
        i
    };
    let x = make(&mut sim, 0, a);
    let y = make(&mut sim, 1, b);
    (sim, x, y)
}

/// §13.2: **a ship takes `detect_unit_collision`'s second arm.** A
/// quick probe without `nocoll` answers no collision before any scan,
/// where a land unit on the same two points collides; `nocoll` —
/// `valid_ucoord`'s — skips the arm and scans. Made to fail on
/// purpose by answering `false` from `takes_boat_arm`.
#[test]
fn a_ship_s_quick_probe_never_scans() {
    let a = Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18);
    let b = Pos::new(12 * 0x30 + 0x18, 10 * 0x30 + 0x18);
    let (sim, x, _) = pair(a, b);
    assert!(sim.detect_quick(x, b, false), "a land unit collides");
    let (sim, x, _) = fleet(a, b);
    assert!(sim.takes_boat_arm(x));
    assert!(!sim.detect_quick(x, b, false), "a ship does not scan");
    assert!(sim.detect_quick(x, b, true), "nocoll skips the arm");
}

/// §13.3: `detect_boat_collision` **passes a group-mate and pushes an
/// idle stranger** of its own side, at least 45° off its facing and by
/// at most 48; a foreign ship in the way answers 0 and leaves the step
/// to the land scan. Made to fail on purpose by dropping the group-mate
/// test, which pushes the mate too.
#[test]
fn a_ship_pushes_an_idle_stranger_and_passes_its_group_mate() {
    let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let b = Pos::new(a.x + 150, a.y);
    // The group-mate: nothing moves, and the push is handled.
    let (mut sim, x, y) = fleet(a, b);
    let slot = sim.init_army(0, None);
    sim.army_add_unit(0, slot, x);
    sim.army_add_unit(0, slot, y);
    assert!(sim.units[x].group_ptr.is_some() && sim.units[x].group_ptr == sim.units[y].group_ptr);
    assert!(sim.detect_boat_collision(x, a, true));
    assert_eq!(sim.units[y].pos, b, "a group-mate is not pushed");
    // The stranger: pushed, and stamped.
    let (mut sim, x, y) = fleet(a, b);
    sim.frame = 7;
    assert!(sim.detect_boat_collision(x, a, true));
    let moved = sim.units[y].pos;
    assert_ne!(moved, b, "the stranger is pushed");
    let (dx, dy) = (moved.x - b.x, moved.y - b.y);
    assert!(dx * dx + dy * dy <= 49 * 49, "by at most 48: ({dx}, {dy})");
    assert_eq!(sim.units[y].collide_frame, 7);
    // Due ahead and standing, it goes at least 45° off the bow: south
    // of east here, since the bearing is exactly the facing.
    assert!(moved.y > b.y, "turned off the bow: {moved:?}");
    // A foreign ship is not pushed and hands the step to the scan.
    let (mut sim, x, y) = fleet(a, b);
    sim.units[y].owner = 1;
    assert!(!sim.detect_boat_collision(x, a, true));
    assert_eq!(sim.units[y].pos, b);
}

/// **A ship pushed within four frames pushes back from where it
/// stands** (`Unit::work@0060d180`, `docs/COLLISION.md` §24, item
/// 1452): run621's idle ship `1/16`, pushed by the transport `1/39` on
/// 6140, calls `detect_boat_collision` at its own position with
/// `mates` clear on 6141 and moves the transport. Here: `x` pushes
/// the idle `y` on frame 7; on frame 8, `y`'s own `work` pushes `x`.
/// Four frames on (`frame − 4 < collide_frame` false) it does not.
/// The move-job exemption is read and not tested here: a moving ship's
/// own step pushes through the same function. Made to fail on purpose
/// by removing the block from `work`.
#[test]
fn a_pushed_ship_pushes_back_from_where_it_stands() {
    let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let b = Pos::new(a.x + 150, a.y);
    let push = |frame_back: i64| {
        let (mut sim, x, y) = fleet(a, b);
        sim.frame = 7;
        assert!(sim.detect_boat_collision(x, a, true));
        assert_eq!(sim.units[y].collide_frame, 7);
        sim.frame = 7 + frame_back;
        sim.work(y, 7 + frame_back);
        sim.units[x].pos
    };
    assert_ne!(push(1), a, "pushed back on the next frame");
    assert_ne!(push(3), a, "and within four");
    assert_eq!(push(4), a, "not on the fourth frame after");
}

/// **A pushed ship lights its ring when the push crosses a half-cell**
/// (`docs/VISION.md` §6, item 3569, `docs/AI.md` §310): the push is
/// `set_new_location(pushed, x, y, 0, 0)`, and `005f8d20` throws
/// `update_seen(param_3 == 0)` inside it on a half-cell change. East
/// Indies' Fishermen `1/68`, pushed by `1/73` on 2614, lit the half-cell
/// the Dromon `1/50`'s region scan then refused on 2616. Here the
/// stranger `y` stands one unit short of a half-cell row and is pushed
/// south across it. Made to fail on purpose by dropping the reveal from
/// `SimBoat::set_new_location`.
#[test]
fn a_pushed_ship_lights_its_ring_when_it_crosses_a_half_cell() {
    let row = 3 * 0x180;
    let a = Pos::new(20 * 0x30 + 0x18, row - 1);
    let b = Pos::new(a.x + 150, row - 1);
    let (mut sim, x, y) = fleet(a, b);
    assert!(sim.world.set_fog(vec![0; 40 * 40 * 4]));
    sim.units[y].on_map = true;
    sim.units[y].mylos = 6;
    sim.frame = 7;
    assert!(sim.detect_boat_collision(x, a, true));
    let moved = sim.units[y].pos;
    assert!(moved.y >= row, "pushed across the row: {moved:?}");
    let lit = (0..80)
        .flat_map(|fy| (0..80).map(move |fx| (fx, fy)))
        .filter(|&(fx, fy)| sim.world.seen2(fx, fy).is_some_and(|v| v & 1 != 0))
        .count();
    assert!(lit > 0, "the pushed ship's ring is lit for its owner");
}

/// A land convoy: a supply wagon of player 0 (`uflags2::SUPPLY_OR_HERO`,
/// two push circles of 144) at `a`, and a one-circle land unit of the
/// same player at `b`, standing; `other_flags` is its `unit_flags`.
fn convoy(a: Pos, b: Pos, other_flags: u32) -> (Sim, usize, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let profile = |push_size, push_circles| crate::combat::Profile {
        block_radius: 96,
        big_radius: 96,
        uber_size: 1,
        push_size,
        push_circles,
        ..crate::combat::Profile::default()
    };
    let wagon = sim.add_unit_type(UnitType {
        hits: 90,
        moves: 25,
        combat: profile(288, 2),
        cols: crate::ai_load::UnitCols {
            unit_flags2: crate::ai_load::uflags2::SUPPLY_OR_HERO,
            ..crate::ai_load::UnitCols::default()
        },
        ..UnitType::default()
    });
    let other = sim.add_unit_type(UnitType {
        hits: 65,
        moves: 33,
        combat: profile(72, 1),
        cols: crate::ai_load::UnitCols {
            unit_flags: other_flags,
            ..crate::ai_load::UnitCols::default()
        },
        ..UnitType::default()
    });
    let make = |sim: &mut Sim, o: i16, at: Pos, ty| {
        let mut u = Unit::new(0, o, at, 40);
        u.ty = Some(ty);
        let i = sim.add_unit(u);
        sim.units[i].movement.facing = crate::movement::Angle::EAST;
        i
    };
    let x = make(&mut sim, 7, a, wagon);
    let y = make(&mut sim, 6, b, other);
    (sim, x, y)
}

/// Both search strategies must include a neighbour across a cell-region
/// boundary when its coastal tile shares the query's region, and exclude
/// a dry tile in the query's own cell. Item 1427, COLLISION §13.3.
#[test]
fn push_candidates_use_each_coastal_tile_in_both_search_strategies() {
    for chains in [false, true] {
        let mut world = World::new(12, 12);
        let land = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(11, 11));
        let other_land = world.add_region(Terrain::Land);
        let sea = world.add_region(Terrain::Sea);
        world.set_region(Cell::new(4, 3), other_land);
        for c in [Cell::new(3, 3), Cell::new(4, 3)] {
            let mut d = world.cell_data(c);
            d.flags |= crate::world::cell::HALFLAND;
            d.region2 = Some(sea);
            world.set_cell_data(c, d);
        }
        for t in [Pos::new(15, 13), Pos::new(16, 13)] {
            world.set_tile_field(t, tile::SURFACE, tile::SURFACE_OCEAN);
        }
        let at = Pos::new(3048, 2592);
        let wet = Pos::new(3096, 2592);
        let dry = Pos::new(3048, 2450);
        assert_eq!(world.region_of(at.cell()), Some(land));
        assert_eq!(world.region_of(wet.cell()), Some(other_land));
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let u = sim.add_unit(Unit::new(0, 0, at, 40));
        let same = sim.add_unit(Unit::new(0, 1, wet, 40));
        let different = sim.add_unit(Unit::new(0, 2, dry, 40));
        let threshold = crate::ai_place::circle().radius[1];
        if chains {
            for o in 3..=threshold {
                sim.add_unit(Unit::new(0, o as i16, Pos::new(7000, 7000), 40));
            }
        }
        assert_eq!(
            threshold <= sim.units.len(),
            chains,
            "exercise both search branches"
        );
        let found = sim.find_push_candidates(u, at, 144);
        assert!(
            found.contains(&same),
            "same coastal region across cells: {chains}"
        );
        assert!(
            !found.contains(&different),
            "different tile region in same cell: {chains}"
        );
    }
}

/// **The list branch walks by leader and number and subtracts the
/// candidate's radius** (twenty-fourth pass, group 9; A2 row 11): a unit
/// whose centre is `range + R` away is kept, and the order is `(owner,
/// index)`, not creation order.
#[test]
fn the_list_branch_subtracts_the_candidates_radius_and_walks_by_leader() {
    let at = Pos::new(7000, 7000);
    let mut world = World::new(20, 20);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 19));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let t = sim.add_unit_type(crate::UnitType {
        hits: 40,
        ..crate::UnitType::default()
    });
    sim.tables.unit_types[t].combat.big_radius = 96;
    let put = |sim: &mut Sim, owner: u8, index: i16, p: Pos| {
        let mut un = Unit::new(owner, index, p, 40);
        un.ty = Some(t);
        un.on_map = true;
        sim.add_unit(un)
    };
    let me = put(&mut sim, 0, 0, at);
    let late = put(&mut sim, 1, 3, Pos::new(at.x + 144 + 50, at.y));
    let early = put(&mut sim, 0, 5, Pos::new(at.x + 100, at.y));
    put(&mut sim, 0, 6, Pos::new(at.x + 144 + 97, at.y));
    assert!(crate::ai_place::circle().radius[1] > sim.units.len());
    let found = sim.find_push_candidates(me, at, 144);
    assert_eq!(
        found,
        vec![early, late],
        "within range + 96 is kept, in leader order; `far` is 97 past"
    );
}

/// §13.3's **land half** (item 696): a supply wagon takes
/// `detect_unit_collision`'s second arm as a ship does, and pushes a
/// standing land unit of its own side aside, naming itself in the
/// pushed unit's `collide_o`; a tank it will not shove, and hands the
/// step to the land scan. run190's wagon pushes its guard (21, 1) on
/// its first step. Made to fail on purpose by answering the arm for a
/// ship alone, and by dropping the tank refusal.
#[test]
fn a_supply_wagon_pushes_a_standing_unit_and_not_a_tank() {
    let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let b = Pos::new(a.x + 150, a.y);
    let (mut sim, x, y) = convoy(a, b, 0);
    assert!(sim.takes_boat_arm(x), "a supply wagon takes the arm");
    assert!(!sim.takes_boat_arm(y), "a plain land unit does not");
    sim.frame = 721;
    assert!(sim.detect_boat_collision(x, a, true));
    let moved = sim.units[y].pos;
    assert_ne!(moved, b, "the standing unit is pushed");
    let (dx, dy) = (moved.x - b.x, moved.y - b.y);
    assert!(dx * dx + dy * dy <= 49 * 49, "by at most 48: ({dx}, {dy})");
    assert_eq!(sim.units[y].collide_frame, 721);
    assert_eq!(
        (sim.units[y].collide_o, sim.units[y].collide_who),
        (7, 0),
        "a land pusher names itself on the pushed unit"
    );
    let (mut sim, x, y) = convoy(a, b, crate::ai_load::uflags::TANK);
    assert!(!sim.detect_boat_collision(x, a, true), "a tank is refused");
    assert_eq!(sim.units[y].pos, b);
}

/// §13.3's **gaia arm and the pushed unit's turn** (item 1332): the
/// stranger refusal is `cmpb $8, %dl; jb` (`5fad3c`) — a player's
/// unit only — so a wagon shoves gaia's animal aside where it would
/// refuse a stranger's; and an idle unit it pushes has guy 0 turned
/// toward the push by `turn_angles(bearing, &out, 1, 1)`, the body's
/// rate halved. run511's Supply Wagon `1/86` and peacock `8/2` on
/// block 9348. Made to fail on purpose with the refusal read for gaia,
/// with the turn dropped, and with the rate not halved.
#[test]
fn a_supply_wagon_shoves_gaia_s_animal_and_turns_it() {
    use crate::movement::Angle;
    let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let b = Pos::new(a.x + 150, a.y);
    let run = |facing: Angle| {
        let (mut sim, x, y) = convoy(a, b, 0);
        sim.units[y].owner = 8;
        sim.units[y].movement.facing = facing;
        sim.units[y].movement.heading = facing;
        sim.units[y].movement.turning.instant_from_stop = true;
        sim.frame = 9347;
        assert!(sim.detect_boat_collision(x, a, true));
        let pushed = &sim.units[y];
        assert_ne!(pushed.pos, b, "gaia's animal is pushed");
        assert_eq!(
            (pushed.collide_o, pushed.collide_who, pushed.collide_frame),
            (7, 0, 9347),
            "and names its pusher"
        );
        // Standing still, it is pushed 45° off the wagon's facing.
        assert_eq!(pushed.movement.heading, Angle(0x6000_0000));
        pushed.movement.facing
    };
    // Owing 45°, inside the halved instant rate: it snaps.
    assert_eq!(run(Angle::EAST), Angle(0x6000_0000));
    // Owing 135°, past it: a quarter turn, the short way round.
    assert_eq!(run(Angle::WEST), Angle::SOUTH);
}

/// §4.3's **escort row** (item 696): a collider whose *action* is a
/// `GUARD` on me is soft, whatever transit leg is at its head; the same
/// guard on another unit is not. The original's scan on run190's tick
/// 726 reaches `is_here` on the walking guard and never `is_corner`.
/// Made to fail on purpose by deleting the row.
#[test]
fn an_escort_never_blocks_its_charge() {
    let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
    let b = Pos::new(a.x + 150, a.y);
    let (mut sim, wagon, guard) = convoy(a, b, 0);
    sim.add_guard_order(guard, wagon, 0, 372, QueuePos::New);
    sim.add_move_order(
        guard,
        Pos::new(b.x, b.y + 480),
        crate::orders::MoveKind::AttackTo,
        QueuePos::First,
        false,
    );
    assert!(sim.soft_collision(wagon, guard, 0), "an escort is soft");
    let (mut sim, wagon, guard) = convoy(a, b, 0);
    let stranger = sim.add_unit(Unit::new(0, 8, Pos::new(a.x, a.y + 960), 40));
    sim.add_guard_order(guard, stranger, 0, 372, QueuePos::New);
    assert!(
        !sim.soft_collision(wagon, guard, 0),
        "another's escort is not"
    );
}

/// §4.3's **caravan row** (item 1127, `docs/COLLISION.md` §19): two
/// units whose actions are both `TRADE_ROUTE`, each with a move at its
/// head, are soft to each other, so the step goes through and the
/// walk sets the one-shot half step. Either without the move, or
/// either without the trade, is hard. East Indies' 6151 is the frame:
/// `1/15` steps into `1/33`, paused at its leg's end with its next leg
/// already at its head, and the original prints `collide_o -1` and the
/// half-step bit on block 6152. Made to fail on purpose by deleting the
/// row.
#[test]
fn two_caravans_on_their_legs_pass_through_each_other() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let trade = |sim: &mut Sim, u: usize| {
        let order = Order {
            flags: 0,
            body: Body::Trade(crate::orders::TradeOrder {
                home: 0,
                dest: None,
                started: true,
                loaded: false,
            }),
        };
        sim.enqueue_order(u, order, QueuePos::New);
    };
    let leg = |sim: &mut Sim, u: usize, to: Pos| {
        sim.add_move_order(
            u,
            to,
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
    };
    let set = |trades: [bool; 2], legs: [bool; 2]| {
        let (mut sim, x, y) = pair(a, b);
        for (u, t, l, to) in [(x, trades[0], legs[0], b), (y, trades[1], legs[1], a)] {
            if t {
                trade(&mut sim, u);
            }
            if l {
                leg(&mut sim, u, to);
            }
        }
        (sim, x, y)
    };
    let (mut sim, x, y) = set([true, true], [true, true]);
    assert!(sim.soft_collision(x, y, 0), "a caravan pair is soft");
    assert_eq!(sim.detect_unit_collision(x, b), None, "and steps through");
    assert!(sim.units[x].half_step, "paying the one-shot half step");
    assert_eq!(sim.units[x].collide_o, -1);
    for (trades, legs, what) in [
        ([true, false], [true, true], "a stranger on a leg"),
        (
            [true, true],
            [true, false],
            "a caravan standing on its route",
        ),
        (
            [true, true],
            [false, true],
            "a caravan with no leg of its own",
        ),
    ] {
        let (mut sim, x, y) = set(trades, legs);
        assert!(!sim.soft_collision(x, y, 0), "{what} is hard");
        assert_eq!(sim.detect_unit_collision(x, b), Some(y), "{what}");
    }
}

/// §4.3's **attackers' row** asks the action's `TargetOrder` whether
/// its target still stands (`TargetOrder::target_exists@0072ff10`)
/// before it measures the overreach (`detect_unit_collision@00617060:
/// 103-119`): two attackers of one player marching on a target far
/// beyond their range are soft to each other, and once the target is
/// dead they are hard. The battery word 12836 (`docs/AI.md` §245),
/// where the Scout `0/0` dies on 12835 and four of who=1's marchers
/// collide hard on 12836. Made to fail on purpose by dropping the
/// `obj_alive` gate in `attack_overreach`.
#[test]
fn attackers_on_a_dead_target_are_hard_to_each_other() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    let mut e = sim.units[y].clone();
    e.owner = 1;
    e.index = 0;
    e.pos = Pos::new(4 * 0x30 + 0x18, 4 * 0x30 + 0x18);
    let t = sim.add_unit(e);
    for (u, to) in [(x, b), (y, a)] {
        let order = Order {
            flags: 0,
            body: Body::Attack(crate::orders::AttackOrder {
                defensive: false,
                def: None,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        };
        sim.enqueue_order(u, order, QueuePos::New);
        sim.add_move_order(
            u,
            to,
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        sim.units[u].combat.target = Some(crate::combat::Obj::Unit(t));
    }
    assert!(sim.attack_overreach(x, b) > 0x300, "the target is far");
    assert_eq!(sim.detect_unit_collision(x, b), None, "so the pair is soft");
    assert!(sim.units[x].half_step);
    let (mut sim2, x2) = (sim.clone(), x);
    sim2.units[x2].half_step = false;
    sim2.units[t].health = 0;
    assert_eq!(sim2.attack_overreach(x2, b), 0, "a dead target is no reach");
    assert_eq!(
        sim2.detect_unit_collision(x2, b),
        Some(y),
        "and the pair is hard"
    );
    assert!(!sim2.units[x2].half_step);
}

#[test]
fn a_hard_hit_leaves_no_half_step_whatever_soft_it_passed() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    for mate_first in [false, true] {
        let (mut sim, x, y) = pair(a, b);
        // A third unit on the same point, of the same type and owner
        // and in no group: a stranger, and so hard.
        let mut v = sim.units[y].clone();
        v.index = 2;
        let z = sim.add_unit(v);
        let slot = sim.init_army(0, None);
        sim.army_add_unit(0, slot, x);
        sim.army_add_unit(0, slot, y);
        // The chain's order is the insertion's; re-link the mate to
        // put it on the other side of the stranger.
        if mate_first {
            sim.chain_remove(y);
            sim.chain_add(y);
        }
        let hard = sim.detect_unit_collision(x, b);
        assert_eq!(
            hard,
            Some(z),
            "the stranger is hard (mate_first {mate_first})"
        );
        assert!(
            !sim.units[x].half_step,
            "a hard hit leaves no one-shot (mate_first {mate_first})"
        );
    }
    // And the soft mate alone still raises it.
    let (mut sim, x, y) = pair(a, b);
    let slot = sim.init_army(0, None);
    sim.army_add_unit(0, slot, x);
    sim.army_add_unit(0, slot, y);
    assert_eq!(sim.detect_unit_collision(x, b), None);
    assert!(sim.units[x].half_step, "the soft walk raises it");
}

/// §4.3: **the corner rule is decided on the blocker's *figure 0*, not
/// on the blocker** — and not on its crew (item 1524).
/// `UnitData::is_corner@0060a040` walks
/// `0 .. guy_mark` (1) and returns the first non-zero
/// `GuyData::is_corner@005de270`, which measures against that figure's
/// own `GuyData::x/y`. The test immediately before it —
/// `UnitData::is_here` — reads the **unit's** `x_internal`, so the
/// original genuinely mixes the two, and a crew figure standing on a
/// track offset can turn a hard collision into a slip-past its leader
/// alone never would.
///
/// The blocker stands on unit cell `(28, 28)`, whose own corner
/// against the hit cell `(28, 29)` is **0** — a face, not a diagonal —
/// so the unit-centred reading is a hard collision. Its crew figure
/// stands a cell west, on `(27, 28)`, for which the same hit cell is
/// the **SE** corner: `|NW − SE| = |1 − 5| = 4`, the two opposite
/// diagonals, and the two slip past.
#[test]
fn the_corner_rule_reads_the_blocker_s_figures_and_not_the_blocker() {
    let at_cell = |x: i32, y: i32| ucell_centre(Pos::new(x, y));
    let (mut sim, x, y) = pair(at_cell(30, 30), at_cell(28, 28));
    // Guy 0 stands on its unit, which is what leaves the corner at 0.
    sim.units[y].guys = vec![crate::anim::Guy::fresh(1)];
    sim.units[x].guys = vec![crate::anim::Guy::fresh(1)];
    sim.units[y].movement.body.pos = sim.units[y].pos;
    let into = at_cell(29, 30);
    assert_eq!(
        sim.detect_unit_collision(x, into),
        Some(y),
        "guy 0 alone: the hit cell is a face of the blocker, so it is hard"
    );
    // The crew figure, a cell west of its leader and cornered on the
    // same hit cell from the opposite side.
    let mut crew = crate::anim::Guy::fresh(2);
    crew.follow = Some(crate::anim::Follow {
        body: crate::movement::Body {
            pos: at_cell(27, 28),
            ..crate::movement::Body::default()
        },
        des: at_cell(27, 28),
        facing: crate::movement::Angle(0),
        des_angle: crate::movement::Angle(0),
        track: (-48, 0),
    });
    sim.units[y].guys.push(crew);
    assert_eq!(
        sim.detect_unit_collision(x, into),
        Some(y),
        "the crew figure is past `guy_mark` (1): `is_corner` never asks it, and the \
             collision stays hard (item 1524)"
    );
}

/// §18: `do_move`'s blocker probe is the **quick** form. East Indies'
/// `1/71` on 19498 stands a cell east of the point it was refused, and
/// `1/75` has walked on until only its north-east corner touches the
/// proposal's south-west. The full form lets the two slip past, names
/// nobody and clears `collide_o`. The original's quick form stops at
/// the occupied cell, and the blocker is still there.
#[test]
fn the_blocker_probe_counts_a_corner_the_step_would_slip_past() {
    let at_cell = |x: i32, y: i32| ucell_centre(Pos::new(x, y));
    let (mut sim, x, y) = pair(at_cell(30, 30), at_cell(27, 32));
    let coll = at_cell(29, 30);
    sim.units[x].collide_o = sim.units[y].index;
    assert!(
        sim.blocker_still_there(x, coll),
        "the quick form: an occupied cell is the blocker, corner or not"
    );
    assert_eq!(
        sim.units[x].collide_o, sim.units[y].index,
        "and it writes nothing"
    );
    assert_eq!(
        sim.detect_unit_collision(x, coll),
        None,
        "the full form lets the corners pass, as a step would"
    );
    assert_eq!(sim.units[x].collide_o, -1, "and clears the blocker");
}

/// §4.1: `safe` — the cooldown a failed 48-grid search buys — turns the
/// test off entirely while it runs, and `Unit::work` counts it down.
#[test]
fn the_safe_cooldown_suppresses_the_test_and_expires() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, _y) = pair(a, b);
    let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    assert!(
        sim.detect_quick(x, into, false),
        "blocked with `safe` clear"
    );
    sim.units[x].safe = 3;
    assert!(!sim.detect_quick(x, into, false), "and clear with it set");
    sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    for _ in 0..3 {
        sim.tick();
    }
    assert_eq!(sim.units[x].safe, 0, "one a frame, in `Unit::work`");
}

/// §5: **the blocked stand.** A refused step asks for `CHAR_DEFAULT`
/// before any of the three give-up tests, so a walker re-rolls its
/// idle on the frame it is blocked — the draw run14 spends on frames
/// 122, 184 and 256, [`crate::anim::SITE_BLOCKED`].
#[test]
fn a_refused_step_re_rolls_the_idle_before_the_give_up_tests() {
    let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, _y) = pair(a, b);
    // A guy with a piece whose idle has a length, so the roll has
    // somewhere to land.
    for slot in 0..4 {
        sim.art.lengths.insert((1, slot), 40);
    }
    sim.units[x].guys = vec![crate::anim::Guy::fresh(1)];
    sim.trace_phases = true;
    sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    let mut marks = None;
    for _ in 0..40 {
        sim.tick();
        if sim.units[x].collide_o >= 0 {
            marks = Some(sim.phase_marks.clone());
            break;
        }
    }
    let marks = marks.expect("a frame on which the step is refused");
    let at = marks
        .iter()
        .position(|(l, _)| l == crate::anim::SITE_BLOCKED)
        .expect("the blocked stand is marked on that frame");
    // And it *drew*: the word moves between this mark and the next.
    let after = marks.get(at + 1).map_or(sim.rng.seed, |(_, w)| *w);
    assert_ne!(marks[at].1, after, "the blocked stand spends a draw");
}

/// §17: **the give-up does not step.** A unit blocked for 26 frames
/// (`collide >= 0x1a`) on a final waypoint within three reaches widens
/// `tolerance` to twice the Manhattan distance still owed and jumps
/// straight to the step's arrival tail (`005fb7b5`–`005fb7bb`,
/// `jmp 005fb82c`), past `set_new_location`. The test, from where it
/// stands, then takes the waypoint: run218's `1/23` on Great Lakes
/// 16459, which this crate walked a step into The Despot.
#[test]
fn a_blocked_unit_that_gives_up_takes_its_waypoint_where_it_stands() {
    let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let a = Pos::new(b.x + 0x60, b.y);
    let (mut sim, x, y) = pair(a, b);
    sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
    sim.tick();
    let here = sim.units[x].pos;
    // One final leg, past the unit in front and inside three reaches
    // (`(48 + 48) * 3 = 288`), with a tolerance of its own.
    let to = Pos::new(b.x - 0x30, b.y);
    sim.units[x].path.clear();
    sim.units[x].path.push(PathData {
        to,
        tolerance: 0x30,
        flags: path_flag::FINAL,
    });
    sim.units[x].line_ok = true;
    if let Some(front) = sim.units[x].orders.front_mut()
        && let Some(m) = front.move_mut()
    {
        m.has_waypoint = true;
        m.waypoint = to;
    }
    sim.units[x].collide = 0x1a;
    sim.units[x].collide_frame = sim.frame;
    let manh = (to.x - here.x).abs() + (to.y - here.y).abs();
    assert!(manh < 3 * 96, "inside three reaches: {manh}");
    sim.tick();
    assert_eq!(
        (sim.units[x].collide_o, sim.units[x].collide_who),
        (
            sim.units[y].index,
            i8::try_from(sim.units[y].owner).unwrap()
        ),
        "the step was refused by the unit in front"
    );
    assert_eq!(sim.units[x].tolerance, manh * 2, "the widened tolerance");
    assert_eq!(sim.units[x].pos, here, "and no step was taken");
    assert!(
        sim.units[x].path.is_empty() && sim.units[x].orders.is_empty(),
        "the arrival test took the final leg where it stands: {:?} {:?}",
        sim.units[x].path,
        sim.units[x].orders
    );
}

/// §6 step 2's fence, and what it is: the target type's `+0x94` is
/// `BuildTypeData::is_flat` — `build_flags & 0x10000000`, the derived
/// bit only the Farm, the Oil Well and the Oil Platform carry
/// (`crate::build::init_final_flags`). So the step that abandons a walk
/// because the unit is standing inside its own gather target's
/// footprint fires for a **farmer on its field** and for nothing else.
/// A woodcutter bumped while standing on its camp's footprint — which
/// is run10's `1/6` on frame 206 — falls through to step 6 instead, and
/// this crate read the virtual as `true` and killed its order for ever
/// until item 64.
#[test]
fn only_a_flat_gather_target_abandons_the_walk_where_the_unit_stands() {
    for flat in [true, false] {
        // Near the west edge of its cell, so one step west lands in
        // the next cell along — the same-cell gate (§4.1 step 6) is
        // what a step that stays put would fall out of.
        let a = Pos::new(29 * 0x30 + 4, 30 * 0x30 + 0x14);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        let bt = sim.add_build_type(crate::build::BuildType {
            ident: if flat {
                crate::build::Ident::Farm
            } else {
                crate::build::Ident::Woodcutter
            },
            x_size: 5,
            y_size: 5,
            flags: if flat { crate::build::flags::FLAT } else { 0 },
            ..crate::build::BuildType::default()
        });
        let camp = sim.add_building(0, a, 0);
        sim.buildings[camp].ty = Some(bt);
        assert!(
            sim.covers_tile(camp, sim.units[x].pos.tile()),
            "the walker stands inside the footprint either way"
        );
        assert_eq!(sim.is_flat(camp), flat, "and only one of them is flat");

        // A transit move under the gather, as `do_gather` queues one:
        // the `ACTION` flag belongs to the order beneath, which is the
        // one `action_of` has to find for step 2 to look at all.
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        sim.add_move_order(
            x,
            goal,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        sim.units[x].orders.push_back(Order {
            flags: crate::orders::flag::ACTION,
            body: Body::Gather(crate::orders::GatherOrder {
                building: camp,
                tile: None,
                wait: 0,
                goto_build: true,
                dist_mod: 0,
                been_there: false,
            }),
        });
        sim.units[x].path.push(PathData {
            to: goal,
            tolerance: 0,
            flags: path_flag::FINAL,
        });

        // The step that is refused: one speed west, into the block of
        // the unit standing still.
        let proposed = Pos::new(sim.units[x].pos.x - 25, sim.units[x].pos.y);
        assert_eq!(
            sim.detect_unit_collision(x, proposed),
            Some(y),
            "the unit in front is named"
        );
        sim.resolve_unit_collision(x);

        let moving = sim
            .current_order(x)
            .is_some_and(|o| matches!(o.body, Body::Move(_)));
        if flat {
            assert!(
                !moving,
                "the farmer on its own field abandons the walk: {:?}",
                sim.units[x].orders
            );
        } else {
            assert!(
                moving,
                "the woodcutter on its camp keeps the walk: {:?}",
                sim.units[x].orders
            );
            assert_eq!(
                sim.units[x].pos,
                ucell_centre(ucell(a)),
                "and takes step 6's snap onto its own cell centre"
            );
            assert!(
                sim.units[x].path.len() > 1,
                "and step 6's plan on the 48-grid: {:?}",
                sim.units[x].path
            );
        }
    }
}

/// §5.1's second arm: a **parked** unit on the waypoint widens the
/// tolerance to three of its `big_radius` instead of killing the move,
/// so the walker gives up short of it rather than colliding. The kill
/// arm needs a `GATHER`/`ATTACK`/`BUILD_AT` action under the move and
/// is tested end to end in `cities_tests`; this is the other branch,
/// which no capture has entered.
#[test]
fn a_parked_unit_on_the_waypoint_widens_the_tolerance() {
    let a = Pos::new(34 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(24 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    // `big_radius` is `ObjectType +0x244` and `block_radius` `+0x240`;
    // they are the same number for every unit in every capture, so
    // give them different ones here and the test says which is read.
    sim.tables.unit_types[0].combat.big_radius = 96;
    assert_eq!(sim.coll_size(y), 1, "still a one-cell block");
    // Straight onto the standing unit, ten cells away.
    sim.order_move(x, b);
    sim.tick();
    assert_eq!(
        sim.units[x].tolerance,
        3 * 96,
        "three of the parked unit's `big_radius`"
    );
    assert_eq!(
        sim.units[x].path.last().map(|p| p.tolerance),
        Some(3 * 96),
        "and the path top carries it"
    );
    for _ in 0..40 {
        sim.tick();
    }
    assert!(
        sim.units[x].orders.is_empty(),
        "the walk ended: {:?}",
        sim.units[x].orders
    );
    let gap = (sim.units[x].pos.x - sim.units[y].pos.x).abs();
    assert!(
        (0x30..=3 * 96).contains(&gap),
        "it stopped short of the parked unit rather than on it: {gap}"
    );
}

/// §5.1's kill arm under a **`TRADE_ROUTE`**: the same walk onto the
/// same parked unit, with a trade order beneath the move, is not
/// widened but killed where the caravan stands — `5f8721`'s `cmpl
/// $0xf` is the first of the four actions the listing tests, ahead of
/// `GATHER`, `ATTACK` and `BUILD_AT`. Great Sahara at Toughest's
/// caravan `1/52` stopped so on frame 10390, a builder on its last
/// node, and its `do_trade` rolled the next frame (item 1371). The
/// same walk under a move alone widens (the test above).
#[test]
fn a_trade_walk_onto_a_parked_unit_is_killed_where_it_stands() {
    let a = Pos::new(34 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let b = Pos::new(24 * 0x30 + 0x18, 30 * 0x30 + 0x18);
    let (mut sim, x, y) = pair(a, b);
    sim.enqueue_order(
        x,
        Order {
            flags: 0,
            body: Body::Trade(crate::orders::TradeOrder {
                home: 0,
                dest: Some(1),
                started: true,
                loaded: false,
            }),
        },
        QueuePos::New,
    );
    sim.add_move_order(
        x,
        b,
        crate::orders::MoveKind::MoveTo,
        QueuePos::First,
        false,
    );
    assert!(matches!(sim.units[x].orders[0].body, Body::Move(_)));
    assert_eq!(sim.action_of(x), Some(1), "the trade is the action");
    sim.tick();
    assert_eq!(sim.units[x].pos, a, "not one step of the walk was taken");
    assert_eq!(
        sim.units[x].orders.len(),
        1,
        "the move is gone: {:?}",
        sim.units[x].orders
    );
    assert!(matches!(sim.units[x].orders[0].body, Body::Trade(_)));
    assert_eq!(sim.units[x].collide_o, sim.units[y].index);
    assert_ne!(
        sim.units[x].tolerance,
        3 * 48,
        "and the tolerance was never widened"
    );
}

/// §2.1: the generated spiral is the Chebyshev disc, ring by ring, and
/// its first nine entries are the compass the pathfinder uses.
#[test]
fn the_spiral_is_the_chebyshev_disc_and_ring_two_is_the_odd_one() {
    let s = spiral(7);
    assert_eq!(s.len(), 15 * 15, "radius[7] = (2·7+1)²");
    for r in 0..=7usize {
        let n = (2 * r + 1) * (2 * r + 1);
        let got: std::collections::BTreeSet<_> = s[..n].iter().copied().collect();
        let want: std::collections::BTreeSet<_> = (-(r as i32)..=r as i32)
            .flat_map(|x| (-(r as i32)..=r as i32).map(move |y| (x, y)))
            .collect();
        assert_eq!(got, want, "radius[{r}] is the disc");
    }
    assert_eq!(
        &s[..9],
        &[
            (0, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0)
        ],
        "the compass, as `docs/PATHFINDER.md` §4.2 has it"
    );
    assert_eq!(
        &s[21..25],
        &[(-2, -2), (2, -2), (2, 2), (-2, 2)],
        "ring 2 keeps its corners for last"
    );
}

/// §4.2's fast path, and **the reason it is not an optimisation**: it
/// sweeps the leading edge alone, so it stops at a different first hit
/// cell than the disc — and §4.3's corner rule is decided on the cell,
/// so the two probes disagree about whether there is a collision at
/// all. run66's sim-frame 6570 is the shape, rebuilt here at the
/// origin: a `coll_size 2` walker stepping one cell north between two
/// `coll_size 1` neighbours, one of them corner to corner with it and
/// the other square on its leading edge.
///
/// Written to fail first: with the whole disc the walker names the
/// square neighbour and refuses the step, which is exactly what East
/// Indies' word parted on.
/// **The pathfinder's copy of a block outlives the probe that took
/// it** (`CollCheck::fill_slots@006820e0`, `docs/PATHFINDER.md` §26).
/// A `nocoll` probe reads a world cell's copy when the tree holds one
/// and copies what it read live when it does not; only `kill_lists`
/// empties the tree. So once a probe has copied a neighbour's block, a
/// later `nocoll` probe still finds the neighbour where it stood,
/// while a probe of the live blocks finds it gone.
///
/// Written to fail first: with the live blocks alone the second
/// `nocoll` probe finds nothing, which is what Great Lakes 12625's
/// resumed search for `1/41` read on every cell it first probed.
#[test]
fn a_nocoll_probe_reads_the_block_the_last_one_copied() {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let t = sim.add_unit_type(UnitType {
        hits: 40,
        moves: 25,
        combat: crate::combat::Profile {
            block_radius: 48,
            big_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let put = |sim: &mut Sim, o: i16, cell: Pos| {
        let mut u = Unit::new(0, o, ucell_centre(cell), 40);
        u.ty = Some(t);
        sim.add_unit(u)
    };
    let prober = put(&mut sim, 0, Pos::new(20, 20));
    let other = put(&mut sim, 1, Pos::new(24, 20));
    let at = Pos::new(23, 20);
    assert!(
        sim.collide_here(prober, at, true).is_some(),
        "the neighbour is there, and the probe copies its world cell"
    );
    assert!(sim.set_new_location(other, ucell_centre(Pos::new(30, 30)), true));
    assert_eq!(
        sim.collide_here(prober, at, false),
        None,
        "the live blocks have it gone"
    );
    assert!(
        sim.collide_here(prober, at, true).is_some(),
        "the copy still has it where it stood"
    );
    sim.kill_lists();
    assert_eq!(
        sim.collide_here(prober, at, true),
        None,
        "`kill_lists` empties the tree, and the next probe reads live"
    );
}

#[test]
fn the_leading_edge_finds_the_corner_the_disc_walks_past() {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = |sim: &mut Sim, block: i32| {
        sim.add_unit_type(UnitType {
            hits: 40,
            moves: 25,
            combat: crate::combat::Profile {
                block_radius: block,
                big_radius: block,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        })
    };
    let (wide, narrow) = (ty(&mut sim, 96), ty(&mut sim, 48));
    let put = |sim: &mut Sim, o: i16, cell: Pos, t| {
        let mut u = Unit::new(0, o, ucell_centre(cell), 40);
        u.ty = Some(t);
        sim.add_unit(u)
    };
    // The walker's own cell is `(20, 20)` and it proposes `(20, 19)`,
    // so its leading edge is the row `y = 17` at `x` 18, 20 and 22.
    let big = put(&mut sim, 0, Pos::new(20, 20), wide);
    // `(16..=18, 15..=17)` — the corner cell `(18, 17)` and no more.
    let corner = put(&mut sim, 1, Pos::new(17, 16), narrow);
    // `(20..=22, 15..=17)` — the edge's middle cell, and the one the
    // disc reaches first.
    let square = put(&mut sim, 2, Pos::new(21, 16), narrow);
    assert_eq!((sim.coll_size(big), sim.coll_size(corner)), (2, 1));
    assert_eq!(sim.coll_size(square), 1);

    let at = ucell_centre(Pos::new(20, 19));
    assert_eq!(
        sim.collide_here(big, ucell(at), false),
        Some(Pos::new(18, 17)),
        "the leading edge stops at the corner cell"
    );
    // The disc — which `valid_ucoord` and the stack unwind still ask —
    // takes the ring's edge cells before its corners.
    assert_eq!(
        sim.collide_here(big, ucell(at), true),
        Some(Pos::new(20, 17)),
        "the whole disc's first parity cell is the middle of the edge"
    );
    // And the corner rule turns the first into no collision at all:
    // `will_be_corner 1` against `is_corner 5` is the two opposite
    // diagonals, and they pass.
    assert_eq!(
        sim.detect_unit_collision(big, at),
        None,
        "corner to opposite corner: the two slip past"
    );
    assert_eq!(sim.units[big].collide_o, -1, "and nothing is recorded");
}

/// §4.2's fast path **does not advance past an empty slot** (item 539,
/// `00682540:71`-`116`). An even pass tests its cell and advances the
/// swept axis on a miss only when the cell's world cell holds a bit;
/// a cell in an empty world cell is skipped where it stands, so the
/// next cell tested is one step on, not two. Great Lakes' sim-frame
/// 11304 is the shape, rebuilt at the origin: a `coll_size 1` walker
/// stepping west from `(17, 16)` onto `(16, 16)` sweeps the column
/// `x = 15` from `y = 15`, which is the last row of world cell
/// `(0, 0)`, and a group-mate's block starts at `y = 17`, which is
/// world cell `(0, 1)`.
///
/// Written to fail first: with the fixed two-step stride the sweep
/// tests `(15, 17)`, finds the neighbour, and the walker takes a half
/// step that the original does not take.
#[test]
fn the_leading_edge_does_not_step_past_an_empty_world_cell() {
    let mut world = World::new(4, 4);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 3));
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let ty = sim.add_unit_type(UnitType {
        hits: 40,
        moves: 25,
        combat: crate::combat::Profile {
            block_radius: 48,
            big_radius: 48,
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..UnitType::default()
    });
    let put = |sim: &mut Sim, o: i16, cell: Pos| {
        let mut u = Unit::new(0, o, ucell_centre(cell), 40);
        u.ty = Some(ty);
        sim.add_unit(u)
    };
    let walker = put(&mut sim, 0, Pos::new(17, 16));
    // `(14..=16, 17..=19)`: world cells `(0, 1)` and `(1, 1)`.
    put(&mut sim, 1, Pos::new(15, 18));
    let at = Pos::new(16, 16);
    assert!(!sim.coll.any_in_cell(0, 0), "world cell (0, 0) is empty");
    assert_eq!(
        sim.collide_here(walker, at, false),
        None,
        "(15, 15)'s slot is empty, so the sweep tests (15, 16) and misses"
    );
    // The control: one bit anywhere in world cell `(0, 0)` makes the
    // slot live, and the stride is two again — `(15, 15)`, `(15, 17)`.
    put(&mut sim, 2, Pos::new(3, 3));
    assert!(sim.coll.any_in_cell(0, 0));
    assert_eq!(
        sim.collide_here(walker, at, false),
        Some(Pos::new(15, 17)),
        "a live slot advances past its miss"
    );
}

/// §4.2: for a `coll_size 1` unit the parity filter leaves exactly the
/// four diagonals, which is what makes two 3×3 blocks detect each
/// other at Chebyshev 2.
#[test]
fn the_parity_filter_leaves_the_four_diagonals() {
    let probed: Vec<_> = spiral(1)
        .into_iter()
        .filter(|(dx, dy)| (dx + 1) % 2 == 0 && (dy + 1) % 2 == 0)
        .collect();
    assert_eq!(probed, vec![(-1, -1), (1, -1), (1, 1), (-1, 1)]);
}

/// §4.3's corner rule, on the pair run10's frame 122 actually has:
/// `1/6` proposing cell `(872, 355)` with the hit at `(871, 354)`,
/// against `1/3` sitting on `(870, 354)`. `1` against `0` is not a
/// difference of four, so it is a hard collision.
#[test]
fn run10_s_frame_122_pair_is_a_hard_collision() {
    let hit = Pos::new(871, 354);
    let will = Sim::corner_of(1, Pos::new(872, 355), hit);
    let theirs = Sim::corner_of(1, Pos::new(870, 354), hit);
    assert_eq!((will, theirs), (1, 0));
    assert!(will != 0 && (will - theirs).abs() != 4, "hard");
    // The opposite-diagonal pair that does slip past.
    assert_eq!(Sim::corner_of(1, Pos::new(870, 354), Pos::new(871, 355)), 5);
    assert_eq!(Sim::corner_of(1, Pos::new(872, 356), Pos::new(871, 355)), 1);
}
