//! Costs: what a thing is worth, and what stops you paying for it.
//!
//! `docs/COSTS.md` is the specification. Three things decide a price — a base
//! written in the data and multiplied by ten, a **ramp** against how many of
//! the thing you already have, and a **redirect** that charges a different
//! resource entirely for anything the age has not made available. Then the
//! two questions a caller actually asks: can I pay, and is there room in the
//! population.
//!
//! Nothing here is fractional. Every step is an integer multiply followed by a
//! divide by a hundred or a shift by eight, in the original's order, so the
//! truncations land where the original's land.

use crate::core::tuning::Tuning;
use crate::economy::{Ledger, RESOURCES, Resource};

/// Which of the four cost factors a type is scaled by.
///
/// The original tells these apart by numeric identity — units are type ids
/// `0x32`–`0x19d`, buildings `0x19e`–`0x21e`, spells `0x275`–`0x2ab`, and
/// everything else is a tech. All four factors ship as ten, so the distinction
/// is invisible in a stock game and load-bearing in a modded one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Unit,
    Building,
    Spell,
    Tech,
}

impl Kind {
    pub const fn factor(self, t: &Tuning) -> i32 {
        match self {
            Kind::Unit => t.unit_cost_factor,
            Kind::Building => t.build_cost_factor,
            Kind::Spell => t.spell_cost_factor,
            Kind::Tech => t.tech_cost_factor,
        }
    }
}

/// The `PROGRESSION` column: two bits deciding what the ramp counts and what
/// shape the count takes.
///
/// Bit 0 counts the whole production group rather than the type; bit 1
/// replaces the count with its triangular number. The designers' comment
/// enumerates all four combinations and the shipped data uses three of them —
/// 232 unit records are progressive-by-group, 128 are linear-by-type, four
/// (the elephants) are progressive-by-type, and none are linear-by-group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progression(pub u8);

impl Progression {
    /// Linear, counted by unit type. Citizens, scholars, generals, supply.
    pub const BY_TYPE: Progression = Progression(0);
    /// Linear, counted by production group. Unused by the shipped data.
    pub const BY_GROUP: Progression = Progression(1);
    /// Triangular, counted by unit type. The elephants.
    pub const PROGRESSIVE_BY_TYPE: Progression = Progression(2);
    /// Triangular, counted by production group. Nearly every fighting unit.
    pub const PROGRESSIVE_BY_GROUP: Progression = Progression(3);

    pub const fn by_group(self) -> bool {
        self.0 & 1 != 0
    }

    pub const fn progressive(self) -> bool {
        self.0 & 2 != 0
    }
}

/// Which ceiling holds a type's ramp down.
///
/// The original picks between these in one nested test: scholars by name, then
/// citizens, merchants and anything with `unit_flags2 & 8` — the caravan and
/// merchant-fleet bit — as workers, and only then `obj_masks & 4`, the
/// designers' letter `C` for Civilian, separating the rest from the fighting
/// units. Taking it as a field is the same choice movement made for `speed`:
/// the mechanic is complete and the classification arrives with the layer that
/// produces it.
///
/// **All four are the unit arm's.** `TypeData::get_cost` has two ramps, not
/// one: the unit arm (`00665196`..`006656b8`, where `UNIT_COST_FACTOR` at
/// `+0x354` and all four `*_RAMP_MAX` at `+0x394`..`+0x3a0` are read) and the
/// building arm (`00665787`..`00665b5a`, `BUILD_COST_FACTOR` at `+0x358` and
/// `BUILD_SUPPORT_FACTOR` at `+0x37c`). The building arm loads no `RAMP_MAX`
/// at all — [`RampClass::Building`] is that absence, and it is what makes a
/// second Small City cost sixty rather than twenty-two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampClass {
    /// 2000%, and the only class with a second, convex term.
    Scholar,
    /// 500%. Citizens, merchants, caravans and merchant fleets — the last two
    /// by `unit_flags2 & 8`, which is tested before the civilian mask and
    /// therefore wins over it.
    Worker,
    /// 200%. Generals, spies and supply wagons — the civilian mask, reached
    /// only once the worker tests have all failed.
    OtherCivilian,
    /// 125%. Everything that fights.
    #[default]
    Military,
    /// **No ceiling.** Every building: the arm that prices them never reads a
    /// `RAMP_MAX`, so a building's support term is whatever the count makes it.
    Building,
}

impl RampClass {
    /// The ceiling as a percentage of the scaled base price; zero is "none",
    /// which [`cost_of`] reads as no clamp.
    pub const fn ceiling_percent(self, t: &Tuning) -> i32 {
        match self {
            RampClass::Scholar => t.unit_scholar_ramp_max,
            RampClass::Worker => t.unit_worker_ramp_max,
            RampClass::OtherCivilian => t.unit_other_civilian_ramp_max,
            RampClass::Military => t.unit_military_ramp_max,
            RampClass::Building => 0,
        }
    }
}

/// How many scholars one university holds, and the period of the scholar
/// ramp's second term.
pub const SCHOLARS_PER_UNIVERSITY: i32 = 7;

/// The price of one type, as the data writes it.
///
/// `base` is the `COST` column, unscaled — the file's own numbers, before the
/// factor. `support` is the `SUPPORT` column, and it is **two ordered slots**
/// rather than a six-slot array, which is not a detail: `ObjectType::load_support`
/// walks the slash-separated pairs in order, skips any whose amount is zero,
/// stops after two, and leaves the rest at "no resource". A field naming the
/// same resource twice therefore ramps twice — the militia line's
/// `2f/2f support` is four food per militia and no metal at all, which reads
/// as a typo for `2f/2m` and behaves as written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Price {
    pub kind: Kind,
    /// `COST`, before the factor.
    pub base: [i32; RESOURCES],
    /// `SUPPORT`: the ramp, in whole resources, in the two slots the original
    /// keeps it in.
    pub support: [Option<(Resource, i32)>; 2],
    pub progression: Progression,
    pub class: RampClass,
    /// `POP`, the population this type occupies. Ships as 1 for most units, 2
    /// for the larger warships, 0 for transports and meta-units.
    pub pop: i32,
}

impl Price {
    /// A price with nothing in it.
    pub fn free() -> Price {
        Price::default()
    }

    /// Sets one resource's base cost, in the file's units.
    pub fn with_base(mut self, r: Resource, n: i32) -> Price {
        self.base[r.index()] = n;
        self
    }

    /// Fills the next free `SUPPORT` slot, the way `ObjectType::load_support`
    /// fills it: an amount of zero takes no slot, and there are only two.
    pub fn with_support(mut self, r: Resource, n: i32) -> Price {
        if n != 0
            && let Some(slot) = self.support.iter_mut().find(|s| s.is_none())
        {
            *slot = Some((r, n));
        }
        self
    }
}

/// What `LeaderData::get_support_count` resolves to: how many of a thing the
/// player is considered to have, by type and by production group.
///
/// Both are `built + queued` in the original, and for nukes and missiles the
/// count also includes the ones already spent — a used nuke goes on making the
/// next one more expensive forever.
///
/// Three predicates belong to whatever fills this in, because they are about
/// unit identity rather than arithmetic: the group count is only taken when the
/// type's `PROGRESSION` has bit 0 set **and** its `attack` is non-zero; a
/// citizen's count adds the militia, minutemen and partisans made from citizens
/// and subtracts the ones made from scholars; and a scholar's count adds those
/// back. `docs/COSTS.md` §The ramp states them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub of_type: i32,
    pub of_group: i32,
}

impl Counts {
    /// The count this progression actually reads.
    pub const fn for_progression(self, p: Progression) -> i32 {
        if p.by_group() {
            self.of_group
        } else {
            self.of_type
        }
    }
}

/// Everything outside the data that moves a price.
///
/// The original applies about forty discounts, and with three exceptions every
/// one is `cost * (100 - X) / 100`. They are folded into two numbers here
/// because the shape carries no information and the position does: most of the
/// national, wonder and government tail lands **before** the ramp and the rest
/// lands **after** it, so the two cannot be one number without changing what
/// the ceiling is measured against.
///
/// The fold is itself a simplification, and a stated one: the original
/// truncates after every single percentage, so a fold of two of them can differ
/// by a unit from the original's chain. It stands until the nation, wonder and
/// government layers exist to apply their own, one at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// `get_cost`'s **Salt arm** (`TypeData::get_cost@00664090:208`–`213`,
    /// `× 9 / 10` on the scaled base, ahead of every other
    /// term and of the research fork): a unit whose trainer (`+0x40`) is
    /// the Barracks, by equality, of a leader holding Salt (`+0x6da4 &
    /// 0x80`, or `rare_conquest`'s at `+0x6dcc`) costs nine tenths. The ten
    /// percent is written in the executable; `SALT_BARRACKS_COST`'s 15% is
    /// never read here (item 3318, `docs/AI.md` §264).
    pub salt: bool,
    /// The **American aircraft arm** (`get_cost:268`–`282`): up to three
    /// `AMERICANS_AIRCRAFT` percentages — the Airbase trainer, then
    /// `is(0x15f, 0)`, then `is(0x134, 0)` — each its own truncating step
    /// after the German arm and before the British AA's, so a Fighter
    /// Bomber trained at the Airbase takes it twice (item 3755,
    /// `docs/AI.md` §372.1). Zero is the identity.
    pub american_aircraft: [i32; 3],
    /// The British Anti-Aircraft Gun percentage (`get_cost:283`–`289`),
    /// its own truncating step after the American aircraft arm and before
    /// the Nubian merchants' and the Russian spy's (item 3678, `docs/AI.md`
    /// §347).
    pub british_aa: i32,
    /// The Russians' Spy percentage (`get_cost:299`), its own
    /// `(100 - x) * cost / 100` before [`Modifiers::discount`] and the
    /// Spy/General arm that follows it in the original's tail.
    pub russian_spy: i32,
    /// The Romans' Legion percentage (`get_cost:306`–`311`), its own
    /// truncating step right after the Russian spy's.
    pub roman_legion: i32,
    /// The Citizen arms (`get_cost`, `TypeIndex` `0x32`/`0x33`): Wool's
    /// `WOOL_CITIZENS_COST` then a Turkish leader's `TURK_CITIZEN`, each its
    /// own truncating step after the Spy/General arm ([`Modifiers::discount`])
    /// and the stable rares, before the Statue of Liberty.
    pub citizens: [i32; 2],
    /// `get_cost`'s **domain arm** (`UnitTypeData +0x218`, the pre-ramp
    /// tail after the Citizen arms and before Despotism, on the train and
    /// research arms alike): a sea unit takes Angkor Wat's `ANGKOR_SHIPS`;
    /// an air unit takes Aluminium's `ALUMINUM_AIR_COST`, then the Space
    /// Program's `SPACE_AIR_COST` — each its own `(100 − x) × cost / 100`,
    /// in that order. Zero is the identity (item 3514, `docs/AI.md` §301).
    pub domain: [i32; 2],
    /// Percentage off the scaled base, before the ramp.
    pub discount: i32,
    /// Horses', Rubber's and then Sulphur's (a Siege Factory unit's) percentage off a Stable or Auto Plant
    /// unit, each applied as its own `(100 − x) × cost / 100` right after
    /// [`Modifiers::discount`], so each truncates where the original's does
    /// (`get_cost`'s pre-ramp tail, before the research arm and the ramp;
    /// `docs/AI.md` §62). Zero is the identity.
    pub stable_rares: [i32; 3],
    /// `get_cost`'s **Despotism arm** on a Barracks unit
    /// (`TypeData::get_cost@00664090:382`–`404`, the shared pre-ramp tail,
    /// so the train and research arms alike): a unit whose trainer is the
    /// Barracks (`+0x40 == 0x1ab`, by equality) takes the highest held
    /// tier's `DESPOTISM_MILITARY_CHEAPER{,2,3}` — `has_preq(DESPOTISM_3)`,
    /// then `_2`, then `_1` — one `(100 − x) × cost / 100` after the Citizen
    /// arms and before the Statue of Liberty. Zero is the identity (item
    /// 3242, `docs/AI.md` §255).
    pub despotism: i32,
    /// The **Statue of Liberty**'s percentage (`get_cost`'s pre-ramp tail,
    /// listing `00664da7`..`00664e1e`): with `has_wonder(0x219)`, a missile
    /// type (`UnitTypeData::is_missile`, vslot `+0x114`) takes
    /// `LIBERTY_MISSILES` and any other type of the Bomber's line
    /// (`is(0x130, 0)`) `LIBERTY_BOMBERS`, one `(100 − x) × cost / 100`
    /// after Despotism and before Uranium. Zero is the identity
    /// (`docs/AI.md` §204).
    pub liberty: i32,
    /// `URANIUM_NUKE_COST`'s percentage off a type of the Nuclear Missile's
    /// line when the player holds Uranium, one more `(100 − x) × cost / 100`
    /// in the same pre-ramp tail (`get_cost:416`–`420`, after Despotism and
    /// the Statue of Liberty, before the research arm and the ramp;
    /// `docs/AI.md` §167). Zero is the identity.
    pub uranium: i32,
    /// Percentage off the whole price, after the ramp. `MILITARY_UNIT_DISCOUNT`,
    /// Monarchy on stable units, Socialism on siege, air and dock units, Salmon
    /// on ships, then Sugar, Coal, Gold, Iron and Gypsum by resource, and — in
    /// the original, after the captured-building doubling rather than before it
    /// — the Supercollider surcharge and the Indian elephant discount.
    pub late_discount: i32,
    /// `get_cost`'s **Monarchy arm** (`TypeData::get_cost@00664090:657`–`665`,
    /// the train arm only): `MONARCHY_UNIT_PROD` off a unit whose trainer is
    /// the Stable (`+0x40 == 0x1ac`, by equality) when either `MONARCHY_1` or
    /// `MONARCHY_2` is held — `_PROD2` is never read here. Its own
    /// `(100 − x) × cost / 100`, after [`Modifiers::late_discount`]'s
    /// `MILITARY_UNIT_DISCOUNT` (item 2331, `docs/GOLDEN.md` §84). Zero is
    /// the identity.
    pub monarchy: i32,
    /// `get_cost`'s **Socialism arm** (`TypeData::get_cost@00664090:666`–
    /// `672`, the train arm only, the line after Monarchy's):
    /// `SOCIALISM_UNIT_BONUS` off a unit whose trainer is the Siege Factory,
    /// the Airbase or the Dock (`+0x40` `0x1ae`, `0x1bf`, `0x1b0`, each by
    /// equality) when `has_preq(SOCIALISM_1)` holds. Its own
    /// `(100 − x) × cost / 100`, after [`Modifiers::monarchy`] (item 2357,
    /// `docs/GOLDEN.md` §85). Zero is the identity.
    pub socialism: i32,
    /// `get_cost`'s **Salmon arm** (`TypeData::get_cost@00664090:673`–`677`,
    /// the train arm only, the line after Socialism's): `SALMON_NAVAL_COST`
    /// off a sea unit (`+0x218 == 1`) of a player holding Salmon. Its own
    /// `(100 − x) × cost / 100`, after [`Modifiers::socialism`] (item 2389,
    /// `docs/GOLDEN.md` §86). Zero is the identity.
    pub salmon: i32,
    /// `get_cost`'s **library-line tail** (`00666b7d`): up to three
    /// percentages off a library epoch's price, each its own
    /// `(100 − x) × cost / 100` in order, after
    /// [`Modifiers::late_discount`] — the line's rare, its government or
    /// nation, and a second nation (`docs/AI.md` §99.8). Zero is the
    /// identity.
    pub line_discounts: [i32; 3],
    /// `SILVER_AGE_COST` when the price is an age's and the player holds
    /// Silver, else zero: `get_cost`'s `00666b13`..`00666b51`, its own
    /// `(100 − x) × cost / 100` after [`Modifiers::late_discount`] and
    /// before [`Modifiers::line_discounts`] — which an age never takes, so
    /// the two never meet.
    pub silver: i32,
    /// How many Science levels the player is **ahead of the technology's
    /// own** — the only term of `LeaderData::calc_science_discount@006da630`,
    /// and signed: a player whose Science line is behind the tech's age pays a
    /// *surcharge* out of the same expression. Zero for anything that is not a
    /// technology, because the original guards the whole function on
    /// `is_tech_type`, and zero is the identity here.
    ///
    /// It is a level count rather than a percentage because the original
    /// multiplies by `TECH_SCIENCE_DISCOUNT` itself, and it is not folded into
    /// [`Modifiers::discount`] because the shape is a subtraction rather than a
    /// scale — see [`science_discount`].
    pub science_ahead: i32,
    /// Non-library technology discount, after Science and before the
    /// British taxation term. Its own truncation (`docs/AI.md` §109).
    pub democracy: i32,
    /// `get_cost`'s **industry arm** (`TypeData::get_cost@00664090:1270`–
    /// `1276`): `HANGING_GARDENS_INDUSTRY_DISCOUNT` off a food, timber or
    /// metal gathering-line technology (`0x261..=0x266`, `0x26c..=0x26e`)
    /// for a leader holding the Hanging Gardens, every resource. Its own
    /// `(100 − x) × cost / 100`, after [`Modifiers::democracy`] and before
    /// [`Modifiers::late_discount`] (item 3332, `docs/AI.md` §269). Zero is
    /// the identity.
    pub industry: i32,
    /// `get_cost`'s **German industry term** (`:1277`–`1281`), on the same
    /// industry predicate as [`Modifiers::industry`] and right after it:
    /// `GERMAN_INDUSTRY_DISCOUNT` off for a leader with
    /// `has_tribe_bonus(0xc)`, its own `(100 − x) × cost / 100` (item 3378,
    /// `docs/AI.md` §289). Zero is the identity.
    pub german_industry: i32,
    /// `get_cost`'s **Greek research arm** (`TypeData::get_cost@00664090`,
    /// `LAB_006667c4`, `:1250`–`1255`): an age or a library epoch priced
    /// for a leader with `has_tribe_bonus(5)` takes `GREEK_RESEARCH_COST`
    /// off **every good but knowledge** (`local_c != 3`), its own
    /// `(100 − x) × cost / 100` after the lobby's `tech_cost` term and
    /// ahead of Silver and the library-line tail. Zero is the identity
    /// (item 3326, `docs/AI.md` §266).
    pub greek_research: i32,
    /// `get_cost`'s **rares' tail** (`00666e46`..`00666f7f`), after the
    /// library-line tail and before the captured-building doubling: the
    /// percentage off this resource of the rare that names it — Coal on
    /// timber, Sugar on food, Gold on wealth, Iron on metal — indexed by
    /// resource, then Gypsum's on every resource, each its own
    /// `(100 − x) × cost / 100` (item 3343; [`crate::economy::SUGAR`]).
    /// Zero is the identity.
    pub rare_goods: [i32; RESOURCES],
    /// See [`Modifiers::rare_goods`]: Gypsum, every resource.
    pub gypsum: i32,
    /// Maize halves the ramp term — `MAIZE_RAMPING_BONUS`.
    pub maize: bool,
    /// Producing at a captured, unassimilated building doubles the price.
    pub unassimilated: bool,
    /// `get_cost`'s **Indian elephant arm** (`:1556`–`1576`), the listing's
    /// last step before the redirect: percent off a unit of the elephant
    /// line for an Indian leader, after the captured-building doubling
    /// (`INDIANS_ELEPHANT_COST`; `Sim::indian_elephant_discount`).
    pub indian_elephant: i32,
    /// `get_cost`'s **wonder arm** (`00665848`..`0066594c`): the count is
    /// [`wonder_count`] rather than the type's own, and the building arm's
    /// term is **halved** after the factor and the count (`00665af9`,
    /// `cltd; sub; sar`), so it truncates where a count of half cannot
    /// (`docs/COSTS.md`, "A wonder is ramped by every wonder").
    pub wonder: bool,
    /// `get_cost`'s **city tail** (`00665c46`..`00665c9b`): on a type whose
    /// `is_city` holds, `BANTU_CITY_COST` for a Bantu leader and then
    /// `PYRAMIDS_CITY_DISCOUNT` for one holding the Pyramids, each its own
    /// `(100 − x) × cost / 100` right after the building ramp
    /// (`docs/COSTS.md`, "The Pyramids take a third off a city"). Zero is
    /// the identity.
    pub city: [i32; 2],
    /// `get_cost`'s wonder tail (`00665ced`..): `EGYPTIAN_WONDERS` off a wonder
    /// (`0x20d < type < 0x21f`) for a leader with the Egyptian bonus
    /// (`has_tribe_bonus(7)`), after the city tail and the (uncarried) marble
    /// term. Zero is the identity.
    pub egyptian_wonders: i32,
    /// `get_cost`'s **Korean tower step** (`:916`–`922`, after the wonder
    /// tail and before the fort arm): a building of the Tower's line
    /// (`is(0x1b7, 0)`) priced for a leader with `has_tribe_bonus(0x10)`
    /// takes `KOREAN_TOWER_COST` off, its own `(100 − x) × cost / 100`. Zero
    /// is the identity (item 3365, `docs/AI.md` §283).
    pub korean_tower: i32,
    /// `get_cost`'s **fort arm** (`00665e0f`..`00665e8a`): on a type that
    /// `is_fort` (vslot `+0xfc`) or is of the Tower's line (`is(0x1b7, 0)`)
    /// and is not a wonder, `COLOSSEUM_FORT_COST` with the Colosseum held
    /// (`has_wonder(0x212)`) and then `ROMAN_FORT_COST` for a Roman leader
    /// (`has_tribe_bonus(6)`), each its own `(100 − x) × cost / 100` after
    /// the wonder tail and the Korean tower step. Zero is the
    /// identity (item 3314, `docs/AI.md` §263).
    pub fort: [i32; 2],
    /// `get_cost`'s **Greek Library arm** (`TypeData::get_cost@00664090:966`–`972`,
    /// through `LAB_00665f31`): a Library (`0x1b3`) or a University (`0x1a4`),
    /// by equality, priced for a leader with `has_tribe_bonus(5)` takes
    /// `GREEK_LIBRARY_COST` off, its own `(100 − x) × cost / 100` after the
    /// fort arm and Tikal's temple term. Zero is the identity (item 3326,
    /// `docs/AI.md` §266).
    pub greek_library: i32,
    /// `get_cost`'s **Maya arm** (`TypeData::get_cost@00664090:973`–`987`,
    /// `00665f3e`..): for a leader with `has_tribe_bonus(1)`, on **timber**
    /// alone (`local_c == 1`), a building that is no wonder (vslot `+0x1c`)
    /// and none of the military trainers, the docks, the towers, the forts
    /// or the lookout line by type equality takes `MAYA_BUILDING_TIMBER`
    /// off, `(100 − x) × cost / 100`, after the Greek Library arm and ahead
    /// of the (uncarried) British anti-air arm and the Japanese Farm. Zero is
    /// the identity (item 3338, `docs/AI.md` §273).
    pub maya_timber: i32,
    /// `get_cost`'s **Japanese Farm arm** (`:1003`–`1008`): a Farm (`0x1a1`)
    /// for a leader with tribe bonus `0xf` takes `JAPANESE_FARMS_CHEAPER`
    /// off, `(100 − x) × cost / 100`, after the Greek Library arm in the
    /// same chain. Zero is the identity (item 3331, `docs/AI.md` §268).
    pub japanese_farms: i32,
    /// `get_cost`'s **Indian ramp arm** (`:843`–`876`, `00665b0a`..`00665c1f`),
    /// inside the building arm's support loop, after the wonder's halving:
    /// for a leader with `has_tribe_bonus(0x15)` and a type that is not a
    /// city (vslot `+0x64`), each support term is zeroed — or, for a wonder
    /// (`0x20e ≤ type < 0x21f`) and the towers, forts, lookouts and air
    /// defences (`0x1b7`..`0x1be`, `0x209`..`0x20d`), halved rounding up,
    /// `(term + 1) / 2` (item 3335, `docs/AI.md` §271).
    pub indian_ramp: IndianRamp,
    /// `get_cost`'s **research** arm: the type is not yet available to the
    /// player (the `leader + 0x6c18` bit is clear), so what is priced is its
    /// research, and it takes the place of the ramp. `None` is the train arm.
    pub research: Option<Research>,
}

/// [`Modifiers::indian_ramp`]: what the Indian arm does to a building's
/// support term.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndianRamp {
    /// Not an Indian leader, or a city: the term stands.
    #[default]
    Off,
    /// The term is zeroed.
    Zero,
    /// The term is halved, rounding up.
    Half,
}

/// What `TypeData::get_cost@00664090` charges to **research** a unit type
/// rather than train one (`docs/COSTS.md`, "Researching an upgrade is not
/// building a unit"; `docs/AI.md` §56). In the original's order, after the
/// scaled base and its pre-ramp tail and in place of the ramp:
///
/// 1. `WINE_UNIT_UPGRADES` off when the player holds Wine (`get_cost:427`–
///    `428`, `00664edd`..`00664f11`: `(100 − wine) × cost / 100`, toward
///    zero; `docs/AI.md` §94);
/// 2. `× RESEARCH_PREMIUM >> 8`, then `× RESEARCH_PREMIUM_COST >> 8`, each
///    truncating toward zero (`get_cost:430`–`432`);
/// 3. the refit surcharge, already multiplied out per resource;
/// 4. `MILITARY_UPGRADE_DISCOUNT`, clamped at zero (`get_cost:507`–`525`).
///
/// Not carried, and each is a seam: `SPECIAL_UPGRADE` (every one of the shipped 364 records has an
/// empty `<UPGRADE/>`), and the American and Dutch nation discounts after
/// the military one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Research {
    /// `WINE_UNIT_UPGRADES` when the player holds Wine, else zero — the
    /// percentage the arm takes off before the premium.
    pub wine: i32,
    /// `RESEARCH_PREMIUM_COST`, 8.8 (`UnitTypeData +0x2e0`).
    pub premium_cost: i32,
    /// The refit surcharge by resource, `UNIT_COST_FACTOR × n × d` summed
    /// over the army it upgrades — see `Sim::research_modifiers`.
    pub refit: [i32; RESOURCES],
    /// `MILITARY_UPGRADE_DISCOUNT`'s percentage, already scaled for a short
    /// scenario.
    pub discount: i32,
    /// `DUTCH_SHIP_DISCOUNT` for a Dutch leader's sea unit of a military
    /// role, whether or not the Military discount applied (`get_cost:532`–
    /// `535`, after that block's close; item 3329): its own `(100 − x) ×
    /// cost / 100` after it.
    pub dutch_ship: i32,
    /// `AMERICANS_UPGRADE_DISCOUNT` for an American leader's land unit of a
    /// military role (`get_cost:528`–`531`, the step before the Dutch
    /// arm's; item 3374): its own `(100 − x) × cost / 100`.
    pub american_land: i32,
}

/// The count after the progression has shaped it.
///
/// The triangular number is applied before anything else reads the count,
/// which matters for the scholar term below: it sees the shaped count, not the
/// raw one. Scholars are linear, so in the shipped data the two are the same.
pub const fn ramp_steps(count: i32, p: Progression) -> i32 {
    if p.progressive() {
        (count + 1) * count / 2
    } else {
        count
    }
}

/// `get_cost`'s building branch's **per-family escalation** of the count a
/// building's support term is multiplied by (`00665951`..`00665aa5`, item
/// 1870), keyed on the type's `TypeIndex` and applied to `num_buildings +
/// num_queued` (`n`).
///
/// - **Production buildings** (`0x1ab..=0x1af`, `0x1bf`, `0x208`: Barracks,
///   Stable, Autoplant, Siege Factory, Factory, Air Base, Missile Silo):
///   `n ≤ 2` is itself, past two `2n − 2`, then `+ 2n − 10` past five,
///   `+ 3n − 24` past eight and `+ 4n − 48` past twelve.
/// - **The Fort line** (`0x1bb..=0x1be`: Fort, Castle, Fortress, Redoubt):
///   past one `2n − 1`, then `+ n − 3` past three, `+ n − 5` past five and
///   `+ n − 7` past seven. Two Redoubts count three, which is what the
///   original prices the third at: 175 wealth and 525 metal on a 100/300
///   base with a 25/75 support.
///
/// Every comparison reads the original `n`, never the running value.
pub const fn building_family_count(tree: usize, n: i32) -> i32 {
    let mut v = n;
    match tree {
        0x1ab..=0x1af | 0x1bf | 0x208 => {
            if n > 2 {
                v = 2 * n - 2;
            }
            if n > 5 {
                v += 2 * n - 10;
            }
            if n > 8 {
                v += 3 * n - 24;
            }
            if n > 12 {
                v += 4 * n - 48;
            }
        }
        0x1bb..=0x1be => {
            if n > 1 {
                v = 2 * n - 1;
            }
            if n > 3 {
                v += n - 3;
            }
            if n > 5 {
                v += n - 5;
            }
            if n > 7 {
                v += n - 7;
            }
        }
        _ => {}
    }
    v
}

/// Whether a Supercollider or Space Program is already a site of mine or of
/// a teammate's — `get_cost`'s last wonder test, asked only when the type
/// priced is one of the two (`0x21d`, `0x21e`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpaceRace {
    /// Neither, or the type priced is some other wonder.
    #[default]
    None,
    /// `has_unbuilt_wonder(0x21d || 0x21e)`: my own site counts once more.
    Own,
    /// `team_has_unbuilt_wonder(…)` and not my own: one step more.
    Team,
}

/// **`get_cost`'s wonder count** (`TypeData::get_cost@00664090`, listing
/// `00665848`..`0066594c`): `held` is `get_wonders + get_unbuilt_wonders`,
/// the leader's standing wonders and its unbuilt sites. The team term is
/// zero with teams unlocked (`xor ecx, ecx` at `00665882`); past three
/// held it adds `held − 3`, past six `held − 6`; then twice `held`, with
/// one more held for my own space-race site or one more step for a
/// teammate's. The building arm then halves `amount × factor × count`
/// ([`Modifiers::wonder`]).
pub const fn wonder_count(held: i32, space: SpaceRace) -> i32 {
    let mut c = 0;
    if held > 3 {
        c += held - 3;
    }
    if held > 6 {
        c += held - 6;
    }
    match space {
        SpaceRace::None => c + held * 2,
        SpaceRace::Own => c + (held + 1) * 2,
        SpaceRace::Team => c + 1 + held * 2,
    }
}

/// The scholar ramp's second term.
///
/// Seven is a university's capacity, so the first university's worth of
/// scholars ramps linearly and every university after that adds a convex term
/// on top. Written the original's way, including its final comparison, which
/// stops one step earlier than a symmetric sum would.
pub fn scholar_surcharge(count: i32) -> i32 {
    let mut extra = 0;
    let mut k = SCHOLARS_PER_UNIVERSITY;
    if k < count {
        loop {
            extra += count - k;
            k += SCHOLARS_PER_UNIVERSITY;
            if k >= count {
                break;
            }
        }
    }
    extra
}

/// One resource's price, before the redirect.
///
/// This is `TypeData::get_cost` with its discount tail collapsed into
/// [`Modifiers`]: the base times the factor, the discounts, the capped ramp,
/// the late discounts, and the captured-building doubling — in that order,
/// because the ceiling is a percentage of the *undiscounted* scaled base and
/// swapping any two of these changes the answer.
pub fn cost_of(t: &Tuning, price: &Price, r: Resource, counts: Counts, m: &Modifiers) -> i32 {
    let scaled = price.base[r.index()] * price.kind.factor(t);
    let mut cost = scaled;
    if m.salt {
        cost = cost * 9 / 10;
    }
    for pct in m.american_aircraft {
        cost = (100 - pct) * cost / 100;
    }
    if m.british_aa != 0 {
        cost = (100 - m.british_aa) * cost / 100;
    }
    if m.russian_spy != 0 {
        cost = (100 - m.russian_spy) * cost / 100;
    }
    if m.roman_legion != 0 {
        cost = (100 - m.roman_legion) * cost / 100;
    }
    cost = cost * (100 - m.discount) / 100;
    for pct in m.stable_rares {
        cost = (100 - pct) * cost / 100;
    }
    for pct in m.citizens {
        cost = (100 - pct) * cost / 100;
    }
    for pct in m.domain {
        cost = (100 - pct) * cost / 100;
    }
    if m.despotism != 0 {
        cost = (100 - m.despotism) * cost / 100;
    }
    if m.liberty != 0 {
        cost = (100 - m.liberty) * cost / 100;
    }
    if m.uranium != 0 {
        cost = (100 - m.uranium) * cost / 100;
    }
    // `LeaderData::calc_science_discount`, where the original calls it: after
    // the factor and the nation tail, before the age-behind discount and the
    // final-tech ramp, and **inside** the per-resource computation, so the
    // redirect at the end of `get_cost` carries the discounted number rather
    // than discounting a redirected one.
    cost = science_discount(t, m.science_ahead, cost);

    if let Some(rs) = m.research {
        // The research arm, in place of the ramp and the military discount.
        if rs.wine != 0 {
            cost = (100 - rs.wine) * cost / 100;
        }
        cost = cost * t.research_premium / 256;
        cost = cost * rs.premium_cost / 256;
        cost += rs.refit[r.index()];
        if rs.discount != 0 {
            cost = ((100 - rs.discount) * cost / 100).max(0);
        }
        if rs.american_land != 0 {
            cost = (100 - rs.american_land) * cost / 100;
        }
        if rs.dutch_ship != 0 {
            cost = (100 - rs.dutch_ship) * cost / 100;
        }
    } else {
        let raw = counts.for_progression(price.progression);
        if raw > 0 {
            let steps = ramp_steps(raw, price.progression);
            let ceiling = price.class.ceiling_percent(t) * scaled / 100;
            let extra = match price.class {
                RampClass::Scholar => scholar_surcharge(steps),
                _ => 0,
            };
            // The building arm multiplies the written amount by
            // `BUILD_SUPPORT_FACTOR` (`imull 0x37c(%eax), %esi` at `00665ad1`)
            // before the count; the unit arm has no such factor. It ships as one.
            let support_factor = match price.kind {
                Kind::Building => t.build_support_factor,
                _ => 1,
            };
            for slot in price.support.iter().flatten() {
                let (good, amount) = *slot;
                if good != r {
                    continue;
                }
                let mut term = amount * support_factor * steps + extra;
                if m.wonder {
                    term /= 2;
                }
                match m.indian_ramp {
                    IndianRamp::Off => {}
                    IndianRamp::Zero => term = 0,
                    IndianRamp::Half => term = (term + 1) / 2,
                }
                if ceiling != 0 && term > ceiling {
                    term = ceiling;
                }
                if m.maize {
                    term = term * (100 - t.maize_ramping_bonus) / 100;
                }
                cost += term;
            }
        }
    }

    for pct in m.city {
        cost = (100 - pct) * cost / 100;
    }
    if m.egyptian_wonders != 0 {
        cost = (100 - m.egyptian_wonders) * cost / 100;
    }
    if m.korean_tower != 0 {
        cost = (100 - m.korean_tower) * cost / 100;
    }
    for pct in m.fort {
        cost = (100 - pct) * cost / 100;
    }
    if m.greek_library != 0 {
        cost = (100 - m.greek_library) * cost / 100;
    }
    if m.maya_timber != 0 && r == Resource::Timber {
        cost = (100 - m.maya_timber) * cost / 100;
    }
    if m.japanese_farms != 0 {
        cost = (100 - m.japanese_farms) * cost / 100;
    }
    if m.greek_research != 0 && r != Resource::Knowledge {
        cost = (100 - m.greek_research) * cost / 100;
    }
    if m.democracy != 0 {
        cost = cost.wrapping_mul(100 - m.democracy) / 100;
    }
    if m.industry != 0 {
        cost = (100 - m.industry) * cost / 100;
    }
    if m.german_industry != 0 {
        cost = (100 - m.german_industry) * cost / 100;
    }
    cost = cost * (100 - m.late_discount) / 100;
    if m.monarchy != 0 {
        cost = (100 - m.monarchy) * cost / 100;
    }
    if m.socialism != 0 {
        cost = (100 - m.socialism) * cost / 100;
    }
    if m.salmon != 0 {
        cost = (100 - m.salmon) * cost / 100;
    }
    if m.silver != 0 {
        cost = (100 - m.silver) * cost / 100;
    }
    for d in m.line_discounts {
        cost = (100 - d) * cost / 100;
    }
    if m.rare_goods[r.index()] != 0 {
        cost = (100 - m.rare_goods[r.index()]) * cost / 100;
    }
    if m.gypsum != 0 {
        cost = (100 - m.gypsum) * cost / 100;
    }
    if m.unassimilated {
        cost *= 2;
    }
    if m.indian_elephant != 0 {
        cost = (100 - m.indian_elephant) * cost / 100;
    }
    cost
}

/// Where a cost written in an unavailable resource is charged instead.
///
/// The rate is 8.8 fixed point, because `GoodType::init` loads it through
/// `String::fraction(text, 0x100)` and `TypeData::get_cost` multiplies and
/// shifts right by eight. `1/1` is 256, `3/2` is 384, `5/4` is 320, `1/2` is
/// 128, and the shipped tables use all four.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redirect {
    pub good: Resource,
    pub rate: i32,
}

/// The two redirect tables from `resourcerules.xml`, one for a resource whose
/// prerequisite the player does not hold and one for a resource that has gone
/// obsolete.
///
/// They are the `UNDISC_COST_GOOD`/`UNDISC_COST_RATE` and
/// `OBS_COST_GOOD`/`OBS_COST_RATE` columns — `GoodTypeData +0x2b4`/`+0x2c8` and
/// `+0x2b8`/`+0x2cc`. The four `*_SUPPORT_GOOD`/`*_SUPPORT_RATE` columns that
/// follow them in the file are a different pair of tables and no reader of them
/// has been found.
///
/// The undiscovered table is the interesting one, because
/// `resourcerules.xml` gives Knowledge and Metal the Classical Age as their
/// prerequisite and Oil the Industrial Age. So in the Ancient Age a tech
/// priced in knowledge is charged in **food at three halves** — Mathematics is
/// `8k/12g` and a player who researches it early pays a hundred and twenty food
/// — and anything priced in metal is charged in **timber at five quarters**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redirects {
    pub undiscovered: [Redirect; RESOURCES],
    pub obsolete: [Redirect; RESOURCES],
}

impl Redirects {
    /// What Rise of Nations ships, read from `resourcerules.xml` and re-read
    /// from the user's own copy by `cargo run -p rondata -- <install>`.
    ///
    /// Wealth's undiscovered entry names Wealth, which would recur forever. It
    /// cannot fire: wealth is never unavailable. Oil's names Metal, which
    /// *can* be unavailable, and that chain is why [`charges`] recurses.
    pub const RON: Redirects = {
        const fn to(good: Resource, rate: i32) -> Redirect {
            Redirect { good, rate }
        }
        // The four rates the tables use, in 8.8.
        const THREE_HALVES: i32 = 384;
        const FIVE_QUARTERS: i32 = 320;
        const ONE: i32 = crate::ids::scale::ONE_8_8;
        const HALF: i32 = 128;
        Redirects {
            undiscovered: [
                to(Resource::Wealth, THREE_HALVES),
                to(Resource::Wealth, THREE_HALVES),
                to(Resource::Wealth, THREE_HALVES),
                to(Resource::Food, THREE_HALVES),
                to(Resource::Timber, FIVE_QUARTERS),
                to(Resource::Metal, THREE_HALVES),
            ],
            obsolete: [
                to(Resource::Wealth, ONE),
                to(Resource::Oil, HALF),
                to(Resource::Wealth, ONE),
                to(Resource::Wealth, ONE),
                to(Resource::Wealth, ONE),
                to(Resource::Wealth, ONE),
            ],
        }
    };

    /// The redirect that applies to a resource, chosen the way the original
    /// chooses: a resource the player has the prerequisite for but cannot use
    /// is obsolete, and one they do not is undiscovered.
    pub const fn of(&self, r: Resource, discovered: bool) -> Redirect {
        if discovered {
            self.obsolete[r.index()]
        } else {
            self.undiscovered[r.index()]
        }
    }
}

/// One resource's price with the redirect loop folded in — `TypeData::get_cost`
/// with its `include_redirect` argument set, which is how every caller that
/// matters calls it.
///
/// The redirect is the last thing the original does, and it does it by calling
/// itself: for every good the player cannot use whose redirect names `r`, it
/// adds `get_cost(that good) * rate >> 8`, and that inner call runs the loop
/// again. So the chain composes. In the Ancient age an oil price is charged in
/// metal at three halves, and metal is not available either, so the whole thing
/// arrives in timber at a further five quarters.
///
/// `seen` is the one place this departs from the original. The original has no
/// cycle guard and a table pointing a good at itself would recurse until the
/// stack ran out; the shipped table has no cycle, so refusing to re-enter a
/// good already on the stack is exact for it and terminates for anything else.
fn redirected_cost(
    t: &Tuning,
    price: &Price,
    r: Resource,
    counts: Counts,
    m: &Modifiers,
    goods: Goods<'_>,
    seen: u8,
) -> i32 {
    let mut cost = cost_of(t, price, r, counts, m);
    for g in Resource::ALL {
        let i = g.index();
        if goods.available[i] || seen & (1 << i) != 0 {
            continue;
        }
        let to = goods.redirects.of(g, goods.discovered[i]);
        if to.good != r {
            continue;
        }
        let inner = redirected_cost(t, price, g, counts, m, goods, seen | (1 << i));
        cost += apply_rate(inner, to.rate);
    }
    cost
}

/// What the redirect loop needs to know about the player: which goods they can
/// spend, which they hold the prerequisite for, and the two tables that say
/// where the rest is charged instead.
#[derive(Clone, Copy)]
struct Goods<'a> {
    available: &'a [bool; RESOURCES],
    discovered: &'a [bool; RESOURCES],
    redirects: &'a Redirects,
}

/// An 8.8 rate applied the original's way: multiply, then shift right by eight
/// with the sign fix that makes the truncation go toward zero rather than down.
/// It only matters if a price ever goes negative, which a discount tail summing
/// past a hundred percent can do.
const fn apply_rate(cost: i32, rate: i32) -> i32 {
    let scaled = cost * rate;
    (scaled + ((scaled >> 31) & 0xff)) >> 8
}

/// What a player is actually charged, per resource.
///
/// Unavailable resources come back as zero — `Type::pay_cost` charges only the
/// available ones — and their prices are redirected into whatever the tables
/// name, through [`redirected_cost`]. The redirected amount is the source
/// resource's own full price, added after the target's own discounts, which is
/// where the original adds it.
pub fn charges(
    t: &Tuning,
    price: &Price,
    counts: Counts,
    m: &Modifiers,
    available: &[bool; RESOURCES],
    discovered: &[bool; RESOURCES],
    redirects: &Redirects,
) -> [i32; RESOURCES] {
    let goods = Goods {
        available,
        discovered,
        redirects,
    };
    let mut out = [0; RESOURCES];
    for r in Resource::ALL {
        if !available[r.index()] {
            continue;
        }
        out[r.index()] = redirected_cost(t, price, r, counts, m, goods, 0);
    }
    out
}

/// The sentinel `TypeData::can_pay_cost` returns when nothing constrains the
/// answer — a free game, a scenario with costs switched off, or a price of
/// nothing at all. It is not a count of anything.
pub const PLENTY: i32 = 10;

/// How many of a thing the player can afford — `TypeData::can_pay_cost`.
///
/// **This takes the maximum across resources, not the minimum**, and that is
/// the original's arithmetic rather than a transcription slip. The early
/// return blunts it: any resource you cannot afford even one of returns zero
/// at once, so for a single item the maximum and the minimum agree, and a
/// single item is what almost every caller asks about. For a queue order of
/// five it does not agree, and `docs/COSTS.md` says so.
///
/// The early return is `== 0`, not `<= 0`, and the difference is reachable: a
/// purse smaller than its own escrow divides to a negative, which neither
/// returns zero nor raises the maximum, so a price met by nothing but that one
/// resource answers the [`PLENTY`] sentinel. It is the original's quirk and
/// this reproduces it.
pub fn affordable(
    charges: &[i32; RESOURCES],
    ledger: &Ledger,
    available: &[bool; RESOURCES],
    from_escrow: bool,
) -> i32 {
    let mut best = -1;
    for r in Resource::ALL {
        let i = r.index();
        if !available[i] || charges[i] == 0 {
            continue;
        }
        let purse = if from_escrow {
            ledger.bucket[i]
        } else {
            ledger.bucket[i] - ledger.escrow[i]
        };
        let n = purse / charges[i];
        if n == 0 {
            return 0;
        }
        if n > best {
            best = n;
        }
    }
    if best == -1 { PLENTY } else { best }
}

/// Whether the player can afford `want` of a thing — `Leader::can_pay`, which
/// is [`affordable`] compared against the count in a queue order.
pub fn can_pay(
    charges: &[i32; RESOURCES],
    ledger: &Ledger,
    available: &[bool; RESOURCES],
    want: i32,
) -> bool {
    want <= affordable(charges, ledger, available, false)
}

/// Takes the price out of the stockpile — `Type::pay_cost`.
///
/// The clamp at zero is the original's and is not decoration: a payment can be
/// larger than the purse when the caller did not ask [`can_pay`] first, and
/// the bucket floors rather than going negative.
///
/// Escrow is a soft reservation. An ordinary payment that would need more than
/// the non-escrowed part abandons the whole reservation rather than consuming
/// part of it; a payment marked against the escrow draws the escrow down
/// instead.
pub fn pay(
    charges: &[i32; RESOURCES],
    ledger: &mut Ledger,
    available: &[bool; RESOURCES],
    from_escrow: bool,
) {
    for r in Resource::ALL {
        let i = r.index();
        if !available[i] {
            continue;
        }
        let due = charges[i];
        if !from_escrow && ledger.bucket[i] - ledger.escrow[i] < due {
            ledger.escrow[i] = 0;
        }
        ledger.bucket[i] = (ledger.bucket[i] - due).max(0);
        if from_escrow {
            ledger.escrow[i] = (ledger.escrow[i] - due).max(0);
        }
    }
}

/// What Science takes off a technology's price —
/// `LeaderData::calc_science_discount@006da630`, whole.
///
/// ```text
/// cost - ahead * TECH_SCIENCE_DISCOUNT * cost / 100
/// ```
///
/// `ahead` is the player's `epoch[3]` less the technology's own level, where
/// that level is the `AGE` column **plus one** for a tech that is neither an
/// age nor a library epoch. It is deliberately **not** written as
/// `cost * (100 - pct) / 100`: the original truncates the term and subtracts
/// it, and the two disagree by one wherever `pct * cost` is not a multiple of
/// a hundred. The divide truncates toward zero on both signs, which is what
/// makes the surcharge arm (negative `ahead`) round the player's way exactly
/// as the discount arm does.
///
/// [`reprice`] is this function's inverse, and the asymmetry between them —
/// the plus-one, which `reprice` does not have — is recorded there.
pub const fn science_discount(t: &Tuning, ahead: i32, cost: i32) -> i32 {
    cost - ahead * t.tech_science_discount * cost / 100
}

/// What a queued technology costs now that the player's Science level has
/// risen by one — `Build::refund_cost@00620490`, per `(resource, amount)`
/// pair.
///
/// The number is **reconstructed, not remembered**: the amount paid is
/// divided by the science discount that was in force when it was paid, and
/// the base that comes out is re-discounted one level further along. So the
/// queue never stores a price; it stores a price *and* the level it was
/// struck at, implicitly, and this recovers both.
///
/// `level` is `epoch[3]` as it stands **before** the gain — `Leader::gain_tech`
/// calls this before it raises the counter — and `age` is the tech's own
/// `AGE` column, taken raw. That raw reading is the asymmetry worth knowing:
/// [`crate::economy`]'s purchase-side twin, `LeaderData::calc_science_discount`,
/// adds one to `age` for a tech that is neither an age nor a library tech, and
/// this does not. The reconstruction is therefore exact for an age or a
/// library tech and one level out for a plain one — a Ancient plain tech
/// queued at Science 0 comes back a little dearer than it went in.
///
/// Integer throughout, with the original's two truncations: the divide that
/// recovers the base rounds toward zero, and so does the per-cent that
/// re-applies the discount, which the original spells as a divide by −100.
pub fn reprice(t: &Tuning, paid: i32, level: i32, age: i32) -> i32 {
    let ahead = level - age;
    let denom = 100 - t.tech_science_discount * ahead;
    if denom == 0 {
        return paid;
    }
    let base = paid * 100 / denom;
    base - (ahead + 1) * t.tech_science_discount * base / 100
}

/// Puts the price back — `Type::unpay_cost`.
pub fn unpay(charges: &[i32; RESOURCES], ledger: &mut Ledger, available: &[bool; RESOURCES]) {
    for r in Resource::ALL {
        let i = r.index();
        if available[i] {
            ledger.bucket[i] += charges[i];
        }
    }
}

/// A city's level, which is what it contributes to the population cap and
/// nothing else that has been read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CityLevel {
    #[default]
    Village,
    Town,
    Metropolis,
}

impl CityLevel {
    /// `CityData::pop_cap` — `VILLAGE_POP`, twice it, or three times it.
    /// `VILLAGE_POP` ships as zero, so shipped, a city raises nothing.
    pub const fn pop_cap(self, t: &Tuning) -> i32 {
        t.village_pop
            * match self {
                CityLevel::Village => 1,
                CityLevel::Town => 2,
                CityLevel::Metropolis => 3,
            }
    }
}

/// The things outside the military table that move a population cap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PopBonuses {
    /// A scenario's explicit cap for this player, which overrides everything
    /// before the last two lines.
    pub scenario: Option<i32>,
    /// The leader flag `0x100000`, the scenario editor's "ignore population
    /// cap". It hands the player the lobby limit outright and skips the same
    /// way a scenario cap does.
    pub ignore_cap: bool,
    pub cities: Vec<CityLevel>,
    pub bantu: bool,
    pub virtual_reality: bool,
    pub peacocks: bool,
    pub colossus: bool,
}

/// The step the three dead clauses of `calc_pop_cap` add per military level.
///
/// A literal in the original with no constant behind it, and exactly
/// `POP_CAP`'s own step.
const POP_CAP_STEP: i32 = 25;

/// A player's population cap — `Leader::calc_pop_cap`.
///
/// `military_level` indexes `POP_CAP` and `limit` is the lobby's population
/// setting. **It is the Military library level, not the age**: the original
/// indexes with `epoch[0]`, which `LeaderData::compute_epoch` computes by
/// counting consecutive `has_tech` up the `BASE_MILITARYTYPES` line, and the
/// shipped line is seven techs — The Art of War through Selective Service. So
/// the eight entries of `POP_CAP` are "no military tech" through "all seven",
/// and the ages never enter it.
///
/// The three bracketed clauses below cannot fire in a stock game: the largest
/// setting the lobby offers is exactly `POP_CAP[7]`, so `limit > POP_CAP[7]`
/// is only reachable from a scenario. They are here because a scenario is
/// exactly what would reach them.
///
/// Shipped and stripped of everything that never fires, this is
/// `min(POP_CAP[military_level], limit)` plus the Colossus. Cities do not
/// raise it.
pub fn pop_cap(t: &Tuning, military_level: usize, limit: i32, b: &PopBonuses) -> i32 {
    let mut cap = match b.scenario {
        Some(explicit) => explicit,
        None if b.ignore_cap => limit,
        None => {
            let level = military_level.min(t.pop_cap.len() - 1);
            let base = t.pop_cap[level];
            let top = t.pop_cap[t.pop_cap.len() - 1];
            let level = level as i32;

            let mut cap = base;
            let mut limit = limit;
            if limit > top {
                let over = limit - top;
                if level == 7 {
                    cap = limit;
                } else if over >= 100 {
                    if level > 3 {
                        cap = base + (level - 3) * POP_CAP_STEP;
                    }
                } else if over > 49 && level > 5 {
                    cap = base + (level - 5) * POP_CAP_STEP;
                }
            }

            for city in &b.cities {
                cap += city.pop_cap(t);
            }

            if b.bantu {
                cap = (100 + t.bantu_pop_cap) * cap / 100;
                limit = (100 + t.bantu_final_pop_cap) * limit / 100;
            }
            cap = cap.min(limit);
            if b.virtual_reality {
                cap = limit;
            }
            cap
        }
    };

    if b.peacocks {
        cap = (100 + t.peacocks_pop) * cap / 100;
    }
    if b.colossus {
        cap += t.colossus_pop_cap;
    }
    cap
}

/// Whether adding one more of a type would exceed the cap —
/// `LeaderData::check_population`.
///
/// Note the strictness: a unit that lands exactly on the cap is allowed.
pub const fn exceeds_population(cap: i32, control: i32, pop: i32) -> bool {
    cap < control + pop
}

/// `BuildTypeData::get_kill_value@00639790`: the type's base value (its slot `+0x7c` handed 0, asked first and
/// always) and, for a player (`who >= 0`), the two support words `+0x270` and `+0x274` summed, times
/// `BUILD_SUPPORT_FACTOR`, times the player's built and ordered count of the type (`num_buildings[t - 0x19e] +
/// num_queued[t]`, both unsigned 16-bit) on top — 32-bit and wrapping, as the `imull`s are. `counts` is `None`
/// for a negative `who`, which reads no leader. Item 2112's sweep holds it on random words.
#[must_use]
pub fn kill_value_by(
    base: i32,
    support: (i32, i32),
    factor: i32,
    counts: Option<(u16, u16)>,
) -> i32 {
    let Some((built, queued)) = counts else {
        return base;
    };
    support
        .1
        .wrapping_add(support.0)
        .wrapping_mul(factor)
        .wrapping_mul(i32::from(built) + i32::from(queued))
        .wrapping_add(base)
}

#[cfg(test)]
mod tests;
