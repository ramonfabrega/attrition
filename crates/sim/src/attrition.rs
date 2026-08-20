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
    /// The unit type's attrition mode. See [`Mode`].
    pub mode: Mode,
    /// Workers and merchants are subject to attrition; heroes, supply units,
    /// spies, "special" units and caravans are not.
    pub exempt_kind: bool,
    /// A worker actively gathering at a site the scenario flagged exempt.
    pub gathering_at_exempt_site: bool,
    /// Whether the unit is standing still, for the Foraging tier 1 exemption.
    pub idle: bool,
}

/// A unit type's attrition mode, the original's three-way switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The ordinary case, and the only one that gets a computed period.
    #[default]
    Normal,
    /// Outright immune.
    Immune,
    /// Takes the special periods at half length, and never a computed one.
    /// The halving inside [`susceptibility`] is therefore unreachable in the
    /// original's own tick — it is kept because the function is called from
    /// elsewhere.
    Special,
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
    /// Team, for the same-team exemption. Players on no team are their own.
    pub team: u8,
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
            team: 0,
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
    if kind.mode == Mode::Special {
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
    /// The territory belongs to the unit's own team.
    SameTeam,
    /// Both sides hold a mutual treaty.
    Treaty,
    /// A scenario disabled taking or giving attrition.
    Disabled,
    /// The unit is a kind attrition does not apply to.
    UnitKind,
    /// The unit's type is outright immune.
    ImmuneType,
    /// The victim has a neutral-ground period set, which in the original
    /// replaces territorial attrition rather than adding to it.
    NeutralOverride,
    /// A Conquer the World conquest bonus.
    ConquestBonus,
    /// Inside the grace window after an ally became an enemy.
    WarGrace,
    /// The unit is subject in principle but resists completely.
    Immune,
}

/// What one tick of attrition does to one unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing, for this reason.
    Exempt(Exempt),
    /// The unit is subject to attrition on this period, in frames.
    Period(i32),
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
    /// Inside the grace window after an ally turned into an enemy, during
    /// which the border does not bite at all.
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
        return if victim.neutral_attrition == 0 || kind.mode == Mode::Immune {
            Outcome::Exempt(NeutralGround)
        } else {
            Outcome::Period(victim.neutral_attrition)
        };
    };

    if owner_id == victim_id {
        return Outcome::Exempt(OwnTerritory);
    }
    let owner = owner_of(owner_id);
    if !owner.active {
        return Outcome::Exempt(NoOwner);
    }
    if owner_id == victim.team {
        return Outcome::Exempt(SameTeam);
    }
    if situation.mutual_treaty {
        return Outcome::Exempt(Treaty);
    }
    if victim.take_disabled || owner.give_disabled {
        return Outcome::Exempt(Disabled);
    }
    if kind.exempt_kind || kind.gathering_at_exempt_site {
        return Outcome::Exempt(UnitKind);
    }
    if kind.mode == Mode::Immune {
        return Outcome::Exempt(ImmuneType);
    }
    if victim.neutral_attrition != 0 {
        return Outcome::Exempt(NeutralOverride);
    }
    if situation.conquest_exempt {
        return Outcome::Exempt(ConquestBonus);
    }

    // A short period can be assigned outright — halved for a `Special`-mode
    // unit — and the ordinary calculation still runs afterwards. Where both
    // apply the shorter wins, so peace is a floor on how slowly a border
    // violation hurts, not a replacement for the rate.
    let mut assigned = None;
    if !situation.at_war {
        if situation.in_war_grace {
            return Outcome::Exempt(WarGrace);
        }
        assigned = Some(t.peace_attrition);
    } else if situation.assassin {
        assigned = Some(t.assassin_attrition);
    }
    if kind.mode == Mode::Special {
        assigned = assigned.map(|p| p / 2);
    }

    // Only `Normal`-mode units ever reach the computed period.
    if kind.mode == Mode::Normal
        && let Some(p) = period(t, susceptibility(t, kind, victim, &owner))
    {
        assigned = Some(assigned.map_or(p, |a| a.min(p)));
    }

    match assigned {
        Some(p) => Outcome::Period(p),
        None => Outcome::Exempt(Immune),
    }
}

/// Damage one attrition tick deals, by squad size.
///
/// Rise of Nations units are squads of one to four figures. The damage is
/// `16 / size`, except that size three is special-cased to 6 rather than the 5
/// that would give, so the per-squad total stays near sixteen.
pub const fn damage(squad_size: i32) -> i32 {
    match squad_size {
        0 => 0,
        3 => 6,
        n => 16 / n,
    }
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
        assert_eq!(damage(1), 16);
        assert_eq!(damage(2), 8);
        // The special case: 16/3 would be 5, and 6 keeps the squad total near
        // sixteen.
        assert_eq!(damage(3), 6);
        assert_eq!(damage(4), 4);
        // Total damage per squad stays close to constant, which is the point.
        for n in 1..=4 {
            let total = damage(n) * n;
            assert!((16..=18).contains(&total), "size {n} totals {total}");
        }
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
        assert_eq!(out, Outcome::Period(30));
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
        assert_eq!(out, Outcome::Period(T.peace_attrition));

        // Against a strong owner the computed period is shorter still, and it
        // is the one that wins — the two are combined, not substituted.
        let out = process(&T, &UnitKind::default(), 0, &me, &s, |_| owner_with(16));
        assert_eq!(out, Outcome::Period(3));

        // A Special-mode unit gets the peace period halved, and never reaches
        // the computed one at all.
        let special = UnitKind {
            mode: Mode::Special,
            ..UnitKind::default()
        };
        let out = process(&T, &special, 0, &me, &s, |_| owner_with(16));
        assert_eq!(out, Outcome::Period(T.peace_attrition / 2));
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
            mode: Mode::Immune,
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
        assert_eq!(out, Outcome::Period(24));
    }
}
