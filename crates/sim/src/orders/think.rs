//! `Unit::work` and `Unit::think`: the per-frame dispatch, idle, patrol, guard, follow and the think order (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // The step — `Unit::work` → `do_job`
    // ------------------------------------------------------------------

    /// `Unit::work` (§2.3): the liveness test on the action's target, the
    /// idle reset, then one `do_job` on the front order.
    pub(crate) fn work(&mut self, u: usize, frame: i64) {
        // **`ObjectData::visible`'s clock**, and it is the first thing
        // `Unit::work@0060d180:63` does — above every gate below, so a
        // unit that returns early still keeps it
        // (`docs/VISION.md` §7).
        //
        // One frame in 32, phased by `o`, a unit that is **not** carrying
        // the attack latch forgets everyone it has made itself visible to.
        // The latch is dropped on any frame the front order is not
        // `ATTACK`, and the test above reads it *before* that drop — so
        // the frame a unit stops attacking still counts as attacking, and
        // the visibility outlives the order by up to 32 frames.
        // **The recharging-melee gate** — a recharging melee unit steps only an
        // order carrying the action bit, never an ATTACK through here, and an
        // orderless one steps nothing (`:79-86`; the battery word 13054,
        // `docs/AI.md` §245: the Knight `1/65` stands orderless at `idle` 0
        // until the countdown ends). A type with `unit_flags & 0x400` ("melee
        // AND ranged") has no gate (parked 3106). An animal has none:
        // `Animal::work@005d7330` is its own entry. The steps — the clock,
        // the latch drop and the gate — are [`crate::work_gate::work_gate_steps`],
        // swept to the first call after them (item 3348).
        if crate::work_gate::work_gate_steps(&mut crate::work_gate::SimWorkGate::new(
            self, u, frame,
        )) == crate::work_gate::Gate::Return
        {
            return;
        }
        // Target liveness: a build, repair, garrison or gather whose target's
        // **slot was reused** dies before its `do_*` ever runs. It is the
        // `uid` test (`Unit::work@0060d180:319`, the action's target uid
        // against the object's `+0x30`), and a closed building keeps its
        // uid until `Objects::find_free` hands its number to the next one.
        // So a dead target alone is not enough: the order stands, the walk
        // under it goes on, and `do_build`/`do_gather`'s own `exists`
        // tests end it on arrival. run157's `1/7` keeps its `BUILDORDER`
        // on the Woodcutter's Camp the script placed and destroyed on 1176
        // to the end of the capture (item 644). This crate's handles are
        // never reused, so "reused" is a later building carrying the same
        // owner and number.
        if let Some(a) = self.action_of(u) {
            let dead = match self.units[u].orders[a].body {
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    self.building_slot_reused(b)
                }
                Body::Gather(g) => self.building_slot_reused(g.building),
                _ => false,
            };
            if dead {
                self.repath(u);
                self.kill_current_order(u);
                return;
            }
        }
        // `Unit::work@0060d180:378`, before the dispatch: the collision
        // cooldown a failed 48-grid search bought counts down
        // (`docs/COLLISION.md` §4.1).
        if self.units[u].safe != 0 {
            self.units[u].safe -= 1;
        }
        if !self.units[u].orders.is_empty() {
            self.units[u].idle = 0;
        }
        // `Unit::work@0060d180:380`: an order of any type blows a stealth
        // unit (`CVar15 != NONE`, a `0x40000` type without `0x4000`, and
        // neither `unit_masks & 0x800` — not carried — nor
        // `unit_masks2 & 0x8000`), item 2785.
        if self.order_type(u) != index::NONE && self.unit_stealth_type(u) {
            self.units[u].blown = true;
        }
        // `Unit::work@0060d180:268`: a unit that has an order and is still
        // carrying the idle latch drops it, and a **fisherman or merchant**
        // marks its owner's economy dirty on the way — it has just left
        // `Leader::calc_gather` step 6's walk ([`crate::rares`]).
        // The steps are [`crate::work_gate::fresh_latch_steps`] (swept, item
        // 3348). SEAM: the original runs them after the dispatch, with the
        // type the dispatch left; this crate runs them here, with the front
        // order's.
        let ty = i32::from(self.order_type(u));
        crate::work_gate::fresh_latch_steps(
            &mut crate::work_gate::SimWorkGate::new(self, u, frame),
            ty,
        );
        // `Unit::work@0060d180:97` — the caravan block, ahead of the
        // dispatch: a linked trade route runs another frame of its road
        // plan, or is ended when the order under it is no longer one
        // (`crate::caravan` §6).
        self.caravan_work(u);
        // `Unit::work@0060d180:440` — **the chase's sixteen-frame review**,
        // and it sits above the dispatch, so a chase it ends is answered by
        // the order underneath in the *same* frame (`docs/COMBAT.md` §36).
        // An unpacked packer under a move takes the pack arm in its place
        // (`0060d440`, `docs/ORDERS.md` §6.9.2).
        match self.pack_before_move(u) {
            PackArm::Not => self.check_target_path_review(u, frame),
            PackArm::Killed => return,
            PackArm::Held | PackArm::Packing => {}
        }
        // `Unit::work@0060d180:283-313`, after the review and before the
        // dispatch: an action that is an `ATTACK` with a target marks the
        // attacker's squad in danger, and the target's when it is a unit.
        // It runs ahead of the target's `uid` test, so a stale target is
        // marked too (`docs/ORDERS.md` §22).
        //
        // And `update_action` is **unconditional** there (`:283`): every
        // frame's `Unit::work` rewrites `orders_x/y` and `dest_angle`
        // before the dispatch, whatever the order.
        // `Unit::work@0060d180:0060d77d`, just above it: an open path search
        // under a head order that is not a move is dropped
        // ([`crate::work_gate::partial_path_steps`], swept, item 3348).
        let has_head = self.current_order(u).is_some();
        crate::work_gate::partial_path_steps(
            &mut crate::work_gate::SimWorkGate::new(self, u, frame),
            has_head,
        );
        let action = self.update_action(u);
        if action.is_some_and(|a| self.units[u].orders[a].index() == index::ATTACK)
            && let Some(t) = self.units[u].combat.target
        {
            self.set_in_danger(u);
            if let Obj::Unit(v) = t {
                self.set_in_danger(v);
            }
        }
        // `Unit::work@0060d180:285-296`, beside it: the **cast clock**
        // (`spell_time`, `+0x98`) is zeroed on every frame the front order
        // is not a cast and the action is not one either — so a cast an
        // order displaced starts again from 0 (run346's `1/98` on 11508,
        // item 1302). `Animal::work` has no such line, and a bird keeps its
        // counter in the same field (`crate::gaia`). SEAM: the action's
        // `+0x20` test, read as always true.
        let is_cast = |o: &Order| matches!(o.body, Body::Cast(_));
        if !self.units[u].is_gaia()
            && !self.units[u].orders.front().is_some_and(is_cast)
            && !action.is_some_and(|a| is_cast(&self.units[u].orders[a]))
        {
            self.units[u].spell_time = 0;
        }
        // `Unit::work@0060d180`, the last block before `do_job` (`docs/
        // COLLISION.md` §24, item 1452): a unit that takes the boat arm
        // — sea, siege, supply or hero, [`Self::takes_boat_arm`] — and
        // was pushed or collided within the last four frames (`frame − 4
        // < collide_frame`) pushes back from where it stands, unless its
        // job is a move (`MOVE_TO`, `ATTACK_TO`, `EXPLORE_TO`, `FLEE_TO`,
        // `CHANGE_FORM`, `GROUP_MOVE`, `GROUP_ATTACK_TO`): its own push
        // at its own position, with `mates` clear. run620's idle ship
        // `1/16`, pushed by the transport `1/39` on 6140, pushes it back
        // on 6141 (run621's probe), and the transport's next step starts
        // from there.
        if frame - 4 < self.units[u].collide_frame && self.takes_boat_arm(u) {
            let job = self.order_type(u);
            if !matches!(
                job,
                index::MOVE_TO
                    | index::ATTACK_TO
                    | index::EXPLORE_TO
                    | index::FLEE_TO
                    | index::CHANGE_FORM
                    | index::GROUP_MOVE
                    | index::GROUP_ATTACK_TO
            ) {
                let at = self.units[u].pos;
                self.detect_boat_collision(u, at, false);
            }
        }
        match self.current_order(u).map(|o| o.body) {
            None => self.do_idle(u, frame),
            Some(Body::Move(m)) => {
                // `pUVar1 == param_1` after the step: the tails below run
                // only while the order they were dispatched for still heads
                // the list — not one put there in its place (AI §135).
                let head = self.units[u].head_serial.0;
                // §8.3: a `GroupMoveOrder` is stepped by `do_group_move`,
                // which runs `do_move` for the **leader** alone and steers
                // every follower off the leader's own position.
                if let Some(gm) = m.group {
                    self.do_group_move(u, frame);
                    // `Unit::do_group_attack_to@005e74e0`: the same look as
                    // `do_attack_to`'s, after the group step (item 1012).
                    if m.kind == MoveKind::AttackTo {
                        self.do_group_attack_to_tail(u, frame, gm.id);
                    }
                } else {
                    self.do_move(u, frame);
                }
                let same = self.units[u].head_serial.0 == head;
                if m.kind == MoveKind::ExploreTo && same {
                    self.do_explore_to_tail(u, frame, m.dest);
                }
                if m.kind == MoveKind::AttackTo && m.group.is_none() && same {
                    self.do_attack_to_tail(u, frame, m.dest);
                }
            }
            Some(Body::Trade(_)) => self.do_trade(u),
            Some(Body::Build(_)) => self.do_build(u, frame),
            Some(Body::Repair(_)) => self.do_repair(u, frame),
            Some(Body::Garrison { .. }) => self.do_garrison_order(u),
            Some(Body::Gather(_)) => self.do_gather(u, frame),
            Some(Body::Attack(_)) => self.do_attack(u, frame),
            Some(Body::Cast(c)) => self.do_cast(u, c),
            Some(Body::Guard(_)) => self.do_guard(u, frame),
            Some(Body::Follow(f)) => self.do_follow(u, f, frame),
            Some(Body::AttackGround(_)) => self.do_attack_ground(u, frame),
            Some(Body::Patrol(p)) => self.do_patrol(u, p),
            Some(Body::Strafe(_)) => self.do_strafe(u, frame),
            Some(Body::AirPatrol(_)) => self.do_air_patrol(u, frame),
            Some(Body::AirAttackGround(_)) => self.do_air_attack_ground(u, frame),
            Some(Body::Think) => self.do_think_order(u, frame),
        }
        // `60dadd`, after `do_job`: two Helicopters set apart.
        self.helicopter_spread(u);
        // `Unit::work@0060d180:449`, the last line: `unit_masks &= ~0x10`,
        // on every path that reaches the end ([`crate::Unit::contact`]).
        self.units[u].contact = false;
    }

    /// `Unit::do_explore_to@005f24a0`'s tail — everything the case does
    /// beyond `do_move`.
    ///
    /// **One frame in fifteen, phased by `o`**, a unit still walking the
    /// same `EXPLORE_TO` looks around for a goody box and re-targets onto
    /// it (`docs/GOODY.md` §7). The phase is `(o + frame) % 15`, and the
    /// original's `pUVar2 == param_1` — the order list's head is still the
    /// order this call was dispatched for — is what `dest` stands in for:
    /// an arrival that popped the order, or a collision that replaced it,
    /// skips the look.
    ///
    /// `is_captain` (`o_up < 0`) is the second guard: a figure marching
    /// under someone else's formation does not go off on its own.
    pub(super) fn do_explore_to_tail(&mut self, u: usize, frame: i64, dest: Pos) {
        if (frame + i64::from(self.units[u].index)).rem_euclid(15) != 0 {
            return;
        }
        let same = matches!(
            self.current_order(u).map(|o| o.body),
            Some(Body::Move(m)) if m.kind == MoveKind::ExploreTo && m.dest == dest
        );
        if !same || !self.is_captain(u) {
            return;
        }
        self.find_goody_box(u);
    }

    /// `Unit::do_attack_to@005f2320`'s tail — the attack-move's look
    /// around, after `do_move` (`docs/ORDERS.md` §7.4,
    /// `docs/ORDERS.md` §22).
    ///
    /// **One frame in fifteen, phased by `o`**, while the head is still
    /// this order: an armed unit that is not a supply wagon runs
    /// `find_melee_target(−1, NULL, 0, 1, 0)`, which adds what it finds
    /// `QUEUE_FIRST` above the attack-move. The attack-move resumes when
    /// that attack dies. One gate comes first, read off the listing at
    /// `005f23ca`–`005f243f`. An AI-driven unit (`unit_masks & 0x40000`)
    /// whose army is hurrying (`ArmyData +0x2c`) skips the look when it is
    /// more than six cells from the army's muster, by `vector_dist`.
    ///
    /// `find_melee_target`'s head is the squad's. A **follower** takes
    /// its captain's target when the captain's action is an `ATTACK`
    /// (`005ff9c0:32-91`), and looks for nothing when it is not. The
    /// search is only reached when the captain's target is no longer a
    /// valid one.
    ///
    /// **The unarmed arm is [`Self::do_attack_to_pause`]** (item 569):
    /// the listing's test is the type's attack **and** the raw
    /// `is_supply`, `unit_flags2 & 0x40`, so a Supply Wagon and every
    /// unarmed unit wait for their group instead of looking.
    ///
    /// The search's `flags` word, which `find_melee_target` derives from
    /// the type's vslots `+0x10c`/`+0x110` for an attack-move, is
    /// [`Sim::melee_search_flags`] (item 997, `docs/COMBAT.md` §64): it
    /// passes over an unarmed building.
    ///
    /// SEAM: `find_nearby_target`'s naval refusal (`+0x218 == 2`), and the
    /// siege-on-a-city `mandatory` arm. Neither is carried by
    /// [`Sim::find_nearby_target`].
    pub(crate) fn do_attack_to_tail(&mut self, u: usize, frame: i64, dest: Pos) {
        if (frame + i64::from(self.units[u].index)).rem_euclid(15) != 0 {
            return;
        }
        let same = matches!(
            self.current_order(u).map(|o| o.body),
            Some(Body::Move(m)) if m.kind == MoveKind::AttackTo && m.group.is_none() && m.dest == dest
        );
        if !same {
            return;
        }
        self.attack_move_look(u, true);
    }

    /// `Unit::do_group_attack_to@005e74e0`'s tail — the look an army's
    /// marching group takes (item 1012, `docs/COMBAT.md` §66).
    ///
    /// The same fifteen-frame phase as [`Self::do_attack_to_tail`], the
    /// same test that the head is still this order (the original's
    /// `pUVar2 == param_1`, which the group order's `id` stands in for:
    /// `ungroup_move_order` replaces it with a plain one), and the same
    /// `find_melee_target(−1, NULL, 0, 1, 0)` — **without the hurry
    /// gate**, which `do_attack_to` alone carries. What the look finds is
    /// handed to the group rather than to the unit when the whole group is
    /// still walking its attack-move ([`Self::attack_move_add`]).
    pub(crate) fn do_group_attack_to_tail(&mut self, u: usize, frame: i64, id: i64) {
        if (frame + i64::from(self.units[u].index)).rem_euclid(15) != 0 {
            return;
        }
        if !self.still_group_move(u, id) {
            return;
        }
        self.attack_move_look(u, false);
    }

    /// The body `do_attack_to` and `do_group_attack_to` share once the
    /// phase and the head have passed: an unarmed unit or a supply wagon
    /// pauses (`do_attack_to_pause`); `hurry_gate` is `do_attack_to`'s
    /// `005f23ca`–`005f243f`; then `find_melee_target`'s squad head and its
    /// search, whose find [`Self::attack_move_add`] adds.
    pub(super) fn attack_move_look(&mut self, u: usize, hurry_gate: bool) {
        let me = Obj::Unit(u);
        let supply = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO));
        if self.profile(me).attack == 0 || supply {
            self.do_attack_to_pause(u);
            return;
        }
        let who = self.units[u].owner;
        if hurry_gate
            && self.ai_driven(who)
            && let Some(slot) = self.army_of(u)
            && let Some(a) = self.armies[who as usize].list.get(slot)
            && a.hurry != 0
        {
            let (c, m) = (self.units[u].pos.cell(), a.muster);
            if crate::world::vector_dist((c.x - m.x).abs(), (c.y - m.y).abs()) > 6 {
                return;
            }
        }
        match self.melee_squad_head(u) {
            SquadHead::Nothing => {}
            SquadHead::Captain(t, mandatory) => {
                self.add_attack_order(u, t, QueuePos::First, mandatory, false);
            }
            SquadHead::Search => {
                if self.take_siege_army_target(u).is_some() {
                    return;
                }
                if let Some(t) = self.find_melee_target(u, -1) {
                    self.attack_move_add(u, t);
                }
            }
        }
    }

    /// `find_melee_target@005ff9c0`'s head (lines 32–91), for a caller
    /// whose third argument is 0 — here `do_attack_to`'s and
    /// `do_group_attack_to`'s look (item 1012; `Group::action_attack`'s
    /// retarget is the other, and the floor refused it there,
    /// `docs/COMBAT.md` §66.3): a **follower**
    /// takes its captain's target when the captain's action is an
    /// `ATTACK` on a valid one it may chase (not STAND_GROUND, neither of
    /// the two entrenched, or already in range), and finds **nothing** when
    /// the captain's action is not an `ATTACK`. A captain, or a follower
    /// whose captain's target has gone, searches.
    /// `find_melee_target(−1, NULL, 0, 1, 0)` as `find_new_target@005ff6a0`
    /// calls it, with the order it adds (item 1089, `docs/COMBAT.md` §79).
    ///
    /// The head first ([`Self::melee_squad_head`], `005ff9c0:32-91`): a
    /// follower's captain's target is added `QUEUE_FIRST` under an
    /// `ATTACK_TO`, a `GROUP_ATTACK_TO` or a `GUARD` in front and
    /// `QUEUE_NEW` under anything else, with the captain's `mandatory`
    /// byte (the action's `+0x1c`). Great Lakes' `1/13` on tick 5066 is
    /// the witness: its target `0/2` dead, it takes its captain `1/12`'s
    /// `0/4` where the search would have ranked `0/5` first (run400's
    /// packet, entered directly: `add_attack_order(4, 0, …)` from
    /// `find_melee_target+0x1c5`, and no `find_nearby_target` at all).
    ///
    /// The search's own add is the one this arm always had, `QUEUE_FIRST`.
    pub(crate) fn find_melee_target_added(&mut self, u: usize) -> Option<Obj> {
        self.find_melee_target_added_in(u, -1)
    }

    /// [`Self::find_melee_target_added`] at the caller's own `range` — its
    /// second argument, `-1` for the idle radius.
    /// `Group::target_opportunity@007107d0`'s member arm passes
    /// `min(dist + 0xc0, unit_respond_range × 0x240)` (`docs/GOLDEN.md` §61).
    pub(crate) fn find_melee_target_added_in(&mut self, u: usize, range: i32) -> Option<Obj> {
        match self.melee_squad_head(u) {
            SquadHead::Nothing => None,
            SquadHead::Captain(t, mandatory) => {
                self.squad_head_add(u, t, mandatory);
                Some(t)
            }
            SquadHead::Search => {
                // `find_nearby_target`'s siege shortcut answers this caller
                // too ([`Self::take_siege_army_target`], item 3654).
                if let Some(t) = self.take_siege_army_target(u) {
                    return Some(t);
                }
                let t = self.find_melee_target(u, range)?;
                self.nearby_normalize(u);
                let mandatory = self.nearby_siege_city(u, t);
                // A guard's attack-move leg goes under the find
                // ([`Self::nearby_add`]'s `local_2c`); every other head
                // keeps the `QUEUE_FIRST` this arm always had.
                if self.guard_activity(u).is_some() && self.order_type(u) == index::ATTACK_TO {
                    self.kill_current_order(u);
                }
                self.add_attack_order(u, t, QueuePos::First, mandatory, false);
                Some(t)
            }
        }
    }

    /// `Object::find_nearby_target@00648da0`'s packed AI siege shortcut:
    /// take a valid army target strictly inside the response radius,
    /// mandatory and QUEUE_FIRST, before ranking nearby candidates.
    /// Used by the attack-move look, the packer's chase re-search
    /// (`docs/COMBAT.md` §88), `think`'s idle search (item 2864), and
    /// since item 3654 (`docs/AI.md` §336.2) `find_new_target`'s search
    /// ([`Self::find_melee_target_headed`]) and `Group::target_opportunity`'s
    /// member search ([`Self::find_melee_target_added_in`]): the shortcut
    /// sits inside `find_nearby_target@00648da0:198`–`222`, ahead of its
    /// add flag, so every caller takes it — `Group::action_attack`'s
    /// per-member search too (`attack_view.rs`, item 3690: run1359c's
    /// `1/131` on 5678, `find_nearby_target+0x4c8`).
    pub(crate) fn take_siege_army_target(&mut self, u: usize) -> Option<Obj> {
        let who = self.units[u].owner;
        let p = self.profile(Obj::Unit(u));
        if !p.packs
            || !p.siege
            || !self.units[u].combat.packed
            || !self.ai_driven(who)
            || self.search_ai(who)
        {
            return None;
        }
        // The shortcut sits behind `find_melee_target@005ff9c0`'s own exit:
        // a captain that is not on duty and holds fire (`get_combat_stance()
        // > 4`, `LAB_005ffbb2`) returns −1 before `find_nearby_target` runs
        // (twenty-fourth pass, group 10; A6 row 18).
        if self.units[u].captain
            && self.units[u].combat.stance == combat::Stance::HoldFire
            && !self.on_duty(u)
        {
            return None;
        }
        let slot = self.army_of(u)?;
        let target = self.armies[who as usize].list.get(slot)?.target?;
        if !self.valid_target(Obj::Unit(u), target)
            || self.attack_dist(Obj::Unit(u), target) >= self.tuning.unit_respond_range * 0x180
        {
            return None;
        }
        self.add_attack_order(u, target, QueuePos::First, true, false);
        Some(target)
    }

    pub(crate) fn melee_squad_head(&mut self, u: usize) -> SquadHead {
        if self.units[u].captain {
            return SquadHead::Search;
        }
        let me = Obj::Unit(u);
        let cap = self.squad_captain(u);
        let Some(k) = self.action_of(cap) else {
            return SquadHead::Nothing;
        };
        if !matches!(self.units[cap].orders[k].body, Body::Attack(_)) {
            return SquadHead::Nothing;
        }
        if let Some(t) = self.units[cap].combat.target
            && self.valid_target(me, t)
            && ((self.units[u].combat.stance != combat::Stance::StandGround
                && !(self.hard_trench(u) && self.hard_trench(cap)))
                || self.is_in_range(me, t))
        {
            return SquadHead::Captain(t, self.units[cap].combat.mandatory);
        }
        SquadHead::Search
    }

    /// `Object::find_nearby_target`'s add arm for a searcher on an
    /// attack-move (`00648da0`, the decompile's lines 545–606; item 1012,
    /// `docs/COMBAT.md` §66): a unit whose `order_type` is
    /// `GROUP_ATTACK_TO`, still an active member of its group
    /// (`Group::normalize`, `GroupData::member(o, who, 1)`), in a group
    /// every member of which is still walking its attack-move
    /// ([`Sim::group_is_attacking_to`]), and **not siege** hands its find
    /// to the whole group — `Group::action_attack(o, whom, 0, QUEUE_FIRST,
    /// 4)`. Anything else takes it alone, `add_attack_order(…,
    /// QUEUE_FIRST, 0, 0)` (`LAB_00649ba0`).
    ///
    /// SEAM: the siege arm (`local_2c`: an AI siege unit on a building
    /// whose `+8 & 0x20` is set takes it `mandatory` and writes the army's
    /// target, and a plain `ATTACK_TO` is killed first), and the naval
    /// refusal (`+0x218 == 2`). No capture reaches either.
    pub(super) fn attack_move_add(&mut self, u: usize, t: Obj) {
        self.nearby_normalize(u);
        let mandatory = self.nearby_siege_city(u, t);
        // `local_2c` first: a guard's leg is killed under the attack, and
        // its group never takes the find (`:551`, [`Self::nearby_add`]).
        let group_move = self.guard_activity(u).is_none()
            && self.current_order(u).map(Order::index) == Some(index::GROUP_ATTACK_TO);
        let siege = self.units[u]
            .ty
            .is_some_and(|ty| self.unit_types[ty].cols.flag(uflags::FLAG_SIEGE));
        if group_move
            && !siege
            && let Some(g) = self.group_of(u)
            && g.list.contains(&u)
            && self.group_is_attacking_to(&g)
        {
            self.group_action_attack(&g, t, false, QueuePos::First, 4);
            return;
        }
        self.nearby_add_order(u, t, mandatory);
    }

    /// `Unit::do_attack_to_pause@005f22a0` (`docs/ORDERS.md` §24.9): an
    /// unarmed unit on an attack-move **waits for its group** when the
    /// group is fighting beside it — or, as it is written, when no more
    /// than half of the armed captains near it are still walking.
    ///
    /// The unit's group (`+0x80`) must hold it, active
    /// (`GroupData::member(o, who, 1)@0070f8f0`); then
    /// `Group::is_attacking_near(x, y)@00710e40`, read off the listing at
    /// `710e40`–`711061` because the decompile prints `unaff_EDI/ESI` for
    /// both the `vector_dist` pair and the order type:
    ///
    /// - `normalize` the group first;
    /// - over every member that is active, on the map, **a captain**,
    ///   armed (type `+0x1e8`), and neither `is_supply` (`& 0x40`) nor
    ///   `is_hero` (`& 0x20`) in `unit_flags2`: `near += 1` when
    ///   `vector_dist(|x − ux|, |y − uy|) ≤ 0x600`, and then `attacking
    ///   += 1` when its head order's type is not a move (`is_move@0046f050`:
    ///   1–4, `0x12`, `0x13`, `0x15`), not `NONE` and not `GUARD`;
    /// - yes when `near ≠ 0 && attacking ≥ near / 2`.
    ///
    /// **One armed captain near and none fighting is a yes** — `1 / 2` is
    /// 0 — so a Supply Wagon with its escort's captain inside `0x600`
    /// stops. The order's `pause` becomes 15, and `do_move` stands it out.
    pub(super) fn do_attack_to_pause(&mut self, u: usize) {
        let seat = self.seat_of(u);
        let pause = attack_to_pause_steps(
            self,
            seat,
            |w, &s| w.units[u].alive() && w.seat_list(s).contains(&u),
            |w, &s| w.is_attacking_near(s, w.units[u].pos),
        );
        if pause
            && let Some(front) = self.units[u].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.pause = 15;
        }
    }

    /// `Group::is_attacking_near@00710e40`, as
    /// [`Self::do_attack_to_pause`] describes it: `normalize`, then the
    /// armed captains within `0x600` of `at`, yes when `near ≠ 0` and no
    /// more than half of them are walking.
    pub(super) fn is_attacking_near(&mut self, seat: crate::group::Seat, at: Pos) -> bool {
        self.seat_normalize(seat);
        let (mut near, mut attacking) = (0, 0);
        for m in self.seat_list(seat).clone() {
            let unit = &self.units[m];
            if !unit.alive() || !unit.on_map || !unit.captain {
                continue;
            }
            if self.profile(Obj::Unit(m)).attack == 0 {
                continue;
            }
            let special = unit.ty.is_some_and(|t| {
                let c = self.unit_types[t].cols;
                c.flag2(uflags2::SUPPLY_OR_HERO) || c.flag2(uflags2::FLAG2_GENERAL)
            });
            if special {
                continue;
            }
            let (dx, dy) = ((at.x - unit.pos.x).abs(), (at.y - unit.pos.y).abs());
            if vector_dist(dx, dy) > 0x600 {
                continue;
            }
            near += 1;
            let t = self.current_order(m).map_or(index::NONE, Order::index);
            let moving = matches!(t, 1..=4 | 0x12 | 0x13 | 0x15);
            if !moving && t != index::NONE && t != index::GUARD {
                attacking += 1;
            }
        }
        near != 0 && attacking >= near / 2
    }

    /// **The pack arm** of `Unit::work@0060d180` (`0060d440`–`0060d626`,
    /// `docs/ORDERS.md` §6.9.2): a type that packs (`+0x2b8 & 4`,
    /// [`crate::combat::Profile::packs`]) standing **unpacked** under a
    /// move-family head order does not take the sixteen-frame review. It
    /// takes one of three arms instead, in this order:
    ///
    /// 1. **Held** — the head carries the action bit, it is the only order
    ///    (`+0xd8 length == 1`), and its point is inside the type's
    ///    `block_radius` (`+0x240`) by `vector_dist`: the order's point,
    ///    waypoint and `last` become the unit's own position and its
    ///    `tolerance` 0 (`get_move_order`, vslot `+0x40`); and when the
    ///    order is pathed with a path standing, the stack is popped down to
    ///    its first `FINAL` leg, which is popped too and replaced by the
    ///    unit's own position carrying that leg's tolerance and flags. The
    ///    move then runs as usual.
    /// 2. **Killed** — the unit stands in the order point's 48-unit cell
    ///    (`div_3_table[x >> 4]`, both axes): `kill_current_order(0)`, and
    ///    `work` returns there.
    /// 3. **Packing** — `add_cast_order(PACK, QUEUE_FIRST, 0)`, then the
    ///    head is read again (`update_order`), so the cast's first frame
    ///    is this frame's dispatch.
    ///
    /// run544's Bombard `1/132` (type 267) is arm 3: its army's march to
    /// (7224, 4200) reaches it on 15868 standing deployed at (42031,
    /// 40399), and block 15869 holds a `CASTORDER` `spell 651` over the
    /// `GROUPATTACKTOORDER`, `cur_anim 23` (`CHAR_PACK`) for 80 frames.
    ///
    /// SEAM: the order's `tolerance` (`MoveOrder +0x14`), which arm 1
    /// zeroes, is not a field this crate's move carries.
    pub(crate) fn pack_before_move(&mut self, u: usize) -> PackArm {
        let Some(&head) = self.current_order(u) else {
            return PackArm::Not;
        };
        let Body::Move(m) = head.body else {
            return PackArm::Not;
        };
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs);
        if !packs || self.units[u].combat.packed {
            return PackArm::Not;
        }
        let at = self.units[u].pos;
        if head.has(flag::ACTION)
            && self.units[u].orders.len() == 1
            && crate::world::vector_dist((at.x - m.dest.x).abs(), (at.y - m.dest.y).abs())
                < self.profile(Obj::Unit(u)).block_radius
        {
            if let Some(front) = self.units[u].orders.front_mut()
                && let Some(mo) = front.move_mut()
            {
                mo.dest = at;
                mo.waypoint = at;
                mo.last = Some(at);
            }
            if head.has(flag::PATHED) && !self.units[u].path.is_empty() {
                let path = &mut self.units[u].path;
                while let Some(top) = path.pop() {
                    if top.flags & path_flag::FINAL != 0 {
                        path.push(PathData {
                            to: at,
                            tolerance: top.tolerance,
                            flags: top.flags,
                        });
                        break;
                    }
                }
            }
            return PackArm::Held;
        }
        if crate::collide::ucell(at) == crate::collide::ucell(m.dest) {
            self.kill_current_order(u);
            return PackArm::Killed;
        }
        self.add_cast_order(u, spell::PACK);
        PackArm::Packing
    }

    /// `Unit::work@0060d180:440`'s gate on [`Self::check_target_path`] —
    /// **one frame in sixteen, phased by `o`** (`docs/COMBAT.md` §36).
    ///
    /// It runs for a head order of the **move family** only, and only when
    /// the order underneath it is an action the review knows how to ask
    /// about. `Unit::work` re-reads the head afterwards
    /// (`0060d710: update_order`), which is what this crate's dispatch
    /// `match` does anyway by reading `current_order` after the call — so a
    /// chase ended here is answered by `do_attack` on the same frame, and
    /// that is exactly what separates it from `do_move`'s own kill
    /// (§36.2).
    ///
    /// **The `GUARD` arm** (`action type == 0xc`, `0060d4a0`–`0060d4d3`)
    /// is the review's other branch, and it asks no question of the
    /// target at all: on the sixty-four-frame phase `(frame + o) % 64 ==
    /// 0` it `repath`s — the reposition leg goes — and jumps to the head
    /// re-read, so `do_guard` answers on the same frame with a fresh post
    /// (`docs/ORDERS.md` §24). Off that phase it does nothing.
    ///
    /// ~~SEAM — the guard above it that no capture has reached:
    /// `ptype +0x2b8 & 4` with `unit_masks & 0x80000` (the packable
    /// lineage, which takes an `add_cast_order` branch instead).~~ Built
    /// as [`Self::pack_before_move`] (item 1370): run544's Bombard `1/132`
    /// packs under its army's march on 15868.
    ///
    /// ~~SEAM — the head-order conjunct `head->vt+0x2c() == 0 ||
    /// head->vt+0x94()` names *me*.~~ Built (item 3939, `docs/GOLDEN.md`
    /// §171): a group move's head is reviewed only by the member its
    /// `+0x94` record names, [`GroupMove::leader`]. run1522's Catapult
    /// leader reviews `[GROUP_MOVE][MOVE_TO][ATTACK]` on 714 and enters
    /// `Group::is_moving_to`; a follower never does.
    pub(super) fn check_target_path_review(&mut self, u: usize, frame: i64) {
        let Some(Body::Move(m)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        // `Unit::work:187-195`: a group order's head is reviewed only by the
        // member its `+0x94` record names, the group's leader (item 3939,
        // `docs/GOLDEN.md` §171).
        if m.group.is_some_and(|gm| gm.leader != u)
            || self.units[u].phase(frame).rem_euclid(16) != 0
        {
            return;
        }
        // `action->vt+0x20() != 0` is "this order can be an action", which
        // [`Self::action_of`] has already answered; `!= 9` excludes
        // `AWAIT_BOARD`, which this crate never puts under a move.
        let Some(a) = self.action_of(u) else { return };
        match self.units[u].orders[a].body {
            Body::Guard(_) if self.units[u].phase(frame).rem_euclid(64) == 0 => {
                self.repath(u);
            }
            // The attack, the garrison and the cast arms are one function in
            // the original, `Unit::check_target_path@005e22d0`, whose arm is
            // picked by the action's type ([`check_target_path_steps`]).
            Body::Attack(_) | Body::Garrison { .. } | Body::Cast(_) => {
                self.check_target_path(u);
            }
            _ => {}
        }
    }

    /// `Unit::change_target@005e36c0` — one decision, written down the
    /// **whole squad** in place (`docs/COMBAT.md` §37.4).
    ///
    /// The function is a walk of `o_down` from the unit it is called on,
    /// and at each link it does two things: rewrite the action order's
    /// target from `old` to `new` *if it still holds `old`*, and then pop
    /// head orders until the head is the targeted one — which is what
    /// drops the chase the retarget has just made pointless. The walk
    /// stops at `o_down < 0` or at the first link whose `flags & 1` is
    /// clear.
    ///
    /// **The rewrite is in place, and that is measurable.** run112's
    /// `0/10` and `0/11` take `1/6` on block 645 with `in_range 1`,
    /// `ever_in_range 1` and `new_ord 0` — the flags they were already
    /// carrying against `1/8`. A fresh `add_attack_order` through the
    /// captain mirror would have reset all three, so the dump
    /// distinguishes this from every order-creating path, and it is what
    /// rules out `Unit::think`'s own `near_o` arm (§37.5) as the
    /// mechanism even before one notices that `think` is reached from
    /// `do_idle` alone.
    ///
    /// SEAM: the head-popping test is `UnitOrder::is_targeted` (`vt+0x20`
    /// by the type record — every class deriving `TargetOrder` answers
    /// it), and this crate asks for [`Body::Attack`] instead. They agree
    /// wherever the action is an attack, which is every capture on file
    /// that reaches this at all; a `GATHER` or a `BUILD_AT` under a move
    /// would part.
    pub(super) fn change_target(&mut self, u: usize, old: Obj, new: Obj) {
        let mut v = u;
        loop {
            if let Some(a) = self.update_action(v)
                && matches!(self.units[v].orders[a].body, Body::Attack(_))
                && self.units[v].combat.target == Some(old)
            {
                self.units[v].combat.target = Some(new);
                while self.units[v]
                    .orders
                    .front()
                    .is_some_and(|o| !matches!(o.body, Body::Attack(_)))
                {
                    self.kill_current_order(v);
                }
            }
            let Some(next) = self.units[v].o_down else {
                return;
            };
            if !self.active(Obj::Unit(next)) {
                return;
            }
            v = next;
        }
    }

    /// `Unit::check_target_path@005e22d0` (item 3304): [`check_target_path_steps`]
    /// over [`CheckTargetView`] (`SimCheckTarget`), for the action order the
    /// sixteen-frame review found. The three action types this crate routes
    /// here are the ones its orders carry: an `ATTACK` (10) on the unit's
    /// combat target, a `CAST` (`0xe`) on the spell's target object and a
    /// `GARRISON` (`0x1a`) on its building.
    ///
    /// Returns what the original returns: whether the review changed the
    /// order list (the only caller ignores it).
    pub(super) fn check_target_path(&mut self, u: usize) -> bool {
        let Some(a) = self.action_of(u) else {
            return false;
        };
        let (ty, target, spell) = match self.units[u].orders[a].body {
            Body::Attack(_) => (10, self.units[u].combat.target, 0),
            Body::Cast(c) => (0xe, c.target, c.spell),
            Body::Garrison { building, .. } => (0x1a, Some(Obj::Building(building)), 0),
            _ => return false,
        };
        check_target_path_steps(&mut SimCheckTarget {
            s: self,
            u,
            ty,
            target,
            spell,
        })
    }

    /// `Unit::repath@005e29b0` ([`repath_steps`], item 2341): while the
    /// head is of the move family it goes — a `GROUP_MOVE` of a unit in a
    /// group through `Group::kill_group_move` (the whole group's plan),
    /// anything else through `kill_current_order(0)`. The action bit is
    /// not asked (it was until item 2341's sweep: the port popped only
    /// transit legs and kept a player's own move).
    ///
    /// SEAM: a `GROUP_MOVE` whose group's walk leaves the head standing
    /// (the unit not in the list, dead or off the map) loops for ever in
    /// the original; here it falls through to `kill_current_order`.
    pub(crate) fn repath(&mut self, u: usize) {
        repath_steps(
            self,
            |s| s.current_order(u).is_some(),
            |s| i32::from(s.order_type(u)),
            |s| s.current_order(u).is_some_and(|o| o.has(flag::ACTION)),
            |s| s.group_of(u),
            |s| match s.current_order(u).map(|o| o.body) {
                Some(Body::Move(MoveOrder {
                    group: Some(gm), ..
                })) => gm.id,
                _ => 0,
            },
            |s, g, id| {
                let head = s.units[u].head_serial;
                s.kill_group_move(&g, id);
                if s.units[u].head_serial == head && s.current_order(u).is_some() {
                    s.kill_current_order(u);
                }
            },
            |s| s.kill_current_order(u),
        );
    }

    // ------------------------------------------------------------------
    // Idle — `do_idle`, `check_idle`, `think`
    // ------------------------------------------------------------------

    /// `Unit::do_idle`: `set_anim(CHAR_DEFAULT, 0, 1)`, `collide = 0`,
    /// `check_idle`, `think`. An animal's is `Animal::do_idle`, which
    /// replaces the whole of it (`anim.rs`).
    pub(crate) fn do_idle(&mut self, u: usize, frame: i64) {
        if self.units[u].is_gaia() {
            // A gaia bird is never idle in the original: it carries an air
            // order from birth, and `do_job` dispatches on that order
            // rather than reaching `do_idle` at all. Both orders end in
            // `Unit::do_air_physics`, which ends in `set_anim(CHAR_WALK,
            // 0, 1)` on every frame the bird is not a flock's
            // (`field_0xae`, only ever set for `FLOCKBIRD`). SEAM: this
            // crate has no air order, so the bird stands here instead and
            // the *flight* is unmodelled — but what reaches the stream is
            // here (`docs/SYNC.md` §3.9, `docs/TRANSPORT.md` §5.2).
            let t = self.units[u].type_index;
            if crate::anim::is_air_gaia(t) {
                // `Animal::think_bird`'s `0x192` arm, under
                // `Unit::do_air_patrol+0x28`. A **gull** is on a
                // `StrafeOrder`, so `Unit::do_strafe` calls the same
                // `+0x180` virtual — and `think_bird`'s `0x194` arm
                // returns after an `order_type` call and draws nothing.
                if t == crate::anim::BIRD_TYPE {
                    self.think_bird(u, frame);
                }
                self.set_anim(u, crate::anim::WALK, false, true);
                // **The flight**, `Unit::do_air_patrol`'s second half —
                // [`crate::air`]'s bank, step and edge coin
                // (`docs/SYNC.md` §3.9). run61 proxies the original's own
                // `do_air_physics` and puts 45,712 of its air frames
                // beside this module's: ten birds, every position and
                // every bank zero-crossing, birth to frame 5,400, exact.
                // The thirty-three-frame error this used to carry was
                // never the flight — it was the birth, and
                // `Gaia::spawn_bird` now takes `Unit::init`'s tile snap.
                if t == crate::anim::BIRD_TYPE {
                    let goal = self.bird_goal(u);
                    self.do_air_physics(u, goal, frame);
                }
                // **The gull's flight** (item 1453): `Unit::do_strafe@
                // 005eab00` on the gull's `StrafeOrder` takes its target,
                // the dock that hatched it, and — the target valid — hands
                // `do_air_physics` the dock's own position as the goal, on
                // every frame. run622's probe shows both gulls' goals at
                // their docks' centres and the second gull's edge coin on
                // 9655. SEAM: the order itself is not carried; the goal
                // is read off the dock slot that holds the gull.
                if t == crate::anim::GULL_TYPE
                    && let Some(goal) = self.gull_dock(u)
                {
                    self.do_air_physics(u, goal, frame);
                }
                //
                // `do_air_patrol`'s own tail, after `do_air_physics`
                // returns 1: the caller branches on `vtable+0x30`,
                // `SubObjectData::is_animal`, and the animal arm is
                // `else if (spell_time == 0) spell_time = 1`. The
                // counter is only ever 0 there on a **landing** frame —
                // `think_bird` steps it on every other — so this is one
                // extra step per landing, and it is what run39's frame
                // 1256 turned on (`docs/SYNC.md` §3.9). The tail is
                // `do_air_patrol`'s; `do_strafe` has none, so the gull
                // does not take it.
                if t == crate::anim::BIRD_TYPE && self.units[u].spell_time == 0 {
                    self.units[u].spell_time = 1;
                }
                return;
            }
            self.animal_idle(u);
            return;
        }
        self.mark(crate::anim::SITE_IDLE_UNIT);
        // **The own-land cloak is lifted for the idle step** (`do_idle@
        // 0060dcd0:13-16`, `:34-36`; item 3958, `docs/AI.md` §402): a unit
        // holding `unit_masks2 & 0x8000` drops it before `set_anim`,
        // `check_idle` and `think`, and takes it back after them.
        let land_cloak = self.units[u].unit_masks2 & combat::umask2::LAND_CLOAK;
        self.units[u].unit_masks2 &= !land_cloak;
        self.set_default_anim(u);
        // `Unit::do_idle`'s own `collide = 0` (`docs/COLLISION.md` §6).
        self.units[u].collide = 0;
        self.check_idle(u, frame);
        self.think(u, frame);
        if let Some(un) = self.units.get_mut(u) {
            un.unit_masks2 |= land_cloak;
        }
    }

    /// The carrying walk a gatherer plays — `unit_masks & 0x78000000`, in
    /// `Guy::set_anim`'s own order (`005db61f`–`005db665`): `WALK_TO_WOOD`
    /// first, then `WALK_WITH_WOOD`, `WALK_TO_ORE`, `WALK_WITH_ORE`.
    ///
    /// **It is the mask, not the order's `goto_build`.** Reading the order
    /// made every walk of a gather a carrying walk, including the first
    /// walk to the camp — which `do_non_flat_gather` has never run for, so
    /// the original plays it as the plain `CHAR_WALK` and takes the
    /// arrival stand `Guy::move+0x19f` at the end of it. run33's `1/7`
    /// carries `unit_masks 262146` on the frame it arrives, and this
    /// crate had it on `WALK_WITH_WOOD` (`docs/ANIM.md` §4.4).
    pub(crate) fn gather_walk(&self, u: usize) -> Option<i8> {
        let m = self.units[u].carry;
        if m & CARRY_ANY == 0 {
            return None;
        }
        Some(if m & CARRY_TO_WOOD != 0 {
            crate::anim::WALK_TO_WOOD
        } else if m & CARRY_WITH_WOOD != 0 {
            crate::anim::WALK_WITH_WOOD
        } else if m & CARRY_TO_ORE != 0 {
            crate::anim::WALK_TO_ORE
        } else {
            crate::anim::WALK_WITH_ORE
        })
    }

    /// `Unit::check_idle`: 0 → 1 → 2 on consecutive frames, then +1 every
    /// 16 frames phased by `o`, wrapping to 1.
    pub(super) fn check_idle(&mut self, u: usize, frame: i64) {
        let unit = &mut self.units[u];
        if unit.idle == 0xff {
            unit.idle = 1;
        } else if unit.idle <= 1 || (frame + i64::from(unit.index)) & 15 == 0 {
            unit.idle += 1;
            // `check_idle@006032c0:30`: the fourth count of a unit in a
            // trench turns it back to the trench's facing (item 3661).
            if unit.idle == 4 && unit.combat.entrenched {
                let a = unit.combat.trench_angle;
                self.unit_set_angle(u, a);
            }
        }
        // **The marines dig in** (`check_idle@006032c0:33`–`41`, item
        // 3661): a unit of the Marines line (`unit_masks2 & 0x800`) idle
        // for `AMERICANS_MARINE_ENTRENCH` counts as loaded (7) and not yet
        // in a trench takes `unit_masks |= 0x2000000`, `unit_masks2 |=
        // 0x20000`, and the trench's facing from its angle (`+0x5c =
        // +0x50`). The graphic event is not state. run1325's `1/29` is in
        // one at idle 7; ours played idle variants a trench refuses, and
        // `1/55`'s seventy-frame idle wrapped on 3215 in theirs alone
        // (`docs/AI.md` §341).
        let unit = &mut self.units[u];
        if unit.unit_masks2 & combat::umask2::MARINE != 0
            && i32::from(unit.idle) >= self.tuning.marine_entrench_idle()
            && !unit.combat.entrenched
        {
            unit.combat.entrenched = true;
            unit.unit_masks2 |= combat::umask2::MARINE_TRENCH;
            unit.combat.trench_angle = unit.movement.heading;
        }
        // `check_idle@006032c0:47`: `(unit_masks & 0x11000) == 0x1000`
        // drops the blown bit once the counter has reached 10.
        if self.units[u].blown && !self.units[u].fought && self.units[u].idle >= 10 {
            self.units[u].blown = false;
        }
        // `check_idle@006032c0`'s **tail**, which runs whatever the counter
        // did: the first frame a unit is idle it takes the latch, and a
        // fisherman or a merchant marks its owner's economy dirty — it has
        // just joined `Leader::calc_gather` step 6's walk. That is what
        // drops the recompute period from 512 frames to 8 and puts a
        // newly-arrived boat's deposit on the books within the same
        // handful of frames ([`crate::rares`]).
        //
        // **The latch is `0x18`, not `8`** (`check_idle@006032c0:69`, item
        // 3983, `docs/AI.md` §415): the same write raises `0x10`, the bit
        // `think`'s cadence gate reads as `cant_reach`. A captain drops it
        // past the gate on its next think; a non-captain returns before the
        // gate and keeps it until something else clears it — run547's
        // decoy members print `flags` 25 against their captains' 9 on the
        // block they are born.
        if !self.units[u].idle_latch {
            self.units[u].idle_latch = true;
            self.units[u].cant_reach = true;
            if self.unit_line_is(u, crate::fish::FISHERMEN) || self.is_merchant(u) {
                self.economy_changed(self.units[u].owner);
            }
        }
    }

    /// `Unit::think` (§2.4): the auto-attack on the first idle frame and
    /// every 32 after, phased by `o`; a worker's `think_peasant(0)`.
    pub(crate) fn think(&mut self, u: usize, frame: i64) {
        // **A non-captain's whole think is the captain mirror**, and it is
        // `Unit::think@005f6e40`'s *first* statement — above the citizen
        // mask-clear, above both cadence gates, above everything
        // (`docs/COMBAT.md` §21). The listing is five tests and a return:
        //
        //     if (!is_captain(this)) {
        //         a = get_action(units[who][get_captain()]);
        //         if (a == 0 || a->get_type() != 10) return;   // ATTACK
        //         if (!valid_target(this, a->o, a->who)) return;
        //         add_attack_order(this, a->o, a->who, QUEUE_NEW,
        //                          a->mandatory, 0);
        //         return;
        //     }
        //
        // — so a squad member never searches, never picks a target of its
        // own, and never reaches `think_attack` at all: it copies whatever
        // its captain is attacking. That is what the golden record's
        // `near_o` witnesses (`docs/COMBAT.md` §18): over 901 frames only
        // `1/6` and `0/6`, the two captains, ever carry one.
        if !self.units[u].captain {
            self.captain_mirror(u);
            return;
        }
        // `think:82` — a **citizen** (`TypeIndex` 0x32 or 0x33) drops its
        // carrying walk here, and the leader takes `0x80000`. In the
        // original this sits after the auto-attack arm and before the
        // cadence gate; here the gate stands first, so a citizen that
        // takes an auto-attack keeps the nibble the original would have
        // cleared. SEAM: the two orderings differ only for an armed
        // citizen, and no capture has one.
        if matches!(self.units[u].type_index, 0x32 | 0x33) {
            self.units[u].carry &= !CARRY_ANY;
        }
        let unit = &self.units[u];
        let phase = frame + i64::from(unit.index);
        // The global cadence gate (`Unit::think@005f6e40:87`, found by the
        // second reading — `docs/audit/2026-08-21-orders.md` R1): once a unit
        // has been idle for more than two frames it thinks only one frame in
        // sixteen, phased by `o`. `SubObjectData::flags & 0x10` exempts it —
        // raised by a gather that could not reach (`do_gather`) and by a
        // stance press (`Group::action_stance`), so either thinks on its
        // next frame.
        if !unit.cant_reach && unit.idle > 2 && phase % 16 != 0 {
            return;
        }
        // **A defeated leader's units think no further, and keep the bit**
        // (`think:98`, item 3983, `docs/AI.md` §415): the first test past
        // the gate is `leader_flags2 & 2`, which `Leader::defeat@006ecb00:67`
        // sets as it clears `leader_flags & 2` — so `:155`, a leader
        // without `& 2`, is never reached by a defeated seat either.
        // run1450's idle human is defeated on the closing block 2601, and
        // its seven citizens print `flags` 25 on their first idle frame
        // where a live seat's captains print 9. SEAM: the scenario's
        // `disable_all_unit_ai` (the same bit, `docs/AI.md` §1) is not
        // modelled.
        if self
            .defeated
            .get(self.units[u].owner as usize)
            .copied()
            .unwrap_or(false)
        {
            return;
        }
        self.think_past_gate(u, frame, phase);
        // **Every exit past the gate drops the bit** (item 3962,
        // `docs/AI.md` §406): `think:130`, `LAB_005f761a` (`:296`) and the
        // tail (`:339`) each write `flags &= 0xef`, and every `return 1` of
        // an arm is a `goto LAB_005f761a`. Of the three exits that keep it,
        // `:98` is the defeated seat's above; `:101` (`unit_masks &
        // 0x1000000`, the scenario's `disable_unit_ai`) is not modelled,
        // and `:155` is reached only by a seat `:98` has already sent
        // back.
        if let Some(unit) = self.units.get_mut(u) {
            unit.cant_reach = false;
        }
    }

    /// [`Sim::think`] past its cadence gate, whose every exit the caller
    /// follows with the bit's clear.
    pub(super) fn think_past_gate(&mut self, u: usize, frame: i64, phase: i64) {
        let me = Obj::Unit(u);
        let unit = &self.units[u];
        // **The off-phase `near_o` arm** (`think@005f6e40:104`–`135`,
        // `5f7017`–`5f70fc`; `docs/AI.md` §205, `docs/COMBAT.md` §37.5):
        // on the thirty-one frames in thirty-two that are not the search's,
        // past an idle of 1, a captain whose cached nearest (`near_o`) is
        // still standing, whose combat stance is under 5 (vslot `+0xf4` is
        // `get_combat_stance`: anything but `HOLD_FIRE`) and which is
        // `is_in_range@00648d70` of it (margin 0, the slot and never the
        // health: [`Sim::is_in_range_struck`]) takes it —
        // `add_attack_order(near_o, QUEUE_NEW, 0, 0)` — clears its
        // could-not-reach bit, and returns; a packed packer whose packer
        // stance is not `PACKER_AUTO` (vslot `+0x100`) returns without it.
        // Every other answer skips the search to the tail (`5f7179`). The
        // search writes `near_o` whether or not it takes a candidate, so
        // this is what gives a computer's raiding ship the finished
        // building its `compare_target` weighed at 0 (run809's `1/113` on
        // 3647).
        if unit.idle != 1
            && phase & 0x1f != 0
            && let Some(t) = unit.near
            && unit.combat.stance != combat::Stance::HoldFire
            && self.is_in_range_struck(me, t)
        {
            let packer_auto = self.unit_stance_type(u) == crate::group::StanceType::Packer
                && self.units[u].stance == PACKER_AUTO;
            if self.units[u]
                .ty
                .is_some_and(|ty| self.unit_types[ty].combat.packs)
                && self.units[u].combat.packed
                && !packer_auto
            {
                return;
            }
            self.add_attack_order(u, t, QueuePos::New, false, false);
            self.units[u].cant_reach = false;
            return;
        }
        let unit = &self.units[u];
        // Step 3's own cadence — the first idle frame and then one in
        // thirty-two, phased by `o` (`think@005f6e40`, the arm at
        // `5f70fd`). Past it the original enters `Unit::think_attack`,
        // whose **head** joins an army before its target search ever runs
        // (`docs/ARMY.md` §4.2): that is how the AI's first soldier gets
        // into one, and this crate had only the search.
        //
        // **The military bit belongs to the whole arm, not only to the
        // join** (`docs/COMBAT.md` §33). `think@005f6e40:150` reads
        //
        //     (is(0x3e, 1) && (merchant arm || (!packing && think_attack())))
        //     || (type->attack != 0 && (type->role & 0x10000) != 0 && think_attack())
        //
        // — so a type with an attack column and no `role & 0x10000` never
        // enters `think_attack` at all. [`Sim::think_attack_join_army`]
        // has carried that pair since item 350 and the search beside it
        // carried only the attack, so **a citizen** — `attack 40`,
        // `obj_masks & CIVILIAN`, no military bit — took an attack order
        // on its first idle frame. The base column is the original's
        // (`+0x1e8`), not the runtime stat.
        //
        // **The Dutch merchant's leg** of the same disjunction (item 3329,
        // `docs/MERCHANT.md` §4): a unit of the `MERCHANTDUTCH` lineage
        // (`is(0x3e, 1)`) that is packed asks `think_merchant` here, on the
        // auto-attack's cadence — the first idle frame and then one in
        // **thirty-two**, phased by `o` — and a `1` back ends the think.
        // The tail's own `is_merchant` call runs one in a hundred and
        // twenty-eight, so without this leg a Dutch merchant searched a
        // quarter as often: run963's `1/2` (`TypeIndex` 62, land), idle
        // since before 1561, took its `MOVE_TO` (31800, 37176) on 1566 in
        // theirs, `(2 + 1566) & 31 == 0` and `& 127 == 32`, and stood idle
        // in ours (the battery's Dutch word, 1567).
        //
        // **And the leg's other half** (item 3369, `docs/AI.md` §285): a
        // packed merchant whose trade search answered 0, or a deployed
        // one, asks `think_attack` when its current order is neither a
        // pack nor an unpack (`UnitData::is_packing_or_unpacking@0060a410`)
        // — `(packed && think_merchant) || (!packing_or_unpacking &&
        // think_attack)`. `think_attack` skips its packed arm for the
        // lineage and its army join for the three merchant ids, so what it
        // can do for a merchant is the target search and the tail's walk
        // to a city ([`Sim::think_attack_merchant`]). run995's `1/1`
        // (`TypeIndex` 62) is the diff: seated by its unpack on 309, on
        // 310 it was in group 64 with an `ATTACK_TO` (41304, 17304) in
        // theirs and idle in ours.
        let dutch = self
            .unit_tree(u)
            .is_some_and(|ti| self.tech_tree.is(ti, MERCHANTDUTCH, true));
        if dutch && (unit.idle == 1 || phase & 0x1f == 0) {
            if unit.combat.packed && self.think_merchant(u) {
                return;
            }
            if !self.is_packing_or_unpacking(u) && self.think_attack_merchant(u) {
                return;
            }
        }
        let unit = &self.units[u];
        let p = self.profile(me);
        if p.attack != 0
            && p.combat_role
            && (unit.idle == 1 || phase & 0x1f == 0)
            && !self.think_attack_packed(u)
        {
            let joined = self.think_attack_join_army(u);
            let unit = &self.units[u];
            // **The packed computer siege engine takes its army's target
            // first** (`find_nearby_target@00648da0:198`–`222`, reached
            // from `think_attack`'s `find_melee_target`; item 2864,
            // `docs/AI.md` §238): `QUEUE_FIRST`, **mandatory**, no group
            // normalize, no `near` write — the search's own return. The
            // peace pair's idle MLRS `1/124` (`TypeIndex` 273) took the
            // human's `0/2000` this way on 6820, `mandatory 1`, and on
            // 6821 walked the ring that ours, holding a found target's
            // `mandatory 0`, dropped.
            if unit.combat.stance != combat::Stance::HoldFire
                && unit.combat.target.is_none()
                && self.take_siege_army_target(u).is_some()
            {
                return;
            }
            let unit = &self.units[u];
            if unit.combat.stance != combat::Stance::HoldFire
                && unit.combat.target.is_none()
                && let Some(t) = self.with_second_range(u, |s| s.find_melee_target(u, -1))
            {
                // The search runs at a melee-and-ranged type's second range
                // (`think_attack@005f5a80:167`–`174`, item 3383), the
                // bracket `cavarch_fight` puts round its `fight`.
                // `find_melee_target(−1, &whom, 0, 1, 0)` adds its own
                // order (`find_nearby_target`'s add, [`Self::nearby_add`]):
                // `QUEUE_FIRST` under an attack-move, so the walk
                // `add_to_army` just issued stays beneath the attack. A
                // `QUEUE_NEW` here dropped run818's `1/9` walk on 701
                // (item 2117, `docs/GOLDEN.md` §77).
                self.nearby_add(u, t);
                // `think_attack`'s tail (`005f5da6`..): a **unit** find by
                // a unit its group lists is handed to
                // `Group::target_opportunity(…, param_4 = 1)`
                // (`docs/GOLDEN.md` §61).
                if matches!(t, Obj::Unit(_))
                    && let Some(seat) = self.seat_of(u)
                    && self.seat_list(seat).contains(&u)
                {
                    self.group_target_opportunity(seat, t, u, true);
                }
                // **Only a unit find ends the think** (`005f5da6`..: the
                // `return 1` is gated on the target's vslot `+0x18`,
                // `is_unit`). A building's attack order is added all the
                // same, and `think_attack` returns 0, so `Unit::think`
                // goes on to its tail — the scout's explore among it. The
                // East Indies Spanish Caravel `1/49` found the human's
                // `0/2004` on 12742 and still ran `think_scout` (item 3367).
                if matches!(t, Obj::Unit(_)) {
                    return;
                }
            }
            if joined.is_none() {
                self.think_attack_tail(u);
            }
        }
        // `005f7195`: a worker whose `think_peasant` **found something**
        // ends the think there — the listing's `goto LAB_005f761a`, which
        // is the function's own exit. Ignoring the return value put a
        // citizen that had just been given a gather job through the tail
        // below on the same frame.
        if self.worker_of(u) != Worker::None && self.think_peasant(u, false) {
            return;
        }
        // Step 5's first arm, and it sits **above** the tail's gate: a
        // caravan takes `think_caravan`, whose own idle threshold is its
        // only cadence (`crate::caravan` §3).
        if self.units[u].caravan.is_some() && self.think_caravan(u) {
            return;
        }
        // **The human rare collector's deploy** (`think@005f6e40:163`–`199`,
        // `docs/ORDERS.md` §23), between the caravan and the computer
        // block, and so **above** `ai off`'s exit. A packed merchant or
        // fishing boat whose leader is human (`leader_flags & 4`) asks
        // `do_gather(…, 1, 1, −1, −1)` — `calc_gather` with `param_7` set,
        // [`Sim::calc_gather`] — where it stands, and if that answers
        // `unpack_merchant(this, 4)`: the ring-4 spot search and the
        // `[MOVE_TO, CAST]` pair. A computer's boat never enters it; its
        // deploy is `think_fish`'s, in the tail below. run127's fisher
        // `0/7` is the diff: born on 620 under a human leader, it walks
        // to (11736, 35352) and casts `0x292` for forty frames, and the
        // deploy lands on block 665 — golden chapter five's word 664.
        //
        // The cadence is the tail's (`idle == 1`, else one in thirty-two
        // phased by `o`). A search that does not deploy bumps `idle` a
        // second time this frame (`:196`); the console's message and
        // `S_INVALID_ORDER` beside it are interface.
        //
        // SEAM: `(unit_masks & 0x100) == 0 || is_merchant` — the "ordered
        // recently" bit (`docs/MERCHANT.md` §7), which this crate does not
        // keep, so a boat is always taken as not recently ordered.
        if self.is_rare_collector(u)
            && self.nation[self.units[u].owner as usize].human
            && self.units[u].combat.packed
            && (self.units[u].idle == 1 || phase & 31 == 0)
        {
            if self.calc_gather(u) && self.unpack_merchant(u, 4) {
                return;
            }
            self.units[u].idle = self.units[u].idle.wrapping_add(1);
        }
        // **The computer block, and the second place `ai off` is read**
        // (`docs/INPUT.md` §11). `think@005f6e40:205` opens
        // `if ((leader_flags & 4) != 0 || ai_off != 0)`, and its only
        // unconditional statement is the exit below it:
        // `if ((unit_masks & 0x40000) == 0) goto LAB_005f761a` — the
        // function's return. The arms inside are the computer leader's
        // alone (`uVar4 != 0` guards them), so for a human leader with the
        // cheat on the block is exactly one thing: **a unit that is not
        // AI-driven loses the whole tail** — `think_fish`,
        // `think_merchant`, `think_scout`, `think_carry`, the army join.
        // Everything above this line — the auto-attack arm,
        // `think_peasant`, `think_caravan` — is untouched, which is what
        // `docs/RUNS.md` run101–run105 measured as "auto-engage survives
        // AI-off".
        //
        // **Why the predicate is one term here and two there.** This crate
        // has a single stand-in, `ai_driven(owner)`, for both of the
        // original's words — `leader_flags & 4` on the leader and
        // `unit_masks & 0x40000` on the unit. Substituting it for both,
        // `(x || ai_off) && !x` is `ai_off && !x`, which is what stands.
        // SEAM: the two are not the same word in the original — a human
        // who leaves the auto-manage option on has citizens carrying
        // `0x40000` under a leader that is not computer-controlled — so
        // when a per-unit bit lands, this reverts to the two-term form.
        if self.ai_off && !self.ai_driven(self.units[u].owner) {
            return;
        }
        // **The tail's own cadence gate** (§2.4 step 5), the second of the
        // two in this function and the one the code was missing: after the
        // human block's `unit_masks & 0x40000` exit, `think@005f6e40`
        // returns when
        //
        //     idle != 1 && ((o + frame) & 31) != 0
        //
        // — so everything from `think_fish` down, the tail included, runs
        // on a unit's **first** idle frame and then once in thirty-two,
        // phased by `o`. The mod-16 gate above only lets the function be
        // entered; this one is what decides the tail.
        //
        // run56's scout is the diff: it arrives on 2664 with `idle == 1`,
        // thinks, and then — with `o == 0` — thinks again on 2688, 2720,
        // 2752, … exactly. Without this gate the simulation ran a whole
        // second `think_scout` on 2665, where `idle == 2` still passes the
        // mod-16 gate, and that was East Indies' word.
        if self.units[u].idle != 1 && phase & 31 != 0 {
            return;
        }
        // `is(0x13d, 0)` — the `FISHERMEN` lineage — takes `think_fish`,
        // and a `1` back ends the think (`docs/ORDERS.md` §6.8). It sits
        // above the scout/army tail and below everything else, which is
        // why a fishing boat never joins an army.
        if self.unit_line_is(u, crate::fish::FISHERMEN) && self.think_fish(u, frame) {
            return;
        }
        // `is_merchant` — the three exact ids — takes `think_merchant`
        // (`crate::merchant`), and it has a **cadence of its own** on top
        // of the tail's: `think@005f7515` runs it on the first idle frame
        // and then one frame in a hundred and twenty-eight, phased by
        // `o`, where everything either side of it runs one in thirty-two.
        if self.is_merchant(u)
            && (self.units[u].idle == 1 || phase & 127 == 0)
            && self.think_merchant(u)
        {
            return;
        }
        // `think@005f74xx`'s carry arm (`docs/AI.md` §195): a type with a
        // `CARRY` that is not strictly an aircraft carrier asks
        // `think_carry`, and a `1` ends the think. It sits between the
        // merchant and the supply/hero/scout tail.
        // SEAM: `leader_flags & 8` (never set) returns above it.
        if self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.carry != 0 && !self.is_aircraft_carrier(t))
            && self.think_carry(u)
        {
            return;
        }
        // The tail (`docs/SCOUT.md` §2): a scout or a spy not in an army
        // takes `think_scout`; a supply wagon or a hero takes
        // `add_to_army` (`docs/ARMY.md` §4). The two are exclusive in the
        // original and are kept so — and nothing else joins anything.
        if self.scout_thinks(u) {
            self.think_scout(u);
            return;
        }
        self.think_join_army(u);
    }

    /// `Unit::think_attack@005f5a80`'s **packed-unit arm**
    /// (`docs/COMBAT.md` §51), between the function's stance head and its
    /// army join — so above both the join and the target search. `true`
    /// is the function's `return 0` before either: the think goes on
    /// past the auto-attack arm exactly as if nothing had been found.
    ///
    /// The listing (`llvm-objdump 0x5f5b0a..0x5f5c00`):
    ///
    /// ```text
    /// if type.unit_flags2 & 4 and unit_masks & 0x80000:      # packs, packed
    ///     if !is(MERCHANTDUTCH 0x3e, 1):
    ///         if get_packer_stance() == PACKER_AUTO:          # vslot +0x100
    ///             thr = is(MACHINEGUN 0x7b, 0) ? 3
    ///                 : leader_flags & 4 ? 7 : 0x15           # human : computer
    ///             if idle >= thr:
    ///                 add_cast_order(-1, -1, -1, -1, 0x28c, QUEUE_FIRST, 0)
    ///                 return 0
    ///         if manual: return 0                             # 5f5bf6
    /// ```
    ///
    /// `manual` is the head's: not AI-driven (`unit_masks & 0x40000`
    /// clear), or AI-driven under a leader without `leader_flags & 2`, or
    /// `leader_flags2 & 8` — this crate's [`Sim::think_attack_join_army`]
    /// reads the same word as `!ai_driven || defeated`, and so does this.
    /// `get_packer_stance@00610970` is the unit's `stance` byte when the
    /// type's stance type is `STANCE_PACKER` (3), else `PACKER_NEVER` (1);
    /// `PACKER_AUTO` is 0 (the PDB's `LF_ENUMERATE`s).
    ///
    /// So **a human's packed siege engine never searches**: it waits,
    /// packed and orderless, for the first auto-attack frame with `idle ≥
    /// 7`, and its first order is the unpack. run145's catapult `0/9` is
    /// the diff — born on 621 at `idle 1`, cast on 696 at `idle 7`,
    /// unpacked on 776 — and run146's `0/6` is the cadence's: `idle 7` on
    /// 683, which is a 16-phase frame and not a 32-phase one, and the cast
    /// on 699 at `idle 8`. A computer's packed unit (`manual` clear) falls
    /// through to the join and the search while it waits out its 21.
    ///
    /// SEAM: `is(0x3e, 1)` is read as the strict lineage of the Dutch
    /// merchant, and a merchant never reaches this arm through the
    /// military one.
    pub(super) fn think_attack_packed(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.unit_types[t].combat.packs || !self.units[u].combat.packed {
            return false;
        }
        if self
            .unit_tree(u)
            .is_some_and(|ti| self.tech_tree.is(ti, MERCHANTDUTCH, true))
        {
            return false;
        }
        let who = self.units[u].owner;
        if self.unit_stance_type(u) == crate::group::StanceType::Packer
            && self.units[u].stance == PACKER_AUTO
        {
            let threshold = if self.unit_line_is(u, MACHINEGUN) {
                3
            } else if self.nation[who as usize].human {
                7
            } else {
                0x15
            };
            if self.units[u].idle >= threshold {
                self.add_cast_order(u, spell::UNPACK);
                return true;
            }
        }
        !self.ai_driven(who) || self.defeated[who as usize]
    }

    /// `Unit::think@005f6e40`'s opening arm — the **captain mirror**
    /// (`docs/COMBAT.md` §21). A unit with a captain takes that captain's
    /// standing ATTACK order and nothing else; the think ends here whether
    /// the mirror fires or not.
    ///
    /// Four things it is not. It is not gated by the stance, by
    /// `unit_masks & 0x100`, or by either cadence — an idle member mirrors
    /// on the first frame it is idle. It reads the captain's **action**
    /// (`UnitData::get_action`, the intent under the pathing legs), so a
    /// captain walking to a chase point still hands its target down. It
    /// tests `Object::valid_target` on the **member**, not on the captain.
    /// And the order it adds is `QUEUE_NEW` with the captain's own
    /// `mandatory` byte — `add_attack_order`'s `action` argument is 0, so
    /// a DEFENSIVE member still takes a post.
    ///
    /// **The frame it explains**, twice over (`docs/COMBAT.md` §21): who=1's
    /// three hoplites all carry `type 10 ox 7 whom 0` at the end of frame
    /// 615, the frame they are born, and only the captain `1/6` carries a
    /// `near_o`; and who=0's `0/7`/`0/8` take their retaliation ATTACKORDER
    /// on **617**, one frame after the captain `0/6` took it at 616, because
    /// the captain's order is written after they have already been processed.
    pub(crate) fn captain_mirror(&mut self, u: usize) {
        let cap = self.squad_captain(u);
        if cap == u {
            return;
        }
        // `get_action`'s order must be an ATTACK — `get_type() == 10`.
        let Some(i) = self.action_of(cap) else {
            return;
        };
        if !matches!(self.units[cap].orders[i].body, Body::Attack(_)) {
            return;
        }
        let Some(t) = self.units[cap].combat.target else {
            return;
        };
        if !self.valid_target(Obj::Unit(u), t) {
            return;
        }
        let mandatory = self.units[cap].combat.mandatory;
        self.add_attack_order(u, t, QueuePos::New, mandatory, false);
    }

    /// `Unit::think_peasant(forced)` (§5.9): the idle gate, the colonist
    /// arm, then the job search — here `find_gather_spot`;
    /// `find_build_spot`/`find_repair_spot` need the object searches and are
    /// not yet modelled.
    ///
    /// **The gate's threshold is 1 for an AI-driven worker**, whatever the
    /// owner's idle-citizen option says: `think_peasant@005f5760:16` reads
    /// the option's switch only when `unit_masks & 0x40000` is clear, and
    /// takes 1 when it is set. So a computer player's new citizen finds a
    /// job on the *first* frame it is idle. That one frame is what run33's
    /// trained citizen spends: it comes out at 99, is first visited at 100,
    /// and its walk — and the collision that ends it at 122 — hangs off that
    /// frame (`docs/SYNC.md` §3.16).
    ///
    /// **A human's is [`crate::stance::LeaderOptions::idle_wait`], and it is
    /// 12** — the option's switch, not the option itself (item 494,
    /// `docs/ORDERS.md` §21). Reading `peasants_wait` as the wait put the
    /// human's citizen back to work ten frames early, which is Great Lakes'
    /// word at 10294: `0/5` takes its flight to `(792, 31800)` with a
    /// `GATHERORDER` queued under it, so it never has the single order
    /// [`Self::arrive`] needs to face the order's angle, and the frame the
    /// original spends in `do_idle` it spends walking away.
    pub fn think_peasant(&mut self, u: usize, forced: bool) -> bool {
        let unit = &self.units[u];
        let ai = self.ai_driven(unit.owner);
        if !forced {
            let t = if ai {
                1
            } else {
                i32::from(self.leader_options(unit.owner).idle_wait())
            };
            let idle = i32::from(unit.idle);
            if idle < t {
                return false;
            }
            if idle != t && (idle - 2).rem_euclid(5) != 0 {
                return false;
            }
        }
        // **The colonist arm** (`think_peasant@005f5760:53`,
        // `docs/TRANSPORT.md` §7): an AI-driven **citizen** — the base type
        // itself, `0x32` or `0x33`, not the worker category, so a scholar
        // never asks — offers itself for the boat *before* it looks for a
        // job, and a `1` back ends the think. It is the only caller with
        // `colonise = 1`, and so the only writer of the census's
        // `xport_peasants` throttle between sweeps.
        if ai
            && matches!(self.units[u].type_index, 0x32 | 0x33)
            && self.think_civilian_transport(u, true)
        {
            return true;
        }
        let unit = &self.units[u];
        let stance = unit.stance;
        // The same call's other half of the AI split: an AI-driven worker
        // searches without a range limit (§5.9's `find_gather_spot(AI ? −1
        // : UNIT_GATHER_RESPOND_RANGE × 192)`), as a scholar does.
        let range = if ai {
            -1
        } else {
            self.tuning.unit_gather_respond_range * TILE
        };
        // §5.9's **build arm**, ahead of the gather search: `not a scholar
        // and (unit_masks & 0x400 or worker_stance ∈ {1, 2}) and
        // find_build_spot()`. It waited on `stance` (item 190) and could
        // not land while this crate wrote a flat 1 — with that, every
        // citizen asked, and run69's `1/6` left the original's point on
        // **103**. `Unit::init` and `Build::train` between them make an
        // AI's trained citizen **0**, so the arm now asks who the original
        // asks (`crate::stance`, §5.10).
        if !matches!(self.units[u].type_index, 0x34 | 0x35)
            && (self.units[u].was_builder || stance == 1 || stance == 2)
            && self.find_build_spot(u)
        {
            return true;
        }
        if stance <= 1 && self.find_gather_spot(u, range) {
            // **A human's found gather drops its group** (§5.9; the
            // listing at `5f5900`–`5f590c`: `testl $0x40000, 0x68(%esi)`,
            // and for a human `orl $-1` into `movw %ax, 0x80(%esi)`). The
            // pool keeps listing the unit; only the back-pointer goes, so
            // `find_ordered_collision`'s own-group arm stops seeing the
            // swarm it came from. Without it, chapter twenty-one's `0/7`
            // was refused the camp spot `0/8` was already sent to and
            // walked a tile west (item 824, `docs/GOLDEN.md` §29).
            //
            // SEAM: the `deselect` ahead of it (`5f58d2`–`5f58fd`, the
            // console player's select list, vslot `+0x10`, when it holds
            // more than one) is the UI's selection, which this crate does
            // not model.
            if !ai {
                self.units[u].group_ptr = None;
            }
            return true;
        }

        // **The tail, `LAB_005f5920`** — everything below the job search,
        // and the arm East Indies' word at 4313 was (`docs/SCOUT.md`
        // §11.1). A **scholar** (`0x34`/`0x35`) skips the whole of it,
        // `unit_masks & 0x400` included, so it is the one worker that
        // keeps "has been a builder" across a failed search.
        if matches!(self.units[u].type_index, 0x34 | 0x35) {
            return false;
        }
        // A **human's** arm here is `(was_builder or stance ∈ {1, 2}) and
        // find_repair_spot()`; the function is built (item 3187) but this
        // human call is left unwired (`docs/ORDERS.md` §5.9).
        self.units[u].was_builder = false;
        if !ai {
            return false;
        }

        // The AI's: a worker standing in a region where its leader has no
        // city, and which none of the leader's ten sites claims, gives up
        // on the region and **explores** — `think_scout(0)` from here, the
        // second of that function's three call sites and the only one a
        // citizen ever reaches (`docs/SCOUT.md` §11.1). A region a site
        // does claim is worth waiting in, but only for six idle frames.
        //
        // The region is `get_tregion@006b52e0`, the coastal-refined one
        // (`World::tregion_alt`), as the listing calls it.
        let who = self.units[u].owner as usize;
        let t = self.units[u].pos.tile();
        let Some(region) = self.world.tregion_alt(t) else {
            return false;
        };
        if crate::ai::Census::reg(&self.ai[who].census.reg_cities, region) == 0 {
            let claimed = self.ai[who]
                .sites
                .iter()
                .any(|s| s.val != 0 && s.reg == i32::from(region));
            if !claimed || self.units[u].idle > 6 {
                return self.think_scout(u);
            }
        }
        // The AI's tail: `find_repair_spot` (`docs/ORDERS.md` §5.9).
        self.find_repair_spot(u)
    }

    /// `Unit::do_guard@005e5c70` (`docs/ORDERS.md` §7.5, §24) — one frame
    /// of an escort.
    ///
    /// A dead target ends the order with an idle stand. A pending `retry`
    /// counts down and does nothing else. Beside a target that is not
    /// moving, one frame in sixteen looks for a fight and another counts
    /// `idle`. Otherwise the post is recomputed — the target's position
    /// plus `(dx, dy)` turned into the target's own frame, snapped to its
    /// 48-unit cell — and a unit off that cell is given an `ATTACK_TO`
    /// leg to it at the head of its list, stepped **this same frame**,
    /// with a `timer` of thirty frames a world cell of Manhattan distance.
    /// A unit on its post turns to the facing and stands.
    ///
    /// The four fastcall pairs are the listing's, not the decompiler's
    /// (`tools/ghidra/README.md`): the post is `x + sinx(a, dy) + cosx(a,
    /// dx)`, `y − cosx(a, dy) + sinx(a, dx)` with `a` the target's heading
    /// (`5e6022`–`5e6061`), and the idle facing is `find_angle(post −
    /// target)` — outward, away from what is guarded (`5e626f`).
    ///
    /// The target's own `unit_masks & 2` ([`crate::Movement::mirror`])
    /// negates `dx` before the turn (`5e5fed`–`5e5ff6`): run196's Despot
    /// carries it on 15095, and its escort's flanks stand on the other
    /// side for it (`docs/GROUPS.md` §25).
    ///
    /// SEAM, none of them reached by a capture on file: the building-target
    /// arm (`action_guard`'s building arm
    /// is itself a seam, so no building is ever a target here); and the
    /// sixteen-frame engagement, which asks [`Self::find_melee_target`]'s
    /// idle radius rather than `find_melee_target(−1, 0, 0, 1, 0)`'s own
    /// guard arm.
    /// `Unit::do_patrol@005f1910` (`docs/ORDERS.md` §27) — a patrol at the
    /// head of the list.
    ///
    /// - **The leader** (`order.leader` is this unit): step `waypoint`
    ///   modulo the point count and hand the group the next leg,
    ///   `Group::action_move_to(group, point, QUEUE_FIRST, 0, 0,
    ///   ATTACK_TO, action 0, −1, −1, 0)`. A group's `QUEUE_FIRST` halts
    ///   every member and re-issues the leader's action orders behind the
    ///   leg — the patrol among them, through `redo_patrol_order`
    ///   ([`Self::group_redo_patrol_order`]) — so the step is carried to
    ///   every member's rebuilt patrol.
    /// - **A follower**: `set_anim(CHAR_DEFAULT, 0, 1)`, and nothing else;
    ///   its legs are the leader's group moves.
    ///
    /// SEAMS: the arm for a unit in no group (`+0x80 < 0`), which steps the
    /// waypoint and pushes a bare `ATTACK_TO` of its own — no command
    /// reaches it, because `process_group` forces the push even for one
    /// unit; and the tail's `inside_down >= 0` scramble, a transport's.
    pub(crate) fn do_patrol(&mut self, u: usize, p: PatrolOrder) {
        let Some(g) = self.group_of(u) else {
            return;
        };
        if p.leader != u {
            self.set_default_anim(u);
            return;
        }
        let waypoint = (p.waypoint + 1) % p.points.len();
        if let Some(Body::Patrol(x)) = self.units[u].orders.front_mut().map(|o| &mut o.body) {
            x.waypoint = waypoint;
        }
        self.group_action_move_to(
            &g,
            p.points[waypoint],
            QueuePos::First,
            false,
            Angle(0),
            MoveKind::AttackTo,
            false,
        );
    }

    pub(super) fn do_guard(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Guard(mut g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let store = |sim: &mut Sim, at: usize, g: GuardOrder| {
            if let Some(Body::Guard(x)) = sim.units[u].orders.get_mut(at).map(|o| &mut o.body) {
                *x = g;
            }
        };
        let t = g.target;
        // `ObjectData::flags & 1` — the target's slot is still an object.
        if !self.units[t].alive() {
            self.mark(SITE_GUARD_DEAD);
            self.set_anim(u, anim::DEFAULT, false, true);
            self.kill_current_order(u);
            return;
        }
        if g.retry != 0 {
            g.retry -= 1;
            store(self, 0, g);
            return;
        }
        // `is_active() && is_on_map() && is_moving()` (vslots `+0x8`,
        // `+0xbc`, `+0xd8`): `iStack_9e4`.
        let moving = self.units[t].on_map && self.is_moving(t);
        if !moving {
            let ph = self.units[u].phase(frame);
            if (ph + 8).rem_euclid(16) == 0 {
                // `find_melee_target@005ff9c0`'s head answers a follower
                // first (item 1960, `docs/GOLDEN.md` §71, run801): nothing
                // when its captain is not attacking, the captain's target
                // when it is. Only a captain, or a follower whose
                // captain's target has gone, searches.
                self.find_melee_target_added(u);
                return;
            }
            if ph.rem_euclid(16) == 0 {
                g.idle += 1;
                store(self, 0, g);
                return;
            }
        }
        let a = self.units[t].movement.heading;
        let tp = self.units[t].pos;
        // `5e5fed`: the target's own `unit_masks & 2` mirrors the offset
        // (`docs/GROUPS.md` §25).
        let dx = if self.units[t].movement.mirror {
            -g.dx
        } else {
            g.dx
        };
        let gx = tp.x + movement::sin_component(a, g.dy) + movement::cos_component(a, dx);
        let gy = tp.y - movement::cos_component(a, g.dy) + movement::sin_component(a, dx);
        // `div_3_table[v >> 4]`: the 48-unit cell, a floor divide.
        let q = |v: i32| (v >> 4).div_euclid(3);
        let (mut qx, mut qy) = (q(gx), q(gy));
        if self.invalid_loc(
            u,
            Pos::new(qx >> 2, qy >> 2),
            false,
            false,
            false,
            false,
            false,
        ) != 0
        {
            let centre = Pos::new(qx * SNAP + SNAP_CENTRE, qy * SNAP + SNAP_CENTRE);
            let transport = self.units[u].ty.is_some_and(|ty| {
                self.unit_types[ty]
                    .cols
                    .flag(crate::ai_load::uflags::FLAG_TRANSPORT)
            });
            let water = self.world.tile_mask(Pos::new(qx >> 2, qy >> 2)) & tile::SURFACE
                == tile::SURFACE_OCEAN;
            let spot = if !transport || water {
                let bearing = Angle(0x5555_5555);
                self.find_nearby_spot(u, centre, 0xc0, 0x180, 0x60, bearing, None)
                    .or_else(|| {
                        let d = vector_dist(g.dx, g.dy);
                        self.find_nearby_spot(u, tp, d, d + 0x180, 0xc0, bearing, None)
                    })
                    .unwrap_or(self.units[u].pos)
            } else {
                centre
            };
            (qx, qy) = (q(spot.x), q(spot.y));
        }
        g.guard = Pos::new(qx * SNAP + SNAP_CENTRE, qy * SNAP + SNAP_CENTRE);
        let mut facing = if moving {
            a
        } else {
            find_angle(g.guard.x - tp.x, g.guard.y - tp.y)
        };
        // `5e6297`: an AI-driven siege engine, wagon or hero faces where
        // its charge faces, moving or not.
        if self.ai_driven(self.units[u].owner)
            && (self.is_siege_unit(u) || self.is_supply_unit(u) || self.is_hero_unit(u))
        {
            facing = a;
        }
        let here = self.units[u].pos;
        if q(here.x) != qx || q(here.y) != qy {
            g.idle = 0;
            store(self, 0, g);
            self.add_move_facing_order(
                u,
                g.guard,
                MoveKind::AttackTo,
                QueuePos::First,
                false,
                facing,
                None,
                false,
            );
            // `div_3_table[v >> 8]`: the world cell, 768 units.
            let cell = |v: i32| (v >> 8).div_euclid(3);
            let d = (cell(here.x) - cell(g.guard.x)).abs() + (cell(here.y) - cell(g.guard.y)).abs();
            if let Some(m) = self.units[u].orders.front_mut().and_then(Order::move_mut) {
                m.timer = 0x1e * d.max(1);
            }
            self.do_move(u, frame);
            if self.order_type(u) != index::GUARD {
                return;
            }
            self.mark(SITE_GUARD_STAND);
            self.set_anim(u, anim::DEFAULT, false, true);
            self.mark(SITE_GUARD_RETRY);
            let r = self.rng.roll();
            if let Some(Body::Guard(x)) = self.units[u].orders.front_mut().map(|o| &mut o.body) {
                x.retry = r % 3 + 6;
            }
            return;
        }
        if self.units[u].movement.heading != facing && !moving {
            self.unit_set_angle(u, facing);
        }
        g.idle += 1;
        store(self, 0, g);
        if self.guard_unpacks(u, g.idle) {
            return;
        }
        self.mark(SITE_GUARD_IDLE);
        self.set_anim(u, anim::DEFAULT, false, true);
    }

    /// **A packed packer on its post unpacks** (`do_guard@005e5c70:298`–
    /// `:316`, `docs/AI.md` §170): past the idle increment, a type that
    /// packs (`+0x2b8 & 4`) whose packer stance is `PACKER_AUTO` (vslot
    /// `+0x100`), standing packed (`unit_masks & 0x80000`) and not already
    /// unpacking, casts its unpack at `QUEUE_FIRST` once the guard's `idle`
    /// reaches `0x1e` for the machine gun's lineage (`is(0x7b, 0)`) and
    /// `0x46` for any other — and returns, so the stand's `set_anim` is not
    /// asked. `add_cast_order` re-aims the machine gun's `0x28c` at `0x28e`.
    /// The coverage pair's Advanced Machine Gun `1/74` guarding `1/72` casts
    /// on 2868 at `idle 30`; its unpacking body is what the AA missile
    /// `1/94`'s step meets that frame.
    pub(super) fn guard_unpacks(&mut self, u: usize, idle: i32) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.unit_types[t].combat.packs
            || self.unit_stance_type(u) != crate::group::StanceType::Packer
            || self.units[u].stance != PACKER_AUTO
            || !self.units[u].combat.packed
            || self.is_unpacking(u)
        {
            return false;
        }
        let threshold = if self.unit_line_is(u, MACHINEGUN) {
            0x1e
        } else {
            0x46
        };
        if idle < threshold {
            return false;
        }
        self.add_cast_order(u, spell::UNPACK);
        true
    }

    /// `Unit::do_follow@005e65d0` (`docs/ORDERS.md` §28), with the
    /// `FOLLOW` at the head.
    ///
    /// A leader no longer active and on the map, or not seen by the
    /// follower's player (`UnitData::is_seen@00607a60(who, 0)`, which
    /// answers 1 for the owner), kills the order. Otherwise the standoff:
    ///
    /// - `k = los × 0x60` when the follower is the faster
    ///   (`UnitData::speed@0060aae0`), else `los × 0x300 / 5`, truncated;
    /// - `k` doubles while the leader `is_moving`;
    /// - `s = clamp(los × 0x180 − k, 0x180, 0x600)`, `los` the follower's.
    ///
    /// Within `s + 0xc0` of the leader (`vector_dist`), the follower stands:
    /// `set_anim(CHAR_DEFAULT, 0, 1)`. Farther, the point `s` from the
    /// leader toward the follower, placed by `find_nearby_spot`; failing
    /// that the point `s` behind the leader's heading; then a ring
    /// `s .. s + 0xc0` round the leader on its heading; then the leader's
    /// own place. A `MOVE_TO` leg goes on at `QUEUE_FIRST` without the
    /// action bit, facing the leader's heading, and `do_move` runs this
    /// frame (`5e6b1c`–`5e6b29`). The leg has no timer.
    ///
    /// SEAM: the container swap at the head (`5e6643`–`5e66fd`) and its
    /// tail (`5e6b5b`, the swap back and `Unit::work`), with
    /// [`FollowOrder`]'s `oxx/whose/uid2`.
    pub(super) fn do_follow(&mut self, u: usize, f: FollowOrder, frame: i64) {
        let t = f.target;
        let who = self.units[u].owner;
        let seen = self.units[t].owner == who || self.target_is_seen(Obj::Unit(u), Obj::Unit(t));
        if !(self.units[t].alive() && self.units[t].on_map) || !seen {
            self.kill_current_order(u);
            return;
        }
        let me = self.units[u].pos;
        let tp = self.units[t].pos;
        let d = vector_dist((tp.x - me.x).abs(), (tp.y - me.y).abs());
        // `UnitData::los` (vslot `+0x128`, `5e66e1`…), the CEO's term with it.
        let los = self.unit_data_los(u);
        let mut k = if self.unit_speed(t) < self.unit_speed(u) {
            los * 0x60
        } else {
            los * 0x300 / 5
        };
        if self.is_moving(t) {
            k *= 2;
        }
        let s = (los * 0x180 - k).clamp(0x180, 0x600);
        if d <= s + 0xc0 {
            self.mark(SITE_FOLLOW_STAND);
            self.set_anim(u, anim::DEFAULT, false, true);
            return;
        }
        let heading = self.units[t].movement.heading;
        let bearing = Angle(0x5555_5555);
        let toward = crate::army::step_along(tp, find_angle(me.x - tp.x, me.y - tp.y), s);
        let behind = crate::army::step_along(tp, Angle(heading.0.wrapping_sub(i32::MIN)), s);
        let spot = self
            .find_nearby_spot(u, toward, 0, -1, 0, bearing, None)
            .or_else(|| self.find_nearby_spot(u, behind, 0, -1, 0, bearing, None))
            .or_else(|| self.find_nearby_spot(u, tp, s, s + 0xc0, 0x30, heading, None))
            .unwrap_or(tp);
        self.add_move_facing_order(
            u,
            spot,
            MoveKind::MoveTo,
            QueuePos::First,
            false,
            heading,
            None,
            false,
        );
        self.do_move(u, frame);
    }

    /// `Unit::do_think_order`: consume the THINK; if nothing follows, a
    /// worker runs the forced search.
    pub(super) fn do_think_order(&mut self, u: usize, _frame: i64) {
        think_order_steps(
            self,
            |s| s.kill_current_order(u),
            |s| (!s.units[u].orders.is_empty()).then(|| i32::from(s.order_type(u))),
            |s| s.worker_of(u) != Worker::None,
            |s| {
                s.think_peasant(u, true);
            },
        );
    }
}

/// **`Unit::do_think_order@005e5bf0`'s steps** (item 2287):
/// `kill_current_order(0)`; then, when an order is left (the list's head,
/// `+0xdc`), its type (vslot `+0x10`) ends the call unless it is `NONE`;
/// a worker type (`0x32`..`0x35`) runs `think_peasant(1)`.
/// **`Unit::do_attack_to_pause@005f22a0`'s steps** (item 2314): a unit
/// in a group (`+0x80 >= 0`) that holds it active (`GroupData::member(o,
/// who, 1)`) pauses when `Group::is_attacking_near` at its position says
/// yes; `near` is not asked when `member` says no.
pub(crate) fn attack_to_pause_steps<C, G>(
    cx: &mut C,
    group: Option<G>,
    member: impl FnOnce(&mut C, &G) -> bool,
    near: impl FnOnce(&mut C, &G) -> bool,
) -> bool {
    let Some(g) = group else { return false };
    member(cx, &g) && near(cx, &g)
}

pub(crate) fn think_order_steps<C>(
    cx: &mut C,
    kill: impl FnOnce(&mut C),
    head_type: impl FnOnce(&mut C) -> Option<i32>,
    worker: impl FnOnce(&mut C) -> bool,
    think: impl FnOnce(&mut C),
) {
    kill(cx);
    if head_type(cx).is_some_and(|t| t != i32::from(index::NONE)) {
        return;
    }
    if worker(cx) {
        think(cx);
    }
}
