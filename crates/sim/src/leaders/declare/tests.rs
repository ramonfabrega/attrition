//! `Leader::action_declare`, `pay_dow` and `afford_dow` on a two-seat
//! fixture (`docs/LEADERS.md` §7.2). run1575 and run1078 are the diffs
//! (`rondata::diff::diplomacy_take`); these pin the arms no capture holds.

use super::{CANCEL_ALLIANCE, DECLARE_WAR};
use crate::economy::Resource;
use crate::orders::SpellType;
use crate::{Sim, Tuning, World};

const WEALTH: usize = Resource::Wealth.index();

/// Two seats in the Information age (`ages` 7, so `cost_factor_dow`'s
/// `ages + 1` is 8), each declaration craft `COST2 8g` as shipped, and
/// `purse` wealth for seat 0; the pair at `level`.
fn two(level: i32, purse: i32) -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    let mut rows = vec![SpellType::default(); CANCEL_ALLIANCE + 8 - crate::tech::SPELL_BASE];
    for r in &mut rows[DECLARE_WAR - crate::tech::SPELL_BASE..] {
        r.cost2[WEALTH] = 8;
    }
    sim.orders.spells = rows;
    for p in &mut sim.tech {
        p.ages = 7;
    }
    sim.economy.ledgers[0].bucket[WEALTH] = purse;
    sim.economy.ledgers[0].escrow[WEALTH] = 100;
    sim.frame = 986;
    sim.set_diplo(0, 1, level);
    sim
}

fn leaves(sim: &Sim) -> (i32, i32, i64, i32) {
    let d = &sim.leaders.declare[0];
    (d.dow[1], d.blacken, d.broke_alliance[1], sim.diplos(0, 1))
}

/// run1575's two Declares, as a fixture: on the ally (level 1, the
/// Cancel Alliance craft at `1 × 8 × 80 = 640`), counted in block B and
/// stamped; then at peace (level 0, Declare War at `(0 + 1) × 8 × 80`),
/// counted in block A, and `blacken` counts the repeat. The escrow is
/// floored at zero; the target's own record is not written.
#[test]
fn run1575_s_two_declares_count_stamp_and_pay() {
    let mut sim = two(2, 120_021);
    sim.action_declare(0, 1, 1, false, false);
    assert_eq!(leaves(&sim), (1, 0, 986, 1));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 119_381);
    assert_eq!(sim.economy.ledgers[0].escrow[WEALTH], 0);
    assert_eq!(sim.leaders.declare[1], Default::default(), "no mirror");
    sim.frame = 1001;
    sim.action_declare(0, 1, 0, false, false);
    assert_eq!(leaves(&sim), (2, 1, 986, 0));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 118_741);
}

/// A level-0 ask on an ally is turned into level 1 before anything else:
/// the alliance is cancelled, not war declared.
#[test]
fn a_war_on_an_ally_is_asked_as_peace() {
    let mut sim = two(2, 1000);
    sim.action_declare(0, 1, 0, false, false);
    assert_eq!(leaves(&sim), (1, 0, 986, 1));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 360);
}

/// `quiet` (every computer caller): no price and no count, but an
/// alliance broken is still stamped.
#[test]
fn a_quiet_declaration_stamps_and_neither_pays_nor_counts() {
    let mut sim = two(2, 1000);
    sim.action_declare(0, 1, 1, true, false);
    assert_eq!(leaves(&sim), (0, 0, 986, 1));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 1000);
    let mut sim = two(1, 1000);
    sim.action_declare(0, 1, 0, true, false);
    assert_eq!(leaves(&sim), (0, 0, 0, 0), "no alliance, no stamp");
}

/// Five or more of the target's units in my land: the price drops ten a
/// unit and the declaration is not counted; four still counts.
#[test]
fn invaders_cheapen_the_war_and_five_justify_it() {
    for (invaders, counted, price) in [(4, 1, 600), (5, 0, 590)] {
        let mut sim = two(1, 1000);
        sim.ai.ai[0].census.invaders = vec![0, invaders];
        sim.action_declare(0, 1, 0, false, false);
        assert_eq!(leaves(&sim), (counted, 0, 0, 0), "{invaders}");
        assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 1000 - price);
    }
}

/// A purse a coin short refuses, and an enemy already is left alone.
#[test]
fn a_short_purse_and_an_enemy_change_nothing() {
    let mut sim = two(2, 639);
    sim.action_declare(0, 1, 1, false, false);
    assert_eq!(leaves(&sim), (0, 0, 0, 2));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 639);
    let mut sim = two(0, 1000);
    sim.action_declare(0, 1, 0, false, false);
    assert_eq!(leaves(&sim), (0, 0, 0, 0));
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 1000);
}

/// Rush rules above every leader's age (8, untimed): an unforced war is refused, a forced one
/// is asked again as peace, which a pair at peace already is.
#[test]
fn rush_rules_refuse_a_war_or_force_it_to_peace() {
    for force in [false, true] {
        let mut sim = two(1, 1000);
        sim.ai.lobby.rush_rules = 8;
        sim.action_declare(0, 1, 0, false, force);
        assert_eq!(leaves(&sim), (0, 0, 0, 1), "force {force}");
        // The forced ask pays as Cancel Alliance on a non-ally:
        // `cost_factor_dow` is `(0 + 0) × 8`, so nothing.
        assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 1000, "force {force}");
    }
}
