use super::*;
use crate::world::{Cell, Terrain, World};
use crate::{Tuning, Unit};

/// `find_wpath` as run1608's packet answered it for `1/84`: the goal
/// popped, no path, −1.
fn wpath_refuses(s: &mut Sim, u: usize) -> i32 {
    s.units[u].path.pop();
    -1
}

/// `find_tpath` as the packet answered it: 0, the stack untouched.
fn tpath_refuses(_: &mut Sim, _: usize) -> i32 {
    0
}

/// **A planner that fails leaves the goal on the stack's top, not the
/// entry under it** (`do_move@005f7b30`, `5f83f0`–`5f84a9`: after each of
/// `find_wpath` and `find_tpath` returns below 1, the goal is pushed again
/// unless the top is already it — the same `to`, `flags == 1`, `tolerance
/// == 0` — not only on an empty stack; item 4159, `docs/LEADERS.md`
/// §3.6). run1075's Attack Submarine `1/84` on trace frame 2553, run under
/// `step4` on run1608's packet: its chase move to (33144, 44184) is pushed
/// over the `ATTACK_TO`'s final (43042, 44265), `find_wpath` returns −1
/// and pops it, the goal goes back on, `find_tpath` returns 0, and the
/// stack is [(43042, 44265), (33144, 44184)] when `find_path` refuses the
/// line and the grid roll (8116) is paid. Made to fail by re-pushing only
/// on an empty stack, as this crate did: the stale final 160 away became
/// the waypoint, a straight line, and no roll.
#[test]
fn a_failed_plan_leaves_the_goal_over_the_stale_final_under_it() {
    let mut w = World::new(64, 64);
    w.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(63, 63));
    let mut s = Sim::new(Tuning::RON, w, 2);
    let u = s.add_unit(Unit::new(1, 84, Pos::new(43176, 44328), 50));
    let stale = PathData {
        to: Pos::new(43042, 44265),
        tolerance: 0,
        flags: path_flag::FINAL,
    };
    let goal = PathData {
        to: Pos::new(33144, 44184),
        tolerance: 0,
        flags: path_flag::FINAL,
    };
    s.units[u].path = vec![stale, goal].into();
    s.plan_unpathed(u, goal, wpath_refuses, tpath_refuses);
    let stack: Vec<PathData> = s.units[u].path.iter().copied().collect();
    assert_eq!(stack, vec![stale, goal], "the goal is the top the walk takes");
    // An empty stack under the goal is the case this crate already had.
    s.units[u].path = vec![goal].into();
    s.plan_unpathed(u, goal, wpath_refuses, tpath_refuses);
    let stack: Vec<PathData> = s.units[u].path.iter().copied().collect();
    assert_eq!(stack, vec![goal]);
}
