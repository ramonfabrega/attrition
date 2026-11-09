use super::*;
use crate::orders::{MoveKind, Worker};
use crate::world::{Cell, Terrain};
use crate::{Tuning, Unit, World};

/// `find_wpath`'s army mode takes an unarmed unit whose `is_supply`
/// answers yes, `unit_flags2 & 0x40` (`docs/PATHFINDER.md` §25).
/// Golden chapter four's Supply Wagon is the first in the corpus to
/// plan a world path, and without the arm it cut through the `0x200`
/// cells an army pays thirty-two times the base for.
#[test]
fn a_supply_wagon_plans_as_an_army_and_an_unarmed_plain_unit_does_not() {
    let mut sim = flat_sim(8);
    let wagon = sim.add_unit_type(crate::UnitType {
        cols: crate::ai_load::UnitCols {
            unit_flags2: crate::ai_load::uflags2::SUPPLY_OR_HERO,
            ..crate::ai_load::UnitCols::default()
        },
        ..crate::UnitType::default()
    });
    let plain = sim.add_unit_type(crate::UnitType::default());
    let u = walker(&mut sim, Pos::new(0x480, 0x480));
    sim.units[u].ty = Some(wagon);
    assert!(sim.army_mode(u), "attack 0, and still an army: is_supply");
    sim.units[u].ty = Some(plain);
    assert!(!sim.army_mode(u), "attack 0 and not supply: no army mode");
}

fn flat_sim(cells: i32) -> Sim {
    let mut world = World::new(cells, cells);
    world.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(cells - 1, cells - 1),
    );
    Sim::new(Tuning::RON, world, 2)
}

fn walker(sim: &mut Sim, pos: Pos) -> usize {
    let mut u = Unit::new(0, 1, pos, 10);
    u.movement.speed = 25;
    let i = sim.units.len();
    sim.units.push(u);
    i
}

/// `astar_path@00683770:119-127`'s `no_danger`: an attack action, a
/// gaia owner, **or a current order of `ATTACK_TO`/`GROUP_ATTACK_TO`**
/// (`docs/PATHFINDER.md` §28). East Indies' army column on tick 17146
/// walks under an `ATTACK_TO` and the original prices its world steps
/// without who=1's danger; ours read it, and `1/58` cut north.
#[test]
fn an_attack_to_order_plans_without_the_danger_map() {
    for (kind, action, want) in [
        (MoveKind::AttackTo, true, true),
        (MoveKind::AttackTo, false, true),
        (MoveKind::MoveTo, true, false),
        (MoveKind::MoveTo, false, false),
    ] {
        let mut sim = flat_sim(12);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        sim.add_move_order(
            u,
            Pos::new(0x180 + 6 * 0x300, 0x180),
            kind,
            crate::orders::QueuePos::New,
            action,
        );
        assert_eq!(
            sim.no_danger_mode(u),
            want,
            "{kind:?}, action {action}: no_danger"
        );
    }
    let mut sim = flat_sim(12);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    sim.units[u].owner = 8;
    assert!(sim.no_danger_mode(u), "gaia plans without danger");
}

fn push_goal(sim: &mut Sim, u: usize, to: Pos) {
    sim.units[u].path.push(PathData {
        to,
        tolerance: 0,
        flags: path_flag::FINAL,
    });
}

#[test]
fn a_near_goal_leaves_only_the_goal_on_the_stack() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    // Two cells away: cell-Manhattan 2 < 3 — no A*.
    push_goal(&mut sim, u, Pos::new(0x180 + 2 * 0x300, 0x180));
    let r = sim.find_wpath(u);
    assert_eq!(r, 1);
    assert_eq!(sim.units[u].path.len(), 1);
    assert_eq!(sim.units[u].path[0].to, Pos::new(0x180 + 2 * 0x300, 0x180));
}

#[test]
fn a_far_goal_gets_a_chain_of_cell_centres() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    // Six cells east.
    let goal = Pos::new(0x180 + 6 * 0x300, 0x180);
    push_goal(&mut sim, u, goal);
    let r = sim.find_wpath(u);
    assert!(r > 1, "expected a planned chain, got {r}");
    let path = &sim.units[u].path;
    // Bottom is the final goal; above it the goal-cell centre with
    // tolerance 0x180; the waypoints walk from the start side (top)
    // toward the goal.
    assert_eq!(path[0].to, goal);
    assert_eq!(path[0].flags & path_flag::FINAL, path_flag::FINAL);
    assert_eq!(path[1].tolerance, 0x180);
    // Every intermediate entry is a cell centre.
    for p in &path[1..] {
        assert_eq!((p.to.x - 0x180) % 0x300, 0, "{:?}", p.to);
        assert_eq!((p.to.y - 0x180) % 0x300, 0, "{:?}", p.to);
    }
    // The top entry is the first step away from the start cell, one
    // cell along the line — the walk is straight on open ground.
    let top = path.last().unwrap();
    assert_eq!(top.to, Pos::new(0x180 + 0x300, 0x180));
}

/// §7's `toff`: with a move order current, every reconstructed world
/// node comes out at `cell*0x300 + off`, where `off` is the **order's
/// own** `off_x/off_y` — the cell centre only when the offset happens
/// to be `0x180`. The seam this replaced emitted the centre always, so
/// pinning `toff` back to zero fails this on the first waypoint.
#[test]
fn a_move_orders_world_chain_carries_the_orders_own_sub_cell_offset() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    // A destination six cells east whose 48-grid snap puts it at
    // `+0x1f8` inside its cell — run20's own `off_x = 504`.
    let goal = Pos::new(6 * 0x300 + 0x1f8, 0x1f8);
    sim.add_move_order(u, goal, MoveKind::MoveTo, orders::QueuePos::New, false);
    let dest = match sim.current_order(u).expect("the move").body {
        Body::Move(mo) => mo.dest,
        _ => unreachable!(),
    };
    assert_eq!(
        (dest.x % 0x300, dest.y % 0x300),
        (504, 504),
        "the order's off"
    );
    assert_eq!(sim.toff(u), Some((504, 504)));

    push_goal(&mut sim, u, dest);
    let r = sim.find_wpath(u);
    assert!(r > 1, "expected a planned chain, got {r}");
    let path = &sim.units[u].path;
    assert_eq!(path[0].to, dest, "the goal is pushed as it stands");
    for p in &path[1..] {
        assert_eq!(p.to.x % 0x300, 504, "not on the order's offset: {:?}", p.to);
        assert_eq!(p.to.y % 0x300, 504, "not on the order's offset: {:?}", p.to);
    }
}

/// And the same walk with **no** move order current leaves the nodes on
/// the cell centre: `toff` is `None`, and then nothing is added or
/// subtracted at all.
#[test]
fn without_a_move_order_the_world_chain_stays_on_the_cell_centres() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    assert_eq!(sim.toff(u), None, "a unit with no order has no offset");
    push_goal(&mut sim, u, Pos::new(6 * 0x300 + 0x180, 0x180));
    assert!(sim.find_wpath(u) > 1);
    for p in &sim.units[u].path[1..] {
        assert_eq!(p.to.x % 0x300, 0x180, "{:?}", p.to);
        assert_eq!(p.to.y % 0x300, 0x180, "{:?}", p.to);
    }
}

#[test]
fn the_straight_chain_matches_the_axis_on_open_ground() {
    let mut sim = flat_sim(24);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let goal = Pos::new(0x180, 0x180 + 8 * 0x300);
    push_goal(&mut sim, u, goal);
    let r = sim.find_wpath(u);
    assert!(r > 1);
    for p in &sim.units[u].path[1..] {
        assert_eq!(p.to.x, 0x180, "the chain should hug the axis: {:?}", p.to);
    }
}

#[test]
fn a_goal_off_the_map_returns_minus_one_and_empties_the_stack() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    push_goal(&mut sim, u, Pos::new(-0x300, 0x180));
    assert_eq!(sim.find_wpath(u), -1);
    assert!(sim.units[u].path.is_empty());
}

#[test]
fn a_blocked_band_of_forest_is_routed_around_on_the_tile_grid() {
    let mut sim = flat_sim(12);
    // A forest wall across x-tile 8, y-tiles 0..20, with a gap at 21+.
    for ty in 0..20 {
        let t = Pos::new(8, ty);
        sim.world
            .set_tile_field(t, tile::SURFACE, tile::SURFACE_FOREST);
        sim.world.set_tile_bits(t, tile::BLOCKED);
    }
    let u = walker(&mut sim, Pos::new(4 * 0xc0 + 0x60, 4 * 0xc0 + 0x60));
    let goal = Pos::new(12 * 0xc0 + 0x60, 4 * 0xc0 + 0x60);
    push_goal(&mut sim, u, goal);
    let r = sim.find_tpath(u);
    assert!(r > 1, "expected a route, got {r}");
    // No waypoint sits on a forest tile.
    for p in &sim.units[u].path {
        let m = sim.world.tile_mask(p.to.tile());
        assert_ne!(
            m & tile::SURFACE,
            tile::SURFACE_FOREST,
            "waypoint in the trees: {:?}",
            p.to
        );
    }
}

/// **The tile grid's tolerance is the unit's, not a constant**
/// (`00684bfc`). A unit that may auto-transport is given the waypoint
/// exactly; everything else may call it reached within `0x60`.
///
/// The difference is three frames a leg at speed 25, and thirteen by
/// the time East Indies' AI citizen `1/13` reaches its wood tile —
/// which is what put the long capture's word at 4275 rather than
/// 4313 (`docs/PATHFINDER.md` §7).
#[test]
fn a_unit_that_can_transport_gets_its_tile_waypoints_exactly() {
    let mut sim = flat_sim(12);
    let u = walker(&mut sim, Pos::new(4 * 0xc0 + 0x60, 4 * 0xc0 + 0x60));
    let goal = Pos::new(20 * 0xc0 + 0x60, 12 * 0xc0 + 0x60);
    push_goal(&mut sim, u, goal);
    assert!(sim.find_tpath(u) > 1, "expected a tile route");
    let plain: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
    assert!(
        !plain.is_empty() && plain.iter().all(|&t| t == 0x60),
        "a unit that cannot transport takes the half-tile tolerance: {plain:?}"
    );

    // The same walk with `unit_masks & 0x800000` set.
    sim.units[u].path.clear();
    sim.units[u].auto_transport = true;
    push_goal(&mut sim, u, goal);
    assert!(sim.find_tpath(u) > 1, "expected a tile route");
    let exact: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
    assert_eq!(exact.len(), plain.len(), "the same route, twice");
    assert!(
        exact.iter().all(|&t| t == 0),
        "and it walks onto each waypoint: {exact:?}"
    );

    // `unit_masks2 & 0x2000` is the scenario's veto and takes it back.
    sim.units[u].path.clear();
    sim.units[u].never_transport = true;
    push_goal(&mut sim, u, goal);
    assert!(sim.find_tpath(u) > 1, "expected a tile route");
    let vetoed: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
    assert_eq!(vetoed, plain, "the veto puts the half-tile back");
}

/// **`find_tpath`'s pull-back walk is `find_upath`'s gate** (item
/// 1589, `006897d0`, listing `68994c`–`689960`): a goal tile in another
/// tile region from the unit's own is walked back toward the unit only
/// when the unit cannot transport. run710's Freighter `1/52` kept its
/// goal across the regions and searched to it; this crate walked the
/// goal home and refused the plan (`docs/AI.md` §163).
#[test]
fn a_transport_keeps_a_tile_goal_in_another_region() {
    let start = Pos::new(4 * 0xc0 + 0x60, 4 * 0xc0 + 0x60);
    let goal = Pos::new(20 * 0xc0 + 0x60, 4 * 0xc0 + 0x60);
    let bottom = |transport: bool| {
        let mut sim = flat_sim(12);
        let other = sim.world.add_region(Terrain::Land);
        for x in 4..12 {
            for y in 0..12 {
                sim.world.set_region(Cell::new(x, y), other);
            }
        }
        assert_ne!(
            sim.world.tregion_alt(start.tile()),
            sim.world.tregion_alt(goal.tile()),
            "the goal is in another region"
        );
        let u = walker(&mut sim, start);
        sim.units[u].auto_transport = transport;
        push_goal(&mut sim, u, goal);
        assert!(sim.find_tpath(u) > 0, "expected a tile route");
        sim.units[u].path[0].to
    };
    assert_eq!(bottom(true), goal, "a transport keeps its goal");
    let walked = bottom(false);
    assert_ne!(walked, goal, "a unit that cannot transport walks it back");
    assert!(walked.x < goal.x, "toward its own region: {walked:?}");
}

#[test]
fn only_ai_transport_types_drop_same_region_terrain_preferences() {
    for water in [false, true] {
        for human in [false, true] {
            for transport in [false, true] {
                let mut sim = flat_sim(4);
                sim.cities.nation[0].human = human;
                if water {
                    sim.world
                        .fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(3, 3));
                }
                let start = Pos::new(384, 384);
                let goal = Pos::new(1152, 384);
                if water {
                    for p in [start, goal] {
                        sim.world
                            .set_tile_field(p.tile(), tile::SURFACE, tile::SURFACE_OCEAN);
                        let mut data = sim.world.cell_data(p.cell());
                        data.land = 1;
                        sim.world.set_cell_data(p.cell(), data);
                    }
                }
                let u = walker(&mut sim, start);
                let ty = sim.add_unit_type(crate::UnitType::default());
                sim.units[u].ty = Some(ty);
                if transport {
                    sim.tables.unit_types[ty].cols.unit_flags |=
                        crate::ai_load::uflags::FLAG_TRANSPORT;
                }
                let want = if !human && transport {
                    (0, 0)
                } else if water {
                    (1, 0)
                } else {
                    (0, 1)
                };
                for step in [STEP_WORLD, STEP_TILE, STEP_UNIT] {
                    assert_eq!(
                        sim.avoid_flags(u, start, goal, step),
                        want,
                        "water={water} human={human} transport={transport} step={step}"
                    );
                }
            }
        }
    }
}

/// **`find_upath`'s near test zeroes the waypoint's tolerance**
/// (`683273`–`68327d`, `docs/AI.md` §231): a goal one 48-cell from the
/// unit is pushed back alone, as the same-cell test pushes it, with
/// `tolerance` 0 — a leg's 384 kept would pop it where the unit stands.
#[test]
fn a_near_unit_goal_is_pushed_with_no_tolerance() {
    let mut sim = flat_sim(20);
    let here = Pos::new(0x18 + 0x30 * 10, 0x18 + 0x30 * 10);
    let u = walker(&mut sim, here);
    let to = Pos::new(here.x + 0x30, here.y);
    sim.units[u].path.push(PathData {
        to,
        tolerance: 0x180,
        flags: 0,
    });
    assert_eq!(sim.find_upath(u, false), 1);
    assert_eq!(
        sim.units[u].path.last().copied(),
        Some(PathData {
            to,
            tolerance: 0,
            flags: 0,
        }),
        "the near goal alone, its tolerance zeroed"
    );
}

/// **And a final goal the pull-back gives up on keeps its tolerance**
/// (`68322e`–`683232`, `docs/AI.md` §231): within `0x18` of the unit on
/// both axes, in a region not its own, the give-up jumps to the plain
/// push at `683082`, past the near test that would zero it.
#[test]
fn a_final_unit_goal_given_up_on_keeps_its_tolerance() {
    let mut sim = flat_sim(8);
    sim.world
        .fill_region(Terrain::Sea, Cell::new(3, 0), Cell::new(7, 7));
    let here = Pos::new(3 * 768 - 6, 1152);
    let u = walker(&mut sim, here);
    let to = Pos::new(3 * 768 + 4, 1152);
    assert_ne!(
        sim.world.tregion_alt(to.tile()),
        sim.world.tregion_alt(here.tile()),
        "the goal is across the shore"
    );
    let goal = PathData {
        to,
        tolerance: 0x180,
        flags: path_flag::FINAL,
    };
    sim.units[u].path.push(goal);
    assert_eq!(sim.find_upath(u, false), 1);
    assert_eq!(
        sim.units[u].path.last().copied(),
        Some(goal),
        "pushed as it stands"
    );
}

/// A large unit searches at its own stride and cannot jump a refused
/// intermediate diagonal cell (PATHFINDER §30).
#[test]
fn large_recovery_steps_check_intermediate_diagonals() {
    for radius in [48, 144] {
        let mut sim = flat_sim(8);
        let start = Pos::new(1560, 1560);
        let u = walker(&mut sim, start);
        let ty = sim.add_unit_type(crate::UnitType {
            combat: crate::combat::Profile {
                block_radius: radius,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        sim.units[u].ty = Some(ty);
        sim.trace.trace_costs = true;
        sim.trace.probe_refuse = Some(RefuseProbe {
            frame: sim.frame,
            unit: (0, 1),
            cell: crate::collide::ucell(Pos::new(start.x + 48, start.y + 48)),
        });
        push_goal(&mut sim, u, Pos::new(start.x + 960, start.y + 960));
        assert!(sim.find_upath(u, false) > 0);
        let first: Vec<_> = sim
            .trace
            .cost_marks
            .iter()
            .filter(|m| m.from == (start.x, start.y))
            .collect();
        assert!(!first.is_empty());
        let stride = if radius == 48 { 48 } else { 96 };
        assert!(
            first
                .iter()
                .all(|m| { (m.to.0 - start.x).abs().max((m.to.1 - start.y).abs()) == stride })
        );
        assert!(
            !first
                .iter()
                .any(|m| m.to == (start.x + stride, start.y + stride)),
            "the refused intermediate point blocks the diagonal"
        );
    }
}

/// **The suspend, and the resume that finishes what it started**
/// (§4.3 step 3, §18, item 301). A 48-grid plan across a long stretch
/// of open ground with the budget cut to a handful of probes stops
/// short, hands its whole state to the unit, and returns −1 with the
/// stack holding nothing but the caller's final goal. Resumed — over
/// and over, exactly as `do_move`'s suspended-search block resumes it —
/// it reaches the same goal it would have reached in one call.
///
/// Made to fail on purpose twice: with the stash dropped instead of
/// stored the resume answers 0 on the first call, and with the stop
/// node **not** re-inserted into the open list the resumed plan is a
/// different chain — 52 entries where the one call's compacts to 3,
/// because the node the search stopped on is reached again through its
/// neighbours instead of being expanded first.
#[test]
fn a_suspended_unit_search_is_resumed_where_it_stopped() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x18, 0x18));
    let goal = Pos::new(0x18 + 40 * 0x30, 0x18 + 40 * 0x30);

    // The whole plan, in one call, is the control.
    push_goal(&mut sim, u, goal);
    assert!(sim.find_upath(u, false) > 1, "the control plan");
    let whole = sim.units[u].path.clone();
    sim.units[u].path.clear();

    // Now the same plan on a budget of ten probes.
    push_goal(&mut sim, u, goal);
    assert_eq!(
        sim.upath(u, false, 10, false),
        -1,
        "a 48-grid plan over its limit and short of the goal suspends"
    );
    assert!(
        sim.units[u].search.is_some(),
        "the suspend hands the search to the unit"
    );
    assert_eq!(
        sim.units[u].path,
        vec![PathData {
            to: goal,
            tolerance: 0,
            flags: path_flag::FINAL
        }],
        "`astar_path` popped its own start and goal and pushed nothing: \
             the stack is the caller's final goal and nothing else"
    );
    let stashed = sim.units[u].search.as_ref().expect("the stash");
    assert_eq!(
        stashed.start_dist,
        (goal.x - 0x18).abs() + (goal.y - 0x18).abs(),
        "`UnitData::start_dist` is the start-to-goal Manhattan, and the \
             suspend is its only writer"
    );
    assert_eq!(stashed.goal, goal, "`endx`/`endy`");

    // Resume until it lands. Each resume is `find_upath_restore`'s own
    // call with the same small budget.
    let mut resumes = 0;
    let r = loop {
        resumes += 1;
        assert!(resumes < 400, "the resumed search never finished");
        let r = sim.upath(u, false, 10, true);
        if r != -1 {
            break r;
        }
    };
    assert!(resumes > 1, "one resume would not have tested anything");
    assert!(r > 1, "the resumed search lands a path, got {r}");
    assert!(
        sim.units[u].search.is_none(),
        "a finished search leaves no stash"
    );
    assert_eq!(
        sim.units[u].path, whole,
        "the resumed plan is the one call's plan, entry for entry"
    );

    // And `clear_partial_path` is what frees a stash that never
    // finishes — the original's `Unit::clear_partial_path@005e3920`.
    sim.units[u].path.clear();
    push_goal(&mut sim, u, goal);
    assert_eq!(sim.upath(u, false, 10, false), -1);
    sim.clear_partial_path(u);
    assert!(sim.units[u].search.is_none(), "the teardown frees it");
    assert_eq!(
        sim.upath(u, false, 10, true),
        0,
        "a resume with nothing stashed is a failure, not a panic"
    );
}

/// **A gatherer stuck on a suspended search gives its walk up**
/// (`docs/COLLISION.md` §15, item 698). `do_move@005f7b30`'s suspended
/// block, ahead of the blocker probe: when the action under the move is
/// a `GATHER`, on the frame `(frame − collide_frame + 2) & 7 == 0`, and
/// within `vector_dist < 0x120` of the move's own point, `avoid` takes
/// that point, the gather order forgets its tile (`tx`/`ty`/`wait` −1,
/// `goto_build` 1), and the move dies without a `collide` count.
///
/// run178's `1/43` is the case: six frames after it met the standing
/// `1/18`, 96 and 240 units short of its point, `vector_dist` 259.
/// Each control changes one input: seven frames, 288 units short
/// (`vector_dist` 304), and a plain walk with no gather under it. Made
/// to fail on purpose with the arm taken out: the walk is kept.
#[test]
fn a_gatherer_on_a_suspended_search_gives_its_walk_up_near_its_point() {
    use crate::orders::{Body, MoveKind, QueuePos};
    let run = |elapsed: i64, short: i32, gathering: bool| {
        let mut sim = flat_sim(20);
        let at = Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18);
        let u = walker(&mut sim, at);
        let camp = sim.add_building(1, Pos::new(0x600, 0x600), 1);
        if gathering {
            sim.add_gather_order(u, camp, QueuePos::New, true);
            if let Some(Body::Gather(g)) = sim.units[u].orders.front_mut().map(|o| &mut o.body) {
                g.tile = Some(Pos::new(30, 30));
                g.wait = 408;
                g.goto_build = false;
            }
        }
        let point = Pos::new(at.x + 96, at.y + short);
        sim.add_move_order(u, point, MoveKind::MoveTo, QueuePos::First, false);
        // The suspended 48-grid search, on a goal of its own.
        push_goal(&mut sim, u, Pos::new(at.x + 40 * 0x30, at.y + 40 * 0x30));
        assert_eq!(sim.upath(u, false, 10, false), -1, "the plan suspends");
        sim.units[u].collide = 6;
        sim.units[u].collide_frame = 1000;
        sim.work(u, 1000 + elapsed);
        let dest = match sim.units[u].orders.front().map(|o| o.body) {
            Some(Body::Move(m)) => Some(m.dest),
            _ => None,
        };
        (sim, u, dest)
    };

    let (sim, u, front) = run(6, 240, true);
    assert_eq!(front, None, "the walk is gone");
    let Some(Body::Gather(g)) = sim.units[u].orders.front().map(|o| o.body) else {
        panic!(
            "the gather order is the front order: {:?}",
            sim.units[u].orders
        );
    };
    assert_eq!(
        (g.tile, g.wait, g.goto_build),
        (None, -1, true),
        "the gather order forgets its tile"
    );
    assert_eq!(sim.units[u].collide, 6, "no count: the arm is above it");
    let (_, _, kept) = run(7, 240, true);
    assert_eq!(sim.units[u].avoid, kept, "`avoid` takes the move's point");

    for (elapsed, short, gathering, why) in [
        (7, 240, true, "seven frames"),
        (6, 288, true, "vector_dist 304"),
        (6, 240, false, "no gather under the walk"),
    ] {
        let (_, _, front) = run(elapsed, short, gathering);
        assert!(front.is_some(), "{why}: the walk is kept");
    }
}

/// **An attacker stuck on a suspended search re-finds its target
/// every fourth frame** (`do_move@005f7b30`'s `get_type == 10` arm,
/// item 3336; run967g's Fusilier `1/162` on 20029, four frames after
/// it met `1/119`): under a non-mandatory `ATTACK` action, with its
/// player's `repaths` non-zero, the move dies, `find_new_target` runs
/// and `collide_frame` takes the frame — no `collide` count. Each
/// control changes one input: three frames, a mandatory attack, and
/// `repaths` zero.
///
/// Made to fail by taking the arm out, with the Egyptian word at 20031
/// (item 3336's M1).
#[test]
fn an_attacker_on_a_suspended_search_retargets_every_fourth_frame() {
    use crate::combat::Obj;
    use crate::orders::{Body, MoveKind, QueuePos};
    let run = |elapsed: i64, mandatory: bool, repaths: i32| {
        let mut sim = flat_sim(20);
        let at = Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18);
        let u = walker(&mut sim, at);
        let foe = sim.units.len();
        sim.units
            .push(Unit::new(1, 2, Pos::new(at.x + 12 * 0x30, at.y), 10));
        sim.add_attack_order(u, Obj::Unit(foe), QueuePos::New, mandatory, true);
        sim.add_move_order(
            u,
            Pos::new(at.x + 96, at.y),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        push_goal(&mut sim, u, Pos::new(at.x + 40 * 0x30, at.y + 40 * 0x30));
        assert_eq!(sim.upath(u, false, 10, false), -1, "the plan suspends");
        sim.movement.repaths[0] = repaths;
        sim.units[u].collide = 4;
        sim.units[u].collide_frame = 1000;
        sim.work(u, 1000 + elapsed);
        let walking = sim.units[u]
            .orders
            .iter()
            .any(|o| matches!(o.body, Body::Move(_)));
        (walking, sim.units[u].collide, sim.units[u].collide_frame)
    };
    assert_eq!(run(4, false, 1), (false, 4, 1004), "the retarget");
    for (elapsed, mandatory, repaths, why) in [
        (3, false, 1, "three frames"),
        (4, true, 1, "a mandatory attack"),
        (4, false, 0, "repaths zero"),
    ] {
        let (walking, _, stamp) = run(elapsed, mandatory, repaths);
        assert!(walking || stamp != 1000 + elapsed, "{why}: no retarget");
    }
}

/// **`kill_current_path` frees the stash, and an empty stack does
/// not** (§18.4, item 304). `Unit::kill_current_path@005e31d0` pops
/// the stack back through the current segment's final waypoint and
/// then calls `clear_partial_path` — and the call sits *inside* the
/// function's own `0 < length` guard, so a unit with nothing on its
/// stack keeps whatever search it had.
///
/// That guard is the whole difference between this and
/// `kill_current_order`, which clears unconditionally, and it is why
/// the assertion below is two-sided rather than one. The behaviour it
/// buys is `Unit::ungroup_move_order@005fd140`'s: a follower dropped
/// out of formation loses its suspended search on the same frame, and
/// re-plans on the next one instead of standing in `do_move`'s
/// suspended block for the rest of the capture.
///
/// Made to fail on purpose both ways: without the `clear_partial_path`
/// call the first assertion fails (and run76's `1/28` stops re-planning
/// on 6861), and without the emptiness guard the second does.
#[test]
fn killing_the_current_path_frees_a_suspended_search() {
    let mut sim = flat_sim(20);
    let u = walker(&mut sim, Pos::new(0x18, 0x18));
    let goal = Pos::new(0x18 + 40 * 0x30, 0x18 + 40 * 0x30);

    push_goal(&mut sim, u, goal);
    assert_eq!(sim.upath(u, false, 10, false), -1, "the plan suspends");
    assert!(sim.units[u].search.is_some(), "the stash is on the unit");
    // The suspend leaves the caller's final goal on the stack, so the
    // pop runs and the clear with it.
    assert_eq!(sim.units[u].path.len(), 1);
    sim.kill_current_path(u);
    assert!(
        sim.units[u].path.is_empty() && sim.units[u].search.is_none(),
        "a non-empty stack: the segment goes and the stash goes with it"
    );

    // And the guard. Suspend a second search, empty the stack by hand,
    // and the teardown is a no-op on both.
    push_goal(&mut sim, u, goal);
    assert_eq!(sim.upath(u, false, 10, false), -1);
    sim.units[u].path.clear();
    sim.kill_current_path(u);
    assert!(
        sim.units[u].search.is_some(),
        "`0 < length` gates the clear: an empty stack frees nothing"
    );
}

/// **A walled-in attack-move buys the retry, and keeps its order**
/// (item 673, `docs/PATHFINDER.md` §21.6). The gate on
/// `astar_path`'s open-list-exhausted roll (`call *0x14` at
/// `00684d5b`) and on `find_upath`'s reprieve (`006833c9`) is vslot
/// `+0x14`, `is_move`, which every move class answers with `return 1`:
/// the action bit is not read. So an `ATTACK_TO` with the action bit
/// set fares exactly as a transit leg does: one roll, `retry` in
/// `6..=8`, and the order stays. Against the old `is_transit` gate the
/// action-bit arm lost its order here. That was Great Lakes' `1/27` on
/// 12536, whose ungroup never came and so left its squad in formation.
#[test]
fn a_walled_in_attack_move_buys_the_retry_and_keeps_its_order() {
    for action in [true, false] {
        let mut sim = flat_sim(12);
        // A ring of blocked tiles two out from tile (5, 5).
        for d in -2..=2 {
            for t in [(5 + d, 3), (5 + d, 7), (3, 5 + d), (7, 5 + d)] {
                let t = Pos::new(t.0, t.1);
                sim.world
                    .set_tile_field(t, tile::SURFACE, tile::SURFACE_FOREST);
                sim.world.set_tile_bits(t, tile::BLOCKED);
            }
        }
        let u = walker(&mut sim, Pos::new(5 * 0xc0 + 0x60, 5 * 0xc0 + 0x60));
        let goal = Pos::new(20 * 0xc0 + 0x60, 5 * 0xc0 + 0x60);
        sim.add_move_order(
            u,
            goal,
            MoveKind::AttackTo,
            crate::orders::QueuePos::New,
            action,
        );
        push_goal(&mut sim, u, goal);
        // A budget no ring this small can spend, so the open list is
        // exhausted rather than the search suspended.
        assert_eq!(
            sim.upath(u, false, 100_000, false),
            0,
            "the ring refuses the plan"
        );
        let m = sim
            .current_move(u)
            .unwrap_or_else(|| panic!("action {action}: the order is spared"));
        assert!(
            (6..=8).contains(&m.retry),
            "action {action}: `retry` is the roll, got {}",
            m.retry
        );
    }
}

#[test]
fn the_unit_grid_plans_in_48_cells_and_compacts_the_line() {
    let mut sim = flat_sim(12);
    let u = walker(&mut sim, Pos::new(0x18, 0x18));
    // Ten 48-cells east: far enough for a chain, straight enough for
    // the collinear compaction to bite.
    let goal = Pos::new(0x18 + 10 * 0x30, 0x18);
    push_goal(&mut sim, u, goal);
    let r = sim.find_upath(u, false);
    assert!(r >= 1, "expected a unit-grid plan, got {r}");
    let path = &sim.units[u].path;
    assert_eq!(path[0].to, goal);
    // Every planned waypoint carries the side-step flag and zero
    // tolerance, and the collinear middle was compacted away.
    for p in &path[1..] {
        assert_eq!(p.flags & path_flag::SIDESTEP, path_flag::SIDESTEP);
        assert_eq!(p.tolerance, 0);
    }
    assert!(
        path.len() < 11,
        "collinear 48-centres should compact: {} entries",
        path.len()
    );
}

#[test]
fn ties_are_broken_newest_first() {
    // Two equal-value keys: the later insertion must come out first.
    let mut open: BTreeMap<(i32, std::cmp::Reverse<u64>), u32> = BTreeMap::new();
    open.insert((100, std::cmp::Reverse(1)), 10);
    open.insert((100, std::cmp::Reverse(2)), 20);
    let (_, &first) = open.first_key_value().unwrap();
    assert_eq!(first, 20, "LIFO on equal f");
}

/// **The `army` mode's predicate, all four terms** (§22, §29).
///
/// Made to fail on purpose: read `attacking` as "has a combat target"
/// and the second case flips (it is the shape that cost Great Lakes
/// 322 frames, §22.3); drop `!attacking` and the third flips (item
/// 899: Great Lakes 8186's walk home and 17656's, §29); drop the
/// `!river` term and the fifth flips; drop `!worker` and the sixth.
/// The second and third have a value diff behind them, and the fifth
/// and sixth rest on the listing alone.
#[test]
fn an_army_is_armed_unworked_and_off_the_river() {
    let mut sim = flat_sim(10);
    let archer = sim.add_unit_type(crate::UnitType {
        hits: 40,
        combat: crate::combat::Profile {
            attack: 5,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let citizen = sim.add_unit_type(crate::UnitType {
        hits: 40,
        worker: Worker::Citizen,
        combat: crate::combat::Profile {
            attack: 5,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let at = Pos::new(0x180, 0x180);
    let u = walker(&mut sim, at);
    sim.units[u].ty = Some(archer);
    assert!(sim.army_mode(u), "an armed non-worker on plain ground");

    // **`is_attacking` is not a term.** The original's clause calls
    // the current order's `+0x18` virtual, and that slot is a bare
    // `return 0` in every order vtable the executable ships (§22.1),
    // so a unit marching on a target is still an army — which is the
    // only kind of unit the mode exists for.
    sim.units[u].combat.target = Some(crate::combat::Obj::Unit(0));
    assert!(
        sim.army_mode(u),
        "a combat target is not what `is_attacking` tests, and nothing is"
    );
    sim.units[u].combat.target = None;

    // **`is_attacking` is the current order's `is_attack`** (§29):
    // `1` for an `AttackOrder` (and a `StrafeOrder`), `0` for every
    // move, an `ATTACK_TO` among them. Only the *current* order asks:
    // an attack queued behind a move leaves the unit an army.
    sim.add_move_order(
        u,
        Pos::new(0x480, 0x480),
        MoveKind::AttackTo,
        orders::QueuePos::New,
        true,
    );
    assert!(sim.army_mode(u), "an `ATTACK_TO` is a move, and an army");
    sim.add_attack_order(
        u,
        crate::combat::Obj::Unit(0),
        orders::QueuePos::Last,
        false,
        true,
    );
    assert!(
        sim.army_mode(u),
        "an attack queued behind the move is not the current order"
    );
    sim.add_attack_order(
        u,
        crate::combat::Obj::Unit(0),
        orders::QueuePos::New,
        false,
        true,
    );
    assert!(
        !sim.army_mode(u),
        "a unit whose current order is an `ATTACK` plans as a citizen"
    );
    sim.units[u].orders.clear();
    sim.units[u].combat.target = None;
    assert!(sim.army_mode(u), "and with no order it is an army again");

    // The river clause, on the unit's **own** cell.
    let mut d = sim.world.cell_data(at.cell());
    d.flags |= crate::world::cell::HALFLAND;
    sim.world.set_cell_data(at.cell(), d);
    assert!(!sim.army_mode(u), "a unit standing on a `0x100` cell");
    d.flags &= !crate::world::cell::HALFLAND;
    sim.world.set_cell_data(at.cell(), d);

    // And a worker is never an army, however well armed.
    sim.units[u].ty = Some(citizen);
    assert!(
        !sim.army_mode(u),
        "a worker with an attack is still a worker"
    );
}

/// **The depth-1 shore test reads the wrapper's start**, not the unit's
/// own tile (twenty-fourth pass, group 7; A4 row 28): `calc_cost`
/// redoes `needs_transport` from `PathFinder +0x58/+0x5c`, which
/// `find_wpath` writes from its start argument. A group's plan starts
/// somewhere else than the unit stands.
#[test]
fn the_first_step_s_shore_test_starts_where_the_plan_does() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    sim.units[u].auto_transport = true;
    let m = Modes::default();
    let from = Pos::new(0x180 + 0x300, 0x180);
    let to = Pos::new(0x180 + 0x600, 0x180);
    let wet = Pos::new(0x180 + 0x600 + 12, 0x180);
    for t in [to.tile(), wet.tile()] {
        sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
    }
    // The plan starts on dry ground: the step into the sea embarks.
    sim.movement.pf_start = from.tile();
    let (_, dry) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 1, 0, 1);
    assert!(dry, "dry start, wet target");
    // The plan starts at sea, though the unit stands on land: no
    // shoreline crossed from where the search began.
    sim.movement.pf_start = wet.tile();
    let (_, wetstart) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 1, 0, 1);
    assert!(!wetstart, "the unit's own tile is not asked");
}

/// **An exhausted budget drains on every grid but the unit grid**
/// (group 8; A4 row 31): the tile grid without `anti_unit` is not an
/// exception.
#[test]
fn the_work_cap_drains_every_grid_but_the_unit_grid() {
    assert_eq!(cap_tail(STEP_WORLD, false), CapTail::Drain);
    assert_eq!(cap_tail(STEP_WORLD, true), CapTail::Drain);
    assert_eq!(cap_tail(STEP_TILE, false), CapTail::Drain);
    assert_eq!(cap_tail(STEP_TILE, true), CapTail::Refuse);
    assert_eq!(cap_tail(STEP_UNIT, false), CapTail::Pause);
}

#[test]
fn the_diagonal_costs_eight_more() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let m = Modes::default();
    let from = Pos::new(0x180 + 0x300, 0x180 + 0x300);
    let east = Pos::new(from.x + 0x300, from.y);
    let se = Pos::new(from.x + 0x300, from.y + 0x300);
    let (c_card, _) = sim.calc_cost(u, &m, from, east, 4, STEP_WORLD, 2, 0, 1);
    let (c_diag, _) = sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 2, 0, 1);
    assert_eq!(c_card, 32);
    assert_eq!(c_diag, 40);
}

#[test]
fn enemy_territory_charges_and_the_own_discount_clamps_at_zero() {
    // The additive term is clamped at zero after the terrain add
    // (`docs/PATHFINDER.md` §5), so with no danger to offset, one's own
    // territory costs the same as unowned ground — the −4 only ever
    // cancels danger. Enemy territory's +4 survives the clamp.
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let m = Modes::default();
    let from = Pos::new(0x180, 0x180);
    let to = Pos::new(0x180 + 0x300, 0x180);
    sim.world
        .set_owner(to.cell(), Owner::Player(0), Owner::None);
    let (own, _) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1);
    assert_eq!(own, 32);
    sim.world
        .set_owner(to.cell(), Owner::Player(1), Owner::None);
    sim.leaders.at_war[0][1] = true;
    let (enemy, _) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1);
    assert_eq!(enemy, 36);
}

/// §5's no-rush timer (item 2423, `docs/AI.md` §223): while the rush
/// rules forbid war, a cell another leader owns costs 500 more unless
/// the two are allies; one's own cell, an unowned one, and any cell once
/// the peace is over pay nothing.
#[test]
fn the_no_rush_timer_charges_another_leader_s_cell_during_the_peace() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let m = Modes::default();
    let from = Pos::new(0x180, 0x180);
    let to = Pos::new(0x180 + 0x300, 0x180);
    sim.ai.lobby.rush_rules = 10;
    sim.ai.lobby.rush_minutes = 5;
    sim.frame = 4499;
    let cost = |sim: &Sim| sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 0).0;
    assert_eq!(cost(&sim), 32, "unowned");
    sim.world
        .set_owner(to.cell(), Owner::Player(0), Owner::None);
    assert_eq!(cost(&sim), 32, "its own leader's");
    sim.world
        .set_owner(to.cell(), Owner::Player(1), Owner::None);
    assert_eq!(cost(&sim), 532, "another leader's, during the peace");
    sim.leaders.allied[0][1] = true;
    assert_eq!(cost(&sim), 532, "an ally one way only");
    sim.leaders.allied[1][0] = true;
    assert_eq!(cost(&sim), 32, "an ally both ways");
    sim.leaders.allied[0][1] = false;
    sim.leaders.allied[1][0] = false;
    sim.frame = 4500;
    assert_eq!(cost(&sim), 32, "the peace is over");
    sim.frame = 0;
    sim.ai.lobby.rush_rules = 0;
    assert_eq!(cost(&sim), 32, "no rush rules");
}

/// §5's terrain term: `+20 × WData.blocked`, the `+100000` at 13, and
/// an army's `+10000` at 5. None of it fires on a flat world, which is
/// why it sat behind a seam for so long; all of it fires the moment a
/// cell record carries a cost.
#[test]
fn rough_ground_costs_twenty_a_point_and_thirteen_is_a_wall() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let from = Pos::new(0x180, 0x180);
    let to = Pos::new(0x180 + 0x300, 0x180);
    let cost = |sim: &Sim, m: &Modes| sim.calc_cost(u, m, from, to, 4, STEP_WORLD, 2, 0, 1).0;
    let plain = Modes::default();
    let army = Modes {
        army: true,
        ..Modes::default()
    };
    assert_eq!(cost(&sim, &plain), 32, "clean ground");

    let mut d = sim.world.cell_data(to.cell());
    d.blocked = 4;
    sim.world.set_cell_data(to.cell(), d);
    assert_eq!(cost(&sim, &plain), 32 + 80);
    assert_eq!(cost(&sim, &army), 32 + 80, "4 is under the army's line");

    d.blocked = 5;
    sim.world.set_cell_data(to.cell(), d);
    assert_eq!(cost(&sim, &plain), 32 + 100);
    assert_eq!(cost(&sim, &army), 32 + 100 + 10_000, "rough, to an army");

    d.blocked = 13;
    sim.world.set_cell_data(to.cell(), d);
    assert_eq!(cost(&sim, &plain), 32 + 260 + 100_000, "impassable");

    // The forest-walker reads the **signed** `solid` byte instead, and
    // a negative one is a discount the clamp then eats.
    d.solid = -1;
    sim.world.set_cell_data(to.cell(), d);
    let iroquois = Modes {
        iroquois: true,
        ..Modes::default()
    };
    assert_eq!(cost(&sim, &iroquois), 32, "solid −1, clamped at zero");
}

/// §5's fog branch, and the half of it that is the whole of scouting:
/// on ground it cannot see, a scout's base is **8** against the `0x400`
/// it pays for ground it can — a factor of 128 — so a scout's cheapest
/// route is the one through the dark. Everyone else pays `0x124 + 8`
/// for the unknown, slightly more than the `0x100` of the known.
///
/// The read is at the step's **half-cell**, `to / 0x180`, not the
/// cell's `2c + 1` sample: with the first half-cell of the destination
/// dark and the second lit, a step landing in the first is priced as
/// unseen. No capture on disk separates those two readings — every
/// world node in every dump sits past the half-cell line — so this is
/// the only thing holding the convention.
#[test]
fn the_unseen_is_cheap_to_a_scout_and_the_read_is_the_half_cell() {
    let mut sim = flat_sim(10);
    // A fog grid of all-dark. `seen2` is one byte a half-cell, a bit a
    // player, `2 × width` across.
    let fw = 20usize;
    let mut fog = vec![0u8; fw * fw];
    assert!(sim.world.set_fog(fog.clone()));
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    assert_eq!(sim.units[u].owner, 0);
    let from = Pos::new(0x180, 0x180);
    let scout = Modes {
        scouting: true,
        ..Modes::default()
    };
    let plain = Modes::default();
    let cost =
        |sim: &Sim, m: &Modes, to: Pos| sim.calc_cost(u, m, from, to, 4, STEP_WORLD, 2, 0, 1).0;

    // Dark everywhere: `base 8` for the scout, `0x124 + 8` for the rest.
    let to = Pos::new(0x180 + 0x300, 0x180);
    assert_eq!(cost(&sim, &scout, to), 1, "8 × 32 / 256");
    assert_eq!(cost(&sim, &plain, to), 0x124 * 32 / 256 + 8);

    // Light the destination cell's **second** half-cell only. `to` is
    // at `+0x180` inside its cell, which is the second half — so it
    // reads lit, and the scout pays the seen base.
    let c = to.cell();
    for fy in [2 * c.y, 2 * c.y + 1] {
        fog[fy as usize * fw + (2 * c.x + 1) as usize] = 1;
    }
    assert!(sim.world.set_fog(fog));
    assert_eq!(cost(&sim, &scout, to), 0x400 * 32 / 256, "seen: 128");
    assert_eq!(cost(&sim, &plain, to), 32);

    // A step landing in the same cell's **first** half-cell is still
    // dark. `2c + 1` would call it seen; `to / 0x180` does not.
    let first_half = Pos::new(c.x * 0x300 + 0x80, 0x80);
    assert_eq!(first_half.cell(), c);
    assert_eq!(cost(&sim, &scout, first_half), 1, "the near half is dark");
}

/// A lake of sea cells at `x` in `2..=4`, rows `2..=7`, on a ten-cell
/// land map: region, `land` and every tile ocean, as run180's sand.
fn lake_sim() -> Sim {
    let mut sim = flat_sim(10);
    let sea = sim.world.add_region(Terrain::Sea);
    for x in 2..=4 {
        for y in 2..=7 {
            let c = Cell::new(x, y);
            sim.world.set_region(c, sea);
            let mut d = sim.world.cell_data(c);
            d.land = 1;
            sim.world.set_cell_data(c, d);
            for ty in 0..TILES_PER_CELL {
                for tx in 0..TILES_PER_CELL {
                    let t = Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty);
                    sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
                }
            }
        }
    }
    // All dark: `seen2` a byte a half-cell, `2 × width` across.
    assert!(sim.world.set_fog(vec![0u8; 20 * 20]));
    sim
}

/// **`invalid_loc`'s fog arm** (item 676, `00607c9e`–`00607d4c`): with
/// `fog_relax` set and a **human** leader, a tile whose cell's four
/// fog half-cells are all unseen is valid before its terrain is read.
/// Made to fail first on the seam it replaces, which refused the lake
/// tile whatever the fog.
#[test]
fn a_human_s_probe_takes_an_unseen_cell_as_valid() {
    let mut sim = lake_sim();
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let t = Pos::new(3 * TILES_PER_CELL + 1, 4 * TILES_PER_CELL + 1);
    sim.cities.nation[0].human = true;
    assert_eq!(
        sim.invalid_loc(u, t, true, true, false, true, false),
        loc::VALID
    );
    assert_eq!(
        sim.invalid_loc(u, t, true, false, false, true, false),
        loc::TERRAIN,
        "without fog_relax the lake refuses"
    );
    sim.cities.nation[0].human = false;
    assert_eq!(
        sim.invalid_loc(u, t, true, true, false, true, false),
        loc::TERRAIN,
        "a computer leader's probe reads the terrain"
    );
    // One of the four half-cells seen is enough to read the terrain.
    sim.cities.nation[0].human = true;
    let mut fog = vec![0u8; 20 * 20];
    fog[9 * 20 + 7] = 1; // (2·3 + 1, 2·4 + 1)
    assert!(sim.world.set_fog(fog));
    assert_eq!(
        sim.invalid_loc(u, t, true, true, false, true, false),
        loc::TERRAIN
    );
}

/// A walker of player 0 armed (`attack` 40, a Citizen's) or not, and a
/// building of player 1 whose one footprint tile is blocked, `(10,
/// 10)`. `add_building` leaves the type unset, so `covers_tile`
/// answers the building's own tile alone.
fn ring_tile(armed: bool) -> (Sim, usize, usize, Pos) {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let mut ty = crate::UnitType::default();
    ty.combat.domain = crate::attrition::Domain::Land;
    ty.combat.attack = if armed { 40 } else { 0 };
    let t = sim.add_unit_type(ty);
    sim.units[u].ty = Some(t);
    let tile_at = Pos::new(10, 10);
    sim.world
        .set_tile_mask(tile_at, tile::OBJECT_BUILDING | tile::BLOCKED);
    let b = sim.add_building(1, Pos::new(10 * 0xc0, 10 * 0xc0), 0);
    (sim, u, b, tile_at)
}

/// **`invalid_loc`'s armed arm** (item 1209, `00607fb6`..`0060800a`):
/// under `param_5` a walker whose type is armed passes a blocked,
/// built tile when the building found there is **not its own**, and
/// when none is found; its own refuses, as does any building to an
/// unarmed walker or without `param_5`. Made to fail first on the seam
/// it replaces, which refused every blocked tile.
#[test]
fn an_armed_walker_passes_another_player_s_footprint_and_not_its_own() {
    let (mut sim, u, b, t) = ring_tile(true);
    let tile_search = |sim: &Sim| sim.invalid_loc(u, t, false, true, true, true, false);
    assert_eq!(tile_search(&sim), loc::VALID, "player 1's footprint");
    assert_eq!(
        sim.invalid_loc(u, t, false, true, false, true, false),
        loc::ON_BUILDING,
        "without `param_5` every footprint refuses"
    );
    sim.buildings[b].owner = 0;
    assert_eq!(tile_search(&sim), loc::ON_BUILDING, "its own refuses");
    sim.buildings[b].alive = false;
    assert_eq!(
        tile_search(&sim),
        loc::VALID,
        "nothing found: `find_who` −1 is not the walker's"
    );
    let (sim, u, _, t) = ring_tile(false);
    assert_eq!(
        sim.invalid_loc(u, t, false, true, true, true, false),
        loc::ON_BUILDING,
        "an unarmed walker is refused whoever owns it"
    );
}

/// **`invalid_loc`'s cell arm** (item 776, `00607e6f`–`00607ead`): on
/// the land domain, a tile whose **cell** carries mountain, forest or
/// `0x40` (`WData.flags & 0x70`) refuses with 2 when the caller passes
/// both `param_3` and `param_6` — which is `valid_wcoord`'s probe, and
/// no other caller's unless a transport-flagged path top forces
/// `param_6`. A forest-walker is let through a forest cell. The tile
/// itself is plain ground here, so the tile arm below passes it: run226's
/// `1/72` was planned through cell (56, 18), forest-flagged, where the
/// original's world search refused it (`docs/PATHFINDER.md` §27).
#[test]
fn a_world_probe_refuses_a_forest_cell_on_plain_ground() {
    let mut sim = lake_sim();
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let c = Cell::new(7, 7);
    let mut d = sim.world.cell_data(c);
    d.flags |= crate::world::cell::FOREST;
    sim.world.set_cell_data(c, d);
    let t = Pos::new(7 * TILES_PER_CELL + 1, 7 * TILES_PER_CELL + 2);
    assert_eq!(sim.world.tile_mask(t) & tile::SURFACE, 0, "plain ground");
    assert_eq!(
        sim.invalid_loc(u, t, true, false, false, true, false),
        loc::TERRAIN,
        "`valid_wcoord`'s flags read the cell"
    );
    assert_eq!(
        sim.invalid_loc(u, t, false, false, false, true, false),
        loc::VALID,
        "without `param_3` the cell is not read"
    );
    assert_eq!(
        sim.invalid_loc(u, t, true, false, false, false, false),
        loc::VALID,
        "without `param_6` the cell is not read"
    );
    let mut d = sim.world.cell_data(c);
    d.flags = (d.flags & !crate::world::cell::FOREST) | crate::world::cell::MOUNTAIN;
    sim.world.set_cell_data(c, d);
    assert_eq!(
        sim.invalid_loc(u, t, true, false, false, true, false),
        loc::TERRAIN,
        "a mountain cell refuses the same way"
    );
}

/// **`find_wpath`'s human variant** (item 676, `00689109`–`006892e4`):
/// a human's goal in another region, or unseen, is **popped** for the
/// entry under it, down to the final one, and the search runs to it;
/// a computer's is dragged back toward the start until its region
/// matches, and returns without a search. run180's chariot, a cell
/// short of the sand, is the first case. Made to fail first with the
/// variant switched off, when the human's goal stopped on the shore.
#[test]
fn a_human_s_world_plan_pops_past_water_to_its_goal() {
    let run = |human: bool| {
        let mut sim = lake_sim();
        sim.cities.nation[0].human = human;
        let u = walker(&mut sim, Pos::new(0x180, 4 * 0x300 + 0x180));
        let goal = Pos::new(8 * 0x300 + 0x180, 4 * 0x300 + 0x180);
        push_goal(&mut sim, u, goal);
        // A world waypoint past the lake, then one in it, on top.
        for x in [6, 3] {
            sim.units[u].path.push(PathData {
                to: Pos::new(x * 0x300 + 0x180, 4 * 0x300 + 0x180),
                tolerance: 0x180,
                flags: 0,
            });
        }
        let r = sim.find_wpath(u);
        (r, sim.units[u].path.clone(), goal)
    };
    let (r, path, goal) = run(true);
    // Both waypoints popped, the goal kept, and a chain of cell centres
    // searched to it on top: more than the three entries it was given.
    assert!(r > 3 && path.len() > 3, "the human's search ran: {path:?}");
    assert_eq!(
        path.first().map(|p| p.to),
        Some(goal),
        "the human's plan ends on the order's own goal"
    );
    let (r, path, _) = run(false);
    let top = path.last().expect("a goal").to;
    assert_eq!(r, 3, "the computer's walk returns without a search");
    assert_eq!(top.cell().x, 1, "pulled back onto the shore: {top:?}");
}

/// §5's halfland multiplier — `base ×= 3` on a cell whose `flags` carry
/// `0x100`, and **skipped** whenever the step crossed a shoreline
/// (audit V20). Nothing on disk exercises it: the routes the captures
/// take either miss halfland cells or cross into them.
#[test]
fn a_halfland_cell_triples_the_base_unless_the_step_crossed_the_shore() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let from = Pos::new(0x180, 0x180);
    let to = Pos::new(0x180 + 0x300, 0x180);
    let m = Modes::default();
    assert_eq!(sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0, 32);

    let mut d = sim.world.cell_data(to.cell());
    d.flags |= crate::world::cell::HALFLAND;
    sim.world.set_cell_data(to.cell(), d);
    assert_eq!(
        sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0,
        96,
        "0x300 × 32 / 256"
    );
    // The multiplier is the *base*'s, so the diagonal's +8 sits outside
    // it, and the tile grid never sees it at all.
    let se = Pos::new(to.x, to.y + 0x300);
    let mut d2 = sim.world.cell_data(se.cell());
    d2.flags |= crate::world::cell::HALFLAND;
    sim.world.set_cell_data(se.cell(), d2);
    assert_eq!(
        sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 2, 0, 1).0,
        104
    );
}

/// §16 — `calc_cost`'s water row is **`WorldData::is_ocean`**, the
/// `WData` test, and not "the cell's region is a sea region". The two
/// answers part on a `HALFLAND` cell, which is where a boat actually
/// sails: the coastal cells of East Indies' channel were costing this
/// crate the `avoid_land` 200 the original never charges.
#[test]
fn a_halfland_cell_in_a_sea_region_is_land_to_the_cost_function() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let from = Pos::new(0x180, 0x180);
    let to = Pos::new(0x180 + 0x300, 0x180);
    let m = Modes::default();
    // A cell the region layer calls sea and `WData` calls land: ocean's
    // own `land` kind, and `HALFLAND` set.
    let sea = sim.world.add_region(Terrain::Sea);
    sim.world.set_region(to.cell(), sea);
    let mut d = sim.world.cell_data(to.cell());
    d.land = 1;
    d.flags |= crate::world::cell::HALFLAND;
    sim.world.set_cell_data(to.cell(), d);
    assert_eq!(sim.world.region_of(to.cell()), Some(sea));
    assert!(!sim.world.is_ocean(to.cell()), "HALFLAND is never ocean");
    // The halfland base is 0x300 × 32 / 256 = 96. `avoid_land` charges
    // its 200 on top; `avoid_sea` charges nothing, because the cell is
    // not ocean.
    assert_eq!(
        sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 1, 0).0,
        296
    );
    assert_eq!(sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0, 96);
}

/// §16 — the world grid's same-region test reads `WData.region` raw.
/// A boat standing on a `HALFLAND` cell is in the **land** region by
/// that field and in the sea region by `get_tregion`, so the two rules
/// disagree about whether it is crossing: `get_tregion` says "same
/// region, and the start is not ocean" and hands the boat
/// `avoid_sea = 1`, which prices every cell of its own sea at 232.
#[test]
fn a_boat_leaving_a_halfland_cell_is_crossing_regions_on_the_world_grid() {
    let mut sim = flat_sim(12);
    let sea = sim.world.add_region(Terrain::Sea);
    for x in 5..12 {
        for y in 0..12 {
            let c = Cell::new(x, y);
            sim.world.set_region(c, sea);
            let mut d = sim.world.cell_data(c);
            d.land = 1;
            sim.world.set_cell_data(c, d);
            for ty in 0..TILES_PER_CELL {
                for tx in 0..TILES_PER_CELL {
                    let t = Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty);
                    sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
                }
            }
        }
    }
    // The berth: a coastal cell whose `region` is the land it belongs
    // to and whose `region2` is the water beside it — and whose tiles
    // are water, so `get_tregion` answers the sea.
    let berth = Cell::new(4, 4);
    let mut d = sim.world.cell_data(berth);
    d.flags |= crate::world::cell::HALFLAND;
    d.region2 = Some(sea);
    sim.world.set_cell_data(berth, d);
    for ty in 0..TILES_PER_CELL {
        for tx in 0..TILES_PER_CELL {
            let t = Pos::new(berth.x * TILES_PER_CELL + tx, berth.y * TILES_PER_CELL + ty);
            sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
        }
    }
    assert_ne!(
        sim.world.region_of(berth),
        sim.world
            .tregion_alt(Pos::new(berth.x * TILES_PER_CELL, berth.y * TILES_PER_CELL)),
        "the berth is the cell the two rules disagree about"
    );

    let u = walker(
        &mut sim,
        Pos::new(berth.x * 0x300 + 0x180, berth.y * 0x300 + 0x180),
    );
    sim.units[u].kind.domain = crate::attrition::Domain::Sea;
    push_goal(&mut sim, u, Pos::new(10 * 0x300 + 0x180, 4 * 0x300 + 0x180));
    sim.trace.trace_costs = true;
    assert!(sim.find_wpath(u) >= 1, "the boat found its way out");

    // Every step the search priced into open water cost the plain 32
    // (or 40 on a diagonal). With `avoid_sea` derived they would all
    // carry 200 more.
    let open: Vec<i32> = sim
        .trace
        .cost_marks
        .iter()
        .filter(|c| {
            let cell = Pos::new(c.to.0, c.to.1).cell();
            cell.x >= 5 && sim.world.is_ocean(cell)
        })
        .map(|c| c.cost)
        .collect();
    assert!(!open.is_empty(), "the search priced some open water");
    assert!(
        open.iter().all(|&c| c == 32 || c == 40),
        "open water is free to a boat that is crossing regions: {open:?}"
    );
}

/// **A sea unit's goal that its region accepts but its tile refuses is
/// stepped back** (`find_wpath@00688fc0:00689375`, item 1868): East
/// Indies' Fishermen `1/122` was sent at (31896, 44952), a shore tile of
/// its own sea, and the original planned to (32277, 44997) — one 0x180
/// step back toward the boat — where this crate planned to the goal.
#[test]
fn a_sea_goal_whose_tile_refuses_the_boat_is_stepped_back_over_water() {
    let mut sim = flat_sim(12);
    let sea = sim.world.add_region(Terrain::Sea);
    for x in 0..12 {
        for y in 0..12 {
            let c = Cell::new(x, y);
            sim.world.set_region(c, sea);
            let mut d = sim.world.cell_data(c);
            d.land = 1;
            sim.world.set_cell_data(c, d);
            for ty in 0..TILES_PER_CELL {
                for tx in 0..TILES_PER_CELL {
                    let t = Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty);
                    sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
                }
            }
        }
    }
    let goal = Pos::new(0x300 + 0x18, 6 * 0x300 + 0x18);
    let shore = goal.tile();
    sim.world
        .set_tile_mask(shore, tile::SURFACE_OCEAN | tile::BLOCKED);
    let u = walker(&mut sim, Pos::new(10 * 0x300 + 0x180, 6 * 0x300 + 0x180));
    sim.units[u].kind.domain = crate::attrition::Domain::Sea;
    assert_ne!(
        sim.invalid_loc(u, shore, false, true, true, true, false),
        0,
        "the shore tile refuses the boat"
    );
    assert_eq!(
        sim.world.tregion_alt(shore),
        sim.world.tregion_alt(sim.units[u].pos.tile()),
        "and its region is the boat's, so the region test alone would stop"
    );
    push_goal(&mut sim, u, goal);
    assert!(sim.find_wpath(u) >= 1);
    let end = sim.units[u].path[0].to;
    assert_ne!(end, goal, "the goal was stepped back");
    assert_eq!(
        sim.invalid_loc(u, end.tile(), false, true, true, true, false),
        0,
        "to a tile the boat may stand on: {end:?}"
    );
}

/// §5.1's probe table: a diagonal is refused when **either** of its two
/// probes is blocked, a cardinal only when **all four** of its are —
/// and the whole block is skipped on ground that costs nothing, which
/// is why a flat world never sees it.
#[test]
fn a_diagonal_is_refused_past_one_corner_and_a_cardinal_needs_all_four() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let from = Pos::new(3 * 0x300 + 0x180, 3 * 0x300 + 0x180);
    let (cx, cy) = (from.tile().x, from.tile().y);
    // Give the destination cells a cost, or §5.1 never runs.
    for (dx, dy) in [(1, 1), (1, 0)] {
        let c = crate::world::Cell::new(from.cell().x + dx, from.cell().y + dy);
        let mut d = sim.world.cell_data(c);
        d.blocked = 1;
        sim.world.set_cell_data(c, d);
    }
    let m = Modes::default();
    let se = Pos::new(from.x + 0x300, from.y + 0x300);
    let east = Pos::new(from.x + 0x300, from.y);
    let cost =
        |sim: &Sim, to: Pos, dir: usize| sim.calc_cost(u, &m, from, to, dir, STEP_WORLD, 2, 0, 1).0;
    assert_ne!(cost(&sim, se, 5), REFUSED, "clear to begin with");

    // SE probes `(cx+1, cy+1)` then `(cx+2, cy+2)`; either is enough.
    sim.world
        .set_tile_bits(Pos::new(cx + 2, cy + 2), tile::BLOCKED);
    assert_eq!(cost(&sim, se, 5), REFUSED, "the far corner alone");
    sim.world
        .clear_tile_bits(Pos::new(cx + 2, cy + 2), tile::BLOCKED);
    sim.world
        .set_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);
    assert_eq!(cost(&sim, se, 5), REFUSED, "the near corner alone");
    sim.world
        .clear_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);

    // East probes the whole column `(cx+2, cy-2 ..= cy+1)` and needs
    // every one of them.
    let col = [
        Pos::new(cx + 2, cy - 2),
        Pos::new(cx + 2, cy - 1),
        Pos::new(cx + 2, cy),
        Pos::new(cx + 2, cy + 1),
    ];
    for t in &col[..3] {
        sim.world.set_tile_bits(*t, tile::BLOCKED);
    }
    assert_ne!(cost(&sim, east, 4), REFUSED, "three of four is a gap");
    sim.world.set_tile_bits(col[3], tile::BLOCKED);
    assert_eq!(cost(&sim, east, 4), REFUSED, "the edge is built over");

    // And a fully forested tile is not a wall to a forest-walker.
    for t in &col {
        sim.world.set_tile_bits(*t, tile::SURFACE_FOREST);
    }
    let iroquois = Modes {
        iroquois: true,
        ..Modes::default()
    };
    assert_ne!(
        sim.calc_cost(u, &iroquois, from, east, 4, STEP_WORLD, 2, 0, 1)
            .0,
        REFUSED,
        "the Iroquois walk through the trees"
    );
}

/// The depth gate: §5.1 runs only for the first nine steps of a search
/// (`depth < 10`), so a corner ten steps out is not probed at all.
#[test]
fn the_corner_probes_stop_after_nine_steps() {
    let mut sim = flat_sim(10);
    let u = walker(&mut sim, Pos::new(0x180, 0x180));
    let from = Pos::new(3 * 0x300 + 0x180, 3 * 0x300 + 0x180);
    let (cx, cy) = (from.tile().x, from.tile().y);
    let se = Pos::new(from.x + 0x300, from.y + 0x300);
    let mut d = sim.world.cell_data(se.cell());
    d.blocked = 1;
    sim.world.set_cell_data(se.cell(), d);
    sim.world
        .set_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);
    let m = Modes::default();
    assert_eq!(
        sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 9, 0, 1).0,
        REFUSED
    );
    assert_ne!(
        sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 10, 0, 1).0,
        REFUSED,
        "depth 10 is past the gate"
    );
}

/// **The original's own answers, under an emulator.** `tools/emu/callfn.py`
/// enters `vector_dist@0046cff0` and `get_estimate@00688310` in the
/// executable's image under unicorn — no game, no capture — over a seeded
/// sweep, and prints `<name> <args…> -> <eax>`; every row is asserted
/// against this crate here. The table is `$RON_EMU_TABLE`, or is generated
/// from the install on the spot; a machine with neither says so.
///
/// The sweep's first catch, 2026-09-01: `lo * lo` in `world::vector_dist`
/// overflowed `i32` above `lo = 46340`, where the original squares in
/// `unsigned` — 59999² is 3,599,880,001.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(table) = emu_table() else { return };
    let mut rows = 0;
    for line in table.lines() {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row");
        let f: Vec<&str> = lhs.split(' ').collect();
        let a: Vec<i32> = f[1..].iter().map(|s| s.parse().unwrap()).collect();
        let want: i32 = rhs.parse().unwrap();
        let got = match f[0] {
            "vector_dist" => crate::world::vector_dist(a[0], a[1]),
            "get_estimate" => Sim::estimate(a[0] - a[2], a[1] - a[3], a[4]),
            other => panic!("unknown function {other}"),
        };
        assert_eq!(got, want, "{line}");
        rows += 1;
    }
    assert!(rows > 3000, "{rows} rows");
}

/// The emulator's table: `$RON_EMU_TABLE` as written, else the sweep run
/// against the install (`testenv::install_root`) through `uv`.
fn emu_table() -> Option<String> {
    if let Ok(p) = std::env::var("RON_EMU_TABLE") {
        return Some(std::fs::read_to_string(p).expect("RON_EMU_TABLE"));
    }
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: set RON_EMU_TABLE or RON_INSTALL");
        return None;
    };
    // On `tools/emu`'s shared environment, not a per-path one (item 3354).
    let out =
        crate::sweep::uv_command(std::path::Path::new(&format!("{root}/tools/emu/callfn.py")))
            .arg(format!("{install}/riseofnations.exe"))
            .arg("sweep")
            .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).unwrap()),
        Ok(o) => panic!("callfn.py failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
    }
}
