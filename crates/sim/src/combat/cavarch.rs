//! **Fire on the move** — `Unit::cavarch_fight@005ff4b0`, and the three
//! places the original reaches it from (`docs/AI.md` §134).
//!
//! A unit type whose `FLAGS` carry `v` (`unit_flags & 0x200000`,
//! [`uflags::FIRES_ON_MOVE`]: the Dragoon, the Horse Archer) and which has
//! an attack keeps a second target beside its order — `UnitData +0xa2`
//! `cavarch_o`, `+0xa8` `cavarch_who` and `+0xa6` `cavarch_uid` — and, on
//! every frame it steps a move order, looks for one and shoots at it
//! without stopping:
//!
//! 1. **`Unit::do_move@005f7b30:60-86`**, before anything else in the
//!    step: an unarmed type, a `HOLD_FIRE` stance (`get_combat_stance()
//!    == 5`) or an `ATTACK` action under the move (on a type without
//!    `k`, `unit_flags & 0x400`) clears `cavarch_o` to
//!    −1 and does nothing; otherwise `cavarch_fight` runs — on every frame
//!    while `cavarch_o ≥ −1`, and only on `(o + frame) % 32 == 0` once it
//!    has fallen to −2 ([`Sim::cavarch_head`]).
//! 2. **`Unit::cavarch_fight`** calls `Unit::fight(cavarch_o, cavarch_who,
//!    unit_masks2 & 0x100, 0, 1)`, whose fifth argument takes its own
//!    path through the function ([`Sim::cavarch_fight_body`]), and then
//!    turns an empty answer after an empty one into −2: the search
//!    throttle.
//! 3. **`Unit::move_step@005faf30:66-100`**, at the head of every step:
//!    a type that holds a target aims every figure at it and asks its
//!    pivots whether they bear ([`Sim::cavarch_step_anim`]); when guy 0's
//!    `des_node_flags` reads 0 the target is dropped (−1), and otherwise
//!    the step's walk animation is `CHAR_ATTACKWALK`.
//!
//! **The witness** is French East Indies' Dragoon `1/80` (TypeIndex 189,
//! `DRAGOON`, land), walking an attack-move leg to its guard post past
//! Napata in run657: −2 through block 17168; on frame 17168, `(80 +
//! 17168) % 32 == 0`, the search finds Napata — `cavarch_o 2000`,
//! `cavarch_who 0`, `near_o 2000`, and the step's `CHAR_ATTACKWALK` with
//! its turret turned; on 17169 the strike, `recharging 30` and `visible`
//! 1, with the walk unbroken. Without it this crate's `1/80` reached
//! 17171 unrecharged and swung, the word's extra draws (`Unit::fight+0x824`
//! and its swing).
//!
//! **What the strike does not do**: the shot. A fire-on-the-move type's
//! round leaves on its `CHAR_ATTACKWALK` release event (the Dragoon's
//! `RELEASEEVENT starttime="866" anim="CHAR_ATTACKWALK"`), so the frame
//! `fight` runs puts nothing in the air; run657 has no `1/80` round on
//! 17170..17178. **The release is aimed by this pair** when the current
//! order is no attack: `Guy::execute_events@005d99c0`'s `LAB_005d9c14` reads
//! `cavarch_o`/`cavarch_who` for a move order (item 1740, `docs/GOLDEN.md`
//! §65), where an attack order's own target stands first.

use crate::ai::ai_load::uflags;
use crate::combat::{Obj, Stance};
use crate::orders::{Body, flag};

impl crate::Sim {
    /// `unit_flags & 0x200000`, the `FLAGS` letter `v`.
    pub(crate) fn fires_on_move(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.flag(uflags::FIRES_ON_MOVE))
    }

    /// **`cavarch_o` as the dump prints it**: the target's own number
    /// (a unit's index, a building's `2000 +`), −1 for none, −2 while the
    /// search waits for its phase.
    pub fn cavarch_o(&self, u: usize) -> i64 {
        match self.units[u].cast_target {
            Some(Obj::Unit(t)) => i64::from(self.units[t].index),
            Some(Obj::Building(b)) => i64::from(self.buildings[b].index),
            None if self.units[u].cavarch_idle => -2,
            None => -1,
        }
    }

    /// `cavarch_o` −1: no target, and the next frame searches.
    fn cavarch_clear(&mut self, u: usize) {
        self.units[u].cast_target = None;
        self.units[u].cavarch_idle = false;
    }

    /// The pair a search answers: `cavarch_o` and `cavarch_who` both
    /// written, −1 and −1 for nothing (`find_melee_target`'s head writes
    /// `*who = −1` before it looks). The uid beside them is not carried.
    fn cavarch_take(&mut self, u: usize, found: Option<Obj>) {
        self.units[u].cavarch_idle = false;
        self.units[u].cast_target = found;
        self.units[u].cavarch_who = found.map_or(-1, |t| self.owner_of(t) as i8);
    }

    /// **`Unit::do_move@005f7b30:60-86`** — the fire on the move's gate,
    /// the first thing a step does.
    ///
    /// `(short)o + frame & 0x8000001f`, with the signed fix-up the compiler
    /// lays after it, is `% 32` taken the C way; both terms are
    /// non-negative here.
    ///
    /// **The `ATTACK` action's exemption** (`005f7b30:68-72`): a type with
    /// `unit_flags & 0x400` ([`uflags::MELEE_AND_RANGED`], `k`, the
    /// two-mode cavalry archer of `docs/COMBAT.md` §8.5) is not cleared by
    /// an attack under its move; it goes on to the phase and the fight, at
    /// its second range ([`Self::with_second_range`], item 3383). The three
    /// Indian Mahouts carry `k` (`mjvyk`): run1181's `1/27` and `1/46`
    /// (TypeIndex 262) keep −2 on 4001 under `[AttackTo, Attack(0/10),
    /// Move]`, where ours read −1 before item 3554 (`docs/AI.md` §308).
    pub(crate) fn cavarch_head(&mut self, u: usize, frame: i64) {
        if !self.fires_on_move(u) || self.attack_of(Obj::Unit(u)) == 0 {
            return;
        }
        let attacking = self.attack_action_clears(u);
        if self.units[u].combat.stance == Stance::HoldFire || attacking {
            self.cavarch_clear(u);
            return;
        }
        if self.units[u].cavarch_idle && (i64::from(self.units[u].index) + frame) % 32 != 0 {
            return;
        }
        self.cavarch_fight(u, frame);
    }

    /// **`Unit::do_group_move@005e79a0:392`–`406`** — a formation
    /// follower's fire on the move, just before its `move_step` (item 3336,
    /// `docs/AI.md` §272). The same test as [`Self::cavarch_head`]'s, with
    /// one difference the listing keeps: a `HOLD_FIRE` stance or an
    /// `ATTACK` action does not clear `cavarch_o` here — the call is
    /// skipped and the pair kept. Great Sahara's Royal Mameluke `1/68`
    /// (TypeIndex 198) on tick 20186, a follower of a `GROUP_ATTACK_TO`
    /// given that frame: theirs spends `fight`'s re-search roll on the
    /// Citizen `0/4` it held (run967g).
    ///
    /// The `unit_flags & 0x400` exemption is the head's (`005e79a0:395-396`).
    pub(crate) fn cavarch_follower(&mut self, u: usize, frame: i64) {
        if !self.fires_on_move(u) || self.attack_of(Obj::Unit(u)) == 0 {
            return;
        }
        let attacking = self.attack_action_clears(u);
        if self.units[u].combat.stance == Stance::HoldFire || attacking {
            return;
        }
        if self.units[u].cavarch_idle && (i64::from(self.units[u].index) + frame) % 32 != 0 {
            return;
        }
        self.cavarch_fight(u, frame);
    }

    /// An `ATTACK` action under the move, on a type without `k`
    /// (`unit_flags & 0x400`): the arm both callers skip the fight on.
    fn attack_action_clears(&self, u: usize) -> bool {
        let melee_and_ranged = self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t]
                .cols
                .flag(uflags::MELEE_AND_RANGED)
        });
        !melee_and_ranged
            && self
                .action_of(u)
                .is_some_and(|i| matches!(self.units[u].orders[i].body, Body::Attack(_)))
    }

    /// **`Unit::cavarch_fight@005ff4b0`**: the fight, and then the
    /// throttle — an answer of nothing after an answer of nothing (the old
    /// `cavarch_o` read as unsigned past `0x7fff`) is written −2.
    ///
    /// SEAM: `unit_masks2 & 0x100`, `fight`'s third argument and the bit
    /// `cavarch_fight` clears when the target changes. `Unit::do_attack@
    /// 005f1b80:226-240` sets it, with the pair, under a **mandatory**
    /// attack of a `v` type; this crate carries neither, so the third
    /// argument is 0 and the squad arm it opens (`fight:206-215`) is not
    /// reached.
    pub(crate) fn cavarch_fight(&mut self, u: usize, frame: i64) {
        let was_empty = self.units[u].cast_target.is_none();
        // **A melee-and-ranged type fights here at its second range**
        // (`cavarch_fight@005ff4b0`'s bracket, item 3383): the Gun Mahout
        // (`GUNMAHOUT`, `mjvyk`) melees by `max_range` 0 and keeps its 9
        // in `second_max_range`, which this call alone reads — without it
        // the search's must-reach gate refuses everything past melee.
        self.with_second_range(u, |s| s.cavarch_fight_body(u, frame));
        if was_empty && self.units[u].cast_target.is_none() {
            self.units[u].cavarch_idle = true;
        }
    }

    /// `fight@005fd4d0:394`–`400`'s `local_14`, the captain's re-search of
    /// a unit target, whichever way `fight` is reached: a unit with
    /// `unit_masks2 & 4` (a Bark, item 2165) re-finds a target whose role
    /// is combat and whose type is not `n` (`unit_flags & 0x2000`), any
    /// other one whose role is not combat. The fire on the move took only
    /// the second until item 3516: run1161b's Cruiser `1/25` (a Bark,
    /// `unit_masks2` 4) keeps the citizen `0/5` on 3681 and fires, where
    /// ours re-found `0/11`.
    pub(crate) fn captain_research_admits(&self, u: usize, t: usize) -> bool {
        let p = self.profile(Obj::Unit(t));
        if self.units[u].unit_masks2 & crate::combat::umask2::MASK2_BARK != 0 {
            p.combat_role
                && self.units[t]
                    .ty
                    .is_none_or(|ty| self.tables.unit_types[ty].cols.unit_flags & 0x2000 == 0)
        } else {
            !p.combat_role
        }
    }

    /// **`Unit::fight(o, who, 0, 0, 1)`** — `fight@005fd4d0` along its
    /// `param_5` path, which never touches the order list:
    ///
    /// - `:102-104`: a recharging unit returns at once (no snap, no idle).
    /// - `:196-229`: an invalid target writes −1; `LAB_005fdb9e`, the
    ///   search budget's arm, then runs `find_melee_target(−1, &who, 1, 0,
    ///   0)` and writes what it finds, and counts one search on the
    ///   leader (`field_0x9f4`, [`crate::Sim::retargets`]).
    /// - `:337-366`: no guard's leash (`param_5 == 0` only).
    /// - `:367-461`: **a captain re-searches** whatever it holds that is
    ///   not a unit — and, for a unit, when
    ///   [`Sim::captain_research_admits`] does, unless `Random::get % 5 == 0` or
    ///   the current order carries `flags & 0x10` (the roll is the same
    ///   draw `do_attack`'s captain takes, `SITE_FIGHT_RESEARCH`). A
    ///   different answer is written and the call returns; the same one
    ///   goes on to the range.
    /// - `:462-469`, `:1037-1046`: out of range, the search again, written.
    /// - in range: the pair is written (`:585-592`), `set_attack`, and
    ///   [`Sim::fight_as`]'s cavarch strike — angle, `set_attacking`, the
    ///   damage arm and the reload — with no `set_angle`, snap or swing.
    ///
    /// - `LAB_005fdb9e`'s first arm, `waiting < 5 && retargets > 10` (the
    ///   search budget, item 3598): 2 added to `waiting` and a return,
    ///   with no search, no count, and — `param_5` set — no frozen mark.
    ///
    /// SEAM: `unit_masks |= 0x11000` before the strike, as for every
    /// strike here.
    pub(crate) fn cavarch_fight_body(&mut self, u: usize, frame: i64) {
        let me = Obj::Unit(u);
        if self.units[u].combat.recharging != 0 {
            return;
        }
        let who = usize::from(self.units[u].owner);
        let Some(target) = self.units[u]
            .cast_target
            .filter(|&t| self.valid_target(me, t))
        else {
            self.cavarch_clear(u);
            if self.units[u].waiting < 5 && self.combat.retargets[who] > 10 {
                self.units[u].waiting += 2;
                return;
            }
            let found = self.find_melee_target_as(u, -1, 0, true);
            self.cavarch_take(u, found);
            self.combat.retargets[who] += 1;
            return;
        };
        if self.units[u].combat.captain == i32::from(self.units[u].index) {
            let research = match target {
                Obj::Building(_) => true,
                Obj::Unit(t) => {
                    self.mark(crate::combat::fight::SITE_FIGHT_RESEARCH);
                    let roll = self.rng.roll();
                    let reentry = self
                        .current_order(u)
                        .is_some_and(|o| o.flags & flag::FIGHT_REENTRY != 0);
                    self.captain_research_admits(u, t) && roll % 5 != 0 && !reentry
                }
            };
            if research {
                let found = self.find_melee_target_as(u, -1, 0, true);
                if found != Some(target) {
                    self.cavarch_take(u, found);
                    return;
                }
            }
        }
        if !self.is_in_range(me, target) {
            let found = self.find_melee_target_as(u, -1, 0, true);
            self.cavarch_take(u, found);
            return;
        }
        self.cavarch_take(u, Some(target));
        self.fight_as(u, target, frame, true);
    }

    /// **`Unit::move_step@005faf30:66-100`** — the step's walk slot for a
    /// fire-on-the-move type: every figure is aimed at `cavarch_o` (the
    /// guy's `ox`/`whom`) and its pivots asked (`Guy::set_all_pivots`);
    /// then guy 0's `des_node_flags` (`+0x98`) decides. Zero — the target
    /// is not in any node's arc — drops it (`cavarch_o` −1) and puts every
    /// figure's first turret back on the bow (`des_turret_angles[0]` 0,
    /// `des_node_flags |= 1`, and `node_flags |= 1` where the turret is
    /// already within 15°), and the walk is `CHAR_WALK`; non-zero walks
    /// `CHAR_ATTACKWALK`.
    pub(crate) fn cavarch_step_anim(&mut self, u: usize) -> i8 {
        let Some(target) = self.units[u].cast_target else {
            return crate::units::anim::WALK;
        };
        if !self.fires_on_move(u) {
            return crate::units::anim::WALK;
        }
        let n = self.units[u].guys.len().min(crate::units::anim::SQUAD_SIZE);
        for g in 0..n {
            self.units[u].guys[g].aim = Some(target);
            self.set_all_pivots(u, g, Some(target));
        }
        if self.units[u]
            .guys
            .first()
            .is_some_and(|g| g.turret.des_flags == 0)
        {
            self.units[u].cast_target = None;
            for g in &mut self.units[u].guys[..n] {
                let t = &mut g.turret;
                t.des[0] = 0;
                t.des_flags |= 1;
                // `5fb039`–`5fb05b`: `neg`, then `not` above `0x80000000`
                // unsigned — the turret's distance from the bow, folded.
                let neg = t.angles[0].wrapping_neg() as u32;
                let off = if neg > 0x8000_0000 { !neg } else { neg };
                if off < 0x0aaa_aaaa {
                    t.node_flags |= 1;
                }
            }
            return crate::units::anim::WALK;
        }
        crate::units::anim::ATTACKWALK
    }
}

#[cfg(test)]
mod tests;
