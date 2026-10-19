//! Buildings: the type, the construction clock, the site's hit points, repair
//! and refunds — `docs/CITIES.md` §1.5, §3, §9.
//!
//! A building type is a row of `buildingrules.xml` with a footprint, a flag
//! string and a lineage, and the arithmetic here is what turns that row into a
//! site that a builder advances and a rival knocks down. The nation, wonder and
//! tech layers enter as inputs — [`BuildMods`], [`ClockMods`], [`HitsMods`] —
//! the way every other mechanic takes them, because the layers that produce
//! them do not exist yet and the arithmetic is complete without them.

use crate::combat;
use crate::cost;
use crate::tech;
use crate::tuning::Tuning;

/// The `BUILD_FLAGS` letters the placement, counting and capture code test —
/// `docs/CITIES.md` §1.5. Letter `c` is bit `c − 'a'`, digit `d` is bit
/// `d − '0' + 25`.
pub mod flags {
    /// `a`: with `b`, domain 2. Nearly every shipped building carries `a`
    /// alone, where it means nothing to this mechanic.
    pub const DOCK_DOMAIN: u32 = 0x1;
    /// `b`: has a water domain.
    pub const WATER: u32 = 0x2;
    /// `c`: upgrades in place (`docs/TECH.md`).
    pub const UPGRADES: u32 = 0x4;
    /// `e`: does not need a city; counted in `num_buildings` even outside one;
    /// not converted on capture.
    pub const NO_CITY: u32 = 0x10;
    /// `f`: exempt from the friendly-territory test (no shipped building).
    pub const TERRITORY_EXEMPT: u32 = 0x20;
    /// `g`: a gatherer.
    pub const GATHER: u32 = 0x40;
    /// `j`: one per city.
    pub const ONE_PER_CITY: u32 = 0x200;
    /// `k`: always `BLOCKED_NEED_WALL` (unused).
    pub const NEED_WALL: u32 = 0x400;
    /// `l`: must stand inside a Large City or better (unused).
    pub const NEEDS_TOWN: u32 = 0x800;
    /// `n`: not civilian — exempt from conversion on capture.
    pub const NOT_CIVILIAN: u32 = 0x2000;
    /// `1` — **derived**: in an upgrade line. `Types::init` marks both the
    /// child and its `FROM` parent. Nothing reads it yet.
    pub const UPGRADE_LINE: u32 = 0x400_0000;
    /// `2` — **derived**, and the previous name for it, `DEEP_QUEUE`, was
    /// only half right: `TechType::set_research` marks every building in the
    /// lineage of a tech's `WHERE`, so this is "a technology is researched
    /// here" and the queue depth of 10 is one of its consequences
    /// (`docs/DATALAYER.md`, "The derived words no column carries").
    pub const RESEARCH_HERE: u32 = 0x800_0000;
    /// `3`: a flat gatherer (farm, oil) — derived, see [`init_final_flags`].
    pub const FLAT: u32 = 0x1000_0000;
    /// `4` — **derived**: `SpellType::init` marks a craft's `FROM`/`FROM2`
    /// when it is a building. Only the Small City carries it.
    pub const CRAFT_HERE: u32 = 0x2000_0000;
    /// `5` — **derived**: `UnitType::init` marks a producible unit's `WHERE`
    /// when the unit also has an `ATTACK`, is military, and the building is
    /// not in the city line. `BuildTypeData::is_military_trainer` reads it on
    /// the **root** of the `FROM` chain — [`is_military_trainer`].
    pub const MILITARY_TRAINER: u32 = 0x4000_0000;
    /// `6` — **derived**: `UnitType::init` marks the `WHERE` of every unit
    /// that is not `FLAGS p`. `is_training_building` reads it on the root —
    /// [`is_training_building`].
    pub const TRAINS: u32 = 0x8000_0000;

    /// Parses a `BUILD_FLAGS` string the way `BuildType::init` does: a letter
    /// `c` is bit `c − 'a'` and a digit **`1`..`9`** is bit `d + 25`. `0` is
    /// below the digit branch's own `< '1'` test and falls into the letter
    /// one, where `(0x30 − 0x61) & 0x1f` is 15; no shipped row has one.
    pub fn parse(s: &str) -> u32 {
        let mut f = 0;
        for c in s.chars() {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_lowercase() {
                f |= 1 << (c as u32 - 'a' as u32);
            } else if ('1'..='9').contains(&c) {
                f |= 1 << (c as u32 - '0' as u32 + 25);
            } else if c == '0' {
                f |= 1 << 15;
            }
        }
        f
    }
}

/// The named root of a type's lineage — the `TypeIndex` values the rules
/// test with `is(t, 0)`. A derived type (Keep, Castle, Large City) names its
/// own identity where the rules test it and otherwise `Other`, and reaches its
/// root through [`BuildType::from`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Ident {
    /// `VILLAGE` (0x19e) — the Small City, the root of every city.
    Village,
    /// `TOWN` (0x19f) — the Large City.
    Town,
    /// `METROPOLIS` (0x1a0) — the Major City.
    Metropolis,
    /// `FORBIDDENCITY` (0x213) — a wonder that is also a level-three city.
    ForbiddenCity,
    Farm,
    Woodcutter,
    Mine,
    University,
    OilWell,
    OilPlatform,
    Granary,
    Lumbermill,
    Smelter,
    Refinery,
    Library,
    Market,
    Temple,
    Senate,
    /// `TOWER` (0x1b7), the root of Keep and Stockade.
    Tower,
    /// `FORTX` (0x1bb), the root of Castle, Fortress, Redoubt and the Red Fort.
    Fort,
    Lookout,
    Barracks,
    Stable,
    AutoPlant,
    SiegeFactory,
    Factory,
    /// `DOCK`, the root of Anchorage and Shipyard.
    Dock,
    Airbase,
    MissileSilo,
    AirDefense,
    RedFort,
    /// A wonder with no rule of its own here.
    Wonder,
    #[default]
    Other,
}

/// A building's domain — `ObjectTypeData::domain` as `BuildType::set_domain`
/// derives it from the flags: 0 land, 1 water, 2 both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildDomain {
    #[default]
    Land,
    Water,
    Both,
}

/// A building type — one `BUILDING` record, as far as this mechanic reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildType {
    pub ident: Ident,
    /// The type this one upgrades from — its `FROM` line — as an index into
    /// [`crate::Sim::build_types`]. The lineage tests walk it.
    pub from: Option<usize>,
    /// The type this one upgrades to, if any.
    pub to: Option<usize>,
    /// The footprint, in tiles.
    pub x_size: i32,
    pub y_size: i32,
    /// `LOS` and `SCIENCE_LOS`, in tiles — what `Wall::update_los@0063eeb0`
    /// (the slot `Build` shares) reads for a finished building, and the
    /// input to the fog disc `Build::activate` throws
    /// (`docs/VISION.md` §2.1).
    pub los: i32,
    pub science_los: i32,
    /// The `BUILD_FLAGS` bits; see [`flags`].
    pub flags: u32,
    /// `JOB_TIME`: the construction time's base, in frames at `1/1`.
    pub job_time: i32,
    /// `HITS`.
    pub hits: i32,
    /// `GARRISON_MAX`.
    pub garrison_max: i32,
    /// `ATTACK` — non-zero makes a building defensive.
    pub attack: i32,
    /// `PLUNDER` and `PLUNDER_GOOD`, a resource index.
    pub plunder_value: i32,
    pub plunder_good: Option<usize>,
    /// Whether the type is a wonder (`PYRAMIDS..SPACEPROGRAM`).
    pub wonder: bool,
    /// `WONDER_VAL` (`BuildTypeData::wonder_val`, `+0x2d0`, read by
    /// `BuildType::init@00632340`): a wonder's points — Pyramids 1, Tikal
    /// 2, Kremlin 6. Only [`BuildType::wonder_points`] reads it.
    pub wonder_val: i32,
    /// The price, for refunds and repairs.
    pub price: cost::Price,
    /// Its entry in the tech tree, which gates `type_avail` for the city
    /// level-up.
    pub tree: Option<tech::TypeId>,
    /// Its combat columns, for a building that shoots — `docs/COMBAT.md`.
    pub combat: Option<combat::Profile>,
    /// `BuildData::get_shot@0062dd90`'s exact-type arms (`TypeIndex`, not
    /// a line): `Some(n)` for the types its switch names, `None` for every
    /// other type, which reads the owner's `ages` (`docs/COMBAT.md` §9.1).
    pub shot: Option<i32>,
    /// `BuildType::mask` — the per-tile blocking template
    /// `BuildType::init_build_mask@006310b0` reads, `x_size × y_size` bytes
    /// row-major by `y` (`mask[y × x_size + x]`), 1 where
    /// `BuildType::mask_me@006312a0` calls `World::set_blocked_at` and 0
    /// where it *clears* the bit. Empty when nothing loaded it, in which
    /// case [`BuildType::blocks`] falls back on [`flags::FLAT`].
    ///
    /// The names come from the graphic, not the rules row: `building_
    /// graphics.xml`'s `mask=` attribute selects one of the 36 named grids
    /// in `masks.txt` (`docs/DATALAYER.md`). It is the difference between a
    /// Mine (`2x2 solid`) and a Woodcutter's Camp (`2x2 gather`, which
    /// blocks nothing), and between a city's 7×7 footprint and the 6×6 it
    /// actually blocks (`7x7 extra space`).
    pub block_mask: Vec<u8>,
}

impl BuildType {
    pub const fn has(&self, f: u32) -> bool {
        self.flags & f != 0
    }

    /// `BuildType::set_domain`.
    pub const fn domain(&self) -> BuildDomain {
        if self.flags & flags::WATER != 0 {
            if self.flags & flags::DOCK_DOMAIN != 0 {
                BuildDomain::Both
            } else {
                BuildDomain::Water
            }
        } else {
            BuildDomain::Land
        }
    }

    /// `ObjectTypeData::get_wonder_value@00661ba0`, the slot `+0x118` of
    /// `BuildTypeData`'s vtable (`0xb428d4`, read off the PE's RTTI): the
    /// type's `WONDER_VAL` when `TypeData::is_wonder_type` answers — the
    /// `TypeIndex` range `0x20e..=0x21e` that is [`BuildType::wonder`] —
    /// and 0 for any other type. The Courtyard Fountain and the Red Fort
    /// carry a `WONDER_VAL` and are worth nothing here (`docs/AI.md` §124).
    pub const fn wonder_points(&self) -> i32 {
        if self.wonder { self.wonder_val } else { 0 }
    }

    /// `x_size × y_size`.
    pub const fn area(&self) -> i32 {
        self.x_size * self.y_size
    }

    /// `BuildTypeData::is_civilian`: neither `n` nor `e`.
    pub const fn is_civilian(&self) -> bool {
        self.flags & (flags::NOT_CIVILIAN | flags::NO_CITY) == 0
    }

    /// Whether the footprint tile `(u, v)` from the corner takes the blocked
    /// bit — [`BuildType::block_mask`] where one is loaded, and otherwise
    /// "everything but a flat type", which is what this crate assumed before
    /// the templates were read and is still what a hand-built type gets.
    pub fn blocks(&self, u: i32, v: i32) -> bool {
        if self.block_mask.is_empty() {
            return !self.has(flags::FLAT);
        }
        if !(0..self.x_size).contains(&u) || !(0..self.y_size).contains(&v) {
            return false;
        }
        usize::try_from(v * self.x_size + u)
            .ok()
            .and_then(|i| self.block_mask.get(i))
            .is_some_and(|&b| b == 1)
    }
}

/// The lineage test — `ObjectTypeData::is(t, strict = 0)`: `t` itself or
/// anything reached by walking `from`.
pub fn is(types: &[BuildType], t: usize, ident: Ident) -> bool {
    let mut cur = Some(t);
    let mut guard = 0;
    while let Some(i) = cur {
        if types[i].ident == ident {
            return true;
        }
        cur = types[i].from;
        guard += 1;
        if guard > types.len() {
            break;
        }
    }
    false
}

/// `BuildType::init_final_flags@00632070` — the **derived** `FLAT` bit.
///
/// `FLAT` is not a `BUILD_FLAGS` letter on any shipped row: the Farm's string
/// is `gda`, with no `3`. The loader ORs the bit in afterwards for three
/// lineages — `is(FARM) || is(OILWELL) || is(OILPLATFORM)` — and every
/// consumer reads it as `build_flags & 0x10000000` (the vtable's `is_flat`,
/// slot `+0x94`, is exactly `mov eax,[ecx+0x2c0]; and eax,0x10000000`).
///
/// Reading only the flag string therefore makes **nothing** flat, which sends
/// every farm through the non-flat wood/ore machine of `docs/ORDERS.md` §6.4
/// instead of the farm path of §6.5. Call this once per type after `from` is
/// linked. It is the tech audit's "read the loaders" lesson a third time.
pub fn init_final_flags(types: &mut [BuildType]) {
    for t in 0..types.len() {
        if is(types, t, Ident::Farm)
            || is(types, t, Ident::OilWell)
            || is(types, t, Ident::OilPlatform)
        {
            types[t].flags |= flags::FLAT;
        }
    }
}

/// The four building-index lists the derived `build_flags` bits are built
/// from — `docs/DATALAYER.md`, "The derived words no column carries".
///
/// Each is what the original's loader has in hand when it sets the bit: the
/// `WHERE` of every producible unit, the subset of those whose unit is a
/// military type outside the city line, the `WHERE` of every technology, and
/// the `FROM`/`FROM2` of every craft that names a building. Duplicates are
/// fine; the marks are idempotent.
#[derive(Clone, Debug, Default)]
pub struct Derived {
    pub trains: Vec<usize>,
    pub military_trains: Vec<usize>,
    pub research_at: Vec<usize>,
    pub craft_at: Vec<usize>,
}

/// The derived half of `build_flags`, in the order the original derives it:
/// [`flags::TRAINS`] and [`flags::MILITARY_TRAINER`] from `UnitType::init`'s
/// tail, [`flags::RESEARCH_HERE`] from `TechType::set_research` (which marks
/// the whole lineage of the `WHERE`, not just the named type),
/// [`flags::CRAFT_HERE`] from `SpellType::init`, [`flags::UPGRADE_LINE`] from
/// `Types::init` (both ends of every `FROM` link) and [`flags::FLAT`] from
/// `ObjectType::finalize_init_all`.
///
/// Reading only the flag string leaves **every** one of these clear, which
/// makes the Library's queue two deep, the Barracks no trainer, and half of
/// `create_buildings` unreachable. Call it once, after `from` is linked.
pub fn init_derived_flags(types: &mut [BuildType], d: &Derived) {
    for &b in &d.trains {
        types[b].flags |= flags::TRAINS;
    }
    for &b in &d.military_trains {
        types[b].flags |= flags::MILITARY_TRAINER;
    }
    for &root in &d.research_at {
        for t in 0..types.len() {
            if is_of(types, t, root) {
                types[t].flags |= flags::RESEARCH_HERE;
            }
        }
    }
    for &b in &d.craft_at {
        types[b].flags |= flags::CRAFT_HERE;
    }
    for t in 0..types.len() {
        if let Some(f) = types[t].from {
            types[t].flags |= flags::UPGRADE_LINE;
            types[f].flags |= flags::UPGRADE_LINE;
        }
    }
    init_final_flags(types);
}

/// `ObjectTypeData::is(root, 0)` over the `FROM` chain, by record index.
pub fn is_of(types: &[BuildType], t: usize, root: usize) -> bool {
    let mut cur = Some(t);
    let mut guard = 0;
    while let Some(c) = cur {
        if c == root {
            return true;
        }
        cur = types[c].from;
        guard += 1;
        if guard > types.len() {
            break;
        }
    }
    false
}

/// The root of a type's `FROM` chain — `BuildTypeData::fundamental_type`,
/// which is also what `basic_type` returns.
pub fn fundamental(types: &[BuildType], t: usize) -> usize {
    let mut cur = t;
    for _ in 0..types.len() {
        match types[cur].from {
            Some(f) => cur = f,
            None => break,
        }
    }
    cur
}

/// `BuildTypeData::is_military_trainer@0063bcf0` — the flag on the **root**.
pub fn is_military_trainer(types: &[BuildType], t: usize) -> bool {
    types[fundamental(types, t)].has(flags::MILITARY_TRAINER)
}

/// `BuildTypeData::is_training_building@00639de0` — likewise.
pub fn is_training_building(types: &[BuildType], t: usize) -> bool {
    types[fundamental(types, t)].has(flags::TRAINS)
}

/// `BuildTypeData::is_spellcaster@00639840` — [`flags::CRAFT_HERE`] on **any**
/// record of the `FROM` chain, the type's own first (unlike the two above,
/// which read the root alone; item 2083's sweep).
pub fn is_spellcaster(types: &[BuildType], t: usize) -> bool {
    let mut cur = t;
    for _ in 0..=types.len() {
        if types[cur].has(flags::CRAFT_HERE) {
            return true;
        }
        match types[cur].from {
            Some(f) => cur = f,
            None => return false,
        }
    }
    false
}

/// `Build::init@00629740`: the build queue's capacity. A trainer of any kind
/// holds twenty, a research building ten, and everything else **two** — the
/// last of which the first reading had as twenty.
pub fn queue_capacity(types: &[BuildType], t: usize) -> usize {
    if is_military_trainer(types, t) || is_training_building(types, t) {
        20
    } else if types[t].has(flags::RESEARCH_HERE) {
        10
    } else {
        2
    }
}

/// `BuildTypeData::is_city` = `is(VILLAGE, 0)`.
pub fn is_city(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Village)
}

/// `BuildTypeData::is_fort` = `is(FORTX, 0)`.
pub fn is_fort(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Fort)
}

/// `BuildTypeData::is_defensive@00473600`, and `WallData::is_defensive@00473650`, its body inlined on the
/// wall's record (`+0x18`): `is(TOWER, 0)`, `is(FORTX, 0)` and `is(AIRBASE, 0)` asked through the record's slot
/// `+0x60` **in that order**, the first true answering, and with none of them `attack` (`+0x1e8`) non-zero
/// (item 2083's sweep holds the order and the short circuit). `is` answers a lineage test.
pub fn is_defensive_by(mut is: impl FnMut(Ident) -> bool, attack: i32) -> bool {
    is(Ident::Tower) || is(Ident::Fort) || is(Ident::Airbase) || attack != 0
}

/// `BuildTypeData::is_gather_enhancer@00472b50`: `is(GRANARY, 0)`, `is(LUMBERMILL, 0)`, `is(SMELTER, 0)` and
/// `is(REFINERY, 0)` (`0x1a7..=0x1aa`) asked through the record's slot `+0x60` **in that order**, the first true
/// answering (item 2090's sweep holds the order and the short circuit). `is` answers a lineage test.
pub fn is_gather_enhancer_by(mut is: impl FnMut(Ident) -> bool) -> bool {
    is(Ident::Granary) || is(Ident::Lumbermill) || is(Ident::Smelter) || is(Ident::Refinery)
}

/// [`is_defensive_by`] on the port's records.
pub fn is_defensive(types: &[BuildType], t: usize) -> bool {
    is_defensive_by(|i| is(types, t, i), types[t].attack)
}

/// `BuildTypeData::is_tower` = `is(TOWER, 0)`.
pub fn is_tower(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Tower)
}

/// `BuildTypeData::is_dock` = `is(DOCK, 0)`.
pub fn is_dock(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Dock)
}

/// `BuildTypeData::get_city_level`: 1 for a city, 2 for a Large City, 3 for a
/// Major City or the Forbidden City; 0 for anything that is not a city.
pub fn city_level(types: &[BuildType], t: usize) -> i32 {
    match types[t].ident {
        Ident::Town => 2,
        Ident::Metropolis | Ident::ForbiddenCity => 3,
        _ if is_city(types, t) => 1,
        _ => 0,
    }
}

/// `CityData::upgrades_to`: the next level's type, found by ident among
/// `types`, or `None` at the top.
pub fn upgrades_to(types: &[BuildType], t: usize) -> Option<usize> {
    let next = match types[t].ident {
        Ident::Metropolis | Ident::ForbiddenCity => return None,
        Ident::Village => Ident::Town,
        _ => Ident::Metropolis,
    };
    types.iter().position(|b| b.ident == next)
}

// ----------------------------------------------------------------------
// The construction clock
// ----------------------------------------------------------------------

/// The nation, wonder and tech layer of `Wall::update_construct_time`,
/// evaluated **once, at placement** — `docs/CITIES.md` §3.2.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildMods {
    pub maya: bool,
    /// `has_preq(BUILDINGS_CREATED_FASTER)`.
    pub created_faster: bool,
    pub versailles: bool,
    pub tobacco: bool,
    pub british: bool,
    pub dutch: bool,
    pub romans: bool,
    /// The owner has no city yet — `city_num == 0` — at placement.
    pub no_city: bool,
    /// `LeaderData::get_building_speed_upgrade`, 0..3.
    pub speed_upgrade: i32,
}

/// `Wall::update_construct_time`: the stored base, `constr_time`. Evaluated
/// at placement and again whenever the owner's wall stats are recomputed
/// (`Leader::calc_wall_stats`, on the `0x8000000` dirty flag), so a tech or a
/// first city arriving mid-build does reach a site.
pub fn construct_base(t: &Tuning, types: &[BuildType], ty: usize, m: &BuildMods) -> i32 {
    let b = &types[ty];
    let mut c = b.job_time * 100;
    if m.maya && !b.wonder {
        c = c * 100 / (t.maya_building_speed + 100);
    }
    if m.created_faster {
        c = (c * 3) >> 2;
    }
    if m.versailles {
        c = c * 100 / (t.versailles_building_speed + 100);
    }
    if m.tobacco {
        c = c * 100 / (t.tobacco_building_speed + 100);
    }
    if m.british && b.ident == Ident::AirDefense {
        c = c * 100 / (t.british_aa_speed + 100);
    }
    if m.dutch && is_fort(types, ty) && !b.wonder {
        c = c * 100 / (t.dutch_fort_speed + 100);
    }
    if m.romans && is_fort(types, ty) && !b.wonder {
        c = c * 100 / (t.roman_fort_speed + 100);
    }
    if m.no_city {
        c = t.capital_build_time * c / 100;
    }
    (10 - m.speed_upgrade.clamp(0, 3)) * c / 10
}

/// The four per-call clauses of `BuildData::construct_time`, as inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockMods {
    /// An American's first wonder with no rival building the same one.
    pub free_first_wonder: bool,
    /// The owner's Hanging Gardens stand in this building's city.
    pub hanging_gardens: bool,
    /// The President is near.
    pub president: bool,
    /// Iroquois, a senate, and no senate ever finished.
    pub iroquois_first_senate: bool,
}

/// `BuildData::construct_time(0)`: what `job_counter` is measured against.
/// Never below 1.
pub fn construct_time(t: &Tuning, ty: &BuildType, base: i32, m: &ClockMods) -> i32 {
    let mut c = base;
    if ty.wonder && m.free_first_wonder {
        c = 1;
    }
    if !ty.wonder && m.hanging_gardens {
        c = (100 - t.hanging_gardens_build_time) * c / 100;
    }
    if m.president {
        c = c * 100 / (t.thepresident_building_speed + 100);
    }
    if m.iroquois_first_senate && ty.ident == Ident::Senate && t.iroquois_quick_senate != 0 {
        c = 0;
    }
    c.max(1)
}

/// What one builder adds this frame, before the harmonic share —
/// `Unit::do_build`: `ACCEL_CONSTRUCT`, quartered under attack unless the
/// Koreans' bonus applies.
pub const fn builder_amount(t: &Tuning, under_attack: bool, korean: bool) -> i32 {
    let a = t.accel_construct;
    if under_attack && !(korean && t.korean_build_under_fire != 0) {
        // The sign-fixed `>> 2` of the original.
        a.div_euclid(4)
    } else {
        a
    }
}

/// `Wall::do_construct`'s share for one builder: `max(1, amount / (helpers +
/// 1))`, `helpers` being how many already contributed this frame.
pub const fn contribution(amount: i32, helpers: i32) -> i32 {
    let s = amount / (helpers + 1);
    if s < 1 { 1 } else { s }
}

/// `v` rounded to the nearest `f32` (ties to even), as an integer: what
/// `cvtdq2ps` / `cvtpd2ps` make of an integer below 2^40.
fn f32_round(v: i64) -> i64 {
    let mag = v.unsigned_abs();
    let bits = 64 - mag.leading_zeros();
    if bits <= 24 {
        return v;
    }
    let shift = bits - 24;
    let (q, r) = (mag >> shift, mag & ((1 << shift) - 1));
    let half = 1u64 << (shift - 1);
    let q = if r > half || (r == half && q & 1 == 1) {
        q + 1
    } else {
        q
    };
    let out = (q << shift) as i64;
    if v < 0 { -out } else { out }
}

/// The test at the tail of `Wall::do_construct@006434d0`: `recalc_builds`
/// is raised when the job counter crossed 0.84 of the construct time this
/// call — `(float)new / (float)ct > 0.84f` and `(float)old / (float)ct <=
/// 0.84f`, `old` read signed (`cvtdq2ps`), the others unsigned through a
/// double, each division a single-precision one. `0x3f570a3d` is
/// `0xD70A3D * 2^-24` with an odd mantissa, so a quotient is above it exactly
/// when it reaches the midpoint to the next float, `0x1AE147B * 2^-25` (a
/// tie rounds to the even neighbour, which is above); a zero divisor is an
/// infinity or a NaN, and a NaN fails both tests.
pub fn recalc_crossed(old: i32, new: u32, ct_old: u32, ct_new: u32) -> bool {
    const MID: i128 = 0x1AE_147B;
    let (old, new) = (f32_round(i64::from(old)), f32_round(i64::from(new)));
    let (ct_old, ct_new) = (f32_round(i64::from(ct_old)), f32_round(i64::from(ct_new)));
    let above = if ct_new == 0 {
        new > 0
    } else {
        i128::from(new) << 25 >= MID * i128::from(ct_new)
    };
    let at_most = if ct_old == 0 {
        old < 0
    } else {
        i128::from(old) << 25 < MID * i128::from(ct_old)
    };
    above && at_most
}

// ----------------------------------------------------------------------
// Hit points
// ----------------------------------------------------------------------

/// The multipliers of `Wall::update_hits`, as inputs — `docs/CITIES.md` §3.4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitsMods {
    pub maya: bool,
    pub romans: bool,
    /// `LeaderData::get_building_hp_upgrade`, 0..3.
    pub hp_upgrade: i32,
    pub taj_mahal: bool,
    pub red_fort: bool,
    pub nubians: bool,
    pub tikal: bool,
    /// The temple border tech level, 1..4, of a player whose city has a
    /// temple; 0 without.
    pub temple_level: i32,
}

/// The constants `Wall::update_hits@0063f0d0` reads, by their `Constants`
/// offsets: `+0x58c`, `+0x5fc`, `+0x154`, `+0x4f8`, `+0x4c8`, `+0x5cc`,
/// `+0x4a0`, `+0xc48`, `+0xb08` and the temple table at `+0x158`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitsConsts {
    pub maya: i32,
    pub roman: i32,
    pub upgrade: i32,
    pub taj: i32,
    pub red_fort: i32,
    pub nubian: i32,
    pub tikal: i32,
    pub senate: i32,
    /// `ctw_missionaries_bonus`, read only under the Conquer the World
    /// semaphore, which this port does not carry (0 here).
    pub ctw: i32,
    pub temple: [i32; 5],
}

impl HitsConsts {
    pub fn of(t: &Tuning) -> Self {
        Self {
            maya: t.maya_building_hp,
            roman: t.roman_fort_hp,
            upgrade: t.building_hp_upgrade,
            taj: t.taj_building_hp,
            red_fort: t.red_fort_fort_hps,
            nubian: t.nubian_hit_points,
            tikal: t.tikal_temple_hp,
            senate: t.senate_hp_bonus,
            ctw: 0,
            temple: t.temple_upgrade_hp,
        }
    }
}

/// What `Wall::update_hits` asks of the world, one method a call it makes, in
/// the order it makes them (`Wall::update_hits@0063f0d0`; item 2400's sweep
/// holds the order, the arguments and the short circuits).
pub(crate) trait HitsAsk {
    /// `Object::update_hits(param)` — the type's hits.
    fn base_hits(&mut self, param: i32) -> i32;
    /// `LeaderData::has_tribe_bonus(n)`.
    fn tribe(&mut self, n: i32) -> bool;
    /// The type record's slot `+0xfc`: is a fort.
    fn is_fort(&mut self) -> bool;
    /// `is(id, 0)`: slot `+0xb8` of the object, or — it being the base
    /// `ObjectData::is` — slot `+0x60` of the type record.
    fn is_line(&mut self, id: i32) -> bool;
    /// The object's slot `+0x2c`.
    fn wonder_flag(&mut self) -> bool;
    /// `LeaderData::get_building_hp_upgrade`.
    fn hp_upgrade(&mut self) -> i32;
    /// `LeaderData::has_wonder(id)`.
    fn has_wonder(&mut self, id: i32) -> bool;
    /// The object's slot `+0x20`.
    fn slot20(&mut self) -> bool;
    /// The object's slot `+0x4c`, `WallData::is_active` (`flags & 4`).
    fn is_active(&mut self) -> bool;
    /// The owning city's index (`+0x72` of the objects table's slot `+0xac`
    /// answer) is not negative.
    fn city_index_ok(&mut self) -> bool;
    /// `CityData::get_level` of that city.
    fn city_level(&mut self) -> i32;
    /// That city's flag `+4 & 0x80`: a Temple.
    fn city_has_temple(&mut self) -> bool;
    /// `LeaderData::has_preq(TEMPLEHP4)`.
    fn has_preq_temple4(&mut self) -> bool;
    /// `LeaderData::get_temple_hp`.
    fn temple_hp(&mut self) -> i32;
    /// `game->semaphore.ptr[2] & 2`: the Conquer the World mode.
    fn ctw_semaphore(&mut self) -> bool;
    /// `LeaderData::has_conquest_bonus(0x20)`.
    fn has_conquest_missionaries(&mut self) -> bool;
    /// The object's slot `+0x18c` called with 0: `construct_time(0)`.
    fn construct_time(&mut self) -> u32;
    /// `ObjectData::can_carry(AIR)`.
    fn can_carry_air(&mut self) -> bool;
    /// `Object::eject_contents(1, -1, 0, 1)`.
    fn eject_contents(&mut self);
}

/// `Wall::update_hits`'s own fields: `+8 & 0x20` (a city), `+0x48` the job
/// counter (read unsigned), `+0x24` the damage taken, `+0x28 >= 0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HitsSelf {
    pub city_flag: bool,
    pub job_counter: u32,
    pub taken: i32,
    pub slot28_nonneg: bool,
    pub param: i32,
}

/// What `Wall::update_hits` wrote and answered: `+0x20`, `+0x54`, the answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HitsOut {
    pub full: i32,
    pub current: i32,
    pub ret: i32,
}

/// `(k + 100) * h / 100` in the original's 32-bit signed arithmetic.
const fn hits_pct(k: i32, h: i32) -> i32 {
    k.wrapping_add(100).wrapping_mul(h) / 100
}

/// `Wall::update_hits@0063f0d0` through `+0x20`: the type's hits through the
/// multipliers — `docs/CITIES.md` §3.4.
pub(crate) fn full_hits_steps(c: &HitsConsts, me: &HitsSelf, q: &mut impl HitsAsk) -> i32 {
    let mut h = q.base_hits(me.param);
    if q.tribe(1) {
        h = hits_pct(c.maya, h);
    }
    if q.tribe(6) && (q.is_fort() || q.is_line(0x1b7)) && !q.wonder_flag() {
        h = hits_pct(c.roman, h);
    }
    let up = q.hp_upgrade();
    h = c.upgrade.wrapping_mul(up).wrapping_add(100).wrapping_mul(h) / 100;
    if q.has_wonder(0x21b) {
        h = hits_pct(c.taj, h);
    }
    if q.has_wonder(0x216) && q.is_fort() && !q.wonder_flag() {
        h = hits_pct(c.red_fort, h);
    }
    if q.is_line(0x1b4) && q.tribe(4) && c.nubian != 0 {
        h = hits_pct(c.nubian, h);
    }
    if !me.city_flag {
        // Not a city: the Senate's bonus, once it is built and standing in a
        // city, and not a fort, tower or lookout.
        if q.slot20()
            && q.is_active()
            && q.city_index_ok()
            && !q.is_fort()
            && !q.is_line(0x1b7)
            && !q.is_line(0x209)
        {
            let level = q.city_level();
            h = h.wrapping_add(c.senate.wrapping_mul(level.wrapping_sub(1)).wrapping_mul(h) / 100);
        }
    } else if q.is_active() && q.city_index_ok() && q.city_has_temple() {
        let k = if q.has_preq_temple4() {
            4
        } else {
            q.temple_hp()
        };
        let mut bonus = c.temple[(k.clamp(1, 5) - 1) as usize];
        if q.has_wonder(0x214) {
            bonus = c
                .tikal
                .wrapping_add(100)
                .wrapping_mul(bonus)
                .wrapping_add(99)
                / 100;
        }
        h = hits_pct(bonus, h);
        if q.ctw_semaphore() && q.has_conquest_missionaries() {
            h = hits_pct(c.ctw, h);
        }
    }
    h
}

/// The whole of `Wall::update_hits@0063f0d0`: [`full_hits_steps`], the site's
/// growing figure ([`site_hits`], while not active), and the ejection of a
/// garrison a hit has outgrown.
pub(crate) fn update_hits_steps(c: &HitsConsts, me: &HitsSelf, q: &mut impl HitsAsk) -> HitsOut {
    let full = full_hits_steps(c, me, q);
    let mut cur = full;
    if !q.is_active() {
        let ct = q.construct_time();
        if me.job_counter < ct {
            cur = site_hits(full, me.job_counter as i32, ct as i32, q.wonder_flag());
        }
    }
    if me.taken >= cur && me.slot28_nonneg && !q.can_carry_air() {
        q.eject_contents();
    }
    HitsOut {
        full,
        current: cur,
        ret: if me.param != 0 { full } else { cur },
    }
}

/// [`HitsAsk`] on the port's records: the building type, the nation's
/// layers as [`HitsMods`], and what the city says.
pub(crate) struct PortAsk<'a> {
    pub types: &'a [BuildType],
    pub ty: usize,
    pub active: bool,
    pub city_level: i32,
    pub has_temple: bool,
    pub m: &'a HitsMods,
    pub construct_time: u32,
    /// Set by `eject_contents`.
    pub ejects: bool,
}

impl HitsAsk for PortAsk<'_> {
    fn base_hits(&mut self, _param: i32) -> i32 {
        self.types[self.ty].hits
    }
    fn tribe(&mut self, n: i32) -> bool {
        match n {
            1 => self.m.maya,
            6 => self.m.romans,
            4 => self.m.nubians,
            _ => false,
        }
    }
    fn is_fort(&mut self) -> bool {
        is_fort(self.types, self.ty)
    }
    fn is_line(&mut self, id: i32) -> bool {
        match id {
            // The Tower and the Lookout are read as lines, `is(0x1b7, 0)` and
            // `is(0x209, 0)`: the Keep, the Stockade and the Bunker are
            // Towers here (item 1275). The Market is read by its ident.
            0x1b7 => is_tower(self.types, self.ty),
            0x209 => is(self.types, self.ty, Ident::Lookout),
            0x1b4 => self.types[self.ty].ident == Ident::Market,
            _ => false,
        }
    }
    fn wonder_flag(&mut self) -> bool {
        self.types[self.ty].wonder
    }
    fn hp_upgrade(&mut self) -> i32 {
        self.m.hp_upgrade.clamp(0, 3)
    }
    fn has_wonder(&mut self, id: i32) -> bool {
        match id {
            0x21b => self.m.taj_mahal,
            0x216 => self.m.red_fort,
            0x214 => self.m.tikal,
            _ => false,
        }
    }
    fn slot20(&mut self) -> bool {
        true
    }
    fn is_active(&mut self) -> bool {
        self.active
    }
    fn city_index_ok(&mut self) -> bool {
        self.city_level > 0
    }
    fn city_level(&mut self) -> i32 {
        self.city_level
    }
    fn city_has_temple(&mut self) -> bool {
        self.has_temple && self.m.temple_level > 0
    }
    fn has_preq_temple4(&mut self) -> bool {
        self.m.temple_level == 4
    }
    fn temple_hp(&mut self) -> i32 {
        self.m.temple_level
    }
    fn ctw_semaphore(&mut self) -> bool {
        false
    }
    fn has_conquest_missionaries(&mut self) -> bool {
        false
    }
    fn construct_time(&mut self) -> u32 {
        self.construct_time
    }
    fn can_carry_air(&mut self) -> bool {
        false
    }
    fn eject_contents(&mut self) {
        self.ejects = true;
    }
}

/// `Wall::update_hits`'s full-health figure, `myhits`: the type's hits
/// through the multipliers. `city_level` is the level of the city the
/// building stands in (0 outside one); `has_temple` whether that city has a
/// temple (read only for a city building).
pub fn full_hits(
    t: &Tuning,
    types: &[BuildType],
    ty: usize,
    active: bool,
    city_level: i32,
    has_temple: bool,
    m: &HitsMods,
) -> i32 {
    let mut q = PortAsk {
        types,
        ty,
        active,
        city_level,
        has_temple,
        m,
        construct_time: 0,
        ejects: false,
    };
    let me = HitsSelf {
        city_flag: is_city(types, ty),
        ..HitsSelf::default()
    };
    full_hits_steps(&HitsConsts::of(t), &me, &mut q)
}

/// The construction scaling of `update_hits`: a site's current full-health
/// figure, `construct_hits`. Linear from ~0 to full; a wonder from half.
pub const fn site_hits(full: i32, job_counter: i32, construct_time: i32, wonder: bool) -> i32 {
    // The original compares and shifts them unsigned (`jae`, `shr`).
    let (jc, ct) = (job_counter as u32, construct_time as u32);
    if jc >= ct {
        return full;
    }
    let a = if (jc >> 5) < 1 { 1 } else { (jc >> 5) as i32 };
    let b = if (ct >> 5) < 1 { 1 } else { (ct >> 5) as i32 };
    if !wonder {
        let h = a.wrapping_mul(full) / b;
        if h < 1 { 1 } else { h }
    } else {
        let grown = (full / 2).wrapping_mul(a) / b;
        full.wrapping_add(1) / 2 + if grown < 1 { 1 } else { grown }
    }
}

/// `Object::take_damage` on a site: `lost × 50` off `job_counter` per hit,
/// `lost` the whole points taken after the sixteenths carry (item 1350).
pub const fn progress_lost(whole: i32) -> i32 {
    whole * 50
}

// ----------------------------------------------------------------------
// Refund and repair
// ----------------------------------------------------------------------

/// `Object::disband`'s partial refund for an unfinished building: the
/// fraction of the price not yet built. The original computes `cost × (ct −
/// jc2) / ct` in `float`; this is the pinned rational, which agrees except
/// within rounding of an integer boundary (`docs/CITIES.md` §11).
pub const fn refund(cost: i32, construct_time: i32, job_counter_2: i32) -> i32 {
    if construct_time <= 0 {
        return cost;
    }
    let left = construct_time - job_counter_2;
    if left <= 0 {
        return 0;
    }
    ((cost as i64) * (left as i64) / (construct_time as i64)) as i32
}

/// What a repairer needs to know about its target — `Unit::do_repair`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RepairTarget {
    /// `helpers` already counted this frame.
    pub helpers: i32,
    /// `construct_time(0)`.
    pub construct_time: i32,
    /// `hits(0)`.
    pub hits: i32,
    /// A city building; and whether it is a captured one (`race != who`).
    pub city: bool,
    pub captured: bool,
    /// `build_masks & 0x20` and `& 0x10`.
    pub under_attack: bool,
    pub fresh_hit: bool,
}

/// The repair period, in frames × 256 per hit point.
pub fn repair_period(t: &Tuning, r: &RepairTarget, korean: bool) -> i64 {
    let denom = i64::from(r.hits.max(1)) * i64::from(t.accel_construct.max(1));
    let mut period = i64::from(r.helpers + 1) * ((i64::from(r.construct_time) << 9) / denom);
    if r.city {
        period *= if r.captured { 4 } else { 2 };
    }
    if korean && t.korean_repair != 0 {
        period = i64::from(100 - t.korean_repair) * period / 100;
    }
    if !(korean && t.korean_build_under_fire != 0) {
        if r.under_attack {
            period <<= 2;
        }
        if r.fresh_hit {
            period <<= 2;
        }
    }
    period
}

/// The hit points repaired this frame: one every `period / 256` frames. A
/// period below one repairs everything at once.
pub const fn repair_amount(frame: i64, period: i64, damage: i32) -> i32 {
    if period < 1 {
        return damage;
    }
    ((frame * 256) / period - (frame * 256 - 256) / period) as i32
}

/// The building-side attrition — `Wall::process`, every 32 frames phased by
/// `o` (16 only under rush rules before war is allowed, which is not
/// modelled): eight hit points through `take_damage` with the attrition flag
/// for a started building on a tile an enemy owns; an unstarted one is
/// simply removed. `docs/CITIES.md` §9.5.
pub const ENEMY_TERRITORY_DAMAGE: i32 = 8;
pub const ENEMY_TERRITORY_PERIOD: i64 = 32;

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Tuning {
        Tuning::RON
    }

    /// The template, and the fallback for a type nothing loaded one for.
    #[test]
    fn the_blocking_template_is_read_row_major_by_y() {
        // `7x7 extra space`, cut down: a 3×3 footprint blocking 2×2.
        let mut b = BuildType {
            x_size: 3,
            y_size: 3,
            block_mask: vec![1, 1, 0, 1, 1, 0, 0, 0, 0],
            ..BuildType::default()
        };
        assert!(b.blocks(0, 0) && b.blocks(1, 1));
        assert!(!b.blocks(2, 0), "the last column");
        assert!(!b.blocks(0, 2), "the last row");
        assert!(!b.blocks(3, 0), "off the template");
        // No template: the old flat/non-flat line, which every hand-built
        // type in this crate still takes.
        b.block_mask.clear();
        assert!(b.blocks(2, 2));
        b.flags |= flags::FLAT;
        assert!(!b.blocks(0, 0));
    }

    fn types() -> Vec<BuildType> {
        let mk = |ident, from, job, hits, wonder| BuildType {
            ident,
            from,
            job_time: job,
            hits,
            wonder,
            x_size: 4,
            y_size: 4,
            ..BuildType::default()
        };
        vec![
            mk(Ident::Village, None, 600, 1200, false),
            mk(Ident::Town, Some(0), 600, 2500, false),
            mk(Ident::Metropolis, Some(1), 600, 5000, false),
            mk(Ident::Fort, None, 2000, 2000, false),
            mk(Ident::Other, Some(3), 2000, 2800, false), // Castle
            mk(Ident::RedFort, Some(3), 3000, 6000, true),
            mk(Ident::Tower, None, 1000, 750, false),
            mk(Ident::Other, Some(6), 1000, 1000, false), // Keep
            mk(Ident::Wonder, None, 4000, 2000, true),    // Pyramids
            mk(Ident::Senate, None, 500, 1200, false),
            mk(Ident::Market, None, 420, 1200, false),
            mk(Ident::ForbiddenCity, Some(0), 5000, 3000, true),
        ]
    }

    #[test]
    fn the_derived_flags_are_marked_where_the_loader_marks_them() {
        let mut ty = types();
        // The Fort is a military trainer and a research building; a tech is
        // researched at the Tower line's root; the Village trains and holds a
        // craft.
        let d = Derived {
            trains: vec![0, 3],
            military_trains: vec![3],
            research_at: vec![6, 9],
            craft_at: vec![0],
        };
        init_derived_flags(&mut ty, &d);
        assert!(ty[0].has(flags::TRAINS) && ty[0].has(flags::CRAFT_HERE));
        assert!(!ty[0].has(flags::MILITARY_TRAINER));
        assert!(ty[3].has(flags::MILITARY_TRAINER) && ty[3].has(flags::TRAINS));
        // `RESEARCH_HERE` walks the lineage: the Keep gets it from the Tower.
        assert!(ty[6].has(flags::RESEARCH_HERE) && ty[7].has(flags::RESEARCH_HERE));
        assert!(!ty[8].has(flags::RESEARCH_HERE));
        // The upgrade mark lands on both ends of every `FROM`.
        assert!(ty[0].has(flags::UPGRADE_LINE) && ty[1].has(flags::UPGRADE_LINE));
        assert!(!ty[9].has(flags::UPGRADE_LINE));
        // And the three predicates read the root, not the type.
        assert!(is_military_trainer(&ty, 4), "the Castle's root is the Fort");
        assert!(
            is_training_building(&ty, 2),
            "the Major City's is the Village"
        );
        assert_eq!(queue_capacity(&ty, 2), 20);
        assert_eq!(queue_capacity(&ty, 7), 10, "a research building holds ten");
        assert_eq!(queue_capacity(&ty, 8), 2, "everything else holds two");
    }

    #[test]
    fn flags_parse_letters_and_digits_the_loaders_way() {
        assert_eq!(
            flags::parse("ean"),
            flags::NO_CITY | 0x1 | flags::NOT_CIVILIAN
        );
        assert_eq!(flags::parse("jam"), flags::ONE_PER_CITY | 0x1 | 0x1000);
        assert_eq!(flags::parse("gda"), flags::GATHER | 0x8 | 0x1);
        assert_eq!(
            flags::parse("ebn") & (flags::WATER | flags::NO_CITY),
            flags::WATER | flags::NO_CITY
        );
        assert_eq!(flags::parse("3"), flags::FLAT);
        assert_eq!(flags::parse("2"), flags::RESEARCH_HERE);
    }

    #[test]
    fn the_domain_is_derived_from_two_flags() {
        let mut b = BuildType::default();
        assert_eq!(b.domain(), BuildDomain::Land);
        // A dock is `ebn`: water. No shipped building carries `a` with `b`.
        b.flags = flags::parse("ebn");
        assert_eq!(b.domain(), BuildDomain::Water);
        b.flags = flags::parse("igbe");
        assert_eq!(b.domain(), BuildDomain::Water);
        b.flags = flags::parse("ab");
        assert_eq!(b.domain(), BuildDomain::Both);
    }

    #[test]
    fn lineage_walks_from_and_levels_follow_the_ident() {
        let ty = types();
        assert!(is_city(&ty, 0) && is_city(&ty, 1) && is_city(&ty, 2) && is_city(&ty, 11));
        assert!(!is_city(&ty, 3));
        assert!(is_fort(&ty, 4) && is_fort(&ty, 5) && !is_fort(&ty, 6));
        assert!(is_tower(&ty, 7) && !is_tower(&ty, 3));
        assert_eq!(city_level(&ty, 0), 1);
        assert_eq!(city_level(&ty, 1), 2);
        assert_eq!(city_level(&ty, 2), 3);
        assert_eq!(city_level(&ty, 11), 3);
        assert_eq!(city_level(&ty, 3), 0);
        assert_eq!(upgrades_to(&ty, 0), Some(1));
        assert_eq!(upgrades_to(&ty, 1), Some(2));
        assert_eq!(upgrades_to(&ty, 2), None);
        assert_eq!(upgrades_to(&ty, 11), None);
    }

    #[test]
    fn an_unmodified_type_takes_job_time_frames_at_one_accel() {
        let ty = types();
        let base = construct_base(&t(), &ty, 0, &BuildMods::default());
        assert_eq!(base, 60_000);
        let ct = construct_time(&t(), &ty[0], base, &ClockMods::default());
        assert_eq!(ct, 60_000);
        // One builder a frame: exactly job_time frames.
        let mut jc = 0;
        let mut frames = 0;
        while jc < ct {
            jc += contribution(builder_amount(&t(), false, false), 0);
            frames += 1;
        }
        assert_eq!(frames, 600);
    }

    #[test]
    fn the_first_city_of_a_nomad_takes_three_times_as_long() {
        let ty = types();
        let m = BuildMods {
            no_city: true,
            ..BuildMods::default()
        };
        assert_eq!(construct_base(&t(), &ty, 0, &m), 180_000);
    }

    #[test]
    fn speed_techs_and_nations_stack_in_the_loaders_order() {
        let ty = types();
        let m = BuildMods {
            maya: true,
            created_faster: true,
            speed_upgrade: 3,
            ..BuildMods::default()
        };
        // 60000 × 100/120 = 50000; ×3>>2 = 37500; ×7/10 = 26250.
        assert_eq!(construct_base(&t(), &ty, 0, &m), 26_250);
        // A Roman fort: 200000 × 100/150 = 133333.
        let m = BuildMods {
            romans: true,
            ..BuildMods::default()
        };
        assert_eq!(construct_base(&t(), &ty, 3, &m), 133_333);
        // And the castle, through the lineage.
        assert_eq!(construct_base(&t(), &ty, 4, &m), 133_333);
        // But not the Red Fort, a wonder.
        assert_eq!(construct_base(&t(), &ty, 5, &m), 300_000);
    }

    #[test]
    fn the_per_call_clauses() {
        let ty = types();
        let base = 60_000;
        assert_eq!(
            construct_time(
                &t(),
                &ty[8],
                base,
                &ClockMods {
                    free_first_wonder: true,
                    ..Default::default()
                }
            ),
            1
        );
        // The Gardens ship at 0 %: no change.
        assert_eq!(
            construct_time(
                &t(),
                &ty[0],
                base,
                &ClockMods {
                    hanging_gardens: true,
                    ..Default::default()
                }
            ),
            60_000
        );
        assert_eq!(
            construct_time(
                &t(),
                &ty[0],
                base,
                &ClockMods {
                    president: true,
                    ..Default::default()
                }
            ),
            45_112
        );
        // The Iroquois first senate: 0, floored to 1 — done on the first contribution.
        assert_eq!(
            construct_time(
                &t(),
                &ty[9],
                base,
                &ClockMods {
                    iroquois_first_senate: true,
                    ..Default::default()
                }
            ),
            1
        );
        // The same flag on a market does nothing.
        assert_eq!(
            construct_time(
                &t(),
                &ty[10],
                base,
                &ClockMods {
                    iroquois_first_senate: true,
                    ..Default::default()
                }
            ),
            60_000
        );
    }

    #[test]
    fn builders_are_harmonic_and_the_under_attack_quarter_is_sign_fixed() {
        assert_eq!(contribution(100, 0), 100);
        assert_eq!(contribution(100, 1), 50);
        assert_eq!(contribution(100, 2), 33);
        assert_eq!(contribution(100, 3), 25);
        assert_eq!(contribution(3, 5), 1);
        assert_eq!(builder_amount(&t(), true, false), 25);
        assert_eq!(builder_amount(&t(), true, true), 100);
        assert_eq!(builder_amount(&t(), false, false), 100);
    }

    #[test]
    fn a_site_grows_linearly_and_a_wonder_from_half() {
        assert_eq!(site_hits(1200, 0, 60_000, false), 1);
        // 937 × 1200 / 1875: the `>> 5` truncation shows.
        assert_eq!(site_hits(1200, 30_000, 60_000, false), 599);
        assert_eq!(site_hits(1200, 60_000, 60_000, false), 1200);
        assert_eq!(site_hits(2000, 0, 400_000, true), 1001);
        assert_eq!(site_hits(2000, 200_000, 400_000, true), 1500);
        assert_eq!(site_hits(2000, 400_000, 400_000, true), 2000);
        // A tiny clock still works through the >> 5 guard.
        assert_eq!(site_hits(100, 0, 1, false), 100);
    }

    #[test]
    fn the_senate_bonus_and_the_temple_bonus() {
        let ty = types();
        let m = HitsMods::default();
        // A market in a Large City: 1200 + 35 % = 1620; in a Major City 1200 + 70 % = 2040.
        assert_eq!(full_hits(&t(), &ty, 10, true, 2, false, &m), 1620);
        assert_eq!(full_hits(&t(), &ty, 10, true, 3, false, &m), 2040);
        // Not before it is active, and not outside a city.
        assert_eq!(full_hits(&t(), &ty, 10, false, 2, false, &m), 1200);
        assert_eq!(full_hits(&t(), &ty, 10, true, 0, false, &m), 1200);
        // A tower in a Large City: no senate bonus.
        assert_eq!(full_hits(&t(), &ty, 6, true, 2, false, &m), 750);
        // Nor a Keep: `is(0x1b7, 0)` reads the line, and run488's Keep
        // `1/2015` in a Large City holds 1000 (item 1275).
        assert_eq!(full_hits(&t(), &ty, 7, true, 2, false, &m), 1000);
        // A city with a level-one temple: +25 %; with Tikal, (150 × 25 + 99) / 100 = 38 → +38 %.
        let m = HitsMods {
            temple_level: 1,
            ..Default::default()
        };
        assert_eq!(full_hits(&t(), &ty, 0, true, 1, true, &m), 1500);
        let m = HitsMods {
            temple_level: 1,
            tikal: true,
            ..Default::default()
        };
        assert_eq!(full_hits(&t(), &ty, 0, true, 1, true, &m), 1656);
        // Three HP techs: +30 %.
        let m = HitsMods {
            hp_upgrade: 3,
            ..Default::default()
        };
        assert_eq!(full_hits(&t(), &ty, 0, false, 0, false, &m), 1560);
    }

    #[test]
    fn the_refund_is_what_was_not_built() {
        assert_eq!(refund(120, 60_000, 0), 120);
        assert_eq!(refund(120, 60_000, 30_000), 60);
        assert_eq!(refund(120, 60_000, 60_000), 0);
        assert_eq!(refund(120, 60_000, 70_000), 0);
        assert_eq!(refund(100, 3, 1), 66);
    }

    #[test]
    fn a_full_repair_takes_twice_the_build_time() {
        let r = RepairTarget {
            helpers: 0,
            construct_time: 60_000,
            hits: 1200,
            ..Default::default()
        };
        // (60000 << 9) / (1200 × 100) = 256: one hit point a frame, 1200 frames for 1200 hits
        // against 600 frames to build.
        let p = repair_period(&t(), &r, false);
        assert_eq!(p, 256);
        assert_eq!(repair_amount(7, p, 1200), 1);
        // A city doubles it, a captured city quadruples it.
        let c = RepairTarget { city: true, ..r };
        assert_eq!(repair_period(&t(), &c, false), 512);
        let c = RepairTarget {
            city: true,
            captured: true,
            ..r
        };
        assert_eq!(repair_period(&t(), &c, false), 1024);
        // Under attack, freshly hit: × 16.
        let u = RepairTarget {
            under_attack: true,
            fresh_hit: true,
            ..r
        };
        assert_eq!(repair_period(&t(), &u, false), 4096);
        // Koreans under fire shrug it off and repair in half the time.
        assert_eq!(repair_period(&t(), &u, true), 128);
        // A second repairer's own period is longer: harmonic, like builders.
        let two = RepairTarget { helpers: 1, ..r };
        assert_eq!(repair_period(&t(), &two, false), 512);
        // A period below one repairs everything now.
        assert_eq!(repair_amount(3, 0, 77), 77);
    }

    #[test]
    fn repair_amount_is_one_point_every_period_frames() {
        let p = 4 * 256;
        let total: i32 = (1..=40).map(|f| repair_amount(f, p, 1000)).sum();
        assert_eq!(total, 10);
    }
}
