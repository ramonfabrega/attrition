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
//!
//! The first of those is **named but not spent**: [`SITE_ATTACK_POS_FIGHT`]
//! and [`SITE_ATTACK_POS_GROUP`] exist so the two chains the original draws
//! on read as themselves in a trace comparison rather than as the bare
//! `602129` the harness printed until item 324. Nothing in this crate marks
//! either yet — `docs/COMBAT.md` §17 is the specification and says so.

use crate::ai_load::uflags;
use crate::anim;
use crate::attrition::Domain;
use crate::combat::{self, Obj, Profile, Side, Sixteenths, Stance, Taken, mask, role};
use crate::movement::{Angle, find_angle};
use crate::world::{Pos, UNITS_PER_CELL, UNITS_PER_TILE, vector_dist};
use crate::{Player, Sim};

/// `Unit::find_attack_pos@00601280+0xea9` — the ring walk's one draw
/// (`docs/COMBAT.md` §17), reached through the seven-argument overload
/// `find_attack_pos@00602e60`, whose `+0x2d` is the return address the
/// `ebp` chain shows, from `Unit::fight@005fd4d0+0xcb4`. That is the
/// chase: a unit whose attack order's target is out of range asks where
/// to stand and walks there.
/// `TypeIndex::HOPLITES` — the lineage `ObjectData::is_in_range@006486b0`
/// tests with `is(0x84, 0)` before it chooses between the two melee
/// reaches (`docs/COMBAT.md` §13.2). The golden record's `add hoplite`
/// lands on the type whose tech-tree id is this.
pub const HOPLITES: crate::tech::TypeId = 0x84;

pub const SITE_ATTACK_POS_FIGHT: &str = "Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4";

/// **The one-in-five re-search's own draw** — `Unit::fight@005fd4d0`,
/// `005fde80`, the roll `docs/COMBAT.md` §8.2 step 0 spends *before* it
/// reads either suppression. The simulation has spent it since item 288
/// and never named it, so a frame that took it read as the unit-loop's
/// bare phase mark; item 364's golden record is where that cost a
/// comparison (`docs/INPUT.md` §11).
pub const SITE_FIGHT_RESEARCH: &str = "Unit::fight+0x9b0";

/// The same draw from `Group::action_attack@00712490+0x41a`, which calls
/// the nine-argument form **once**, on the group's leader, and only when
/// `ObjectData::is_in_range` says the leader cannot already shoot
/// (`action_attack@00712490:215`). One call a group order, so this label
/// can never run longer than one call's budget.
pub const SITE_ATTACK_POS_GROUP: &str = "Unit::find_attack_pos+0xea9 < Group::action_attack+0x41a";

/// The most draws **one** `find_attack_pos` call can take when the
/// stand-off `local_18` is over `0x300` — the ranged arm.
///
/// `00601280`'s ring loop counts iterations in `local_28` and stops on
/// `iter - 4 >= local_24`, where `local_24` starts at 100 and is cut to
/// `min(local_24, iter + 11)` the first time a candidate beats the
/// best-so-far (`60215d`-`60216a`). A candidate that reaches the draw
/// *always* beats a best-so-far of −1, so the cut happens on the first
/// draw and the loop then runs fourteen more iterations: fifteen draws is
/// the ceiling, whatever the map looks like. `docs/COMBAT.md` §17.4.
pub const ATTACK_POS_CAP_RANGED: usize = 15;

/// The same ceiling on the near arm — `local_18 <= 0x300`, where the cut
/// is `min(local_24, iter)` (`60216f`-`602179`) and three iterations
/// follow the first draw.
pub const ATTACK_POS_CAP_NEAR: usize = 4;

/// A unit that is not a combatant for the search: no type, no attack.
fn no_profile() -> Profile {
    Profile::default()
}

impl Sim {
    // ------------------------------------------------------------------
    // Reading an object
    // ------------------------------------------------------------------

    /// The combat table's entry for an attacker and a target of either
    /// family — `return_modifier` over `TypeIndex`, here over unit ids and
    /// building ids. 100 where an object has no type.
    fn table_pct(&self, attacker: Obj, target: Obj) -> i32 {
        let at = |o: Obj| match o {
            Obj::Unit(u) => self.units[u].ty.map(combat::TypeRef::Unit),
            Obj::Building(b) => self.buildings[b].ty.map(combat::TypeRef::Build),
        };
        match (at(attacker), at(target)) {
            (Some(a), Some(b)) => self.table.pct_of(a, b),
            _ => 100,
        }
    }

    /// The combat profile of an object.
    pub fn profile(&self, o: Obj) -> Profile {
        match o {
            Obj::Unit(i) => self.units[i]
                .ty
                .map_or_else(no_profile, |t| self.unit_types[t].combat),
            Obj::Building(b) => self.buildings[b].combat.unwrap_or_default(),
        }
    }

    pub(crate) fn owner_of(&self, o: Obj) -> Player {
        match o {
            Obj::Unit(i) => self.units[i].owner,
            Obj::Building(b) => self.buildings[b].owner,
        }
    }

    pub(crate) fn pos_of(&self, o: Obj) -> Pos {
        match o {
            Obj::Unit(i) => self.units[i].pos,
            Obj::Building(b) => self.buildings[b].pos,
        }
    }

    /// `flags & 1` and, for a unit, `is_on_map`.
    pub(crate) fn active(&self, o: Obj) -> bool {
        match o {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive() && u.on_map),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some() && b.health > 0),
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
        self.is_in_range_at(attacker, self.pos_of(attacker), target)
    }

    /// The same question asked from a **given** point rather than the
    /// attacker's own — the eight-argument overload, which
    /// `Group::action_attack` uses to ask whether a member's current move
    /// order would carry it into range (`docs/GROUPS.md` §10).
    pub fn is_in_range_at(&self, attacker: Obj, at: Pos, target: Obj) -> bool {
        if !self.active(target) {
            return false;
        }
        let ap = self.profile(attacker);
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
            self.reaches_like_a_hoplite(attacker),
            big,
            false,
        )
    }

    /// `is_in_range@006486b0`'s melee arm asks the **attacker** `is(0x84,
    /// 0)` and, for the lineage it names, takes `0xf6` where everything
    /// else takes `0x66` — a reach of two thirds of a tile rather than a
    /// half (`docs/COMBAT.md` §13.2). The constant had been in the
    /// document since the second reading and in [`combat::in_range`]'s
    /// signature since it was written; nothing ever passed it, so every
    /// melee unit in this simulation fought at `0x66`.
    ///
    /// It is the whole of the golden record's engagement frame
    /// (`docs/COMBAT.md` §12.5): six range verdicts on frame 615, and the
    /// `0x66` reading gets all six wrong in the same direction.
    fn reaches_like_a_hoplite(&self, attacker: Obj) -> bool {
        match attacker {
            Obj::Unit(u) => self.unit_line_is(u, HOPLITES),
            Obj::Building(_) => false,
        }
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
    /// A player's attack click: `add_attack_order(QUEUE_NEW, mandatory = 1,
    /// action = 1)`.
    pub fn order_attack(&mut self, unit: usize, target: Obj) {
        self.add_attack_order(unit, target, crate::orders::QueuePos::New, true, true);
    }

    pub(crate) fn bump_targeted_pub(&mut self, o: Obj, by: i32) {
        self.bump_targeted(o, by);
    }

    pub(crate) fn fight_pub(&mut self, i: usize, target: Obj, frame: i64) {
        self.fight(i, target, frame);
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

    /// The strike itself — `Unit::fight` from step 2 on (§8.2).
    fn fight(&mut self, i: usize, target: Obj, frame: i64) {
        let me = Obj::Unit(i);
        let p = self.profile(me);
        // Facing: toward the target.
        let (from, to) = (self.units[i].pos, self.pos_of(target));
        let angle = find_angle(to.x - from.x, to.y - from.y);
        self.units[i].movement.set_heading(angle);
        self.swing_anim(i, angle);
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
    ///   `param_3 = 0`, with its own per-figure damage;
    /// - everything else: `CHAR_ATTACK1`, `param_3 = 1`.
    ///
    /// SEAM: the middle two are not modelled — three elephant types and
    /// one target type — and neither draws, so the cost is which slot
    /// plays rather than a word. SEAM: the `z` arm's comparison is
    /// against `Unit::fight`'s own attack angle, which the **sideways**
    /// flag (`g`, `unit_flags & 0x40`, "most ships") offsets by a quarter
    /// turn either way; this crate does not model that offset, so the
    /// difference is always zero here and a ship always rocks the one
    /// way.
    fn swing_anim(&mut self, i: usize, angle: Angle) {
        let rocks = self.units[i]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag(uflags::ROCKS));
        if rocks {
            let attack_angle = self.units[i].movement.heading;
            let slot = if angle.0.wrapping_sub(attack_angle.0) >= 0 {
                anim::ATTACK2
            } else {
                anim::ATTACK1
            };
            self.set_anim(i, slot, false, false);
        } else {
            self.set_anim(i, anim::ATTACK1, false, true);
        }
    }

    /// `Object::fire_ammo` for a unit (§9.1): one `Ammo` per figure, here one.
    ///
    /// A siege type firing at a **unit** fires at the ground under it
    /// (`fight` inserts an `ATTACK_GROUND` order at the target's position,
    /// §8.2 step 1): the shot has no target to home on or to test against,
    /// and finds what it finds where it lands.
    fn fire_ammo(&mut self, shooter: Obj, target: Obj, angle: Angle, frame: i64) {
        let p = self.profile(shooter);
        let launch = self.pos_of(shooter);
        let tp = self.profile(target);
        let target_pos = self.pos_of(target);
        let ground_fire = p.siege && p.packs && matches!(target, Obj::Unit(_));
        // Accuracy and scatter. A ground shot's accuracy is against the plain
        // distance to the point, and its scatter the land-unit formula unless
        // the point is at sea (`accuracy` flag set by `fight`), which is exact.
        let acc = if ground_fire {
            combat::accuracy(
                p.to_hit,
                p.attenuate,
                vector_dist(target_pos.x - launch.x, target_pos.y - launch.y),
            )
        } else {
            combat::accuracy(p.to_hit, p.attenuate, self.attack_dist(shooter, target))
        };
        let land_unit = matches!(target, Obj::Unit(_)) && matches!(tp.domain, Domain::Land);
        let s = if ground_fire {
            if matches!(tp.domain, Domain::Sea) {
                0
            } else {
                combat::scatter(&self.tuning, acc, true, false, false)
            }
        } else {
            combat::scatter(&self.tuning, acc, land_unit, p.has(mask::MISSILE), false)
        };
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
        let clamp = |q: Pos, w: &crate::World| {
            Pos::new(
                q.x.clamp(0, w.width() * UNITS_PER_CELL - 1),
                q.y.clamp(0, w.height() * UNITS_PER_CELL - 1),
            )
        };
        let mut landing = clamp(landing, &self.world);
        // Flight time — never zero.
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
        }
        .max(1);
        // The lead: a unit target that is moving has the landing point pushed
        // along its heading by its speed for the time of flight (§9.1).
        if !ground_fire
            && let Obj::Unit(t) = target
            && self.units[t].movement.dest.is_some()
        {
            let m = self.units[t].movement;
            landing = clamp(
                Pos::new(
                    landing.x + crate::movement::sin_component(m.facing, m.speed) * total_time,
                    landing.y - crate::movement::cos_component(m.facing, m.speed) * total_time,
                ),
                &self.world,
            );
        }
        let _ = frame;
        self.projectiles.push(combat::Projectile {
            shooter,
            owner: self.owner_of(shooter),
            target: if ground_fire { None } else { Some(target) },
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
        let pct = self.table_pct(attacker, target);
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
        // `Armies::emergency`: an AI leader's object hit by another
        // player's (`docs/ARMY.md` §15.8).
        {
            let tw = self.owner_of(target);
            if !quiet && self.owner_of(attacker) != tw && (tw as usize) < self.armies.len() {
                self.armies_emergency(tw);
            }
        }
        // Step 8 and 9 on a building: a kill by a non-air, non-splash,
        // non-allied unit plunders it; a hit that did not kill a capturable
        // building by another player's unit is a capture attempt
        // (`docs/CITIES.md` §7.1, §8.3).
        if let Obj::Building(b) = target {
            let who = self.owner_of(attacker);
            let bowner = self.buildings[b].owner;
            if killed {
                if matches!(attacker, Obj::Unit(_))
                    && !splash
                    && !matches!(ap.domain, Domain::Air)
                    && !self.is_ally(who, bowner)
                {
                    self.plunder_kill(b, who);
                }
            } else if let Obj::Unit(u) = attacker
                && who != bowner
                && self.capture_eligible(b)
            {
                self.check_capture(b, u);
            }
        }
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
            Obj::Building(b) => self.damage_building(b, hit, Some(_by), _frame, false),
        }
    }

    /// `Object::take_damage` on a building (§7.2): the under-attack latch, a
    /// site's lost progress and the building-on-site quadrupling, the city
    /// clamp — a city never dies, it sits at zero — and death through
    /// `Build::close`. `attrition` is the building-side attrition's call
    /// (`docs/CITIES.md` §9.5), which latches nothing.
    pub(crate) fn damage_building(
        &mut self,
        b: usize,
        hit: Sixteenths,
        by: Option<Obj>,
        frame: i64,
        attrition: bool,
    ) -> Taken {
        if !self.buildings[b].alive {
            return Taken::Alive { lost: 0 };
        }
        let mut hit = hit;
        let owner = self.buildings[b].owner;
        if !attrition
            && let Some(by) = by
            && self.owner_of(by) != owner
        {
            let bd = &mut self.buildings[b];
            bd.under_attack |= 0x3;
            bd.hit_frame = Some(frame);
        }
        let site = !self.buildings[b].active && self.buildings[b].ty.is_some();
        if site {
            if matches!(by, Some(Obj::Building(_))) {
                hit.whole *= 4;
                hit.frac *= 4;
            }
            let bd = &mut self.buildings[b];
            bd.job_counter = (bd.job_counter - combat_progress_lost(hit.whole)).max(0);
        }
        let bd = &self.buildings[b];
        let share = bd.health;
        let (taken, _, frac) = combat::take(0, bd.damage_frac, share, hit);
        let lost = match taken {
            Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
        };
        let city = self.building_is_city(b);
        let bd = &mut self.buildings[b];
        bd.damage_frac = frac;
        bd.damage += lost;
        let hits = bd.hits_now();
        if city && bd.active {
            // The clamp: a city never dies from damage.
            if bd.damage >= hits {
                bd.damage = hits;
                bd.damage_frac = 0;
                if !bd.garrison.is_empty() {
                    bd.eject_pending = true;
                }
            }
            bd.sync_health();
            return Taken::Alive { lost };
        }
        bd.sync_health();
        if matches!(taken, Taken::Died { .. }) {
            bd.damage = hits;
            bd.sync_health();
            self.die_building(b);
        } else if site && bd.construct_hits <= bd.damage && !bd.garrison.is_empty() {
            bd.eject_pending = true;
        }
        taken
    }

    /// A dead object is dropped from every target slot and from ammo in
    /// flight — what `close` does through `hold_frames` and `valid_target`.
    pub(crate) fn forget(&mut self, dead: Obj) {
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
        let (a, b) = (self.units[victim].owner, self.owner_of(attacker));
        if !self.at_war_with(a, b) && !self.at_war_with(b, a) {
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
            let pct = self.table_pct(attacker, target);
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
        // **`in_range` is the caller's permission to test, not its verdict.**
        // `0064f1ed` reads `param_3 != 0 && !raiding` and then calls
        // `is_in_range` *itself*; a candidate the search deemed in range
        // without testing (every candidate of a non-guarding unit that is
        // not STAND_GROUND) is the one this arm actually measures. Reading
        // it the other way round made the golden record's three candidates
        // score equal — see `docs/COMBAT.md` §18.
        let raiding =
            matches!(attacker, Obj::Unit(i) if self.units[i].combat.stance == Stance::Raid);
        if in_range && !raiding && !self.is_in_range(attacker, target) {
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
        let phase = bd.phase(frame) & 0x1f;
        // Without a target, `do_attack` runs every 32nd frame.
        if bd.target.is_none() && phase != 0 {
            return;
        }
        if bd.recharging > 0 {
            self.buildings[b].recharging -= 1;
            return;
        }
        let p = self.profile(me);
        // The garrison's arrows: the input the earlier mechanic took, plus
        // what the squads actually inside contribute (`docs/CITIES.md` §6).
        let garrison_attack = bd.garrison_attack + self.garrison_attack_sum(b);
        let arrows = combat::garrison_arrows(
            self.attack_of(me),
            p.base_arrows,
            p.most_shots,
            garrison_attack,
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
                        && ((bd.phase(frame) + 14) & 0x1f) == 0
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
            let total_time = combat::flight_time(dx * dx + dy * dy, d).max(1);
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
        // Splash: everything on the square of cells the spiral table walks,
        // the intended target at full count and everything else as a fringe.
        let k = combat::splash_cells(p.splash_area);
        let landing_cell = p.landing.cell();
        let candidates: Vec<Obj> = (0..self.units.len())
            .map(Obj::Unit)
            .chain((0..self.buildings.len()).map(Obj::Building))
            .filter(|&o| self.active(o))
            .filter(|&o| {
                let c = self.pos_of(o).cell();
                (c.x - landing_cell.x).abs() <= k && (c.y - landing_cell.y).abs() <= k
            })
            .filter(|&o| {
                let owner = self.owner_of(o);
                owner != p.owner
                    && (self.at_war_with(p.owner, owner) || self.at_war_with(owner, p.owner))
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
                    // The x term is explicit in the decompile and the y term
                    // is a lost register; by symmetry both axes less the
                    // footprint, then the hypotenuse.
                    let dx = ((p.landing.x - pos.x).abs() - op.x_size * 0xc0).max(0);
                    let dy = ((p.landing.y - pos.y).abs() - op.y_size * 0xc0).max(0);
                    let d = vector_dist(dx, dy);
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
            // `check_hit` is a `find_unit`, whose leader loop stops at eight:
            // an arrow never lands on gaia (`world::PLAYER_SLOTS`).
            if u.is_gaia() {
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

/// `Object::take_damage` on a site: `whole × 50` off the progress.
const fn combat_progress_lost(whole: i32) -> i32 {
    crate::build::progress_lost(whole)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{PLAYER_SLOTS, World};

    /// Two players at war, and a soldier type that can shoot.
    fn at_war() -> (Sim, usize) {
        at_war_with_slots(2)
    }

    /// The same, with `slots` player slots — so a test can *widen* the
    /// diplomacy table past the eight the original has and check that the
    /// leader bound, not the table's length, is what refuses gaia.
    fn at_war_with_slots(slots: usize) -> (Sim, usize) {
        let mut sim = Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), slots);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 15,
                max_range: 4,
                uber_size: 1,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        (sim, ty)
    }

    fn put(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = crate::Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = sim.unit_types[ty].kind;
        sim.add_unit(u)
    }

    /// `ObjectData::valid_target_const@006472c0`'s first line, `7 < who`,
    /// which runs *before* the diplomacy question. To show that it is the
    /// leader bound doing the work and not the sim's short table, the table
    /// here is ten wide and gaia is declared at war: the animal is still
    /// nobody's target, and the ring search — which walks the cell chains
    /// with no leader bound of its own — steps over it to the enemy behind.
    #[test]
    fn a_gaia_animal_is_no_ones_target_even_at_war() {
        let (mut sim, ty) = at_war_with_slots(usize::from(PLAYER_SLOTS) + 2);
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        let sheep = put(&mut sim, PLAYER_SLOTS, ty, Pos::new(0x1040, 0x1000));
        sim.at_war[0][usize::from(PLAYER_SLOTS)] = true;
        sim.at_war[usize::from(PLAYER_SLOTS)][0] = true;
        assert!(sim.units[sheep].is_gaia());
        assert!(sim.is_enemy(0, PLAYER_SLOTS), "the table says enemy");
        assert!(sim.valid_target(Obj::Unit(me), Obj::Unit(foe)));
        assert!(!sim.valid_target(Obj::Unit(me), Obj::Unit(sheep)));
        assert_eq!(
            sim.find_nearby_target(Obj::Unit(me), 0),
            Some(Obj::Unit(foe))
        );
    }

    /// And with the table the size a real lobby gives it — two — asking the
    /// question at all is what took the first fuzzed seed down. Both
    /// accessors are total now (`Sim::is_ally`).
    #[test]
    fn the_diplomacy_accessors_are_total_past_the_table() {
        let (sim, _) = at_war();
        assert_eq!(sim.at_war.len(), 2);
        assert!(!sim.is_enemy(0, PLAYER_SLOTS));
        assert!(!sim.is_ally(0, PLAYER_SLOTS));
        assert!(!sim.is_enemy(PLAYER_SLOTS, 0));
        assert!(!sim.is_ally(PLAYER_SLOTS, 0));
        // Reflexivity survives it.
        assert!(sim.is_ally(PLAYER_SLOTS, PLAYER_SLOTS));
    }

    /// `Ammo::check_hit@00678d90` is an `ObjectsData::find_unit`, whose
    /// leader loop runs eight slots. So an arrow that comes down on top of a
    /// sheep passes through it.
    #[test]
    fn an_arrow_never_lands_on_gaia() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let landing = Pos::new(0x1400, 0x1000);
        let sheep = put(&mut sim, PLAYER_SLOTS, ty, landing);
        let ammo = combat::Projectile {
            shooter: Obj::Unit(me),
            owner: 0,
            target: None,
            launch: sim.units[me].pos,
            landing,
            cur_time: 0,
            total_time: 1,
            accuracy: 100,
            angle: crate::movement::Angle(0),
            splash_area: 0,
            num_guys: 1,
            air: false,
        };
        assert_eq!(sim.check_hit(&ammo), None);
        // The same arrow does find a player's unit standing there.
        let foe = put(&mut sim, 1, ty, landing);
        assert_eq!(sim.check_hit(&ammo), Some(Obj::Unit(foe)));
        assert!(sim.units[sheep].is_gaia());
    }
}
