//! Combat: what an attack is worth, how it is delivered, how often it lands,
//! and where a shot falls.
//!
//! `docs/COMBAT.md` is the specification; section numbers below refer to it.
//! Everything here is integer arithmetic at the original's own scales: attack
//! in tenths, damage carried in sixteenths, modifiers in percent or 8.8, one
//! 32-bit LCG for the scatter. The one place the original reaches for a float
//! — a projectile's flight time — is reproduced by rounding each IEEE step
//! exactly in integers ([`flight_time`]).
//!
//! The layer of the original that is *not* here is the one that reads the
//! nation, wonder and patriot tables: every "+N attack under Alexander" is a
//! [`Modifiers`] input, the same choice `docs/COSTS.md` made for discounts.

use crate::core::tuning::Tuning;
use crate::movement::Angle;
use crate::num::single::Single;
use crate::territory::attrition::Domain;
use crate::world::{Pos, vector_dist};

/// `UNIT_MOVE_SPEED`: the data's `1/192 tile`, which in position units is one —
/// the identity converter `docs/MOVEMENT.md` describes. Projectile speeds go
/// through it the way unit speeds do.
pub const UNIT_MOVE_SPEED: i32 = 1;

/// The game's random number generator — `Random`, one `ulong` seed.
///
/// `get(lo, hi)` advances the seed by the classic 32-bit LCG and maps the
/// high half of the low word onto `lo..hi`. Combat draws from it for every
/// projectile's scatter (§9.5), so reproducing the sequence exactly is what
/// reproduces the misses. A call with `lo == hi` returns `lo` **without
/// advancing**, which is the original's short-circuit and matters for the
/// draw count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    pub seed: u32,
}

impl Rng {
    pub const fn new(seed: u32) -> Rng {
        Rng { seed }
    }

    /// `Random::get(lo, hi)`.
    pub const fn get(&mut self, lo: i32, hi: i32) -> i32 {
        if lo == hi {
            return lo;
        }
        let (lo, hi) = if hi < lo { (hi, lo) } else { (lo, hi) };
        self.seed = self.seed.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
        let span = (hi - lo) as u32;
        (((self.seed & 0xffff) * span) >> 16) as i32 + lo
    }

    /// The draw every combat site takes: `Random::get(0, 0xffff)`, a value in
    /// `0..=0xfffe`.
    pub const fn roll(&mut self) -> i32 {
        self.get(0, 0xffff)
    }

    /// `Random::reseed` — XOR-swap with `x`, returning the old seed.
    pub const fn reseed(&mut self, x: u32) -> u32 {
        let old = self.seed;
        self.seed = x;
        old
    }
}

/// The bits of `UnitData::unit_masks2` (`+0x6c`) this crate writes —
/// [`crate::Unit::unit_masks2`].
pub mod umask2 {
    /// **`4` — a unit of the Bark line** (`Unit::init@00612100:604-611`,
    /// `is(BARK)`, item 2165, `docs/GOLDEN.md` §79): set at birth on every
    /// such unit, whoever owns it, and cleared on every other. `Unit::fight`
    /// reads it for its captain's re-search predicate ([`crate::Sim`]'s
    /// `fight`, parked 2100).
    pub const MASK2_BARK: u32 = 0x4;
    /// **`0x80` and `0x4000` — an Iroquois Scout** (`Unit::init@00612100:
    /// 341-357`, item 3107, `docs/AI.md` §246): a unit of a leader with
    /// tribe bonus 0x12 that `is(SCOUT)` takes `0x4080` at birth.
    /// `0x4000` is the forest walker `UnitData::invalid_loc@00607c30`
    /// reads (`:121`, `:134`, `:152`): forest does not refuse it. run897's
    /// Scout `1/0` prints `unit_masks2` 49280 on block 1, these two and the
    /// cloak `0x8000` frame 0's idle adds.
    pub const IROQUOIS_SCOUT: u32 = 0x4080;
    /// **`0x80` — an Iroquois soldier** (`Unit::init@00612100:341-352`,
    /// item 3958, `docs/AI.md` §402): a land type without objmask
    /// `CIVILIAN` takes it at birth; `Unit::process_cloak@005e1c20` reads
    /// it for [`LAND_CLOAK`].
    pub const CLOAK_MARK: u32 = 0x80;
    /// **`0x8000` — the own-land cloak** (`Unit::process_cloak@005e1c20:
    /// 47-58`): cleared on every pass and raised again on a
    /// [`CLOAK_MARK`] unit standing on its owner's (or a mutual ally's)
    /// ground. `UnitData::is_seen` and `is_cloaked` read it.
    pub const LAND_CLOAK: u32 = 0x8000;
    pub const FOREST_WALKER: u32 = 0x4000;
    /// **`0x10` — "still ordered to attack, reload open, and not firing
    /// this frame"** (`docs/COMBAT.md` §43.2).
    ///
    /// `Unit::fight@005fd4d0` sets it on two arms and both read the same
    /// pair above them, `order_type() == ATTACK && recharging == 0`: the
    /// tail of the invalid-target branch at `LAB_005fdb9e`, after
    /// `find_new_target` has killed the order and the search has put a new
    /// one in front; and `LAB_005fe502`, where `find_attack_pos` returned
    /// nothing or the defend point is out of
    /// `unit_defensive_respond_range`. `Unit::process@00610bc0` clears it
    /// (`& 0xffffffef`) immediately after `process_healing` under
    /// `inside_up < 0`, on every frame and outside the sixteen-frame
    /// cadence the `targeted` decay sits in — so it is a **one-frame**
    /// mark, written inside the frame's order step and read by phase 7's
    /// `Objects::inc_time`.
    ///
    /// What reads it is [`crate::Sim::guy_inc_time`]: a figure whose unit
    /// carries the bit steps its animation clock by **zero**
    /// (`docs/ANIM.md` §5). It is the third value of a step that looks
    /// like it has two.
    pub const NOT_FIRING: u32 = 0x10;
    /// **`0x200` — an American marines' transport** (item 3661,
    /// `docs/TRANSPORT.md` §6): `SpellType::cast_transport@00670db0`'s
    /// `MARINES` arm sets it on the boat it builds for a caster that
    /// `is(0x77, 0)`, and `Unit::update_hits@0060e930` adds
    /// `AMERICANS_MARINE_HP_BONUS` and `Unit::update_speed@006055c0`
    /// `AMERICANS_MARINE_SPEED_BONUS` per Military library level for it.
    pub const MARINES: u32 = 0x200;
    /// **`0x800` — a unit of the Marines line** (item 3661):
    /// `Unit::set_type@00612fa0` sets it on a type that `is(0x77, 0)`, at
    /// birth and on every upgrade, and never clears it. It is what
    /// `Unit::check_idle@006032c0`'s marine trench asks.
    pub const MARINE: u32 = 0x800;
    /// **`0x20000` — the marines' own trench** (item 3661):
    /// `check_idle` sets it beside `unit_masks |= 0x2000000`, and
    /// `move_step` and `close` clear it with the trench
    /// (`& 0xfffdefff`). `fight`, `find_melee_target` and
    /// `find_nearby_target` read a trench that carries it as none
    /// ([`crate::Sim::hard_trench`]); `get_damage`, the two collision
    /// refusals and `Guy::set_anim` read the trench alone.
    pub const MARINE_TRENCH: u32 = 0x20000;
    /// **`0x1000` — Antipater's trench** (item 3919): `cast_entrench`
    /// sets it beside `unit_masks |= 0x2000000` on each unit an Antipater
    /// (0x169) entrenches, and `move_step` clears it with the trench
    /// (`& 0xfffdefff`). SEAM: `get_damage`'s reader of it
    /// (the damage target's `antipater_entrenched`) is not wired.
    pub const ANTIPATER_TRENCH: u32 = 0x1000;
}

/// The `obj_masks` bits by the names the shipped `balance.xml` gives them
/// (§3). Bit `n` is letter `'A' + n`.
pub mod mask {
    pub const ARMORED: u32 = 1 << 0;
    pub const BOMBARD: u32 = 1 << 1;
    pub const CIVILIAN: u32 = 1 << 2;
    pub const MUSKET_INF: u32 = 1 << 3;
    pub const ELEPHANT: u32 = 1 << 4;
    pub const FOOT: u32 = 1 << 5;
    pub const GUN: u32 = 1 << 6;
    pub const HEAVY_INF: u32 = 1 << 7;
    pub const MODERN_INF: u32 = 1 << 8;
    pub const CARRY_AIR: u32 = 1 << 9;
    pub const FOOT_ARCHER: u32 = 1 << 10;
    pub const LARGE: u32 = 1 << 11;
    pub const MOUNTED: u32 = 1 << 12;
    pub const NAVAL: u32 = 1 << 13;
    pub const HORSE_ARCHER: u32 = 1 << 14;
    pub const SPARSE: u32 = 1 << 15;
    pub const LIGHT_INF: u32 = 1 << 16;
    pub const ARCHERY: u32 = 1 << 17;
    pub const SIEGE: u32 = 1 << 18;
    pub const WAR_MACHINE: u32 = 1 << 19;
    pub const ARMORPIERCE: u32 = 1 << 20;
    pub const VEHICLE: u32 = 1 << 21;
    pub const MELEE: u32 = 1 << 22;
    pub const EXPLOSIVE: u32 = 1 << 23;
    pub const HEAVY_CAV: u32 = 1 << 24;
    pub const DETECT: u32 = 1 << 25;
    pub const UNUSED: u32 = 1 << 26;
    pub const MISSILE: u32 = 1 << 27;
    pub const AIR: u32 = 1 << 28;
    pub const LIGHT_CAV: u32 = 1 << 29;
    pub const PIKE: u32 = 1 << 30;
    pub const ANTI_AIR: u32 = 1 << 31;

    /// Parses an `OBJ_MASK` column the way `UnitType::init` does: upper-case
    /// it, letters `A`–`Z` set bits 0–25, digits `1`–`9` set bit `c − 0x17`
    /// (26 up, wrapping at 32 as the original's `1 << (n & 0x1f)` does).
    /// Anything else is skipped.
    pub fn parse(column: &str) -> u32 {
        let mut m = 0u32;
        for c in column.chars() {
            let c = c.to_ascii_uppercase();
            let bit = match c {
                'A'..='Z' => c as u32 - 'A' as u32,
                '1'..='9' => c as u32 - 0x17,
                _ => continue,
            };
            m |= 1 << (bit & 0x1f);
        }
        m
    }
}

/// The rules name a handful of types by name (`is(CATAPULT)`, "made at a
/// `STABLE`", …). Per `docs/DECISIONS.md` entry 18 a type declares the roles
/// it plays rather than the rule testing a type id; a profile without the
/// role leaves the rule inert.
///
/// Not [`crate::combat::balance::line`], whose nine shared words (`CITIZEN`,
/// `MILITIA`, `MERCHANT`, `FLAMETHROWER`, `BARK`, `TRIREME`, `V2ROCKET`,
/// `CITY`, `FORT`) are filled by other predicates — a type here by its
/// name, a line there by its lineage root — and read at another time; nor
/// `crate::ai::ai_load::role`, the original's role word. Each member is `IS_`
/// so the three cannot be confused by name (item 4013).
pub mod role {
    /// `PEASANTS`/`PEASANTSKOREAN`/`SCHOLARS`/`SCHOLARSKOREAN` — citizens,
    /// scholars: half against buildings outside their borders.
    pub const IS_CITIZEN: u32 = 1 << 0;
    /// `is(MILITIA)`.
    pub const IS_MILITIA: u32 = 1 << 1;
    /// `CASTLE`/`FORTX`: a fort whose arrows turn to bullets with Gunpowder.
    pub const IS_FORT: u32 = 1 << 2;
    /// A city building (object flag `0x20`).
    pub const IS_CITY: u32 = 1 << 3;
    /// `is(V2ROCKET)`.
    pub const IS_V2ROCKET: u32 = 1 << 4;
    /// `is(REDFORT, strict)`.
    pub const IS_REDFORT: u32 = 1 << 5;
    /// The type's `where` is the Factory (Wellington's bonus).
    pub const IS_FACTORY_MADE: u32 = 1 << 6;
    /// The type's `where` is the Barracks (the Japanese bonus).
    pub const IS_BARRACKS_MADE: u32 = 1 << 7;
    /// The type's `where` is the Stable (Cossacks).
    pub const IS_STABLE_MADE: u32 = 1 << 8;
    /// `is(BARK)`: quarter splash.
    pub const IS_BARK: u32 = 1 << 9;
    /// `is(CATAPULT)`: halves overkill from a non-siege attacker.
    pub const IS_CATAPULT: u32 = 1 << 10;
    /// `is(FLAMETHROWER)`: ignores entrenchment, ejects garrisons.
    pub const IS_FLAMETHROWER: u32 = 1 << 11;
    /// The `SUPERCOLLIDER` wonder.
    pub const IS_SUPERCOLLIDER: u32 = 1 << 12;
    /// `is(BOMBARD)`: the artillery reload class.
    pub const IS_BOMBARD: u32 = 1 << 13;
    /// Merchant, Dutch merchant, fur trapper.
    pub const IS_MERCHANT: u32 = 1 << 14;
    /// A supply unit (`UnitData::is_supply`).
    pub const IS_SUPPLY: u32 = 1 << 15;
    /// A caravan (`UnitData::is_caravan`).
    pub const IS_CARAVAN: u32 = 1 << 16;
    /// `is(IMMORTALS)`: a melee type whose special strike lands inline.
    pub const IS_IMMORTALS: u32 = 1 << 17;
    /// A `GUN`-flagged ship that does not broadside — `PATROLBOAT`.
    pub const IS_PATROLBOAT: u32 = 1 << 18;
    /// `is(TRIREME)` (`TypeIndex` 0x154: Trireme → Galley → Carrack …):
    /// a Bark's target search takes a four-hundredth of one
    /// (`Object::compare_target@0064e5c0`'s sea arm, `0064f09b`).
    pub const IS_TRIREME: u32 = 1 << 19;
}

/// The combat columns of a type and the derived bits the formula reads (§2.1).
///
/// `attack` is **in tenths**, as `UnitType::init` stores it; `max_range` is in
/// tiles; `target_size` and `guy_radius` in position units. `siege` is
/// `UnitTypeData::is_siege` — the `FLAGS` letter `r`, not the SIEGE object
/// mask. `packs` is `unit_flags2 & 4`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    pub attack: i32,
    pub to_hit: i32,
    pub attenuate: i32,
    pub min_range: i32,
    pub max_range: i32,
    /// A cavalry archer's ranged mode; zero otherwise.
    pub second_max_range: i32,
    pub splash_area: i32,
    pub splash_percent: i32,
    pub ammo_per_att: i32,
    pub recharge: i32,
    pub armor: i32,
    pub proj_speed: i32,
    pub obj_masks: u32,
    pub roles: u32,
    pub uber_size: i32,
    /// `+0x30c crew_size`, the `CREW_SIZE` column — the figures a unit has
    /// **beyond** the one its `squad_size` gives it. `UnitType::init@0061ab50`
    /// stores `squad_size` as the literal 1 and reads `UBER_SIZE` and
    /// `CREW_SIZE` from the record; `Unit::init@00612100:471`-`508` sizes the
    /// guy stack to `crew_size + squad_size` and gives every slot a
    /// `Guy::init_real`. `docs/ANIM.md` section 3.5.
    pub crew_size: i32,
    pub target_size: i32,
    pub guy_radius: i32,
    pub domain: Domain,
    /// `ObjectTypeData +0x250 fly_high` and `+0x254 fly_low`: the
    /// `FLY_HIGH` and `FLY_LOW` columns, read by `UnitType::init` and
    /// `BuildType::init` as `get_text_num(…, -1)`, so a written `25%` is 25
    /// and a missing column is −1. Nothing writes either afterwards. The
    /// one reader this crate has is `valid_target`'s air ladder, which asks
    /// only whether each is zero (`docs/COMBAT.md` §61).
    pub fly_high: i32,
    pub fly_low: i32,
    pub siege: bool,
    pub packs: bool,
    /// The type's age, for the combat table's age bonus.
    pub age: i32,
    /// A building type's footprint, in tiles.
    ///
    /// SEAM: this is `ObjectTypeData::x_size` (`+0x234`) for a **building**
    /// only. A *unit* type's `+0x234` is the XML's `CIRCLE_RADIUS`
    /// (`UnitType::init@0061ab50:655`-`662`, `min(col, 10)` written to
    /// `+0x234` and `+0x238` together), which this crate carries in
    /// [`Profile::circle_radius`] instead so that the several unit paths
    /// reading a *target's* `x_size` keep the 0 they have always read.
    /// `docs/VISION.md` §7 names what that is worth checking against.
    pub x_size: i32,
    pub y_size: i32,
    /// **`ObjectTypeData::x_size` for a unit type** — the XML's
    /// `CIRCLE_RADIUS`, clamped to 10.
    ///
    /// `Unit::update_local_seen@0060e410` is the one consumer this crate
    /// has: it is the ring index of the disc a unit lights into the fog of
    /// everyone it has attacked (`docs/VISION.md` §7).
    pub circle_radius: i32,
    /// `+0x228 x_spacing` and `+0x22c y_spacing`: how far apart two of this
    /// type stand in a formation, across and back, in position units.
    /// `UnitType::init@0061ab50:646`–`654` stores each as the `X_SPACING` /
    /// `Y_SPACING` column **times `UNIT_FORMATION_SPACING`** (12), and
    /// `ObjectType::log_data` dumps them under exactly those names. Read by
    /// `Form::categorize` and nothing else — `docs/GROUPS.md` §6.4.
    pub x_spacing: i32,
    pub y_spacing: i32,
    /// `+0x224 guy_spacing`: how far apart two **figures of one unit** stand,
    /// the `GUY_SPACING` column times `UNIT_GUY_SPACING` (also 12). Read only
    /// by `Form::compute_dests`' follower arm, which places a non-captain
    /// beside the last captain at `± guy_spacing` alternating
    /// (`docs/GROUPS.md` §6.4) — a branch no dump reached until run31, when a
    /// human's selection put every figure in the group.
    pub guy_spacing: i32,
    /// A building type's `BASE_ARROWS` and `MOST_SHOTS`.
    pub base_arrows: i32,
    pub most_shots: i32,
    /// `BLOCK_RADIUS × UNIT_BLOCK_RADIUS` and `big_radius`, position units.
    pub block_radius: i32,
    pub big_radius: i32,
    /// `UnitTypeData +0x2f8 push_size` and `+0x2fc push_circles`, by the
    /// type record: a pushing unit's collision profile, `push_circles`
    /// circles of radius `push_size / push_circles` strung along its
    /// facing (`Unit::detect_boat_collision`, `docs/COLLISION.md` §13).
    /// Position units; zero for a building.
    pub push_size: i32,
    pub push_circles: i32,
    /// `role & 0x10000` — a combat unit, for the target ranking and `on_duty`.
    pub combat_role: bool,
    /// `is(HOPLITES)` — the heavy-infantry lineage, precomputed the way
    /// [`crate::UnitType::archer`] is. One reader:
    /// `Unit::find_open_slots@00600e30` spreads its ring by four
    /// quarter-tiles for a type in this line and by one for every other
    /// (`600e94`'s devirtualised `ObjectData::is(0x84, 0)`).
    pub hoplites: bool,
    /// `Type::sum_rules_cost` — what the type costs, summed, for the target
    /// ranking (§12.3).
    pub cost: i32,
    /// A building type's class for the ranking: city, defensive, military
    /// trainer, training, other.
    pub build_class: BuildClass,
}

/// A building's class as `compare_target` sees it (§12.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildClass {
    #[default]
    Other,
    City,
    /// The Tower/Fort/Airbase lines, or `most_shots != 0`.
    Defensive,
    MilitaryTrainer,
    Training,
}

/// An object that can attack or be attacked: a unit or a building, by index
/// into [`crate::Sim::units`] / [`crate::Sim::buildings`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Obj {
    Unit(usize),
    Building(usize),
}

/// `CombatStanceIndex`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stance {
    #[default]
    Aggressive,
    Defensive,
    StandGround,
    Raid,
    Raze,
    HoldFire,
}

/// A unit's combat state — the fields of `UnitData` this mechanic keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// `UnitData::recharging`: frames until it may attack again.
    pub recharging: u8,
    /// The current attack target, if any.
    pub target: Option<Obj>,
    /// The order is mandatory (given by the player); a found target is not.
    pub mandatory: bool,
    pub stance: Stance,
    /// `unit_masks & 0x80000`.
    pub packed: bool,
    pub entrenched: bool,
    /// `UnitData +0x5c`, the trench's facing: `check_idle`'s marine arm
    /// copies the unit's angle (`+0x50`) into it as it digs in, its fourth
    /// idle count turns the unit back to it, and `get_damage`'s
    /// entrenchment step measures the blow against it (item 3661).
    pub trench_angle: crate::movement::Angle,
    /// The overkill record (§7.1 step 2): the frame of the first hit in the
    /// current window and the captain index of whoever struck it.
    pub damage_frame: i64,
    pub damage_o: i32,
    /// `ObjectData::damage_who` — the **owner** of whoever struck, which
    /// `do_damage` step 2 writes beside `damage_o` and the dump prints
    /// beside it. This crate kept only the captain until item 485 put the
    /// record in `rondata::diff::compare` (`docs/COMBAT.md` §41).
    pub damage_who: i32,
    /// `ObjectData::targeted`: how many attackers have picked this object.
    pub targeted: i32,
    /// The captain's unit index — its own for a lone figure or the captain.
    pub captain: i32,
}

/// A projectile in flight — the fields of `AmmoData` that decide where and
/// when it lands and what it does then (§9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projectile {
    pub shooter: Obj,
    pub owner: crate::Player,
    /// The intended target; forgotten if it dies in flight.
    pub target: Option<Obj>,
    pub launch: Pos,
    /// Where it comes down.
    pub landing: Pos,
    /// `cur_time` and `total_time`.
    pub cur_time: i32,
    pub total_time: i32,
    pub accuracy: i32,
    pub angle: Angle,
    pub splash_area: i32,
    /// The shooting unit's figure count (`num_guys`); zero for a building.
    pub num_guys: i32,
    /// The ammo's domain class — air or not.
    pub air: bool,
    /// `Ammo` flag **4**, `Ammo::init@0067bbf0+0x96f`: the shot **rolls
    /// on** when it finds nothing where it lands, instead of puncturing
    /// the ground there (`docs/COMBAT.md` §42.2). Set when the shooter's
    /// order is not `ATTACK_GROUND`/`AIR_ATTACK_GROUND`, the target is a
    /// unit of land domain, and the piece is not lofted.
    pub rolling: bool,
    /// `Ammo` flag **8**: a rolling shot that has already found nothing
    /// and is travelling on past its landing point. Set once, by
    /// `Ammo::inc_time`, and it is what stops `hit_target`/`check_hit`
    /// being tried a second time.
    pub missed: bool,
    /// `Ammo` flag **0x10**, `Ammo::init@0067bbf0:151`: the round's ammo
    /// piece carries `ammo_flags & 0x80`, the `do_damage="0"` of
    /// `effects_graphics.xml`. It flies and is stepped like any other, and
    /// at its end `Ammo::inc_time@0067d380:260` closes it instead of
    /// calling `Ammo::do_damage` — no damage, no splash, no puncture draw
    /// (`docs/ORDERS.md` §39.5). SEAM: `init:226`, the flag's other
    /// writer (a roll against the target type's `+0x250`/`+0x254`), is
    /// not built.
    pub harmless: bool,
    /// The round's ammo piece carries `ammo_flags & 0x10`, the `flame="1"`
    /// of `effects_graphics.xml` (`GraphicPieces::init_ammo_piece_ranges@008f6140`'s
    /// fifth attribute, `internal_strings.xml` 3809) — the Fire Lances', the
    /// Flaming Arrows' and the Flamethrower's. `Object::do_damage@0064a480`
    /// tests it with `verify_ammo_flags(…, 0x10)` on every blow a round
    /// delivers and overrides `dtype` to **4**, which `Unit::close` reads
    /// as a burning death: three draws, not one (`docs/COMBAT.md` §7.1
    /// step 6, §42.1; item 3580, `docs/AI.md` §313).
    pub flame: bool,
    /// The round's ammo piece carries `ammo_flags & 1`, the `puncture="1"`
    /// of `effects_graphics.xml` (`init_ammo_piece_ranges`' first
    /// attribute, `internal_strings.xml` 3794): the arrows, the flaming
    /// arrows and the Spear. `Ammo::do_damage@00678060` asks it with
    /// `verify_ammo_flags(…, 1)` on a round that struck nothing, and such a
    /// round **sticks** ([`Self::spent`]) where the others close.
    pub punctures: bool,
    /// **A spent arrow** (`flags 1`, item 3679, `docs/COMBAT.md` §46.7): a
    /// puncturing round that struck nothing on ground that is not ocean,
    /// left by `do_damage` in its slot with `cur_time` reset.
    /// `Ammo::inc_time@0067d380` ages it a frame at a time, bumps its dead
    /// shooter's `hold_frames` as for any round, and closes it at 200 —
    /// so the shooter's number stays held that long.
    pub spent: bool,
    /// `AmmoData::sz` and `ez` — the height the shot leaves from and the
    /// one it is aimed to come down at (`docs/COMBAT.md` §46.1). Only a
    /// rolled shot reads them, through [`arc_z`].
    pub sz: i32,
    pub ez: i32,
    /// `AmmoData::v1z` (`+0x54`), the arc's vertical speed, which
    /// `Ammo::init` fixes once from `sz`, `ez` and the flight time
    /// ([`arc_v1z`]).
    pub v1z: Single,
    /// The shot's slot in the ammo pool — `AmmoData::index`, the lowest
    /// free one when it was fired (`Objects::add_ammo@00658b10`), which is
    /// the order `Objects::inc_time@0065db70` steps the pool in
    /// (`docs/COMBAT.md` §46.4). [`crate::Sim::add_ammo`] assigns it.
    pub slot: u32,
    /// What `Ammo::init_crash@0067b800` adds to a round: an aircraft shot
    /// down falling, not a shot fired (`docs/COMBAT.md` §81.3, `docs/AI.md`
    /// §228). `None` on every round `Ammo::init` lays.
    pub crash: Option<Crash>,
    /// A guided round's path (`Ammo +0x68`, item 3802, `docs/GOLDEN.md`
    /// §152.2): the spline `Ammo::init` lays for a missile with a target,
    /// which `Ammo::inc_time` steps, closes on and re-plans. `None` on a
    /// round flown as an arc — and on the V2's and the nuke's, whose
    /// spline holds no target, so `inc_time`'s spline arm only counts.
    pub spline: Option<Spline>,
}

/// **A falling aircraft's round** — the fields `Ammo::init_crash@0067b800`
/// writes that `Ammo::init` does not (`docs/AI.md` §228).
///
/// The round's `who`/`o` are the plane's own, so `Ammo::inc_time`'s first
/// statement holds the dead plane's object number for as long as it falls,
/// as any round holds a dead shooter's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crash {
    /// `+0x5c`/`+0x60`, `bank_dx`/`bank_dy`: two `Random::get(-30, 30)` on
    /// a local `Random` seeded by the figure's `x + y + z`, added to the
    /// landing point on every frame of the fall (`inc_time`'s `+0x50 == 1`
    /// arm). Integers in the original's singles, printed `23.000000`.
    pub bank: (i32, i32),
    /// `+0x5`, `rolling`: `r % 7 − 3` of the crash's one draw from the
    /// game's stream.
    pub rolling: i32,
}

/// `GRAV_Z` at `0xcab378`: `0xc127cccd`, −10.4875 — written once, by
/// `GraphicPieces::init@008ffcc0`, before the first frame, and read by
/// `Ammo::init` and `Ammo::inc_time` alone (§46.1).
pub const GRAV_Z: Single = Single::from_bits(0xc127_cccd);
/// The `0.5` at `0xb694c0` that both multiply `GRAV_Z` by first.
const HALF: Single = Single::from_bits(0x3f00_0000);

/// `Ammo::init@0067bbf0`, `0x67cf2a`–`0x67cf7d`: `v1z = ((float)(ez − sz) − GRAV_Z × 0.5
/// × T × T) / T`, each step an SSE single in the listing's order — the
/// integer difference converted, the gravity term multiplied left to
/// right, one subtract, one divide.
pub fn arc_v1z(sz: i32, ez: i32, total_time: i32) -> Single {
    // The listing converts `total_time` unsigned (`0x67cf..`'s `+ 2^32`
    // fix-up on a set sign bit), so a fall of `0x80000000` is 2^31 here:
    // run1468's `v1z 11260867584` (item 3836).
    let t = Single::from_u32(total_time as u32);
    let g = GRAV_Z.mulss(HALF).mulss(t).mulss(t);
    Single::from_i32(ez - sz).subss(g).divss(t)
}

/// `Ammo::inc_time@0067d380`'s rolling arm, `0x67d922`–`0x67d996`: the
/// shot's height at `cur_time`, `(v1z × t + (float)sz) + GRAV_Z × 0.5 ×
/// t × t`.
pub fn arc_z(v1z: Single, sz: i32, cur_time: i32) -> Single {
    let t = Single::from_i32(cur_time);
    let g = GRAV_Z.mulss(HALF).mulss(t).mulss(t);
    v1z.mulss(t).addss(Single::from_i32(sz)).addss(g)
}

/// The same arm's point on the line past the landing: `frac = t / T`, and
/// each axis `(int)((float)(e − s) × frac + (float)s)`, truncated.
pub fn arc_point(launch: Pos, landing: Pos, cur_time: i32, total_time: i32) -> Pos {
    let frac = Single::from_i32(cur_time).divss(Single::from_i32(total_time));
    let axis = |s: i32, e: i32| {
        Single::from_i32(e - s)
            .mulss(frac)
            .addss(Single::from_i32(s))
            .to_i32()
    };
    Pos::new(axis(launch.x, landing.x), axis(launch.y, landing.y))
}

/// **One death object** — the `DEATH_OBJS` record `Unit::close` makes for
/// a unit that dies with a non-zero `dtype` (`docs/COMBAT.md` §42.1).
///
/// The dump prints six keys at detail 1 and this carries five of them:
/// `valid` is 1 for every record it prints at all, `first_frame` the sim
/// frame of the death, `cur_anim` the animation the draw chose, and
/// `who`/`o` the dead figure. `gpiece` is the sixth — the unit's own death
/// piece, `DeathObj::init`'s `vtable[0x178](param_7, packed)` — and this
/// crate loads no piece for it, so it is parsed and not compared (§42.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Death {
    pub who: i32,
    pub o: i32,
    pub first_frame: i64,
    pub cur_anim: i32,
    /// `DeathObj::inc_time@008d5240`'s end: the frames the record lives,
    /// `get_game_frames(cur_anim)` of the death piece plus the fade
    /// (`docs/GOLDEN.md` §127). `None` where the install does not describe
    /// the piece — the record is then never culled, as before.
    pub life: Option<i64>,
    /// **One record per figure** (item 3784, `docs/GOLDEN.md` §149.10):
    /// `Objects::kill_guy` makes a `DeathObj` for each guy, each on its own
    /// piece and its own life; the unit's number is culled with the last,
    /// which is [`Self::life`], but every record's `inc_time` bumps the
    /// number's `hold_frames` while it lives.
    pub figures: Figures,
}

/// **`Objects`' death pool's size** — `Objects::init@0065ea80:384` writes
/// 100 to the `ObjectArray<DeathObj>`'s count (`+0x140`) and nothing grows
/// it: `Objects::add_death@00653b60` walks those 100 slots and no more
/// (item 3971, `docs/GOLDEN.md` §177).
pub const DEATH_POOL: usize = 100;

/// **One slot of the death pool** — which figure's record it holds: the
/// dead unit's number, its death frame (`DeathObj +0x4`, the age
/// `add_death` evicts by) and the figure, an index into that
/// [`Death`]'s [`Figures`]. A slot is free when its record has ended —
/// the [`Death`] culled, or that figure's life run out — which is
/// `DeathObj +0x0` (`valid`) zeroed by `DeathObj::inc_time`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeathSlot {
    pub who: i32,
    pub o: i32,
    pub first_frame: i64,
    pub figure: u8,
}

/// The lives of a dead unit's `DeathObj` records, one per guy, in frames;
/// `i64::MAX` for a record whose piece the install does not describe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Figures {
    pub n: u8,
    pub life: [i64; Figures::MAX],
}

impl Figures {
    /// More guys than any shipped crew has; a unit past it keeps the first.
    pub const MAX: usize = 8;

    /// A unit of one record.
    pub const fn one(life: Option<i64>) -> Self {
        let mut f = Self {
            n: 1,
            life: [0; Self::MAX],
        };
        f.life[0] = match life {
            Some(l) => l,
            None => i64::MAX,
        };
        f
    }

    /// The records still in the list at `age` frames past the death: each
    /// is zeroed on the call its counter reaches its life, so it bumps on
    /// every frame before.
    pub fn live(&self, age: i64) -> i32 {
        let n = usize::from(self.n).min(Self::MAX);
        self.life[..n].iter().filter(|&&l| age < l).count() as i32
    }
}

/// One delivery of damage, for tests and logs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub frame: i64,
    pub attacker: Obj,
    pub target: Obj,
    /// The net of `get_damage`, before scaling.
    pub damage: i32,
    pub dealt: Sixteenths,
    pub splash: bool,
    pub killed: bool,
}

/// `ObjectData::attack_dist` (§13.1): per-axis edge-to-edge on the
/// quarter-tile grid, then the integer hypotenuse. `a_ext`/`t_ext` are each
/// side's `(ex, ey)` extent; `plane` is "a plane that is not a missile".
pub const fn attack_dist(
    at: Pos,
    target: Pos,
    a_ext: (i32, i32),
    t_ext: (i32, i32),
    plane: bool,
) -> i32 {
    let mut dx = (snap(at.x) - snap(target.x)).abs();
    let mut dy = (snap(at.y) - snap(target.y)).abs();
    if plane {
        return vector_dist(dx, dy);
    }
    dx = if dx > t_ext.0 { dx - t_ext.0 } else { 0 };
    dy = if dy > t_ext.1 { dy - t_ext.1 } else { 0 };
    dx = if dx > a_ext.0 { dx - a_ext.0 } else { 0 };
    dy = if dy > a_ext.1 { dy - a_ext.1 } else { 0 };
    vector_dist(dx, dy)
}

/// The quarter-tile grid `attack_dist` works on: `div_3_table[v >> 4] * 0x30`.
pub const fn snap(v: i32) -> i32 {
    v.div_euclid(48) * 48
}

/// The extent `attack_dist` subtracts for one object: half the footprint for a
/// building, `block_radius + 0x18` on both axes for a unit.
pub const fn extent(p: &Profile, building: bool) -> (i32, i32) {
    if building {
        (p.x_size * 0x60, p.y_size * 0x60)
    } else {
        (p.block_radius + 0x18, p.block_radius + 0x18)
    }
}

/// `ObjectData::is_in_range`'s distance test (§13.2), given `d =
/// attack_dist`: melee within `0x66` (`0xf6` for the Hoplites line), ranged
/// within `[min × 0xc0 − 6, max × 0xc0 + 6]` with the big radii rescuing the
/// inner bound.
pub const fn in_range(
    d: i32,
    max_range: i32,
    min_range: i32,
    hoplites: bool,
    big_radii: i32,
    melee_bonus: bool,
) -> bool {
    if max_range == 0 {
        return d <= if hoplites { 0xf6 } else { 0x66 };
    }
    if d < min_range * 0xc0 - 6 && d + big_radii < min_range * 0xc0 - 6 {
        return false;
    }
    d + if melee_bonus { 0x90 } else { 0 } <= max_range * 0xc0 + 6
}

/// `flanking@0092cfe0` — how far round the target's back the attacker
/// stands, from the **bias** its two callers hand it:
/// `e = target.angle − reference + 0x80000000`, unsigned.
///
/// Zero is "not flanking at all", and it is the only answer two of its
/// callers look at. Both of them (`do_move@005f7fbe` and
/// `check_target_path@005e2493`) test the window's negative half inline —
/// `e < 0x2aaaaaaa` jumps straight past the call — so the predicate a
/// caller means is `0x2aaaaaaa <= e && flanking(e) != 0`, which is
/// `|signed angle difference| <= 120°`. What differs between them is the
/// `reference`: `do_move` uses the **attacker's own heading**,
/// `check_target_path` the **bearing from the attacker to the target**.
///
/// The 1/2 split is `e − 0x60000000 > 0x40000000`, and the third caller
/// reads it: `Unit::find_attack_pos@00601280` leads a running target only
/// on `flanking(e) == 1` (`6028fa`), where `e`'s reference is again the
/// bearing to the target (`Sim::chase_lead`, `docs/GOLDEN.md`
/// §150.3; COMBAT §36.3 once said nothing read it).
pub const fn flanking(e: u32) -> i32 {
    if e > 0xd555_5555 {
        return 0;
    }
    if e.wrapping_sub(0x6000_0000) > 0x4000_0000 {
        2
    } else {
        1
    }
}

/// The side of the square of cells a splash searches (§9.3): the spiral
/// table `move_x/move_y` walked to `radius[k]`, `k = splash_area / 4 + 1`
/// capped at 10, which visits the `(2k+1)²` cells within Chebyshev distance
/// `k` of the landing cell.
pub const fn splash_cells(splash_area: i32) -> i32 {
    let k = splash_area / 4 + 1;
    if k > 10 { 10 } else { k }
}

/// **The one cell past the square** (item 3636, parked 3356c): the walk is
/// `while i <= radius[k]` (`678b49`..`678b50`, a `jle`), and `radius[k]` is
/// `(2k+1)²`, so it reads one spiral entry more than the square holds —
/// the first cell of ring `k + 1`, `(−1, −2)` for `k = 1` and
/// `(−(k+1), −(k+1))` beyond. An offset from the landing cell. The shipped
/// table reaches entry 288 in this crate ([`crate::world::MOVE_289`]); a
/// splash of `k >= 8` (no shipped type has one) answers `None`.
pub const fn splash_extra_cell(k: i32) -> Option<(i32, i32)> {
    let i = ((2 * k + 1) * (2 * k + 1)) as usize;
    if i < crate::world::MOVE_289.len() {
        Some(crate::world::MOVE_289[i])
    } else {
        None
    }
}

/// The ring a cell offset belongs to in `circle_x/circle_y` — the original's
/// own integer distance, `max + min² / (2·max)` — and the order within it,
/// `dx` ascending then `dy`.
pub const fn ring_of(dx: i32, dy: i32) -> i32 {
    let (ax, ay) = (dx.abs(), dy.abs());
    let (hi, lo) = if ax >= ay { (ax, ay) } else { (ay, ax) };
    if hi == 0 {
        0
    } else {
        hi + (lo * lo) / (2 * hi)
    }
}

impl Profile {
    pub const fn has(&self, m: u32) -> bool {
        self.obj_masks & m != 0
    }
    pub const fn is(&self, r: u32) -> bool {
        self.roles & r != 0
    }
    /// Whether this type fires anything — `fire_proj`, derived at load as
    /// "has non-zero attack" (§2.1).
    pub const fn fires(&self) -> bool {
        self.attack != 0
    }
    pub const fn is_ranged(&self) -> bool {
        self.max_range != 0
    }
}

/// A type in the combat table: the simulation's unit ids and building ids
/// are two families, as the original's `TypeIndex` ranges are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeRef {
    Unit(usize),
    Build(usize),
}

/// The combat table (§5): percent of attack, by attacker type against target
/// type, over the simulation's own type ids — units first, then buildings,
/// the way the original lays `final_balance_table` out over
/// `BASE_UNITTYPES..END_BUILDTYPES`.
///
/// An entry the table does not hold is 100, which is what the original holds
/// for any pair it has no rule for and exactly what an empty table is. The
/// unit ids are [`crate::Sim::unit_types`] indices and the building ids
/// [`crate::Sim::build_types`] indices; [`Table::pct`] is the unit-versus-unit
/// lookup and [`Table::pct_of`] takes either family on either side.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    units: usize,
    builds: usize,
    cells: Vec<i16>,
}

impl Table {
    /// A table over `n` unit types and no buildings, all 100.
    pub fn uniform(n: usize) -> Table {
        Table::uniform_of(n, 0)
    }

    /// A table over `units` unit types and `builds` building types, all 100.
    pub fn uniform_of(units: usize, builds: usize) -> Table {
        let side = units + builds;
        Table {
            units,
            builds,
            cells: vec![100; side * side],
        }
    }

    /// Builds a table from a closure over `(attacker, target)`, units first
    /// then buildings on both axes.
    pub fn build(units: usize, builds: usize, mut f: impl FnMut(TypeRef, TypeRef) -> i32) -> Table {
        let side = units + builds;
        let at = |i: usize| {
            if i < units {
                TypeRef::Unit(i)
            } else {
                TypeRef::Build(i - units)
            }
        };
        let mut cells = Vec::with_capacity(side * side);
        for a in 0..side {
            for b in 0..side {
                cells.push(f(at(a), at(b)) as i16);
            }
        }
        Table {
            units,
            builds,
            cells,
        }
    }

    /// The number of unit types the table covers.
    pub fn width(&self) -> usize {
        self.units
    }

    /// The number of building types the table covers.
    pub fn builds(&self) -> usize {
        self.builds
    }

    fn index(&self, t: TypeRef) -> Option<usize> {
        match t {
            TypeRef::Unit(i) if i < self.units => Some(i),
            TypeRef::Build(i) if i < self.builds => Some(self.units + i),
            _ => None,
        }
    }

    /// `return_modifier(a, b)` for two unit types.
    pub fn pct(&self, a: usize, b: usize) -> i32 {
        self.pct_of(TypeRef::Unit(a), TypeRef::Unit(b))
    }

    /// `return_modifier(a, b)` for any two types; 100 for a type the table
    /// does not cover.
    pub fn pct_of(&self, a: TypeRef, b: TypeRef) -> i32 {
        match (self.index(a), self.index(b)) {
            (Some(x), Some(y)) => i32::from(self.cells[x * (self.units + self.builds) + y]),
            _ => 100,
        }
    }

    pub fn set(&mut self, a: usize, b: usize, pct: i32) {
        self.set_of(TypeRef::Unit(a), TypeRef::Unit(b), pct);
    }

    pub fn set_of(&mut self, a: TypeRef, b: TypeRef, pct: i32) {
        if let (Some(x), Some(y)) = (self.index(a), self.index(b)) {
            let side = self.units + self.builds;
            self.cells[x * side + y] = pct as i16;
        }
    }

    /// The same table over a larger type space — new rows and columns at
    /// 100, every existing entry kept.
    pub fn grown(&self, units: usize, builds: usize) -> Table {
        let units = units.max(self.units);
        let builds = builds.max(self.builds);
        let mut t = Table::uniform_of(units, builds);
        let at = |i: usize, n: usize| {
            if i < n {
                TypeRef::Unit(i)
            } else {
                TypeRef::Build(i - n)
            }
        };
        let side = self.units + self.builds;
        for a in 0..side {
            for b in 0..side {
                let v = i32::from(self.cells[a * side + b]);
                t.set_of(at(a, self.units), at(b, self.units), v);
            }
        }
        t
    }
}

/// The nation, wonder, general and patriot layer of the damage formula, as
/// inputs (§4, §6 steps 12, 13, 25). Each is `false`/zero until the layer
/// that produces it exists, which is also the stock game with no such bonus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// The attacker is under Wellington's aura and its owner has him:
    /// `+ wellington_siege_attack` against factory-made targets.
    pub wellington: bool,
    /// The Japanese barracks bonus: `Some(min(ages, military level))` for a
    /// Japanese attacker; the bonus applies to barracks-made units.
    pub japanese_level: Option<i32>,
    /// The Russian cossack bonus: the attacker's owner is Russian.
    pub russian: bool,
    /// Whether the attacker's owner has reached the Gunpowder age (§6 step 1).
    pub gunpowder: bool,
    /// The flat additions of §4.1, §4.2 and §4.4 for this player's units:
    /// attack in tenths, armour, range in tiles.
    pub attack: i32,
    pub armor: i32,
    pub range: i32,
}

/// What the formula needs to know about one side that is not on its
/// [`Profile`] — the object's state at the moment of the attack (§6).
///
/// Everything defaults to "not the case", so a test or a caller states only
/// what holds. `z` is the object's height, which the original keeps on the
/// object's z coordinate; `facing` and `trench_facing` are the angles the
/// flank and entrenchment tests read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Side {
    pub unit: bool,
    /// A `Build` rather than a `Wall` — the `+0x20` virtual.
    pub build_proper: bool,
    /// A building or a wall — the `+0x1c` virtual.
    pub building: bool,
    pub packed: bool,
    pub moving: bool,
    /// Has a non-zero `attack()` (the target side of step 5).
    pub attacks: bool,
    /// A building not yet `WallData::is_active`.
    pub under_construction: bool,
    /// The target's tile is owned by the attacker (step 7).
    pub tile_owned_by_attacker: bool,
    pub decoy: bool,
    /// `WallData::in_unfriendly_territory` (step 18).
    pub in_unfriendly_territory: bool,
    pub entrenched: bool,
    pub antipater_entrenched: bool,
    /// `unit_masks & 0x10` and `& 0x400000` — read but unnamed (open question 3).
    pub mask_10: bool,
    pub mask_400000: bool,
    /// A sea unit in a team game whose owner is not targeting the attacker.
    pub team_untargeted_ship: bool,
    /// The target's tile has the rocky terrain flag.
    pub rocky: bool,
    pub z: i32,
    pub facing: Angle,
    pub trench_facing: Angle,
    /// The recapture case: a city whose original owner is the attacker.
    pub recapturable_city: bool,
    /// The attacker's squad captain index, and the target's overkill record.
    pub captain: i32,
    pub damage_frame: i64,
    pub damage_o: i32,
}

/// One call of `ObjectData::get_damage`, with the two objects reduced to what
/// it reads (§6). Returns whole hits, possibly zero or negative; `scale`
/// clamps and splits next.
///
/// `attack` and `armor` are the already-modified stats (§4.1, §4.2);
/// `pct` is the combat table entry; `angle` is the attacker's facing toward
/// the target; `splash` says this is a splash fringe; `frame` is the game
/// frame for the overkill window.
///
/// This is the call `Object::do_damage` makes, with `check_overkill` set
/// (`get_damage(…, splash, 1, …)`); [`get_damage_checked`] takes the flag.
#[allow(clippy::too_many_arguments)]
pub fn get_damage(
    t: &Tuning,
    a: &Profile,
    at: Side,
    tp: &Profile,
    tt: Side,
    attack: i32,
    armor: i32,
    pct: i32,
    angle: Angle,
    splash: bool,
    frame: i64,
    m: &Modifiers,
) -> i32 {
    get_damage_checked(
        t, a, at, tp, tt, attack, armor, pct, angle, splash, true, frame, m,
    )
}

/// [`get_damage`] with its fifth argument, `check_overkill`, which gates
/// step 21 and nothing else (`get_damage@00644130:371`, `param_5 != 0`).
/// `Object::do_damage` passes 1. **`Object::compare_target` passes 0**
/// (item 601, `docs/COMBAT.md` §53): the listing pushes `$0x0` three times
/// at `0064ebce`–`0064ebdf` — `param_6`, `param_5` and `param_4` — before
/// the bearing, so a target search ranks a freshly wounded target at its
/// whole damage rather than at the overkill third.
#[allow(clippy::too_many_arguments)]
pub fn get_damage_checked(
    t: &Tuning,
    a: &Profile,
    at: Side,
    tp: &Profile,
    tt: Side,
    attack: i32,
    armor: i32,
    pct: i32,
    angle: Angle,
    splash: bool,
    check_overkill: bool,
    frame: i64,
    m: &Modifiers,
) -> i32 {
    // The twenty-eight steps are `damage::get_damage_steps`, over the view this
    // profile and these two sides make (item 3356).
    let mut view = crate::combat::damage::Reduced {
        t,
        a,
        at,
        tp,
        tt,
        attack,
        armor,
        pct,
        frame,
        m,
    };
    crate::combat::damage::get_damage_steps(&mut view, angle.0, splash, check_overkill, None)
}

/// The flank level of an attack arriving along `angle` at a target facing
/// `facing` (§6 step 19): `None` when the target faces the attacker to within
/// 60°, `Some(1)` when it faces away to within 45°, `Some(2)` in the sectors
/// between.
pub const fn flank_level(facing: Angle, angle: Angle) -> Option<i32> {
    let e = (facing.0.wrapping_sub(angle.0) as u32).wrapping_add(0x8000_0000);
    if e < 0x2aaa_aaaa || e > 0xd555_5555 {
        return None;
    }
    Some(1 + (e.wrapping_add(0xa000_0000) > 0x4000_0000) as i32)
}

/// The compiler's signed `/ 256`: add `0xff` to a negative value, then shift.
pub const fn shr8(v: i32) -> i32 {
    (v + ((v >> 31) & 0xff)) >> 4 >> 4
}

/// The compiler's signed `/ 16`, and the low four bits with the same fix.
const fn div16(v: i32) -> (i32, i32) {
    let q = (v + ((v >> 31) & 0xf)) >> 4;
    let r = v - q * 16;
    (q, r)
}

/// Damage ready to be taken: whole hits and sixteenths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sixteenths {
    pub whole: i32,
    pub frac: i32,
}

/// `Object::do_damage`'s scaling step (§7.1 step 5): `dmg` from
/// [`get_damage`], `count` in 8.8 (`0x100` a full hit), whether the attacker is
/// a unit, whether the hit came from a projectile (`ammo`), and the
/// attacker's `ammo_per_att` and `uber_size`.
pub const fn scale(
    dmg: i32,
    count: i32,
    unit: bool,
    ammo: bool,
    ammo_per_att: i32,
    uber: i32,
) -> Sixteenths {
    let mut s = dmg * count;
    if unit {
        if s < 0x101 {
            s = 0x100;
        }
        if ammo {
            s /= if ammo_per_att < 1 { 1 } else { ammo_per_att };
        }
        s /= if uber < 1 { 1 } else { uber };
    } else {
        s /= if ammo_per_att < 1 { 1 } else { ammo_per_att };
    }
    let (sixteenths, _) = div16(s);
    let (whole, frac) = div16(sixteenths);
    Sixteenths { whole, frac }
}

/// A figure's share of its squad's hit points (§7.2 step 8, §7.3): `hits /
/// uber_size`, and for the captain that is the last one standing the
/// remainder as well.
pub const fn share(hits: i32, uber: i32, captain_alone: bool) -> i32 {
    if uber <= 1 {
        return hits;
    }
    if captain_alone {
        hits - ((uber - 1) * hits) / uber
    } else {
        hits / uber
    }
}

impl Projectile {
    /// **`Ammo::inc_time@0067d380`'s end test is unsigned**: `if ((uint)
    /// total_time <= (uint)cur_time) break` (`uVar6 <= uVar4`), so a fall
    /// of `0x80000000` ([`fall_time`]'s NaN) stays up for 2^31 frames,
    /// bumping its dead shooter's `hold_frames` every one (item 3836).
    pub fn in_flight(&self) -> bool {
        in_flight_at(self.cur_time, self.total_time)
    }

    /// **The bearing a landed shot hits at**: `find_angle(ex − sx, ey −
    /// sy)`, its own landing point less its launch. `docs/COMBAT.md` §47.3.
    ///
    /// `Ammo::do_damage@00678060` passes it to `Object::do_damage` on
    /// both arms, and the listing is what says so: the no-splash arm loads
    /// `ecx = ex − sx` and `edx = ey − sy` at `00678c2e`-`00678c37` and
    /// calls `find_angle@0092d130` fastcall. The splash arm does the same
    /// at `006786a5`-`006786cd`, once for every object it then walks. The
    /// decompiler dropped the register pair and printed the two stack
    /// pushes behind it, `num_guys` (`+0x44`) and `index` (`+0x34`), as
    /// `find_angle`'s arguments. This crate took the target's position at
    /// landing instead, and a target that walked while the shot flew was
    /// struck on the wrong flank sector: run112's `1/7`, on 770, at level
    /// 1 where the original's level is 2.
    ///
    /// `landing` is `ex`/`ey`, which a rolled shot rewrites when it comes
    /// down (§46.1), so the bearing is taken after the roll as the
    /// original takes it.
    pub fn bearing(&self) -> crate::movement::Angle {
        crate::movement::find_angle(
            self.landing.x - self.launch.x,
            self.landing.y - self.launch.y,
        )
    }
}

/// What `Object::take_damage` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taken {
    /// Still standing; the whole points actually deducted are reported.
    Alive { lost: i32 },
    /// Dead; `overflow` is the damage past the share, which a decoy passes to
    /// its captain and everything else discards.
    Died { lost: i32, overflow: i32 },
}

/// `Object::take_damage`'s arithmetic (§7.2 steps 2, 4, 8, 9, 10) on one
/// figure: `damage`/`frac` are its accumulated whole hits and sixteenths,
/// `share` what it can absorb.
pub const fn take(damage: i32, frac: i32, share: i32, hit: Sixteenths) -> (Taken, i32, i32) {
    let mut hit = hit;
    if hit.whole < 1 && hit.frac < 1 {
        hit.frac = 1;
    }
    let acc = frac + hit.frac;
    let (carry, rem) = div16(acc);
    let lost = hit.whole + carry;
    let damage = damage + lost;
    if damage < share {
        (Taken::Alive { lost }, damage, rem)
    } else {
        (
            Taken::Died {
                lost,
                overflow: damage - share,
            },
            damage,
            rem,
        )
    }
}

/// `UnitData::recharge()` (§8.3): the reload delay after an attack, in
/// frames, from the type's `RECHARGE` and the siege/supply state. The supply
/// rule itself is `supply::reload_frames`; this is the combat-side wrapper
/// that adds the Bombard class and the byte truncation `recharging` suffers.
pub const fn recharge(base: i32, out_of_supply_ratio: bool, bombard: bool) -> i32 {
    let r = if !out_of_supply_ratio {
        base
    } else if bombard {
        base * 2
    } else {
        base * 3 / 2
    };
    // `recharging` is a `uchar`.
    r & 0xff
}

/// A projectile's accuracy at launch (§9.1): `to_hit − attenuate × (dist /
/// 192)`, floored at 5 — `attenuate` percent lost per whole tile of
/// `attack_dist`. (An earlier draft had `+ (dist / 96) × attenuate`; the
/// second reading and the disassembly's `/ -192` settled it. `attenuate` is
/// the column's absolute value, §2.1.)
pub const fn accuracy(to_hit: i32, attenuate: i32, dist: i32) -> i32 {
    let acc = to_hit - attenuate * (dist / 0xc0);
    if acc < 5 { 5 } else { acc }
}

/// The scatter radius of a shot (§9.1), in position units: `None` means the
/// shot lands exactly. `land_unit` is "the target is a land-domain unit";
/// `missile` doubles it; `exact` is the `unit_flags & 0x400000` type.
pub const fn scatter(t: &Tuning, acc: i32, land_unit: bool, missile: bool, exact: bool) -> i32 {
    if exact {
        return 0;
    }
    let mut s = if land_unit {
        let s = t.target_radius * 100 / ((100 - acc) / 5 + acc);
        if acc > 100 {
            (s + ((s >> 31) & 3)) >> 2
        } else {
            s
        }
    } else {
        0xc0
    };
    if missile {
        s *= 2;
    }
    s
}

/// **`dtype`** — the death class `ObjectData::get_damage` writes through its
/// out-parameter (§6 step 11) before `Object::do_damage` overrides it
/// (§7.1 step 6). It decides the death animation `Unit::close` rolls
/// (`docs/COMBAT.md` §42.1), and nothing else this crate models.
///
/// `get_damage` opens at 2 and the only writers are three: **EXPLOSIVE**
/// (`obj_masks & 0x800000`) and **BOMBARD** (`& 0x2`) each make it 3, and
/// a **building** attacker overwrites whatever those left — 3 when
/// `BuildData::get_shot() == 4` (the `REDOUBT`), 2 otherwise. So an
/// ordinary arrow from an ordinary archer is 2, which run112's four
/// `DEATH_OBJS` records confirm from the other end: `cur_anim 17` is
/// `2 * dtype + 0xd + roll % 2` with an even roll, and the trace's own
/// seed at `0060fb06` on frame 683 makes that roll 64832.
///
/// **`get_shot` is not modelled**, so a `REDOUBT`'s shot is 2 here and 3
/// in the original; it changes a death animation index and no capture on
/// disk has one.
pub const fn death_type(amask: u32, attacker_is_building: bool) -> i32 {
    if attacker_is_building {
        return 2;
    }
    if amask & (mask::EXPLOSIVE | mask::BOMBARD) != 0 {
        return 3;
    }
    2
}

/// The death animation `Unit::close` picks for a `dtype` (§42.1):
/// `dtype * 2 + 0xd + roll % 2`, or `roll % 2 + 0x11` when `dtype == 4`.
pub const fn death_anim(dtype: i32, roll: i32) -> i32 {
    if dtype == 4 {
        roll % 2 + 0x11
    } else {
        dtype * 2 + 0xd + roll % 2
    }
}

/// Applies the scatter to a landing point, taking the original's two draws
/// (§9.1): `ex += get(0, 0xffff) % s − s / 2`, likewise `ey`. No draw is
/// taken when `s < 2`.
pub fn scatter_point(rng: &mut Rng, at: Pos, s: i32) -> Pos {
    if s - 1 < 1 {
        return at;
    }
    let dx = rng.roll() % s - s / 2;
    let dy = rng.roll() % s - s / 2;
    Pos::new(at.x + dx, at.y + dy)
}

/// `(int)(sqrtf(n) / (float)d)` — a non-siege unit's or a building's flight
/// time (§9.1) — with each IEEE single-precision step rounded exactly in
/// integers. `n` is `dx² + dy²`, `d` is `proj_speed × unit_move_speed`.
///
/// Both operations are correctly rounded in IEEE 754, so the result is a
/// function of `n` and `d` alone; this reproduces it without a float. It is
/// the one place combat touches the argument of `docs/DECISIONS.md` entry 10,
/// and open question 4 in `docs/COMBAT.md` records the residual risk.
pub fn flight_time(n: i64, d: i64) -> i32 {
    if d <= 0 {
        return 0;
    }
    let r = f32_sqrt(n);
    let q = f32_div(r, f32_of_int(d));
    q.trunc_to_int()
}

/// **A missile's flight time** (item 1625, `docs/AI.md` §174): the point
/// count of the spline `Ammo::init@0067bbf0` builds for a round whose
/// ammo carries `missile="1"` (`ammo_flags & 8`, `local_30`), in place
/// of [`flight_time`]'s straight line.
///
/// `Spline::calc_from_dir@00913960` takes three control points — the
/// launch, a point `lead` ahead of it, the landing — and
/// `set_min_seg_length@009125e0` sets the resolution to `(int)(L /
/// speed)`, `L` the control polygon's length and `speed` the
/// `(float)(unit_move_speed × proj_speed)` that `67cd6a`–`67cdcc` load
/// into `xmm1`. `calc_spline@00912f00` caps it at `3 × 4.0 = 12` and
/// raises it to the degree, 2, and `generate_bspline@00911820` emits
/// one point more than the resolution: the round lands on its
/// thirteenth frame however far it flies, once `L` reaches twelve
/// speeds. run717 prints `total_time 13` on all six of the Advanced
/// Battleship `1/49`'s rounds, 5,440 to 6,279 long, where the straight
/// line gave 19.
///
/// `lead` is the middle point's offset: a sea or air shooter's is `5 ×
/// speed` out, turned by the figure's pitch and angle ([`missile_lead`],
/// item 3808; run717's `1/49` puts it 1,500 out at the launch's height).
/// A building's and a land unit's lead (`67c437`–`67c469`, item 3816,
/// `docs/GOLDEN.md` §152.9) is `attack_dist / 2` out at a pitch of 45° on
/// the ammo's angle. `None` takes the control point on the straight line
/// (`L` the plain distance), which no shot in `Ammo::init` does now.
pub fn missile_flight_time(
    launch: Pos,
    sz: i32,
    landing: Pos,
    ez: i32,
    lead: Option<Lead>,
    speed: i64,
) -> i32 {
    i32::try_from(missile_spline(launch, sz, landing, ez, lead, speed).len())
        .expect("at most SPLINE_MAX")
}

/// A missile's lead: the figure a sea or air shooter fires from — its
/// angle (`GuyData +0x18`), its pitch (`+0x4c`, a single) — and the lead's
/// length, `5 × speed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lead {
    pub angle: Angle,
    pub pitch: Single,
    pub r: i32,
}

/// **A missile's spline as `Ammo::init` lays it** (item 3802): the launch
/// `(sx, sy, sz)`, the middle control point, the landing `(ex, ey, ez)`,
/// at `speed` — [`Spline::from_dir`]. The middle point is [`missile_lead`]'s
/// (item 3808); `None` takes it half way along the straight line, which
/// keeps the polygon's length the plain distance.
pub fn missile_spline(
    launch: Pos,
    sz: i32,
    landing: Pos,
    ez: i32,
    lead: Option<Lead>,
    speed: i64,
) -> Spline {
    let v = |x: i32, y: i32, z: i32| {
        [
            Single::from_i32(x),
            Single::from_i32(y),
            Single::from_i32(z),
        ]
    };
    let mid = match lead {
        Some(l) => missile_lead(launch, sz, l),
        None => v(
            launch.x + (landing.x - launch.x) / 2,
            launch.y + (landing.y - launch.y) / 2,
            sz + (ez - sz) / 2,
        ),
    };
    let speed = Single::from_i32(i32::try_from(speed).unwrap_or(i32::MAX));
    Spline::from_dir(
        v(launch.x, launch.y, sz),
        mid,
        v(landing.x, landing.y, ez),
        speed,
    )
}

/// **The middle control point of a sea or air shooter's missile**
/// (`Ammo::init@0067bbf0`, listing `67cb1f`–`67cd00`; item 3808,
/// `docs/GOLDEN.md` §152.5): the vector `(0, −r, 0)`, rotated by two
/// quaternions with [`quat_rotate`] — the figure's pitch about x, then its
/// angle about z — and added to the launch's integers, in singles. It is
/// never truncated: run1471's `2/14` lays (13233.875, 17114.279297, −36)
/// from (12591, 16728, −36).
///
/// - The pitch's index is `cvttss2si(pitch)` folded into `0..360` (`67cbb3`
///   –`67cc05`: a negative `p` reads `359 − (−p mod 360)`).
/// - The angle's is `fast_angle_to_degrees` ([`crate::data::pivot::fast_degrees`],
///   the top byte's whole degree) **negated** and folded the same way
///   (`67cc4d`–`67cc90`), so a degree `d > 0` reads `359 − d`, one more than
///   the turn back: a figure parked at 120° leads at 121°.
/// - Each quaternion is `(s·k, s·k, s, c)` with the axis's slot `s`, `k`
///   the `0.0` at `0xb693fc`, and `(c, s)` [`HALF_DEGREE_COS`] and
///   [`HALF_DEGREE_SIN`] at the index.
pub fn missile_lead(launch: Pos, sz: i32, lead: Lead) -> [Single; 3] {
    let fold = |n: i32| {
        if n < 0 {
            (359 - n.unsigned_abs() % 360) as usize
        } else {
            (n % 360) as usize
        }
    };
    let quat = |i: usize, axis: usize| {
        let (c, s) = (
            Single::from_bits(HALF_DEGREE_COS[i]),
            Single::from_bits(HALF_DEGREE_SIN[i]),
        );
        let k = s.mulss(Single::ZERO);
        let mut q = [k, k, k, c];
        q[axis] = s;
        q
    };
    let v = [Single::ZERO, Single::from_i32(lead.r).neg(), Single::ZERO];
    let v = quat_rotate(quat(fold(lead.pitch.to_i32()), 0), v);
    let d = crate::data::pivot::fast_degrees(lead.angle.0);
    let v = quat_rotate(quat(fold(-d), 2), v);
    [
        v[0].addss(Single::from_i32(launch.x)),
        Single::from_i32(launch.y).addss(v[1]),
        Single::from_i32(sz).addss(v[2]),
    ]
}

/// **`Point<float>::rotate(Quaternion *)@00420b10`**: `v` turned by the
/// unit quaternion `q = (x, y, z, w)`, as the rotation matrix's rows times
/// `v`, operation for operation in the listing's order (`addss` and
/// `mulss` commute; their grouping is what is kept). `1` is the constant at
/// `0xb69514`.
pub fn quat_rotate(q: [Single; 4], v: [Single; 3]) -> [Single; 3] {
    let one = Single::from_bits(0x3f80_0000);
    let [qx, qy, qz, qw] = q;
    let [vx, vy, vz] = v;
    let (xx, xy, yy) = (qx.mulss(qx), qy.mulss(qx), qy.mulss(qy));
    let (zx, zz, zy) = (qz.mulss(qx), qz.mulss(qz), qz.mulss(qy));
    let (wz, wy, wx) = (qw.mulss(qz), qw.mulss(qy), qw.mulss(qx));
    let a = zz.addss(yy);
    let t = zx.subss(wy).mulss(vz).addss(wz.addss(xy).mulss(vy));
    let x = one.subss(a.addss(a)).mulss(vx).addss(t.addss(t));
    let h = zz.addss(xx);
    let k = wx.addss(zy).mulss(vz).addss(xy.subss(wz).mulss(vx));
    let y = one.subss(h.addss(h)).mulss(vy).addss(k.addss(k));
    let m = yy.addss(xx);
    let n = zy.subss(wx).mulss(vy).addss(wy.addss(zx).mulss(vx));
    let z = one.subss(m.addss(m)).mulss(vz).addss(n.addss(n));
    [x, y, z]
}

/// The two tables `fast_half_degree_to_cosine@00a29010` and
/// `fast_half_degree_to_sine@00a29110` build at their first call:
/// `cosf`/`sinf` of `(float)i × 0.5 × 0.017453292` (`0xb694c0`,
/// `0xb6944c`) for `i` in `0..360`, the CRT's own transcendental — so
/// pinned here, as the original holds them (`docs/DECISIONS.md` entry 16),
/// read from run1469's packet. `fast_half_degree_to_cosine(i)` is entry `i`.
const HALF_DEGREE_COS: [u32; 360] = [
    0x3f80_0000,
    0x3f7f_fd81,
    0x3f7f_f605,
    0x3f7f_e98b,
    0x3f7f_d814,
    0x3f7f_c1a0,
    0x3f7f_a62f,
    0x3f7f_85c3,
    0x3f7f_605c,
    0x3f7f_35f9,
    0x3f7f_069e,
    0x3f7e_d249,
    0x3f7e_98fd,
    0x3f7e_5aba,
    0x3f7e_1781,
    0x3f7d_cf55,
    0x3f7d_8235,
    0x3f7d_3025,
    0x3f7c_d925,
    0x3f7c_7d37,
    0x3f7c_1c5c,
    0x3f7b_b698,
    0x3f7b_4beb,
    0x3f7a_dc58,
    0x3f7a_67e2,
    0x3f79_ee89,
    0x3f79_7051,
    0x3f78_ed3c,
    0x3f78_654d,
    0x3f77_d886,
    0x3f77_46ea,
    0x3f76_b07c,
    0x3f76_153f,
    0x3f75_7536,
    0x3f74_d064,
    0x3f74_26cb,
    0x3f73_7871,
    0x3f72_c557,
    0x3f72_0d81,
    0x3f71_50f4,
    0x3f70_8fb2,
    0x3f6f_c9c0,
    0x3f6e_ff20,
    0x3f6e_2fd9,
    0x3f6d_5bec,
    0x3f6c_835e,
    0x3f6b_a635,
    0x3f6a_c473,
    0x3f69_de1d,
    0x3f68_f339,
    0x3f68_03c9,
    0x3f67_0fd4,
    0x3f66_175e,
    0x3f65_1a6b,
    0x3f64_1901,
    0x3f63_1324,
    0x3f62_08da,
    0x3f60_fa29,
    0x3f5f_e714,
    0x3f5e_cfa1,
    0x3f5d_b3d7,
    0x3f5c_93ba,
    0x3f5b_6f51,
    0x3f5a_46a0,
    0x3f59_19ae,
    0x3f57_e880,
    0x3f56_b31d,
    0x3f55_798b,
    0x3f54_3bcf,
    0x3f52_f9ef,
    0x3f51_b3f3,
    0x3f50_69e0,
    0x3f4f_1bbd,
    0x3f4d_c991,
    0x3f4c_7361,
    0x3f4b_1934,
    0x3f49_bb13,
    0x3f48_5902,
    0x3f46_f309,
    0x3f45_8931,
    0x3f44_1b7d,
    0x3f42_a9f7,
    0x3f41_34a6,
    0x3f3f_bb90,
    0x3f3e_3ebd,
    0x3f3c_be35,
    0x3f3b_39ff,
    0x3f39_b223,
    0x3f38_26a7,
    0x3f36_9795,
    0x3f35_04f3,
    0x3f33_6eca,
    0x3f31_d522,
    0x3f30_3801,
    0x3f2e_9772,
    0x3f2c_f37b,
    0x3f2b_4c24,
    0x3f29_a178,
    0x3f27_f37c,
    0x3f26_423a,
    0x3f24_8dbb,
    0x3f22_d606,
    0x3f21_1b25,
    0x3f1f_5d1f,
    0x3f1d_9bfe,
    0x3f1b_d7ca,
    0x3f1a_108d,
    0x3f18_464e,
    0x3f16_7919,
    0x3f14_a8f3,
    0x3f12_d5e8,
    0x3f11_0000,
    0x3f0f_2744,
    0x3f0d_4bbe,
    0x3f0b_6d77,
    0x3f09_8c77,
    0x3f07_a8ca,
    0x3f05_c278,
    0x3f03_d989,
    0x3f01_ee09,
    0x3eff_ffff,
    0x3efc_1ef1,
    0x3ef8_38f6,
    0x3ef4_4e27,
    0x3ef0_5e95,
    0x3eec_6a50,
    0x3ee8_7172,
    0x3ee4_7409,
    0x3ee0_722f,
    0x3edc_6bf5,
    0x3ed8_616b,
    0x3ed4_52ad,
    0x3ed0_3fc8,
    0x3ecc_28d7,
    0x3ec8_0deb,
    0x3ec3_ef15,
    0x3ebf_cc71,
    0x3ebb_a60c,
    0x3eb7_7c02,
    0x3eb3_4e60,
    0x3eaf_1d44,
    0x3eaa_e8bd,
    0x3ea6_b0de,
    0x3ea2_75c3,
    0x3e9e_3779,
    0x3e99_f61d,
    0x3e95_b1c1,
    0x3e91_6a75,
    0x3e8d_2058,
    0x3e88_d377,
    0x3e84_83ef,
    0x3e80_31cd,
    0x3e77_ba60,
    0x3e6f_0c51,
    0x3e66_5991,
    0x3e5d_a25b,
    0x3e54_e6cb,
    0x3e4c_271c,
    0x3e43_6373,
    0x3e3a_9bf3,
    0x3e31_d0d7,
    0x3e29_023b,
    0x3e20_305d,
    0x3e17_5b5e,
    0x3e0e_8365,
    0x3e05_a8ac,
    0x3df9_96a0,
    0x3de7_d71a,
    0x3dd6_12ff,
    0x3dc4_4ac8,
    0x3db2_7ebe,
    0x3da0_af28,
    0x3d8e_dc80,
    0x3d7a_0e00,
    0x3d56_5e3f,
    0x3d32_aa50,
    0x3d0e_f2c6,
    0x3cd6_7123,
    0x3c8e_f84d,
    0x3c0e_f9da,
    0xb33b_bd2e,
    0xbc0e_f9b8,
    0xbc8e_f83c,
    0xbcd6_7112,
    0xbd0e_f2bd,
    0xbd32_aa48,
    0xbd56_5e36,
    0xbd7a_0df7,
    0xbd8e_dc7c,
    0xbda0_af24,
    0xbdb2_7eb9,
    0xbdc4_4ac4,
    0xbdd6_130b,
    0xbde7_d716,
    0xbdf9_969b,
    0xbe05_a8aa,
    0xbe0e_8363,
    0xbe17_5b5c,
    0xbe20_305a,
    0xbe29_0239,
    0xbe31_d0d5,
    0xbe3a_9bf0,
    0xbe43_6371,
    0xbe4c_271a,
    0xbe54_e6c9,
    0xbe5d_a259,
    0xbe66_598e,
    0xbe6f_0c4f,
    0xbe77_ba5e,
    0xbe80_31d0,
    0xbe84_83ee,
    0xbe88_d376,
    0xbe8d_2057,
    0xbe91_6a74,
    0xbe95_b1c0,
    0xbe99_f61c,
    0xbe9e_3778,
    0xbea2_75c2,
    0xbea6_b0dd,
    0xbeaa_e8bc,
    0xbeaf_1d43,
    0xbeb3_4e63,
    0xbeb7_7c01,
    0xbebb_a60b,
    0xbebf_cc70,
    0xbec3_ef14,
    0xbec8_0dea,
    0xbecc_28d6,
    0xbed0_3fc7,
    0xbed4_52ac,
    0xbed8_616e,
    0xbedc_6bf4,
    0xbee0_722e,
    0xbee4_7408,
    0xbee8_716e,
    0xbeec_6a52,
    0xbef0_5e94,
    0xbef4_4e26,
    0xbef8_38f5,
    0xbefc_1eed,
    0xbf00_0001,
    0xbf01_ee09,
    0xbf03_d988,
    0xbf05_c276,
    0xbf07_a8cb,
    0xbf09_8c78,
    0xbf0b_6d77,
    0xbf0d_4bbd,
    0xbf0f_2743,
    0xbf11_0001,
    0xbf12_d5e8,
    0xbf14_a8f3,
    0xbf16_7917,
    0xbf18_464d,
    0xbf1a_108e,
    0xbf1b_d7ca,
    0xbf1d_9bfd,
    0xbf1f_5d1e,
    0xbf21_1b23,
    0xbf22_d607,
    0xbf24_8dbb,
    0xbf26_423a,
    0xbf27_f37b,
    0xbf29_a176,
    0xbf2b_4c26,
    0xbf2c_f37b,
    0xbf2e_9771,
    0xbf30_3800,
    0xbf31_d520,
    0xbf33_6ecb,
    0xbf35_04f3,
    0xbf36_9794,
    0xbf38_26a6,
    0xbf39_b224,
    0xbf3b_3a00,
    0xbf3c_be35,
    0xbf3e_3ebd,
    0xbf3f_bb8f,
    0xbf41_34a7,
    0xbf42_a9f8,
    0xbf44_1b7d,
    0xbf45_8930,
    0xbf46_f308,
    0xbf48_5903,
    0xbf49_bb13,
    0xbf4b_1934,
    0xbf4c_7360,
    0xbf4d_c98f,
    0xbf4f_1bbe,
    0xbf50_69e0,
    0xbf51_b3f3,
    0xbf52_f9ee,
    0xbf54_3bcd,
    0xbf55_798b,
    0xbf56_b31d,
    0xbf57_e880,
    0xbf59_19ad,
    0xbf5a_469f,
    0xbf5b_6f51,
    0xbf5c_93bb,
    0xbf5d_b3d7,
    0xbf5e_cfa1,
    0xbf5f_e715,
    0xbf60_fa29,
    0xbf62_08da,
    0xbf63_1324,
    0xbf64_1900,
    0xbf65_1a6c,
    0xbf66_175e,
    0xbf67_0fd4,
    0xbf68_03c9,
    0xbf68_f338,
    0xbf69_de1e,
    0xbf6a_c473,
    0xbf6b_a635,
    0xbf6c_835e,
    0xbf6d_5beb,
    0xbf6e_2fd9,
    0xbf6e_ff21,
    0xbf6f_c9bf,
    0xbf70_8fb2,
    0xbf71_50f3,
    0xbf72_0d82,
    0xbf72_c557,
    0xbf73_7870,
    0xbf74_26cb,
    0xbf74_d063,
    0xbf75_7536,
    0xbf76_153f,
    0xbf76_b07c,
    0xbf77_46ea,
    0xbf77_d886,
    0xbf78_654d,
    0xbf78_ed3c,
    0xbf79_7051,
    0xbf79_ee89,
    0xbf7a_67e2,
    0xbf7a_dc59,
    0xbf7b_4beb,
    0xbf7b_b698,
    0xbf7c_1c5c,
    0xbf7c_7d37,
    0xbf7c_d925,
    0xbf7d_3025,
    0xbf7d_8235,
    0xbf7d_cf54,
    0xbf7e_1781,
    0xbf7e_5aba,
    0xbf7e_98fd,
    0xbf7e_d249,
    0xbf7f_069d,
    0xbf7f_35fa,
    0xbf7f_605c,
    0xbf7f_85c3,
    0xbf7f_a62f,
    0xbf7f_c1a0,
    0xbf7f_d814,
    0xbf7f_e98b,
    0xbf7f_f605,
    0xbf7f_fd81,
];

/// `fast_half_degree_to_sine`'s table, as [`HALF_DEGREE_COS`].
const HALF_DEGREE_SIN: [u32; 360] = [
    0x0000_0000,
    0x3c0e_f9be,
    0x3c8e_f859,
    0x3cd6_710b,
    0x3d0e_f2c6,
    0x3d32_aa3e,
    0x3d56_5e3b,
    0x3d7a_0e09,
    0x3d8e_dc7b,
    0x3da0_af2a,
    0x3db2_7eb5,
    0x3dc4_4ac7,
    0x3dd6_1305,
    0x3de7_d716,
    0x3df9_96a3,
    0x3e05_a8a9,
    0x3e0e_8365,
    0x3e17_5b59,
    0x3e20_305c,
    0x3e29_023d,
    0x3e31_d0d4,
    0x3e3a_9bf3,
    0x3e43_636f,
    0x3e4c_271b,
    0x3e54_e6ce,
    0x3e5d_a259,
    0x3e66_5991,
    0x3e6f_0c4d,
    0x3e77_ba60,
    0x3e80_31cf,
    0x3e84_83ee,
    0x3e88_d378,
    0x3e8d_2057,
    0x3e91_6a76,
    0x3e95_b1be,
    0x3e99_f61c,
    0x3e9e_377a,
    0x3ea2_75c1,
    0x3ea6_b0de,
    0x3eaa_e8bc,
    0x3eaf_1d43,
    0x3eb3_4e62,
    0x3eb7_7c01,
    0x3ebb_a60c,
    0x3ebf_cc6f,
    0x3ec3_ef16,
    0x3ec8_0de9,
    0x3ecc_28d7,
    0x3ed0_3fc9,
    0x3ed4_52ac,
    0x3ed8_616c,
    0x3edc_6bf3,
    0x3ee0_722f,
    0x3ee4_740a,
    0x3ee8_7171,
    0x3eec_6a50,
    0x3ef0_5e94,
    0x3ef4_4e27,
    0x3ef8_38f7,
    0x3efc_1ef1,
    0x3f00_0000,
    0x3f01_ee09,
    0x3f03_d989,
    0x3f05_c277,
    0x3f07_a8ca,
    0x3f09_8c78,
    0x3f0b_6d77,
    0x3f0d_4bbe,
    0x3f0f_2744,
    0x3f11_0000,
    0x3f12_d5e8,
    0x3f14_a8f3,
    0x3f16_7918,
    0x3f18_464e,
    0x3f1a_108c,
    0x3f1b_d7ca,
    0x3f1d_9bfe,
    0x3f1f_5d1f,
    0x3f21_1b24,
    0x3f22_d605,
    0x3f24_8dba,
    0x3f26_423a,
    0x3f27_f37c,
    0x3f29_a178,
    0x3f2b_4c25,
    0x3f2c_f37a,
    0x3f2e_9772,
    0x3f30_3801,
    0x3f31_d522,
    0x3f33_6eca,
    0x3f35_04f3,
    0x3f36_9795,
    0x3f38_26a7,
    0x3f39_b222,
    0x3f3b_39ff,
    0x3f3c_be35,
    0x3f3e_3ebe,
    0x3f3f_bb90,
    0x3f41_34a5,
    0x3f42_a9f7,
    0x3f44_1b7d,
    0x3f45_8930,
    0x3f46_f309,
    0x3f48_5902,
    0x3f49_bb12,
    0x3f4b_1934,
    0x3f4c_7361,
    0x3f4d_c990,
    0x3f4f_1bbd,
    0x3f50_69e0,
    0x3f51_b3f3,
    0x3f52_f9ef,
    0x3f54_3bce,
    0x3f55_798b,
    0x3f56_b31d,
    0x3f57_e881,
    0x3f59_19ae,
    0x3f5a_46a0,
    0x3f5b_6f51,
    0x3f5c_93ba,
    0x3f5d_b3d8,
    0x3f5e_cfa1,
    0x3f5f_e714,
    0x3f60_fa29,
    0x3f62_08da,
    0x3f63_1325,
    0x3f64_1901,
    0x3f65_1a6b,
    0x3f66_175e,
    0x3f67_0fd4,
    0x3f68_03ca,
    0x3f68_f339,
    0x3f69_de1e,
    0x3f6a_c473,
    0x3f6b_a634,
    0x3f6c_835e,
    0x3f6d_5bec,
    0x3f6e_2fd9,
    0x3f6e_ff20,
    0x3f6f_c9c0,
    0x3f70_8fb2,
    0x3f71_50f4,
    0x3f72_0d81,
    0x3f72_c557,
    0x3f73_7871,
    0x3f74_26cb,
    0x3f74_d063,
    0x3f75_7536,
    0x3f76_153f,
    0x3f76_b07c,
    0x3f77_46ea,
    0x3f77_d886,
    0x3f78_654d,
    0x3f78_ed3c,
    0x3f79_7051,
    0x3f79_ee89,
    0x3f7a_67e2,
    0x3f7a_dc58,
    0x3f7b_4beb,
    0x3f7b_b698,
    0x3f7c_1c5c,
    0x3f7c_7d37,
    0x3f7c_d925,
    0x3f7d_3025,
    0x3f7d_8235,
    0x3f7d_cf54,
    0x3f7e_1781,
    0x3f7e_5aba,
    0x3f7e_98fd,
    0x3f7e_d249,
    0x3f7f_069e,
    0x3f7f_35f9,
    0x3f7f_605c,
    0x3f7f_85c3,
    0x3f7f_a62f,
    0x3f7f_c1a0,
    0x3f7f_d814,
    0x3f7f_e98b,
    0x3f7f_f605,
    0x3f7f_fd81,
    0x3f80_0000,
    0x3f7f_fd81,
    0x3f7f_f605,
    0x3f7f_e98b,
    0x3f7f_d814,
    0x3f7f_c1a0,
    0x3f7f_a62f,
    0x3f7f_85c3,
    0x3f7f_605c,
    0x3f7f_35f9,
    0x3f7f_069e,
    0x3f7e_d249,
    0x3f7e_98fd,
    0x3f7e_5aba,
    0x3f7e_1781,
    0x3f7d_cf55,
    0x3f7d_8235,
    0x3f7d_3025,
    0x3f7c_d925,
    0x3f7c_7d37,
    0x3f7c_1c5c,
    0x3f7b_b698,
    0x3f7b_4beb,
    0x3f7a_dc59,
    0x3f7a_67e2,
    0x3f79_ee89,
    0x3f79_7051,
    0x3f78_ed3c,
    0x3f78_654d,
    0x3f77_d886,
    0x3f77_46ea,
    0x3f76_b07d,
    0x3f76_153f,
    0x3f75_7536,
    0x3f74_d063,
    0x3f74_26cb,
    0x3f73_7871,
    0x3f72_c557,
    0x3f72_0d82,
    0x3f71_50f4,
    0x3f70_8fb2,
    0x3f6f_c9bf,
    0x3f6e_ff21,
    0x3f6e_2fd9,
    0x3f6d_5bec,
    0x3f6c_835f,
    0x3f6b_a635,
    0x3f6a_c473,
    0x3f69_de1e,
    0x3f68_f339,
    0x3f68_03c9,
    0x3f67_0fd4,
    0x3f66_175e,
    0x3f65_1a6c,
    0x3f64_1902,
    0x3f63_1324,
    0x3f62_08da,
    0x3f60_fa29,
    0x3f5f_e714,
    0x3f5e_cfa3,
    0x3f5d_b3d7,
    0x3f5c_93ba,
    0x3f5b_6f51,
    0x3f5a_46a1,
    0x3f59_19ad,
    0x3f57_e880,
    0x3f56_b31d,
    0x3f55_798b,
    0x3f54_3bcf,
    0x3f52_f9ee,
    0x3f51_b3f2,
    0x3f50_69e0,
    0x3f4f_1bbd,
    0x3f4d_c991,
    0x3f4c_7360,
    0x3f4b_1934,
    0x3f49_bb13,
    0x3f48_5903,
    0x3f46_f30b,
    0x3f45_8930,
    0x3f44_1b7d,
    0x3f42_a9f7,
    0x3f41_34a6,
    0x3f3f_bb91,
    0x3f3e_3ebd,
    0x3f3c_be35,
    0x3f3b_39ff,
    0x3f39_b223,
    0x3f38_26a9,
    0x3f36_9794,
    0x3f35_04f3,
    0x3f33_6ecb,
    0x3f31_d523,
    0x3f30_3800,
    0x3f2e_9771,
    0x3f2c_f37b,
    0x3f2b_4c25,
    0x3f29_a179,
    0x3f27_f37b,
    0x3f26_423a,
    0x3f24_8dbb,
    0x3f22_d606,
    0x3f21_1b26,
    0x3f1f_5d1d,
    0x3f1d_9bfd,
    0x3f1b_d7ca,
    0x3f1a_108e,
    0x3f18_4650,
    0x3f16_7917,
    0x3f14_a8f3,
    0x3f12_d5e8,
    0x3f11_0001,
    0x3f0f_2746,
    0x3f0d_4bbd,
    0x3f0b_6d77,
    0x3f09_8c78,
    0x3f07_a8cb,
    0x3f05_c279,
    0x3f03_d988,
    0x3f01_ee09,
    0x3f00_0001,
    0x3efc_1ef4,
    0x3ef8_38f5,
    0x3ef4_4e26,
    0x3ef0_5e94,
    0x3eec_6a52,
    0x3ee8_7174,
    0x3ee4_7408,
    0x3ee0_722e,
    0x3edc_6bf4,
    0x3ed8_616e,
    0x3ed4_52b0,
    0x3ed0_3fc7,
    0x3ecc_28d6,
    0x3ec8_0dea,
    0x3ec3_ef17,
    0x3ebf_cc73,
    0x3ebb_a60a,
    0x3eb7_7c00,
    0x3eb3_4e63,
    0x3eaf_1d46,
    0x3eaa_e8c0,
    0x3ea6_b0dd,
    0x3ea2_75c1,
    0x3e9e_377b,
    0x3e99_f61f,
    0x3e95_b1c3,
    0x3e91_6a74,
    0x3e8d_2057,
    0x3e88_d37a,
    0x3e84_83f1,
    0x3e80_31cc,
    0x3e77_ba5d,
    0x3e6f_0c4e,
    0x3e66_5996,
    0x3e5d_a260,
    0x3e54_e6c8,
    0x3e4c_2719,
    0x3e43_6370,
    0x3e3a_9bf7,
    0x3e31_d0dc,
    0x3e29_0239,
    0x3e20_305a,
    0x3e17_5b5b,
    0x3e0e_836a,
    0x3e05_a8b1,
    0x3df9_969a,
    0x3de7_d714,
    0x3dd6_1309,
    0x3dc4_4ad2,
    0x3db2_7ec8,
    0x3da0_af22,
    0x3d8e_dc7b,
    0x3d7a_0e14,
    0x3d56_5e53,
    0x3d32_aa64,
    0x3d0e_f2ba,
    0x3cd6_710c,
    0x3c8e_f875,
    0x3c0e_fa2b,
];

/// The most points a round's spline holds here. A guided round's is
/// fourteen at most: `calc_spline@00912f00` caps a three-point spline's
/// resolution at `3 × 4.0 = 12`, and `generate_bspline@00911820` lays the
/// first point, one a step, and the last control point. A Cruise
/// Missile's ([`Spline::cruise`]) is uncapped, one point per 96 of its
/// control polygon: 29 for run1470's 2,700, about 85 at its full range
/// of 40 tiles over flat ground.
///
/// SEAM: the original's point array grows; this one stops laying at
/// [`SPLINE_MAX`], a polygon past 12,000.
pub const SPLINE_MAX: usize = 128;

/// **A guided round's flight path** (item 3802, `docs/GOLDEN.md` §152.2):
/// the points `Spline::calc_from_dir@00913960` lays for a round whose ammo
/// carries `missile="1"`, which `Ammo::inc_time@0067d380` steps one a
/// frame, measures against its target and re-plans every fourth frame.
/// Each point is three singles, by their bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spline {
    pts: [[Single; 3]; SPLINE_MAX],
    len: u8,
}

impl Spline {
    /// The points laid: `Spline +0xd8`, which `Ammo::init` and the re-plan
    /// take as `total_time`.
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    /// Never: the first control point is always laid.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Point `i`, `(x, y, z)`.
    pub fn point(&self, i: usize) -> [Single; 3] {
        self.pts[i]
    }

    fn push(&mut self, p: [Single; 3]) {
        if usize::from(self.len) < SPLINE_MAX {
            self.pts[usize::from(self.len)] = p;
            self.len += 1;
        }
    }

    /// **`Spline::calc_from_dir@00913960` with a zero third vector** — the
    /// control points `start`, `dir`, `end`, degree 2 — at the speed
    /// `set_min_seg_length@009125e0` divides the control polygon by, and
    /// the quadratic B-spline `generate_bspline@00911820` lays over the
    /// clamped knots `clamp_knots@00910be0` builds, `0 0 0 1 1 1`. Every
    /// step is the listing's single-precision operation:
    ///
    /// - the polygon's length is `Σ sqrt((dx·dx + dy·dy) + dz·dz)`, each
    ///   root taken in double and rounded back (`9125e0`–`912656`), which
    ///   is the correctly rounded single root, and the resolution its
    ///   quotient by `speed`, truncated to a `u16` (`+0x62`);
    /// - `calc_spline` caps it at `(int)(3 × 4.0)` and raises it to the
    ///   degree;
    /// - the parameter starts at `k₀ + δ`, `δ = (k₅ − k₀) / res`, and is
    ///   *accumulated*, `t += δ`; a point is laid while `k₅ > t`, each the
    ///   sum over the span's three control points of `P · N(t)`, the basis
    ///   Cox–de Boor's with `calc_bspline_basis@009116e0`'s guards;
    /// - and the last control point is laid when `k₅ − (t − δ)`, widened
    ///   to a double, exceeds 0.001 (`911b15`–`911b2d`).
    pub fn from_dir(
        start: [Single; 3],
        dir: [Single; 3],
        end: [Single; 3],
        speed: Single,
    ) -> Spline {
        const DEGREE: usize = 2;
        let ctl = [start, dir, end];
        let zero = Single::from_bits(0);
        let one = Single::from_bits(0x3f80_0000);
        let mut res = i32::from(polygon_length(&ctl).divss(speed).to_i32() as u16);
        let cap = Single::from_i32(3)
            .mulss(Single::from_bits(0x4080_0000))
            .to_i32();
        if cap != 0 && cap < res {
            res = cap;
        }
        res = res.max(DEGREE as i32);
        Spline::generate(&ctl, DEGREE, &[zero, zero, zero, one, one, one], res)
    }

    /// **`Spline::calc_nuke_spline@00913ad0`'s Cruise Missile arm**
    /// (item 3806, `docs/GOLDEN.md` §152.4), from its control points on:
    /// the cubic (`+0x60 = 3`) over [`cruise_control_points`]'s polygon.
    ///
    /// - `set_min_seg_length@009125e0` takes the resolution as the
    ///   polygon's length over **96** (`0xb696a8`, loaded into `xmm1` at
    ///   `913de7`), truncated to a `u16`;
    /// - `calc_spline` caps nothing (`+0x48` is 0) and raises it to the
    ///   degree;
    /// - every knot weight is 30 (`913e96`), so `clamp_knots@00910be0`
    ///   lays `degree + 1` zeros, then one step of `3 × 30` for each
    ///   control point past the degree, the last value repeated to `n +
    ///   degree + 1` knots — run1470's five points read `0 0 0 0 90 180
    ///   180 180 180`;
    /// - and `generate_bspline` lays them as [`Spline::from_dir`] does.
    ///
    /// run1470's Cruise Missile `2/19` on `1/20`: a 2,700.6 polygon, 28
    /// steps, 29 points; `Ammo::init` takes `total_time` as 28.
    pub fn cruise(ctl: &[[Single; 3]]) -> Spline {
        const DEGREE: usize = 3;
        let speed = Single::from_bits(0x42c0_0000);
        let res = i32::from(polygon_length(ctl).divss(speed).to_i32() as u16).max(DEGREE as i32);
        let step = Single::from_i32(30);
        let mut knots = [Single::from_bits(0); SPLINE_CTL_MAX + DEGREE + 1];
        let mut k = Single::from_bits(0);
        let n = ctl.len().min(SPLINE_CTL_MAX);
        for (i, slot) in knots.iter_mut().enumerate().take(n + DEGREE + 1) {
            if i > DEGREE && i <= n {
                for _ in 0..DEGREE {
                    k = k.addss(step);
                }
            }
            *slot = k;
        }
        Spline::generate(&ctl[..n], DEGREE, &knots[..n + DEGREE + 1], res)
    }

    /// **`Spline::calc_nuke_spline@00913ad0`'s other arm** (`param_4 ==
    /// 0`: a missile-flag shooter that is not the Cruise Missile — the V2,
    /// a nuke; item 3814, `docs/GOLDEN.md` §152.8), from the launch `s` to
    /// the landing `e`. Twelve control points, each coordinate a single:
    ///
    /// - `s`, then `s` raised by 250, 750, 2,250 and 6,750 (`0xb696d0`,
    ///   `443b8000`, `450ca000`, `45d2f000`);
    /// - `((3·sx + ex)·¼, (3·sy + ey)·¼, (3·sz + ez)·¼ + 20,000)`,
    ///   `((sx + ex)·½, (sy + ey)·½, (sz + ez)·½ + 20,000)` and
    ///   `((3·ex + sx)·¼, (3·ey + sy)·¼, (3·ez + sz)·¼ + 10,000)`;
    /// - `e` raised by 6,000, 4,000 and 2,000, then `e`.
    ///
    /// The cubic (`+0x40 = 3`) at the fixed resolution 120 (`+0x60 =
    /// 0x780003`; no `set_min_seg_length`, and `+0x48` is 0, so
    /// `calc_spline` caps nothing). The knot weights are 10 but for the
    /// first three, 100, 30 and 15 (`+0x90`), and `clamp_knots@00910be0`
    /// steps each knot past the degree by the sum of `degree` weights from
    /// the step's own index: `0 0 0 0 145 200 235 265 295 325 355 385 415
    /// 415 415 415`.
    ///
    /// run1471's V2 `0/16` on `2/21`: 121 points, `total_time` 120,
    /// `total_spline_length` 39,282.046875; `Ammo::inc_time` re-plans it at
    /// its fourth point as a guided round's.
    pub fn nuke(s: (i32, i32, i32), e: (i32, i32, i32)) -> Spline {
        const DEGREE: usize = 3;
        const N: usize = 12;
        let f = Single::from_i32;
        let (sx, sy, sz) = (f(s.0), f(s.1), f(s.2));
        let (ex, ey, ez) = (f(e.0), f(e.1), f(e.2));
        let three = f(3);
        let quarter = Single::from_bits(0x3e80_0000);
        let half = Single::from_bits(0x3f00_0000);
        let up = |z: Single, by: i32| z.addss(f(by));
        let ctl: [[Single; 3]; N] = [
            [sx, sy, sz],
            [sx, sy, up(sz, 250)],
            [sx, sy, up(sz, 750)],
            [sx, sy, up(sz, 2250)],
            [sx, sy, up(sz, 6750)],
            [
                sx.mulss(three).addss(ex).mulss(quarter),
                sy.mulss(three).addss(ey).mulss(quarter),
                up(sz.mulss(three).addss(ez).mulss(quarter), 20_000),
            ],
            [
                sx.addss(ex).mulss(half),
                sy.addss(ey).mulss(half),
                up(sz.addss(ez).mulss(half), 20_000),
            ],
            [
                ex.mulss(three).addss(sx).mulss(quarter),
                ey.mulss(three).addss(sy).mulss(quarter),
                up(ez.mulss(three).addss(sz).mulss(quarter), 10_000),
            ],
            [ex, ey, up(ez, 6000)],
            [ex, ey, up(ez, 4000)],
            [ex, ey, up(ez, 2000)],
            [ex, ey, ez],
        ];
        let mut weights = [f(10); N + 4];
        weights[0] = f(100);
        weights[1] = f(30);
        weights[2] = f(15);
        let mut knots = [Single::from_bits(0); N + DEGREE + 1];
        let mut k = Single::from_bits(0);
        let mut step = 0;
        for (i, slot) in knots.iter_mut().enumerate() {
            if i > DEGREE && i <= N {
                for w in &weights[step..step + DEGREE] {
                    k = k.addss(*w);
                }
                step += 1;
            }
            *slot = k;
        }
        Spline::generate(&ctl, DEGREE, &knots, crate::air::MISSILE_FLIGHT)
    }

    /// **`Spline::generate_bspline@00911820`** over `ctl` at `degree`, on
    /// the clamped `knots`, at the resolution `res`. Every step is the
    /// listing's single-precision operation:
    ///
    /// - the parameter starts at `k₀ + δ`, `δ = (k_last − k₀) / res`, and is
    ///   *accumulated*, `t += δ`; a point is laid while `k_last > t`, each
    ///   the sum over the span's `degree + 1` control points of `P · N(t)`,
    ///   the basis Cox–de Boor's with `calc_bspline_basis@009116e0`'s
    ///   guards;
    /// - and the last control point is laid when `k_last − (t − δ)`,
    ///   widened to a double, exceeds 0.001 (`911b15`–`911b2d`).
    fn generate(ctl: &[[Single; 3]], degree: usize, knots: &[Single], res: i32) -> Spline {
        let zero = Single::from_bits(0);
        let last = knots.len() - 1;
        let delta = knots[last].subss(knots[0]).divss(Single::from_i32(res));
        let mut s = Spline {
            pts: [[zero; 3]; SPLINE_MAX],
            len: 0,
        };
        s.push(ctl[0]);
        let mut t = knots[0].addss(delta);
        let mut span = 0;
        while knots[last].gt(t) {
            let mut next = span + 1;
            while next < knots.len() {
                if !t.gt(knots[next]) && knots[next].gt(knots[next - 1]) {
                    break;
                }
                next += 1;
                span += 1;
            }
            if span > last {
                break;
            }
            let mut p = [zero; 3];
            for (j, c) in ctl
                .iter()
                .enumerate()
                .take(span + 1)
                .skip(span.saturating_sub(degree))
            {
                let b = bspline_basis(t, j, degree, knots);
                for k in 0..3 {
                    p[k] = p[k].addss(c[k].mulss(b));
                }
            }
            t = t.addss(delta);
            s.push(p);
        }
        // `0.001` as a double lies between these two singles, so a single
        // above the lower is one above the double.
        if knots[last]
            .subss(t.subss(delta))
            .gt(Single::from_bits(0x3a83_126e))
        {
            s.push(ctl[ctl.len() - 1]);
        }
        s
    }
}

/// The most control points [`Spline::cruise`] takes: the launch and its
/// rise, the descent and the landing, and one ground point per 768 —
/// 13 at a Cruise Missile's full range.
pub const SPLINE_CTL_MAX: usize = 64;

/// `set_min_seg_length@009125e0`'s length: `Σ sqrt((dx·dx + dy·dy) +
/// dz·dz)` over the control polygon, each root taken in double and rounded
/// back (`9125e0`–`912656`), which is the correctly rounded single root.
fn polygon_length(ctl: &[[Single; 3]]) -> Single {
    let mut length = Single::from_bits(0);
    for w in ctl.windows(2) {
        let d = |c: usize| w[1][c].subss(w[0][c]);
        let (dx, dy, dz) = (d(0), d(1), d(2));
        let sq = dx.mulss(dx).addss(dy.mulss(dy)).addss(dz.mulss(dz));
        length = sq.sqrtss().addss(length);
    }
    length
}

/// **`Spline::calc_nuke_spline@00913ad0`'s Cruise Missile arm**, its
/// control points (item 3806, `docs/GOLDEN.md` §152.4): the arm
/// `Ammo::init@0067bbf0` takes for a shooter whose type carries the
/// missile flag (`+0x1e4 & 0x8000000`) and is the Cruise Missile
/// (`is(0x13a)`, the fourth argument), whose ammo is not `missile="1"`.
/// The launch `s`, then `s` 250 up (`0xb696d0`), then for each `i` of
/// `1..n`, `n = vector_dist(|sx − ex|, |sy − ey|) / 768`, the point
/// `((i·ex + (n−i)·sx) / n, (i·ey + (n−i)·sy) / n)` in singles,
/// truncated (`913c10`–`913c55`), at `ground(x, y)` plus 250 —
/// [`cruise_clearance`] — then `e` 250 up, then `e`.
///
/// `ground` is the caller's `find_data_z(x, y, 0)`; `mask` the tile word
/// (`worldc +0x138`) under the point.
pub fn cruise_control_points(
    s: (i32, i32, i32),
    e: (i32, i32, i32),
    mut ground: impl FnMut(i32, i32) -> (i32, u16),
) -> ([[Single; 3]; SPLINE_CTL_MAX], usize) {
    let v = |x: i32, y: i32, z: i32| {
        [
            Single::from_i32(x),
            Single::from_i32(y),
            Single::from_i32(z),
        ]
    };
    let mut out = [[Single::from_bits(0); 3]; SPLINE_CTL_MAX];
    let mut n = 0;
    let mut put = |p: [Single; 3]| {
        if n < SPLINE_CTL_MAX - 2 {
            out[n] = p;
            n += 1;
        }
    };
    put(v(s.0, s.1, s.2));
    put(v(s.0, s.1, s.2 + 250));
    let steps = vector_dist((s.0 - e.0).abs(), (s.1 - e.1).abs()) / 0x300;
    let f = Single::from_i32(steps);
    for i in 1..steps {
        let (a, b) = (Single::from_i32(i), Single::from_i32(steps - i));
        let x = a
            .mulss(Single::from_i32(e.0))
            .addss(b.mulss(Single::from_i32(s.0)))
            .divss(f)
            .to_i32();
        let y = a
            .mulss(Single::from_i32(e.1))
            .addss(b.mulss(Single::from_i32(s.1)))
            .divss(f)
            .to_i32();
        let (z, mask) = ground(x, y);
        put(v(x, y, z + cruise_clearance(mask)));
    }
    out[n] = v(e.0, e.1, e.2 + 250);
    out[n + 1] = v(e.0, e.1, e.2);
    (out, n + 2)
}

/// The height a Cruise Missile's ground point keeps over the terrain
/// (`913c6e`–`913ccf`): 250, and 1,000 more where the tile word carries
/// bit 14 and its low two bits are both set, else 500 more where its
/// surface field (`0x30`) is full.
pub fn cruise_clearance(mask: u16) -> i32 {
    if mask & 0x4000 != 0 && mask & 3 == 3 {
        250 + 1000
    } else if mask & 0x30 == 0x30 {
        250 + 500
    } else {
        250
    }
}

/// `Spline::calc_bspline_basis@009116e0`: `N(i, p)` at `t`, `1.0` past the
/// knots, the half-open indicator at degree 0, and each recursive term
/// taken only when both its numerator and its denominator are positive —
/// `N(i, p−1) · (t − kᵢ)/(kᵢ₊ₚ − kᵢ)`, then that plus
/// `N(i+1, p−1) · (kᵢ₊ₚ₊₁ − t)/(kᵢ₊ₚ₊₁ − kᵢ₊₁)`.
fn bspline_basis(t: Single, i: usize, p: usize, k: &[Single]) -> Single {
    let zero = Single::from_bits(0);
    let one = Single::from_bits(0x3f80_0000);
    if i + p >= k.len() {
        return one;
    }
    if p == 0 {
        return if !k[i].gt(t) && k[i + 1].gt(t) {
            one
        } else {
            zero
        };
    }
    let mut acc = zero;
    let (a, b) = (t.subss(k[i]), k[i + p].subss(k[i]));
    if a.gt(zero) && b.gt(zero) {
        acc = bspline_basis(t, i, p - 1, k).mulss(a.divss(b));
    }
    let (c, d) = (k[i + p + 1].subss(t), k[i + p + 1].subss(k[i + 1]));
    if c.gt(zero) && d.gt(zero) {
        acc = acc.addss(bspline_basis(t, i + 1, p - 1, k).mulss(c.divss(d)));
    }
    acc
}

/// [`Projectile::in_flight`]'s test: `total_time` and `cur_time` read
/// unsigned, as `Ammo::inc_time@0067d380` compares them.
pub fn in_flight_at(cur_time: i32, total_time: i32) -> bool {
    (cur_time as u32) < (total_time as u32)
}

/// **A bomb's flight time** (`docs/ORDERS.md` §35.3): `Ammo::init@0067bbf0`'s
/// Bomber arm, `0x67d24b`–`0x67d26f` — `(int)sqrtf((float)((ez − sz) · 2) /
/// GRAV_Z)`, the time a round dropped from `sz` takes to fall to `ez` under
/// [`GRAV_Z`]. The listing doubles the integer difference (`addl %eax,
/// %eax`), converts it (`cvtdq2ps`), divides by `GRAV_Z` (`divss`), and
/// calls `0x41e6f0`, which is `(float)sqrt((double)x)` — a correctly
/// rounded single root, since a double carries more than twice a single's
/// bits — and truncates it (`cvtss2sd`, `cvttsd2si`).
///
/// **A round released under its landing height falls for 2^31 frames**
/// (item 3836, `docs/GOLDEN.md` §152.14): its quotient is negative, the
/// CRT's `sqrt` (`0xa571b0`) answers the NaN indefinite, and `cvttsd2si`
/// truncates it to `0x80000000` — which `Ammo::inc_time`'s unsigned end
/// test ([`Projectile::in_flight`]) never reaches. run1468 prints twelve
/// such Stealth Bomber rounds, parked on the ground at `sz` 0, 12 and 17
/// over `ez` 511–595, with `total_time 2147483648` from block 801 to the
/// game's end. A zero quotient (either sign) roots to 0, which the caller
/// raises to 1 as `Ammo::init` does a zero time.
pub fn fall_time(sz: i32, ez: i32) -> i32 {
    let q = Single::from_i32((ez - sz).wrapping_mul(2)).divss(GRAV_Z);
    let bits = q.bits();
    if bits & 0x7fff_ffff == 0 {
        return 0;
    }
    if bits & 0x8000_0000 != 0 {
        return i32::MIN;
    }
    let e = ((bits >> 23) & 0xff) as i32;
    let f = u128::from(bits & 0x007f_ffff);
    let (mut mant, mut exp) = if e == 0 {
        (f, -149)
    } else {
        (f | 1 << 23, e - 150)
    };
    if exp % 2 != 0 {
        mant <<= 1;
        exp -= 1;
    }
    // Sixty guard bits under the root's own twenty-four, and an even shift
    // so the exponent halves exactly.
    let scaled = mant << 60;
    let root = isqrt(scaled);
    let exact = root * root == scaled;
    round24(root, exp / 2 - 30, !exact).trunc_to_int()
}

/// An IEEE single as an exact rational: `mant × 2^exp`, `mant < 2^24`,
/// normalised so `mant ≥ 2^23` unless the value is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct F32 {
    mant: u64,
    exp: i32,
}

impl F32 {
    const fn zero() -> F32 {
        F32 { mant: 0, exp: 0 }
    }
    fn trunc_to_int(self) -> i32 {
        if self.mant == 0 {
            return 0;
        }
        let v: u64 = if self.exp >= 0 {
            self.mant << self.exp.min(40)
        } else {
            self.mant >> (-self.exp).min(63)
        };
        v.min(i32::MAX as u64) as i32
    }
}

/// Rounds `mant × 2^exp` (`mant` any size) to 24 significant bits, round to
/// nearest even, given the exact sticky information in `inexact`.
fn round24(mut mant: u128, mut exp: i32, mut inexact: bool) -> F32 {
    if mant == 0 {
        return F32::zero();
    }
    let bits = 128 - mant.leading_zeros() as i32;
    if bits > 24 {
        let drop = bits - 24;
        let dropped = mant & ((1u128 << drop) - 1);
        let half = 1u128 << (drop - 1);
        mant >>= drop;
        exp += drop;
        let round_up = dropped > half || (dropped == half && (inexact || mant & 1 == 1));
        if dropped != 0 {
            inexact = true;
        }
        let _ = inexact;
        if round_up {
            mant += 1;
            if mant == 1 << 24 {
                mant >>= 1;
                exp += 1;
            }
        }
    } else {
        let up = 24 - bits;
        mant <<= up;
        exp -= up;
    }
    F32 {
        mant: mant as u64,
        exp,
    }
}

fn f32_of_int(n: i64) -> F32 {
    round24(n.unsigned_abs() as u128, 0, false)
}

/// Correctly rounded `sqrtf` of a non-negative integer.
fn f32_sqrt(n: i64) -> F32 {
    if n <= 0 {
        return F32::zero();
    }
    // Scale so the integer square root carries 26+ significant bits: the
    // 24 of the result plus a guard and a sticky.
    let n = n as u128;
    let bits = 128 - n.leading_zeros() as i32;
    let mut shift = (2 * 28 - bits).max(0);
    if shift % 2 == 1 {
        shift += 1;
    }
    let scaled = n << shift;
    let root = isqrt(scaled);
    let exact = root * root == scaled;
    round24(root, -(shift / 2), !exact)
}

/// Correctly rounded `a / b` for two singles.
fn f32_div(a: F32, b: F32) -> F32 {
    if a.mant == 0 {
        return F32::zero();
    }
    // a.mant / b.mant with 40 extra bits, then round.
    let num = (a.mant as u128) << 40;
    let q = num / b.mant as u128;
    let exact = q * b.mant as u128 == num;
    round24(q, a.exp - b.exp - 40, !exact)
}

const fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// A siege unit's flight time (§9.1): `max_range × 0xc0 / (proj_speed ×
/// unit_move_speed)` — integer, and independent of the real distance.
pub const fn siege_flight_time(max_range: i32, proj_speed: i32, unit_move_speed: i32) -> i32 {
    let d = proj_speed * unit_move_speed;
    if d <= 0 { 0 } else { max_range * 0xc0 / d }
}

/// `Ammo::hit_target`'s test for a unit target (§9.4): the landing point is
/// within the target's `target_size`, with the distance halved when accuracy
/// is over 100.
pub const fn hits_unit(landing: Pos, target: Pos, target_size: i32, acc: i32) -> bool {
    let mut d = vector_dist(landing.x - target.x, landing.y - target.y);
    if acc > 100 {
        d /= 2;
    }
    d <= target_size
}

/// `Ammo::hit_target`'s test for a building (§9.4): inside the footprint.
pub const fn hits_building(landing: Pos, centre: Pos, x_size: i32, y_size: i32) -> bool {
    let dx = landing.x - centre.x;
    let dy = landing.y - centre.y;
    dx <= x_size * 0x60 && -dx <= x_size * 0x60 && dy <= y_size * 0x60 && -dy <= y_size * 0x60
}

/// The splash fringe's count for a victim `d` position units from the landing
/// point (§9.3), already reduced by the `0xc0 + guy_radius` (units) or the
/// `x_size × 0xc0` (buildings) the original subtracts: `0x100 − (d << 8) /
/// (splash_area × 0xc0)`.
pub const fn splash_count(d: i32, splash_area: i32) -> i32 {
    let d = if d < 0 { 0 } else { d };
    0x100 - (d << 8) / (splash_area * 0xc0)
}

/// `BuildData::get_garrison_arrows` (§8.6): the building's `attack` (tenths),
/// `base_arrows`, `most_shots`, and the garrison sum `g` as
/// `count_inside(GARRISON_ARROWS)` computes it.
pub const fn garrison_arrows(attack: i32, base_arrows: i32, most_shots: i32, garrison: i32) -> i32 {
    let a = (attack + 5) / 10;
    if a == 0 {
        return 0;
    }
    let mut g = garrison;
    if base_arrows == 0 {
        g += a / 2;
    }
    g /= a;
    let g = if g > most_shots { most_shots } else { g };
    base_arrows + g
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod table_tests;

// The files of `combat/` (item 4012, `docs/STRUCTURE.md` §3).
pub mod attack_pos;
pub mod balance;
pub mod build_attack_view;
pub(crate) mod build_family;
pub mod build_plunder_view;
pub mod cavarch;
pub mod cloak;
pub mod damage;
pub mod fight;
pub mod hero_radius_view;
pub mod nuke;
pub mod object_attack_view;
pub mod plunder;
pub mod take_damage_view;
pub mod unit_attack_view;

/// The `combat/` subsystem's fields of [`crate::Sim`] (`docs/STRUCTURE.md`
/// §3, item 4016): moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct CombatState {
    /// The combat table — `docs/COMBAT.md` §5 — over [`Sim::unit_types`]
    /// ids. [`combat::Table::uniform`] until something builds one.
    pub table: crate::combat::Table,
    /// Ammo in flight.
    pub projectiles: Vec<crate::combat::Projectile>,
    /// The nukes' effects and what a nuke writes on its shooter's leader
    /// and silo (`crate::combat::nuke`, item 1091).
    pub nukes: crate::combat::nuke::Nukes,
    /// **The death objects**, the `DEATH_OBJS` list the dump prints at the
    /// frame level (`docs/COMBAT.md` §42.1). One is made by `Unit::close`
    /// for every unit death whose `dtype` is non-zero, and it carries the
    /// animation index the draw at `Unit::close+0xcb6` chose.
    ///
    /// Each figure's record ends at its life (`DeathObj::inc_time`,
    /// [`Sim::death_objs_inc_time`], `docs/GOLDEN.md` §127), and the pool
    /// of 100 behind them evicts the oldest non-blocking record when full
    /// ([`Self::death_pool`], §177).
    pub deaths: Vec<crate::combat::Death>,
    /// **The death pool's slots** (item 3971, `docs/GOLDEN.md` §177):
    /// which figure's record each of `Objects`' 100 `DeathObj` slots
    /// holds, in slot order, grown to [`combat::DEATH_POOL`] as records
    /// arrive. [`Self::deaths`] is the list the dump prints; this is the
    /// capacity behind it, and the order `Objects::add_death` evicts in.
    pub death_pool: Vec<Option<crate::combat::DeathSlot>>,
    /// One per player: the nation, wonder and patriot layer of the damage
    /// formula, as inputs.
    pub mods: Vec<crate::combat::Modifiers>,
    /// Every delivery of damage so far, newest last. Tests read it; nothing
    /// in the simulation does.
    pub hits: Vec<crate::combat::Hit>,
    /// `LeaderData::retargets` (`+0x9f4`): the frame's count of attacks
    /// whose target went invalid. `Leader::process@006b88b0` zeroes it
    /// at the head of each leader's frame, `Unit::fight@005fd4d0`'s
    /// invalid-target tail adds one, and `resolve_unit_collision`'s
    /// enemy ladder lets a captain re-search only while it is under ten
    /// (`docs/COLLISION.md` §14). Not dumped per frame at `LEADERS=5`.
    pub retargets: Vec<i32>,
    /// **A melee-and-ranged type's second range, swapped in for one call**
    /// (item 3383, `docs/AI.md` §291): `Unit::cavarch_fight@005ff4b0`
    /// writes the type's `second_max_range` (`UnitTypeData +0x2d0`) into
    /// its `max_range` (`+0x1fc`) before `fight` when the type's flags
    /// carry `0x400` ([`ai_load::uflags::MELEE_AND_RANGED`]), and 0 back after. The
    /// write is to the **type**, so every reader inside the call sees it;
    /// [`Sim::profile`] answers the type's second range for a unit whose
    /// type this holds. `None` outside such a call.
    pub(crate) second_range: Option<usize>,
}
