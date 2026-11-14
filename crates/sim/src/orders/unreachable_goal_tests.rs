use super::*;
use crate::world::{Cell, Terrain, World, tile};
use crate::{Tuning, Unit, UnitType};

/// **An unreachable goal kills its own move and nothing under it**
/// (`do_move@005f7b30`, `5f8990`–`5f8997` → `5f82a3`: one
/// `kill_current_order(0)` and `return 1`; item 1614, `docs/AI.md`
/// §172). A walker sent into the sea with a guard queued behind keeps
/// the guard — the coverage pair's Transport Freighter `1/152` on block
/// 2995. Made to fail by killing twice, as this crate did.
#[test]
fn an_unreachable_goal_kills_the_move_and_keeps_the_guard() {
    let mut w = World::new(16, 16);
    // A channel the whole height of the map: no detour round it.
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 15));
    w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(9, 15));
    w.fill_region(Terrain::Land, Cell::new(10, 0), Cell::new(15, 15));
    for tx in 32..40 {
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
    let mut walker = Unit::new(1, 0, Pos::new(7 * 768 + 384, 5 * 768 + 384), 50);
    walker.ty = Some(ty);
    walker.on_map = true;
    let u = s.add_unit(walker);
    s.units[u].movement.speed = 30;
    // The goal is in the channel and the walk is planned across it:
    // the stack's top is the far shore, and the line is unverified.
    let sea = Pos::new(9 * 768 + 384, 5 * 768 + 384);
    let shore = Pos::new(10 * 768 + 384, 5 * 768 + 384);
    s.order_move(u, sea);
    if let Some(o) = s.units[u].orders.front_mut() {
        o.flags |= flag::PATHED;
        if let Body::Move(m) = &mut o.body {
            m.dest = sea;
            m.has_waypoint = true;
            m.waypoint = shore;
        }
    }
    s.units[u].path = vec![
        PathData {
            to: sea,
            tolerance: 0,
            flags: path_flag::FINAL,
        },
        PathData {
            to: shore,
            tolerance: 0,
            flags: 0,
        },
    ]
    .into();
    s.units[u].line_ok = false;
    let post = s.units[u].pos;
    s.units[u].orders.push_back(Order {
        flags: 0,
        body: Body::Guard(GuardOrder {
            target: u,
            dx: -1,
            dy: -1,
            guard: post,
            idle: 0,
            retry: 0,
        }),
    });
    for _ in 0..4 {
        if !matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::Move(_))
        ) {
            break;
        }
        s.tick();
    }
    assert!(
        matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::Guard(_))
        ),
        "the guard stands under the dead move: {:?}",
        s.units[u].orders
    );
}
