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
//!    == 5`) or an `ATTACK` action under the move clears `cavarch_o` to
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
//! `RELEASEEVENT starttime="866" anim="CHAR_ATTACKWALK"`), aimed by the
//! *order*'s target as every release is (`docs/COMBAT.md` §9.0), so the
//! frame `fight` runs puts nothing in the air; run657 has no `1/80` round
//! on 17170..17178.

use crate::ai_load::uflags;
use crate::combat::{Obj, Stance};
use crate::orders::{Body, flag};

impl crate::Sim {
    /// `unit_flags & 0x200000`, the `FLAGS` letter `v`.
    pub(crate) fn fires_on_move(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag(uflags::FIRES_ON_MOVE))
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
    /// SEAM: the `ATTACK` action's exemption for a type with `unit_flags &
    /// 0x400` (the two-mode cavalry archer of `docs/COMBAT.md` §8.5), and
    /// `cavarch_fight`'s `max_range` swap for it. No shipped `v` type
    /// carries `k`.
    pub(crate) fn cavarch_head(&mut self, u: usize, frame: i64) {
        if !self.fires_on_move(u) || self.attack_of(Obj::Unit(u)) == 0 {
            return;
        }
        let attacking = self
            .action_of(u)
            .is_some_and(|i| matches!(self.units[u].orders[i].body, Body::Attack(_)));
        if self.units[u].combat.stance == Stance::HoldFire || attacking {
            self.cavarch_clear(u);
            return;
        }
        if self.units[u].cavarch_idle && (i64::from(self.units[u].index) + frame) % 32 != 0 {
            return;
        }
        self.cavarch_fight(u, frame);
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
    fn cavarch_fight(&mut self, u: usize, frame: i64) {
        let was_empty = self.units[u].cast_target.is_none();
        self.cavarch_fight_body(u, frame);
        if was_empty && self.units[u].cast_target.is_none() {
            self.units[u].cavarch_idle = true;
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
    ///   not a unit — and, for a unit, unless `Random::get % 5 == 0` or
    ///   the current order carries `flags & 0x10` (the roll is the same
    ///   draw `do_attack`'s captain takes, `SITE_FIGHT_RESEARCH`). A
    ///   different answer is written and the call returns; the same one
    ///   goes on to the range.
    /// - `:462-469`, `:1037-1046`: out of range, the search again, written.
    /// - in range: the pair is written (`:585-592`), `set_attack`, and
    ///   [`Sim::fight_as`]'s cavarch strike — angle, `set_attacking`, the
    ///   damage arm and the reload — with no `set_angle`, snap or swing.
    ///
    /// SEAM: `LAB_005fdb9e`'s first arm, `waiting < 5 && retargets > 10`
    /// (the search budget), which adds 2 to `waiting` and returns without
    /// searching or counting — as for `do_attack`'s, not modelled.
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
            let found = self.find_melee_target_as(u, -1, 0, true);
            self.cavarch_take(u, found);
            self.retargets[who] += 1;
            return;
        };
        if self.units[u].combat.captain == i32::from(self.units[u].index) {
            let research = match target {
                Obj::Building(_) => true,
                Obj::Unit(t) => {
                    self.mark(crate::fight::SITE_FIGHT_RESEARCH);
                    let roll = self.rng.roll();
                    let reentry = self
                        .current_order(u)
                        .is_some_and(|o| o.flags & flag::FIGHT_REENTRY != 0);
                    !self.profile(Obj::Unit(t)).combat_role && roll % 5 != 0 && !reentry
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
            return crate::anim::WALK;
        };
        if !self.fires_on_move(u) {
            return crate::anim::WALK;
        }
        let n = self.units[u].guys.len().min(crate::anim::SQUAD_SIZE);
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
            return crate::anim::WALK;
        }
        crate::anim::ATTACKWALK
    }
}

#[cfg(test)]
mod tests {
    use crate::ai_load::uflags;
    use crate::combat::{Obj, Profile};
    use crate::orders::{Body, MoveKind, QueuePos, index};
    use crate::world::World;
    use crate::{Player, Pos, Sim};

    /// Two players at war on open ground, and a Dragoon-shaped type: an
    /// armed, ranged combat unit whose `FLAGS` carry `v` when `fires`.
    fn field(fires: bool) -> (Sim, usize) {
        let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        sim.nation[0].human = true;
        sim.nation[1].human = true;
        let mut t = crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 17,
                max_range: 9,
                recharge: 30,
                uber_size: 1,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        };
        if fires {
            t.cols.unit_flags |= uflags::FIRES_ON_MOVE;
        }
        let ty = sim.add_unit_type(t);
        (sim, ty)
    }

    fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = crate::Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.unit_types[ty].kind;
        let i = sim.add_unit(u);
        sim.units[i].guys.push(crate::anim::Guy::fresh(-1));
        i
    }

    /// **A `v` type walking past an enemy finds it, then shoots it, and
    /// walks on** (`docs/AI.md` §134, run657's `1/80` on 17168 and
    /// 17169). The first frame's search writes the pair (`cavarch_o`,
    /// `cavarch_who`) and spends no reload; the second, in range, is the
    /// strike: the reload is set and the move is still the only order.
    /// Without `v` the gate does nothing at all.
    ///
    /// Made to fail first with [`Sim::cavarch_head`]'s call removed from
    /// `do_move`: run657 kept 71 rows, `1/80` unrecharged on 17170.
    #[test]
    fn a_unit_that_fires_on_the_move_finds_shoots_and_walks_on() {
        for fires in [true, false] {
            let (mut sim, ty) = field(fires);
            let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
            let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
            sim.add_move_order(
                me,
                Pos::new(9000, 3000),
                MoveKind::AttackTo,
                QueuePos::New,
                false,
            );
            let frame = 64 - i64::from(sim.units[me].index);
            sim.cavarch_head(me, frame);
            if !fires {
                assert_eq!(sim.cavarch_o(me), -1, "no `v`, no search");
                assert_eq!(sim.units[me].near, None);
                continue;
            }
            assert_eq!(
                sim.units[me].cast_target,
                Some(Obj::Unit(foe)),
                "the search"
            );
            assert_eq!(sim.units[me].cavarch_who, 0);
            assert_eq!(
                sim.units[me].combat.recharging, 0,
                "a search does not shoot"
            );
            sim.cavarch_head(me, frame + 1);
            assert_eq!(sim.units[me].combat.recharging, 30, "the strike's reload");
            assert_eq!(sim.cavarch_o(me), i64::from(sim.units[foe].index));
            assert_eq!(sim.units[me].orders.len(), 1, "the walk is untouched");
            assert_eq!(sim.order_type(me), index::ATTACK_TO);
        }
    }

    /// **An empty search after an empty one waits for the phase**
    /// (`cavarch_fight@005ff4b0`'s tail, `do_move@005f7b30:66`): −1, then
    /// −2, and from −2 a search runs only on `(o + frame) % 32 == 0`.
    /// The one that does run finds a foe that walked in.
    #[test]
    fn an_empty_search_twice_waits_for_the_thirty_two_frame_phase() {
        let (mut sim, ty) = field(true);
        let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
        sim.add_move_order(
            me,
            Pos::new(9000, 3000),
            MoveKind::AttackTo,
            QueuePos::New,
            false,
        );
        let o = i64::from(sim.units[me].index);
        let phase = 64 - o;
        sim.cavarch_head(me, phase + 1);
        assert_eq!(sim.cavarch_o(me), -2, "an empty answer after −1 at init");
        let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
        sim.cavarch_head(me, phase + 2);
        assert_eq!(sim.cavarch_o(me), -2, "off the phase, no search");
        sim.cavarch_head(me, phase + 32);
        assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(foe)), "on it");
    }

    /// **The step walks under `CHAR_ATTACKWALK` while the pivots bear,
    /// and drops the target when they do not** (`move_step@005faf30:66-100`).
    /// The turret's arc is ±90° off the bow: a foe on the bow bears; one
    /// behind does not, and the target goes (−1) with the walk.
    #[test]
    fn the_step_attack_walks_while_the_turret_bears() {
        for (at, bears) in [(Pos::new(3000, 4200), true), (Pos::new(3000, 1800), false)] {
            let (mut sim, ty) = field(true);
            let ti = sim.unit_types[ty].type_index;
            sim.art
                .pivots
                .insert(ti, [(4, (-90, 90))].into_iter().collect());
            let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
            let foe = put(&mut sim, 0, ty, at);
            sim.units[me].movement.facing = crate::movement::find_angle(0, 1);
            sim.units[me].cast_target = Some(Obj::Unit(foe));
            let walk = sim.cavarch_step_anim(me);
            assert_eq!(sim.units[me].guys[0].aim, Some(Obj::Unit(foe)), "aimed");
            if bears {
                assert_eq!(walk, crate::anim::ATTACKWALK);
                assert_eq!(sim.units[me].cast_target, Some(Obj::Unit(foe)));
            } else {
                assert_eq!(walk, crate::anim::WALK);
                assert_eq!(sim.cavarch_o(me), -1, "dropped");
                assert_eq!(sim.units[me].guys[0].turret.des_flags & 1, 1);
            }
        }
    }

    /// **A guard's attack-move leg goes under the attack its look finds**
    /// (`find_nearby_target@00648da0:551`, `local_2c`; run657's `1/69` on
    /// 17166: `[Attack, Guard]`, path 0). A bare attack-move keeps its
    /// place beneath (`QUEUE_FIRST` over it), which is run658's `1/90` on
    /// 17040.
    ///
    /// Made to fail first with the kill removed from
    /// [`Sim::nearby_add`]: run657's `1/69` rows on 17167.
    #[test]
    fn a_guards_attack_move_leg_goes_under_the_attack_it_finds() {
        for guarding in [true, false] {
            let (mut sim, ty) = field(false);
            let post = put(&mut sim, 1, ty, Pos::new(3000, 2000));
            let me = put(&mut sim, 1, ty, Pos::new(3000, 3000));
            let foe = put(&mut sim, 0, ty, Pos::new(3000, 4200));
            if guarding {
                sim.add_guard_order(me, post, 0, 372, QueuePos::New);
                sim.add_move_order(
                    me,
                    Pos::new(3000, 2400),
                    MoveKind::AttackTo,
                    QueuePos::First,
                    false,
                );
            } else {
                sim.add_move_order(
                    me,
                    Pos::new(3000, 2400),
                    MoveKind::AttackTo,
                    QueuePos::New,
                    false,
                );
            }
            assert_eq!(sim.order_type(me), index::ATTACK_TO);
            sim.nearby_add(me, Obj::Unit(foe));
            let kinds: Vec<_> = sim.units[me].orders.iter().map(|o| o.index()).collect();
            if guarding {
                assert_eq!(kinds, [index::ATTACK, index::GUARD], "the leg is killed");
            } else {
                assert_eq!(kinds, [index::ATTACK, index::ATTACK_TO], "kept beneath");
            }
            assert!(matches!(sim.units[me].orders[0].body, Body::Attack(_)));
        }
    }
}
