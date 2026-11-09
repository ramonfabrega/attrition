//! Target validity and sight: the guard leash, `poor_target`, `valid_target` and its constant half, and who sees whom (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    /// **`UnitData::get_activity@00608370`, asked whether it is a
    /// `GUARD`** (`docs/COMBAT.md` §63.1): the first order that is neither
    /// a move nor an attack (`UnitOrder::is_move_attack@0047ff00`, vslot
    /// `+0x1c`). A list that is all moves and attacks answers its tail
    /// only for `ATTACK_TO` or `0x15`, never a `GUARD`, so that arm is not
    /// carried.
    ///
    /// SEAM: `is_attack` (vslot `+0x18`) is taken as the attack and
    /// ground-attack orders; its overrides are folded in the export.
    pub(crate) fn guard_activity(&self, u: usize) -> Option<crate::orders::GuardOrder> {
        use crate::orders::Body;
        let o = self.units[u].orders.iter().find(|o| {
            !(o.is_move() || matches!(o.body, Body::Attack(_) | Body::AttackGround(_)))
        })?;
        match o.body {
            Body::Guard(g) => Some(g),
            _ => None,
        }
    }

    /// **`Object::check_target@00649e00`'s guarding arm**, `0064a00d`–
    /// `0064a190` (`docs/COMBAT.md` §63.1): may a unit guarding a post
    /// take `target`?
    ///
    /// A follower whose captain's action is an `ATTACK` on `target` may,
    /// and the whole of `check_target` answers 1 there (`0064a0eb`). The
    /// captain's action is read through **`Unit::update_action(captain)`**
    /// (`0064a0d4`), whatever its target, so the read rewrites the
    /// captain's `orders_x/y` and `dest_angle` (`+0x58`) from its current
    /// heading: a captain that turned to a new target earlier in the frame
    /// has its `dest_angle` follow the same frame (item 3269, `docs/AI.md`
    /// §258: run946's packet, `1/34`'s `fight` on 10843 copies `1/32`'s).
    /// Otherwise the radius is `unit_guard_respond_range × k × 0x60`,
    /// `k` 3 against a unit that is not a worker (`is_worker@0046fa10`,
    /// type `0x32`–`0x35`) when the guard carries `unit_masks & 0x40000`
    /// ([`Sim::ai_driven`], the one stand-in), else 2. Both the **target**
    /// and **the guard itself** must stand within it of the post, the
    /// order's `+0x1c/+0x20` (`0064a100`–`0064a186`, the listing: the
    /// decompiler lost both `vector_dist`s' operands).
    pub(crate) fn guard_leash(
        &mut self,
        u: usize,
        g: &crate::orders::GuardOrder,
        target: Obj,
    ) -> Leash {
        use crate::orders::Body;
        if !self.units[u].captain {
            let cap = self.squad_captain(u);
            if let Some(a) = self.action_of(cap)
                && matches!(self.units[cap].orders[a].body, Body::Attack(_))
            {
                self.update_action(cap);
                if self.units[cap].combat.target == Some(target) {
                    return Leash::Captain;
                }
            }
        }
        let worker = match target {
            Obj::Unit(t) => self.units[t]
                .ty
                .is_some_and(|ty| (0x32..=0x35).contains(&self.tables.unit_types[ty].type_index)),
            Obj::Building(_) => true,
        };
        let k = if !worker && self.ai_driven(self.units[u].owner) {
            3
        } else {
            2
        };
        let r = self.tables.tuning.unit_guard_respond_range * k * 0x60;
        let (to, me, post) = (self.pos_of(target), self.units[u].pos, g.guard);
        if vector_dist(to.x - post.x, to.y - post.y) > r
            || vector_dist(me.x - post.x, me.y - post.y) > r
        {
            Leash::Out
        } else {
            Leash::In
        }
    }

    /// **`Object::check_target(o, who, 1, NULL, 1, use_poor, 0)`**, the
    /// call `Unit::fight@005fd4d0` makes for a unit whose activity is a
    /// `GUARD` (`005fdd0c`, `docs/COMBAT.md` §63.1): a target in another
    /// `tregion` must be in range; then [`Sim::guard_leash`]; then, with
    /// `use_poor`, not [`Sim::poor_target`].
    ///
    /// The region test is [`Sim::check_target_reaches`], with the call's
    /// third argument 1, so the defensive arm is off here.
    ///
    /// SEAM: the tail's building-cell test is not carried; a guard here
    /// is on land and its target a unit.
    pub(crate) fn guard_check_target(
        &mut self,
        u: usize,
        g: &crate::orders::GuardOrder,
        target: Obj,
        use_poor: bool,
    ) -> bool {
        let me = Obj::Unit(u);
        if !self.check_target_reaches(u, target, true) {
            return false;
        }
        match self.guard_leash(u, g, target) {
            Leash::Captain => true,
            Leash::Out => false,
            Leash::In => !(use_poor && self.poor_target(me, target)),
        }
    }

    /// **`Object::check_target@00649e00`'s head** for a unit searcher
    /// (`docs/COMBAT.md` §86): the candidate is refused when it is out of
    /// range and either
    ///
    /// - the searcher is not `duty` (the call's third argument), stands
    ///   `DEFENSIVE` (vslot `+0xf4` answering 1) and has an order
    ///   (`Unit::update_order` non-null), or
    /// - the two stand in different `WorldData::get_tregion`s — unless
    ///   the searcher is a computer's (`unit_masks & 0x40000`), carries
    ///   the `SIEGE` objmask (`has_objmask(0x40000)`, vslot `+0x148`)
    ///   and its type is a ship (`+0x218 == 1`).
    ///
    /// The listing's head, before `attack_dist`'s out-pointer is read by
    /// the caller and so before `near_o` is written:
    ///
    /// ```text
    /// if this->is_unit() && param_7 == 0:
    ///     other = get_tregion(target tile) != get_tregion(my tile)
    ///     if other && (unit_masks & 0x40000) && has_objmask(0x40000)
    ///              && type->domain == 1:
    ///         other = 0
    ///     if ((param_3 == 0 && stance() == 1 && update_order() != 0)
    ///         || other) && !is_in_range(target):
    ///         return 0
    /// ```
    ///
    /// `Object::find_nearby_target@00648da0` calls it with `param_3 =
    /// Unit::on_duty` and `param_7` its own cavalry-archer argument, which
    /// no caller in this crate passes; `Unit::fight`'s guard call passes
    /// `param_3 = 1`. So a ship's idle search never takes a land building
    /// it cannot hit from where it floats — East Indies' Caravel `1/35`
    /// on 8907 (item 1214), which took `0/2004` five thousand units inland
    /// here and `think_scout` there.
    pub(crate) fn check_target_reaches(&self, u: usize, target: Obj, duty: bool) -> bool {
        let me = Obj::Unit(u);
        let (here, there) = (self.units[u].pos, self.pos_of(target));
        let other = self.world.tregion_alt(here.tile()) != self.world.tregion_alt(there.tile())
            && !(self.ai_driven(self.units[u].owner)
                && self.profile(me).has(mask::SIEGE)
                && self.unit_domain_of(u) == Domain::Sea);
        let defensive = !duty
            && self.units[u].combat.stance == Stance::Defensive
            && self.current_order(u).is_some();
        !((defensive || other) && !self.is_in_range(me, target))
    }

    /// `Object::poor_target@0064a270` — "chasing this one is futile", the
    /// refusal that stands between a captain's cached incumbent and a
    /// retarget (`docs/COMBAT.md` §37.3).
    ///
    /// Three conjuncts, and all three have to hold for the answer to be
    /// *yes*:
    ///
    /// 1. the candidate's head order **is a move** — it is walking;
    /// 2. `get_speed(me) < get_speed(candidate)` — it is **faster than
    ///    me**, each speed taken at its own owner's position, which is
    ///    what [`Sim::get_speed`](crate::Sim::get_speed) already answers;
    /// 3. it is facing **away** from me — `candidate.heading − bearing(me
    ///    → candidate) + 0x80000000` inside the window, and
    ///    [`combat::flanking`] answering **2** rather than 1, so this is
    ///    the one caller in the executable that reads the 1/2 split;
    ///
    /// and then a distance floor: further than a tile, or than my own
    /// reach for the lineage `role & 0x400` names.
    ///
    /// It is `0` for run112's slinger captain and the conjunct that says
    /// so is the **second**: `0/9` prints `myspeed 28` against `1/6`'s
    /// `25`, so a slinger never thinks chasing a hoplite is futile and
    /// the distance floor — one tile, against the 1311 between them — is
    /// never reached. That ordering is why this is worth having exactly
    /// rather than stubbed: stubbed `true` it kills §37.2 outright, and
    /// stubbed `false` it would accept every chase the original refuses.
    ///
    /// **A plane takes the other arm** (`docs/COMBAT.md` §61). Above
    /// conjunct 1, the candidate's first `Guy` carrying `guy_flags & 0x40`
    /// is refused outright by a searcher without `ANTI_AIR`
    /// (`has_objmask(0x80000000)`), and by one with it when it is further
    /// than its own `max_range` tiles. `Guy::init_real@005db6b0` sets the
    /// bit on every figure of a unit `UnitData::is_plane` calls a plane,
    /// an air-domain type without `unit_flags & 0x20`, and only
    /// `Guy::clear` and `init_real`'s own reset write the word otherwise.
    /// So the bit is the type's, and `Sim::is_plane` reads it there.
    ///
    /// SEAM: `role & 0x400` is unloaded. This crate's [`combat::role`]
    /// word is its own synthesis and not the original's, so the floor is
    /// always the tile.
    pub(crate) fn poor_target(&self, me: Obj, cand: Obj) -> bool {
        let (Obj::Unit(u), Obj::Unit(c)) = (me, cand) else {
            return false;
        };
        if self.is_plane(c) {
            return !self.profile(me).has(mask::ANTI_AIR)
                || self.attack_dist(me, cand) > self.max_range_of(me) * 0xc0;
        }
        if !crate::orders::index::is_move_family(self.order_type(c)) {
            return false;
        }
        if self.get_speed(u, 0) >= self.get_speed(c, 0) {
            return false;
        }
        let (mp, cp) = (self.units[u].pos, self.units[c].pos);
        let bearing = crate::movement::find_angle(cp.x - mp.x, cp.y - mp.y);
        let e = (self.units[c].movement.heading.0 as u32)
            .wrapping_sub(bearing.0 as u32)
            .wrapping_add(0x8000_0000);
        if e < 0x2aaa_aaaa || combat::flanking(e) <= 1 {
            return false;
        }
        // The distance floor: `type +0x2c8 & 0x400` — `determine_roles`'s
        // ranged bit, `+0x1fc max_range != 0` — takes the searcher's own
        // reach, anything else the tile (`64a42c`; item 1711: the packet of
        // run773 reads a Dragoon's role as 0x5040c with max_range 9, and
        // `poor_target` answers 0 on a Scout 1162 away).
        let floor = if self.profile(me).max_range != 0 {
            self.max_range_of(me) * 0xc0
        } else {
            0xc0
        };
        self.attack_dist(me, cand) > floor
    }

    /// **`Object::valid_target@00648ba0`** (`docs/COMBAT.md` §12.1,
    /// `docs/CITIES.md` §7.1's fourth caller): the const's answer, and then,
    /// for a **capture-eligible city building**, a capture attempt with the
    /// asking unit as the captor — unless the asker is a missile — before
    /// the answer is given. A city at its ceiling is a valid target only to a
    /// `VEHICLE` **and** `WAR_MACHINE` unit on an `ATTACK` order that is
    /// `mandatory` and names exactly it; to everything else it is refused,
    /// so the call is the capture path's, not the attack's. The attempt may
    /// change the city's owner, so the call is a mutation (item 1535).
    pub fn valid_target(&mut self, attacker: Obj, target: Obj) -> bool {
        self.valid_target_out(attacker, target, None)
    }

    /// [`Self::valid_target`] with the original's fourth argument, the
    /// out-word: 1 when the target is a capture-eligible city building and
    /// the capture the call tried did not take. `Unit::fight@005fd4d0:196`
    /// is the one caller in this crate that reads it — its capture
    /// approach (item 3528, `docs/ORDERS.md` §7.2).
    pub(crate) fn valid_target_out(
        &mut self,
        attacker: Obj,
        target: Obj,
        out: Option<&mut i32>,
    ) -> bool {
        valid_target_steps(
            self,
            out,
            |s| s.valid_target_const(attacker, target),
            |s| matches!(target, Obj::Building(b) if s.capture_eligible(b)),
            |s| s.profile(attacker).has(mask::MISSILE),
            |s| match (attacker, target) {
                (Obj::Unit(u), Obj::Building(b)) => s.check_capture(b, u),
                _ => false,
            },
            |s| {
                let Obj::Unit(u) = attacker else {
                    return false;
                };
                let ap = s.profile(attacker);
                s.units[u].on_map
                    && ap.has(mask::VEHICLE)
                    && ap.has(mask::WAR_MACHINE)
                    && s.order_type(u) == crate::orders::index::ATTACK
                    && s.units[u].combat.target == Some(target)
                    && s.units[u].combat.mandatory
            },
        )
    }

    /// `ObjectData::valid_target_const` + `Object::valid_target` (§12.1), as
    /// far as the simulation's state reaches: not mine, at war, active, on the
    /// map, **and seen**; and the air ladder for a plane target, less its
    /// helicopter and `is(0x132)` arms (`docs/COMBAT.md` §61.2).
    pub fn valid_target_const(&self, attacker: Obj, target: Obj) -> bool {
        if attacker == target {
            return false;
        }
        let (me, them) = (self.owner_of(attacker), self.owner_of(target));
        // `ObjectData::valid_target_const@006472c0`'s first line: `7 < who`
        // is refused before the diplomacy question is even asked, so gaia's
        // animals and birds are nobody's target (`world::PLAYER_SLOTS`).
        // `Object::find_nearby_target@00648da0` walks the cell chains with no
        // leader bound of its own, so this is the only thing keeping them out
        // of the ring search.
        if them >= crate::world::PLAYER_SLOTS {
            return false;
        }
        if me == them {
            return false;
        }
        if !self.at_war_with(me, them) && !self.at_war_with(them, me) {
            return false;
        }
        // A city building at its ceiling is `active` here (its flags still
        // say so): this crate's [`Sim::active`] also asks a building for
        // health left, the target *search's* question, which a city at zero
        // fails and which would refuse it before the capture arm of
        // [`Sim::valid_target`] is reached.
        if !self.active(target) && !matches!(target, Obj::Building(b) if self.capture_eligible(b)) {
            return false;
        }
        // **`ObjectData::valid_target_const@006472c0`'s fifth test, and the
        // one this crate had no term for**: the candidate must be *seen*.
        // The call is `target->vtable[0x48](this->who, 0)` —
        // `UnitData::is_seen@00607a60` — and it sits above every domain and
        // mask test below, so a candidate the searcher's player cannot see
        // is refused before its class is ever asked about
        // (`docs/COMBAT.md` §31).
        if !self.target_is_seen(attacker, target) {
            return false;
        }
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        // **A sea-only attacker** (`6473c3`–`6473ea`, item 3646): a unit
        // whose type's `unit_flags` (`+0x2b4`) carries `0x102000` — `n`,
        // "destroyed when it attacks" (the fire rafts and fire ships), or
        // `u` (the three submarines) — takes only a domain-1 (sea) target,
        // and not one whose vslot `+0x1c` answers (a building). run1304b's
        // Fire Ship `1/86` (TypeIndex 336) is never handed its group's
        // Citizen `0/12` on 4859, where the Frigate and the Bombard Ships
        // beside it are (`docs/AI.md` §335.4).
        //
        // SEAM: the domain-1 arm's other refusal (`647472`) — a unit
        // target whose type carries `0x100000` (a submarine) to a domain-0
        // asker — is not carried; which domain a building asker's type
        // reports is not established here.
        if let Obj::Unit(u) = attacker
            && self.units[u]
                .ty
                .is_some_and(|ty| self.tables.unit_types[ty].cols.flag(0x10_2000))
            && (!matches!(tp.domain, Domain::Sea) || matches!(target, Obj::Building(_)))
        {
            return false;
        }
        if matches!(tp.domain, Domain::Air) {
            if self.max_range_of(attacker) == 0 {
                return false;
            }
            // **The air ladder** (`docs/COMBAT.md` §61, the listing
            // `006474c3`–`0064771c`). For a fixed-wing target, unless the
            // searcher is an `ANTI_AIR` aircraft:
            //
            //     if S.fly_high == 0 && S.fly_low == 0:        refuse
            //     if !S.anti_air:                               # the target's own
            //         if T.fly_high == 0:
            //             if T.fly_low == 0 || high(T):         refuse
            //         elif T.fly_low == 0 && low(T):            refuse
            //     if S.fly_high == 0: if high(T):               refuse
            //     elif S.fly_low == 0 && low(T):                refuse
            //
            // `high` and `low` are `UnitData::is_flying_high@0060a310` and
            // `is_flying_low@0060a140`, and `low` is true only under an air
            // order near its point: slot `0xfc`, which it reads, is
            // `AirOrder::get_air_order` on the three air-order classes and
            // returns 0 on every other. This crate gives no player's unit
            // an air order, so `low` is false and every fixed-wing aircraft
            // on the map is high. run168's Bomber (`FLY_HIGH` 0) therefore
            // refuses the Fighter, while the Fighter, an `ANTI_AIR` aircraft,
            // may take the Bomber.
            //
            // **A helicopter target** (`unit_flags & 0x20`) takes
            // `006474ee`'s arm in place of the ladder (item 3800,
            // `docs/GOLDEN.md` §152.1): a unit searcher whose type is siege
            // (vslot `+0x10c`, `ObjectTypeData::is_siege`) or a tank
            // (`+0x110`, `UnitTypeData::is_tank`), or a missile
            // (`+0x1e4 & 0x8000000`, refused below), or of the Bomber line
            // (`is(0x130)`, vslot `+0xb8`) is refused. run1468's Stealth
            // Bomber `1/17` (TypeIndex 306) is never handed the Attack
            // Helicopter `2/15` on 622, so it spends no `fight` roll on
            // 623; this crate's searcher took it and rolled.
            //
            // SEAM: a building searcher's arm (`+0xb8` the base `is`, so
            // the type's vslot `+0x60`) keeps "ranged and not a missile";
            // and `0064756b`'s `is(0x132)` arm, which refuses a non-air
            // searcher that cannot carry aircraft, is not built. No capture
            // reaches either.
            if let (Obj::Unit(a), Obj::Unit(t)) = (attacker, target)
                && self.is_helicopter(t)
                && (self.is_siege_unit(a)
                    || self.units[a].ty.is_some_and(|ty| {
                        self.tables.unit_types[ty]
                            .cols
                            .flag(crate::ai_load::uflags::TANK)
                    })
                    || self.is_bomber(a))
            {
                return false;
            }
            if matches!(target, Obj::Unit(t) if self.is_plane(t)) {
                let high = !tp.has(mask::MISSILE);
                let anti_air = ap.has(mask::ANTI_AIR);
                if !(anti_air && matches!(ap.domain, Domain::Air)) {
                    if ap.fly_high == 0 && ap.fly_low == 0 {
                        return false;
                    }
                    if !anti_air && tp.fly_high == 0 && (tp.fly_low == 0 || high) {
                        return false;
                    }
                    if ap.fly_high == 0 && high {
                        return false;
                    }
                }
            }
            if tp.has(mask::MISSILE) || ap.has(mask::MISSILE) {
                return false;
            }
            return true;
        }
        // A land-domain or building attacker with ANTI_AIR never targets
        // ground or sea.
        if (matches!(attacker, Obj::Building(_)) || matches!(ap.domain, Domain::Land))
            && ap.has(mask::ANTI_AIR)
        {
            return false;
        }
        true
    }

    /// `UnitData::is_seen@00607a60` with `param_2 == 0`, on the plane this
    /// simulation keeps — the fog half of it.
    ///
    /// ```text
    ///     if (who != owner && reveal_map != 3) {
    ///         if (!World::is_seen(pos / 0x180, who))
    ///             return (visible >> who) & 1;
    ///     }
    ///     return 1;
    /// ```
    ///
    /// and `WorldData::is_seen@006b55c0` is [`Sim::world_is_seen_fog`]:
    /// `seen[fy * fog_xs + fx] & ally_mask` under its three always-true
    /// arms and its territory arm.
    ///
    /// The fallback under the fog test is `ObjectData::visible`, and since
    /// item 457 it is modelled: [`Sim::set_attacking`] sets the target's
    /// own bit for the player it shoots at, so **a unit that attacks you
    /// stays a legal target of yours through the fog** until it goes back
    /// to work. `docs/VISION.md` §7.
    ///
    /// ~~SEAM: `WorldData::is_seen`'s **third** arm — `leader_flags &
    /// 0x2000` and the cell's `WData::who` being an ally — is not
    /// carried, and nor are the two always-true leader arms above it~~ —
    /// all three carried since item 1558 ([`Sim::world_is_seen_fog`]).
    ///
    /// The stealth arm above the fog test is [`Sim::unit_stealth_hides`]
    /// (item 2725, `docs/GOLDEN.md` §101); `unit_masks` `0x800` (not
    /// modelled, nothing sets it here) and `0x1000` (blown; likewise) are
    /// the arm's two bits this crate does not carry.
    pub(crate) fn target_is_seen(&self, attacker: Obj, target: Obj) -> bool {
        self.target_seen_by(self.owner_of(attacker), target)
    }

    /// [`Sim::target_is_seen`] for a player rather than an attacker —
    /// `ObjectData::is_seen(who, 0)` (`SpellTypeData::is_valid_target`'s
    /// Counterintelligence arm asks it of `who` directly).
    pub(crate) fn target_seen_by(&self, who: Player, target: Obj) -> bool {
        if !self.world.has_fog() {
            return true;
        }
        if who >= 8 {
            return true;
        }
        if let Obj::Building(b) = target {
            return self.build_is_seen(b, who);
        }
        if let Obj::Unit(u) = target
            && self.units[u].is_gaia()
        {
            return self.animal_is_seen(u, who);
        }
        if let Obj::Unit(u) = target
            && self.unit_stealth_hides(u, who)
        {
            return false;
        }
        if self.world_sees(self.pos_of(target), who) {
            return true;
        }
        // `return (visible >> who) & 1` — the fallback, and the whole of
        // item 457.
        self.visible_of(target) & Self::who_bit(who) != 0
    }

    /// **`AnimalData::is_seen@005d8160`** for `param_1 = who`, as far as
    /// the fog goes (NATIONS §5.1; `reveal_map == 3` is [`Sim::has_fog`]'s).
    ///
    /// ```text
    /// if herd == -1:                       return was_seen(pos, who)
    /// if good(herd).ever_seen & (1 << who): return 1           // SEAM
    /// if kind(good) not in {6, 0x1f} && (tribe 9 || tribe 4):  // Spanish, 4
    ///                                      return was_seen(pos, who)
    /// return was_really_seen(pos, who)
    /// ```
    ///
    /// An animal never takes `UnitData::is_seen`'s stealth arm or its
    /// `visible` fallback: its answer is the **ever-seen** plane, so a
    /// herd once spotted stays a legal target after the line of sight
    /// moves on.
    ///
    /// SEAM: the herd's good (`HerdData::good_o`) is not carried, so the
    /// good's `ever_seen` bit and its kind (the fish/`0x1f` exemption) are
    /// not read; the kind is taken as not exempt.
    pub(crate) fn animal_is_seen(&self, u: usize, who: Player) -> bool {
        let fog = crate::vision::UNITS_PER_FOG;
        let p = self.units[u].pos;
        let (fx, fy) = (p.x / fog, p.y / fog);
        let bonus = |n| {
            self.tech.get(who as usize).is_some_and(|t| {
                self.tables
                    .tech_tree
                    .has_tribe_bonus(&self.tables.setup, t, n)
            })
        };
        if self.units[u].herd.is_none() || bonus(9) || bonus(4) {
            self.was_seen_fog(fx, fy, who)
        } else {
            self.was_really_seen_fog(fx, fy, who)
        }
    }

    /// `UnitData::is_seen@00607a60`'s stealth arm, for `param_2 == 0`:
    /// answers whether it **refuses** (`return 0`) before the fog test.
    ///
    /// ```text
    /// if (unit_masks & 0x800) || (type flags & 0x4000) || (unit_masks2 & 0x8000)
    ///    || ((type flags & 0x40000) && get_order() == 0):
    ///     if !(unit_masks & 0x1000) && !is_detected(who):   return 0
    /// ```
    ///
    /// `UnitData::is_detected@0060a630`: the owner always, else
    /// `ally_mask(who) & seen3[pos / 0x180]`, the inner disc of a `DETECT`
    /// object ([`crate::world::World::seen3`]). The viewer is never the
    /// owner's own side here — `who == owner` is detected.
    /// `Unit::work@0060d180:380`'s type test: `STEALTH` without `CLOAKED`,
    /// no `unit_masks & 0x800` and no `unit_masks2 & 0x8000`.
    pub(crate) fn unit_stealth_type(&self, u: usize) -> bool {
        let un = &self.units[u];
        let Some(t) = un.ty else {
            return false;
        };
        let cols = self.tables.unit_types[t].cols;
        cols.flag(crate::ai_load::uflags::STEALTH)
            && !cols.flag(crate::ai_load::uflags::CLOAKED)
            && !un.cloak
            && un.unit_masks2 & 0x8000 == 0
    }

    pub fn unit_stealth_hides(&self, u: usize, who: crate::Player) -> bool {
        let un = &self.units[u];
        let Some(t) = un.ty else {
            return false;
        };
        let cols = self.tables.unit_types[t].cols;
        let stealthy = un.cloak
            || cols.flag(crate::ai_load::uflags::CLOAKED)
            || un.unit_masks2 & 0x8000 != 0
            || (cols.flag(crate::ai_load::uflags::STEALTH) && self.is_idle(u));
        // `!(unit_masks & 0x1000)`: a blown unit is seen (item 2785).
        if !stealthy || un.blown || who == un.owner || who >= 8 {
            return false;
        }
        let fog = crate::vision::UNITS_PER_FOG;
        self.world.seen3(un.pos.x / fog, un.pos.y / fog) & self.seen_ally_mask(who) == 0
    }

    /// **`BuildData::is_seen@0062e1a0`**, a building target's vslot
    /// `+0x48`: not the fog plane but the building's own `ever_seen` byte
    /// (`docs/VISION.md` §10.4).
    ///
    /// ```text
    /// if visible & (1 << who):               return 1
    /// if infiltrated by who:                 return 1
    /// WallData::is_seen@00642bd0(who):
    ///     if who == owner:                   return 1
    ///     if (started || ally_mask[who] & (1 << owner))
    ///        && (reveal_map == 3 || ever_seen & ally_mask[who]
    ///            || leader_flags[who] & 0x800 || leader[who] +0x59e4):
    ///                                        return 1
    ///     return 0
    /// ```
    ///
    /// `ally_mask` is `LeaderData +0x6929` ([`Sim::seen_ally_mask`]).
    /// While `seen` never forgot, the fog plane under a building answered
    /// as `ever_seen` does; the hundredth-frame clear (§10) parts them.
    ///
    /// SEAM: a building's `visible` byte is never set here (§9.1), and
    /// infiltration, the two leader arms and the `flags & 0x20` arm under
    /// the semaphore are not carried. Each can only *refuse* further here.
    pub fn build_is_seen(&self, b: usize, who: crate::Player) -> bool {
        let bd = &self.buildings[b];
        if bd.owner == who {
            return true;
        }
        // `visible & (1 << who)`, the first arm: a building that has fired
        // on `who` is seen by it (`Build::do_attack`'s write).
        if bd.visible & Self::who_bit(who) != 0 {
            return true;
        }
        let mask = self.seen_ally_mask(who);
        (bd.started || mask & Self::who_bit(bd.owner) != 0) && bd.ever_seen & mask != 0
    }

    /// `WorldData::is_seen@006b55c0` at one object's position: does `who`'s
    /// alliance currently light the half-cell it stands on?
    pub fn world_sees(&self, p: Pos, who: crate::Player) -> bool {
        let fog = crate::vision::UNITS_PER_FOG;
        self.world_is_seen_fog(p.x / fog, p.y / fog, who)
    }

    /// **`WorldData::is_seen@006b55c0`** at the fog grid's own coordinates —
    /// the *current* line of sight, with its four arms in the original's
    /// order:
    ///
    /// ```text
    /// if who > 7 || reveal_map == 3:                      return 1
    /// if leader_flags & 0x800 || num_units[0x141]:        return 1
    /// if leader_flags & 0x2000 && wdata[fy/2][fx/2].who >= 0
    ///    && is_ally(who, that who):                       return 1
    /// return seen[fy * fog_xs + fx] & ally_mask
    /// ```
    ///
    /// The third arm is the eighth `TECHBONUSES` row, "All units and
    /// buildings in your territory revealed" (Computerization,
    /// [`crate::tech::Roles::territory_reveal_preq`]): a half-cell whose
    /// **cell** an ally of `who` owns is seen whatever the fog says. The
    /// coverage pair's All Technologies start holds it from frame 0
    /// (`leader_flags` `0x2063003` on run678), and it is what lets its scout
    /// walk to a goody box no unit of its has ever seen (`docs/AI.md` §153).
    ///
    /// Off the grid keeps [`crate::world::World::seen`]'s "no answer"
    /// reading, which is what its callers take — the original has no
    /// bounds test here at all and would read past the plane.
    pub(crate) fn world_is_seen_fog(&self, fx: i32, fy: i32, who: crate::Player) -> bool {
        if who > 7 || self.lobby.reveal_map == 3 {
            return true;
        }
        if self.sees_every_unit(who) {
            return true;
        }
        if self.holds_bonus(who, self.tables.tech_tree.roles.territory_reveal_preq)
            && let crate::world::Owner::Player(o) =
                self.world.owner(crate::Cell::new(fx >> 1, fy >> 1))
            && self.is_ally(who, o)
        {
            return true;
        }
        let Some(bits) = self.world.seen(fx, fy) else {
            return true;
        };
        bits & self.seen_ally_mask(who) != 0
    }

    /// `1 << (who & 0x1f)`, **truncated to the byte `ObjectData::visible`
    /// is**. Gaia (`who == 8`) therefore sets no bit at all, which is the
    /// original's own arithmetic and not a guard bolted on here.
    pub(crate) const fn who_bit(who: crate::Player) -> u8 {
        (1u32 << (who & 31)) as u8
    }

    /// `ObjectData::visible` for any object. A building's is modelled for
    /// the units' sake only: nothing sets it yet (`docs/VISION.md` §7).
    pub(crate) fn visible_of(&self, o: Obj) -> u8 {
        match o {
            Obj::Unit(u) => self.units[u].visible,
            Obj::Building(b) => self.buildings[b].visible,
        }
    }
}

/// **`Object::valid_target@00648ba0`'s steps** (item 2287): the out-word
/// zeroed; `valid_target_const` (vslot `+0x138`), refused on a no; a
/// target that is not a capture-eligible city building (`flags & 1`,
/// vslots `+0x20` and `+0x4c`, `check_capture_eligible`) is valid. For
/// one, the out-word is 1, a missile asker (`type +0x1e4 & 0x8000000`)
/// is answered yes, and otherwise the capture is tried (`check_capture`,
/// the asker as captor), the out-word going back to 0 when it takes; the
/// answer is then the armed-attack test alone — on the map, `VEHICLE`
/// and `WAR_MACHINE`, on an `ATTACK` order naming exactly this target,
/// mandatory.
pub(crate) fn valid_target_steps<C>(
    cx: &mut C,
    mut out: Option<&mut i32>,
    is_const: impl FnOnce(&mut C) -> bool,
    eligible: impl FnOnce(&mut C) -> bool,
    missile: impl FnOnce(&mut C) -> bool,
    capture: impl FnOnce(&mut C) -> bool,
    armed_attack: impl FnOnce(&mut C) -> bool,
) -> bool {
    if let Some(o) = out.as_deref_mut() {
        *o = 0;
    }
    if !is_const(cx) {
        return false;
    }
    if !eligible(cx) {
        return true;
    }
    if let Some(o) = out.as_deref_mut() {
        *o = 1;
    }
    if missile(cx) {
        return true;
    }
    if capture(cx)
        && let Some(o) = out
    {
        *o = 0;
    }
    armed_attack(cx)
}

/// What `ObjectData::valid_target_const@006472c0` reads, one method a read,
/// in the order the original makes them (item 3272). `this` is the asker;
/// "the target" is `objects[p][i]` and "its unit" `units[p][i]`; "the asker's
/// slot" is `objects[who][idx]` / `units[who][idx]` for the asker's own
/// `who` byte and slot short. Every method is a pure read except the vslot
/// calls, which may be asked more than once and are answered in call order.
///
/// **Not implemented by [`Sim`]**: this crate's [`Sim::valid_target_const`]
/// is the older partial port (no type-flag arms, no fog-cell arm, no
/// helicopter arm), kept as it stands until a capture parts on one of them;
/// the steps below are the original's whole decision tree, held to it by
/// `sweep::object_data_valid_target_const`.
pub trait TargetConstView {
    /// The asker's `who` byte (`this+9`).
    fn who(&mut self) -> i32;
    /// `LeaderData::is_enemy@006ebaa0` of `leaders[who]` against `other`.
    fn is_enemy(&mut self, who: i32, other: i32) -> bool;
    /// The target's `flags & 1`.
    fn target_active(&mut self) -> bool;
    /// The target's vslot `+0x48` (`is_seen(who, 0)`).
    fn target_is_seen(&mut self, who: i32) -> bool;
    /// The target's vslot `+0x18`, asked up to twice.
    fn target_is_unit(&mut self) -> bool;
    /// The target's unit's vslot `+0xbc` (`UnitData::is_on_map`).
    fn target_on_map(&mut self) -> bool;
    /// The target's type, `+0x218` (its domain).
    fn target_domain(&mut self) -> i32;
    /// The asker's vslot `+0x18`.
    fn self_is_unit(&mut self) -> bool;
    /// The asker's own unit's type, `+0x2b4`.
    fn self_unit_flags(&mut self) -> u32;
    /// The target's vslot `+0x1c`.
    fn target_vslot_1c(&mut self) -> bool;
    /// The asker's vslot `+0x130`.
    fn self_vslot_130(&mut self) -> bool;
    /// The byte of the world's cell the target stands on (`worldc +0x138`,
    /// indexed through `div_3_table`).
    fn target_cell_flags(&mut self) -> u8;
    /// The target's unit's type, `+0x2b4`.
    fn target_unit_flags(&mut self) -> u32;
    /// The asker's type, `+0x218`.
    fn self_domain(&mut self) -> i32;
    /// The asker's vslot `+0x20`.
    fn self_vslot_20(&mut self) -> bool;
    /// The asker's type, `+0x1e4`.
    fn self_type_flags(&mut self) -> i32;
    /// The asker's type, `+0x1fc` (its range).
    fn self_range(&mut self) -> i32;
    /// The asker's type's vslot `+0x10c`.
    fn self_type_vslot_10c(&mut self) -> bool;
    /// The asker's type's vslot `+0x110`.
    fn self_type_vslot_110(&mut self) -> bool;
    /// The asker's vslot `+0xb8` is the base `ObjectData::is@00653790`.
    fn self_is_base(&mut self) -> bool;
    /// The asker's type's vslot `+0x60` (what `is` forwards to).
    fn self_type_vslot_60(&mut self) -> bool;
    /// The asker's vslot `+0xb8` called with `(what, 0)`.
    fn self_vslot_b8(&mut self, what: i32) -> bool;
    /// The target's unit's vslot `+0xb8` is the base `is`.
    fn target_unit_is_base(&mut self) -> bool;
    /// The target's unit's type's vslot `+0x60`.
    fn target_unit_type_vslot_60(&mut self) -> bool;
    /// The target's unit's vslot `+0xb8` called with `(what, 0)`.
    fn target_unit_vslot_b8(&mut self, what: i32) -> bool;
    /// `ObjectData::can_carry@00646c40(this, AIR)`.
    fn can_carry_air(&mut self) -> bool;
    /// The asker's vslot `+0x148` called with `what`.
    fn self_vslot_148(&mut self, what: u32) -> bool;
    /// The asker's slot's object type (`objects[who][idx]`), `+0x250`.
    fn slot_fly_high(&mut self) -> i32;
    /// The same type's `+0x254`.
    fn slot_fly_low(&mut self) -> i32;
    /// The target's unit's type, `+0x250`.
    fn target_fly_high(&mut self) -> i32;
    /// The same type's `+0x254`.
    fn target_fly_low(&mut self) -> i32;
    /// `UnitData::is_flying_high@0060a310` of the target's unit.
    fn target_is_flying_high(&mut self) -> bool;
    /// `UnitData::is_flying_low@0060a140` of the target's unit.
    fn target_is_flying_low(&mut self) -> bool;
    /// The target's own type (`objects[p][i]`), `+0x1e4`.
    fn target_type_flags(&mut self) -> u32;
}

/// **`ObjectData::valid_target_const@006472c0`'s steps** (item 3272): the
/// whole decision tree over [`TargetConstView`], the reads in the
/// original's order, from the `(i, p)` it is handed (`param_1`, `param_2`).
///
/// - refused outright: `i < 0`, `p` outside `0..=7`, `p` the asker's own
///   `who`, `p` not an enemy of it, the target's `flags & 1` clear, not
///   seen by `who`, a unit target (`+0x18`) not on the map;
/// - an asker that is a unit whose own type has `0x102000` set takes only
///   a domain-1 target, and not one whose `+0x1c` answers;
/// - unless the asker's vslot `+0x130` is true, a world cell with both
///   `0x30` bits refuses the target;
/// - domain 1 refuses a unit target flagged `0x100000` to a domain-0
///   asker; every domain but 2 then takes the closing test, a type-flag
///   word below zero refusing when the asker's `+0x20` answers or its
///   domain is 0;
/// - domain 2 (air) needs the asker's range, takes the helicopter arm
///   (unit flag `0x20`: vslots `+0x10c`, `+0x110`, the `0x8000000` bit,
///   then `is(0x130, 0)` unless the base) or the aircraft ladder (the
///   `is(0x132, 0)` answer with `can_carry`, then `+0x148(0x80000000)`
///   choosing which of fly-high / fly-low each side must have, with the
///   real `is_flying_high` and `is_flying_low` of the target), and ends
///   in the missile bit of either type.
pub fn valid_target_const_steps<V: TargetConstView>(v: &mut V, i: i32, p: i32) -> bool {
    if !valid_target_const_args(i, p) {
        return false;
    }
    let who = v.who();
    if p == who || !v.is_enemy(who, p) {
        return false;
    }
    if !v.target_active() || !v.target_is_seen(who) {
        return false;
    }
    if v.target_is_unit() && !v.target_on_map() {
        return false;
    }
    let domain = v.target_domain();
    if v.self_is_unit()
        && v.self_unit_flags() & 0x10_2000 != 0
        && (domain != 1 || v.target_vslot_1c())
    {
        return false;
    }
    if !v.self_vslot_130() && v.target_cell_flags() & 0x30 == 0x30 {
        return false;
    }
    if domain == 1 {
        if v.target_is_unit() && v.target_unit_flags() & 0x10_0000 != 0 && v.self_domain() == 0 {
            return false;
        }
        return closing_test(v);
    }
    if domain != 2 {
        return closing_test(v);
    }
    if v.self_range() == 0 {
        return false;
    }
    if v.target_unit_flags() & 0x20 != 0 {
        if v.self_type_vslot_10c()
            || v.self_type_vslot_110()
            || v.self_type_flags() & 0x800_0000 != 0
        {
            return false;
        }
        return if v.self_is_base() {
            !v.self_type_vslot_60()
        } else {
            !v.self_vslot_b8(0x130)
        };
    }
    let aircraft = if v.target_unit_is_base() {
        v.target_unit_type_vslot_60()
    } else {
        v.target_unit_vslot_b8(0x132)
    };
    if aircraft && v.self_domain() != 2 && !v.can_carry_air() {
        return false;
    }
    if !v.self_vslot_148(0x8000_0000) {
        if v.slot_fly_high() == 0 && v.slot_fly_low() == 0 {
            return false;
        }
        if v.target_fly_high() == 0 {
            if v.target_fly_low() == 0 || v.target_is_flying_high() {
                return false;
            }
        } else if v.target_fly_low() == 0 && v.target_is_flying_low() {
            return false;
        }
        if slot_ladder_refuses(v) {
            return false;
        }
    } else if v.self_domain() != 2 {
        if v.slot_fly_high() == 0 {
            if v.slot_fly_low() == 0 {
                return false;
            }
            if v.target_is_flying_high() {
                return false;
            }
        } else if v.slot_fly_low() == 0 && v.target_is_flying_low() {
            return false;
        }
    }
    v.target_type_flags() & 0x800_0000 == 0 && v.self_type_flags() & 0x800_0000 == 0
}

/// `valid_target_const@006472c0`'s first test: `param_1` a slot index
/// (`>= 0`) and `param_2` a player of the eight (`0..=7`).
pub fn valid_target_const_args(i: i32, p: i32) -> bool {
    i >= 0 && (0..=7).contains(&p)
}

/// `valid_target_const`'s domain-1 and other-domain tail (`0064748e`).
pub(super) fn closing_test<V: TargetConstView>(v: &mut V) -> bool {
    !((v.self_vslot_20() || v.self_domain() == 0) && v.self_type_flags() < 0)
}

/// `006476b3`: the asker's slot type's fly-high / fly-low against the
/// target's unit, after the target's own side passed.
pub(super) fn slot_ladder_refuses<V: TargetConstView>(v: &mut V) -> bool {
    if v.slot_fly_high() != 0 {
        v.slot_fly_low() == 0 && v.target_is_flying_low()
    } else {
        v.target_is_flying_high()
    }
}
