//! The order list: reading the current order, the three enqueue modes, the kills, every `add_*_order` and the strafe run (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    // ------------------------------------------------------------------
    // The list
    // ------------------------------------------------------------------

    /// The current order — `Unit::update_order`, `head->prev`.
    pub fn current_order(&self, u: usize) -> Option<&Order> {
        self.units[u].orders.front()
    }

    /// `UnitData::is_idle@0046fa40`: no current order — the original's head null or its first node's data
    /// null (item 2090's sweep holds both, and the iterator reset it writes on the way).
    pub fn is_idle(&self, u: usize) -> bool {
        self.current_order(u).is_none()
    }

    /// `UnitData::is_fleeing@0046efa0`: [`is_fleeing_type`] of the current order's `get_type()`.
    pub fn is_fleeing(&self, u: usize) -> bool {
        is_fleeing_type(self.current_order(u).map(|o| i32::from(o.index())))
    }

    /// `get_speed(x, y, 0)` — the virtual at slot `+0x17c` that both the
    /// unit step (`do_move@005f7b30`) and the pull-back
    /// (`find_path@005fb910`) call for the step length.
    ///
    /// For a player's unit this is `UnitData::get_speed@00608720`, whose
    /// three layers `docs/MOVEMENT.md`, "The speed pipeline", describes and
    /// which this crate takes as the cached `Movement::speed`.
    ///
    /// **An animal replaces it whole.** `AnimalData::get_speed@005d8380`
    /// occupies the same slot on `Animal`'s and `AnimalData`'s vtables, and
    /// it does not call `UnitData::get_speed` at all — it calls
    /// `UnitData::speed` (the aura layer) and then, on a land or sea
    /// animal:
    ///
    /// - if the current order `is_move` (slot `+0x14`) and the animal is
    ///   more than `0x180` from that order's **goal** — `get_move_order`
    ///   (slot `+0xb8`), then `MoveOrder +0x4/+0x8`, the ordered point and
    ///   not the current waypoint — the speed becomes `speed * 3 / 2`,
    ///   truncated toward zero;
    /// - and the result is floored at 3.
    ///
    /// An **air** animal — a bird, `type +0x218 == 2` — returns before
    /// both: no hurry and no floor.
    ///
    /// So none of `UnitData::get_speed`'s own layers reach an animal: not
    /// the order scale, not the `unit_masks & 0x10` halving, not the
    /// `0x800` tile, not the group cap. The two things `do_move` applies
    /// *after* the virtual — `ai_speed` and the modern-infantry `5/4` —
    /// still do.
    ///
    /// The `0x180` is the same threshold `Animal::do_idle` measures its
    /// wander with ([`crate::anim::WANDER_NEAR`] is `0x181`, the strict
    /// `<`), but it is a **different distance**: `do_idle` measures from
    /// the herd's centre, this from the order's goal. `docs/MOVEMENT.md`,
    /// "The animal's own `get_speed`".
    ///
    /// SEAM: the original decides this by class — a unit is an `Animal` or
    /// it is not — and this crate has no class, so it asks
    /// [`crate::Unit::is_gaia`], the same stand-in
    /// [`Sim::do_idle`](Self::do_idle) uses to reach `Animal::do_idle`.
    ///
    /// `flag` is the original's third argument and it selects **one**
    /// thing: the group cap at the end. `do_move`, `find_path`,
    /// `Object::poor_target` and `do_group_move`'s cross-formation arm
    /// pass 0 and are capped; `GuyData::get_speed`, the animal's air
    /// step and `do_group_move`'s in-formation arm pass 1 and are not.
    pub fn get_speed(&self, u: usize, flag: i32) -> i32 {
        self.get_speed_at(u, self.units[u].pos, flag)
    }

    /// [`Sim::get_speed`] asked **at a point**: the virtual's own
    /// `(x, y)` arguments, which only the slow-ground tile reads.
    /// `UnitData::get_speed@006086f0` hands it the unit's own point;
    /// `GuyData::get_speed@005de410` hands it **the figure's** — `+0xc` /
    /// `+0x10` pushed at `5de425`/`5de428`, then `calll *0x17c` — and
    /// passes 1, so a tracked crew figure takes the order scale and the
    /// slow-ground halving where it stands, and never the group cap.
    /// Its `+9` is behind `guy_num < squad_size` (`5de473`, `jge`), and
    /// [`crate::anim::SQUAD_SIZE`] is the literal 1, so no figure that
    /// steps on its own body reaches it (`docs/MOVEMENT.md`, "The
    /// crew's speed").
    pub fn get_speed_at(&self, u: usize, at: Pos, flag: i32) -> i32 {
        let unit = &self.units[u];
        // Both classes open on `UnitData::speed`, layer two.
        let speed = self.unit_speed(u);
        if unit.is_gaia() {
            // SEAM: the original's air test is the type's `+0x218 == 2`;
            // this crate's is the unit's domain.
            return animal_speed_steps(
                &mut (),
                |_| speed,
                |_| unit.kind.domain == crate::attrition::Domain::Air,
                |_| self.current_order(u),
                |_, o| o.is_move(),
                |_, o| o.move_dest().unwrap_or(unit.pos),
                |_| unit.pos,
            );
        }
        // The **action**'s own scale, `00608743`–`00608777`. `get_action`
        // is the first order that is neither a transit move nor a
        // `CHANGE_FORM`; [`Sim::action_of`] is that walk without the
        // `CHANGE_FORM` half, which no capture reaches.
        let action = self
            .action_of(u)
            .and_then(|i| unit.orders.get(i))
            .map_or(index::NONE, Order::index);
        let mut speed = match action {
            index::ATTACK => speed * 9 / 8,
            index::GUARD if self.ai_driven(unit.owner) => speed * 10 / 8,
            index::GUARD => speed * 9 / 8,
            _ => speed,
        };
        if unit.kind.domain == crate::attrition::Domain::Land {
            // `unit_masks & 0x10` halves it first (`0060878c`) —
            // `Unit::target_opportunity`'s "struck while moving in
            // contact" ([`crate::Unit::contact`], `docs/AI.md` §309),
            // cleared at the end of the unit's next `Unit::work`.
            if unit.contact {
                speed /= 2;
            }
            if self.on_river(unit.pos, at) {
                speed /= 2;
            }
            // `has_general(0, 0x162)` and the type's `is_siege` (vslot
            // `+0x10c`, `UnitTypeData::is_siege@00470460`, `unit_flags &
            // 0x20000`) double it, `006087b0`-`006087d8`: a siege unit
            // within a Monarch's radius (item 3390).
            if self.is_siege_unit(u) && self.near_monarch(u) {
                speed *= 2;
            }
        }
        // **The group cap**, `00608789`-`006087a7`: with flag 0, a unit
        // that is in a group and has **no action order** — `get_action`
        // answering null, which is `local_8 == 0` and the same value the
        // order scale above keyed on — walks no faster than its group's
        // own speed (`docs/GROUPS.md` §14). `action` is `index::NONE`
        // exactly when `get_action` is null, since no order has index 0.
        if flag == 0 && action == index::NONE {
            speed = movement::group_capped(speed, self.group_speed_of(u));
        }
        speed.max(movement::SPEED_FLOOR)
    }

    /// `UnitData::get_speed`'s slow-ground test, `0060879b`–`006087d8`: the
    /// **tile** the unit stands on carries `0x800`, and the unit's own
    /// `z_internal` is not above zero.
    ///
    /// Both halves are the position the unit is standing at when the step
    /// is computed. `z_internal` is written by
    /// `SubObject::set_new_location@00662680` — `TerrainOut::find_tcoord_z`
    /// of the tile it has just moved to — which is
    /// [`World::tile_z`](crate::world::World::tile_z) here, so the stored
    /// field and the tile read are the same tile and this needs no field of
    /// its own. A world with no height grid reads zero everywhere, which is
    /// the flat harness world and passes the test.
    ///
    /// The `^ 0x63637` all over the decompilation of this and of
    /// `SubObjectData::log_data` is not a predicate: **the three
    /// coordinates are stored XORed with `0x63637`** and every reader
    /// decodes them, the gamelog's own printer included. So
    /// `(z ^ 0x63637) > 0` skipping the halving is `z > 0` skipping it.
    ///
    /// `z_internal` is the **unit's** (`this +0xc`) and the tile is the
    /// virtual's `(x, y)` (`param_1`/`param_2`), so a crew figure asked
    /// at its own point reads the unit's height and its own tile.
    pub(crate) fn on_river(&self, unit: Pos, at: Pos) -> bool {
        self.world.tile_z(unit.tile()) <= 0 && self.world.tile_mask(at.tile()) & tile::RIVER != 0
    }

    /// `UnitData::order_type`: the current order's `OrderIndex`, `NONE` for
    /// an empty list.
    pub fn order_type(&self, u: usize) -> u8 {
        self.current_order(u).map_or(index::NONE, Order::index)
    }

    /// `UnitData::get_action`: the position in the list of the first order
    /// that is not a transit move — the intent under the pathing legs.
    pub fn action_of(&self, u: usize) -> Option<usize> {
        self.units[u].orders.iter().position(|o| !o.is_transit())
    }

    /// `Unit::update_action`: `orders_x/y` = the final destination of the
    /// leading run of transit moves, `dest_angle` = the last one's angle;
    /// returns the action's position (§3.3).
    ///
    /// The walk is [`update_action_steps`] (item 2341), which the sweep
    /// `sweep::unit_update_action` holds to the original.
    pub fn update_action(&mut self, u: usize) -> Option<usize> {
        let unit = &mut self.units[u];
        let n = unit.orders.len();
        update_action_steps(
            unit,
            n,
            |unit| {
                unit.orders_pos = unit.pos;
                unit.movement.des_angle = unit.movement.heading;
            },
            // The engine's iterator (`+0xcc`, `+0xd0`, `+0xd4`): the port
            // indexes the list and keeps none.
            |_, _| {},
            |unit, i| unit.orders[i].is_move(),
            |unit, i| unit.orders[i].has(flag::ACTION),
            // SEAM: `CHANGE_FORM` (`FormOrder`) is not an order this crate
            // has, so this is never true here (`index::CHANGE_FORM`).
            |unit, i| unit.orders[i].index() == index::CHANGE_FORM,
            |unit, i| {
                if let Body::Move(m) = unit.orders[i].body {
                    unit.orders_pos = m.dest;
                    unit.movement.des_angle = m.angle;
                }
            },
        )
    }

    /// The generic enqueue (§3.1): `QUEUE_NEW` clears first, then the order
    /// is appended (`Last`, `New`) or rotated to the front (`First`).
    pub(crate) fn enqueue_order(&mut self, u: usize, order: Order, pos: QueuePos) {
        self.enqueue(u, order, pos);
    }

    pub(crate) fn enqueue(&mut self, u: usize, order: Order, pos: QueuePos) {
        match pos {
            QueuePos::New => {
                self.units[u].path.clear();
                self.close_orders(u);
                self.clear_partial_path(u);
                self.update_action(u);
                self.units[u].orders.push_back(order);
                self.new_head(u);
            }
            QueuePos::Last => {
                if self.units[u].orders.is_empty() {
                    self.new_head(u);
                }
                self.units[u].orders.push_back(order);
            }
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
                self.new_head(u);
            }
        }
        self.update_action(u);
    }

    /// `Unit::clear_partial_path@005e3920` — the drop of a suspended
    /// pathfinder search, and nothing else.
    ///
    /// The original's body is the suspended pathfinder search and nothing
    /// else: `UnitData +0x104` and `+0x10c`, the two `Tree<PathNode *>`s
    /// it hands back to `PathFinder::kill_tree` and the recycler, and the
    /// `Tree<CollBlock *>` beside them. It does **not** touch
    /// `unit_masks` — grep the function for `0x68` and there is no hit —
    /// so an order teardown, a `close_orders`, an animal's give-up in
    /// `resolve_unit_collision` step 0 and every other caller leave the
    /// verified-line bit exactly as `do_move` last wrote it.
    ///
    /// This crate used to clear [`crate::Unit::line_ok`] here, and that
    /// one line was most of `line_ok`'s wrong lifecycle: a unit that
    /// finished a walk kept the bit **set** over there and lost it here,
    /// which is 335 of run65's 450 unit-frames (`docs/MOVEMENT.md`,
    /// "The verified line's lifecycle", 2026-09-05).
    ///
    /// ~~SEAM: the suspended search itself is not modelled, so there is
    /// nothing left to free.~~ It is, since item 301: the stash is
    /// [`crate::Unit::search`] and this is the function that drops it.
    pub(crate) fn clear_partial_path(&mut self, u: usize) {
        self.units[u].search = None;
    }

    /// `Unit::kill_current_order(0)` (§3.2): the per-kind teardown, then the
    /// pop, the path segment, `update_action`.
    pub fn kill_current_order(&mut self, u: usize) {
        self.kill_order(u, false);
    }

    /// A different order object now heads `u`'s list ([`Unit::head_serial`]).
    pub(crate) fn new_head(&mut self, u: usize) {
        let h = &mut self.units[u].head_serial;
        h.0 = h.0.wrapping_add(1);
    }

    /// `Unit::kill_current_order(closing)`: `closing` is the argument
    /// `Unit::close@0060ee50` hands `close_orders` (`flags & 1`, 1 on a
    /// live unit), and every other caller's 0. It gates the move family's
    /// facing hand-back alone of the arms this crate carries (`5e3026`,
    /// `param_1 == 0`); the gather arm's `remove_gatherer` is gated on
    /// nothing but the building (§3.2, `docs/GOLDEN.md` §52).
    pub(super) fn kill_order(&mut self, u: usize, closing: bool) {
        let Some(order) = self.units[u].orders.front().copied() else {
            return;
        };
        // Before the teardown: a dying **move** carries the mirror its
        // formation was laid out with, and the group's leader hands it back
        // (`docs/GROUPS.md` §6.3). This is the write that makes a group's
        // second right-click start from the first click's mirror instead of
        // from whatever the leader's turning left behind.
        if let Body::Move(m) = order.body
            && let Some(f) = m.facing
            && !closing
        {
            self.hand_back_facing(u, f, m.angle);
        }
        match order.body {
            Body::Gather(g) => {
                let who = self.units[u].owner;
                self.economy.ledgers[who as usize].dirty = true;
                if self.buildings.get(g.building).is_some_and(|b| b.alive)
                    && self.buildings[g.building].owner == who
                {
                    self.remove_gatherer(g.building, u);
                }
                // `kill_current_order:107`, inside the same arm and after
                // the chain: the carrying walk goes with the job.
                self.units[u].carry &= !CARRY_ANY;
            }
            Body::Attack(_) => {
                // The order is the target's home; dropping it drops the target.
                let unit = &mut self.units[u];
                unit.combat.target = None;
                unit.combat.mandatory = false;
            }
            // `kill_current_order`'s `CAST_SPELL` arm (`0xe`): a cast that
            // was **paid** and dies before its craft is cast hands the craft
            // back — `SpellType::unpay_cast_costs@00676ff0`, `mana_burn -=
            // min(mana_burn, MANA)` — unless the unit is closing. `do_cast`
            // clears `paid` before its own kill, so only an interrupted cast
            // reaches it: run346's `1/98`, whose Create Decoys the army's
            // tick replaced with an `ATTACK_TO` on 11508, `mana_burn` 903 →
            // 0 (item 1302). The refund's resource half (the craft's
            // `Type::unpay_cost`, vslot `+0xd8`) moves nothing on shipped
            // data: every craft whose `COST2` is not empty — Restock
            // Supplies, Research Project, Move Capital, Morph, Upgrade,
            // the diplomacy crafts — has `FROM` and `FROM2` `none`, so
            // `SpellTypeData::is_castable@00675bc0` refuses it to every
            // caster and no unit holds it as a cast (item 3985,
            // `docs/GOLDEN.md` §178).
            Body::Cast(c) if c.paid && !closing => self.unpay_cast_mana(u, c.spell),
            _ => {}
        }
        // `kill_current_order@005e2cb0:30`: `unit_masks &= ~0x20000`, a
        // targeted cast's "started" bit, for whatever order dies.
        self.units[u].casting = false;
        self.units[u].orders.pop_front();
        self.new_head(u);
        if order.is_move() && order.has(flag::PATHED) {
            self.kill_current_path(u);
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// Remove the order the cursor sits on, `i` places behind the current
    /// one. Only `check_build_order` walks a cursor, and only onto a
    /// `BUILD_AT`, which carries no per-kind teardown — so this is
    /// [`Sim::kill_current_order`]'s generic tail applied at an index.
    pub(super) fn remove_order_at(&mut self, u: usize, i: usize) {
        if self.units[u].orders.remove(i).is_none() {
            return;
        }
        if i == 0 {
            self.new_head(u);
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// `Unit::kill_current_path@005e31d0`: pop the stack until an entry
    /// with the final flag is popped — **and then free the suspended
    /// search**.
    ///
    /// The `clear_partial_path` call sits inside the function's own `0 <
    /// length` guard, so an already-empty stack frees nothing; that guard
    /// is the whole of the difference between this and
    /// [`Sim::kill_current_order`], which clears unconditionally.
    ///
    /// It is what ends a suspended search that no longer has an order to
    /// serve. `Unit::ungroup_move_order@005fd140` calls this on every
    /// member that is not the leader (`docs/GROUPS.md`, and
    /// `docs/PATHFINDER.md` §18.4), so the frame a formation degrades is
    /// the frame each follower's stash goes — and the follower re-plans on
    /// the next frame instead of standing in `do_move`'s suspended block
    /// for the rest of the capture.
    pub(crate) fn kill_current_path(&mut self, u: usize) {
        if self.units[u].path.is_empty() {
            return;
        }
        self.units[u].discard_current_path_segment();
        self.clear_partial_path(u);
    }

    /// `Unit::close_orders`: kill everything, oldest first.
    pub fn close_orders(&mut self, u: usize) {
        while !self.units[u].orders.is_empty() {
            self.kill_current_order(u);
        }
    }

    /// **`Unit::close@0060ee50`'s `close_orders(this, flags & 1)`**, the
    /// last call before `Object::close` (`60fa1c`): a unit that dies has
    /// its orders killed, each through `kill_current_order(1)`. So a dead
    /// gatherer leaves its building's chain on the frame it dies
    /// (`remove_gatherer`, `kill_current_order@005e2cb0:104`), and not when
    /// the next `check_gatherers` finds it: run466's `1/2001` has
    /// `gather_down` 2 on block 1488, the block after the who=1 Citizen
    /// `1/6` died at its head (`docs/GOLDEN.md` §52, item 1248).
    pub(crate) fn close_dead_orders(&mut self, u: usize) {
        while !self.units[u].orders.is_empty() {
            self.kill_order(u, true);
        }
    }

    /// `Unit::clear_orders`: the `QUEUE_NEW` clear without the add.
    pub fn clear_orders(&mut self, u: usize) {
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
    }

    // ------------------------------------------------------------------
    // The adders
    // ------------------------------------------------------------------

    /// `Unit::add_move_order` → `add_move_facing_order` (§4.3): the 48-unit
    /// snap, the angle at order time, the action bit from the caller.
    ///
    /// **The angle is taken to the caller's point, not to the snapped one**
    /// (§4.3). The listing at `00616ed0` computes the two `find_angle`
    /// arguments in registers from the *arguments* — `ecx = x − (this->
    /// field_0x10 ^ 0x63637)`, `edx = y − (field_0x14 ^ 0x63637)` — and only
    /// then indexes `div_3_table` for the pair it pushes. So a caller that
    /// hands over an unsnapped point gets the bearing to that point and a
    /// destination 48-snapped away from it, and the two differ by up to
    /// twenty-four units on each axis. A farmer's re-picked cell is
    /// `t · 0xc0 + 0x60` — the centre of a 192-unit tile, which is never a
    /// 48-unit cell centre — so every farm walk in the game ends facing
    /// somewhere the snapped bearing does not name (`docs/ORDERS.md` §4.3,
    /// `docs/SYNC.md` §3.12).
    pub fn add_move_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
    ) {
        let here = self.units[u].pos;
        let angle = find_angle(to.x - here.x, to.y - here.y);
        self.add_move_facing_order(u, to, kind, pos, action, angle, None, false);
    }

    /// `Unit::add_move_facing_order@005e55c0` (§4.3), and
    /// `add_group_move_order@005e4710` where they agree: the same snap and
    /// the same record, with the **caller's** angle and the formation's own
    /// `facing` instead of the bearing `add_move_order` derives.
    ///
    /// `add_move_order` is that call with `angle = find_angle(dest − here)`
    /// and `facing = −1`, which is exactly what the listing at `616ed0`
    /// pushes. The two arguments only ever come from a group move
    /// (`docs/GROUPS.md` §6.6 step 6), which is why the ordinary adder does
    /// not take them.
    ///
    /// `pathed` is the original's `param_5`, and it writes [`flag::PATHED`]
    /// straight into the order at birth (`005e55c0`: `*pbVar1 |= 1`). Only
    /// the group move passes it: `action_move_near` plans the whole chain
    /// itself a few lines later (`docs/GROUPS.md` §6.7), so the order it
    /// hands out must not send `do_move` off to plan again.
    #[allow(clippy::too_many_arguments)] // the original's twelve, minus the seven this does not model
    pub fn add_move_facing_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
        angle: Angle,
        facing: Option<bool>,
        pathed: bool,
    ) {
        self.add_move_facing_order_grouped(
            u, to, kind, pos, action, angle, facing, pathed, None, None,
        );
    }

    /// `Unit::add_group_move_order@005e4710` and `add_move_facing_order`
    /// in one — the two write the **same** record, and the group adder
    /// adds the `GroupOrder` base's five fields on top (`docs/GROUPS.md`
    /// §6.6 step 6). It writes the angle a second time, into
    /// `group_angle`, which is why the two are equal on every record run31
    /// holds; `pathed` is 1 from both.
    #[allow(clippy::too_many_arguments)]
    pub fn add_move_facing_order_grouped(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
        angle: Angle,
        facing: Option<bool>,
        pathed: bool,
        group: Option<GroupMove>,
        orig: Option<Pos>,
    ) {
        let mode = i32::from(match kind {
            MoveKind::MoveTo => index::MOVE_TO,
            MoveKind::AttackTo => index::ATTACK_TO,
            MoveKind::ExploreTo => index::EXPLORE_TO,
            MoveKind::FleeTo => index::FLEE_TO,
        });
        let plain = group.is_none();
        add_move_facing_steps(
            self,
            (to.x.div_euclid(SNAP), to.y.div_euclid(SNAP)),
            mode,
            pos,
            MoveBits {
                pathed,
                action,
                // SEAM: the disembark argument (`param_11`, the order's
                // `0x20`) — no caller in this crate passes it.
                disembark: false,
            },
            |sim| {
                // `add_group_move_order@005e4710` has no attack to keep.
                if !plain {
                    return None;
                }
                match sim.current_order(u).map(|o| o.body) {
                    Some(Body::Attack(_)) => sim.units[u].combat.target,
                    _ => None,
                }
            },
            |sim, &t| matches!(sim.profile(t).domain, crate::attrition::Domain::Air),
            |sim| sim.profile(Obj::Unit(u)).has(combat::mask::ANTI_AIR),
            // SEAM: the scout's shift-queued `MOVE_TO` turned `EXPLORE_TO`
            // with `unit_masks |= 0x4000000` (`role & 0x10`): the mask is
            // not carried (`work`'s and `think`'s readers), so neither is
            // the conversion. The group adder has no such arm.
            |_| false,
            |_| {},
            // `unit_masks &= ~0x4000000` (not carried), `+0xc0 = 0`.
            |sim| sim.units[u].path.clear(),
            |sim| sim.close_orders(u),
            |sim| sim.clear_partial_path(u),
            |sim| {
                sim.update_action(u);
            },
            |_, idx| new_move_order(idx, angle, facing, orig, group),
            fill_move_order,
            |sim, o| {
                // `LinkListBase::add` appends; an empty list's new node is
                // a new head (the generic enqueue's `Last`/`New`).
                if sim.units[u].orders.is_empty() && pos != QueuePos::First {
                    sim.new_head(u);
                }
                sim.units[u].orders.push_back(o);
            },
            |sim| {
                let o = sim.units[u].orders.pop_back().expect("just linked");
                sim.units[u].orders.push_front(o);
                sim.new_head(u);
            },
            |sim, t| sim.add_attack_order(u, t, QueuePos::First, false, false),
        );
    }

    /// `Unit::add_build_order`.
    pub fn add_build_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Build(b),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_repair_order`.
    pub fn add_repair_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Repair(b),
        };
        // The one adder that is **not** §3.1's common shape: `add_repair_
        // order@005e4ff0` has no `QUEUE_FIRST` tail at all — after the list
        // add at `0x5e51f0` come only `update_action` and `ret 0x10`, where
        // `add_build_order` has an explicit `cmp [ebp+0x10],0` →
        // `clear_partial_path; head = head->next`. No caller passes
        // `QUEUE_FIRST`, so this is fidelity, not a live bug.
        // (`docs/audit/2026-08-21-orders.md` R3 P1.)
        debug_assert!(
            pos != QueuePos::First,
            "add_repair_order has no QUEUE_FIRST branch in the original"
        );
        let pos = if pos == QueuePos::First {
            QueuePos::Last
        } else {
            pos
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_garrison_order(o, who, search, pos, action)`.
    pub fn add_garrison_order(
        &mut self,
        u: usize,
        b: usize,
        search: bool,
        pos: QueuePos,
        action: bool,
    ) {
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Garrison {
                building: b,
                search,
            },
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_gather_order@0061a5c0(o, pos, action)` (§6.2): the unit
    /// joins the building's chain **now**, before it has moved, if the type
    /// is a gatherer; the non-flat fields are set for wood and ore. The
    /// steps are [`add_gather_order_steps`], which the sweep also drives
    /// over the emulated original's windows (item 3268).
    pub fn add_gather_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        let mut v = GatherSim {
            sim: self,
            u,
            b,
            order: None,
        };
        add_gather_order_steps(&mut v, b, pos, action);
    }

    /// `Unit::add_attack_order(o, who, pos, mandatory, action)` (§7.1). The
    /// target and `mandatory` live in [`combat::State`]; the order carries
    /// the rest.
    pub fn add_attack_order(
        &mut self,
        u: usize,
        target: Obj,
        pos: QueuePos,
        mandatory: bool,
        action: bool,
    ) {
        if pos == QueuePos::New {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        let unit = &self.units[u];
        let defensive = unit.combat.stance == combat::Stance::Defensive && !action;
        let def = if defensive {
            Some(self.find_def_pos(u))
        } else {
            None
        };
        // **No `targeted` bump.** `Unit::add_attack_order` never writes
        // `ObjectData +0x3d`; only `Object::find_nearby_target` does, on
        // the winner it returns (`docs/COMBAT.md` §33). Bumping here
        // counted a squad's mirror copies as separate attackers.
        let unit = &mut self.units[u];
        unit.combat.target = Some(target);
        unit.combat.mandatory = mandatory;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Attack(AttackOrder {
                defensive,
                def,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        };
        match pos {
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
                self.new_head(u);
            }
            _ => self.units[u].orders.push_back(order),
        }
        self.update_action(u);
    }

    /// `Unit::add_strafe_order(ox, whom, home_o, home_who, mandatory,
    /// queue, action)@005e48c0` (`docs/ORDERS.md` §32): a `StrafeOrder`
    /// with the target, its point in `xx/yy`, `returning` set exactly when
    /// there is no target, the home base, `cruising_alt` 0x640, and the
    /// action bit as asked. The `QUEUE_NEW` head is [`Self::enqueue`]'s
    /// and the `QUEUE_FIRST` rotation its tail's, but no queue closes with
    /// `update_action` ([`add_strafe_steps`], item 2341).
    ///
    /// **The missile arm at its head** (`5e48c0`..`5e4919`, item 1050): a
    /// type with the missile flag (`type +0x1e4 & 0x8000000`) and a target
    /// `Object::valid_target` passes is given
    /// [`Sim::add_air_attack_ground_order`] at the target's point instead,
    /// with the home, the queue and the action bit; `mandatory` is not
    /// passed on. run371's V2 `0/10` on 2672: an `AIRATTACKGROUNDORDER` on
    /// `1/2006`'s point, home `0/2009`, flags 4.
    pub fn add_strafe_order(
        &mut self,
        u: usize,
        target: Option<Obj>,
        home: Option<usize>,
        mandatory: bool,
        pos: QueuePos,
        action: bool,
    ) {
        // The steps are [`add_strafe_steps`]'s (item 2341's sweep).
        // **`QUEUE_FIRST` takes no `update_action`** (`0x5e4a2b`), and
        // neither does any other queue: the adder's tail is the link and,
        // under `QUEUE_FIRST`, `clear_partial_path` and the list's rotate,
        // so `orders_x/orders_y` and `dest_angle` keep the frame's own —
        // run223's 777, where the patrol's search pushes the strike after
        // the step and the dump still names the point before it
        // (`docs/ORDERS.md` §34.5). The generic enqueue's closing
        // `update_action` is the other adders'.
        add_strafe_steps(
            self,
            target,
            target.is_some(),
            pos,
            action,
            |s| s.profile(Obj::Unit(u)).has(crate::combat::mask::MISSILE),
            |s, t| t.is_some_and(|t| s.valid_target(Obj::Unit(u), t)),
            |s, t| s.pos_of(t.expect("a live target")),
            |s, at| s.add_air_attack_ground_order(u, at, home, pos, action),
            // SEAM: `unit_masks &= ~0x4000000` (`+0x68`), the defended
            // return's bit, which this crate does not carry.
            |s| s.units[u].path.clear(),
            |s| s.close_orders(u),
            |s| s.clear_partial_path(u),
            |s| {
                s.update_action(u);
            },
            |_, at, returning| Order {
                flags: 0,
                body: Body::Strafe(StrafeOrder {
                    target,
                    mandatory,
                    home,
                    cruising_alt: CRUISING_ALT,
                    sharp_turn: 0,
                    returning,
                    at,
                }),
            },
            |_, o, on| {
                if on {
                    o.flags |= flag::ACTION;
                } else {
                    o.flags &= !flag::ACTION;
                }
            },
            |s, o| {
                if s.units[u].orders.is_empty() {
                    s.new_head(u);
                }
                s.units[u].orders.push_back(o);
            },
            |s| {
                let orders = &mut s.units[u].orders;
                if orders.len() > 1 {
                    orders.rotate_right(1);
                    s.new_head(u);
                }
            },
        );
    }

    /// **`Unit::add_air_attack_ground_order(x, y, home_o, home_who, queue,
    /// action)@005e41c0`** (item 1050): under `QUEUE_NEW`, `unit_masks &=
    /// ~0x4000000`, the path emptied (`+0xc0 = 0`), `close_orders`,
    /// `clear_partial_path` and `update_action`; then one
    /// [`AirAttackGroundOrder`] appended — the point, `accuracy` and
    /// `attack_unit` 0, the home, `cruising_alt` 0x640, `returning` 0 —
    /// with the action bit as asked; `QUEUE_FIRST` rotates it to the
    /// front; and `update_action` again — the generic enqueue's shape.
    pub(crate) fn add_air_attack_ground_order(
        &mut self,
        u: usize,
        at: Pos,
        home: Option<usize>,
        pos: QueuePos,
        action: bool,
    ) {
        // The steps are [`add_air_attack_ground_steps`]'s (item 2368's
        // sweep): the generic enqueue's shape, its `QUEUE_FIRST` rotate
        // taken after the link as the original's is.
        add_air_attack_ground_steps(
            self,
            pos,
            action,
            // SEAM: `unit_masks &= ~0x4000000` under `QUEUE_NEW`, the
            // defended return's bit, which this crate does not carry;
            // `+0xc0 = 0` is the path.
            |s| s.units[u].path.clear(),
            |s| s.close_orders(u),
            |s| s.clear_partial_path(u),
            |s| {
                s.update_action(u);
            },
            |_| Order {
                flags: 0,
                body: Body::AirAttackGround(AirAttackGroundOrder {
                    at,
                    home,
                    cruising_alt: CRUISING_ALT,
                    sharp_turn: 0,
                    returning: false,
                }),
            },
            |_, o, on| {
                if on {
                    o.flags |= flag::ACTION;
                } else {
                    o.flags &= !flag::ACTION;
                }
            },
            |s, o| {
                if s.units[u].orders.is_empty() {
                    s.new_head(u);
                }
                s.units[u].orders.push_back(o);
            },
            |s| {
                let orders = &mut s.units[u].orders;
                if orders.len() > 1 {
                    orders.rotate_right(1);
                    s.new_head(u);
                }
            },
        );
    }

    /// **`Unit::add_air_patrol_order(x, y, home_o, home_who, action,
    /// queue)@005e4350`** (`docs/ORDERS.md` §34.1): for a plane, the old
    /// orders go (`field_0xc0 = 0`, `close_orders`, `clear_partial_path`,
    /// `update_action`) and one `AirPatrolOrder` is appended over the
    /// point — `waypoint 0`, the home, `cruising_alt` 0x640, `sharp_turn`
    /// and `returning` 0, the action bit as asked. The queue argument is
    /// not read on this arm.
    ///
    /// **A helicopter (`unit_flags & 0x20`) is given a move instead**
    /// (`5e4363`..`5e43c5`, item 1019, `docs/GOLDEN.md` §44):
    /// `add_move_facing_order(x / 48, y / 48, find_angle(point − here), 2,
    /// 0, QUEUE_NEW, action)` — kind 2 is `ATTACK_TO` (ORDERS §1.2), an
    /// attack-move to the point's 48-unit cell with no home, so a
    /// helicopter a point sends is homed nowhere. run371's `0/12`, trained
    /// under `0/2008`'s point: `ATTACKTOORDER` type 2 to (5784, 12312),
    /// flags 4, on 2659.
    ///
    /// SEAM: a home that is a unit stores the point relative to it; not
    /// reached.
    pub(crate) fn add_air_patrol_order(
        &mut self,
        u: usize,
        point: Pos,
        home: Option<usize>,
        action: bool,
    ) {
        add_air_patrol_steps(
            self,
            point,
            action,
            |s| s.is_helicopter(u),
            |s| s.units[u].pos,
            |s, to, angle, action| {
                s.add_move_facing_order(
                    u,
                    to,
                    MoveKind::AttackTo,
                    QueuePos::New,
                    action,
                    angle,
                    None,
                    false,
                );
            },
            // SEAM: `unit_masks &= ~0x4000000`, the defended return's bit,
            // which this crate does not carry; `+0xc0 = 0` is the path.
            |s| s.units[u].path.clear(),
            |s| s.close_orders(u),
            |s| s.clear_partial_path(u),
            |s| {
                s.update_action(u);
            },
            |_| Order {
                flags: 0,
                body: Body::AirPatrol(AirPatrolOrder::over(Pos::new(0, 0), home)),
            },
            // SEAM: a home here is a building, whose `is_active` slot
            // (`Window::get_button`, 0) never answers; a unit home (a
            // carrier) would store the point relative to it.
            |_| None,
            |_, o, at| {
                if let Body::AirPatrol(ap) = &mut o.body {
                    ap.points[0] = at;
                }
            },
            |o| &mut o.flags,
            |s, o| s.units[u].orders.push_back(o),
        );
    }

    /// `Unit::do_strafe@005eab00`, the air order a flight is
    /// (`docs/ORDERS.md` §33, §34).
    ///
    /// **A flight home** (`returning 1`, `+0x3c`) goes straight to
    /// `do_air_physics(order, −1, −1)` (`0x5eb0ae`) — [`crate::air`]'s
    /// plane arm — and, when that returns 1, a non-bomber with no live
    /// target sets `CHAR_WALK` a second time (`0x5eb0cd`), and the order
    /// count test at `0x5eb62b` returns for a single order.
    ///
    /// **A strike** (`returning 0`, §34.2) first asks `valid_target` of
    /// its target. One it may not take — dead, or unseen, as run223's
    /// Barracks is on 665 — ends the strike: with another order behind it
    /// the strafe is killed and `work` runs again; alone, it becomes an
    /// `AirPatrolOrder` over the strike's point (`xx/yy`), with no action
    /// bit (the `UnitOrder` slot `+0x2c` it passes is `xor eax, eax`), and
    /// `work` runs the patrol in the same frame. One it may take is flown
    /// at, and bombed once in range and within 15° of the heading
    /// ([`Sim::strafe_attack`]).
    ///
    /// **A strike on one's own or an ally's unit is an escort** (item 976,
    /// `docs/GOLDEN.md` §42): it flies at the unit, `xx/yy` following it,
    /// and looks round it every sixteenth frame.
    ///
    /// SEAM: the escort's search — `find_new_bomber_target`'s `param_3 ≥
    /// 0` arm (the ally as the search's own object) and a non-bomber's
    /// `find_new_air_target` — is the bomber's point search and nothing;
    /// an escorted unit that is gone (its captain, else a patrol over
    /// `xx/yy`); a flying unit escorted, projected ahead past `0xc00`; a
    /// missile, a helicopter, and a strike with no point left (`is_valid(xx, yy)` false turns it for home).
    pub(super) fn do_strafe(&mut self, u: usize, frame: i64) {
        let Some(Body::Strafe(sf)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        let mut goal = None;
        if !sf.returning {
            let Some(target) = sf.target else {
                return;
            };
            let me = crate::combat::Obj::Unit(u);
            let whom = self.owner_of(target);
            if whom < crate::world::PLAYER_SLOTS && self.is_ally(self.units[u].owner, whom) {
                // **The escort** (`0x5eab9b`..`0x5ead3a`, item 976): a
                // strike on one's own or an ally's unit follows it. Its
                // point becomes `xx/yy` each frame, and on `(o + frame) &
                // 15 == 0` the plane looks round it — a Bomber's
                // `find_new_bomber_target`, any other's
                // `find_new_air_target` — and pushes what it finds
                // `QUEUE_FIRST`, `mandatory 0`, and runs `work` again.
                let at = self.pos_of(target);
                if let Some(Order {
                    body: Body::Strafe(s),
                    ..
                }) = self.units[u].orders.front_mut()
                {
                    s.at = Some(at);
                }
                if (i64::from(self.units[u].index) + frame) & 15 == 0 {
                    let found = if self.is_bomber(u) {
                        self.find_new_bomber_target(u, at)
                    } else {
                        None
                    };
                    if let Some(t) = found.filter(|&t| self.valid_target(me, t)) {
                        self.add_strafe_order(u, Some(t), sf.home, false, QueuePos::First, false);
                        self.work(u, frame);
                        return;
                    }
                }
                goal = Some(at);
            } else if !self.valid_target(me, target) {
                if self.units[u].orders.len() > 1 {
                    self.kill_current_order(u);
                    self.set_anim(u, crate::anim::WALK, true, true);
                    self.work(u, frame);
                    return;
                }
                let Some(at) = sf.at else {
                    return;
                };
                self.kill_current_order(u);
                self.add_air_patrol_order(u, at, sf.home, false);
                self.work(u, frame);
                return;
            } else {
                goal = Some(self.strafe_lead(u, target));
            }
        }
        if self.plane_air_physics(u, goal, frame) == crate::air::Flew::Done {
            return;
        }
        let target = sf.target;
        let me = crate::combat::Obj::Unit(u);
        let live = target.filter(|&t| self.valid_target(me, t));
        match live {
            Some(t) if self.units[u].combat.recharging == 0 => {
                if !self.strafe_attack(u, t, sf) {
                    return;
                }
            }
            _ => {
                if let Some(t) = target
                    && self.strafe_on_line(u, t)
                {
                    // `0x5eb62b` with the heading on the target.
                } else if !self.is_bomber(u) {
                    self.set_anim(u, crate::anim::WALK, false, true);
                }
            }
        }
        self.strafe_retarget(u, frame);
    }

    /// **A flying target's lead point** (`do_strafe@005eab00`,
    /// `0x5eafcf`–`0x5eb0a8`; item 3991, `docs/GOLDEN.md` §184): a plane
    /// striking a live plane (domain 2, no helicopter) farther than `0xc00`
    /// from it aims at the target's point `project`ed along the target's
    /// own heading (`+0x50`) by `(dist − 0xc00) / 3`, restricted to the
    /// world. run1549's Fighter `0/12` on 934: (7878, 11686) for who=1's
    /// Bomber standing at (7800, 11640) on its first 120°, 3,343 off.
    ///
    /// SEAM: a helicopter striker inside its reach, which aims at a point
    /// projected from its own.
    pub(crate) fn strafe_lead(&self, u: usize, target: crate::combat::Obj) -> crate::world::Pos {
        let at = self.pos_of(target);
        let crate::combat::Obj::Unit(t) = target else {
            return at;
        };
        if self.is_helicopter(u)
            || !self.units[t].alive()
            || !matches!(self.profile(target).domain, crate::attrition::Domain::Air)
            || self.is_helicopter(t)
        {
            return at;
        }
        let me = self.units[u].pos;
        let dist = crate::world::vector_dist((at.x - me.x).abs(), (at.y - me.y).abs());
        if dist <= 0xc00 {
            return at;
        }
        let d = (dist - 0xc00) / 3;
        let heading = self.units[t].movement.heading;
        self.restrict_pos(crate::world::Pos::new(
            at.x + crate::movement::sin_component(heading, d),
            at.y - crate::movement::cos_component(heading, d),
        ))
    }

    /// `do_strafe`'s arm for a strike it may take and is not reloading
    /// (`0x5eb0f4`–`0x5eb62b`, `docs/ORDERS.md` §34.3). Returns false when
    /// the strike was killed and the frame is over.
    ///
    /// - **The leash.** A strike that is not `mandatory`, on a target that
    ///   does not fly, with an `AirPatrolOrder` behind it dies when the
    ///   target is more than `AIRCRAFT_RESPOND_RANGE × 0x100` from the
    ///   patrol's last point — the *target's* distance (`0x5eb22a`).
    /// - **The release.** In range (`ObjectData::is_in_range@00648d70`,
    ///   from the plane's own point) and within 15° of the heading — 60°
    ///   for the `0x127` line — a Bomber plays `CHAR_ATTACK2` (the draw
    ///   at `do_strafe+0x9d0`, chapter seventeen's 805) and reloads
    ///   `recharge() + 1`.
    ///
    /// SEAM: the other order behind a strike (`+0x100`'s arm), the
    /// `0x400000` type arm, a non-bomber's `fire_ammo`, the missile's
    /// death, and `BOMBING_MANA_COST` (0 in the shipped rules, and the
    /// tank is not carried).
    pub(super) fn strafe_attack(
        &mut self,
        u: usize,
        t: crate::combat::Obj,
        sf: StrafeOrder,
    ) -> bool {
        let air_target = matches!(self.profile(t).domain, crate::attrition::Domain::Air);
        if !air_target
            && self.units[u].orders.len() > 1
            && !sf.mandatory
            && let Some(Body::AirPatrol(p)) = self.units[u].orders.get(1).map(|o| o.body)
        {
            let at = self.pos_of(t);
            let last = p.last();
            let d = crate::world::vector_dist(at.x - last.x, at.y - last.y);
            if d > self.tables.tuning.aircraft_respond_range * 0x100 {
                self.kill_current_order(u);
                return false;
            }
        }
        let off = self.strafe_off_line(u, t);
        // **`+0x3c` is read again after the flight** (`do_strafe@005eab00`,
        // `5eb0f4`, item 1937): `check_fuel` sets it on an empty tank inside
        // `do_air_physics`, and a strike flying home does not release.
        let now_returning = matches!(
            self.current_order(u).map(|o| o.body),
            Some(Body::Strafe(StrafeOrder {
                returning: true,
                ..
            }))
        );
        if now_returning || !self.is_in_range(crate::combat::Obj::Unit(u), t) {
            return true;
        }
        if off >= 0x0aaa_aaaa && (!self.is_fighter_line(u) || off > 0x2aaa_aaaa) {
            return true;
        }
        self.mark(crate::air::SITE_STRAFE_BOMB);
        self.set_anim(u, crate::anim::ATTACK2, false, true);
        self.units[u].combat.recharging = self.reload_frames(u) + 1;
        true
    }

    /// `fold(heading − find_angle(target − plane))`, the angle
    /// `do_strafe` measures its target off the nose with (`0x5eb3ef`).
    pub(super) fn strafe_off_line(&self, u: usize, t: crate::combat::Obj) -> u32 {
        let (at, to) = (self.units[u].pos, self.pos_of(t));
        let bearing = crate::movement::find_angle(to.x - at.x, to.y - at.y);
        let d = (self.units[u].movement.heading.0 as u32).wrapping_sub(bearing.0 as u32);
        if d > 0x8000_0000 { !d } else { d }
    }

    /// The reloading arm's test (`0x5eb54c`): the target within 30° of
    /// the heading, 90° for the `0x127` line, skips the `CHAR_WALK`.
    pub(super) fn strafe_on_line(&self, u: usize, t: crate::combat::Obj) -> bool {
        let lim = if self.is_fighter_line(u) {
            0x4000_0000
        } else {
            0x1555_5555
        };
        self.strafe_off_line(u, t) <= lim
    }

    /// **`do_strafe`'s tail, `0x5eb62b`** (`docs/ORDERS.md` §34.4): a
    /// strike that is not alone and not `mandatory`, on every frame
    /// `(frame + 2·o) & 31 == 0`, asks the order behind it for a target —
    /// an `AirPatrolOrder`'s search at its last point — and is re-pointed
    /// at a valid one, or killed.
    ///
    /// **Its head kills three strikes** (`5eb642`..`5eb673`, item 1200):
    /// past the count and `mandatory`, a type with a tank (`type +0x2ec`,
    /// `MANA`) whose `mana_left` is 0, a strike with no target and one
    /// flying home each go to `5eb395` — `kill_current_order` and
    /// `CHAR_WALK` — and the order behind flies. Chapter forty-one's
    /// Biplane on 1108, its `mana_burn` at 300 of 300 (`docs/GOLDEN.md`
    /// §50): the patrol turns for home on 1109 alone.
    ///
    /// SEAM: an order behind that is not a patrol (`+0x100`'s arm, which
    /// kills and walks).
    pub(super) fn strafe_retarget(&mut self, u: usize, frame: i64) {
        if self.units[u].orders.len() < 2 {
            return;
        }
        let Some(Body::Strafe(sf)) = self.current_order(u).map(|o| o.body) else {
            return;
        };
        if sf.mandatory {
            return;
        }
        let dry = self.unit_mana(u) != 0 && self.mana_left(u) == 0;
        if dry || sf.target.is_none() || sf.returning {
            self.kill_current_order(u);
            self.set_anim(u, crate::anim::WALK, true, true);
            return;
        }
        if (frame + 2 * i64::from(self.units[u].index)) & 31 != 0 {
            return;
        }
        let Some(Body::AirPatrol(p)) = self.units[u].orders.get(1).map(|o| o.body) else {
            return;
        };
        // A non-bomber asks `find_new_air_target(x, y, −1, −1)`
        // (`0x5eb7a3`, item 1200) — the patrol's own search, at its last
        // point: chapter forty-one's Biplane re-points from `0/8` to
        // `0/7` on 852 (`docs/GOLDEN.md` §50).
        let found = if self.is_bomber(u) {
            self.find_new_bomber_target(u, p.last())
        } else {
            self.find_new_air_target(u, p.last())
        };
        let me = crate::combat::Obj::Unit(u);
        match found.filter(|&t| self.valid_target(me, t)) {
            Some(t) => {
                // `ox/whom/uid` and `returning 0`; `xx/yy` stand.
                if let Some(Order {
                    body: Body::Strafe(sf),
                    ..
                }) = self.units[u].orders.front_mut()
                {
                    sf.target = Some(t);
                    sf.returning = false;
                }
            }
            None => self.kill_current_order(u),
        }
    }

    /// `UnitData::find_def_pos`: an existing DEFENSIVE attack's post, else a
    /// head move carrying the post flag, else the unit's own quarter-tile
    /// centre.
    pub(super) fn find_def_pos(&self, u: usize) -> Pos {
        if let Some(p) = self.def_post(u) {
            return p;
        }
        let p = self.units[u].pos;
        Pos::new(
            p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
            p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        )
    }

    /// `Unit::add_cast_order(o, who, x, y, spell, QUEUE_FIRST, 0)@005e4a60`,
    /// in the one shape this crate issues it: `set_new_location`'s
    /// transport conversion, untargeted, `QUEUE_FIRST`, and **without** the
    /// action bit — `param_7` is 0, which clears [`flag::ACTION`] rather
    /// than setting it (`docs/TRANSPORT.md` §6).
    ///
    /// **The pack/unpack rewrite at the head of the original is here**, and
    /// `Unit::think_fish` is what reaches it: `add_cast_order@005e4a60`
    /// re-aims the two generic spells at the caster's own before it builds
    /// the order, so a Fisherman asking for `UNPACK` (`0x28c`) gets
    /// `0x292` — which is the `spell 658` run58's block 4949 prints
    /// (`docs/ORDERS.md` §6.8).
    ///
    /// The machine gun's arm (`is(0x7b, 0)` → `0x28e`) is modelled for
    /// the unpack, which `think_attack`'s packed arm reaches. ~~SEAM: the
    /// pack rewrite (`0x28b` → `0x28d`/`0x28f`/`0x291`) has no caller.~~
    /// `Unit::work`'s pack arm reaches it since item 1370
    /// ([`Self::pack_before_move`]).
    pub fn add_cast_order(&mut self, u: usize, spell: i32) {
        self.add_cast_order_at(u, spell, QueuePos::First);
    }

    /// The same with the original's `param_6`. `Unit::unpack_merchant`
    /// is the one caller that passes `QUEUE_NEW`
    /// ([`crate::merchant`] §3); every other one this crate reaches
    /// passes `QUEUE_FIRST`.
    pub fn add_cast_order_at(&mut self, u: usize, spell: i32, pos: QueuePos) {
        // The rewrite at `add_cast_order@005e4a60`'s head, in the
        // original's own order: the machine-gun lineage (`is(0x7b, 0)`)
        // first, then the three **exact** merchant ids, then the
        // `FISHERMEN` lineage. The machine gun's unpack is reached by
        // `think_attack`'s packed arm ([`Self::think_attack_packed`]),
        // and the pack (`0x28b`) by `Unit::work`'s pack arm
        // ([`Self::pack_before_move`]), which the same three tests re-aim.
        let id = if spell == spell::PACK {
            if self.unit_line_is(u, MACHINEGUN) {
                spell::PACK_MACHINEGUN
            } else if self.is_merchant(u) {
                spell::PACK_MERCHANT
            } else if self.unit_line_is(u, crate::fish::FISHERMEN) {
                spell::PACK_FISHERMEN
            } else {
                spell
            }
        } else if spell != crate::fish::UNPACK {
            spell
        } else if self.unit_line_is(u, MACHINEGUN) {
            spell::UNPACK_MACHINEGUN
        } else if self.is_merchant(u) {
            spell::UNPACK_MERCHANT
        } else if self.unit_line_is(u, crate::fish::FISHERMEN) {
            crate::fish::UNPACK_FISHERMEN
        } else {
            spell
        };
        // One field to a line: `rondata::writers`' line parser does not
        // see the shorthand `spell` inside a one-line literal, and this is
        // the field's one writer (item 590).
        let order = Order {
            flags: 0,
            body: Body::Cast(CastOrder {
                spell: id,
                paid: false,
                target: None,
                at: Pos::new(-1, -1),
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_guard_order(o, who, dx, dy, queue, ·)@005e3e40`
    /// (`docs/ORDERS.md` §7.5, §24): the target, the offset asked, the
    /// post at the target's own position, `idle = retry = 0`, and the
    /// action bit set unconditionally.
    ///
    /// The `QUEUE_NEW` head is the generic one: the original clears
    /// `unit_masks & 0x4000000` (the deferred `EXPLORE_TO` conversion,
    /// which this crate does not carry), zeroes `UnitData +0xc0` — the
    /// path stack's `length`, so the stack is emptied — and then
    /// `close_orders`, `clear_partial_path`, `update_action`, which is
    /// [`Self::enqueue`]'s `New` arm exactly. `QUEUE_FIRST` rotates the
    /// list head onto the new order after a `clear_partial_path`, which is
    /// its `First` arm.
    pub fn add_guard_order(&mut self, u: usize, target: usize, dx: i32, dy: i32, pos: QueuePos) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Guard(GuardOrder {
                target,
                dx,
                dy,
                guard: self.units[target].pos,
                idle: 0,
                retry: 0,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_follow_order(ox, whom, queue, ·)@005e3f60`
    /// (`docs/ORDERS.md` §28): the leader, and the action bit set
    /// unconditionally. The `QUEUE_NEW` head and the `QUEUE_FIRST`
    /// rotation are [`Self::enqueue`]'s, as for the guard.
    pub fn add_follow_order(&mut self, u: usize, target: usize, pos: QueuePos) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Follow(FollowOrder { target }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_patrol_order@005e4560(x1, y1, x2, y2, id, form_id,
    /// leader, who, queue)` (`docs/ORDERS.md` §27): a `GroupPatrolOrder`
    /// with the two points — each passed as `div_3_table[v >> 4]` and
    /// stored `× 0x30 + 0x18`, the 48-unit grid's centre — `waypoint 0`,
    /// and the action bit set unconditionally. The `QUEUE_NEW` head and
    /// the `QUEUE_FIRST` rotation are [`Self::enqueue`]'s, as for the
    /// guard.
    pub fn add_patrol_order(
        &mut self,
        u: usize,
        (from, to): (Pos, Pos),
        id: i64,
        form_id: usize,
        leader: usize,
        pos: QueuePos,
    ) {
        let snap = |v: i32| (v >> 4) / 3 * 0x30 + 0x18;
        let order = Order {
            flags: flag::ACTION,
            body: Body::Patrol(PatrolOrder {
                points: [
                    Pos::new(snap(from.x), snap(from.y)),
                    Pos::new(snap(to.x), snap(to.y)),
                ],
                waypoint: 0,
                leader,
                id,
                form_id,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::update_patrol_order(id)@005e35e0` — the first
    /// `GROUP_PATROL` in the list, front to back, with this `id` (`None`
    /// for any): its position, for the caller to rewrite in place.
    pub fn update_patrol_order(&self, u: usize, id: Option<i64>) -> Option<usize> {
        self.units[u]
            .orders
            .iter()
            .position(|o| matches!(o.body, Body::Patrol(p) if id.is_none_or(|i| p.id == i)))
    }

    /// `Unit::update_guard_order(o, who)@005e3220` — the first `GUARD` in
    /// the list, front to back, whose target is `target`: its position in
    /// the list, for the caller to rewrite in place.
    pub fn update_guard_order(&self, u: usize, target: usize) -> Option<usize> {
        self.units[u]
            .orders
            .iter()
            .position(|o| matches!(o.body, Body::Guard(g) if g.target == target))
    }

    /// `Unit::add_think_order`: the argument is ignored — it is always
    /// rotated to the front, with the action bit.
    /// The steps are [`add_think_steps`]'s (item 2314's sweep).
    pub fn add_think_order(&mut self, u: usize) {
        add_think_steps(
            self,
            |_| Order {
                flags: 0,
                body: Body::Think,
            },
            |o| &mut o.flags,
            |sim, o| sim.units[u].orders.push_front(o),
            |sim| sim.clear_partial_path(u),
            |sim| sim.new_head(u),
            |sim| {
                sim.update_action(u);
            },
        );
    }
}

/// **`GuyData::get_speed@005de410`'s steps** (item 2341): the unit's
/// `get_speed(x, y, 1)` (vslot `+0x17c`, asked at the figure's point —
/// `speed`); then, for a figure inside the squad (`guy_num`, `+0xa2`,
/// signed, `<` the type's `squad_size` `+0x304`, `5de473` `jge`) whose
/// unit has an order list (`+0xdc`, `head`), the list's iterator is
/// rewound to the head's next node (`rewind`, which answers whether the
/// current order `+0xcc` is non-null), and a current order that
/// `is_group` (`UnitData::get_order`, then the order's vslot `+0x2c`)
/// adds 9. The sum wraps, as the original's `add` does.
pub(crate) fn guy_speed_steps<C>(
    cx: &mut C,
    guy_num: i32,
    speed: impl FnOnce(&mut C) -> i32,
    squad_size: impl FnOnce(&mut C) -> i32,
    head: impl FnOnce(&mut C) -> bool,
    rewind: impl FnOnce(&mut C) -> bool,
    is_group: impl FnOnce(&mut C) -> bool,
) -> i32 {
    let v = speed(cx);
    if guy_num < squad_size(cx) && head(cx) && rewind(cx) && is_group(cx) {
        return v.wrapping_add(9);
    }
    v
}

/// **`Unit::add_air_patrol_order@005e4350`'s steps** (item 2341, swept by
/// `sweep::unit_add_air_patrol_order`). A helicopter (`unit_flags & 0x20`)
/// is handed `add_move_facing_order` to the point with `find_angle(point −
/// here)`, the subtraction wrapping as the listing's `sub` does, and
/// nothing else. A plane: the masks and the path (`+0xc0 = 0`),
/// `close_orders(0)`, `clear_partial_path`, `update_action`; the order
/// made (`get_obj(AIR_PATROL)` and its slot `+0x8c`); then a home that
/// answers `is_active` (`objects[who][o]`, vslot `+0x8`, asked only when
/// both indices are non-negative) has its position subtracted from the
/// point; the point laid; the action bit set or cleared as asked; and the
/// order linked (`LinkListBase::add` on `+0xcc`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_air_patrol_steps<C, O>(
    cx: &mut C,
    point: Pos,
    action: bool,
    helicopter: impl FnOnce(&mut C) -> bool,
    here: impl FnOnce(&mut C) -> Pos,
    move_facing: impl FnOnce(&mut C, Pos, Angle, bool),
    clear: impl FnOnce(&mut C),
    close: impl FnOnce(&mut C),
    clear_path: impl FnOnce(&mut C),
    update: impl FnOnce(&mut C),
    make: impl FnOnce(&mut C) -> O,
    home: impl FnOnce(&mut C) -> Option<Pos>,
    lay: impl FnOnce(&mut C, &mut O, Pos),
    flags: impl FnOnce(&mut O) -> &mut u8,
    link: impl FnOnce(&mut C, O),
) {
    if helicopter(cx) {
        let h = here(cx);
        let angle = find_angle(point.x.wrapping_sub(h.x), point.y.wrapping_sub(h.y));
        move_facing(cx, point, angle, action);
        return;
    }
    clear(cx);
    close(cx);
    clear_path(cx);
    update(cx);
    let mut o = make(cx);
    let at = match home(cx) {
        Some(h) => Pos::new(point.x.wrapping_sub(h.x), point.y.wrapping_sub(h.y)),
        None => point,
    };
    lay(cx, &mut o, at);
    let f = flags(&mut o);
    if action {
        *f |= flag::ACTION;
    } else {
        *f &= !flag::ACTION;
    }
    link(cx, o);
}

/// What `Unit::add_gather_order@0061a5c0` asks of the unit, the order and
/// the building, in the order it asks (item 3268). [`GatherSim`] answers
/// from this crate's state; the sweep answers from the emulated original's
/// windows.
pub trait GatherOrderView {
    /// `QUEUE_NEW`'s opening: the path dropped (`+0x68 &= ~0x4000000`,
    /// `+0xc0 = 0`), `close_orders(0)`, `clear_partial_path`,
    /// `update_action`.
    fn restart(&mut self);
    /// `+0x68 &= ~0x400`, then `OrdersMemManager::get_obj(GATHER)` and its
    /// slot `+0x6c` (a fresh order, its building, player, type short and
    /// the node's `+4` of the building's data laid). SEAM: this crate's order
    /// carries none of those bytes; the sweep's view lays them.
    fn take_order(&mut self);
    /// The building's data record, vslot `+0x90`: `build_flags & 0x40`.
    fn is_gather_type(&mut self) -> bool;
    /// `Build::add_gatherer(unit id, player)` on the building.
    fn add_gatherer(&mut self);
    /// The type's vslot `+0x94`: `build_flags & FLAT`.
    fn is_flat(&mut self) -> bool;
    /// `building_ident == University` (`Build::vftable`'s slot `+0x60`, else
    /// the object's `+0xb8(0x1a4, 0)`).
    fn is_university(&mut self) -> bool;
    /// `building_ident == Mine` (`0x1a3`).
    fn is_mine(&mut self) -> bool;
    /// The order finished: `+0x25` set from `order.dist_mod != 0`, `+0x26`
    /// `dist_mod`, the action bit by `action`.
    fn lay(&mut self, order: GatherOrder, action: bool);
    /// `LinkListBase::add` of the finished order (after the transport arm).
    fn link(&mut self);
    /// The tail: both ends' `div_3_table` tiles through
    /// `WorldData::get_tregion`; when the regions differ and the leader's
    /// transport level reaches the type's `transport_type` and the type can
    /// ever transport, `+0x68 |= 0x800000`. SEAM: this crate has no such
    /// flag, so [`GatherSim`] does nothing.
    fn transport_arm(&mut self);
    /// `Unit::update_action`.
    fn update_action(&mut self);
}

/// **`Unit::add_gather_order@0061a5c0`'s steps** (item 3268): the head of
/// `QUEUE_NEW`, the order's fields, the gather-type arm (`add_gatherer`,
/// and for a non-flat non-University building `dist_mod` 10 for a Mine, 4
/// otherwise), the tail and `update_action`.
///
/// **There is no `QUEUE_FIRST` arm.** The original appends the order with
/// `LinkListBase::add` for every position but `QUEUE_NEW`'s opening and
/// does not rotate the list as `add_garrison_order` does; this crate used to
/// push it to the front and bump the head serial. No caller passes `First`
/// (every call is `New` or `Last`), so no run could see it.
pub fn add_gather_order_steps<V: GatherOrderView>(
    v: &mut V,
    b: usize,
    pos: QueuePos,
    action: bool,
) {
    if pos == QueuePos::New {
        v.restart();
    }
    v.take_order();
    let mut dist_mod = 0;
    if v.is_gather_type() {
        v.add_gatherer();
        if !v.is_flat() && !v.is_university() {
            dist_mod = if v.is_mine() { 10 } else { 4 };
        }
    }
    v.lay(
        GatherOrder {
            building: b,
            tile: None,
            wait: 0,
            goto_build: true,
            dist_mod,
            been_there: false,
        },
        action,
    );
    v.transport_arm();
    v.link();
    v.update_action();
}

/// [`GatherOrderView`] over this crate's state.
pub(super) struct GatherSim<'a> {
    pub(super) sim: &'a mut Sim,
    pub(super) u: usize,
    pub(super) b: usize,
    pub(super) order: Option<Order>,
}

impl GatherOrderView for GatherSim<'_> {
    fn restart(&mut self) {
        let u = self.u;
        self.sim.units[u].path.clear();
        self.sim.close_orders(u);
        self.sim.clear_partial_path(u);
        self.sim.update_action(u);
    }
    /// **A gather order takes the builder's mark off** (`61a5f7`: `andl
    /// $0xfffffbff, 0x68(%ebx)`, every position, ahead of the order). This
    /// crate carries the bit as `was_builder`, and only `think_peasant`'s
    /// tail cleared it, so a citizen sent from a site to a gather kept it
    /// and its next idle think searched for a build. run1387's `1/95`
    /// carried it from 2861 to 4841 in ours and lost it before 4101 in
    /// theirs (`unit_masks` 0x840408 → 0x840008 on run1370's stride), and
    /// on 4841 ours sent it to the unbuilt `1/2052` (`docs/AI.md` §358.4,
    /// item 3710).
    fn take_order(&mut self) {
        self.sim.units[self.u].was_builder = false;
    }
    fn is_gather_type(&mut self) -> bool {
        self.sim.is_gather_type(self.b)
    }
    fn add_gatherer(&mut self) {
        self.sim.add_gatherer(self.b, self.u);
    }
    fn is_flat(&mut self) -> bool {
        self.sim.is_flat(self.b)
    }
    fn is_university(&mut self) -> bool {
        self.sim.building_ident(self.b) == Ident::University
    }
    fn is_mine(&mut self) -> bool {
        self.sim.building_ident(self.b) == Ident::Mine
    }
    fn lay(&mut self, order: GatherOrder, action: bool) {
        self.order = Some(Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Gather(order),
        });
    }
    fn link(&mut self) {
        let order = self.order.take().expect("laid before it is linked");
        self.sim.units[self.u].orders.push_back(order);
    }
    fn transport_arm(&mut self) {}
    fn update_action(&mut self) {
        self.sim.update_action(self.u);
    }
}

/// **`Unit::update_action@0060a870`'s steps** (item 2341), over a list of
/// `n` orders, `0` the current one (the engine's `head_node->next`, `+0xdc`)
/// and `n - 1` the last (the node equal to `head_node`):
///
/// - `reset`: `orders_x/y` (`+0x70`/`+0x74`) the unit's own position and
///   `dest_angle` (`+0x58`) its `angle` (`+0x50`); an empty list ends
///   there, answering `None`.
/// - `at(i)`: the iterator stepped onto order `i` (`+0xd4` the node,
///   `+0xcc` its order, `+0xd0` its byte `+0xc`).
/// - A lone order that is not a move (vslot `+0x14`) resets again.
/// - Otherwise every **transit** order — a move without the action bit
///   (`+4 & 4`), or a `CHANGE_FORM` (vslot `+0x10` `== 0x12`) — before the
///   last is walked past, a move among them copying its record's `x, y,
///   angle` (vslot `+0x40`, `+4`/`+8`/`+0xc`) as it goes; the order it stops
///   on copies too when it is a move and the last or the action.
/// - It answers the order it stopped on, or `None` when that one is
///   transit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_action_steps<C>(
    cx: &mut C,
    n: usize,
    reset: impl Fn(&mut C),
    at: impl Fn(&mut C, usize),
    is_move: impl Fn(&mut C, usize) -> bool,
    action: impl Fn(&mut C, usize) -> bool,
    change_form: impl Fn(&mut C, usize) -> bool,
    copy: impl Fn(&mut C, usize),
) -> Option<usize> {
    reset(cx);
    if n == 0 {
        return None;
    }
    let last = n - 1;
    let transit = |cx: &mut C, i: usize| (is_move(cx, i) && !action(cx, i)) || change_form(cx, i);
    let mut i = 0;
    at(cx, i);
    if i == last && !is_move(cx, i) {
        reset(cx);
    } else {
        while transit(cx, i) && i != last {
            if is_move(cx, i) {
                copy(cx, i);
            }
            i += 1;
            at(cx, i);
        }
        if is_move(cx, i) && (i == last || action(cx, i)) {
            copy(cx, i);
        }
    }
    (!transit(cx, i)).then_some(i)
}

/// **`Unit::add_move_facing_order@005e55c0`'s steps** (item 2341).
///
/// Under `QUEUE_NEW` an attack-move (`mode == 2`) first asks whether the
/// current order is an `ATTACK` (`UnitData::order_type`) and, if it is,
/// reads its target (`get_order`'s vslot `+0xcc`, `+8`/`+0xc`); a target
/// whose type is of the air domain (`+0x218 == 2`) is kept only by a unit
/// that `has_objmask(0x80000000)` (`ANTI_AIR`), any other is kept. Then
/// the clear: `unit_masks &= ~0x4000000` and `+0xc0 = 0`, `close_orders`,
/// `clear_partial_path`, `update_action`. A `QUEUE_LAST` `MOVE_TO` of a
/// `role & 0x10` type sets `unit_masks |= 0x4000000` and becomes
/// `EXPLORE_TO`. The order is made by mode (2 `ATTACK_TO`, 3
/// `EXPLORE_TO`, 4 `FLEE_TO`, any other `MOVE_TO`), filled, linked;
/// `QUEUE_FIRST` runs `clear_partial_path` and rotates the head; a kept
/// target is re-attacked `QUEUE_FIRST`; and `update_action`.
///
/// `add_group_move_order@005e4710` is the same steps without the kept
/// attack and the scout's conversion, which its port closures answer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_move_facing_steps<C, O, T>(
    cx: &mut C,
    cell: (i32, i32),
    mode: i32,
    pos: QueuePos,
    bits: MoveBits,
    attack_target: impl FnOnce(&mut C) -> Option<T>,
    air: impl FnOnce(&mut C, &T) -> bool,
    anti_air: impl FnOnce(&mut C) -> bool,
    scout: impl FnOnce(&mut C) -> bool,
    set_explore: impl FnOnce(&mut C),
    clear_new: impl FnOnce(&mut C),
    close: impl FnOnce(&mut C),
    mut clear_path: impl FnMut(&mut C),
    mut update: impl FnMut(&mut C),
    make: impl FnOnce(&mut C, i32) -> O,
    fill: impl FnOnce(&mut O, &MoveFields),
    link: impl FnOnce(&mut C, O),
    rotate: impl FnOnce(&mut C),
    attack: impl FnOnce(&mut C, T),
) {
    let mut mode = mode;
    let mut keep = None;
    if pos == QueuePos::New {
        if mode == i32::from(index::ATTACK_TO)
            && let Some(t) = attack_target(cx)
            && (!air(cx, &t) || anti_air(cx))
        {
            keep = Some(t);
        }
        clear_new(cx);
        close(cx);
        clear_path(cx);
        update(cx);
    } else if pos == QueuePos::Last && mode == i32::from(index::MOVE_TO) && scout(cx) {
        set_explore(cx);
        mode = i32::from(index::EXPLORE_TO);
    }
    let idx = match mode {
        2..=4 => mode,
        _ => i32::from(index::MOVE_TO),
    };
    let mut o = make(cx, idx);
    let at = |v: i32| v.wrapping_mul(SNAP).wrapping_add(SNAP_CENTRE);
    let fields = MoveFields {
        at: Pos::new(at(cell.0), at(cell.1)),
        bits,
    };
    fill(&mut o, &fields);
    link(cx, o);
    if pos == QueuePos::First {
        clear_path(cx);
        rotate(cx);
    }
    if let Some(t) = keep {
        attack(cx, t);
    }
    update(cx);
}

/// **`Unit::add_think_order@005e3df0`'s steps** (item 2314): a `THINK`
/// order made (`OrdersMemManager::get_obj(THINK)`, then its vslot `+0xb0`),
/// its flags `| ACTION`; linked (`LinkListBase::add`, which makes the new
/// node the list's `head_node`); `clear_partial_path`; then `head_node =
/// head_node->next` (`+0xdc`), which leaves the new node at `head->prev`,
/// the end the engine reads as the current order — so the order goes to
/// the front whatever `QueuePos` says; and `update_action`.
/// **`AnimalData::get_speed@005d8380`'s steps** (item 2341): `speed`
/// (`UnitData::speed`) is the answer outright for an air animal (`air`,
/// the type's `+0x218 == 2`) — no hurry and no floor. Otherwise, when the
/// list's current order (`order`, the `+0xdc` head refreshed into
/// `+0xcc`) answers `is_move` (vslot `+0x14`), the unit's point (`at`,
/// the objects table's `+0x10`/`+0x14`) is measured against
/// `get_move_order`'s goal (vslot `+0xb8`, asked twice: its `+0x8` first,
/// then its `+0x4`) with `vector_dist(|dx|, |dy|)`; more than `0x180`
/// makes the speed `speed * 3 / 2`, truncated. The result is floored at
/// [`movement::SPEED_FLOOR`].
pub(crate) fn animal_speed_steps<C, O>(
    cx: &mut C,
    speed: impl FnOnce(&mut C) -> i32,
    air: impl FnOnce(&mut C) -> bool,
    order: impl FnOnce(&mut C) -> Option<O>,
    is_move: impl FnOnce(&mut C, &O) -> bool,
    mut goal: impl FnMut(&mut C, &O) -> Pos,
    at: impl FnOnce(&mut C) -> Pos,
) -> i32 {
    let mut speed = speed(cx);
    if air(cx) {
        return speed;
    }
    if let Some(o) = order(cx)
        && is_move(cx, &o)
    {
        let at = at(cx);
        let gy = goal(cx, &o).y;
        let gx = goal(cx, &o).x;
        let dx = at.x.wrapping_sub(gx).wrapping_abs();
        let dy = at.y.wrapping_sub(gy).wrapping_abs();
        if vector_dist(dx, dy) > 0x180 {
            speed = speed * 3 / 2;
        }
    }
    speed.max(movement::SPEED_FLOOR)
}

pub(crate) fn add_think_steps<C, O>(
    cx: &mut C,
    make: impl FnOnce(&mut C) -> O,
    flags: impl FnOnce(&mut O) -> &mut u8,
    link: impl FnOnce(&mut C, O),
    clear_path: impl FnOnce(&mut C),
    rotate: impl FnOnce(&mut C),
    update: impl FnOnce(&mut C),
) {
    let mut o = make(cx);
    *flags(&mut o) |= flag::ACTION;
    link(cx, o);
    clear_path(cx);
    rotate(cx);
    update(cx);
}

/// **`Unit::add_strafe_order@005e48c0`'s steps** (item 2341;
/// `docs/ORDERS.md` §32). A type with the missile bit (`type +0x1e4 &
/// 0x8000000`, asked first) whose target `Object::valid_target(ox, whom,
/// 0)` passes is handed to `add_air_attack_ground_order` at the target's
/// point (`objects[whom][ox] +0x10/+0x14`, unscrambled), and nothing else
/// happens. Otherwise, under `QUEUE_NEW`: the path dropped (`+0x68`'s bit,
/// `+0xc0 = 0`), `close_orders(0)`, `clear_partial_path`, `update_action`.
/// Then `get_obj(STRAFE)` is made — the target's point when `live` (`ox`
/// and `whom` both non-negative), `returning` exactly when not — its
/// action bit set or cleared as asked, and linked at the list's tail;
/// under `QUEUE_FIRST`, `clear_partial_path` and the rotate that heads
/// the list with it. **No queue takes a closing `update_action`.**
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_strafe_steps<C, T: Copy, P, O>(
    cx: &mut C,
    target: T,
    live: bool,
    pos: QueuePos,
    action: bool,
    missile: impl FnOnce(&mut C) -> bool,
    valid: impl FnOnce(&mut C, T) -> bool,
    mut point: impl FnMut(&mut C, T) -> P,
    attack_ground: impl FnOnce(&mut C, P),
    drop_path: impl FnOnce(&mut C),
    close: impl FnOnce(&mut C),
    mut clear_path: impl FnMut(&mut C),
    update: impl FnOnce(&mut C),
    make: impl FnOnce(&mut C, Option<P>, bool) -> O,
    set_action: impl FnOnce(&mut C, &mut O, bool),
    link: impl FnOnce(&mut C, O),
    rotate: impl FnOnce(&mut C),
) {
    if missile(cx) && valid(cx, target) {
        let at = point(cx, target);
        attack_ground(cx, at);
        return;
    }
    if pos == QueuePos::New {
        drop_path(cx);
        close(cx);
        clear_path(cx);
        update(cx);
    }
    let at = if live { Some(point(cx, target)) } else { None };
    let mut o = make(cx, at, !live);
    set_action(cx, &mut o, action);
    link(cx, o);
    if pos == QueuePos::First {
        clear_path(cx);
        rotate(cx);
    }
}

/// **`Unit::add_air_attack_ground_order@005e41c0`'s steps** (item 2368,
/// swept by `sweep::unit_add_air_attack_ground_order`). Under `QUEUE_NEW`
/// (`5e41c4`): the path dropped (`+0x68 &= ~0x4000000`, `+0xc0 = 0`),
/// `close_orders(0)`, `clear_partial_path`, `update_action`. Then
/// `get_obj(AIR_ATTACK_GROUND)` and its slot `+0x58` make the order, filled
/// (`+4/+8` the point, `+0x18/+0x1c` the home, `+0x20` 0x640, `+0xc`,
/// `+0x10`, `+0x2c` 0); its action bit set or cleared as asked; linked at
/// the list's tail (`LinkListBase::add` on `+0xcc`); under `QUEUE_FIRST`
/// (`5e4261`), `clear_partial_path` and the rotate that heads the list
/// with it (`+0xdc = *+0xdc`); and `update_action` for every queue.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_air_attack_ground_steps<C, O>(
    cx: &mut C,
    pos: QueuePos,
    action: bool,
    drop_path: impl FnOnce(&mut C),
    close: impl FnOnce(&mut C),
    mut clear_path: impl FnMut(&mut C),
    mut update: impl FnMut(&mut C),
    make: impl FnOnce(&mut C) -> O,
    set_action: impl FnOnce(&mut C, &mut O, bool),
    link: impl FnOnce(&mut C, O),
    rotate: impl FnOnce(&mut C),
) {
    if pos == QueuePos::New {
        drop_path(cx);
        close(cx);
        clear_path(cx);
        update(cx);
    }
    let mut o = make(cx);
    set_action(cx, &mut o, action);
    link(cx, o);
    if pos == QueuePos::First {
        clear_path(cx);
        rotate(cx);
    }
    update(cx);
}
