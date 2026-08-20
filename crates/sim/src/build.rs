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
    /// `2`: queue depth 10.
    pub const DEEP_QUEUE: u32 = 0x800_0000;
    /// `3`: a flat gatherer (farm, oil).
    pub const FLAT: u32 = 0x1000_0000;

    /// Parses a `BUILD_FLAGS` string the way `BuildType::init` does.
    pub fn parse(s: &str) -> u32 {
        let mut f = 0;
        for c in s.chars() {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_lowercase() {
                f |= 1 << (c as u32 - 'a' as u32);
            } else if c.is_ascii_digit() {
                f |= 1 << (c as u32 - '0' as u32 + 25);
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
    /// The price, for refunds and repairs.
    pub price: cost::Price,
    /// Its entry in the tech tree, which gates `type_avail` for the city
    /// level-up.
    pub tree: Option<tech::TypeId>,
    /// Its combat columns, for a building that shoots — `docs/COMBAT.md`.
    pub combat: Option<combat::Profile>,
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

    /// `x_size × y_size`.
    pub const fn area(&self) -> i32 {
        self.x_size * self.y_size
    }

    /// `BuildTypeData::is_civilian`: neither `n` nor `e`.
    pub const fn is_civilian(&self) -> bool {
        self.flags & (flags::NOT_CIVILIAN | flags::NO_CITY) == 0
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

/// `BuildTypeData::is_city` = `is(VILLAGE, 0)`.
pub fn is_city(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Village)
}

/// `BuildTypeData::is_fort` = `is(FORTX, 0)`.
pub fn is_fort(types: &[BuildType], t: usize) -> bool {
    is(types, t, Ident::Fort)
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

/// `Wall::update_construct_time`: the stored base, `constr_time`.
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
    let b = &types[ty];
    let mut h = b.hits;
    if m.maya {
        h = (t.maya_building_hp + 100) * h / 100;
    }
    if m.romans && (is_fort(types, ty) || b.ident == Ident::Tower) && !b.wonder {
        h = (t.roman_fort_hp + 100) * h / 100;
    }
    h = (t.building_hp_upgrade * m.hp_upgrade.clamp(0, 3) + 100) * h / 100;
    if m.taj_mahal {
        h = (t.taj_building_hp + 100) * h / 100;
    }
    if m.red_fort && is_fort(types, ty) && !b.wonder {
        h = (t.red_fort_fort_hps + 100) * h / 100;
    }
    if b.ident == Ident::Market && m.nubians && t.nubian_hit_points != 0 {
        h = (t.nubian_hit_points + 100) * h / 100;
    }
    if !is_city(types, ty) {
        if active
            && city_level > 0
            && !is_fort(types, ty)
            && b.ident != Ident::Tower
            && b.ident != Ident::Lookout
        {
            h += t.senate_hp_bonus * (city_level - 1) * h / 100;
        }
    } else if active && city_level > 0 && has_temple && m.temple_level > 0 {
        let k = m.temple_level.clamp(1, 5) as usize;
        let mut bonus = t.temple_upgrade_hp[k - 1];
        if m.tikal {
            bonus = ((t.tikal_temple_hp + 100) * bonus + 99) / 100;
        }
        h = (bonus + 100) * h / 100;
    }
    h
}

/// The construction scaling of `update_hits`: a site's current full-health
/// figure, `construct_hits`. Linear from ~0 to full; a wonder from half.
pub const fn site_hits(full: i32, job_counter: i32, construct_time: i32, wonder: bool) -> i32 {
    if job_counter >= construct_time {
        return full;
    }
    let a = if (job_counter >> 5) < 1 {
        1
    } else {
        job_counter >> 5
    };
    let b = if (construct_time >> 5) < 1 {
        1
    } else {
        construct_time >> 5
    };
    if !wonder {
        let h = a * full / b;
        if h < 1 { 1 } else { h }
    } else {
        let grown = (full / 2) * a / b;
        (full + 1) / 2 + if grown < 1 { 1 } else { grown }
    }
}

/// `Object::take_damage` on a site: `whole × 50` off `job_counter` per hit.
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

/// The building-side attrition — `Wall::process`, every 16 frames phased by
/// `o`: eight hit points through `take_damage` with the attrition flag for a
/// started building on a tile an enemy owns; an unstarted one is simply
/// removed. `docs/CITIES.md` §9.5.
pub const ENEMY_TERRITORY_DAMAGE: i32 = 8;
pub const ENEMY_TERRITORY_PERIOD: i64 = 16;

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Tuning {
        Tuning::RON
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
        assert_eq!(flags::parse("2"), flags::DEEP_QUEUE);
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
