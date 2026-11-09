//! Acquisition: `target_opportunity`, the melee and nearby target searches (`Build::find_target`) and `compare_target` (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    /// A dead object is dropped from ammo in flight — what `close` does
    /// through `hold_frames` and `valid_target`.
    ///
    /// **Nor is a building's target** (item 1131): `Build::process`
    /// runs `do_attack` every frame `attack_ox`/`attack_whom` are set,
    /// and `do_attack`'s own `find_target` or `valid_target` replaces a
    /// dead one. run404's Radar `1/2007` still prints `attack_ox 6` on
    /// block 1007, after `0/6` was shot down on 1006, and fires at `0/7`
    /// on tick 1007; clearing it here left `do_attack` waiting for its
    /// 32-frame phase.
    ///
    /// **A unit's attack target is not dropped here, and that is the
    /// original's own behaviour** (item 502, `docs/COMBAT.md` §43.3).
    /// `Unit::do_attack@005f1b80` reads the target off the *order* — an
    /// `AttackOrder` **is** a `TargetOrder` — and nothing in the
    /// executable walks the object list clearing it when something dies.
    /// The order keeps `ox`/`whom`/`uid` where they are and
    /// `Object::valid_target` finds out at the next use, which is behind
    /// `Unit::fight`'s reload gate. run112's three bowmen carry the dead
    /// `1/8` on their `ATTACKORDER` from the block after it dies (684) to
    /// the block their reload opens (696), twelve frames — and that hold
    /// is what puts all three into `fight` on **one** frame, which is
    /// chapter two's word at 695.
    ///
    /// The same shape as item 463 one field along: `mandatory` outlives
    /// its target too. `Unit::do_attack@005f1b80` reads it off
    /// `TargetOrder +0x1c`, and run100's block 10233 still prints
    /// `ox 2004 whom 0 uid 4 mandatory 1` on `1/27` two blocks after the
    /// building `0/2004` left the dump. Clearing that sent a HOLD_FIRE
    /// raider into [`crate::Sim::do_attack`]'s stance arm, which returns,
    /// so the order it should have dropped stood for ever.
    pub(crate) fn forget(&mut self, dead: Obj) {
        for p in &mut self.projectiles {
            if p.target == Some(dead) {
                p.target = None;
            }
        }
    }

    /// `Unit::target_opportunity` (§12.4), reduced to the retaliation: a
    /// combat unit with no target, not holding fire, attacks whoever hit it;
    /// a non-combatant does nothing (it would flee).
    ///
    /// **The hit is answered by the victim's captain, never by the figure
    /// that took it.** `Unit::target_opportunity@005fffc0` opens with a
    /// `while (true)` whose body ends
    ///
    /// ```text
    ///     if (is_captain(this)) break;           // (ushort)o_up >> 15
    ///     param_3 = 0;
    ///     this = objects[who][this->o_up]->get_unit();
    /// ```
    ///
    /// — so every test below it, and the `add_attack_order` at its foot,
    /// runs on the head of the squad. `docs/COMBAT.md` §18.2 has the frame
    /// that measures it: the golden record's `1/6` strikes `0/7` on 616 and
    /// it is **`0/6`**, `0/7`'s captain, that carries the ATTACKORDER at the
    /// end of that frame, with `0/7` and `0/8` taking theirs a frame later.
    ///
    /// The loop's other arm, taken at each level of the captain walk — a
    /// group member whose type is *not* combat-role (`type +0x2c8 &
    /// 0x10000`) forwards to `Group::target_opportunity` — is built
    /// (item 1403, `docs/GOLDEN.md` §58): see [`Sim::target_opportunity_in`].
    pub(super) fn target_opportunity(&mut self, victim: usize, attacker: Obj, _frame: i64) {
        self.target_opportunity_in(victim, attacker, false);
    }

    /// [`Sim::target_opportunity`] with the original's `param_3`: `true`
    /// when `Group::target_opportunity` is the caller, which skips the
    /// group arm below (`005fffc0`'s `param_3 == 0` test).
    pub(crate) fn target_opportunity_in(&mut self, victim: usize, attacker: Obj, from_group: bool) {
        // **The head** (`5fffe1`..`60010b`): the diplomacy gate, the group arm
        // and the captain walk, `valid_target` and the hold-fire return, as
        // [`crate::topp::target_opportunity_head`] over [`crate::topp::SimTopp`]
        // (item 3348, swept). The gate and the group arm run at each level of
        // the walk, and `param_3` is zero from the second level on.
        let responder = match crate::topp::target_opportunity_head(
            &mut crate::topp::SimTopp { sim: self },
            victim,
            attacker,
            from_group,
        ) {
            crate::topp::Head::Go(u) => u,
            _ => return,
        };
        let me = Obj::Unit(responder);
        let st = self.units[responder].combat;
        // `UnitData::is_fleeing@0046efa0` — `order_type() == FLEE_TO`
        // (`docs/ORDERS.md` §1.3). A unit already running does not answer
        // the hit again, and the test sits **above** everything below it
        // (`6002cf`).
        let front = self.units[responder].orders.front().copied();
        self.mark_contact(responder, attacker, front);
        if self.is_fleeing(responder) {
            return;
        }
        // **`param_3 = update_action(this)`** (`Unit::target_opportunity@005fffc0`,
        // `LAB_006002cf`, the decompile's line 138), the first thing past the
        // `is_fleeing` return and ahead of every gate below: a hit on a unit
        // that has not fled rewrites its `orders_x/y` and `dest_angle` from
        // where it stands and which way it faces *now*. Chapter 103's `0/10`
        // is struck on 935 after its own step snapped it a cell on and
        // turned it; the dump's `orders_y` 16008 and `dest_angle`
        // 1181417472 are this call's, not the step's start (`docs/GOLDEN.md`
        // §140). The air orders (`STRAFE`, `AIR_PATROL`, `AIR_ATTACK_GROUND`)
        // return above it in the original.
        if !front.is_some_and(|o| {
            matches!(
                o.body,
                crate::orders::Body::Strafe(_)
                    | crate::orders::Body::AirPatrol(_)
                    | crate::orders::Body::AirAttackGround(_)
            )
        }) {
            self.update_action(responder);
        }
        // A held attack that yields is killed and the arm ends in `goto
        // LAB_00600877` (`6004e7`..), **past** the flee arm and the on-duty
        // gate below: a decoy with no post, whose attack on a building
        // `Group::target_opportunity` overrides, takes the new target
        // (run954's `1/77` on 12155, item 3306).
        let yielded = if st.target.is_some() {
            if !self.attack_action_yields(responder, attacker) {
                return;
            }
            true
        } else {
            false
        };
        if !yielded && self.counter_craft(responder, attacker, front) {
            return;
        }
        // **The flee arm** (`docs/COMBAT.md` §34), and it is reached only
        // with `local_c == 0` — the front order is not one of the seven
        // move kinds (`6002a0`'s `is_move(order_type())`).
        if !yielded
            && !front.is_some_and(|o| crate::orders::index::is_move_family(o.index()))
            && self.flee_from(responder, attacker)
        {
            return;
        }
        // **A busy unit that is not on duty does not answer the hit**
        // (item 1040, `docs/COMBAT.md` §70): every way past the flee arm
        // reaches `600863`, `call on_duty@005fff70; jne 600877`, then
        // `cmp %eax, -0x1c(%ebp)` — the front order's type, `local_20`,
        // stored at `600150` — `jne 600b16`, the return. So a unit with
        // any order retaliates only if it is on duty. The one way round it
        // is the action-is-an-attack arm (`600516`, `jmp 600877`), which
        // this crate carries as the `target` test above; an attack action
        // with no target keeps the old path (SEAM).
        if !yielded
            && front.is_some()
            && !self.action_is_attack(responder)
            && !self.on_duty(responder)
        {
            return;
        }
        // `LAB_00600877`'s own gate — `type->attack != 0`, the base column.
        // The `obj_masks & CIVILIAN && max_range == 0` test that used to
        // stand here was this crate's **stand-in for the flee arm above**
        // and has no counterpart in the original's tail; with the arm in,
        // it would swallow the retaliation of an armed civilian that is
        // neither a worker nor idle.
        if self.profile(me).attack == 0 {
            return;
        }
        // **An action order holds a human's unit** (`LAB_00600877`,
        // `docs/COMBAT.md` §63.4): the attack is added only when the
        // action (`update_action`) is null or not flagged `ACTION`, when
        // the unit carries `unit_masks & 0x40000` ([`Sim::ai_driven`]),
        // or when the action is an `ATTACK_TO`, a `0x15` or a patrol
        // (vslot `+0x34`). A player's guard under its `GUARD` never
        // answers the hit: chapter eleven's `0/6` on 1091.
        if !self.ai_driven(self.units[responder].owner)
            && let Some(k) = self.action_of(responder)
        {
            let o = self.units[responder].orders[k];
            let free = matches!(o.index(), crate::orders::index::ATTACK_TO | 0x15)
                || matches!(o.body, crate::orders::Body::Patrol(_));
            if o.has(crate::orders::flag::ACTION) && !free {
                return;
            }
        }
        self.retarget(me, Some(attacker), false);
    }

    /// `Group::target_opportunity@007107d0` for a hit its member `asker`
    /// took (`param_4 == 0`): a 15-frame cooldown on the group (`+0x38`,
    /// `think_frame`, `frame − last > 0xe`), then every member that is
    /// alive, on the map, a captain and combat-role takes **one of three
    /// arms** (`7108c8`..`710a69`, `docs/GOLDEN.md` §61):
    ///
    /// 1. the asker's own squad captain — `Unit::target_opportunity(member,
    ///    o, who, 1)`, no test on its orders (`71092e`..`710946`);
    /// 2. a member whose `get_action` is absent or whose action's vslot
    ///    `+0x10` (`get_type`) reads 0 (`NONE`), and whose `order_type` is
    ///    `NONE`, `ATTACK_TO` or `GROUP_ATTACK_TO` (`710950`..`710989`) —
    ///    its **own** `find_melee_target(member, min(dist(member, attacker)
    ///    + 0xc0, unit_respond_range × 0x240), NULL, 0, 1, 0)`, the order it
    ///    adds ([`Sim::find_melee_target_added_in`]);
    /// 3. any other — `Unit::target_opportunity(member, o, who, 1)` again.
    ///
    /// `from_think` is `param_4 == 1`, `Unit::think_attack`'s tail
    /// (`005f5da6`..): arm 1 never applies there.
    pub(crate) fn group_target_opportunity(
        &mut self,
        seat: crate::group::Seat,
        attacker: Obj,
        asker: usize,
        from_think: bool,
    ) {
        let frame = self.frame;
        {
            let (_, st) = self.seat_parts(seat);
            if frame - st.opportunity <= 0xe {
                return;
            }
            st.opportunity = frame;
        }
        let asker_captain = self.squad_captain(asker);
        let members = self.seat_list(seat).clone();
        for m in members {
            if self.units[m].alive()
                && self.units[m].on_map
                && self.units[m].captain
                && self.profile(Obj::Unit(m)).combat_role
            {
                if !from_think && m == asker_captain {
                    self.target_opportunity_in(m, attacker, true);
                    continue;
                }
                let no_action = self
                    .action_of(m)
                    .is_none_or(|k| self.units[m].orders[k].index() == crate::orders::index::NONE);
                if no_action
                    && matches!(
                        self.order_type(m),
                        crate::orders::index::NONE
                            | crate::orders::index::ATTACK_TO
                            | crate::orders::index::GROUP_ATTACK_TO
                    )
                {
                    let here = self.units[m].pos;
                    let there = self.pos_of(attacker);
                    let d = crate::world::vector_dist(
                        (here.x - there.x).abs(),
                        (here.y - there.y).abs(),
                    );
                    let range = (d + 0xc0).min(self.tables.tuning.unit_respond_range * 0x240);
                    self.find_melee_target_added_in(m, range);
                } else {
                    self.target_opportunity_in(m, attacker, true);
                }
            }
        }
    }

    /// **`Unit::target_opportunity@005fffc0`'s attack-action arm**
    /// (`600118`..`6004e7`, `docs/GOLDEN.md` §123): a unit whose action is an
    /// ATTACK on someone else drops it for the new attacker unless the
    /// order is mandatory, the held target is a combat-role unit, the new
    /// attacker is a building, the stance is Raid or Raze, or — when the
    /// held target is a unit and the attacker's type has an attack at all
    /// — the held target is in range or the attacker is not; a held
    /// building yields without the range tests (item 3306). Returns
    /// whether the old order is killed and
    /// the retaliation goes on to the tail.
    ///
    /// SEAMs: the siege/bombard/explosive shields (`has_objmask(0x840002)`)
    /// and a DEFENSIVE order's `find_def_pos` write.
    pub(super) fn attack_action_yields(&mut self, u: usize, attacker: Obj) -> bool {
        let Some(k) = self.action_of(u) else {
            return false;
        };
        let crate::orders::Body::Attack(_) = self.units[u].orders[k].body else {
            return false;
        };
        let Some(held) = self.units[u].combat.target else {
            return false;
        };
        if held == attacker || self.units[u].combat.mandatory {
            return false;
        }
        let me = Obj::Unit(u);
        if self.profile(me).has(
            crate::combat::mask::SIEGE
                | crate::combat::mask::BOMBARD
                | crate::combat::mask::EXPLOSIVE,
        ) {
            return false;
        }
        if matches!(held, Obj::Unit(_)) && self.profile(held).combat_role {
            return false;
        }
        if matches!(attacker, Obj::Building(_)) {
            return false;
        }
        if matches!(self.units[u].combat.stance, Stance::Raid | Stance::Raze) {
            return false;
        }
        // **A held target that is not a unit yields at once**
        // (`target_opportunity@005fffc0:202`–`:209`, the listing's
        // `LAB_006004e7`): when the held object's vslot `+0x18` answers 0
        // and the unit has none of `0x840002`, the two range tests below
        // are never asked. A guard striking the human's city in range drops
        // it for a Citizen its group names: run954's `1/25`, `1/48` and
        // `1/51` on 12155, handed `0/5` by decoy `1/88`'s
        // `Group::target_opportunity` (item 3306, `docs/AI.md` §262).
        if matches!(held, Obj::Unit(_))
            && self.profile(attacker).attack != 0
            && (self.is_in_range(me, held) || !self.is_in_range(me, attacker))
        {
            return false;
        }
        self.repath(u);
        self.kill_current_order(u);
        true
    }

    /// **`Unit::on_duty@005fff70`**: a combat-role type (`+0x2c8 &
    /// 0x10000`) whose activity is an `ATTACK_TO`, a `PATROL` (5), a
    /// `GUARD`, a `GROUP_ATTACK_TO` or a `GROUP_PATROL` (2, 5, `0xc`,
    /// `0x15`, `0x16`, in the function's own order).
    ///
    /// The activity is `UnitData::get_activity@00608370`'s: the first
    /// order that is neither a move nor an attack (vslot `+0x1c`,
    /// `is_move_attack`), and when every order is one, the last, answered
    /// only if it is an `ATTACK_TO` or a `0x15`.
    pub(crate) fn on_duty(&self, u: usize) -> bool {
        use crate::orders::{Body, index};
        if !self.profile(Obj::Unit(u)).combat_role {
            return false;
        }
        let orders = &self.units[u].orders;
        let move_attack = |o: &crate::orders::Order| {
            o.is_move() || matches!(o.body, Body::Attack(_) | Body::AttackGround(_))
        };
        let activity = match orders.iter().find(|o| !move_attack(o)) {
            Some(o) => Some(o.index()),
            None => orders
                .back()
                .map(crate::orders::Order::index)
                .filter(|&k| k == index::ATTACK_TO || k == index::GROUP_ATTACK_TO),
        };
        matches!(
            activity,
            Some(
                index::ATTACK_TO | 5 | index::GUARD | index::GROUP_ATTACK_TO | index::GROUP_PATROL
            )
        )
    }

    /// `update_action`'s order answers `is_attack` (vslot `+0x18`): the
    /// arm of `Unit::target_opportunity` that jumps past `on_duty`
    /// (`600516`). SEAM: `is_attack`'s overrides are folded in the export,
    /// so it is taken as the attack and ground-attack orders, as
    /// [`Sim::guard_activity`] takes it.
    /// **Struck while moving in contact** — `Unit::target_opportunity@005fffc0`,
    /// `600175`..`6002c8`, after the head and ahead of `is_fleeing`
    /// (`docs/AI.md` §309). The responder's front order is a move
    /// (`is_move@0046f050` of `order_type()`), the attacker is a unit
    /// (vslot `+0x18`), and `vector_dist` between them is inside the two
    /// types' `big_radius` (`+0x244`) summed: then `unit_masks |= 0x10`
    /// ([`crate::Unit::contact`]) — unless the responder's action is an
    /// `ATTACK` (`action_type@0060a850 == 10`) whose target (the
    /// `update_action` record's `+8`/`+0xc`) is inside twice that sum
    /// (`6002c6`, `jle`). The listing is the reading: the decompile's
    /// `is_move(unaff_EDI)` takes `order_type`'s answer in `ecx`
    /// (`600175`), and both distances are `vector_dist(|dx|, |dy|)` from
    /// the responder (`600211`, `6002b9`).
    pub(super) fn mark_contact(
        &mut self,
        u: usize,
        attacker: Obj,
        front: Option<crate::orders::Order>,
    ) {
        let Obj::Unit(a) = attacker else {
            return;
        };
        if !front.is_some_and(|o| crate::orders::index::is_move_family(o.index())) {
            return;
        }
        let at = self.units[u].pos;
        let reach = |p: crate::Pos| crate::world::vector_dist(at.x - p.x, at.y - p.y);
        let r = self
            .profile(Obj::Unit(u))
            .big_radius
            .wrapping_add(self.profile(attacker).big_radius);
        if reach(self.units[a].pos) > r {
            return;
        }
        if let Some(k) = self.action_of(u)
            && self.units[u].orders[k].index() == crate::orders::index::ATTACK
            && let Some(t) = self.units[u].combat.target
            && reach(self.pos_of(t)) <= r.wrapping_mul(2)
        {
            return;
        }
        self.units[u].contact = true;
    }

    pub(super) fn action_is_attack(&self, u: usize) -> bool {
        use crate::orders::Body;
        self.action_of(u).is_some_and(|k| {
            matches!(
                self.units[u].orders[k].body,
                Body::Attack(_) | Body::AttackGround(_)
            )
        })
    }

    /// `Unit::target_opportunity`'s **flee arm** — `6006f0`..`60085b`, the
    /// answer a non-combatant gives to being hit (`docs/COMBAT.md` §34).
    ///
    /// Returns whether it fired; the caller's retaliation is the `else`.
    ///
    /// The gate, in the listing's own order:
    ///
    /// ```text
    ///   ((has_objmask(CIVILIAN) && type->max_range == 0)
    ///        || type->attack == 0 || local_8)
    ///   && (is_worker(this) || is_idle(this))
    ///   && ((type->unit_flags & 4) == 0
    ///        || (!is_packing() && (unit_masks & 0x80000)))
    ///   && (!is_hero() || (unit_masks & 0xa000) == 0)
    /// ```
    ///
    /// then the radius — `0x600` for anything that is neither a hero nor
    /// a supply unit, and for those `0x180` when the attacker is a unit
    /// that answers vslot `+0x130` and `0x300` otherwise — and
    ///
    /// ```text
    ///   find_nearby_spot(type, x, y, &ox, &oy, radius, -1, 0,
    ///                    find_angle(x - ax, y - ay), FILTER_NOT_ME, o, who, …)
    ///   add_move_order(this, ox, oy, FLEE_TO, 0, QUEUE_FIRST, 0, …)
    /// ```
    ///
    /// **The bearing's arguments are the listing's, not the decompiler's**
    /// — Ghidra prints `find_angle(unaff_EDI, unaff_ESI)`. `6007e8`..
    /// `6007f2` builds `ecx = this->x − attacker->x` and
    /// `edx = this->y − attacker->y` and calls `0x92d130` fastcall, so the
    /// sweep's base bearing is **directly away from whoever hit it**.
    ///
    /// `find_nearby_spot` leaves its out-pair at the input point when it
    /// finds nothing (`docs/ORDERS.md` §10), so a failed sweep still
    /// issues a `FLEE_TO` — to where the unit already stands.
    ///
    /// **The hero and supply rings** (item 4005, `docs/COMBAT.md` §34.2,
    /// `6006e2`–`600785`): a hero (`+0xc4`, `unit_flags2 & 0x20`) or a
    /// supply unit (`+0xcc`, `& 0x40`) asks the attacker's vslot `+0x20`
    /// — the folded `Window::get_button` on a unit, answering 0, and
    /// `Buffer::is_pending_load` on a building — and then its `+0x130`,
    /// `max_range`: `0x180` from a unit that shoots, `0x300` from a
    /// building or a unit that does not. A hero in an Ambush or on a
    /// Forced March (`unit_masks & 0xa000`) does not flee at all.
    ///
    /// SEAMS, each a clause this crate cannot yet ask and no capture on
    /// disk reaches: the packing latch `local_8` (a packer mid-pack, set
    /// at `600130`); and the `unit_flags & 4` packer arm.
    /// **The counter-craft arms** — `Unit::target_opportunity@005fffc0`,
    /// `60051b`..`60061e`, between the action block and the flee arm
    /// (`docs/AI.md` §366.2, item 3739). Returns whether the hit was
    /// answered here.
    ///
    /// ```text
    ///   action a cast (order type 0xe):                       return
    ///       spell 0x280, and the attacker's who AND o both differ
    ///       from the cast's target's:   spell_time /= 2 (signed short)
    ///   the front order a move (-0x8(%ebp), 600190):          go on
    ///   no action:
    ///     is(SPY):  a land unit attacker, Bribe's can_pay_cost,
    ///               is_valid_target, mana 0 or <= mana_left     -> 0x275
    ///     is(COMMANDO): the same with Sabotage                  -> 0x280
    ///     add_cast_order(who, o, my x, my y, spell, QUEUE_NEW, 1); return
    /// ```
    ///
    /// The halving needs both to differ (`600547`, `600550`: either equal
    /// jumps to the return), not either. A craft whose test fails falls
    /// through to the next, then to the flee arm.
    ///
    /// SEAM: `can_pay_cost` (vslot `+0x84` of `types[spell]`) is taken as
    /// yes: both crafts' shipped `COST` is empty.
    pub(super) fn counter_craft(
        &mut self,
        u: usize,
        attacker: Obj,
        front: Option<crate::orders::Order>,
    ) -> bool {
        use crate::orders::Body;
        let action = self.action_of(u).map(|k| self.units[u].orders[k]);
        if let Some(Body::Cast(c)) = action.map(|o| o.body) {
            if c.spell == 0x280 {
                let differs = match c.target {
                    Some(t) => {
                        self.owner_of(t) != self.owner_of(attacker)
                            && self.obj_number(t) != self.obj_number(attacker)
                    }
                    None => true,
                };
                if differs {
                    self.units[u].spell_time /= 2;
                }
            }
            return true;
        }
        if action.is_some()
            || front.is_some_and(|o| crate::orders::index::is_move_family(o.index()))
        {
            return false;
        }
        let Obj::Unit(v) = attacker else {
            return false;
        };
        if self.unit_domain_of(v) != crate::attrition::Domain::Land {
            return false;
        }
        let who = self.units[u].owner;
        for (line, s) in [
            (crate::scout::SPY, crate::orders::spell::BRIBE),
            (COMMANDO, 0x280),
        ] {
            if !self.unit_line_is(u, line) {
                continue;
            }
            let Some(d) = self.spell(s) else {
                continue;
            };
            if !self.spell_valid_target(s, who, attacker)
                || (d.mana != 0 && d.mana > self.mana_left(u))
            {
                continue;
            }
            let at = self.units[u].pos;
            self.add_cast_order_on(u, s, Some(attacker), at, crate::orders::QueuePos::New, true);
            return true;
        }
        false
    }

    /// The object's own number, `o`: a unit's index, a building's.
    pub(super) fn obj_number(&self, o: Obj) -> i16 {
        match o {
            Obj::Unit(v) => self.units[v].index,
            Obj::Building(b) => self.buildings[b].index,
        }
    }

    pub(super) fn flee_from(&mut self, u: usize, attacker: Obj) -> bool {
        let me = Obj::Unit(u);
        let p = self.profile(me);
        let civilian = p.has(mask::CIVILIAN) && self.max_range_of(me) == 0;
        if !civilian && p.attack != 0 {
            return false;
        }
        if self.worker_of(u) == crate::orders::Worker::None && !self.is_idle(u) {
            return false;
        }
        if p.packs {
            return false;
        }
        let hero = self.is_hero_unit(u);
        if hero && (self.units[u].ambushing.is_some() || self.units[u].marching.is_some()) {
            return false;
        }
        let ring = if !hero && !self.is_supply_unit(u) {
            0x600
        } else {
            match attacker {
                Obj::Unit(_) if self.max_range_of(attacker) != 0 => 0x180,
                _ => 0x300,
            }
        };
        let here = self.units[u].pos;
        let from = self.pos_of(attacker);
        let angle = find_angle(here.x - from.x, here.y - from.y);
        let spot = self
            .find_nearby_spot(u, here, ring, -1, 0, angle, None)
            .unwrap_or(here);
        self.add_move_order(
            u,
            spot,
            crate::orders::MoveKind::FleeTo,
            crate::orders::QueuePos::First,
            false,
        );
        true
    }

    // ------------------------------------------------------------------
    // The search
    // ------------------------------------------------------------------

    /// `Unit::find_melee_target(range, …)` (§12.4): the idle radius for
    /// `range == −1`, then `find_nearby_target`.
    pub fn find_melee_target(&mut self, i: usize, range: i32) -> Option<Obj> {
        self.find_melee_target_with(i, range, 0)
    }

    /// [`Sim::find_melee_target`] with the caller's own `flags` word, its
    /// sixth argument. Every caller in this crate passes 0 but
    /// `Group::action_attack`'s retarget (`00712490:456`–`470`, item 1012,
    /// `docs/COMBAT.md` §66), which passes **1 when the group's target is
    /// a unit and 2 when it is a building** (the target's vslot `+0x1c`),
    /// so a member sent at a city takes a building, never a passing unit.
    /// [`Sim::melee_search_flags_with`] keeps the word or rewrites it.
    pub fn find_melee_target_with(&mut self, i: usize, range: i32, word: u32) -> Option<Obj> {
        self.find_melee_target_as(i, range, word, false)
    }

    /// [`Sim::find_melee_target_with`] with the original's third argument,
    /// `cavarch` (`Unit::find_melee_target@005ff9c0`'s `param_3`), which
    /// it hands on to [`Sim::find_nearby_target_as`]. Its head is the
    /// one a captain takes (a non-captain's `cavarch` jumps to
    /// `LAB_005ffbb2`, past the squad mirror), so the radius and the
    /// `flags` word are the same here.
    ///
    /// **A fire-on-the-move type's search that finds nothing resets every
    /// figure's pivots** (`Guy::reset_pivots(0)`, `:245`–`262`, for every
    /// caller): each turret's `des` goes to its rest angle
    /// ([`Sim::reset_pivots`]). The Dragoon's rest is 0, which its figures
    /// already held on run657's `1/80` (17165); item 1829's run783 is the
    /// first with a turret aimed away from it when a search fails (`0/6`
    /// on 793, `des` 0 on the next block).
    pub(crate) fn find_melee_target_as(
        &mut self,
        i: usize,
        range: i32,
        word: u32,
        cavarch: bool,
    ) -> Option<Obj> {
        let me = Obj::Unit(i);
        let st = self.units[i].combat;
        if st.stance == Stance::HoldFire {
            return None;
        }
        let r = self.max_range_of(me);
        let t = &self.tables.tuning;
        let radius = if range == -1 {
            if st.stance == Stance::Defensive {
                (if r == 0 { 0x120 } else { r * 0xc0 }).max(t.unit_defensive_respond_range * 0xc0)
            } else {
                // **The AGGRESSIVE bonus is inside the `r != 0` arm**, and
                // the respond floor has a second, larger rung.
                // `find_melee_target@005ff9c0`'s tail reads
                //
                //     if (r == 0) d = 0x120;
                //     else { d = (r + 1) * 0xc0; if (stance == 0) d += 0x180; }
                //     d = max(d, unit_respond_range * 0xc0);
                //     if (unit_masks & 0x40000) d = max(d, unit_respond_range * 0x180);
                //
                // — so a melee type (`r == 0`) gets a flat `0x120` whatever
                // its stance, where this crate added `0x180` to it, and an
                // AI-driven unit searches half again as far as a human's.
                let mut d = if r == 0 {
                    0x120
                } else {
                    (r + 1) * 0xc0
                        + if st.stance == Stance::Aggressive {
                            0x180
                        } else {
                            0
                        }
                };
                d = d.max(t.unit_respond_range * 0xc0);
                // SEAM: the original's word is the **unit's** `0x40000`;
                // [`Sim::ai_driven`] is this crate's one stand-in for it and
                // for the leader's `flags & 4` alike (`crate::orders`'
                // `think`). The golden record has it exactly — who=1's
                // hoplites carry `unit_masks 262144` and who=0's carry 0.
                if self.ai_driven(self.units[i].owner) {
                    d = d.max(t.unit_respond_range * 0x180);
                }
                d
            }
        } else {
            range
        };
        let flags = self.melee_search_flags_with(i, word);
        let found = self.find_nearby_target_as(me, radius, flags, cavarch);
        if found.is_none() && self.fires_on_move(i) {
            let n = self.units[i].guys.len().min(crate::anim::SQUAD_SIZE);
            for g in 0..n {
                self.reset_pivots(i, g);
            }
        }
        found
    }

    /// **`Guy::reset_pivots@005d8a80(0)`**: every node's `des` angle is
    /// the middle of its restriction, `degrees_to_angle((lo + hi) / 2)`
    /// (the signed halving, `cltd; sub; sar`), plus half a turn when
    /// `hi < lo` — the wrapped arc. `param_1 == 0` leaves the turret's own
    /// angle and both node-flag words alone; the turret walks back by
    /// [`crate::anim::Turret::step`].
    pub(crate) fn reset_pivots(&mut self, i: usize, g: usize) {
        let Some(ty) = self.units[i].ty else {
            return;
        };
        let Some(nodes) = self
            .art
            .pivots
            .get(&self.tables.unit_types[ty].type_index)
            .cloned()
        else {
            return;
        };
        if g >= self.units[i].guys.len() {
            return;
        }
        for node in 4..4 + nodes.len() as i32 {
            let (lo, hi) = nodes.get(&node).copied().unwrap_or((0, 0));
            let mut a = crate::movement::degrees_to_angle((lo + hi) / 2).0;
            if hi < lo {
                a = a.wrapping_add(i32::MIN);
            }
            if let Some(k) = usize::try_from(node - 4).ok().filter(|&k| k < 4) {
                self.units[i].guys[g].turret.des[k] = a;
            }
        }
    }

    /// **The search's `flags` word, which `find_melee_target` derives from
    /// the searcher's own head order** (`docs/COMBAT.md` §64.1; the listing
    /// `005ffc6a`–`005ffcde`). Every caller in the executable but
    /// `do_move`'s passes 0; the word is rewritten only when the head
    /// order is an `ATTACK_TO` (`== 2`) or `order_type` answers
    /// `GROUP_ATTACK_TO` (`== 0x15`):
    ///
    /// ```text
    ///   type->is_siege (+0x10c)  → 0x20002, or the caller's 0 for a
    ///                              human leader (`leader_flags & 4`)
    ///   else !type->is_tank (+0x110) → 0x20010, else the caller's 0
    ///   stance == RAZE (vslot +0xf4 == 4) → 0x20, whatever came above
    /// ```
    ///
    /// So an attack-move's look passes over an unarmed building: a
    /// Woodcutter's Camp the human left standing is no reason for a
    /// Hoplite on its way to the city to stop (item 997, Great Lakes 4555).
    ///
    /// `do_move`'s own call (`005f7b30`) passes 1 or 2 as the caller's
    /// word through [`Sim::find_melee_target_with`] (§37.2, item 3355).
    #[cfg(test)]
    pub(crate) fn melee_search_flags(&self, i: usize) -> u32 {
        self.melee_search_flags_with(i, 0)
    }

    /// [`Sim::melee_search_flags`] over the caller's `word`: the word
    /// stands unless the head is an attack-move, and survives that for a
    /// human's siege (`cmovne` at `005ffcab`) and for a tank (`cmove` at
    /// `005ffcc6`) — the two arms that read "the caller's word" above.
    pub(crate) fn melee_search_flags_with(&self, i: usize, word: u32) -> u32 {
        use crate::orders::index;
        let head = self.current_order(i).map(crate::orders::Order::index);
        if !matches!(head, Some(index::ATTACK_TO | index::GROUP_ATTACK_TO)) {
            return word;
        }
        let cols = self.units[i].ty.map(|t| self.tables.unit_types[t].cols);
        let mut flags = if cols.is_some_and(|c| c.flag(uflags::FLAG_SIEGE)) {
            if self.ai_driven(self.units[i].owner) {
                search::BUILDINGS | search::ARMED
            } else {
                word
            }
        } else if cols.is_some_and(|c| c.flag(uflags::TANK)) {
            word
        } else {
            search::ARMED | search::HALVE_NON_UNITS
        };
        if self.units[i].combat.stance == Stance::Raze {
            flags = search::HALVE_NON_BUILDS;
        }
        flags
    }

    /// `Object::find_nearby_target(max_dist, …)` (§12.2): the ring scan, the
    /// range gate, the distance shaping, the ranking, and the `targeted`
    /// bump on the winner. `max_dist == 0` is unlimited.
    pub fn find_nearby_target(&mut self, attacker: Obj, max_dist: i32) -> Option<Obj> {
        self.find_nearby_target_with(attacker, max_dist, 0)
    }

    /// [`Sim::find_nearby_target`] with the original's fifth argument, the
    /// `flags` word ([`search`]; `docs/COMBAT.md` §64). A building's own
    /// search (`Build::find_target@00622c80`, `622c88`) passes 0, and so
    /// does every unit's but [`Sim::find_melee_target`]'s on an attack-move.
    pub fn find_nearby_target_with(
        &mut self,
        attacker: Obj,
        max_dist: i32,
        flags: u32,
    ) -> Option<Obj> {
        self.find_nearby_target_as(attacker, max_dist, flags, false)
    }

    /// [`Sim::find_nearby_target_with`] with the original's fourth
    /// argument, `cavarch` — the fire on the move's search
    /// ([`crate::cavarch`]). `00648f42`'s first arm takes it with a unit
    /// that is not a unit (`param_4 != 0`): `local_54` (on duty) 0,
    /// `local_24` (must reach) 1 and `local_2c` (guarding) 0, so neither
    /// the guard's post nor the computer's packed-siege shortcut is
    /// asked; and `check_target`'s seventh argument skips both its
    /// region-and-stance head (`:35`) and its `poor_target` tail
    /// (`:130`). The near pair and the `targeted` bump are written as for
    /// any search.
    pub(crate) fn find_nearby_target_as(
        &mut self,
        attacker: Obj,
        max_dist: i32,
        flags: u32,
        cavarch: bool,
    ) -> Option<Obj> {
        if self.attack_of(attacker) == 0 {
            return None;
        }
        let ap = self.profile(attacker);
        let at = self.pos_of(attacker);
        let mut rings = if max_dist == 0 {
            32
        } else {
            (max_dist + UNITS_PER_CELL - 1) / UNITS_PER_CELL
        };
        if matches!(attacker, Obj::Building(_)) {
            rings += 1;
        }
        if ap.has(mask::ANTI_AIR) {
            rings += 1;
        }
        let stance = match attacker {
            Obj::Unit(i) => self.units[i].combat.stance,
            Obj::Building(_) => Stance::Aggressive,
        };
        if stance == Stance::Raid {
            rings += 1;
        }
        let rings = rings.min(32);
        // **`local_24`: the searchers that must reach what they take**
        // (`docs/COMBAT.md` §60, the listing `64911b`–`64918d`). A unit in
        // STAND_GROUND, an entrenched one without `unit_masks2 & 0x20000`,
        // and an **unpacked packer**. The gate below tests their
        // candidates with `is_in_range` and skips any it fails, so an
        // unpacked catapult's idle search never takes a target inside its
        // minimum range. This crate had the flag read the other way round,
        // as "takes anything".
        // **`local_5c`, and the computer's packed siege engine**
        // (`find_nearby_target@00648da0:203-205`, twenty-fourth pass, group
        // 10; A6 row 14): a non-EASIEST computer's packed siege type sets
        // `local_24` **and** `local_5c` before `get_army`, so an engine
        // whose army shortcut failed is still range-tested — an
        // out-of-range unit is skipped, an out-of-range non-unit kept only
        // if it attacks or is a wonder — and scored as out of range.
        let packed_ai_siege = !cavarch
            && match attacker {
                Obj::Unit(i) => {
                    let who = self.units[i].owner;
                    ap.packs
                        && self.units[i].combat.packed
                        && ap.siege
                        && self.ai_driven(who)
                        && !self.search_ai(who)
                }
                Obj::Building(_) => false,
            };
        let must_reach = packed_ai_siege
            || cavarch
            || match attacker {
                Obj::Unit(i) => {
                    let c = self.units[i].combat;
                    c.stance == Stance::StandGround
                        || self.hard_trench(i)
                        || (ap.packs && !c.packed)
                }
                // **A building must reach what it takes** (item 4004,
                // `docs/COMBAT.md` §60.2, `docs/GOLDEN.md` §192): vslot
                // `+0x18` answers 0 for a non-unit, so `64911b`'s first arm
                // sets `local_24` to 1 and `local_5c` stays 0, and the range
                // gate skips every candidate out of range. Ours kept an
                // armed or unit one untested; run1545's capital `0/2000`
                // held the departing `1/14` over `1/12` on 911 and cleared.
                Obj::Building(_) => true,
            };
        let minr = ap.min_range * 0xc0;
        let maxr = self.max_range_of(attacker) * 0xc0;
        // **`local_2c`: a searcher whose activity is a `GUARD`**
        // (`find_nearby_target@00648da0:176`–`184`, `docs/COMBAT.md`
        // §63.3). It scans the rings round its **post**, the guard order's
        // `+0x1c/+0x20` (`:240`–`250`), hands `check_target` its guarding
        // argument, so every candidate is leashed to the post
        // ([`Sim::guard_leash`]) before `near` is written, and it must
        // reach a candidate that is unarmed (`:346`–`352`).
        //
        // The cavalry archer's call (`param_4`) is never guarding
        // ([`Sim::find_nearby_target_as`]).
        let guard = match attacker {
            Obj::Unit(i) if !cavarch => self.guard_activity(i).map(|g| (i, g)),
            _ => None,
        };
        let centre = guard.map_or(at, |(_, g)| g.guard).cell();
        // `local_54`, `Unit::on_duty` of a unit searcher (`006490b6`):
        // `check_target`'s third argument.
        let duty = match attacker {
            Obj::Unit(i) => !cavarch && self.on_duty(i),
            Obj::Building(_) => false,
        };
        // **`local_40`, computed once before the rings** (`00648e6e`), off
        // the *searcher's* own leader and not the candidate's. It is
        // `compare_target`'s fourth argument and nothing else here reads
        // it; `00648e85`'s companion adjustment of the `flags` word on the
        // same arm is a seam (`docs/COMBAT.md` §33.5).
        let ai = self.search_ai(self.owner_of(attacker));

        // The buildings, which this crate does not thread onto the world
        // cell's object chain (`Sim::cell_chain`). The original's chain
        // holds every object; here a cell's units come off the chain and
        // its buildings are appended after them, which is the order the
        // index-ordered scan below this comment already had between the
        // two classes and so changes nothing but the units among
        // themselves. **Stated seam**: a cell holding both a building and
        // a unit that tie can still be scanned in the wrong order.
        let buildings: Vec<Obj> = (0..self.buildings.len())
            .map(Obj::Building)
            .filter(|&o| {
                // A city at its ceiling is still on the chain, and the
                // ring search's `valid_target` call is a capture attempt
                // (`docs/CITIES.md` §7.1's fourth caller, item 1535).
                o != attacker
                    && (self.active(o) || matches!(o, Obj::Building(b) if self.capture_eligible(b)))
            })
            .collect();
        let mut best: Option<(i32, Obj)> = None;
        // **`ObjectData::near_o`/`near_who`, and it is not `best`**
        // (`docs/COMBAT.md` §37.1). `00649527`-`0064953f` keeps the
        // nearest candidate by `check_target`'s own out-distance —
        // `attack_dist` from the searcher's own position — in a local
        // seeded at `9999999`, writes the pair every time that local
        // falls, and does it **above** the `max_dist` gate and above the
        // whole of the scoring. On the way out (`006498de`, and again
        // when the ring table is empty) it clears both to `-1` unless the
        // nearest one it saw is inside `0xf00`.
        let mut near: Option<(i32, Obj)> = None;
        let mut unit_candidates = 0;
        'rings: for r in 0..=rings {
            for dx in -r..=r {
                for dy in -r..=r {
                    if combat::ring_of(dx, dy) != r {
                        continue;
                    }
                    let cell = crate::Cell::new(centre.x + dx, centre.y + dy);
                    if !self.world.contains(cell) {
                        continue;
                    }
                    // **The cell's own `down` chain, head first** — the
                    // order `Object::find_nearby_target@00648da0` walks
                    // (`docs/COMBAT.md` §12.2, and §18.1's measurement of
                    // it). Until item 462 this walked the unit **index**,
                    // which agrees with the chain only when nothing on the
                    // cell ties; chapter two's slinger squad is three
                    // identical hoplites on one cell, two of which tie
                    // exactly, and index order hands the tie to the
                    // oldest where the chain hands it to the newest.
                    let here: Vec<Obj> = self
                        .cell_chain(cell)
                        .into_iter()
                        .map(Obj::Unit)
                        .filter(|&o| o != attacker && self.active(o))
                        .chain(
                            buildings
                                .iter()
                                .copied()
                                .filter(|&o| self.pos_of(o).cell() == cell),
                        )
                        .collect();
                    for o in here {
                        if !self.valid_target(attacker, o) {
                            continue;
                        }
                        // **The `flags` filter** (`00649430`–`00649515`),
                        // above `check_target` and so above `near`.
                        if !self.search_admits(o, flags) {
                            continue;
                        }
                        // **`check_target`'s head** (§86): a candidate
                        // in another region, or any for a defensive unit
                        // off duty with an order, must be in range.
                        if let Obj::Unit(i) = attacker
                            && !cavarch
                            && !self.check_target_reaches(i, o, duty)
                        {
                            continue;
                        }
                        // `check_target`'s guarding arm, above its tail:
                        // out of the leash is refused before `near_o` is
                        // written, and the captain's own target answers 1
                        // there, before `poor_target` is asked.
                        let leash = guard.map(|(i, g)| self.guard_leash(i, &g, o));
                        if leash == Some(Leash::Out) {
                            continue;
                        }
                        // **`check_target`'s tail**, `use_poor` 1
                        // (`00649e00`, `docs/COMBAT.md` §61): a futile
                        // chase is refused before `near_o` is written, and
                        // an unarmed plane's search of a plane is futile.
                        if leash != Some(Leash::Captain)
                            && !cavarch
                            && self.poor_target(attacker, o)
                        {
                            continue;
                        }
                        let mut dist = self.attack_dist(attacker, o);
                        let is_unit = matches!(o, Obj::Unit(_));
                        // `00649527`'s conjunct: one of the two ends has
                        // to be a unit, so a building's search of another
                        // building records no incumbent.
                        if (matches!(attacker, Obj::Unit(_)) || is_unit)
                            && near.is_none_or(|(d, _)| dist < d)
                        {
                            near = Some((dist, o));
                        }
                        if max_dist > 0 && dist > max_dist {
                            continue;
                        }
                        // The range gate (`006495c2`–`0064963b`). A unit
                        // that need not reach is deemed in range untested.
                        // One that must is tested, and a candidate out of
                        // range is skipped: `local_5c`, the one exception,
                        // is a computer's packed siege engine, which this
                        // crate does not flag (§60.4). A building must
                        // reach (item 4004).
                        // A guard deems in range only an armed candidate,
                        // and under `unit_masks & 0x40000` only one without
                        // `ANTI_AIR` (`:346`–`352`); the rest it must reach.
                        let guard_reach = guard.is_some()
                            && (self.attack_of(o) == 0
                                || (self.ai_driven(self.owner_of(attacker))
                                    && self.profile(o).has(mask::ANTI_AIR)));
                        let in_range = if must_reach || guard_reach {
                            if !self.is_in_range(attacker, o) {
                                let wonder = matches!(o, Obj::Building(b)
                                    if self.buildings[b].ty.is_some_and(|t| self.tables.build_types[t].wonder));
                                if !(packed_ai_siege
                                    && !is_unit
                                    && (self.attack_of(o) != 0 || wonder))
                                {
                                    continue;
                                }
                            }
                            false
                        } else {
                            true
                        };
                        if matches!(attacker, Obj::Unit(_)) {
                            if minr == 0 && maxr != 0 && dist + 0x180 < maxr {
                                dist /= 2;
                            }
                            if minr != 0 && dist < minr {
                                dist = minr + (maxr - dist);
                            }
                        }
                        let targeted = match o {
                            Obj::Unit(u) => self.units[u].combat.targeted,
                            Obj::Building(b) => self.buildings[b].targeted,
                        };
                        dist += (targeted + 8) * 0x30;
                        let value = self.compare_target(attacker, o, in_range, ai);
                        let mut score = value / (dist / 0xc0 + 1);
                        // **The `flags` preferences** (`00649766`–
                        // `006497b2`): `0x10` halves what is not a unit,
                        // else `0x20` what is not a `Build` (a wall
                        // answers vslot `+0x20` with 0). A signed `/ 2`
                        // (`cltd; sub; sar`), before the clamp below.
                        let halve = if flags & search::HALVE_NON_UNITS != 0 {
                            !is_unit
                        } else if flags & search::HALVE_NON_BUILDS != 0 {
                            !matches!(o, Obj::Building(b) if self.buildings[b].index < crate::WALL_BASE)
                        } else {
                            false
                        };
                        if halve {
                            score /= 2;
                        }
                        // `local_44` counts a unit candidate here, above the
                        // bearing's skip (`0064979a`), and the ten-unit stop
                        // below is reached only by one that was scored.
                        if is_unit {
                            unit_candidates += 1;
                        }
                        // **The fire on the move looks ahead** (`param_4`,
                        // `006497b4`–`00649869`; item 3385, `docs/AI.md`
                        // §293): the bearing to the candidate,
                        // `find_angle(o.x − x, o.y − y)` off the two raw
                        // points (the listing's `ecx`/`edx`; Ghidra prints
                        // `unaff_EDI, unaff_ESI`), less the searcher's own
                        // `UnitData +0x50`, folded at a half turn. Within
                        // 30° the score is quadrupled, within 60° doubled;
                        // past 135° the candidate is passed over (`ja
                        // 006498a1`), and past 90° a signed tenth.
                        // **`+0x50` is `UnitData::angle`, this crate's
                        // `heading`, not the figure's facing** (item 3530,
                        // `docs/AI.md` §305): the Armored Car `1/23`'s body
                        // lags its heading by 30° on run1155b's 3016, which
                        // puts the Citizen `0/11` at ×2 where facing put it
                        // at ×4, and theirs keeps `0/3`.
                        if cavarch && let Obj::Unit(me) = attacker {
                            let to = self.pos_of(o);
                            let bearing = find_angle(to.x - at.x, to.y - at.y);
                            let mut off =
                                bearing.0.wrapping_sub(self.units[me].movement.heading.0) as u32;
                            if off > 0x8000_0000 {
                                off = !off;
                            }
                            if off < 0x1555_5555 {
                                score <<= 2;
                            } else if off < 0x2aaa_aaaa {
                                score *= 2;
                            } else if off > 0x6000_0000 {
                                continue;
                            } else if off > 0x4000_0000 {
                                score /= 10;
                            }
                        }
                        if score == 0 && value != 0 {
                            score = 1;
                        }
                        // `local_70`, seeded at 0 (`find_nearby_target@00648da0:232`) and taken past
                        // `jle` (`64987d`): a score of 0 — a candidate
                        // `compare_target` answered 0 for — is never the
                        // winner, even the first (item 2001).
                        if score > best.map_or(0, |(b, _)| b) {
                            best = Some((score, o));
                        }
                        if unit_candidates > 10 && best.is_some() {
                            break 'rings;
                        }
                    }
                }
            }
        }
        // **The bump is here and nowhere else.** `00649ba6`, on the way
        // out with a winner: `if (target->targeted < 100) target->targeted++`.
        // Every other write to `ObjectData +0x3d` in the executable is
        // `Object::init`'s zero or the `/4` decay in `Unit::process` /
        // `Wall::process` — `add_attack_order` does not touch it, so a
        // squad handed its captain's target through the mirror adds
        // nothing, and a unit that drops a target subtracts nothing.
        // This crate bumped on the *order* instead, which counted a
        // three-figure squad three times and never let the count fall.
        // `docs/COMBAT.md` §33.
        if let Some((_, o)) = best {
            self.bump_targeted(o, 1);
        }
        // `006498de`: the pair survives only if the nearest candidate the
        // rings saw is inside `0xf00`, and is cleared otherwise — so an
        // empty search *overwrites* a good incumbent rather than leaving
        // it standing. A building carries the pair too, and
        // `Build::process` reads it ([`Sim::process_building_combat`],
        // item 1415).
        let near = near.filter(|&(d, _)| d <= 0xf00).map(|(_, o)| o);
        match attacker {
            Obj::Unit(me) => self.units[me].near = near,
            Obj::Building(b) => self.buildings[b].near = near,
        }
        best.map(|(_, o)| o)
    }

    /// **`Object::compare_target`'s building-class arm** (`0064e79b`–
    /// `0064e9e2`; item 3615, `docs/AI.md` §326): the weight a finished
    /// Build-proper target (vslot `+0x4c`, `WallData::is_active`, and vslot
    /// `+0x20`, which a wall answers 0) takes on its `cost × 4` before the
    /// threat formula. `None` is the arm's `return 100`.
    ///
    /// - `param_4` (the search's `ai`, [`Sim::search_ai`]) first: `× 10`.
    /// - Then, for a searcher **not raiding** (`bVar17`) whose own leader
    ///   is **not human** (`leader_flags & 4`, `0064e836`), by the target's
    ///   class, the first that answers:
    ///   - a **city** (`SubObjectData::flags & 0x20`): `return 100` under
    ///     `ai`, else `× 5`;
    ///   - **defensive** (`WallData::is_defensive@00473650`) and not
    ///     ANTI_AIR (`local_20`, the target's objmask `0x80000000`): `× 4`,
    ///     and `× 40` under `ai` (`× 4 × 5 × 2`, `0064e8ae`–`0064e8be` →
    ///     `0064e9e2`);
    ///   - a **military trainer** (`BuildTypeData::is_military_trainer`):
    ///     `× 3`; without `ai`, a Missile Silo (`is(0x208, 0)`) with
    ///     `queued` (`BuildData +0x82`) non-zero or something inside
    ///     (`inside_down`, `+0x28`, `>= 0`) is `× 15`; and a raiding
    ///     (`bVar3`, `bVar16`) computer ship with SIEGE (`0x148(0x40000)`)
    ///     takes `× 5000` more (the type's `+0x108`, `is_build_type`, is
    ///     every building's);
    ///   - a **training building** (`BuildTypeData::is_training_building`):
    ///     `× 2`.
    ///
    /// run1268's packet (logger frame 3347): the Elite Javelineers `1/27`'s
    /// guard search scores the Market `0/2006` at `320 × 2 × 11` → 71 and
    /// the Library `0/2005` at `240 × 11` → 27, so the Market's 17 beats
    /// the Library's 9 where this crate tied them at 9 and kept the first.
    pub(crate) fn building_class_weight(
        &self,
        attacker: Obj,
        b: usize,
        raiding: bool,
        ai: bool,
    ) -> Option<impl Fn(i32) -> i32> {
        let ai_mul: i32 = if ai { 10 } else { 1 };
        let human = self
            .nation
            .get(self.owner_of(attacker) as usize)
            .is_some_and(|n| n.human);
        let mut class: i32 = 1;
        if !raiding && !human {
            let bd = &self.buildings[b];
            let ty = bd.ty;
            let types = &self.tables.build_types;
            if self.building_is_city(b) {
                if ai {
                    return None;
                }
                class = 5;
            } else if ty.is_some_and(|t| crate::build::is_defensive(types, t))
                && !self.profile(Obj::Building(b)).has(mask::ANTI_AIR)
            {
                class = if ai { 40 } else { 4 };
            } else if ty.is_some_and(|t| crate::build::is_military_trainer(types, t)) {
                class = 3;
                if !ai
                    && ty.is_some_and(|t| {
                        crate::build::is(types, t, crate::build::Ident::MissileSilo)
                    })
                    && (!bd.queue.items.is_empty() || !bd.garrison.is_empty())
                {
                    class = 15;
                }
                if let Obj::Unit(i) = attacker
                    && self.units[i].combat.stance == Stance::Raid
                    && self.unit_ai_bit(self.units[i].owner)
                    && matches!(self.profile(attacker).domain, Domain::Sea)
                    && self.profile(attacker).has(mask::SIEGE)
                {
                    class *= 5000;
                }
            } else if ty.is_some_and(|t| crate::build::is_training_building(types, t)) {
                class = 2;
            }
        }
        Some(move |v: i32| v.wrapping_mul(ai_mul).wrapping_mul(class))
    }

    /// Whether the search's `flags` let a candidate through
    /// (`Object::find_nearby_target`, `00649430`–`00649515`; `docs/COMBAT.md`
    /// §64.2). The candidate's own virtuals, as the PDB's tables name them:
    /// `+0x8` answers for a unit (`SubObjectData::is_active`) and 0 for a
    /// building or wall; `+0x1c` answers 1 for a building **or a wall**;
    /// `+0x120` is its `attack`; `+0x2c` `BuildData::is_wonder`.
    ///
    /// - `& 1`: units only.
    /// - else `& 2`: buildings only, and under `& 0x20000` only an armed
    ///   one, a wonder, or a military trainer.
    /// - else `& 0x20000`: a building must be armed.
    ///
    /// SEAM: `BuildData::attack`'s garrison arm (a general, Antipater,
    /// the Obsidian bonus, §4.1) is read through [`Sim::attack_of`], which
    /// has none of it; no capture on disk garrisons a building an
    /// attack-move passes.
    pub(crate) fn search_admits(&self, o: Obj, flags: u32) -> bool {
        let building = matches!(o, Obj::Building(_));
        if flags & search::UNITS != 0 {
            return !building;
        }
        if flags & search::BUILDINGS != 0 {
            let Obj::Building(b) = o else { return false };
            if flags & search::ARMED == 0 || self.attack_of(o) != 0 {
                return true;
            }
            let bd = &self.buildings[b];
            return bd.index < crate::WALL_BASE
                && bd.ty.is_some_and(|t| {
                    self.tables.build_types[t].wonder || self.is_military_trainer(t)
                });
        }
        !(flags & search::ARMED != 0 && building && self.attack_of(o) == 0)
    }

    /// `Object::compare_target(o, who, in_range, ai)` (§12.3), the skeleton the
    /// simulation can evaluate.
    ///
    /// **`ai` is the searcher's leader, and it inverts the damage weight**
    /// — `docs/COMBAT.md` §33.2. `0064ef4b` reads
    /// `if (ai == 0) v = v * dmg; else { if (dmg == 0) return 0; v = v /
    /// dmg; }`, so a human's search is drawn to what it kills fastest and
    /// a computer leader's to what it kills *slowest*, cost and fragility
    /// carrying the pick instead. It is computed once per search by
    /// [`Sim::search_ai`] and not per candidate, exactly as
    /// `00648e6e`-`00648e8d` computes `local_40` before the rings.
    ///
    /// `ai`'s other arms, the building-class multipliers (`0064e79b`–
    /// `0064e9e2`), are [`Sim::building_class_weight`]'s (item 3615).
    pub fn compare_target(&self, attacker: Obj, target: Obj, in_range: bool, ai: bool) -> i32 {
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let is_build = matches!(target, Obj::Building(_));
        let aa = is_build && tp.has(mask::ANTI_AIR);
        let mut v: i32 = tp.cost.wrapping_mul(4);
        // **A wonder's worth is a twenty-fifth** (`0064e783`–`0064e799`, the
        // target's vslot `+0x2c`, `BuildData::is_wonder`): `cost × 4 / 25`,
        // signed, ahead of every multiplier below.
        if matches!(target, Obj::Building(b)
            if self.buildings[b].ty.is_some_and(|t| self.tables.build_types[t].wonder))
        {
            v /= 25;
        }
        // **The RAID arm** (`0064e6b6`–`0064e742`, `docs/COMBAT.md` §68.1):
        // `bVar17`, the attacker's own stance 3, qualified for an AI-driven
        // (`bVar3`, `unit_masks & 0x40000`) ship (`bVar16`, type `+0x218 ==
        // 1`), which raids only without the SIEGE objmask. It reweights
        // every candidate below and waives the range test's `/5`.
        //
        // SEAM: `has_stance_type`'s refusal (a type with no stance type is
        // never raiding) is not read; a stance this crate sets is RAID only
        // on a type that has one.
        let (raiding, ai_raider) = match attacker {
            Obj::Unit(i) if self.units[i].combat.stance == Stance::Raid => {
                let ai_raider = self.ai_driven(self.units[i].owner);
                let sea = matches!(ap.domain, Domain::Sea);
                (!(sea && ai_raider && ap.has(mask::SIEGE)), ai_raider)
            }
            _ => (false, false),
        };
        // **A computer's raiding ship leaves a building alone**
        // (`0064e7a5`–`0064e81f`, item 2001, `docs/AI.md` §203). Under
        // `iVar5 && iVar4` — a finished (`WallData::is_active`, `flags &
        // 4`) Build-proper target (vslot `+0x20`; a wall answers 0) —
        // `bVar3 && bVar16`, the raid stance on an AI-driven unit
        // (`unit_masks & 0x40000`) of the sea domain, returns 0 when the
        // ship fires on the move (`unit_flags & 0x200000`, `v`), and
        // otherwise when the target is a **dock** and the ship lacks
        // SIEGE. A zero is never taken (`find_nearby_target`'s `local_70
        // < score`, from 0), though `near_o` is already written. run810's
        // packet: the Aegis Cruiser `1/113` returns 0 at `64e7e5` for
        // `0/2000`.
        //
        // **The second test is `is_dock`, not `is_build_type`** (item
        // 3642, `docs/AI.md` §335.2): `64e7ff`–`64e804` call vslot
        // `+0x108` on the target's type (`+0x18`), which the PDB's
        // `ObjectTypeData` field list names `is_dock` (`is_build_type` is
        // `TypeData`'s `+0x14`), and `BuildTypeData::is_dock@00472a70` is
        // `is(DOCK, 0)`. run1303: the computer's raiding Frigate `1/29`
        // takes the human's Farm `0/2004` on its first idle frame, 3859.
        if let Obj::Unit(i) = attacker
            && let Obj::Building(b) = target
            && self.units[i].combat.stance == Stance::Raid
            && self.unit_ai_bit(self.units[i].owner)
            && matches!(ap.domain, Domain::Sea)
            && self.buildings[b].index < crate::WALL_BASE
            && self.buildings[b].active
            && (self.fires_on_move(i)
                || (self.buildings[b]
                    .ty
                    .is_some_and(|t| crate::build::is_dock(&self.tables.build_types, t))
                    && !ap.has(mask::SIEGE)))
        {
            return 0;
        }
        if let Obj::Building(b) = target
            && self.buildings[b].index < crate::WALL_BASE
            && self.buildings[b].active
        {
            match self.building_class_weight(attacker, b, raiding, ai) {
                Some(w) => v = w(v),
                None => return 100,
            }
        }
        let t_attack = self.attack_of(target);
        let left = self.hits_left(target);
        if t_attack != 0 && left != 0 && !aa {
            // A raider looks at an active building's worth, not its threat:
            // `iVar5 == 0 || !bVar17 || iVar4 == 0` takes the formula, and
            // the one case left takes `/ 20` (`0064ea57`–`0064ea7a`).
            if is_build && raiding && self.active(target) {
                v /= 20;
            } else {
                v = v.wrapping_mul(t_attack).wrapping_mul(100) / left;
                if is_build {
                    v = v.wrapping_mul(10);
                }
            }
        }
        if let Obj::Building(b) = attacker
            && self.buildings[b].target == Some(target)
        {
            // `get_base_garrison_arrows` and the squads inside, as
            // `do_attack` counts them (item 3970).
            let arrows = combat::garrison_arrows(
                self.attack_of(attacker),
                self.base_garrison_arrows(b),
                ap.most_shots,
                self.buildings[b].garrison_attack + self.garrison_attack_sum(b),
            );
            if arrows < 2 {
                v = v.wrapping_mul(2);
            } else {
                v /= 2;
            }
        }
        if matches!(attacker, Obj::Building(_))
            && let Obj::Unit(u) = target
        {
            if self.units[u].damage_frac != 0 || self.units[u].health < self.hits_of(target) {
                v = v.wrapping_mul(3) / 2;
            }
            if self.units[u].movement.dest.is_some() {
                v /= 4;
            }
            if tp.is(role::IS_SUPPLY) {
                v = v.wrapping_mul(5000);
            }
        }
        let dmg = {
            let at = self.attacker_side(attacker);
            let tt = self.target_side(target, attacker);
            let pct = self.table_pct(attacker, target);
            // **The real bearing, not `find_angle(0, 0)`** (§46.5). The
            // decompiler prints the call with two zero arguments because
            // `find_angle@0092d130` takes its pair in `ecx`/`edx`, and the
            // listing loads them at `0064ebe7`–`0064ec00` as the target's
            // position less this object's. So the flank sector
            // `get_damage` step 19 reads is the attacker's own, and two
            // otherwise equal candidates at different bearings rank apart.
            let (from, to) = (self.pos_of(attacker), self.pos_of(target));
            // `splash` and `check_overkill` both 0 (`0064ebce`–`0064ebdf`):
            // the ranking never discounts a target another shooter has
            // just hit (item 601, `docs/COMBAT.md` §53).
            combat::get_damage_checked(
                &self.tables.tuning,
                &ap,
                at,
                &tp,
                tt,
                self.attack_of(attacker),
                self.armor_of(target),
                pct,
                find_angle(to.x - from.x, to.y - from.y),
                false,
                false,
                self.frame,
                &self.mods_of(self.owner_of(attacker) as usize),
            )
        };
        // **`0064ef4b`, both arms.** A human multiplies; a computer
        // leader on the lobby's own difficulty divides, and a candidate it
        // cannot hurt at all scores zero rather than dividing by nothing.
        if ai {
            if dmg == 0 {
                return 0;
            }
            v /= dmg;
        } else {
            v = v.wrapping_mul(dmg);
        }
        if is_build {
            // **Armed buildings, a computer's weight** (`0064f124`–`0064f1ed`,
            // item 1131, `docs/COMBAT.md` §12.3): `param_4` is the target
            // type's `attack`, zeroed for an ANTI_AIR target of an attacker
            // not of the air domain. Then `testb $0x4,
            // leader_flags` on the **attacker's** leader: a human (`jne`)
            // skips it, and a computer takes `+1,000,000` with the SIEGE
            // mask, else `×5`. Siege against armed adds `+100,000` for any
            // owner. A raider jumps past all of it (`if (bVar17) goto
            // LAB_0064f1ed`).
            let armed = t_attack != 0 && !(aa && !matches!(ap.domain, Domain::Air));
            // **A city takes the computer's weight only with someone
            // inside** (`64f171`–`64f199`, item 3615): a target whose
            // object flags carry `0x20` (a city) asks `num_inside(0)`, and
            // an empty one jumps to `64f1d1` past the `×5` and the
            // `+1,000,000`, keeping SIEGE's `+100,000`. It is the other
            // half of the class arm's city `×5` ([`Sim::building_class_weight`]):
            // with both, run1268's packet values the empty city `0/2000` at
            // 2787 for `1/27` where this crate read 2785; with the arm alone
            // the city counted `×25`, and the gate alone moved Great Lakes'
            // second game off its close (5930 → 5158, `1/9`'s strike).
            let empty_city = matches!(target, Obj::Building(b)
                if self.building_is_city(b) && self.num_inside(b) == 0);
            if armed && !raiding {
                let human = self
                    .nation
                    .get(self.owner_of(attacker) as usize)
                    .is_some_and(|n| n.human);
                if !human && !empty_city {
                    if ap.has(mask::SIEGE) {
                        v = v.wrapping_add(1_000_000);
                    } else {
                        v = v.wrapping_mul(5);
                    }
                }
                if ap.has(mask::SIEGE) {
                    v = v.wrapping_add(100_000);
                }
            }
        } else {
            if tp.combat_role {
                v = v.wrapping_mul(20);
            }
            // **A caster is weighed by what it is doing** (`0064ec6f`–`0064ed7b`,
            // `docs/COMBAT.md` §12.3, item 3142): a unit target whose type is
            // a spellcaster (`unit_flags2 & 2`) with no cast in its action
            // (`get_action`, order index `CAST_SPELL`) is `+6,000,000` if it
            // is a Spy; with a cast as its action it is `×20`, and `+10,000,000`
            // more when that cast's target is this very attacker. Run920's
            // Commando `0/7` sabotaging the Tank `1/7` outranks every decoy.
            if let Obj::Unit(u) = target
                && self.units[u].ty.is_some_and(|t| {
                    self.tables.unit_types[t].cols.unit_flags2 & crate::ai_load::uflags2::CASTER
                        != 0
                })
            {
                let action = self.action_of(u).map(|i| &self.units[u].orders[i]);
                match action.map(|o| (o.index(), o.body)) {
                    Some((crate::orders::index::CAST_SPELL, crate::orders::Body::Cast(c))) => {
                        v = v.wrapping_mul(20);
                        if c.target == Some(attacker) {
                            v = v.wrapping_add(10_000_000);
                        }
                    }
                    _ => {
                        if self.unit_line_is(u, 0x3a) {
                            v = v.wrapping_add(6_000_000);
                        }
                    }
                }
            }
            // **The raid weights** (`0064edc4`–`0064eff9`): a raider wants
            // the economy. A peasant — exactly `PEASANTS`/`PEASANTSKOREAN`,
            // `ObjectData::is_peasant`, type `0x32`/`0x33` — or a caravan is
            // worth `+900,000` to a computer's raider and `+9,000,000` to a
            // human's; a computer's also takes a combat-role unit at
            // `+10,000`; anything else is worth a tenth.
            //
            // **A computer's raiding ship has its own arm** (`bVar3 &&
            // bVar16`, `0064edd8`–`0064ef64`; item 3662, `docs/AI.md` §342),
            // and a peasant is worth nothing extra in it:
            //
            // - a **Fireship** (`is(FIRESHIP)`, `0x150`) returns 0 for a
            //   fishing boat (`is(FISHERMEN)`, `0x13d`) and takes a tenth of
            //   a target that fires on the move (`unit_flags & 0x200000`);
            // - any other ship that fires on the move takes a Fireship at
            //   `+1,000,000`;
            // - otherwise a transport (`unit_flags & 0x10`) is `+900,000`, a
            //   fishing boat `+100,000`, a military role (`+0x2c8 &
            //   0x10000`) `+10,000`, and anything else stands.
            //
            // run1326's packet runs who=1's Galley `1/21` on 4567: its
            // re-search scores the Citizens `0/3` and `0/10` at 512 and 596,
            // where the land arm's `+900,000` made them 9512 and 9596 and
            // took the nearer.
            let peasant = matches!(target, Obj::Unit(u) if self.is_peasant(u));
            let sea_raider = raiding && ai_raider && matches!(ap.domain, Domain::Sea);
            if sea_raider {
                let line = |o: Obj, t: crate::tech::TypeId| matches!(o, Obj::Unit(u) if self.unit_line_is(u, t));
                let flags_of = |o: Obj| match o {
                    Obj::Unit(u) => self.units[u]
                        .ty
                        .map_or(0, |t| self.tables.unit_types[t].cols.unit_flags),
                    Obj::Building(_) => 0,
                };
                let moving_fire = |o: Obj| flags_of(o) & uflags::FIRES_ON_MOVE != 0;
                if line(attacker, FIRESHIP) {
                    if line(target, crate::fish::FISHERMEN) {
                        return 0;
                    }
                    if moving_fire(target) {
                        v /= 10;
                    }
                } else if moving_fire(attacker) && line(target, FIRESHIP) {
                    v = v.wrapping_add(1_000_000);
                } else if flags_of(target) & uflags::FLAG_TRANSPORT != 0 {
                    v = v.wrapping_add(900_000);
                } else if line(target, crate::fish::FISHERMEN) {
                    v = v.wrapping_add(100_000);
                } else if tp.combat_role {
                    v = v.wrapping_add(10_000);
                }
            } else if raiding {
                if peasant || tp.is(role::IS_CARAVAN) {
                    v = v.wrapping_add(if ai_raider { 900_000 } else { 9_000_000 });
                } else if ai_raider && tp.combat_role {
                    v = v.wrapping_add(10_000);
                } else {
                    v /= 10;
                }
            } else if tp.combat_role {
                v = v.wrapping_add(1_000_000);
            } else if tp.is(role::IS_SUPPLY) {
                v = v.wrapping_add(if matches!(attacker, Obj::Building(_)) {
                    4_000_000
                } else {
                    100_000
                });
            } else {
                v = v.wrapping_add(100_000);
            }
            if matches!(ap.domain, Domain::Land)
                && ap.has(mask::SIEGE)
                && !matches!(tp.domain, Domain::Sea)
                && !tp.has(mask::SIEGE)
            {
                v /= 10_000;
            }
            // **A Bark weighs ships by their line** (`0064f04f`–`0064f0fe`,
            // item 2205, `docs/GOLDEN.md` §80): a sea attacker of the Bark
            // line (`is(BARK)`, `0x143`) against a sea target takes a
            // four-hundredth of a Trireme-line one (`is(TRIREME)`, `0x154`;
            // `imul 0x51eb851f; sar 7`, a signed `/ 400`) and forty times a
            // transport (`unit_flags & 0x10`, `lea; shl 3`). Every human or
            // computer Bark, raiding or not: the arm sits past the raid
            // weights' join. run835's `0/7` takes the Bark `1/8` at 2,082
            // units over the Trireme `1/6` at 768 on 634.
            //
            if matches!(ap.domain, Domain::Sea)
                && matches!(tp.domain, Domain::Sea)
                && ap.is(role::IS_BARK)
            {
                if tp.is(role::IS_TRIREME) {
                    v /= 400;
                } else if let Obj::Unit(u) = target
                    && self.units[u].ty.is_some_and(|t| {
                        self.tables.unit_types[t].cols.unit_flags & uflags::FLAG_TRANSPORT != 0
                    })
                {
                    v = v.wrapping_mul(40);
                }
            }
            // **The target's `full + 1`** (`0064f107`–`0064f11b`, item
            // 3385): every unit target's score, Bark or not, is divided
            // by its [`crate::Unit::full`] plus one, `idiv` toward zero.
            if let Obj::Unit(u) = target {
                v /= i32::from(self.units[u].full) + 1;
            }
        }
        if v < 0 {
            v = 9_999_999;
        }
        // **`in_range` is the caller's permission to test, not its verdict.**
        // `0064f1ed` reads `param_3 != 0 && !raiding` and then calls
        // `is_in_range` *itself*; a candidate the search deemed in range
        // without testing (every candidate of a non-guarding unit that is
        // not STAND_GROUND) is the one this arm actually measures. Reading
        // it the other way round made the golden record's three candidates
        // score equal — see `docs/COMBAT.md` §18.
        if in_range && !raiding && !self.is_in_range(attacker, target) {
            v /= 5;
        }
        v = v.wrapping_add(99) / 100;
        if v > 100_000 {
            v = 100_000 + (v - 99_996) / 5;
        }
        if v < 15 {
            v = 15;
        }
        if matches!(tp.domain, Domain::Air) && !ap.has(mask::ANTI_AIR) {
            v = 2;
        }
        v
    }

    pub(super) fn hits_of(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(i) => self.units[i]
                .ty
                .map_or(self.units[i].health, |t| self.tables.unit_types[t].hits),
            Obj::Building(b) => self.buildings[b].hits,
        }
    }
}
