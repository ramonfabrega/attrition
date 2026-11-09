//! `Group::action_move_near`'s path — `docs/GROUPS.md` §6.7.
//!
//! The tail of the group move, and the piece the simulation went without
//! until 2026-08-26. It matters for three reasons at once, all of them
//! measurable on run20's unit `1/0`:
//!
//! - **the goal**. The bottom of a group member's path stack is the
//!   *leader's raw slot destination*, un-snapped — `form +0x514/+0x714` read
//!   straight into the `PathData` at `70612f`–`706137`. The order's own
//!   `dest` went through `add_move_facing_order`'s `u × 0x30 + 0x18` snap
//!   and is not the same number.
//! - **the timing**. The plan happens inside `action_move_near`, on the
//!   frame the order is issued, so the order is born `PATHED` with a full
//!   stack. Leaving it to each member's own `do_move` costs a frame and
//!   plans from wherever the member has drifted to.
//! - **the route**. Every member walks the *leader's* chain, translated by
//!   `slot[i] − slot[leader]`. Only the leader's own search ever runs.
//!
//! Nothing in here is a fresh reading of the pathfinder: it is
//! `PathFinder::find_wpath` (`docs/PATHFINDER.md` §3) called once, on a
//! stack that is not a unit's, from a start that is not the unit's own
//! position.

use crate::attrition::Domain;
use crate::form::Form;
use crate::group::{Group, Member};
use crate::orders::{MoveKind, PathData, path_flag};
use crate::world::{Pos, UNITS_PER_CELL, vector_dist};

use crate::Sim;

/// `Form::compute`'s formation ids, the two this section keys on
/// (`docs/GROUPS.md` §6.4: `rules.xml` 1446–1487, in document order).
const FORM_COLUMN: i32 = 8;
const FORM_MOB: i32 = 9;

/// Under this, the group plans nothing beyond the goal itself
/// (`70616d: cmpl $0x900`).
const NO_PLAN_WITHIN: i32 = 0x900;

/// A follower that is *not* in the formation still gets the leader's
/// intermediate waypoints, but only while it is this close
/// (`706845: cmpl $0x600`).
const FOLLOWER_REACH: i32 = 0x600;

impl Sim {
    /// §6.7 whole: plan one path off the leader's slot, then hand every
    /// member the same chain shifted by its own offset.
    ///
    /// `from` is `GroupData::get_loc`'s answer, which `action_move_near`
    /// took before the layout; `formation` is the resolved form index
    /// (`local_38`); `members` is the per-member decision the order loop
    /// already made (§6.5), which is the same predicate this loop
    /// re-derives — see [`Sim::group_plan_takes`].
    pub(crate) fn group_plan_path(
        &mut self,
        g: &Group,
        f: &Form,
        members: &[Member],
        kind: MoveKind,
        formation: i32,
        from: Pos,
    ) {
        // `Group::action_move_to@0070fba0` is the only caller the
        // simulation has, and it is one line: `action_move_near(this, x, y,
        // tolerance 0, …)` (§2). The tolerance rides on every entry pushed
        // below, and is what run20's dump prints as the goal's
        // `tolerance 0`.
        const TOLERANCE: i32 = 0;

        let Some(base) = f.to.get(f.idx).copied() else {
            return;
        };
        // The plan is gated on the leader being active and on the map —
        // and on *nothing else*: the plane test the member loops make is
        // not asked here (`706128` onwards).
        let leader = self
            .group_find_leader(g)
            .filter(|&u| self.units[u].alive() && self.units[u].on_map);

        // `local_50`: a move whose goal is already near enough plans no
        // route at all, so the stack is the goal and nothing else. It also
        // switches off both of the corrections below, which is why it is
        // carried rather than recomputed.
        let mut short = false;
        let mut chain: Vec<PathData> = Vec::new();
        if let Some(l) = leader {
            // The start is the leader's **top of stack** when it has one,
            // and the group's location otherwise. After a `QUEUE_NEW`
            // clear it never has one; after a `QUEUE_LAST` it does, and the
            // group's new legs are planned from the end of the old.
            let start = self.units[l].path.last().map_or(from, |p| p.to);
            let goal = base;
            chain.push(PathData {
                to: goal,
                tolerance: TOLERANCE,
                flags: path_flag::FINAL,
            });
            if vector_dist(start.x - goal.x, start.y - goal.y) < NO_PLAN_WITHIN {
                short = true;
            } else {
                // The static `grouppath` is not a unit's stack. Install it
                // on the leader for the call — `find_wpath` reads the
                // object for its modes, its `toff` and its `calc_cost`, and
                // the original passes exactly that object — then take it
                // back, leaving the leader's own stack as it was.
                let saved = std::mem::replace(&mut self.units[l].path, chain);
                self.find_wpath_from(l, start, g.army.is_some() || self.slot_keeps_army_link(g));
                chain = std::mem::replace(&mut self.units[l].path, saved);
            }
        }

        // `if (iVar30 == 0)`: no leader, or a search that cleared the stack
        // (an off-map goal returns −1 with nothing on it). Then every
        // member plans its own.
        let (Some(leader), false) = (leader, chain.is_empty()) else {
            self.group_plan_each(g, f, members, from, TOLERANCE);
            return;
        };

        let human = self.cities_state.nation[g.who as usize].human;
        while let Some(w) = chain.pop() {
            let lpos = self.units[leader].pos;
            let last = w.flags & path_flag::FINAL != 0;
            for i in 0..g.list.len() {
                let u = g.list[i];
                if !Self::group_plan_takes(members, i) {
                    continue;
                }
                // An **AI**'s sea member takes the final waypoint only,
                // unless it is the leader (`706473`–`7064d5`). A human's
                // navy is exempt: the test opens on `leaders & 4`.
                if !human && self.group_domain(u) == Domain::Sea && !last && u != leader {
                    continue;
                }
                // The offset, off the leader's slot. SEAM, and the
                // original's: for **Column** on a non-final waypoint of a
                // planned move the index is `cols[i]` (`7064db`–`706507`) —
                // a function-static `SimpleArray<int>` at `0xee155c`, sized
                // to `num` on every call and **written by nothing**, so the
                // original reads uninitialised heap and lands on an
                // arbitrary member's slot. `formation` is 0 in every
                // simulated group; there is no defensible number to
                // reproduce, so the member's own slot stands.
                debug_assert!(
                    formation != FORM_COLUMN || last || short,
                    "a Column's non-final waypoint indexes an uninitialised `cols`"
                );
                let slot = f.to[i];
                let p =
                    self.restrict_pos(Pos::new(w.to.x + slot.x - base.x, w.to.y + slot.y - base.y));
                let p = self.group_plan_keep_area(u, p, w.to, short);
                // The leader walks the whole chain and so does every final
                // waypoint. An intermediate leg reaches a follower only
                // when that follower is **not** in the formation — the
                // `GroupMoveOrder` gate of §6.6 step 6, minus its
                // `ATTACK_TO` arm — and is still within `0x600` of the
                // leader. A group-move follower is steered by
                // `do_group_move` instead, off the leader's own position.
                let alone = kind != MoveKind::MoveTo
                    || self.is_modern_infantry(u)
                    || self.group_domain(u) == Domain::Sea
                    || formation == FORM_MOB;
                let near = vector_dist(self.units[u].pos.x - lpos.x, self.units[u].pos.y - lpos.y)
                    < FOLLOWER_REACH;
                if u == leader || last || (alone && near) {
                    self.units[u].path.push(PathData {
                        to: p,
                        tolerance: w.tolerance,
                        flags: w.flags,
                    });
                }
            }
        }

        // The tail invert, and it asks **none** of the questions the
        // waypoint loop asked: active, on the map, not a plane, and that is
        // all (`706909`–`7069a8`). A member the move stabled in a city, or
        // left shooting, has its own untouched stack turned over with
        // everyone else's.
        for i in 0..g.list.len() {
            let u = g.list[i];
            if self.units[u].alive() && self.units[u].on_map && !self.is_plane(u) {
                self.units[u].path.reverse();
            }
        }
    }

    /// Whether a member is in the path loop at all.
    ///
    /// The original re-derives this rather than remembering it — the
    /// active/on-map/plane triple, then §6.5's AI arms — and it re-reads
    /// `order_type()` *after* the order loop has replaced it. The answer is
    /// the same one either way: a member the order loop cleared and gave a
    /// move to now reads `MOVE_TO` and passes the shooting-siege test it
    /// would otherwise fail, and a member the order loop skipped still
    /// reads `ATTACK` and fails it. So the decision the order loop already
    /// made ([`Member`]) is the decision here, and reading it twice would
    /// only invite the two to drift apart.
    fn group_plan_takes(members: &[Member], i: usize) -> bool {
        matches!(members.get(i), Some(Member::Move))
    }

    /// The terrain guard on a translated waypoint (`7065c0`–`70678e`).
    ///
    /// A slot offset can push a member's waypoint across a coastline. When
    /// the **area id** of its cell differs from the leader's waypoint's —
    /// `WorldData +0x134`'s `+0x4` land, or `+0x6` water when the cell is
    /// `HALFLAND 0x100` and the tile's own surface is ocean, which is
    /// [`crate::world::World::tregion_alt`] — *and* the member could not
    /// stand there, the waypoint is pulled back: to the leader's own point
    /// on a short move, and otherwise into the leader's **cell**, keeping
    /// the offset the member had inside its own.
    ///
    /// Ghidra renders the `x` half of that last step with a `0xc0` stride
    /// against `y`'s `0x300`, which would make it an asymmetric original
    /// bug worth reproducing. It is not one: `70672d`–`706764` is
    /// `leal (%eax,%eax,2)` then `shll $0x8` — × 3 × 256 — on **both**
    /// axes, and the `imull $0x2aaaaaab` / `sarl $0x7` pair either side of
    /// it is a signed divide by `0x300`, not by `0xc0`.
    fn group_plan_keep_area(&self, u: usize, p: Pos, w: Pos, short: bool) -> Pos {
        if self.world.tregion_alt(p.tile()) == self.world.tregion_alt(w.tile()) {
            return p;
        }
        if self.invalid_loc(u, p.tile(), true, true, false, false, false) == 0 {
            return p;
        }
        if short {
            return w;
        }
        Pos::new(
            p.x + (w.x / UNITS_PER_CELL - p.x / UNITS_PER_CELL) * UNITS_PER_CELL,
            p.y + (w.y / UNITS_PER_CELL - p.y / UNITS_PER_CELL) * UNITS_PER_CELL,
        )
    }

    /// §6.7's fallback: with no group path, every member plans its own
    /// single-entry path to its own slot (`7069b4`–`706b76`).
    ///
    /// The `pathfinder +0x70` hint is **not** set here — this arm is not an
    /// army's plan, it is what is left when the leader has gone.
    fn group_plan_each(
        &mut self,
        g: &Group,
        f: &Form,
        members: &[Member],
        from: Pos,
        tolerance: i32,
    ) {
        for i in 0..g.list.len() {
            let u = g.list[i];
            if !Self::group_plan_takes(members, i) {
                continue;
            }
            let goal = PathData {
                to: f.to[i],
                tolerance,
                flags: path_flag::FINAL,
            };
            let start = self.units[u].path.last().map_or(from, |p| p.to);
            let saved = std::mem::replace(&mut self.units[u].path, vec![goal]);
            self.find_wpath_from(u, start, false);
            let mut chain = std::mem::replace(&mut self.units[u].path, saved);
            if chain.is_empty() {
                // The search refused: the goal goes on by itself, off the
                // copy `action_move_near` kept before the call.
                self.units[u].path.push(goal);
            } else {
                while let Some(e) = chain.pop() {
                    self.units[u].path.push(e);
                }
            }
            self.units[u].path.reverse();
        }
    }
}

#[cfg(test)]
mod tests;
