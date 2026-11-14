//! An alliance asked, on a fixture (`docs/LEADERS.md` §7.2): who=0 a human,
//! who=1 a computer, who=2 a computer, all at peace, team style 0,
//! difficulty 0, `VICTORY` 0, every Military epoch 0. run1575's French
//! answer of 1051 is the diff (`rondata::diff::diplomacy_take::
//! run1575_s_alliance_is_found_short`).

use super::Rivals;
use crate::{Sim, Tuning, World};

fn three() -> Sim {
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 3);
    sim.cities.nation[0].human = true;
    sim.cities.nation[1].human = false;
    sim.cities.nation[2].human = false;
    sim.ai.lobby.team_style = 0;
    sim.ai.lobby.difficulty = 0;
    sim.ai.lobby.victory = 0;
    sim.frame = 5_000;
    sim
}

/// `p` asks `who=1` for an alliance (`action_treaty`) and commits to it.
fn ask(sim: &mut Sim, p: crate::Player) {
    sim.action_treaty(p, 1, 2);
    sim.action_respond(p, 1, 1);
}

fn answer(sim: &mut Sim, t: usize, rivals: Rivals) {
    sim.offers_arm(1, t, false, rivals);
}

/// run1575's shape: a human asks at peace for nothing. The price `(0 + 1) ·
/// 200` comes off `S` = 0, the need is 200, and with no give of mine it is
/// the counteroffer: `counteroffer[0]` 1, both agreements withdrawn, the
/// records kept (`treaty` 2), no alliance, `good_deeds` untouched.
#[test]
fn an_alliance_asked_for_nothing_is_found_short() {
    let mut sim = three();
    ask(&mut sim, 0);
    assert_eq!(sim.leaders.tribute[0].dip[1].agree, 1, "the human committed");
    answer(&mut sim, 0, Rivals::default());
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 1);
    assert_eq!((a.agree, b.agree), (0, 0));
    assert_eq!((a.treaty, b.treaty), (2, 2));
    assert_eq!((sim.diplos(0, 1), sim.diplos(1, 0)), (1, 1));
    assert_eq!(sim.leaders.declare[1].good_deeds[0], 0);
}

/// The comparison can flip: `good_deeds` of 200 pays the price, the need is
/// 0 and the alliance is accepted — set both ways, both records cleared,
/// `counteroffer` 0, and `good_deeds` charged the need (0) again.
#[test]
fn good_deeds_that_pay_the_price_accept() {
    let mut sim = three();
    sim.leaders.declare[1].good_deeds[0] = 200;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!((sim.diplos(0, 1), sim.diplos(1, 0)), (2, 2));
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 0);
    assert_eq!(sim.leaders.tribute[0].dip[1].treaty, -1);
    assert_eq!(sim.leaders.declare[1].good_deeds[0], 200);
}

/// At war the price doubles: 400 against `good_deeds` 200 is short, 400
/// pays it. A capital lost to a third party takes 400 off the price.
#[test]
fn war_doubles_the_price_and_a_lost_capital_takes_400_off() {
    let mut sim = three();
    sim.set_diplo(0, 1, 0);
    sim.leaders.declare[1].good_deeds[0] = 200;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 1, "400 against 200");
    assert_eq!(sim.diplos(1, 0), 0);
    let mut sim = three();
    sim.set_diplo(0, 1, 0);
    ask(&mut sim, 0);
    let lost = Rivals {
        capital: Some(2),
        racer: None,
    };
    answer(&mut sim, 0, lost);
    assert_eq!((sim.diplos(0, 1), sim.diplos(1, 0)), (2, 2), "400 − 400");
}

/// A rusher asks 100 more, a boomer below epoch 4 100 less.
#[test]
fn a_rusher_asks_more_and_a_boomer_less() {
    let mut sim = three();
    sim.ai.ai[1].player_flags = 1 << 13;
    sim.leaders.declare[1].good_deeds[0] = 100;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!(sim.diplos(1, 0), 2, "the boomer's 100 is paid");
    let mut sim = three();
    sim.ai.ai[1].player_flags = 1 << 12;
    sim.leaders.declare[1].good_deeds[0] = 200;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 1, "the rusher's 300");
}

/// An ally of mine refuses the alliance at team style 0 (`num_allies`):
/// rejected, since I had not committed — both records cleared, both
/// agreements withdrawn, no counteroffer.
#[test]
fn an_ally_of_mine_refuses() {
    let mut sim = three();
    sim.set_diplo(1, 2, 2);
    assert_eq!(sim.num_allies(1), 1);
    sim.leaders.declare[1].good_deeds[0] = 10_000;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!((a.treaty, b.treaty, a.agree, b.agree), (-1, -1, 0, 0));
    assert_eq!(sim.leaders.tribute[1].counteroffer[0], 0);
    assert_eq!(sim.diplos(1, 0), 1);
}

/// A victory clock anywhere refuses it, and so does any team style but 0,
/// 7, 8 and 11.
#[test]
fn a_victory_clock_or_a_locked_style_refuses() {
    let mut sim = three();
    sim.ai.ai[2].popwin_timer = 1;
    assert!(sim.is_victory_timer());
    sim.leaders.declare[1].good_deeds[0] = 10_000;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!(sim.leaders.tribute[0].dip[1].treaty, -1);
    assert_eq!(sim.diplos(1, 0), 1);
    sim.ai.lobby.victory = 1;
    assert!(!sim.is_victory_timer(), "no clock counts outside VICTORY 0");
    let mut sim = three();
    sim.ai.lobby.team_style = 1;
    sim.leaders.declare[1].good_deeds[0] = 10_000;
    ask(&mut sim, 0);
    answer(&mut sim, 0, Rivals::default());
    assert_eq!(sim.leaders.tribute[0].dip[1].treaty, -1);
}

/// Two computers with `aggression[t]` 0 ally at once, whatever the price,
/// and the counteroffer is cleared after the accept.
#[test]
fn two_computers_ally_at_once() {
    let mut sim = three();
    sim.leaders.tribute[1].counteroffer[2] = 1;
    ask(&mut sim, 2);
    answer(&mut sim, 2, Rivals::default());
    assert_eq!((sim.diplos(1, 2), sim.diplos(2, 1)), (2, 2));
    assert_eq!(sim.leaders.tribute[1].counteroffer[2], 0);
    let mut sim = three();
    sim.ai.ai[1].aggression[2] = 1;
    sim.set_diplo(1, 2, 0);
    ask(&mut sim, 2);
    answer(&mut sim, 2, Rivals::default());
    assert_eq!(sim.leaders.tribute[1].counteroffer[2], 1, "priced instead");
}

/// `num_allies` counts live mutual allies only; `num_team_members(·, 1)`
/// counts me and, at team style 0, a lobby teammate (0..3) I am allied
/// with; and `action_respond`'s team gate refuses an alliance to a party
/// with an ally off its team.
#[test]
fn the_counts_and_the_team_gate() {
    let mut sim = three();
    sim.leaders.team = vec![0, 1, 0];
    sim.set_diplo(0, 2, 2);
    assert_eq!((sim.num_allies(0), sim.num_team_members(0, true)), (1, 2));
    sim.leaders.defeated[2] = true;
    assert_eq!((sim.num_allies(0), sim.num_team_members(0, true)), (0, 1));
    sim.leaders.defeated[2] = false;
    assert!(sim.alliance_open(0, 1), "the human's one ally is his teammate");
    sim.leaders.team = vec![0, 1, 1];
    assert!(!sim.alliance_open(0, 1), "an ally off his team");
    // The gate on an executed alliance: both committed, nothing exchanged.
    sim.action_treaty(1, 0, 2);
    sim.leaders.tribute[1].dip[0].agree = 1;
    sim.action_respond(0, 1, 1);
    let (a, b) = (sim.leaders.tribute[0].dip[1], sim.leaders.tribute[1].dip[0]);
    assert_eq!((a.treaty, b.treaty, a.agree, b.agree), (-1, -1, 0, 0));
    assert_eq!(sim.diplos(0, 1), 1, "refused");
    sim.leaders.team = vec![0, 1, 0];
    sim.action_treaty(1, 0, 2);
    sim.leaders.tribute[1].dip[0].agree = 1;
    sim.action_respond(0, 1, 1);
    assert_eq!(sim.diplos(0, 1), 2, "allowed");
}

/// `LAB_006be372`'s third party: a live slot neither side's ally running a
/// clock.
#[test]
fn a_third_party_racing_a_clock_is_noted() {
    let mut sim = three();
    assert_eq!(sim.racing_third_party(1, 0), None);
    sim.leaders.endgame[2].wonderwin_timer = 1;
    assert_eq!(sim.racing_third_party(1, 0), Some(2));
    sim.set_diplo(0, 2, 2);
    assert_eq!(sim.racing_third_party(1, 0), None, "the target's ally");
}
