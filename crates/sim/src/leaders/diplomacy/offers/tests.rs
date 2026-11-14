//! The offers arm on a fixture (`docs/LEADERS.md` §7.2, §8.2): who=0 a
//! human, who=1 a computer at war with it, team style 0, difficulty 0.
//! run1575's accept of 1027 is the diff (`rondata::diff::diplomacy_take`).

use crate::economy::Resource;
use crate::{Sim, Tuning, World};

const FOOD: usize = Resource::Food.index();

/// who=0 human, who=1 computer, at war, `TEAM_STYLE 0`, Easiest, no rush
/// rules, each with 5000 of every good it can use.
fn at_war() -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    sim.cities.nation[0].human = true;
    sim.cities.nation[1].human = false;
    sim.ai.lobby.team_style = 0;
    sim.ai.lobby.difficulty = 0;
    sim.ai.lobby.rush_rules = 0;
    for (l, h) in sim
        .economy
        .ledgers
        .iter_mut()
        .zip(sim.economy.holdings.iter_mut())
    {
        l.bucket = [5_000; 6];
        h.available = [true; 6];
    }
    sim.set_diplo(0, 1, 0);
    sim.frame = 5_000;
    sim
}

/// The human asks peace (`action_treaty`) and commits to it
/// (`action_respond`), which forces the computer's pass.
fn propose(sim: &mut Sim, w: i32) {
    sim.action_treaty(0, 1, w);
    sim.action_respond(0, 1, 1);
}

/// `action_treaty` writes the treaty into both records and withdraws both
/// agreements, its own `any_offer` with them; a treaty is a proposal.
#[test]
fn a_treaty_is_written_both_ways_and_withdraws_agreement() {
    let mut sim = at_war();
    sim.leaders.tribute[1].dip[0].agree = 1;
    assert!(!sim.any_proposals(0, 1));
    sim.action_treaty(0, 1, 1);
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!((a.treaty, b.treaty), (1, 1));
    assert_eq!((a.agree, a.any_offer, b.agree), (0, 0, 0));
    assert!(sim.any_proposals(0, 1) && sim.any_proposals(1, 0));
}

/// run1575's shape: a peace asked for nothing, the computer not weak, the
/// target's `dow` under 2. `S` is 0 plus 100 for each difficulty gate under
/// 2 and under 3, so the need is −200: accepted, the peace set both ways,
/// `made_peace` this frame on both, both records cleared, and the surplus
/// credited to `good_deeds` (`good_deeds −= need`).
#[test]
fn a_peace_asked_for_nothing_is_accepted() {
    let mut sim = at_war();
    propose(&mut sim, 1);
    assert_eq!(
        sim.leaders.agendas[1][0] & 4,
        4,
        "the commit forces the pass"
    );
    sim.offers_arm(1, 0, false);
    assert_eq!((sim.diplos(0, 1), sim.diplos(1, 0)), (1, 1));
    let d = &sim.leaders.declare;
    assert_eq!((d[0].made_peace[1], d[1].made_peace[0]), (5_000, 5_000));
    assert_eq!(d[1].good_deeds[0], 200);
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!((a.treaty, b.treaty, a.agree, b.agree), (-1, -1, 0, 0));
}

/// A fresh tribute of mine held by the target (`is_tribute_period(t, me)`)
/// skips the shortfall's adjustments: `S` is `good_deeds` alone, run1575's
/// need of 0, still an accept, and `good_deeds` unchanged.
#[test]
fn a_fresh_tribute_skips_the_adjustments() {
    let mut sim = at_war();
    sim.leaders.tribute[0].tribute_stamp[1] = 4_000;
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 1);
    assert_eq!(sim.leaders.declare[1].good_deeds[0], 0);
}

/// Strong, not weak, against a target fighting one war: `S −= (epoch + 1)
/// · strong`, a need of 64, no give of mine — a counteroffer: the flag set,
/// both agreements withdrawn, the treaty left standing, still at war. A
/// pending counteroffer forgives 99 next time: accepted.
#[test]
fn found_short_is_a_counteroffer_and_its_grace_accepts() {
    let mut sim = at_war();
    sim.ai.ai[1].strong[0] = 64;
    sim.ai.ai[0].census.wars = 1;
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 1);
    assert_eq!(sim.leaders.tribute[0].dip[1].agree, 0, "withdrawn");
    assert_eq!(sim.leaders.tribute[0].dip[1].treaty, 1, "still asked");
    assert_eq!(sim.diplos(1, 0), 0);
    sim.action_respond(0, 1, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 1);
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 0);
    assert_eq!(sim.leaders.declare[1].good_deeds[0], 35, "need 64 − 99");
}

/// The selector is `aggression[t]`, not the treaty: with it set the need is
/// `(epoch + t.blacken + 1) · 100 · ((strong != 0) + 1) − S'`, `S'` here
/// 100 (difficulty under 2), so 0 accepts — and a target's `blacken` of 1
/// makes it 100, a counteroffer.
#[test]
fn aggression_selects_the_price_of_peace() {
    let mut sim = at_war();
    sim.ai.ai[1].aggression[0] = 1;
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 1, "a need of 0");
    let mut sim = at_war();
    sim.ai.ai[1].aggression[0] = 1;
    sim.leaders.declare[0].blacken = 1;
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 0);
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 1, "a need of 100");
}

/// A give of mine over my stock refuses a deal I would otherwise take:
/// rejected, both records cleared.
#[test]
fn an_over_ask_refuses() {
    let mut sim = at_war();
    sim.leaders.declare[1].good_deeds[0] = 1_000_000;
    sim.leaders.tribute[1].dip[0].offers[FOOD] = 6_000;
    sim.leaders.tribute[0].dip[1].offers[FOOD] = -6_000;
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 0);
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!((a.treaty, b.treaty, a.offers[FOOD]), (-1, -1, 0));
}

/// An ask of the target's goods raises `S` by their worth to me
/// (`resource_value`: 0xaa, over 1000 in stock with no `econ` bit),
/// unmultiplied with no give before it: a peace that also asks 300 food is
/// credited `300 · 0xaa / 256` = 199 more, and the food arrives through
/// the human's scale (51 %).
#[test]
fn an_ask_is_valued_at_its_worth_to_me() {
    let mut sim = at_war();
    sim.action_offer(1, 0, Resource::Food, -300);
    propose(&mut sim, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 1);
    assert_eq!(sim.leaders.declare[1].good_deeds[0], 200 + 300 * 0xaa / 256);
    assert_eq!(
        sim.economy.ledgers[1].bucket[FOOD],
        5_000 + (300 * 51 + 99) / 100
    );
}

/// With nothing committed to me, a standing proposal ends the pass: the
/// offers arm neither answers nor proposes.
#[test]
fn an_uncommitted_proposal_waits() {
    let mut sim = at_war();
    sim.action_treaty(0, 1, 1);
    sim.offers_arm(1, 0, false);
    assert_eq!(sim.diplos(1, 0), 0);
    assert_eq!(sim.leaders.tribute[1].dip[0].treaty, 1);
}

/// The victory arm at team style 0: a target running the territory clock
/// is turned on — its committed proposal rejected, `agendas |= 0x20` — and
/// the pass ends before the offers arm; at war already, no declaration.
#[test]
fn a_target_racing_a_clock_is_turned_on() {
    let mut sim = at_war();
    sim.ai.lobby.victory = 0;
    sim.ai.ai[0].popwin_timer = 1;
    propose(&mut sim, 1);
    assert!(!sim.style_zero_arms(1, 0));
    assert_eq!(sim.leaders.agendas[1][0] & 0x20, 0x20);
    assert_eq!(sim.leaders.tribute[0].dip[1].treaty, -1, "rejected");
    assert_eq!(sim.diplos(1, 0), 0);
    sim.ai.ai[0].popwin_timer = 0;
    assert!(sim.style_zero_arms(1, 0), "no clock, no arm");
}

/// The hires: an executed deal that names a third slot to attack stamps
/// both parties' `hire_stamp` and `hire_who`, and each party not yet at war
/// with it declares — a computer quietly, a human counted.
#[test]
fn an_attack_deal_hires_both_parties() {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 3);
    sim.cities.nation[0].human = true;
    sim.cities.nation[1].human = false;
    sim.cities.nation[2].human = false;
    sim.frame = 7_000;
    sim.leaders.tribute[0].dip[1].attacks[2] = 1;
    sim.leaders.tribute[1].dip[0].attacks[2] = 1;
    sim.leaders.tribute[1].dip[0].agree = 1;
    sim.action_respond(0, 1, 1);
    let d = &sim.leaders.declare;
    assert_eq!((d[0].hire_stamp[2], d[1].hire_stamp[2]), (7_000, 7_000));
    assert_eq!((d[0].hire_who[2], d[1].hire_who[2]), (1, 0));
    assert_eq!(d[0].hire_who[1], -1, "an unnamed slot stays −1");
    assert!(sim.is_enemy(0, 2) && sim.is_enemy(1, 2));
    assert_eq!((d[0].dow[2], d[1].dow[2]), (1, 0), "the human's counts");
}
