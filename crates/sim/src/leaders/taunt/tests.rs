//! The taunt ring and `process_taunt`'s goods arm on a three-seat fixture
//! (`docs/LEADERS.md` §9.2). run1575 is the diff
//! (`rondata::diff::diplomacy_take`); these pin the arms no capture holds.

use super::TAUNT_HELP;
use crate::economy::Resource;
use crate::{Sim, Tuning, World};

const FOOD: usize = Resource::Food.index();

/// run1575's seats: the human 0 allied with the French 1, the Russians 2
/// at peace with both; the French at Commerce 1 (`scale_tribute`'s 58 %),
/// holding `food`; team style 0.
fn seats(food: i32) -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 3);
    for (k, n) in sim.cities.nation.iter_mut().enumerate() {
        n.human = k == 0;
    }
    sim.set_diplo(0, 1, 2);
    sim.tech[1].epoch[crate::tech::Line::Commerce.index()] = 1;
    sim.economy.ledgers[0].bucket[FOOD] = 120_085;
    sim.economy.ledgers[1].bucket[FOOD] = food;
    sim.economy.ledgers[1].escrow[FOOD] = 40;
    sim.ai.lobby.team_style = 0;
    sim
}

/// The chat applied on 906 is due on 966, and the drain there gives a
/// third of the French food: run1575's block 967 and its packet at 1090.
#[test]
fn run1575_s_food_taunt_is_a_third_of_the_stock_given() {
    let mut sim = seats(9_870);
    sim.frame = 906;
    sim.receive_taunt(1, 1, 0);
    let t = &sim.leaders.taunts[1];
    assert_eq!(
        (
            t.incoming_taunt[0],
            t.incoming_taunt_who[0],
            t.incoming_taunt_frame[0],
            t.incoming
        ),
        (1, 0, 966, 1)
    );
    sim.frame = 965;
    sim.drain_taunts(1);
    assert_eq!(sim.leaders.taunts[1].last_taunt[0], 0, "not due yet");
    sim.frame = 966;
    sim.drain_taunts(1);
    let t = &sim.leaders.taunts[1];
    assert_eq!((t.last_taunt[0], t.taunt_frame[0]), (1, 966));
    let (fr, hu) = (&sim.leaders.tribute[1], &sim.leaders.tribute[0]);
    assert_eq!(fr.gift_stamp[0], 966);
    assert_eq!(hu.tribute_stamp[1], 966);
    assert_eq!((fr.tribute_sent, hu.tribute_received), (3_290, 1_909));
    assert_eq!(fr.tributes, [0; 6], "committed and exchanged in one call");
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 6_580);
    assert_eq!(sim.economy.ledgers[1].escrow[FOOD], 0);
    assert_eq!(sim.economy.ledgers[0].bucket[FOOD], 121_994);
    assert_eq!(
        sim.leaders.agendas[0][1], 4,
        "the commit forces the human's pass"
    );
    assert_eq!(fr.dip[0], Default::default(), "both records cleared");
    // The ring is not cleared, and fires only on its frame.
    sim.frame = 967;
    sim.drain_taunts(1);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 6_580);
}

/// At team style 0 a second ask inside 4500 frames of the gift is refused;
/// the head still stores. At a locked style the window is not tested and
/// `process_taunt` stamps nothing itself, but the exchange's
/// `consider_tribute` does.
#[test]
fn the_gift_window_binds_unlocked_teams_only() {
    let mut sim = seats(9_870);
    sim.frame = 966;
    sim.process_taunt(1, 1, 0);
    sim.frame = 966 + 4_499;
    sim.process_taunt(1, 1, 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 6_580, "refused");
    assert_eq!(sim.leaders.taunts[1].taunt_frame[0], 5_465);
    sim.frame = 966 + 4_500;
    sim.process_taunt(1, 1, 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 6_580 - 6_580 / 3);
    let mut sim = seats(9_870);
    sim.ai.lobby.team_style = 1;
    sim.leaders.tribute[1].gift_stamp[0] = 900;
    sim.frame = 966;
    sim.process_taunt(1, 1, 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 6_580, "not refused");
    assert_eq!(
        sim.leaders.tribute[1].gift_stamp[0], 966,
        "consider_tribute"
    );
}

/// A stock of 149 gives nothing; a good either side lacks gives nothing; a
/// request outside 1..5 only stores.
#[test]
fn the_goods_arm_needs_stock_and_the_good() {
    let mut sim = seats(149);
    sim.frame = 966;
    sim.process_taunt(1, 1, 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 149);
    assert_eq!(sim.leaders.taunts[1].last_taunt[0], 1);
    let mut sim = seats(9_870);
    sim.economy.holdings[0].available[FOOD] = false;
    sim.process_taunt(1, 1, 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 9_870);
    let mut sim = seats(9_870);
    sim.process_taunt(1, 17, 0);
    assert_eq!(sim.leaders.taunts[1].last_taunt[0], 17, "stored first");
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 9_870);
}

/// The ring refuses a non-positive request, a human receiver, a
/// non-ally and the sender itself; `HELP` is due on its own frame; the
/// cursor wraps at eight and overwrites.
#[test]
fn the_ring_s_exemptions_and_cursor() {
    let mut sim = seats(9_870);
    sim.frame = 100;
    sim.receive_taunt(1, 0, 0);
    sim.receive_taunt(1, -3, 0);
    sim.receive_taunt(0, 1, 1);
    sim.receive_taunt(2, 1, 0);
    sim.receive_taunt(1, 1, 1);
    assert_eq!(sim.leaders.taunts[1], Default::default());
    assert_eq!(sim.leaders.taunts[2], Default::default());
    sim.receive_taunt(1, TAUNT_HELP, 0);
    assert_eq!(sim.leaders.taunts[1].incoming_taunt_frame[0], 100);
    for _ in 0..8 {
        sim.receive_taunt(1, 2, 0);
    }
    let t = &sim.leaders.taunts[1];
    assert_eq!(t.incoming, 1);
    assert_eq!(t.incoming_taunt[0], 2, "the ninth overwrote the first");
}

/// The alliance is re-tested when the request fires: an ally turned to
/// peace inside the delay stores nothing.
#[test]
fn process_taunt_re_tests_the_alliance() {
    let mut sim = seats(9_870);
    sim.frame = 906;
    sim.receive_taunt(1, 1, 0);
    sim.set_diplo(0, 1, 1);
    sim.frame = 966;
    sim.drain_taunts(1);
    assert_eq!(sim.leaders.taunts[1].last_taunt[0], 0);
    assert_eq!(sim.economy.ledgers[1].bucket[FOOD], 9_870);
}
