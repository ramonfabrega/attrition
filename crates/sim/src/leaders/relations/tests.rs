use super::Seat;
use crate::{Sim, Tuning, World};

/// Three seats, the lobby defaulting to the harness's (`TEAM_STYLE 1`,
/// no rush rules, Ancient); every seat on team 8 unless given one.
fn three(teams: [i32; 3]) -> (Sim, Vec<Seat>) {
    let sim = Sim::new(Tuning::RON, World::new(2, 2), 3);
    let seats = (0..3)
        .map(|w| Seat {
            who: w as crate::Player,
            flags: 1,
            team: teams[w],
        })
        .collect();
    (sim, seats)
}

fn rows(sim: &mut Sim, seats: &[Seat]) -> (Vec<Vec<bool>>, Vec<Vec<bool>>) {
    for w in 0..3 {
        sim.init_diplos(w, seats);
    }
    (sim.leaders.at_war.clone(), sim.leaders.allied.clone())
}

/// The harness lobby's arm, which every start dump on disk prints (item
/// 4046: 633 logs, 1,322 pairs): no teams under a locked style is war
/// off the diagonal, and the diagonal is left alone.
#[test]
fn a_locked_style_with_no_teams_opens_at_war() {
    let (mut sim, seats) = three([8, 8, 8]);
    let (war, ally) = rows(&mut sim, &seats);
    assert_eq!(
        war,
        [
            [false, true, true],
            [true, false, true],
            [true, true, false]
        ]
    );
    assert!(ally.iter().flatten().all(|a| !a));
}

/// Rush rules above the starting age are peace (`RUSH_RULES 10`, the
/// peace pair's lobby — 12 pairs on disk); at or below it, war again.
#[test]
fn rush_rules_above_the_starting_age_open_at_peace() {
    let (mut sim, seats) = three([8, 8, 8]);
    sim.ai.lobby.rush_rules = 10;
    let (war, _) = rows(&mut sim, &seats);
    assert!(war.iter().flatten().all(|w| !w));
    let (mut sim, seats) = three([8, 8, 8]);
    sim.ai.lobby.rush_rules = 1;
    sim.ai.lobby.starting_technology = 1;
    sim.setup().sync_setup_from_lobby();
    let (war, _) = rows(&mut sim, &seats);
    assert!(war[0][1], "a rush row at the starting age is no peace");
}

/// Teammates (the same team in `0..4`) are allied whatever the style;
/// the others follow the style. Reading-only: no capture on disk has a
/// team game.
#[test]
fn teammates_open_allied() {
    let (mut sim, seats) = three([0, 0, 1]);
    let (war, ally) = rows(&mut sim, &seats);
    assert!(ally[0][1] && ally[1][0] && !war[0][1]);
    assert!(war[0][2] && war[2][1] && !ally[2][0]);
}

/// An unlocked style (0, 8 or 11) opens at peace, and a conquest game
/// leaves the matrix as it was. Reading-only.
#[test]
fn an_unlocked_style_opens_at_peace_and_conquest_writes_nothing() {
    for style in [0, 8, 11] {
        let (mut sim, seats) = three([8, 8, 8]);
        sim.ai.lobby.team_style = style;
        let (war, ally) = rows(&mut sim, &seats);
        assert!(
            war.iter().chain(&ally).flatten().all(|b| !b),
            "style {style}"
        );
    }
    let (mut sim, seats) = three([0, 0, 8]);
    sim.ai.lobby.conquest = true;
    sim.leaders.at_war[0][2] = true;
    let (war, ally) = rows(&mut sim, &seats);
    assert!(war[0][2] && !war[2][0] && !ally[0][1]);
}
