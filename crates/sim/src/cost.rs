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
    /// Percentage off the scaled base, before the ramp.
    pub discount: i32,
    /// Percentage off the whole price, after the ramp. `MILITARY_UNIT_DISCOUNT`,
    /// Monarchy on stable units, Socialism on siege, air and dock units, Salmon
    /// on ships, then Sugar, Coal, Gold, Iron and Gypsum by resource, and — in
    /// the original, after the captured-building doubling rather than before it
    /// — the Supercollider surcharge and the Indian elephant discount.
    pub late_discount: i32,
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
    /// Maize halves the ramp term — `MAIZE_RAMPING_BONUS`.
    pub maize: bool,
    /// Producing at a captured, unassimilated building doubles the price.
    pub unassimilated: bool,
    /// `get_cost`'s **research** arm: the type is not yet available to the
    /// player (the `leader + 0x6c18` bit is clear), so what is priced is its
    /// research, and it takes the place of the ramp. `None` is the train arm.
    pub research: Option<Research>,
}

/// What `TypeData::get_cost@00664090` charges to **research** a unit type
/// rather than train one (`docs/COSTS.md`, "Researching an upgrade is not
/// building a unit"; `docs/AI.md` §56). In the original's order, after the
/// scaled base and its pre-ramp tail and in place of the ramp:
///
/// 1. `× RESEARCH_PREMIUM >> 8`, then `× RESEARCH_PREMIUM_COST >> 8`, each
///    truncating toward zero (`get_cost:430`–`432`);
/// 2. the refit surcharge, already multiplied out per resource;
/// 3. `MILITARY_UPGRADE_DISCOUNT`, clamped at zero (`get_cost:507`–`525`).
///
/// Not carried, and each is a seam: Wine's `WINE_UNIT_UPGRADES` before the
/// premium, `SPECIAL_UPGRADE` (every one of the shipped 364 records has an
/// empty `<UPGRADE/>`), and the American and Dutch nation discounts after
/// the military one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Research {
    /// `RESEARCH_PREMIUM_COST`, 8.8 (`UnitTypeData +0x2e0`).
    pub premium_cost: i32,
    /// The refit surcharge by resource, `UNIT_COST_FACTOR × n × d` summed
    /// over the army it upgrades — see `Sim::research_modifiers`.
    pub refit: [i32; RESOURCES],
    /// `MILITARY_UPGRADE_DISCOUNT`'s percentage, already scaled for a short
    /// scenario.
    pub discount: i32,
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
    // `LeaderData::calc_science_discount`, where the original calls it: after
    // the factor and the nation tail, before the age-behind discount and the
    // final-tech ramp, and **inside** the per-resource computation, so the
    // redirect at the end of `get_cost` carries the discounted number rather
    // than discounting a redirected one.
    cost = science_discount(t, m.science_ahead, cost);

    if let Some(rs) = m.research {
        // The research arm, in place of the ramp and the military discount.
        cost = cost * t.research_premium / 256;
        cost = cost * rs.premium_cost / 256;
        cost += rs.refit[r.index()];
        if rs.discount != 0 {
            cost = ((100 - rs.discount) * cost / 100).max(0);
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

    cost = cost * (100 - m.late_discount) / 100;
    if m.unassimilated {
        cost *= 2;
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
        const ONE: i32 = 256;
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

    /// The Small City, from `buildingrules.xml`: `COST 1t/1f`,
    /// `SUPPORT food 50 / timber 50`, and no progression column at all.
    fn small_city() -> Price {
        Price {
            kind: Kind::Building,
            class: RampClass::Building,
            ..Price::free()
                .with_base(Resource::Food, 1)
                .with_base(Resource::Timber, 1)
                .with_support(Resource::Food, 50)
                .with_support(Resource::Timber, 50)
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

    /// **A building's ramp has no ceiling**, and that is the difference
    /// between an AI that founds its second city on frame 576 and one that
    /// founds it on 776.
    ///
    /// `TypeData::get_cost` has two ramps. The unit arm reads
    /// `UNIT_COST_FACTOR` (`+0x354`) and one of the four `*_RAMP_MAX`
    /// (`+0x394`..`+0x3a0`); the building arm (`00665787`..`00665b5a`) reads
    /// `BUILD_COST_FACTOR` (`+0x358`) and `BUILD_SUPPORT_FACTOR` (`+0x37c`)
    /// and loads no `RAMP_MAX` at all. With the military 125% applied — which
    /// is what `RampClass::default()` gave every building — the Small City's
    /// `SUPPORT 50` clamps to 12 and the second city costs 22 instead of 60.
    ///
    /// Measured, not inferred: run40 and run41 (`rondata::diff`) put the AI
    /// at 69 food / 59 timber on the frame it cannot buy and 83 / 73 on the
    /// frame it can, and its buckets fall by sixty of each when it does.
    #[test]
    fn a_building_s_ramp_is_not_capped() {
        let city = small_city();
        // The first is free of the ramp, as everything's first is.
        assert_eq!(plain(&city, Resource::Food, owned(0)), 10);
        assert_eq!(plain(&city, Resource::Timber, owned(0)), 10);
        // The second: 10 + 50 x BUILD_SUPPORT_FACTOR x 1, and the military
        // ceiling would have made it 10 + 12.
        assert_eq!(plain(&city, Resource::Food, owned(1)), 60);
        assert_eq!(plain(&city, Resource::Timber, owned(1)), 60);
        assert_eq!(plain(&city, Resource::Metal, owned(1)), 0);
        // And it keeps climbing, linearly, for ever.
        assert_eq!(plain(&city, Resource::Food, owned(4)), 210);
        let capped = Price {
            class: RampClass::Military,
            ..small_city()
        };
        assert_eq!(
            plain(&capped, Resource::Food, owned(4)),
            22,
            "what this crate priced it at before item 81"
        );
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
    /// before Metal is available and metal after — and the timber is five
    /// quarters of the metal, because `UNDISC_COST_RATE` is `5/4`.
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
        // (30 * 320) >> 8.
        assert_eq!(before[Resource::Timber.index()], 37);
    }

    /// A tech priced in knowledge before the Classical Age is charged three
    /// halves of it in food. Mathematics is `8k/12g`: a hundred and twenty
    /// food, not eighty.
    #[test]
    fn an_ancient_tech_pays_three_halves_of_its_knowledge_in_food() {
        let mathematics = Price {
            kind: Kind::Tech,
            ..Price::free()
                .with_base(Resource::Knowledge, 8)
                .with_base(Resource::Wealth, 12)
        };
        let mut available = [true; RESOURCES];
        available[Resource::Knowledge.index()] = false;
        let out = charges(
            &T,
            &mathematics,
            owned(0),
            &Modifiers::default(),
            &available,
            &[false; RESOURCES],
            &Redirects::RON,
        );
        assert_eq!(out[Resource::Food.index()], 120);
        assert_eq!(out[Resource::Wealth.index()], 120);
        assert_eq!(out[Resource::Knowledge.index()], 0);
    }

    /// The redirect is a recursive call, so it chains. Oil names Metal and
    /// Metal names Timber, and in the Ancient age neither is available: an oil
    /// price arrives in timber, through metal, at both rates.
    #[test]
    fn a_redirect_chains_through_a_resource_that_is_itself_unavailable() {
        let tanker = Price {
            kind: Kind::Unit,
            ..Price::free().with_base(Resource::Oil, 3)
        };
        let mut available = [true; RESOURCES];
        available[Resource::Oil.index()] = false;
        available[Resource::Metal.index()] = false;
        let out = charges(
            &T,
            &tanker,
            owned(0),
            &Modifiers::default(),
            &available,
            &[false; RESOURCES],
            &Redirects::RON,
        );
        // (30 * 384) >> 8 = 45 in metal, and (45 * 320) >> 8 = 56 in timber.
        assert_eq!(out[Resource::Timber.index()], 56);
        assert_eq!(out[Resource::Metal.index()], 0);
        assert_eq!(out[Resource::Oil.index()], 0);

        // With Metal available the chain stops one link short.
        available[Resource::Metal.index()] = true;
        let out = charges(
            &T,
            &tanker,
            owned(0),
            &Modifiers::default(),
            &available,
            &[false; RESOURCES],
            &Redirects::RON,
        );
        assert_eq!(out[Resource::Metal.index()], 45);
        assert_eq!(out[Resource::Timber.index()], 0);
    }

    /// An obsolete resource redirects somewhere else than an undiscovered one,
    /// and the choice is the prerequisite. Every `<OBS>` ships as `disable`, so
    /// the obsolete table is inert in a stock game — including its one
    /// interesting entry, Timber at half into Oil.
    #[test]
    fn obsolete_and_undiscovered_are_different_tables() {
        let r = Redirects::RON;
        assert_eq!(r.of(Resource::Metal, false).good, Resource::Timber);
        assert_eq!(r.of(Resource::Metal, false).rate, 320);
        assert_eq!(r.of(Resource::Metal, true).good, Resource::Wealth);
        assert_eq!(r.of(Resource::Metal, true).rate, 256);
        assert_eq!(r.of(Resource::Knowledge, false).good, Resource::Food);
        assert_eq!(r.of(Resource::Knowledge, false).rate, 384);
        assert_eq!(r.of(Resource::Timber, true).good, Resource::Oil);
        assert_eq!(r.of(Resource::Timber, true).rate, 128);
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

    /// The early return is `== 0`. A purse smaller than its own escrow divides
    /// to a negative, which is neither zero nor larger than the running
    /// maximum, so the loop ends having raised nothing and answers the
    /// sentinel. The original's quirk, reproduced rather than corrected.
    #[test]
    fn a_purse_below_its_own_escrow_falls_through_to_the_sentinel() {
        let mut l = ledger_with([10, 0, 0, 0, 0, 0]);
        l.escrow[Resource::Food.index()] = 100;
        let all = [true; RESOURCES];
        assert_eq!(affordable(&[30, 0, 0, 0, 0, 0], &l, &all, false), PLENTY);
        // Against the escrow the purse is the whole bucket, and thirty of ten
        // is a plain zero.
        assert_eq!(affordable(&[30, 0, 0, 0, 0, 0], &l, &all, true), 0);
    }

    /// The index is the Military library level, so this is the table a player
    /// climbs by researching The Art of War and the six techs above it —
    /// twenty-five with none of them, two hundred with all seven.
    #[test]
    fn the_population_cap_is_the_military_table_clamped_by_the_lobby() {
        let b = PopBonuses::default();
        assert_eq!(pop_cap(&T, 0, 200, &b), 25);
        assert_eq!(pop_cap(&T, 1, 200, &b), 50);
        assert_eq!(pop_cap(&T, 7, 200, &b), 200);
        assert_eq!(pop_cap(&T, 7, 100, &b), 100);
        assert_eq!(pop_cap(&T, 3, 50, &b), 50);
    }

    /// The scenario editor's ignore-cap flag hands over the lobby limit and
    /// skips the table, the cities and the clamp — but not the wonders.
    #[test]
    fn the_ignore_cap_flag_is_the_lobby_limit_outright() {
        let b = PopBonuses {
            ignore_cap: true,
            ..PopBonuses::default()
        };
        assert_eq!(pop_cap(&T, 0, 200, &b), 200);
        let with_colossus = PopBonuses {
            colossus: true,
            ..b
        };
        assert_eq!(pop_cap(&T, 0, 200, &with_colossus), 250);
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
    #[test]
    fn science_prices_an_epoch_flat_and_surcharges_a_plain_tech() {
        // run40's own number: City State is `12f`, so `120` after
        // `TECH_COST_FACTOR`, and the AI pays exactly that. It is an *epoch*,
        // so its level is its `AGE` of zero with no plus-one, and a player at
        // Science 0 is neither ahead nor behind.
        let t = Tuning::RON;
        assert_eq!(science_discount(&t, 0, 120), 120);
        // A level of Science ahead takes ten percent off, two levels twenty —
        // off the base each time, not compounding.
        assert_eq!(science_discount(&t, 1, 120), 108);
        assert_eq!(science_discount(&t, 2, 120), 96);
        // Behind, the same expression charges more. A *plain* tech is where
        // this bites in a shipped game: its level is `AGE + 1`, so an Ancient
        // one is already a level ahead of a player who has researched no
        // Science at all, and costs 110% until they do.
        assert_eq!(science_discount(&t, -1, 120), 132);
        assert_eq!(science_discount(&t, -3, 120), 156);
        // The truncation is the original's: the term is truncated toward zero
        // and then taken away, on both signs.
        assert_eq!(science_discount(&t, 1, 5), 5, "10% of 5 truncates to 0");
        assert_eq!(science_discount(&t, -1, 5), 5);
    }

    #[test]
    fn the_refund_inverts_the_discount_exactly_for_an_epoch() {
        // [`reprice`] reads the `AGE` column raw where [`science_discount`]
        // adds one for a plain tech, so the pair is an exact inverse for an
        // age or an epoch and one level out for anything else. That asymmetry
        // is the original's, and it is the reason a plain tech queued at one
        // Science level and refunded at the next comes back slightly dearer
        // than it went in.
        let t = Tuning::RON;
        for base in [120, 250, 1_000] {
            for level in 0..4 {
                // An epoch: `science_discount`'s `ahead` and `reprice`'s
                // `level - age` are the same number, so the refund of a price
                // struck at `level` is the price struck at `level + 1`.
                let paid = science_discount(&t, level, base);
                assert_eq!(
                    reprice(&t, paid, level, 0),
                    science_discount(&t, level + 1, base),
                    "base {base} at level {level}"
                );
            }
        }
    }

    #[test]
    fn a_queued_tech_is_repriced_by_reconstruction_not_by_memory() {
        // run40's own numbers: the AI pays 120 food for City State at
        // Science 0, and Written Word lands while it is still in the queue.
        let t = Tuning::RON;
        assert_eq!(reprice(&t, 120, 0, 0), 108, "120 - 10% of 120");
        // And again at the next level, from the *new* stored amount: 108 is
        // divided by the 90% that was in force to recover the 120 base, and
        // re-struck at 80%. That is what makes the chain lossless where a
        // remembered number would compound its truncation.
        assert_eq!(reprice(&t, 108, 1, 0), 96);
        assert_eq!(reprice(&t, 96, 2, 0), 84);
        // A tech whose own age is above the player's Science level costs
        // *more*, from the same expression — the surcharge `docs/COSTS.md`
        // names — and the reconstruction inverts that too.
        assert_eq!(
            reprice(&t, 120, 0, 2),
            110,
            "120 was 100 at a 20% surcharge; three levels behind it is 110"
        );
    }
}
