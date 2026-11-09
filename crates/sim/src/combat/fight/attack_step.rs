//! The attack step: `Unit::fight`, the attack setters and pivots, the swing, and firing ammo (`Ammo::init`) (STRUCTURE §5, move 7).

use super::*;

impl Sim {
    /// **`Unit::set_attacking@005ff5b0`** — the write that makes an
    /// attacker visible to the player it is attacking (`docs/VISION.md`
    /// §7).
    ///
    /// Called from the tail of [`Sim::fight`], between the swing animation
    /// and the damage, which is where the original has it. Two things
    /// happen and only the second is conditional:
    ///
    /// - `flags |= 0x80` unconditionally — the latch [`Sim::work`] reads;
    /// - if the victim's player cannot already see this unit's half-cell,
    ///   the bit goes in **and the unit lights its own disc into that
    ///   player's fog** (`vtable[0x164]`,
    ///   [`Sim::update_local_seen_unit`]); if it can, the bit goes in and
    ///   nothing is lit, because there is nothing to reveal.
    ///
    /// Before either, its first two lines **meet**: `00e3a424` is
    /// `leaders.list` (`00e3a390`) `+ 0x94`, `treaties`, and the two writes
    /// are `treaties[owner][who] |= 1` and `treaties[who][owner] |= 1` — a
    /// strike is first contact, both ways, with no `Leader::meet` and no
    /// message (item 3763; run1413b enters this on 620 and `Leader::meet`
    /// never). A victim outside the table (gaia) is the original's write
    /// past `treaties[8]`, which [`Sim::treaty_on`] drops.
    pub(crate) fn set_attacking(&mut self, i: usize, who: crate::Player) {
        self.treaty_on(self.units[i].owner, who, 1);
        let pos = self.units[i].pos;
        let (mut flags, mut visible) = (0u8, self.units[i].visible);
        let reveal = attacking_bits(&mut flags, &mut visible, who, || self.world_sees(pos, who));
        self.units[i].attacking |= flags & 0x80 != 0;
        self.units[i].visible = visible;
        if reveal {
            self.update_local_seen_unit(i);
        }
    }

    // ------------------------------------------------------------------
    // Orders
    // ------------------------------------------------------------------

    /// Gives a unit an attack order on a target — `add_attack_order` with
    /// `mandatory` set, which is what a player's click does.
    /// A player's attack click: `add_attack_order(QUEUE_NEW, mandatory = 1,
    /// action = 1)`.
    pub fn order_attack(&mut self, unit: usize, target: Obj) {
        self.add_attack_order(unit, target, crate::orders::QueuePos::New, true, true);
    }

    pub(crate) fn fight_pub(&mut self, i: usize, target: Obj, frame: i64) -> bool {
        self.fight(i, target, frame)
    }

    /// Tells a building to attack something — `Build::add_attack_order`.
    pub fn order_building_attack(&mut self, building: usize, target: Obj) {
        let b = &mut self.buildings[building];
        b.target = Some(target);
        b.ordered = true;
    }

    pub fn set_stance(&mut self, unit: usize, stance: Stance) {
        self.units[unit].combat.stance = stance;
    }

    pub(super) fn bump_targeted(&mut self, o: Obj, by: i32) {
        match o {
            Obj::Unit(i) => {
                let t = &mut self.units[i].combat.targeted;
                *t = (*t + by).clamp(0, 100);
            }
            Obj::Building(b) => {
                let t = &mut self.buildings[b].targeted;
                *t = (*t + by).clamp(0, 100);
            }
        }
    }

    /// Sets a unit's target, keeping the `targeted` counts.
    pub(super) fn retarget(&mut self, attacker: Obj, target: Option<Obj>, mandatory: bool) {
        let Obj::Unit(i) = attacker else { return };
        let has_order = self.units[i]
            .orders
            .iter()
            .any(|o| matches!(o.body, crate::orders::Body::Attack(_)));
        if let Some(t) = target
            && !has_order
        {
            // A target found by the unit itself becomes an attack order in
            // front of whatever it was doing.
            self.add_attack_order(i, t, crate::orders::QueuePos::First, mandatory, false);
            return;
        }
        let u = &mut self.units[i];
        u.combat.target = target;
        u.combat.mandatory = mandatory && target.is_some();
    }

    // ------------------------------------------------------------------
    // The attack step
    // ------------------------------------------------------------------

    /// The strike itself — `Unit::fight` from step 2 on (§8.2).
    pub(super) fn fight(&mut self, i: usize, target: Obj, frame: i64) -> bool {
        self.fight_as(i, target, frame, false)
    }

    /// [`Sim::fight`] with `fight`'s fifth argument, `cavarch`: the fire
    /// on the move ([`crate::combat::cavarch`]) takes the angle and the strike,
    /// and jumps from `005fe7c5` (`if (param_5 != 0) goto LAB_005feec6`)
    /// over the `set_angle`, the figures' desired angles, the cell-centre
    /// snap and the swing — the walk goes on, under `CHAR_ATTACKWALK`.
    ///
    /// Answers `fight`'s own `1`: the strike reached the reload
    /// (`fight@005fd4d0:1035`), which `Unit::do_attack`'s tail reads to run
    /// a melee-and-ranged type's `cavarch_fight` (item 3570).
    pub(crate) fn fight_as(&mut self, i: usize, target: Obj, frame: i64, cavarch: bool) -> bool {
        let me = Obj::Unit(i);
        let p = self.profile(me);
        // `local_1c`, `is_in_range`'s out-distance (`fight@005fd4d0:481`),
        // measured before the cell-centre snap below: the Immortals' arm
        // reads it at `:784`.
        let reach = self.attack_dist(me, target);
        // Facing: toward the target — or, for a ship that attacks
        // sideways, a quarter turn off it (`docs/COMBAT.md` §49).
        let (from, to) = (self.units[i].pos, self.pos_of(target));
        let direct = find_angle(to.x - from.x, to.y - from.y);
        // `Unit::set_attack@005fce70`, from `fight:591`, before the angle
        // is chosen: the unit's own figures are aimed, and a type with a
        // pivot asks it whether it can bear. When it can, `fight:722`
        // keeps `this->angle` and the unit shoots **without turning**
        // (`docs/COMBAT.md` §52) — and so does an `ANTI_AIR` unit at an
        // aircraft, whatever its pivots answered (`5fe81f`, §83).
        // `fight@005fd4d0:580`: `unit_masks |= 0x11000` ahead of
        // `set_attack` — blown, and never to be dropped (item 2785).
        self.units[i].blown = true;
        self.units[i].fought = true;
        let pivoted = self.set_attack(i, target) || self.anti_air_keeps_angle(i, target);
        let angle = if pivoted {
            self.units[i].movement.heading
        } else {
            let faced = self.building_side(i, target).unwrap_or(direct);
            self.attack_angle(i, target, faced)
        };
        // `Unit::fight@005fd4d0:724`: `Unit::set_angle(angle, …, 0)` when
        // the angle is new, and that is the setter with the turn-around
        // test in it — a group's leader swinging round past 90° toggles
        // the group's `facing` (`docs/GROUPS.md` §6.3). This crate wrote
        // the heading bare until item 530, so who=1's army group kept
        // `facing 0` where run110 flips it on 616, and every layout the
        // army asked for afterwards was mirrored (`docs/ORDERS.md` §22).
        if !cavarch {
            if angle != self.units[i].movement.heading {
                self.unit_set_angle(i, angle);
            }
            // The strike re-seats the figures after changing heading and
            // before asking for the attack. Guy 0 still turns; its tracked
            // crew is snapped to its current facing and may attack now.
            // `Unit::fight@005fd4d0`, before 005feec1 (COMBAT §89).
            let snap = crate::movement::collide::ucell_centre(crate::movement::collide::ucell(
                self.units[i].pos,
            ));
            self.set_new_location(i, snap, true);
            if p.is(role::IS_IMMORTALS) && reach < 0xc0 {
                // **The Immortals' special strike** (`fight@005fd4d0:784`
                // –`:803`; item 3355, `docs/AI.md` §279): inside `0xc0`
                // the swing is `CHAR_ATTACKSPECIAL` with no roll, and the
                // blow lands now, once per figure `0 .. guy_mark` — before
                // `set_attacking`, as `goto LAB_005feec6` has it. The
                // ranged tail below still runs and launches nothing: the
                // piece has no release on the special slot. The Persian
                // word's 6387: `1/10`, 144 from the citizen `0/4`, struck
                // it for 3 where this crate swung `CHAR_ATTACK2` and threw.
                self.set_anim(i, anim::ATTACKSPECIAL, false, false);
                for _ in 0..anim::SQUAD_SIZE {
                    self.do_damage(me, target, angle, false, 0x100, false, false, frame);
                }
            } else {
                self.swing_anim(i, direct);
            }
        }
        // `Unit::fight@005fd4d0`'s `LAB_005feec6`, immediately after
        // `set_anim` and before the damage: the strike makes this unit
        // visible to whoever it just hit (`docs/VISION.md` §7).
        self.set_attacking(i, self.owner_of(target));
        // `LAB_005feec6`'s tail (`005feecb`): under `param_5 == 0` a unit
        // still recharging has swung and aimed, and returns before the
        // blow and the reload. Only a melee-and-ranged type reaches here
        // recharging (the reload gate's `0x400`, item 3568).
        if !cavarch && self.units[i].combat.recharging != 0 {
            return false;
        }
        if self.max_range_of(me) == 0 {
            // Melee lands now, once per figure of this `UnitData` — one here.
            //
            // **Only while `fire_proj` (`UnitType +0x2cc`) stands**
            // (`fight@005fd4d0:853`–`854`, item 3570): `UnitType::init`
            // sets it to 1 for an attacking type, and
            // `GraphicEvents::init_unit_events@008e2520:325` writes it 0
            // for a type whose graphic carries a `RELEASEEVENT` naming a
            // known ammo — the same condition [`Self::launches_from_anim`]
            // reads for the ranged arm below. Such a type swings in melee
            // and lands nothing: its round leaves on the release event.
            // run1189b's Mahout `1/46` (TypeIndex 262, `RANGE 0-8`) swings
            // at `0/2000` on 4240 from its first range, and the building's
            // `damage` stands at 108 through block 4244 in theirs, where
            // ours struck it 4 on 4241.
            if p.fires() && !self.launches_from_anim(i) {
                self.do_damage(me, target, angle, false, 0x100, false, false, frame);
            }
        } else if self.launches_from_anim(i) {
            // **Deferred** — `docs/COMBAT.md` §9.0. `Unit::fight` sets the
            // swing and the reload and launches nothing; the arrow is
            // added by the attack animation's own release event, frames
            // later, from [`Sim::guy_release_events`].
        } else {
            // Not animation-driven, so no release node: the shot leaves
            // the unit's own square, which is what §22's table returns for
            // every piece it has not measured — and `Object::fire_ammo`'s
            // unit arm starts it 100 above the figure (§46.1).
            let sz = self.ground_z(from) + 100;
            self.fire_ammo(me, target, direct, frame, from, sz);
        }
        // Reload.
        self.units[i].combat.recharging = self.reload_frames(i);
        true
    }

    /// **`UnitData::recharge()`** (vslot `+0x134`, §8.3), as the byte
    /// `recharging` holds it: a siege type out of supply and off its own
    /// ground reloads slower.
    pub(crate) fn reload_frames(&self, i: usize) -> u8 {
        let p = self.profile(Obj::Unit(i));
        let unit = &self.units[i];
        let out = if p.siege {
            let owner = unit.owner;
            let at = unit.pos;
            !(self.supplied_at(owner, at)
                || matches!(self.world.owner_at(at), crate::Owner::Player(o) if o == owner))
        } else {
            false
        };
        combat::recharge(p.recharge, out, p.is(role::IS_BOMBARD)) as u8
    }

    /// **`Unit::set_attack(−1, −1)`** (`005fce70`): the aimed figures,
    /// `0 .. guy_mark`, forget their target, and with no target the pivot
    /// test is not reached. `do_attack_ground`'s shot has no object to aim
    /// at, which is why run146's catapult prints `ox −1` from 781.
    pub(crate) fn clear_attack(&mut self, i: usize) {
        self.set_attack_on(i, None);
    }

    /// **`Unit::set_attack@005fce70`** — aim the unit's own figures at
    /// `target`, and answer whether its pivot can shoot without the unit
    /// turning (`docs/COMBAT.md` §52).
    ///
    /// The aim (`GuyData +0x8e`/`+0x9f`) goes into figures `0 ..
    /// guy_mark` only, and `guy_mark` is [`anim::SQUAD_SIZE`]: a crew
    /// figure — a chariot's horse — is never aimed. run145 prints it,
    /// `ox −1 whom −1` on `0/8`'s second figure from 634.
    ///
    /// The answer is `Guy::set_all_pivots` of the **last** of those
    /// figures ([`Sim::set_all_pivots`]), asked only when the type has
    /// `<RESTRICTION>` rows ([`anim::Art::pivots`]), a `max_range`
    /// (`UnitType +0x1fc`), and a target that is a live unit or a
    /// building. Anything else answers 0, and the unit turns.
    pub(crate) fn set_attack(&mut self, i: usize, target: Obj) -> bool {
        self.set_attack_on(i, Some(target))
    }

    /// [`Sim::set_attack`] with the original's `(−1, −1)` as `None`
    /// ([`Sim::clear_attack`]): the one body both run, through
    /// [`set_attack_steps`].
    pub(super) fn set_attack_on(&mut self, i: usize, target: Option<Obj>) -> bool {
        let n = self.units[i].guys.len().min(anim::SQUAD_SIZE);
        set_attack_steps(
            self,
            n,
            target,
            |s, g, t| s.units[i].guys[g].aim = t,
            |s| {
                s.units[i].ty.is_some_and(|ty| {
                    s.roster
                        .art
                        .pivots
                        .get(&s.tables.unit_types[ty].type_index)
                        .is_some_and(|nodes| !nodes.is_empty())
                })
            },
            |s| s.profile(Obj::Unit(i)).max_range != 0,
            |s, t| match t {
                Obj::Unit(t) => s.units[t].alive(),
                Obj::Building(_) => true,
            },
            |s, g, t| s.set_all_pivots(i, g, Some(t)),
        )
    }

    /// **`Guy::set_all_pivots@005d8bc0`** — can every restricted node of
    /// figure `g` bear on `target` from where the unit stands?
    ///
    /// The bearing is measured per node, from the **unit's** position
    /// (`+0x10`/`+0x14` of `objects[who][o]`, read through the figure's
    /// own `who` and `o`) **plus the node's own vector**
    /// ([`crate::data::pivot::offset`], `docs/COMBAT.md` §54), to the target's,
    /// and taken against the **figure's** facing (`GuyData +0x18`) in
    /// whole degrees by
    /// [`crate::movement::angle_to_degrees`], folded to −180..180. The
    /// answer is 0 as soon as that is past ±45° (the executable's own
    /// `float`s at `00b69674` and `00b697bc`), or outside any node's
    /// `minangle..maxangle` — read wrapped when `minangle ≥ maxangle`, so
    /// a range like the Dreadnought's `45..−45` is the rear arc. Nodes
    /// run from 4 for as many rows as the type has; a node with no row
    /// reads `(0, 0)`, as `GraphicPieces::get_restrictions@0090b680`
    /// leaves it.
    ///
    /// Every comparison is on integer degrees, so the `float`s change
    /// nothing.
    ///
    /// **And it aims the turret** (`docs/COMBAT.md` §55.3): the call first
    /// clears `node_flags` (`*(u32 *)(+0x96) = 0`, the bits and their
    /// desired twin), and a target that is gone answers 1 there and writes
    /// nothing more. Each node whose range holds the bearing then takes
    /// `des_turret_angles[node − 4] = bearing − the figure's angle`, and
    /// its bit when that is already within 15° of the turret
    /// (`005d8ed7`–`005d8f02`: the unsigned difference, `~` past a half
    /// turn, below `0xaaa_aaaa`). The ±45° test does not gate the write.
    ///
    /// SEAM: the node's vector is pinned for the Chariot's figure and the
    /// Bomb Vessel's only (`pivot::NODES`); any other piece bears from the
    /// unit's point.
    /// SEAM: an `ATTACK_GROUND` order with no aim bears on the order's
    /// point in the original; this crate writes nothing for it.
    pub(crate) fn set_all_pivots(&mut self, i: usize, g: usize, target: Option<Obj>) -> bool {
        let Some(ty) = self.units[i].ty else {
            return false;
        };
        let Some(nodes) = self
            .roster
            .art
            .pivots
            .get(&self.tables.unit_types[ty].type_index)
            .cloned()
        else {
            return false;
        };
        if nodes.is_empty() || g >= self.units[i].guys.len() {
            return false;
        }
        self.units[i].guys[g].turret.node_flags = 0;
        self.units[i].guys[g].turret.des_flags = 0;
        // `005d8c69`: the gone test is the target slot's `flags & 1`, never
        // its hit points — a city at its ceiling is still borne on
        // ([`Sim::struck`]; item 3635).
        let Some(target) = target.filter(|&t| self.struck(t)) else {
            return true;
        };
        let (from, to) = (self.units[i].pos, self.pos_of(target));
        let guy = self.units[i].guys.get(g).copied();
        let facing = match guy.and_then(|x| x.follow) {
            Some(f) => f.facing,
            None => self.units[i].movement.facing,
        };
        let piece = guy.map_or(-1, |x| x.gpiece);
        let turret = &mut self.units[i].guys[g].turret;
        pivot_steps(
            nodes.len(),
            |node| nodes.get(&node).copied().unwrap_or((0, 0)),
            |node| crate::data::pivot::offset(piece, node, facing),
            (from, to),
            facing,
            turret,
        )
    }

    /// **A building is struck square to its side** (item 1040,
    /// `docs/COMBAT.md` §70): `Unit::fight@005fd4d0`'s `5fe8a7`–`5feb4c`.
    /// For a target whose vslot `+0xc` answers — `SubObjectData::is_active`
    /// on `Build` and `Wall`, a folded `return 0` on `Unit` and `Animal` —
    /// the bearing to the centre is replaced by a whole quarter when the
    /// unit stands beside the footprint: the unit's own tile covered keeps
    /// the bearing (`5fe8f6`); else the first of the tiles **west, north,
    /// east, south** (`x − 0xc0`, `y − 0xc0`, `x + 0xc0`, `y + 0xc0`, each
    /// `div_3_table[v >> 6]`) that the building covers (`WallData::
    /// covers_tile@006439b0`) **and** whose world mask carries `0x4000`
    /// (`world +0x138`, [`crate::world::tile::BLOCKED`]) answers `0xc0000000`,
    /// `0`, `0x40000000` or `0x80000000`. A covered tile that is not
    /// blocked moves on to the next side; the last falls back to the
    /// bearing (`cmovne` at `5feb46`). The sideways ship's quarter turn is
    /// taken after it (`5feb51`).
    pub(super) fn building_side(&self, i: usize, target: Obj) -> Option<Angle> {
        let Obj::Building(b) = target else {
            return None;
        };
        if !self.active(target) {
            return None;
        }
        let here = self.units[i].pos;
        if self.covers_tile(b, here.tile()) {
            return None;
        }
        const SIDES: [(i32, i32, u32); 4] = [
            (-0xc0, 0, 0xc000_0000),
            (0, -0xc0, 0),
            (0xc0, 0, 0x4000_0000),
            (0, 0xc0, 0x8000_0000),
        ];
        SIDES.iter().find_map(|&(dx, dy, a)| {
            let t = Pos::new(here.x + dx, here.y + dy).tile();
            (self.covers_tile(b, t) && self.world.tile_mask(t) & crate::world::tile::BLOCKED != 0)
                .then_some(Angle(a as i32))
        })
    }

    /// **The angle a unit attacks on** — `Unit::fight@005fd4d0:698–714`
    /// (`docs/COMBAT.md` §49).
    ///
    /// The bearing to the target, except for a type carrying `g`
    /// (`unit_flags & 0x40`, "Unit attacks sideways (most ships)"): that
    /// one attacks **broadside**, on the bearing plus or minus a quarter
    /// turn, whichever is nearer the heading it already has. The nearness
    /// is the listing's own: the unsigned difference folded by `~` past
    /// half a turn, and the minus side taken only when it is strictly
    /// nearer. The one exemption is a `PATROLBOAT` (`0x185`, the attacker's
    /// own `TypeIndex`) whose target is at sea (`domain == 1`).
    ///
    /// SEAM: `Object::fire_ammo` takes no angle, so the shot is still
    /// handed the direct bearing here, which only the near-face aim at a
    /// building reads. No capture has a ship shooting a building.
    pub(crate) fn attack_angle(&self, i: usize, target: Obj, direct: Angle) -> Angle {
        let Some(ty) = self.units[i].ty else {
            return direct;
        };
        let t = &self.tables.unit_types[ty];
        if !t.cols.flag(uflags::SIDEWAYS) {
            return direct;
        }
        /// `TypeIndex::PATROLBOAT`.
        const PATROLBOAT: i32 = 0x185;
        if t.type_index == PATROLBOAT && matches!(self.profile(target).domain, Domain::Sea) {
            return direct;
        }
        const QUARTER: i32 = crate::ids::scale::QUARTER_TURN;
        let heading = self.units[i].movement.heading.0;
        let off = |a: i32| {
            let d = heading.wrapping_sub(a) as u32;
            if d > 0x8000_0000 { !d } else { d }
        };
        let minus = direct.0.wrapping_sub(QUARTER);
        let plus = direct.0.wrapping_add(QUARTER);
        Angle(if off(minus) < off(plus) { minus } else { plus })
    }

    /// **The swing's animation** — `Unit::fight@005fd4d0`'s tail, the block
    /// that ends at the `Unit::set_anim` call at `005feec1`
    /// (`docs/ANIM.md` §6.2).
    ///
    /// It sits here, between the facing and the damage, because that is
    /// where the original has it: `set_angle`, every guy's `des_angle`
    /// written to the attack angle, `set_new_location`, **this**, then
    /// `set_attacking` and `Object::do_damage` at `+0x1e72`.
    ///
    /// Four arms reach the call and **only the last carries the third
    /// argument**, so only the last can roll:
    ///
    /// - `unit_flags & 0x2000000` — `z`, "Unit rocks left/right when it
    ///   attacks (attack1 is left, attack2 is right)", the eighteen ship
    ///   types: `CHAR_ATTACK2` when the direct angle to the target is at
    ///   or past the angle the unit is attacking on, else `CHAR_ATTACK1`,
    ///   `param_3 = 0`;
    /// - a **target** of type `PATROLBOAT` (`0x185`): the same pair by the
    ///   target's own domain, `param_3 = 0`;
    /// - `is(IMMORTALS)` (`0xa2`) inside `0xc0`: `CHAR_ATTACKSPECIAL`,
    ///   `param_3 = 0`, with its own per-figure damage — taken in
    ///   [`Sim::fight_as`] ahead of this (item 3355);
    /// - everything else: `CHAR_ATTACK1`, `param_3 = 1`.
    ///
    /// SEAM: the patrol-boat arm is not modelled — one target type — nor
    /// the `unit_flags & 0x400` arm ahead of the Immortals' (`CHAR_
    /// ATTACKSPECIAL` for a ranged type), and neither draws, so the cost is which slot
    /// plays rather than a word. The `z` arm compares the direct bearing,
    /// `angle` here, against `Unit::fight`'s own attack angle, which is the
    /// heading [`Sim::attack_angle`] has just set. That is the bearing, or
    /// for a sideways (`g`) ship a quarter turn off it, so since item 535
    /// a ship rocks to the side it turned to (`docs/COMBAT.md` §49). The
    /// second arm's `0x185` is the **attacker's** own `TypeIndex`, not the
    /// target's: `local_20` is written from `this->ptype + 4` at
    /// `fight:609` and not again before `:805`.
    pub(super) fn swing_anim(&mut self, i: usize, angle: Angle) {
        let rocks = self.units[i]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.flag(uflags::ROCKS));
        if rocks {
            let attack_angle = self.units[i].movement.heading;
            let slot = if angle.0.wrapping_sub(attack_angle.0) >= 0 {
                anim::ATTACK2
            } else {
                anim::ATTACK1
            };
            self.set_anim(i, slot, false, false);
        } else if self.radar_jams(i) {
            self.set_anim(i, 0, false, true);
        } else {
            self.set_anim(i, anim::ATTACK1, false, true);
        }
    }

    /// Whether this unit's shot is launched by its **animation** rather
    /// than by `Unit::fight` — `docs/COMBAT.md` §9.0.
    ///
    /// True exactly when the install's own `<RELEASEEVENT>` table knows
    /// the unit's graphic piece. That is the original's own condition read
    /// the only way this crate can read it: `Unit::fight@005fd4d0` calls
    /// `Object::fire_ammo` only when the type's `+0x2cc` (its fire-projectile
    /// graphic) is set or the merchant arm holds, and Great Lakes'
    /// Longbowmen reach neither — their arrow is a `RELEASEEVENT` on
    /// `CHAR_ATTACK1`, `2` and `3`. `+0x2cc` has no reading here and no
    /// capture on this disk spends it (§9.0's SEAM), so a piece the table
    /// does not name keeps the immediate launch this crate has always
    /// taken, which is what leaves every sim built from tables alone —
    /// the unit tests, the soak — meaning what it meant.
    pub(crate) fn launches_from_anim(&self, u: usize) -> bool {
        self.units[u]
            .guys
            .first()
            .is_some_and(|g| self.roster.art.releases.contains_key(&g.gpiece))
    }

    /// **A strafer's landing walks with its gun** (`Ammo::init`,
    /// `0x67c9b2`–`0x67cb1a`; `docs/ORDERS.md` §39.3, item 853). After
    /// the scatter and `ez`, before the world's clamp, for a unit shooter
    /// whose type strafes (`0x400000`), whose target is not a flying one
    /// (a target of either index −1 — an attack-ground round — is walked
    /// too), and whose **guy 0** plays an animation of `CHAR_ATTACK2`'s
    /// category (`UnitAnimCat[cur_anim] == 12`), the landing is
    /// `project`ed twice:
    ///
    /// - along the shooter's heading (`UnitData +0x50`, [`crate::Movement::heading`]) by
    ///   `trunc(((float)cur_time / (float)end_time − 0.3f) · 6 · 192)`,
    ///   guy 0's clocks, in singles: `divss`, `subss 0x3e99999a`, `mulss
    ///   6`, `mulss 192`, `cvttss2si`. So the rounds sweep from 345 short
    ///   of the target to 806 past it across the swing;
    /// - then 48 across it: `heading + 90°` for an even node, `− 90°` for
    ///   an odd one (`test byte [package +0x1c], 1`).
    ///
    /// Done in [`Single`], which is the original's arithmetic to the bit
    /// (`CLAUDE.md`, the hard constraints). SEAM: an `end_time` of zero,
    /// whose quotient is not a number, is refused rather than walked; an
    /// attack animation never has one.
    pub(crate) fn strafe_walk(&self, shooter: Obj, target: Option<Obj>, at: Pos, node: i8) -> Pos {
        const F0_3: Single = Single::from_bits(0x3e99_999a);
        const F6: Single = Single::from_bits(0x40c0_0000);
        const F192: Single = Single::from_bits(0x4340_0000);
        let Obj::Unit(u) = shooter else {
            return at;
        };
        if !self.strafes(shooter) {
            return at;
        }
        if target.is_some_and(|t| matches!(self.profile(t).domain, Domain::Air)) {
            return at;
        }
        let Some(g) = self.units[u].guys.first() else {
            return at;
        };
        if crate::units::anim::category(g.anim) != crate::units::anim::ATTACK2 || g.end_time == 0 {
            return at;
        }
        let along = Single::from_u32(g.cur_time)
            .divss(Single::from_u32(g.end_time))
            .subss(F0_3)
            .mulss(F6)
            .mulss(F192)
            .to_i32();
        let heading = self.units[u].movement.heading;
        let project = |a: Angle, d: i32, p: Pos| {
            Pos::new(
                p.x + crate::movement::sin_component(a, d),
                p.y - crate::movement::cos_component(a, d),
            )
        };
        let at = project(heading, along, at);
        let side = if node & 1 == 0 {
            Angle(heading.0.wrapping_add(0x4000_0000))
        } else {
            Angle(heading.0.wrapping_sub(0x4000_0000))
        };
        project(side, 0x30, at)
    }

    /// The landing scatter's two draws, under the original's own site
    /// names — `Ammo::init+0xcd9` and `+0xd0b` (§9.1).
    ///
    /// [`combat::scatter_point`] is the arithmetic; this is the same thing
    /// with a mark before each draw, because the two addresses are two
    /// entries in the compared sequence and a single label would fold
    /// them into one.
    pub(super) fn scatter_landing(&mut self, at: Pos, s: i32, sites: (&str, &str)) -> Pos {
        if s - 1 < 1 {
            return at;
        }
        self.mark(sites.0);
        let dx = self.rng.roll() % s - s / 2;
        self.mark(sites.1);
        let dy = self.rng.roll() % s - s / 2;
        Pos::new(at.x + dx, at.y + dy)
    }

    /// `Object::fire_ammo` for a unit (§9.1): one `Ammo` per figure, here one.
    ///
    /// A siege type firing at a **unit** fires at the ground under it
    /// (`fight` inserts an `ATTACK_GROUND` order at the target's position,
    /// §8.2 step 1): the shot has no target to home on or to test against,
    /// and finds what it finds where it lands.
    ///
    /// `launch` is where the shot actually leaves from — the release
    /// node's world position for a unit whose animation fires it (§22),
    /// and the shooter's own position for everything else.
    ///
    /// `sz` is the height it leaves from: the figure's ground plus the
    /// release node's `dz`, or plus 100 for a unit that fires from its
    /// square (§46.1).
    ///
    /// `node` is the release event's node, which `execute_game_events`
    /// puts in the package's `angle` for `Ammo::init` (a strafer's side,
    /// `docs/ORDERS.md` §39.3).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fire_ammo_pub(
        &mut self,
        shooter: Obj,
        target: Obj,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
        node: i8,
        harmless: bool,
        missile: bool,
    ) {
        self.fire_ammo_release(
            shooter, target, angle, frame, launch, sz, node, harmless, missile, false, false,
        );
    }

    /// [`Self::fire_ammo_pub`] for a release event, which also says whether
    /// its ammo burns (`flame="1"`, [`combat::Projectile::flame`]).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fire_ammo_release(
        &mut self,
        shooter: Obj,
        target: Obj,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
        node: i8,
        harmless: bool,
        missile: bool,
        flame: bool,
        punctures: bool,
    ) {
        self.fire_ammo_aim(
            shooter,
            Aim::At(target),
            angle,
            frame,
            launch,
            sz,
            node,
            harmless,
            missile,
            flame,
            punctures,
        );
    }

    /// `Objects::add_ammo@00658b10`: the shot takes the **lowest free
    /// slot** of the pool — it scans from 0 and breaks at the first whose
    /// `flags & 3` is clear — and the pool is stepped in slot order
    /// (`Objects::inc_time@0065db70`), so the list is kept sorted by slot
    /// and a landing removes without disturbing the rest (§46.4).
    ///
    /// Which of two shots due on the same frame lands first is decided
    /// here: chapter two's `0/7` and `0/8` fire together on 677, hold
    /// slots 1 and 2, and the first to come down on 683 takes `1/8` while
    /// the second finds it dead and rolls on.
    pub(crate) fn add_ammo(&mut self, mut p: combat::Projectile) {
        let live: Vec<u32> = self.combat.projectiles.iter().map(|q| q.slot).collect();
        let slot = lowest_free_slot(&live);
        p.slot = slot;
        let at = self.combat.projectiles.partition_point(|q| q.slot < slot);
        self.combat.projectiles.insert(at, p);
    }

    /// `TerrainOut::find_data_z(x, y, 0)` — [`crate::World::data_z`] — with
    /// the read counted in [`Sim::ground_inexact`] when it touched a
    /// corner this crate cannot pin to the original's single.
    pub fn ground_z(&mut self, at: Pos) -> i32 {
        let (z, exact) = self.world.data_z(at.x, at.y);
        if !exact {
            self.trace.ground_inexact += 1;
        }
        z
    }

    /// `Ammo::init`'s `ez` (`+0x20`), `0x67c93a`–`0x67c9a4`: the target
    /// object's own `z` (`ObjectData +0xc`, unmasked) — `find_tcoord_z` at
    /// its tile with the fourth argument 1, which is
    /// [`crate::World::object_z`]: an ocean tile or a height under the sea
    /// reads 0 — plus **75** when the shot rolls, and never below zero
    /// (§46.1). A Citizen on a shore tile at −22 is aimed at 75, not 53
    /// (item 3593, `docs/AI.md` §315.4).
    ///
    /// **An air unit is aimed at its first figure's altitude**
    /// (`0x67c949`..: vslot `+0x18` non-zero and the type's domain
    /// `+0x218 == 2` load `guys[0] +0x14`; item 3804, `docs/GOLDEN.md`
    /// §152.3), the `GUY` record's `z`, which is
    /// [`crate::Unit::airframe`]'s — not the ground under it. run1469's
    /// Attack Helicopter `1/15` at `z` 60 is aimed at 60, and the parked
    /// Jet Fighter `1/14` at 0, where the ground read 533 and 529.
    pub(super) fn aim_z(&mut self, target: Obj, rolling: bool) -> i32 {
        let at = self.pos_of(target);
        let mut z = self.world.object_z(at.tile()) + if rolling { 0x4b } else { 0 };
        if let Obj::Unit(u) = target
            && matches!(self.profile(target).domain, Domain::Air)
        {
            z = self.units[u].airframe.z;
        }
        z.max(0)
    }

    /// `Object::fire_ammo`'s round, from `Unit::fight`. SEAM: the package
    /// it hands `Ammo::init` is unread here, so its `angle` is taken as 0;
    /// only a strafer's landing reads it, and a strafer's round is its
    /// animation's (`docs/ORDERS.md` §39.2), never this one.
    pub(super) fn fire_ammo(
        &mut self,
        shooter: Obj,
        target: Obj,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
    ) {
        self.fire_ammo_aim(
            shooter,
            Aim::At(target),
            angle,
            frame,
            launch,
            sz,
            0,
            false,
            false,
            false,
            false,
        );
    }

    /// A round at the shooter's **attack-ground order's point**
    /// (`docs/COMBAT.md` §57.4) — the release event's arm for a unit
    /// whose current order is `ATTACK_GROUND`, and `do_attack_ground`'s
    /// own shot for a unit no animation launches.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn fire_ammo_ground(
        &mut self,
        shooter: Obj,
        g: crate::orders::AttackGroundOrder,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
        node: i8,
        harmless: bool,
        missile: bool,
        flame: bool,
        punctures: bool,
    ) {
        self.fire_ammo_aim(
            shooter,
            Aim::Ground(g),
            angle,
            frame,
            launch,
            sz,
            node,
            harmless,
            missile,
            flame,
            punctures,
        );
    }

    /// The lead `Ammo::init@0067bbf0` pushes the landing by (`0067cf1a`'s
    /// block, shared by a unit shooter and a building's): a unit target
    /// whose order is a move or an air order, `sinx`/`cosx` of its
    /// `UnitData +0x50` at its first figure's `avg_speed`, times the
    /// time of flight, `WorldData::restrict`ed. The building arm takes it
    /// too (item 1380: the Tower's shot at the moving Scout on 1482).
    pub(super) fn lead_landing(&self, target: Option<Obj>, landing: Pos, total_time: i32) -> Pos {
        let Some(Obj::Unit(t)) = target else {
            return landing;
        };
        let k = self.order_type(t);
        if !(crate::orders::index::is_move_family(k) || crate::orders::index::is_air_family(k)) {
            return landing;
        }
        let u = &self.units[t];
        let angle = u.movement.heading;
        let avg = u
            .guys
            .first()
            .and_then(|g| g.follow)
            .map_or(u.movement.body.avg_speed, |f| f.body.avg_speed);
        let q = Pos::new(
            landing.x + crate::movement::sin_component(angle, avg) * total_time,
            landing.y - crate::movement::cos_component(angle, avg) * total_time,
        );
        Pos::new(
            q.x.clamp(0, self.world.width() * UNITS_PER_CELL - 1),
            q.y.clamp(0, self.world.height() * UNITS_PER_CELL - 1),
        )
    }

    /// `BuildData::get_shot@0062dd90`: the exact-type arms
    /// ([`build::BuildType::shot`]), else the owner's `ages` — 0 up to the
    /// third, 1 for the fourth and fifth, 2 past them.
    pub(super) fn building_shot(&self, b: usize) -> i32 {
        let bl = &self.buildings[b];
        if let Some(n) = bl.ty.and_then(|t| self.tables.build_types[t].shot) {
            return n;
        }
        match self.tech[bl.owner as usize].ages {
            a if a < 3 => 0,
            a if a < 5 => 1,
            _ => 2,
        }
    }

    /// `Ammo::init@0067bbf0`, for either aim.
    ///
    /// **Ground fire is the shooter's order, not its type.**
    /// `Ammo::init`'s `local_38` is the shooter's current order read as an
    /// `AttackGroundOrder` (`get_order`, vslot `+0xd0`), null for any
    /// other: the target half is then `−1`, the accuracy is against the
    /// plain distance to the order's `att_x/att_y`, the scatter is the
    /// land-unit formula unless the order's `accuracy` says the point is
    /// at sea (then none), the landing is the **order's point** plus the
    /// scatter, and `ez` is `find_data_z` at the point, clamped at zero
    /// (`init:458`–`492`). This crate read ground fire off the type — a
    /// siege packer at a unit — and aimed it at the unit's position on the
    /// release frame, seventeen frames after the original fixed its point
    /// (run146's catapult, §57.4).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fire_ammo_aim(
        &mut self,
        shooter: Obj,
        aim: Aim,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
        node: i8,
        harmless: bool,
        missile: bool,
        flame: bool,
        punctures: bool,
    ) {
        // **A round at an aircraft rolls to hit** (item 1102,
        // `docs/COMBAT.md` §81): `Ammo::init`'s air arm, ahead of the
        // Bomber's and the scatter's; its miss is the same flag `0x10`.
        let harmless = match aim {
            Aim::At(t) => self.air_round_misses(shooter, t) == Some(true) || harmless,
            Aim::Ground(_) => harmless,
        };
        let p = self.profile(shooter);
        // **A missile leaves its own point** (item 3806, `docs/GOLDEN.md`
        // §152.4; `Ammo::init@0067bbf0`, `67c1d5`..`67c263`): a shooter
        // whose type carries the missile flag (`+0x1e4 & 0x8000000`)
        // launches from its object's `(x, y, z_internal)` plus
        // [`crate::air::MISSILE_OFFSET`], not from the release point the
        // figure's piece names. run1470's Cruise Missile `2/19` at
        // (16632, 16632), `z_internal` 570, fires from (16523, 16633, 958).
        let missile_unit = match shooter {
            Obj::Unit(u) if p.has(mask::MISSILE) => Some(u),
            _ => None,
        };
        let (launch, sz) = match missile_unit {
            Some(u) => {
                let here = self.units[u].pos;
                let off = crate::air::MISSILE_OFFSET;
                (
                    Pos::new(here.x + off.0, here.y + off.1),
                    self.world.object_z(here.tile()) + off.2,
                )
            }
            None => (launch, sz),
        };
        let (target, ground) = match aim {
            Aim::At(t) => (Some(t), None),
            Aim::Ground(g) => (None, Some(g)),
        };
        let tp = target.map_or_else(no_profile, |t| self.profile(t));
        // **A bomb falls** (`docs/ORDERS.md` §35.3). `Ammo::init`'s
        // `is(0x130)` test (`0x67c289`, `[ebp − 0x30]`) takes a Bomber's
        // round past the whole accuracy-and-scatter block: no accuracy, no
        // draw, no near-face aim and no lead. The landing is the launch
        // `project`ed one tile (`edx = 0xc0`) along the shooter's own
        // heading (`ecx = UnitData +0x50`), `WorldData::restrict`ed,
        // `find_data_z` there unclamped; the time is the fall's.
        if let Obj::Unit(su) = shooter
            && self.is_bomber(su)
        {
            let facing = self.units[su].movement.facing;
            let w = self.world.width() * UNITS_PER_CELL;
            let h = self.world.height() * UNITS_PER_CELL;
            let landing = Pos::new(
                (launch.x + crate::movement::sin_component(facing, 0xc0)).clamp(0, w - 1),
                (launch.y - crate::movement::cos_component(facing, 0xc0)).clamp(0, h - 1),
            );
            let ez = self.ground_z(landing);
            // `Ammo::init:795`: a zero time is raised to 1, and nothing
            // else — a NaN fall's `0x80000000` stands ([`combat::fall_time`]).
            let total_time = match combat::fall_time(sz, ez) {
                0 => 1,
                t => t,
            };
            let _ = (angle, frame);
            self.add_ammo(combat::Projectile {
                shooter,
                owner: self.owner_of(shooter),
                target,
                launch,
                landing,
                cur_time: 0,
                total_time,
                // SEAM: the arm writes no `accuracy` (`+0x6`), so the
                // recycled slot keeps its last round's; only a unit
                // target's hit test reads it, and a building is struck.
                accuracy: 0,
                angle: crate::movement::find_angle(landing.x - launch.x, landing.y - launch.y),
                splash_area: p.splash_area,
                num_guys: 1,
                rolling: false,
                missed: false,
                harmless,
                air: target.is_some() && matches!(tp.domain, Domain::Air),
                sz,
                ez,
                v1z: combat::arc_v1z(sz, ez, total_time),
                slot: 0,
                flame,
                punctures,
                spent: false,
                crash: None,
                spline: None,
            });
            return;
        }
        let target_pos = match (target, ground) {
            (Some(t), _) => self.pos_of(t),
            (None, Some(g)) => g.at,
            (None, None) => unreachable!(),
        };
        // Accuracy and scatter. A ground shot's accuracy is against the plain
        // distance to the point, and its scatter the land-unit formula unless
        // the point is at sea (`accuracy` flag set by `fight`), which is exact.
        let dist = match target {
            None => vector_dist(target_pos.x - launch.x, target_pos.y - launch.y),
            Some(t) => self.attack_dist(shooter, t),
        };
        let acc = combat::accuracy(p.to_hit, p.attenuate, dist);
        let land_unit = matches!(target, Some(Obj::Unit(_))) && matches!(tp.domain, Domain::Land);
        // **A strafing type's round is exact** (`0x67c33a`): a unit
        // shooter with `unit_flags & 0x400000` (flag `w`, the Fighter
        // line) takes no scatter and no draw, whatever it aims at
        // (`docs/ORDERS.md` §39).
        let strafes = self.strafes(shooter);
        let s = match ground {
            Some(g) if g.sea => 0,
            Some(_) => combat::scatter(&self.tables.tuning, acc, true, false, strafes),
            None => combat::scatter(
                &self.tables.tuning,
                acc,
                land_unit,
                p.has(mask::MISSILE),
                strafes,
            ),
        };
        // A building target shot by a non-siege, **non-tank** unit: aim at the
        // near face. The gate is the shooter type's vslots `+0x10c`
        // (`is_siege`) and `+0x110` (`UnitTypeData::is_tank`, `unit_flags &
        // 0x80000`), both zero (`Ammo::init@0067bbf0`, `0067c1c0`): a Tank
        // aims at the building's centre (`docs/GOLDEN.md` §135).
        let tank = match shooter {
            Obj::Unit(u) => self.units[u]
                .ty
                .is_some_and(|t| self.tables.unit_types[t].cols.flag(uflags::TANK)),
            Obj::Building(_) => false,
        };
        let mut aim = target_pos;
        if matches!(target, Some(Obj::Building(_))) && !p.siege && !tank && s != 0 {
            let back = Angle(angle.0.wrapping_add(Angle::SOUTH.0));
            aim = Pos::new(
                aim.x + crate::movement::sin_component(back, tp.x_size * 0x30),
                aim.y - crate::movement::cos_component(back, tp.x_size * 0x30),
            );
        }
        let sites = if ground.is_some() {
            (SITE_AMMO_GROUND_SCATTER_X, SITE_AMMO_GROUND_SCATTER_Y)
        } else {
            (SITE_AMMO_SCATTER_X, SITE_AMMO_SCATTER_Y)
        };
        let landing = self.scatter_landing(aim, s, sites);
        // **A ground round's angle is the scattered landing's** (item
        // 3979, `docs/AI.md` §411; `Ammo::init@0067bbf0`, `67c747`–`67c75a`):
        // the order's arm takes `+0x2c = find_angle(+0x18 − sx, +0x1c − sy)`
        // once the scatter is in, before the clamp, where the target arm
        // (`67c792`–`67c7af`) takes the bearing to the target unscattered.
        // run1542's MLRS `2/155` on 651.
        let angle = match ground {
            Some(_) => crate::movement::find_angle(landing.x - launch.x, landing.y - launch.y),
            None => angle,
        };
        let landing = self.strafe_walk(shooter, target, landing, node);
        let clamp = |q: Pos, w: &crate::World| {
            Pos::new(
                q.x.clamp(0, w.width() * UNITS_PER_CELL - 1),
                q.y.clamp(0, w.height() * UNITS_PER_CELL - 1),
            )
        };
        let mut landing = clamp(landing, &self.world);
        // **A missile never rolls** (`Ammo::init@0067bbf0:355`–`:360`,
        // the `local_30 == 0` conjunct; item 1625, `docs/AI.md` §174):
        // flag `4` and the `0x4b` over a land unit are a shell's.
        let rolling = land_unit && !strafes && !missile;
        let ez = match target {
            Some(t) => self.aim_z(t, rolling),
            None => self.ground_z(target_pos).max(0),
        };
        // Flight time — never zero.
        let d = i64::from(p.proj_speed) * i64::from(combat::UNIT_MOVE_SPEED);
        let mut spline = None;
        let figure = match shooter {
            Obj::Unit(u) if missile && p.domain != Domain::Land => {
                Some((self.units[u].movement.facing, self.units[u].airframe.pitch))
            }
            _ => None,
        };
        let total_time = if missile {
            // **A missile flies a spline** (item 1625, `docs/AI.md`
            // §174): `Ammo::init`'s `local_30` arm, whose time is the
            // spline's point count. A sea or air shooter's middle control
            // point is five speeds along its angle; a land shooter's is
            // [`combat::missile_flight_time`]'s SEAM.
            //
            // **The lead is along the shooter's figure** (item 3802,
            // `Ammo::init@0067bbf0:704`–`:706`): an air or sea unit's round
            // takes `+0x2c` from `guys[0]->angle` (`GuyData +0x18`), not the
            // bearing to the target, and the middle point is rotated by it.
            // run1469's parked Jet Fighter `2/14` faces 120° and fires on
            // `1/14` to its south; its round swings 285 east in four frames.
            //
            // **And it is rotated as a float vector** (item 3808,
            // [`combat::missile_lead`]): the figure's pitch, then its whole
            // degree, through two quaternions, so a figure parked at 120°
            // leads at 121° — run1471's `2/14`, whose second round on
            // `1/14` re-planned to 10 points against theirs' 11.
            //
            // **A land shooter's middle point is half the distance out, at
            // 45° up** (item 3816, `docs/GOLDEN.md` §152.9;
            // `Ammo::init@0067bbf0`, `67c437`–`67c469`): a shooter whose
            // object type's `domain` (`+0x218`) is 0 — a land unit or a
            // building — takes `r = local_34 / 2`, the distance the
            // accuracy was read at, truncated toward zero, the pitch the
            // `45.0` at `0xb69674`, and the ammo's own angle (`+0x2c`), and
            // joins the air arm's rotation at `67cbaa`. run1469's
            // Anti-Aircraft Missile `1/20` on `2/18` lays
            // (16908.123047, 15295.732422, 1334.793091), 984 out at 219°.
            let lead = Some(if p.domain == Domain::Land {
                combat::Lead {
                    angle,
                    pitch: Single::from_bits(0x4234_0000),
                    r: dist / 2,
                }
            } else {
                let (angle, pitch) = figure.unwrap_or((angle, Single::ZERO));
                combat::Lead {
                    angle,
                    pitch,
                    r: i32::try_from(d * 5).unwrap_or(i32::MAX),
                }
            });
            // The spline is kept (item 3802): `Ammo::inc_time` steps it.
            let path = combat::missile_spline(launch, sz, landing, ez, lead, d);
            spline = Some(path);
            i32::try_from(path.len()).expect("at most SPLINE_MAX")
        } else if let Some(u) = missile_unit {
            // **A missile unit flies `calc_nuke_spline`** (item 3806,
            // `docs/GOLDEN.md` §152.4; `Ammo::init@0067bbf0`, the
            // `local_30 == 0` arm's `+0x1e4 & 0x8000000` test, ahead of
            // the Bomber's and the straight line): its time is the
            // spline's points less one. The Cruise Missile (`is(0x13a)`)
            // takes the arm that follows the ground; it is kept, and
            // `Ammo::inc_time` steps and re-plans it as a guided round's.
            // run1470's `2/19` on `1/20`: 29 points, 28 frames, where the
            // straight line gave 4 and ours killed `1/20` on 648.
            //
            // **The other arm is laid too** (item 3814, `docs/GOLDEN.md`
            // §152.8): a V2's twelve points climb over the launch and drop
            // onto the landing ([`combat::Spline::nuke`]), and the round
            // re-plans at its fourth point as a guided one's — run1471's
            // `0/16` on `2/21` lands on 690 where 120 frames gave 790.
            if self.air_line_is(u, crate::air::airbase::CRUISEMISSILE) {
                let (ctl, n) = combat::cruise_control_points(
                    (launch.x, launch.y, sz),
                    (landing.x, landing.y, ez),
                    |x, y| {
                        let at = Pos::new(x, y);
                        (self.ground_z(at), self.world.tile_mask(at.tile()))
                    },
                );
                let path = combat::Spline::cruise(&ctl[..n]);
                spline = Some(path);
                i32::try_from(path.len()).expect("at most SPLINE_MAX") - 1
            } else {
                let path =
                    combat::Spline::nuke((launch.x, launch.y, sz), (landing.x, landing.y, ez));
                spline = Some(path);
                i32::try_from(path.len()).expect("at most SPLINE_MAX") - 1
            }
        } else if p.siege {
            combat::siege_flight_time(
                self.max_range_of(shooter),
                p.proj_speed,
                combat::UNIT_MOVE_SPEED,
            )
        } else {
            let dx = i64::from(landing.x - launch.x);
            let dy = i64::from(landing.y - launch.y);
            combat::flight_time(dx * dx + dy * dy, d)
        }
        .max(1);
        // The lead: a unit target that is moving has the landing point pushed
        // along its angle for the time of flight (§9.1, §47.4). The
        // magnitude is its **first figure's `avg_speed`**, not the unit's
        // per-frame speed. `Ammo::init@0067bbf0` loads `ecx = T.angle`
        // (`UnitData +0x50`) and `edx = guys.list[0]->avg_speed`
        // (`+0xf4`, then `GuyData +0x84`) at `0067ceb4`-`0067cec7`, and
        // calls `cosx` and `sinx` fastcall on that pair. The decompiler
        // dropped both operands, and §9.1 had filled them in with "the
        // natural reading". A target that has just set off is still
        // ramping its average: run112's `1/7` on 766 walks 24 a frame at
        // `avg_speed` 9, and a lead of 24 put `0/9`'s shot 55 units past
        // where the original's lands.
        //
        // **The gate is the target's order, and the angle is `+0x50`**
        // (§74). `67ce62`-`67ce99`: `UnitData::order_type`, then
        // `is_move@0046f050`, and on a zero `order_type` again and
        // `is_air@0046f000`, whose zero jumps past the lead. A fleeing
        // citizen between two legs has a `FLEE_TO` head and no
        // `movement.dest`, which this crate asked until item 1081, and
        // run373's `0/1` on 5024 was led by the original and missed by
        // this crate. `UnitData +0x50` is `angle`, which is
        // [`crate::Movement::heading`] here, not the figure's facing.
        // **Not a missile's** (`Ammo::init@0067bbf0:798`: `local_30 == 0`
        // beside the bomb and ground tests, item 1625): a spline round is
        // not led.
        if !missile {
            landing = self.lead_landing(target, landing, total_time);
        }
        let _ = frame;
        // **A strafer's round never rolls** (`67c548`..`67c557`, item
        // 1200): the unit-strafer test jumps past `67c633`, where flag `4`
        // and the `0x4b` over a land unit are set. Chapter forty-one's
        // Biplane on 1529, whose round lands behind it, short of `0/9`.
        self.add_ammo(combat::Projectile {
            shooter,
            owner: self.owner_of(shooter),
            target,
            launch,
            landing,
            cur_time: 0,
            total_time,
            accuracy: acc,
            angle: figure.map_or(angle, |f| f.0),
            splash_area: p.splash_area,
            num_guys: 1,
            // `Ammo::init`'s flag `4` (§42.2): not a ground shot — the
            // `ATTACK_GROUND`/`AIR_ATTACK_GROUND` test is `target` being
            // `None` here — and the target a land-domain unit. The third term,
            // "the piece is not a missile", is the ammo flag `8`, `rolling`
            // above (item 1625); a siege shot is a ground shot and so
            // never reaches the question.
            rolling,
            missed: false,
            harmless,
            air: target.is_some() && matches!(tp.domain, Domain::Air),
            sz,
            ez,
            v1z: combat::arc_v1z(sz, ez, total_time),
            slot: 0,
            flame,
            punctures,
            spent: false,
            crash: None,
            spline,
        });
    }
}

/// **The node loop of `Guy::set_all_pivots@005d8bc0`** (`005d8d80`–
/// `005d8f8b`) over plain data, so the sweep drives the body the crate
/// runs: for each of `count` nodes from 4, the bearing from `from` plus
/// the node's `offset(node)` (truncated, per component) to `to`, taken
/// against `facing` in whole degrees and folded to −180..180; the answer
/// is false once any node is past ±45° or outside `range(node)` (wrapped
/// when `lo >= hi`); a node whose range holds the bearing takes
/// `des = bearing − facing` and its bit, and `node_flags`' bit when that
/// is within 15° of the turret. The caller has cleared both flag words.
pub(crate) fn pivot_steps(
    count: usize,
    range: impl Fn(i32) -> (i32, i32),
    offset: impl Fn(i32) -> (i32, i32),
    (from, to): (Pos, Pos),
    facing: Angle,
    t: &mut anim::Turret,
) -> bool {
    let mut can = true;
    for node in 4..4 + count as i32 {
        // `005d8e0a`–`005d8e35`: the target less the unit's point less
        // the node's truncated vector, each component on its own.
        let (vx, vy) = offset(node);
        let bearing = find_angle(to.x - from.x - vx, to.y - from.y - vy);
        let mut deg = crate::movement::angle_to_degrees(Angle(bearing.0.wrapping_sub(facing.0)));
        if deg > 180 {
            deg -= 360;
        }
        can &= (-45..=45).contains(&deg);
        let (lo, hi) = range(node);
        let inside = if lo < hi {
            lo <= deg && deg <= hi
        } else {
            deg >= lo || deg <= hi
        };
        can &= inside;
        if inside && let Some(k) = usize::try_from(node - 4).ok().filter(|&k| k < 4) {
            t.des[k] = bearing.0.wrapping_sub(facing.0);
            t.des_flags |= 1 << k;
            if turret_near(t.des[k], t.angles[k]) {
                t.node_flags |= 1 << k;
            }
        }
    }
    can
}

/// **Within a turret's step of each other**: `(a − b)` as unsigned, folded
/// by `~` past a half turn, below 15° (`0xaaa_aaaa`) — the test
/// `Guy::set_all_pivots` and `Guy::process` both make (`005d8ed7`,
/// `005e0263`).
pub(crate) const fn turret_near(a: i32, b: i32) -> bool {
    let d = a.wrapping_sub(b) as u32;
    let d = if d > 0x8000_0000 { !d } else { d };
    d < 0x0aaa_aaaa
}

/// **`Unit::set_attacking@005ff5b0`'s own bytes** (item 2287): `flags |=
/// 0x80` whatever follows; then, when the victim `who`'s bit of
/// `visible` (`+0x40`, read sign-extended, `1 << who`) is clear, the bit
/// goes in and the answer is whether `who` could **not** already see the
/// unit's half-cell (`WorldData::is_seen`, asked only then) — the caller
/// lights the unit's disc (`vtable[0x164]`) on a yes.
pub(crate) fn attacking_bits(
    flags: &mut u8,
    visible: &mut u8,
    who: crate::Player,
    sees: impl FnOnce() -> bool,
) -> bool {
    *flags |= 0x80;
    let bit = Sim::who_bit(who);
    if *visible & bit != 0 {
        return false;
    }
    let reveal = !sees();
    *visible |= bit;
    reveal
}

/// **`Unit::set_attack@005fce70`**'s steps, the callees as closures
/// ([`Sim::set_attack`]; `sweep::unit_set_attack` runs it against the
/// emulated original).
///
/// - `aim(g, target)` for every figure `0 .. guy_mark` (`GuyData +0x8e`,
///   `+0x9f`; `None` is the `(−1, −1)`), `005fce81`–`005fceb9`;
/// - then `restricted` (the type's `get_gpiece`, `GraphicPieces::get_type`
///   and its `pivot_restrictions` row, `005fcebb`–`005fcef1`), `ranged`
///   (`UnitType +0x1fc`, `005fcefa`), a target at all (`o`, `who` ≥ 0,
///   `005fcf07`), and `live` (`is_active && is_on_map || +0x1c`,
///   `005fcf3d`–`005fcf63`), each short-circuiting to 0;
/// - then `pivots(g, target)` for every figure, the answer the last one's
///   (`005fcf80`–`005fcf9c`), whole (`eax`); no figure answers 0.
#[allow(clippy::too_many_arguments)]
pub(crate) fn set_attack_steps<C, T: Copy, R: Default>(
    cx: &mut C,
    guy_mark: usize,
    target: Option<T>,
    mut aim: impl FnMut(&mut C, usize, Option<T>),
    restricted: impl FnOnce(&mut C) -> bool,
    ranged: impl FnOnce(&mut C) -> bool,
    live: impl FnOnce(&mut C, T) -> bool,
    mut pivots: impl FnMut(&mut C, usize, T) -> R,
) -> R {
    for g in 0..guy_mark {
        aim(cx, g, target);
    }
    if !restricted(cx) || !ranged(cx) {
        return R::default();
    }
    let Some(t) = target else {
        return R::default();
    };
    if !live(cx, t) {
        return R::default();
    }
    let mut can = R::default();
    for g in 0..guy_mark {
        can = pivots(cx, g, t);
    }
    can
}

/// [`crate::combat::object_attack_view::ObjectAttackView`] over the crate's [`Sim`]: the object's type record and
/// owner. A building answers false on the unit slots (`+0x18`, `+0xd0`, `+0xcc`, `+0xc8`).
pub(super) struct SimObjectAttack<'a> {
    pub(super) sim: &'a Sim,
    pub(super) o: Obj,
}

impl SimObjectAttack<'_> {
    pub(super) fn rec(&self) -> Option<usize> {
        match self.o {
            Obj::Unit(u) => self.sim.units[u].ty,
            Obj::Building(_) => None,
        }
    }
    pub(super) fn owner(&self) -> usize {
        self.sim.owner_of(self.o) as usize
    }
}

impl crate::combat::object_attack_view::ObjectAttackView for SimObjectAttack<'_> {
    fn type_attack(&mut self) -> i32 {
        self.sim.profile(self.o).attack
    }
    fn is_caravan(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.tables.unit_types[r]
                .cols
                .flag2(crate::ai::ai_load::uflags2::FLAG2_CARAVAN)
        })
    }
    fn has_caravan_preq(&mut self) -> bool {
        // A tree that does not name the bonus's prerequisite holds it for nobody.
        match (
            self.sim.tables.tech_tree.roles.caravan_attack_preq,
            self.sim.tech.get(self.owner()),
        ) {
            (Some(t), Some(p)) => self
                .sim
                .tables
                .tech_tree
                .has_tech(&self.sim.tables.setup, p, t),
            _ => false,
        }
    }
    fn caravan_attack_bonus(&mut self) -> i32 {
        self.sim.tables.tuning.caravan_attack_bonus
    }
    fn has_dutch_bonus(&mut self) -> bool {
        self.sim.tech.get(self.owner()).is_some_and(|p| {
            self.sim.tables.tech_tree.has_tribe_bonus(
                &self.sim.tables.setup,
                p,
                crate::ai::tribe::DUTCH,
            )
        })
    }
    fn dutch_attack_bonus(&mut self) -> i32 {
        self.sim.tables.tuning.dutch_attack_bonus
    }
    fn is_unit(&mut self) -> bool {
        matches!(self.o, Obj::Unit(_))
    }
    fn type_index(&mut self) -> i32 {
        self.rec()
            .map_or(0, |r| self.sim.tables.unit_types[r].type_index)
    }
    fn is_supply(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.tables.unit_types[r]
                .cols
                .flag2(crate::ai::ai_load::uflags2::SUPPLY_OR_HERO)
        })
    }
    fn gov_hero_slot(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.tables.unit_types[r]
                .tree
                .and_then(|i| self.sim.tables.tech_tree.types.get(i))
                .and_then(|d| d.kind.unit())
                .is_some_and(|u| u.patriot)
        })
    }
    fn ages(&mut self) -> i32 {
        self.sim.tech.get(self.owner()).map_or(0, |p| p.ages)
    }
}
