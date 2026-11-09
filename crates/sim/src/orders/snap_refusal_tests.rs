use super::*;
use crate::world::{Cell, Terrain, World, tile};
use crate::{Tuning, Unit, UnitType};

/// **The snap arm consumes its waypoint on a refused tile**
/// (`move_step@005faf30`, `005fb44a`: `jne 005fb48e`, past the walk and
/// `set_new_location`, onto `dest = 0` and the pop; `docs/AI.md` §229).
/// A walker one snap short of a sidestep across the shore stays where
/// it is, drops the sidestep, keeps its verified line and spends no
/// grid draw — run866's Heavy Tank `1/11` on block 5315. Made to fail
/// by the partial step's refusal on both arms, as this crate had it.
#[test]
fn a_snap_onto_a_refused_tile_pops_the_waypoint_where_it_stands() {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 15));
    w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(15, 15));
    for tx in 32..64 {
        for ty in 0..64 {
            w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
        }
    }
    let mut s = Sim::new(Tuning::RON, w, 2);
    let ty = s.add_unit_type(UnitType {
        hits: 50,
        moves: 30,
        ..UnitType::default()
    });
    // The last land tile's east edge, and a sidestep 26 east of it in
    // the first sea tile: inside one step, so `move_step` snaps.
    let start = Pos::new(8 * 768 - 16, 5 * 768 + 384);
    let side = Pos::new(8 * 768 + 10, 5 * 768 + 384);
    let goal = Pos::new(2 * 768 + 384, 5 * 768 + 384);
    let mut walker = Unit::new(1, 0, start, 50);
    walker.ty = Some(ty);
    walker.on_map = true;
    let u = s.add_unit(walker);
    s.units[u].movement.speed = 30;
    // Already facing it, so no turn is owed.
    let east = crate::movement::find_angle(side.x - start.x, side.y - start.y);
    s.units[u].movement.facing = east;
    s.units[u].movement.heading = east;
    s.order_move(u, goal);
    if let Some(o) = s.units[u].orders.front_mut() {
        o.flags |= flag::PATHED;
        if let Body::Move(m) = &mut o.body {
            m.dest = goal;
            m.has_waypoint = true;
            m.waypoint = side;
        }
    }
    s.units[u].path = vec![
        PathData {
            to: goal,
            tolerance: 0,
            flags: path_flag::FINAL,
        },
        PathData {
            to: side,
            tolerance: 0,
            flags: path_flag::SIDESTEP,
        },
    ];
    s.units[u].line_ok = true;
    s.trace.trace_phases = true;
    let mut grid = false;
    for _ in 0..16 {
        s.trace.phase_marks.clear();
        s.tick();
        grid |= s.trace.phase_marks.iter().any(|(l, _)| l == SITE_MOVE_GRID);
        if s.units[u].path.len() < 2 {
            break;
        }
    }
    assert_eq!(s.units[u].path.len(), 1, "the sidestep is popped");
    assert_eq!(s.units[u].pos, start, "where it stands");
    assert!(s.units[u].line_ok, "the line stays verified");
    assert!(!grid, "and no re-plan spent the grid draw");
}
