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

use crate::attrition::Domain;
use crate::movement::Angle;
use crate::tuning::Tuning;
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
pub mod role {
    /// `PEASANTS`/`PEASANTSKOREAN`/`SCHOLARS`/`SCHOLARSKOREAN` — citizens,
    /// scholars: half against buildings outside their borders.
    pub const CITIZEN: u32 = 1 << 0;
    /// `is(MILITIA)`.
    pub const MILITIA: u32 = 1 << 1;
    /// `CASTLE`/`FORTX`: a fort whose arrows turn to bullets with Gunpowder.
    pub const FORT: u32 = 1 << 2;
    /// A city building (object flag `0x20`).
    pub const CITY: u32 = 1 << 3;
    /// `is(V2ROCKET)`.
    pub const V2ROCKET: u32 = 1 << 4;
    /// `is(REDFORT, strict)`.
    pub const REDFORT: u32 = 1 << 5;
    /// The type's `where` is the Factory (Wellington's bonus).
    pub const FACTORY_MADE: u32 = 1 << 6;
    /// The type's `where` is the Barracks (the Japanese bonus).
    pub const BARRACKS_MADE: u32 = 1 << 7;
    /// The type's `where` is the Stable (Cossacks).
    pub const STABLE_MADE: u32 = 1 << 8;
    /// `is(BARK)`: quarter splash.
    pub const BARK: u32 = 1 << 9;
    /// `is(CATAPULT)`: halves overkill from a non-siege attacker.
    pub const CATAPULT: u32 = 1 << 10;
    /// `is(FLAMETHROWER)`: ignores entrenchment, ejects garrisons.
    pub const FLAMETHROWER: u32 = 1 << 11;
    /// The `SUPERCOLLIDER` wonder.
    pub const SUPERCOLLIDER: u32 = 1 << 12;
    /// `is(BOMBARD)`: the artillery reload class.
    pub const BOMBARD: u32 = 1 << 13;
    /// Merchant, Dutch merchant, fur trapper.
    pub const MERCHANT: u32 = 1 << 14;
    /// A supply unit (`UnitData::is_supply`).
    pub const SUPPLY: u32 = 1 << 15;
    /// A caravan (`UnitData::is_caravan`).
    pub const CARAVAN: u32 = 1 << 16;
    /// `is(IMMORTALS)`: a melee type whose special strike lands inline.
    pub const IMMORTALS: u32 = 1 << 17;
    /// A `GUN`-flagged ship that does not broadside — `PATROLBOAT`.
    pub const PATROLBOAT: u32 = 1 << 18;
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
    pub target_size: i32,
    pub guy_radius: i32,
    pub domain: Domain,
    pub siege: bool,
    pub packs: bool,
    /// The type's age, for the combat table's age bonus.
    pub age: i32,
    /// A building type's footprint, in tiles.
    pub x_size: i32,
    pub y_size: i32,
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
    /// `role & 0x10000` — a combat unit, for the target ranking and `on_duty`.
    pub combat_role: bool,
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
    /// The overkill record (§7.1 step 2): the frame of the first hit in the
    /// current window and the captain index of whoever struck it.
    pub damage_frame: i64,
    pub damage_o: i32,
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

/// The side of the square of cells a splash searches (§9.3): the spiral
/// table `move_x/move_y` walked to `radius[k]`, `k = splash_area / 4 + 1`
/// capped at 10, which visits the `(2k+1)²` cells within Chebyshev distance
/// `k` of the landing cell.
pub const fn splash_cells(splash_area: i32) -> i32 {
    let k = splash_area / 4 + 1;
    if k > 10 { 10 } else { k }
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
    let mut armor = armor;
    let mut amask = a.obj_masks;
    let tmask = tp.obj_masks;

    // 1. Gunpowder arrows.
    if (a.is(role::FORT) || a.is(role::CITY)) && m.gunpowder {
        amask &= !mask::ARCHERY;
    }
    // 2.
    let mut base = attack * pct / 100;
    // 3.
    if amask & mask::MUSKET_INF != 0 {
        armor = armor * 133 / 100;
    }
    // 4.
    if at.build_proper && tmask & mask::SIEGE != 0 && tt.unit && tp.packs && !tt.packed {
        base /= 3;
    }
    // 5.
    if at.build_proper && tt.unit && tt.attacks && tt.moving {
        if tmask & mask::FOOT == 0 {
            if tp.siege || tp.is(role::SUPPLY) || tp.is(role::CARAVAN) {
                base /= 2;
            }
        } else {
            base = (base * 3) / 4;
        }
    }
    // 6.
    if at.build_proper && tt.build_proper && tt.under_construction {
        base *= 4;
    }
    // 7.
    if tt.build_proper && (a.is(role::CITIZEN) || a.is(role::MILITIA)) && !tt.tile_owned_by_attacker
    {
        base /= 2;
    }
    // 8.
    if a.is(role::V2ROCKET) && tt.build_proper && tt.under_construction {
        base /= 2;
    }
    // 9.
    if at.unit
        && matches!(a.domain, Domain::Air)
        && amask & mask::MISSILE == 0
        && tp.is(role::REDFORT)
    {
        base = base * (100 - t.red_fort_air_defense) / 100;
    }
    // 10.
    if tt.unit {
        if tt.packed {
            base *= 2;
        }
        if tp.packs && !tt.packed {
            armor += 1;
        }
    }
    // 11. dtype — cosmetic, not carried.
    // 12. Wellington.
    if at.unit && m.wellington && tp.is(role::FACTORY_MADE) {
        base += t.wellington_siege_attack;
    }
    // 13. Japanese.
    if at.unit
        && a.is(role::BARRACKS_MADE)
        && let Some(n) = m.japanese_level
    {
        let mut j = t.japanese_damage;
        if j < 0 {
            j = -(j * n);
        }
        base = (j + 100) * base / 100;
    }
    // 14. Double damage.
    if (at.build_proper || (matches!(a.domain, Domain::Land) && !matches!(tp.domain, Domain::Land)))
        && tt.unit
        && (tt.mask_400000 || (matches!(tp.domain, Domain::Sea) && tt.team_untargeted_ship))
    {
        base *= 2;
    }
    // 15. Splash.
    if splash {
        if tt.unit {
            base /= tp.uber_size.max(1);
        }
        base = a.splash_percent * base / 100;
        if tt.unit {
            if tp.siege && tp.packs && !tt.packed {
                base *= 3;
            }
            if tp.is(role::BARK) {
                base = base * 25 / 100;
            }
        }
    }
    // 16. River.
    if tt.unit && tt.z < 0 {
        base = shr8(base * t.river_modifier);
    }
    // 17. Decoy.
    if tt.unit && tt.decoy {
        armor = 0;
        base = attack.max(base) * 1000;
    }
    // 18. Unfriendly territory.
    if tt.building && tt.in_unfriendly_territory {
        armor = 0;
        if !tt.build_proper || tp.attack == 0 {
            base *= 4;
        }
    }
    // 19. Flanking.
    if at.unit
        && tt.unit
        && amask & mask::CIVILIAN == 0
        && tmask & mask::CIVILIAN == 0
        && amask & mask::AIR == 0
        && tmask & mask::AIR == 0
        && (amask & mask::NAVAL) == (tmask & mask::NAVAL)
        && tmask & mask::NAVAL == 0
        && let Some(level) = flank_level(tt.facing, angle)
    {
        let mut fb = t.flank_bonus;
        if amask & mask::VEHICLE != 0 {
            fb = shr8(fb * t.vehicle_flank_bonus);
        } else if amask & mask::MOUNTED != 0 {
            fb = shr8(fb * t.cavalry_flank_bonus);
        }
        base = (fb * level + 100) * base / 100;
    }
    // 20. Net damage.
    let mut dmg = (base + 5) / 10 - armor;
    // 21. Overkill.
    if a.is_ranged()
        && at.unit
        && tt.unit
        && tt.damage_frame != 0
        && frame - tt.damage_frame < i64::from(t.overkill_frames)
        && at.captain != tt.damage_o
    {
        dmg = shr8(dmg * t.overkill_damage);
        if tp.is(role::CATAPULT) && !a.siege {
            dmg /= 2;
        }
    }
    // 22. Rocky.
    if tmask & (mask::LIGHT_INF | mask::MODERN_INF | mask::MUSKET_INF) != 0 && tt.rocky {
        dmg = shr8(dmg * t.rocky_modifier);
    }
    // 23. Height.
    if !matches!(tp.domain, Domain::Air)
        && !matches!(a.domain, Domain::Air)
        && !a.siege
        && tt.z < at.z
    {
        dmg += (at.z - tt.z) * t.height_bonus * dmg / (t.height_increment * 100);
    }
    // 24. Entrenchment.
    if tt.unit && tt.entrenched && !a.is(role::FLAMETHROWER) {
        let d = tt.trench_facing.0.wrapping_sub(angle.0) as u32;
        let c = if d.wrapping_add(0x5555_5556) < 0xaaaa_aaac {
            1 + u32::from(d.wrapping_add(0x2000_0000) > 0x4000_0000)
        } else {
            0
        };
        if splash || c == 0 {
            dmg = shr8(dmg * t.entrenchment_modifier);
            if tt.antipater_entrenched {
                dmg = shr8(dmg * t.antipater_entrench_bonus);
            }
        }
    }
    // 25. Cossacks.
    if t.russian_cossack_damage != 0
        && m.russian
        && at.unit
        && a.is(role::STABLE_MADE)
        && (tp.is(role::SUPPLY) || tp.siege)
    {
        dmg = (t.russian_cossack_damage + 100) * dmg / 100;
    }
    // 26. At least one.
    if dmg < 1
        && (!matches!(a.domain, Domain::Land) || !matches!(tp.domain, Domain::Sea))
        && (tmask & mask::AIR != 0) == (amask & mask::ANTI_AIR != 0)
        && !splash
    {
        dmg = 1;
    }
    // 27.
    if tp.is(role::SUPERCOLLIDER) && matches!(a.domain, Domain::Air) && t.super_immune != 0 {
        dmg = 0;
    }
    // 28. Recapture.
    if tt.build_proper && tt.recapturable_city {
        dmg = shr8(dmg * t.recapture_city_modifier);
    }
    dmg
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
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    fn unit_profile(attack_tenths: i32, armor: i32, masks: u32) -> Profile {
        Profile {
            attack: attack_tenths,
            armor,
            obj_masks: masks,
            uber_size: 1,
            ammo_per_att: 1,
            splash_percent: 100,
            ..Profile::default()
        }
    }

    /// A unit facing south — toward an attack travelling north, so that the
    /// default angle of the calls below is head-on and no flank applies.
    fn unit_side() -> Side {
        Side {
            unit: true,
            facing: Angle::SOUTH,
            ..Side::default()
        }
    }

    #[test]
    fn the_rng_is_the_lcg_and_a_degenerate_range_does_not_advance() {
        let mut r = Rng::new(1);
        let s0 = r.seed;
        assert_eq!(r.get(7, 7), 7);
        assert_eq!(r.seed, s0);
        let v = r.roll();
        assert_eq!(r.seed, 1u32.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f));
        assert_eq!(v, (((r.seed & 0xffff) * 0xffff) >> 16) as i32);
        assert!(v < 0xffff);
        // lo > hi swaps.
        let mut a = Rng::new(99);
        let mut b = Rng::new(99);
        assert_eq!(a.get(10, 3), b.get(3, 10));
    }

    #[test]
    fn obj_mask_letters_parse_as_the_loader_does() {
        assert_eq!(mask::parse("A"), mask::ARMORED);
        assert_eq!(mask::parse("fh"), mask::FOOT | mask::HEAVY_INF);
        assert_eq!(mask::parse("3"), mask::AIR);
        assert_eq!(mask::parse("6"), mask::ANTI_AIR);
        assert_eq!(mask::parse("FW "), mask::FOOT | mask::MELEE);
    }

    #[test]
    fn base_damage_is_attack_times_table_then_rounded_to_hits_minus_armor() {
        let a = unit_profile(120, 0, mask::FOOT | mask::HEAVY_INF);
        let b = unit_profile(100, 3, mask::FOOT | mask::LIGHT_INF);
        // 12 attack, 100 %: (120 + 5) / 10 − 3 = 9.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                unit_side(),
                a.attack,
                b.armor,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            9
        );
        // 150 %: 180 → (180 + 5) / 10 − 3 = 15.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                unit_side(),
                a.attack,
                b.armor,
                150,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            15
        );
        // Armour that exceeds the attack gives at least one.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                unit_side(),
                a.attack,
                30,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            1
        );
        // …unless it is splash.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                unit_side(),
                a.attack,
                30,
                100,
                Angle::NORTH,
                true,
                1,
                &Modifiers::default()
            ),
            -18
        );
    }

    #[test]
    fn musket_infantry_face_a_third_more_armour_and_anti_air_never_scratches_the_ground() {
        let a = unit_profile(100, 0, mask::MUSKET_INF);
        let b = unit_profile(100, 6, mask::FOOT);
        // armour 6 × 133 / 100 = 7: (100 + 5) / 10 − 7 = 3.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                unit_side(),
                100,
                6,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            3
        );
        let aa = unit_profile(10, 0, mask::ANTI_AIR);
        assert_eq!(
            get_damage(
                &T,
                &aa,
                unit_side(),
                &b,
                unit_side(),
                10,
                5,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            -4
        );
    }

    #[test]
    fn flanking_is_rear_one_side_two_and_front_nothing() {
        // Target faces north. An attack travelling north comes from behind.
        assert_eq!(flank_level(Angle::NORTH, Angle::NORTH), Some(1));
        // Travelling south it meets the target's face.
        assert_eq!(flank_level(Angle::NORTH, Angle::SOUTH), None);
        // From the sides.
        assert_eq!(flank_level(Angle::NORTH, Angle::EAST), Some(2));
        assert_eq!(flank_level(Angle::NORTH, Angle::WEST), Some(2));
        // 59° off the face is still the face; 61° is a flank.
        let deg = |d: i32| {
            Angle(
                Angle::SOUTH
                    .0
                    .wrapping_add(crate::movement::ONE_DEGREE.wrapping_mul(d)),
            )
        };
        assert_eq!(flank_level(Angle::NORTH, deg(59)), None);
        assert_eq!(flank_level(Angle::NORTH, deg(61)), Some(2));
        // 44° off the back is the back; 46° is a side.
        let back = |d: i32| {
            Angle(
                Angle::NORTH
                    .0
                    .wrapping_add(crate::movement::ONE_DEGREE.wrapping_mul(d)),
            )
        };
        assert_eq!(flank_level(Angle::NORTH, back(44)), Some(1));
        assert_eq!(flank_level(Angle::NORTH, back(46)), Some(2));

        let a = unit_profile(100, 0, mask::FOOT);
        let b = unit_profile(100, 0, mask::FOOT);
        let t = Side {
            facing: Angle::NORTH,
            ..unit_side()
        };
        // Rear: (50 × 1 + 100) × 100 / 100 = 150 → 15.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                t,
                100,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            15
        );
        // Side: 200 → 20.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                t,
                100,
                0,
                100,
                Angle::EAST,
                false,
                1,
                &Modifiers::default()
            ),
            20
        );
        // A mounted attacker's bonus is 40/256 of that: 50 × 40 >> 8 = 7, rear 107 → 11.
        let cav = unit_profile(100, 0, mask::MOUNTED);
        assert_eq!(
            get_damage(
                &T,
                &cav,
                unit_side(),
                &b,
                t,
                100,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            11
        );
        // Civilians neither flank nor are flanked.
        let civ = unit_profile(100, 0, mask::CIVILIAN);
        assert_eq!(
            get_damage(
                &T,
                &civ,
                unit_side(),
                &b,
                t,
                100,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            10
        );
    }

    #[test]
    fn overkill_cuts_a_second_ranged_squads_hit_to_a_third_inside_the_window() {
        let mut a = unit_profile(300, 0, mask::FOOT_ARCHER);
        a.max_range = 5;
        let b = unit_profile(100, 0, mask::FOOT);
        let first = Side {
            captain: 1,
            ..unit_side()
        };
        let target = Side {
            damage_frame: 100,
            damage_o: 1,
            ..unit_side()
        };
        // The owner of the window: full 30.
        assert_eq!(
            get_damage(
                &T,
                &a,
                first,
                &b,
                target,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                110,
                &Modifiers::default()
            ),
            30
        );
        // Another squad inside it: 30 × 85 >> 8 = 9.
        let second = Side {
            captain: 2,
            ..unit_side()
        };
        assert_eq!(
            get_damage(
                &T,
                &a,
                second,
                &b,
                target,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                110,
                &Modifiers::default()
            ),
            9
        );
        // After 30 frames the window is gone.
        assert_eq!(
            get_damage(
                &T,
                &a,
                second,
                &b,
                target,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                130,
                &Modifiers::default()
            ),
            30
        );
        // A melee attacker neither suffers it…
        let mut m = a;
        m.max_range = 0;
        assert_eq!(
            get_damage(
                &T,
                &m,
                second,
                &b,
                target,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                110,
                &Modifiers::default()
            ),
            30
        );
        // …and a catapult target halves it again for a non-siege attacker.
        let mut cat = b;
        cat.roles = role::CATAPULT;
        assert_eq!(
            get_damage(
                &T,
                &a,
                second,
                &cat,
                target,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                110,
                &Modifiers::default()
            ),
            4
        );
    }

    #[test]
    fn terrain_and_entrenchment_apply_to_the_net() {
        let a = unit_profile(300, 0, mask::FOOT);
        let b = unit_profile(100, 0, mask::LIGHT_INF);
        let rocky = Side {
            rocky: true,
            ..unit_side()
        };
        // 30 × 170 >> 8 = 19.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                rocky,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            19
        );
        // Height: attacker 400 above → + 400 × 10 × 30 / (200 × 100) = +6.
        let high = Side {
            z: 400,
            ..unit_side()
        };
        assert_eq!(
            get_damage(
                &T,
                &a,
                high,
                &b,
                unit_side(),
                300,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            36
        );
        // Entrenched facing north, attacked from the north (the attack
        // travels south, and the unit faces north into it): protected,
        // 30 × 170 >> 8 = 19.
        let trench = Side {
            entrenched: true,
            trench_facing: Angle::NORTH,
            facing: Angle::NORTH,
            ..unit_side()
        };
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                trench,
                300,
                0,
                100,
                Angle::SOUTH,
                false,
                1,
                &Modifiers::default()
            ),
            19
        );
        // Attacked from behind (the attack travels north): the trench does
        // not help, and it is a rear flank besides: 45.
        assert_eq!(
            get_damage(
                &T,
                &a,
                unit_side(),
                &b,
                trench,
                300,
                0,
                100,
                Angle::NORTH,
                false,
                1,
                &Modifiers::default()
            ),
            45
        );
    }

    #[test]
    fn scaling_splits_into_whole_and_sixteenths() {
        // A unit of three firing for 30: each figure's shot is 10.
        assert_eq!(
            scale(30, 0x100, true, true, 1, 3),
            Sixteenths { whole: 10, frac: 0 }
        );
        // For 31: 31 × 256 / 3 = 2645 → 165 sixteenths → 10 + 5/16.
        assert_eq!(
            scale(31, 0x100, true, true, 1, 3),
            Sixteenths { whole: 10, frac: 5 }
        );
        // Nothing below one whole hit before the division.
        assert_eq!(
            scale(0, 0x100, true, false, 1, 4),
            Sixteenths { whole: 0, frac: 4 }
        );
        assert_eq!(
            scale(-5, 0x100, true, false, 1, 1),
            Sixteenths { whole: 1, frac: 0 }
        );
        // A building's volley divides by ammo per attack and has no floor.
        assert_eq!(
            scale(8, 0x100, false, true, 4, 1),
            Sixteenths { whole: 2, frac: 0 }
        );
        // A splash fringe at half count.
        assert_eq!(
            scale(10, 0x80, true, true, 1, 1),
            Sixteenths { whole: 5, frac: 0 }
        );
    }

    #[test]
    fn a_figure_takes_its_share_and_the_last_captain_the_remainder() {
        assert_eq!(share(100, 3, false), 33);
        assert_eq!(share(100, 3, true), 34);
        assert_eq!(share(90, 3, true), 30);
        assert_eq!(share(50, 1, false), 50);
        // Sixteenths carry.
        let (t, d, f) = take(0, 12, 33, Sixteenths { whole: 0, frac: 8 });
        assert_eq!((t, d, f), (Taken::Alive { lost: 1 }, 1, 4));
        // A hit of nothing is a sixteenth.
        let (t, d, f) = take(0, 0, 33, Sixteenths { whole: 0, frac: 0 });
        assert_eq!((t, d, f), (Taken::Alive { lost: 0 }, 0, 1));
        // Death at the share.
        let (t, ..) = take(30, 0, 33, Sixteenths { whole: 3, frac: 0 });
        assert_eq!(
            t,
            Taken::Died {
                lost: 3,
                overflow: 0
            }
        );
        let (t, ..) = take(30, 0, 33, Sixteenths { whole: 10, frac: 0 });
        assert_eq!(
            t,
            Taken::Died {
                lost: 10,
                overflow: 7
            }
        );
    }

    #[test]
    fn recharge_and_accuracy_and_scatter() {
        assert_eq!(recharge(40, false, false), 40);
        assert_eq!(recharge(40, true, false), 60);
        assert_eq!(recharge(40, true, true), 80);
        // The byte.
        assert_eq!(recharge(200, true, true), 400 & 0xff);
        // Accuracy: to_hit 80, attenuate 1, ten tiles = 1920 units → 80 − 10;
        // a tile and a half is still one whole tile.
        assert_eq!(accuracy(80, 1, 1920), 70);
        assert_eq!(accuracy(80, 1, 287), 79);
        assert_eq!(accuracy(3, 0, 0), 5);
        // Scatter at 60 %: 96 × 100 / (8 + 60) = 141; at 120 %: 96 × 100 /
        // (−4 + 120) = 82 → quartered 20.
        assert_eq!(scatter(&T, 60, true, false, false), 141);
        assert_eq!(scatter(&T, 120, true, false, false), 20);
        assert_eq!(scatter(&T, 60, false, false, false), 0xc0);
        assert_eq!(scatter(&T, 60, true, true, false), 282);
        assert_eq!(scatter(&T, 60, true, false, true), 0);
        // Two draws, or none.
        let mut r = Rng::new(5);
        let s0 = r.seed;
        assert_eq!(
            scatter_point(&mut r, Pos::new(100, 100), 1),
            Pos::new(100, 100)
        );
        assert_eq!(r.seed, s0);
        let p = scatter_point(&mut r, Pos::new(100, 100), 10);
        assert!((p.x - 100).abs() <= 5 && (p.y - 100).abs() <= 5);
        assert_ne!(r.seed, s0);
    }

    #[test]
    fn flight_time_matches_the_float_expression() {
        // Spot-checked against (int)(sqrtf(n) / (float)d) evaluated in
        // single precision.
        let check = |n: i64, d: i64, want: i32| {
            assert_eq!(flight_time(n, d), want, "n={n} d={d}");
            // The reference, in f32, for the values where the host's sqrt is
            // correctly rounded (it is, for IEEE hosts).
            let r = ((n as f32).sqrt() / d as f32) as i32;
            assert_eq!(flight_time(n, d), r, "host n={n} d={d}");
        };
        check(0, 100, 0);
        check(1, 1, 1);
        check(1_000_000, 100, 10);
        check(999_999, 100, 9);
        check(2_000_000, 137, 10);
        check(36_864 * 36_864 + 1920 * 1920, 200, 184);
        for n in [3, 7, 123_456, 9_999_991, 1 << 30, (1i64 << 40) + 17] {
            for d in [1, 3, 96, 200, 777, 4_000] {
                let r = ((n as f32).sqrt() / d as f32) as i32;
                assert_eq!(flight_time(n, d), r, "n={n} d={d}");
            }
        }
        assert_eq!(siege_flight_time(10, 200, 1), 1920 / 200);
    }

    #[test]
    fn the_hit_tests_and_splash_count() {
        assert!(hits_unit(Pos::new(0, 0), Pos::new(30, 0), 48, 50));
        assert!(!hits_unit(Pos::new(0, 0), Pos::new(60, 0), 48, 50));
        // Over 100 % accuracy the distance is halved.
        assert!(hits_unit(Pos::new(0, 0), Pos::new(60, 0), 48, 101));
        assert!(hits_building(Pos::new(95, 0), Pos::new(0, 0), 1, 1));
        assert!(!hits_building(Pos::new(97, 0), Pos::new(0, 0), 1, 1));
        assert_eq!(splash_count(0, 2), 0x100);
        assert_eq!(splash_count(192, 2), 0x80);
        assert_eq!(splash_count(384, 2), 0);
        assert_eq!(splash_count(-10, 2), 0x100);
    }

    #[test]
    fn attack_dist_is_edge_to_edge_on_the_quarter_tile_grid() {
        let u = (24 + 0x18, 24 + 0x18);
        // Two figures 4 tiles apart on the x axis, each with block radius
        // 24: 768 − 48 − 48 = 672.
        assert_eq!(
            attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, u, false),
            672
        );
        // Inside the grid step nothing changes; one step out, one step more.
        assert_eq!(
            attack_dist(Pos::new(47, 0), Pos::new(768, 0), u, u, false),
            672
        );
        assert_eq!(
            attack_dist(Pos::new(48, 0), Pos::new(768, 0), u, u, false),
            624
        );
        // Overlapping extents clamp to zero.
        assert_eq!(attack_dist(Pos::new(0, 0), Pos::new(60, 0), u, u, false), 0);
        // A 2×2 building's extent is a tile each way.
        let b = (2 * 0x60, 2 * 0x60);
        assert_eq!(
            attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, b, false),
            768 - 192 - 48
        );
        // A plane ignores extents.
        assert_eq!(
            attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, u, true),
            768
        );
        // Range: melee 102; ranged 3 tiles = 582 inclusive.
        assert!(in_range(102, 0, 0, false, 0, false));
        assert!(!in_range(103, 0, 0, false, 0, false));
        assert!(in_range(246, 0, 0, true, 0, false));
        assert!(in_range(3 * 192 + 6, 3, 0, false, 0, false));
        assert!(!in_range(3 * 192 + 7, 3, 0, false, 0, false));
        // Inside the minimum, rescued by the big radii.
        assert!(!in_range(100, 5, 1, false, 0, false));
        assert!(in_range(100, 5, 1, false, 100, false));
        // Rings.
        assert_eq!(ring_of(0, 0), 0);
        assert_eq!(ring_of(1, 0), 1);
        assert_eq!(ring_of(1, 1), 1);
        assert_eq!(ring_of(2, 1), 2);
        assert_eq!(ring_of(2, 2), 3);
    }

    #[test]
    fn garrison_arrows() {
        // A tower with attack 8, base 1, most 4: empty → 1; two archers of
        // attack 6 each → 1 + 12/8 = 2; capped.
        assert_eq!(super::garrison_arrows(80, 1, 4, 0), 1);
        assert_eq!(super::garrison_arrows(80, 1, 4, 12), 2);
        assert_eq!(super::garrison_arrows(80, 1, 4, 400), 5);
        // No base: half the attack is added first.
        assert_eq!(super::garrison_arrows(80, 0, 4, 0), 0);
        assert_eq!(super::garrison_arrows(80, 0, 4, 4), 1);
        // An attack under a whole hit fires nothing at all.
        assert_eq!(super::garrison_arrows(3, 1, 4, 100), 0);
    }
}

#[cfg(test)]
mod table_tests {
    use super::{Table, TypeRef};

    #[test]
    fn two_families_and_growth_keep_every_cell() {
        let t = Table::build(2, 2, |a, b| match (a, b) {
            (TypeRef::Unit(x), TypeRef::Unit(y)) => 10 + (x * 2 + y) as i32,
            (TypeRef::Unit(x), TypeRef::Build(y)) => 20 + (x * 2 + y) as i32,
            (TypeRef::Build(x), TypeRef::Unit(y)) => 30 + (x * 2 + y) as i32,
            (TypeRef::Build(x), TypeRef::Build(y)) => 40 + (x * 2 + y) as i32,
        });
        assert_eq!(t.pct(1, 0), 12);
        assert_eq!(t.pct_of(TypeRef::Unit(0), TypeRef::Build(1)), 21);
        assert_eq!(t.pct_of(TypeRef::Build(1), TypeRef::Unit(1)), 33);
        assert_eq!(t.pct_of(TypeRef::Build(1), TypeRef::Build(1)), 43);
        assert_eq!(t.pct_of(TypeRef::Build(2), TypeRef::Unit(0)), 100);
        assert_eq!(t.pct(5, 0), 100);
        let g = t.grown(3, 3);
        assert_eq!(g.width(), 3);
        assert_eq!(g.builds(), 3);
        assert_eq!(g.pct(1, 0), 12);
        assert_eq!(g.pct_of(TypeRef::Build(1), TypeRef::Build(1)), 43);
        assert_eq!(g.pct_of(TypeRef::Unit(2), TypeRef::Build(2)), 100);
        assert_eq!(g.pct_of(TypeRef::Build(2), TypeRef::Unit(0)), 100);
        // Growing never shrinks.
        assert_eq!(g.grown(1, 1), g);
    }
}
