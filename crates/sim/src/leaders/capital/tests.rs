use super::*;
use crate::world::{Cell, Terrain, World};

/// A two-player sim on a 60×60 all-land map (row 0 a sea, so region 1 is
/// the land): `land_size` 3540, `w = 1770` a nation.
fn sim() -> Sim {
    let mut w = World::new(60, 60);
    let sea = w.add_region(Terrain::Sea);
    let land = w.add_region(Terrain::Land);
    for y in 0..60 {
        for x in 0..60 {
            w.set_region(Cell::new(x, y), if y == 0 { sea } else { land });
        }
    }
    Sim::new(crate::Tuning::RON, w, 2)
}

/// **The loss's modifier** (§11): `(To + w)·256 / (Ta + w)` clamped to
/// `[128, 512]`, `w` the land over the nations. Equal teams give ×1; a
/// loser with a third of the taker's land, less; a loser with more, more.
/// A first loss stores the frame.
#[test]
fn the_loss_writes_the_territory_ratio_and_the_frame() {
    let mut s = sim();
    assert_eq!(s.land_size(), 3540);
    s.economy.holdings[0].territory = 400;
    s.economy.holdings[1].territory = 400;
    s.frame = 5639;
    s.lost_capital(0, 1);
    let e = s.leader_endgame(0);
    assert_eq!(
        (
            e.lost_capital_timer,
            e.lost_capital_modifier,
            e.lost_capital_stamp
        ),
        (1, 256, 5639)
    );

    let mut s = sim();
    s.economy.holdings[0].territory = 100;
    s.economy.holdings[1].territory = 3000;
    s.lost_capital(0, 1);
    // (100 + 1770)·256 / (3000 + 1770) = 100
    assert_eq!(
        s.leader_endgame(0).lost_capital_modifier,
        MODIFIER_FLOOR as i32
    );

    let mut s = sim();
    s.economy.holdings[0].territory = 1500;
    s.economy.holdings[1].territory = 300;
    s.lost_capital(0, 1);
    // (1500 + 1770)·256 / (300 + 1770) = 404
    assert_eq!(s.leader_endgame(0).lost_capital_modifier, 404);
}

/// A second loss while the clock runs stores nothing (§11).
#[test]
fn a_loss_while_the_clock_runs_is_a_no_op() {
    let mut s = sim();
    s.frame = 100;
    s.lost_capital(0, 1);
    s.economy.holdings[0].territory = 3000;
    s.frame = 200;
    s.lost_capital(0, 1);
    let e = s.leader_endgame(0);
    assert_eq!((e.lost_capital_stamp, e.lost_capital_modifier), (100, 256));
}

/// **The stop and the refresh** (§11): a recapture turns the run into a
/// debt `(stamp − frame)·m / 256` truncating toward zero, keeps the
/// modifier, and the debt is paid back one frame in five; a later loss
/// folds the debt in, rescaled.
#[test]
fn a_stop_banks_a_debt_the_refresh_pays_and_a_new_loss_folds_in() {
    let mut s = sim();
    s.economy.holdings[0].territory = 1500;
    s.economy.holdings[1].territory = 300;
    s.frame = 1000;
    s.lost_capital(0, 1);
    assert_eq!(s.leader_endgame(0).lost_capital_modifier, 404);
    s.frame = 1101;
    s.stop_capital_clock(0);
    let e = s.leader_endgame(0);
    // −101·404 / 256 = −159.39…, toward zero −159 (a floor gives −160).
    assert_eq!(
        (
            e.lost_capital_timer,
            e.lost_capital_stamp,
            e.lost_capital_modifier
        ),
        (0, -159, 404)
    );
    s.frame = 1105;
    s.refresh_capital_clock(0);
    assert_eq!(s.leader_endgame(0).lost_capital_stamp, -158);
    s.frame = 1106;
    s.refresh_capital_clock(0);
    assert_eq!(s.leader_endgame(0).lost_capital_stamp, -158);
    s.economy.holdings[0].territory = 400;
    s.economy.holdings[1].territory = 400;
    s.frame = 2000;
    s.lost_capital(0, 1);
    // 2000 + (−158·256) / 256
    assert_eq!(s.leader_endgame(0).lost_capital_stamp, 2000 - 158);
}

/// **The countdown** (§11): `retake_capital` is 3086 frames on a 60-wide
/// map at ×1, scaled by the modifier; the loser is defeated, type 1, on
/// the frame it reaches 0, by itself when no one holds its capital.
#[test]
fn the_countdown_defeats_the_loser_when_the_clock_runs_out() {
    let mut s = sim();
    s.economy.holdings[0].territory = 1500;
    s.economy.holdings[1].territory = 300;
    s.frame = 1000;
    s.lost_capital(0, 1);
    // 3086·404 / 256 = 4870
    assert_eq!(s.retake_capital(0), 4870);
    s.frame = 1000 + 4869;
    s.process_elimination(0);
    assert!(!s.leaders.defeated[0]);
    s.frame = 1000 + 4870;
    s.process_elimination(0);
    let e = s.leader_endgame(0);
    assert!(s.leaders.defeated[0]);
    assert_eq!(
        (e.defeated_by, e.defeat_stamp, e.defeat_type),
        (0, 5870, crate::leaders::endgame::DEFEAT_ELIMINATION)
    );
}

/// The countdown is elimination 1's alone.
#[test]
fn no_countdown_outside_elimination_one() {
    let mut s = sim();
    s.ai.lobby.elimination = 0;
    s.lost_capital(0, 1);
    s.frame = 100_000;
    s.process_elimination(0);
    assert!(!s.leaders.defeated[0]);
}
