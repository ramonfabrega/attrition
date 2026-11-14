//! The deal record, `action_respond`'s commit and exchange, the scale and
//! `is_tribute_period` on a two-seat fixture (`docs/LEADERS.md` §8.2).
//! run1575's taunt gift is the diff (`rondata::diff::diplomacy_take`).

use crate::economy::Resource;
use crate::{Sim, Tuning, World};

const FOOD: usize = Resource::Food.index();
const WEALTH: usize = Resource::Wealth.index();

/// Two computers at peace, each with 5000 of every good.
fn two() -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    for n in &mut sim.cities.nation {
        n.human = false;
    }
    for l in &mut sim.economy.ledgers {
        l.bucket = [5_000; 6];
    }
    sim.frame = 500;
    sim
}

/// `scale_tribute`'s three roundings at 58 %, the whole gift at 100 % and
/// over, and the floor of 1 %.
#[test]
fn scale_tribute_rounds_by_size() {
    let mut sim = two();
    sim.tech[0].epoch[crate::tech::Line::Commerce.index()] = 1;
    let got: Vec<i32> = [10, 19, 20, 99, 100, 3_290]
        .iter()
        .map(|&x| sim.scale_tribute(0, x))
        .collect();
    assert_eq!(got, vec![5, 11, 12, 57, 58, 1_909]);
    sim.tech[0].epoch[crate::tech::Line::Commerce.index()] = 7;
    assert_eq!(sim.scale_tribute(0, 333), 333, "51 + 49 = 100");
    sim.tables.tuning.base_tribute = -60;
    sim.tech[0].epoch[crate::tech::Line::Commerce.index()] = 0;
    assert_eq!(sim.scale_tribute(0, 333), 4, "(333 + 99) / 100 at 1 %");
}

/// A deal that asks something of the partner waits for it: the commit
/// moves the give into `tributes` and nothing is exchanged until the
/// partner accepts. Withdrawing refunds the bucket, not the escrow.
#[test]
fn a_two_way_deal_waits_for_the_partner() {
    let mut sim = two();
    sim.economy.ledgers[0].escrow[FOOD] = 300;
    sim.action_offer(0, 1, Resource::Food, 1_000);
    sim.action_offer(1, 0, Resource::Wealth, 200);
    sim.action_offer(0, 1, Resource::Wealth, -200);
    assert_eq!(
        sim.leaders.tribute[1].dip[0].offers[WEALTH], 400,
        "both asks"
    );
    sim.action_respond(0, 1, 1);
    assert_eq!(sim.economy.ledgers[0].bucket[FOOD], 4_000);
    assert_eq!(sim.economy.ledgers[0].escrow[FOOD], 0);
    assert_eq!(sim.leaders.tribute[0].tributes[FOOD], 1_000);
    assert_eq!(sim.leaders.tribute[0].dip[1].agree, 1);
    assert_eq!(sim.leaders.agendas[1][0], 4);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 5_000, "not exchanged");
    // Withdrawn: the bucket back, the escrow not.
    sim.action_respond(0, 1, 0);
    assert_eq!(sim.economy.ledgers[0].bucket[FOOD], 5_000);
    assert_eq!(sim.economy.ledgers[0].escrow[FOOD], 0);
    assert_eq!(sim.leaders.tribute[0].tributes[FOOD], 0);
    // Committed again, and accepted by the partner: both gifts scaled at
    // 51 % and stamped on both sides.
    sim.action_respond(0, 1, 1);
    sim.action_respond(1, 0, 1);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 5_000 + 510);
    assert_eq!(sim.economy.ledgers[0].bucket[WEALTH], 5_000 + 204);
    assert_eq!(sim.economy.ledgers[1].bucket[WEALTH], 5_000 - 400);
    let (a, b) = (&sim.leaders.tribute[0], &sim.leaders.tribute[1]);
    assert_eq!((a.tribute_sent, a.tribute_received), (1_000, 204));
    assert_eq!((b.tribute_sent, b.tribute_received), (400, 510));
    assert_eq!((a.gift_stamp[1], b.gift_stamp[0]), (500, 500));
    assert_eq!((a.tributes, b.tributes), ([0; 6], [0; 6]));
}

/// The purse test refuses the whole answer: nothing moves.
#[test]
fn a_short_purse_refuses_the_commit() {
    let mut sim = two();
    sim.action_offer(0, 1, Resource::Food, 4_000);
    sim.economy.ledgers[0].bucket[FOOD] = 3_999;
    sim.action_respond(0, 1, 1);
    assert_eq!(sim.leaders.tribute[0].dip[1].agree, 0);
    assert_eq!(sim.economy.ledgers[0].bucket[FOOD], 3_999);
    assert_eq!(sim.leaders.agendas[1][0], 0);
}

/// An offer the bucket cannot cover is refused at the offer.
#[test]
fn an_uncovered_offer_is_refused() {
    let mut sim = two();
    sim.action_offer(0, 1, Resource::Food, 5_001);
    assert_eq!(sim.leaders.tribute[0].dip[1].offers[FOOD], 0);
    assert_eq!(sim.leaders.tribute[0].dip[1].any_offer, 0);
}

/// A computer receiver stamps tribute only for a gift of at least
/// `100 · blacken`, under 3000 of the good after the credit, and of wealth
/// or a good its AI wants; the giver's `gift_stamp` regardless.
#[test]
fn a_computer_counts_tribute_by_need() {
    let mut sim = two();
    sim.economy.ledgers[1].bucket[FOOD] = 2_000;
    sim.action_offer(0, 1, Resource::Food, 300);
    sim.action_respond(0, 1, 1);
    assert_eq!(sim.leaders.tribute[0].gift_stamp[1], 500);
    assert_eq!(
        sim.leaders.tribute[1].tribute_stamp[0], 0,
        "food not wanted"
    );
    sim.ai.ai[1].econ[FOOD] = 4;
    sim.frame = 600;
    sim.action_offer(0, 1, Resource::Food, 300);
    sim.action_respond(0, 1, 1);
    assert_eq!(sim.leaders.tribute[1].tribute_stamp[0], 600);
    sim.leaders.declare[0].blacken = 4;
    sim.frame = 700;
    sim.action_offer(0, 1, Resource::Food, 300);
    sim.action_respond(0, 1, 1);
    assert_eq!(
        sim.leaders.tribute[1].tribute_stamp[0], 600,
        "under 100 · blacken"
    );
    // Wealth counts without the AI's want, under 3000 after the credit
    // (`+153`, the 51 % of 300).
    sim.leaders.declare[0].blacken = 0;
    sim.economy.ledgers[1].bucket[WEALTH] = 3_000 - 153;
    sim.frame = 800;
    sim.action_offer(0, 1, Resource::Wealth, 300);
    sim.action_respond(0, 1, 1);
    assert_eq!(
        sim.leaders.tribute[1].tribute_stamp[0], 600,
        "3000 after credit"
    );
    sim.economy.ledgers[1].bucket[WEALTH] = 3_000 - 154;
    sim.frame = 900;
    sim.action_offer(0, 1, Resource::Wealth, 300);
    sim.action_respond(0, 1, 1);
    assert_eq!(
        sim.leaders.tribute[1].tribute_stamp[0], 900,
        "2999 after credit"
    );
}

/// `is_tribute_period`: a stamp under 3000 frames old answers yes, a zero
/// no, an older one lets the walk go on to the next qualifying slot.
#[test]
fn the_tribute_period_is_three_thousand_frames() {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 3);
    sim.set_diplo(1, 2, 2);
    sim.frame = 4_000;
    assert!(!sim.is_tribute_period(0, 1));
    sim.leaders.tribute[0].tribute_stamp[1] = 1_001;
    assert!(sim.is_tribute_period(0, 1));
    sim.leaders.tribute[0].tribute_stamp[1] = 1_000;
    assert!(!sim.is_tribute_period(0, 1), "stale, and slot 2's is 0");
    sim.leaders.tribute[0].tribute_stamp[2] = 3_000;
    assert!(sim.is_tribute_period(0, 1), "the ally's fresh stamp");
    sim.leaders.tribute[0].tribute_stamp[1] = 0;
    assert!(
        !sim.is_tribute_period(0, 1),
        "the first qualifying slot's 0"
    );
    sim.frame = 0;
    sim.leaders.tribute[0].tribute_stamp[1] = 0;
    assert!(!sim.is_tribute_period(0, 1));
}
