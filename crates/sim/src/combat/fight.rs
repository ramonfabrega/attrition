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
pub use crate::ids::ty::HOPLITES;

/// `Ammo::inc_time@0067d380`'s `cur_time < 200` on a round whose `flags & 2`
/// is clear — the frames a spent arrow keeps its slot (item 3679,
/// `docs/COMBAT.md` §46.7). run1353b: who=1's Royal Hwarang `1/139`'s
/// arrow reads `cur_time` 57 on block 4195 and 199 on 4337, and is gone on
/// 4338.
pub const SPENT_LIFE: i32 = 200;

pub const SITE_ATTACK_POS_FIGHT: &str = "Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4";

/// `TypeIndex::FIRESHIP` (`0x150`), the lineage `Object::compare_target`'s
/// sea raid arm tests the attacker and the target against (`0064ede2`).
pub const FIRESHIP: crate::tech::TypeId = 0x150;

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

/// `Objects::add_ammo@00658b10`'s scan, `0x658b36`–`0x658b56`: from slot 0 the
/// first whose `flags & 3` is clear, else the pool's size — over the slots in
/// use, in ascending order, the first number not among them (the sweep
/// `sweep::objects_add_ammo` holds it to the original's pool).
pub(crate) fn lowest_free_slot(live: &[u32]) -> u32 {
    let mut slot = 0;
    for &q in live {
        if q == slot {
            slot += 1;
        } else {
            break;
        }
    }
    slot
}

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

/// `Object::fire_ammo@0064c8b0+0x3d5` — a building's launch point in its
/// footprint, the **x** draw (`64cc80`'s call, `idivl` by `x_size × 0x60`);
/// none when the span is under two (item 3970: chapter 126's 7 × 7 City
/// `1/2000` on 752, where a 2 × 2 Tower's draws had always been taken
/// unnamed and matched by count).
pub const SITE_FIRE_FOOTPRINT_X: &str = "Object::fire_ammo+0x3d5";

/// `Object::fire_ammo@0064c8b0+0x406` — the footprint's **y** draw.
pub const SITE_FIRE_FOOTPRINT_Y: &str = "Object::fire_ammo+0x406";

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

/// `Unit::plunder@00605b00+0x4e5` — **the plunder's pick of a good**
/// (`docs/AI.md` §259): one `Random::get(0, 0xffff)` per good the killer's
/// leader has available, knowledge skipped, on the EE arm (`00605fe0`).
pub const SITE_UNIT_PLUNDER: &str = "Unit::plunder+0x4e5";

/// `Unit::close@0060ee50+0xce6` — the **ammo-graphic** death's second
/// draw, taken only when `dtype == 4`.
///
/// `dtype` 4 is `do_damage`'s override for an ammo whose graphic piece
/// carries flag `0x10` (§7.1 step 6), and it replaces the anim outright:
/// `cur_anim = roll % 2 + 0x11`, then [`SITE_DEATH_ANIM_FACING`] picks
/// the facing. Named from the listing; the first capture to reach it is
/// run1202, the Fire Lances `1/31`'s `flame="1"` round on the Citizen
/// `0/10` at 3030 (item 3580, `docs/AI.md` §313).
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

/// The ring draw of the computer's siege arm — `Unit::fight@005fd4d0`'s
/// packed branch asking for a better spot against an armed building
/// (`fight:533`, returning to `+0x115e`, item 1937).
pub const SITE_ATTACK_POS_SIEGE: &str = "Unit::find_attack_pos+0xea9 < Unit::fight+0x115e";

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
            Obj::Unit(i) => self.units[i].ty.map_or_else(no_profile, |t| {
                let mut p = self.unit_types[t].combat;
                if self.second_range == Some(t) {
                    p.max_range = p.second_max_range;
                }
                p
            }),
            Obj::Building(b) => self.buildings[b].combat.unwrap_or_default(),
        }
    }

    /// **`f` with a melee-and-ranged type's second range in its
    /// `max_range`** — the bracket `Unit::cavarch_fight@005ff4b0` puts
    /// round its `fight` (`005ff4c5`, `005ff4f0`): for a type whose flags
    /// carry `0x400` the type's `+0x1fc` reads its `+0x2d0` for the call
    /// and 0 after it, which is the value [`uflags::MELEE_AND_RANGED`]
    /// leaves there at load ([`Sim::second_range`]).
    pub(crate) fn with_second_range<R>(&mut self, u: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        let swap = self.units[u]
            .ty
            .filter(|&t| self.unit_types[t].cols.flag(uflags::MELEE_AND_RANGED));
        let Some(t) = swap else {
            return f(self);
        };
        let held = self.second_range.replace(t);
        let r = f(self);
        self.second_range = held;
        r
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

    /// **A thing a landing round or a blow can still strike**
    /// (`Ammo::hit_target@00678f90`, `Object::do_damage@0064a480`): the
    /// slot's `flags & 1` and, for a unit, `is_on_map` — and nothing about
    /// hit points. [`Sim::active`] also asks a building for health left,
    /// which is the target *search's* question; a city building at its
    /// ceiling (`health` 0) is still struck by the round already in the air
    /// (item 1528: run668's second siege shot on 15377, which charges the
    /// army a second time).
    pub(crate) fn struck(&self, o: Obj) -> bool {
        match o {
            Obj::Unit(_) => self.active(o),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some()),
        }
    }

    /// `attack()` — the type's, plus the player's flat modifier (§4.1).
    ///
    /// The base is `ObjectData::attack@006469f0`'s
    /// ([`crate::object_attack_view::object_attack_steps`], swept: item
    /// 3387). Its caravan arm replaces a caravan's attack with
    /// `CARAVAN_ATTACK_BONUS` once its owner holds that bonus's
    /// prerequisite, and its Dutch term (item 3329) is inside it: a Dutch
    /// kind ([`Sim::dutch_ages`]) adds `ages × DUTCH_ATTACK_BONUS × 10`
    /// **before** `UnitData::attack`'s zero test, so an armed merchant of
    /// an age past the first is a combatant on a type with no attack.
    pub fn attack_of(&self, o: Obj) -> i32 {
        let base =
            crate::object_attack_view::object_attack_steps(&mut SimObjectAttack { sim: self, o });
        if base == 0 {
            return 0;
        }
        let family = match o {
            Obj::Building(b) => self.build_attack_terms(b),
            Obj::Unit(_) => 0,
        };
        base + self.mods_of(self.owner_of(o) as usize).attack
            + self.citizen_attack(o)
            + self.monarch_cav_attack(o)
            + family
    }

    /// **`UnitData::attack@006103c0`'s Monarch arm** (item 3688, `docs/AI.md`
    /// §349), the override's last term (`610800`–`61083c`): a unit whose
    /// type's trainer is the Stable (`type +0x40 == 0x1ac`, by equality —
    /// [`Sim::trainer_ident`]) and which has one of its owner's Monarchs
    /// (`0x162`) in reach — `has_general(0, 0x162) >= 0`,
    /// [`Sim::has_general`] — adds `THEMONARCH_CAV_ATTACK × 10`. The gate
    /// ahead of both, the leader's short `+0x59c2`, is `+0x56fe`'s
    /// per-type hero count at `0x162` (`0x56fe + 2 × 0x162`), which a live
    /// Monarch the walk finds already implies. run1357's packet (logger
    /// frame 3439 of the Russian Gunpowder Great Lakes game) runs the
    /// original's `get_damage` on who=1's Heavy Knight `1/32` and the
    /// human's Citizen `0/5`: table 150, armour 0 and attack **190**, the
    /// base 170 and 20 off this arm (`has_general` answered 18 at
    /// `0x61082a`, the Monarch `1/18` the knight guards), so the blow is 29
    /// where this crate dealt 26.
    ///
    /// SEAM: the leader's per-type count is not read (a Monarch counted
    /// and off the map), as in [`Sim::near_monarch`].
    fn monarch_cav_attack(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u)
                if self.units[u].ty.is_some_and(|ty| {
                    self.trainer_ident(ty) == Some(crate::build::Ident::Stable)
                }) && self.has_general(u, 0x162) =>
            {
                self.tuning.themonarch_cav_attack * 10
            }
            _ => 0,
        }
    }

    /// **`UnitData::attack@006103c0`'s Citizen arm** (item 3677, `docs/AI.md`
    /// §346): a unit with one of its owner's Citizens (`0x164`, THECITIZEN)
    /// in reach — `has_general(0, 0x164) >= 0`, [`Sim::has_general`] — adds
    /// `THECITIZEN_ATTACK_BONUS × 10`. run1351c's packet (logger frame 3521
    /// of the German Industrial Great Lakes game) runs the original's
    /// `get_damage` on who=1's Armored Car `1/27` and the human's Citizen
    /// `0/13`: table 170, armour 0 and attack **220**, the base 200 and 20
    /// off this arm (`has_general` answered 17, the Citizen `1/17` 368
    /// units off), so the blow is 37 where this crate dealt 34.
    ///
    /// SEAM: the override's other terms (Terra Cotta, the Obsidian Archers,
    /// the Space Program's scale and the named generals') — the view
    /// [`crate::unit_attack_view`] holds them all, swept, unwired.
    fn citizen_attack(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u) if self.has_general(u, 0x164) => self.tuning.thecitizen_attack_bonus * 10,
            _ => 0,
        }
    }

    /// `ages × DUTCH_ATTACK_BONUS` for a unit of a Dutch kind, else 0
    /// (`ObjectData::attack`'s and `::armor`'s shared arm; a building is
    /// never one).
    fn dutch_term(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(u) => self.units[u].ty.map_or(0, |rec| {
                self.dutch_ages(self.units[u].owner, rec) * self.tuning.dutch_attack_bonus
            }),
            Obj::Building(_) => 0,
        }
    }

    /// The damage formula's modifiers of an owner: the player's, or none
    /// for a gaia owner (animals, goods), which the leaders' list holds as
    /// zeroes and `mods` — one a player — does not (a nuke's ring strikes
    /// the animals around a city, `docs/AI.md` §158).
    ///
    /// **The Japanese level is read live** (item 3590): `get_damage@
    /// 00644130:243`–`262` asks `has_tribe_bonus(0xf)` of the attacker's
    /// leader on every blow and takes `n = min(ages, epoch[0])` off its
    /// `LeaderData` there, so nothing caches it; before this nothing wrote
    /// `japanese_level` at all (parked 3510a), and run1208d's Arquebusiers
    /// `1/20` struck the Citizen `0/12` for 12 where the original's +15 %
    /// (ages 3, `epoch[0]` 3) struck 13 + 10/16 (`docs/AI.md` §316).
    fn mods_of(&self, owner: usize) -> combat::Modifiers {
        let mut m = self.mods.get(owner).copied().unwrap_or_default();
        if let Some(p) = self.tech.get(owner)
            && self.tech_tree.has_tribe_bonus(&self.setup, p, 0xf)
        {
            m.japanese_level = Some(p.ages.min(p.epoch[crate::tech::Line::Military.index()]));
        }
        m
    }

    /// `armor()` (§4.2). **A building that is not finished holds half its
    /// armour** (item 1350): a building's `armor()` is vslot `+0x124`,
    /// `WallData::armor@0063fa60` (read off the PE at `Build::vftable +
    /// 0x124`), and its tail at `63fb25`–`63fb41` tests `is_active`
    /// (`flags & 4`) and, when it is clear, halves with `cltd; sub; sar`
    /// — toward zero, as `/ 2` does here. run514's Keep site, armour 4,
    /// took a Hoplite's blow at 2: 13 whole hits, 69 sixteenths a figure.
    ///
    /// **The Senator's arm above the halving** (`63fab0`–`63fb1f`, item
    /// 3986, `docs/GOLDEN.md` §179): with a leader whose `+0x59c0` is set
    /// (`+0x56fe`'s per-type count at `0x161`), `HeroesData::find_hero@
    /// 0073a1b0` for one of the owner's `THESENATOR`s (`0x161`) whose
    /// radius covers the building less its footprint's `(type +0x234 +
    /// +0x238) × 96` adds `THESENATOR_BUILD_ARMOR` — before the halving,
    /// so an unfinished building under a Senator holds half the sum
    /// ([`Sim::hero_near_building`]).
    pub fn armor_of(&self, o: Obj) -> i32 {
        // `ObjectData::armor@00647db0`'s Dutch term (item 3329): `ages ×
        // DUTCH_ATTACK_BONUS` on a Dutch kind.
        let armor = self.profile(o).armor
            + self.dutch_term(o)
            + self.mods_of(self.owner_of(o) as usize).armor;
        match o {
            Obj::Unit(u) => armor + self.rally_armor(u),
            Obj::Building(b) => {
                let armor = armor + self.senator_build_armor(b);
                if self.buildings[b].active {
                    armor
                } else {
                    armor / 2
                }
            }
        }
    }

    /// `WallData::armor@0063fa60`'s Senator term: `THESENATOR_BUILD_ARMOR`
    /// when one of the owner's Senators stands within reach of `b`. The
    /// arm's first branch — the object itself `is(0x161)` behind vslot
    /// `+0x18` — is a unit's, never a building's.
    fn senator_build_armor(&self, b: usize) -> i32 {
        const THESENATOR: i32 = 0x161;
        if self.hero_near_building(b, THESENATOR) {
            self.tuning.thesenator_build_armor
        } else {
            0
        }
    }

    /// **`UnitData::armor@00610160`'s rally arm** (item 3385): a unit that
    /// is itself a hero, or that `HeroesData::find_hero@0073a1b0` places
    /// inside one of its owner's heroes' radii, adds `GENERAL_RALLY_ARMOR ×
    /// (general_upgrade + 1)`. `find_hero` is asked with no mask, no type
    /// and no reach (vslot `+0x20` is clear for a unit): any active hero
    /// on the map whose `get_radius × 0xc0` covers the unit's position by
    /// `vector_dist`. run1011e's packet: the Horse Archer `1/72`'s round on
    /// the Inti Maceman `1/61` on 13297, `get_damage` 13 at armour 4 (the
    /// cached `myarmor` 2 and the General's 2) where this crate dealt 15.
    ///
    /// **The patriots' terms inside it** (item 3986, `docs/GOLDEN.md`
    /// §179), each its own `has_general` walk: `has_general(0, 0x164)`
    /// (`THECITIZEN`, shipped as The Comrade) adds
    /// `THECITIZEN_ARMOR_BONUS`, and a Stable-trained unit (`type +0x40 ==
    /// 0x1ac`, [`Sim::trainer_ident`]) with `has_general(0, 0x162)` (The
    /// Monarch) adds `THEMONARCH_CAV_ARMOR`. Either hero in reach opens
    /// the gate itself, so the rally term is always beside them.
    ///
    /// SEAM: the arm's named generals between the two — Ptolemy, Darius,
    /// Wellington, Blücher, Memnon — are not carried; each is gated on a
    /// leader's counter of its own a skirmish leaves at 0 (territory
    /// adjudication 4, row 61).
    fn rally_armor(&self, u: usize) -> i32 {
        let me = &self.units[u];
        let rallied = (me.alive() && self.is_hero_unit(u))
            || (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == me.owner
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist(
                        (x.pos.x - me.pos.x).abs(),
                        (x.pos.y - me.pos.y).abs(),
                    ) <= self.hero_radius(h) * 0xc0
            });
        if !rallied {
            return 0;
        }
        let citizen = if self.has_general(u, 0x164) {
            self.tuning.thecitizen_armor_bonus
        } else {
            0
        };
        let monarch = if me
            .ty
            .is_some_and(|ty| self.trainer_ident(ty) == Some(crate::build::Ident::Stable))
            && self.has_general(u, 0x162)
        {
            self.tuning.themonarch_cav_armor
        } else {
            0
        };
        self.tuning.general_rally_armor * (self.general_upgrade_level(me.owner) + 1)
            + citizen
            + monarch
    }

    /// `max_range()` (§4.4): zero stays zero.
    pub fn max_range_of(&self, o: Obj) -> i32 {
        let p = self.profile(o);
        if p.max_range == 0 {
            return 0;
        }
        p.max_range
            + self.mods_of(self.owner_of(o) as usize).range
            + match o {
                Obj::Building(b) => self.building_range_terms(b),
                Obj::Unit(_) => 0,
            }
            + self.ceo_unit_range(o)
            + self.siege_unit_range(o)
    }

    /// **`UnitData::max_range@0060fe90`'s siege arm** (item 3380): a unit
    /// whose type is `is_siege` (the type's vslot `+0x10c`, the same test
    /// [`Self::ceo_unit_range`] refuses on) and whose leader
    /// `has_tribe_bonus(8)` adds `TURK_SIEGE_RANGE`, three tiles.
    /// `GreatLakesTurkishTier4`'s Trebuchet `1/70` walks its bombard ring
    /// round Napata `0/2000` on 14991 at a stand-off of 19 tiles' 3552, not
    /// 16's 2976, and stops at (7368, 29544) in run1005.
    ///
    /// SEAM: the same arm's Liberty (`0x219`, `LIBERTY_SIEGE_RANGE`) and
    /// Eiffel (`0x21c`, `EIFFEL_SIEGE_RANGE`) wonders are not carried; no
    /// capture holds either wonder (scan: `grep -h '^    wonder [0-9]'
    /// gamelog*.txt`, no wonder held in the 66 of 573 dumps that print
    /// `WONDERS`).
    fn siege_unit_range(&self, o: Obj) -> i32 {
        let Obj::Unit(u) = o else {
            return 0;
        };
        if !self.is_siege_unit(u) {
            return 0;
        }
        let who = self.units[u].owner as usize;
        if self.nation[who].turks {
            self.tuning.turk_siege_range
        } else {
            0
        }
    }

    /// **`UnitData::max_range@0060fe90`'s first block** (`docs/AI.md`
    /// §236): a unit whose type is not `is_siege` (the type's vslot
    /// `+0x10c`), whose leader counts a `0x165` (`+0x56fe + 0x165 × 2`,
    /// the per-type count at `+0x59c8`), and which `has_general(0, 0x165)`
    /// — it is the `0x165` hero itself, or `HeroesData::find_hero@0073a1b0`
    /// finds one of its owner's, active and on the map, within the hero's
    /// `get_radius × 0xc0` (`vector_dist(|dx|, |dy|)`, less a building's
    /// footprint, 0 for a unit) — adds `THECEO_UNIT_RANGE`, two tiles.
    /// run881's Strategic Bomber `1/137` bombs `0/2000` on 6452 at
    /// `attack_dist` 918, past three tiles' 582 and inside five's 966, with
    /// who=1's `1/161` (type 357) 2371 off.
    ///
    /// SEAM: of the siege arm below it only `TURK_SIEGE_RANGE` is carried
    /// (item 3380); the Liberty and Eiffel wonders and Napoleon are not —
    /// nor Ptolemy's general, the rum naval range
    /// and the obsidian archers are not carried; `is(0x165)` is taken as
    /// the type itself, as the patriot tests of the radius are.
    fn ceo_unit_range(&self, o: Obj) -> i32 {
        let Obj::Unit(u) = o else {
            return 0;
        };
        if self.near_own_ceo(u) {
            self.tuning.theceo_unit_range
        } else {
            0
        }
    }

    /// The CEO block `UnitData::max_range@0060fe90` and
    /// `UnitData::los@006100c0` share: the unit is not `is_siege` (the
    /// type's vslot `+0x10c`), its leader counts a `0x165` (`+0x59c8`), and
    /// `has_general(0, 0x165)` — it is the `0x165` hero itself, or one of its
    /// owner's stands active and on the map within the hero's
    /// `get_radius × 0xc0` (`vector_dist`).
    pub(crate) fn near_own_ceo(&self, u: usize) -> bool {
        const CEO: i32 = 0x165;
        if self.is_siege_unit(u) {
            return false;
        }
        let who = self.units[u].owner;
        let counted = self
            .unit_types
            .iter()
            .position(|t| t.type_index == CEO)
            .and_then(|r| self.muster.get(who as usize).and_then(|m| m.by_type.get(r)))
            .is_some_and(|&n| n != 0);
        if !counted {
            return false;
        }
        let at = self.units[u].pos;
        self.units[u].type_index == CEO && self.is_hero_unit(u)
            || (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == who
                    && x.type_index == CEO
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                        <= self.hero_radius(h) * 0xc0
            })
    }

    fn hits_left(&self, o: Obj) -> i32 {
        match o {
            Obj::Unit(i) => self.units[i].health.max(0),
            Obj::Building(b) => self.buildings[b].health.max(0),
        }
    }

    /// `WallData::in_unfriendly_territory@0063eca0`: the cell under the
    /// building is owned (`>= 0`) by a player who is not its owner and not
    /// allied with it both ways (`diplos == 2` each way). `get_damage` reads
    /// it through the target's vslot `+0x184` (step 18, `0063eca0` off the
    /// PE at `Build::vftable`, `BuildData::vftable` and `WallData::vftable`
    /// alike), and there it zeroes the armour (item 1375: run551's Tower
    /// site on who=0's claim took the Elite Pikemen's blow at 133
    /// sixteenths, where its halved armour 2 dealt 122). The same predicate
    /// as `Wall::process`'s enemy-land hit (`crate::city`).
    pub(crate) fn in_unfriendly_territory(&self, b: usize) -> bool {
        let who = self.buildings[b].owner;
        matches!(self.world.owner_at(self.buildings[b].pos),
            crate::Owner::Player(t) if !crate::place::friendly_ground(
                i32::from(t), i32::from(who), self.allied_with(who, t), self.allied_with(t, who)))
    }

    /// `Wall::process@00640450`'s enemy-land owner, `:209`–`:219`: the
    /// cell under the building, or for a dock (`is_dock`, the type's vslot
    /// `+0x108`) [`Sim::check_enemy_adjacent`]'s ring in its place; then
    /// the same three refusals — below 0, the owner's own, an ally both
    /// ways (item 3992, `docs/GOLDEN.md` §185). `get_damage`'s armour read
    /// ([`Self::in_unfriendly_territory`], `WallData::in_unfriendly_territory`)
    /// asks the own cell for a dock too.
    pub(crate) fn bleeds_in_enemy_land(&self, b: usize) -> bool {
        let dock = self.buildings[b]
            .ty
            .is_some_and(|t| crate::build::is_dock(&self.build_types, t));
        if !dock {
            return self.in_unfriendly_territory(b);
        }
        let who = self.buildings[b].owner;
        let o = self.check_enemy_adjacent(b);
        u8::try_from(o).is_ok_and(|p| {
            !crate::place::friendly_ground(
                o,
                i32::from(who),
                self.allied_with(who, p),
                self.allied_with(p, who),
            )
        })
    }

    /// The side of the damage formula an object presents as a **target**.
    fn target_side(&self, o: Obj, attacker: Obj) -> Side {
        match o {
            Obj::Unit(i) => {
                let u = &self.units[i];
                Side {
                    unit: true,
                    packed: u.combat.packed,
                    moving: self.is_moving(i),
                    attacks: self.attack_of(o) != 0,
                    entrenched: u.combat.entrenched,
                    facing: u.movement.heading,
                    trench_facing: u.combat.trench_angle,
                    // The object's own `z` (`+0xc`), which `Unit::update_z`
                    // writes as `find_tcoord_z` at its tile, clamped at zero
                    // (§46.2, item 1131).
                    z: self.world.object_z(u.pos.tile()),
                    damage_frame: u.combat.damage_frame,
                    damage_o: u.combat.damage_o,
                    captain: u.combat.captain,
                    // `unit_masks & 1`, step 17's arm: anything kills a
                    // decoy (item 3385: run1011e's packet, `get_damage`
                    // 19688 on the General's decoy Light Horse `1/82`,
                    // `take_damage` 0x19a2 after the uber divide).
                    decoy: u.decoy,
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
                    in_unfriendly_territory: self.in_unfriendly_territory(b),
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
                captain: crate::flag_reads::object_get_captain(self.buildings[b].index),
                z: self.world.object_z(self.buildings[b].pos.tile()),
                ..Side::default()
            },
        }
    }

    // ------------------------------------------------------------------
    // Distance and validity
    // ------------------------------------------------------------------

    /// `ObjectData::attack_dist(o, who, x, y)` from `at` (§13.1).
    ///
    /// **The centre-to-centre arm is the attacker's `is_plane`** (vslot
    /// `+0xc0`, `006488f0`; `UnitData::is_plane@0046ce40`, `domain == 2 &&
    /// !(unit_flags & 0x20)`) **and not a missile** (type `+0x1e4 &
    /// 0x8000000`): a helicopter measures edge to edge, as a ground unit
    /// does. This crate read the domain alone, so an Attack Helicopter's
    /// look at a city measured from the city's centre and passed it over
    /// (`docs/AI.md` §227: run866's `1/145` on 5135).
    pub fn attack_dist_from(&self, attacker: Obj, at: Pos, target: Obj) -> i32 {
        let ap = self.profile(attacker);
        let tp = self.profile(target);
        let plane = matches!(attacker, Obj::Unit(u) if self.is_plane(u)) && !ap.has(mask::MISSILE);
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
        self.in_range_from(attacker, at, target, margin, true)
    }

    /// `is_in_range@00648d70` as `006486b0`'s head has it: **the slot,
    /// never the health** — `flags & 1`, and for a unit `is_on_map`
    /// ([`Self::struck`]). A city building at its ceiling (health 0) that
    /// still stands is in range: run809's `0/2000` on 3647, which
    /// `think`'s `near_o` arm hands the raiding `1/113` (`docs/AI.md`
    /// §205). SEAM: [`Self::is_in_range_at_margin`] still asks
    /// [`Self::active`] for every other caller — made `struck` there, the
    /// coverage pair parts at 3142 (`1/59`), so some reader leans on the
    /// health term (parked 2051).
    pub(crate) fn is_in_range_struck(&self, attacker: Obj, target: Obj) -> bool {
        self.in_range_from(attacker, self.pos_of(attacker), target, false, false)
    }

    /// `006486b0` whole, through [`in_range_steps`]: the slot test is
    /// [`Self::active`]'s when `health` (a building with no health left is
    /// out), [`Self::struck`]'s when not — split into the slot's own
    /// condition and, for a unit, `is_on_map`.
    fn in_range_from(
        &self,
        attacker: Obj,
        at: Pos,
        target: Obj,
        margin: bool,
        health: bool,
    ) -> bool {
        let slot = match target {
            Obj::Unit(u) => self.units.get(u).is_some_and(|u| u.alive()),
            Obj::Building(b) => self
                .buildings
                .get(b)
                .is_some_and(|b| b.alive && b.combat.is_some() && (!health || b.health > 0)),
        };
        let ap = self.profile(attacker);
        in_range_steps(
            slot,
            || matches!(target, Obj::Unit(_)),
            || match target {
                Obj::Unit(u) => self.units[u].on_map,
                Obj::Building(_) => true,
            },
            at,
            // **The world-cell test** (`6486b0`..`64875b`, item 1200): the
            // tile under the point asked from — `div_3_table[x >> 6]`, the
            // tile, of the eight-argument overload's own `x/y` — answers no
            // when its surface is forest (`TData.mask & 0x30 == 0x30`). It
            // asks no domain, so a plane over a wood holds its fire:
            // chapter forty-one's Biplane on 839, at `0x7138`
            // (`docs/GOLDEN.md` §50).
            |at| {
                self.world.tile_mask(at.tile()) & crate::world::tile::SURFACE
                    == crate::world::tile::SURFACE_FOREST
            },
            |centre| self.attack_dist_from(attacker, centre, target),
            None,
            ap.max_range != 0,
            || self.reaches_like_a_hoplite(attacker),
            || ap.min_range,
            |of_target| {
                let o = if of_target { target } else { attacker };
                (matches!(o, Obj::Unit(_)), self.profile(o).big_radius)
            },
            || self.max_range_of(attacker),
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
                .is_some_and(|ty| self.unit_types[ty].cols.flag(0x10_2000))
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
                        self.unit_types[ty].cols.flag(crate::ai_load::uflags::TANK)
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
            self.tech
                .get(who as usize)
                .is_some_and(|t| self.tech_tree.has_tribe_bonus(&self.setup, t, n))
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
        let cols = self.unit_types[t].cols;
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
        let cols = self.unit_types[t].cols;
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
        if self.holds_bonus(who, self.tech_tree.roles.territory_reveal_preq)
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
    fn fight(&mut self, i: usize, target: Obj, frame: i64) -> bool {
        self.fight_as(i, target, frame, false)
    }

    /// [`Sim::fight`] with `fight`'s fifth argument, `cavarch`: the fire
    /// on the move ([`crate::cavarch`]) takes the angle and the strike,
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
            let snap = crate::collide::ucell_centre(crate::collide::ucell(self.units[i].pos));
            self.set_new_location(i, snap, true);
            if p.is(role::IMMORTALS) && reach < 0xc0 {
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
        combat::recharge(p.recharge, out, p.is(role::BOMBARD)) as u8
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
    fn set_attack_on(&mut self, i: usize, target: Option<Obj>) -> bool {
        let n = self.units[i].guys.len().min(anim::SQUAD_SIZE);
        set_attack_steps(
            self,
            n,
            target,
            |s, g, t| s.units[i].guys[g].aim = t,
            |s| {
                s.units[i].ty.is_some_and(|ty| {
                    s.art
                        .pivots
                        .get(&s.unit_types[ty].type_index)
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
            |node| crate::pivot::offset(piece, node, facing),
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
        let live: Vec<u32> = self.projectiles.iter().map(|q| q.slot).collect();
        let slot = lowest_free_slot(&live);
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
    fn aim_z(&mut self, target: Obj, rolling: bool) -> i32 {
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
    fn fire_ammo(
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
    fn lead_landing(&self, target: Option<Obj>, landing: Pos, total_time: i32) -> Pos {
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
    fn building_shot(&self, b: usize) -> i32 {
        let bl = &self.buildings[b];
        if let Some(n) = bl.ty.and_then(|t| self.build_types[t].shot) {
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
            Some(_) => combat::scatter(&self.tuning, acc, true, false, strafes),
            None => combat::scatter(&self.tuning, acc, land_unit, p.has(mask::MISSILE), strafes),
        };
        // A building target shot by a non-siege, **non-tank** unit: aim at the
        // near face. The gate is the shooter type's vslots `+0x10c`
        // (`is_siege`) and `+0x110` (`UnitTypeData::is_tank`, `unit_flags &
        // 0x80000`), both zero (`Ammo::init@0067bbf0`, `0067c1c0`): a Tank
        // aims at the building's centre (`docs/GOLDEN.md` §135).
        let tank = match shooter {
            Obj::Unit(u) => self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.flag(uflags::TANK)),
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
            if self.air_line_is(u, crate::airbase::CRUISEMISSILE) {
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

    // ------------------------------------------------------------------
    // Delivery
    // ------------------------------------------------------------------

    /// **The tail of `Object::do_damage@0064c1d8`: a city at its ceiling**
    /// (`docs/AI.md` §142). After `Build::check_capture` has answered no, a
    /// hit by another player's object on an active city building whose
    /// `damage` has reached `hits(0)` (1) has a computer leader's **siege**
    /// attacker, itself carrying `unit_masks & 0x40000` and sitting in an
    /// army, drag that army onto the city (`Army::charge`, an `ATTACK_TO`
    /// at the front of the queue), and (2) stamps the city's
    /// `reduce_stamp` — always on the hit that brought it to the ceiling
    /// (`fresh`, `local_70`), and once in 300 frames after. The
    /// notice, the sound and the text bubble that follow are the
    /// interface's and are not carried.
    fn city_reduced(&mut self, attacker: Obj, b: usize, fresh: bool) {
        let who = self.owner_of(attacker);
        let bd = &self.buildings[b];
        if !self.building_is_city(b) || who == bd.owner || !bd.active || bd.damage < bd.hits_now() {
            return;
        }
        let Some(c) = bd.city else {
            return;
        };
        if let Obj::Unit(u) = attacker
            && self.ai_driven(who)
            && self.is_siege_unit(u)
            && self.unit_ai_bit(who)
            && let Some(slot) = self.army_of(u)
        {
            self.army_charge(who, slot, Obj::Building(b));
        }
        if !fresh && self.frame - self.cities[c].reduce_stamp < 300 {
            return;
        }
        self.cities[c].reduce_stamp = self.frame;
    }

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
        self.do_damage_round(
            attacker, target, angle, ammo, false, count, splash, quiet, frame,
        )
    }

    /// [`Self::do_damage`] for a round's blow, which also says whether the
    /// round's ammo piece burns ([`combat::Projectile::flame`]): `ammo` is
    /// `Object::do_damage`'s `param_5 >= 0`, and `flame` its
    /// `verify_ammo_flags(ammo[param_5].gpiece, 0x10)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn do_damage_round(
        &mut self,
        attacker: Obj,
        target: Obj,
        angle: Angle,
        ammo: bool,
        flame: bool,
        count: i32,
        splash: bool,
        quiet: bool,
        frame: i64,
    ) -> Option<Taken> {
        if count < 1 || !self.struck(target) {
            return None;
        }
        // **A decoy's blow is no blow** (`Object::do_damage@0064a480`'s head,
        // item 1522): after the `count < 1` return, an attacker whose
        // vtable `+0x18` answers 1 (a unit) and whose `unit_masks & 1` is
        // set returns at once — before `get_damage`, before the first-wound
        // draw, before any bookkeeping. A General's decoys fire (the launch
        // draws are spent and the round flies its whole arc) and every round
        // lands on nothing: run666's `1/96`, `1/95` and `1/106`, three
        // Bombard decoys (`mana_burn` counting, `unit_masks` 0x5100d, 0x5100d,
        // 0x5100f) struck `0/2000` for nothing where `1/139` and `1/84`
        // (0x5100e) struck it for 135.
        if let Obj::Unit(a) = attacker
            && self.units[a].decoy
        {
            return None;
        }
        // **A hit on another player's object marks both leaders' treaties**
        // (`Object::do_damage@0064a480:327`–`:332`, after the scaling and
        // before `take_damage`): `treaties[victim] |= 2` on the attacker's
        // leader and `treaties[attacker] |= 2` on the victim's — bit 1, the
        // "has struck" bit beside `meet`'s bit 0. Unconditional on what the
        // hit does, so a nuke's ring marks the struck leader's row.
        let (hit_by, hit_who) = (self.owner_of(attacker), self.owner_of(target));
        if hit_by != hit_who {
            self.treaty_on(hit_by, hit_who, 2);
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
            &self.mods_of(owner),
        );
        // Step 2: the target's captain reacts, and the overkill record.
        if let Obj::Unit(t) = target
            && !quiet
        {
            self.target_opportunity(t, attacker, frame);
            // `0064a5dd`: the **struck** unit's `unit_masks & 0x10` — which
            // [`Sim::mark_contact`] raises on the responder, the captain the
            // walk reached — doubles the blow.
            if self.units[t].contact {
                dmg = dmg.wrapping_mul(2);
            }
            let u = &mut self.units[t];
            if u.combat.damage_frame == 0
                || frame - u.combat.damage_frame >= i64::from(self.tuning.overkill_frames)
            {
                u.combat.damage_frame = frame;
                u.combat.damage_o = at.captain;
                u.combat.damage_who = i32::try_from(owner).unwrap_or(-1);
            }
        }
        // `do_damage@0064a480:194`–`:198`: a live, non-missile attacker's
        // blow scrambles a base's fighter (item 3990).
        if let Obj::Building(b) = target
            && !ap.has(combat::mask::MISSILE)
            && self.obj_alive(attacker)
        {
            let at = self.pos_of(attacker);
            self.attempt_launch(b, at);
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
        // target — the second test runs after the first, so a burning round
        // on a building is 1. The `0x10` piece is `effects_graphics.xml`'s
        // `flame="1"` (item 3580: run1202b's Fire Lances `1/31` kills the
        // Citizen `0/10` on 3030 with `Unit::close`'s three draws).
        let dtype = if tt.build_proper {
            1
        } else if ammo && flame {
            4
        } else {
            combat::death_type(ap.obj_masks, !matches!(attacker, Obj::Unit(_)))
        };
        // `local_70` (`do_damage@0064a480`, `0064a8ac`): the struck city
        // building was still below its ceiling when the hit began.
        let fresh = matches!(target, Obj::Building(b)
            if self.building_is_city(b)
                && self.buildings[b].active
                && self.buildings[b].hits_now() > self.buildings[b].damage);
        // The city alarm, before the take (`0064a9f8`..`0064b2c4`).
        let alarm = self.city_alarm(attacker, target, ap.siege, dealt.whole, frame);
        // Step 7: take.
        let taken = self.take_damage_typed(target, dealt, attacker, frame, dtype, splash);
        let killed = matches!(taken, Taken::Died { .. });
        // **`Armies::emergency` is the CITY alarm's, and a unit never
        // reaches it** (`docs/ARMY.md` §15.8, item 399). The call at
        // `do_damage@0064a480:952` is
        // `if (local_30 != 0 && (leaders[param_2].leader_flags & 4) == 0)`,
        // after `take_damage`'s non-zero return has left the function (a
        // kill never reaches it), and `local_30` is [`Self::city_alarm`]'s
        // answer: a heavy hit on a city building, once in 300 frames
        // (item 2235, `docs/GOLDEN.md` §81).
        if alarm && !killed {
            let tw = self.owner_of(target);
            if (tw as usize) < self.armies.len() {
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
            } else {
                let captured = if let Obj::Unit(u) = attacker
                    && who != bowner
                    && self.capture_eligible(b)
                {
                    self.check_capture(b, u)
                } else {
                    false
                };
                if !captured {
                    self.city_reduced(attacker, b, fresh);
                }
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
            self.spill_onto_builders(attacker, b, angle, (ammo, flame), count, frame);
        }
        Some(taken)
    }

    /// **The city alarm** — `Object::do_damage@0064a480`'s building branch
    /// (`0064a9f8`..`0064b2c4`, the decompile's lines 428–735), run before
    /// the take, for a hit across owners on an active building that belongs
    /// to a city (`BuildData +0x72 >= 0`). Item 2235, `docs/GOLDEN.md` §81.
    ///
    /// ```text
    /// heavy = the attacker's type is_siege (UnitTypeData: unit_flags & 0x20000)
    ///      or damage + dealt >= hits / 10   (a fort or tower — type vslot +0x104,
    ///                                         ObjectTypeData::is_defense — or a city)
    ///                         hits / 4     (anything else)
    /// city c not the capital (city_flags & 0x10 clear):
    ///   heavy: alarm when frame − c.attack_stamp > 300;
    ///          c.attack_stamp = c.raid_stamp = frame
    ///   light: c.raid_stamp = frame
    /// the capital:
    ///   heavy: alarm when frame − victim.capital_stamp[attacker] > 300, and the
    ///          stamp takes the frame then; the city's own stamps are not written
    ///   light: c.raid_stamp = frame
    /// ```
    ///
    /// Answers the alarm, `local_30`, which gates `Armies::emergency`.
    /// The victim leader's `attack_stamp`/`raid_stamp`/`ally_stamp` rows,
    /// the notices and the sounds are not carried: the dump prints the
    /// three arrays only at `LEADERS=9`, and nothing here reads them.
    pub(crate) fn city_alarm(
        &mut self,
        attacker: Obj,
        target: Obj,
        siege: bool,
        whole: i32,
        frame: i64,
    ) -> bool {
        let (att, vic) = (self.owner_of(attacker), self.owner_of(target));
        if att == vic {
            return false;
        }
        let b = match target {
            Obj::Building(b) if self.buildings[b].active => b,
            // **The worker arm** (`0064a9xx`, the decompile's lines 400–
            // 424): a struck worker (`ObjectData::is_worker`) whose action
            // (`UnitData::get_action@00608450`) answers `get_gather_order`
            // (order vslot `+0xe4`, by the PDB's method record) with an
            // active building of the victim's in a city raids that city —
            // always the light arm, `local_34` still 0. run437's citizens
            // gathering at the Farm `1/2001`, struck on 671, 672 and 690.
            Obj::Unit(t) if self.is_worker(t) => {
                let gather = self
                    .action_of(t)
                    .and_then(|i| match self.units[t].orders[i].body {
                        crate::orders::Body::Gather(g) => Some(g.building),
                        _ => None,
                    });
                if let Some(g) = gather
                    && let Some(bd) = self.buildings.get(g)
                    && bd.active
                    && bd.owner == vic
                    && let Some(c) = bd.city
                {
                    self.cities[c].raid_stamp = frame;
                }
                return false;
            }
            _ => return false,
        };
        let Some(c) = self.buildings[b].city else {
            return false;
        };
        let heavy = (matches!(attacker, Obj::Unit(_)) && siege) || {
            let bd = &self.buildings[b];
            let defense = bd.ty.is_some_and(|t| {
                crate::build::is_fort(&self.build_types, t)
                    || crate::build::is_tower(&self.build_types, t)
            });
            let ceiling = if defense || self.building_is_city(b) {
                bd.hits_now() / 10
            } else {
                bd.hits_now() / 4
            };
            bd.damage + whole >= ceiling
        };
        if !heavy {
            self.cities[c].raid_stamp = frame;
            return false;
        }
        if self.cities[c].capital {
            let stamp = &mut self.ai[vic as usize].capital_stamp[att as usize];
            if frame - *stamp > 300 {
                *stamp = frame;
                return true;
            }
            return false;
        }
        let city = &mut self.cities[c];
        let alarm = frame - city.attack_stamp > 300;
        city.attack_stamp = frame;
        city.raid_stamp = frame;
        alarm
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
        (ammo, flame): (bool, bool),
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
                self.do_damage_round(
                    attacker,
                    Obj::Unit(u),
                    angle,
                    ammo,
                    flame,
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
        self.take_damage_typed(target, hit, by, frame, 0, false)
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
        splash: bool,
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
                let alone = self.takes_remainder_share(i);
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
                // `take_damage@00652020:619`: the whole points past the
                // figure's share, read before `die` — what a decoy's death
                // hands on to its captain below.
                let overflow = u.max_health - u.health - share;
                let decoy = u.decoy;
                // `take_damage@00652020:176`-`:214`: a Citizen struck rings
                // its city's bell before the kill arm (item 3984).
                self.peasant_alarm(i);
                if matches!(taken, Taken::Alive { .. }) {
                    self.struck_alive(i);
                }
                if matches!(taken, Taken::Died { .. }) {
                    let u = &mut self.units[i];
                    u.health = u.health.min(0);
                    // `take_damage@00652020:607`, before `die` (vslot
                    // `+0x158`): the kill's plunder (`docs/AI.md` §259).
                    if !splash && let Obj::Unit(k) = _by {
                        self.unit_plunder(i, k);
                    }
                    self.inca_refund(i, _by);
                    // `Object::die` is `close()` — the draw arm below —
                    // and **then** the slot hold, which is not gated on
                    // the draw arm's `dtype` at all (§42.3).
                    self.close_count(i);
                    self.relink_squad(i);
                    self.close_unit(i, dtype, _frame);
                    self.kill_contents(i);
                    self.kremlin_spy_closed(i);
                    self.close_dead_orders(i);
                    self.hold_dead_slot(i);
                    let keep = self.keeps_disc(i, dtype);
                    self.close_supply_in(i, keep);
                    self.forget(Obj::Unit(i));
                    self.clear_decoy_at_close(i);
                    // **A decoy's overflow strikes its captain**
                    // (`take_damage@00652020:619`–`631`, item 3393): after
                    // `die`, a figure that was a decoy (`unit_masks & 1`,
                    // read before `close` cleared it) with whole points
                    // past its share hands them to `get_captain` — the
                    // chain as `close` re-linked it — unless that is
                    // itself: `take_damage(overflow, 0, dtype, …, captain,
                    // who, 0)`, the captain its own attacker. A decoy's
                    // blow is `get_damage`'s step 17, so the squad goes
                    // down figure by figure on the one round. run1015b's
                    // packet: the tail `1/92` takes 13370, `1/90` the
                    // 13345 past its 39, `1/91` the 13327 past 1/90's.
                    if decoy && overflow > 0 {
                        let (c, who) = (self.units[i].combat.captain, self.units[i].owner);
                        let captain = (c != i32::from(self.units[i].index))
                            .then(|| {
                                self.units.iter().position(|u| {
                                    u.owner == who && i32::from(u.index) == c && u.alive()
                                })
                            })
                            .flatten();
                        if let Some(k) = captain {
                            let hit = combat::Sixteenths {
                                whole: overflow,
                                frac: 0,
                            };
                            self.take_damage_typed(
                                Obj::Unit(k),
                                hit,
                                Obj::Unit(k),
                                _frame,
                                dtype,
                                false,
                            );
                        }
                    }
                }
                taken
            }
            Obj::Building(b) => self.damage_building(b, hit, Some(_by), _frame, false),
        }
    }

    /// **A struck unit that survives raises `0x10` on its captain**
    /// (`Object::take_damage@00652020:345`–`350`, item 3983, `docs/AI.md`
    /// §415): after the hit is accumulated, a unit whose whole points taken
    /// (`+0x24`) are still under its figure's share sets
    /// `SubObjectData::flags |= 0x10` on `get_captain` (vslot `+0xe4`) —
    /// the bit `Unit::think`'s idle gate reads, carried as `cant_reach`.
    /// Attrition comes through here too (`suffer_attrition`'s slot `0x16c`
    /// is this function), so [`crate::suffer_view::SimSuffer`] calls it as
    /// well. A building's own arm (the same write on its captain, itself)
    /// is not carried: no building flag is compared.
    pub(crate) fn struck_alive(&mut self, i: usize) {
        let c = self.captain_of(i);
        self.units[c].cant_reach = true;
    }

    /// **The Despot's unit plunder** (`docs/AI.md` §259): `Object::take_damage
    /// @00652020`'s kill arm (`:520`–`:609`) and the EE arm of
    /// `Unit::plunder@00605b00` it calls, for a victim `v` killed by the unit
    /// `k`.
    ///
    /// The caller's gate: `v` is its squad's captain, the last figure of it
    /// (`curr_uber_size == 1`) and no decoy; the killer is a unit, the hit no
    /// splash (`param_9`), and the two leaders not allied. A victim that
    /// `is` a General, a caravan or a supply wagon plunders by its own
    /// constants; any other plunders only when `HeroesData::find_hero
    /// @0073a1b0` finds one of the killer's heroes of type THEDESPOT
    /// (`0x160`), else SPITAMENES (`0x16d`), active, on the map and within
    /// `get_radius × 0xc0` of the victim, and `THEDESPOT_UNIT_PLUNDER` is
    /// non-zero — and then `plunder(v, k, who, 0, 1)`.
    ///
    /// In `plunder` (`get_patch_version >= 9`, `00605f78`..): one
    /// `Random::get(0, 0xffff)` per good of the killer's leader available
    /// (`type_avail(g, 1)`), knowledge skipped, food to oil in order; the good
    /// whose `bucket + roll % 25` is least, first on a tie, takes
    /// `THEDESPOT_UNIT_PLUNDER`. `leaders +0x914` (the scenario's plunder
    /// scale, `-1.0` from `Leader::init` and written only by
    /// `ScenarioFuncSet::set_plunder_scale`) is negative and scales nothing.
    /// run943's Citizen `0/3`, killed on 10852 by `1/58` beside The Despot
    /// `1/46`: four draws (oil not yet available) and who=1's metal 44 → 59.
    ///
    /// **The Aztec slay arm** comes first (`:553`–`:555`, item 3657): a
    /// killer whose leader has tribe bonus 0 (the Aztecs) and whose type's
    /// trainer (`+0x40`) is **by equality** the Barracks (`0x1ab`), the
    /// Stable (`0x1ac`) or the Dock (`0x1b0`) calls `plunder(v, k, who, 1,
    /// 0)` whatever the victim, Despot or none. The pay is `(ages + 1) ×
    /// AZTEC_SLAY_BONUS` up to `AZTEC_SLAY_CAP`, in the good the same four
    /// draws pick; run1308b's Heavy Horse Archer `1/29` (a Stable type) kills
    /// the Citizen `0/13` on 2556 at Medieval (`ages` 2) and who=1's timber
    /// goes 12714 → 12759.
    ///
    /// The body is [`crate::plunder::plunder_steps`] over [`crate::plunder::SimPlunder`]
    /// (the original's whole function, swept against it in item 3356).
    ///
    /// A General, caravan or supply-wagon victim (`uflags2::GENERAL`,
    /// `CARAVAN`, `SUPPLY_OR_HERO`) plunders by its own constant, `p3` and
    /// `p4` both 0 (item 3788). The Inca refund is [`Sim::inca_refund`].
    ///
    /// SEAM: the victim redirect of a type with `+0x2b4 & 0x10` is not
    /// carried.
    pub(crate) fn unit_plunder(&mut self, v: usize, k: usize) {
        use crate::ai_load::uflags2;
        const DESPOT: i32 = 0x160;
        const SPITAMENES: i32 = 0x16d;
        let amount = self.tuning.thedespot_unit_plunder;
        let victim = &self.units[v];
        if !self.is_captain(v) || victim.decoy || self.curr_uber_size(v) != 1 {
            return;
        }
        let (vw, at) = (victim.owner, victim.pos);
        let who = self.units[k].owner;
        if who == vw || self.is_ally(vw, who) {
            return;
        }
        let slay = self.nation[who as usize].aztecs
            && self.units[k]
                .ty
                .and_then(|t| self.trainer_ident(t))
                .is_some_and(|i| {
                    use crate::build::Ident;
                    matches!(i, Ident::Barracks | Ident::Stable | Ident::Dock)
                });
        if slay {
            crate::plunder::plunder_steps(
                &mut crate::plunder::SimPlunder { sim: self, v },
                k as i32,
                i32::from(who),
                1,
                0,
            );
            return;
        }
        let kind = self.units[v]
            .ty
            .map_or(0, |t| self.unit_types[t].cols.unit_flags2);
        // `take_damage@00652020`'s `is(GENERAL)`/`is(CARA)`/`is(SUPPLYWAGON)`
        // arm (`LAB_0065332c`): `plunder(v, k, who, 0, 0)` — no Despot search,
        // the base constant in wealth (item 3788; run1461b's who=2 takes
        // `GENERAL_PLUNDER` 100 for who=1's Despot `1/28` on 1463).
        if kind & (uflags2::GENERAL | uflags2::CARAVAN | uflags2::SUPPLY_OR_HERO) != 0 {
            crate::plunder::plunder_steps(
                &mut crate::plunder::SimPlunder { sim: self, v },
                k as i32,
                i32::from(who),
                0,
                0,
            );
            return;
        }
        if amount == 0
            || !(self.hero_near(who, DESPOT, at, 0) || self.hero_near(who, SPITAMENES, at, 0))
        {
            return;
        }
        // `Unit::plunder(victim, k, who, 0, 1)`: the Extended Edition's arm,
        // over the crate's own buckets and draws.
        crate::plunder::plunder_steps(
            &mut crate::plunder::SimPlunder { sim: self, v },
            k as i32,
            i32::from(who),
            0,
            1,
        );
    }

    /// **The Inca refund** (item 3788, `docs/GOLDEN.md` §149.12):
    /// `Object::take_damage@00652020`'s `LAB_00653354` arm, reached by the
    /// kill arm's own gate — the victim `v` its squad's captain, the last
    /// figure (`curr_uber_size == 1`), no decoy, a killer — and run after
    /// the plunder block whether that block ran or not, so a building's
    /// kill and a splash kill refund too. When the victim's leader is not
    /// allied with the killer's (`is_ally` answers itself, so an own kill
    /// pays nothing), `INCA_REFUND` is non-zero and the victim's leader
    /// holds tribe bonus 2, it calls `Unit::plunder(v, v.index, v.who, 0,
    /// 0)`: `INCA_REFUND`% of the unit's price in each available good,
    /// added to the bucket uncapped. run1461b's who=2 (the Inca) takes
    /// `+16` food and `+11` timber on logger frame 860 and `+25` timber
    /// and `+25` metal on 864, the Bombard `2/23`'s death.
    pub(crate) fn inca_refund(&mut self, v: usize, by: Obj) {
        let victim = &self.units[v];
        if !self.is_captain(v) || victim.decoy || self.curr_uber_size(v) != 1 {
            return;
        }
        let vw = victim.owner;
        if self.tuning.inca_refund == 0
            || self.is_ally(vw, self.owner_of(by))
            || !self.holdings[vw as usize].inca
        {
            return;
        }
        crate::plunder::plunder_steps(
            &mut crate::plunder::SimPlunder { sim: self, v },
            v as i32,
            i32::from(vw),
            0,
            0,
        );
    }

    /// The leader's per-type count at `+0x56fe + ti × 2` — the muster's
    /// count of type `ti` — non-zero: `has_general`'s early `return -1`.
    pub(crate) fn hero_counted(&self, who: Player, ti: i32) -> bool {
        self.unit_types
            .iter()
            .position(|t| t.type_index == ti)
            .and_then(|r| self.muster.get(who as usize).and_then(|m| m.by_type.get(r)))
            .is_some_and(|&n| n != 0)
    }

    /// **`HeroesData::find_hero@0073a1b0`** behind a leader's per-type count
    /// (`+0x56fe + ti × 2`, the muster here): one of `who`'s heroes of type
    /// `ti`, active and on the map, with `vector_dist(hero, at) − slack ≤
    /// get_radius × 0xc0`. `slack` is the caller's sixth argument: 0 for a
    /// unit's plunder, a building's footprint for its own
    /// ([`Sim::plunder_kill`], `docs/AI.md` §261).
    pub(crate) fn hero_near(&self, who: Player, ti: i32, at: Pos, slack: i32) -> bool {
        self.hero_counted(who, ti)
            && (0..self.units.len()).any(|h| {
                let x = &self.units[h];
                x.owner == who
                    && x.type_index == ti
                    && x.alive()
                    && x.on_map
                    && self.is_hero_unit(h)
                    && crate::world::vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs())
                        - slack
                        <= self.hero_radius(h) * 0xc0
            })
    }

    /// `SpellType::cast_sabotage@006720e0`'s unit arm for the Sniper
    /// (`0x281`, item 2971, `docs/GOLDEN.md` §115): the target figure
    /// `die`s with `dtype` 2 — `Unit::die@0060eda0` through vslot `+0x158`,
    /// the combat death's tail without a wound.
    ///
    /// SEAM: the caster's leader counters (`+0xa58`, `+0xa5a`, `+0x80c`)
    /// and the score `+0x44 += cost / 5` for a squad captain, and the
    /// message and sound.
    pub(crate) fn cast_sniper_unit(&mut self, v: usize, dtype: i32) {
        self.die_with(v, dtype);
    }

    /// The `die o,who` cheat on a unit (`run_cmd`'s `die` arm,
    /// `007dde60`–`007ddf1d`): vslot `+0x158` with `(0, −1, 0)`,
    /// [`Sim::die_with`] at `dtype` 0 — no death draw (item 3989,
    /// `docs/GOLDEN.md` §182).
    pub fn cheat_die_unit(&mut self, v: usize) {
        self.die_with(v, 0);
    }

    /// `Object::die(o, dtype, −1, 0)` on a unit, through `Unit::die`
    /// (vslot `+0x158`): the combat death's tail without a wound — the
    /// Sniper's kill ([`Sim::cast_sniper_unit`]) and a disband
    /// ([`Sim::disband_unit`], item 3914).
    pub(crate) fn die_with(&mut self, v: usize, dtype: i32) {
        let frame = self.frame;
        self.units[v].health = 0;
        self.close_count(v);
        self.relink_squad(v);
        self.close_unit(v, dtype, frame);
        self.kill_contents(v);
        self.kremlin_spy_closed(v);
        self.close_dead_orders(v);
        self.hold_dead_slot(v);
        let keep = self.keeps_disc(v, dtype);
        self.close_supply_in(v, keep);
        self.forget(Obj::Unit(v));
        self.clear_decoy_at_close(v);
    }

    /// A unit's death by attrition: `Unit::suffer_attrition@005e1a10` calls
    /// the unit's `take_damage` (vslot `+0x16c`) with `dtype` **1**, not 0 —
    /// `(whole, sixteenths, 1, -1, 1, 0x55555555, ...)` — so `Object::die`
    /// runs the whole combat tail: the death animation's draw
    /// (`Unit::close+0xcb6`), the sound, and a Tank's disc left standing
    /// (`docs/GOLDEN.md` §135, run936: the original draws on 1890).
    pub(crate) fn die_by_attrition(&mut self, i: usize, frame: i64) {
        const ATTRITION_DTYPE: i32 = 1;
        self.close_count(i);
        self.relink_squad(i);
        self.close_unit(i, ATTRITION_DTYPE, frame);
        self.kill_contents(i);
        self.kremlin_spy_closed(i);
        self.close_dead_orders(i);
        self.hold_dead_slot(i);
        let keep = self.keeps_disc(i, ATTRITION_DTYPE);
        self.close_supply_in(i, keep);
        self.forget(Obj::Unit(i));
        self.clear_decoy_at_close(i);
    }

    /// `Object::close@00647160`'s `blocks_while_dead` arm: any death with a
    /// non-zero `param_1` (`dtype`) of a type carrying `x` leaves its
    /// occupancy disc standing until the death object's cull — a combat
    /// death or a Sabotage's boom alike (item 3228, `docs/GOLDEN.md` §130);
    /// attrition (`dtype` 1, `Unit::suffer_attrition`) keeps it like any other (item 3252, `docs/GOLDEN.md` §135).
    fn keeps_disc(&self, i: usize, dtype: i32) -> bool {
        dtype != 0
            && self.units[i].on_map
            && self.units[i].ty.is_some_and(|t| {
                self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::BLOCKS_WHILE_DEAD != 0
            })
    }

    /// `SpellType::cast_sabotage@006720e0`'s building arm (item 2994,
    /// `docs/GOLDEN.md` §116): half the building's `hits_left`, floored at
    /// `COMMANDO_MIN_DAMAGE`, taken as `take_damage(dmg, 0, dtype 3, -1,
    /// attrition 1, ..)` — no attacker, so no latch and no first-wound
    /// draws. (The `this->field_4 == 0x4d / 0x4b` tests name a Type index
    /// no craft carries, so the Commando's floor stands.)
    ///
    /// SEAM: the caster's leader counters and score (`+0xa58`, `+0xa5a`,
    /// `+0x80c`; `+0x818` and `+0x44 += cost / 5` on the razed building),
    /// the message and the sound; the dead-target refund arm.
    pub(crate) fn cast_sabotage_building(&mut self, b: usize) {
        let frame = self.frame;
        let dmg = (self.buildings[b].health / 2).max(self.tuning.commando_min_damage);
        self.damage_building(
            b,
            Sixteenths {
                whole: dmg,
                frac: 0,
            },
            None,
            frame,
            true,
        );
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
        // birds over the building, from its tile (`div_3_table[v >> 6]`,
        // `6521ba`–`6521e3`) and with no player's stamp to wait on
        // (`crate::flock`, item 1293).
        let n = self.rng.roll() % 2 + 3;
        let at = self.buildings[b].pos.tile();
        self.add_flock(at, None, n);
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
        if site && matches!(by, Some(Obj::Building(_))) {
            hit.whole *= 4;
            hit.frac *= 4;
        }
        let bd = &self.buildings[b];
        let share = bd.health;
        let (taken, _, frac) = combat::take(0, bd.damage_frac, share, hit);
        let lost = match taken {
            Taken::Alive { lost } | Taken::Died { lost, .. } => lost,
        };
        if site {
            // **The progress a site loses is the points the blow took,
            // carry and all** (item 1350): the listing rewrites `param_1`,
            // the blow's whole hits, as `whole + (frac + hit.frac) / 16`
            // at `0065230b`, and the `imul $0x32` at `0065239b` reads it
            // back. run514's Keep site, a blow of 4 whole and 5 sixteenths:
            // 200 off its `job_counter` on 706, and 250 on 738, where the
            // sixteenths carried. **An aircraft's blow costs nothing**
            // (`0065234e`–`0065237c`): an attacker in use whose type's
            // `domain` (`+0x218`) is 2 sets the skip, and so does
            // attrition (`param_5`, `00652341`).
            let air = by.is_some_and(|o| matches!(self.profile(o).domain, Domain::Air));
            if !attrition && !air {
                let bd = &mut self.buildings[b];
                bd.job_counter = (bd.job_counter - combat_progress_lost(lost)).max(0);
            }
        }
        let city = self.building_is_city(b);
        let bd = &mut self.buildings[b];
        bd.damage_frac = frac;
        bd.damage += lost;
        let hits = bd.hits_now();
        if city && bd.active {
            // The clamp: a city never dies from damage. **It keeps the
            // fraction** (`Object::take_damage@00652020`, the `flags & 0x20`
            // arm: `damage = hits`, `flags |= 0x10`, and `+0x3b` untouched;
            // `docs/AI.md` §230): run866's capital at its ceiling steps
            // `damage_frac` 10, 15, 4, 9 … from 5161.
            if bd.damage >= hits {
                bd.damage = hits;
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

    /// **`Unit::close@0060ee50`'s count, at `0060f3db`** — the general
    /// death path's `track_unit_type(type, −1)`, with `control` and
    /// `active` beside it (`docs/AI.md` §232; parked 1092, 1328).
    ///
    /// The listing takes the count off only for the squad's head
    /// (`+0x8e < 0`) when no live figure follows it: a head whose next
    /// figure (`+0x90`) is active hands the squad down at `0060f679`
    /// instead, and the squad is still one unit. Then the gate
    /// [`Sim::counts_in_muster`] shares with `set_type`'s `+1` — a type
    /// with population, and `unit_masks & 1` (the decoy bit) clear. Called
    /// before [`Self::relink_squad`], which is `close`'s relink after it.
    /// run867's who=1 kept three shot-down Jet Fighters in `num_units`
    /// (7 against 4), and its next Fighter's ramp ran twelve frames long.
    ///
    /// **A government hero is counted off with no population** (item 3794,
    /// `docs/GOLDEN.md` §151.1): the gate's `control_cost == 0` arm
    /// (`0060f350`..`0060f3d9`) goes on to `0060f3db` when the unit
    /// `is(0x134)` or `UnitData::is_gov_hero@0046ce70` answers — the type's
    /// `+0x2b4 & 0x4000000`, `OBJ_MASK`'s hero digit, [`Sim::type_is_hero`].
    /// The Despot is `POP 0`: the first melee's who=1 kept its `1/28` in
    /// `num_units[302]` from its death on 1463 to the close.
    ///
    /// SEAM: the `is(0x134)` half of that arm is not read here, and answers
    /// no.
    pub(crate) fn close_count(&mut self, i: usize) {
        self.gov_hero_closed(i);
        // `Unit::close@0060ee50:157-170`, ahead of the count and ungated by
        // the squad: an American Barracks unit that is not a scout raises
        // its leader's `0x2000000` (item 3384, `docs/AI.md` §292).
        if let Some(ty) = self.units[i].ty {
            let who = self.units[i].owner;
            if self.american_barracks_unit(who, i, ty) {
                self.economy_changed(who);
            }
        }
        let u = &self.units[i];
        if u.o_up.is_some() || u.o_down.is_some_and(|d| self.units[d].alive()) {
            return;
        }
        // `Unit::close@0060ee50:287`-`:289`: the hotkey arm sits on the
        // squad gate alone, below the count's decoy and population tests,
        // and reaches `Build::replace_gather` (item 3989).
        if u.owner < 8 {
            self.replace_gather(i);
        }
        let u = &self.units[i];
        if u.decoy {
            return;
        }
        let Some(ty) = u.ty else { return };
        if self.unit_types[ty].price.pop == 0 && !self.type_is_hero(ty) {
            return;
        }
        // Gaia's animals carry population in some rows and hold no muster
        // here: this crate keeps one per player slot.
        let who = u.owner;
        if (who as usize) < self.muster.len() && (who as usize) < self.ai.len() {
            self.track_unit_type(who, ty, -1);
        }
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
            // The whole-squad arm (`Unit::close@0060ee50:292`-`316`): the last of the
            // squad dies, and every figure still linked below it — the dead
            // tail — has both links cleared, then this one. Only then is a
            // dead number a captain (`o_up < 0`) to `Objects::find_free`'s
            // reuse test; a dead tail of a live squad stays linked and its
            // number is held (`docs/AI.md` §324.2).
            let mut at = down;
            for _ in 0..self.units.len() {
                let Some(n) = at else { break };
                at = self.units[n].o_down;
                self.units[n].o_down = None;
                self.units[n].o_up = None;
            }
            self.units[i].o_down = None;
            self.units[i].o_up = None;
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
        let units = &self.units;
        let marked = crate::squad_walk(
            u,
            units.len(),
            |i| units[i].o_up,
            |i| units[i].o_down.filter(|&d| units[d].alive()),
        );
        for n in marked {
            self.units[n].in_danger = true;
            self.units[n].guy_flag_0x20 = true;
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
    pub(crate) fn close_unit(&mut self, i: usize, dtype: i32, frame: i64) {
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
            // draw is still spent, and its value discarded. A burning
            // round's kill (`flame="1"`, [`combat::Projectile::flame`]).
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
        // **Every figure's own disc is cleared, the crew's too** (item
        // 3787, `docs/GOLDEN.md` §150.2). `Unit::close`'s two loops — the
        // squad's `0 .. guy_mark` and the crew's `squad_size .. guy_num` —
        // call `Objects::kill_guy@00659410` on each figure, and its last
        // arm clears `radius[coll_size]` around **that figure's** cell
        // unless the type's vslot `+0x120` (`blocks_while_dead`) answers.
        // `remove_from_world` below takes guy 0's again; a crew figure
        // standing a cell off its engine punches its own disc out of any
        // neighbour's block, and nothing puts it back but the neighbour's
        // sixty-fourth-frame repaint.
        if !self.units[i].ty.is_some_and(|t| {
            self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::BLOCKS_WHILE_DEAD != 0
        }) {
            for k in 0..self.units[i].guys.len() {
                self.coll_kill_guy(i, k);
            }
        }
        let figures = self.death_figures(i, cur_anim);
        let life = self.death_life(i, cur_anim);
        let u = &self.units[i];
        let (who, o) = (i32::from(u.owner), i32::from(u.index));
        self.deaths.push(combat::Death {
            who,
            o,
            first_frame: frame,
            cur_anim,
            life,
            figures,
        });
        // `Objects::kill_guy`'s `add_death`, one call a figure, in the
        // order `Unit::close`'s loops reach them.
        for k in 0..figures.n {
            self.add_death(combat::DeathSlot {
                who,
                o,
                first_frame: frame,
                figure: k,
            });
        }
    }

    /// Whether the record a pool slot names is still live on `self.frame`,
    /// before this frame's `DeathObj::inc_time`: its [`combat::Death`]
    /// stands and that figure's life has not run out.
    fn death_slot_live(&self, r: &combat::DeathSlot) -> bool {
        let age = self.frame - r.first_frame;
        self.deaths.iter().any(|d| {
            d.who == r.who
                && d.o == r.o
                && d.first_frame == r.first_frame
                && r.figure < d.figures.n
                && age < d.figures.life[usize::from(r.figure)]
        })
    }

    /// `DeathObjData::blocking@008d4a90`: the type of the object holding
    /// the record's number answers vslot `+0x120`, `blocks_while_dead`.
    fn death_slot_blocks(&self, r: &combat::DeathSlot) -> bool {
        self.units
            .iter()
            .rposition(|u| i32::from(u.owner) == r.who && i32::from(u.index) == r.o)
            .and_then(|u| self.units[u].ty)
            .is_some_and(|t| {
                self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::BLOCKS_WHILE_DEAD != 0
            })
    }

    /// **`Objects::add_death@00653b60`'s slot** (item 3971, `docs/GOLDEN.md`
    /// §177): the pool is [`combat::DEATH_POOL`] slots and nothing grows
    /// it. The walk takes the first slot whose record is not valid; on the
    /// way it remembers the oldest record whose type does not block while
    /// dead — `first_frame < best`, `best` starting at `frame + 1`, so the
    /// lowest slot wins a tie — and when no slot is free that record is
    /// overwritten, or slot 0 when every record blocks. An overwritten
    /// record that blocks has its disc cleared first
    /// (`DeathObj::clear_blocking`); this crate leaves that to the cull
    /// below, the same frame, since only the all-blocking fallback can
    /// reach it.
    ///
    /// The evicted record ends where it stands: its figure's life is cut
    /// to its age, so it bumps its number no more
    /// ([`Self::death_objs_inc_time`]), and a [`combat::Death`] left with
    /// no live figure is culled this frame and leaves the dump's list.
    pub(crate) fn add_death(&mut self, rec: combat::DeathSlot) -> usize {
        if self.death_pool.len() < combat::DEATH_POOL {
            self.death_pool.resize(combat::DEATH_POOL, None);
        }
        let mut best = (0usize, self.frame + 1);
        let mut free = None;
        for k in 0..combat::DEATH_POOL {
            let Some(r) = self.death_pool[k].filter(|r| self.death_slot_live(r)) else {
                free = Some(k);
                break;
            };
            if !self.death_slot_blocks(&r) && r.first_frame < best.1 {
                best = (k, r.first_frame);
            }
        }
        let k = free.unwrap_or(best.0);
        if let Some(old) = self.death_pool[k].filter(|r| self.death_slot_live(r)) {
            let age = self.frame - old.first_frame;
            if let Some(d) = self
                .deaths
                .iter_mut()
                .find(|d| d.who == old.who && d.o == old.o && d.first_frame == old.first_frame)
            {
                d.figures.life[usize::from(old.figure)] = age;
                let n = usize::from(d.figures.n).min(combat::Figures::MAX);
                let last = d.figures.life[..n].iter().copied().max().unwrap_or(age);
                d.life = (last != i64::MAX).then_some(last);
            }
        }
        self.death_pool[k] = Some(rec);
        k
    }

    /// **`Object::close@00647160`'s contents arm on a unit** (item 3862,
    /// `docs/GOLDEN.md` §153.6): an object on the map (vslot `+0xbc`) that
    /// holds anything (`num_inside(1) != 0`) ejects it only when it cannot
    /// carry air **and** its vslot `+0x18` answers 0 — and a `Unit`'s
    /// `+0x18` is the folded `return 1` (`0041e0e0`), so a unit never
    /// ejects: it takes `Object::kill_contents@00645550`, which for each
    /// head of `inside_down` calls `Unit::come_out(p, 0)` and
    /// `Unit::die_uber(p, 0)` — `Object::die(p, 0, −1, 0)` on the squad,
    /// no death draw. A transport sunk at sea takes its riders with it:
    /// run1483's Freighter `1/54` and the Elite Special Forces `1/0`
    /// aboard it. A boat off the map (inside an oil platform) takes the
    /// `remove_from_inside` arm instead ([`Sim::boat_dies_into`]).
    pub(crate) fn kill_contents(&mut self, boat: usize) {
        if !self.units[boat].on_map {
            return;
        }
        let riders: Vec<usize> = (0..self.units.len())
            .filter(|&r| self.units[r].inside_unit == Some(boat) && self.units[r].alive())
            .collect();
        for r in riders {
            self.units[r].inside_unit = None;
            self.die_quietly(r);
        }
    }

    /// `DeathObj body_fade_end`, in frames: a land unit whose death piece
    /// has no skeleton lives its animation and this (`DeathObj::inc_time
    /// @008d5240`, `.rdata` `0002:378732`; `docs/GOLDEN.md` §127).
    pub(crate) const BODY_FADE_END: i64 = 135;
    /// `corpse_fade_end`, `.rdata` `0002:378740`: the same, for a piece
    /// with a skeleton (a corpse).
    pub(crate) const CORPSE_FADE_END: i64 = 627;

    /// How many frames the death object `close_unit` makes lives
    /// (`DeathObj::inc_time@008d5240`): the death piece's animation packet
    /// length for `cur_anim`, plus [`Self::CORPSE_FADE_END`] when the piece
    /// has a skeleton (`DeathObj +0x44`, `RData::unit_data +0x20`),
    /// [`Self::BODY_FADE_END`] when it has none, and nothing for a sea
    /// unit (`type +0x218 == 1`). `None` when the install does not name
    /// the piece's slot.
    fn death_life(&self, i: usize, cur_anim: i32) -> Option<i64> {
        // One `DeathObj` per guy, each on its own `gpiece` (ch94's four-man
        // crew prints four, `gpiece` 219, 12891, 25563, 38235): the unit's
        // number is held by the last to end.
        let mut life = None;
        for k in 0..self.units[i].guys.len() {
            life = life.max(Some(self.figure_life(i, k, cur_anim)?));
        }
        life
    }

    /// One guy's record's life, as [`Self::death_life`] reads it.
    fn figure_life(&self, i: usize, k: usize, cur_anim: i32) -> Option<i64> {
        let anim = i8::try_from(cur_anim).ok()?;
        let sea = self.units[i].kind.domain == crate::attrition::Domain::Sea;
        let gpiece = self.units[i].guys[k].gpiece;
        if gpiece < 0 || !self.art.piece_lengths.contains_key(&gpiece) {
            return None;
        }
        let frames = i64::from(self.slot_length(i, gpiece, anim)?);
        let fade = if sea {
            0
        } else if self.art.piece_skeletons.contains(&gpiece) {
            Self::CORPSE_FADE_END
        } else {
            Self::BODY_FADE_END
        };
        Some(frames + fade)
    }

    /// **Every guy's record** (item 3784, `docs/GOLDEN.md` §149.10): the
    /// lives [`Self::hold_frames_tick`] counts, one bump a record a frame.
    /// A record whose piece the install does not describe lives for good.
    fn death_figures(&self, i: usize, cur_anim: i32) -> combat::Figures {
        let mut f = combat::Figures::one(None);
        let n = self.units[i].guys.len().clamp(1, combat::Figures::MAX);
        f.n = n as u8;
        for k in 0..n {
            f.life[k] = if k < self.units[i].guys.len() {
                self.figure_life(i, k, cur_anim).unwrap_or(i64::MAX)
            } else {
                i64::MAX
            };
        }
        f
    }

    /// **`Object::die@00647080`'s tail** — `hold_frames = max(hold_frames,
    /// 1, for every live ammo this object fired: nuke_effect[0x108] + 1 +
    /// total_time − cur_time)` (§11, §42.3). The slot is held until the
    /// dead archer's last arrow has landed.
    ///
    /// The ammo is matched on its **shooter**: `+0x3c`/`+0x40`, which
    /// `Ammo::init` fills from the firing object, and the same pair
    /// `Ammo::inc_time` reads for its own per-frame bump. `nuke_effect`'s
    /// term is [`Self::NUKE_SPREAD`], the 30 `Nuke::init` writes — on
    /// every ammo, not a nuke's alone (item 1594, `docs/AI.md` §165): the
    /// listing adds `[0xc0a888]` at `64710b` with no test of the round.
    ///
    /// **And the `hold_frames` it maxes against is already
    /// [`Self::CLOSE_HOLD`]** (§59.3): `die` calls `close` through vslot
    /// `+0x150` first, and `Unit::close` ends in `Object::close`, whose
    /// last write to an active object is `hold_frames = 0x1e`. So every
    /// death holds its number thirty frames, whatever `dtype` it carried —
    /// the transport that dies putting its passenger ashore
    /// ([`Sim::disembark`]), attrition, the upgrade's squad trim.
    pub(crate) fn hold_dead_slot(&mut self, i: usize) {
        // The type's `+0x1fc` gate is a field this crate does not carry (no
        // shipped type with a round in flight is known to hold it zero), so
        // it is read as set; `die_hold_steps` takes it for the sweep.
        let (who, idx) = (self.units[i].owner, self.units[i].index);
        let ammo: Vec<DieAmmo> = self
            .projectiles
            .iter()
            .filter(|p| p.shooter == Obj::Unit(i))
            .map(|p| DieAmmo {
                flags: 1,
                who: i32::from(who),
                idx: i32::from(idx),
                total: p.total_time,
                cur: p.cur_time,
            })
            .collect();
        let hold = die_hold_steps(
            u16::try_from(Self::CLOSE_HOLD).expect("0x1e"),
            who,
            idx,
            true,
            Self::NUKE_SPREAD,
            &ammo,
        );
        self.units[i].hold_frames = i32::from(hold);
    }

    /// **`Object::close@00647160`'s hold**: the last write it makes to an
    /// object that was active, after `Objects::remove`, is
    /// `hold_frames = 0x1e` — on both of its arms, so on every close
    /// (`docs/COMBAT.md` §59.3). `Objects::process_all` takes one off per
    /// frame, and `Objects::find_free` will not hand the number out until
    /// it is zero.
    pub(crate) const CLOSE_HOLD: i32 = 0x1e;

    /// **`nuke_effect +0x108`**, a nuke's spread time in frames:
    /// `Nuke::init@0092c960` writes `0x1e` to it (`92c987`, `0xc0a888`), and
    /// nothing in the simulation writes it again (`Nuke::registration`
    /// exposes it as a tuning variable). `Nuke::do_damage` reads it as the
    /// blast's growth after `+0x104`'s 10 frames, and `Object::die` adds it
    /// to every live round's remaining time when it holds a dead shooter's
    /// number (item 1594, `docs/AI.md` §165).
    pub(crate) const NUKE_SPREAD: i32 = 0x1e;

    /// **`Ammo::inc_time@0067d380`'s first statement**: every frame, a
    /// shot in flight whose shooter is no longer active bumps the
    /// shooter's `hold_frames` (§42.3). On dead slots only, which is why
    /// the dump prints `hold_frames 0` on every living unit of every
    /// capture — the row `crate::diff::compare` now carries. The death
    /// objects' own bump is [`Self::death_objs_inc_time`], after the walk.
    pub(crate) fn hold_frames_tick(&mut self) {
        for k in 0..self.projectiles.len() {
            // A crash round bumps its own plane's slot in its step
            // ([`Sim::process_projectiles`]).
            if self.projectiles[k].crash.is_some() {
                continue;
            }
            let Obj::Unit(s) = self.projectiles[k].shooter else {
                continue;
            };
            if !self.active(Obj::Unit(s)) {
                self.units[s].hold_frames += 1;
            }
        }
    }

    /// **`DeathObj::inc_time@008d5240`'s first statement**: every frame, a
    /// death object bumps its own object's `hold_frames`, and then ends at
    /// its life ([`Self::cull_deaths`]).
    ///
    /// **After the ammo walk, not before it** (item 3873, `docs/GOLDEN.md`
    /// §156.1): `Objects::inc_time@0065db70` runs every `Ammo::inc_time`
    /// (line 74) and then every live record (line 86), so a ship a round
    /// sinks gets its record's first bump on its own frame. The hold it
    /// keeps while the record lives is then 31, not 30 — run1496's packet
    /// at 1539 reads the dead Carrier `1/18`'s `+0x32` at 2 where the
    /// bump ahead of the walk left 1, and the Jet that ours put into 18 on
    /// tick 1539 takes 41 in theirs.
    pub(crate) fn death_objs_inc_time(&mut self) {
        for k in 0..self.deaths.len() {
            let (who, o) = (self.deaths[k].who, self.deaths[k].o);
            // **One bump a record** (item 3784, `docs/GOLDEN.md` §149.10):
            // `DeathObj::inc_time`'s first statement is on each guy's
            // record, and `process_all` takes one off a frame — so a dead
            // Bombard's four records hold its number long past their cull.
            let bumps = self.deaths[k]
                .figures
                .live(self.frame - self.deaths[k].first_frame);
            // The object in the slot, dead or not: `unit_by_o` finds only
            // the living, and this bump is on the dead (§59).
            if let Some(u) = self
                .units
                .iter()
                .rposition(|u| i32::from(u.owner) == who && i32::from(u.index) == o)
            {
                self.units[u].hold_frames = self.units[u].hold_frames.saturating_add(bumps);
            }
        }
        self.cull_deaths();
    }

    /// **The death object's end** (`DeathObj::inc_time@008d5240`, item
    /// 3205, `docs/GOLDEN.md` §127): its counter, one per call and zero at
    /// `init`, reaches `life`, the record is zeroed and — for a type whose
    /// `blocks_while_dead` kept the disc — `clear_blocking` takes the disc
    /// out of the occupancy index. The bump above has already run, as the
    /// listing's first statement does.
    fn cull_deaths(&mut self) {
        let frame = self.frame;
        let mut k = 0;
        while k < self.deaths.len() {
            let d = self.deaths[k];
            let ended = d.life.is_some_and(|l| frame - d.first_frame + 1 >= l);
            if !ended {
                k += 1;
                continue;
            }
            self.deaths.remove(k);
            if let Some(u) = self
                .units
                .iter()
                .rposition(|u| i32::from(u.owner) == d.who && i32::from(u.index) == d.o)
            {
                let blocks = !self.units[u].alive()
                    && self.units[u].ty.is_some_and(|t| {
                        self.unit_types[t].cols.unit_flags
                            & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                            != 0
                    });
                if blocks && self.units[u].coll_at.is_none() {
                    self.coll_remove(u);
                }
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
    /// The loop's other arm, taken at each level of the captain walk — a
    /// group member whose type is *not* combat-role (`type +0x2c8 &
    /// 0x10000`) forwards to `Group::target_opportunity` — is built
    /// (item 1403, `docs/GOLDEN.md` §58): see [`Sim::target_opportunity_in`].
    fn target_opportunity(&mut self, victim: usize, attacker: Obj, _frame: i64) {
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
                    let range = (d + 0xc0).min(self.tuning.unit_respond_range * 0x240);
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
    fn attack_action_yields(&mut self, u: usize, attacker: Obj) -> bool {
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
    fn mark_contact(&mut self, u: usize, attacker: Obj, front: Option<crate::orders::Order>) {
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
    fn counter_craft(
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
    fn obj_number(&self, o: Obj) -> i16 {
        match o {
            Obj::Unit(v) => self.units[v].index,
            Obj::Building(b) => self.buildings[b].index,
        }
    }

    fn flee_from(&mut self, u: usize, attacker: Obj) -> bool {
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
            .get(&self.unit_types[ty].type_index)
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
                                    if self.buildings[b].ty.is_some_and(|t| self.build_types[t].wonder));
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
            let types = &self.build_types;
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
            if self.buildings[b].ty.is_some_and(|t| self.build_types[t].wonder))
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
                    .is_some_and(|t| crate::build::is_dock(&self.build_types, t))
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
            if tp.is(role::SUPPLY) {
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
                    self.unit_types[t].cols.unit_flags2 & crate::ai_load::uflags2::CASTER != 0
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
                        .map_or(0, |t| self.unit_types[t].cols.unit_flags),
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
                } else if flags_of(target) & uflags::TRANSPORT != 0 {
                    v = v.wrapping_add(900_000);
                } else if line(target, crate::fish::FISHERMEN) {
                    v = v.wrapping_add(100_000);
                } else if tp.combat_role {
                    v = v.wrapping_add(10_000);
                }
            } else if raiding {
                if peasant || tp.is(role::CARAVAN) {
                    v = v.wrapping_add(if ai_raider { 900_000 } else { 9_000_000 });
                } else if ai_raider && tp.combat_role {
                    v = v.wrapping_add(10_000);
                } else {
                    v /= 10;
                }
            } else if tp.combat_role {
                v = v.wrapping_add(1_000_000);
            } else if tp.is(role::SUPPLY) {
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
                && ap.is(role::BARK)
            {
                if tp.is(role::TRIREME) {
                    v /= 400;
                } else if let Obj::Unit(u) = target
                    && self.units[u].ty.is_some_and(|t| {
                        self.unit_types[t].cols.unit_flags & uflags::TRANSPORT != 0
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
        // **A site does not shoot** (item 1323, `docs/COMBAT.md` §8.6):
        // `Build::process@0061edf0` returns straight after `Wall::process`
        // unless the building is finished — `61ee1b` reads `flags & 4`
        // inline for `Build::vftable@00b42174`, else vslot `+0x4c`, which
        // the PE holds as `0x472350`, `WallData::is_active` (`flags & 4`
        // again) — and `61ee31` jumps past `do_attack` to the epilogue.
        // Chapter forty-seven's Tower site `0/2008` fired a round on 712
        // here and never did in run514.
        if !self.buildings[b].active {
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
        // **Or on any frame its nearest sighting is in range** — the head of
        // `Build::process@0061edf0` (`61ee9e`..`61eec4`, `docs/GOLDEN.md`
        // §59): with no target, `do_attack` runs when `(o + frame) & 0x1f`
        // is 0, else when `near_o`/`near_who` (+0x34/+0x36), the last
        // search's nearest sighting inside `0xf00`, is in range.
        if bd.target.is_none() && phase != 0 && !bd.near.is_some_and(|n| self.is_in_range(me, n)) {
            return;
        }
        if !cycles && bd.recharging > 0 {
            self.buildings[b].recharging -= 1;
            return;
        }
        // **The head of `do_attack` past the countdown** (`62295a`..
        // `622978`): on the building's 32-frame phase, with the latch
        // down, `visible` is cleared; the latch drops every call.
        if phase == 0 && !self.buildings[b].attacking {
            self.buildings[b].visible = 0;
        }
        self.buildings[b].attacking = false;
        let bd = &self.buildings[b];
        let p = self.profile(me);
        // The garrison's arrows: the input the earlier mechanic took, plus
        // what the squads actually inside contribute (`docs/CITIES.md` §6).
        let garrison_attack = bd.garrison_attack + self.garrison_attack_sum(b);
        // `BuildData::get_base_garrison_arrows@0062e340`: the Maya's
        // `MAYA_GARRISON_ARROWS` (item 3970).
        let arrows = combat::garrison_arrows(
            self.attack_of(me),
            self.base_garrison_arrows(b),
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
            let ox = if xs - 1 < 1 {
                0
            } else {
                self.mark(SITE_FIRE_FOOTPRINT_X);
                self.rng.roll() % xs
            };
            let oy = if ys - 1 < 1 {
                0
            } else {
                self.mark(SITE_FIRE_FOOTPRINT_Y);
                self.rng.roll() % ys
            };
            let launch = Pos::new(centre.x - xs / 2 + ox, centre.y - ys / 2 + oy);
            let angle = find_angle(target_pos.x - launch.x, target_pos.y - launch.y);
            let landing =
                self.scatter_landing(target_pos, s, (SITE_AMMO_SCATTER_X, SITE_AMMO_SCATTER_Y));
            // **A building's arrow flies at 90, not at its type's
            // `PROJ_SPEED`** (`Ammo::init@0067bbf0`'s building arm,
            // `0067d1e0`): `get_shot() == 0` divides by
            // `unit_move_speed × 0x5a`. A Tower in the first three ages
            // reads 0 (item 1380, `docs/GOLDEN.md` §57).
            let d = if self.building_shot(b) == 0 {
                i64::from(combat::UNIT_MOVE_SPEED) * 0x5a
            } else {
                i64::from(p.proj_speed) * i64::from(combat::UNIT_MOVE_SPEED)
            };
            let dx = i64::from(landing.x - launch.x);
            let dy = i64::from(landing.y - launch.y);
            let total_time = combat::flight_time(dx * dx + dy * dy, d).max(1);
            let landing = self.lead_landing(Some(target), landing, total_time);
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
                flame: false,
                // SEAM: the building's ammo piece is not read, so its
                // `BuildingArrow` (`puncture="1"`) never sticks here
                // (item 3679, `docs/COMBAT.md` §46.7).
                punctures: false,
                spent: false,
                crash: None,
                spline: None,
            });
        }
        // `do_attack`'s tail after `fire_ammo` (`622b5b`..`622bd0`): the
        // latch is raised, and the target's player gets the building's
        // `visible` bit; a player that has never seen it has the building
        // light itself into its fog (vslot `+0x164`).
        self.buildings[b].attacking = true;
        let bit = Self::who_bit(self.owner_of(target));
        if self.buildings[b].visible & bit == 0 {
            self.buildings[b].visible |= bit;
            if self.buildings[b].ever_seen & bit == 0 {
                self.update_local_seen_build(b);
            }
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
            // **A spent arrow** (item 3679, `docs/COMBAT.md` §46.7):
            // `inc_time`'s `flags & 2` clear arm returns until `cur_time`
            // reaches 200, and then closes it. Its dead shooter's bump is
            // [`Sim::hold_frames_tick`]'s, as for any round.
            if p.spent {
                if p.cur_time >= SPENT_LIFE {
                    self.projectiles.remove(i);
                } else {
                    i += 1;
                }
                continue;
            }
            if p.crash.is_some() {
                // `inc_time`'s first statement, here rather than in
                // [`Sim::hold_frames_tick`]: a crash is laid inside this walk,
                // and the frame it is laid on bumps its plane's slot too.
                if let Obj::Unit(s) = p.shooter
                    && !self.active(p.shooter)
                {
                    self.units[s].hold_frames += 1;
                }
                if p.in_flight() {
                    if !self.crash_comes_down(i) {
                        i += 1;
                        continue;
                    }
                    // The landing arm jumps back to the loop's head
                    // (`67d726` → `67d39a`): `cur_time` and a dead
                    // shooter's `hold_frames` are bumped a second time,
                    // and `cur_time >= total_time` ends it.
                    self.projectiles[i].cur_time += 1;
                    if let Obj::Unit(s) = p.shooter
                        && !self.active(p.shooter)
                    {
                        self.units[s].hold_frames += 1;
                    }
                }
                // **And it strikes there** (item 2677, `docs/AI.md` §230):
                // `inc_time`'s tail restricts the point and runs
                // `Ammo::do_damage` — `init_crash` cleared flag `0x10` —
                // and only then closes, as an ordinary round does below.
                let p = self.projectiles[i];
                self.land(p, frame);
                if let Some(k) = self.projectiles.iter().position(|q| q.slot == p.slot) {
                    self.projectiles.remove(k);
                }
                i = self.projectiles.partition_point(|q| q.slot <= p.slot);
                continue;
            }
            // **A guided round** (item 3802): `inc_time`'s spline arm, which
            // either closes it on this frame or holds it, re-planned or not.
            if p.in_flight() && !(p.spline.is_some() && self.guided_closes(i)) {
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
                    if (p.total_time.wrapping_mul(3) as u32) < p.cur_time as u32 {
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
            // **The round keeps its pool slot while it lands** (item 2609,
            // `docs/AI.md` §228): `Ammo::inc_time` runs `do_damage` and only
            // then closes, so a plane it brings down lays its crash round in
            // the next free slot — run405's `0/6` crash takes index 1 under
            // the Battery's round in 0. The walk then goes on from the next
            // slot past this one, as the pool's own walk does.
            let p = self.projectiles[i];
            // `inc_time:260`: a round with flag `0x10` is closed, and
            // `Ammo::do_damage` is never called for it (§39.5).
            if !p.harmless {
                self.land(p, frame);
            }
            if let Some(k) = self.projectiles.iter().position(|q| q.slot == p.slot)
                && !self.projectiles[k].spent
            {
                self.projectiles.remove(k);
            }
            i = self.projectiles.partition_point(|q| q.slot <= p.slot);
        }
    }

    /// **`Ammo::inc_time@0067d380`'s spline arm** (`67d3e6`–`67d57a`,
    /// `67d72b`–`67d8ca`; item 3802, `docs/GOLDEN.md` §152.2), on a frame
    /// short of `total_time`, for a round with a spline. It answers whether
    /// the round closes now.
    ///
    /// - With no target, or one no longer active, or no point past this
    ///   one (`len ≤ cur + 1`), it holds: the round flies on to
    ///   `total_time` and lands on its end point.
    /// - Point `cur`, truncated, is measured against the target's point:
    ///   `vector_dist(|dx|, |dy|)` under the target type's `target_size`
    ///   (`UnitTypeData +0x300`; `0xc0` for a building), and `|dz| < 0xc0`
    ///   against the target figure's height. Inside both, the end point
    ///   becomes that point and `total_time` `cur − 1`, and the arm jumps
    ///   back to the loop's head (`67d57a` → `67d392`): `cur_time` and a
    ///   dead shooter's `hold_frames` are bumped a second time, and the
    ///   round lands there.
    /// - Otherwise, on every fourth frame of a spline (`cur ≠ 0`, `cur & 3
    ///   == 0`) and a unit target, it **re-plans**: a new spline from the
    ///   point (its integers), through `P + 2·(point[cur+1] − P)`, to the
    ///   target's point now, at `200 × unit_move_speed`; the end point is
    ///   the target's, `total_time` the new point count and `cur_time` 0.
    ///
    /// run1469: the Russian Jet Fighter `2/14`'s round on `1/14` re-plans
    /// on 620 (nine points) and 624 (six), and closes on 628 at `cur` 4,
    /// (12848, 14983) against the target's (12792, 14712), a frame before
    /// the arc ours flew would have landed.
    ///
    /// SEAM: a ground unit's figure height (`guys[0] +0x14`) is read as its
    /// object's, which this crate carries; an aircraft's is its airframe's.
    fn guided_closes(&mut self, i: usize) -> bool {
        let p = self.projectiles[i];
        let Some(path) = p.spline else {
            return false;
        };
        let Some(t) = p.target.filter(|&t| self.active(t)) else {
            return false;
        };
        let cur = usize::try_from(p.cur_time).unwrap_or(0);
        if path.len() <= cur + 1 {
            return false;
        }
        let pt = path.point(cur).map(Single::to_i32);
        let at = self.pos_of(t);
        let tz = match t {
            Obj::Unit(u) if self.units[u].kind.domain == Domain::Air => self.units[u].airframe.z,
            _ => self.world.object_z(at.tile()),
        };
        let radius = match t {
            Obj::Unit(_) => self.profile(t).target_size,
            Obj::Building(_) => 0xc0,
        };
        if vector_dist((pt[0] - at.x).abs(), (pt[1] - at.y).abs()) < radius
            && (pt[2] - tz).abs() < 0xc0
        {
            let q = &mut self.projectiles[i];
            q.landing = Pos::new(pt[0], pt[1]);
            q.ez = pt[2];
            q.total_time = q.cur_time - 1;
            q.cur_time += 1;
            if let Obj::Unit(s) = p.shooter
                && !self.active(p.shooter)
            {
                self.units[s].hold_frames += 1;
            }
            return true;
        }
        if cur != 0 && cur % 4 == 0 && matches!(t, Obj::Unit(_)) {
            let from = pt.map(Single::from_i32);
            let ahead = path.point(cur + 1);
            let two = Single::from_bits(0x4000_0000);
            let dir = [0, 1, 2].map(|c| ahead[c].subss(from[c]).mulss(two).addss(from[c]));
            let end = [at.x, at.y, tz].map(Single::from_i32);
            let speed = Single::from_i32(200 * combat::UNIT_MOVE_SPEED);
            let next = combat::Spline::from_dir(from, dir, end, speed);
            let q = &mut self.projectiles[i];
            q.landing = at;
            q.ez = tz;
            q.spline = Some(next);
            q.total_time = i32::try_from(next.len()).expect("at most SPLINE_MAX");
            q.cur_time = 0;
        }
        false
    }

    /// **`Ammo::inc_time@0067d380`'s crash arm** (`+0x50 == 1`,
    /// `67d590`–`67d726`; `docs/AI.md` §228), on a frame short of
    /// `total_time`: the landing drifts by the bank pair and is
    /// `WorldData::restrict`ed, and then the round is tested against the
    /// ground at its point on the line, `(float)find_data_z > arc` —
    /// [`combat::arc_point`] and [`combat::arc_z`], the rolling arm's own
    /// arithmetic. Under the ground it comes down there: the point is its
    /// landing, `ez` the ground, `total_time` one past `cur_time`.
    /// Returns whether it came down.
    ///
    /// SEAM: the arm returns before the test when
    /// `GraphicPieces::get_gpiece_type(+0x8)` is non-zero; this crate loads
    /// no graphic piece and reads it as zero. run405's two crashes end
    /// short of their `total_time` (13 of 18, 12 of 17), which only the
    /// test does.
    fn crash_comes_down(&mut self, i: usize) -> bool {
        let p = self.projectiles[i];
        let Some(c) = p.crash else { return false };
        let mut landing = p.landing;
        if c.bank != (0, 0) {
            landing = self.restrict_pos(Pos::new(landing.x + c.bank.0, landing.y + c.bank.1));
            self.projectiles[i].landing = landing;
        }
        let at = combat::arc_point(p.launch, landing, p.cur_time, p.total_time);
        let ground = self.ground_z(at);
        let z = combat::arc_z(p.v1z, p.sz, p.cur_time);
        if !crate::single::Single::from_i32(ground).gt(z) {
            return false;
        }
        let q = &mut self.projectiles[i];
        q.landing = at;
        q.total_time = q.cur_time + 1;
        q.ez = ground;
        true
    }

    /// `Ammo::hit_target@00678f90` (§9.4): the shot's own target, still
    /// active and still inside its `target_size` (or its footprint, for a
    /// building), at the landing point.
    fn hit_target(&self, p: &combat::Projectile) -> bool {
        p.target
            .filter(|&t| self.struck(t))
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

    /// **`do_damage`'s puncture arm, after its two draws** (`678c59`..,
    /// item 3679, `docs/COMBAT.md` §46.7): the point is
    /// `WorldData::restrict`ed and, when the round's piece punctures
    /// (`verify_ammo_flags(…, 1)`) and the tile under the point is not
    /// ocean (`mask & 0x30 != 0x20`; `is_valid` cannot refuse a restricted
    /// point), the round is left in its slot as `flags 1` with `cur_time`
    /// 0 — [`combat::Projectile::spent`] — rather than closed. The point
    /// is the decal's; the record keeps its own `ex`/`ey` and its target.
    fn stick(&mut self, p: combat::Projectile, at: Pos) {
        if !p.punctures || p.crash.is_some() {
            return;
        }
        let at = self.restrict_pos(at);
        if self.world.is_tocean(at.tile()) {
            return;
        }
        // `flags = 1` whole, so the rolled and missed bits go with the live
        // one (run1331b: `1/52`'s spent arrow reads `flags 1` from 3333,
        // where ours kept `4` and `8`); the target pair is not written.
        if let Some(q) = self.projectiles.iter_mut().find(|q| q.slot == p.slot) {
            q.spent = true;
            q.cur_time = 0;
            q.rolling = false;
            q.missed = false;
        }
    }

    /// `Ammo::do_damage` (§9.3): the hit test at the landing point, then
    /// `Object::do_damage` on what was hit — the target, or whatever stood
    /// there instead — and the splash around it.
    fn land(&mut self, p: combat::Projectile, frame: i64) {
        let mut target = p.target.filter(|&t| self.struck(t));
        // **A round whose target is gone lands as a ground round**
        // (`Ammo::do_damage@00678060`'s first test, item 3829, `docs/
        // GOLDEN.md` §152.12): an inactive target object (`+8 & 1` clear)
        // has `whom`/`ox` set to −1 before anything else, and the domain
        // `check_hit` and the splash's unit arm are handed is the target
        // type's `+0x218` only while those are set — `GROUND` otherwise.
        // So an air shooter's round on a plane that died in flight finds
        // and splashes only what is not air: run1476's `0/15` round of 792
        // on the dead Stealth Bomber `1/17` comes down on 797 455 from the
        // Jet Fighter Bomber `1/18` and leaves it whole (205 on blocks
        // 796–799), where ours splashed it for 3½.
        let p = combat::Projectile {
            air: p.air && target.is_some(),
            ..p
        };
        if !self.hit_target(&p) {
            target = self.check_hit(&p);
        }
        if p.splash_area == 0 {
            let Some(t) = target else {
                // A shot into the ground: two draws for where it punctures
                // (§39). Marked per draw, because the original spends them
                // at two addresses.
                self.mark(SITE_PUNCTURE_X);
                let dx = self.rng.roll() % 41 - 20;
                self.mark(SITE_PUNCTURE_Y);
                let dy = self.rng.roll() % 41 - 20;
                self.stick(p, Pos::new(p.landing.x + dx, p.landing.y + dy));
                return;
            };
            if t == p.shooter {
                return;
            }
            let angle = p.bearing();
            self.do_damage_round(
                p.shooter, t, angle, true, p.flame, 0x100, false, false, frame,
            );
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
        // and the one cell past it the inclusive walk reads
        // ([`combat::splash_extra_cell`]), the intended target at full count
        // and everything else as a fringe.
        let k = combat::splash_cells(p.splash_area);
        let extra = combat::splash_extra_cell(k);
        let landing_cell = p.landing.cell();
        let candidates: Vec<Obj> = (0..self.units.len())
            .map(Obj::Unit)
            .chain((0..self.buildings.len()).map(Obj::Building))
            // The round's own target is struck while it stands, hit points
            // or none (`struck`): a second siege shot lands on a city the
            // first has taken to its ceiling (item 1528).
            .filter(|&o| self.struck(o))
            .filter(|&o| {
                let c = self.pos_of(o).cell();
                let (dx, dy) = (c.x - landing_cell.x, c.y - landing_cell.y);
                // The square, and the spiral's next entry the inclusive
                // walk reads (item 3636): run1296b's Bombard round on 5047
                // lands in cell (8, 9) and wounds the Library `0/2005`,
                // chained in (7, 7).
                (dx.abs() <= k && dy.abs() <= k) || extra == Some((dx, dy))
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
            self.do_damage_round(
                p.shooter, o, angle, true, p.flame, count, splash, false, frame,
            );
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
    /// the tile. This crate compared the tile against the building's point
    /// in position units, and no building was ever found: a V2 with no
    /// target that came down on run371's Barracks `1/2006` struck it as a
    /// splash fringe, 400 of its 1,200, where the original struck it as
    /// the round's own target and destroyed it on 2821.
    ///
    /// **Active is the two flag reads and nothing about hit points** (item
    /// 3539, `docs/AI.md` §303): `0065ac7f` calls vslot `0xc` and `0065ac8e`
    /// vslot `0x4c`, flags `& 1` and `& 4`. [`Sim::active`] also asks for
    /// health left, the target search's question, and refused a city at
    /// its ceiling: run1171's Napata `0/2000`, reduced on 4341, whose
    /// footprint holds the tile `1/28`'s missed round comes down on at
    /// 4443. Theirs strikes it (`build_masks` takes `0x4000` on block
    /// 4444) and makes no puncture draws; ours punctured the ground. A
    /// building with no combat profile is still refused, as
    /// [`Sim::struck`] refuses it.
    ///
    /// **The walk's order picks when footprints overlap** (item 3970,
    /// `docs/GOLDEN.md` §176): the nine cells are `move_x`/`move_y`'s
    /// first nine round the tile's cell (`tile >> 2`), the centre first,
    /// and each cell's chain holds the buildings whose point is in it. A
    /// console `add` overlaps: chapter 126's Temple `1/2006` stands on the
    /// Library `1/2005`'s footprint, and the City's arrow missing `0/7` on
    /// 804 comes down on the tile both hold — theirs strikes the Temple,
    /// whose point is in the landing's own cell, where the first building by
    /// index took the Library. Inside one cell the order is the building
    /// index, as the target search's is; the chain's own order there is not
    /// established.
    fn building_under(&self, landing: Pos) -> Option<usize> {
        if landing.x < 0
            || landing.y < 0
            || landing.x >= self.world.width() * UNITS_PER_CELL
            || landing.y >= self.world.height() * UNITS_PER_CELL
        {
            return None;
        }
        let tile = landing.tile();
        let at = landing.cell();
        let (mx, my) = (crate::ai_place::MOVE_X, crate::ai_place::MOVE_Y);
        mx.iter().zip(my).find_map(|(&dx, dy)| {
            let cell = crate::Cell::new(at.x + dx, at.y + dy);
            (0..self.buildings.len()).find(|&b| {
                self.buildings[b].pos.cell() == cell
                    && self.buildings[b].owner < crate::world::PLAYER_SLOTS
                    && self.struck(Obj::Building(b))
                    && self.buildings[b].active
                    && self.covers_tile(b, tile)
            })
        })
    }
}

/// `TypeIndex` `0x49`, the Commando line (`enums/TypeIndex.txt`): the
/// Special Forces rows `is` it.
const COMMANDO: crate::tech::TypeId = 0x49;

/// `Object::take_damage` on a site: `lost × 50` off the progress, `lost`
/// the whole points the blow took after its sixteenths carried.
const fn combat_progress_lost(whole: i32) -> i32 {
    crate::build::progress_lost(whole)
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

/// **`ObjectData::is_in_range@006486b0`'s steps** (item 2314,
/// `docs/COMBAT.md` §13.2), in its order: the target's slot (`flags & 1`)
/// and, for a unit (vslot `+8`), `is_on_map` (vslot `+0xbc`); the world
/// cell under `at`, refused when forest; `attack_dist` at the quarter-tile
/// centre of `at` (`div_3_table[v >> 4] * 0x30 + 0x18`), written to the
/// out-word. A type with no range (`+0x1fc` zero) reaches `0x66`, or
/// `0xf6` for the HOPLITES lineage (`is(0x84, 0)`). Otherwise inside
/// `min_range × 0xc0 − 6` (vslot `+0x12c`) the attacker's and the target's
/// `big_radius` (`type +0x244`, each only for a unit, vslot `+0x18`) are
/// added before the inner bound is asked again, and the outer bound is
/// `max_range × 0xc0 + 6` (vslot `+0x130`) with `0x90` added for the
/// margin. The sums are the original's 32-bit ones, wrapping.
#[allow(clippy::too_many_arguments)]
pub(crate) fn in_range_steps(
    slot: bool,
    target_unit: impl FnOnce() -> bool,
    on_map: impl FnOnce() -> bool,
    at: Pos,
    forest: impl FnOnce(Pos) -> bool,
    dist: impl FnOnce(Pos) -> i32,
    out: Option<&mut i32>,
    ranged: bool,
    hoplite: impl FnOnce() -> bool,
    min_range: impl FnOnce() -> i32,
    mut big: impl FnMut(bool) -> (bool, i32),
    max_range: impl FnOnce() -> i32,
    margin: bool,
) -> bool {
    if !slot || (target_unit() && !on_map()) || forest(at) {
        return false;
    }
    let centre = Pos::new(
        at.x.div_euclid(48) * 48 + 0x18,
        at.y.div_euclid(48) * 48 + 0x18,
    );
    let d = dist(centre);
    if let Some(o) = out {
        *o = d;
    }
    if !ranged {
        return d <= if hoplite() { 0xf6 } else { 0x66 };
    }
    let inner = min_range().wrapping_mul(0xc0).wrapping_sub(6);
    if d < inner {
        let radius = |(unit, r): (bool, i32)| if unit { r } else { 0 };
        let mine = radius(big(false));
        let theirs = radius(big(true));
        if d.wrapping_add(theirs).wrapping_add(mine) < inner {
            return false;
        }
    }
    let reach = max_range().wrapping_mul(0xc0).wrapping_add(6);
    d.wrapping_add(if margin { 0x90 } else { 0 }) <= reach
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
fn closing_test<V: TargetConstView>(v: &mut V) -> bool {
    !((v.self_vslot_20() || v.self_domain() == 0) && v.self_type_flags() < 0)
}

/// `006476b3`: the asker's slot type's fly-high / fly-low against the
/// target's unit, after the target's own side passed.
fn slot_ladder_refuses<V: TargetConstView>(v: &mut V) -> bool {
    if v.slot_fly_high() != 0 {
        v.slot_fly_low() == 0 && v.target_is_flying_low()
    } else {
        v.target_is_flying_high()
    }
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

/// One live record of `GameAccess::objects`' array as `Object::die@00647080`
/// reads it: the flag byte (`+4`), the shooter pair (`+0x3c` owner, `+0x40`
/// slot) and the two times (`+0x24` current, `+0x28` total).
#[derive(Clone, Copy)]
pub struct DieAmmo {
    pub flags: u8,
    pub who: i32,
    pub idx: i32,
    pub total: i32,
    pub cur: i32,
}

/// **`Object::die@00647080`'s tail** after the `close` it calls through
/// vslot `+0x150` (which leaves `hold`): when the type's `+0x1fc` is set,
/// every record with `flags & 3 != 0` whose shooter pair is this object
/// raises the hold to `nuke + 1 + (total - cur)` (32-bit wrapping, compared
/// signed); the hold is at least 1, and is stored as a short.
pub fn die_hold_steps(
    hold: u16,
    who: u8,
    idx: i16,
    gate: bool,
    nuke: i32,
    ammo: &[DieAmmo],
) -> u16 {
    let mut v: i32 = 1;
    if gate {
        for a in ammo {
            if a.flags & 3 != 0 && a.idx == i32::from(idx) && a.who == i32::from(who) {
                let t = nuke
                    .wrapping_add(1)
                    .wrapping_add(a.total.wrapping_sub(a.cur));
                v = v.max(t);
            }
        }
    }
    v = v.max(i32::from(hold));
    v as u16
}

/// [`crate::object_attack_view::ObjectAttackView`] over the crate's [`Sim`]: the object's type record and
/// owner. A building answers false on the unit slots (`+0x18`, `+0xd0`, `+0xcc`, `+0xc8`).
struct SimObjectAttack<'a> {
    sim: &'a Sim,
    o: Obj,
}

impl SimObjectAttack<'_> {
    fn rec(&self) -> Option<usize> {
        match self.o {
            Obj::Unit(u) => self.sim.units[u].ty,
            Obj::Building(_) => None,
        }
    }
    fn owner(&self) -> usize {
        self.sim.owner_of(self.o) as usize
    }
}

impl crate::object_attack_view::ObjectAttackView for SimObjectAttack<'_> {
    fn type_attack(&mut self) -> i32 {
        self.sim.profile(self.o).attack
    }
    fn is_caravan(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.unit_types[r]
                .cols
                .flag2(crate::ai_load::uflags2::CARAVAN)
        })
    }
    fn has_caravan_preq(&mut self) -> bool {
        // A tree that does not name the bonus's prerequisite holds it for nobody.
        match (
            self.sim.tech_tree.roles.caravan_attack_preq,
            self.sim.tech.get(self.owner()),
        ) {
            (Some(t), Some(p)) => self.sim.tech_tree.has_tech(&self.sim.setup, p, t),
            _ => false,
        }
    }
    fn caravan_attack_bonus(&mut self) -> i32 {
        self.sim.tuning.caravan_attack_bonus
    }
    fn has_dutch_bonus(&mut self) -> bool {
        self.sim.tech.get(self.owner()).is_some_and(|p| {
            self.sim
                .tech_tree
                .has_tribe_bonus(&self.sim.setup, p, crate::ai::tribe::DUTCH)
        })
    }
    fn dutch_attack_bonus(&mut self) -> i32 {
        self.sim.tuning.dutch_attack_bonus
    }
    fn is_unit(&mut self) -> bool {
        matches!(self.o, Obj::Unit(_))
    }
    fn type_index(&mut self) -> i32 {
        self.rec().map_or(0, |r| self.sim.unit_types[r].type_index)
    }
    fn is_supply(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.unit_types[r]
                .cols
                .flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
        })
    }
    fn gov_hero_slot(&mut self) -> bool {
        self.rec().is_some_and(|r| {
            self.sim.unit_types[r]
                .tree
                .and_then(|i| self.sim.tech_tree.types.get(i))
                .and_then(|d| d.kind.unit())
                .is_some_and(|u| u.patriot)
        })
    }
    fn ages(&mut self) -> i32 {
        self.sim.tech.get(self.owner()).map_or(0, |p| p.ages)
    }
}

#[cfg(test)]
mod tests;
