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

use crate::economy::{Ledger, RESOURCES, Resource};
use crate::tuning::Tuning;

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
/// The original picks between these by type identity and object flags —
/// scholars by name, workers and merchants by predicate, then a civilian flag
/// separating everything else from the fighting units. Taking it as a field is
/// the same choice movement made for `speed`: the mechanic is complete and the
/// classification arrives with the layer that produces it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampClass {
    /// 2000%, and the only class with a second, convex term.
    Scholar,
    /// 500%. Citizens and merchants.
    Worker,
    /// 200%. Generals, spies, supply wagons, caravans.
    OtherCivilian,
    /// 125%. Everything that fights.
    #[default]
    Military,
}

impl RampClass {
    /// The ceiling as a percentage of the scaled base price.
    pub const fn ceiling_percent(self, t: &Tuning) -> i32 {
        match self {
            RampClass::Scholar => t.unit_scholar_ramp_max,
            RampClass::Worker => t.unit_worker_ramp_max,
            RampClass::OtherCivilian => t.unit_other_civilian_ramp_max,
            RampClass::Military => t.unit_military_ramp_max,
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
/// because the shape carries no information and the position does: the
/// national, wonder and government tail lands **before** the ramp and the
/// per-resource rare bonuses land **after** it, so the two cannot be one
/// number without changing what the ceiling is measured against.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Percentage off the scaled base, before the ramp.
    pub discount: i32,
    /// Percentage off the whole price, after the ramp. Sugar, Coal, Gold,
    /// Iron and Gypsum.
    pub late_discount: i32,
    /// Maize halves the ramp term — `MAIZE_RAMPING_BONUS`.
    pub maize: bool,
    /// Producing at a captured, unassimilated building doubles the price.
    pub unassimilated: bool,
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
    let mut cost = scaled * (100 - m.discount) / 100;

    let raw = counts.for_progression(price.progression);
    if raw > 0 {
        let steps = ramp_steps(raw, price.progression);
        let ceiling = price.class.ceiling_percent(t) * scaled / 100;
        let extra = match price.class {
            RampClass::Scholar => scholar_surcharge(steps),
            _ => 0,
        };
        for slot in price.support.iter().flatten() {
            let (good, amount) = *slot;
            if good != r {
                continue;
            }
            let mut term = amount * steps + extra;
            if ceiling != 0 && term > ceiling {
                term = ceiling;
            }
            if m.maize {
                term = term * (100 - t.maize_ramping_bonus) / 100;
            }
            cost += term;
        }
    }

    cost = cost * (100 - m.late_discount) / 100;
    if m.unassimilated {
        cost *= 2;
    }
    cost
}

/// Where a cost written in an unavailable resource is charged instead.
///
/// The rate is 8.8 fixed point, because the original multiplies and shifts
/// right by eight. Every rate in the shipped tables is `1/1`, which is 256.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redirect {
    pub good: Resource,
    pub rate: i32,
}

/// The two redirect tables from `resourcerules.xml`, one for a resource whose
/// prerequisite the player does not hold and one for a resource that has gone
/// obsolete.
///
/// The undiscovered table is the interesting one, because
/// `resourcerules.xml` gives Knowledge and Metal the Classical Age as their
/// prerequisite and Oil the Industrial Age. So in the Ancient Age a tech
/// priced in knowledge is charged in **food** — Mathematics is `8k/12g` and a
/// player who researches it early pays eighty food — and anything priced in
/// metal is charged in **timber**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Redirects {
    pub undiscovered: [Redirect; RESOURCES],
    pub obsolete: [Redirect; RESOURCES],
}

impl Redirects {
    /// What Rise of Nations ships.
    ///
    /// Wealth's undiscovered entry names Wealth, which would recur forever.
    /// It cannot fire: wealth is never unavailable. Nothing else in either
    /// table points at a resource that can itself be unavailable, so one pass
    /// is exact and no recursion is needed.
    pub const RON: Redirects = {
        const fn w() -> Redirect {
            Redirect {
                good: Resource::Wealth,
                rate: 256,
            }
        }
        Redirects {
            undiscovered: [
                w(),
                w(),
                w(),
                Redirect {
                    good: Resource::Food,
                    rate: 256,
                },
                Redirect {
                    good: Resource::Timber,
                    rate: 256,
                },
                w(),
            ],
            obsolete: [w(), w(), w(), w(), w(), w()],
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

/// What a player is actually charged, per resource.
///
/// Unavailable resources come back as zero — `Type::pay_cost` charges only the
/// available ones — and their prices are redirected into whatever the tables
/// name. The redirected amount is the source resource's own full price, added
/// after the target's own discounts, which is where the original adds it.
pub fn charges(
    t: &Tuning,
    price: &Price,
    counts: Counts,
    m: &Modifiers,
    available: &[bool; RESOURCES],
    discovered: &[bool; RESOURCES],
    redirects: &Redirects,
) -> [i32; RESOURCES] {
    let mut raw = [0; RESOURCES];
    for r in Resource::ALL {
        raw[r.index()] = cost_of(t, price, r, counts, m);
    }

    let mut out = [0; RESOURCES];
    for r in Resource::ALL {
        if available[r.index()] {
            out[r.index()] = raw[r.index()];
        }
    }
    for g in Resource::ALL {
        if available[g.index()] || raw[g.index()] == 0 {
            continue;
        }
        let to = redirects.of(g, discovered[g.index()]);
        if available[to.good.index()] {
            out[to.good.index()] += (raw[g.index()] * to.rate) >> 8;
        }
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
        if n <= 0 {
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

/// The things outside the age table that move a population cap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PopBonuses {
    /// A scenario's explicit cap for this player, which overrides everything
    /// before the last two lines.
    pub scenario: Option<i32>,
    pub cities: Vec<CityLevel>,
    pub bantu: bool,
    pub virtual_reality: bool,
    pub peacocks: bool,
    pub colossus: bool,
}

/// The step the three dead clauses of `calc_pop_cap` add per age.
///
/// A literal in the original with no constant behind it, and exactly
/// `POP_CAP`'s own step.
const POP_CAP_STEP: i32 = 25;

/// A player's population cap — `Leader::calc_pop_cap`.
///
/// `age` indexes `POP_CAP` and `limit` is the lobby's population setting. The
/// three bracketed clauses below cannot fire in a stock game: the largest
/// setting the lobby offers is exactly `POP_CAP[7]`, so `limit > POP_CAP[7]`
/// is only reachable from a scenario. They are here because a scenario is
/// exactly what would reach them.
///
/// Shipped and stripped of everything that never fires, this is
/// `min(POP_CAP[age], limit)` plus the Colossus. Cities do not raise it.
pub fn pop_cap(t: &Tuning, age: usize, limit: i32, b: &PopBonuses) -> i32 {
    let mut cap = match b.scenario {
        Some(explicit) => explicit,
        None => {
            let age = age.min(t.pop_cap.len() - 1);
            let base = t.pop_cap[age];
            let top = t.pop_cap[t.pop_cap.len() - 1];
            let age = age as i32;

            let mut cap = base;
            let mut limit = limit;
            if limit > top {
                let over = limit - top;
                if age == 7 {
                    cap = limit;
                } else if over >= 100 {
                    if age > 3 {
                        cap = base + (age - 3) * POP_CAP_STEP;
                    }
                } else if over > 49 && age > 5 {
                    cap = base + (age - 5) * POP_CAP_STEP;
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

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    /// The Citizen, from `unitrules.xml`: `2f`, `1f support`, `PROGRESSION 0`.
    fn citizen() -> Price {
        Price {
            class: RampClass::Worker,
            pop: 1,
            ..Price::free()
                .with_base(Resource::Food, 2)
                .with_support(Resource::Food, 1)
        }
    }

    /// The Hoplite: `5f/3m`, `1f/1m support`, `PROGRESSION 3`.
    fn hoplite() -> Price {
        Price {
            progression: Progression::PROGRESSIVE_BY_GROUP,
            class: RampClass::Military,
            pop: 1,
            ..Price::free()
                .with_base(Resource::Food, 5)
                .with_base(Resource::Metal, 3)
                .with_support(Resource::Food, 1)
                .with_support(Resource::Metal, 1)
        }
    }

    /// The Scholar: `3g`, `2g support`, `PROGRESSION 0`.
    fn scholar() -> Price {
        Price {
            class: RampClass::Scholar,
            pop: 1,
            ..Price::free()
                .with_base(Resource::Wealth, 3)
                .with_support(Resource::Wealth, 2)
        }
    }

    fn owned(n: i32) -> Counts {
        Counts {
            of_type: n,
            of_group: n,
        }
    }

    fn plain(price: &Price, r: Resource, counts: Counts) -> i32 {
        cost_of(&T, price, r, counts, &Modifiers::default())
    }

    #[test]
    fn base_cost_is_ten_times_the_file() {
        assert_eq!(plain(&citizen(), Resource::Food, owned(0)), 20);
        assert_eq!(plain(&hoplite(), Resource::Food, owned(0)), 50);
        assert_eq!(plain(&hoplite(), Resource::Metal, owned(0)), 30);
        assert_eq!(plain(&scholar(), Resource::Wealth, owned(0)), 30);
    }

    /// The first one is never ramped: the count is what you already have, and
    /// the guard in the original is `count > 0`.
    #[test]
    fn the_first_of_anything_is_unramped() {
        for n in [0, 1, 2] {
            assert_eq!(plain(&citizen(), Resource::Food, owned(n)), 20 + n);
        }
    }

    #[test]
    fn a_citizen_ramps_by_one_food_each_to_five_times_its_base() {
        let c = citizen();
        assert_eq!(plain(&c, Resource::Food, owned(19)), 39);
        assert_eq!(plain(&c, Resource::Food, owned(49)), 69);
        // 500% of the scaled base is a hundred, and nothing past it moves.
        assert_eq!(plain(&c, Resource::Food, owned(100)), 120);
        assert_eq!(plain(&c, Resource::Food, owned(500)), 120);
    }

    /// The triangular count reaches the 125% ceiling at eleven, and a hoplite
    /// never costs more than the twelfth one did.
    #[test]
    fn a_military_ramp_is_triangular_and_plateaus() {
        let h = hoplite();
        assert_eq!(plain(&h, Resource::Food, owned(1)), 51);
        assert_eq!(plain(&h, Resource::Food, owned(10)), 50 + 55);
        assert_eq!(plain(&h, Resource::Food, owned(11)), 50 + 62);
        assert_eq!(plain(&h, Resource::Food, owned(100)), 112);
        assert_eq!(plain(&h, Resource::Metal, owned(100)), 67);
    }

    /// Elephants are the four records that ramp triangularly against their own
    /// type rather than the stable's whole output.
    #[test]
    fn progression_picks_which_count_is_read() {
        let by_group = Progression::PROGRESSIVE_BY_GROUP;
        let by_type = Progression::PROGRESSIVE_BY_TYPE;
        let counts = Counts {
            of_type: 2,
            of_group: 10,
        };
        assert_eq!(counts.for_progression(by_group), 10);
        assert_eq!(counts.for_progression(by_type), 2);
        assert_eq!(ramp_steps(4, by_type), 10);
        assert_eq!(ramp_steps(4, Progression::BY_TYPE), 4);
    }

    #[test]
    fn a_scholar_carries_a_second_term_every_seven() {
        assert_eq!(scholar_surcharge(7), 0);
        assert_eq!(scholar_surcharge(8), 1);
        assert_eq!(scholar_surcharge(21), 21);
        let s = scholar();
        // The eighth scholar: seven owned, two wealth each, no second term yet.
        assert_eq!(plain(&s, Resource::Wealth, owned(7)), 44);
        // The ninth: the term opens.
        assert_eq!(plain(&s, Resource::Wealth, owned(8)), 47);
        assert_eq!(plain(&s, Resource::Wealth, owned(21)), 93);
    }

    /// The ceiling is a percentage of the base *before* the discount, so a
    /// discount does not shrink the ramp's headroom.
    #[test]
    fn the_ceiling_ignores_the_discount() {
        let c = citizen();
        let m = Modifiers {
            discount: 50,
            ..Modifiers::default()
        };
        assert_eq!(cost_of(&T, &c, Resource::Food, owned(500), &m), 10 + 100);
    }

    #[test]
    fn maize_halves_the_ramp_and_nothing_else() {
        let c = citizen();
        let m = Modifiers {
            maize: true,
            ..Modifiers::default()
        };
        assert_eq!(cost_of(&T, &c, Resource::Food, owned(40), &m), 20 + 20);
    }

    #[test]
    fn a_captured_building_doubles_the_finished_price() {
        let c = citizen();
        let m = Modifiers {
            unassimilated: true,
            ..Modifiers::default()
        };
        assert_eq!(cost_of(&T, &c, Resource::Food, owned(10), &m), 60);
    }

    /// The headline of the whole document: the same hoplite costs timber
    /// before Metal is available and metal after.
    #[test]
    fn metal_is_charged_as_timber_until_it_is_available() {
        let h = hoplite();
        let m = Modifiers::default();
        let all = [true; RESOURCES];
        let none = [false; RESOURCES];

        let after = charges(&T, &h, owned(0), &m, &all, &none, &Redirects::RON);
        assert_eq!(after[Resource::Food.index()], 50);
        assert_eq!(after[Resource::Metal.index()], 30);
        assert_eq!(after[Resource::Timber.index()], 0);

        let mut early = all;
        early[Resource::Metal.index()] = false;
        let before = charges(&T, &h, owned(0), &m, &early, &none, &Redirects::RON);
        assert_eq!(before[Resource::Food.index()], 50);
        assert_eq!(before[Resource::Metal.index()], 0);
        assert_eq!(before[Resource::Timber.index()], 30);
    }

    /// An obsolete resource redirects somewhere else than an undiscovered one,
    /// and the choice is the prerequisite.
    #[test]
    fn obsolete_and_undiscovered_are_different_tables() {
        let r = Redirects::RON;
        assert_eq!(r.of(Resource::Metal, false).good, Resource::Timber);
        assert_eq!(r.of(Resource::Metal, true).good, Resource::Wealth);
        assert_eq!(r.of(Resource::Knowledge, false).good, Resource::Food);
    }

    fn ledger_with(bucket: [i32; RESOURCES]) -> Ledger {
        Ledger {
            bucket,
            ..Ledger::default()
        }
    }

    #[test]
    fn paying_takes_only_available_resources_and_floors_at_zero() {
        let mut l = ledger_with([100, 0, 0, 0, 100, 0]);
        let charges = [50, 0, 0, 0, 200, 0];
        let all = [true; RESOURCES];
        pay(&charges, &mut l, &all, false);
        assert_eq!(l.bucket[Resource::Food.index()], 50);
        assert_eq!(l.bucket[Resource::Metal.index()], 0);
    }

    #[test]
    fn unpaying_puts_it_back() {
        let mut l = ledger_with([100, 0, 0, 0, 0, 0]);
        let charges = [30, 0, 0, 0, 0, 0];
        let all = [true; RESOURCES];
        pay(&charges, &mut l, &all, false);
        unpay(&charges, &mut l, &all);
        assert_eq!(l.bucket[Resource::Food.index()], 100);
    }

    #[test]
    fn escrow_is_invisible_until_it_is_needed_and_then_abandoned() {
        let mut l = ledger_with([100, 0, 0, 0, 0, 0]);
        l.escrow[Resource::Food.index()] = 60;
        let all = [true; RESOURCES];

        // Forty is spendable; fifty is not.
        assert!(can_pay(&[40, 0, 0, 0, 0, 0], &l, &all, 1));
        assert!(!can_pay(&[50, 0, 0, 0, 0, 0], &l, &all, 1));

        // Paying fifty anyway abandons the whole reservation.
        pay(&[50, 0, 0, 0, 0, 0], &mut l, &all, false);
        assert_eq!(l.bucket[Resource::Food.index()], 50);
        assert_eq!(l.escrow[Resource::Food.index()], 0);
    }

    /// Recorded because it is the original's arithmetic and it is wrong: the
    /// number affordable is the minimum over resources, and this is the
    /// maximum. The early return is why nobody notices.
    #[test]
    fn the_affordability_count_takes_the_maximum() {
        let l = ledger_with([100, 0, 0, 0, 100, 0]);
        let all = [true; RESOURCES];
        // Ten food's worth, two metal's worth.
        let charges = [10, 0, 0, 0, 50, 0];
        assert_eq!(affordable(&charges, &l, &all, false), 10);
        // But one you cannot afford at all still answers zero.
        let charges = [10, 0, 0, 0, 200, 0];
        assert_eq!(affordable(&charges, &l, &all, false), 0);
        // And a price of nothing answers the sentinel.
        assert_eq!(affordable(&[0; RESOURCES], &l, &all, false), PLENTY);
    }

    #[test]
    fn the_population_cap_is_the_age_table_clamped_by_the_lobby() {
        let b = PopBonuses::default();
        assert_eq!(pop_cap(&T, 0, 200, &b), 25);
        assert_eq!(pop_cap(&T, 7, 200, &b), 200);
        assert_eq!(pop_cap(&T, 7, 100, &b), 100);
        assert_eq!(pop_cap(&T, 3, 50, &b), 50);
    }

    #[test]
    fn cities_raise_nothing_because_village_pop_ships_as_zero() {
        let b = PopBonuses {
            cities: vec![CityLevel::Metropolis; 4],
            ..PopBonuses::default()
        };
        assert_eq!(pop_cap(&T, 2, 200, &b), 75);
    }

    #[test]
    fn the_colossus_and_the_bantu_move_the_cap() {
        let colossus = PopBonuses {
            colossus: true,
            ..PopBonuses::default()
        };
        assert_eq!(pop_cap(&T, 7, 200, &colossus), 250);

        // The Bantu double the cap and raise the ceiling a quarter, so the
        // clamp lands on the raised ceiling rather than the doubled cap.
        let bantu = PopBonuses {
            bantu: true,
            ..PopBonuses::default()
        };
        assert_eq!(pop_cap(&T, 7, 200, &bantu), 250);
        assert_eq!(pop_cap(&T, 0, 200, &bantu), 50);
    }

    #[test]
    fn a_scenario_cap_overrides_the_age_table_but_not_the_wonders() {
        let b = PopBonuses {
            scenario: Some(17),
            colossus: true,
            ..PopBonuses::default()
        };
        assert_eq!(pop_cap(&T, 0, 200, &b), 67);
    }

    #[test]
    fn landing_exactly_on_the_cap_is_allowed() {
        assert!(!exceeds_population(25, 24, 1));
        assert!(exceeds_population(25, 25, 1));
        assert!(!exceeds_population(25, 25, 0));
    }
}
