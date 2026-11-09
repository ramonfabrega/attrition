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

use crate::ai::ai_load::uflags;
use crate::combat::{self, Obj, Profile, Side, Sixteenths, Stance, Taken, mask, role};
use crate::movement::{Angle, find_angle};
use crate::num::single::Single;
use crate::territory::attrition::Domain;
use crate::units::anim;
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

/// `TypeIndex` `0x49`, the Commando line (`enums/TypeIndex.txt`): the
/// Special Forces rows `is` it.
const COMMANDO: crate::tech::TypeId = 0x49;

/// `Object::take_damage` on a site: `lost × 50` off the progress, `lost`
/// the whole points the blow took after its sixteenths carried.
const fn combat_progress_lost(whole: i32) -> i32 {
    crate::cities::build::progress_lost(whole)
}

// Item 4015 cut this file six ways (STRUCTURE §5, move 7). Each piece is a
// child, so an item private here is `pub(super)` there and reaches what it
// reached; the globs keep every `crate::combat::fight::x` path. `stats`,
// `acquire` and `projectiles` hold `impl Sim` methods and nothing named
// through this path, so they take no glob.
mod acquire;
mod attack_step;
mod delivery;
mod projectiles;
mod stats;
mod target_valid;
#[cfg(test)]
mod tests;
pub(crate) use self::attack_step::*;
pub use self::delivery::*;
pub use self::target_valid::*;
