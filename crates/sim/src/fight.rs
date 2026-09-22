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

/// `Ammo::init@0067bbf0+0xcd9` — the landing scatter's **x** draw, the
/// first of the two a shot spends (`docs/COMBAT.md` §9.1, §9.5).
///
/// It is named here because the frame it lands on is the whole of
/// §9.0: run53's first `Objects::add_ammo` in 24,000 frames is
/// sim-frame 9425, and this crate spent it on 9415 until the launch
/// moved to the animation's own event track.
pub const SITE_AMMO_SCATTER_X: &str = "Ammo::init+0xcd9";

/// `Ammo::init@0067bbf0+0xd0b` — the landing scatter's **y** draw.
pub const SITE_AMMO_SCATTER_Y: &str = "Ammo::init+0xd0b";

/// `Ammo::do_damage@00678060+0xc59` — **where a shot that hit nothing
/// punctures the ground**, the x draw of the pair (`docs/COMBAT.md` §39).
///
/// The arm is `do_damage`'s own: after `hit_target` and `check_hit` have
/// both failed to name an object, the landing point is jittered by
/// `Random::get(0, 0xffff) % 41 - 20` on each axis, `WorldData::restrict`
/// pulls it back inside the world, and `AmmoOut::puncture_ground` leaves
/// the decal there. Cosmetic, and it still moves the stream: this crate
/// has spent both draws since the ammo list existed and named neither, so
/// the frame read as the tick's bare `projectiles` phase mark against the
/// original's raw `678cb9` — a comparison that cannot fail and cannot
/// pass.
pub const SITE_PUNCTURE_X: &str = "Ammo::do_damage+0xc59";

/// `Ammo::do_damage@00678060+0xc7e` — the puncture point's **y** draw.
///
/// A second entry rather than one label for the pair, for
/// [`SITE_AMMO_SCATTER_X`]'s reason: the original spends two draws at two
/// addresses and a single label folds them into one.
pub const SITE_PUNCTURE_Y: &str = "Ammo::do_damage+0xc7e";

/// `Object::take_damage@00652020+0xe1` — **a building's first wound**
/// (`docs/COMBAT.md` §7.2 step 3).
///
/// One `Random::get(0, 0xffff)` whenever combat damage reaches an object
/// whose `damage` field is still zero and whose vtable slot `+0x1c`
/// answers 1 — `Build` and `Wall`, never a `Unit` or an `Animal`, whose
/// slot is the folded `return 0`. The roll is read `% 100 < 5` and only
/// then does the fort/temple/town test below it run, so **the draw is
/// unconditional on the building's kind**: §9.5 had it as a fort,
/// temple or town's own cost and that was a reading error, corrected
/// here by item 394's measurement.
///
/// `damage` is the whole-hit count, not the sixteenths — so a hit small
/// enough to move only `damage_frac` leaves the gate open and the **next**
/// hit draws again. That is exactly what Great Lakes 9451 is: two arrows
/// landing on the farm `0/2004` in one frame, each worth thirteen
/// sixteenths, the first leaving `damage` at 0.
pub const SITE_FIRST_WOUND: &str = "Object::take_damage+0xe1";

/// `Object::take_damage@00652020+0x18b` — the flock's size, the second
/// draw of a first wound and the only one [`SITE_FIRST_WOUND`] gates.
///
/// Taken when the roll above is `% 100 < 5`, the target is a fort,
/// `TEMPLE` or `TOWN`, the attacker exists (`o >= 0`) and its type is
/// siege: `Objects::add_flock(x, y, -1, roll % 2 + 3)` puts three or four
/// birds up over the building. Cosmetic, and it still moves the stream.
pub const SITE_FIRST_WOUND_FLOCK: &str = "Object::take_damage+0x18b";

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
    ///
    /// This overload passes `006486b0`'s sixth argument **zero**, which
    /// every call site in the executable but one does; the one is
    /// [`Self::is_in_range_at_margin`]'s. Item 470 measured the margin
    /// and did not land it, because on its own it moved `0/10` from one
    /// frame early to one frame late; item 472 landed it beside
    /// `Unit::check_target_path`, which is what actually ends `0/10`'s
    /// chase (`docs/COMBAT.md` §36).
    pub fn is_in_range_at(&self, attacker: Obj, at: Pos, target: Obj) -> bool {
        self.is_in_range_at_margin(attacker, at, target, false)
    }

    /// The same with `006486b0`'s **sixth argument** — the `0x90` the test
    /// adds to the measured distance before the max-range bound
    /// (`docs/COMBAT.md` §35.3). `do_move@005f7b30:216` is the only call
    /// site in the executable that passes it non-zero, and what it passes
    /// is `attack->mandatory == 0`: an ordered attack is dropped at the
    /// edge of reach, an opportunistic one three quarters of a tile
    /// inside it.
    pub fn is_in_range_at_margin(&self, attacker: Obj, at: Pos, target: Obj, margin: bool) -> bool {
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
            margin,
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
    /// SEAM, and both are above conjunct 1: the candidate's first `Guy`
    /// carrying `guy_flags & 0x40` takes a **different** arm entirely
    /// (`has_objmask(0x80000000)`, then the reach floor), and this crate
    /// has no such guy flag, so the `== 0` arm is always taken.
    /// `role & 0x400` is likewise unloaded — this crate's
    /// [`combat::role`] word is its own synthesis and not the original's
    /// — so the floor is always the tile.
    pub(crate) fn poor_target(&self, me: Obj, cand: Obj) -> bool {
        let (Obj::Unit(u), Obj::Unit(c)) = (me, cand) else {
            return false;
        };
        if !crate::orders::index::is_move_family(self.order_type(c)) {
            return false;
        }
        if self.get_speed(u) >= self.get_speed(c) {
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
        self.attack_dist(me, cand) > 0xc0
    }

    /// `ObjectData::valid_target_const` + `Object::valid_target` (§12.1), as
    /// far as the simulation's state reaches: not mine, at war, active, on the
    /// map, **and seen**; the air ladder reduced to "air targets need a
    /// ranged attacker and the two AIR/ANTI_AIR rules"; no capture.
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
    /// and `WorldData::is_seen@006b55c0` is `seen[fy * fog_xs + fx] &
    /// ally_mask`, with three always-true arms above it: `who > 7`,
    /// `reveal_map == 3`, and the two leader flags (`0x800`, and a
    /// `num_units[0x141]` count) this crate does not carry.
    ///
    /// The fallback under the fog test is `ObjectData::visible`, and since
    /// item 457 it is modelled: [`Sim::set_attacking`] sets the target's
    /// own bit for the player it shoots at, so **a unit that attacks you
    /// stays a legal target of yours through the fog** until it goes back
    /// to work. `docs/VISION.md` §7.
    ///
    /// SEAM: `WorldData::is_seen`'s **third** arm — `leader_flags &
    /// 0x2000` and the cell's `WData::who` being an ally, which returns 1
    /// over friendly ground whatever the fog says — is not carried, and
    /// nor are the two always-true leader arms above it (`0x800`, and a
    /// `num_units` count). All three can only *refuse* further here.
    ///
    /// SEAM: `is_seen`'s stealth arm above the fog test — `unit_masks`
    /// `0x800`/`0x1000`, `unit_masks2 0x8000`, the type's
    /// `0x4000`/`0x40000` and `is_detected` — is not carried. It can only
    /// *refuse* further.
    pub(crate) fn target_is_seen(&self, attacker: Obj, target: Obj) -> bool {
        if !self.world.has_fog() {
            return true;
        }
        let who = self.owner_of(attacker);
        if who >= 8 {
            return true;
        }
        if self.world_sees(self.pos_of(target), who) {
            return true;
        }
        // `return (visible >> who) & 1` — the fallback, and the whole of
        // item 457.
        self.visible_of(target) & Self::who_bit(who) != 0
    }

    /// `WorldData::is_seen@006b55c0` at one object's position: does `who`'s
    /// alliance currently light the half-cell it stands on?
    ///
    /// Off the grid keeps [`crate::world::World::seen`]'s "no answer"
    /// reading, which is what its other callers take — the original has no
    /// bounds test here at all and would read past the plane.
    pub(crate) fn world_sees(&self, p: Pos, who: crate::Player) -> bool {
        let fog = crate::vision::UNITS_PER_FOG;
        let Some(bits) = self.world.seen(p.x / fog, p.y / fog) else {
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
            Obj::Building(_) => 0,
        }
    }

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
    /// SEAM: the two leader tables the function's first two lines write —
    /// the per-pair "has attacked" record at `00e3a424` and the victim's
    /// own counter-array — are not modelled; nothing here reads either.
    pub(crate) fn set_attacking(&mut self, i: usize, who: crate::Player) {
        self.units[i].attacking = true;
        let bit = Self::who_bit(who);
        if self.units[i].visible & bit != 0 {
            return;
        }
        let reveal = !self.world_sees(self.units[i].pos, who);
        self.units[i].visible |= bit;
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

    pub(crate) fn fight_pub(&mut self, i: usize, target: Obj, frame: i64) {
        self.fight(i, target, frame);
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
        // `Unit::fight@005fd4d0`'s `LAB_005feec6`, immediately after
        // `set_anim` and before the damage: the strike makes this unit
        // visible to whoever it just hit (`docs/VISION.md` §7).
        self.set_attacking(i, self.owner_of(target));
        if self.max_range_of(me) == 0 {
            // Melee lands now, once per figure of this `UnitData` — one here.
            if p.fires() {
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
            // every piece it has not measured.
            self.fire_ammo(me, target, angle, frame, from);
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
            .is_some_and(|g| self.art.releases.contains_key(&g.gpiece))
    }

    /// The landing scatter's two draws, under the original's own site
    /// names — `Ammo::init+0xcd9` and `+0xd0b` (§9.1).
    ///
    /// [`combat::scatter_point`] is the arithmetic; this is the same thing
    /// with a mark before each draw, because the two addresses are two
    /// entries in the compared sequence and a single label would fold
    /// them into one.
    fn scatter_landing(&mut self, at: Pos, s: i32) -> Pos {
        if s - 1 < 1 {
            return at;
        }
        self.mark(SITE_AMMO_SCATTER_X);
        let dx = self.rng.roll() % s - s / 2;
        self.mark(SITE_AMMO_SCATTER_Y);
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
    pub(crate) fn fire_ammo_pub(
        &mut self,
        shooter: Obj,
        target: Obj,
        angle: Angle,
        frame: i64,
        launch: Pos,
    ) {
        self.fire_ammo(shooter, target, angle, frame, launch);
    }

    fn fire_ammo(&mut self, shooter: Obj, target: Obj, angle: Angle, frame: i64, launch: Pos) {
        let p = self.profile(shooter);
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
        let landing = self.scatter_landing(aim, s);
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
        // **`Armies::emergency` is the CITY alarm's, and a unit never
        // reaches it** (`docs/ARMY.md` §15.8, item 399). The call at
        // `do_damage@0064a480:951` is
        // `if (local_30 != 0 && (leaders[param_2].leader_flags & 4) == 0)`,
        // and `local_30` has exactly **two** writers between its `= 0` at
        // `0064a8dc` and the test — both inside the city-alarm arms of the
        // **building** branch, beside `S_CITY_BEING_ATTACKED` and
        // `S_YOUR_CAPITAL_ATTACKED`. So the emergency is "a city of mine is
        // under attack", not "something of mine was hit": the target must be
        // a building that belongs to a city (`BuildData +0x72 >= 0`).
        //
        // What is *not* modelled here, and both only make it rarer: the
        // alarm's own damage threshold (`local_34`) and its 300-frame
        // cooldown on `CityData +0x14 attack_stamp`. §15.8 carries them.
        if let Obj::Building(b) = target
            && !quiet
            && self.buildings[b].city.is_some()
        {
            let tw = self.owner_of(target);
            if self.owner_of(attacker) != tw && (tw as usize) < self.armies.len() {
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

    /// The **first wound**'s draws (§7.2 step 3), spent before the
    /// accumulate and only on the combat path.
    ///
    /// `Object::take_damage@00652020` reaches `+0xe1` when `damage == 0`
    /// and the object's vtable slot `+0x1c` answers 1 — a `Build` or a
    /// `Wall`, never a unit — and takes one roll there whatever kind of
    /// building it is. Only if that roll is `% 100 < 5` does the
    /// fort/`TEMPLE`/`TOWN` test run, and only then, with a siege
    /// attacker, the second roll at `+0x18b` that sizes a flock of birds.
    ///
    /// The gate reads `damage`, the whole-hit count — a hit that moves
    /// only `damage_frac` leaves it at zero and the next hit draws again.
    fn first_wound_draws(&mut self, b: usize, by: Option<Obj>, attrition: bool) {
        if attrition || self.buildings[b].damage != 0 {
            return;
        }
        self.mark(SITE_FIRST_WOUND);
        if self.rng.roll() % 100 >= 5 {
            return;
        }
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let fortlike = crate::build::is_fort(&self.build_types, ty)
            || crate::build::is(&self.build_types, ty, crate::build::Ident::Temple)
            || crate::build::is(&self.build_types, ty, crate::build::Ident::Town);
        if !fortlike {
            return;
        }
        // `if (param_7 < 0) goto LAB_006522f2` — no attacker, no flock
        // and no war declaration either.
        let Some(by) = by else { return };
        if !self.profile(by).siege {
            return;
        }
        self.mark(SITE_FIRST_WOUND_FLOCK);
        // `Objects::add_flock(x, y, -1, roll % 2 + 3)` — three or four
        // birds over the building. Cosmetic; the draw is not.
        let _ = self.rng.roll();
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
        self.first_wound_draws(b, by, attrition);
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
                // **The order's `mandatory` is not the target's, and it
                // outlives it** (item 463). `Unit::do_attack@005f1b80`
                // reads `mandatory` off the *order* — `TargetOrder
                // +0x1c` — and run100's block 10233 still prints
                // `ox 2004 whom 0 uid 4 mandatory 1` on `1/27`'s
                // `ATTACKORDER` two blocks after the building `0/2004`
                // left the dump. Clearing it here sent a HOLD_FIRE
                // raider into [`crate::Sim::do_attack`]'s stance arm,
                // which returns, so the order it should have dropped on
                // its own reload's last frame stood for ever.
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
    /// SEAM: the loop's other arm, taken **before** the captain walk at each
    /// level — a group member whose type is *not* combat-role
    /// (`type +0x2c8 & 0x10000`) forwards to `Group::target_opportunity`
    /// instead. Every unit in the golden record's two squads is combat-role,
    /// so no run on disk takes it (`docs/GROUPS.md` §13).
    fn target_opportunity(&mut self, victim: usize, attacker: Obj, _frame: i64) {
        let (a, b) = (self.units[victim].owner, self.owner_of(attacker));
        if !self.at_war_with(a, b) && !self.at_war_with(b, a) {
            return;
        }
        let responder = self.squad_captain(victim);
        let me = Obj::Unit(responder);
        if !self.valid_target(me, attacker) {
            return;
        }
        let st = self.units[responder].combat;
        if st.stance == Stance::HoldFire {
            return;
        }
        // `UnitData::is_fleeing@0046efa0` — `order_type() == FLEE_TO`
        // (`docs/ORDERS.md` §1.3). A unit already running does not answer
        // the hit again, and the test sits **above** everything below it
        // (`6002cf`).
        let front = self.units[responder].orders.front().copied();
        if front.is_some_and(|o| o.index() == crate::orders::index::FLEE_TO) {
            return;
        }
        if st.target.is_some() {
            return;
        }
        // **The flee arm** (`docs/COMBAT.md` §34), and it is reached only
        // with `local_c == 0` — the front order is not one of the seven
        // move kinds (`6002a0`'s `is_move(order_type())`).
        if !front.is_some_and(|o| crate::orders::index::is_move_family(o.index()))
            && self.flee_from(responder, attacker)
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
        self.retarget(me, Some(attacker), false);
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
    /// SEAMS, each a clause this crate cannot yet ask and no capture on
    /// disk reaches: the packing latch `local_8` (a packer mid-pack, set
    /// at `600130`); the `unit_flags & 4` packer arm; and the hero and
    /// supply radii, which need `is_hero`/`is_supply` and the attacker's
    /// vslot `+0x130`.
    fn flee_from(&mut self, u: usize, attacker: Obj) -> bool {
        let me = Obj::Unit(u);
        let p = self.profile(me);
        let civilian = p.has(mask::CIVILIAN) && self.max_range_of(me) == 0;
        if !civilian && p.attack != 0 {
            return false;
        }
        if self.worker_of(u) == crate::orders::Worker::None && !self.units[u].orders.is_empty() {
            return false;
        }
        if p.packs {
            return false;
        }
        let here = self.units[u].pos;
        let from = self.pos_of(attacker);
        let angle = find_angle(here.x - from.x, here.y - from.y);
        let spot = self
            .find_nearby_spot(u, here, 0x600, -1, 0, angle, None)
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
            .filter(|&o| o != attacker && self.active(o))
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
                        let value = self.compare_target(attacker, o, in_range, ai);
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
        // it standing. SEAM: this crate holds the pair on a unit only;
        // the original's is an `ObjectData` field and a building carries
        // one too, read by nothing either crate models.
        if let Obj::Unit(me) = attacker {
            self.units[me].near = near.filter(|&(d, _)| d <= 0xf00).map(|(_, o)| o);
        }
        best.map(|(_, o)| o)
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
    /// `ai`'s other arms — the building-class multipliers at `0064e6a5`,
    /// `0064e9e2` and `0064e7bb` — are still unmodelled, and no capture on
    /// disk puts a building in a target search (§33.5).
    pub fn compare_target(&self, attacker: Obj, target: Obj, in_range: bool, ai: bool) -> i32 {
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
        // **`0064ef4b`, both arms.** A human multiplies; a computer
        // leader on the lobby's own difficulty divides, and a candidate it
        // cannot hurt at all scores zero rather than dividing by nothing.
        if ai {
            if dmg == 0 {
                return 0;
            }
            v /= i64::from(dmg);
        } else {
            v *= i64::from(dmg);
        }
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
            self.buildings[b].target = self.find_nearby_target(me, radius);
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
                // A shot into the ground: two draws for where it punctures
                // (§39). Marked per draw, because the original spends them
                // at two addresses.
                self.mark(SITE_PUNCTURE_X);
                let _ = self.rng.roll();
                self.mark(SITE_PUNCTURE_Y);
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

    /// **A hit is answered by the victim's captain, never by the figure that
    /// took it.** `Unit::target_opportunity@005fffc0`'s opening `while`
    /// breaks on `is_captain` and otherwise re-enters on
    /// `objects[who][this->o_up]`, so every test past it — and the
    /// `add_attack_order` at its foot — runs on the head of the squad. The
    /// golden record measures it (`docs/COMBAT.md` §18.2): `1/6` strikes
    /// `0/7` on frame 616 and it is `0/6` that carries the ATTACKORDER at
    /// that frame's end, with `0/7` a frame behind.
    ///
    /// Made to fail on purpose: answering on the victim gives
    /// `(None, Some(foe))`, which is what this crate did until item 391 and
    /// what put the retaliation a frame early on the golden record's 617.
    #[test]
    fn the_hit_is_answered_by_the_victim_s_captain() {
        let (mut sim, ty) = at_war();
        let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        sim.units[sub].captain = false;
        sim.units[sub].o_up = Some(cap);
        sim.units[cap].o_down = Some(sub);
        sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(sub),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        assert_eq!(
            (sim.units[cap].combat.target, sim.units[sub].combat.target),
            (Some(Obj::Unit(foe)), None),
            "the captain retaliates and the figure that was hit does not"
        );
    }

    /// **A hit civilian runs away from whoever hit it** —
    /// `Unit::target_opportunity`'s flee arm, `docs/COMBAT.md` §34. The
    /// sweep's base bearing is `find_angle(me − attacker)` and its ring
    /// is `0x600`, so the spot is on the far side of the victim from the
    /// attacker, and the order goes in at `QUEUE_FIRST` as a `FLEE_TO`.
    ///
    /// **Made to fail on purpose**, both ways that matter. With the arm
    /// removed the citizen **retaliates** — the `obj_masks & CIVILIAN`
    /// return that used to stand in its place is gone with it, so the
    /// failure is an `AttackOrder` rather than nothing, which is the
    /// shape run100's `0/5` had until item 464. And with the bearing
    /// taken from the attacker's side (`attacker − me`) the spot lands
    /// *past* the attacker at `(17928, 16392)`, which the `x` assertion
    /// below refuses. `QUEUE_FIRST` is the listing's (`60084a` pushes
    /// `FLEE_TO` and `600856` calls `add_move_order`), not a diff's.
    ///
    /// run100's own measurement is Great Lakes 10233: the human's citizen
    /// `0/5` at `(2232, 31224)` is struck by `1/27` at `(4584, 29784)`
    /// and the dump's block 10234 carries `FLEETOORDER x 792 y 31800` —
    /// 1,551 units away, on the `0x600` ring, and this crate now lands on
    /// that pair exactly.
    #[test]
    fn a_hit_citizen_flees_away_from_whoever_hit_it() {
        let (mut sim, soldier) = at_war();
        let citizen = sim.add_unit_type(crate::UnitType {
            hits: 40,
            // The gate's own pair: `obj_masks & CIVILIAN` with no range,
            // and an `attack` column that is **not** zero — a citizen's
            // is 40, which is why the attack half of the disjunction is
            // not what carries this.
            combat: Profile {
                attack: 40,
                max_range: 0,
                uber_size: 1,
                obj_masks: mask::CIVILIAN,
                ..Profile::default()
            },
            worker: crate::orders::Worker::Citizen,
            ..crate::UnitType::default()
        });
        let me = put(&mut sim, 0, citizen, Pos::new(0x4000, 0x4000));
        let foe = put(&mut sim, 1, soldier, Pos::new(0x4600, 0x4000));
        sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        let front = sim.units[me].orders.front().copied().expect("an order");
        let crate::orders::Body::Move(m) = front.body else {
            panic!("the hit citizen did not flee: {front:?}");
        };
        assert_eq!(m.kind, crate::orders::MoveKind::FleeTo);
        assert!(
            m.dest.x < 0x4000,
            "the flight is away from the attacker, not past it: {:?}",
            m.dest
        );
        assert_eq!(
            sim.units[me].combat.target, None,
            "a fleeing civilian takes no target"
        );
        // And it does not answer a second hit while it runs —
        // `UnitData::is_fleeing@0046efa0`, the test at `6002cf`.
        let before = sim.units[me].orders.len();
        sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        assert_eq!(
            sim.units[me].orders.len(),
            before,
            "a unit already fleeing answers the hit again"
        );
    }

    /// **`Unit::think`'s step 3 needs the military bit as well as the
    /// attack column** — `think@005f6e40:150`'s second arm is
    /// `type->attack != 0 && (type->role & 0x10000) != 0`, and until item
    /// 464 this crate's target search read only the first half. A
    /// citizen's `attack` is 40, so every idle citizen in run100 ran
    /// `find_melee_target` and one of them — the human's `0/5` on Great
    /// Lakes 10233 — took an `ATTACKORDER` where the original was still
    /// standing still.
    ///
    /// [`Sim::think_attack_join_army`] has carried the same pair since
    /// item 350 (`docs/ARMY.md` §4.2's own test says so in as many
    /// words); this is the other half of the arm it gates.
    ///
    /// **Made to fail on purpose**: with `combat_role` dropped from the
    /// gate the citizen below takes the same order the soldier does.
    #[test]
    fn an_armed_citizen_does_not_take_an_attack_order_on_its_idle_frame() {
        let armed_citizen = |military: bool| {
            let (mut sim, _) = at_war();
            let ty = sim.add_unit_type(crate::UnitType {
                hits: 40,
                combat: Profile {
                    attack: 40,
                    max_range: 0,
                    uber_size: 1,
                    obj_masks: mask::CIVILIAN,
                    combat_role: military,
                    ..Profile::default()
                },
                worker: crate::orders::Worker::Citizen,
                ..crate::UnitType::default()
            });
            let foe_ty = sim.add_unit_type(crate::UnitType {
                hits: 100,
                combat: Profile {
                    attack: 15,
                    uber_size: 1,
                    ..Profile::default()
                },
                ..crate::UnitType::default()
            });
            let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
            put(&mut sim, 1, foe_ty, Pos::new(0x4060, 0x4000));
            sim.tick();
            sim.units[me].combat.target
        };
        assert_eq!(
            armed_citizen(false),
            None,
            "a type with an attack column and no `role & 0x10000` entered \
             `think_attack`"
        );
        assert!(
            armed_citizen(true).is_some(),
            "the same type with the military bit must still search"
        );
    }

    /// The flee arm's `else`: a **combat** unit still retaliates, which
    /// is the row item 464's restructure could have swallowed. Its gate
    /// is `is_worker || is_idle`, and a soldier standing idle passes the
    /// second half — so what keeps it out of the flight is the first
    /// clause, `(CIVILIAN && max_range == 0) || attack == 0`.
    #[test]
    fn a_hit_soldier_retaliates_where_a_citizen_flees() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x4100, 0x4000));
        sim.do_damage(
            Obj::Unit(foe),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        assert_eq!(
            sim.units[me].combat.target,
            Some(Obj::Unit(foe)),
            "an idle soldier answers the hit with an attack, not a flight"
        );
    }

    /// **A dead target outlives its order by the reload** —
    /// `docs/ORDERS.md` §7.12, and the arm is `Unit::fight@005fd4d0:102`
    /// returning before `:196`'s `Object::valid_target`.
    /// `Unit::do_attack@005f1b80` asks nothing about the target for a
    /// unit whose type has `attack` (both of its aliveness tests sit
    /// under `ptype->attack == 0`), so a recharging attacker whose target
    /// has just died keeps the order, and drops it on the frame the
    /// reload reaches nought — never before.
    ///
    /// Made to fail on purpose, both ways: with the validity test back in
    /// front of the reload gate the order dies on the **first** frame;
    /// with [`Sim::forget`] clearing `mandatory` again — which is the
    /// order's own field, `TargetOrder +0x1c`, and not the dead object's
    /// — a HOLD_FIRE attacker returns at `do_attack`'s stance arm and the
    /// order never dies at all.
    ///
    /// run100's own measurement is the six raiders of Great Lakes blocks
    /// 10231–10239, each dropping on its own `recharging` clock: `1/42`
    /// on 10231, `1/28` on 10233, `1/27` on 10239.
    #[test]
    fn a_recharging_attacker_keeps_a_dead_target_s_order_until_the_reload_ends() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        // HOLD_FIRE and mandatory, which is run100's raiders exactly: the
        // stance arm is what the cleared `mandatory` used to reach.
        sim.units[me].combat.stance = Stance::HoldFire;
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            true,
            false,
        );
        sim.units[me].combat.recharging = 3;
        sim.units[foe].health = 0;
        sim.forget(Obj::Unit(foe));
        assert_eq!(
            sim.units[me].combat.target, None,
            "the dead target is dropped"
        );
        assert!(
            sim.units[me].combat.mandatory,
            "the order's own `mandatory` outlives the object it named"
        );
        // `Unit::process` counts the reload down before `work`.
        for f in 0..2 {
            sim.units[me].combat.recharging -= 1;
            sim.work(me, f);
            assert_eq!(
                sim.units[me].orders.len(),
                1,
                "frame {f}: the order died while `recharging` was {}",
                sim.units[me].combat.recharging
            );
        }
        sim.units[me].combat.recharging -= 1;
        sim.work(me, 2);
        assert!(
            sim.units[me].orders.is_empty(),
            "the order outlived the reload: {:?}",
            sim.units[me].orders
        );
    }

    /// **A move whose action's target has died is only re-pathed when
    /// the walk is nearly over** (item 487, `docs/ORDERS.md` §20).
    ///
    /// `do_move@005f7b30`'s `action->type == 10` block falls out of the
    /// valid-target arm when the object is gone, and what it does then is
    /// `005f8221`-`005f825d`:
    ///
    /// ```text
    /// vector_dist(|x − move.x|, |y − move.y|)   ; the order's own dest
    /// cmp eax, 0x480 / jg   → the planner
    /// test $0x10, 0x2b4(ptype) / jne → the planner   ; a sea transport
    /// call Unit::repath
    /// ```
    ///
    /// `MoveOrder +0x4`/`+0x8` are `x`/`y` by the type record, so the
    /// distance is to where the unit was walking. Inside `0x480` the walk
    /// is pointless and the transit legs are popped; beyond it the unit
    /// keeps the order and plans, dead target and all. This crate popped
    /// them at any distance, which is what stood run100's `1/29` still
    /// 28,000 units from home for the length of its reload.
    ///
    /// Both arms are asserted, and the gate was made to fail on purpose
    /// by taking the distance test out again — which kills the far move
    /// on its first frame.
    #[test]
    fn a_dead_target_s_move_is_repathed_only_within_0x480_of_its_destination() {
        for (name, dest, survives) in [
            ("near", Pos::new(0x1000 + 0x400, 0x1000), false),
            ("far", Pos::new(0x1000 + 0x4000, 0x1000), true),
        ] {
            let (mut sim, ty) = at_war();
            let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
            let foe = put(&mut sim, 1, ty, dest);
            // The attack is the **action** under the move, which is the
            // only shape the block reads.
            sim.add_attack_order(
                me,
                Obj::Unit(foe),
                crate::orders::QueuePos::First,
                true,
                true,
            );
            sim.add_move_order(
                me,
                dest,
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::First,
                false,
            );
            assert_eq!(
                sim.units[me].orders.len(),
                2,
                "{name}: the move and its action"
            );
            sim.units[foe].health = 0;
            sim.forget(Obj::Unit(foe));
            sim.work(me, 0);
            assert_eq!(
                sim.units[me]
                    .orders
                    .iter()
                    .any(crate::orders::Order::is_move),
                survives,
                "{name}: the move at {dest:?} should {} have survived: {:?}",
                if survives { "" } else { "not" },
                sim.units[me].orders
            );
        }
    }

    /// **A squad member never searches: it mirrors its captain.**
    /// `Unit::think@005f6e40`'s first statement, above every gate in the
    /// function (`docs/COMBAT.md` §21) — a non-captain reads its captain's
    /// **action**, and takes that target if the action is an ATTACK order
    /// (`get_type() == 10`) on something it can validly attack.
    ///
    /// Three arms, each made to fail on purpose:
    ///
    /// - the captain holding an ATTACK order hands it down;
    /// - a captain whose action is a **move** hands nothing down — the
    ///   `get_type() != 10` return, and the member keeps whatever it had;
    /// - a member whose `valid_target` fails on the captain's target takes
    ///   nothing either, which is the only one of the five returns the
    ///   golden record cannot see.
    ///
    /// The captain itself is never mirrored: `captain_mirror` is a no-op on
    /// it, because `Unit::think` reaches the rest of the function instead.
    #[test]
    fn a_squad_member_mirrors_its_captain_s_attack_order() {
        let (mut sim, ty) = at_war();
        let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        sim.units[sub].captain = false;
        sim.units[sub].o_up = Some(cap);
        sim.units[cap].o_down = Some(sub);
        // Nothing to mirror while the captain is idle.
        sim.captain_mirror(sub);
        assert_eq!(sim.units[sub].combat.target, None);
        // The captain takes an attack order; the member takes the same one.
        sim.add_attack_order(
            cap,
            Obj::Unit(foe),
            crate::orders::QueuePos::New,
            false,
            false,
        );
        sim.captain_mirror(sub);
        assert_eq!(
            sim.units[sub].combat.target,
            Some(Obj::Unit(foe)),
            "the member is handed the captain's target"
        );
        // **A captain walking to its chase point still hands the target
        // down**: `get_action` skips the leading transit moves, so the
        // ATTACK order under them is still what is read.
        sim.add_move_order(
            cap,
            Pos::new(0x1080, 0x1000),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        sim.units[sub].combat.target = None;
        sim.captain_mirror(sub);
        assert_eq!(
            sim.units[sub].combat.target,
            Some(Obj::Unit(foe)),
            "the action is the attack under the transit move"
        );
        // A captain whose **action** is a move hands nothing down — the
        // `get_type() != 10` return, taken with the target still on the
        // captain, so it is the order's kind that refuses and not a
        // missing target.
        sim.add_move_order(
            cap,
            Pos::new(0x1200, 0x1000),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            true,
        );
        sim.units[sub].combat.target = None;
        sim.captain_mirror(sub);
        assert_eq!(
            (sim.units[cap].combat.target, sim.units[sub].combat.target),
            (Some(Obj::Unit(foe)), None),
            "`get_type() != 10` returns before the target is read"
        );
        // And the **member's own** `valid_target` is what is tested: an
        // ally target passes for nobody, and this is the one of the five
        // returns the golden record cannot see.
        let (mut sim, ty) = at_war();
        let cap = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let sub = put(&mut sim, 0, ty, Pos::new(0x1030, 0x1000));
        let friend = put(&mut sim, 0, ty, Pos::new(0x1100, 0x1000));
        sim.units[sub].captain = false;
        sim.units[sub].o_up = Some(cap);
        sim.units[cap].o_down = Some(sub);
        sim.add_attack_order(
            cap,
            Obj::Unit(friend),
            crate::orders::QueuePos::New,
            false,
            false,
        );
        assert!(!sim.valid_target(Obj::Unit(sub), Obj::Unit(friend)));
        sim.captain_mirror(sub);
        assert_eq!(sim.units[sub].combat.target, None);
        // A captain mirrors nothing — the arm is `!is_captain` only, and
        // `squad_captain` answers the unit itself for one.
        sim.captain_mirror(cap);
        assert_eq!(sim.units[cap].combat.target, Some(Obj::Unit(friend)));
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
    /// **The tie goes to the cell's chain head, not to the lowest object
    /// number** (§12.2, §32.1) — chapter two's own geometry, which is the
    /// case that separates the two orders.
    ///
    /// `0/9` at `(888, 8376)` looks at three identical hoplites on one
    /// world cell. Two of them tie exactly: `attack_dist` 1459 and 1468,
    /// nine units apart on a score that divides by `0xc0`, so both land
    /// in the same bucket and `compare_target` returns the same value for
    /// two undamaged figures of one type. The keep test is strictly
    /// greater on both sides, so the winner is whichever the scan reached
    /// first — and `Object::add_to_world` pushes on the **head**, so the
    /// chain reaches the newest. run112's `ATTACKORDER` on all three
    /// slingers reads `ox 8 whom 1`.
    ///
    /// Made to fail on purpose by scanning `(0..units.len())` instead of
    /// [`crate::Sim::cell_chain`] — the pick is then `1/6`, which is the
    /// answer this crate gave for six items.
    #[test]
    fn a_tied_target_goes_to_the_cell_chain_s_head() {
        use crate::combat::Profile;
        use crate::world::World;
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        let mut kind = |max_range: i32| {
            sim.add_unit_type(crate::UnitType {
                hits: 120,
                combat: Profile {
                    attack: 15,
                    max_range,
                    uber_size: 1,
                    block_radius: 48,
                    big_radius: 48,
                    combat_role: true,
                    ..Profile::default()
                },
                ..crate::UnitType::default()
            })
        };
        let slinger = kind(6);
        let hoplite = kind(0);
        let put = |sim: &mut crate::Sim, who: crate::Player, ty: usize, p: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(who, index, p, 120);
            u.ty = Some(ty);
            u.on_map = true;
            let h = sim.add_unit(u);
            sim.units[h].orders_pos = p;
            h
        };
        let a9 = put(&mut sim, 0, slinger, Pos::new(888, 8376));
        // Block 622's three, in the dump's own order — so the chain, which
        // is newest first, is `1/8, 1/7, 1/6`.
        let b6 = put(&mut sim, 1, hoplite, Pos::new(2424, 7800));
        let b7 = put(&mut sim, 1, hoplite, Pos::new(2568, 7800));
        let b8 = put(&mut sim, 1, hoplite, Pos::new(2472, 7944));
        // **Anti-vacuity, and it is the whole point of the fixture.** The
        // three must sit on one cell — otherwise the ring order decides
        // and the chain never gets asked — and two of them must tie.
        let cell = sim.units[b6].pos.cell();
        assert_eq!(
            (sim.units[b7].pos.cell(), sim.units[b8].pos.cell()),
            (cell, cell),
            "the three hoplites are not on one world cell"
        );
        assert_eq!(
            sim.cell_chain(cell),
            vec![b8, b7, b6],
            "`chain_add` no longer pushes on the head"
        );
        let me = Obj::Unit(a9);
        let bucket =
            |sim: &crate::Sim, t: usize| (sim.attack_dist(me, Obj::Unit(t)) + 8 * 0x30) / 0xc0;
        assert_eq!(
            (bucket(&sim, b6), bucket(&sim, b8), bucket(&sim, b7)),
            (9, 9, 10),
            "`1/6` and `1/8` no longer tie, or `1/7` no longer fails to"
        );
        assert_eq!(
            sim.find_nearby_target(me, 4608),
            Some(Obj::Unit(b8)),
            "the tie went to the lowest object number instead of the \
             cell chain's head"
        );
    }
}
