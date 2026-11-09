use super::*;
use crate::group::Group;
use crate::movement::Angle;
use crate::orders::QueuePos;
use crate::world::{Cell, Terrain, World};
use crate::{Player, Sim, Tuning, Unit, UnitType, combat};

/// A flat land world, player 0 human and player 1 a computer.
fn flat(cells: i32) -> Sim {
    let mut world = World::new(cells, cells);
    world.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(cells - 1, cells - 1),
    );
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.nation[0].human = true;
    s.nation[1].human = false;
    s
}

fn fighter(s: &mut Sim) -> usize {
    let mut t = UnitType {
        hits: 100,
        combat: combat::Profile {
            attack: 15,
            uber_size: 1,
            // `unitrules.xml`'s own `X_SPACING`/`Y_SPACING`, times the
            // engine's 12 — without them every slot lands on the
            // anchor and there is no translation to measure.
            x_spacing: 0xc0,
            y_spacing: 0x180,
            ..combat::Profile::default()
        },
        ..UnitType::default()
    };
    t.cols.role |= crate::ai_load::role::ROLE_MILITARY;
    s.add_unit_type(t)
}

fn spawn(s: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
    let index = i16::try_from(s.units.len()).unwrap();
    let hits = s.unit_types[ty].hits;
    let mut u = Unit::new(who, index, p, hits);
    u.ty = Some(ty);
    u.on_map = true;
    u.movement.speed = 25;
    s.add_unit(u)
}

fn stack(who: Player, list: &[usize]) -> Group {
    Group {
        who,
        army: None,
        pushed: None,
        list: list.to_vec(),
    }
}

fn chain(s: &Sim, u: usize) -> Vec<(i32, i32, i32, u8)> {
    s.units[u]
        .path
        .iter()
        .map(|p| (p.to.x, p.to.y, p.tolerance, p.flags))
        .collect()
}

/// **A slot past the map's edge is planned clamped** (`Group::
/// action_move_near@00704990:521`–`536`, item 3711, `docs/AI.md`
/// §363.1): the order loop's clamp is a store into the slot table
/// itself, so the plan below it — the leader's base, a member's
/// translation, `group_plan_each`'s goal — never sees the raw slot.
/// A move aimed at the map's corner puts a member's slot off the map;
/// its stack's goal is inside. Greek East Indies' Heavy Fire Raft
/// `1/70` on 5626 carried x 46259 on a map 46080 wide, and `find_path`
/// pulled it back onto its own square.
///
/// Made to fail by dropping the write-back (`slots.to[i] = slot`) in
/// `group_action_move_to`: the leader's off-map base refuses the
/// search, and every member takes its raw slot.
#[test]
fn a_slot_past_the_map_s_edge_is_planned_clamped() {
    let mut s = flat(24);
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x480, 0x480));
    let b = spawn(&mut s, 1, t, Pos::new(0x480 + 0xc0, 0x480));
    let mut g = stack(1, &[a, b]);
    assert!(s.push_group(&mut g, true));
    let edge = 24 * 0x300;
    s.group_action_move_to(
        &g,
        Pos::new(edge - 10, edge - 10),
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    for u in [a, b] {
        let goal = s.units[u].path.first().copied().expect("a planned goal");
        assert!(
            (0..edge).contains(&goal.to.x) && (0..edge).contains(&goal.to.y),
            "unit {u}'s goal {:?} is inside the world; chain {:?}",
            goal.to,
            chain(&s, u)
        );
    }
}

/// **A copied move replays to its click, not its slot**
/// (`Group::finish_insert@0070e620` cases 1–4, `docs/GOLDEN.md` §24).
/// A player's explore to an unaligned point stores the click as its
/// `orig`; a group `QUEUE_FIRST` — the goody box's leg — copies it
/// aside and re-issues it `QUEUE_LAST` to that click, so the replay
/// carries the same `orig` and plans to it again. run219's chariot
/// stands on (2400, 17280), its click, and not on the snap.
///
/// Made to fail by dropping `.or(m.orig)` from `finish_insert`, which
/// replays to the snapped `dest` and hands the copy that as its `orig`.
#[test]
fn a_copied_plain_move_replays_to_its_click_not_its_slot() {
    let mut s = flat(24);
    let t = fighter(&mut s);
    let a = spawn(&mut s, 0, t, Pos::new(0x480, 0x480));
    let mut g = stack(0, &[a]);
    assert!(s.push_group(&mut g, true));
    let click = Pos::new(0x480 + 10, 0x480 + 8 * 0x300 + 10);
    s.group_action_move_to(
        &g,
        click,
        QueuePos::New,
        false,
        Angle(0),
        MoveKind::ExploreTo,
        true,
    );
    let first = match s.units[a].orders.front().map(|o| o.body) {
        Some(crate::orders::Body::Move(m)) => m,
        other => panic!("an explore, not {other:?}"),
    };
    assert_eq!(first.orig, Some(click), "the click is the explore's orig");
    assert_ne!(first.dest, click, "and the order's point is its snap");
    // The box leg: a one-member group's QUEUE_FIRST, no action bit.
    s.group_action_move_to(
        &g,
        Pos::new(0x480 + 3 * 0x300, 0x480 + 3 * 0x300),
        QueuePos::First,
        false,
        Angle(0),
        MoveKind::ExploreTo,
        false,
    );
    assert_eq!(s.units[a].orders.len(), 2, "the leg and the replay");
    let replay = match s.units[a].orders.back().map(|o| (o.flags, o.body)) {
        Some((f, crate::orders::Body::Move(m))) => {
            assert_ne!(
                f & crate::orders::flag::ACTION,
                0,
                "the replay is an action"
            );
            m
        }
        other => panic!("a replayed explore, not {other:?}"),
    };
    assert_eq!(replay.kind, MoveKind::ExploreTo);
    assert_eq!(replay.dest, first.dest, "the same snapped point");
    assert_eq!(
        replay.orig,
        Some(click),
        "replayed to the click, not the slot"
    );
}

/// §6.7's centre: **one** search, and the followers get the leader's
/// answer rather than one of their own.
///
/// Two members eight cells from the goal. The leader carries the whole
/// planned chain; the follower — in the formation, because this is a
/// plain `MOVE_TO` at form 0 — carries the **final waypoint only**, at
/// its own raw slot. Made to fail by pushing every waypoint to every
/// member (the follower then holds the leader's chain, translated), and
/// again by giving each member its own `find_wpath` (the goal is then
/// the same but the entries above it are not the leader's).
#[test]
fn a_group_move_plans_one_path_and_the_follower_takes_only_the_goal() {
    let mut s = flat(24);
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x480, 0x480));
    let b = spawn(&mut s, 1, t, Pos::new(0x480 + 0x60, 0x480));
    let g = stack(1, &[a, b]);
    s.group_action_move_to(
        &g,
        Pos::new(0x480 + 8 * 0x300, 0x480),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let leader = s.group_find_leader(&g).expect("a leader");
    assert_eq!(leader, a, "the first of two equal categories leads");
    let follower = b;
    let lead = chain(&s, leader);
    let foll = chain(&s, follower);
    assert!(lead.len() > 2, "the leader planned nothing: {lead:?}");
    assert_eq!(
        foll.len(),
        1,
        "a group-move follower takes the goal alone: {foll:?}"
    );
    assert_eq!(foll[0].3, path_flag::FINAL);
    // Both goals are raw slots off the same layout: two members, two
    // slots, the same tolerance and the same flag.
    assert_ne!((foll[0].0, foll[0].1), (lead[0].0, lead[0].1));
    assert_eq!((foll[0].2, foll[0].3), (lead[0].2, lead[0].3));
    // And each is the member's **raw** slot, not the order's snapped
    // `dest` — which is the whole of item 31's goal half, driven from
    // the simulation's own side rather than replayed out of a dump.
    for u in [leader, follower] {
        let dest = match s.current_order(u).expect("a move order").body {
            crate::orders::Body::Move(mo) => mo.dest,
            _ => unreachable!(),
        };
        let bottom = s.units[u].path[0].to;
        assert_ne!(bottom, dest, "{u}: the goal is the order's dest");
        assert_eq!(
            (
                dest.x.div_euclid(0x30) * 0x30 + 0x18,
                dest.y.div_euclid(0x30) * 0x30 + 0x18
            ),
            (dest.x, dest.y),
            "{u}: and the order's dest is that raw slot, snapped"
        );
    }
}

/// The `< 0x900` short-circuit (`70616d`): a move whose goal is already
/// close plans no route, and the member's stack is its slot and nothing
/// else.
///
/// The distance has to be chosen against the *other* short-circuit or
/// the test proves nothing: `find_wpath`'s own near test exits on a
/// cell-Manhattan under 3 (`docs/PATHFINDER.md` §3), so a goal two
/// cells due east is one entry either way. `(1500, 1500)` is four cells
/// away by that measure and `vector_dist 2250` by this one — inside
/// `0x900` and outside the near test. Made to fail by removing the
/// `0x900` test, which turns the same move into a planned chain.
#[test]
fn a_short_group_move_plans_no_route_at_all() {
    let mut s = flat(24);
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x180, 0x180));
    let g = stack(1, &[a]);
    let goal = Pos::new(0x180 + 1500, 0x180 + 1500);
    assert!(vector_dist(1500, 1500) < NO_PLAN_WITHIN);
    assert_eq!(vector_dist(1500, 1500), 2250);
    assert_eq!(
        goal.cell().x + goal.cell().y,
        4,
        "and outside `find_wpath`'s own near test"
    );
    s.group_action_move_to(
        &g,
        goal,
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::MoveTo,
        false,
    );
    let c = chain(&s, a);
    assert_eq!(c.len(), 1, "{c:?}");
    assert_eq!(c[0].3, path_flag::FINAL);
    assert_eq!((c[0].0, c[0].1), (goal.x, goal.y), "the raw slot");
}

/// The follower cutoff (`706845`): a member that is **not** in the
/// formation walks the leader's intermediate legs too, but only while
/// it is inside `0x600` of the leader.
///
/// `EXPLORE_TO` is not `MOVE_TO`, so no member of this group is on a
/// group move at all. The near follower ends up with the leader's whole
/// chain; the far one with the goal alone. Made to fail by dropping the
/// distance test, which hands the far member the chain as well.
#[test]
fn a_follower_out_of_formation_takes_the_legs_only_while_it_is_near() {
    let mut s = flat(24);
    let t = fighter(&mut s);
    let a = spawn(&mut s, 1, t, Pos::new(0x480, 0x480));
    let near = spawn(&mut s, 1, t, Pos::new(0x480 + 0x300, 0x480));
    let far = spawn(&mut s, 1, t, Pos::new(0x480 + 4 * 0x300, 0x480));
    let g = stack(1, &[a, near, far]);
    s.group_action_move_to(
        &g,
        Pos::new(0x480, 0x480 + 8 * 0x300),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::ExploreTo,
        false,
    );
    let lead = chain(&s, a);
    assert!(lead.len() > 2, "the leader planned nothing: {lead:?}");
    assert_eq!(chain(&s, near).len(), lead.len(), "the near follower");
    assert_eq!(chain(&s, far).len(), 1, "the far follower");
}

/// The tail invert asks none of the questions the waypoint loop asked
/// (`706909`–`7069a8`): a member the move left entirely alone still has
/// its own stack turned over.
///
/// The fixture is §6.5's: an AI army that is not hurrying, holding a
/// siege unit already shooting. The move skips it — it keeps its
/// `ATTACK` order — and its path stack, which the move never wrote to,
/// comes out reversed. Made to fail by gating the invert on the same
/// `Member::Move` the rest of the section uses, which leaves the stack
/// as it was.
#[test]
fn the_tail_invert_turns_over_a_stack_the_move_never_wrote_to() {
    use crate::orders::index;
    let mut s = flat(24);
    let siege = {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 40,
                uber_size: 1,
                siege: true,
                x_spacing: 0xc0,
                y_spacing: 0x180,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        t.cols.role |= crate::ai_load::role::ROLE_MILITARY;
        s.add_unit_type(t)
    };
    let slot = s.init_army(1, None);
    let m = spawn(&mut s, 1, siege, Pos::new(0x480, 0x480));
    let escort = spawn(&mut s, 1, siege, Pos::new(0x480 + 0x60, 0x480));
    let foe = spawn(&mut s, 0, siege, Pos::new(0x4e0, 0x480));
    s.army_add_unit(1, slot, m);
    s.army_add_unit(1, slot, escort);
    s.add_attack_order(m, crate::combat::Obj::Unit(foe), QueuePos::New, true, true);
    assert_eq!(s.order_type(m), index::ATTACK);
    // A stack of its own, in a known order.
    let marks = [Pos::new(0x900, 0x900), Pos::new(0xc00, 0xc00)];
    for p in marks {
        s.units[m].path.push(PathData {
            to: p,
            tolerance: 0,
            flags: 0,
        });
    }
    let g = s.army_group(1, slot);
    s.group_action_move_to(
        &g,
        Pos::new(0x480 + 8 * 0x300, 0x480),
        QueuePos::New,
        true,
        Angle(0),
        MoveKind::AttackTo,
        true,
    );
    assert_eq!(
        s.order_type(m),
        index::ATTACK,
        "the shooting siege keeps its order"
    );
    let c: Vec<Pos> = s.units[m].path.iter().map(|p| p.to).collect();
    assert_eq!(
        c,
        vec![marks[1], marks[0]],
        "the skipped member's own stack is inverted with everyone else's"
    );
}
