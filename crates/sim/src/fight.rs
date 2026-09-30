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
use crate::single::Single;
use crate::world::{Pos, UNITS_PER_CELL, vector_dist};
use crate::{Player, Sim};

/// `Object::find_nearby_target`'s fifth argument, the `flags` word
/// (`docs/COMBAT.md` §12.2, §64). [`Sim::find_melee_target`] on an
/// attack-move sets it ([`Sim::melee_search_flags`]), and so does
/// `Group::action_attack`'s retarget (§66, [`Sim::find_melee_target_with`]).
pub mod search {
    /// `& 1`: units only (the candidate's vslot `+0x8`).
    pub const UNITS: u32 = 0x1;
    /// `& 2`: buildings only (vslot `+0x1c`).
    pub const BUILDINGS: u32 = 0x2;
    /// `& 0x10`: a candidate that is not a unit scores half.
    pub const HALVE_NON_UNITS: u32 = 0x10;
    /// `& 0x20`: a candidate that is not a `Build` (vslot `+0x20`) scores
    /// half; a wall is not one.
    pub const HALVE_NON_BUILDS: u32 = 0x20;
    /// `& 0x20000`: a building must be armed (vslot `+0x120`), or under
    /// [`BUILDINGS`] a wonder or a military trainer.
    pub const ARMED: u32 = 0x20000;
}

/// What `check_target`'s guarding arm answers (`docs/COMBAT.md` §63.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Leash {
    /// The captain is attacking this target: `check_target` answers 1 at
    /// once, before its tail.
    Captain,
    /// Target and guard both inside the radius of the post.
    In,
    /// One of them is outside it: refused.
    Out,
}

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

/// **The AI guard's charge roll** — `Unit::fight@005fd4d0`, the
/// `Random::get` at `005fdcef` (`docs/COMBAT.md` §63.2): a captain whose
/// activity is a `GUARD` and whose `unit_masks` carries `0x40000` spends
/// it on every unrecharged `fight`, and an odd draw drops the attack.
pub const SITE_FIGHT_GUARD_ROLL: &str = "Unit::fight+0x824";

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

/// `Ammo::init@0067bbf0+0xae8` — a **ground** shot's scatter, x: the
/// attack-ground arm (`init:461`–`484`) has its own pair of calls, the
/// listing's `67c6d3` and `67c710` (return addresses `67c6d8`, `67c715`),
/// ahead of `find_data_z` at `67c738` (`docs/COMBAT.md` §57.4, §59.5).
pub const SITE_AMMO_GROUND_SCATTER_X: &str = "Ammo::init+0xae8";

/// `Ammo::init@0067bbf0+0xb25` — a ground shot's scatter, y.
pub const SITE_AMMO_GROUND_SCATTER_Y: &str = "Ammo::init+0xb25";

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

/// `Unit::close@0060ee50+0xcb6` — **the death animation's own draw**
/// (`docs/COMBAT.md` §42.1).
///
/// One `Random::get(0, 0xffff)`, read `% 2`, on every unit death whose
/// `dtype` is non-zero: `cur_anim = dtype * 2 + 0xd + roll % 2`, the
/// index the `DEATH_OBJS` record then prints for the life of the death
/// object. The listing is `call Random::get; and $0x80000001; lea
/// 0xd(,%ebx,2); add %eax`, with `%ebx` the `(signed char)dtype`
/// argument — so the anim and the draw are one expression.
pub const SITE_DEATH_ANIM: &str = "Unit::close+0xcb6";

/// `Unit::close@0060ee50+0xce6` — the **ammo-graphic** death's second
/// draw, taken only when `dtype == 4`.
///
/// `dtype` 4 is `do_damage`'s override for an ammo whose graphic piece
/// carries flag `0x10` (§7.1 step 6), and it replaces the anim outright:
/// `cur_anim = roll % 2 + 0x11`, then [`SITE_DEATH_ANIM_FACING`] picks
/// the facing. No capture on disk reaches it — every death in run112 is
/// `dtype` 2 — so the two sites are named from the listing and spent from
/// the reading.
pub const SITE_DEATH_ANIM_ALT: &str = "Unit::close+0xce6";

/// `Unit::close@0060ee50+0xd0a` — the third draw of a `dtype == 4` death:
/// `facing = 0xe - roll % 4`, where every other death takes the attack's
/// own angle.
pub const SITE_DEATH_ANIM_FACING: &str = "Unit::close+0xd0a";

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

/// `find_attack_pos`'s ring draw under `Unit::check_target_path`'s re-aim
/// of a fleeing target (`5e2710`, returning to `+0x445`; item 1023). Named
/// and never spent: the re-aim asks only for a unit target, whose sweep
/// draws nothing.
pub const SITE_ATTACK_POS_REVIEW: &str =
    "Unit::find_attack_pos+0xea9 < Unit::check_target_path+0x445";

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
/// What a round is fired at: an object, or the point of the shooter's
/// `ATTACK_GROUND` order (`docs/COMBAT.md` §57.4).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Aim {
    At(Obj),
    Ground(crate::orders::AttackGroundOrder),
}

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
                    // The object's own `z` (`+0xc`), which `Unit::update_z`
                    // writes as `find_tcoord_z` at its tile, clamped at zero
                    // (§46.2, item 1131).
                    z: self.world.object_z(u.pos.tile()),
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
                    z: self.world.object_z(bd.pos.tile()),
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
                z: self.world.object_z(self.units[i].pos.tile()),
                ..Side::default()
            },
            // `ObjectData::get_captain@00472400` is the object's own `o`,
            // which `do_damage` step 2 writes as the target's `damage_o`
            // (item 1131: run404's `0/7` reads 2007 from the Radar's hit
            // on 828).
            Obj::Building(b) => Side {
                building: true,
                build_proper: true,
                captain: i32::from(self.buildings[b].index),
                z: self.world.object_z(self.buildings[b].pos.tile()),
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
        // **The world-cell test** (`6486b0`..`64875b`, item 1200): the
        // tile under the point asked from — `div_3_table[x >> 6]`, the
        // tile, of the eight-argument overload's own `x/y` — answers no
        // when its surface is forest (`TData.mask & 0x30 == 0x30`). It
        // asks no domain, so a plane over a wood holds its fire:
        // chapter forty-one's Biplane on 839, at `0x7138` (`docs/GOLDEN.md`
        // §50).
        if self.world.tile_mask(at.tile()) & crate::world::tile::SURFACE
            == crate::world::tile::SURFACE_FOREST
        {
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
    /// and the whole of `check_target` answers 1 there (`0064a0eb`).
    /// Otherwise the radius is `unit_guard_respond_range × k × 0x60`,
    /// `k` 3 against a unit that is not a worker (`is_worker@0046fa10`,
    /// type `0x32`–`0x35`) when the guard carries `unit_masks & 0x40000`
    /// ([`Sim::ai_driven`], the one stand-in), else 2. Both the **target**
    /// and **the guard itself** must stand within it of the post, the
    /// order's `+0x1c/+0x20` (`0064a100`–`0064a186`, the listing: the
    /// decompiler lost both `vector_dist`s' operands).
    pub(crate) fn guard_leash(
        &self,
        u: usize,
        g: &crate::orders::GuardOrder,
        target: Obj,
    ) -> Leash {
        use crate::orders::Body;
        if !self.units[u].captain {
            let cap = self.squad_captain(u);
            if let Some(a) = self.action_of(cap)
                && matches!(self.units[cap].orders[a].body, Body::Attack(_))
                && self.units[cap].combat.target == Some(target)
            {
                return Leash::Captain;
            }
        }
        let worker = match target {
            Obj::Unit(t) => self.units[t]
                .ty
                .is_some_and(|ty| (0x32..=0x35).contains(&self.unit_types[ty].type_index)),
            Obj::Building(_) => true,
        };
        let k = if !worker && self.ai_driven(self.units[u].owner) {
            3
        } else {
            2
        };
        let r = self.tuning.unit_guard_respond_range * k * 0x60;
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
        &self,
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
    /// (`docs/COMBAT.md` §72): the candidate is refused when it is out of
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
        self.attack_dist(me, cand) > 0xc0
    }

    /// `ObjectData::valid_target_const` + `Object::valid_target` (§12.1), as
    /// far as the simulation's state reaches: not mine, at war, active, on the
    /// map, **and seen**; and the air ladder for a plane target, less its
    /// helicopter and `is(0x132)` arms (`docs/COMBAT.md` §61.2).
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
            // SEAM: a **helicopter** target (`unit_flags & 0x20`) takes
            // `006474ee`'s arm instead, which refuses four classes of searcher
            // (two by vtable, the missile, and `is(0x130)`); this crate keeps
            // it as "ranged and not a missile". And `0064756b`'s
            // `is(0x132)` arm, which refuses a non-air searcher that cannot
            // carry aircraft, is not built. No capture reaches either.
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
        if let Obj::Building(b) = target {
            return self.build_is_seen(b, who);
        }
        if self.world_sees(self.pos_of(target), who) {
            return true;
        }
        // `return (visible >> who) & 1` — the fallback, and the whole of
        // item 457.
        self.visible_of(target) & Self::who_bit(who) != 0
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
    pub(crate) fn build_is_seen(&self, b: usize, who: crate::Player) -> bool {
        let bd = &self.buildings[b];
        if bd.owner == who {
            return true;
        }
        let mask = self.seen_ally_mask(who);
        (bd.started || mask & Self::who_bit(bd.owner) != 0) && bd.ever_seen & mask != 0
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
        if angle != self.units[i].movement.heading {
            self.unit_set_angle(i, angle);
        }
        self.swing_anim(i, direct);
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
            // every piece it has not measured — and `Object::fire_ammo`'s
            // unit arm starts it 100 above the figure (§46.1).
            let sz = self.ground_z(from) + 100;
            self.fire_ammo(me, target, direct, frame, from, sz);
        }
        // Reload.
        self.units[i].combat.recharging = self.reload_frames(i);
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
        combat::recharge(p.recharge, out, p.is(role::BOMBARD)) as u8
    }

    /// **`Unit::set_attack(−1, −1)`** (`005fce70`): the aimed figures,
    /// `0 .. guy_mark`, forget their target, and with no target the pivot
    /// test is not reached. `do_attack_ground`'s shot has no object to aim
    /// at, which is why run146's catapult prints `ox −1` from 781.
    pub(crate) fn clear_attack(&mut self, i: usize) {
        let n = self.units[i].guys.len().min(anim::SQUAD_SIZE);
        for g in &mut self.units[i].guys[..n] {
            g.aim = None;
        }
    }

    /// `do_attack_ground`'s facing: the bearing, or for a broadside type
    /// (`unit_flags & 0x40`, [`uflags::SIDEWAYS`]) whichever quarter turn
    /// off it is nearer the heading. Unlike `fight`'s
    /// ([`Sim::attack_angle`]) there is no patrol-boat exception: the
    /// point has no domain to ask.
    pub(crate) fn broadside_angle(&self, i: usize, direct: Angle) -> Angle {
        let sideways = self.units[i]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag(uflags::SIDEWAYS));
        if !sideways {
            return direct;
        }
        const QUARTER: i32 = 0x4000_0000;
        let heading = self.units[i].movement.heading.0;
        let off = |a: i32| {
            let d = heading.wrapping_sub(a) as u32;
            if d > 0x8000_0000 { !d } else { d }
        };
        let minus = direct.0.wrapping_sub(QUARTER);
        let plus = direct.0.wrapping_add(QUARTER);
        Angle(if off(minus) < off(plus) { minus } else { plus })
    }

    pub(crate) fn swing_anim_pub(&mut self, i: usize, angle: Angle) {
        self.swing_anim(i, angle);
    }

    /// `do_attack_ground`'s `Object::fire_ammo(−1, −1)`: deferred to the
    /// release event for a unit its animation launches (§9.0), whose arm
    /// reads the order's point ([`Sim::guy_release_events`]); from the
    /// unit's own square otherwise, as `fight`'s strike does.
    pub(crate) fn fire_ground(
        &mut self,
        i: usize,
        g: crate::orders::AttackGroundOrder,
        direct: Angle,
        frame: i64,
    ) {
        if self.max_range_of(Obj::Unit(i)) == 0 || self.launches_from_anim(i) {
            return;
        }
        let from = self.units[i].pos;
        let sz = self.ground_z(from) + 100;
        self.fire_ammo_ground(Obj::Unit(i), g, direct, frame, from, sz, 0, false);
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
        let n = self.units[i].guys.len().min(anim::SQUAD_SIZE);
        for g in &mut self.units[i].guys[..n] {
            g.aim = Some(target);
        }
        let Some(ty) = self.units[i].ty else {
            return false;
        };
        let Some(nodes) = self.art.pivots.get(&self.unit_types[ty].type_index) else {
            return false;
        };
        let live = match target {
            Obj::Unit(t) => self.units[t].alive(),
            Obj::Building(_) => true,
        };
        if nodes.is_empty() || self.profile(Obj::Unit(i)).max_range == 0 || !live || n == 0 {
            return false;
        }
        // Every aimed figure's own `set_all_pivots`, and the answer is the
        // last one's (`005fce70`); each writes its own turret.
        let mut can = false;
        for g in 0..n {
            can = self.set_all_pivots(i, g, Some(target));
        }
        can
    }

    /// **`Guy::set_all_pivots@005d8bc0`** — can every restricted node of
    /// figure `g` bear on `target` from where the unit stands?
    ///
    /// The bearing is measured per node, from the **unit's** position
    /// (`+0x10`/`+0x14` of `objects[who][o]`, read through the figure's
    /// own `who` and `o`) **plus the node's own vector**
    /// ([`crate::pivot::offset`], `docs/COMBAT.md` §54), to the target's,
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
    /// SEAM: the node's vector is pinned for the Chariot's figure only
    /// (`pivot::NODES`); any other piece bears from the unit's point.
    /// SEAM: an `ATTACK_GROUND` order with no aim bears on the order's
    /// point in the original; this crate writes nothing for it.
    pub(crate) fn set_all_pivots(&mut self, i: usize, g: usize, target: Option<Obj>) -> bool {
        let Some(ty) = self.units[i].ty else {
            return false;
        };
        let Some(nodes) = self
            .art
            .pivots
            .get(&self.unit_types[ty].type_index)
            .cloned()
        else {
            return false;
        };
        if nodes.is_empty() || g >= self.units[i].guys.len() {
            return false;
        }
        self.units[i].guys[g].turret.node_flags = 0;
        self.units[i].guys[g].turret.des_flags = 0;
        let Some(target) = target.filter(|&t| self.active(t)) else {
            return true;
        };
        let (from, to) = (self.units[i].pos, self.pos_of(target));
        let guy = self.units[i].guys.get(g).copied();
        let facing = match guy.and_then(|x| x.follow) {
            Some(f) => f.facing,
            None => self.units[i].movement.facing,
        };
        let piece = guy.map_or(-1, |x| x.gpiece);
        let mut can = true;
        for node in 4..4 + nodes.len() as i32 {
            // `005d8e0a`–`005d8e35`: the target less the unit's point less
            // the node's truncated vector, each component on its own.
            let (vx, vy) = crate::pivot::offset(piece, node, facing);
            let bearing = find_angle(to.x - from.x - vx, to.y - from.y - vy);
            let mut deg =
                crate::movement::angle_to_degrees(Angle(bearing.0.wrapping_sub(facing.0)));
            if deg > 180 {
                deg -= 360;
            }
            can &= (-45..=45).contains(&deg);
            let (lo, hi) = nodes.get(&node).copied().unwrap_or((0, 0));
            let inside = if lo < hi {
                lo <= deg && deg <= hi
            } else {
                deg >= lo || deg <= hi
            };
            can &= inside;
            if inside && let Some(k) = usize::try_from(node - 4).ok().filter(|&k| k < 4) {
                let t = &mut self.units[i].guys[g].turret;
                t.des[k] = bearing.0.wrapping_sub(facing.0);
                t.des_flags |= 1 << k;
                if turret_near(t.des[k], t.angles[k]) {
                    t.node_flags |= 1 << k;
                }
            }
        }
        can
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
    fn building_side(&self, i: usize, target: Obj) -> Option<Angle> {
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
        let t = &self.unit_types[ty];
        if !t.cols.flag(uflags::SIDEWAYS) {
            return direct;
        }
        /// `TypeIndex::PATROLBOAT`.
        const PATROLBOAT: i32 = 0x185;
        if t.type_index == PATROLBOAT && matches!(self.profile(target).domain, Domain::Sea) {
            return direct;
        }
        const QUARTER: i32 = 0x4000_0000;
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
    ///   `param_3 = 0`, with its own per-figure damage;
    /// - everything else: `CHAR_ATTACK1`, `param_3 = 1`.
    ///
    /// SEAM: the middle two are not modelled — three elephant types and
    /// one target type — and neither draws, so the cost is which slot
    /// plays rather than a word. The `z` arm compares the direct bearing,
    /// `angle` here, against `Unit::fight`'s own attack angle, which is the
    /// heading [`Sim::attack_angle`] has just set. That is the bearing, or
    /// for a sideways (`g`) ship a quarter turn off it, so since item 535
    /// a ship rocks to the side it turned to (`docs/COMBAT.md` §49). The
    /// second arm's `0x185` is the **attacker's** own `TypeIndex`, not the
    /// target's: `local_20` is written from `this->ptype + 4` at
    /// `fight:609` and not again before `:805`.
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
            .is_some_and(|g| self.art.releases.contains_key(&g.gpiece))
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
        if crate::anim::category(g.anim) != crate::anim::ATTACK2 || g.end_time == 0 {
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
    fn scatter_landing(&mut self, at: Pos, s: i32, sites: (&str, &str)) -> Pos {
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
        let mut slot = 0;
        for q in &self.projectiles {
            if q.slot == slot {
                slot += 1;
            } else {
                break;
            }
        }
        p.slot = slot;
        let at = self.projectiles.partition_point(|q| q.slot < slot);
        self.projectiles.insert(at, p);
    }

    /// `TerrainOut::find_data_z(x, y, 0)` — [`crate::World::data_z`] — with
    /// the read counted in [`Sim::ground_inexact`] when it touched a
    /// corner this crate cannot pin to the original's single.
    pub fn ground_z(&mut self, at: Pos) -> i32 {
        let (z, exact) = self.world.data_z(at.x, at.y);
        if !exact {
            self.ground_inexact += 1;
        }
        z
    }

    /// `Ammo::init`'s `ez` (`+0x20`), `0x67c93a`–`0x67c9a4`: the target
    /// object's own `z` — for a unit `find_tcoord_z` at its tile, which is
    /// [`crate::World::tile_z`] — plus **75** when the shot rolls, the
    /// first figure's ground for an air unit, and never below zero (§46.1).
    fn aim_z(&mut self, target: Obj, rolling: bool) -> i32 {
        let at = self.pos_of(target);
        let mut z = self.world.tile_z(at.tile()) + if rolling { 0x4b } else { 0 };
        if matches!(self.profile(target).domain, Domain::Air) {
            z = self.ground_z(at);
        }
        z.max(0)
    }

    /// `Object::fire_ammo`'s round, from `Unit::fight`. SEAM: the package
    /// it hands `Ammo::init` is unread here, so its `angle` is taken as 0;
    /// only a strafer's landing reads it, and a strafer's round is its
    /// animation's (`docs/ORDERS.md` §39.2), never this one.
    fn fire_ammo(
        &mut self,
        shooter: Obj,
        target: Obj,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
    ) {
        self.fire_ammo_aim(shooter, Aim::At(target), angle, frame, launch, sz, 0, false);
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
        );
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
    fn fire_ammo_aim(
        &mut self,
        shooter: Obj,
        aim: Aim,
        angle: Angle,
        frame: i64,
        launch: Pos,
        sz: i32,
        node: i8,
        harmless: bool,
    ) {
        // **A round at an aircraft rolls to hit** (item 1102,
        // `docs/COMBAT.md` §81): `Ammo::init`'s air arm, ahead of the
        // Bomber's and the scatter's; its miss is the same flag `0x10`.
        let harmless = match aim {
            Aim::At(t) => self.air_round_misses(shooter, t) == Some(true) || harmless,
            Aim::Ground(_) => harmless,
        };
        let p = self.profile(shooter);
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
            let total_time = combat::fall_time(sz, ez).max(1);
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
        let acc = match target {
            None => combat::accuracy(
                p.to_hit,
                p.attenuate,
                vector_dist(target_pos.x - launch.x, target_pos.y - launch.y),
            ),
            Some(t) => combat::accuracy(p.to_hit, p.attenuate, self.attack_dist(shooter, t)),
        };
        let land_unit = matches!(target, Some(Obj::Unit(_))) && matches!(tp.domain, Domain::Land);
        // **A strafing type's round is exact** (`0x67c33a`): a unit
        // shooter with `unit_flags & 0x400000` (flag `w`, the Fighter
        // line) takes no scatter and no draw, whatever it aims at
        // (`docs/ORDERS.md` §39).
        let strafes = self.strafes(shooter);
        let s = match ground {
            Some(g) if g.sea => 0,
            Some(_) => combat::scatter(&self.tuning, acc, true, false, strafes),
            None => combat::scatter(&self.tuning, acc, land_unit, p.has(mask::MISSILE), strafes),
        };
        // A building target shot by a non-siege unit: aim at the near face.
        let mut aim = target_pos;
        if matches!(target, Some(Obj::Building(_))) && !p.siege && s != 0 {
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
        let landing = self.strafe_walk(shooter, target, landing, node);
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
        if let Some(Obj::Unit(t)) = target
            && {
                let k = self.order_type(t);
                crate::orders::index::is_move_family(k) || crate::orders::index::is_air_family(k)
            }
        {
            let u = &self.units[t];
            let angle = u.movement.heading;
            let avg = u
                .guys
                .first()
                .and_then(|g| g.follow)
                .map_or(u.movement.body.avg_speed, |f| f.body.avg_speed);
            landing = clamp(
                Pos::new(
                    landing.x + crate::movement::sin_component(angle, avg) * total_time,
                    landing.y - crate::movement::cos_component(angle, avg) * total_time,
                ),
                &self.world,
            );
        }
        let _ = frame;
        // **A strafer's round never rolls** (`67c548`..`67c557`, item
        // 1200): the unit-strafer test jumps past `67c633`, where flag `4`
        // and the `0x4b` over a land unit are set. Chapter forty-one's
        // Biplane on 1529, whose round lands behind it, short of `0/9`.
        let rolling = land_unit && !strafes;
        let ez = match target {
            Some(t) => self.aim_z(t, rolling),
            None => self.ground_z(target_pos).max(0),
        };
        self.add_ammo(combat::Projectile {
            shooter,
            owner: self.owner_of(shooter),
            target,
            launch,
            landing,
            cur_time: 0,
            total_time,
            accuracy: acc,
            angle,
            splash_area: p.splash_area,
            num_guys: 1,
            // `Ammo::init`'s flag `4` (§42.2): not a ground shot — the
            // `ATTACK_GROUND`/`AIR_ATTACK_GROUND` test is `target` being
            // `None` here — and the target a land-domain unit. The third term,
            // "the piece is not lofted", is the ammo flag `8` this crate
            // loads no art for; a siege shot is a ground shot and so
            // never reaches the question.
            rolling,
            missed: false,
            harmless,
            air: target.is_some() && matches!(tp.domain, Domain::Air),
            sz,
            ez,
            v1z: combat::arc_v1z(sz, ez, total_time),
            slot: 0,
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
                u.combat.damage_who = i32::try_from(owner).unwrap_or(-1);
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
        // Step 6: `dtype`, which is the death animation's gate (§42.1).
        // `get_damage` leaves 2, or 3 for an EXPLOSIVE/BOMBARD attacker;
        // `do_damage` then overrides it to **4** for an ammo whose graphic
        // piece carries flag `0x10`, and to **1** for a Build-proper
        // target. The `0x10` arm is not reachable here: this crate loads
        // no ammo flags (§42.5), and every shot in every capture on disk
        // is an ordinary one.
        let dtype = if tt.build_proper {
            1
        } else {
            combat::death_type(ap.obj_masks, !matches!(attacker, Obj::Unit(_)))
        };
        // Step 7: take.
        let taken = self.take_damage_typed(target, dealt, attacker, frame, dtype);
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
        if let Obj::Building(b) = target {
            self.spill_onto_builders(attacker, b, angle, ammo, count, frame);
        }
        Some(taken)
    }

    /// **A hit on a building spills onto the hands at work on it**
    /// (`Object::do_damage@0064a480`'s tail, `64c10c`..`64c4e3`, item
    /// 1182, `docs/GOLDEN.md` §50). Reached for a building target only
    /// (`64bec5`, `is_build`), after `take_damage`, whether it killed or
    /// not. For each cell of `circle_x/y[..circle_radius[1]]` round the
    /// building's cell, its object chain, in chain order:
    ///
    /// ```text
    /// an enemy of the attacker's (Search::valid_search mode 3), active and
    ///   on the map, whose FRONT order is BUILD_AT or REPAIR (valid_filter
    ///   9, UnitData::order_type) and whose action's target is this
    ///   building (valid_filter 10);
    /// not a Korean's under KOREAN_BUILD_UNDER_FIRE (has_tribe_bonus 0x10);
    /// vector_dist(it, building) <= max(x_size, y_size) * 192;
    /// the attacker: air, none; sea and is_siege, none (`64c3b3`,
    ///   ObjectData::is_siege@0046ef90);
    ///   a land siege type: count / 4, no reach test;
    ///   else vector_dist(it, building) <= max(max_range * 192, 0x180):
    ///   count / 8;
    /// do_damage(attacker, it, angle, ·, ammo, count', splash 0, quiet 1)
    /// ```
    ///
    /// Both divisions are the sign-fixed shifts of the listing (`cdq; and
    /// edx, 3|7; add; sar`), toward zero. SEAM: `num_guys` (§7.1), which
    /// this crate's `do_damage` does not carry, is passed through there.
    fn spill_onto_builders(
        &mut self,
        attacker: Obj,
        b: usize,
        angle: Angle,
        ammo: bool,
        count: i32,
        frame: i64,
    ) {
        let ap = self.profile(attacker);
        let bp = self.profile(Obj::Building(b));
        let at = self.owner_of(attacker);
        let centre = self.buildings[b].pos;
        let home = centre.cell();
        let circ = crate::ai_place::circle();
        for k in 0..circ.radius[1] {
            let cell = crate::Cell::new(home.x + circ.x[k], home.y + circ.y[k]);
            if cell.x < 0
                || cell.y < 0
                || cell.x >= self.world.width()
                || cell.y >= self.world.height()
            {
                continue;
            }
            for u in self.cell_chain(cell) {
                let unit = &self.units[u];
                if !self.is_enemy(unit.owner, at) || !unit.alive() || !unit.on_map {
                    continue;
                }
                let front = unit.orders.front().map(|o| o.body);
                if !matches!(front, Some(crate::orders::Body::Build(x) | crate::orders::Body::Repair(x)) if x == b)
                {
                    continue;
                }
                if self.nation[unit.owner as usize].koreans
                    && self.tuning.korean_build_under_fire != 0
                {
                    continue;
                }
                let d = vector_dist(unit.pos.x - centre.x, unit.pos.y - centre.y);
                if d > bp.x_size.max(bp.y_size) * 192 {
                    continue;
                }
                let share = match ap.domain {
                    Domain::Air => continue,
                    Domain::Sea if ap.siege => continue,
                    Domain::Land if ap.siege => (count + ((count >> 31) & 3)) >> 2,
                    _ => {
                        let reach = (self.max_range_of(attacker) * 192).max(0x180);
                        if d > reach {
                            continue;
                        }
                        (count + ((count >> 31) & 7)) >> 3
                    }
                };
                self.do_damage(
                    attacker,
                    Obj::Unit(u),
                    angle,
                    ammo,
                    share,
                    false,
                    true,
                    frame,
                );
            }
        }
    }

    /// `Object::take_damage` (§7.2) on a unit figure or a building.
    #[cfg(test)]
    fn take_damage(&mut self, target: Obj, hit: Sixteenths, by: Obj, frame: i64) -> Taken {
        // Every caller outside `do_damage` is a test or the attrition
        // path, and `Object::die`'s `dtype` there is 0 — the argument
        // `Sim::attrition_tick` and the scripting paths pass, and the one
        // value `Unit::close`'s arm refuses (§42.1).
        self.take_damage_typed(target, hit, by, frame, 0)
    }

    /// [`Self::take_damage`] with `Object::take_damage`'s own `dtype`
    /// argument, which it forwards to `T.die(dtype, gpiece, angle)` and
    /// which is the whole of the death draw's gate (§42.1).
    fn take_damage_typed(
        &mut self,
        target: Obj,
        hit: Sixteenths,
        _by: Obj,
        _frame: i64,
        dtype: i32,
    ) -> Taken {
        // `Object::take_damage@00652020:67-71`, first after the sixteenth
        // floor: a hit by anything but attrition (`param_5 == 0`), at
        // difficulty below 2, stamps the **struck object's owner's**
        // `frame_attacked` — human or computer, the leader of whatever was
        // hit. `Army::find_target`'s difficulty gate reads it for 7,200
        // frames (`docs/ARMY.md` §12, `docs/AI.md` §71).
        if self.ai_difficulty() < 2 {
            let owner = self.owner_of(target) as usize;
            if let Some(l) = self.ai.get_mut(owner) {
                l.frame_attacked = _frame;
            }
        }
        // `Object::take_damage@00652020:306-311`, ahead of the accumulate:
        // a unit hit by anything but attrition (`param_5 == 0`) marks its
        // whole squad in danger. Attrition never comes through here.
        if let Obj::Unit(i) = target {
            self.set_in_danger(i);
        }
        match target {
            Obj::Unit(i) => {
                // **The threshold is the figure's share of the squad, not
                // the squad's own number** (§7.2 step 8, §7.3, §41.4).
                // `myhits` is written onto every figure of a squad and
                // `take_damage` divides it on the way in, so a hoplite of
                // `UBER_SIZE` 3 and `HITS` 120 falls at 40; this crate
                // passed the whole 120 and its figures absorbed a squad's
                // worth apiece. run112's `1/8` reached `damage` 51 on
                // block 685 still standing where the original's died on
                // 683 at its fortieth point, and `extra 1/8` was that.
                //
                // `health` stays the **complement** of the dump's
                // `damage` against the squad-sized `myhits`, which is what
                // item 484's row pinned on 245,679 field-frames; the
                // accumulated whole points therefore go into `take` as
                // `max_health - health` and the share beside them.
                let u = &self.units[i];
                let uber = self.profile(target).uber_size;
                let alone = u.combat.captain == i32::from(u.index) && u.squad_size == 1;
                let share = combat::share(u.max_health, uber, alone);
                let u = &self.units[i];
                let (taken, _, frac) =
                    combat::take(u.max_health - u.health, u.damage_frac, share, hit);
                let lost = match taken {
                    Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
                };
                let u = &mut self.units[i];
                u.damage_frac = frac;
                u.health -= lost;
                if matches!(taken, Taken::Died { .. }) {
                    u.health = u.health.min(0);
                    // `Object::die` is `close()` — the draw arm below —
                    // and **then** the slot hold, which is not gated on
                    // the draw arm's `dtype` at all (§42.3).
                    self.relink_squad(i);
                    self.close_unit(i, dtype, _frame);
                    self.hold_dead_slot(i);
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
            // **`city_flags |= 0xe`** (`Object::take_damage@00652020`, the
            // `orw $0xe, 0x4(%eax)` at `00652561`; `docs/COMBAT.md` §69):
            // under the same `param_5 == 0 && who != param_8` as the latch,
            // and whatever the latch's peasant arm decided, a building that
            // belongs to a city (`BuildData +0x72 >= 0`) marks its city
            // under attack, attacking and ever attacked. The first of the
            // three is the city heal's veto.
            if let Some(c) = self.buildings[b].city {
                let cd = &mut self.cities[c];
                cd.no_heal = true;
                cd.attacking = true;
                cd.ever_attacked = true;
            }
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

    /// **`Unit::close@0060ee50`'s squad relink** (`docs/COMBAT.md` §47.2),
    /// from the listing, `0060f28e`-`0060f758`. It sits above the death
    /// draw's `dtype` gate and under nothing but the slot's `flags & 1`, so
    /// it runs on every death.
    ///
    /// The dying figure is taken out of its chain and put back **at the
    /// tail**, where `Unit::repair_damage@0060de10` finds its slot to regrow
    /// a figure into:
    ///
    /// ```text
    /// if o_up >= 0:  above.o_down = this.o_down ; edi = 0
    /// if o_down >= 0:
    ///     below.o_up = this.o_up                 ; a dying head's -1
    ///     if below.flags & 1:  goto append
    /// if edi != 0:   the whole-squad arm         ; nothing left to lead
    /// append:
    ///     tail = walk o_down from this.o_down, or this if it has none
    ///     if tail != this:  tail.o_down = this ; this.o_down = -1 ; this.o_up = tail
    ///     else:             above.o_down = this ; this.o_down = -1 ; this.o_up = above
    /// ```
    ///
    /// So a dying tail leaves its captain's `o_down` where it was, and a
    /// dying **head** hands the squad to the figure below it, whose `o_up`
    /// is now the head's own −1: `UnitData::is_captain` is `o_up < 0`, and
    /// the new head is a captain from this frame. run112 prints both. After
    /// `0/11` dies on 729, `0/10` still carries `o_down 11`. After `1/6` dies
    /// on 743, `1/7` carries `o_up -1` and `o_down 8`, a chain of 7 → 8 → 6.
    ///
    /// This crate caches the captain in two places, [`crate::Unit::captain`]
    /// and [`combat::State::captain`], which the original reads live
    /// (`UnitData::get_captain@00610ab0` walks `o_up`). Both are re-pointed
    /// down the whole chain here, dead slots included, because a dead slot's
    /// `get_captain` walks to the new head as well.
    pub(crate) fn relink_squad(&mut self, i: usize) {
        let (up, down) = (self.units[i].o_up, self.units[i].o_down);
        let head = up.is_none();
        if let Some(a) = up {
            self.units[a].o_down = down;
        }
        let mut append = !head;
        if let Some(b) = down {
            self.units[b].o_up = up;
            append |= self.units[b].alive();
        }
        if !append {
            return;
        }
        let mut tail = i;
        let mut at = down;
        // Bounded by the list: a cycle would hang the original, and here it
        // stops rather than spins.
        for _ in 0..self.units.len() {
            let Some(n) = at else { break };
            tail = n;
            at = self.units[n].o_down;
        }
        if tail != i {
            self.units[tail].o_down = Some(i);
            self.units[i].o_up = Some(tail);
        } else if let Some(a) = up {
            self.units[a].o_down = Some(i);
        }
        self.units[i].o_down = None;
        if head && let Some(b) = down {
            self.units[b].captain = true;
            let c = i32::from(self.units[b].index);
            let mut at = Some(b);
            for _ in 0..self.units.len() {
                let Some(n) = at else { break };
                self.units[n].combat.captain = c;
                at = self.units[n].o_down;
            }
        }
    }

    /// **`Unit::set_in_danger(this, 0)@005fcfb0`** — the squad's
    /// in-danger mark (`docs/ORDERS.md` §22).
    ///
    /// It climbs `o_up` to the captain without asking whether a figure is
    /// alive, then walks `o_down`. Each figure it reaches gets
    /// `unit_masks |= 4` ([`crate::Unit::in_danger`]) and
    /// `guy_flags |= 0x20` on every guy ([`crate::Unit::guy_flag_0x20`]).
    /// The walk stops at the first figure whose slot is not active
    /// (`flags & 1`). The captain it starts from is marked unasked.
    pub(crate) fn set_in_danger(&mut self, u: usize) {
        let mut at = u;
        // Bounded by the list, as `relink_squad`'s walks are: a cycle
        // would hang the original, and here it stops.
        for _ in 0..self.units.len() {
            let Some(up) = self.units[at].o_up else { break };
            at = up;
        }
        let mut cur = Some(at);
        for step in 0..self.units.len() {
            let Some(n) = cur else { break };
            if step > 0 && !self.units[n].alive() {
                break;
            }
            self.units[n].in_danger = true;
            self.units[n].guy_flag_0x20 = true;
            cur = self.units[n].o_down;
        }
    }

    /// **`Unit::close@0060ee50`'s death-animation arm** (`docs/COMBAT.md`
    /// §42.1) — one `Random::get(0, 0xffff)` per death, and the
    /// `DEATH_OBJS` record the dump then prints.
    ///
    /// The listing's gate, in its own order (`0060fa4c`-`0060fa8f`): the
    /// slot answers `SubObjectData::is_active` and `UnitData::is_on_map`,
    /// and `unit_masks & 1` — the decoy bit — is clear. Above it the
    /// function's own `local_1c`, which is set only on the arm that
    /// unblocks four tiles for a **Merchant**, a Dutch Merchant or a Fur
    /// Trapper (type index `0x3d`, `0x3e`, `400`); those three take no
    /// death draw. And above everything, `dtype != 0`: the argument
    /// `Object::die` was called with, which combat always fills and the
    /// disband and scripting paths leave at zero.
    ///
    /// **`is_active` is the slot's allocation flag, not its health.** The
    /// object is still allocated and still on the map while `close` runs
    /// — it is `Object::close`, at the foot of this function, that takes
    /// it off — so the gate this crate applies is `on_map` and the decoy
    /// bit, and never [`crate::Unit::alive`], which the caller has
    /// already cleared.
    fn close_unit(&mut self, i: usize, dtype: i32, frame: i64) {
        if dtype == 0 {
            return;
        }
        let u = &self.units[i];
        if !u.on_map || u.decoy {
            return;
        }
        if self.profile(Obj::Unit(i)).is(combat::role::MERCHANT) {
            return;
        }
        self.mark(SITE_DEATH_ANIM);
        let roll = self.rng.roll() % 2;
        let mut cur_anim = combat::death_anim(dtype, roll);
        if dtype == 4 {
            // The ammo-graphic death **replaces** the anim the first draw
            // chose with one from a second, and rolls its own facing
            // where every other death takes the attack's angle. The first
            // draw is still spent, and its value discarded. Unreachable
            // from any capture on disk (this crate loads no ammo flags),
            // and spent here so the stream is right the day one is.
            self.mark(SITE_DEATH_ANIM_ALT);
            let alt = self.rng.roll() % 2;
            cur_anim = combat::death_anim(4, alt);
            self.mark(SITE_DEATH_ANIM_FACING);
            let _ = self.rng.roll();
        }
        // **An aircraft shot down falls, and leaves no death object**
        // (item 1102, `docs/COMBAT.md` §81): `Objects::kill_guy`'s arm for
        // a guy of `CAT` Air that is not a missile.
        if self.crashes(i) {
            return;
        }
        let u = &self.units[i];
        self.deaths.push(combat::Death {
            who: i32::from(u.owner),
            o: i32::from(u.index),
            first_frame: frame,
            cur_anim,
        });
    }

    /// **`Object::die@00647080`'s tail** — `hold_frames = max(hold_frames,
    /// 1, for every live ammo this object fired: nuke_effect[0x108] + 1 +
    /// total_time − cur_time)` (§11, §42.3). The slot is held until the
    /// dead archer's last arrow has landed.
    ///
    /// The ammo is matched on its **shooter**: `+0x3c`/`+0x40`, which
    /// `Ammo::init` fills from the firing object, and the same pair
    /// `Ammo::inc_time` reads for its own per-frame bump. `nuke_effect`'s
    /// term is an effect-table entry this crate does not load and is
    /// taken as zero (§42.5); no capture on disk has a nuke.
    ///
    /// **And the `hold_frames` it maxes against is already
    /// [`Self::CLOSE_HOLD`]** (§59.3): `die` calls `close` through vslot
    /// `+0x150` first, and `Unit::close` ends in `Object::close`, whose
    /// last write to an active object is `hold_frames = 0x1e`. So every
    /// death holds its number thirty frames, whatever `dtype` it carried —
    /// the transport that dies putting its passenger ashore
    /// ([`Sim::disembark`]), attrition, the upgrade's squad trim.
    pub(crate) fn hold_dead_slot(&mut self, i: usize) {
        let mut hold = Self::CLOSE_HOLD;
        for p in &self.projectiles {
            if p.shooter == Obj::Unit(i) {
                hold = hold.max(p.total_time - p.cur_time + 1);
            }
        }
        self.units[i].hold_frames = hold;
    }

    /// **`Object::close@00647160`'s hold**: the last write it makes to an
    /// object that was active, after `Objects::remove`, is
    /// `hold_frames = 0x1e` — on both of its arms, so on every close
    /// (`docs/COMBAT.md` §59.3). `Objects::process_all` takes one off per
    /// frame, and `Objects::find_free` will not hand the number out until
    /// it is zero.
    pub(crate) const CLOSE_HOLD: i32 = 0x1e;

    /// **`DeathObj::inc_time@008d5240`'s first statement**, and
    /// `Ammo::inc_time@0067d380`'s: every frame, a death object bumps its
    /// own object's `hold_frames`, and so does every shot in flight whose
    /// shooter is no longer active (§42.3).
    ///
    /// Both are on dead slots only, which is why the dump prints
    /// `hold_frames 0` on every living unit of every capture — the row
    /// `crate::diff::compare` now carries.
    pub(crate) fn hold_frames_tick(&mut self) {
        for k in 0..self.deaths.len() {
            let (who, o) = (self.deaths[k].who, self.deaths[k].o);
            // The object in the slot, dead or not: `unit_by_o` finds only
            // the living, and this bump is on the dead (§59).
            if let Some(u) = self
                .units
                .iter()
                .rposition(|u| i32::from(u.owner) == who && i32::from(u.index) == o)
            {
                self.units[u].hold_frames += 1;
            }
        }
        for k in 0..self.projectiles.len() {
            let Obj::Unit(s) = self.projectiles[k].shooter else {
                continue;
            };
            if !self.active(Obj::Unit(s)) {
                self.units[s].hold_frames += 1;
            }
        }
    }

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
        // **A busy unit that is not on duty does not answer the hit**
        // (item 1040, `docs/COMBAT.md` §70): every way past the flee arm
        // reaches `600863`, `call on_duty@005fff70; jne 600877`, then
        // `cmp %eax, -0x1c(%ebp)` — the front order's type, `local_20`,
        // stored at `600150` — `jne 600b16`, the return. So a unit with
        // any order retaliates only if it is on duty. The one way round it
        // is the action-is-an-attack arm (`600516`, `jmp 600877`), which
        // this crate carries as the `target` test above; an attack action
        // with no target keeps the old path (SEAM).
        if front.is_some() && !self.action_is_attack(responder) && !self.on_duty(responder) {
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
    fn action_is_attack(&self, u: usize) -> bool {
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
        let flags = self.melee_search_flags_with(i, word);
        self.find_nearby_target_with(me, radius, flags)
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
    /// SEAM: `do_move`'s own call (`005f7b30`) passes 1 or 2 as the
    /// caller's word, and this crate has no such call (§37.2).
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
        let cols = self.units[i].ty.map(|t| self.unit_types[t].cols);
        let mut flags = if cols.is_some_and(|c| c.flag(uflags::SIEGE)) {
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
        let must_reach = match attacker {
            Obj::Unit(i) => {
                let c = self.units[i].combat;
                c.stance == Stance::StandGround || c.entrenched || (ap.packs && !c.packed)
            }
            Obj::Building(_) => false,
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
        // SEAM: the cavalry archer's call (`param_4`) is never guarding;
        // this crate's search has no such caller.
        let guard = match attacker {
            Obj::Unit(i) => self.guard_activity(i).map(|g| (i, g)),
            Obj::Building(_) => None,
        };
        let centre = guard.map_or(at, |(_, g)| g.guard).cell();
        // `local_54`, `Unit::on_duty` of a unit searcher (`006490b6`):
        // `check_target`'s third argument.
        let duty = match attacker {
            Obj::Unit(i) => self.on_duty(i),
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
                        // **The `flags` filter** (`00649430`–`00649515`),
                        // above `check_target` and so above `near`.
                        if !self.search_admits(o, flags) {
                            continue;
                        }
                        // **`check_target`'s head** (§72): a candidate
                        // in another region, or any for a defensive unit
                        // off duty with an order, must be in range.
                        if let Obj::Unit(i) = attacker
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
                        if leash != Some(Leash::Captain) && self.poor_target(attacker, o) {
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
                        // crate does not flag (§60.4). A building needs the
                        // target in range or worth waiting for.
                        // A guard deems in range only an armed candidate,
                        // and under `unit_masks & 0x40000` only one without
                        // `ANTI_AIR` (`:346`–`352`); the rest it must reach.
                        let guard_reach = guard.is_some()
                            && (self.attack_of(o) == 0
                                || (self.ai_driven(self.owner_of(attacker))
                                    && self.profile(o).has(mask::ANTI_AIR)));
                        let in_range = if must_reach || guard_reach {
                            if !self.is_in_range(attacker, o) {
                                continue;
                            }
                            false
                        } else if matches!(attacker, Obj::Unit(_)) || self.is_in_range(attacker, o)
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
                && bd
                    .ty
                    .is_some_and(|t| self.build_types[t].wonder || self.is_military_trainer(t));
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
        let t_attack = self.attack_of(target);
        let left = self.hits_left(target);
        if t_attack != 0 && left != 0 && !aa {
            // A raider looks at an active building's worth, not its threat:
            // `iVar5 == 0 || !bVar17 || iVar4 == 0` takes the formula, and
            // the one case left takes `/ 20` (`0064ea57`–`0064ea7a`).
            if is_build && raiding && self.active(target) {
                v /= 20;
            } else {
                v = v * i64::from(t_attack) * 100 / i64::from(left);
                if is_build {
                    v *= 10;
                }
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
                &self.tuning,
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
            // SEAM: the arm at `64f171` — a target whose object flags
            // carry `0x20` takes the weight only with `num_inside` non-zero
            // — is not carried: read as "a city" it moves Great Lakes'
            // second game off its close (5930 → 5158, `1/9`'s strike).
            if armed && !raiding {
                let human = self
                    .nation
                    .get(self.owner_of(attacker) as usize)
                    .is_some_and(|n| n.human);
                if !human {
                    if ap.has(mask::SIEGE) {
                        v += 1_000_000;
                    } else {
                        v *= 5;
                    }
                }
                if ap.has(mask::SIEGE) {
                    v += 100_000;
                }
            }
        } else {
            if tp.combat_role {
                v *= 20;
            }
            // **The raid weights** (`0064edc4`–`0064eff9`): a raider wants
            // the economy. A peasant — exactly `PEASANTS`/`PEASANTSKOREAN`,
            // `ObjectData::is_peasant`, type `0x32`/`0x33` — or a caravan is
            // worth `+900,000` to a computer's raider and `+9,000,000` to a
            // human's; a computer's also takes a combat-role unit at
            // `+10,000`; anything else is worth a tenth.
            //
            // SEAM: an AI ship's raid arm (`bVar16`, `0064edd8`–`0064ef64`:
            // the `0x150`/`0x13d` lineage tests and the stealth-ship
            // weights) is read as the land arm; no capture reaches one.
            let peasant = matches!(target, Obj::Unit(u) if self.is_peasant(u));
            if raiding {
                if peasant || tp.is(role::CARAVAN) {
                    v += if ai_raider { 900_000 } else { 9_000_000 };
                } else if ai_raider && tp.combat_role {
                    v += 10_000;
                } else {
                    v /= 10;
                }
            } else if tp.combat_role {
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
        // **An anti-air building's `recharging` is its `Wall::inc_time`
        // cycle's, not this countdown** (item 1112, `docs/COMBAT.md`
        // §84): `Build::do_attack@006228f0`'s head (`6228fe`..`622946`) skips the countdown for
        // an `ANTI_AIR` building that is neither a Lookout nor an
        // Observation Post, and it never reaches `Object::fire_ammo`.
        let cycles = self.wall_cycle_of(b).is_some();
        let bd = &self.buildings[b];
        let phase = bd.phase(frame) & 0x1f;
        // Without a target, `do_attack` runs every 32nd frame.
        if bd.target.is_none() && phase != 0 {
            return;
        }
        if !cycles && bd.recharging > 0 {
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
        // **The target, as `Build::do_attack@006228f0`'s listing takes it**
        // (`622a37`..`622b4c`, item 1131, `docs/COMBAT.md` §8.6):
        // - a building without an explicit order (`build_masks & 4`) calls
        //   `Build::find_target@00622c80` on **every** call (`622a3f`), and
        //   `compare_target`'s current-target ×2 (or /2) is what holds it;
        // - no target: cleared, and return (`622a5a` → `622c1c`);
        // - one `valid_target` refuses: `find_target`, and return without
        //   a shot (`622a75` → `622b25`);
        // - unordered, on `(o + frame + 14) & 0x1f == 0`: `find_target` and
        //   return, unless the target is a unit that is not moving and is
        //   combat-role (`type +0x2c8 & 0x10000`) or casting
        //   (`action_type == CAST_SPELL`, `622b20`);
        // - out of range (`622b45`): the target is cleared (`622c1c`).
        // `find_target` itself clears the order bit (`& 0xfffb`).
        let radius = (p.x_size.max(p.y_size) + 2 * self.max_range_of(me)) * 0x60;
        if !bd.ordered {
            self.buildings[b].target = self.find_nearby_target(me, radius);
        }
        let Some(target) = self.buildings[b].target else {
            self.buildings[b].ordered = false;
            return;
        };
        if !self.valid_target(me, target) {
            self.buildings[b].target = self.find_nearby_target(me, radius);
            self.buildings[b].ordered = false;
            return;
        }
        if !self.buildings[b].ordered && ((self.buildings[b].phase(frame) + 14) & 0x1f) == 0 {
            let keep = match target {
                Obj::Unit(u) => {
                    self.units[u].movement.dest.is_none()
                        && (self.profile(target).combat_role
                            || matches!(
                                self.current_order(u).map(|o| o.body),
                                Some(crate::orders::Body::Cast(_))
                            ))
                }
                Obj::Building(_) => false,
            };
            if !keep {
                self.buildings[b].target = self.find_nearby_target(me, radius);
                return;
            }
        }
        if !self.is_in_range(me, target) {
            self.buildings[b].target = None;
            self.buildings[b].ordered = false;
            return;
        }
        // `do_attack`'s in-range arm returns before `fire_ammo` for the
        // same buildings (`622b52`..`622ba3`, to the return at `622c2e`): their round is the cycle's
        // release event ([`Sim::walls_inc_time`]).
        if cycles {
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
            // `Object::fire_ammo`'s building arm: 250 above the building's
            // own `z` (§46.1).
            let sz = self.world.tile_z(centre.tile()) + 0xfa;
            let ez = self.aim_z(target, land_unit);
            self.add_ammo(combat::Projectile {
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
                // A building has no order, so `Ammo::init`'s `local_38`
                // is zero and the flag turns on the target alone.
                rolling: land_unit,
                missed: false,
                harmless: false,
                air: matches!(tp.domain, Domain::Air),
                sz,
                ez,
                v1z: combat::arc_v1z(sz, ez, total_time),
                slot: 0,
            });
        }
        self.buildings[b].recharging = p.recharge / arrows;
    }

    // ------------------------------------------------------------------
    // Ammo
    // ------------------------------------------------------------------

    /// `Ammo::inc_time` for every live projectile (§9.2, §9.3), and the
    /// **rolling** arm at its foot (§42.2).
    ///
    /// `inc_time@0067d380` breaks its own loop at `cur_time >=
    /// total_time` and then, for a shot with flag `4`, tries
    /// `hit_target` and `check_hit(GROUND)` **once**: if neither names an
    /// object it sets flag `8` and the shot travels on along its line —
    /// up to `3 × total_time` — until the terrain rises to meet its
    /// parabola, and only then does `do_damage` run. So a rolling shot
    /// whose target died in flight spends **no draws on the frame it was
    /// due**, where a non-rolling one punctures the ground with two.
    ///
    /// **The terrain test** is §46's: the arc `(v1z × t + sz) + GRAV_Z ×
    /// 0.5 × t × t` ([`combat::arc_z`]) against the ground under the point
    /// ([`Sim::ground_z`], `TerrainOut::find_data_z@00866560`), both in
    /// [`crate::single::Single`] arithmetic, operation for operation as the
    /// listing does them. The shot comes down on the first step its height
    /// is not above the ground, and `do_damage` runs at the new point — on
    /// whatever `check_hit` finds there, or into the ground if nothing.
    pub(crate) fn process_projectiles(&mut self, frame: i64) {
        let mut i = 0;
        while i < self.projectiles.len() {
            self.projectiles[i].cur_time += 1;
            let p = self.projectiles[i];
            if p.cur_time < p.total_time {
                i += 1;
                continue;
            }
            if p.rolling {
                if !p.missed {
                    // `hit_target` forgets the target on a miss, in
                    // `inc_time` exactly as in `do_damage`: run112's
                    // `0/8` arrow prints `whom -1 ox -1` on the block
                    // after it rolled, with `flags 14` beside it.
                    let hit = self.hit_target(&p);
                    if !hit {
                        self.projectiles[i].target = None;
                        if self.check_hit(&p).is_none() {
                            self.projectiles[i].missed = true;
                        }
                    }
                }
                if self.projectiles[i].missed {
                    // `if ((uint)(total_time * 3) < cur_time) close()` —
                    // and `close` is not `do_damage`, so a shot that
                    // never meets the ground damages nothing.
                    if p.cur_time > p.total_time * 3 {
                        self.projectiles.remove(i);
                        continue;
                    }
                    // On along the line, and down the arc (§46.1): the
                    // point at `t / T` past the launch, off the world is
                    // a `close`, and the shot comes down on the first step
                    // its height is at or under the ground's.
                    let at = combat::arc_point(p.launch, p.landing, p.cur_time, p.total_time);
                    if at.x < 0
                        || at.y < 0
                        || at.x >= self.world.width() * UNITS_PER_CELL
                        || at.y >= self.world.height() * UNITS_PER_CELL
                    {
                        self.projectiles.remove(i);
                        continue;
                    }
                    let z = combat::arc_z(p.v1z, p.sz, p.cur_time);
                    let ground = crate::single::Single::from_i32(self.ground_z(at));
                    if z.gt(ground) {
                        i += 1;
                        continue;
                    }
                    // Landed: `pos = (x, y, (int)z)` and `total_time =
                    // cur_time`, then `do_damage` at the new point.
                    let q = &mut self.projectiles[i];
                    q.landing = at;
                    q.ez = z.to_i32();
                    q.total_time = q.cur_time;
                }
            }
            let p = self.projectiles.remove(i);
            // `inc_time:260`: a round with flag `0x10` is closed, and
            // `Ammo::do_damage` is never called for it (§39.5).
            if !p.harmless {
                self.land(p, frame);
            }
        }
    }

    /// `Ammo::hit_target@00678f90` (§9.4): the shot's own target, still
    /// active and still inside its `target_size` (or its footprint, for a
    /// building), at the landing point.
    fn hit_target(&self, p: &combat::Projectile) -> bool {
        p.target
            .filter(|&t| self.active(t))
            .is_some_and(|t| match t {
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
            })
    }

    /// `Ammo::do_damage` (§9.3): the hit test at the landing point, then
    /// `Object::do_damage` on what was hit — the target, or whatever stood
    /// there instead — and the splash around it.
    fn land(&mut self, p: combat::Projectile, frame: i64) {
        let mut target = p.target.filter(|&t| self.active(t));
        if !self.hit_target(&p) {
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
            let angle = p.bearing();
            self.do_damage(p.shooter, t, angle, true, 0x100, false, false, frame);
            return;
        }
        // **The missile arm's shield** (item 1078, `678337`..`67845d`;
        // `docs/PRODUCTION.md` "The missile's other arms"): a missile's
        // round is closed with no damage and no walk when the landing
        // cell's territory owner holds `MISSILE_DEFENSE_BONUS` and is not
        // the shooter's player. run390's V2b on 3169, in who=1's land ten
        // frames' flight after `tech who=1 missile_shield on`. SEAM: a
        // landing on ocean, whose owner is the first owned cell of a ring.
        if self.profile(p.shooter).has(mask::MISSILE)
            && let Some(w) = self.world.owner_at(p.landing).player()
            && w != p.owner
            && self.missile_defense_held(w)
        {
            return;
        }
        // **The nuke arm** (item 1091, `67846d`..`6785db`): a nuke's round
        // strikes nothing where it lands; it lays the blast (`crate::nuke`).
        if self.is_nuke(p.shooter) {
            self.nuke_land(p, frame);
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
                // **The round's own target skips the team test**
                // (`6787d9`..`6787e1`, item 1200): the shooter itself is
                // left out and a player past seven, and then only an
                // object that is not the round's target is asked whether
                // it is the shooter's own or its mutual ally. So a round
                // that `check_hit` put on one of its own side's buildings
                // strikes it: chapter forty-one's Biplane on who=1's
                // `1/2003`, 858 (`docs/GOLDEN.md` §50). SEAM: a neutral
                // object, neither allied nor at war, is struck there and
                // left out here.
                let owner = self.owner_of(o);
                if o == p.shooter || owner >= crate::world::PLAYER_SLOTS {
                    return false;
                }
                target == Some(o)
                    || (owner != p.owner
                        && (self.at_war_with(p.owner, owner) || self.at_war_with(owner, p.owner)))
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
            let angle = p.bearing();
            self.do_damage(p.shooter, o, angle, true, count, splash, false, frame);
        }
    }

    /// `Ammo::check_hit` (§9.4): a unit within two tiles of the landing point
    /// in the ammo's domain class — any player's — else a building on the
    /// tile.
    fn check_hit(&self, p: &combat::Projectile) -> Option<Obj> {
        // **An aircraft's round passes over its own side** (`678db9`..
        // `678dcc`, item 1200): a shooter of the air domain that is not a
        // missile searches `SEARCH_NON_FRIENDLY` (6), which
        // `Search::valid_search@0067daa0`'s case 6 reads as "not the
        // searcher's own player" — allies are found. Every other shooter
        // searches `SEARCH_ALL`. Unasked while [`Sim::land`] left the
        // shooter's side out of the splash; it is asked since the round's
        // own target skips that test.
        let sp = self.profile(p.shooter);
        let own_passed = matches!(sp.domain, Domain::Air) && !sp.has(mask::MISSILE);
        let mut best: Option<(i32, usize)> = None;
        for (i, u) in self.units.iter().enumerate() {
            if !(u.alive() && u.on_map) || Obj::Unit(i) == p.shooter {
                continue;
            }
            if own_passed && u.owner == p.owner {
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
        self.building_under(p.landing).map(Obj::Building)
    }

    /// **`check_hit`'s second half** (item 1077, `docs/COMBAT.md` §73):
    /// `ObjectsData::find_building_at@0065ab40(landing tile, SEARCH_ALL,
    /// −1, FILTER_ALL)` at `678ece`, reached only for a landing inside the
    /// world (`678e7a`..`678eac`, `world+0x18 × 0xc0` a side) when
    /// `find_unit` found nothing. The tile is `div_3_table[c >> 6]`, the
    /// landing's own tile; the building is the first, on the 3×3 cells
    /// round it, of a player below eight, active (vslots `0xc` and `0x4c`,
    /// `SubObjectData::is_active` and `WallData::is_active`) whose
    /// footprint, `WallData::tile_corner` to `+ x_size`/`+ y_size`, holds
    /// the tile. Footprints do not overlap, so the walk's order picks
    /// nothing. This crate compared the tile against the building's point
    /// in position units, and no building was ever found: a V2 with no
    /// target that came down on run371's Barracks `1/2006` struck it as a
    /// splash fringe, 400 of its 1,200, where the original struck it as
    /// the round's own target and destroyed it on 2821.
    fn building_under(&self, landing: Pos) -> Option<usize> {
        if landing.x < 0
            || landing.y < 0
            || landing.x >= self.world.width() * UNITS_PER_CELL
            || landing.y >= self.world.height() * UNITS_PER_CELL
        {
            return None;
        }
        let tile = landing.tile();
        (0..self.buildings.len()).find(|&b| {
            self.buildings[b].owner < crate::world::PLAYER_SLOTS
                && self.active(Obj::Building(b))
                && self.covers_tile(b, tile)
        })
    }
}

/// `Object::take_damage` on a site: `whole × 50` off the progress.
const fn combat_progress_lost(whole: i32) -> i32 {
    crate::build::progress_lost(whole)
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

    /// A round of `shooter`'s landing at `at`, on `target`, done.
    fn round(
        sim: &Sim,
        shooter: usize,
        target: Option<Obj>,
        at: Pos,
        splash: i32,
    ) -> combat::Projectile {
        combat::Projectile {
            shooter: Obj::Unit(shooter),
            owner: sim.units[shooter].owner,
            target,
            launch: sim.units[shooter].pos,
            landing: at,
            cur_time: 1,
            total_time: 1,
            accuracy: 100,
            angle: Angle(0),
            splash_area: splash,
            num_guys: 1,
            air: false,
            rolling: false,
            missed: false,
            harmless: false,
            sz: 0,
            ez: 0,
            v1z: combat::arc_v1z(0, 0, 1),
            slot: 0,
        }
    }

    /// **A splash round strikes its own target whoever owns it**
    /// (`Ammo::do_damage`, `6787d9`..`6787e1`, item 1200): the team test
    /// is asked only of an object that is not the round's target. Chapter
    /// forty-one's Biplane on 858, whose round `check_hit` put on who=1's
    /// own `1/2003`: the original spends that building's first-wound roll.
    /// Another of the shooter's buildings in the splash is left out, and
    /// so is an enemy's standing clear of it. Made to fail with the
    /// target's exemption dropped.
    #[test]
    fn a_splash_round_strikes_its_own_target_whoever_owns_it() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 1, ty, Pos::new(1000, 1000));
        let building = |sim: &mut Sim, who: Player, p: Pos| {
            let b = sim.add_building(who, p, 0);
            sim.buildings[b].started = true;
            sim.buildings[b].active = true;
            sim.buildings[b].combat = Some(Profile {
                x_size: 1,
                y_size: 1,
                ..Profile::default()
            });
            sim.buildings[b].health = 500;
            b
        };
        let mine = building(&mut sim, 1, Pos::new(2016, 2016));
        let other = building(&mut sim, 1, Pos::new(2208, 2016));
        let p = round(&sim, me, Some(Obj::Building(mine)), Pos::new(2016, 2016), 1);
        sim.land(p, 858);
        let struck: Vec<Obj> = sim.hits.iter().map(|h| h.target).collect();
        assert_eq!(struck, vec![Obj::Building(mine)], "the target alone");
        assert!(!sim.hits[0].splash, "at the full count");
        let _ = other;
    }

    /// **An aircraft's round passes over its own side** (`Ammo::check_hit`,
    /// `678db9`..`678dcc`, item 1200): an air shooter that is not a
    /// missile searches `SEARCH_NON_FRIENDLY`, which leaves out its own
    /// player's units and nothing else; any other shooter searches them
    /// all. Chapter forty-one's Biplane on 969, whose round came down
    /// beside who=1's `1/1`. Made to fail with the test dropped.
    #[test]
    fn an_aircraft_s_round_passes_over_its_own_side() {
        for air in [true, false] {
            let (mut sim, ty) = at_war();
            let mut t = sim.unit_types[ty].clone();
            t.combat.domain = if air { Domain::Air } else { Domain::Land };
            let shooter_ty = sim.add_unit_type(t);
            let mut foot = sim.unit_types[ty].clone();
            foot.combat.target_size = 300;
            let foot = sim.add_unit_type(foot);
            let me = put(&mut sim, 1, shooter_ty, Pos::new(1000, 1000));
            let own = put(&mut sim, 1, foot, Pos::new(3050, 3000));
            let foe = put(&mut sim, 0, foot, Pos::new(3100, 3000));
            let p = round(&sim, me, None, Pos::new(3000, 3000), 0);
            sim.land(p, 969);
            let want = if air { foe } else { own };
            assert_eq!(
                sim.hits.last().map(|h| h.target),
                Some(Obj::Unit(want)),
                "air {air}"
            );
        }
    }

    /// **A ship that attacks sideways turns broadside, to the nearer side**
    /// (`Unit::fight@005fd4d0:698–714`, `docs/COMBAT.md` §49). run127's
    /// own numbers: who=1's trireme at (12408, 35832), heading `0x55555555`
    /// (120°), attacking who=0's at (11640, 34680). The bearing is
    /// `−402259968` (−33.7°); the plus side, `671481856`, is 63.7° from the
    /// heading and the minus side 116.3°, so the plus side it is, which is
    /// what the dump's `angle` holds on block 617. The same geometry from a
    /// heading on the other side takes the minus side, a type without `g`
    /// takes the bearing, and a Patrol Boat shooting at a ship does too.
    ///
    /// Made to fail first by returning the bearing: the golden widening's
    /// `1/6` angle rows on 617 are that failure.
    #[test]
    fn a_sideways_ship_attacks_broadside_on_the_nearer_side() {
        let (mut sim, ty) = at_war();
        let ship = |sim: &mut Sim, flags: u32, type_index: i32| {
            let mut t = sim.unit_types[ty].clone();
            t.cols.unit_flags = flags;
            t.type_index = type_index;
            t.combat.domain = Domain::Sea;
            sim.add_unit_type(t)
        };
        let trireme = ship(&mut sim, uflags::SIDEWAYS, 0x154);
        let plain = ship(&mut sim, 0, 0x154);
        let patrol = ship(&mut sim, uflags::SIDEWAYS, 0x185);
        let me = put(&mut sim, 1, trireme, Pos::new(12408, 35832));
        let foe = put(&mut sim, 0, trireme, Pos::new(11640, 34680));
        let direct = find_angle(11640 - 12408, 34680 - 35832);
        assert_eq!(direct, Angle(-402_259_968), "run127's bearing");
        sim.units[me].movement.heading = Angle(0x5555_5555);
        assert_eq!(
            sim.attack_angle(me, Obj::Unit(foe), direct),
            Angle(671_481_856),
            "the plus side, as the dump's block 617 holds"
        );
        sim.units[me].movement.heading = Angle(direct.0.wrapping_sub(0x3000_0000));
        assert_eq!(
            sim.attack_angle(me, Obj::Unit(foe), direct),
            Angle(direct.0.wrapping_sub(0x4000_0000)),
            "the minus side when it is the nearer"
        );
        sim.units[me].ty = Some(plain);
        assert_eq!(sim.attack_angle(me, Obj::Unit(foe), direct), direct);
        sim.units[me].ty = Some(patrol);
        assert_eq!(
            sim.attack_angle(me, Obj::Unit(foe), direct),
            direct,
            "a Patrol Boat does not broadside a ship"
        );
    }

    /// **A type whose pivot bears shoots on its own heading** —
    /// `Unit::set_attack@005fce70` and `Guy::set_all_pivots@005d8bc0`
    /// (`docs/COMBAT.md` §52). run145's chariot `0/8`: born facing
    /// `0x5555_5555` (120°), its target 36.9° off that, and the dump's
    /// `angle` still `1431655765` on 634, after the shot. The same unit
    /// with the target 60° off turns, a type with no `<RESTRICTION>` turns
    /// at 37°, and a node whose own range does not cover the bearing turns
    /// too, both for a plain range (a Katyusha's −20..20) and for a
    /// wrapped one (a Dreadnought's `45..−45`, the rear arc). The crew
    /// figure is never aimed.
    ///
    /// Made to fail first: with `set_attack` answering 0, the first
    /// assertion reads the bearing, `993918976` — the golden widening's
    /// 634 row in shape (the capture's own bearing is `991232000`; this
    /// geometry is the capture's to within a degree).
    #[test]
    fn a_pivot_that_bears_shoots_without_turning() {
        let (mut sim, ty) = at_war();
        const CHARIOT: i32 = 195;
        let chariot = {
            let mut t = sim.unit_types[ty].clone();
            t.type_index = CHARIOT;
            t.combat.max_range = 8;
            sim.add_unit_type(t)
        };
        sim.art
            .pivots
            .insert(CHARIOT, [(4, (-180, 180))].into_iter().collect());
        let here = Pos::new(984, 8136);
        let shoot = |sim: &mut Sim, ty: usize, at: Pos| -> (Angle, Option<Obj>) {
            let me = put(sim, 0, ty, here);
            sim.units[me].guys = vec![anim::Guy::fresh(-1), anim::Guy::fresh(-1)];
            sim.units[me].movement.heading = Angle(0x5555_5555);
            sim.units[me].movement.facing = Angle(0x5555_5555);
            let foe = put(sim, 1, ty, at);
            sim.fight_pub(me, Obj::Unit(foe), 633);
            let aim = sim.units[me].guys[0].aim;
            assert_eq!(aim, Some(Obj::Unit(foe)), "figure 0 is aimed");
            assert_eq!(sim.units[me].guys[1].aim, None, "the crew is not");
            (sim.units[me].movement.heading, aim)
        };
        // run145's `0/8` and a target on the dump's bearing, 36.9° off.
        let near = Pos::new(here.x + 1536, here.y - 192);
        let bearing = find_angle(1536, -192);
        let off = crate::movement::angle_to_degrees(Angle(bearing.0.wrapping_sub(0x5555_5555)));
        assert_eq!(off - 360, -37, "the geometry is the capture's");
        assert_eq!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
        // Due south, 60° off: past the ±45° every pivot is held to.
        let far = Pos::new(here.x, here.y + 1536);
        assert_ne!(shoot(&mut sim, chariot, far).0, Angle(0x5555_5555));
        // No `<RESTRICTION>`: the plain type turns at 37°.
        assert_ne!(shoot(&mut sim, ty, near).0, Angle(0x5555_5555));
        // A node that cannot reach 37°, plain and wrapped.
        sim.art
            .pivots
            .insert(CHARIOT, [(4, (-20, 20))].into_iter().collect());
        assert_ne!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
        sim.art
            .pivots
            .insert(CHARIOT, [(4, (45, -45))].into_iter().collect());
        assert_ne!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
        sim.art
            .pivots
            .insert(CHARIOT, [(4, (-45, -30))].into_iter().collect());
        assert_eq!(shoot(&mut sim, chariot, near).0, Angle(0x5555_5555));
    }

    /// **The pivot bears from its node, not the unit's square**
    /// (`docs/COMBAT.md` §54). run145's `0/8` on 684: at `984, 8136`,
    /// facing 120°, its target `1/7` at `1512, 7944` bears 69.9°, 50° off
    /// the heading from the square. The Chariot's figure (piece 145) has
    /// its node at `(−102, −59)` from the square at that facing, and from
    /// there `1/7` is 42° off, so the original shoots without turning:
    /// run147's packet answers `Unit::set_attack(0/8, 7, 1)` with 1, and
    /// `(6, 1)`, `1/6` at `1608, 7800` (51° off from the node), with 0.
    ///
    /// Made to fail first: with `pivot::offset` answering `(0, 0)` the
    /// first assertion reads the bearing, `834011136` — the widening's
    /// 685 row.
    #[test]
    fn a_pivot_bears_from_its_node() {
        let (mut sim, ty) = at_war();
        const CHARIOT: i32 = 195;
        let chariot = {
            let mut t = sim.unit_types[ty].clone();
            t.type_index = CHARIOT;
            t.combat.max_range = 8;
            sim.add_unit_type(t)
        };
        sim.art
            .pivots
            .insert(CHARIOT, [(4, (-180, 180))].into_iter().collect());
        let here = Pos::new(984, 8136);
        let heading = Angle(0x5555_5555);
        let shoot = |sim: &mut Sim, piece: i32, at: Pos| -> Angle {
            let me = put(sim, 0, chariot, here);
            sim.units[me].guys = vec![anim::Guy::fresh(piece), anim::Guy::fresh(12817)];
            sim.units[me].movement.heading = heading;
            sim.units[me].movement.facing = heading;
            let foe = put(sim, 1, ty, at);
            sim.fight_pub(me, Obj::Unit(foe), 684);
            sim.units[me].movement.heading
        };
        let seven = Pos::new(1512, 7944);
        let off = |from: Pos| {
            let b = find_angle(seven.x - from.x, seven.y - from.y);
            crate::movement::angle_to_degrees(Angle(heading.0.wrapping_sub(b.0)))
        };
        assert_eq!(
            (off(here), off(Pos::new(here.x - 102, here.y - 59))),
            (50, 42),
            "the geometry is the capture's"
        );
        assert_eq!(shoot(&mut sim, 145, seven), heading, "from the node");
        assert_ne!(shoot(&mut sim, -1, seven), heading, "a piece with no node");
        assert_ne!(
            shoot(&mut sim, 145, Pos::new(1608, 7800)),
            heading,
            "1/6 is past 45° from the node too"
        );
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

    /// **A figure of a squad falls at its share of the squad's hit
    /// points, not at the whole of them** — `Object::take_damage`'s step
    /// 8 (`docs/COMBAT.md` §7.2, §7.3, §41.4).
    ///
    /// `update_hits` writes one number, the squad's, onto every figure;
    /// the divide is on the way in. So a squad of three with `HITS` 120
    /// is three figures of 40, and the dump's `myhits` is 120 on each of
    /// them with `damage` counting that figure's own share of what has
    /// been taken. This crate kept the squad-sized maximum (item 484's
    /// row pins it on 245,679 field-frames) and used it as the
    /// **threshold** too, so a figure absorbed three figures' worth:
    /// run112's hoplite `1/8` reached `damage` 51 still standing where
    /// the original's died on block 683 at its fortieth point, and
    /// `extra 1/8` was that.
    ///
    /// **Made to fail on purpose** by passing `u.health` again, which is
    /// what stood here: the figure survives the third hit and the
    /// assertion below reddens on `alive`.
    ///
    /// `combat::share`'s captain remainder is the second half and is not
    /// reachable from here — it needs the squad down to one figure — so
    /// the arithmetic is tested directly in `combat`'s own tests and the
    /// call site is what this pins.
    #[test]
    fn a_figure_falls_at_its_share_of_the_squad_s_hits() {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 120,
            combat: Profile {
                attack: 15,
                max_range: 4,
                uber_size: 3,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        sim.units[me].squad_size = 3;
        sim.units[me].combat.captain = i32::from(sim.units[me].index);
        // Three fifteen-point hits: 15, 30, 45 against a share of 40.
        let hit = combat::Sixteenths { whole: 15, frac: 0 };
        for expected in [false, false, true] {
            let died = matches!(
                sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10),
                Taken::Died { .. }
            );
            assert_eq!(died, expected, "share is 120 / 3 = 40");
        }
        assert!(!sim.units[me].alive(), "a dead figure is not alive");
        // And the squad-sized maximum is untouched, because item 484's
        // row compares it against the dump's `myhits`.
        assert_eq!(sim.units[me].max_health, 120);
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

    /// **A human's packed siege engine never searches**
    /// (`Unit::think_attack@005f5a80`'s packed arm, `docs/COMBAT.md` §51):
    /// it holds no order until the first auto-attack frame with `idle ≥
    /// 7`, and that order is the unpack, `0x28c`, at the head. A
    /// computer's packed engine is not `manual`, so it falls through the
    /// arm and searches while it waits out its 21; and a human's engine
    /// whose packer stance is `PACKER_NEVER` (1) neither casts nor
    /// searches. Golden chapter three's word 621 was the first case: this
    /// crate put run145's packed catapult into `Unit::fight` on its birth
    /// block.
    ///
    /// **Made to fail on purpose**: with the arm's call taken out of
    /// `think`, the human's catapult takes the attack order on its first
    /// idle frame, which is 587's word.
    /// **A hit on a building spills onto the enemy hands repairing it**
    /// (`Object::do_damage@0064a480`'s tail, item 1182, `docs/GOLDEN.md`
    /// §50): an eighth for a sea attacker that is not siege and a land one,
    /// a quarter for a land siege type, nothing for a sea siege type (the
    /// `is_siege` arm at `64c3b3`) or an aircraft, and nothing onto a
    /// citizen of the building's side that is not at work on it. Made to
    /// fail with the sea arm's `is_siege` test dropped, and with the
    /// quarter read as an eighth.
    #[test]
    fn a_building_s_hit_spills_onto_its_repairers_by_the_attacker_s_domain() {
        use crate::orders::{Body, Order};
        for (domain, siege, repairing, share) in [
            (Domain::Sea, false, true, Some(8)),
            (Domain::Sea, true, true, None),
            (Domain::Land, true, true, Some(4)),
            (Domain::Land, false, true, Some(8)),
            (Domain::Air, false, true, None),
            (Domain::Land, false, false, None),
        ] {
            let (mut sim, ty) = at_war();
            let mut at = sim.unit_types[ty].clone();
            at.combat.domain = domain;
            at.combat.siege = siege;
            at.kind.domain = domain;
            let at = sim.add_unit_type(at);
            let b = sim.add_building(1, Pos::new(3000, 3000), 0);
            sim.buildings[b].started = true;
            sim.buildings[b].active = true;
            sim.buildings[b].health = 1200;
            sim.buildings[b].combat = Some(Profile {
                x_size: 2,
                y_size: 2,
                ..Profile::default()
            });
            let hand = put(&mut sim, 1, ty, Pos::new(3300, 3000));
            if repairing {
                sim.units[hand].orders.push_back(Order {
                    flags: 0,
                    body: Body::Repair(b),
                });
            }
            let me = put(&mut sim, 0, at, Pos::new(3600, 3000));
            let count = 0x800;
            sim.do_damage(
                Obj::Unit(me),
                Obj::Building(b),
                Angle(0),
                false,
                count,
                false,
                false,
                700,
            );
            let spilt: Vec<_> = sim
                .hits
                .iter()
                .filter(|h| h.target == Obj::Unit(hand))
                .copied()
                .collect();
            match share {
                None => assert!(spilt.is_empty(), "{domain:?} siege {siege}: a spill"),
                Some(k) => {
                    assert_eq!(spilt.len(), 1, "{domain:?} siege {siege}: no spill");
                    let ap = sim.profile(Obj::Unit(me));
                    let want = combat::scale(
                        spilt[0].damage,
                        count / k,
                        true,
                        false,
                        ap.ammo_per_att,
                        ap.uber_size,
                    );
                    assert_eq!(spilt[0].dealt, want, "{domain:?} siege {siege}: the share");
                }
            }
        }
    }

    /// **A packed packer in range of its attack unpacks first**
    /// (`Unit::fight@005fd4d0:512`–`543`, item 1182): a human's packed
    /// catapult holding an attack on a building in range is given the
    /// unpack at the head and strikes nothing, where it used to fall through
    /// to the strike. Made to fail with [`Sim::packed_unpacks`] answering
    /// false.
    #[test]
    fn a_packed_packer_in_range_unpacks_before_it_strikes() {
        use crate::orders::{Body, spell};
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 15,
                uber_size: 1,
                siege: true,
                packs: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
        assert!(sim.units[me].combat.packed);
        let b = sim.add_building(1, Pos::new(0x4000 + 6 * 192, 0x4000), 0);
        sim.buildings[b].started = true;
        sim.buildings[b].active = true;
        sim.buildings[b].health = 1200;
        sim.buildings[b].combat = Some(Profile::default());
        sim.add_attack_order(
            me,
            Obj::Building(b),
            crate::orders::QueuePos::New,
            true,
            true,
        );
        let frame = sim.frame;
        sim.work(me, frame);
        assert!(
            matches!(sim.units[me].orders.front().map(|o| o.body), Some(Body::Cast(c)) if c.spell == spell::UNPACK),
            "no unpack at the head: {:?}",
            sim.units[me].orders
        );
        assert!(sim.hits.is_empty(), "a packed engine struck");
    }

    #[test]
    fn a_human_s_packed_siege_engine_unpacks_before_it_searches() {
        use crate::orders::{Body, spell};
        let catapult = |human: bool, stance: u8| {
            let (mut sim, _) = at_war();
            sim.nation[0].human = human;
            let ty = sim.add_unit_type(crate::UnitType {
                hits: 100,
                combat: Profile {
                    attack: 40,
                    max_range: 15,
                    uber_size: 1,
                    obj_masks: mask::SIEGE,
                    siege: true,
                    packs: true,
                    combat_role: true,
                    ..Profile::default()
                },
                ..crate::UnitType::default()
            });
            // The loader writes `role & 0x10000` twice: the profile's
            // `combat_role` and the column `get_stance_type` reads.
            sim.unit_types[ty].cols.role |= crate::ai_load::role::MILITARY;
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
            assert!(
                sim.units[me].combat.packed,
                "a type that packs is born packed"
            );
            sim.units[me].stance = stance;
            put(&mut sim, 1, foe_ty, Pos::new(0x4000 + 6 * 192, 0x4000));
            // Tick until the engine holds an order or 200 frames pass;
            // answer the `idle` it thought on and what it was given.
            for _ in 0..200 {
                let idle = sim.units[me].idle;
                let frame = sim.frame;
                sim.tick();
                if let Some(o) = sim.units[me].orders.front() {
                    return Some((idle, frame, o.body, sim.units[me].combat.target));
                }
            }
            None
        };
        // The human's: nothing on its first idle frame, and its first
        // order is the unpack, on a thirty-two phase at `idle` 7 or more.
        let Some((idle, frame, body, target)) = catapult(true, 0) else {
            panic!("the human's packed catapult never unpacked");
        };
        assert!(
            matches!(body, Body::Cast(c) if c.spell == spell::UNPACK),
            "the human's first order is not the unpack: {body:?}"
        );
        assert_eq!(target, None, "a packed engine took a target");
        assert!(idle >= 6, "the unpack came before `idle` 7: {idle}");
        // `(frame + o) & 31 == 0` on the frame the think runs, and the
        // engine is `o` 0.
        assert_eq!(frame & 31, 0, "the unpack is on the auto-attack's phase");
        // The computer's searches while packed, on its first idle frame.
        let Some((_, _, body, target)) = catapult(false, 0) else {
            panic!("the computer's packed catapult took no order");
        };
        assert!(
            matches!(body, Body::Attack(_)) && target.is_some(),
            "a computer's packed engine is not `manual` and must search: {body:?}"
        );
        // `PACKER_NEVER`: the human's engine waits, orderless.
        assert_eq!(
            catapult(true, 1).map(|r| r.2),
            None,
            "a `PACKER_NEVER` engine cast or searched"
        );
    }

    /// **An unpacked siege engine shoots a unit by shooting the ground
    /// under it** (`Unit::fight@005fd4d0`'s siege arm and
    /// `Unit::do_attack_ground@005f1410`, `docs/COMBAT.md` §57). On the
    /// frame the attack comes into range an `ATTACK_GROUND` goes on top of
    /// it at the foe's own position and fires in the same frame
    /// (`attack_unit` 1, the fired bit), the figure forgets its aim, the
    /// attack beneath keeps `new_ord`, and the reload is one frame longer
    /// than a strike's. On the ready frame the order dies and the attack
    /// beneath, still in range, pushes the next.
    ///
    /// **Made to fail on purpose**: with the arm returning `false`, the
    /// engine strikes the unit directly — one order, `new_ord` cleared,
    /// the plain reload.
    #[test]
    fn an_unpacked_siege_engine_fires_on_the_ground_under_its_target() {
        use crate::orders::{Body, flag, index};
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 15,
                recharge: 30,
                uber_size: 1,
                obj_masks: mask::SIEGE,
                siege: true,
                packs: true,
                combat_role: true,
                ..Profile::default()
            },
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
        let me = put(&mut sim, 0, ty, Pos::new(0x4000 + 24, 0x4000 + 24));
        sim.units[me].combat.packed = false;
        let at = Pos::new(0x4000 + 6 * 192 + 24, 0x4000 + 24);
        let foe = put(&mut sim, 1, foe_ty, at);
        sim.order_attack(me, Obj::Unit(foe));
        sim.tick();
        let u = &sim.units[me];
        let kinds: Vec<u8> = u.orders.iter().map(|o| o.index()).collect();
        assert_eq!(kinds, vec![index::ATTACK_GROUND, index::ATTACK]);
        let head = u.orders[0];
        let Body::AttackGround(g) = head.body else {
            unreachable!()
        };
        assert_eq!((g.at, g.sea, g.attack_unit), (at, false, 1));
        assert!(head.has(flag::FIRED) && !head.has(flag::ACTION));
        let Body::Attack(a) = u.orders[1].body else {
            unreachable!()
        };
        assert!(a.in_range && a.ever_in_range && a.new_ord);
        let reload = sim.reload_frames(me);
        assert_eq!(u.combat.recharging, reload + 1, "the ground shot's reload");
        // Held through the reload, then gone on the ready frame, and the
        // attack beneath pushes the next at once.
        let mut pushes = 0;
        for _ in 0..=reload {
            sim.tick();
            let head = sim.units[me].orders.front().copied();
            if let Some(o) = head
                && let Body::AttackGround(g) = o.body
                && sim.units[me].combat.recharging == reload + 1
            {
                assert_eq!(g.attack_unit, 1);
                pushes += 1;
            }
        }
        assert_eq!(pushes, 1, "the ready frame re-entered work and fired again");
    }

    /// **A building is struck square to its side** (item 1040,
    /// `docs/COMBAT.md` §70.6): `Unit::fight@005fd4d0`'s `5fe8a7`–`5feb4c`.
    /// Great Lakes' Hoplites on the human's city face due south from the
    /// row above its footprint, where the centre's bearing is 20° off.
    /// West is asked first, then north, east and south; a covered tile that
    /// is not blocked, or the unit's own tile covered, keeps the bearing.
    ///
    /// Made to fail first with the arm answering `None`.
    #[test]
    fn a_building_is_struck_square_to_its_side() {
        let (mut sim, ty) = at_war();
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 3,
            y_size: 3,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let b = sim.add_building(0, at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].hits = 1000;
        sim.buildings[b].health = 1000;
        sim.buildings[b].combat = Some(Profile::default());
        let corner = sim.tile_corner(bt, at);
        for t in sim.footprint(bt, corner) {
            sim.world.set_blocked_at(t, true);
        }
        let me = put(&mut sim, 1, ty, at);
        let tile = |x: i32, y: i32| Pos::new(x * 192 + 96, y * 192 + 96);
        let side = |sim: &mut Sim, p: Pos| {
            sim.units[me].pos = p;
            sim.building_side(me, Obj::Building(b)).map(|a| a.0 as u32)
        };
        let (cx, cy) = (corner.x, corner.y);
        assert_eq!(
            side(&mut sim, tile(cx, cy - 1)),
            Some(0x8000_0000),
            "above: south"
        );
        assert_eq!(
            side(&mut sim, tile(cx + 2, cy + 3)),
            Some(0),
            "below: north"
        );
        assert_eq!(
            side(&mut sim, tile(cx - 1, cy + 1)),
            Some(0x4000_0000),
            "left: east"
        );
        assert_eq!(
            side(&mut sim, tile(cx + 3, cy)),
            Some(0xc000_0000),
            "right: west"
        );
        assert_eq!(
            side(&mut sim, tile(cx - 1, cy - 1)),
            None,
            "a corner: the bearing"
        );
        assert_eq!(
            side(&mut sim, tile(cx + 1, cy + 1)),
            None,
            "inside: the bearing"
        );
        sim.world.set_blocked_at(Pos::new(cx, cy), false);
        assert_eq!(
            side(&mut sim, tile(cx, cy - 1)),
            None,
            "a covered tile that is not blocked"
        );
        let foe = put(&mut sim, 0, ty, at);
        sim.units[me].pos = tile(cx, cy - 1);
        assert_eq!(sim.building_side(me, Obj::Unit(foe)), None, "a unit target");
    }

    /// **The one-in-five retarget freezes the frame** (item 1040,
    /// `docs/COMBAT.md` §70.7): `Unit::fight@005fd4d0`'s `005fdf68`–
    /// `005fdfea`. A captain whose roll runs the re-search and finds
    /// another target keeps its attack under the new target and carries
    /// `unit_masks2 |= 0x10`, so its figures' clocks stand still for the
    /// frame. Great Lakes' `1/24` on 4852: its Slinger's attack slot held
    /// at 32 of 33 there, and wrapped to the idle here.
    ///
    /// Made to fail first with the mark left off.
    #[test]
    fn the_one_in_five_retarget_freezes_the_frame() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 1, ty, Pos::new(0x4000, 0x4000));
        let far = put(&mut sim, 0, ty, Pos::new(0x4000 + 9 * 0xc0, 0x4000));
        let near = put(&mut sim, 0, ty, Pos::new(0x4000 + 2 * 0xc0, 0x4000));
        sim.add_attack_order(
            me,
            Obj::Unit(far),
            crate::orders::QueuePos::New,
            false,
            false,
        );
        assert_eq!(sim.find_melee_target(me, -1), Some(Obj::Unit(near)));
        let seed = (1u32..)
            .find(|&k| {
                let mut r = sim.rng;
                r.seed = k;
                r.roll() % 5 != 0
            })
            .unwrap();
        sim.rng.seed = seed;
        sim.units[me].unit_masks2 = 0;
        sim.work(me, 0);
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(near)));
        assert_ne!(
            sim.units[me].unit_masks2 & crate::combat::umask2::NOT_FIRING,
            0,
            "the retarget left the frame unfrozen"
        );
    }

    /// **The one-in-five re-search kills the attack first** (item 1099,
    /// `docs/COMBAT.md` §80): `Unit::fight@005fd4d0:413` is
    /// `find_new_target(this, &who, 0)`, so with the attack gone the head
    /// is the attack-move again and the search runs under its `0x20010`,
    /// which halves a building. run347's `1/24` on 5161: under the attack a
    /// search names the human's city, under the attack-move the scout it
    /// is already attacking, and the original strikes the scout.
    ///
    /// Made to fail with the arm searching under the attack and
    /// re-pointing it in place, as it did before: the target is the
    /// building.
    #[test]
    fn the_one_in_five_re_search_runs_under_the_attack_move() {
        let (mut sim, ty) = at_war();
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let b = sim.add_building(0, at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].hits = 800;
        sim.buildings[b].health = 800;
        sim.buildings[b].combat = Some(Profile {
            attack: 10,
            cost: 1600,
            ..Profile::default()
        });
        let me = put(&mut sim, 1, ty, Pos::new(at.x + 3 * 0xc0, at.y));
        let scout = put(&mut sim, 0, ty, Pos::new(at.x + 4 * 0xc0, at.y + 0xc0));
        sim.add_move_order(
            me,
            Pos::new(at.x - 20 * 0xc0, at.y),
            crate::orders::MoveKind::AttackTo,
            crate::orders::QueuePos::New,
            false,
        );
        sim.add_attack_order(
            me,
            Obj::Unit(scout),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        // Under the attack the search takes the building; under the
        // attack-move, the scout.
        assert_eq!(sim.melee_search_flags(me), 0);
        assert_eq!(
            sim.clone().find_melee_target(me, -1),
            Some(Obj::Building(b)),
            "the attack's own search"
        );
        let mut bare = sim.clone();
        bare.kill_current_order(me);
        assert_eq!(
            bare.find_melee_target(me, -1),
            Some(Obj::Unit(scout)),
            "the attack-move's search"
        );
        let seed = (1u32..)
            .find(|&k| {
                let mut r = sim.rng;
                r.seed = k;
                r.roll() % 5 != 0
            })
            .unwrap();
        sim.rng.seed = seed;
        sim.work(me, 0);
        assert_eq!(
            sim.units[me].combat.target,
            Some(Obj::Unit(scout)),
            "the re-search searched under the attack"
        );
    }

    /// **A captain's attack on a building re-searches every frame**
    /// (`docs/COMBAT.md` §62, `Unit::fight@005fd4d0`'s `LAB_005fddf7`,
    /// `005fdeb4` → `005fdf50` → `005fdeea`). Out of its search's reach the
    /// attack dies where the unit stands, and nothing walks: run177's
    /// Fighter on tick 632. In reach, the search re-finds the building and
    /// the chase goes on under a fresh order.
    ///
    /// Made to fail both ways: with the arm off, the far unit keeps its
    /// attack and adds a chase; with the search's answer dropped, the near
    /// one loses its attack.
    #[test]
    fn a_captains_attack_on_a_building_re_searches_every_frame() {
        let (mut sim, ty) = at_war();
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 3,
            y_size: 3,
            ..crate::build::BuildType::default()
        });
        let target_at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let b = sim.add_building(1, target_at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].hits = 1000;
        sim.buildings[b].health = 1000;
        sim.buildings[b].combat = Some(Profile::default());
        let run = |sim: &mut Sim, from: Pos| {
            let u = put(sim, 0, ty, from);
            sim.add_attack_order(
                u,
                Obj::Building(b),
                crate::orders::QueuePos::New,
                false,
                false,
            );
            let before = sim.find_melee_target(u, -1);
            assert!(sim.valid_target(Obj::Unit(u), Obj::Building(b)));
            sim.tick();
            (u, before)
        };
        // Far: sixteen tiles off, past a human's twelve-tile idle search
        // (`UNIT_RESPOND_RANGE × 0xc0`), as run175's Fighter at its attack
        // point is at about twelve.
        let mut far = sim.clone();
        let (u, before) = run(&mut far, Pos::new(target_at.x - 16 * 0xc0, target_at.y));
        assert_eq!(before, None, "the far unit's search reaches the building");
        assert!(
            far.units[u].orders.is_empty(),
            "the far unit kept an order: {:?}",
            far.units[u].orders
        );
        // Near: eight tiles off, out of range 4 and inside the search.
        let (u, before) = run(&mut sim, Pos::new(target_at.x - 8 * 0xc0, target_at.y));
        assert_eq!(
            before,
            Some(Obj::Building(b)),
            "the near unit's search misses"
        );
        assert!(
            sim.units[u]
                .orders
                .iter()
                .any(|o| matches!(o.body, crate::orders::Body::Attack(_))),
            "the near unit lost its attack: {:?}",
            sim.units[u].orders
        );
        assert_eq!(sim.units[u].combat.target, Some(Obj::Building(b)));
    }

    /// **A candidate in another region is taken only in range**
    /// (`Object::check_target@00649e00`'s head, `docs/COMBAT.md` §72, item
    /// 1214). East Indies 8907's shape: the computer's Caravel, idle on
    /// the sea, and the human's building inland. Across the regions the
    /// building out of range is refused, and in range it is taken; a
    /// computer's `SIEGE` ship ignores the regions. The head's other arm:
    /// a `DEFENSIVE` unit off duty with an order must reach whatever it
    /// takes, in its own region too.
    ///
    /// Made to fail by the search not asking `check_target_reaches`: the
    /// first assertion names the building.
    #[test]
    fn a_candidate_in_another_region_is_taken_only_in_range() {
        use crate::world::Terrain;
        let (mut sim, ty) = at_war();
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let land = sim.world.add_region(Terrain::Land);
        let sea = sim.world.add_region(Terrain::Sea);
        for x in 0..60 {
            for y in 0..60 {
                let r = if x < 30 { land } else { sea };
                sim.world.set_region(crate::Cell::new(x, y), r);
            }
        }
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(29 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let site = sim.add_building(0, at, 0);
        sim.buildings[site].ty = Some(bt);
        sim.buildings[site].hits = 400;
        sim.buildings[site].health = 400;
        sim.buildings[site].combat = Some(Profile::default());
        // A ship: the domain in the type's kind, the type's combat and the
        // unit's kind.
        sim.unit_types[ty].kind.domain = Domain::Sea;
        sim.unit_types[ty].combat.domain = Domain::Sea;
        let far = put(&mut sim, 1, ty, Pos::new(at.x + 10 * 0xc0, at.y));
        assert_eq!(sim.units[far].kind.domain, Domain::Sea);
        assert!(!sim.is_in_range(Obj::Unit(far), Obj::Building(site)));
        assert_eq!(
            sim.clone().find_melee_target(far, -1),
            None,
            "the ship took a building in another region out of its range"
        );
        // In range, across the regions, it is taken.
        let mut near = sim.clone();
        near.units[far].pos = Pos::new(at.x + 3 * 0xc0, at.y);
        assert!(near.is_in_range(Obj::Unit(far), Obj::Building(site)));
        assert_eq!(near.find_melee_target(far, -1), Some(Obj::Building(site)));
        // A computer's siege ship does not ask the regions.
        let mut siege = sim.clone();
        siege.unit_types[ty].combat.obj_masks |= mask::SIEGE;
        assert_eq!(
            siege.find_melee_target(far, -1),
            Some(Obj::Building(site)),
            "a computer's siege ship asked the regions"
        );
        // One region: the same ship takes it out of range.
        let mut one = sim.clone();
        for x in 0..60 {
            for y in 0..60 {
                one.world.set_region(crate::Cell::new(x, y), land);
            }
        }
        assert_eq!(
            one.clone().find_melee_target(far, -1),
            Some(Obj::Building(site))
        );
        // Defensive, off duty and with an order: it must reach.
        let mut guarded = one.clone();
        guarded.units[far].combat.stance = Stance::Defensive;
        // The defensive radius is the range itself, so the search is
        // handed a wider one, as `find_melee_target`'s callers may.
        let wide = 12 * 0xc0;
        assert_eq!(
            guarded.clone().find_melee_target(far, wide),
            Some(Obj::Building(site)),
            "defensive with no order is the plain search"
        );
        guarded.add_move_order(
            far,
            Pos::new(at.x + 20 * 0xc0, at.y),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        assert_eq!(
            guarded.find_melee_target(far, wide),
            None,
            "a defensive unit with an order took what it cannot reach"
        );
    }

    /// **An attack-move's look passes over an unarmed building**
    /// (`docs/COMBAT.md` §64, item 997). Great Lakes 4555's shape: an AI
    /// soldier on an `ATTACK_TO` past the human's Woodcutter's Camp. Under
    /// the attack-move's `0x20010` the camp is refused before it is
    /// scored, and an armed building is taken; the same soldier with no
    /// attack-move at its head searches with 0 and takes the camp. The
    /// word itself is checked for each of `melee_search_flags`' arms.
    ///
    /// Made to fail by `search_admits` answering true: the first
    /// assertion names the camp.
    #[test]
    fn an_attack_move_passes_over_an_unarmed_building() {
        let (mut sim, ty) = at_war();
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let camp = sim.add_building(0, at, 0);
        sim.buildings[camp].ty = Some(bt);
        sim.buildings[camp].hits = 800;
        sim.buildings[camp].health = 800;
        sim.buildings[camp].combat = Some(Profile::default());
        let me = put(&mut sim, 1, ty, Pos::new(at.x + 8 * 0xc0, at.y));
        let walk = |sim: &mut Sim| {
            sim.add_move_order(
                me,
                Pos::new(at.x - 20 * 0xc0, at.y),
                crate::orders::MoveKind::AttackTo,
                crate::orders::QueuePos::New,
                false,
            );
        };
        let mut moving = sim.clone();
        walk(&mut moving);
        assert_eq!(
            moving.melee_search_flags(me),
            search::ARMED | search::HALVE_NON_UNITS
        );
        assert_eq!(
            moving.find_melee_target(me, -1),
            None,
            "the attack-move took the unarmed camp"
        );
        // Idle, the same search takes it.
        assert_eq!(sim.melee_search_flags(me), 0);
        assert_eq!(sim.find_melee_target(me, -1), Some(Obj::Building(camp)));
        // An armed building is taken on the move.
        let mut armed = moving.clone();
        armed.buildings[camp].combat = Some(Profile {
            attack: 10,
            ..Profile::default()
        });
        assert_eq!(
            armed.find_melee_target(me, -1),
            Some(Obj::Building(camp)),
            "the attack-move refused an armed building"
        );
        // The word's other arms: a siege type (AI 0x20002, human 0), a
        // tank (0), and RAZE over any of them (0x20).
        let typed = |flags: u32, stance: Stance, human: bool| {
            let mut s = moving.clone();
            let mut t = s.unit_types[ty].clone();
            t.cols.unit_flags = flags;
            let t = s.add_unit_type(t);
            s.units[me].ty = Some(t);
            s.units[me].combat.stance = stance;
            s.nation[1].human = human;
            s.melee_search_flags(me)
        };
        assert_eq!(
            typed(uflags::SIEGE, Stance::Aggressive, false),
            search::BUILDINGS | search::ARMED
        );
        assert_eq!(typed(uflags::SIEGE, Stance::Aggressive, true), 0);
        assert_eq!(typed(uflags::TANK, Stance::Aggressive, false), 0);
        assert_eq!(
            typed(uflags::TANK, Stance::Raze, false),
            search::HALVE_NON_BUILDS
        );
    }

    /// **A ranged chase on a building stops at the edge of its reach**
    /// (`do_move@005f7b30`, the listing `5f7f27`–`5f7faa`; `docs/COMBAT.md`
    /// §65). The target's vslot `+0x1c` splits the kill: a unit target is
    /// asked with `is_in_range`'s `0x90` margin when the attack is not
    /// mandatory, a building through `is_in_range@00648d70`, whose sixth
    /// argument is 0. run347's `1/26` stops on the human's city at
    /// `attack_dist` 1128 against a reach of 1158 (Great Lakes 4593).
    ///
    /// Four arms: a building inside the reach but not inside the margin
    /// kills the move; a unit at the same distance does not; an AI siege
    /// type is passed over at `5f7f34`; and a building past the reach
    /// keeps the chase. Made to fail on purpose by asking the building
    /// with the margin again: the first arm keeps its move.
    #[test]
    fn a_ranged_chase_on_a_building_stops_at_the_edge_of_its_reach() {
        let (mut sim, ty) = at_war();
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let city = sim.add_building(0, at, 0);
        sim.buildings[city].ty = Some(bt);
        sim.buildings[city].hits = 800;
        sim.buildings[city].health = 800;
        sim.buildings[city].combat = Some(Profile::default());
        let foe = put(&mut sim, 0, ty, at);
        let me = put(&mut sim, 1, ty, at);
        // The reach is `4 × 0xc0 + 6`; find a spot east of the target
        // inside it but not inside the margin, for each kind of target.
        let reach = sim.max_range_of(Obj::Unit(me)) * 0xc0 + 6;
        let spot = |sim: &mut Sim, t: Obj, lo: i32, hi: i32| {
            (0..0x1000)
                .map(|dx| Pos::new(at.x + dx, at.y))
                .find(|&p| {
                    sim.units[me].pos = p;
                    let d = sim.attack_dist(Obj::Unit(me), t);
                    (lo..=hi).contains(&d)
                })
                .expect("a spot at that distance")
        };
        let inside = spot(&mut sim, Obj::Building(city), reach - 0x8f, reach);
        let past = spot(&mut sim, Obj::Building(city), reach + 1, reach + 0x40);
        let unit_inside = spot(&mut sim, Obj::Unit(foe), reach - 0x8f, reach);
        let chase = |sim: &Sim, from: Pos, t: Obj, siege: bool| {
            let mut s = sim.clone();
            if siege {
                let mut ut = s.unit_types[ty].clone();
                ut.cols.unit_flags = uflags::SIEGE;
                let ut = s.add_unit_type(ut);
                s.units[me].ty = Some(ut);
            }
            s.units[me].pos = from;
            s.add_attack_order(me, t, crate::orders::QueuePos::First, false, true);
            s.add_move_order(
                me,
                Pos::new(at.x - 0x400, at.y),
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::First,
                false,
            );
            assert!(!s.units[me].combat.mandatory);
            s.work(me, 0);
            s.units[me].orders.iter().any(crate::orders::Order::is_move)
        };
        assert!(
            !chase(&sim, inside, Obj::Building(city), false),
            "a building inside the reach kept the chase"
        );
        assert!(
            chase(&sim, unit_inside, Obj::Unit(foe), false),
            "a unit inside the reach but not the margin ended the chase"
        );
        assert!(
            chase(&sim, inside, Obj::Building(city), true),
            "an AI siege type stopped on the building"
        );
        assert!(
            chase(&sim, past, Obj::Building(city), false),
            "a building past the reach ended the chase"
        );
    }

    /// **A chase on a building holds while another unit stands in the
    /// chaser's block** (`do_move@005f7b30`, `5f7f73`–`5f7f9d`: the
    /// building arm's last conjunct is `Objects::find_collision(my spot,
    /// o, who, 1) == 0`; `docs/COMBAT.md` §65.8). The fifth argument set
    /// skips the land shortcut, so the query is the nine world cells'
    /// chains at current positions, Chebyshev in unit cells against the
    /// two `coll_size`s. run426's Longbowman `1/29` reaches the human's
    /// city at `attack_dist` 2284 with its squad-mate `1/30` two unit
    /// cells off, walks one more step, and ends the chase there (Great
    /// Sahara 15586).
    ///
    /// Three arms: a mate two cells off holds the chase; three cells off
    /// ends it; gaia two cells off is not asked. Made to fail on purpose
    /// by dropping the conjunct: the first arm ends its chase.
    #[test]
    fn a_unit_in_the_chaser_s_block_holds_its_chase_on_a_building() {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 15,
                max_range: 4,
                uber_size: 1,
                block_radius: 48,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        sim.nation[0].human = true;
        sim.nation[1].human = false;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 2,
            y_size: 2,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x180, 30 * 0x300 + 0x180);
        let city = sim.add_building(0, at, 0);
        sim.buildings[city].ty = Some(bt);
        sim.buildings[city].hits = 800;
        sim.buildings[city].health = 800;
        sim.buildings[city].combat = Some(Profile::default());
        let me = put(&mut sim, 1, ty, at);
        // `put` chains a unit where it is born; the chaser is taken off
        // the chain while its spot is sought, and put back there.
        sim.chain_remove(me);
        let reach = sim.max_range_of(Obj::Unit(me)) * 0xc0 + 6;
        // A unit-cell centre east of the target, inside the reach.
        let inside = (0..0x1000)
            .map(|dx| Pos::new(at.x + dx * 48 + 24, at.y))
            .find(|&p| {
                sim.units[me].pos = p;
                let d = sim.attack_dist(Obj::Unit(me), Obj::Building(city));
                (reach - 0x8f..=reach).contains(&d)
            })
            .expect("a spot inside the reach");
        let chase = |sim: &Sim, who: Player, cells: i32| {
            let mut s = sim.clone();
            s.units[me].pos = inside;
            s.chain_add(me);
            put(&mut s, who, ty, Pos::new(inside.x, inside.y + cells * 48));
            s.add_attack_order(
                me,
                Obj::Building(city),
                crate::orders::QueuePos::First,
                false,
                true,
            );
            s.add_move_order(
                me,
                Pos::new(at.x - 0x400, at.y),
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::First,
                false,
            );
            s.work(me, 0);
            s.units[me].orders.iter().any(crate::orders::Order::is_move)
        };
        assert!(
            chase(&sim, 1, 2),
            "a mate two unit cells off let the chase end"
        );
        assert!(
            !chase(&sim, 1, 3),
            "a mate three unit cells off held the chase"
        );
        assert!(!chase(&sim, 8, 2), "gaia in the block held the chase");
    }

    /// **A fleeing target re-aims the chase** (`check_target_path@005e22d0`,
    /// the listing `5e24e2`–`5e2873`; `docs/COMBAT.md` §67). On the review's
    /// phase, a ranged chaser whose unit target is walking away from it asks
    /// whether its **walk spot** still reaches the target; when it does not,
    /// the legs go and `find_attack_pos` puts a new one in front. run347's
    /// `1/24` re-aims on 4616 after the citizen `0/3` (Great Lakes 4673).
    ///
    /// Four arms: the fleeing target re-aims; a standing target, a target
    /// walking toward the chaser and an off-phase frame keep the stale spot.
    /// Made to fail on purpose by returning at the fall-through, as this
    /// crate did: the first arm keeps its stale spot.
    #[test]
    fn a_fleeing_target_re_aims_the_chase_on_the_review_s_phase() {
        let (mut sim, ty) = at_war();
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let foe = put(&mut sim, 0, ty, at);
        let me = put(&mut sim, 1, ty, Pos::new(at.x + 12 * 0xc0, at.y));
        let stale = Pos::new(at.x + 10 * 0xc0, at.y);
        let away = crate::movement::find_angle(at.x - sim.units[me].pos.x, 0);
        let toward = crate::movement::find_angle(sim.units[me].pos.x - at.x, 0);
        // The review's phase: `(frame + o) % 16 == 0`.
        let on = 16 - i64::from(sim.units[me].index);
        let chase_to =
            |sim: &Sim, stale: Pos, heading: Option<crate::movement::Angle>, frame: i64| {
                let mut s = sim.clone();
                if let Some(h) = heading {
                    s.add_move_order(
                        foe,
                        Pos::new(at.x - 20 * 0xc0, at.y),
                        crate::orders::MoveKind::MoveTo,
                        crate::orders::QueuePos::New,
                        false,
                    );
                    s.units[foe].movement.heading = h;
                }
                s.add_attack_order(
                    me,
                    Obj::Unit(foe),
                    crate::orders::QueuePos::First,
                    false,
                    true,
                );
                s.add_move_order(
                    me,
                    stale,
                    crate::orders::MoveKind::MoveTo,
                    crate::orders::QueuePos::First,
                    false,
                );
                let before = s.units[me].orders_pos;
                s.work(me, frame);
                let after = s.units[me].orders.iter().find_map(|o| match o.body {
                    crate::orders::Body::Move(m) => Some(m.dest),
                    _ => None,
                });
                (before, after, s)
            };
        let chase = |sim: &Sim, heading, frame| chase_to(sim, stale, heading, frame);
        let (before, after, s) = chase(&sim, Some(away), on);
        let after = after.expect("the chase keeps a move");
        assert_ne!(
            after, before,
            "the fleeing target's chase kept its stale spot"
        );
        assert!(
            s.is_in_range_at(Obj::Unit(me), after, Obj::Unit(foe)),
            "the new spot reaches the target"
        );
        for (heading, frame, why) in [
            (None, on, "a standing target"),
            (Some(toward), on, "a target walking toward the chaser"),
            (Some(away), on + 1, "an off-phase frame"),
        ] {
            let (before, after, _) = chase(&sim, heading, frame);
            assert_eq!(after, Some(before), "{why} re-aimed the chase");
        }
        // The range is asked from the **walk spot** (`orders_x/orders_y`,
        // `5e2553`/`5e2558`), not from where the chaser stands: a spot that
        // still reaches the target keeps the chase, though the chaser, at
        // twelve tiles, is out of reach.
        let near = Pos::new(at.x + 3 * 0xc0, at.y);
        let (before, after, _) = chase_to(&sim, near, Some(away), on);
        assert_eq!(
            after,
            Some(before),
            "a walk spot in reach re-aimed the chase"
        );
    }

    /// **A dead target is re-aimed at** (`check_target_path@005e22d0`,
    /// `5e2429`–`5e242e`; `docs/COMBAT.md` §76). The slot's vslot `+0x8`
    /// is `SubObjectData::is_active`, and a dead target's `je 5e24e8`
    /// jumps the flank triple into the same fall-through a fleeing one
    /// takes: its walk spot cannot reach a corpse, so on the review's phase
    /// the legs go and `find_attack_pos` aims a new one at where it fell.
    /// run347's `1/13` re-aims on 5011, 5027, 5043 and 5059 at `0/2`, dead
    /// since 5000 (Great Lakes 5066).
    ///
    /// Three arms: the dead target re-aims, standing still; the same target
    /// alive and standing, and the dead one off the phase, keep the stale
    /// spot. Made to fail on purpose by returning for a dead target, as
    /// this crate did: the first arm keeps its stale spot.
    #[test]
    fn a_dead_target_is_re_aimed_at_on_the_review_s_phase() {
        let (mut sim, ty) = at_war();
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let foe = put(&mut sim, 0, ty, at);
        // Past `(max_range + 8)` tiles, `find_attack_pos`' far arm, which
        // takes the sweep's spot without asking the range (a corpse is in
        // nobody's): `1/13` was some fifteen tiles out.
        let me = put(&mut sim, 1, ty, Pos::new(at.x + 16 * 0xc0, at.y));
        // Ten tiles from the chaser, so `do_move`'s dead-target arm
        // (`0x480` from the walk's own goal) leaves it standing.
        let stale = Pos::new(at.x + 6 * 0xc0, at.y + 3 * 0xc0);
        // The review's phase: `(frame + o) % 16 == 0`.
        let on = 16 - i64::from(sim.units[me].index);
        let chase = |sim: &Sim, dead: bool, frame: i64| {
            let mut s = sim.clone();
            s.add_attack_order(
                me,
                Obj::Unit(foe),
                crate::orders::QueuePos::First,
                false,
                true,
            );
            s.add_move_order(
                me,
                stale,
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::First,
                false,
            );
            if dead {
                s.units[foe].health = 0;
            }
            let before = s.units[me].orders_pos;
            s.work(me, frame);
            let after = s.units[me].orders.iter().find_map(|o| match o.body {
                crate::orders::Body::Move(m) => Some(m.dest),
                _ => None,
            });
            (before, after)
        };
        let (before, after) = chase(&sim, true, on);
        let after = after.expect("the chase keeps a move");
        assert_ne!(after, before, "the dead target's chase kept its stale spot");
        assert!(
            crate::world::vector_dist((after.x - at.x).abs(), (after.y - at.y).abs())
                < crate::world::vector_dist((before.x - at.x).abs(), (before.y - at.y).abs()),
            "the new spot is not aimed at the corpse: {after:?}"
        );
        for (dead, frame, why) in [
            (false, on, "a living standing target"),
            (true, on + 1, "an off-phase frame"),
        ] {
            let (before, after) = chase(&sim, dead, frame);
            assert_eq!(after, Some(before), "{why} re-aimed the chase");
        }
    }

    /// **An unpacked packer takes only what it can reach, and a hit from
    /// inside its minimum is dropped without a chase**
    /// (`Object::find_nearby_target@00648da0`'s `local_24`, the listing
    /// `64918d` and `6495c2`; `Unit::fight@005fd4d0:1051`'s packer
    /// re-search; `docs/COMBAT.md` §60). run146's catapult `0/6` is the
    /// diff: its idle search on 864 finds nothing inside three tiles, and
    /// each hoplite that hits it after is attacked for one frame and let
    /// go.
    ///
    /// Made to fail on purpose, both ways: with the range gate read back
    /// as "takes anything" the search names the foe inside the minimum;
    /// with the packer re-search off the retaliation walks away to get
    /// its range.
    #[test]
    fn an_unpacked_packer_takes_only_what_it_can_reach() {
        let (mut sim, _) = at_war();
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 40,
                max_range: 15,
                min_range: 3,
                recharge: 30,
                uber_size: 1,
                obj_masks: mask::SIEGE,
                siege: true,
                packs: true,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let foe_ty = sim.add_unit_type(crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 15,
                uber_size: 1,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let here = Pos::new(0x4000 + 24, 0x4000 + 24);
        let me = put(&mut sim, 0, ty, here);
        sim.units[me].combat.packed = false;
        let near = put(
            &mut sim,
            1,
            foe_ty,
            Pos::new(0x4000 + 2 * 192 + 24, 0x4000 + 24),
        );
        assert_eq!(
            sim.find_melee_target(me, -1),
            None,
            "an unpacked packer's search took a foe inside its minimum"
        );
        // Packed, the same engine need not reach, and it names the foe.
        sim.units[me].combat.packed = true;
        assert_eq!(sim.find_melee_target(me, -1), Some(Obj::Unit(near)));
        sim.units[me].combat.packed = false;
        // The hit: an attack on the hitter, then on the engine's next
        // frame the attack is gone and the engine has not moved.
        sim.do_damage(
            Obj::Unit(near),
            Obj::Unit(me),
            crate::movement::Angle(0),
            false,
            1,
            false,
            false,
            10,
        );
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(near)));
        sim.tick();
        assert!(
            sim.units[me].orders.is_empty(),
            "the engine kept an order: {:?}",
            sim.units[me].orders
        );
        assert!(
            sim.units[me].path.is_empty() && sim.units[me].movement.dest.is_none(),
            "the engine walked for its range"
        );
    }

    /// **An aircraft takes no aircraft it cannot reach** (item 650,
    /// `docs/COMBAT.md` §61), on run168's two types. A Bomber (`FLY_HIGH`
    /// 0, no `ANTI_AIR`) is refused a plane by `valid_target`'s air ladder,
    /// because a plane with no air order flies high. A Fighter (`ANTI_AIR`,
    /// seven tiles) may take the Bomber, but its search refuses one eight
    /// tiles off through `poor_target`'s plane arm, and takes it at three.
    /// A land rifleman (`FLY_HIGH` 0) is refused a plane too, and an
    /// anti-aircraft gun is not.
    ///
    /// Made to fail on purpose, both ways. With the ladder read back the
    /// Bomber's `valid_target` is true. With the plane arm off the Fighter's
    /// search names the Bomber at eight tiles.
    #[test]
    fn an_aircraft_takes_no_aircraft_it_cannot_reach() {
        let (mut sim, _) = at_war();
        let plane = |attack, max_range, fly_low, masks| crate::UnitType {
            hits: 100,
            combat: Profile {
                attack,
                max_range,
                fly_high: 0,
                fly_low,
                uber_size: 1,
                obj_masks: mask::AIR | masks,
                domain: Domain::Air,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        };
        let fighter_ty = sim.add_unit_type(plane(450, 7, 25, mask::ANTI_AIR));
        let bomber_ty = sim.add_unit_type(plane(430, 3, 10, mask::EXPLOSIVE));
        let ground = |max_range, fly_high, fly_low, masks| crate::UnitType {
            hits: 100,
            combat: Profile {
                attack: 200,
                max_range,
                fly_high,
                fly_low,
                uber_size: 1,
                obj_masks: masks,
                combat_role: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        };
        let rifle_ty = sim.add_unit_type(ground(8, 0, 20, mask::GUN));
        let flak_ty = sim.add_unit_type(ground(14, 50, 90, mask::ANTI_AIR));
        let y = 0x4000 + 96;
        let fighter = put(&mut sim, 0, fighter_ty, Pos::new(0x4000 + 96, y));
        let bomber = put(&mut sim, 1, bomber_ty, Pos::new(0x4000 + 96 + 8 * 192, y));
        let (f, b) = (Obj::Unit(fighter), Obj::Unit(bomber));
        assert!(!sim.valid_target(b, f), "the Bomber may take a plane");
        assert!(
            sim.valid_target(f, b),
            "the Fighter may not take the Bomber"
        );
        assert_eq!(sim.find_melee_target(bomber, -1), None);
        assert_eq!(
            sim.find_melee_target(fighter, -1),
            None,
            "the Fighter's search took a plane beyond its reach"
        );
        let near = put(&mut sim, 1, bomber_ty, Pos::new(0x4000 + 96, y - 3 * 192));
        assert_eq!(sim.find_melee_target(fighter, -1), Some(Obj::Unit(near)));
        let rifle = put(&mut sim, 0, rifle_ty, Pos::new(0x4000 + 96, y + 192));
        let flak = put(&mut sim, 0, flak_ty, Pos::new(0x4000 + 96, y + 384));
        assert!(!sim.valid_target(Obj::Unit(rifle), b));
        assert!(sim.valid_target(Obj::Unit(flak), b));
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

    /// **A busy unit answers a hit only on duty** (item 1040,
    /// `docs/COMBAT.md` §70): `Unit::target_opportunity`'s `600863`
    /// returns for a unit that holds any order and is not
    /// `Unit::on_duty@005fff70` — a combat-role type whose activity is an
    /// `ATTACK_TO`, a patrol, a `GUARD` or a group attack or patrol. Great
    /// Lakes' `0/4`, a citizen walking to its drop site under its
    /// `GATHER`, took `1/24`'s stone on 4779 and kept walking; this crate
    /// had it turn on the Slinger.
    ///
    /// Made to fail first with the gate removed: the walker answered.
    #[test]
    fn a_busy_unit_answers_a_hit_only_on_duty() {
        use crate::orders::{MoveKind, QueuePos};
        let answer = |role: bool, kind: Option<MoveKind>| {
            let (mut sim, ty) = at_war();
            sim.unit_types[ty].combat.combat_role = role;
            let me = put(&mut sim, 0, ty, Pos::new(0x4000, 0x4000));
            let foe = put(&mut sim, 1, ty, Pos::new(0x4100, 0x4000));
            if let Some(k) = kind {
                sim.add_move_order(me, Pos::new(0x2000, 0x4000), k, QueuePos::First, false);
            }
            let before = sim.units[me].orders.clone();
            sim.target_opportunity(me, Obj::Unit(foe), 10);
            let answered = sim.units[me].combat.target == Some(Obj::Unit(foe));
            assert_eq!(
                answered,
                sim.units[me].orders != before,
                "an answer is an order, and silence leaves the list alone"
            );
            answered
        };
        assert!(!answer(false, Some(MoveKind::MoveTo)), "a walker");
        assert!(
            !answer(true, Some(MoveKind::MoveTo)),
            "a soldier on a plain move is not on duty"
        );
        assert!(
            answer(true, Some(MoveKind::AttackTo)),
            "a soldier on an attack-move is"
        );
        assert!(
            !answer(false, Some(MoveKind::AttackTo)),
            "on duty asks the combat role first"
        );
        assert!(
            answer(false, None),
            "an idle unit answers whatever its role"
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
    /// **And the target itself outlives the object too** (item 502,
    /// `docs/COMBAT.md` §43.3): `Sim::forget` no longer clears it, which
    /// is what run112's three bowmen say — their `ATTACKORDER` carries the
    /// dead `1/8` for twelve blocks. So the first assertion below is the
    /// opposite of what it was, and the reload's twelve frames are now
    /// twelve frames of a **named** dead target rather than of nothing.
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
            sim.units[me].combat.target,
            Some(Obj::Unit(foe)),
            "the dead target is kept until the next use asks about it"
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
        // **No freeze mark**, and the reason is the condition rather than
        // the arm (§44.1). This unit reached `Unit::fight`'s
        // invalid-target branch, but it is HOLD_FIRE, so the search
        // returns nothing, so no attack order goes back in front — and
        // `unit_masks2 |= 0x10` is downstream of `order_type() == ATTACK`
        // *after* the search. A unit that finds nothing has nothing to be
        // "still ordered" under.
        assert_eq!(sim.units[me].unit_masks2, 0);
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
            rolling: false,
            missed: false,
            harmless: false,
            air: false,
            sz: 0,
            ez: 0,
            v1z: crate::single::Single::ZERO,
            slot: 0,
        };
        assert_eq!(sim.check_hit(&ammo), None);
        // The same arrow does find a player's unit standing there.
        let foe = put(&mut sim, 1, ty, landing);
        assert_eq!(sim.check_hit(&ammo), Some(Obj::Unit(foe)));
        assert!(sim.units[sheep].is_gaia());
    }
    /// **A round with no target that comes down on a building strikes it
    /// as its own target** (item 1077, `docs/COMBAT.md` §73). run371's
    /// V2: `whom −1`, landing at (13834, 14969) on who=1's Barracks
    /// `1/2006` at (13824, 14976), a 5×5 footprint, with no unit within two
    /// tiles. `Ammo::check_hit@00678d90` finds no unit and then
    /// `find_building_at` on the landing's tile, so the splash arm's walk
    /// hits the Barracks with `splash` 0: the same damage as a round aimed
    /// at it, where a fringe takes `SPLASH_PERCENT` of it. A landing one
    /// tile past the footprint finds nothing.
    ///
    /// Made to fail by the tile-against-point comparison this replaced:
    /// `check_hit` answers `None` and the Barracks takes the fringe's 400.
    #[test]
    fn a_round_with_no_target_strikes_the_building_it_lands_on() {
        let (mut sim, ty) = at_war();
        sim.unit_types[ty].combat.attack = 150;
        sim.unit_types[ty].combat.splash_area = 1;
        sim.unit_types[ty].combat.splash_percent = 25;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 5,
            y_size: 5,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(13824, 14976);
        let b = sim.add_building(1, at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].hits = 1200;
        sim.buildings[b].health = 1200;
        sim.buildings[b].combat = Some(Profile::default());
        let me = put(&mut sim, 0, ty, Pos::new(9974, 12347));
        let ammo = combat::Projectile {
            shooter: Obj::Unit(me),
            owner: 0,
            target: None,
            launch: Pos::new(9974, 12347),
            landing: Pos::new(13834, 14969),
            cur_time: 120,
            total_time: 120,
            accuracy: 228,
            angle: crate::movement::Angle(0),
            splash_area: 1,
            num_guys: 1,
            rolling: false,
            missed: false,
            harmless: false,
            air: false,
            sz: 466,
            ez: 78,
            v1z: crate::single::Single::ZERO,
            slot: 0,
        };
        assert_eq!(sim.check_hit(&ammo), Some(Obj::Building(b)));
        // The same round aimed at the Barracks, beside it.
        let mut aimed = sim.clone();
        aimed.land(
            combat::Projectile {
                target: Some(Obj::Building(b)),
                ..ammo
            },
            2820,
        );
        sim.land(ammo, 2820);
        assert!(
            sim.buildings[b].health < 1200,
            "the Barracks was not struck"
        );
        assert_eq!(
            sim.buildings[b].health, aimed.buildings[b].health,
            "struck as a fringe, not as the round's own target"
        );
        // One tile past the footprint's east edge: nothing.
        let corner = sim.tile_corner(bt, at);
        let past = Pos::new((corner.x + 5) * 0xc0 + 0x60, 14969);
        assert_eq!(
            sim.check_hit(&combat::Projectile {
                landing: past,
                ..ammo
            }),
            None
        );
    }

    /// **The shield closes a missile's round in its holder's land**
    /// (item 1078, `Ammo::do_damage@00678060`'s `678337`..`67845d`):
    /// run390's V2b came down on who=1's Barracks `1/2007` on 3169, in a
    /// cell of who=1's, after `tech who=1 missile_shield on`, and the
    /// Barracks stood at damage 0. The same round with the shield unheld,
    /// or in a cell no one owns, strikes it. Made to fail with the close
    /// dropped (the Barracks struck).
    #[test]
    fn the_shield_closes_a_missile_s_round_in_its_holder_s_land() {
        let (mut sim, ty) = at_war();
        sim.unit_types[ty].combat.attack = 150;
        sim.unit_types[ty].combat.splash_area = 1;
        sim.unit_types[ty].combat.splash_percent = 25;
        sim.unit_types[ty].combat.obj_masks |= mask::MISSILE;
        let bt = sim.add_build_type(crate::build::BuildType {
            x_size: 5,
            y_size: 5,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(13824, 14976);
        let b = sim.add_building(1, at, 0);
        sim.buildings[b].ty = Some(bt);
        sim.buildings[b].hits = 1200;
        sim.buildings[b].health = 1200;
        sim.buildings[b].combat = Some(Profile::default());
        let me = put(&mut sim, 0, ty, Pos::new(9974, 12347));
        let ammo = combat::Projectile {
            shooter: Obj::Unit(me),
            owner: 0,
            target: None,
            launch: Pos::new(9974, 12347),
            landing: Pos::new(13834, 14969),
            cur_time: 120,
            total_time: 120,
            accuracy: 228,
            angle: crate::movement::Angle(0),
            splash_area: 1,
            num_guys: 1,
            rolling: false,
            missed: false,
            harmless: false,
            air: false,
            sz: 466,
            ez: 78,
            v1z: crate::single::Single::ZERO,
            slot: 0,
        };
        let mut tree = sim.tech_tree.clone();
        let shield = tree.types.len();
        tree.types.push(crate::tech::TypeDef::new(
            "Missile Shield",
            crate::tech::Kind::Final,
        ));
        tree.roles.missile_defense_preq = Some(shield);
        sim.set_tech_tree(tree);
        for p in &mut sim.tech {
            p.tech.resize(shield + 1, false);
        }
        sim.tech[1].tech[shield] = true;
        let cell = ammo.landing.cell();
        let mut unowned = sim.clone();
        unowned
            .world
            .set_owner(cell, crate::world::Owner::None, crate::world::Owner::None);
        sim.world.set_owner(
            cell,
            crate::world::Owner::Player(1),
            crate::world::Owner::None,
        );
        let mut unheld = sim.clone();
        unheld.tech[1].tech[shield] = false;
        sim.land(ammo, 3169);
        assert_eq!(sim.buildings[b].health, 1200, "closed under the shield");
        unheld.land(ammo, 3169);
        assert!(
            unheld.buildings[b].health < 1200,
            "struck with the shield unheld"
        );
        unowned.land(ammo, 3169);
        assert!(
            unowned.buildings[b].health < 1200,
            "struck in no one's land"
        );
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

    /// A shot with nothing to home on, from `launch` to `landing` over
    /// `total` frames, leaving at height `sz` and aimed at `ez`.
    fn shot(
        me: usize,
        launch: Pos,
        landing: Pos,
        total: i32,
        sz: i32,
        ez: i32,
    ) -> combat::Projectile {
        combat::Projectile {
            shooter: Obj::Unit(me),
            owner: 0,
            target: None,
            launch,
            landing,
            cur_time: 0,
            total_time: total,
            accuracy: 100,
            angle: Angle(0),
            splash_area: 0,
            num_guys: 1,
            rolling: true,
            missed: false,
            harmless: false,
            air: false,
            sz,
            ez,
            v1z: combat::arc_v1z(sz, ez, total),
            slot: 0,
        }
    }

    /// `Objects::add_ammo`'s lowest free slot, and `Objects::inc_time`'s
    /// walk in slot order (`docs/COMBAT.md` §46.4): a landing frees its
    /// slot without reordering the rest, and the next shot fired takes the
    /// freed one and is stepped first.
    #[test]
    fn the_pool_takes_the_lowest_free_slot_and_steps_in_slot_order() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1400, 0x1000));
        let mut p = shot(me, a, b, 1, 0, 0);
        p.rolling = false;
        sim.add_ammo(p);
        sim.add_ammo(shot(me, a, b, 5, 0, 0));
        sim.add_ammo(shot(me, a, b, 6, 0, 0));
        let slots = |s: &Sim| {
            s.projectiles
                .iter()
                .map(|p| (p.slot, p.total_time))
                .collect::<Vec<_>>()
        };
        assert_eq!(slots(&sim), vec![(0, 1), (1, 5), (2, 6)]);
        sim.process_projectiles(0);
        assert_eq!(
            slots(&sim),
            vec![(1, 5), (2, 6)],
            "slot 0 landed, the rest kept their order"
        );
        sim.add_ammo(shot(me, a, b, 7, 0, 0));
        assert_eq!(
            slots(&sim),
            vec![(0, 7), (1, 5), (2, 6)],
            "the freed slot, stepped first"
        );
    }

    /// **A `do_damage="0"` round lands and does nothing** (item 853,
    /// `docs/ORDERS.md` §39.5): `Ammo::inc_time@0067d380:260` closes a
    /// round with flag `0x10` instead of calling `Ammo::do_damage`. The
    /// same round without the flag hits its target; with it the target
    /// keeps its health, the round is gone, and a ground shot spends no
    /// puncture draw. Made to fail with `process_projectiles` landing it
    /// regardless.
    #[test]
    fn a_harmless_round_lands_without_damage() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let (a, b) = (Pos::new(0x1000, 0x1000), Pos::new(0x1400, 0x1000));
        let foe = put(&mut sim, 1, ty, b);
        let aimed = |harmless: bool| {
            let mut p = shot(me, a, b, 1, 0, 0);
            (p.target, p.rolling, p.harmless) = (Some(Obj::Unit(foe)), false, harmless);
            p
        };
        let full = sim.units[foe].health;
        sim.add_ammo(aimed(true));
        sim.process_projectiles(0);
        assert!(sim.projectiles.is_empty(), "the round is closed");
        assert_eq!(sim.units[foe].health, full, "and does no damage");
        let before = sim.rng;
        let mut ground = shot(me, a, Pos::new(0x1000, 0x1400), 1, 0, 0);
        (ground.rolling, ground.harmless) = (false, true);
        sim.add_ammo(ground);
        sim.process_projectiles(0);
        assert_eq!(sim.rng, before, "no puncture draw");
        sim.add_ammo(aimed(false));
        sim.process_projectiles(0);
        assert!(
            sim.units[foe].health < full,
            "the same round, harmful, hits"
        );
    }

    /// **A dead unit's number is held while its death object lives**
    /// (`docs/COMBAT.md` §59). `DeathObj::inc_time` bumps the dead slot's
    /// `hold_frames` every frame, `Objects::process_all` takes one off, and
    /// `Objects::find_free` skips a dead number whose hold is not zero.
    /// run146's arena-A hoplites are born into 9–11 for this reason, and
    /// not into the dead 6–8.
    #[test]
    fn a_dead_unit_s_number_is_held_while_its_death_object_lives() {
        let (mut sim, ty) = at_war();
        let a = put(&mut sim, 1, ty, Pos::new(0x1000, 0x1000));
        put(&mut sim, 1, ty, Pos::new(0x1400, 0x1000));
        sim.units[a].health = 0;
        sim.hold_dead_slot(a);
        sim.deaths.push(combat::Death {
            who: 1,
            o: 0,
            first_frame: 0,
            cur_anim: 17,
        });
        for _ in 0..40 {
            sim.tick();
            assert!(sim.units[a].hold_frames > 0, "the death object holds 1/0");
        }
        assert_eq!(
            sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
            Some(2),
            "a held number is not handed out"
        );
        // The death object's end: nothing bumps it, and `process_all`
        // takes one off a frame — from `Object::close`'s thirty, which the
        // bump and the take have held level (§59.3).
        assert_eq!(sim.units[a].hold_frames, Sim::CLOSE_HOLD);
        sim.deaths.clear();
        for _ in 1..Sim::CLOSE_HOLD {
            sim.tick();
        }
        assert_eq!(sim.units[a].hold_frames, 1, "one frame short");
        assert_eq!(
            sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
            Some(2),
            "still held"
        );
        sim.tick();
        assert_eq!(sim.units[a].hold_frames, 0);
        assert_eq!(
            sim.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE),
            Some(0),
            "a number nothing holds is reused"
        );
    }

    /// **A rolled shot comes down on the first step its arc is not above
    /// the ground** (`docs/COMBAT.md` §46.1), at the point on its line
    /// that step names, and damages what stands there — not before.
    #[test]
    fn a_rolled_shot_comes_down_on_the_first_step_the_ground_meets_it() {
        let (mut sim, ty) = at_war();
        let side = (sim.world.corner_stride() * sim.world.corner_stride()) as usize;
        assert!(
            sim.world.set_corner_grid(vec![150_000_000; side]),
            "a square world's grid"
        );
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let (launch, landing) = (Pos::new(0x1000, 0x1000), Pos::new(0x1000 + 700, 0x1000));
        let (sz, ez, total) = (400, 200, 7);
        let v1z = combat::arc_v1z(sz, ez, total);
        let ground = crate::single::Single::from_i32(150);
        let down = (total..=3 * total)
            .find(|&t| !combat::arc_z(v1z, sz, t).gt(ground))
            .expect("the arc meets flat ground at 150 inside three flight times");
        assert!(
            down > total,
            "the shot flies past its landing point first: {down}"
        );
        let at = combat::arc_point(launch, landing, down, total);
        let foe = put(&mut sim, 1, ty, at);
        let full = sim.units[foe].health;
        sim.add_ammo(shot(me, launch, landing, total, sz, ez));
        for step in 1..=down {
            sim.process_projectiles(i64::from(step));
            let hit = sim.units[foe].health < full;
            assert_eq!(
                hit,
                step == down,
                "step {step}: the shot lands on {down} and only then"
            );
        }
        assert!(sim.projectiles.is_empty());
        assert_eq!(
            sim.ground_inexact, 0,
            "a grid of whole numbers is every corner's own single"
        );
    }

    /// A three-figure chain, `a → b → c`, on player 0.
    fn chain3(sim: &mut Sim, ty: usize) -> [usize; 3] {
        let a = put(sim, 0, ty, Pos::new(0x1000, 0x1000));
        let b = put(sim, 0, ty, Pos::new(0x1030, 0x1000));
        let c = put(sim, 0, ty, Pos::new(0x1060, 0x1000));
        for (f, up, down) in [
            (a, None, Some(b)),
            (b, Some(a), Some(c)),
            (c, Some(b), None),
        ] {
            sim.units[f].captain = up.is_none();
            sim.units[f].o_up = up;
            sim.units[f].o_down = down;
            sim.units[f].combat.captain = i32::from(sim.units[a].index);
        }
        [a, b, c]
    }

    fn links(sim: &Sim, f: usize) -> (Option<usize>, Option<usize>) {
        (sim.units[f].o_up, sim.units[f].o_down)
    }

    /// **A dying head hands the squad down, and goes to the tail**
    /// (`docs/COMBAT.md` §47.2, `Unit::close@0060ee50`'s relink). run112's
    /// `1/6` dies on 743 and the dump's next block carries `1/7` with
    /// `o_up -1 o_down 8`, a chain of 7 → 8 → 6. The new head is a captain
    /// from that frame (`UnitData::is_captain` is `o_up < 0`), and every
    /// figure's captain now reads it, the dead slot's included, because
    /// `get_captain` walks `o_up`.
    ///
    /// Made to fail on purpose: with the relink skipped, the golden
    /// widening's `o_up` row parts on block 744 (`1/7` ours 6, theirs −1)
    /// and chapter two's word falls back to 762.
    #[test]
    fn a_dead_captain_hands_the_squad_to_the_figure_below() {
        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        sim.units[a].health = 0;
        sim.relink_squad(a);
        assert!(sim.units[b].captain, "the figure below is the captain now");
        assert_eq!(links(&sim, b), (None, Some(c)));
        assert_eq!(links(&sim, c), (Some(b), Some(a)));
        assert_eq!(links(&sim, a), (Some(c), None), "the dead head is the tail");
        let bi = i32::from(sim.units[b].index);
        for f in [a, b, c] {
            assert_eq!(sim.units[f].combat.captain, bi);
        }
        assert_eq!(sim.squad_captain(c), b);
    }

    /// **A dying tail changes nothing its captain can see.** The unlink
    /// sets `above.o_down = −1` and the append puts it straight back:
    /// run112's `0/10` still carries `o_down 11` on every block after
    /// `0/11` dies on 729. And a dying middle figure is moved behind the
    /// one below it, so the chain the living walk is unbroken.
    #[test]
    fn a_dead_tail_stays_linked_and_a_dead_middle_goes_behind() {
        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        sim.units[c].health = 0;
        sim.relink_squad(c);
        assert_eq!(links(&sim, b), (Some(a), Some(c)));
        assert_eq!(links(&sim, c), (Some(b), None));
        assert!(sim.units[a].captain && !sim.units[b].captain);

        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        sim.units[b].health = 0;
        sim.relink_squad(b);
        assert_eq!(links(&sim, a), (None, Some(c)));
        assert_eq!(links(&sim, c), (Some(a), Some(b)));
        assert_eq!(links(&sim, b), (Some(c), None));
        assert!(sim.units[a].captain, "a middle death moves no captaincy");
    }

    /// The relink is on the death path itself: a head killed by damage
    /// leaves its squad led by the next figure, with nothing called by
    /// hand. A dead head over a dead figure is the whole-squad arm and
    /// promotes nobody.
    #[test]
    fn a_captain_killed_by_damage_is_succeeded() {
        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        sim.units[a].health = 1;
        let hit = combat::Sixteenths { whole: 50, frac: 0 };
        let taken = sim.take_damage(Obj::Unit(a), hit, Obj::Unit(foe), 10);
        assert!(matches!(taken, Taken::Died { .. }));
        assert!(sim.units[b].captain);
        assert_eq!(sim.squad_captain(c), b);

        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        for f in [c, b] {
            sim.units[f].health = 0;
            sim.relink_squad(f);
        }
        sim.units[a].health = 0;
        let before = sim.units.iter().map(|u| u.captain).collect::<Vec<_>>();
        sim.relink_squad(a);
        let after = sim.units.iter().map(|u| u.captain).collect::<Vec<_>>();
        assert_eq!(
            before, after,
            "nobody alive below the head, nobody promoted"
        );
    }

    /// **The lead is the target's first figure's `avg_speed`, along its
    /// angle** (`docs/COMBAT.md` §47.4, `Ammo::init@0067bbf0`,
    /// `0067ceb4`-`0067cec7`), and not the unit's per-frame speed. Two
    /// copies of one sim fire the same shot, the draws identical, and
    /// differ only in the target figure's average. The landings differ by
    /// exactly `sinx`/`cosx` of that average times the flight time,
    /// whatever the unit's `speed` says.
    #[test]
    fn the_lead_is_the_first_figure_s_average_speed() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
        let facing = crate::movement::Angle(723_976_192);
        sim.add_move_order(
            foe,
            Pos::new(0x1800, 0x1000),
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::First,
            false,
        );
        {
            let m = &mut sim.units[foe].movement;
            m.dest = Some(Pos::new(0x1800, 0x1000));
            m.facing = facing;
            m.heading = facing;
            m.speed = 24;
        }
        let fire = |avg: i32| {
            let mut s = sim.clone();
            s.units[foe].movement.body.avg_speed = avg;
            s.fire_ammo(
                Obj::Unit(me),
                Obj::Unit(foe),
                Angle(0),
                0,
                Pos::new(0x1000, 0x1000),
                0,
            );
            let p = *s.projectiles.last().expect("a shot");
            (p.landing, p.total_time)
        };
        let (still, t) = fire(0);
        let (moving, t2) = fire(9);
        assert_eq!(t, t2, "the flight time is taken before the lead");
        assert_eq!(
            (moving.x - still.x, moving.y - still.y),
            (
                crate::movement::sin_component(facing, 9) * t,
                -crate::movement::cos_component(facing, 9) * t
            ),
        );
    }

    /// **The lead's gate is the target's order, and its angle is
    /// `UnitData +0x50`** (item 1081, `docs/COMBAT.md` §74). `Ammo::init`
    /// asks `UnitData::order_type` of `is_move@0046f050`, then of
    /// `is_air@0046f000` (`67ce62`-`67ce99`), and leads along `angle`
    /// (`67ceba`), this crate's `heading`. run373's `0/1` on 5024 fled
    /// between two legs, a `FLEE_TO` head with no `movement.dest`: the
    /// original led `1/11`'s round onto it and this crate, asking
    /// `movement.dest`, missed. A target with a destination and no move
    /// at its head is not led.
    #[test]
    fn the_lead_asks_the_target_s_order_and_leads_along_its_angle() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
        let heading = crate::movement::Angle(117_506_048);
        let facing = crate::movement::Angle(550_174_720);
        let fire = |s: &Sim, avg: i32| {
            let mut s = s.clone();
            s.units[foe].movement.body.avg_speed = avg;
            s.fire_ammo(
                Obj::Unit(me),
                Obj::Unit(foe),
                Angle(0),
                0,
                Pos::new(0x1000, 0x1000),
                0,
            );
            let p = *s.projectiles.last().expect("a shot");
            (p.landing, p.total_time)
        };
        // A flight between two legs: the head is `FLEE_TO`, the body has no
        // destination, and the figure still faces the old leg.
        let mut fleeing = sim.clone();
        fleeing.add_move_order(
            foe,
            Pos::new(0x1300, 0x0800),
            crate::orders::MoveKind::FleeTo,
            crate::orders::QueuePos::First,
            false,
        );
        {
            let m = &mut fleeing.units[foe].movement;
            m.dest = None;
            m.heading = heading;
            m.facing = facing;
        }
        let (still, t) = fire(&fleeing, 0);
        let (moving, _) = fire(&fleeing, 23);
        assert_eq!(
            (moving.x - still.x, moving.y - still.y),
            (
                crate::movement::sin_component(heading, 23) * t,
                -crate::movement::cos_component(heading, 23) * t
            ),
            "a fleeing target is led along its angle, whatever its body's \
             destination and its figure's facing"
        );
        // A destination with no move at the head: not led.
        let mut standing = sim.clone();
        {
            let m = &mut standing.units[foe].movement;
            m.dest = Some(Pos::new(0x1800, 0x1000));
            m.heading = heading;
        }
        assert_eq!(
            fire(&standing, 23).0,
            fire(&standing, 0).0,
            "no move and no air order at the head, no lead"
        );
    }

    /// **A landed shot strikes at its own launch-to-landing bearing**
    /// (`docs/COMBAT.md` §47.3): `find_angle(ex − sx, ey − sy)`, whatever
    /// the target did while the shot flew. run112's `0/9` shot `1/7` on
    /// 766, `1/7` walked on, and the original struck at the shot's bearing
    /// on flank level 2 where the target's current position gives 1.
    #[test]
    fn a_shot_strikes_at_its_own_bearing() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1000));
        sim.fire_ammo(
            Obj::Unit(me),
            Obj::Unit(foe),
            Angle(0),
            0,
            Pos::new(0x1000, 0x1000),
            0,
        );
        let mut p = *sim.projectiles.last().expect("a shot");
        p.launch = Pos::new(964, 8180);
        p.landing = Pos::new(1717, 8421);
        assert_eq!(p.bearing(), crate::movement::find_angle(753, 241));
        assert_eq!(
            combat::flank_level(crate::movement::Angle(724_631_552), p.bearing()),
            Some(2),
            "run112's own numbers: 1/7's facing on 770 and 0/9's shot"
        );
    }

    /// **A hit marks the whole squad in danger, and every figure's guy 0**
    /// (item 530, `docs/ORDERS.md` §22). `Object::take_damage@00652020`
    /// calls `Unit::set_in_danger(this, 0)` for any unit hit by anything
    /// but attrition. That climbs to the captain and walks `o_down`, so
    /// hitting the tail marks the head. The attacker is not the victim's
    /// squad and is not marked here.
    ///
    /// Made to fail on purpose: with the call out of `take_damage`, golden
    /// chapter one's `in_danger` row parts on 617 on all six hoplites and
    /// its word falls back to 774.
    #[test]
    fn a_hit_on_the_tail_marks_the_whole_squad_in_danger() {
        let (mut sim, ty) = at_war();
        let [a, b, c] = chain3(&mut sim, ty);
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        let hit = combat::Sixteenths { whole: 5, frac: 0 };
        let taken = sim.take_damage(Obj::Unit(c), hit, Obj::Unit(foe), 10);
        assert!(matches!(taken, Taken::Alive { .. }));
        for f in [a, b, c] {
            assert!(sim.units[f].in_danger, "figure {f} was not marked");
            assert!(
                sim.units[f].guy_flag_0x20,
                "figure {f}'s guy 0 was not marked"
            );
        }
        assert!(
            !sim.units[foe].in_danger,
            "the attacker is not the victim's squad"
        );
    }

    /// **A hit stamps the struck object's owner's `frame_attacked`, at
    /// difficulty below 2** (item 729, `docs/AI.md` §71).
    /// `Object::take_damage@00652020:67-71` writes
    /// `leaders[owner].frame_attacked = frame` for any combat hit, and
    /// `Army::find_target`'s difficulty gate keeps a leader out for 7,200
    /// frames after it. This crate wrote the field only from `find_target`,
    /// so Great Lakes' human stood at 8186 where the original's stood at
    /// 10233, and on 15608 the gate reached the coin the original skips.
    ///
    /// Made to fail on purpose: with the stamp out of `take_damage`, the
    /// first assertion reads 0.
    #[test]
    fn a_hit_stamps_the_struck_owner_s_frame_attacked_below_difficulty_two() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x1100, 0x1000));
        let hit = combat::Sixteenths { whole: 1, frac: 0 };
        sim.lobby.difficulty = 1;
        sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10_233);
        assert_eq!(sim.ai[0].frame_attacked, 10_233, "the struck owner's stamp");
        assert_eq!(sim.ai[1].frame_attacked, 0, "never the attacker's");
        sim.lobby.difficulty = 2;
        sim.take_damage(Obj::Unit(me), hit, Obj::Unit(foe), 10_300);
        assert_eq!(
            sim.ai[0].frame_attacked, 10_233,
            "difficulty 2 and above: no stamp"
        );
    }

    /// **An attacker marks its own squad and its target's on every
    /// `Unit::work`** (`0060d180:283-313`), whether or not it strikes:
    /// the mark is above the dispatch, keyed on the action being an
    /// `ATTACK` with a target. A target out of reach is still marked.
    #[test]
    fn an_attack_order_marks_both_squads_on_its_own_work() {
        let (mut sim, ty) = at_war();
        let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
        let foe = put(&mut sim, 1, ty, Pos::new(0x6000, 0x1000));
        sim.add_attack_order(
            me,
            Obj::Unit(foe),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        assert!(!sim.units[me].in_danger && !sim.units[foe].in_danger);
        sim.work(me, 1);
        assert!(sim.units[me].in_danger, "the attacker marks itself");
        assert!(sim.units[foe].in_danger, "and the unit it names");
    }

    /// **A reloading unit handed a fresh attack does not turn: it asks for
    /// the idle** (item 530, `Unit::fight@005fd4d0:100-127`). The first
    /// reading had it turn toward the target and return, and golden
    /// chapter one's `1/7` on 774 says otherwise: heading unchanged, and
    /// one `Guy::set_anim+0x97a` under `Unit::fight+0x169`, which is the
    /// chapter's word. Guy 0 already aimed at the target asks for nothing.
    #[test]
    fn a_reloading_unit_asks_for_the_idle_and_does_not_turn() {
        for aimed in [false, true] {
            let (mut sim, ty) = at_war();
            let me = put(&mut sim, 0, ty, Pos::new(0x1000, 0x1000));
            let foe = put(&mut sim, 1, ty, Pos::new(0x6000, 0x3000));
            let mut g = crate::anim::Guy::fresh(0);
            g.anim = crate::anim::WALK;
            g.aim = aimed.then_some(Obj::Unit(foe));
            sim.units[me].guys = vec![g];
            sim.add_attack_order(
                me,
                Obj::Unit(foe),
                crate::orders::QueuePos::First,
                false,
                false,
            );
            sim.units[me].combat.recharging = 13;
            let heading = sim.units[me].movement.heading;
            sim.work(me, 1);
            assert_eq!(
                sim.units[me].movement.heading, heading,
                "aimed {aimed}: a reloading unit does not turn"
            );
            let asked = sim.units[me].guys[0].anim != crate::anim::WALK;
            assert_eq!(asked, !aimed, "aimed {aimed}: the idle request");
        }
    }

    /// **The attack-move looks around one frame in fifteen, phased by
    /// `o`, and stacks what it finds `QUEUE_FIRST`** (item 530,
    /// `Unit::do_attack_to@005f2320`). This crate's attack-move never
    /// looked, so golden chapter one's `1/7` walked past `0/8` on 773
    /// where the original's took an `ATTACK` above its march.
    #[test]
    fn the_attack_move_looks_around_one_frame_in_fifteen() {
        for (frame, looks) in [(15, true), (16, false)] {
            let (mut sim, ty) = at_war();
            let me = put(&mut sim, 0, ty, Pos::new(0x1200, 0x1200));
            let foe = put(&mut sim, 1, ty, Pos::new(0x1300, 0x1200));
            let dest = Pos::new(0x4000, 0x1200);
            sim.units[me].index = 0;
            sim.add_move_order(
                me,
                dest,
                crate::orders::MoveKind::AttackTo,
                crate::orders::QueuePos::New,
                true,
            );
            let dest = sim.units[me].orders[0]
                .move_dest()
                .expect("the attack-move");
            sim.do_attack_to_tail(me, frame, dest);
            let front = sim.units[me]
                .orders
                .front()
                .map(crate::orders::Order::index);
            assert_eq!(
                front == Some(crate::orders::index::ATTACK),
                looks,
                "frame {frame}: {:?}",
                sim.units[me].orders
            );
            if looks {
                assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(foe)));
                assert_eq!(
                    sim.units[me].orders.back().map(crate::orders::Order::index),
                    Some(crate::orders::index::ATTACK_TO),
                    "the attack-move stays under the attack"
                );
            }
        }
    }

    /// run190's guard, in miniature: a player's chariot-ranged unit on its
    /// post (3480, 12264) guarding a standing wagon, with an attack on an
    /// enemy stacked above its `GUARD`.
    fn guard_on_post(enemy_at: Pos) -> (Sim, usize, usize) {
        let (mut sim, ty) = at_war();
        sim.nation[0].human = true;
        sim.nation[1].human = true;
        let mut t = sim.unit_types[ty].clone();
        t.combat.max_range = 8;
        // A chariot is a combat unit (`role & 0x10000`), which is what puts
        // a guard on duty (`Unit::on_duty@005fff70`, item 1040).
        t.combat.combat_role = true;
        let chariot = sim.add_unit_type(t);
        let post = Pos::new(3480, 12264);
        let wagon = put(&mut sim, 0, chariot, Pos::new(3456, 11904));
        let guard = put(&mut sim, 0, chariot, post);
        let enemy = put(&mut sim, 1, chariot, enemy_at);
        sim.add_guard_order(guard, wagon, 0, 372, crate::orders::QueuePos::New);
        if let crate::orders::Body::Guard(g) = &mut sim.units[guard].orders[0].body {
            g.guard = post;
        }
        (sim, guard, enemy)
    }

    /// A frame off `do_guard`'s two sixteen-frame phases and the review's.
    fn off_phase(sim: &Sim, u: usize) -> i64 {
        let o = i64::from(sim.units[u].index);
        (3 - o).rem_euclid(16) + 16
    }

    /// **A guard's attack is leashed to its post** (`Unit::fight@005fd4d0`
    /// `005fdd0c`, `check_target@00649e00`'s guarding arm,
    /// `docs/COMBAT.md` §63.1–§63.2). run190's own tick 1036: the enemy on
    /// (3356, 14040) is ≈1,780 from the post, past `8 × 2 × 0x60` = 1,536,
    /// so the unrecharged `fight` kills the attack with no draw, and the
    /// guard's own search, leashed too, finds nothing and leaves `near`
    /// empty. At 1,400 the attack stands.
    ///
    /// Made to fail first with the arm removed from `do_attack`: the
    /// attack stood and fired, the chapter's 1037 rows.
    #[test]
    fn a_guard_drops_an_attack_whose_target_leaves_its_leash() {
        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3356, 14040));
        sim.add_attack_order(
            guard,
            Obj::Unit(enemy),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        let seed = sim.rng.seed;
        let f = off_phase(&sim, guard);
        sim.work(guard, f);
        assert_eq!(sim.order_type(guard), crate::orders::index::GUARD);
        assert_eq!(
            sim.units[guard].orders.len(),
            1,
            "{:?}",
            sim.units[guard].orders
        );
        assert_eq!(
            sim.units[guard].near, None,
            "the leashed search saw nothing"
        );
        assert_eq!(sim.rng.seed, seed, "a human's guard spends no draw");

        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3480, 13664));
        sim.add_attack_order(
            guard,
            Obj::Unit(enemy),
            crate::orders::QueuePos::First,
            false,
            false,
        );
        let f = off_phase(&sim, guard);
        sim.work(guard, f);
        assert_eq!(
            sim.units[guard].combat.target,
            Some(Obj::Unit(enemy)),
            "inside the leash the attack stands"
        );
    }

    /// **The guard's search is leashed and centred on its post**
    /// (`find_nearby_target@00648da0`'s `local_2c`, §63.3): the enemy at
    /// ≈1,635 from the post is inside the search's radius, and an idle
    /// unit on the same spot takes it; a guard does not, and its `near`
    /// stays empty. Parked 705's non-re-engagement, in miniature.
    ///
    /// Made to fail first with the leash out of the candidate loop.
    #[test]
    fn a_guard_s_search_refuses_what_is_past_its_leash() {
        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
        assert_eq!(sim.find_melee_target(guard, -1), None);
        assert_eq!(sim.units[guard].near, None);
        sim.units[guard].orders.clear();
        assert_eq!(
            sim.find_melee_target(guard, -1),
            Some(Obj::Unit(enemy)),
            "unleashed, the same unit takes it"
        );
    }

    /// **A human's action order holds its retaliation**
    /// (`Unit::target_opportunity@005fffc0`, `LAB_00600877`, §63.4): a
    /// player's guard under its `GUARD` (flags `ACTION`) does not answer a
    /// hit; the same unit idle does, and so does an AI's guard
    /// (`unit_masks & 0x40000`). run190's 1091.
    ///
    /// Made to fail first with the gate removed: the human guard took the
    /// attack.
    #[test]
    fn a_human_guard_does_not_answer_a_hit() {
        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
        sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
        assert_eq!(sim.order_type(guard), crate::orders::index::GUARD);
        assert_eq!(sim.units[guard].combat.target, None);

        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
        sim.nation[0].human = false;
        sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
        assert_eq!(
            sim.units[guard].combat.target,
            Some(Obj::Unit(enemy)),
            "an AI's guard"
        );

        let (mut sim, guard, enemy) = guard_on_post(Pos::new(3384, 13896));
        sim.units[guard].orders.clear();
        sim.target_opportunity(guard, Obj::Unit(enemy), 1091);
        assert_eq!(
            sim.units[guard].combat.target,
            Some(Obj::Unit(enemy)),
            "idle"
        );
    }
}
