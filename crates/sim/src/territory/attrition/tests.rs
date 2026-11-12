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

/// Item 4154: the floor fires on a base of 0, so any applying multiplier
/// lifts a leader with no step to 1 (run647's Russians, `att 1` at the
/// start); with no multiplier it stays 0 (run647's Nubians).
#[test]
fn a_multiplier_floors_no_tech_at_one() {
    for m in [
        StrengthMods {
            russian: true,
            ..StrengthMods::default()
        },
        StrengthMods {
            colosseum: true,
            ..StrengthMods::default()
        },
        StrengthMods {
            kremlin: true,
            ..StrengthMods::default()
        },
    ] {
        assert_eq!(strength(&T, 0, &m), 1, "{m:?}");
    }
    // The floor is per step, so the arms after the first multiply the 1:
    // 0 -> 1 (Colosseum, floored) -> 2 (Russian) -> 3 (CtW) -> 6 (Kremlin).
    let all = StrengthMods {
        colosseum: true,
        russian: true,
        conquest_bonus: true,
        kremlin: true,
    };
    assert_eq!(strength(&T, 0, &all), 6);
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
