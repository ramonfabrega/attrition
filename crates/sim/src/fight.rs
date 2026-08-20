//! Combat as the simulation runs it: the attack step (`Unit::fight`), the
//! delivery (`Object::do_damage`/`take_damage`), the automatic choice of
//! target, buildings that shoot, and ammo in flight. `docs/COMBAT.md` is the
//! specification; the arithmetic lives in [`crate::combat`] and this file is
//! the glue between it and [`Sim`]'s objects.
//!
//! What it leaves out, each said where it applies: the move-to-attack path
//! approaches the target in a straight line (no `find_attack_pos`), a squad
//! is a set of figures that share a captain index and nothing else, and every
//! nation, wonder and patriot layer arrives through [`combat::Modifiers`].

use crate::attrition::Domain;
use crate::combat::{self, Obj, Profile, Side, Sixteenths, Stance, Taken, mask, role};
use crate::movement::{Angle, find_angle};
use crate::world::{Pos, UNITS_PER_CELL, UNITS_PER_TILE, vector_dist};
use crate::{Player, Sim};

/// A unit that is not a combatant for the search: no type, no attack.
fn no_profile() -> Profile {
    Profile::default()
}

impl Sim {
    // ------------------------------------------------------------------
    // Reading an object
    // ------------------------------------------------------------------

    /// The combat profile of an object.
    pub fn profile(&self, o: Obj) -> Profile {
        match o {
            Obj::Unit(i) => self.units[i]
                .ty
                .map_or_else(no_profile, |t| self.unit_types[t].combat),
            Obj::Building(b) => self.buildings[b].combat.unwrap_or_default(),
        }
    }

    fn owner_of(&self, o: Obj) -> Player {
        match o {
            Obj::Unit(i) => self.units[i].owner,
            Obj::Building(b) => self.buildings[b].owner,
        }
    }

    fn pos_of(&self, o: Obj) -> Pos {
        match o {
            Obj::Unit(i) => self.units[i].pos,
            Obj::Building(b) => self.buildings[b].pos,
        }
    }

    /// `flags & 1` and, for a unit, `is_on_map`.
    fn active(&self, o: Obj) -> bool {
        match o {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive() && u.on_map),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.combat.is_some() && b.health > 0),
        }
    }

    /// `attack()` — the type's, plus the player's flat modifier (§4.1).
    pub fn attack_of(&self, o: Obj) -> i32 {
        let base = self.profile(o).attack;
        if base == 0 {
            return 0;
        }
        base + self.mods[self.owner_of(o) as usize].attack
    }

    /// `armor()` (§4.2).
    pub fn armor_of(&self, o: Obj) -> i32 {
        self.profile(o).armor + self.mods[self.owner_of(o) as usize].armor
    }

    /// `max_range()` (§4.4): zero stays zero.
    pub fn max_range_of(&self, o: Obj) -> i32 {
        let p = self.profile(o);
        if p.max_range == 0 {
            return 0;
        }
        p.max_range + self.mods[self.owner_of(o) as usize].range
    }

    fn hits_left(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(i) => self.units[i].health.max(0),
            Obj::Building(b) => self.buildings[b].health.max(0),
        }
    }

    /// The side of the damage formula an object presents as a **target**.
    fn target_side(&self, o: Obj, attacker: Obj) -> Side {
        match o {
            Obj::Unit(i) => {
                let u = &self.units[i];
                Side {
                    unit: true,
                    packed: u.combat.packed,
                    moving: u.movement.dest.is_some(),
                    attacks: self.attack_of(o) != 0,
                    entrenched: u.combat.entrenched,
                    facing: u.movement.facing,
                    trench_facing: u.movement.facing,
                    z: 0,
                    damage_frame: u.combat.damage_frame,
                    damage_o: u.combat.damage_o,
                    captain: u.combat.captain,
                    decoy: false,
                    rocky: false,
                    tile_owned_by_attacker: self
                        .world
                        .owner_at(u.pos)
                        .player()
                        .is_some_and(|p| p == self.owner_of(attacker)),
                    ..Side::default()
                }
            }
            Obj::Building(b) => {
                let bd = &self.buildings[b];
                Side {
                    unit: false,
                    building: true,
                    build_proper: true,
                    under_construction: !bd.active,
                    attacks: self.attack_of(o) != 0,
                    tile_owned_by_attacker: self
                        .world
                        .owner_at(bd.pos)
                        .player()
                        .is_some_and(|p| p == self.owner_of(attacker)),
                    ..Side::default()
                }
            }
        }
    }

    /// The side an object presents as an **attacker**.
    fn attacker_side(&self, o: Obj) -> Side {
        match o {
            Obj::Unit(i) => Side {
                unit: true,
                captain: self.units[i].combat.captain,
                z: 0,
                ..Side::default()
            },
            Obj::Building(_) => Side {
                building: true,
                build_proper: true,
                ..Side::default()
            },
        }
    }

    // ------------------------------------------------------------------
    // Distance and validity
    // ------------------------------------------------------------------

    /// `ObjectData::attack_dist(o, who, x, y)` from `at` (§13.1).
    pub fn attack_dist_from(&self, attacker: Obj, at: Pos, target: Obj) -> i32 {
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let plane = matches!(ap.domain, Domain::Air) && !ap.has(mask::MISSILE);
        combat::attack_dist(
            at,
            self.pos_of(target),
            combat::extent(&ap, matches!(attacker, Obj::Building(_))),
            combat::extent(&tp, matches!(target, Obj::Building(_))),
            plane,
        )
    }

    pub fn attack_dist(&self, attacker: Obj, target: Obj) -> i32 {
        self.attack_dist_from(attacker, self.pos_of(attacker), target)
    }

    /// `ObjectData::is_in_range(o, who, x, y, …)` (§13.2). The unseen-tile
    /// test is absent: the simulation has no fog.
    pub fn is_in_range(&self, attacker: Obj, target: Obj) -> bool {
        if !self.active(target) {
            return false;
        }
        let ap = self.profile(attacker);
        let at = self.pos_of(attacker);
        // `attack_dist` is called at the quarter-tile centre of the position.
        let centre = Pos::new(
            at.x.div_euclid(48) * 48 + 0x18,
            at.y.div_euclid(48) * 48 + 0x18,
        );
        let d = self.attack_dist_from(attacker, centre, target);
        let big = ap.big_radius + self.profile(target).big_radius;
        combat::in_range(
            d,
            self.max_range_of(attacker),
            ap.min_range,
            false,
            big,
            false,
        )
    }

    /// `ObjectData::valid_target_const` + `Object::valid_target` (§12.1), as
    /// far as the simulation's state reaches: not mine, at war, active, on the
    /// map; the air ladder reduced to "air targets need a ranged attacker
    /// and the two AIR/ANTI_AIR rules"; no fog, no capture.
    pub fn valid_target(&self, attacker: Obj, target: Obj) -> bool {
        if attacker == target {
            return false;
        }
        let (me, them) = (self.owner_of(attacker), self.owner_of(target));
        if me == them {
            return false;
        }
        if !self.at_war[me as usize][them as usize] && !self.at_war[them as usize][me as usize] {
            return false;
        }
        if !self.active(target) {
            return false;
        }
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        if matches!(tp.domain, Domain::Air) {
            if self.max_range_of(attacker) == 0 {
                return false;
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

    // ------------------------------------------------------------------
    // Orders
    // ------------------------------------------------------------------

    /// Gives a unit an attack order on a target — `add_attack_order` with
    /// `mandatory` set, which is what a player's click does.
    pub fn order_attack(&mut self, unit: usize, target: Obj) {
        self.retarget(Obj::Unit(unit), Some(target), true);
    }

    /// Tells a building to attack something — `Build::add_attack_order`.
    pub fn order_building_attack(&mut self, building: usize, target: Obj) {
        let b = &mut self.buildings[building];
        b.target = Some(target);
        b.ordered = true;
        self.bump_targeted(target, 1);
    }

    pub fn set_stance(&mut self, unit: usize, stance: Stance) {
        self.units[unit].combat.stance = stance;
    }

    fn bump_targeted(&mut self, o: Obj, by: i32) {
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
    fn retarget(&mut self, attacker: Obj, target: Option<Obj>, mandatory: bool) {
        let Obj::Unit(i) = attacker else { return };
        if let Some(t) = target {
            self.bump_targeted(t, 1);
        }
        let u = &mut self.units[i];
        u.combat.target = target;
        u.combat.mandatory = mandatory && target.is_some();
    }

    // ------------------------------------------------------------------
    // The attack step
    // ------------------------------------------------------------------

    /// One unit's combat for one frame: `Unit::do_attack` → `Unit::fight` for
    /// a unit with a target, `Unit::think` for one without (§8.1, §8.2, §12.4).
    pub(crate) fn process_unit_combat(&mut self, i: usize, frame: i64) {
        let me = Obj::Unit(i);
        if self.attack_of(me) == 0 {
            return;
        }
        let state = self.units[i].combat;
        let Some(target) = state.target else {
            self.think_attack(i, frame);
            return;
        };
        if !self.valid_target(me, target) {
            // An invalid target: `find_new_target`, which is the idle search
            // with the order dropped.
            self.retarget(me, None, false);
            self.think_attack(i, frame);
            return;
        }
        if state.stance == Stance::HoldFire {
            return;
        }
        // The one-in-five retarget roll (§12.4): on a frame the unit is not
        // recharging, with a non-mandatory order on a unit that is not a
        // combat unit, one draw; four times in five it looks again.
        if !state.mandatory
            && state.recharging == 0
            && let Obj::Unit(t) = target
            && !self.profile(Obj::Unit(t)).combat_role
        {
            let roll = self.rng.roll();
            if roll % 5 != 0 {
                let found = self.find_melee_target(i, -1);
                if let Some(f) = found
                    && f != target
                {
                    self.retarget(me, Some(f), false);
                    return;
                }
            }
        }
        if self.is_in_range(me, target) {
            // Standing still to fight: the unit step is not taken.
            self.units[i].movement.dest = None;
            if self.units[i].combat.recharging == 0 {
                self.fight(i, target, frame);
            }
        } else if state.stance != Stance::StandGround {
            // Close the distance. `find_attack_pos` is a straight line here.
            let dest = self.pos_of(target);
            self.units[i].movement.dest = Some(dest);
        }
    }

    /// The strike itself — `Unit::fight` from step 2 on (§8.2).
    fn fight(&mut self, i: usize, target: Obj, frame: i64) {
        let me = Obj::Unit(i);
        let p = self.profile(me);
        // Facing: toward the target.
        let (from, to) = (self.units[i].pos, self.pos_of(target));
        let angle = find_angle(to.x - from.x, to.y - from.y);
        self.units[i].movement.set_facing(angle);
        if self.max_range_of(me) == 0 {
            // Melee lands now, once per figure of this `UnitData` — one here.
            if p.fires() {
                self.do_damage(me, target, angle, false, 0x100, false, false, frame);
            }
        } else {
            self.fire_ammo(me, target, angle, frame);
        }
        // Reload.
        let unit = &self.units[i];
        let out = if p.siege {
            let owner = unit.owner;
            let at = unit.pos;
            !(self.supplied_at(owner, at)
                || matches!(self.world.owner_at(at), crate::Owner::Player(o) if o == owner))
        } else {
            false
        };
        let r = combat::recharge(p.recharge, out, p.is(role::BOMBARD));
        self.units[i].combat.recharging = r as u8;
    }

    /// `Object::fire_ammo` for a unit (§9.1): one `Ammo` per figure, here one.
    fn fire_ammo(&mut self, shooter: Obj, target: Obj, angle: Angle, frame: i64) {
        let p = self.profile(shooter);
        let launch = self.pos_of(shooter);
        let tp = self.profile(target);
        let target_pos = self.pos_of(target);
        // Accuracy and scatter.
        let acc = combat::accuracy(p.to_hit, p.attenuate, self.attack_dist(shooter, target));
        let land_unit = matches!(target, Obj::Unit(_)) && matches!(tp.domain, Domain::Land);
        let s = combat::scatter(&self.tuning, acc, land_unit, p.has(mask::MISSILE), false);
        // A building target shot by a non-siege unit: aim at the near face.
        let mut aim = target_pos;
        if matches!(target, Obj::Building(_)) && !p.siege && s != 0 {
            let back = Angle(angle.0.wrapping_add(Angle::SOUTH.0));
            aim = Pos::new(
                aim.x + crate::movement::sin_component(back, tp.x_size * 0x30),
                aim.y - crate::movement::cos_component(back, tp.x_size * 0x30),
            );
        }
        let landing = combat::scatter_point(&mut self.rng, aim, s);
        let landing = Pos::new(
            landing.x.clamp(0, self.world.width() * UNITS_PER_CELL - 1),
            landing.y.clamp(0, self.world.height() * UNITS_PER_CELL - 1),
        );
        // Flight time.
        let d = i64::from(p.proj_speed) * i64::from(combat::UNIT_MOVE_SPEED);
        let total_time = if p.siege {
            combat::siege_flight_time(
                self.max_range_of(shooter),
                p.proj_speed,
                combat::UNIT_MOVE_SPEED,
            )
        } else {
            let dx = i64::from(landing.x - launch.x);
            let dy = i64::from(landing.y - launch.y);
            combat::flight_time(dx * dx + dy * dy, d)
        };
        let _ = frame;
        self.projectiles.push(combat::Projectile {
            shooter,
            owner: self.owner_of(shooter),
            target: Some(target),
            launch,
            landing,
            cur_time: 0,
            total_time,
            accuracy: acc,
            angle,
            splash_area: p.splash_area,
            num_guys: 1,
            air: matches!(tp.domain, Domain::Air),
        });
    }

    // ------------------------------------------------------------------
    // Delivery
    // ------------------------------------------------------------------

    /// `Object::do_damage` (§7.1): computes, scales and delivers one hit from
    /// `attacker` to `target`. `count` is 8.8; `ammo` says a projectile
    /// delivered it; `splash` marks a fringe; `quiet` suppresses the
    /// target's opportunity response. Returns what `take_damage` did.
    #[allow(clippy::too_many_arguments)]
    pub fn do_damage(
        &mut self,
        attacker: Obj,
        target: Obj,
        angle: Angle,
        ammo: bool,
        count: i32,
        splash: bool,
        quiet: bool,
        frame: i64,
    ) -> Option<Taken> {
        if count < 1 || !self.active(target) {
            return None;
        }
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let at = self.attacker_side(attacker);
        let tt = self.target_side(target, attacker);
        let owner = self.owner_of(attacker) as usize;
        let pct = match (attacker, target) {
            (Obj::Unit(a), Obj::Unit(b)) => match (self.units[a].ty, self.units[b].ty) {
                (Some(x), Some(y)) => self.table.pct(x, y),
                _ => 100,
            },
            _ => 100,
        };
        let mut dmg = combat::get_damage(
            &self.tuning,
            &ap,
            at,
            &tp,
            tt,
            self.attack_of(attacker),
            self.armor_of(target),
            pct,
            angle,
            splash,
            frame,
            &self.mods[owner],
        );
        // Step 2: the target's captain reacts, and the overkill record.
        if let Obj::Unit(t) = target
            && !quiet
        {
            self.target_opportunity(t, attacker, frame);
            let u = &mut self.units[t];
            if u.combat.damage_frame == 0
                || frame - u.combat.damage_frame >= i64::from(self.tuning.overkill_frames)
            {
                u.combat.damage_frame = frame;
                u.combat.damage_o = at.captain;
            }
            let _ = &mut dmg;
        }
        // Step 5: scale.
        let dealt = combat::scale(
            dmg,
            count,
            matches!(attacker, Obj::Unit(_)),
            ammo,
            ap.ammo_per_att,
            ap.uber_size,
        );
        // Step 7: take.
        let taken = self.take_damage(target, dealt, attacker, frame);
        let killed = matches!(taken, Taken::Died { .. });
        self.hits.push(combat::Hit {
            frame,
            attacker,
            target,
            damage: dmg,
            dealt,
            splash,
            killed,
        });
        // Step 9: a hit that did not kill breaks entrenchment.
        if !killed && let Obj::Unit(t) = target {
            self.units[t].combat.entrenched = false;
        }
        Some(taken)
    }

    /// `Object::take_damage` (§7.2) on a unit figure or a building.
    fn take_damage(&mut self, target: Obj, hit: Sixteenths, _by: Obj, _frame: i64) -> Taken {
        match target {
            Obj::Unit(i) => {
                let u = &self.units[i];
                // `health` is what is left of the figure's share; the share
                // itself is `health + damage taken`, which is what the
                // threshold compares against.
                let share = u.health;
                let (taken, _, frac) = combat::take(0, u.damage_frac, share, hit);
                let lost = match taken {
                    Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
                };
                let u = &mut self.units[i];
                u.damage_frac = frac;
                u.health -= lost;
                if matches!(taken, Taken::Died { .. }) {
                    u.health = u.health.min(0);
                    self.close_supply(i);
                    self.forget(Obj::Unit(i));
                }
                taken
            }
            Obj::Building(b) => {
                let bd = &self.buildings[b];
                let share = bd.health;
                let (taken, _, frac) = combat::take(0, bd.damage_frac, share, hit);
                let lost = match taken {
                    Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
                };
                let bd = &mut self.buildings[b];
                bd.damage_frac = frac;
                bd.health -= lost;
                if matches!(taken, Taken::Died { .. }) {
                    bd.health = bd.health.min(0);
                    self.forget(Obj::Building(b));
                }
                taken
            }
        }
    }

    /// A dead object is dropped from every target slot and from ammo in
    /// flight — what `close` does through `hold_frames` and `valid_target`.
    fn forget(&mut self, dead: Obj) {
        for u in &mut self.units {
            if u.combat.target == Some(dead) {
                u.combat.target = None;
                u.combat.mandatory = false;
            }
        }
        for b in &mut self.buildings {
            if b.target == Some(dead) {
                b.target = None;
                b.ordered = false;
            }
        }
        for p in &mut self.projectiles {
            if p.target == Some(dead) {
                p.target = None;
            }
        }
    }

    /// `Unit::target_opportunity` (§12.4), reduced to the retaliation: a
    /// combat unit with no target, not holding fire, attacks whoever hit it;
    /// a non-combatant does nothing (it would flee).
    fn target_opportunity(&mut self, victim: usize, attacker: Obj, _frame: i64) {
        let me = Obj::Unit(victim);
        let (a, b) = (
            self.units[victim].owner as usize,
            self.owner_of(attacker) as usize,
        );
        if !self.at_war[a][b] && !self.at_war[b][a] {
            return;
        }
        if !self.valid_target(me, attacker) {
            return;
        }
        let st = self.units[victim].combat;
        if st.stance == Stance::HoldFire || self.attack_of(me) == 0 {
            return;
        }
        if st.target.is_some() {
            return;
        }
        let p = self.profile(me);
        if p.has(mask::CIVILIAN) && self.max_range_of(me) == 0 {
            return;
        }
        self.retarget(me, Some(attacker), false);
    }

    // ------------------------------------------------------------------
    // The search
    // ------------------------------------------------------------------

    /// `Unit::think` → `think_attack` → `find_melee_target(−1)` on the idle
    /// cadence (§12.4): every 32nd frame by index.
    fn think_attack(&mut self, i: usize, frame: i64) {
        let u = &self.units[i];
        if (i64::from(u.index) + frame) & 0x1f != 0 {
            return;
        }
        if u.combat.stance == Stance::HoldFire {
            return;
        }
        if let Some(t) = self.find_melee_target(i, -1) {
            self.retarget(Obj::Unit(i), Some(t), false);
        }
    }

    /// `Unit::find_melee_target(range, …)` (§12.4): the idle radius for
    /// `range == −1`, then `find_nearby_target`.
    pub fn find_melee_target(&mut self, i: usize, range: i32) -> Option<Obj> {
        let me = Obj::Unit(i);
        let st = self.units[i].combat;
        if st.stance == Stance::HoldFire {
            return None;
        }
        let r = self.max_range_of(me);
        let t = &self.tuning;
        let radius = if range == -1 {
            if st.stance == Stance::Defensive {
                (if r == 0 { 0x120 } else { r * 0xc0 }).max(t.unit_defensive_respond_range * 0xc0)
            } else {
                let mut d = (r + 1) * 0xc0;
                if st.stance == Stance::Aggressive {
                    d += 0x180;
                }
                if r == 0 {
                    d = 0x120
                        + if st.stance == Stance::Aggressive {
                            0x180
                        } else {
                            0
                        };
                }
                d.max(t.unit_respond_range * 0xc0)
            }
        } else {
            range
        };
        self.find_nearby_target(me, radius)
    }

    /// `Object::find_nearby_target(max_dist, …)` (§12.2): the ring scan, the
    /// range gate, the distance shaping, the ranking, and the `targeted`
    /// bump on the winner. `max_dist == 0` is unlimited.
    pub fn find_nearby_target(&mut self, attacker: Obj, max_dist: i32) -> Option<Obj> {
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
        let anything = match attacker {
            Obj::Unit(i) => {
                let c = self.units[i].combat;
                c.stance == Stance::StandGround || c.entrenched || (ap.packs && !c.packed)
            }
            Obj::Building(_) => false,
        };
        let minr = ap.min_range * 0xc0;
        let maxr = self.max_range_of(attacker) * 0xc0;
        let centre = at.cell();

        // Every object, bucketed by cell, so the rings can be walked in the
        // original's order: ring by ring, `dx` then `dy` ascending, and the
        // objects on a cell in index order.
        let candidates: Vec<Obj> = (0..self.units.len())
            .map(Obj::Unit)
            .chain((0..self.buildings.len()).map(Obj::Building))
            .filter(|&o| o != attacker && self.active(o))
            .collect();
        let mut best: Option<(i32, Obj)> = None;
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
                    for &o in &candidates {
                        if self.pos_of(o).cell() != cell {
                            continue;
                        }
                        if !self.valid_target(attacker, o) {
                            continue;
                        }
                        let mut dist = self.attack_dist(attacker, o);
                        if max_dist > 0 && dist > max_dist {
                            continue;
                        }
                        let is_unit = matches!(o, Obj::Unit(_));
                        // The range gate: a unit takes anything, a building
                        // needs the target in range or worth waiting for.
                        let in_range = if anything
                            || matches!(attacker, Obj::Unit(_))
                            || self.is_in_range(attacker, o)
                        {
                            true
                        } else if is_unit || self.attack_of(o) != 0 {
                            false
                        } else {
                            continue;
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
                        let value = self.compare_target(attacker, o, in_range);
                        let mut score = value / (dist / 0xc0 + 1);
                        if score == 0 && value != 0 {
                            score = 1;
                        }
                        if best.is_none_or(|(b, _)| score > b) {
                            best = Some((score, o));
                        }
                        if is_unit {
                            unit_candidates += 1;
                            if unit_candidates > 10 && best.is_some() {
                                break 'rings;
                            }
                        }
                    }
                }
            }
        }
        best.map(|(_, o)| o)
    }

    /// `Object::compare_target(o, who, in_range, ai)` (§12.3), the skeleton the
    /// simulation can evaluate, for a human owner.
    pub fn compare_target(&self, attacker: Obj, target: Obj, in_range: bool) -> i32 {
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let is_build = matches!(target, Obj::Building(_));
        let aa = is_build && tp.has(mask::ANTI_AIR);
        let mut v: i64 = 4 * i64::from(tp.cost);
        if is_build {
            // Not raiding, human: nothing of the class multipliers applies —
            // they are gated on a non-human owner. Kept as the rule for an AI
            // owner, which the simulation does not yet distinguish.
            let _ = tp.build_class;
        }
        let t_attack = self.attack_of(target);
        let left = self.hits_left(target);
        if t_attack != 0 && left != 0 && !aa {
            v = v * i64::from(t_attack) * 100 / i64::from(left);
            if is_build {
                v *= 10;
            }
        }
        if matches!(attacker, Obj::Building(b) if self.buildings[b].target == Some(target)) {
            let bd = &self.buildings[match attacker {
                Obj::Building(b) => b,
                _ => 0,
            }];
            let arrows = combat::garrison_arrows(
                self.attack_of(attacker),
                ap.base_arrows,
                ap.most_shots,
                bd.garrison_attack,
            );
            if arrows < 2 {
                v *= 2;
            } else {
                v /= 2;
            }
        }
        if matches!(attacker, Obj::Building(_))
            && let Obj::Unit(u) = target
        {
            if self.units[u].damage_frac != 0 || self.units[u].health < self.hits_of(target) {
                v = v * 3 / 2;
            }
            if self.units[u].movement.dest.is_some() {
                v /= 4;
            }
            if tp.is(role::SUPPLY) {
                v *= 5000;
            }
        }
        let dmg = {
            let at = self.attacker_side(attacker);
            let tt = self.target_side(target, attacker);
            let pct = match (attacker, target) {
                (Obj::Unit(a), Obj::Unit(b)) => match (self.units[a].ty, self.units[b].ty) {
                    (Some(x), Some(y)) => self.table.pct(x, y),
                    _ => 100,
                },
                _ => 100,
            };
            combat::get_damage(
                &self.tuning,
                &ap,
                at,
                &tp,
                tt,
                self.attack_of(attacker),
                self.armor_of(target),
                pct,
                Angle::NORTH,
                false,
                self.frame,
                &self.mods[self.owner_of(attacker) as usize],
            )
        };
        v *= i64::from(dmg);
        if is_build {
            // Armed buildings: a human owner gets ×5; siege adds 100,000.
            let armed = t_attack != 0 && !(aa && !matches!(ap.domain, Domain::Air));
            if armed {
                v *= 5;
                if ap.has(mask::SIEGE) {
                    v += 100_000;
                }
            }
        } else {
            if tp.combat_role {
                v *= 20;
            }
            if tp.combat_role {
                v += 1_000_000;
            } else if tp.is(role::SUPPLY) {
                v += if matches!(attacker, Obj::Building(_)) {
                    4_000_000
                } else {
                    100_000
                };
            } else {
                v += 100_000;
            }
            if matches!(ap.domain, Domain::Land)
                && ap.has(mask::SIEGE)
                && !matches!(tp.domain, Domain::Sea)
                && !tp.has(mask::SIEGE)
            {
                v /= 10_000;
            }
        }
        if v < 0 {
            v = 9_999_999;
        }
        if !in_range && !self.is_in_range(attacker, target) {
            v /= 5;
        }
        v = (v + 99) / 100;
        if v > 100_000 {
            v = 100_000 + (v - 100_000) / 5;
        }
        if v < 15 {
            v = 15;
        }
        if matches!(tp.domain, Domain::Air) && !ap.has(mask::ANTI_AIR) {
            v = 2;
        }
        v.min(i64::from(i32::MAX)) as i32
    }

    fn hits_of(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(i) => self.units[i]
                .ty
                .map_or(self.units[i].health, |t| self.unit_types[t].hits),
            Obj::Building(b) => self.buildings[b].hits,
        }
    }

    // ------------------------------------------------------------------
    // Buildings
    // ------------------------------------------------------------------

    /// `Build::process` → `Build::do_attack` for a building that shoots (§8.6).
    pub(crate) fn process_building_combat(&mut self, b: usize, frame: i64) {
        let me = Obj::Building(b);
        if !self.active(me) || self.attack_of(me) == 0 {
            return;
        }
        let bd = &self.buildings[b];
        let phase = (frame + b as i64) & 0x1f;
        // Without a target, `do_attack` runs every 32nd frame.
        if bd.target.is_none() && phase != 0 {
            return;
        }
        if bd.recharging > 0 {
            self.buildings[b].recharging -= 1;
            return;
        }
        let p = self.profile(me);
        let arrows = combat::garrison_arrows(
            self.attack_of(me),
            p.base_arrows,
            p.most_shots,
            bd.garrison_attack,
        );
        if arrows == 0 {
            return;
        }
        // Find or re-find a target.
        let needs = match bd.target {
            None => true,
            Some(t) => {
                !self.valid_target(me, t)
                    || (!bd.ordered
                        && ((frame + b as i64 + 14) & 0x1f) == 0
                        && matches!(t, Obj::Unit(u) if self.units[u].movement.dest.is_none()
                            && !self.profile(t).combat_role))
            }
        };
        if needs && !bd.ordered {
            let radius = (p.x_size.max(p.y_size) + 2 * self.max_range_of(me)) * 0x60;
            let found = self.find_nearby_target(me, radius);
            if let Some(t) = found {
                self.bump_targeted(t, 1);
            }
            self.buildings[b].target = found;
        } else if needs {
            self.buildings[b].target = None;
            self.buildings[b].ordered = false;
        }
        let Some(target) = self.buildings[b].target else {
            return;
        };
        if !self.is_in_range(me, target) {
            return;
        }
        // Fire: `ammo_per_att` ammo at random points in the footprint.
        let centre = self.pos_of(me);
        let tp = self.profile(target);
        let target_pos = self.pos_of(target);
        let acc = combat::accuracy(p.to_hit, p.attenuate, self.attack_dist(me, target));
        let land_unit = matches!(target, Obj::Unit(_)) && matches!(tp.domain, Domain::Land);
        let s = combat::scatter(&self.tuning, acc, land_unit, false, false);
        let xs = p.x_size * 0x60;
        let ys = p.y_size * 0x60;
        for _ in 0..p.ammo_per_att.max(1) {
            let ox = if xs - 1 < 1 { 0 } else { self.rng.roll() % xs };
            let oy = if ys - 1 < 1 { 0 } else { self.rng.roll() % ys };
            let launch = Pos::new(centre.x - xs / 2 + ox, centre.y - ys / 2 + oy);
            let angle = find_angle(target_pos.x - launch.x, target_pos.y - launch.y);
            let landing = combat::scatter_point(&mut self.rng, target_pos, s);
            let d = i64::from(p.proj_speed) * i64::from(combat::UNIT_MOVE_SPEED);
            let dx = i64::from(landing.x - launch.x);
            let dy = i64::from(landing.y - launch.y);
            let total_time = combat::flight_time(dx * dx + dy * dy, d);
            self.projectiles.push(combat::Projectile {
                shooter: me,
                owner: self.buildings[b].owner,
                target: Some(target),
                launch,
                landing,
                cur_time: 0,
                total_time,
                accuracy: acc,
                angle,
                splash_area: p.splash_area,
                num_guys: 0,
                air: matches!(tp.domain, Domain::Air),
            });
        }
        self.buildings[b].recharging = p.recharge / arrows;
    }

    // ------------------------------------------------------------------
    // Ammo
    // ------------------------------------------------------------------

    /// `Ammo::inc_time` for every live projectile (§9.2, §9.3).
    pub(crate) fn process_projectiles(&mut self, frame: i64) {
        let mut i = 0;
        while i < self.projectiles.len() {
            self.projectiles[i].cur_time += 1;
            let p = self.projectiles[i];
            if p.cur_time < p.total_time {
                i += 1;
                continue;
            }
            self.projectiles.swap_remove(i);
            self.land(p, frame);
        }
    }

    /// `Ammo::do_damage` (§9.3): the hit test at the landing point, then
    /// `Object::do_damage` on what was hit — the target, or whatever stood
    /// there instead — and the splash around it.
    fn land(&mut self, p: combat::Projectile, frame: i64) {
        let mut target = p.target.filter(|&t| self.active(t));
        // `hit_target`.
        let hit = target.is_some_and(|t| match t {
            Obj::Unit(u) => combat::hits_unit(
                p.landing,
                self.units[u].pos,
                self.profile(t).target_size,
                p.accuracy,
            ),
            Obj::Building(b) => {
                let tp = self.profile(t);
                combat::hits_building(p.landing, self.buildings[b].pos, tp.x_size, tp.y_size)
            }
        });
        if !hit {
            target = self.check_hit(&p);
        }
        if p.splash_area == 0 {
            let Some(t) = target else {
                // A shot into the ground: two draws for where it punctures.
                let _ = self.rng.roll();
                let _ = self.rng.roll();
                return;
            };
            if t == p.shooter {
                return;
            }
            let angle = {
                let to = self.pos_of(t);
                find_angle(to.x - p.launch.x, to.y - p.launch.y)
            };
            self.do_damage(p.shooter, t, angle, true, 0x100, false, false, frame);
            return;
        }
        // Splash: everything within the radius, the intended target at full
        // count and everything else as a fringe.
        let radius = (p.splash_area / 4).min(10);
        let landing_cell = p.landing.cell();
        let candidates: Vec<Obj> = (0..self.units.len())
            .map(Obj::Unit)
            .chain((0..self.buildings.len()).map(Obj::Building))
            .filter(|&o| self.active(o))
            .filter(|&o| {
                let c = self.pos_of(o).cell();
                combat::ring_of(c.x - landing_cell.x, c.y - landing_cell.y) <= radius
            })
            .filter(|&o| {
                let owner = self.owner_of(o);
                owner != p.owner
                    && !self.at_war.is_empty()
                    && (self.at_war[p.owner as usize][owner as usize]
                        || self.at_war[owner as usize][p.owner as usize])
            })
            .collect();
        for o in candidates {
            let op = self.profile(o);
            let pos = self.pos_of(o);
            let count = match o {
                Obj::Unit(_) => {
                    if matches!(op.domain, Domain::Air) != p.air || op.has(mask::MISSILE) {
                        continue;
                    }
                    let d = vector_dist(p.landing.x - pos.x, p.landing.y - pos.y)
                        - 0xc0
                        - op.guy_radius;
                    let c = combat::splash_count(d, p.splash_area);
                    if c <= 0 {
                        continue;
                    }
                    c
                }
                Obj::Building(_) => {
                    if p.air {
                        continue;
                    }
                    let d = (p.landing.x - pos.x).abs() - op.x_size * 0xc0;
                    let c = combat::splash_count(d, p.splash_area);
                    if c < 0 {
                        continue;
                    }
                    c
                }
            };
            let splash = target != Some(o);
            let angle = find_angle(pos.x - p.landing.x, pos.y - p.landing.y);
            self.do_damage(p.shooter, o, angle, true, count, splash, false, frame);
        }
    }

    /// `Ammo::check_hit` (§9.4): a unit within two tiles of the landing point
    /// in the ammo's domain class — any player's — else a building on the
    /// tile.
    fn check_hit(&self, p: &combat::Projectile) -> Option<Obj> {
        let mut best: Option<(i32, usize)> = None;
        for (i, u) in self.units.iter().enumerate() {
            if !(u.alive() && u.on_map) || Obj::Unit(i) == p.shooter {
                continue;
            }
            let up = self.profile(Obj::Unit(i));
            if matches!(up.domain, Domain::Air) != p.air {
                continue;
            }
            let d = vector_dist(u.pos.x - p.landing.x, u.pos.y - p.landing.y);
            if d > 0x180 {
                continue;
            }
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, i));
            }
        }
        if let Some((d, i)) = best {
            if self.profile(Obj::Unit(i)).target_size < d {
                // Found, but too small to be where it landed.
            } else {
                return Some(Obj::Unit(i));
            }
        }
        let tile = p.landing.tile();
        for (b, bd) in self.buildings.iter().enumerate() {
            if bd.combat.is_none() || bd.health <= 0 {
                continue;
            }
            let bp = self.profile(Obj::Building(b));
            let c = bd.pos;
            let dx = (tile.x - c.x).abs();
            let dy = (tile.y - c.y).abs();
            if dx <= bp.x_size * (UNITS_PER_TILE / 2) && dy <= bp.y_size * (UNITS_PER_TILE / 2) {
                return Some(Obj::Building(b));
            }
        }
        None
    }
}
