//! The recorded order stream, into the simulation.
//!
//! A recorded game is the only source of what the players *did*
//! (`docs/RECGAME.md`), and until it is fed in, the harness's simulation
//! stands its roster up from the initial dump and then runs on the engine's
//! own start-of-game logic alone — which is why `docs/DATALAYER.md` §3
//! scored one tick. This module is the seam between the two: the packages
//! decoded by [`crate::commands`] on one side, the order calls of
//! `sim::orders` on the other.
//!
//! **The AI is not in here, and cannot be.** Every class that issues a
//! command is a UI class or the turn pump (`docs/COMMANDS.md` §7); no AI
//! class appears, and `issue_cheat_ai_toggle` — turning the AI on and off is
//! itself a replicated command — only makes sense if the AI runs identically
//! on every client from the same simulation state. So a recording replays
//! the AI by *re-simulating* it. What this module can drive is the units the
//! commands name, and nothing else.
//!
//! ## The frame convention
//!
//! The gamelog's `FRAME n` is the state at the **end** of frame `n` and its
//! frames are numbered from 1; a recording's packages are numbered from 0.
//! A command in the package for recording frame `f` is processed by the game
//! frame that the gamelog then reports as `FRAME f + 1`, so [`Stream::apply`]
//! is called with the frame about to be stepped *into* minus one — i.e.
//! immediately before the tick that produces `FRAME f + 1`.
//!
//! Established from the sample: the recording's `MoveTo` for unit `0/1` sits
//! at frame 376, and the un-fed harness's first order disagreement for that
//! unit is at gamelog frame 377. And diff-backed on a file written here
//! (item 4059, `docs/RECGAME.md` §9): the original's playback of a `move_to`,
//! a `queue_up` and a `build` on recording frames 30, 60 and 90 shows each
//! effect on gamelog frames 31, 61 and 91, and [`Stream::pump`] applied at
//! this convention keeps lockstep with it.

use std::collections::BTreeMap;

use crate::commands::Command;
use crate::diff::Built;
use crate::recgame::RecGame;
use sim::Pos;
use sim::orders::{MoveKind, QueuePos};

/// One command with the frame and the player that issued it.
#[derive(Clone, Debug)]
pub struct Input {
    pub frame: i32,
    /// The issuing player, from the package head.
    pub play: i32,
    /// The issuing player's leader, `GameInfo.player[play].who` — what
    /// `process_group@0094a0c0` and the `process_*` entries act as (item
    /// 4336). Equal to `play` in every lobby this crate mints; the 2017
    /// sample's person is `play` 0 at `who` 3.
    pub who: i32,
    pub cmd: Command,
}

/// Whether a command is input or per-frame housekeeping.
///
/// `Camera` is one a frame and `PlayerSpeed` is the turn pump's; neither
/// reaches the simulation. Everything else is something a player did.
pub fn is_input(cmd: &Command) -> bool {
    !matches!(
        cmd,
        Command::Camera { .. } | Command::PlayerSpeed { .. } | Command::CheckSums { .. }
    )
}

/// Every input command in a recording, in frame order.
///
/// Packages whose payload does not decode are skipped rather than fatal: a
/// multiplayer recording needs [`crate::commands::decode_mp`] and this
/// harness has no MP sample (`docs/COMMANDS.md` §5).
pub fn input(rec: &RecGame) -> Vec<Input> {
    let mut out = Vec::new();
    for p in &rec.packages {
        let who = usize::try_from(p.play)
            .ok()
            .and_then(|k| rec.slots.get(k))
            .map_or(p.play, |s| i32::from(s.who));
        let Ok(cmds) = crate::commands::decode(&p.data) else {
            continue;
        };
        for cmd in cmds {
            if is_input(&cmd) {
                out.push(Input {
                    frame: p.frame,
                    play: p.play,
                    who,
                    cmd,
                });
            }
        }
    }
    out
}

/// The wire's `queued` field, as the simulation's enqueue mode.
///
/// `docs/ORDERS.md` §1.5: 0 ctrl-click, 1 shift-click, 2 plain click. The
/// values are the engine's own and are settled in the listing of
/// `Unit::add_move_facing_order`.
fn queue_pos(queued: i32) -> QueuePos {
    match queued {
        0 => QueuePos::First,
        1 => QueuePos::Last,
        _ => QueuePos::New,
    }
}

/// A `group` command (0x00) and the `move_to` (0x07) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §1 and
/// §3; `docs/ORDERS.md` §8.1) — the command's entry, as the original has
/// it rather than as a per-unit order.
///
/// `CommandPackage::process_group@0094a0c0` builds a `Group` on the stack
/// from the listed objects — `Group::add(o, who, 0, 0)` each, and each
/// captain's `o_down` chain behind it, so a squad's figures come with its
/// captain — and installs it with `Groups::push_group(who, &g, 1)`, forced.
/// `CommandPackage::process_move_to@009497c0` then calls
/// `Group::action_move_to(g, to_x, to_y, queued, set_angle, angle, orders,
/// 1, form, width, disembark)`: the action bit is set on every order it
/// makes, and the group — not each unit — decides who gets a
/// `GroupMoveOrder` and who a plain move (`docs/ORDERS.md` §8.2).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation (the original's group is then empty and
/// `process_group` sets `group = -1`, so the move acts on nothing).
///
/// SEAMS, each with what it leaves out: `form`/`width` are the −1 every
/// army call passes and the only value an issuer line sends, so
/// [`sim::Sim::group_action_move_to`]'s own `−1` is exact here and a packet
/// with any other form is not modelled; `disembark` is 0; the
/// `num = 0` replay of the previous selection is the caller's; and
/// `UnitData::play` (`+0xb6`), which `process_group` sets to the issuing
/// player on every member, is not carried (`docs/GOLDEN.md` §17).
#[expect(
    clippy::too_many_arguments,
    reason = "the group and the move_to command's own fields"
)]
pub fn group_move_to(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    to: Pos,
    queued: i32,
    set_angle: bool,
    angle: i32,
    orders: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    // `OrderIndex`: 1 `MOVE_TO`, 2 `ATTACK_TO`, 3 `EXPLORE_TO`, 4
    // `FLEE_TO` (`docs/ORDERS.md` §1.2). `action_move_near` hands any
    // other value to `add_move_facing_order`, which makes a `MOVE_TO`.
    let kind = match orders {
        2 => MoveKind::AttackTo,
        3 => MoveKind::ExploreTo,
        4 => MoveKind::FleeTo,
        _ => MoveKind::MoveTo,
    };
    built.sim.command().group_action_move_to(
        &g,
        to,
        queue_pos(queued),
        set_angle,
        sim::movement::Angle(angle),
        kind,
        true,
    );
    g.list.len()
}

/// A `group` command (0x00) and the `patrol` (0x0a) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/ORDERS.md` §27) — [`group_move_to`]'s group, then
/// `CommandPackage::process_patrol@00949380`'s one call,
/// `Group::action_patrol(g, to_x, to_y, queued)`
/// ([`sim::Sim::group_action_patrol`]).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s: the
/// `num = 0` replay is the caller's, and `UnitData::play` is not carried.
pub fn group_patrol(built: &mut Built, who: i32, objects: &[i16], to: Pos, queued: i32) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built
        .sim
        .command()
        .group_action_patrol(&g, to, queue_pos(queued));
    g.list.len()
}

/// A `group` command (0x00) and the `attack_ground` behind it, for a group
/// whose leader is a plane (item 3911, `docs/GOLDEN.md` §158):
/// `CommandPackage::process_attack_ground@009494a0` →
/// `Group::action_attack_ground`'s air arm,
/// [`sim::Sim::group_action_air_attack_ground`]. `None` for any other
/// group: the ground arm is walked (`sim::groups::attack_ground_view`) and not
/// wired (parked 3425a).
pub fn group_air_attack_ground(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    to: Pos,
    queued: i32,
) -> Option<usize> {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    let leader = built.sim.group_find_leader(&g)?;
    if !built.sim.is_plane(leader) {
        return None;
    }
    if !built.sim.command().push_group(&mut g, true) {
        return Some(0);
    }
    built
        .sim
        .command()
        .group_action_air_attack_ground(&g, to, queue_pos(queued));
    Some(g.list.len())
}

/// A `group` command (0x00) and the `recall` (0x23) behind it (item 3911,
/// `docs/GOLDEN.md` §158): [`group_patrol`]'s group, then
/// `CommandPackage::process_recall@00947bd0`'s one call,
/// `Group::action_recall`.
pub fn group_recall_units(built: &mut Built, who: i32, objects: &[i16]) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built.sim.command().group_action_recall(&g);
    g.list.len()
}

/// **`Options::do_alarm@0071c3f0`** on a selection of units: one `group`
/// and one `alarm` (0x1b), so the pump's `CommandPackage::process_alarm
/// @00947ef0` → `Group::action_alarm@0070ec30` takes its unit arm,
/// [`sim::Sim::action_alarm_peasant`] (item 3916, `docs/GOLDEN.md` §163).
pub fn group_alarm_peasant(built: &mut Built, who: i32, objects: &[i16]) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built.sim.command().action_alarm_peasant(&g);
    g.list.len()
}

/// A `group` command (0x00) and the `guard` (0x1f) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §19) — [`group_move_to`]'s group, then
/// `CommandPackage::process_guard@009478a0`'s one call,
/// `Group::action_guard(g, ox, whom, queued, 0)`.
///
/// The charge is `whom`'s object `ox`. `action_guard@006fcd30` asks it
/// `is_valid_unit` (object vslot `+0x8`, named by the PDB's `SubObjectData`
/// method list) before anything else, and that is a folded `return 0` on
/// `Build::vftable`: a building charge returns having written only the
/// group's `disband`. So a charge that is no unit of the simulation is
/// that exit, and the group is still pushed, as `process_group` forces it.
/// A unit charge is [`sim::Sim::group_action_guard`] with the siege filter
/// off (the command passes 0).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s.
pub fn group_guard(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let charge = i16::try_from(ox).ok().and_then(|o| {
        built
            .units
            .iter()
            .find(|l| l.who == i64::from(whom) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(whom as sim::Player, o))
    });
    if let Some(t) = charge {
        built
            .sim
            .command()
            .group_action_guard(&g, t, queue_pos(queued), false);
    }
    g.list.len()
}

/// A `group` command (0x00) and the `follow` (0x1e) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §20) — [`group_move_to`]'s group, then
/// `CommandPackage::process_follow@009479c0`'s one call,
/// `Group::action_follow(g, ox, whom, queued)`
/// ([`sim::Sim::group_action_follow`]).
///
/// The leader is `whom`'s object `ox`. `action_follow@006fd510` asks it
/// `is_valid_unit`, as `action_guard` does, so a leader that is no unit of
/// the simulation gives no order, and the group is still pushed, as
/// `process_group` forces it.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s.
pub fn group_follow(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let leader = i16::try_from(ox).ok().and_then(|o| {
        built
            .units
            .iter()
            .find(|l| l.who == i64::from(whom) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(whom as sim::Player, o))
    });
    if let Some(t) = leader {
        built
            .sim
            .command()
            .group_action_follow(&g, t, queue_pos(queued));
    }
    g.list.len()
}

/// A `group` command (0x00) and the `garrison` (0x14) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §21) — [`group_move_to`]'s group, then
/// `CommandPackage::process_garrison@00948760`'s one call,
/// `Group::action_garrison(g, ox, whom, queued, 0)`
/// ([`sim::Sim::group_action_garrison`]).
///
/// The building is `whom`'s object `ox`; `process_garrison` drops a
/// command whose building is dead, and the group is still pushed, as
/// `process_group` forces it.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s.
pub fn group_garrison(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let b = i16::try_from(ox)
        .ok()
        .and_then(|o| built.sim.building_by_o(whom as sim::Player, o));
    if let Some(b) = b {
        built
            .sim
            .command()
            .group_action_garrison(&g, b, queue_pos(queued));
    }
    g.list.len()
}

/// A `group` command (0x00) and the `form` (0x03) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §22) — [`group_move_to`]'s group, then
/// `CommandPackage::process_form@00949d90`'s one call,
/// `Group::action_form(g, form, rotate, queued, g, 0)`
/// ([`sim::Sim::group_action_form`]).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s.
pub fn group_form(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    form: i32,
    rotate: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built
        .sim
        .command()
        .group_action_form(&g, form, rotate, queue_pos(queued));
    g.list.len()
}

/// A `group` command (0x00) and the `attack` (0x04) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §23) — [`group_move_to`]'s group, then
/// `CommandPackage::process_attack@00949c30`'s one call,
/// `Group::action_attack(g, ox, whom, mandatory 1, queued, ignore)`
/// ([`sim::Sim::group_action_attack`]).
///
/// The target is `whom`'s object `ox`, a unit or a building.
/// `process_attack` hands the command on when the target's object is
/// active (or either index is negative, which `action_attack` then drops
/// on its `o < 0` gate), so a target that is no live object of the
/// simulation gives no order, and the group is still pushed, as
/// `process_group` forces it. **`mandatory` is 1**: a player's click
/// names its target, and the `find_melee_target` retarget is the AI's
/// `engagement` alone (`docs/GROUPS.md` §10).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation. The same seams as [`group_move_to`]'s.
pub fn group_attack(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    ignore: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let target = i16::try_from(ox).ok().and_then(|o| {
        built
            .units
            .iter()
            .find(|l| l.who == i64::from(whom) && l.o == i64::from(o))
            .map(|l| sim::combat::Obj::Unit(l.unit))
            .or_else(|| {
                built
                    .sim
                    .unit_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Unit)
            })
            .or_else(|| {
                built
                    .sim
                    .building_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Building)
            })
    });
    if let Some(t) = target {
        built
            .sim
            .command()
            .group_action_attack(&g, t, true, queue_pos(queued), ignore);
    }
    g.list.len()
}

/// A `group` command (0x00) and the `flight` (0x1c) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §25) — [`group_attack`]'s group, then
/// `CommandPackage::process_flight@00947db0`'s one call,
/// `Group::action_flight(g, ox, whom, orders, shift, ctrl, alt)`
/// ([`sim::Sim::group_action_flight`]).
///
/// `orders` is the command's: `MOVE_TO` (1) sends the aircraft home to
/// the base `ox`, `ATTACK` (10) at the target `ox`. The target is `whom`'s
/// object, a unit or a building; one this simulation cannot name gives no
/// order, and the group is still pushed, as `process_group` forces it
/// (run223's block 622). `shift`, `ctrl` and `alt` are 0 on the DLL's
/// call, and `action_flight` reads none of them on the arms here.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_flight(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    orders: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let target = i16::try_from(ox).ok().and_then(|o| {
        built
            .units
            .iter()
            .find(|l| l.who == i64::from(whom) && l.o == i64::from(o))
            .map(|l| sim::combat::Obj::Unit(l.unit))
            .or_else(|| {
                built
                    .sim
                    .unit_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Unit)
            })
            .or_else(|| {
                built
                    .sim
                    .building_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Building)
            })
    });
    let kind = match orders {
        1 => Some(sim::groups::group::Flight::Home),
        10 => Some(sim::groups::group::Flight::Strike),
        _ => None,
    };
    if let (Some(t), Some(k)) = (target, kind) {
        built.sim.command().group_action_flight(&g, t, k);
    }
    g.list.len()
}

/// A `group` command (0x00) and the `build` (0x19) behind it, into the
/// simulation the way the turn pump walks them (`docs/COMMANDS.md` §3;
/// `docs/GOLDEN.md` §26) — [`group_attack`]'s group, then
/// `CommandPackage::process_build@00948110`'s one call,
/// `Group::action_build(g, x, y, x2, y2, type, queued)`
/// ([`sim::Sim::group_action_build`]).
///
/// `build` is the command's `TypeIndex`; the building record is its
/// offset from [`crate::load::BASE_BUILDTYPES`], since the loader keeps
/// record index as identity. `action_build` reads only the first point,
/// so the command's `x2`/`y2` are not carried. A type that is no
/// building record gives no site, and the group is still pushed, as
/// `process_group` forces it.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_build(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    at: Pos,
    build: i32,
    queued: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let record = usize::try_from(build - crate::load::BASE_BUILDTYPES)
        .ok()
        .filter(|&r| r < built.sim.tables.build_types.len());
    if let Some(ty) = record {
        built
            .sim
            .command()
            .group_action_build(&g, at, ty, queue_pos(queued));
    }
    g.list.len()
}

/// A `group` command (0x00) and the `spell` (0x17) behind it
/// (`docs/COMMANDS.md` §3; `docs/GOLDEN.md` §27) — [`group_attack`]'s
/// group, then `CommandPackage::process_spell@00948340`'s one call,
/// `Group::action_spell(g, type, ox, whom, x, y)` when the target is
/// `(−1, −1)` or active ([`sim::Sim::group_action_spell`]).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
#[allow(clippy::too_many_arguments)] // the command's six fields and the harness
pub fn group_spell(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    spell: i32,
    ox: i32,
    whom: i32,
    at: Pos,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let target = i16::try_from(ox).ok().and_then(|o| {
        built
            .units
            .iter()
            .find(|l| l.who == i64::from(whom) && l.o == i64::from(o))
            .map(|l| sim::combat::Obj::Unit(l.unit))
            .or_else(|| {
                built
                    .sim
                    .unit_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Unit)
            })
            .or_else(|| {
                built
                    .sim
                    .building_by_o(whom as sim::Player, o)
                    .map(sim::combat::Obj::Building)
            })
    });
    built
        .sim
        .command()
        .group_action_spell(&g, spell, target, at);
    g.list.len()
}

/// A `group` command (0x00) and the `set_transport` (0x0e) behind it
/// (`docs/COMMANDS.md` §3; `docs/GOLDEN.md` §28) — [`group_spell`]'s
/// group, then `CommandPackage::process_set_transport@00948f60`'s one call,
/// `Group::action_set_transport(g, flag)@007024b0`
/// ([`sim::Sim::set_transport`]): every active member that can ever
/// transport has its auto-transport bit set, or cleared for a flag of 0,
/// and a leader whose transport level is 0 forces the flag to 0. It lays no
/// order and spends no draw. `action_begin`, the call before the loop,
/// zeroes a field of the group's that nothing here reads.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_set_transport(built: &mut Built, who: i32, objects: &[i16], flag: i32) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built
        .sim
        .command()
        .set_transport(player, &g.list, flag != 0);
    g.list.len()
}

/// A `group` command (0x00) and the `stance` (0x02) behind it (item 2165,
/// `docs/GOLDEN.md` §79) — [`group_set_transport`]'s group, then
/// `CommandPackage::process_stance@00949ed0`'s one call, unchecked:
/// `Group::action_stance(g, stance)@0070d440`
/// ([`sim::Sim::group_action_stance`]).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_stance(built: &mut Built, who: i32, objects: &[i16], stance: i32) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built.sim.command().group_action_stance(&g, stance);
    g.list.len()
}

/// **A stance press on the package's group** (item 4282,
/// `docs/CLIENT_INPUT.md` §27): `CommandPackage::process_stance@00949ed0`'s
/// `Group::action_stance(g, stance)@0070d440`. A group of buildings (every
/// object 2000 and up) takes the building arm,
/// [`sim::Sim::action_stance_buildings`], with the group pushed as
/// `process_group` pushes one; any other takes [`group_stance`].
///
/// Returns the members written, or the unit group's size.
pub fn group_stance_any(built: &mut Built, who: i32, objects: &[i16], stance: i32) -> usize {
    let player = who as sim::Player;
    if !objects.is_empty() && objects.iter().all(|&o| o >= 2000) {
        let list: Vec<usize> = objects
            .iter()
            .filter_map(|&o| built.sim.building_by_o(player, o))
            .collect();
        if list.is_empty() {
            return 0;
        }
        built.sim.command().push_command_buildings(player, &list);
        return built.sim.command().action_stance_buildings(&list, stance);
    }
    group_stance(built, who, objects, stance)
}

/// A `group` command (0x00) and the `swarm_around` (0x06) behind it
/// (`docs/COMMANDS.md` §3; `docs/GOLDEN.md` §29) — [`group_spell`]'s
/// group, then `CommandPackage::process_swarm_around@00949970`'s one call,
/// `Group::action_swarm_around(ox, whom, queued, orders, 1)` when the
/// target is `(−1, −1)` or active ([`sim::Sim::group_swarm_around`]). A
/// player's repair is this command with `orders` `REPAIR` (13).
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_swarm_around(
    built: &mut Built,
    who: i32,
    objects: &[i16],
    ox: i32,
    whom: i32,
    queued: i32,
    orders: i32,
) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let site = i16::try_from(ox)
        .ok()
        .and_then(|o| built.sim.building_by_o(whom as sim::Player, o));
    if let (Some(b), Ok(orders)) = (site, u8::try_from(orders)) {
        built
            .sim
            .command()
            .group_swarm_around(&g, b, queue_pos(queued), orders);
    }
    g.list.len()
}

/// A `group` of buildings (0x00) and the `alarm` (0x1b) behind it —
/// `CommandPackage::process_alarm@00947ef0`'s one call,
/// `Group::action_alarm@0070ec30` ([`sim::Sim::action_alarm`]): the
/// City's alarm button (item 1167, `docs/GOLDEN.md` §49). The command's
/// `process_group` pushes the building group into the pool first
/// ([`sim::Sim::push_command_buildings`]), as [`group_buildmask`]'s does.
///
/// Returns the number of the player's live buildings named.
pub fn group_alarm(built: &mut Built, who: i32, buildings: &[i16]) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built.sim.command().action_alarm(player, &list);
    list.len()
}

/// A `group` command (0x00) and the `gather` (0x13) behind it —
/// [`group_swarm_around`]'s group, then `CommandPackage::process_gather@
/// 009488b0`'s one call, `Group::action_gather(ox, queued)` when `ox` is
/// negative or the player's building `ox` is active (`+8 & 1`)
/// ([`sim::Sim::group_gather`]), item 1167, `docs/GOLDEN.md` §49. The
/// building is the group's own player's: the command carries no `whom`.
///
/// Returns the group's size, 0 when no listed object is a live unit of
/// `who` in the simulation.
pub fn group_gather(built: &mut Built, who: i32, objects: &[i16], ox: i32, queued: i32) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    let site = i16::try_from(ox)
        .ok()
        .and_then(|o| built.sim.building_by_o(player, o))
        .filter(|&b| built.sim.buildings[b].alive);
    if let Some(b) = site {
        built.sim.command().group_gather(&g, b, queue_pos(queued));
    }
    g.list.len()
}

/// A `group` of buildings (0x00) and the `eject_all` (0x1a) behind it —
/// `CommandPackage::process_eject_all@00947fe0`'s one call,
/// `Group::action_eject_all(g, back_to_work, who, eject_o, eject_who)`
/// ([`sim::Sim::action_eject_all`]), with the Eject button's `back_to_work
/// 0, who −1, eject_o −1, eject_who −1` (`docs/GOLDEN.md` §21).
///
/// SEAM: the command's `process_group` pushes the building group into the
/// pool, which this crate's pool of units does not hold; no capture on
/// file pushes a group after an eject.
///
/// Returns the number of the player's live buildings named.
pub fn group_eject_all(built: &mut Built, who: i32, buildings: &[i16]) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built.sim.command().action_eject_all(player, &list, -1, -1);
    list.len()
}

/// A `group` of buildings (0x00) and the `buildmask` (0x21) behind it —
/// `CommandPackage::process_buildmask@00947680`'s one call,
/// `Group::action_buildmask(g, buildmask, set)` ([`sim::Sim::action_buildmask`]),
/// which toggles and never reads `set` (`docs/GOLDEN.md` §32). The repeat
/// button's mask is 0x80.
///
/// The command's `process_group` pushes the building group into the pool
/// first (`Groups::push_group@0070f9e0`, called at `0x94a6cf`;
/// [`sim::Sim::push_command_buildings`]): run281's slot 0 holds `[2007]`
/// from 1442, a slot `Groups::get_open_slot@006fa460` counts as open.
///
/// Returns the number of the player's live buildings named.
pub fn group_buildmask(built: &mut Built, who: i32, buildings: &[i16], mask: i32) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built.sim.command().action_buildmask(&list, mask);
    list.len()
}

/// **`CommandPackage::process_queue_up@00948230`** on a group of the
/// player's own buildings: `process_group`'s group, then
/// `Group::action_queue_up@006fdbb0(type, num)` — a unit's button
/// (`GroupOut::issue_queue_up@00708c90`, item 877, `docs/GOLDEN.md` §33)
/// or a technology's (item 883, §35). `ty` is the original's `TypeIndex`,
/// mapped onto this crate's unit type through the tree the harness built,
/// or onto the tree's technology ([`sim::Sim::action_queue_research`]).
///
/// The command's building group is seated in the pool whole, in the
/// command's order (item 888, `docs/GOLDEN.md` §37).
///
/// SEAM: a building type's arm (its `build_flags & 4` upgrade) is not
/// mapped.
///
/// Returns the entries laid.
pub fn group_queue_up(built: &mut Built, who: i32, buildings: &[i16], ty: i32, num: i32) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    // `DISBAND`, a craft: the research arm on its own entry (item 3918,
    // `docs/GOLDEN.md` §165).
    if ty == sim::cities::queue_disband::DISBAND {
        if list.is_empty() {
            return 0;
        }
        built.sim.command().push_command_buildings(player, &list);
        return built.sim.command().action_queue_disband(&list);
    }
    let unit = (0..built.unit_tree.len())
        .find(|&t| built.type_index.get(built.unit_tree[t]).copied() == Some(ty));
    let tech = (0..built.type_index.len()).find(|&t| {
        built.type_index[t] == ty
            && built
                .sim
                .tables
                .tech_tree
                .types
                .get(t)
                .is_some_and(|_| built.sim.tables.tech_tree.kind(t).is_tech())
    });
    if list.is_empty() || (unit.is_none() && tech.is_none()) {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    match (unit, tech) {
        (Some(unit), _) => built.sim.command().action_queue_up(&list, unit, num),
        (None, Some(t)) => built.sim.command().action_queue_research(&list, t),
        (None, None) => 0,
    }
}

/// **`CommandPackage::process_gather_point@00948510`** on a group of the
/// player's own buildings: `process_group`'s group, then
/// `Group::action_gather_point@006ff1b0(x, y, action, add_to_end)`
/// ([`sim::Sim::action_gather_point`]) — the rally point (item 928,
/// `docs/GOLDEN.md` §39). `x` or `y` below 0 is the Clear. The command's
/// building group is seated in the pool whole, as every building command's
/// is (run312's 618, 652 and 702).
///
/// Returns the number of the player's live buildings named.
pub fn group_gather_point(
    built: &mut Built,
    who: i32,
    buildings: &[i16],
    at: Pos,
    action: i32,
    add_to_end: bool,
) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built
        .sim
        .command()
        .action_gather_point(player, &list, at, action, add_to_end);
    list.len()
}

/// **`CommandPackage::process_launch_patrol@00949230`** on the player's
/// own buildings: a right-click on the ground with an Airbase selected,
/// `Group::action_launch_patrol(x, y, queue, shift, ctrl, alt)`
/// ([`sim::Sim::group_action_launch_patrol`]), after `process_group`'s push
/// of the building group (item 976, `docs/GOLDEN.md` §42). `all` is the
/// shift-click's `QUEUE_LAST` and shift; plain, `QUEUE_NEW` and none.
/// `keys`' ctrl and alt are the command's own bytes (item 1009,
/// `docs/GOLDEN.md` §43); its shift is `all`'s.
///
/// Returns the number of the player's live buildings named.
pub fn group_launch_patrol(
    built: &mut Built,
    who: i32,
    buildings: &[i16],
    at: Pos,
    all: bool,
    keys: sim::air::airbase::Keys,
) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    let queue = if all {
        sim::orders::QueuePos::Last
    } else {
        sim::orders::QueuePos::New
    };
    let keys = sim::air::airbase::Keys { shift: all, ..keys };
    built
        .sim
        .command()
        .group_action_launch_patrol(&list, at, queue, keys);
    list.len()
}

/// **`CommandPackage::process_recall@00947bd0`** on the player's own
/// buildings: `OPTION_AIRBASE_RECALL` with an Airbase selected, the
/// selection's `group` and a `recall` (0x23), whose `Group::action_recall`
/// building arm is [`sim::Sim::group_action_recall_buildings`] (item 3912,
/// `docs/GOLDEN.md` §159).
///
/// Returns the number of the player's live buildings named.
pub fn group_recall(built: &mut Built, who: i32, buildings: &[i16]) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built
        .sim
        .command()
        .group_action_recall_buildings(player, &list);
    list.len()
}

/// **`CommandPackage::process_scramble@00947ae0`** on the player's own
/// buildings: `OPTION_AIRBASE_SCRAMBLE`, a `scramble` (0x24), whose entry
/// is [`sim::Sim::group_action_scramble`] (item 3912, `docs/GOLDEN.md`
/// §159).
///
/// Returns the number of the player's live buildings named.
pub fn group_scramble(built: &mut Built, who: i32, buildings: &[i16]) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built.sim.command().group_action_scramble(&list);
    list.len()
}

/// **`CommandPackage::process_city_gather@00948a10`** on the player's own
/// buildings: a City panel's gather button, `OPTION_VILLAGE_GATHER_ALL`
/// with `kind` −1 (item 3915, `docs/GOLDEN.md` §162). `Options::
/// do_city_gather@0071cdb0` issues it when the selection is of buildings,
/// one of them takes a gather point (`COUNT_GATHER_POINT`) and one is a
/// `VILLAGE` (`COUNT_TYPE` 0x19e); its entry is [`sim::Sim::
/// group_action_city_gather`].
///
/// Returns the number of the player's live buildings named.
pub fn group_city_gather(built: &mut Built, who: i32, buildings: &[i16], kind: i32) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    built.sim.command().group_action_city_gather(&list, kind);
    list.len()
}

/// **`CommandPackage::process_stop_spell@00947cc0`** on the player's own
/// units: `OPTION_QUEUE` with a caster selected (item 3913,
/// `docs/GOLDEN.md` §160). `Options::exec@007188c0`'s 0xa6 arm walks the
/// selection and, for each member that is a unit with `num_queued`
/// (`+0xa0`) 0, calls `GroupOut::issue_stop_spell@00708a20` on the whole
/// selection: a `group` and a one-byte `stop_spell` (0x1d) apiece, each
/// processed as [`sim::Sim::group_action_stop_spell`].
///
/// SEAM: a unit's own queue (a carrier's `num_queued`), whose member takes
/// `UnitOut::issue_unqueue` instead; no staging presses stop on run1066's carrier.
///
/// Returns the number of commands processed.
pub fn group_stop_spell(built: &mut Built, who: i32, objects: &[i16]) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    let n = g.list.len();
    for _ in 0..n {
        let mut once = g.clone();
        if !built.sim.command().push_group(&mut once, true) {
            return 0;
        }
        built.sim.command().group_action_stop_spell(&once);
    }
    n
}

/// **`CommandPackage::process_disband@00948660`** on the player's own
/// units: `OPTION_DISBAND` with the seat's units selected (item 3914,
/// `docs/GOLDEN.md` §161). `Options::exec@007188c0`'s 0xf arm calls
/// `Options::do_disband@0071df20(0)` once on the selection: after
/// `GroupOut::validate_disband` (0 for a unit group) it issues one `group`
/// and one `disband` (0x15, `all` 0), processed as
/// [`sim::Sim::group_action_disband`].
///
/// Returns the number of members disbanded, or queued a `DISBAND`.
pub fn group_disband(built: &mut Built, who: i32, objects: &[i16], all: bool) -> usize {
    let player = who as sim::Player;
    // A selection of buildings is a building group (`+0x49`): the pump's
    // `process_group` pushes it and `action_disband` takes its building
    // arms (item 3994, `docs/GOLDEN.md` §187). `Options::do_disband`
    // issues nothing when `validate_disband` refuses.
    let buildings: Vec<usize> = objects
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if !buildings.is_empty() {
        if built.sim.disband_refused(&buildings) {
            return 0;
        }
        built
            .sim
            .command()
            .push_command_buildings(player, &buildings);
        return built
            .sim
            .command()
            .group_action_disband_buildings(&buildings, all);
    }
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built.sim.command().group_action_disband(&g, all)
}

/// **`CommandPackage::process_unitmask@00947790`** (item 4291,
/// `docs/CLIENT_INPUT.md` §31): `process_group`'s push of the selection's
/// units, then `Group::action_unitmask(mask, set)` on it
/// ([`sim::Sim::group_action_unitmask`]). A group of buildings takes no
/// arm of it, so a selection of buildings acts on nothing.
///
/// Returns the number of members toggled.
pub fn group_unitmask(built: &mut Built, who: i32, objects: &[i16], mask: i32) -> usize {
    let player = who as sim::Player;
    let mut g = sim::groups::group::Group::stack(player);
    for &o in objects {
        let unit = built
            .units
            .iter()
            .find(|l| l.who == i64::from(who) && l.o == i64::from(o))
            .map(|l| l.unit)
            .or_else(|| built.sim.unit_by_o(player, o));
        if let Some(u) = unit {
            built.sim.group_add(&mut g, u);
        }
    }
    if g.list.is_empty() || !built.sim.command().push_group(&mut g, true) {
        return 0;
    }
    built.sim.command().group_action_unitmask(&g, mask)
}

/// **`CommandPackage::process_flight@00947db0`** on the player's own
/// buildings: an Airbase's right-click on an enemy, or on another base of
/// its own, `issue_flight` on the building group itself, whose
/// `Group::action_flight@006fb260` is `action_launch_flight`
/// ([`sim::Sim::group_action_launch_flight`]), after `process_group`'s push
/// of the building group (item 976, `docs/GOLDEN.md` §42). `orders` is the
/// command's, `ATTACK` (10) on an enemy and `MOVE_TO` (1) on a base (item
/// 1009, §43); `keys` its shift, ctrl and alt bytes.
///
/// Returns the number of the player's live buildings named.
pub fn group_launch_flight(
    built: &mut Built,
    who: i32,
    buildings: &[i16],
    ox: i32,
    whom: i32,
    orders: i32,
    keys: sim::air::airbase::Keys,
) -> usize {
    let player = who as sim::Player;
    let list: Vec<usize> = buildings
        .iter()
        .filter_map(|&o| built.sim.building_by_o(player, o))
        .collect();
    if list.is_empty() {
        return 0;
    }
    built.sim.command().push_command_buildings(player, &list);
    let target = i16::try_from(ox).ok().and_then(|o| {
        if o >= 2000 {
            built
                .sim
                .building_by_o(whom as sim::Player, o)
                .map(sim::combat::Obj::Building)
        } else {
            built
                .sim
                .unit_by_o(whom as sim::Player, o)
                .map(sim::combat::Obj::Unit)
        }
    });
    let kind = match orders {
        1 => Some(sim::groups::group::Flight::Home),
        10 => Some(sim::groups::group::Flight::Strike),
        _ => None,
    };
    if let (Some(t), Some(k)) = (target, kind) {
        built
            .sim
            .command()
            .group_action_launch_flight(player, &list, t, k, keys);
    }
    list.len()
}

/// **`CommandPackage::process_unqueue@009466f0`** on one of the player's
/// own buildings: `Build::action_unqueue@00620280(p)`
/// ([`sim::Sim::action_unqueue`]), the cancel (item 884, `docs/GOLDEN.md`
/// §34). The command carries no `group` — `CommandManager::
/// issue_unqueue@00942c40` writes `[who][o][p][uid]`, 15 bytes under the
/// emulator — so it seats nothing in the pool. `p` is the slot, or the
/// selector's negatives (−1 the last, −5 five, −10 all).
///
/// `process_unqueue`'s unit arm is [`unqueue_unit`] (item 3981).
/// SEAM: the `uid` test.
///
/// Returns the number of the player's live buildings named, 0 or 1: a
/// single cancel on an infinite queue acts and removes nothing.
pub fn unqueue(built: &mut Built, who: i32, o: i32, p: i32) -> usize {
    let Some(b) = i16::try_from(o)
        .ok()
        .and_then(|o| built.sim.building_by_o(who as sim::Player, o))
    else {
        return 0;
    };
    built.sim.command().action_unqueue(b, p);
    1
}

/// **`CommandPackage::process_unqueue@009466f0`'s unit arm** (item 3981,
/// `docs/AI.md` §413): `units[who][o]` active, then `Unit::action_unqueue
/// @005e1f20(1)` ([`sim::Sim::unit_action_unqueue`]) — one plane off a
/// carrier's own queue, its price back. `@unqueueunit`'s entry.
///
/// SEAM: the command's `uid` test; the staging names the unit it means.
///
/// Returns 1 for a live unit of `who`, else 0.
pub fn unqueue_unit(built: &mut Built, who: i32, o: i16) -> usize {
    let Some(u) = built.sim.unit_by_o(who as sim::Player, o) else {
        return 0;
    };
    if !built.sim.units[u].alive() {
        return 0;
    }
    built.sim.command().unit_action_unqueue(u, true);
    1
}

/// What one frame's commands did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    /// Orders actually enqueued on a simulation unit.
    pub orders: usize,
    /// Commands the harness understood but could not act on, by name and
    /// reason — the honest half of the report. A command counted here is a
    /// named gap, not a silent drop.
    pub skipped: BTreeMap<(&'static str, &'static str), usize>,
}

impl Applied {
    fn skip(&mut self, cmd: &Command, why: &'static str) {
        *self.skipped.entry((cmd.name(), why)).or_default() += 1;
    }

    /// Fold another frame's result into this one.
    pub fn merge(&mut self, other: &Applied) {
        self.orders += other.orders;
        for (k, n) in &other.skipped {
            *self.skipped.entry(*k).or_default() += n;
        }
    }

    pub fn skipped_total(&self) -> usize {
        self.skipped.values().sum()
    }
}

/// Rebuild the group that a hot key copies from the selected live objects.
/// The first member fixes the kind. A repeated selection must still name
/// the same object instance even if its original number has been reused.
fn push_hotkey_group(built: &mut Built, player: sim::Player, received: &[(i16, u16)]) -> bool {
    let mut units = sim::groups::group::Group::stack(player);
    let mut buildings = Vec::new();
    for &(o, uid) in received {
        if o >= sim::units::unit::BUILD_BASE {
            if units.list.is_empty()
                && let Some(b) = built.sim.building_by_o(player, o)
                && built.sim.buildings[b].uid == uid
            {
                buildings.push(b);
            }
        } else if buildings.is_empty()
            && let Some(u) = built.sim.unit_by_o(player, o)
            && built.sim.units[u].uid == uid
        {
            built.sim.group_add(&mut units, u);
        }
    }
    if buildings.is_empty() {
        return !units.list.is_empty() && built.sim.command().push_group(&mut units, true);
    }
    built
        .sim
        .command()
        .push_command_buildings(player, &buildings);
    true
}

/// The stream, with the selection state the commands assume.
///
/// A `group` command sets the issuing player's selection and the
/// unit-targeted commands that follow act on it; `num = 0` — an empty
/// `objects` — means "the same selection as this player's previous group
/// command" (`docs/COMMANDS.md` §3). The selection is therefore *state*, and
/// it has to be carried across frames exactly as the engine carries it.
///
/// Two modes. [`Stream::new`] is the capture replays' stream: `move_to`
/// per selected unit, `queue_up` and `build` skipped, the arms the kept
/// captures' pins were taken under. [`Stream::pump`] is the turn pump's
/// (item 4059, `docs/RECGAME.md` §9): `group` with `move_to`, `queue_up`
/// and `build` go through the command's own entries, the ones the golden
/// chapters issue — [`group_move_to`], [`group_queue_up`], [`group_build`]
/// — and, since item 4179 (a right press on an object), `swarm_around`,
/// `gather`, `garrison` and `attack` through theirs; every other command
/// takes `new`'s arm.
#[derive(Clone)]
pub struct Stream {
    input: Vec<Input>,
    next: usize,
    selection: Vec<Vec<i16>>,
    /// Per player, the objects and uids the last non-empty `group` recorded
    /// (`CommandPackage::last_objects_received`, `last_uids_received`).
    received: Vec<Vec<(i16, u16)>>,
    pump: bool,
}

impl Stream {
    pub fn new(rec: &RecGame) -> Stream {
        Stream {
            input: input(rec),
            next: 0,
            selection: vec![Vec::new(); 8],
            received: vec![Vec::new(); 8],
            pump: false,
        }
    }

    /// The turn pump's stream: `CommandPackage::process_all@0094c500`
    /// walking each package front to back, `process_group` setting the
    /// selection and `process_move_to`, `process_queue_up` and
    /// `process_build` acting on it through the entries this module keeps
    /// for them. A package's `play` is the group's leader, as
    /// `process_group`'s player test requires of a solo game's packages.
    ///
    /// SEAM: each entry pushes its own group, so a package holding one
    /// `group` and two commands behind it pushes two where the original
    /// pushes one; a minted script puts one command behind each `group`.
    pub fn pump(rec: &RecGame) -> Stream {
        Stream {
            pump: true,
            ..Stream::new(rec)
        }
    }

    /// A turn pump's stream with nothing in it yet: the client's, which
    /// [`Stream::push`] feeds as a person issues orders (item 4145).
    pub fn pump_empty() -> Stream {
        Stream {
            input: Vec::new(),
            next: 0,
            selection: vec![Vec::new(); 8],
            received: vec![Vec::new(); 8],
            pump: true,
        }
    }

    /// One package appended: `order`'s commands encoded as a recording
    /// stores them and decoded back, each an [`Input`] on the order's
    /// frame. The stream is consumed in order, so a package for a frame
    /// already applied is never taken.
    pub fn push(&mut self, order: &crate::recgame::Order) {
        let (frame, play, cmds) = order;
        let bytes = crate::commands::encode(cmds);
        let cmds = crate::commands::decode(&bytes).expect("a package this crate encoded decodes");
        self.input
            .extend(cmds.into_iter().filter(is_input).map(|cmd| Input {
                frame: *frame,
                play: *play,
                who: *play,
                cmd,
            }));
    }

    /// How many input commands the stream holds.
    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// The last frame any command is issued on.
    pub fn last_frame(&self) -> i32 {
        self.input.last().map(|i| i.frame).unwrap_or(0)
    }

    /// Apply every command issued on `frame`, in order.
    ///
    /// Call immediately before the tick that produces the gamelog's
    /// `FRAME frame + 1` (see the module note on the frame convention). The
    /// stream is consumed in order, so `frame` must not go backwards.
    pub fn apply(&mut self, frame: i32, built: &mut Built) -> Applied {
        let mut done = Applied::default();
        while self.next < self.input.len() && self.input[self.next].frame < frame {
            // A frame that was never offered: count it rather than lose it.
            let it = &self.input[self.next];
            let cmd = it.cmd.clone();
            done.skip(&cmd, "frame stepped past");
            self.next += 1;
        }
        while self.next < self.input.len() && self.input[self.next].frame == frame {
            let it = self.input[self.next].clone();
            self.next += 1;
            self.one(&it, built, &mut done);
        }
        done
    }

    fn one(&mut self, it: &Input, built: &mut Built, done: &mut Applied) {
        if self.pump && self.pumped(it, built, done) {
            return;
        }
        match &it.cmd {
            // The selection, which every unit-targeted command below reads.
            Command::Group { who, objects } => {
                let slot = *who as usize;
                if slot >= self.selection.len() {
                    done.skip(&it.cmd, "who out of range");
                    return;
                }
                if !objects.is_empty() {
                    self.selection[slot] = objects.clone();
                }
                // `process_group@0094a0c0`'s `UnitData::play` write, for
                // the issuing player's own units — a package whose `who` is
                // not its player's does nothing (`docs/ORDERS.md` §8.1).
                // The package's leader is `info.player[play].who` (item
                // 4336), the team test here as in
                // [`Stream::selected_units`].
                if i32::from(*who) == it.who
                    && let Ok(play) = i8::try_from(it.play)
                {
                    let player = *who as sim::Player;
                    if objects.is_empty() {
                        let last = self.received[slot].clone();
                        built
                            .sim
                            .command()
                            .process_group_play_repeat(player, play, &last);
                    } else {
                        self.received[slot] = built
                            .sim
                            .command()
                            .process_group_play(player, play, objects);
                    }
                }
            }

            Command::MoveTo {
                to_x, to_y, queued, ..
            } => {
                let to = Pos::new(*to_x, *to_y);
                let pos = queue_pos(i32::from(*queued));
                let mut any = false;
                for u in self.selected_units(it.who, built) {
                    // `process_move_to` → `Group::action_move_to(..., 1, ...)`:
                    // the action bit is set (`docs/ORDERS.md` §14's table).
                    built
                        .sim
                        .command()
                        .add_move_order(u, to, MoveKind::MoveTo, pos, true);
                    done.orders += 1;
                    any = true;
                }
                if !any {
                    done.skip(&it.cmd, "no selected unit is in the simulation");
                }
            }

            // Everything below is a real command the harness has no
            // mechanic for yet. Each is a named seam, counted so the report
            // can say what the stream carried and the diff ignored.
            Command::SwarmAround { .. } | Command::Repair { .. } => {
                done.skip(&it.cmd, "the target site is not in the simulation");
            }
            // No kept stream carries an `unqueue` (item 884: `grep
            // process_unqueue` over every kept dump finds none). run7's
            // three `queue_up`s stay skipped: wiring them moves run7's
            // pins, and `group_queue_up` is the golden harness's entry.
            Command::Unqueue { who, o, unit, .. } => {
                if unqueue(built, *who, *o, *unit) == 0 {
                    done.skip(&it.cmd, "the building is not in the simulation");
                }
            }
            Command::QueueUp { .. } => {
                done.skip(&it.cmd, "production is not wired into the harness");
            }
            // **The market** (item 4259, `docs/CLIENT_INPUT.md` §16):
            // `process_sell` and `process_buy` on the order's own `who`,
            // which is the leader they name, not the package's slot.
            Command::Sell { who, good, flags } | Command::Buy { who, good, flags } => {
                match (sim::Player::try_from(*who), usize::try_from(*good)) {
                    (Ok(who), Ok(good)) => {
                        if matches!(it.cmd, Command::Sell { .. }) {
                            built.sim.command().process_sell(who, good, *flags);
                        } else {
                            built.sim.command().process_buy(who, good, *flags);
                        }
                    }
                    _ => done.skip(&it.cmd, "the leader or the good is out of range"),
                }
            }
            // **The rally point** (item 928): the recorded command is the
            // one the golden harness issues, `process_gather_point` on the
            // selection's buildings — [`group_gather_point`]. Only run7's
            // stream carries one (1184, a point on the ground; `grep
            // process_gather_point` over every kept dump), and neither
            // long capture does.
            Command::GatherPoint {
                x,
                y,
                action,
                add_to_end,
            } => {
                let slot = it.who as usize;
                let objects = self.selection.get(slot).cloned().unwrap_or_default();
                let n = group_gather_point(
                    built,
                    it.who,
                    &objects,
                    Pos::new(*x, *y),
                    *action,
                    *add_to_end != 0,
                );
                if n == 0 {
                    done.skip(&it.cmd, "no selected building is in the simulation");
                }
            }
            // **The city's gather** (item 4261, `docs/CLIENT_INPUT.md`
            // §17): `process_city_gather@00948a10` is
            // `Group::action_city_gather(t, …)` on the package's group, the
            // slot's selection; `queued` is not read by it.
            Command::CityGather { t, .. } => {
                let slot = it.who as usize;
                let objects = self.selection.get(slot).cloned().unwrap_or_default();
                if group_city_gather(built, it.who, &objects, *t) == 0 {
                    done.skip(&it.cmd, "no selected building is in the simulation");
                }
            }
            Command::Chat { .. } => {
                // A cheat line. It is input — in solo it travels the order
                // stream — but its effect is `ConsoleWin::run_cmd`'s, not an
                // order's, and staging cheats are outside the simulation.
                done.skip(&it.cmd, "a cheat line, not an order");
            }
            // **A garrisoned unit's own order to leave** (item 4280,
            // `docs/CLIENT_INPUT.md` §26): `process_come_out@009465d0` on
            // `units[who][o]` when its `uid` is the order's. No group.
            Command::ComeOut { who, o, uid } => {
                let done_it = match (sim::Player::try_from(*who), i16::try_from(*o)) {
                    (Ok(who), Ok(o)) => built.sim.command().process_come_out(who, o, *uid as u16),
                    _ => false,
                };
                if !done_it {
                    done.skip(&it.cmd, "no live unit of that number and uid");
                }
            }
            // **A city's name** (item 4268, `docs/CLIENT_INPUT.md` §21):
            // `process_rename_city@00944090` on the order's own `who`, the
            // console's, and the building's `o`.
            Command::RenameCity { who, o, name, .. } => {
                let done_it = match (sim::Player::try_from(*who), i16::try_from(*o)) {
                    (Ok(who), Ok(o)) => built.sim.command().process_rename_city(who, o, name),
                    _ => false,
                };
                if !done_it {
                    done.skip(&it.cmd, "the building is in no city of the simulation");
                }
            }
            // **A hot key** (item 4270, `docs/CLIENT_INPUT.md` §22):
            // `process_group`'s push of the package's group, then
            // `process_hotkey@009474d0`'s copy of that pool slot into the
            // hot-key group the command names.
            Command::Hotkey {
                group,
                clear,
                valid,
                x,
                y,
                zoom,
            } => {
                let player = it.who as sim::Player;
                if *clear == 0
                    && !self
                        .received
                        .get(it.who as usize)
                        .is_some_and(|list| push_hotkey_group(built, player, list))
                {
                    done.skip(&it.cmd, "no selected object is in the simulation");
                    return;
                }
                let k = sim::groups::hotkey::HotKey {
                    group: *group,
                    clear: *clear,
                    valid: *valid,
                    loc: (*x, *y),
                    zoom: *zoom,
                };
                if !built.sim.command().process_hotkey(player, k) {
                    done.skip(&it.cmd, "the package's group is not in the simulation");
                }
            }
            // **The AI toggle** (item 4283, `docs/CLIENT_INPUT.md` §28):
            // `process_cheat_ai_toggle@00944d20`, the global flag flipped.
            // The console's word, so no group and no slot's units: it is
            // the one order whose effect is every computer seat's.
            Command::CheatAiToggle => {
                built.sim.command().process_cheat_ai_toggle();
            }
            Command::Pause { .. } => {
                // The turn control's (`TurnControl::process_pause@00956990`,
                // item 4252): a solo game's pause stops the frame, not the
                // turns, and writes nothing the simulation keeps. The
                // packages a paused game issues share the frame they stood
                // on, and the stream applies them in order before its tick.
                done.skip(&it.cmd, "the turn control's, not the simulation's");
            }
            _ => done.skip(&it.cmd, "not mapped"),
        }
    }

    /// [`Stream::pump`]'s three commands, through their entries; `false`
    /// for any other, which `one` then takes as [`Stream::new`] does.
    fn pumped(&mut self, it: &Input, built: &mut Built, done: &mut Applied) -> bool {
        let objects = self
            .selection
            .get(it.who as usize)
            .cloned()
            .unwrap_or_default();
        let n = match &it.cmd {
            Command::MoveTo {
                to_x,
                to_y,
                set_angle,
                angle,
                orders,
                queued,
                ..
            } => group_move_to(
                built,
                it.who,
                &objects,
                Pos::new(*to_x, *to_y),
                i32::from(*queued),
                *set_angle != 0,
                *angle,
                i32::from(*orders),
            ),
            Command::QueueUp { unit, num } => group_queue_up(built, it.who, &objects, *unit, *num),
            Command::Build {
                x,
                y,
                build,
                queued,
                ..
            } => group_build(built, it.who, &objects, Pos::new(*x, *y), *build, *queued),
            // A right press on an object (item 4179, `docs/CLIENT_INPUT.md`
            // §6): the commands `Console::execute_at_cursor` issues on one.
            Command::SwarmAround {
                ox,
                whom,
                queued,
                orders,
            } => group_swarm_around(built, it.who, &objects, *ox, *whom, *queued, *orders),
            Command::Gather { ox, queued } => group_gather(built, it.who, &objects, *ox, *queued),
            Command::Garrison { ox, whom, queued } => {
                group_garrison(built, it.who, &objects, *ox, *whom, *queued)
            }
            // The infinite-queue button (item 4276, `docs/CLIENT_INPUT.md`
            // §24): `process_buildmask@00947680` on the package's group.
            Command::Buildmask { mask, .. } => group_buildmask(built, it.who, &objects, *mask),
            // The stance button (item 4282, `docs/CLIENT_INPUT.md` §27):
            // `process_stance@00949ed0` on the package's group.
            Command::Stance { stance } => group_stance_any(built, it.who, &objects, *stance),
            // The disband button (item 4287, `docs/CLIENT_INPUT.md` §29):
            // `process_disband@00948660`, `Group::action_disband(all)` on
            // the package's group.
            Command::Disband { all } => group_disband(built, it.who, &objects, *all != 0),
            // The AUTO EXPLORE button (item 4291, `docs/CLIENT_INPUT.md`
            // §31): `process_unitmask@00947790`, `Group::action_unitmask`
            // on the package's group; `set` is never read.
            Command::Unitmask { mask, .. } => group_unitmask(built, it.who, &objects, *mask),
            Command::Attack {
                ox,
                whom,
                ignore,
                queued,
            } => group_attack(built, it.who, &objects, *ox, *whom, *ignore, *queued),
            // A right press on a unit by a selection that cannot attack, or
            // on one's own unit (item 4202, `docs/CLIENT_INPUT.md` §10).
            Command::Follow { ox, whom, queued } => {
                group_follow(built, it.who, &objects, *ox, *whom, *queued)
            }
            // `OPTION_TRANSPORT`'s order (item 4375, `docs/CLIENT_INPUT.md`
            // §32.14): `process_set_transport@00948f60`'s one call.
            Command::SetTransport { flag } => group_set_transport(built, it.who, &objects, *flag),
            Command::Spell {
                ox,
                whom,
                spell,
                x,
                y,
            } => group_spell(
                built,
                it.who,
                &objects,
                *spell,
                *ox,
                *whom,
                Pos::new(*x, *y),
            ),
            Command::Guard { ox, whom, queued } => {
                group_guard(built, it.who, &objects, *ox, *whom, *queued)
            }
            _ => return false,
        };
        if n == 0 {
            done.skip(&it.cmd, "the selection acted on nothing in the simulation");
        } else {
            done.orders += 1;
        }
        true
    }

    /// The issuing player's selection, as simulation unit handles.
    ///
    /// Object numbers are per player and units start from 0
    /// (`docs/ORACLE.md`), so a selection entry is only meaningful with the
    /// issuer's `who`. Buildings — 2000 and up — are not units and drop out
    /// here; the commands that target them are counted as skipped above.
    fn selected_units(&self, play: i32, built: &Built) -> Vec<usize> {
        let slot = play as usize;
        let Some(sel) = self.selection.get(slot) else {
            return Vec::new();
        };
        sel.iter()
            .filter_map(|&o| {
                built
                    .units
                    .iter()
                    .find(|l| l.who == i64::from(play) && l.o == i64::from(o))
                    .map(|l| l.unit)
            })
            .collect()
    }
}

/// **The order kinds the recordings on disk carry** (item 4249,
/// `docs/CLIENT_INPUT.md` §11): every [`is_input`] command of every
/// recording `recgame`'s corpus reads (the bottle's 49 and the 2017 sample),
/// counted by [`Command::name`], most first. The client's `INPUT_KINDS` is
/// counted against it. Measured, and checked against the disk by
/// `the_order_kinds_on_disk`.
pub const KINDS_ON_DISK: &[(&str, usize)] = &[
    ("group", 1059),
    ("move_to", 396),
    ("queue_up", 264),
    ("gather_point", 177),
    ("chat", 160),
    ("attack", 80),
    ("leader_options", 50),
    ("build", 31),
    ("sell", 28),
    ("swarm_around", 28),
    ("pause", 24),
    ("city_gather", 20),
    ("buy", 18),
    ("spell", 17),
    ("unqueue", 13),
    ("gather", 11),
    ("guard", 10),
    ("rename_city", 10),
    ("hotkey", 8),
    ("garrison", 6),
    ("buildmask", 5),
    ("cheat_ai_toggle", 4),
    ("come_out", 4),
    ("stance", 4),
    ("disband", 1),
    ("unitmask", 1),
];

/// **The recordings' pauses** (item 4252): `(pairs, packages)` — every
/// `pause(1)` on disk has its `pause(0)` on the same frame, with this many
/// packages issued between them while the game stood paused
/// (`every_recorded_pause_holds_its_frame`).
pub const PAUSES_HELD: (usize, usize) = (12, 70);

/// Every input command of the recordings, by kind, most first (ties by
/// name): the census [`KINDS_ON_DISK`] pins.
pub fn kinds(recs: &[RecGame]) -> Vec<(&'static str, usize)> {
    let mut by: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for rec in recs {
        for it in input(rec) {
            *by.entry(it.cmd.name()).or_default() += 1;
        }
    }
    let mut out: Vec<(&'static str, usize)> = by.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The census, against the disk** (item 4249): every recording the
    /// corpus reads decodes whole, and its input commands by kind are
    /// [`KINDS_ON_DISK`]. Install-gated, and loud when the corpus is absent.
    /// **Every recorded pause holds its frame** (item 4252,
    /// `docs/CLIENT_INPUT.md` §13): each `pause(1)` the recordings on disk
    /// carry is followed, on the same frame, by the orders issued while
    /// the game stood paused, one package a turn, and then by its
    /// `pause(0)`; the next package after it is on a later frame. The shape
    /// [`crate::recgame::minted_packages`] admits. Pinned as [`PAUSES_HELD`]:
    /// (pairs, packages issued between them).
    #[test]
    fn every_recorded_pause_holds_its_frame() {
        let files = crate::recgame::tests::recordings_on_disk();
        if files.is_empty() {
            eprintln!("skipping: no .rcx on disk (set RON_RECGAME_DIR / RON_INSTALL)");
            return;
        }
        let (mut pairs, mut between) = (0, 0);
        for f in &files {
            let r = crate::recgame::read(f).unwrap();
            let states: Vec<Option<u8>> = r
                .packages
                .iter()
                .map(|p| {
                    crate::commands::decode(&p.data)
                        .unwrap()
                        .iter()
                        .find_map(|c| match c {
                            Command::Pause { state } => Some(*state),
                            _ => None,
                        })
                })
                .collect();
            let mut i = 0;
            while i < states.len() {
                if states[i] != Some(1) {
                    assert_ne!(states[i], Some(0), "{f}: an unpause with no pause");
                    i += 1;
                    continue;
                }
                let frame = r.packages[i].frame;
                let mut j = i + 1;
                while states[j] != Some(0) {
                    assert_eq!(states[j], None, "{f}: a pause inside a pause");
                    assert_eq!(r.packages[j].frame, frame, "{f}: an order while paused");
                    j += 1;
                }
                assert_eq!(r.packages[j].frame, frame, "{f}: the unpause's frame");
                assert!(r.packages[j + 1].frame > frame, "{f}: the frame after");
                pairs += 1;
                between += j - i - 1;
                i = j + 1;
            }
        }
        eprintln!("PAUSES HELD: {pairs} pairs, {between} packages between");
        assert_eq!((pairs, between), PAUSES_HELD);
    }

    #[test]
    fn the_order_kinds_on_disk() {
        let files = crate::recgame::tests::recordings_on_disk();
        if files.is_empty() {
            eprintln!("skipping: no .rcx on disk (set RON_RECGAME_DIR / RON_INSTALL)");
            return;
        }
        let recs: Vec<RecGame> = files
            .iter()
            .map(|f| crate::recgame::read(f).unwrap())
            .collect();
        for (f, r) in files.iter().zip(&recs) {
            for p in &r.packages {
                assert!(crate::commands::decode(&p.data).is_ok(), "{f}");
            }
        }
        let got = kinds(&recs);
        let total: usize = got.iter().map(|k| k.1).sum();
        eprintln!("KINDS: {} kinds, {total} orders: {got:?}", got.len());
        assert_eq!(got, KINDS_ON_DISK);
    }
}
