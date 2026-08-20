//! Attrition: units losing health for standing in hostile national territory.
//!
//! Attrition is **a period, not a damage number**. A unit in hostile territory
//! is assigned a tick period in frames; the smaller the period, the faster it
//! bleeds. Damage per tick depends only on squad size. Frames are fifteenths
//! of a second.
//!
//! Three quantities produce the period: what the territory's owner inflicts
//! ([`strength`]), what the victim resists ([`resistance`]), and what the
//! individual unit is ([`susceptibility`]). See `docs/ATTRITION.md` for how
//! each was established.
//!
//! # No floats, and no float-shaped integers either
//!
//! The original keeps resistance in an `f32` in the middle of its simulation.
//! It gets away with that because it only ever shipped one x86 build; we
//! cannot, and `CLAUDE.md` forbids it outright.
//!
//! The float is never doing anything a rational cannot: its only inputs are
//! `256` and a chain of `100 / (100 - pct)` factors with integer `pct`. So
//! [`Resistance`] carries an exact numerator and denominator and divides once,
//! at the same place the original truncates. Truncating per factor instead
//! would drift, and going through `Fx` would round at 1/65536 where the
//! original does not round at all — exact integer rationals are the only
//! representation that neither invents error nor imports it.

use crate::tuning::Tuning;
use crate::world::{Owner, Player};

/// How much attrition a player inflicts on others inside their borders.
///
/// Recomputed per player, not per victim. Zero means this player inflicts none
/// at all, which is the state everyone starts in: the tech chain has to be
/// entered before borders hurt anybody.
///
/// `tech_steps` is how many steps of the attrition tech chain the player
/// holds. It indexes `ATTRITION_IMPROVED`, so zero steps is zero strength.
///
/// The four multipliers apply **in this order**, each as
/// `x = max(1, (100 + pct) * x / 100)`. Order matters because each step
/// truncates: reordering them changes the result.
pub fn strength(t: &Tuning, tech_steps: usize, m: &StrengthMods) -> i32 {
    if tech_steps == 0 {
        return 0;
    }
    let mut x = t.attrition_improved[(tech_steps - 1).min(t.attrition_improved.len() - 1)];
    let mut bump = |pct: i32| x = ((100 + pct) * x / 100).max(1);
    if m.colosseum {
        bump(t.colosseum_attrition);
    }
    if m.russian {
        bump(t.russian_attrition);
    }
    if m.conquest_bonus {
        bump(t.ctw_attrition);
    }
    if m.kremlin {
        bump(t.kremlin_attrition);
    }
    x
}

/// What multiplies the attrition a player inflicts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StrengthMods {
    pub colosseum: bool,
    pub russian: bool,
    /// The Conquer the World campaign bonus.
    pub conquest_bonus: bool,
    pub kremlin: bool,
}

/// How much attrition a player's units resist.
///
/// An exact rational, because that is what it is. The original's `f32` starts
/// at 256.0 and is multiplied by `100 / (100 - pct)` for each source; the
/// product of those factors over the shipped data is never an integer, so the
/// truncation at the end always has room and an exact rational lands on the
/// same value the float does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resistance {
    /// Total immunity. Any single source at 100% or more produces this, which
    /// is how the Statue of Liberty works.
    Immune,
    Scaled {
        num: i64,
        den: i64,
    },
}

impl Resistance {
    /// The unmodified baseline, 256.
    pub const BASE: Resistance = Resistance::Scaled { num: 256, den: 1 };
}

/// Recomputes a player's resistance from its techs, wonders and resources.
///
/// The original applies its factors in a fixed order — Foraging, then the
/// Statue of Liberty, then the Mongol bonus, then titanium — because with
/// floats the order is observable. With exact rationals it is not, which is
/// one fewer thing that can go subtly wrong.
pub fn resistance(t: &Tuning, m: &ResistanceMods) -> Resistance {
    let mut num: i64 = 256;
    let mut den: i64 = 1;
    let mut apply = |pct: i32| -> bool {
        if pct >= 100 {
            return false;
        }
        num *= 100;
        den *= i64::from(100 - pct);
        true
    };

    let ok = m
        .foraging
        .map(|tier| t.attrition_upgrade[(tier as usize - 1).min(t.attrition_upgrade.len() - 1)])
        .into_iter()
        .chain(m.liberty.then_some(t.liberty_attrition))
        .chain(m.mongol.then_some(t.mongol_attrition))
        .chain(m.titanium.then_some(t.titanium_attrition))
        .all(&mut apply);

    if ok {
        Resistance::Scaled { num, den }
    } else {
        Resistance::Immune
    }
}

/// What multiplies the attrition a player's units resist.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResistanceMods {
    /// Foraging tier, 1 to 3. Tier 1 additionally makes idle units immune.
    pub foraging: Option<u8>,
    pub liberty: bool,
    pub mongol: bool,
    pub titanium: bool,
}

/// What kind of unit a unit is, as far as attrition is concerned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitKind {
    /// Siege units take half rate.
    pub siege: bool,
    /// Militia ignore their player's resistance entirely and take four times
    /// the rate at the shipped 300%.
    pub militia: bool,
    /// Merchants, Dutch merchants and fur trappers take double rate — the
    /// three economic units whose whole job is to stand outside your borders.
    pub trader: bool,
    /// The unit type's domain. See [`Domain`].
    pub domain: Domain,
    /// Whether the unit is of a kind attrition does not apply to at all —
    /// eligibility check 8, whose rule is [`kind_exempt`]. Workers, merchants,
    /// heroes and supply units are **not** exempt by kind; the rest are if
    /// their type has zero base attack, is "special", a spy, or a caravan.
    pub exempt_kind: bool,
    /// A supply unit. Not exempt by kind: exempt from the *computed* period
    /// only, and only when the bleed did not come from the peace or assassin
    /// path — over a peaceful border a wagon bleeds like anything else. Also
    /// read by the supply shelter, which never covers a wagon.
    pub supply_unit: bool,
    /// A worker actively gathering from a site whose `GatherOrder` is
    /// `non_flat_gather` — a property of the building type being gathered
    /// from, not a scenario flag. Exempt while it lasts.
    pub gathering_non_flat: bool,
    /// Whether the unit is standing still, for the Foraging tier 1 exemption.
    pub idle: bool,
}

impl UnitKind {
    /// Whether standing inside a friendly supply radius can shelter this unit
    /// from a tick that is otherwise due.
    ///
    /// Militia never are, which is the other half of why they take four times
    /// the rate: they are the emergency defenders of your own ground, and the
    /// game declines to let them campaign behind a supply wagon. Supply units
    /// are excluded too: a wagon does not shelter itself.
    pub const fn shelterable(&self) -> bool {
        !self.militia && !self.supply_unit
    }
}

/// A unit type's domain — the `i32` at `UnitTypeData + 0x218`, which decides
/// which attrition period a unit can reach at all.
///
/// The PDB leaves the field unnamed; the reading is from
/// `ObjectData::num_aircraft_here` counting `2` and `ObjectData::in_a_ship` /
/// `Unit::add_to_army` testing `1`, and it is consistent everywhere. An
/// earlier draft of this crate called it a three-way "attrition mode"; the code
/// paths were right and the meaning was not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Domain {
    /// The ordinary case, and the only one that gets a computed period.
    #[default]
    Land,
    /// Ships are outright immune.
    Sea,
    /// Aircraft take the special periods at half length, and never a computed
    /// one — in a war zone with no assassin target an aircraft takes nothing.
    /// The halving inside [`susceptibility`] is therefore unreachable in the
    /// original's own tick — it is kept because the function is called from
    /// elsewhere.
    Air,
}

/// What a unit's type is, as far as eligibility check 8 is concerned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KindFacts {
    pub worker: bool,
    pub merchant: bool,
    pub hero: bool,
    pub supply: bool,
    /// The type's base attack, `UnitTypeData + 0x1e8`.
    pub attack: i32,
    pub special: bool,
    pub spy: bool,
    pub caravan: bool,
}

/// Eligibility check 8: whether a unit is of a kind attrition never applies
/// to. The original's block is nested, and the nesting *is* the rule.
///
/// Workers, merchants, heroes and supply units skip the inner tests and are
/// **not** exempt here — a worker has its own gathering exemption
/// ([`UnitKind::gathering_non_flat`]) and a supply unit its own conditional one
/// at the computed-period step. Every other type is exempt if its base attack
/// is zero, or it is "special", a spy, or a caravan.
///
/// The second reading found the earlier draft of this rule wrong in three
/// places at once — heroes exempt, supply units exempt, no zero-attack gate —
/// which is why the predicate is a function with a test rather than a comment
/// on a field.
pub const fn kind_exempt(f: &KindFacts) -> bool {
    if f.worker || f.merchant || f.hero || f.supply {
        return false;
    }
    f.attack == 0 || f.special || f.spy || f.caravan
}

/// Everything about a player that attrition reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerState {
    /// Attrition this player inflicts, from [`strength`].
    pub strength: i32,
    /// Attrition this player's units resist, from [`resistance`].
    pub resistance: Resistance,
    /// Current age, for the aged-up strength bonus.
    pub age: i32,
    /// Period applied to this player's units on unowned ground. Zero, which is
    /// the normal case, means unowned ground is safe.
    pub neutral_attrition: i32,
    /// Set by a scenario: this player's units take no attrition.
    pub take_disabled: bool,
    /// Set by a scenario: this player's territory inflicts none.
    pub give_disabled: bool,
    /// Whether the player slot is in use and initialised.
    pub active: bool,
    /// Whether the player holds Foraging tier 1, which makes idle units immune.
    pub foraging_1: bool,
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            strength: 0,
            resistance: Resistance::BASE,
            age: 0,
            neutral_attrition: 0,
            take_disabled: false,
            give_disabled: false,
            active: true,
            foraging_1: false,
        }
    }
}

/// How readily one unit takes attrition from one owner: the original's
/// `UnitData::get_attrition`, which returns `resistance / strength`.
///
/// Zero means immune. The original uses the same sentinel and its caller tests
/// for it, so the two are the same statement.
pub fn susceptibility(
    t: &Tuning,
    kind: &UnitKind,
    victim: &PlayerState,
    owner: &PlayerState,
) -> i32 {
    if owner.strength == 0 || t.attrition == 0 {
        return 0;
    }

    // Base scale, 256, or the siege units' larger one.
    let scale = if kind.siege {
        if t.siege_attrition > 99 {
            return 0;
        }
        25600 / (100 - t.siege_attrition)
    } else {
        256
    };

    let mut v = if kind.militia {
        // Militia do not get their player's resistance at all.
        scale * 100 / (t.militia_attrition + 100)
    } else {
        if victim.foraging_1 && kind.idle {
            return 0;
        }
        match victim.resistance {
            Resistance::Immune => return 0,
            // One truncation, at the same place the original's single
            // float-to-int conversion lands.
            Resistance::Scaled { num, den } => (i64::from(scale) * num / (den * 256)) as i32,
        }
    };

    if kind.trader {
        v /= 2;
    }
    if kind.domain == Domain::Air {
        v /= 2;
    }

    // Being ahead in age makes your borders hurt more. Rounds up, unlike
    // everything else here.
    let mut strength = owner.strength;
    let ages_ahead = owner.age - victim.age;
    if ages_ahead >= 0 {
        strength = ((t.attrition_aged_up * ages_ahead + 100) * strength + 99) / 100;
    }

    v / strength
}

/// The tick period in frames for a given susceptibility, or `None` if the unit
/// is immune.
///
/// ```text
/// period = max(1, ATTRITION * susceptibility / 256)
/// ```
pub fn period(t: &Tuning, susceptibility: i32) -> Option<i32> {
    if susceptibility == 0 {
        return None;
    }
    // The original divides by 256 as an arithmetic shift with a bias that
    // makes it truncate toward zero rather than floor. For the non-negative
    // values reachable here the two agree, and Rust's `/` is already the
    // truncating one.
    Some((t.attrition * susceptibility / 256).max(1))
}

/// Why a unit is not taking attrition this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exempt {
    /// Inside a scenario-defined attrition-free radius.
    FreeZone,
    /// The ground is unowned and this player has no neutral attrition set.
    NeutralGround,
    /// The unit is standing in its own territory.
    OwnTerritory,
    /// The claimant is not a live player.
    NoOwner,
    /// Both sides hold a mutual treaty.
    Treaty,
    /// A scenario disabled taking or giving attrition.
    Disabled,
    /// The unit is a kind attrition does not apply to — [`kind_exempt`], or a
    /// worker gathering from a non-flat site.
    UnitKind,
    /// The unit is a ship.
    Ship,
    /// The victim has a neutral-ground period set, which in the original
    /// replaces territorial attrition rather than adding to it.
    NeutralOverride,
    /// A Conquer the World conquest bonus.
    ConquestBonus,
    /// Not at war, inside the grace window after an alliance was broken.
    /// Dead code with shipped data: the constants behind it are absent from
    /// `rules.xml`. See [`Situation::in_war_grace`].
    WarGrace,
    /// A land supply unit whose bleed did not come from the peace or assassin
    /// path. Over a peaceful border the wagon bleeds like anything else.
    SupplyUnit,
    /// The unit is subject in principle but resists completely — or is an
    /// aircraft with no special period to take.
    Immune,
}

/// What one tick of attrition does to one unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing, for this reason.
    Exempt(Exempt),
    /// The unit is subject to attrition on this period, in frames.
    Period {
        frames: i32,
        /// Whether this bleed goes through supply. The peace and assassin
        /// paths set a flag that makes the supply check give up immediately,
        /// so a supply wagon shelters an army in a war zone but not a unit
        /// caught over a border in peacetime. See [`Situation::at_war`].
        ignores_supply: bool,
    },
}

impl Outcome {
    /// The period, if there is one.
    pub const fn frames(self) -> Option<i32> {
        match self {
            Outcome::Period { frames, .. } => Some(frames),
            Outcome::Exempt(_) => None,
        }
    }
}

/// Facts about the situation a unit is in, other than the unit itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Situation {
    /// Who owns the cell the unit is standing in.
    pub ground: Owner,
    /// Whether the unit is inside a scenario attrition-free radius.
    pub in_free_zone: bool,
    /// Whether the two players hold the mutual treaty that stops attrition.
    pub mutual_treaty: bool,
    /// Whether the two are at war. At peace, a border violation earns the
    /// short `PEACE_ATTRITION` period *as well as* whatever the ordinary
    /// calculation gives, and the shorter of the two wins.
    pub at_war: bool,
    /// Not at war, and an alliance between the two was broken less than
    /// `ALLY_TO_WAR_GRACE` frames ago, during which the border does not bite
    /// at all. With shipped data this can never be true: `ALLY_TO_WAR_DELAY`
    /// and `ALLY_TO_WAR_GRACE` are absent from `rules.xml`, load as `-1`, and
    /// with a delay of `-1` every "recently broke alliance" test is already
    /// past its window. Only a mod that adds them can make it fire.
    pub in_war_grace: bool,
    /// Assassin game mode, standing in the territory of somebody who is not
    /// your target. Earns `ASSASSIN_ATTRITION` the same way peace does.
    pub assassin: bool,
    /// A Conquer the World conquest bonus that exempts this player.
    pub conquest_exempt: bool,
}

/// The whole per-unit, per-tick decision: the original's
/// `Unit::process_attrition`, minus the diplomacy bookkeeping and the throttled
/// on-screen warning, neither of which changes the damage.
///
/// The checks are in the original's order, which is observable: several of
/// them are reachable in the same tick and only the first one reached is the
/// reason.
///
/// One deliberate divergence. When the ground is unowned and the victim *does*
/// have a neutral period, the original sets that period and then keeps going,
/// reading its per-player arrays at index -1 — a real out-of-bounds read that
/// only stays harmless because `neutral_attrition` is zero in every shipped
/// scenario. Here that case returns the neutral period, which is plainly what
/// was meant.
pub fn process(
    t: &Tuning,
    kind: &UnitKind,
    victim_id: Player,
    victim: &PlayerState,
    situation: &Situation,
    owner_of: impl Fn(Player) -> PlayerState,
) -> Outcome {
    use Exempt::*;

    if situation.in_free_zone {
        return Outcome::Exempt(FreeZone);
    }

    let Some(owner_id) = situation.ground.player() else {
        // Unowned, or claimed ambiguously — the original's `who < 0` test
        // covers both.
        return if victim.neutral_attrition == 0 || kind.domain == Domain::Sea {
            Outcome::Exempt(NeutralGround)
        } else {
            Outcome::Period {
                frames: victim.neutral_attrition,
                ignores_supply: false,
            }
        };
    };

    if owner_id == victim_id {
        return Outcome::Exempt(OwnTerritory);
    }
    let owner = owner_of(owner_id);
    if !owner.active {
        return Outcome::Exempt(NoOwner);
    }
    // The original's next test is `owner == leaders[victim].who` — the
    // victim's own slot index, which is the own-territory test a second time
    // and can never fire. It is not a team test; there is no team here.
    if situation.mutual_treaty {
        return Outcome::Exempt(Treaty);
    }
    if victim.take_disabled || owner.give_disabled {
        return Outcome::Exempt(Disabled);
    }
    if kind.exempt_kind || kind.gathering_non_flat {
        return Outcome::Exempt(UnitKind);
    }
    if kind.domain == Domain::Sea {
        return Outcome::Exempt(Ship);
    }
    if victim.neutral_attrition != 0 {
        return Outcome::Exempt(NeutralOverride);
    }
    if situation.conquest_exempt {
        return Outcome::Exempt(ConquestBonus);
    }

    // A short period can be assigned outright — halved for an aircraft — and
    // the ordinary calculation still runs afterwards. Where both apply the
    // shorter wins, so peace is a floor on how slowly a border violation
    // hurts, not a replacement for the rate.
    let mut assigned = None;
    if !situation.at_war {
        if situation.in_war_grace {
            return Outcome::Exempt(WarGrace);
        }
        assigned = Some(t.peace_attrition);
    } else if situation.assassin {
        assigned = Some(t.assassin_attrition);
    }
    // Both of those paths set a flag the supply check tests first, so a bleed
    // that came from one of them cannot be sheltered.
    let ignores_supply = assigned.is_some();
    if kind.domain == Domain::Air {
        assigned = assigned.map(|p| p / 2);
    }

    // Only land units ever reach the computed period — and a supply wagon
    // only if the same flag that defeats supply is set. At war it returns
    // here with nothing; over a peaceful border it bleeds like anything else.
    if kind.domain == Domain::Land {
        if kind.supply_unit && !ignores_supply {
            return Outcome::Exempt(SupplyUnit);
        }
        if let Some(p) = period(t, susceptibility(t, kind, victim, &owner)) {
            assigned = Some(assigned.map_or(p, |a| a.min(p)));
        }
    }

    match assigned {
        Some(frames) => Outcome::Period {
            frames,
            ignores_supply,
        },
        None => Outcome::Exempt(Immune),
    }
}

/// Damage one attrition tick deals to one **figure**, in **sixteenths of a
/// hit point**, by how many figures its squad currently has.
///
/// Rise of Nations units are squads of one to four figures, and each figure
/// is its own object with its own cadence and its own health — the squad is
/// what the player selects, the figure is what bleeds. The amount is
/// `16 / size`, except that size three is special-cased to 6 rather than the 5
/// that would give, so the per-squad total stays near sixteen: near one whole
/// hit point per squad per tick. A lone figure loses one point a tick; a
/// figure in a squad of four loses one every fourth tick.
///
/// A type whose `uber_size` is exactly 1 is dealt one whole point directly in
/// the original, which is what sixteen sixteenths comes to; the two paths are
/// the same amount and [`take_damage`] treats them the same.
///
/// An earlier draft of this crate applied these as whole points — sixteen
/// times too strong. The second reading caught it.
pub const fn damage(squad_size: i32) -> i32 {
    match squad_size {
        0 => 0,
        3 => 6,
        n => 16 / n,
    }
}

/// Carries sixteenths of damage into whole hit points, the way
/// `Object::take_damage` does: the sixteenths are added to the figure's
/// fractional accumulator, every full sixteen becomes one whole point, and the
/// remainder stays in the accumulator. Nothing is lost to truncation — a squad
/// of three carries its 18/16 forward exactly.
///
/// Returns `(whole points to deduct now, new accumulator)`.
pub const fn take_damage(frac: i32, sixteenths: i32) -> (i32, i32) {
    let acc = frac + sixteenths;
    (acc / 16, acc % 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    fn owner_with(strength: i32) -> PlayerState {
        PlayerState {
            strength,
            ..PlayerState::default()
        }
    }

    #[test]
    fn no_attrition_tech_means_no_attrition() {
        assert_eq!(strength(&T, 0, &StrengthMods::default()), 0);
    }

    #[test]
    fn the_tech_chain_doubles() {
        for (steps, want) in [(1, 1), (2, 2), (3, 4), (4, 8)] {
            assert_eq!(strength(&T, steps, &StrengthMods::default()), want);
        }
    }

    #[test]
    fn strength_multipliers_truncate_in_order() {
        // One tech step is 1, and +50% of 1 truncates to 1, not 2 — but the
        // floor keeps it at 1 rather than letting it reach zero.
        let colosseum = StrengthMods {
            colosseum: true,
            ..StrengthMods::default()
        };
        assert_eq!(strength(&T, 1, &colosseum), 1);
        // Four steps is 8, and +50% is 12.
        assert_eq!(strength(&T, 4, &colosseum), 12);
        // Order is observable: Colosseum then Russian on 8 gives
        // 8 -> 12 -> 24, and both together on 1 give 1 -> 1 -> 2.
        let both = StrengthMods {
            colosseum: true,
            russian: true,
            ..StrengthMods::default()
        };
        assert_eq!(strength(&T, 4, &both), 24);
        assert_eq!(strength(&T, 1, &both), 2);
    }

    #[test]
    fn resistance_is_an_exact_rational() {
        assert_eq!(resistance(&T, &ResistanceMods::default()), Resistance::BASE);
        // Foraging tier 1 is a 25% reduction, so 100/75 = 4/3.
        let f1 = ResistanceMods {
            foraging: Some(1),
            ..ResistanceMods::default()
        };
        assert_eq!(
            resistance(&T, &f1),
            Resistance::Scaled {
                num: 256 * 100,
                den: 75
            }
        );
    }

    #[test]
    fn the_statue_of_liberty_is_total_immunity() {
        let m = ResistanceMods {
            liberty: true,
            ..ResistanceMods::default()
        };
        assert_eq!(resistance(&T, &m), Resistance::Immune);
        // And it swallows everything else, in either order.
        let m = ResistanceMods {
            liberty: true,
            foraging: Some(3),
            mongol: true,
            titanium: true,
        };
        assert_eq!(resistance(&T, &m), Resistance::Immune);
    }

    /// Resistance as `susceptibility` consumes it: the rational scaled by 256
    /// and truncated once.
    fn resolved(m: &ResistanceMods) -> i32 {
        let victim = PlayerState {
            resistance: resistance(&T, m),
            ..PlayerState::default()
        };
        susceptibility(&T, &UnitKind::default(), &victim, &owner_with(1))
    }

    #[test]
    fn every_reachable_resistance_lands_on_a_known_integer() {
        // The shipped factors are 4/3, 2 and 4 (Foraging), 2 (Mongol) and 2
        // (titanium). Only the 4/3 is not a dyadic rational, so these are the
        // only values where a truncation happens at all — and the exact
        // rational lands on the same integer the original's f32 does, since
        // 256 * 4/3 * 2^k is never within a hair of an integer boundary.
        let m = |foraging, mongol, titanium| ResistanceMods {
            foraging,
            liberty: false,
            mongol,
            titanium,
        };
        assert_eq!(resolved(&m(None, false, false)), 256);
        assert_eq!(resolved(&m(Some(1), false, false)), 341); // 1024/3
        assert_eq!(resolved(&m(Some(2), false, false)), 512);
        assert_eq!(resolved(&m(Some(3), false, false)), 1024);
        assert_eq!(resolved(&m(None, true, false)), 512);
        assert_eq!(resolved(&m(None, true, true)), 1024);
        assert_eq!(resolved(&m(Some(1), true, true)), 1365); // 4096/3
        assert_eq!(resolved(&m(Some(3), true, true)), 4096);
    }

    #[test]
    fn the_baseline_is_forty_eight_frames() {
        // The check that gives this module its confidence, arrived at from an
        // entirely independent direction: a plain unit against a player with
        // one attrition tech and nothing else. Resistance 256, strength 1, so
        // 48 * 256 / 256 = 48 frames — 3.2 seconds a tick. That is exactly
        // what the shipped ATTRITION constant's own trailing comment claims.
        let victim = PlayerState::default();
        let owner = owner_with(strength(&T, 1, &StrengthMods::default()));
        let s = susceptibility(&T, &UnitKind::default(), &victim, &owner);
        assert_eq!(s, 256);
        assert_eq!(period(&T, s), Some(48));
    }

    #[test]
    fn stronger_attrition_shortens_the_period() {
        let victim = PlayerState::default();
        let mut last = i32::MAX;
        for steps in 1..=4 {
            let owner = owner_with(strength(&T, steps, &StrengthMods::default()));
            let p = period(
                &T,
                susceptibility(&T, &UnitKind::default(), &victim, &owner),
            )
            .unwrap();
            assert!(p < last, "{steps} steps gave {p}, not shorter than {last}");
            last = p;
        }
        // Four steps is strength 8, so 48 * (256/8) / 256 = 6 frames.
        assert_eq!(last, 6);
    }

    #[test]
    fn siege_units_bleed_at_half_rate_and_militia_at_four_times() {
        let victim = PlayerState::default();
        let owner = owner_with(1);
        let siege = UnitKind {
            siege: true,
            ..UnitKind::default()
        };
        let militia = UnitKind {
            militia: true,
            ..UnitKind::default()
        };
        assert_eq!(
            period(&T, susceptibility(&T, &siege, &victim, &owner)),
            Some(96)
        );
        assert_eq!(
            period(&T, susceptibility(&T, &militia, &victim, &owner)),
            Some(12)
        );
    }

    #[test]
    fn traders_bleed_at_double_rate() {
        let victim = PlayerState::default();
        let owner = owner_with(1);
        let trader = UnitKind {
            trader: true,
            ..UnitKind::default()
        };
        assert_eq!(
            period(&T, susceptibility(&T, &trader, &victim, &owner)),
            Some(24)
        );
    }

    #[test]
    fn being_ahead_in_age_makes_your_borders_hurt_more() {
        let victim = PlayerState::default();
        let ahead = PlayerState {
            strength: 1,
            age: 4,
            ..PlayerState::default()
        };
        // 25% per age, four ages, rounded up: strength 1 becomes 2.
        let s = susceptibility(&T, &UnitKind::default(), &victim, &ahead);
        assert_eq!(s, 128);
        assert_eq!(period(&T, s), Some(24));
        // Being behind changes nothing at all.
        let behind = PlayerState {
            strength: 1,
            age: 0,
            ..PlayerState::default()
        };
        let victim_ahead = PlayerState {
            age: 4,
            ..PlayerState::default()
        };
        assert_eq!(
            susceptibility(&T, &UnitKind::default(), &victim_ahead, &behind),
            256
        );
    }

    #[test]
    fn foraging_one_makes_idle_units_immune_but_not_moving_ones() {
        let victim = PlayerState {
            foraging_1: true,
            resistance: resistance(
                &T,
                &ResistanceMods {
                    foraging: Some(1),
                    ..ResistanceMods::default()
                },
            ),
            ..PlayerState::default()
        };
        let owner = owner_with(1);
        let idle = UnitKind {
            idle: true,
            ..UnitKind::default()
        };
        assert_eq!(susceptibility(&T, &idle, &victim, &owner), 0);
        assert_eq!(period(&T, 0), None);
        // Moving, the same unit takes attrition at the 4/3 resisted rate.
        assert_eq!(
            period(
                &T,
                susceptibility(&T, &UnitKind::default(), &victim, &owner)
            ),
            Some(63)
        );
    }

    #[test]
    fn the_worst_the_shipped_data_can_do_is_one_frame() {
        // Everything at once: four tech steps, the Colosseum, the Russians, a
        // conquest bonus and the Kremlin. 8 -> 12 -> 24 -> 36 -> 72, which is
        // the largest strength the shipped constants can reach.
        let all = StrengthMods {
            colosseum: true,
            russian: true,
            conquest_bonus: true,
            kremlin: true,
        };
        assert_eq!(strength(&T, 4, &all), 72);
        let victim = PlayerState::default();
        let s = susceptibility(&T, &UnitKind::default(), &victim, &owner_with(72));
        // 256 / 72 truncates to 3, and 48 * 3 / 256 truncates to 0, so the
        // floor of one frame is what actually sets the fastest attrition in
        // the game: fifteen ticks a second, 240 damage a second to a lone
        // figure.
        assert_eq!(s, 3);
        assert_eq!(period(&T, s), Some(1));
    }

    #[test]
    fn an_unreachably_strong_owner_would_wrap_round_to_harmless() {
        // A quirk worth knowing about before somebody tunes past it. Zero is
        // the original's "immune" sentinel, and susceptibility is
        // `resistance / strength` with truncation — so a strength above 256
        // divides to zero and stops hurting anyone at all. The shipped data
        // tops out at 72, so this is unreachable in play, but a mod that
        // raised ATTRITION_IMPROVED would fall straight into it.
        let victim = PlayerState::default();
        let s = susceptibility(&T, &UnitKind::default(), &victim, &owner_with(257));
        assert_eq!(s, 0);
        assert_eq!(period(&T, s), None);
    }

    #[test]
    fn damage_by_squad_size() {
        // Sixteenths of a hit point per figure.
        assert_eq!(damage(1), 16);
        assert_eq!(damage(2), 8);
        // The special case: 16/3 would be 5, and 6 keeps the squad total near
        // sixteen.
        assert_eq!(damage(3), 6);
        assert_eq!(damage(4), 4);
        // Total damage per squad stays close to one whole point a tick, which
        // is the point.
        for n in 1..=4 {
            let total = damage(n) * n;
            assert!((16..=18).contains(&total), "size {n} totals {total}");
        }
    }

    #[test]
    fn sixteenths_accumulate_without_loss() {
        // A lone figure: one whole point every tick, nothing carried.
        assert_eq!(take_damage(0, damage(1)), (1, 0));
        // A figure in a squad of four: nothing for three ticks, then one.
        let mut frac = 0;
        let mut lost = Vec::new();
        for _ in 0..8 {
            let (l, f) = take_damage(frac, damage(4));
            frac = f;
            lost.push(l);
        }
        assert_eq!(lost, [0, 0, 0, 1, 0, 0, 0, 1]);
        // A squad of three carries its 18/16 forward exactly: 6, 12, 18 -> 1
        // carry 2, then 8, 14, 20 -> 1 carry 4, ... eight ticks of 6 is 48
        // sixteenths, exactly three points, nothing left over.
        let mut frac = 0;
        let mut total = 0;
        for _ in 0..8 {
            let (l, f) = take_damage(frac, damage(3));
            frac = f;
            total += l;
        }
        assert_eq!((total, frac), (3, 0));
    }

    fn at_war_with(who: Owner) -> Situation {
        Situation {
            ground: who,
            in_free_zone: false,
            mutual_treaty: false,
            at_war: true,
            in_war_grace: false,
            assassin: false,
            conquest_exempt: false,
        }
    }

    #[test]
    fn own_territory_is_safe() {
        let me = PlayerState::default();
        let out = process(
            &T,
            &UnitKind::default(),
            0,
            &me,
            &at_war_with(Owner::Player(0)),
            |_| owner_with(8),
        );
        assert_eq!(out, Outcome::Exempt(Exempt::OwnTerritory));
    }

    #[test]
    fn unowned_ground_is_safe_unless_a_scenario_says_otherwise() {
        let me = PlayerState::default();
        let out = process(
            &T,
            &UnitKind::default(),
            0,
            &me,
            &at_war_with(Owner::None),
            |_| owner_with(8),
        );
        assert_eq!(out, Outcome::Exempt(Exempt::NeutralGround));

        let scripted = PlayerState {
            neutral_attrition: 30,
            ..PlayerState::default()
        };
        let out = process(
            &T,
            &UnitKind::default(),
            0,
            &scripted,
            &at_war_with(Owner::None),
            |_| owner_with(8),
        );
        assert_eq!(out.frames(), Some(30));
    }

    #[test]
    fn an_ambiguous_claim_counts_as_unowned() {
        let me = PlayerState::default();
        let out = process(
            &T,
            &UnitKind::default(),
            0,
            &me,
            &at_war_with(Owner::Ambiguous),
            |_| owner_with(8),
        );
        assert_eq!(out, Outcome::Exempt(Exempt::NeutralGround));
    }

    #[test]
    fn peace_is_a_floor_on_the_period_not_a_replacement() {
        let me = PlayerState::default();
        let mut s = at_war_with(Owner::Player(1));
        s.at_war = false;
        // Against a weak owner the computed period is 48 frames, so the
        // 8-frame peace period is what bites: crossing a peaceful border hurts
        // far more than being in a war zone.
        let out = process(&T, &UnitKind::default(), 0, &me, &s, |_| owner_with(1));
        assert_eq!(
            out,
            Outcome::Period {
                frames: T.peace_attrition,
                // And a supply wagon does not help you here.
                ignores_supply: true,
            }
        );

        // Against a strong owner the computed period is shorter still, and it
        // is the one that wins — the two are combined, not substituted. The
        // supply bypass came from the peace path and survives the swap.
        let out = process(&T, &UnitKind::default(), 0, &me, &s, |_| owner_with(16));
        assert_eq!(
            out,
            Outcome::Period {
                frames: 3,
                ignores_supply: true
            }
        );

        // An aircraft gets the peace period halved, and never reaches the
        // computed one at all.
        let aircraft = UnitKind {
            domain: Domain::Air,
            ..UnitKind::default()
        };
        let out = process(&T, &aircraft, 0, &me, &s, |_| owner_with(16));
        assert_eq!(out.frames(), Some(T.peace_attrition / 2));
    }

    #[test]
    fn ships_are_immune_and_aircraft_only_bleed_over_peaceful_borders() {
        let me = PlayerState::default();
        let at_war = at_war_with(Owner::Player(1));
        let ship = UnitKind {
            domain: Domain::Sea,
            ..UnitKind::default()
        };
        let aircraft = UnitKind {
            domain: Domain::Air,
            ..UnitKind::default()
        };
        assert_eq!(
            process(&T, &ship, 0, &me, &at_war, |_| owner_with(16)),
            Outcome::Exempt(Exempt::Ship)
        );
        // In a war zone with no special period to take, an aircraft takes
        // nothing: only land units reach the computed period.
        assert_eq!(
            process(&T, &aircraft, 0, &me, &at_war, |_| owner_with(16)),
            Outcome::Exempt(Exempt::Immune)
        );
    }

    #[test]
    fn a_supply_wagon_is_safe_at_war_and_bleeds_over_a_peaceful_border() {
        // The supply-unit exemption sits in the computed-period path and is
        // lifted by the same flag that defeats supply. So at war a wagon has
        // no period at all, and over a peaceful border it takes the peace
        // period — and the computed one, smaller wins — like anything else.
        let me = PlayerState::default();
        let wagon = UnitKind {
            supply_unit: true,
            ..UnitKind::default()
        };
        let at_war = at_war_with(Owner::Player(1));
        let mut at_peace = at_war;
        at_peace.at_war = false;
        assert_eq!(
            process(&T, &wagon, 0, &me, &at_war, |_| owner_with(1)),
            Outcome::Exempt(Exempt::SupplyUnit)
        );
        assert_eq!(
            process(&T, &wagon, 0, &me, &at_peace, |_| owner_with(1)),
            Outcome::Period {
                frames: T.peace_attrition,
                ignores_supply: true
            }
        );
        assert_eq!(
            process(&T, &wagon, 0, &me, &at_peace, |_| owner_with(16)),
            Outcome::Period {
                frames: 3,
                ignores_supply: true
            }
        );
    }

    #[test]
    fn the_kind_exemption_is_the_originals_nesting() {
        // Workers, merchants, heroes and supply units skip the inner tests
        // and are not exempt — even with zero attack, even flagged special.
        let zero_attack = KindFacts::default();
        assert!(kind_exempt(&zero_attack));
        for skip in [
            KindFacts {
                worker: true,
                ..zero_attack
            },
            KindFacts {
                merchant: true,
                ..zero_attack
            },
            KindFacts {
                hero: true,
                special: true,
                ..zero_attack
            },
            KindFacts {
                supply: true,
                caravan: true,
                ..zero_attack
            },
        ] {
            assert!(!kind_exempt(&skip), "{skip:?} should not be exempt");
        }
        // Everything else: exempt on zero attack, special, spy or caravan.
        let fighter = KindFacts {
            attack: 10,
            ..KindFacts::default()
        };
        assert!(!kind_exempt(&fighter));
        assert!(kind_exempt(&KindFacts {
            spy: true,
            ..fighter
        }));
        assert!(kind_exempt(&KindFacts {
            special: true,
            ..fighter
        }));
        assert!(kind_exempt(&KindFacts {
            caravan: true,
            ..fighter
        }));
    }

    #[test]
    fn a_war_zone_bleed_can_be_sheltered_but_a_border_violation_cannot() {
        let me = PlayerState::default();
        let at_war = at_war_with(Owner::Player(1));
        let mut at_peace = at_war;
        at_peace.at_war = false;

        let war = process(&T, &UnitKind::default(), 0, &me, &at_war, |_| owner_with(1));
        let peace = process(&T, &UnitKind::default(), 0, &me, &at_peace, |_| {
            owner_with(1)
        });
        assert_eq!(
            war,
            Outcome::Period {
                frames: 48,
                ignores_supply: false
            }
        );
        assert!(matches!(
            peace,
            Outcome::Period {
                ignores_supply: true,
                ..
            }
        ));
    }

    #[test]
    fn militia_and_supply_units_are_never_sheltered() {
        assert!(UnitKind::default().shelterable());
        assert!(
            !UnitKind {
                militia: true,
                ..UnitKind::default()
            }
            .shelterable()
        );
        assert!(
            !UnitKind {
                supply_unit: true,
                ..UnitKind::default()
            }
            .shelterable()
        );
    }

    #[test]
    fn the_war_grace_window_suspends_attrition_entirely() {
        let me = PlayerState::default();
        let mut s = at_war_with(Owner::Player(1));
        s.at_war = false;
        s.in_war_grace = true;
        let out = process(&T, &UnitKind::default(), 0, &me, &s, |_| owner_with(16));
        assert_eq!(out, Outcome::Exempt(Exempt::WarGrace));
    }

    #[test]
    fn the_exemption_order_is_the_originals() {
        // A unit that qualifies for several exemptions at once reports the
        // first one the original would reach, not the most specific.
        let me = PlayerState {
            take_disabled: true,
            ..PlayerState::default()
        };
        let kind = UnitKind {
            exempt_kind: true,
            domain: Domain::Sea,
            ..UnitKind::default()
        };
        let mut s = at_war_with(Owner::Player(1));
        s.in_free_zone = true;
        assert_eq!(
            process(&T, &kind, 0, &me, &s, |_| owner_with(1)),
            Outcome::Exempt(Exempt::FreeZone)
        );
        s.in_free_zone = false;
        assert_eq!(
            process(&T, &kind, 0, &me, &s, |_| owner_with(1)),
            Outcome::Exempt(Exempt::Disabled)
        );
    }

    #[test]
    fn a_full_hostile_tick_produces_a_period() {
        let victim = PlayerState::default();
        let out = process(
            &T,
            &UnitKind::default(),
            0,
            &victim,
            &at_war_with(Owner::Player(1)),
            |_| owner_with(strength(&T, 2, &StrengthMods::default())),
        );
        // Strength 2: 48 * (256/2) / 256 = 24 frames.
        assert_eq!(out.frames(), Some(24));
    }
}
