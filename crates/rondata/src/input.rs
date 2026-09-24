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
//! unit is at gamelog frame 377.

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
        let Ok(cmds) = crate::commands::decode(&p.data) else {
            continue;
        };
        for cmd in cmds {
            if is_input(&cmd) {
                out.push(Input {
                    frame: p.frame,
                    play: p.play,
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
    let mut g = sim::group::Group::stack(player);
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
    if g.list.is_empty() || !built.sim.push_group(&mut g, true) {
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
    built.sim.group_action_move_to(
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
    let mut g = sim::group::Group::stack(player);
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
    if g.list.is_empty() || !built.sim.push_group(&mut g, true) {
        return 0;
    }
    built.sim.group_action_patrol(&g, to, queue_pos(queued));
    g.list.len()
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

/// The stream, with the selection state the commands assume.
///
/// A `group` command sets the issuing player's selection and the
/// unit-targeted commands that follow act on it; `num = 0` — an empty
/// `objects` — means "the same selection as this player's previous group
/// command" (`docs/COMMANDS.md` §3). The selection is therefore *state*, and
/// it has to be carried across frames exactly as the engine carries it.
#[derive(Clone)]
pub struct Stream {
    input: Vec<Input>,
    next: usize,
    selection: Vec<Vec<i16>>,
}

impl Stream {
    pub fn new(rec: &RecGame) -> Stream {
        Stream {
            input: input(rec),
            next: 0,
            selection: vec![Vec::new(); 8],
        }
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
            }

            Command::MoveTo {
                to_x, to_y, queued, ..
            } => {
                let to = Pos::new(*to_x, *to_y);
                let pos = queue_pos(i32::from(*queued));
                let mut any = false;
                for u in self.selected_units(it.play, built) {
                    // `process_move_to` → `Group::action_move_to(..., 1, ...)`:
                    // the action bit is set (`docs/ORDERS.md` §14's table).
                    built.sim.add_move_order(u, to, MoveKind::MoveTo, pos, true);
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
            Command::QueueUp { .. } | Command::Unqueue { .. } => {
                done.skip(&it.cmd, "production is not wired into the harness");
            }
            Command::Buy { .. } | Command::Sell { .. } => {
                done.skip(&it.cmd, "the market is not modelled");
            }
            Command::GatherPoint { .. } => {
                done.skip(&it.cmd, "rally points are not modelled");
            }
            Command::Chat { .. } => {
                // A cheat line. It is input — in solo it travels the order
                // stream — but its effect is `ConsoleWin::run_cmd`'s, not an
                // order's, and staging cheats are outside the simulation.
                done.skip(&it.cmd, "a cheat line, not an order");
            }
            _ => done.skip(&it.cmd, "not mapped"),
        }
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
